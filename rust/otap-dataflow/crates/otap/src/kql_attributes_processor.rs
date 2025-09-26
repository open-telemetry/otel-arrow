// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! KQL-based attributes processor for OTAP pipelines.
//!
//! This processor provides attribute transformations for telemetry data using
//! KQL (Kusto Query Language) configuration. It operates on OTAP Arrow payloads
//! (OtapArrowRecords and OtapArrowBytes) and can convert OTLP bytes to OTAP for
//! processing.
//!
//! The processor parses KQL expressions and converts them to the same
//! transformation operations currently supported by the attributes_processor:
//! - `project-rename`: Renames an attribute key
//! - `project-away`: Removes an attribute by key
//!
//! Example KQL configurations: //! You can optionally scope the transformation
//! using `apply_to`. Valid values: signal, resource, scope. If omitted,
//! defaults to [signal]. Rename an attribute:
//! ```yaml
//! kql: source | project-rename rpc.method = http.method
//! # apply_to: ["signal", "resource"]  # Optional; defaults to ["signal"]
//! ```
//!
//! Delete an attribute:
//! ```yaml
//! kql: source | project-away http.method
//! # apply_to: ["signal", "resource"]  # Optional; defaults to ["signal"]
//! ```
//!
//! Multiple actions can be chained in a single KQL expression:
//! ```yaml
//! kql: source | project-rename rpc.method = http.method | project-away http.status_code
//! # apply_to: ["signal", "resource"]  # Optional; defaults to ["signal"]
//! ```
//!
//! Implementation uses otel_arrow_rust::otap::transform::transform_attributes
//! for efficient batch processing of Arrow record batches.

use crate::attributes_processor::shared::{AttributesProcessorTrait, process_attributes};
use crate::attributes_processor::{metrics::AttributesProcessorMetrics, shared};
use crate::{OTAP_PROCESSOR_FACTORIES, pdata::OtapPdata};
use async_trait::async_trait;
use data_engine_expressions::*;
use data_engine_kql_parser::{KqlParser, Parser, ParserMapSchema, ParserOptions};
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

/// URN for the KqlAttributesProcessor
pub const KQL_ATTRIBUTES_PROCESSOR_URN: &str = "urn:otap:processor:kql_attributes_processor";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Configuration for the KqlAttributesProcessor.
///
/// Accepts configuration in the same format as the OpenTelemetry Collector's
/// attributes processor. Supported actions: rename (deviation), delete. Others
/// are ignored.
///
/// You can control which attribute domains are transformed via `apply_to`.
/// Valid values: "signal" (default), "resource", "scope".
pub struct Config {
    /// KQL query string for attribute transformations
    #[serde(default)]
    pub kql: String,

    /// Attribute domains to apply transforms to. Defaults to ["signal"].
    #[serde(default)]
    pub apply_to: Option<Vec<String>>,
}

/// Processor that applies attribute transformations to OTAP attribute batches.
///
/// Implements the OpenTelemetry Collector's attributes processor functionality
/// using efficient Arrow operations. Supports `update` (rename) and `delete`
/// operations across all attribute types (resource, scope, and signal-specific
/// attributes) for logs, metrics, and traces telemetry.
pub struct KqlAttributesProcessor {
    // Pre-computed transform to avoid rebuilding per message
    transform: AttributesTransform,
    // Pre-computed flags for domain lookup
    has_resource_domain: bool,
    has_scope_domain: bool,
    has_signal_domain: bool,
    // Metrics handle (set at runtime in factory; None when parsed-only)
    metrics: Option<MetricSet<AttributesProcessorMetrics>>,
}

