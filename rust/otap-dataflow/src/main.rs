// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Create and run a multi-core pipeline

use clap::Parser;
use otap_df_config::engine::{
    HttpAdminSettings, OtelDataflowSpec, SYSTEM_OBSERVABILITY_PIPELINE_ID, SYSTEM_PIPELINE_GROUP_ID,
};
use otap_df_config::node::NodeKind;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::policy::{CoreAllocation, CoreRange};
// Keep this side-effect import so the crate is linked and its `linkme`
// distributed-slice registrations (contrib processors/exporters) are visible
// in `OTAP_PIPELINE_FACTORY` at runtime.
use otap_df_config::{PipelineGroupId, PipelineId};
use otap_df_contrib_nodes as _;
// Keep this side-effect import so the crate is linked and its `linkme`
// distributed-slice registrations (contrib processors/exporters) are visible
// in `OTAP_PIPELINE_FACTORY` at runtime.
use otap_df_contrib_nodes as _;
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

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = system_info(),
    after_long_help = concat!(
        "EXAMPLES:\n",
        "  ", env!("CARGO_BIN_NAME"), " --config  config.yaml\n",
    )
)]
struct Args {
    /// Path to the engine configuration file (.json, .yaml, or .yml)
    #[arg(short = 'c', long, value_name = "FILE")]
    config: PathBuf,

    /// Number of cores to use (0 for all available cores)
    #[arg(long, conflicts_with = "core_id_range")]
    num_cores: Option<usize>,

    /// Inclusive range of CPU core IDs to pin threads to (e.g. "0-3", "0..3,5", "0..=3,6-7")
    #[arg(long, value_name = "START..END", value_parser = parse_core_id_allocation, conflicts_with = "num_cores")]
    core_id_range: Option<CoreAllocation>,

    /// Address to bind the HTTP admin server to (e.g., "127.0.0.1:8080", "0.0.0.0:8080")
    #[arg(long)]
    http_admin_bind: Option<String>,

