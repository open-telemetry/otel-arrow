// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable startup helpers for binaries that embed the OTAP dataflow engine.
//!
//! These functions encapsulate common bootstrapping tasks - CLI override
//! application, component validation, system diagnostics, and the final console
//! shutdown and exit - so that custom distributions can share the same logic
//! without copying code from the default binary entry point.
//!
//! # Example
//!
//! ```ignore
//! use otel_arrow_dfe_controller::startup;
//!
//! let mut cfg = OtelDataflowSpec::from_file(&path)?;
//! startup::apply_cli_overrides(&mut cfg, num_cores, core_id_range, http_admin_bind);
//! startup::validate_engine_components(&cfg, &MY_PIPELINE_FACTORY)?;
//! startup::validate_controller_extensions(&cfg, &ControllerRunOptions::default().extensions)?;
//! println!("{}", startup::system_info(&MY_PIPELINE_FACTORY, "system"));
//! let result = Controller::new(&MY_PIPELINE_FACTORY).run_forever(cfg);
//! startup::shutdown_console_and_exit(&result);
//! ```

use crate::{CONTROLLER_EXTENSION_FACTORIES, ControllerExtensionRegistry};
use otel_arrow_dfe_config::engine::{HttpAdminSettings, OtelDataflowSpec};
use otel_arrow_dfe_config::node::NodeKind;
use otel_arrow_dfe_config::pipeline::PipelineConfig;
use otel_arrow_dfe_config::policy::{CoreAllocation, ResolvedPolicies, ResourcesPolicy};
use otel_arrow_dfe_config::{PipelineGroupId, PipelineId};
use otel_arrow_dfe_engine::PipelineFactory;
use otel_arrow_dfe_telemetry::output_service::{
    OutputService, OutputServiceConfig, ShutdownOutcome,
};
use std::fmt::{Debug, Display};
use std::io::Write as _;
use std::sync::mpsc;
use std::time::Duration;
use sysinfo::System;

/// Upper bound on the final status write, so a stalled stderr reader cannot keep the process alive.
const EXIT_STATUS_WRITE_TIMEOUT: Duration = Duration::from_secs(1);

/// Resolves `num_cores` / `core_id_range` CLI flags into a single
/// [`CoreAllocation`] value, if any override was provided.
///
/// Priority: an explicit core-ID range takes precedence over a plain count.
/// A count of `0` is interpreted as "all cores" (`CoreAllocation::all_cores()`).
#[must_use]
pub fn core_allocation_override(
    num_cores: Option<usize>,
    core_id_range: Option<CoreAllocation>,
) -> Option<CoreAllocation> {
    match (core_id_range, num_cores) {
        (Some(range), _) => Some(range),
        (None, Some(0)) => Some(CoreAllocation::all_cores()),
        (None, Some(count)) => Some(CoreAllocation::core_count(count)),
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
        resources.core_allocation = Some(core_allocation);
        engine_cfg.policies.set_resources(resources);
    }
    if let Some(http_admin) = http_admin_bind_override(http_admin_bind) {
        engine_cfg.engine.http_admin = Some(http_admin);
    }
}

