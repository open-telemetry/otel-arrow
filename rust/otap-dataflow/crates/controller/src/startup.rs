// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable startup helpers for binaries that embed the OTAP dataflow engine.
//!
//! These functions encapsulate common bootstrapping tasks - CLI override
//! application, component validation, and system diagnostics - so that custom
//! distributions can share the same logic without copying code from the
//! default binary entry point.
//!
//! # Example
//!
//! ```ignore
//! use otap_df_controller::startup;
//!
//! let mut cfg = OtelDataflowSpec::from_file(&path)?;
//! startup::apply_cli_overrides(&mut cfg, num_cores, core_id_range, http_admin_bind);
//! startup::validate_engine_components(&cfg, &MY_PIPELINE_FACTORY)?;
//! println!("{}", startup::system_info(&MY_PIPELINE_FACTORY, "system"));
//! ```

use otap_df_config::engine::{
    HttpAdminSettings, OtelDataflowSpec, SYSTEM_OBSERVABILITY_PIPELINE_ID, SYSTEM_PIPELINE_GROUP_ID,
};
use otap_df_config::node::NodeKind;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::policy::{CoreAllocation, ResourcesPolicy};
use otap_df_config::{PipelineGroupId, PipelineId};
use otap_df_engine::PipelineFactory;
use std::fmt::Debug;
use sysinfo::System;

