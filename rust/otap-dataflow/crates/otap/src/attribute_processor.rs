// SPDX-License-Identifier: Apache-2.0

//! Attribute processor for OTAP pipelines.
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
//!     from: "http.method"
//!     to: "rpc.method"       # Renames http.method to rpc.method
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
use otap_df_config::experimental::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::processor::ProcessorWrapper;
use otel_arrow_rust::otap::{
    OtapArrowRecords,
    transform::{AttributesTransform, transform_attributes},
};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

/// URN for the AttributeProcessor
pub const ATTRIBUTE_PROCESSOR_URN: &str = "urn:otap:processor:attribute_processor";

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Actions that can be performed on attributes.
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    /// Rename an existing attribute key (non-standard; deviates from Go config).
    Rename {
        /// The source key to rename from.
        from: String,
        /// The destination key to rename to.
        to: String,
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
/// Configuration for the AttributeProcessor.
///
/// Accepts configuration in the same format as the OpenTelemetry Collector's attributes processor.
/// Supported actions: rename (deviation), delete. Others are ignored.
///
/// You can control which attribute domains are transformed via `apply_to`.
/// Valid values: "signal" (default), "resource", "scope".
pub struct AttributeProcessorConfig {
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
pub struct AttributeProcessor {
    // Pre-computed transform to avoid rebuilding per message
    transform: AttributesTransform,
    // Selected attribute domains to transform
    domains: HashSet<ApplyDomain>,
}

impl AttributeProcessor {
    /// Creates a new AttributeProcessor from configuration.
    ///
    /// Transforms the Go collector-style configuration into the operations
    /// supported by the underlying Arrow attribute transform API.
    #[must_use = "AttributeProcessor creation may fail and return a ConfigError"]
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let cfg: AttributeProcessorConfig =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse AttributeProcessor configuration: {e}"),
            })?;
        Ok(Self::new(cfg))
    }

    /// Creates a new AttributeProcessor with the given parsed configuration.
    #[must_use]
    fn new(config: AttributeProcessorConfig) -> Self {
        let mut renames = BTreeMap::new();
        let mut deletes = BTreeSet::new();

        for action in config.actions {
            match action {
                Action::Delete { key } => {
                    let _ = deletes.insert(key);
                }
                Action::Rename { from, to } => {
                    let _ = renames.insert(from, to);
                }
                // Unsupported actions are ignored for now
                Action::Unsupported => {}
            }
        }

        let domains = parse_apply_to(config.apply_to.as_ref());

        Self {
            transform: AttributesTransform {
                rename: if renames.is_empty() {
                    None
                } else {
                    Some(renames)
                },
                delete: if deletes.is_empty() {
                    None
                } else {
                    Some(deletes)
                },
            },
            domains,
        }
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for AttributeProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError<OtapPdata>> {
        match msg {
            Message::Control(_) => Ok(()),
            Message::PData(pdata) => {
                let signal = pdata.signal_type();
                let mut pdata = pdata;

                match pdata {
                    OtapPdata::OtapArrowRecords(ref mut records) => {
                        apply_transform(records, signal, &self.transform, &self.domains)?;
                        effect_handler.send_message(pdata).await
                    }
                    OtapPdata::OtapArrowBytes(_) => {
                        // Convert to records, apply, convert back to bytes to preserve variant
                        let signal_ty = pdata.signal_type();
                        let records: OtapArrowRecords = pdata.clone().try_into()?;
                        let mut records_mut = records;
                        apply_transform(
                            &mut records_mut,
                            signal_ty,
                            &self.transform,
                            &self.domains,
                        )?;
                        let bytes: crate::grpc::OtapArrowBytes = records_mut.try_into()?;
                        let pdata_back: OtapPdata = bytes.into();
                        effect_handler.send_message(pdata_back).await
                    }
                    OtapPdata::OtlpBytes(otlp_bytes) => {
                        // Convert to OTAP, apply transform, convert back
                        let records: OtapArrowRecords = otlp_bytes.try_into()?;
                        let mut records_mut = records;
                        apply_transform(&mut records_mut, signal, &self.transform, &self.domains)?;
                        let bytes: crate::pdata::OtlpProtoBytes = records_mut.try_into()?;
                        let pdata_back = OtapPdata::OtlpBytes(bytes);
                        effect_handler.send_message(pdata_back).await
                    }
                }
            }
        }
    }
}

