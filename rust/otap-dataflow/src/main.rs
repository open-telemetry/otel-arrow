// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Create and run a multi-core pipeline

use clap::{ArgGroup, Parser};
use otap_df_config::engine::{EngineConfig, HttpAdminSettings};
use otap_df_config::pipeline::{CoreAllocation, CoreRange, PipelineConfig, Quota};
use otap_df_config::{PipelineGroupId, PipelineId};
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use std::path::PathBuf;
use sysinfo::System;

#[cfg(all(
    not(windows),
    feature = "jemalloc",
    feature = "mimalloc",
    not(any(test, doc)),
    not(clippy)
))]
compile_error!(
    "Features `jemalloc` and `mimalloc` are mutually exclusive. \
     To build with mimalloc, use: cargo build --release --no-default-features --features mimalloc"
);

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(all(not(windows), feature = "jemalloc", not(feature = "mimalloc")))]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(all(not(windows), feature = "jemalloc", not(feature = "mimalloc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = system_info(),
    group = ArgGroup::new("config_source")
        .required(true)
        .multiple(false)
        .args(["pipeline", "config"])
)]
struct Args {
    /// Path to the pipeline configuration file (.json, .yaml, or .yml)
    #[arg(short, long, value_name = "FILE", group = "config_source")]
    pipeline: Option<PathBuf>,

    /// Path to the engine configuration file (.json, .yaml, or .yml)
    #[arg(short = 'c', long, value_name = "FILE", group = "config_source")]
    config: Option<PathBuf>,

    /// Number of cores to use (0 for default)
    #[arg(long, conflicts_with = "core_id_range")]
    num_cores: Option<usize>,

    /// Inclusive range of CPU core IDs to pin threads to (e.g. "0-3", "0..3,5", "0..=3,6-7").
    #[arg(long, value_name = "START..END", value_parser = parse_core_id_allocation, conflicts_with = "num_cores")]
    core_id_range: Option<CoreAllocation>,

    /// Address to bind the HTTP admin server to (e.g., "127.0.0.1:8080", "0.0.0.0:8080").
    /// Defaults to 127.0.0.1:8080 when unset.
    #[arg(long)]
    http_admin_bind: Option<String>,
}

fn parse_core_id_allocation(s: &str) -> Result<CoreAllocation, String> {
    // Accept format (EBNF):
    //  S -> digit | CoreRange | S,",",S
    //  CoreRange -> digit,"..",digit | digit,"..=",digit | digit,"-",digit
    //  digit -> [0-9]+

    Ok(CoreAllocation::CoreSet {
        set: s
            .split(',')
            .map(|part| {
                part.trim()
                    .parse::<usize>()
                    // A single ID is a range with the same start and end
                    .map(|n| CoreRange { start: n, end: n })
                    .or_else(|_| parse_core_id_range(part))
            })
            .collect::<Result<Vec<CoreRange>, String>>()?,
    })
}

fn parse_core_id_range(s: &str) -> Result<CoreRange, String> {
    // Accept formats: "a..=b", "a..b", "a-b"
    let normalized = s.replace("..=", "-").replace("..", "-");
    let mut parts = normalized.split('-');
    let start = parts
        .next()
        .ok_or_else(|| "missing start of core id range".to_string())?
        .trim()
        .parse::<usize>()
        .map_err(|_| "invalid start (expected unsigned integer)".to_string())?;
    let end = parts
        .next()
        .ok_or_else(|| "missing end of core id range".to_string())?
        .trim()
        .parse::<usize>()
        .map_err(|_| "invalid end (expected unsigned integer)".to_string())?;
    if parts.next().is_some() {
        return Err("unexpected extra data after end of range".to_string());
    }
    Ok(CoreRange { start, end })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize rustls crypto provider (required for rustls 0.23+)
    // We use ring as the default provider
    #[cfg(feature = "experimental-tls")]
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install rustls crypto provider: {e:?}"))?;

    let Args {
        pipeline,
        config,
        num_cores,
        core_id_range,
        http_admin_bind,
    } = Args::parse();

    println!("{}", system_info());

    let default_admin_bind = HttpAdminSettings::default().bind_address;
    let core_allocation_override = match (core_id_range, num_cores) {
        (Some(range), _) => Some(range),
        (None, Some(num_cores)) => Some(if num_cores == 0 {
            CoreAllocation::AllCores
        } else {
            CoreAllocation::CoreCount { count: num_cores }
        }),
        (None, None) => None,
    };
    // Create controller and start pipeline with multi-core support
    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
    let core_allocation_override_engine = core_allocation_override.clone();
    let core_allocation_override_pipeline = core_allocation_override;
    let http_admin_bind_engine = http_admin_bind.clone();
    let http_admin_bind_pipeline = http_admin_bind;

    let result = if let Some(config_path) = config {
        let engine_cfg = EngineConfig::from_file(config_path)?;
        let EngineConfig {
            settings: engine_settings,
            mut pipeline_groups,
        } = engine_cfg;
        if let Some(core_allocation) = core_allocation_override_engine {
            for pipeline_group in pipeline_groups.values_mut() {
                for pipeline_cfg in pipeline_group.pipelines.values_mut() {
                    pipeline_cfg.set_quota(Quota {
                        core_allocation: core_allocation.clone(),
                    });
                }
            }
        }
        let admin_settings = if let Some(bind_address) = http_admin_bind_engine {
            HttpAdminSettings { bind_address }
        } else if let Some(config_admin) = engine_settings.http_admin.clone() {
            config_admin
        } else {
            HttpAdminSettings {
                bind_address: default_admin_bind.clone(),
            }
        };

        for (pipeline_group_id, pipeline_group_cfg) in pipeline_groups.iter() {
            for (pipeline_id, pipeline_cfg) in pipeline_group_cfg.pipelines.iter() {
                let quota = pipeline_cfg.quota();
                match &quota.core_allocation {
                    CoreAllocation::AllCores => println!(
                        "Requested core allocation for {}:{}: all available cores",
                        pipeline_group_id.as_ref(),
                        pipeline_id.as_ref()
                    ),
                    CoreAllocation::CoreCount { count } => println!(
                        "Requested core allocation for {}:{}: {count} cores",
                        pipeline_group_id.as_ref(),
                        pipeline_id.as_ref()
                    ),
                    CoreAllocation::CoreSet { .. } => println!(
                        "Requested core allocation for {}:{}: {}",
                        pipeline_group_id.as_ref(),
                        pipeline_id.as_ref(),
                        quota.core_allocation
                    ),
                }
            }
        }

        controller.run_engine_forever(pipeline_groups, engine_settings, admin_settings)
    } else {
        // For now, we predefine pipeline group and pipeline IDs.
        // That will be replaced with a more dynamic approach in the future.
        let pipeline_group_id: PipelineGroupId = "default_pipeline_group".into();
        let pipeline_id: PipelineId = "default_pipeline".into();
        let pipeline_path = match pipeline {
            Some(path) => path,
            None => {
                return Err("Missing --pipeline argument".into());
            }
        };

        // Load pipeline configuration from file
        let mut pipeline_cfg = PipelineConfig::from_file(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            &pipeline_path,
        )?;
        if let Some(core_allocation) = core_allocation_override_pipeline {
            pipeline_cfg.set_quota(Quota { core_allocation });
        }
        let quota = pipeline_cfg.quota().clone();
        let admin_settings = HttpAdminSettings {
            bind_address: http_admin_bind_pipeline.unwrap_or_else(|| default_admin_bind.clone()),
        };

        // Print the requested core configuration
        match &quota.core_allocation {
            CoreAllocation::AllCores => println!("Requested core allocation: all available cores"),
            CoreAllocation::CoreCount { count } => {
                println!("Requested core allocation: {count} cores")
            }
            CoreAllocation::CoreSet { .. } => {
                println!("Requested core allocation: {}", quota.core_allocation);
            }
        }

        controller.run_forever(pipeline_group_id, pipeline_id, pipeline_cfg, admin_settings)
    };
    match result {
        Ok(_) => {
            println!("Pipeline run successfully");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Pipeline failed to run: {e}");
            std::process::exit(1);
        }
    }
}

fn system_info() -> String {
    // Your custom logic here - this could read files, check system state, etc.
    let available_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let build_mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let memory_allocator = if cfg!(feature = "mimalloc") {
        "mimalloc"
    } else if cfg!(all(feature = "jemalloc", not(windows))) {
        "jemalloc"
    } else {
        "system"
    };

    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let available_memory_gb = sys.available_memory() as f64 / 1_073_741_824.0;

    let debug_warning = if cfg!(debug_assertions) {
        "\n\n⚠️  WARNING: This binary was compiled in debug mode.
   Debug builds are NOT recommended for production, benchmarks, or performance testing.
   Use 'cargo build --release' for optimal performance."
    } else {
        ""
    };

    // Get available OTAP plugins
    let receivers: Vec<&str> = OTAP_PIPELINE_FACTORY
        .get_receiver_factory_map()
        .keys()
        .copied()
        .collect();
    let processors: Vec<&str> = OTAP_PIPELINE_FACTORY
        .get_processor_factory_map()
        .keys()
        .copied()
        .collect();
    let exporters: Vec<&str> = OTAP_PIPELINE_FACTORY
        .get_exporter_factory_map()
        .keys()
        .copied()
        .collect();

    let mut receivers_sorted = receivers;
    let mut processors_sorted = processors;
    let mut exporters_sorted = exporters;
    receivers_sorted.sort();
    processors_sorted.sort();
    exporters_sorted.sort();

    format!(
        "System Information:
  Available CPU cores: {}
  Available memory: {:.2} GB / {:.2} GB
  Build mode: {}
  Memory allocator: {}

Available Plugin URNs:
  Receivers: {}
  Processors: {}
  Exporters: {}

Configuration files can be found in the configs/ directory.{}",
        available_cores,
        available_memory_gb,
        total_memory_gb,
        build_mode,
        memory_allocator,
        receivers_sorted.join(", "),
        processors_sorted.join(", "),
        exporters_sorted.join(", "),
        debug_warning
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_core_range_ok() {
        assert_eq!(
            parse_core_id_range("0..=4"),
            Ok(CoreRange { start: 0, end: 4 })
        );
        assert_eq!(
            parse_core_id_range("0..4"),
            Ok(CoreRange { start: 0, end: 4 })
        );
        assert_eq!(
            parse_core_id_range("0-4"),
            Ok(CoreRange { start: 0, end: 4 })
        );
    }

    #[test]
    fn parse_core_allocation_ok() {
        assert_eq!(
            parse_core_id_allocation("0..=4,5,6-7"),
            Ok(CoreAllocation::CoreSet {
                set: vec![
                    CoreRange { start: 0, end: 4 },
                    CoreRange { start: 5, end: 5 },
                    CoreRange { start: 6, end: 7 }
                ]
            })
        );
        assert_eq!(
            parse_core_id_allocation("0..4"),
            Ok(CoreAllocation::CoreSet {
                set: vec![CoreRange { start: 0, end: 4 }]
            })
        );
    }

    #[test]
    fn parse_core_range_missing_start() {
        assert_eq!(
            parse_core_id_range(""),
            Err("invalid start (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("a..4"),
            Err("invalid start (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("-1..4"),
            Err("invalid start (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("1.."),
            Err("invalid end (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("1..a"),
            Err("invalid end (expected unsigned integer)".to_string())
        );
        assert_eq!(
            parse_core_id_range("1..2a"),
            Err("invalid end (expected unsigned integer)".to_string())
        );
    }
}
