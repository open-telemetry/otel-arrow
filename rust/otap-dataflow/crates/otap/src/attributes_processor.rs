// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attributes processor for OTAP pipelines.
//!
//! This processor provides attribute transformations for telemetry data. It operates
//! on OTAP Arrow payloads (OtapArrowRecords and OtapArrowBytes) and can convert OTLP
//! bytes to OTAP for processing.
//!
//! Supported actions (current subset):
//! - `rename`: Renames an attribute key (non-standard deviation from the Go collector).
//! - `delete`: Removes an attribute by key.
//!
//! Unsupported actions are ignored if present in the config:
//! `insert`, `upsert`, `update` (value update), `hash`, `extract`, `convert`.
//! We may add support for them later.
//!
//! Example configuration (YAML):
//! You can optionally scope the transformation using `apply_to`. Valid values: signal, resource, scope.
//! If omitted, defaults to [signal].
//! ```yaml
//! actions:
//!   - action: "rename"
//!     source_key: "http.method"
//!     destination_key: "rpc.method"       # Renames http.method to rpc.method
//!   - key: "db.statement"
//!     action: "delete"       # Removes db.statement attribute
//!   # apply_to: ["signal", "resource"]  # Optional; defaults to ["signal"]
//! ```
//!
//! Implementation uses otel_arrow_rust::otap::transform::transform_attributes for
//! efficient batch processing of Arrow record batches.

use crate::{OTAP_PROCESSOR_FACTORIES, pdata::OtapPdata};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_telemetry::metrics::MetricSet;
use otel_arrow_rust::otap::transform::{AttributesTransform, DeleteTransform, RenameTransform};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(crate) mod metrics;
pub(crate) mod shared;
use self::metrics::AttributesProcessorMetrics;
use self::shared::{AttributesProcessorTrait, process_attributes};

/// URN for the AttributesProcessor
pub const ATTRIBUTES_PROCESSOR_URN: &str = "urn:otap:processor:attributes_processor";

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Actions that can be performed on attributes.
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    /// Rename an existing attribute key (non-standard; deviates from Go config).
    Rename {
        /// The source key to rename from.
        source_key: String,
        /// The destination key to rename to.
        destination_key: String,
    },

    /// Delete an attribute by key.
    Delete {
        /// The attribute key to delete.
        key: String,
    },

    /// Other actions are accepted for forward-compatibility but ignored.
    /// These variants allow deserialization of Go-style configs without effect.
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Configuration for the AttributesProcessor.
///
/// Accepts configuration in the same format as the OpenTelemetry Collector's attributes processor.
/// Supported actions: rename (deviation), delete. Others are ignored.
///
/// You can control which attribute domains are transformed via `apply_to`.
/// Valid values: "signal" (default), "resource", "scope".
pub struct Config {
    /// List of actions to apply in order.
    #[serde(default)]
    pub actions: Vec<Action>,

    /// Attribute domains to apply transforms to. Defaults to ["signal"].
    #[serde(default)]
    pub apply_to: Option<Vec<String>>,
}

/// Processor that applies attribute transformations to OTAP attribute batches.
///
/// Implements the OpenTelemetry Collector's attributes processor functionality using
/// efficient Arrow operations. Supports `update` (rename) and `delete` operations
/// across all attribute types (resource, scope, and signal-specific attributes)
/// for logs, metrics, and traces telemetry.
pub struct AttributesProcessor {
    // Pre-computed transform to avoid rebuilding per message
    transform: AttributesTransform,
    // Pre-computed flags for domain lookup
    has_resource_domain: bool,
    has_scope_domain: bool,
    has_signal_domain: bool,
    // Metrics handle (set at runtime in factory; None when parsed-only)
    metrics: Option<MetricSet<AttributesProcessorMetrics>>,
}