/// Validates that every node and extension in a single pipeline references a
/// component URN registered in the given [`PipelineFactory`], and runs
/// per-component config validation.
///
/// Structural config validation (connections, node references, policies) is
/// already performed during config deserialization
/// ([`OtelDataflowSpec::from_file`]).  This function adds the semantic check
/// that all referenced components are actually compiled into the binary, and
/// validates their node/extension-specific config statically.
///
/// This per-pipeline helper does not validate node rate-limiter bindings because
/// it does not receive the effective policy catalog. Use
/// [`validate_engine_components`] when validating a complete engine config.
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
            NodeKind::Processor => factory
                .get_processor_factory_map()
                .get(urn_str)
                .map(|f| f.validate_config),
            NodeKind::Exporter => factory
                .get_exporter_factory_map()
                .get(urn_str)
                .map(|f| f.validate_config),
        };

        match validate_config_fn {
            None => {
                let kind_name = match kind {
                    NodeKind::Receiver => "receiver",
                    NodeKind::Processor => "processor",
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

    // Mirror the per-node validation pass for extensions. Extensions are no
    // longer `NodeKind::Extension` (they live in `pipeline_cfg.extensions`,
    // not `pipeline_cfg.nodes`), so they would otherwise slip past static
    // validation entirely and only fail at runtime when the engine tries to
    // resolve them in `get_extension_factory_map()`.
    for (ext_id, ext_cfg) in pipeline_cfg.extension_iter() {
        let urn_str = ext_cfg.r#type.as_str();
        match factory.get_extension_factory_map().get(urn_str) {
            None => {
                return Err(std::io::Error::other(format!(
                    "Unknown extension component `{}` in pipeline_group={} pipeline={} extension={}",
                    urn_str,
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    ext_id.as_ref()
                ))
                .into());
            }
            Some(factory) => {
                (factory.validate_config)(&ext_cfg.config).map_err(|e| {
                    std::io::Error::other(format!(
                        "Invalid config for extension `{}` in pipeline_group={} pipeline={} extension={}: {}",
                        urn_str,
                        pipeline_group_id.as_ref(),
                        pipeline_id.as_ref(),
                        ext_id.as_ref(),
                        e
                    ))
                })?;
            }
        }
    }

    Ok(())
}

fn validate_rate_limiter_bindings(
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    pipeline_cfg: &PipelineConfig,
    policies: &ResolvedPolicies,
) -> Result<(), Box<dyn std::error::Error>> {
    for (node_id, node_cfg) in pipeline_cfg.node_iter() {
        match node_cfg.rate_limiters.as_deref() {
            None | Some([]) => {}
            Some([limiter_name]) => {
                if !policies.rate_limiters.contains_key(limiter_name) {
                    return Err(std::io::Error::other(format!(
                        "Component `{}` in pipeline_group={} pipeline={} node={}: rate limiter binding '{}' does not name an effective limiter",
                        node_cfg.r#type.as_ref(),
                        pipeline_group_id.as_ref(),
                        pipeline_id.as_ref(),
                        node_id.as_ref(),
                        limiter_name,
                    ))
                    .into());
                }
            }
            Some(limiter_names) => {
                return Err(std::io::Error::other(format!(
                    "Component `{}` in pipeline_group={} pipeline={} node={}: V1 supports at most one rate limiter binding per node; found {}",
                    node_cfg.r#type.as_ref(),
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    node_id.as_ref(),
                    limiter_names.len(),
                ))
                .into());
            }
        }
    }

    Ok(())
}

/// Validates that every node in every pipeline (including the engine
/// observability pipeline) references a component URN registered in the
/// given [`PipelineFactory`].
///
/// This is the top-level validation entry point that iterates over all
/// pipeline groups, all pipelines within each group, and the observability
/// pipeline.
pub fn validate_engine_components<PData: 'static + Clone + Debug>(
    engine_cfg: &OtelDataflowSpec,
    factory: &PipelineFactory<PData>,
) -> Result<(), Box<dyn std::error::Error>> {
    for resolved in engine_cfg.resolve().pipelines {
        validate_pipeline_components(
            &resolved.pipeline_group_id,
            &resolved.pipeline_id,
            &resolved.pipeline,
            factory,
        )?;
        validate_rate_limiter_bindings(
            &resolved.pipeline_group_id,
            &resolved.pipeline_id,
            &resolved.pipeline,
            &resolved.policies,
        )?;
    }

    Ok(())
}

/// Validates configured controller extensions against the supplied registry.
///
/// This is static validation only: it verifies that each configured controller
/// extension type has a registered factory and that the extension-specific
/// config validates without starting the extension.
pub fn validate_controller_extensions(
    engine_cfg: &OtelDataflowSpec,
    registry: &ControllerExtensionRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    for (extension_id, extension) in engine_cfg.engine.controller.extensions.iter() {
        let urn_str = extension.r#type.as_str();
        match registry.get(&extension.r#type) {
            None => {
                return Err(std::io::Error::other(format!(
                    "Unknown controller extension `{}` in engine.controller.extensions extension={}",
                    urn_str,
                    extension_id.as_ref()
                ))
                .into());
            }
            Some(factory) => {
                (factory.validate_config)(&extension.config).map_err(|e| {
                    std::io::Error::other(format!(
                        "Invalid config for controller extension `{}` in engine.controller.extensions extension={}: {}",
                        urn_str,
                        extension_id.as_ref(),
                        e
                    ))
                })?;
            }
        }
    }

    Ok(())
}

/// Returns a human-readable string with system information, all component URNs
/// registered in the given [`PipelineFactory`], and linked controller extension
/// URNs.
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
        "\n\n\u{26A0}\u{FE0F}  WARNING: This binary was compiled in debug mode.
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
    let mut controller_extensions_sorted: Vec<&str> = CONTROLLER_EXTENSION_FACTORIES
        .iter()
        .map(|f| f.name)
        .collect();
    receivers_sorted.sort();
    processors_sorted.sort();
    exporters_sorted.sort();
    controller_extensions_sorted.sort();

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
  Controller Extensions: {}

Example configuration files can be found in the configs/ directory.{}",
        available_cores,
        available_memory_gb,
        total_memory_gb,
        build_mode,
        memory_allocator,
        receivers_sorted.join(", "),
        processors_sorted.join(", "),
        exporters_sorted.join(", "),
        controller_extensions_sorted.join(", "),
        debug_warning
    )
}

