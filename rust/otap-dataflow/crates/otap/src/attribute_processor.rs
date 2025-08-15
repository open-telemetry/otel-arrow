// SPDX-License-Identifier: Apache-2.0

//! Attribute processor for OTAP pipelines.
//!
//! This processor provides attribute transformations for telemetry data, supporting
//! the OpenTelemetry Collector's attributes processor configuration format. It operates
//! on OTAP Arrow payloads (OtapArrowRecords and OtapArrowBytes) and can convert OTLP
//! bytes to OTAP for processing.
//!
//! Currently supports a subset of the collector's operations:
//! - `update` with `value`: Renames an attribute by changing its key
//! - `delete`: Removes an attribute by key
//!
//! Other operations (`insert`, `upsert`, `hash`, `extract`, `convert`) are accepted in
//! configuration but not yet implemented.
//!
//! Example configuration (YAML):
//! ```yaml
//! actions:
//!   - key: "http.method"
//!     action: "update"
//!     value: "rpc.method"    # Renames http.method to rpc.method
//!   - key: "db.statement"
//!     action: "delete"       # Removes db.statement attribute
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
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// URN for the AttributeProcessor
pub const ATTRIBUTE_PROCESSOR_URN: &str = "urn:otap:processor:attribute_processor";

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Action types supported by the attributes processor.
pub enum ActionType {
    /// Insert a new attribute with the specified value.
    /// Currently not implemented.
    #[serde(rename = "insert")]
    Insert,
    /// Update the value of an existing attribute or rename it if value is provided.
    /// Currently only supports renaming via the value field.
    #[serde(rename = "update")]
    Update,
    /// Insert a new attribute or update an existing one.
    /// Currently not implemented.
    #[serde(rename = "upsert")]
    Upsert,
    /// Delete an attribute by key.
    #[serde(rename = "delete")]
    Delete,
    /// Hash the value of an attribute.
    /// Currently not implemented.
    #[serde(rename = "hash")]
    Hash,
    /// Extract a value from an attribute using a regex pattern.
    /// Currently not implemented.
    #[serde(rename = "extract")]
    Extract,
    /// Convert an attribute's value to a different type.
    /// Currently not implemented.
    #[serde(rename = "convert")]
    Convert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// An action to perform on attributes.
pub struct Action {
    /// The attribute key to act on.
    pub key: String,
    /// The type of action to perform.
    pub action: ActionType,
    /// Optional value to use in the action.
    #[serde(default)]
    pub value: Option<String>,
    /// Optional source attribute key for extract operations.
    #[serde(default)]
    pub from_attribute: Option<String>,
    /// Optional regex pattern for extract operations.
    #[serde(default)]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Configuration for the AttributeProcessor.
///
/// Accepts configuration in the same format as the OpenTelemetry Collector's attributes processor.
/// Currently supports a subset of operations that can be implemented via
/// otel_arrow_rust::otap::transform::transform_attributes:
/// - update: Implemented as rename when value is provided
/// - delete: Remove attributes by key
///
/// Other operations (insert, upsert, hash, extract, convert) are accepted in configuration
/// but not yet implemented.
pub struct AttributeProcessorConfig {
    /// List of actions to apply in order.
    #[serde(default)]
    pub actions: Vec<Action>,
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
}

impl AttributeProcessor {
    /// Create a new AttributeProcessor with the given configuration.
    ///
    /// Transforms the Go collector-style configuration into the operations
    /// supported by the underlying Arrow attribute transform API.
    #[must_use]
    pub fn new(config: AttributeProcessorConfig) -> Self {
        let mut renames = BTreeMap::new();
        let mut deletes = BTreeSet::new();

        for action in config.actions {
            match action.action {
                ActionType::Delete => {
                    let _ = deletes.insert(action.key);
                }
                ActionType::Update => {
                    // For update actions with a value, treat as rename
                    if let Some(new_value) = action.value {
                        let _ = renames.insert(action.key, new_value);
                    }
                }
                // Other actions not yet supported
                _ => {}
            }
        }

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
                        apply_transform(records, signal, &self.transform)?;
                        effect_handler.send_message(pdata).await
                    }
                    OtapPdata::OtapArrowBytes(_) => {
                        // Convert to records, apply, convert back to bytes to preserve variant
                        let signal_ty = pdata.signal_type();
                        let records: OtapArrowRecords = pdata.clone().try_into()?;
                        let mut records_mut = records;
                        apply_transform(&mut records_mut, signal_ty, &self.transform)?;
                        let bytes: crate::grpc::OtapArrowBytes = records_mut.try_into()?;
                        let pdata_back: OtapPdata = bytes.into();
                        effect_handler.send_message(pdata_back).await
                    }
                    OtapPdata::OtlpBytes(otlp_bytes) => {
                        // Convert to OTAP, apply transform, convert back
                        let records: OtapArrowRecords = otlp_bytes.try_into()?;
                        let mut records_mut = records;
                        apply_transform(&mut records_mut, signal, &self.transform)?;
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
) -> Result<(), EngineError<OtapPdata>> {
    let payloads = attrs_payloads_for_signal(signal);

    // Only apply if we have transforms to apply
    if transform.rename.is_some() || transform.delete.is_some() {
        for &payload_ty in payloads {
            if let Some(rb) = records.get(payload_ty).cloned() {
                let rb = transform_attributes(&rb, transform)
                    .map_err(|e| engine_err(&format!("transform_attributes failed: {e}")))?;
                records.set(payload_ty, rb);
            }
        }
    }

    Ok(())
}

fn attrs_payloads_for_signal(signal: SignalType) -> &'static [ArrowPayloadType] {
    use ArrowPayloadType as A;
    match signal {
        SignalType::Logs => &[A::ResourceAttrs, A::ScopeAttrs, A::LogAttrs],
        SignalType::Metrics => &[
            A::ResourceAttrs,
            A::ScopeAttrs,
            A::MetricAttrs,
            A::NumberDpAttrs,
            A::HistogramDpAttrs,
            A::SummaryDpAttrs,
            A::NumberDpExemplarAttrs,
            A::HistogramDpExemplarAttrs,
        ],
        SignalType::Traces => &[
            A::ResourceAttrs,
            A::ScopeAttrs,
            A::SpanAttrs,
            A::SpanEventAttrs,
            A::SpanLinkAttrs,
        ],
    }
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
    let cfg: AttributeProcessorConfig =
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse AttributeProcessor configuration: {e}"),
        })?;