impl AttributesProcessor {
    /// Creates a new AttributesProcessor from configuration.
    ///
    /// Transforms the Go collector-style configuration into the operations
    /// supported by the underlying Arrow attribute transform API.
    #[must_use = "AttributesProcessor creation may fail and return a ConfigError"]
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let cfg: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse AttributesProcessor configuration: {e}"),
            })?;
        Self::new(cfg)
    }

    /// Creates a new AttributesProcessor with the given parsed configuration.
    fn new(config: Config) -> Result<Self, otap_df_config::error::Error> {
        let mut renames = BTreeMap::new();
        let mut deletes = BTreeSet::new();

        for action in config.actions {
            match action {
                Action::Delete { key } => {
                    let _ = deletes.insert(key);
                }
                Action::Rename {
                    source_key,
                    destination_key,
                } => {
                    let _ = renames.insert(source_key, destination_key);
                }
                // Unsupported actions are ignored for now
                Action::Unsupported => {}
            }
        }

        let domains = shared::parse_apply_to(config.apply_to.as_ref());

        // Pre-compute domain checks
        let has_resource_domain = domains.contains(&shared::ApplyDomain::Resource);
        let has_scope_domain = domains.contains(&shared::ApplyDomain::Scope);
        let has_signal_domain = domains.contains(&shared::ApplyDomain::Signal);

        // TODO: Optimize action composition into a valid AttributesTransform that
        // still reflects the user's intended semantics. Consider:
        // - detecting and collapsing simple rename chains (e.g., a->b, b->c => a->c)
        // - detecting cycles or duplicate destinations and falling back to serial
        //   application of transforms when a composed map would be invalid.
        // For now, we compose a single transform and let transform_attributes
        // enforce validity (which may error for conflicting maps).
        let transform = AttributesTransform {
            rename: if renames.is_empty() {
                None
            } else {
                Some(RenameTransform::new(renames))
            },
            delete: if deletes.is_empty() {
                None
            } else {
                Some(DeleteTransform::new(deletes))
            },
        };

        transform
            .validate()
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                error: format!("Invalid attribute transform configuration: {e}"),
            })?;

        Ok(Self {
            transform,
            has_resource_domain,
            has_scope_domain,
            has_signal_domain,
            metrics: None,
        })
    }
}

impl AttributesProcessorTrait for AttributesProcessor {
    fn transform(&self) -> &AttributesTransform {
        &self.transform
    }

    fn domain_flags(&self) -> (bool, bool, bool) {
        (
            self.has_resource_domain,
            self.has_scope_domain,
            self.has_signal_domain,
        )
    }

    fn metrics_mut(&mut self) -> &mut Option<MetricSet<AttributesProcessorMetrics>> {
        &mut self.metrics
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for AttributesProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        process_attributes(self, msg, effect_handler).await
    }
}

/// Factory function to create an AttributesProcessor.
///
/// Accepts configuration in OpenTelemetry Collector attributes processor format.
/// See the module documentation for configuration examples and supported operations.
pub fn create_attributes_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let mut proc = AttributesProcessor::from_config(&node_config.config)?;
    proc.metrics = Some(pipeline_ctx.register_metrics::<AttributesProcessorMetrics>());
    Ok(ProcessorWrapper::local(
        proc,
        node,
        node_config,
        processor_config,
    ))
}

/// Register AttributesProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static ATTRIBUTES_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: ATTRIBUTES_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_attributes_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
    };

#[cfg(test)]
mod tests {
    use super::shared::test::{build_logs_with_attrs, run_test_processor};
    use super::*;
    use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use serde_json::json;

    fn run_processor(
        cfg: Value,
        input: otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest,
    ) -> otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest {
        run_test_processor(
            ATTRIBUTES_PROCESSOR_URN,
            cfg,
            input,
            create_attributes_processor,
        )
    }

    #[test]
    fn test_config_from_json_parses_actions_and_apply_to_default() {
        let cfg = json!({
            "actions": [
                {"action": "rename", "source_key": "a", "destination_key": "b"},
                {"action": "delete", "key": "x"}
            ]
        });
        let parsed = AttributesProcessor::from_config(&cfg).expect("config parse");
        assert!(parsed.transform.rename.is_some());
        assert!(parsed.transform.delete.is_some());
        // default apply_to should include Signal
        assert!(parsed.has_signal_domain);
        // and not necessarily Resource/Scope unless specified
        assert!(!parsed.has_resource_domain);
        assert!(!parsed.has_scope_domain);
    }

