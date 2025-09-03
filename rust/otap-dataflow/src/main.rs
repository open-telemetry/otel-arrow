// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Create and run a multi-core pipeline

use clap::Parser;
use mimalloc_rust::*;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::pipeline_group::{CoreAllocation, Quota};
use otap_df_config::{PipelineGroupId, PipelineId};
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use std::path::PathBuf;

#[global_allocator]
static GLOBAL_MIMALLOC: GlobalMiMalloc = GlobalMiMalloc;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = system_info()
)]
struct Args {
    /// Path to the pipeline configuration file (.json, .yaml, or .yml)
    #[arg(short, long)]
    pipeline: PathBuf,

    /// Number of cores to use (0 for default)
    #[arg(long, default_value = "0", conflicts_with = "core_id_range")]
    num_cores: usize,

    /// Inclusive range of CPU core IDs to pin threads to (e.g. "0-3", "0..3", "0..=3").
    #[arg(long, value_name = "START..END", value_parser = parse_core_id_range, conflicts_with = "num_cores")]
    core_id_range: Option<CoreAllocation>,

    /// Address to bind the HTTP admin server to (e.g., "127.0.0.1:8080", "0.0.0.0:8080")
    #[arg(long, default_value = "127.0.0.1:8080")]
    http_admin_bind: String,
}

fn parse_core_id_range(s: &str) -> Result<CoreAllocation, String> {
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
    Ok(CoreAllocation::CoreRange { start, end })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // For now, we predefine pipeline group and pipeline IDs.
    // That will be replaced with a more dynamic approach in the future.
    let pipeline_group_id: PipelineGroupId = "default_pipeline_group".into();
    let pipeline_id: PipelineId = "default_pipeline".into();

    // Load pipeline configuration from file
    let pipeline_cfg = PipelineConfig::from_file(
        pipeline_group_id.clone(),
        pipeline_id.clone(),
        &args.pipeline,
    )?;

    // Create controller and start pipeline with multi-core support
    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);

    // Map CLI arguments to the new enum structure
    let core_allocation = if let Some(range) = args.core_id_range {
        range
    } else if args.num_cores == 0 {
        CoreAllocation::AllCores
    } else {
        CoreAllocation::CoreCount {
            count: args.num_cores,
        }
    };

    let quota = Quota { core_allocation };

    // Print what we're doing
    match &quota.core_allocation {
        CoreAllocation::AllCores => println!("Starting pipeline using all available cores"),
        CoreAllocation::CoreCount { count } => println!("Starting pipeline with {} cores", count),
        CoreAllocation::CoreRange { start, end } => {
            println!("Starting pipeline on core ID range [{}-{}]", start, end);
        }
    }

    let admin_settings = otap_df_config::engine::HttpAdminSettings {
        bind_address: args.http_admin_bind,
    };
    let result = controller.run_forever(
        pipeline_group_id,
        pipeline_id,
        pipeline_cfg,
        quota,
        admin_settings,
    );
    match result {
        Ok(_) => {
            println!("Pipeline run successfully");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{e}");
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
        "Examples:
  {} --pipeline configs/otlp-perf.yaml --num-cores 4
  {} --pipeline configs/otlp-perf.yaml --core-id-range 2-5
  {} -p configs/otlp-perf.yaml

System Information:
  Available CPU cores: {}
  Build mode: {}
  Default memory allocator: mimalloc

Available Plugin URNs:
  Receivers: {}
  Processors: {}
  Exporters: {}

Configuration files can be found in the configs/ directory.{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_NAME"),
        available_cores,
        build_mode,
        receivers_sorted.join(", "),
        processors_sorted.join(", "),
        exporters_sorted.join(", "),
        debug_warning
    )
}
