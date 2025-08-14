// SPDX-License-Identifier: Apache-2.0

//! Attribute processor for OTAP pipelines.
//!
//! This processor applies attribute rename rules to OTAP Arrow payloads across
//! logs, metrics, and traces. It operates only on OTAP variants (OtapArrowRecords
//! and OtapArrowBytes). OTLP bytes are passed through unchanged in this first
//! version.
//!
//! Notes:
//! - Keep configuration minimal: only simple rename rules are supported, using
//!   otel_arrow_rust::otap::transform::rename_attributes for exact key matches.

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
use otel_arrow_rust::otap::{OtapArrowRecords, transform::rename_attributes};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

/// URN for the AttributeProcessor
pub const ATTRIBUTE_PROCESSOR_URN: &str = "urn:otap:processor:attribute_processor";

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A single rename rule specifying the source and destination attribute key.
pub struct RenameRule {
    /// The exact attribute key to replace (matched exactly, case-sensitive).
    pub from: String,
    /// The replacement attribute key.
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Configuration for the AttributeProcessor.
///
/// Only simple rename rules are supported; behavior follows
/// otel_arrow_rust::otap::transform::rename_attributes (exact key matches).
pub struct AttributeProcessorConfig {
    /// List of rename rules to apply (in order) to all attribute payloads for the signal.
    pub rules: Vec<RenameRule>,
    // No extra configuration beyond simple renames for now.
}

/// Processor that applies rename rules to OTAP attribute batches.
pub struct AttributeProcessor {
    // Precomputed rename map to avoid rebuilding per message
    renames: BTreeMap<String, String>,
}

impl AttributeProcessor {
    /// Create a new AttributeProcessor with the given configuration.
    #[must_use]
    pub fn new(config: AttributeProcessorConfig) -> Self {
        let mut renames: BTreeMap<String, String> = BTreeMap::new();
        for rule in &config.rules {
            let _ = renames.insert(rule.from.clone(), rule.to.clone());
        }
        Self { renames }
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
                        apply_rules(records, signal, &self.renames)?;
                        effect_handler.send_message(pdata).await
                    }
                    OtapPdata::OtapArrowBytes(_) => {
                        // Convert to records, apply, convert back to bytes to preserve variant
                        let signal_ty = pdata.signal_type();
                        let records: OtapArrowRecords = pdata.clone().try_into()?;
                        let mut records_mut = records;
                        apply_rules(&mut records_mut, signal_ty, &self.renames)?;
                        let bytes: crate::grpc::OtapArrowBytes = records_mut.try_into()?;
                        let pdata_back: OtapPdata = bytes.into();
                        effect_handler.send_message(pdata_back).await
                    }
                    OtapPdata::OtlpBytes(_) => {
                        // OTAP-only for now: pass through unchanged
                        effect_handler.send_message(pdata).await
                    }
                }
            }
        }
    }
}