    #[test]
    fn test_rename_applies_to_signal_only_by_default() {
        // Prepare input with same key present in resource, scope, and log attrs
        let input = build_logs_with_attrs(
            vec![KeyValue::new("a", AnyValue::new_string("rv"))],
            vec![KeyValue::new("a", AnyValue::new_string("sv"))],
            vec![
                KeyValue::new("a", AnyValue::new_string("lv")),
                KeyValue::new("b", AnyValue::new_string("keep")),
            ],
        );
        let cfg = json!({
            "actions": [
                {"action": "rename", "source_key": "a", "destination_key": "b"}
            ]
        });
        let result = run_processor(cfg, input);
        let res_attrs = &result.resource_logs[0]
            .resource
            .as_ref()
            .unwrap()
            .attributes;
        assert!(res_attrs.iter().any(|kv| kv.key == "a"));
        assert!(!res_attrs.iter().any(|kv| kv.key == "b"));
        let scope_attrs = &result.resource_logs[0].scope_logs[0]
            .scope
            .as_ref()
            .unwrap()
            .attributes;
        assert!(scope_attrs.iter().any(|kv| kv.key == "a"));
        assert!(!scope_attrs.iter().any(|kv| kv.key == "b"));
        let log_attrs = &result.resource_logs[0].scope_logs[0].log_records[0].attributes;
        assert!(log_attrs.iter().any(|kv| kv.key == "b"));
        assert!(!log_attrs.iter().any(|kv| kv.key == "a"));
    }

    #[test]
    fn test_delete_applies_to_signal_only_by_default() {
        let input = build_logs_with_attrs(
            vec![KeyValue::new("a", AnyValue::new_string("rv"))],
            vec![KeyValue::new("a", AnyValue::new_string("sv"))],
            vec![
                KeyValue::new("a", AnyValue::new_string("lv")),
                KeyValue::new("b", AnyValue::new_string("keep")),
            ],
        );
        let cfg = json!({
            "actions": [
                {"action": "delete", "key": "a"}
            ]
        });
        let result = run_processor(cfg, input);
        let res_attrs = &result.resource_logs[0]
            .resource
            .as_ref()
            .unwrap()
            .attributes;
        assert!(res_attrs.iter().any(|kv| kv.key == "a"));
        let scope_attrs = &result.resource_logs[0].scope_logs[0]
            .scope
            .as_ref()
            .unwrap()
            .attributes;
        assert!(scope_attrs.iter().any(|kv| kv.key == "a"));
        let log_attrs = &result.resource_logs[0].scope_logs[0].log_records[0].attributes;
        assert!(!log_attrs.iter().any(|kv| kv.key == "a"));
        assert!(log_attrs.iter().any(|kv| kv.key == "b"));
    }

    #[test]
    fn test_delete_scoped_to_resource_only_logs() {
        let input = build_logs_with_attrs(
            vec![
                KeyValue::new("a", AnyValue::new_string("rv")),
                KeyValue::new("r", AnyValue::new_string("keep")),
            ],
            vec![KeyValue::new("a", AnyValue::new_string("sv"))],
            vec![
                KeyValue::new("a", AnyValue::new_string("lv")),
                KeyValue::new("b", AnyValue::new_string("keep")),
            ],
        );
        let cfg = json!({
            "actions": [ {"action": "delete", "key": "a"} ],
            "apply_to": ["resource"]
        });
        let result = run_processor(cfg, input);
        let res_attrs = &result.resource_logs[0]
            .resource
            .as_ref()
            .unwrap()
            .attributes;
        assert!(!res_attrs.iter().any(|kv| kv.key == "a"));
        let scope_attrs = &result.resource_logs[0].scope_logs[0]
            .scope
            .as_ref()
            .unwrap()
            .attributes;
        assert!(scope_attrs.iter().any(|kv| kv.key == "a"));
        let log_attrs = &result.resource_logs[0].scope_logs[0].log_records[0].attributes;
        assert!(log_attrs.iter().any(|kv| kv.key == "a"));
    }

