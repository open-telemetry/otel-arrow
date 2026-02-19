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
//! - `insert`: Inserts a new attribute.
//!
//! Unsupported actions are ignored if present in the config:
//! `upsert`, `update` (value update), `hash`, `extract`, `convert`.
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
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::otap::{
    OtapArrowRecords,
    transform::{
        AttributesTransform, DeleteTransform, InsertTransform, LiteralValue, RenameTransform,
        transform_attributes_with_stats,
    },
};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_telemetry::metrics::MetricSet;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

mod metrics;
use self::metrics::AttributesProcessorMetrics;

/// URN for the AttributesProcessor
pub const ATTRIBUTES_PROCESSOR_URN: &str = "urn:otel:attribute:processor";

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

    /// Insert a new attribute.
    Insert {
        /// The attribute key to insert.
        key: String,
        /// The value to insert.
        value: LiteralValue,
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
/// Supported actions: rename (deviation), delete, insert. Others are ignored.
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
        let mut inserts = BTreeMap::new();

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
                Action::Insert { key, value } => {
                    let _ = inserts.insert(key, value);
                }
                // Unsupported actions are ignored for now
                Action::Unsupported => {}
            }
        }

        let domains = parse_apply_to(config.apply_to.as_ref());

        // Pre-compute domain checks
        let has_resource_domain = domains.contains(&ApplyDomain::Resource);
        let has_scope_domain = domains.contains(&ApplyDomain::Scope);
        let has_signal_domain = domains.contains(&ApplyDomain::Signal);

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
            insert: if inserts.is_empty() {
                None
            } else {
                Some(InsertTransform::new(inserts))
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

    #[inline]
    const fn is_noop(&self) -> bool {
        self.transform.rename.is_none()
            && self.transform.delete.is_none()
            && self.transform.insert.is_none()
    }

    #[inline]
    const fn attrs_payloads(&self, signal: SignalType) -> &'static [ArrowPayloadType] {
        use payload_sets::*;

        match (
            self.has_resource_domain,
            self.has_scope_domain,
            self.has_signal_domain,
            signal,
        ) {
            // Empty cases
            (false, false, false, _) => EMPTY,

            // Signal only
            (false, false, true, SignalType::Logs) => LOGS_SIGNAL,
            (false, false, true, SignalType::Metrics) => METRICS_SIGNAL,
            (false, false, true, SignalType::Traces) => TRACES_SIGNAL,

            // Resource only
            (true, false, false, _) => RESOURCE_ONLY,

            // Scope only
            (false, true, false, _) => SCOPE_ONLY,

            // Resource + Signal
            (true, false, true, SignalType::Logs) => LOGS_RESOURCE_SIGNAL,
            (true, false, true, SignalType::Metrics) => METRICS_RESOURCE_SIGNAL,
            (true, false, true, SignalType::Traces) => TRACES_RESOURCE_SIGNAL,

            // Scope + Signal
            (false, true, true, SignalType::Logs) => LOGS_SCOPE_SIGNAL,
            (false, true, true, SignalType::Metrics) => METRICS_SCOPE_SIGNAL,
            (false, true, true, SignalType::Traces) => TRACES_SCOPE_SIGNAL,

            // Resource + Scope (no signal)
            (true, true, false, _) => RESOURCE_SCOPE,

            // All three
            (true, true, true, SignalType::Logs) => LOGS_ALL,
            (true, true, true, SignalType::Metrics) => METRICS_ALL,
            (true, true, true, SignalType::Traces) => TRACES_ALL,
        }
    }

    #[allow(clippy::result_large_err)]
    fn apply_transform_with_stats(
        &self,
        records: &mut OtapArrowRecords,
        signal: SignalType,
    ) -> Result<(u64, u64), EngineError> {
        let mut deleted_total: u64 = 0;
        let mut renamed_total: u64 = 0;

        // Only apply if we have transforms to apply
        if !self.is_noop() {
            let payloads = self.attrs_payloads(signal);
            for &payload_ty in payloads {
                if let Some(rb) = records.get(payload_ty) {
                    let (rb, stats) = transform_attributes_with_stats(rb, &self.transform)
                        .map_err(|e| engine_err(&format!("transform_attributes failed: {e}")))?;
                    deleted_total += stats.deleted_entries;
                    renamed_total += stats.renamed_entries;
                    if rb.num_rows() == 0 {
                        records.remove(payload_ty);
                    } else {
                        records.set(payload_ty, rb);
                    }
                }
            }
        }

        Ok((deleted_total, renamed_total))
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for AttributesProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            Message::Control(control_msg) => match control_msg {
                otap_df_engine::control::NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    if let Some(metrics) = self.metrics.as_mut() {
                        let _ = metrics_reporter.report(metrics);
                    }
                    Ok(())
                }
                _ => Ok(()),
            },
            Message::PData(pdata) => {
                // Fast path: no actions to apply
                if self.is_noop() {
                    let res = effect_handler
                        .send_message_with_source_node(pdata)
                        .await
                        .map_err(|e| e.into());
                    return res;
                }

                let signal = pdata.signal_type();
                let (context, payload) = pdata.into_parts();

                let mut records: OtapArrowRecords = payload.try_into()?;

                // Update domain counters (count once per message when domains are enabled)
                if let Some(m) = self.metrics.as_mut() {
                    if self.has_resource_domain {
                        m.domains_resource.inc();
                    }
                    if self.has_scope_domain {
                        m.domains_scope.inc();
                    }
                    if self.has_signal_domain {
                        m.domains_signal.inc();
                    }
                }
                // Apply transform across selected domains and collect exact stats
                match self.apply_transform_with_stats(&mut records, signal) {
                    Ok((deleted_total, renamed_total)) => {
                        if let Some(m) = self.metrics.as_mut() {
                            if deleted_total > 0 {
                                m.deleted_entries.add(deleted_total);
                            }
                            if renamed_total > 0 {
                                m.renamed_entries.add(renamed_total);
                            }
                        }
                    }
                    Err(e) => {
                        if let Some(m) = self.metrics.as_mut() {
                            m.transform_failed.inc();
                        }
                        return Err(e);
                    }
                }

                effect_handler
                    .send_message_with_source_node(OtapPdata::new(context, records.into()))
                    .await
                    .map_err(|e| e.into())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ApplyDomain {
    Signal,
    Resource,
    Scope,
}

fn parse_apply_to(apply_to: Option<&Vec<String>>) -> HashSet<ApplyDomain> {
    let mut set = HashSet::new();
    match apply_to {
        None => {
            let _ = set.insert(ApplyDomain::Signal);
        }
        Some(list) => {
            for item in list {
                match item.as_str() {
                    "signal" => {
                        let _ = set.insert(ApplyDomain::Signal);
                    }
                    "resource" => {
                        let _ = set.insert(ApplyDomain::Resource);
                    }
                    "scope" => {
                        let _ = set.insert(ApplyDomain::Scope);
                    }
                    _ => {
                        // Unknown entry: ignore for now; could return config error in future
                    }
                }
            }
            if set.is_empty() {
                let _ = set.insert(ApplyDomain::Signal);
            }
        }
    }
    set
}

fn engine_err(msg: &str) -> EngineError {
    EngineError::PdataConversionError {
        error: msg.to_string(),
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
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<Config>,
    };

// Pre-computed arrays for all domain combinations
mod payload_sets {
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType as A;

    pub(super) const EMPTY: &[A] = &[];

    // Signal only
    pub(super) const LOGS_SIGNAL: &[A] = &[A::LogAttrs];
    pub(super) const METRICS_SIGNAL: &[A] = &[
        A::MetricAttrs,
        A::NumberDpAttrs,
        A::HistogramDpAttrs,
        A::SummaryDpAttrs,
        A::NumberDpExemplarAttrs,
        A::HistogramDpExemplarAttrs,
    ];
    pub(super) const TRACES_SIGNAL: &[A] = &[A::SpanAttrs, A::SpanEventAttrs, A::SpanLinkAttrs];

    // Resource only
    pub(super) const RESOURCE_ONLY: &[A] = &[A::ResourceAttrs];

    // Scope only
    pub(super) const SCOPE_ONLY: &[A] = &[A::ScopeAttrs];

    // Resource + Signal
    pub(super) const LOGS_RESOURCE_SIGNAL: &[A] = &[A::ResourceAttrs, A::LogAttrs];
    pub(super) const METRICS_RESOURCE_SIGNAL: &[A] = &[
        A::ResourceAttrs,
        A::MetricAttrs,
        A::NumberDpAttrs,
        A::HistogramDpAttrs,
        A::SummaryDpAttrs,
        A::NumberDpExemplarAttrs,
        A::HistogramDpExemplarAttrs,
    ];
    pub(super) const TRACES_RESOURCE_SIGNAL: &[A] = &[
        A::ResourceAttrs,
        A::SpanAttrs,
        A::SpanEventAttrs,
        A::SpanLinkAttrs,
    ];

    // Scope + Signal
    pub(super) const LOGS_SCOPE_SIGNAL: &[A] = &[A::ScopeAttrs, A::LogAttrs];
    pub(super) const METRICS_SCOPE_SIGNAL: &[A] = &[
        A::ScopeAttrs,
        A::MetricAttrs,
        A::NumberDpAttrs,
        A::HistogramDpAttrs,
        A::SummaryDpAttrs,
        A::NumberDpExemplarAttrs,
        A::HistogramDpExemplarAttrs,
    ];
    pub(super) const TRACES_SCOPE_SIGNAL: &[A] = &[
        A::ScopeAttrs,
        A::SpanAttrs,
        A::SpanEventAttrs,
        A::SpanLinkAttrs,
    ];

    // Resource + Scope
    pub(super) const RESOURCE_SCOPE: &[A] = &[A::ResourceAttrs, A::ScopeAttrs];

    // All three: Resource + Scope + Signal
    pub(super) const LOGS_ALL: &[A] = &[A::ResourceAttrs, A::ScopeAttrs, A::LogAttrs];
    pub(super) const METRICS_ALL: &[A] = &[
        A::ResourceAttrs,
        A::ScopeAttrs,
        A::MetricAttrs,
        A::NumberDpAttrs,
        A::HistogramDpAttrs,
        A::SummaryDpAttrs,
        A::NumberDpExemplarAttrs,
        A::HistogramDpExemplarAttrs,
    ];
    pub(super) const TRACES_ALL: &[A] = &[
        A::ResourceAttrs,
        A::ScopeAttrs,
        A::SpanAttrs,
        A::SpanEventAttrs,
        A::SpanLinkAttrs,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdata::OtapPdata;
    use bytes::BytesMut;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::{node::test_node, processor::TestRuntime};
    use otap_df_pdata::proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message as _;
    use serde_json::json;

    fn build_logs_with_attrs(
        res_attrs: Vec<KeyValue>,
        scope_attrs: Vec<KeyValue>,
        log_attrs: Vec<KeyValue>,
    ) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource {
                attributes: res_attrs,
                ..Default::default()
            },
            vec![ScopeLogs::new(
                InstrumentationScope {
                    attributes: scope_attrs,
                    ..Default::default()
                },
                vec![
                    LogRecord::build()
                        .time_unix_nano(1u64)
                        .severity_number(SeverityNumber::Info)
                        .event_name("")
                        .attributes(log_attrs)
                        .finish(),
                ],
            )],
        )])
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

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        // Set up processor test runtime and run one message
        let node = test_node("attributes-processor-test");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                // capture output
                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                // Convert output to OTLP bytes for easy assertions
                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                // Resource should still have key "a"
                let res_attrs = &decoded.resource_logs[0]
                    .resource
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(res_attrs.iter().any(|kv| kv.key == "a"));
                assert!(!res_attrs.iter().any(|kv| kv.key == "b"));

                // Scope should still have key "a"
                let scope_attrs = &decoded.resource_logs[0].scope_logs[0]
                    .scope
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(scope_attrs.iter().any(|kv| kv.key == "a"));
                assert!(!scope_attrs.iter().any(|kv| kv.key == "b"));

                // Log attrs should have renamed to "b"
                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
                assert!(log_attrs.iter().any(|kv| kv.key == "b"));
                assert!(!log_attrs.iter().any(|kv| kv.key == "a"));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_delete_applies_to_signal_only_by_default() {
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
                {"action": "delete", "key": "a"}
            ]
        });

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-delete-test");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                // Resource should still have key "a"
                let res_attrs = &decoded.resource_logs[0]
                    .resource
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(res_attrs.iter().any(|kv| kv.key == "a"));

                // Scope should still have key "a"
                let scope_attrs = &decoded.resource_logs[0].scope_logs[0]
                    .scope
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(scope_attrs.iter().any(|kv| kv.key == "a"));
                // Log attrs should have deleted "a" but still contain other keys
                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
                assert!(!log_attrs.iter().any(|kv| kv.key == "a"));
                assert!(log_attrs.iter().any(|kv| {
                    if kv.key != "b" { return false; }
                    match kv.value.as_ref().and_then(|v| v.value.as_ref()) {
                        Some(otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s)) => s == "keep",
                        _ => false,
                    }
                }));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_insert_scoped_to_resource_only_logs() {
        // Resource has 'a', scope has 'a', log has 'a' and another key to keep batch non-empty
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
            "actions": [
                {"action": "insert", "key": "c", "value": "val"},
            ],
            "apply_to": ["resource"]
        });

        // Create a proper pipeline context for the test
        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-insert-resource");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                let res_attrs = &decoded.resource_logs[0]
                    .resource
                    .as_ref()
                    .unwrap()
                    .attributes;

                assert!(
                    res_attrs
                        .iter()
                        .any(|kv| kv.key == "c" && kv.value == Some(AnyValue::new_string("val")))
                );
                assert!(res_attrs.iter().any(|kv| kv.key == "r"));

                // Scope 'c' should not be inserted; 'a' should remain
                let scope_attrs = &decoded.resource_logs[0].scope_logs[0]
                    .scope
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(!scope_attrs.iter().any(|kv| kv.key == "c"));
                assert!(scope_attrs.iter().any(|kv| kv.key == "a"));

                // Log 'c' should not be inserted; 'b' should remain
                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
                assert!(!log_attrs.iter().any(|kv| kv.key == "c"));
                assert!(log_attrs.iter().any(|kv| kv.key == "b"));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_insert_int_value_via_config() {
        // Test inserting an integer value via JSON configuration
        let input = build_logs_with_attrs(
            vec![],
            vec![],
            vec![KeyValue::new("existing", AnyValue::new_string("val"))],
        );

        let cfg = json!({
            "actions": [
                {"action": "insert", "key": "count", "value": 42},
            ],
            "apply_to": ["signal"]
        });

        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-insert-int");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;

                // Verify the int value was inserted correctly
                assert!(log_attrs.iter().any(|kv| kv.key == "count"));
                let count_kv = log_attrs.iter().find(|kv| kv.key == "count").unwrap();
                let inner_val = count_kv.value.as_ref().unwrap().value.as_ref().unwrap();
                match inner_val {
                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::IntValue(
                        i,
                    ) => {
                        assert_eq!(*i, 42);
                    }
                    _ => panic!("expected IntValue, got {:?}", inner_val),
                }
                assert!(log_attrs.iter().any(|kv| kv.key == "existing"));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_insert_double_value_via_config() {
        // Test inserting a double value via JSON configuration
        let input = build_logs_with_attrs(
            vec![],
            vec![],
            vec![KeyValue::new("existing", AnyValue::new_string("val"))],
        );

        let cfg = json!({
            "actions": [
                {"action": "insert", "key": "ratio", "value": 1.2345},
            ],
            "apply_to": ["signal"]
        });

        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-insert-double");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;

                // Verify the double value was inserted correctly
                assert!(log_attrs.iter().any(|kv| kv.key == "ratio"));
                let ratio_kv = log_attrs.iter().find(|kv| kv.key == "ratio").unwrap();
                let inner_val = ratio_kv.value.as_ref().unwrap().value.as_ref().unwrap();
                match inner_val {
                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::DoubleValue(d) => {
                        assert!((d - 1.2345).abs() < f64::EPSILON);
                    }
                    _ => panic!("expected DoubleValue, got {:?}", inner_val),
                }
                assert!(log_attrs.iter().any(|kv| kv.key == "existing"));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_insert_bool_value_via_config() {
        // Test inserting a boolean value via JSON configuration
        let input = build_logs_with_attrs(
            vec![],
            vec![],
            vec![KeyValue::new("existing", AnyValue::new_string("val"))],
        );

        let cfg = json!({
            "actions": [
                {"action": "insert", "key": "enabled", "value": true},
            ],
            "apply_to": ["signal"]
        });

        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-insert-bool");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;

                // Verify the bool value was inserted correctly
                assert!(log_attrs.iter().any(|kv| kv.key == "enabled"));
                let enabled_kv = log_attrs.iter().find(|kv| kv.key == "enabled").unwrap();
                let inner_val = enabled_kv.value.as_ref().unwrap().value.as_ref().unwrap();
                match inner_val {
                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::BoolValue(b) => {
                        assert!(*b);
                    }
                    _ => panic!("expected BoolValue, got {:?}", inner_val),
                }
                assert!(log_attrs.iter().any(|kv| kv.key == "existing"));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_insert_does_not_overwrite_existing() {
        // Verify that insert action does NOT overwrite existing key (per OTel spec)
        let input = build_logs_with_attrs(
            vec![],
            vec![],
            vec![KeyValue::new(
                "target_key",
                AnyValue::new_string("original_value"),
            )],
        );

        let cfg = json!({
            "actions": [
                {"action": "insert", "key": "target_key", "value": "new_value"},
            ],
            "apply_to": ["signal"]
        });

        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-insert-no-overwrite");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;

                // Should have exactly one target_key with original value
                let matching_attrs: Vec<_> = log_attrs
                    .iter()
                    .filter(|kv| kv.key == "target_key")
                    .collect();
                assert_eq!(matching_attrs.len(), 1);

                // Value should be original, not overwritten
                let value = matching_attrs[0]
                    .value
                    .as_ref()
                    .expect("value")
                    .value
                    .as_ref()
                    .expect("inner value");
                match value {
                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                        s,
                    ) => {
                        assert_eq!(s, "original_value");
                    }
                    _ => panic!("expected string value"),
                }
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_insert_multiple_keys_mixed_existing() {
        // Test inserting multiple keys where some exist and some don't
        let input = build_logs_with_attrs(
            vec![],
            vec![],
            vec![
                KeyValue::new("existing_a", AnyValue::new_string("val_a")),
                KeyValue::new("existing_b", AnyValue::new_string("val_b")),
            ],
        );

        let cfg = json!({
            "actions": [
                {"action": "insert", "key": "existing_a", "value": "should_not_replace"},
                {"action": "insert", "key": "new_key", "value": "new_value"},
            ],
            "apply_to": ["signal"]
        });

        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-insert-mixed");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;

                // Should have 3 keys: existing_a (unchanged), existing_b, new_key
                assert_eq!(log_attrs.len(), 3);
                assert!(log_attrs.iter().any(|kv| kv.key == "existing_a"));
                assert!(log_attrs.iter().any(|kv| kv.key == "existing_b"));
                assert!(
                    log_attrs.iter().any(|kv| kv.key == "new_key"
                        && kv.value == Some(AnyValue::new_string("new_value")))
                );
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_delete_scoped_to_resource_only_logs() {
        // Resource has 'a', scope has 'a', log has 'a' and another key to keep batch non-empty
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

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-delete-resource");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();
                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                // Resource 'a' should be deleted
                let res_attrs = &decoded.resource_logs[0]
                    .resource
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(!res_attrs.iter().any(|kv| kv.key == "a"));
                // Scope 'a' should remain
                let scope_attrs = &decoded.resource_logs[0].scope_logs[0]
                    .scope
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(scope_attrs.iter().any(|kv| kv.key == "a"));
                // Log 'a' should remain
                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
                assert!(log_attrs.iter().any(|kv| kv.key == "a"));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_delete_scoped_to_scope_only_logs() {
        // Resource has 'a', scope has 'a' plus another key, log has 'a' and another key to keep batches non-empty
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

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-delete-scope");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();
                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                // Resource 'a' should remain
                let res_attrs = &decoded.resource_logs[0]
                    .resource
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(res_attrs.iter().any(|kv| kv.key == "a"));
                // Scope 'a' should be deleted
                let scope_attrs = &decoded.resource_logs[0].scope_logs[0]
                    .scope
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(!scope_attrs.iter().any(|kv| kv.key == "a"));
                // Log 'a' should remain
                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
                assert!(log_attrs.iter().any(|kv| kv.key == "a"));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_delete_scoped_to_signal_and_resource() {
        // Resource has 'a' and 'r', scope has 'a' and 's', log has 'a' and 'b'
        // Deleting 'a' for [signal, resource] should remove it from resource and logs, keep in scope.
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

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-delete-signal-and-resource");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();
                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                // Resource 'a' should be deleted; 'r' should remain
                let res_attrs = &decoded.resource_logs[0]
                    .resource
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(!res_attrs.iter().any(|kv| kv.key == "a"));
                assert!(res_attrs.iter().any(|kv| kv.key == "r"));

                // Scope 'a' should remain
                let scope_attrs = &decoded.resource_logs[0].scope_logs[0]
                    .scope
                    .as_ref()
                    .unwrap()
                    .attributes;
                assert!(scope_attrs.iter().any(|kv| kv.key == "a"));
                assert!(scope_attrs.iter().any(|kv| kv.key == "s"));

                // Log 'a' should be deleted; 'b' should remain
                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
                assert!(!log_attrs.iter().any(|kv| kv.key == "a"));
                assert!(log_attrs.iter().any(|kv| kv.key == "b"));
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_delete_all_attributes() {
        let input = build_logs_with_attrs(
            vec![],
            vec![],
            vec![
                KeyValue::new("a", AnyValue::new_string("1")),
                KeyValue::new("b", AnyValue::new_string("1")),
            ],
        );

        let cfg = json!({
            "actions": [
                {"action": "delete", "key": "a"},
                {"action": "delete", "key": "b"},
            ]
        });

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("attributes-processor-delete-resource");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc =
            create_attributes_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otap_batch = match first {
                    OtapPayload::OtapArrowRecords(otap_batch) => otap_batch,
                    _ => panic!("unexpected output payload type"),
                };

                assert!(
                    otap_batch.get(ArrowPayloadType::LogAttrs).is_none(),
                    "expected LogAttrs payload type to have been removed"
                );
            })
            .validate(|_| async move {});
    }
}

#[cfg(test)]
mod telemetry_tests {
    use super::*;
    use crate::pdata::OtapPdata;
    use bytes::BytesMut;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NodeControlMsg;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::{node::test_node, processor::TestRuntime};
    use otap_df_pdata::OtlpProtoBytes;
    use prost::Message as _;
    use serde_json::json;

    #[test]
    fn test_metrics_collect_telemetry_reports_counters() {
        use std::sync::Arc;

        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let telemetry_registry = rt.metrics_registry();
        let metrics_reporter = rt.metrics_reporter();

        // 2) Pipeline context sharing the same registry handle
        let controller = ControllerContext::new(rt.metrics_registry().clone());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);

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
            use otap_df_pdata::proto::opentelemetry::{
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
            let mut bytes = BytesMut::new();
            req.encode(&mut bytes).expect("encode");
            bytes.freeze()
        };

        // 5) Drive processor with TestRuntime; start collector inside and then request a telemetry snapshot
        let phase = rt.set_processor(proc);
        phase
            .run_test(|mut ctx| async move {
                // Process one message
                let pdata =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(input_bytes).into());
                ctx.process(Message::PData(pdata)).await.expect("pdata");

                // Trigger telemetry snapshot
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect");
            })
            .validate(move |_| async move {
                // Allow the collector to pull from the channel
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                // Inspect current metrics; fields with non-zero values should be present
                let mut found_renamed_entries = false;
                let mut found_deleted_entries = false;
                let mut found_domain_signal = false;

                telemetry_registry.visit_current_metrics(|desc, _attrs, iter| {
                    if desc.name == "attributes.processor.metrics" {
                        for (field, v) in iter {
                            match (field.name, v.to_u64_lossy()) {
                                ("renamed.entries", x) if x >= 1 => found_renamed_entries = true,
                                ("deleted.entries", x) if x >= 1 => found_deleted_entries = true,
                                ("domains.signal", x) if x >= 1 => found_domain_signal = true,
                                _ => {}
                            }
                        }
                    }
                });

                assert!(found_renamed_entries, "renamed.entries should be >= 1");
                assert!(found_deleted_entries, "deleted.entries should be >= 1");
                assert!(found_domain_signal, "domains.signal should be >= 1");
            });
    }
}