/// Resolves `num_cores` / `core_id_range` CLI flags into a single
/// [`CoreAllocation`] value, if any override was provided.
///
/// Priority: an explicit core-ID range takes precedence over a plain count.
/// A count of `0` is interpreted as [`CoreAllocation::AllCores`].
#[must_use]
pub fn core_allocation_override(
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

/// Converts an optional bind-address string into [`HttpAdminSettings`].
#[must_use]
pub fn http_admin_bind_override(http_admin_bind: Option<String>) -> Option<HttpAdminSettings> {
    http_admin_bind.map(|bind_address| HttpAdminSettings { bind_address })
}

/// Applies core-allocation and HTTP-admin bind overrides to an
/// [`OtelDataflowSpec`].
///
/// This is the standard way for CLI entry points to merge command-line flags
/// into a parsed configuration before starting the engine.
pub fn apply_cli_overrides(
    engine_cfg: &mut OtelDataflowSpec,
    num_cores: Option<usize>,
    core_id_range: Option<CoreAllocation>,
    http_admin_bind: Option<String>,
) {
    if let Some(core_allocation) = core_allocation_override(num_cores, core_id_range) {
        let mut resources = engine_cfg
            .policies
            .resources()
            .cloned()
            .unwrap_or_else(ResourcesPolicy::default);
        resources.core_allocation = core_allocation;
        engine_cfg.policies.set_resources(resources);
    }
    if let Some(http_admin) = http_admin_bind_override(http_admin_bind) {
        engine_cfg.engine.http_admin = Some(http_admin);
    }
}

/// Validates that every node in a single pipeline references a component URN
/// registered in the given [`PipelineFactory`], and runs per-component config
/// validation.
///
/// Structural config validation (connections, node references, policies) is
/// already performed during config deserialization
/// ([`OtelDataflowSpec::from_file`]).  This function adds the semantic check
/// that all referenced components are actually compiled into the binary, and
/// validates their node-specific config statically.
///
/// **Scope:** This is *static* validation only -- it checks that config values
/// can be deserialized into the expected types.  It does **not** detect runtime
/// issues such as port conflicts, unreachable endpoints, or missing files.
pub fn validate_pipeline_components<PData: 'static + Clone + Debug>(
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    pipeline_cfg: &PipelineConfig,
    factory: &PipelineFactory<PData>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (node_id, node_cfg) in pipeline_cfg.node_iter() {
        let kind = node_cfg.kind();
        let urn_str = node_cfg.r#type.as_str();

        let validate_config_fn = match kind {
            NodeKind::Receiver => factory
                .get_receiver_factory_map()
                .get(urn_str)
                .map(|f| f.validate_config),
            NodeKind::Processor | NodeKind::ProcessorChain => factory
                .get_processor_factory_map()
                .get(urn_str)
                .map(|f| f.validate_config),
            NodeKind::Exporter => factory
                .get_exporter_factory_map()
                .get(urn_str)
                .map(|f| f.validate_config),
            NodeKind::Extension => {
                // Extensions are not yet validated here because PipelineFactory
                // does not have an extension factory registry. Once one is added,
                // this should look up and validate extension configs similarly to
                // receivers/processors/exporters.
                continue;
            }
        };

        match validate_config_fn {
            None => {
                let kind_name = match kind {
                    NodeKind::Receiver => "receiver",
                    NodeKind::Processor | NodeKind::ProcessorChain => "processor",
                    NodeKind::Exporter => "exporter",
                    NodeKind::Extension => unreachable!("handled above"),
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

/// Validates that every node in every pipeline (including the observability
/// pipeline, if configured) references a component URN registered in the
/// given [`PipelineFactory`].
///
/// This is the top-level validation entry point that iterates over all
/// pipeline groups, all pipelines within each group, and the optional
/// observability pipeline.
pub fn validate_engine_components<PData: 'static + Clone + Debug>(
    engine_cfg: &OtelDataflowSpec,
    factory: &PipelineFactory<PData>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (pipeline_group_id, pipeline_group) in &engine_cfg.groups {
        for (pipeline_id, pipeline_cfg) in &pipeline_group.pipelines {
            validate_pipeline_components(pipeline_group_id, pipeline_id, pipeline_cfg, factory)?;
        }
    }

    // Also validate the observability pipeline nodes, if configured.
    if let Some(obs_pipeline) = &engine_cfg.engine.observability.pipeline {
        let obs_group_id: PipelineGroupId = SYSTEM_PIPELINE_GROUP_ID.into();
        let obs_pipeline_id: PipelineId = SYSTEM_OBSERVABILITY_PIPELINE_ID.into();
        let obs_pipeline_config = obs_pipeline.clone().into_pipeline_config();
        validate_pipeline_components(
            &obs_group_id,
            &obs_pipeline_id,
            &obs_pipeline_config,
            factory,
        )?;
    }

    Ok(())
}

/// Returns a human-readable string with system information and all component
/// URNs registered in the given [`PipelineFactory`].
///
/// `memory_allocator` should describe the active global allocator (e.g.
/// `"jemalloc"`, `"mimalloc"`, or `"system"`).  The library cannot detect this
/// automatically because allocator selection is a feature of the final binary
/// crate.
///
/// Useful for diagnostics, `--help` output, or startup banners in any
/// distribution.
#[must_use]
pub fn system_info<PData: 'static + Clone + Debug>(
    factory: &PipelineFactory<PData>,
    memory_allocator: &str,
) -> String {
    let available_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let build_mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
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

    let mut receivers_sorted: Vec<&str> =
        factory.get_receiver_factory_map().keys().copied().collect();
    let mut processors_sorted: Vec<&str> = factory
        .get_processor_factory_map()
        .keys()
        .copied()
        .collect();
    let mut exporters_sorted: Vec<&str> =
        factory.get_exporter_factory_map().keys().copied().collect();
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
    use otap_df_config::policy::Policies;

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
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#
    }

    #[test]
    fn core_allocation_override_prefers_range() {
        let range = CoreAllocation::CoreSet {
            set: vec![otap_df_config::policy::CoreRange { start: 2, end: 4 }],
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
    fn apply_cli_overrides_updates_top_level_resources_and_http_admin() {
        let mut cfg =
            OtelDataflowSpec::from_yaml(minimal_engine_yaml()).expect("base config should parse");
        apply_cli_overrides(&mut cfg, Some(3), None, Some("0.0.0.0:28080".to_string()));

        assert_eq!(
            Policies::resolve([&cfg.policies]).resources.core_allocation,
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
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;
        let mut cfg = OtelDataflowSpec::from_yaml(yaml).expect("config should parse");
        apply_cli_overrides(&mut cfg, Some(2), None, None);

        // CLI updates top-level/global policy.
        assert_eq!(
            Policies::resolve([&cfg.policies]).resources.core_allocation,
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

    #[test]
    fn cli_num_cores_not_shadowed_by_implicit_default_resources() {
        let yaml = r#"
version: otel_dataflow/v1
engine: {}
groups:
  default:
    policies:
      channel_capacity:
        pdata: 500
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;
        let mut cfg = OtelDataflowSpec::from_yaml(yaml).expect("config should parse");
        apply_cli_overrides(&mut cfg, Some(4), None, None);

        let resolved = cfg.resolve();
        let main = resolved
            .pipelines
            .iter()
            .find(|p| p.pipeline_group_id.as_ref() == "default" && p.pipeline_id.as_ref() == "main")
            .expect("default/main should exist");
        assert_eq!(
            main.policies.resources.core_allocation,
            CoreAllocation::CoreCount { count: 4 },
            "--num-cores 4 must not be shadowed by an implicit group-level resources default"
        );
    }
}