impl KqlAttributesProcessor {
    /// Creates a new KqlAttributesProcessor from configuration.
    ///
    /// Parses the KQL query and converts it into the operations supported by
    /// the underlying Arrow attribute transform API.
    #[must_use = "KqlAttributesProcessor creation may fail and return a ConfigError"]
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let cfg: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse KqlAttributesProcessor configuration: {e}"),
            })?;
        Self::new(cfg)
    }

    /// Creates a new KqlKqlAttributesProcessor with the given parsed
    /// configuration.
    fn new(config: Config) -> Result<Self, ConfigError> {
        let mut renames = BTreeMap::new();
        let mut deletes = BTreeSet::new();

        // In the KQL grammar, dot syntax normally indicates nested fields in a
        // dynamic, so something like 'rpc.method' would be interpreted as a
        // field 'method' inside a nested field 'rpc'. To circumvent this and
        // treat the entire string as a single key, we can provide a specific
        // schema with a default map configured. This results in the parser
        // assuming all field accesses are within a map named "Attributes".
        let options = ParserOptions::new()
            .with_source_map_schema(ParserMapSchema::new().set_default_map_key("Attributes"));
        let pipeline = KqlParser::parse_with_options(&config.kql, options).map_err(|errors| {
            ConfigError::InvalidUserConfig {
                error: format!("Failed to parse KQL query: {:?}", errors),
            }
        })?;

        // Extract supported expressions from the pipeline
        for expression in pipeline.get_expressions() {
            if let DataExpression::Transform(transform) = expression {
                match transform {
                    // Handle project-away
                    TransformExpression::RemoveMapKeys(remove_keys_expr) => {
                        match remove_keys_expr {
                            RemoveMapKeysTransformExpression::Remove(map_key_list) => {
                                for key_expr in map_key_list.get_keys() {
                                    if let Some(key) = Self::extract_from_scalar(key_expr) {
                                        let _ = deletes.insert(key);
                                    } else {
                                        return Err(ConfigError::InvalidUserConfig {
                                            error: format!(
                                                "Unsupported key expression in remove map keys: {:?}",
                                                key_expr
                                            ),
                                        });
                                    }
                                }
                            }
                            RemoveMapKeysTransformExpression::Retain(_) => {
                                // Not directly mapped yet to attributes_processor capability
                                // This is used by KQL 'project' which is actually a discard of everything not mentioned
                            }
                        }
                    }
                    // Handle project-rename
                    TransformExpression::Move(move_expr) => {
                        match (
                            Self::extract_from_mutable_value(move_expr.get_source()),
                            Self::extract_from_mutable_value(move_expr.get_destination()),
                        ) {
                            (Some(src_key), Some(dst_key)) => {
                                let _ = renames.insert(src_key, dst_key);
                            }
                            _ => {
                                return Err(ConfigError::InvalidUserConfig {
                                    error: format!(
                                        "Unsupported source or destination in move expression: {:?} -> {:?}",
                                        move_expr.get_source(),
                                        move_expr.get_destination()
                                    ),
                                });
                            }
                        }
                    }
                    TransformExpression::RenameMapKeys(rename_keys_expr) => {
                        for key_selector in rename_keys_expr.get_keys() {
                            match (
                                Self::extract_from_value_accessor(key_selector.get_source()),
                                Self::extract_from_value_accessor(key_selector.get_destination()),
                            ) {
                                (Some(src_key), Some(dst_key)) => {
                                    let _ = renames.insert(src_key, dst_key);
                                }
                                _ => {
                                    return Err(ConfigError::InvalidUserConfig {
                                        error: format!(
                                            "Unsupported source or destination in rename map keys expression: {:?} -> {:?}",
                                            key_selector.get_source(),
                                            key_selector.get_destination()
                                        ),
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        // Not directly mapped yet to attributes_processor capability
                    }
                }
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

    /// Extract attribute name from a ValueAccessor, assuming "Attributes" map prefix.
    fn extract_from_value_accessor(accessor: &ValueAccessor) -> Option<String> {
        let selectors = accessor.get_selectors();
        // Expect map access with prefix: ["Attributes", "field_name"]
        if selectors.len() == 2 {
            if let (
                ScalarExpression::Static(StaticScalarExpression::String(map_name)),
                ScalarExpression::Static(StaticScalarExpression::String(field_name)),
            ) = (&selectors[0], &selectors[1])
            {
                if map_name.get_value() == "Attributes" {
                    Some(field_name.get_value().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Extract attribute name from a MutableValueExpression.
    fn extract_from_mutable_value(mutable_value: &MutableValueExpression) -> Option<String> {
        match mutable_value {
            MutableValueExpression::Source(source_expr) => {
                Self::extract_from_value_accessor(source_expr.get_value_accessor())
            }
            MutableValueExpression::Variable(_) => {
                // Variables are not supported for attribute extraction
                None
            }
        }
    }

    /// Extract a column name from a scalar expression.
    fn extract_from_scalar(scalar: &ScalarExpression) -> Option<String> {
        match scalar {
            ScalarExpression::Source(source_expr) => {
                Self::extract_from_value_accessor(source_expr.get_value_accessor())
            }
            ScalarExpression::Static(StaticScalarExpression::String(string_expr)) => {
                Some(string_expr.get_value().to_string())
            }
            _ => None,
        }
    }
}

impl AttributesProcessorTrait for KqlAttributesProcessor {
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
impl local::Processor<OtapPdata> for KqlAttributesProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        process_attributes(self, msg, effect_handler).await
    }
}

/// Factory function to create an KqlAttributesProcessor.
///
/// Accepts configuration in OpenTelemetry Collector attributes processor
/// format. See the module documentation for configuration examples and
/// supported operations.
pub fn create_kql_attributes_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let mut proc = KqlAttributesProcessor::from_config(&node_config.config)?;
    proc.metrics = Some(pipeline_ctx.register_metrics::<AttributesProcessorMetrics>());
    Ok(ProcessorWrapper::local(
        proc,
        node,
        node_config,
        processor_config,
    ))
}

/// Register KqlAttributesProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static KQL_ATTRIBUTES_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: KQL_ATTRIBUTES_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_kql_attributes_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
    };

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes_processor::shared::test::{build_logs_with_attrs, run_test_processor};
    use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use serde_json::json;

    fn run_processor(
        cfg: Value,
        input: otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest,
    ) -> otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest {
        run_test_processor(
            KQL_ATTRIBUTES_PROCESSOR_URN,
            cfg,
            input,
            create_kql_attributes_processor,
        )
    }

    #[test]
    fn test_config_from_json_parses_actions_and_apply_to_default() {
        let cfg = json!({
            "kql": "source | project-rename b.foo = a.bar, d = c | project-away x.fizz, y.buzz, z"
        });
        let parsed = KqlAttributesProcessor::from_config(&cfg).expect("config parse");
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
        let input = build_logs_with_attrs(
            vec![KeyValue::new("a", AnyValue::new_string("rv"))],
            vec![KeyValue::new("a", AnyValue::new_string("sv"))],
            vec![
                KeyValue::new("a", AnyValue::new_string("lv")),
                KeyValue::new("b", AnyValue::new_string("keep")),
            ],
        );
        let cfg = json!({ "kql": "source | project-rename b = a | project-away x" });
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
        let cfg = json!({ "kql": "source | project-away a" });
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
        assert!(log_attrs.iter().any(|kv| {
            if kv.key != "b" { return false; }
            match kv.value.as_ref().and_then(|v| v.value.as_ref()) {
                Some(otel_arrow_rust::proto::opentelemetry::common::v1::any_value::Value::StringValue(s)) => s == "keep",
                _ => false,
            }
        }));
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
            "kql": "source | project-away a",
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
            "kql": "source | project-away a",
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
            "kql": "source | project-away a",
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

        // 1) Telemetry system (no async yet); we will start the collector
        //    inside the test runtime
        let metrics_system = MetricsSystem::default();
        let registry = metrics_system.registry();
        let reporter = metrics_system.reporter();

        // Shared place to store the collector JoinHandle started inside the
        // runtime
        let collector_handle: Arc<
            Mutex<Option<tokio::task::JoinHandle<Result<(), otap_df_telemetry::error::Error>>>>,
        > = Arc::new(Mutex::new(None));

        // 2) Pipeline context sharing the same registry handle
        let controller = ControllerContext::new(registry.clone());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);

        // 3) Build processor with simple rename+delete (applies to signal
        //    domain by default)
        let cfg = json!({
            "kql": "source | project-rename b = a | project-away x"
        });
        let mut node_cfg = NodeUserConfig::new_processor_config(KQL_ATTRIBUTES_PROCESSOR_URN);
        node_cfg.config = cfg;
        let proc_cfg = ProcessorConfig::new("attr_proc");
        let node = test_node(proc_cfg.name.clone());
        let proc =
            create_kql_attributes_processor(pipeline_ctx, node, Arc::new(node_cfg), &proc_cfg)
                .expect("create processor");

        // 4) Build a minimal OTLP logs request that has a signal-level
        //    attribute 'a'
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

        // 5) Drive processor with TestRuntime; start collector inside and then
        //    request a telemetry snapshot
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let phase = rt.set_processor(proc);

        let reporter_run = reporter.clone();
        let collector_handle_rt = collector_handle.clone();
        let metrics_system_rt = metrics_system; // move into runtime
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let cancel_for_rt = cancel_token.clone();

        phase
            .run_test(|mut ctx| async move {
                // Start collector inside the runtime with a cancellation token
                // to ensure shutdown
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

                // Inspect current metrics; fields with non-zero values should
                // be present
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