/// Stops the console writers after an engine run, reports its final status, and exits.
///
/// Only a process host may call this, because the console streams accept no
/// frames afterwards. The status is written only once both writers stopped: a
/// writer that missed its deadline may still hold its stream lock. The write
/// gives up after a short bound and ignores errors, so a stalled or closed
/// stderr can neither hang the exit nor turn it into a panic.
pub fn shutdown_console_and_exit<E: Display>(result: &Result<(), E>) -> ! {
    let output = OutputService::shutdown(OutputServiceConfig::default().shutdown_drain_deadline);
    let (exit_code, status) = terminal_status(result, output);
    exit_with_status(exit_code, status.as_deref())
}

/// Chooses a process exit code and an optional status that is safe to write.
fn terminal_status<E: Display>(
    result: &Result<(), E>,
    output: ShutdownOutcome,
) -> (i32, Option<String>) {
    if output.deadline_expired {
        // At least one writer may still hold its stream lock.
        return (1, None);
    }
    if !output.drained {
        let status = match result {
            Ok(()) => format!(
                "Console output was incomplete: {} frame(s) were not written",
                output.frames_pending
            ),
            Err(error) => format!("Pipeline failed to run: {error}"),
        };
        return (1, Some(status));
    }
    match result {
        Ok(()) => (0, Some("Pipeline ran successfully".to_owned())),
        Err(error) => (1, Some(format!("Pipeline failed to run: {error}"))),
    }
}

/// Writes `status` to stderr within [`EXIT_STATUS_WRITE_TIMEOUT`], then exits with `exit_code`.
fn exit_with_status(exit_code: i32, status: Option<&str>) -> ! {
    if let Some(status) = status {
        write_status_bounded(status, EXIT_STATUS_WRITE_TIMEOUT);
    }
    std::process::exit(exit_code)
}