    /// Validate the provided configuration and exit without starting the engine.
    ///
    /// Checks performed:
    /// - Configuration file parsing (YAML/JSON schema)
    /// - Structural validation (version, policies, connections, node references)
    /// - Component existence (every node URN maps to a registered component in this binary)
    /// - Component-specific config validation (when supported by the component)
    #[arg(long)]
    validate_and_exit: bool,
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
                    // A single ID is a range with the same start and end.
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

fn core_allocation_override(
    num_cores: Option<usize>,
    core_id_range: Option<CoreAllocation>,
) -> Option<CoreAllocation> {
    match (core_id_range, num_cores) {
        (Some(range), _) => Some(range),
        (None, Some(0)) => Some(CoreAllocation::AllCores),
        (None, Some(count)) => Some(CoreAllocation::CoreCount { count }),
        (None, None) => None,
    }
}

fn http_admin_bind_override(http_admin_bind: Option<String>) -> Option<HttpAdminSettings> {
    http_admin_bind.map(|bind_address| HttpAdminSettings { bind_address })
}

fn apply_cli_overrides(
    engine_cfg: &mut OtelDataflowSpec,
    num_cores: Option<usize>,
    core_id_range: Option<CoreAllocation>,
    http_admin_bind: Option<String>,
) {
    if let Some(core_allocation) = core_allocation_override(num_cores, core_id_range) {
        engine_cfg.policies.resources.core_allocation = core_allocation;
    }
    if let Some(http_admin) = http_admin_bind_override(http_admin_bind) {
        engine_cfg.engine.http_admin = Some(http_admin);
    }
}

/// Validates that every node in a pipeline references a component URN
/// that is registered in the `OTAP_PIPELINE_FACTORY`.
///
/// Note: structural config validation (connections, node references, policies)
/// is already performed during config deserialization (`OtelDataflowSpec::from_file`).
/// This function adds the semantic check that all referenced components are actually
/// compiled into this binary, and validates their node-specific config statically.
///
/// **Scope:** This is *static* validation only — it checks that the config values
/// can be deserialized into the expected types. It does **not** detect runtime
/// issues such as port conflicts, unreachable endpoints, missing files, or other
/// conditions that only manifest when the engine actually starts.
fn validate_pipeline_components(
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    pipeline_cfg: &PipelineConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    for (node_id, node_cfg) in pipeline_cfg.node_iter() {
        let kind = node_cfg.kind();
        let urn_str = node_cfg.r#type.as_str();

        let validate_config_fn = match kind {
            NodeKind::Receiver => OTAP_PIPELINE_FACTORY
                .get_receiver_factory_map()
                .get(urn_str)
                .map(|f| f.validate_config),
            NodeKind::Processor | NodeKind::ProcessorChain => OTAP_PIPELINE_FACTORY
                .get_processor_factory_map()
                .get(urn_str)
                .map(|f| f.validate_config),
            NodeKind::Exporter => OTAP_PIPELINE_FACTORY
                .get_exporter_factory_map()
                .get(urn_str)
                .map(|f| f.validate_config),
        };

        match validate_config_fn {
            None => {
                let kind_name = match kind {
                    NodeKind::Receiver => "receiver",
                    NodeKind::Processor | NodeKind::ProcessorChain => "processor",
                    NodeKind::Exporter => "exporter",
                };
                return Err(std::io::Error::other(format!(
                    "Unknown {} component `{}` in pipeline_group={} pipeline={} node={}",
                    kind_name,
                    urn_str,
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    node_id.as_ref()
                ))
                .into());
            }
            Some(validate_fn) => {
                validate_fn(&node_cfg.config).map_err(|e| {
                    std::io::Error::other(format!(
                        "Invalid config for component `{}` in pipeline_group={} pipeline={} node={}: {}",
                        urn_str,
                        pipeline_group_id.as_ref(),
                        pipeline_id.as_ref(),
                        node_id.as_ref(),
                        e
                    ))
                })?;
            }
        }
    }

    Ok(())
}

fn validate_engine_components(
    engine_cfg: &OtelDataflowSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    for (pipeline_group_id, pipeline_group) in &engine_cfg.groups {
        for (pipeline_id, pipeline_cfg) in &pipeline_group.pipelines {
            validate_pipeline_components(pipeline_group_id, pipeline_id, pipeline_cfg)?;
        }
    }

    // Also validate the observability pipeline nodes, if configured.
    if let Some(obs_pipeline) = &engine_cfg.engine.observability.pipeline {
        let obs_group_id: PipelineGroupId = SYSTEM_PIPELINE_GROUP_ID.into();
        let obs_pipeline_id: PipelineId = SYSTEM_OBSERVABILITY_PIPELINE_ID.into();
        let obs_pipeline_config = obs_pipeline.clone().into_pipeline_config();
        validate_pipeline_components(&obs_group_id, &obs_pipeline_id, &obs_pipeline_config)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize rustls crypto provider (required for rustls 0.23+)
    // We use ring as the default provider
    #[cfg(feature = "experimental-tls")]
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install rustls crypto provider: {e:?}"))?;

    let Args {
        config,
        num_cores,
        core_id_range,
        http_admin_bind,
        validate_and_exit,
    } = Args::parse();

    println!("{}", system_info());

    let mut engine_cfg = OtelDataflowSpec::from_file(&config)?;
    apply_cli_overrides(&mut engine_cfg, num_cores, core_id_range, http_admin_bind);

    validate_engine_components(&engine_cfg)?;

    if validate_and_exit {
        println!("Configuration '{}' is valid.", config.display());
        std::process::exit(0);
    }

    let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
    let result = controller.run_forever(engine_cfg);
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

    // Get available OTAP components
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

Available Component URNs:
  Receivers: {}
  Processors: {}
  Exporters: {}

Example configuration files can be found in the configs/ directory.{}",
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
    use clap::error::ErrorKind;

    fn minimal_engine_yaml() -> &'static str {
        r#"
version: otel_dataflow/v1
engine:
  http_admin:
    bind_address: "127.0.0.1:18080"
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:example:receiver"
            config: null
          exporter:
            type: "urn:test:example:exporter"
            config: null
        connections:
          - from: receiver
            to: exporter
"#
    }

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
    fn parse_core_range_errors() {
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

    #[test]
    fn core_allocation_override_prefers_range() {
        let range = CoreAllocation::CoreSet {
            set: vec![CoreRange { start: 2, end: 4 }],
        };
        let resolved = core_allocation_override(Some(3), Some(range.clone()));
        assert_eq!(resolved, Some(range));
    }

    #[test]
    fn core_allocation_override_maps_num_cores() {
        assert_eq!(
            core_allocation_override(Some(5), None),
            Some(CoreAllocation::CoreCount { count: 5 })
        );
        assert_eq!(
            core_allocation_override(Some(0), None),
            Some(CoreAllocation::AllCores)
        );
        assert_eq!(core_allocation_override(None, None), None);
    }

    #[test]
    fn http_admin_bind_override_sets_custom_bind() {
        let settings = http_admin_bind_override(Some("0.0.0.0:18080".to_string()));
        assert_eq!(
            settings.map(|s| s.bind_address),
            Some("0.0.0.0:18080".to_string())
        );
    }

    #[test]
    fn http_admin_bind_override_none_keeps_config_value() {
        assert!(http_admin_bind_override(None).is_none());
    }

    #[test]
    fn parse_validate_and_exit_flag() {
        let args = Args::parse_from([
            "df_engine",
            "--config",
            "config.yaml",
            "--validate-and-exit",
        ]);
        assert!(args.validate_and_exit);
        assert_eq!(args.config, PathBuf::from("config.yaml"));
    }

    #[test]
    fn missing_config_even_with_validate_flag() {
        let result = Args::try_parse_from(["df_engine", "--validate-and-exit"]);
        match result {
            Ok(_) => panic!("Expected missing required argument error"),
            Err(err) => assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument),
        }
    }

    #[test]
    fn validate_unknown_component_rejected() {
        let pipeline_group_id: PipelineGroupId = "test_group".into();
        let pipeline_id: PipelineId = "test_pipeline".into();
        let yaml = r#"
nodes:
  receiver:
    type: "urn:fake:unknown:receiver"
    config: {}
  exporter:
    type: noop:exporter
    config: {}
connections:
  - from: receiver
    to: exporter
"#;

        let pipeline_cfg =
            PipelineConfig::from_yaml(pipeline_group_id.clone(), pipeline_id.clone(), yaml)
                .expect("pipeline YAML should parse");

        let err = validate_pipeline_components(&pipeline_group_id, &pipeline_id, &pipeline_cfg)
            .expect_err("semantic component validation should fail");
        assert!(err.to_string().contains("Unknown receiver component"));
    }

    #[test]
    fn args_reject_conflicting_core_allocation_flags() {
        let err = Args::try_parse_from([
            "df_engine",
            "--config",
            "config.yaml",
            "--num-cores",
            "2",
            "--core-id-range",
            "0-3",
        ])
        .expect_err("clap should reject conflicting options");
        let msg = err.to_string();
        assert!(msg.contains("--num-cores"));
        assert!(msg.contains("--core-id-range"));
    }

    #[test]
    fn args_accept_num_cores_and_http_admin_bind() {
        let args = Args::try_parse_from([
            "df_engine",
            "--config",
            "config.yaml",
            "--num-cores",
            "2",
            "--http-admin-bind",
            "0.0.0.0:28080",
        ])
        .expect("args should parse");
        assert_eq!(args.num_cores, Some(2));
        assert!(args.core_id_range.is_none());
        assert_eq!(args.http_admin_bind.as_deref(), Some("0.0.0.0:28080"));
    }

    #[test]
    fn args_accept_core_id_range() {
        let args = Args::try_parse_from([
            "df_engine",
            "--config",
            "config.yaml",
            "--core-id-range",
            "1..=3,7",
        ])
        .expect("args should parse");
        assert_eq!(
            args.core_id_range,
            Some(CoreAllocation::CoreSet {
                set: vec![
                    CoreRange { start: 1, end: 3 },
                    CoreRange { start: 7, end: 7 }
                ]
            })
        );
        assert_eq!(args.num_cores, None);
    }

    #[test]
    fn apply_cli_overrides_updates_top_level_resources_and_http_admin() {
        let mut cfg =
            OtelDataflowSpec::from_yaml(minimal_engine_yaml()).expect("base config should parse");
        apply_cli_overrides(&mut cfg, Some(3), None, Some("0.0.0.0:28080".to_string()));

        assert_eq!(
            cfg.policies.resources.core_allocation,
            CoreAllocation::CoreCount { count: 3 }
        );
        assert_eq!(
            cfg.engine
                .http_admin
                .as_ref()
                .map(|s| s.bind_address.as_str()),
            Some("0.0.0.0:28080")
        );

        let resolved = cfg.resolve();
        let main = resolved
            .pipelines
            .iter()
            .find(|p| p.pipeline_group_id.as_ref() == "default" && p.pipeline_id.as_ref() == "main")
            .expect("default/main should exist");
        assert_eq!(
            main.policies.resources.core_allocation,
            CoreAllocation::CoreCount { count: 3 }
        );
    }

    #[test]
    fn apply_cli_overrides_only_changes_global_resources_policy() {
        let yaml = r#"
version: otel_dataflow/v1
policies:
  resources:
    core_allocation:
      type: core_count
      count: 9
engine: {}
groups:
  default:
    policies:
      resources:
        core_allocation:
          type: core_count
          count: 5
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:example:receiver"
            config: null
          exporter:
            type: "urn:test:example:exporter"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;
        let mut cfg = OtelDataflowSpec::from_yaml(yaml).expect("config should parse");
        apply_cli_overrides(&mut cfg, Some(2), None, None);

        // CLI updates top-level/global policy.
        assert_eq!(
            cfg.policies.resources.core_allocation,
            CoreAllocation::CoreCount { count: 2 }
        );

        // Pipeline resolution keeps precedence (group-level over top-level).
        let resolved = cfg.resolve();
        let main = resolved
            .pipelines
            .iter()
            .find(|p| p.pipeline_group_id.as_ref() == "default" && p.pipeline_id.as_ref() == "main")
            .expect("default/main should exist");
        assert_eq!(
            main.policies.resources.core_allocation,
            CoreAllocation::CoreCount { count: 5 }
        );
    }
}