    #[test]
    fn test_delete_scoped_to_scope_only_logs() {
        let input = build_logs_with_attrs(
            vec![KeyValue::new("a", AnyValue::new_string("rv"))],
            vec![
                KeyValue::new("a", AnyValue::new_string("sv")),
                KeyValue::new("s", AnyValue::new_string("keep")),
            ],
            vec![
                KeyValue::new("a", AnyValue::new_string("lv")),
                KeyValue::new("b", AnyValue::new_string("keep")),
            ],
        );
        let cfg = json!({
            "actions": [ {"action": "delete", "key": "a"} ],
            "apply_to": ["scope"]
        });
        let result = run_processor(cfg, input);
        let res_attrs = &result.resource_logs[0]
            .resource
            .as_ref()
            .unwrap()
            .attributes;
        assert!(res_attrs.iter().any(|kv| kv.key == "a"));
        let scope_attrs = &result.resource_logs[0].scope_logs[0]
            .scope
            .as_ref()
            .unwrap()
            .attributes;
        assert!(!scope_attrs.iter().any(|kv| kv.key == "a"));
        let log_attrs = &result.resource_logs[0].scope_logs[0].log_records[0].attributes;
        assert!(log_attrs.iter().any(|kv| kv.key == "a"));
    }

    #[test]
    fn test_delete_scoped_to_signal_and_resource() {
        let input = build_logs_with_attrs(
            vec![
                KeyValue::new("a", AnyValue::new_string("rv")),
                KeyValue::new("r", AnyValue::new_string("keep")),
            ],
            vec![
                KeyValue::new("a", AnyValue::new_string("sv")),
                KeyValue::new("s", AnyValue::new_string("keep")),
            ],
            vec![
                KeyValue::new("a", AnyValue::new_string("lv")),
                KeyValue::new("b", AnyValue::new_string("keep")),
            ],
        );
        let cfg = json!({
            "actions": [ {"action": "delete", "key": "a"} ],
            "apply_to": ["signal", "resource"]
        });
        let result = run_processor(cfg, input);
        let res_attrs = &result.resource_logs[0]
            .resource
            .as_ref()
            .unwrap()
            .attributes;
        assert!(!res_attrs.iter().any(|kv| kv.key == "a"));
        assert!(res_attrs.iter().any(|kv| kv.key == "r"));
        let scope_attrs = &result.resource_logs[0].scope_logs[0]
            .scope
            .as_ref()
            .unwrap()
            .attributes;
        assert!(scope_attrs.iter().any(|kv| kv.key == "a"));
        assert!(scope_attrs.iter().any(|kv| kv.key == "s"));
        let log_attrs = &result.resource_logs[0].scope_logs[0].log_records[0].attributes;
        assert!(!log_attrs.iter().any(|kv| kv.key == "a"));
        assert!(log_attrs.iter().any(|kv| kv.key == "b"));
    }
}

#[cfg(test)]
mod telemetry_tests {
    use super::*;
    use crate::pdata::{OtapPdata, OtlpProtoBytes};
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NodeControlMsg;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::{node::test_node, processor::TestRuntime};
    use otap_df_telemetry::MetricsSystem;
    use prost::Message as _;
    use serde_json::json;

    #[test]
    fn test_metrics_collect_telemetry_reports_counters() {
        use std::sync::{Arc, Mutex};

        // 1) Telemetry system (no async yet); we will start the collector inside the test runtime
        let metrics_system = MetricsSystem::default();
        let registry = metrics_system.registry();
        let reporter = metrics_system.reporter();

        // Shared place to store the collector JoinHandle started inside the runtime
        let collector_handle: Arc<
            Mutex<Option<tokio::task::JoinHandle<Result<(), otap_df_telemetry::error::Error>>>>,
        > = Arc::new(Mutex::new(None));

        // 2) Pipeline context sharing the same registry handle
        let controller = ControllerContext::new(registry.clone());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);