#[allow(clippy::result_large_err)]
fn apply_rules(
    records: &mut OtapArrowRecords,
    signal: SignalType,
    owned_map: &BTreeMap<String, String>,
) -> Result<(), EngineError<OtapPdata>> {
    let payloads = attrs_payloads_for_signal(signal);

    if !owned_map.is_empty() {
        // Create a borrowed map view for the transform API
        let replacements: BTreeMap<&str, &str> = owned_map
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        for &payload_ty in payloads {
            if let Some(rb) = records.get(payload_ty).cloned() {
                let rb = rename_attributes(&rb, &replacements)
                    .map_err(|e| engine_err(&format!("rename_attributes failed: {e}")))?;
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

/// Factory function to create an AttributeProcessor
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
    use arrow::array::{Array, DictionaryArray, Int64Array, StringArray, UInt8Array, UInt16Array};
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

    fn make_attrs_batch(parent_ids: Vec<i64>, keys: Vec<&str>) -> RecordBatch {
        assert_eq!(parent_ids.len(), keys.len());
        let pid_arr = Int64Array::from(parent_ids);
        let key_arr = StringArray::from(keys);
        let schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::Int64, false),
            Field::new("key", DataType::Utf8, false),
        ]));
        RecordBatch::try_new(schema, vec![Arc::new(pid_arr), Arc::new(key_arr)]).unwrap()
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

        let config = json!({ "rules": [ { "from": "http.method", "to": "rpc.method" } ] });
        let cfg: AttributeProcessorConfig = serde_json::from_value(config).unwrap();
        let mut map = BTreeMap::new();
        for r in cfg.rules {
            let _ = map.insert(r.from, r.to);
        }
        apply_rules(&mut otap_records, SignalType::Logs, &map).unwrap();

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
            "rules": [
                { "from": "http.method", "to": "rpc.method" },
                { "from": "user_agent", "to": "http.user_agent" }
            ]
        });
        let cfg: AttributeProcessorConfig = serde_json::from_value(config).unwrap();
        let mut map = BTreeMap::new();
        for r in cfg.rules {
            let _ = map.insert(r.from, r.to);
        }
        apply_rules(&mut otap_records, SignalType::Logs, &map).unwrap();

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
        let rb = make_attrs_batch(vec![0, 1], vec!["http.method", "keep.me"]);
        records.set(ArrowPayloadType::MetricAttrs, rb);

        let cfg: AttributeProcessorConfig = serde_json::from_value(json!({
            "rules": [ { "from": "http.method", "to": "rpc.method" } ]
        }))
        .unwrap();

        let mut map = BTreeMap::new();
        for r in cfg.rules {
            let _ = map.insert(r.from, r.to);
        }
        apply_rules(&mut records, SignalType::Metrics, &map).unwrap();
        let rb2 = records.get(ArrowPayloadType::MetricAttrs).unwrap();
        let key_idx = rb2.schema().index_of("key").unwrap();
        let keys = arrow::array::as_string_array(rb2.column(key_idx));
        let all: Vec<&str> = (0..keys.len()).map(|i| keys.value(i)).collect();
        assert_eq!(all, vec!["rpc.method", "keep.me"]);
    }

    #[test]
    fn test_rename_traces_multiple_renames_and_no_chain() {
        let mut records = OtapArrowRecords::Traces(Default::default());
        // Include rows for A and B to verify no chaining A->B->C in a single call
        let rb = make_attrs_batch(vec![7, 7, 8], vec!["A", "B", "keep"]);
        records.set(ArrowPayloadType::SpanAttrs, rb);

        let cfg: AttributeProcessorConfig = serde_json::from_value(json!({
            "rules": [
                { "from": "A", "to": "B" },
                { "from": "B", "to": "C" }
            ]
        }))
        .unwrap();

        let mut map = BTreeMap::new();
        for r in cfg.rules {
            let _ = map.insert(r.from, r.to);
        }
        apply_rules(&mut records, SignalType::Traces, &map).unwrap();
        let rb2 = records.get(ArrowPayloadType::SpanAttrs).unwrap();
        let all = collect_key_strings(rb2);
        // Expect original A->B and original B->C, but A should not chain to C within same call
        let expected: Vec<String> = vec!["B", "C", "keep"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(all, expected);
    }

    #[test]
    fn test_rename_traces_span_attrs_basic() {
        let mut records = OtapArrowRecords::Traces(Default::default());
        let rb = make_attrs_batch(vec![7, 8], vec!["old.key", "other"]);
        records.set(ArrowPayloadType::SpanAttrs, rb);

        let cfg: AttributeProcessorConfig = serde_json::from_value(json!({
            "rules": [ { "from": "old.key", "to": "new.key" } ]
        }))
        .unwrap();

        let mut map = BTreeMap::new();
        for r in cfg.rules {
            let _ = map.insert(r.from, r.to);
        }
        apply_rules(&mut records, SignalType::Traces, &map).unwrap();
        let rb2 = records.get(ArrowPayloadType::SpanAttrs).unwrap();
        let key_idx = rb2.schema().index_of("key").unwrap();
        let keys = arrow::array::as_string_array(rb2.column(key_idx));
        let all: Vec<&str> = (0..keys.len()).map(|i| keys.value(i)).collect();
        assert_eq!(all, vec!["new.key", "other"]);
    }
}