    let user_config = Arc::new(NodeUserConfig::new_processor_config(
        ATTRIBUTE_PROCESSOR_URN,
    ));
    Ok(ProcessorWrapper::local(
        AttributeProcessor::new(cfg),
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
                    "key": "http.method",
                    "action": "update",
                    "value": "rpc.method"
                }
            ]
        });
        let processor = AttributeProcessor::new(serde_json::from_value(config).unwrap());
        apply_transform(&mut otap_records, SignalType::Logs, &processor.transform).unwrap();

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
                    "key": "http.method",
                    "action": "update",
                    "value": "rpc.method"
                },
                {
                    "key": "user_agent",
                    "action": "update",
                    "value": "http.user_agent"
                }
            ]
        });
        let processor = AttributeProcessor::new(serde_json::from_value(config).unwrap());
        apply_transform(&mut otap_records, SignalType::Logs, &processor.transform).unwrap();

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
                    "key": "http.method",
                    "action": "update",
                    "value": "rpc.method"
                }
            ]
        });
        let processor = AttributeProcessor::new(serde_json::from_value(config).unwrap());
        apply_transform(&mut records, SignalType::Metrics, &processor.transform).unwrap();

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
                    "key": "A",
                    "action": "update",
                    "value": "X"
                },
                {
                    "key": "B",
                    "action": "update",
                    "value": "Y"
                }
            ]
        });
        let processor = AttributeProcessor::new(serde_json::from_value(config).unwrap());
        apply_transform(&mut records, SignalType::Traces, &processor.transform).unwrap();

        let rb2 = records.get(ArrowPayloadType::SpanAttrs).unwrap();
        let all = collect_key_strings(rb2);
        let expected: Vec<String> = vec!["X", "Y", "keep"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(all, expected);
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
        let processor = AttributeProcessor::new(serde_json::from_value(config).unwrap());
        apply_transform(&mut records, SignalType::Logs, &processor.transform).unwrap();

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
                    "key": "http.method",
                    "action": "insert",
                    "value": "GET"
                },
                {
                    "key": "http.method",
                    "action": "update",
                    "value": "rpc.method"
                }
            ]
        });
        let processor = AttributeProcessor::new(serde_json::from_value(config).unwrap());
        apply_transform(&mut records, SignalType::Logs, &processor.transform).unwrap();

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
                    "key": "old.key",
                    "action": "update",
                    "value": "new.key"
                }
            ]
        });
        let processor = AttributeProcessor::new(serde_json::from_value(config).unwrap());
        apply_transform(&mut records, SignalType::Traces, &processor.transform).unwrap();

        let rb2 = records.get(ArrowPayloadType::SpanAttrs).unwrap();
        let key_idx = rb2.schema().index_of("key").unwrap();
        let keys = arrow::array::as_string_array(rb2.column(key_idx));
        let all: Vec<&str> = (0..keys.len()).map(|i| keys.value(i)).collect();
        assert_eq!(all, vec!["new.key", "other"]);
    }
}