#[allow(clippy::result_large_err)]
fn apply_transform(
    records: &mut OtapArrowRecords,
    signal: SignalType,
    transform: &AttributesTransform,
    domains: &HashSet<ApplyDomain>,
) -> Result<(), EngineError<OtapPdata>> {
    let payloads = attrs_payloads(signal, domains);

    // Only apply if we have transforms to apply
    if transform.rename.is_some() || transform.delete.is_some() {
        for payload_ty in payloads {
            if let Some(rb) = records.get(payload_ty).cloned() {
                let rb = transform_attributes(&rb, transform)
                    .map_err(|e| engine_err(&format!("transform_attributes failed: {e}")))?;
                records.set(payload_ty, rb);
            }
        }
    }

    Ok(())
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
        Some(list) if list.is_empty() => {
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

fn attrs_payloads(signal: SignalType, domains: &HashSet<ApplyDomain>) -> Vec<ArrowPayloadType> {
    use ArrowPayloadType as A;
    let mut out: Vec<ArrowPayloadType> = Vec::new();
    // Domains are unioned
    if domains.contains(&ApplyDomain::Resource) {
        out.push(A::ResourceAttrs);
    }
    if domains.contains(&ApplyDomain::Scope) {
        out.push(A::ScopeAttrs);
    }
    if domains.contains(&ApplyDomain::Signal) {
        match signal {
            SignalType::Logs => {
                out.push(A::LogAttrs);
            }
            SignalType::Metrics => {
                out.push(A::MetricAttrs);
                out.push(A::NumberDpAttrs);
                out.push(A::HistogramDpAttrs);
                out.push(A::SummaryDpAttrs);
                out.push(A::NumberDpExemplarAttrs);
                out.push(A::HistogramDpExemplarAttrs);
            }
            SignalType::Traces => {
                out.push(A::SpanAttrs);
                out.push(A::SpanEventAttrs);
                out.push(A::SpanLinkAttrs);
            }
        }
    }
    out
}

fn engine_err(msg: &str) -> EngineError<OtapPdata> {
    EngineError::PdataConversionError {
        error: msg.to_string(),
    }
}

/// Factory function to create an AttributeProcessor.
///
/// Accepts configuration in OpenTelemetry Collector attributes processor format.
/// See the module documentation for configuration examples and supported operations.
pub fn create_attribute_processor(
    config: &Value,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let user_config = Arc::new(NodeUserConfig::new_processor_config(
        ATTRIBUTE_PROCESSOR_URN,
    ));
    Ok(ProcessorWrapper::local(
        AttributeProcessor::from_config(config)?,
        user_config,
        processor_config,
    ))
}

/// Register AttributeProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static ATTRIBUTE_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: ATTRIBUTE_PROCESSOR_URN,
        create: |config: &Value, proc_cfg: &ProcessorConfig| {
            create_attribute_processor(config, proc_cfg)
        },
    };

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, DictionaryArray, StringArray, UInt8Array, UInt16Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_rust::proto::opentelemetry::common::v1::InstrumentationScope;
    use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
    use prost::Message;
    use serde_json::json;
    use std::sync::Arc;

    fn make_attrs_batch(parent_ids: Vec<u16>, keys: Vec<&str>) -> RecordBatch {
        assert_eq!(parent_ids.len(), keys.len());
        let len = keys.len();
        let pid_arr = UInt16Array::from(parent_ids);
        let key_arr = StringArray::from(keys);
        let type_arr = UInt8Array::from(vec![0u8; len]);
        let schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, false),
            Field::new("key", DataType::Utf8, false),
            Field::new("type", DataType::UInt8, false),
        ]));
        RecordBatch::try_new(
            schema,
            vec![Arc::new(pid_arr), Arc::new(key_arr), Arc::new(type_arr)],
        )
        .unwrap()
    }

    fn collect_key_strings(rb: &RecordBatch) -> Vec<String> {
        let key_idx = rb.schema().index_of("key").unwrap();
        let col = rb.column(key_idx);
        match col.data_type() {
            DataType::Utf8 => {
                let arr = arrow::array::as_string_array(col);
                (0..arr.len()).map(|i| arr.value(i).to_string()).collect()
            }
            DataType::Dictionary(k, _) => match k.as_ref() {
                DataType::UInt8 => {
                    let dict = col
                        .as_any()
                        .downcast_ref::<DictionaryArray<arrow::datatypes::UInt8Type>>()
                        .unwrap();
                    let keys: &UInt8Array = dict.keys();
                    let values = dict
                        .values()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .unwrap();
                    (0..keys.len())
                        .map(|i| {
                            let idx = keys.value(i) as usize;
                            values.value(idx).to_string()
                        })
                        .collect()
                }
                DataType::UInt16 => {
                    let dict = col
                        .as_any()
                        .downcast_ref::<DictionaryArray<arrow::datatypes::UInt16Type>>()
                        .unwrap();
                    let keys: &UInt16Array = dict.keys();
                    let values = dict
                        .values()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .unwrap();
                    (0..keys.len())
                        .map(|i| {
                            let idx = keys.value(i) as usize;
                            values.value(idx).to_string()
                        })
                        .collect()
                }
                other => panic!("Unsupported dictionary key type: {other:?}"),
            },
            other => panic!("Unsupported key column type: {other:?}"),
        }
    }

    #[test]
    fn test_rename_logs_attrs_basic() {
        // Build a simple logs request with one log having the from/to keys
        let logs_req = ExportLogsServiceRequest::new(vec![
            ResourceLogs::build(Resource::default())
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::default())
                        .log_records(vec![
                            LogRecord::build(1u64, SeverityNumber::Info, "evt")
                                .attributes(vec![
                                    KeyValue::new("http.method", AnyValue::new_string("GET")),
                                    KeyValue::new("rpc.method", AnyValue::new_string("Existing")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);
        let mut bytes = vec![];
        logs_req.encode(&mut bytes).unwrap();

        let pdata = OtapPdata::OtlpBytes(crate::pdata::OtlpProtoBytes::ExportLogsRequest(bytes));
        let mut otap_records: OtapArrowRecords = pdata.clone().try_into().unwrap();

        let config = json!({
            "actions": [
                {
                    "action": "rename",
                    "from": "http.method",
                    "to": "rpc.method"
                }
            ]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut otap_records,
            SignalType::Logs,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        if let Some(rb) = otap_records.get(ArrowPayloadType::LogAttrs) {
            let all_keys = collect_key_strings(rb);
            assert!(all_keys.iter().any(|k| k == "rpc.method"));
            assert!(!all_keys.iter().any(|k| k == "http.method"));
        }
    }

    #[test]
    fn test_rename_logs_multiple_renames() {
        // Build logs with multiple attribute keys to rename
        let logs_req = ExportLogsServiceRequest::new(vec![
            ResourceLogs::build(Resource::default())
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::default())
                        .log_records(vec![
                            LogRecord::build(1u64, SeverityNumber::Info, "evt")
                                .attributes(vec![
                                    KeyValue::new("http.method", AnyValue::new_string("GET")),
                                    KeyValue::new("user_agent", AnyValue::new_string("UA")),
                                    KeyValue::new("keep", AnyValue::new_string("ok")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);
        let mut bytes = vec![];
        logs_req.encode(&mut bytes).unwrap();

        let pdata = OtapPdata::OtlpBytes(crate::pdata::OtlpProtoBytes::ExportLogsRequest(bytes));
        let mut otap_records: OtapArrowRecords = pdata.clone().try_into().unwrap();

        let config = json!({
            "actions": [
                {
                    "action": "rename",
                    "from": "http.method",
                    "to": "rpc.method"
                },
                {
                    "action": "rename",
                    "from": "user_agent",
                    "to": "http.user_agent"
                }
            ]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut otap_records,
            SignalType::Logs,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        if let Some(rb) = otap_records.get(ArrowPayloadType::LogAttrs) {
            let all = collect_key_strings(rb);
            assert!(all.iter().any(|k| k == "rpc.method"));
            assert!(all.iter().any(|k| k == "http.user_agent"));
            assert!(all.iter().any(|k| k == "keep"));
            assert!(!all.iter().any(|k| k == "http.method"));
            assert!(!all.iter().any(|k| k == "user_agent"));
        }
    }

    #[test]
    fn test_rename_metrics_attrs_basic() {
        let mut records = OtapArrowRecords::Metrics(Default::default());
        let rb = make_attrs_batch(vec![0u16, 1u16], vec!["http.method", "keep.me"]);
        records.set(ArrowPayloadType::MetricAttrs, rb);

        let config = json!({
            "actions": [
                {
                    "action": "rename",
                    "from": "http.method",
                    "to": "rpc.method"
                }
            ]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut records,
            SignalType::Metrics,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        let rb2 = records.get(ArrowPayloadType::MetricAttrs).unwrap();
        let key_idx = rb2.schema().index_of("key").unwrap();
        let keys = arrow::array::as_string_array(rb2.column(key_idx));
        let all: Vec<&str> = (0..keys.len()).map(|i| keys.value(i)).collect();
        assert_eq!(all, vec!["rpc.method", "keep.me"]);
    }

    #[test]
    fn test_rename_traces_multiple_renames() {
        let mut records = OtapArrowRecords::Traces(Default::default());
        // Include multiple different keys to verify we can rename them all at once
        let rb = make_attrs_batch(vec![7u16, 7u16, 8u16], vec!["A", "B", "keep"]);
        records.set(ArrowPayloadType::SpanAttrs, rb);

        let config = json!({
            "actions": [
                {
                    "action": "rename",
                    "from": "A",
                    "to": "X"
                },
                {
                    "action": "rename",
                    "from": "B",
                    "to": "Y"
                }
            ]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut records,
            SignalType::Traces,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        let rb2 = records.get(ArrowPayloadType::SpanAttrs).unwrap();
        let all = collect_key_strings(rb2);
        let expected: Vec<String> = vec!["X", "Y", "keep"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(all, expected);
    }

    #[test]
    fn test_resource_only_apply_to() {
        // Resource-only rename
        let mut records = OtapArrowRecords::Logs(Default::default());
        // Put keys in ResourceAttrs
        let rb = make_attrs_batch(vec![0u16, 0u16], vec!["svc.name", "keep"]);
        records.set(ArrowPayloadType::ResourceAttrs, rb);

        let config = json!({
            "actions": [
                { "action": "rename", "from": "svc.name", "to": "service.name" }
            ],
            "apply_to": ["resource"]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut records,
            SignalType::Logs,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        let rb2 = records.get(ArrowPayloadType::ResourceAttrs).unwrap();
        let all = collect_key_strings(rb2);
        assert_eq!(all, vec!["service.name", "keep"]);

        // Ensure LogAttrs are untouched (none present still none)
        assert!(records.get(ArrowPayloadType::LogAttrs).is_none());
    }

    #[test]
    fn test_signal_and_resource_apply_to() {
        // Build a Logs record with both Resource and Log attributes present
        let mut records = OtapArrowRecords::Logs(Default::default());
        // Resource attrs
        let rb_res = make_attrs_batch(vec![0u16, 0u16], vec!["svc.name", "keep_r"]);
        records.set(ArrowPayloadType::ResourceAttrs, rb_res);
        // Log attrs
        let rb_log = make_attrs_batch(vec![0u16, 0u16], vec!["http.method", "keep_l"]);
        records.set(ArrowPayloadType::LogAttrs, rb_log);

        let config = json!({
            "actions": [
                { "action": "rename", "from": "svc.name", "to": "service.name" },
                { "action": "rename", "from": "http.method", "to": "rpc.method" }
            ],
            "apply_to": ["signal", "resource"]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut records,
            SignalType::Logs,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        // Resource rename applied
        let rb2_res = records.get(ArrowPayloadType::ResourceAttrs).unwrap();
        let all_res = collect_key_strings(rb2_res);
        assert_eq!(all_res, vec!["service.name", "keep_r"]);

        // Signal (log) rename applied
        let rb2_log = records.get(ArrowPayloadType::LogAttrs).unwrap();
        let all_log = collect_key_strings(rb2_log);
        assert_eq!(all_log, vec!["rpc.method", "keep_l"]);
    }

    #[test]
    fn test_delete_attrs() {
        let mut records = OtapArrowRecords::Logs(Default::default());
        let rb = make_attrs_batch(
            vec![1u16, 2u16, 3u16],
            vec!["http.method", "secret", "user.id"],
        );
        records.set(ArrowPayloadType::LogAttrs, rb);

        let config = json!({
            "actions": [
                {
                    "key": "secret",
                    "action": "delete"
                }
            ]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut records,
            SignalType::Logs,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        let rb2 = records.get(ArrowPayloadType::LogAttrs).unwrap();
        let all = collect_key_strings(rb2);
        assert_eq!(all, vec!["http.method", "user.id"]);
    }

    #[test]
    fn test_unsupported_actions() {
        let mut records = OtapArrowRecords::Logs(Default::default());
        let rb = make_attrs_batch(vec![1u16, 2u16], vec!["http.method", "keep.me"]);
        records.set(ArrowPayloadType::LogAttrs, rb);

        let config = json!({
            "actions": [
                {
                    "action": "rename",
                    "from": "http.method",
                    "to": "rpc.method"
                },
                {
                    "action": "insert",
                    "key": "http.method",
                    "value": "GET"
                }
            ]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut records,
            SignalType::Logs,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        // Verify only the update action was applied
        let rb2 = records.get(ArrowPayloadType::LogAttrs).unwrap();
        let all = collect_key_strings(rb2);
        assert_eq!(all, vec!["rpc.method", "keep.me"]);
    }

    #[test]
    fn test_rename_traces_span_attrs_basic() {
        let mut records = OtapArrowRecords::Traces(Default::default());
        let rb = make_attrs_batch(vec![7u16, 8u16], vec!["old.key", "other"]);
        records.set(ArrowPayloadType::SpanAttrs, rb);

        let config = json!({
            "actions": [
                {
                    "action": "rename",
                    "from": "old.key",
                    "to": "new.key"
                }
            ]
        });
        let processor = AttributeProcessor::from_config(&config).unwrap();
        apply_transform(
            &mut records,
            SignalType::Traces,
            &processor.transform,
            &processor.domains,
        )
        .unwrap();

        let rb2 = records.get(ArrowPayloadType::SpanAttrs).unwrap();
        let key_idx = rb2.schema().index_of("key").unwrap();
        let keys = arrow::array::as_string_array(rb2.column(key_idx));
        let all: Vec<&str> = (0..keys.len()).map(|i| keys.value(i)).collect();
        assert_eq!(all, vec!["new.key", "other"]);
    }
}