/// Writes one status line on a helper thread, waiting for it at most `timeout`.
///
/// A write blocked on a full pipe then holds only the helper thread, which
/// process exit ends. The result is ignored, because `eprintln!` would panic
/// when stderr is closed.
fn write_status_bounded(status: &str, timeout: Duration) {
    let line = format!("{status}\n");
    let (done_tx, done_rx) = mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("otap-exit-status".to_owned())
        .spawn(move || {
            let _ = std::io::stderr().write_all(line.as_bytes());
            let _ = done_tx.send(());
        });
    if spawned.is_ok() {
        let _ = done_rx.recv_timeout(timeout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::policy::{CoreRange, Policies};
    use otel_arrow_dfe_config::{PipelineGroupId, PipelineId, node::NodeUserConfig};
    use otel_arrow_dfe_engine::config::{ExporterConfig, ProcessorConfig, ReceiverConfig};
    use otel_arrow_dfe_engine::context::PipelineContext;
    use otel_arrow_dfe_engine::exporter::ExporterWrapper;
    use otel_arrow_dfe_engine::processor::ProcessorWrapper;
    use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
    use otel_arrow_dfe_engine::wiring_contract::WiringContract;
    use otel_arrow_dfe_engine::{ExporterFactory, ProcessorFactory, ReceiverFactory};
    use std::process::{Command, Stdio};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    fn test_receiver_create(
        _pipeline_ctx: PipelineContext,
        _node: otel_arrow_dfe_engine::node::NodeId,
        _node_config: Arc<NodeUserConfig>,
        _receiver_config: &ReceiverConfig,
        _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
    ) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
        panic!("test receiver factory should not be constructed")
    }

    fn test_exporter_create(
        _pipeline_ctx: PipelineContext,
        _node: otel_arrow_dfe_engine::node::NodeId,
        _node_config: Arc<NodeUserConfig>,
        _exporter_config: &ExporterConfig,
        _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
    ) -> Result<ExporterWrapper<()>, otel_arrow_dfe_config::error::Error> {
        panic!("test exporter factory should not be constructed")
    }

    fn test_processor_create(
        _pipeline_ctx: PipelineContext,
        _node: otel_arrow_dfe_engine::node::NodeId,
        _node_config: Arc<NodeUserConfig>,
        _processor_config: &ProcessorConfig,
        _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
    ) -> Result<ProcessorWrapper<()>, otel_arrow_dfe_config::error::Error> {
        panic!("test processor factory should not be constructed")
    }

    fn test_factory() -> PipelineFactory<()> {
        let receiver_factories = Box::leak(Box::new([
            ReceiverFactory {
                name: "urn:test:receiver:example",
                create: test_receiver_create,
                context_declarations: None,
                wiring_contract: WiringContract::UNRESTRICTED,
                validate_config: otel_arrow_dfe_config::validation::no_config,
            },
            ReceiverFactory {
                name: "urn:otel:receiver:internal_telemetry",
                create: test_receiver_create,
                context_declarations: None,
                wiring_contract: WiringContract::UNRESTRICTED,
                validate_config: otel_arrow_dfe_config::validation::no_config,
            },
        ]));
        let processor_factories = Box::leak(Box::new([ProcessorFactory {
            name: "urn:otel:processor:type_router",
            create: test_processor_create,
            context_declarations: None,
            wiring_contract: WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::no_config,
        }]));
        let exporter_factories = Box::leak(Box::new([
            ExporterFactory {
                name: "urn:test:exporter:example",
                create: test_exporter_create,
                context_declarations: None,
                wiring_contract: WiringContract::UNRESTRICTED,
                validate_config: otel_arrow_dfe_config::validation::no_config,
            },
            ExporterFactory {
                name: "urn:otel:exporter:console",
                create: test_exporter_create,
                context_declarations: None,
                wiring_contract: WiringContract::UNRESTRICTED,
                validate_config: otel_arrow_dfe_config::validation::no_config,
            },
            ExporterFactory {
                name: "urn:otel:exporter:noop",
                create: test_exporter_create,
                context_declarations: None,
                wiring_contract: WiringContract::UNRESTRICTED,
                validate_config: otel_arrow_dfe_config::validation::no_config,
            },
        ]));
        PipelineFactory::new(
            receiver_factories,
            processor_factories,
            exporter_factories,
            &[],
        )
    }

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

    fn rate_limited_engine_yaml(unit: &str, binding: Option<&str>) -> String {
        let binding = binding
            .map(|binding| format!("            rate_limiters: {binding}\n"))
            .unwrap_or_default();
        format!(
            r#"
version: otel_dataflow/v1
policies:
  resources:
    memory_limiter:
      mode: enforce
      source: auto
    rate_limiters:
      ingress:
        enforcement: enforce
        aggregation: receiver_instance
        unit: {unit}
        pressure: soft
        token_bucket:
          allow: 1
          interval: 1s
          burst: 1
engine: {{}}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
{binding}            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#
        )
    }

    #[test]
    fn core_allocation_override_prefers_range() {
        let range = CoreAllocation::core_set(vec![CoreRange { start: 2, end: 4 }]);
        let resolved = core_allocation_override(Some(3), Some(range.clone()));
        assert_eq!(resolved, Some(range));
    }

    #[test]
    fn core_allocation_override_maps_num_cores() {
        assert_eq!(
            core_allocation_override(Some(5), None),
            Some(CoreAllocation::core_count(5))
        );
        assert_eq!(
            core_allocation_override(Some(0), None),
            Some(CoreAllocation::all_cores())
        );
        assert_eq!(core_allocation_override(None, None), None);
    }

    #[test]
    fn http_admin_bind_override_sets_custom_bind() {
        let settings = http_admin_bind_override(Some("127.0.0.1:18080".to_string()));
        assert_eq!(
            settings.map(|s| s.bind_address),
            Some("127.0.0.1:18080".to_string())
        );
    }

    #[test]
    fn http_admin_bind_override_none_keeps_config_value() {
        assert!(http_admin_bind_override(None).is_none());
    }

    #[test]
    fn validate_pipeline_components_rejects_unknown_extension() {
        // Pipeline with no nodes and one extension referencing a URN
        // that is NOT registered in the empty PipelineFactory below.
        // Mirrors the validation behaviour we already give to unknown
        // node URNs: caught at static validation, not at runtime.
        let yaml = r#"
extensions:
  not-registered:
    type: "urn:test:extension:does-not-exist"
"#;
        let pipeline_cfg =
            PipelineConfig::from_yaml("g".into(), "p".into(), yaml).expect("yaml parses");
        let factory: PipelineFactory<()> = PipelineFactory::new(&[], &[], &[], &[]);

        let err = validate_pipeline_components(
            &PipelineGroupId::from("g"),
            &PipelineId::from("p"),
            &pipeline_cfg,
            &factory,
        )
        .expect_err("unknown extension URN must be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("Unknown extension component"),
            "unexpected error: {msg}"
        );
        assert!(
            msg.contains("urn:test:extension:does-not-exist"),
            "unexpected error: {msg}"
        );
        assert!(msg.contains("not-registered"), "unexpected error: {msg}");
    }

    /// Scenario: a custom receiver is registered while a rate limiter is configured.
    /// Guarantees: static component validation accepts the pipeline and leaves admission
    /// dimension validation to construction-time binding by the participating component.
    #[test]
    fn validate_engine_components_defers_rate_dimension_to_binding() {
        let cfg = OtelDataflowSpec::from_yaml(&rate_limited_engine_yaml("messages", None))
            .expect("rate-limited config should parse");
        let factory = test_factory();

        validate_engine_components(&cfg, &factory)
            .expect("registered components should pass static validation");
    }

    /// Scenario: a receiver binds a limiter name absent from its resolved policy scope.
    /// Guarantees: startup validation rejects the unknown binding before pipeline construction.
    #[test]
    fn validate_engine_components_rejects_unknown_rate_limiter_binding() {
        let cfg =
            OtelDataflowSpec::from_yaml(&rate_limited_engine_yaml("messages", Some("[missing]")))
                .expect("rate-limited config should parse");

        let error = validate_engine_components(&cfg, &test_factory())
            .expect_err("unknown limiter binding must fail static validation");
        assert!(
            error
                .to_string()
                .contains("rate limiter binding 'missing' does not name an effective limiter")
        );
    }

    /// Scenario: a receiver selects more than one limiter in a V1 node binding.
    /// Guarantees: startup validation rejects unsupported multi-limiter bindings before construction.
    #[test]
    fn validate_engine_components_rejects_multiple_rate_limiter_bindings() {
        let cfg = OtelDataflowSpec::from_yaml(&rate_limited_engine_yaml(
            "messages",
            Some("[ingress, other]"),
        ))
        .expect("rate-limited config should parse");

        let error = validate_engine_components(&cfg, &test_factory())
            .expect_err("multiple limiter bindings must fail static validation");
        assert!(
            error
                .to_string()
                .contains("V1 supports at most one rate limiter binding per node; found 2")
        );
    }

    /// Scenario: a receiver explicitly opts out while an inherited limiter is effective.
    /// Guarantees: startup validation accepts an empty node-level limiter binding.
    #[test]
    fn validate_engine_components_accepts_rate_limiter_opt_out() {
        let cfg = OtelDataflowSpec::from_yaml(&rate_limited_engine_yaml("messages", Some("[]")))
            .expect("rate-limited config should parse");

        validate_engine_components(&cfg, &test_factory())
            .expect("an explicit empty binding should pass static validation");
    }

    /// Scenario: a pipeline declares multiple limiters and a receiver explicitly binds one.
    /// Guarantees: startup validates only the selected limiter instead of forcing every policy on every receiver.
    #[test]
    fn validate_engine_components_uses_explicit_receiver_binding() {
        let yaml = r#"
version: otel_dataflow/v1
policies:
  resources:
    memory_limiter:
      mode: enforce
      source: auto
    rate_limiters:
      bytes:
        enforcement: enforce
        aggregation: receiver_instance
        unit: request_bytes
        pressure: soft
        token_bucket: { allow: 1024, interval: 1s, burst: 1024 }
      records:
        enforcement: enforce
        aggregation: receiver_instance
        unit: messages
        pressure: soft
        token_bucket: { allow: 10, interval: 1s, burst: 10 }
engine: {}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            rate_limiters: [records]
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;
        let cfg = OtelDataflowSpec::from_yaml(yaml).expect("named binding config should parse");
        let factory = test_factory();

        validate_engine_components(&cfg, &factory)
            .expect("only the explicitly selected limiter should be validated");
    }

    /// Scenario: the component factory omits the built-in observability components.
    /// Guarantees: engine validation checks the default pipeline under `system/observability`.
    #[test]
    fn validate_engine_components_checks_default_observability_pipeline() {
        let config = OtelDataflowSpec::from_yaml(
            r#"
version: otel_dataflow/v1
groups: {}
"#,
        )
        .expect("minimal engine config should parse");
        let factory: PipelineFactory<()> = PipelineFactory::new(&[], &[], &[], &[]);

        let error = validate_engine_components(&config, &factory)
            .expect_err("missing observability components must fail validation");
        let message = error.to_string();
        assert!(
            message.contains("Unknown ") && message.contains(" component `urn:otel:"),
            "unexpected error: {message}"
        );
        assert!(
            message.contains("pipeline_group=system pipeline=observability"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn system_info_lists_controller_extensions() {
        let factory: PipelineFactory<()> = PipelineFactory::new(&[], &[], &[], &[]);
        let info = system_info(&factory, "system");

        assert!(
            info.contains("Controller Extensions:"),
            "system info should include controller extension URNs: {info}"
        );
        assert!(
            info.contains(crate::CONTROLLER_MONITOR_EXTENSION_URN),
            "system info should include linked controller monitor extension: {info}"
        );
    }

    #[test]
    fn apply_cli_overrides_updates_top_level_resources_and_http_admin() {
        let mut cfg =
            OtelDataflowSpec::from_yaml(minimal_engine_yaml()).expect("base config should parse");
        apply_cli_overrides(&mut cfg, Some(3), None, Some("127.0.0.1:28080".to_string()));

        assert_eq!(
            Policies::resolve([&cfg.policies]).resources.core_allocation,
            CoreAllocation::core_count(3)
        );
        assert_eq!(
            cfg.engine
                .http_admin
                .as_ref()
                .map(|s| s.bind_address.as_str()),
            Some("127.0.0.1:28080")
        );

        let resolved = cfg.resolve();
        let main = resolved
            .pipelines
            .iter()
            .find(|p| p.pipeline_group_id.as_ref() == "default" && p.pipeline_id.as_ref() == "main")
            .expect("default/main should exist");
        assert_eq!(
            main.policies.resources.core_allocation,
            CoreAllocation::core_count(3)
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
            CoreAllocation::core_count(2)
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
            CoreAllocation::core_count(5)
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
            CoreAllocation::core_count(4),
            "--num-cores 4 must not be shadowed by an implicit group-level resources default"
        );
    }

    /// Scenario: the process host maps run and writer outcomes to its final status.
    /// Guarantees: success is reported only after a complete drain, joined failures
    /// remain visible, and any possibly running writer suppresses direct output.
    #[test]
    fn terminal_status_respects_writer_lifetime() {
        let drained = ShutdownOutcome::default();
        assert_eq!(
            terminal_status::<&str>(&Ok(()), drained),
            (0, Some("Pipeline ran successfully".to_owned()))
        );
        assert_eq!(
            terminal_status(&Err("engine failed"), drained),
            (1, Some("Pipeline failed to run: engine failed".to_owned()))
        );

        let writer_failed = ShutdownOutcome {
            drained: false,
            writer_failed: true,
            deadline_expired: false,
            frames_pending: 3,
        };
        assert_eq!(
            terminal_status::<&str>(&Ok(()), writer_failed),
            (
                1,
                Some("Console output was incomplete: 3 frame(s) were not written".to_owned())
            )
        );
        assert_eq!(
            terminal_status(&Err("engine failed"), writer_failed),
            (1, Some("Pipeline failed to run: engine failed".to_owned()))
        );

        let timed_out = ShutdownOutcome {
            drained: false,
            writer_failed: false,
            deadline_expired: true,
            frames_pending: 2,
        };
        assert_eq!(terminal_status::<&str>(&Ok(()), timed_out), (1, None));

        let mixed = ShutdownOutcome {
            writer_failed: true,
            ..timed_out
        };
        assert_eq!(terminal_status(&Err("engine failed"), mixed), (1, None));
    }

    /// Environment variable that turns `exit_status_child_process` into the child under test.
    const EXIT_STATUS_CHILD_ENV: &str = "OTAP_EXIT_STATUS_CHILD";

    /// Exit code the child reports, distinct from success, panics, and test failures.
    const CHILD_EXIT_CODE: i32 = 3;

    /// Scenario: runs as the child process that the exit-status tests below spawn.
    /// Guarantees: when spawned, it reports a status on its inherited stderr and exits
    /// through the same path as the process host; as an ordinary test it does nothing.
    #[test]
    fn exit_status_child_process() {
        if std::env::var_os(EXIT_STATUS_CHILD_ENV).is_some() {
            exit_with_status(CHILD_EXIT_CODE, Some("final status that nobody reads"));
        }
    }

    /// Runs `exit_status_child_process` in a new process whose stderr is `stderr`.
    ///
    /// Returns the child's exit code, and fails if the child does not exit in time.
    fn run_exit_status_child(stderr: std::io::PipeWriter) -> Option<i32> {
        let mut child = Command::new(std::env::current_exe().expect("test binary path"))
            .args([
                "--exact",
                "startup::tests::exit_status_child_process",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(EXIT_STATUS_CHILD_ENV, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr)
            .spawn()
            .expect("child process starts");
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait().expect("child status is readable") {
                return status.code();
            }
            if started.elapsed() > Duration::from_secs(30) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("the child never exited, so its final status write is unbounded");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Writes single bytes until the pipe is full, since one byte also fills the last free space.
    fn fill_pipe(mut writer: std::io::PipeWriter, written: &AtomicU64) {
        while writer.write_all(b"x").is_ok() {
            let _ = written.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Waits until the filler has made no progress for a while, which means the pipe is full.
    fn wait_until_pipe_full(written: &AtomicU64) {
        let started = Instant::now();
        let mut last = written.load(Ordering::Relaxed);
        let mut unchanged_since = Instant::now();
        loop {
            std::thread::sleep(Duration::from_millis(20));
            let current = written.load(Ordering::Relaxed);
            if current != last {
                last = current;
                unchanged_since = Instant::now();
            } else if current > 0 && unchanged_since.elapsed() >= Duration::from_millis(300) {
                return;
            }
            assert!(
                started.elapsed() < Duration::from_secs(30),
                "the pipe never filled up"
            );
        }
    }

    /// Scenario: the process host writes its final status while stderr is a pipe that is
    /// full and never read.
    /// Guarantees: the status write gives up after its bound and the process still exits
    /// with the run's exit code, instead of hanging on stderr forever.
    #[test]
    fn exit_status_write_does_not_wait_on_a_full_stderr_pipe() {
        let (reader, writer) = std::io::pipe().expect("pipe is created");
        let filler_writer = writer.try_clone().expect("pipe writer is cloned");
        let written = Arc::new(AtomicU64::new(0));
        // Detached: its last write stays blocked until the reader below is dropped.
        let _filler = std::thread::spawn({
            let written = Arc::clone(&written);
            move || fill_pipe(filler_writer, &written)
        });
        wait_until_pipe_full(&written);

        assert_eq!(run_exit_status_child(writer), Some(CHILD_EXIT_CODE));
        drop(reader);
    }

    /// Scenario: the process host writes its final status while stderr is a pipe whose
    /// reader has already closed.
    /// Guarantees: the failed write is ignored instead of panicking, so the process still
    /// exits with the run's exit code.
    #[test]
    fn exit_status_write_survives_a_closed_stderr_pipe() {
        let (reader, writer) = std::io::pipe().expect("pipe is created");
        drop(reader);

        assert_eq!(run_exit_status_child(writer), Some(CHILD_EXIT_CODE));
    }
}