        // 3) Build processor with simple rename+delete (applies to signal domain by default)
        let cfg = json!({
            "actions": [
                {"action": "rename", "source_key": "a", "destination_key": "b"},
                {"action": "delete", "key": "x"}
            ]
        });
        let mut node_cfg = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_cfg.config = cfg;
        let proc_cfg = ProcessorConfig::new("attr_proc");
        let node = test_node(proc_cfg.name.clone());
        let proc = create_attributes_processor(pipeline_ctx, node, Arc::new(node_cfg), &proc_cfg)
            .expect("create processor");

        // 4) Build a minimal OTLP logs request that has a signal-level attribute 'a'
        let input_bytes = {
            use otel_arrow_rust::proto::opentelemetry::{
                collector::logs::v1::ExportLogsServiceRequest,
                common::v1::{
                    AnyValue, InstrumentationScope, KeyValue, any_value::Value as AnyVal,
                },
                logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
                resource::v1::Resource,
            };

            let req = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource {
                        attributes: vec![],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope {
                            attributes: vec![],
                            ..Default::default()
                        }),
                        log_records: vec![LogRecord {
                            time_unix_nano: 1,
                            attributes: vec![
                                KeyValue {
                                    key: "a".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(AnyVal::StringValue("v".into())),
                                    }),
                                },
                                KeyValue {
                                    key: "x".to_string(), // ensure at least one deletion occurs
                                    value: Some(AnyValue {
                                        value: Some(AnyVal::StringValue("to_delete".into())),
                                    }),
                                },
                            ],
                            severity_number: SeverityNumber::Info as i32,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let mut bytes = Vec::new();
            req.encode(&mut bytes).expect("encode");
            bytes
        };

        // 5) Drive processor with TestRuntime; start collector inside and then request a telemetry snapshot
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let phase = rt.set_processor(proc);

        let reporter_run = reporter.clone();
        let collector_handle_rt = collector_handle.clone();
        let metrics_system_rt = metrics_system; // move into runtime
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let cancel_for_rt = cancel_token.clone();

        phase
            .run_test(|mut ctx| async move {
                // Start collector inside the runtime with a cancellation token to ensure shutdown
                let handle = tokio::spawn(metrics_system_rt.run(cancel_for_rt));
                *collector_handle_rt.lock().unwrap() = Some(handle);

                // Process one message
                let pdata =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(input_bytes).into());
                ctx.process(Message::PData(pdata)).await.expect("pdata");

                // Trigger telemetry snapshot
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter_run.clone(),
                }))
                .await
                .expect("collect");
            })
            .validate(move |_| async move {
                // Allow the collector to pull from the channel
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                // Inspect current metrics; fields with non-zero values should be present
                let mut found_consumed = false;
                let mut found_forwarded = false;
                let mut found_renamed_entries = false;
                let mut found_deleted_entries = false;
                let mut found_domain_signal = false;

                registry.visit_current_metrics(|desc, _attrs, iter| {
                    if desc.name == "attributes.processor.metrics" {
                        for (field, v) in iter {
                            match (field.name, v) {
                                ("msgs.consumed", x) if x >= 1 => found_consumed = true,
                                ("msgs.forwarded", x) if x >= 1 => found_forwarded = true,
                                ("renamed.entries", x) if x >= 1 => found_renamed_entries = true,
                                ("deleted.entries", x) if x >= 1 => found_deleted_entries = true,
                                ("domains.signal", x) if x >= 1 => found_domain_signal = true,
                                _ => {}
                            }
                        }
                    }
                });

                assert!(found_consumed, "msgs.consumed should be >= 1");
                assert!(found_forwarded, "msgs.forwarded should be >= 1");
                assert!(found_renamed_entries, "renamed.entries should be >= 1");
                assert!(found_deleted_entries, "deleted.entries should be >= 1");
                assert!(found_domain_signal, "domains.signal should be >= 1");

                // Tear down collector by cancelling and awaiting the handle
                cancel_token.cancel();
                let handle_opt = {
                    let mut guard = collector_handle.lock().unwrap();
                    guard.take()
                };
                if let Some(handle) = handle_opt {
                    handle.await.unwrap().unwrap();
                }
            });
    }
}
