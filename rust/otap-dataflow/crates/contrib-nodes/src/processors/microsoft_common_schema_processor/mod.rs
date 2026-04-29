// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Microsoft Common Schema log promotion processor.
//!
//! This processor expects Microsoft Common Schema fields that have already been decoded
//! into log attributes, for example by the Linux `user_events` receiver in
//! `event_header` mode. It promotes known Microsoft Common Schema fields to
//! typed OTLP log fields and leaves non-Microsoft-Common-Schema records unchanged.
//!
//! TODO: Promote directly on `OtapArrowRecords`. The current implementation
//! converts input payloads to OTLP proto bytes because the promotion code is
//! written against mutable OTLP `LogsData` / `LogRecord` structs. OTAP Arrow
//! has the target fields, but they are split across immutable columnar
//! `Logs` and `LogAttrs` record batches. A native implementation needs a
//! reusable cross-batch transform that can read attributes by `parent_id`, set
//! root log columns such as body, event name, severity, trace/span IDs, flags,
//! and timestamp, then rebuild `LogAttrs` with promoted fields removed and
//! remaining fields preserved. Existing attribute/transform processors cannot
//! express that today: they operate on attribute batches with rename/delete and
//! literal insert/upsert operations, but they do not move per-log attribute
//! values into root log columns or parse Common Schema values into typed log
//! fields. Until that Arrow-native promotion path exists, Arrow-native
//! pipelines pay an Arrow-to-OTLP conversion here.

mod metrics;

use std::sync::Arc;

use arrow::array::{Array, DictionaryArray, RecordBatch, StringArray};
use arrow::datatypes::{UInt8Type, UInt16Type};
use async_trait::async_trait;
use bytes::BytesMut;
use chrono::DateTime;
use linkme::distributed_slice;
use metrics::MicrosoftCommonSchemaProcessorMetrics;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error as EngineError, ProcessorErrorKind};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::{OTAP_PROCESSOR_FACTORIES, pdata::OtapPdata};
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData};
use otap_df_pdata::schema::consts;
use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
use otap_df_telemetry::metrics::MetricSet;
use prost::Message as _;
use serde::Deserialize;

/// URN for the Microsoft Common Schema processor.
pub const MICROSOFT_COMMON_SCHEMA_PROCESSOR_URN: &str =
    "urn:microsoft:processor:common_schema_otel_logs";

/// Configuration for the Microsoft Common Schema processor.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

/// Processor that promotes decoded Microsoft Common Schema log attributes.
pub struct MicrosoftCommonSchemaProcessor {
    metrics: MetricSet<MicrosoftCommonSchemaProcessorMetrics>,
}

impl MicrosoftCommonSchemaProcessor {
    /// Creates a processor from user configuration.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, ConfigError> {
        let _config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse MicrosoftCommonSchemaProcessor configuration: {e}"),
            })?;
        Ok(Self {
            metrics: pipeline_ctx.register_metrics::<MicrosoftCommonSchemaProcessorMetrics>(),
        })
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for MicrosoftCommonSchemaProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            Message::Control(control) => {
                if let NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } = control
                {
                    let _ = metrics_reporter.report(&mut self.metrics);
                }
                Ok(())
            }
            Message::PData(pdata) => {
                if pdata.signal_type() != SignalType::Logs {
                    return effect_handler
                        .send_message_with_source_node(pdata)
                        .await
                        .map_err(Into::into);
                }

                let (context, payload) = pdata.into_parts();
                let result = promote_microsoft_common_schema_payload(
                    payload,
                    effect_handler.processor_id(),
                )?;
                if result.arrow_skipped_no_csver {
                    self.metrics.arrow_batches_skipped_no_csver.inc();
                }
                let stats = result.stats;
                self.metrics.records_seen.add(stats.records_seen);
                self.metrics.records_promoted.add(stats.records_promoted);
                self.metrics
                    .records_skipped_not_common_schema
                    .add(stats.records_skipped_not_common_schema());
                if result.promoted {
                    self.metrics.batches_promoted.inc();
                }

                effect_handler
                    .send_message_with_source_node(OtapPdata::new(context, result.payload))
                    .await
                    .map_err(Into::into)
            }
        }
    }
}

/// Factory function to create a Microsoft Common Schema processor.
pub fn create_microsoft_common_schema_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let proc = MicrosoftCommonSchemaProcessor::from_config(pipeline_ctx, &node_config.config)?;
    Ok(ProcessorWrapper::local(
        proc,
        node,
        node_config,
        processor_config,
    ))
}

/// Register MicrosoftCommonSchemaProcessor as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static MICROSOFT_COMMON_SCHEMA_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: MICROSOFT_COMMON_SCHEMA_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_microsoft_common_schema_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<Config>,
    };

fn processor_error(processor: NodeId, error: String) -> EngineError {
    EngineError::ProcessorError {
        processor,
        kind: ProcessorErrorKind::Other,
        error,
        source_detail: String::new(),
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct PromotionStats {
    records_seen: u64,
    records_promoted: u64,
}

impl PromotionStats {
    const fn promoted_any(self) -> bool {
        self.records_promoted > 0
    }

    fn records_skipped_not_common_schema(self) -> u64 {
        self.records_seen.saturating_sub(self.records_promoted)
    }
}

#[derive(Debug)]
struct PayloadPromotion {
    payload: OtapPayload,
    stats: PromotionStats,
    arrow_skipped_no_csver: bool,
    promoted: bool,
}

fn promote_microsoft_common_schema_payload(
    payload: OtapPayload,
    processor: NodeId,
) -> Result<PayloadPromotion, EngineError> {
    if let OtapPayload::OtapArrowRecords(records) = &payload {
        if arrow_logs_have_no_csver(records) {
            return Ok(PayloadPromotion {
                payload,
                stats: PromotionStats::default(),
                arrow_skipped_no_csver: true,
                promoted: false,
            });
        }
    }

    let original_arrow_payload = match &payload {
        OtapPayload::OtapArrowRecords(_) => Some(payload.clone()),
        OtapPayload::OtlpBytes(_) => None,
    };

    let otlp_bytes: OtlpProtoBytes = payload.try_into().map_err(|e| {
        processor_error(
            processor.clone(),
            format!("failed to convert input to OTLP logs: {e}"),
        )
    })?;
    let OtlpProtoBytes::ExportLogsRequest(bytes) = &otlp_bytes else {
        return Ok(PayloadPromotion {
            payload: otlp_bytes.into(),
            stats: PromotionStats::default(),
            arrow_skipped_no_csver: false,
            promoted: false,
        });
    };

    let mut logs = LogsData::decode(bytes.as_ref()).map_err(|e| {
        processor_error(
            processor.clone(),
            format!("failed to decode OTLP logs: {e}"),
        )
    })?;
    let stats = promote_microsoft_common_schema_logs(&mut logs);
    if !stats.promoted_any() {
        return Ok(PayloadPromotion {
            payload: original_arrow_payload.unwrap_or_else(|| otlp_bytes.into()),
            stats,
            arrow_skipped_no_csver: false,
            promoted: false,
        });
    }

    let mut out = BytesMut::new();
    logs.encode(&mut out)
        .map_err(|e| processor_error(processor, format!("failed to encode OTLP logs: {e}")))?;
    Ok(PayloadPromotion {
        payload: OtlpProtoBytes::ExportLogsRequest(out.freeze()).into(),
        stats,
        arrow_skipped_no_csver: false,
        promoted: true,
    })
}

fn promote_microsoft_common_schema_logs(logs: &mut LogsData) -> PromotionStats {
    let mut stats = PromotionStats::default();
    for resource_logs in &mut logs.resource_logs {
        for scope_logs in &mut resource_logs.scope_logs {
            for log in &mut scope_logs.log_records {
                stats.records_seen = stats.records_seen.saturating_add(1);
                if promote_microsoft_common_schema_log(log) {
                    stats.records_promoted = stats.records_promoted.saturating_add(1);
                }
            }
        }
    }
    stats
}

fn arrow_logs_have_no_csver(records: &OtapArrowRecords) -> bool {
    !arrow_log_attrs_have_key(records, "__csver__")
}

fn arrow_log_attrs_have_key(records: &OtapArrowRecords, target: &str) -> bool {
    let Some(attrs) = records.get(ArrowPayloadType::LogAttrs) else {
        return false;
    };
    record_batch_has_attr_key(attrs, target)
}

fn record_batch_has_attr_key(attrs: &RecordBatch, target: &str) -> bool {
    let Some(key_col) = attrs.column_by_name(consts::ATTRIBUTE_KEY) else {
        return false;
    };

    if let Some(keys) = key_col.as_any().downcast_ref::<StringArray>() {
        return (0..keys.len()).any(|index| keys.is_valid(index) && keys.value(index) == target);
    }

    if let Some(keys) = key_col
        .as_any()
        .downcast_ref::<DictionaryArray<UInt8Type>>()
    {
        let Some(values) = keys.values().as_any().downcast_ref::<StringArray>() else {
            return false;
        };
        return (0..keys.len()).any(|index| {
            if !keys.is_valid(index) {
                return false;
            }
            let value_index = keys.keys().value(index) as usize;
            values.is_valid(value_index) && values.value(value_index) == target
        });
    }

    if let Some(keys) = key_col
        .as_any()
        .downcast_ref::<DictionaryArray<UInt16Type>>()
    {
        let Some(values) = keys.values().as_any().downcast_ref::<StringArray>() else {
            return false;
        };
        return (0..keys.len()).any(|index| {
            if !keys.is_valid(index) {
                return false;
            }
            let value_index = keys.keys().value(index) as usize;
            values.is_valid(value_index) && values.value(value_index) == target
        });
    }

    false
}

fn promote_microsoft_common_schema_log(log: &mut LogRecord) -> bool {
    let Some(csver) = find_attr_value(&log.attributes, "__csver__").and_then(any_int) else {
        return false;
    };
    if csver != 0x400 {
        return false;
    }
    if find_attr_value(&log.attributes, "PartB._typeName").and_then(any_str) != Some("Log") {
        return false;
    }

    let mut promoted = Vec::with_capacity(log.attributes.len());
    let mut part_a_name = None;
    let mut part_b_name = None;
    for attr in log.attributes.drain(..) {
        match attr.key.as_str() {
            "__csver__" | "PartB._typeName" => {}
            "PartA.time" => {
                if let Some(time) = attr.value.as_ref().and_then(any_str) {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(time) {
                        if let Some(nanos) = dt.timestamp_nanos_opt() {
                            if let Ok(nanos) = u64::try_from(nanos) {
                                log.time_unix_nano = nanos;
                            }
                        }
                    }
                }
            }
            "PartA.name" => {
                if let Some(name) = attr.value.as_ref().and_then(any_str) {
                    if !name.is_empty() {
                        part_a_name = Some(name.to_owned());
                    }
                }
            }
            "PartB.name" => {
                if let Some(name) = attr.value.as_ref().and_then(any_str) {
                    if !name.is_empty() {
                        part_b_name = Some(name.to_owned());
                    }
                }
            }
            "PartA.ext_dt_traceId" => {
                if let Some(trace_id) = attr.value.as_ref().and_then(any_str) {
                    if let Some(bytes) = parse_hex_bytes::<16>(trace_id) {
                        log.trace_id = bytes.to_vec();
                    } else {
                        promoted.push(KeyValue::new("trace.id", AnyValue::new_string(trace_id)));
                    }
                }
            }
            "PartA.ext_dt_spanId" => {
                if let Some(span_id) = attr.value.as_ref().and_then(any_str) {
                    if let Some(bytes) = parse_hex_bytes::<8>(span_id) {
                        log.span_id = bytes.to_vec();
                    } else {
                        promoted.push(KeyValue::new("span.id", AnyValue::new_string(span_id)));
                    }
                }
            }
            "PartA.ext_dt_traceFlags" => {
                if let Some(flags) = attr.value.as_ref().and_then(any_int) {
                    log.flags = u32::try_from(flags).unwrap_or(u32::MAX);
                }
            }
            "PartA.ext_cloud_role" => {
                if let Some(service_name) = attr.value.as_ref().and_then(any_str) {
                    if !service_name.is_empty() {
                        promoted.push(KeyValue::new(
                            "service.name",
                            AnyValue::new_string(service_name),
                        ));
                    }
                }
            }
            "PartA.ext_cloud_roleInstance" => {
                if let Some(instance) = attr.value.as_ref().and_then(any_str) {
                    if !instance.is_empty() {
                        promoted.push(KeyValue::new(
                            "service.instance.id",
                            AnyValue::new_string(instance),
                        ));
                    }
                }
            }
            "PartB.body" => {
                log.body = attr.value;
            }
            "PartB.severityNumber" => {
                if let Some(number) = attr.value.as_ref().and_then(any_int) {
                    if let Ok(number) = i32::try_from(number) {
                        if number > 0 {
                            log.severity_number = number.clamp(1, 24);
                        }
                    }
                }
            }
            "PartB.severityText" => {
                if let Some(text) = attr.value.as_ref().and_then(any_str) {
                    log.severity_text = text.to_owned();
                }
            }
            key if key.starts_with("PartC.") => {
                if let Some(value) = attr.value {
                    let key = key.trim_start_matches("PartC.");
                    promoted.push(KeyValue::new(key, value));
                }
            }
            key if key.starts_with("PartA.") || key.starts_with("PartB.") => {
                promoted.push(attr);
            }
            _ => promoted.push(attr),
        }
    }
    if let Some(name) = part_b_name.or(part_a_name) {
        log.event_name = name;
    }
    log.attributes = promoted;
    true
}

fn find_attr_value<'a>(attrs: &'a [KeyValue], key: &str) -> Option<&'a AnyValue> {
    attrs
        .iter()
        .find(|attr| attr.key == key)
        .and_then(|attr| attr.value.as_ref())
}

fn any_str(value: &AnyValue) -> Option<&str> {
    match value.value.as_ref()? {
        any_value::Value::StringValue(value) => Some(value),
        _ => None,
    }
}

fn any_int(value: &AnyValue) -> Option<i64> {
    match value.value.as_ref()? {
        any_value::Value::IntValue(value) => Some(*value),
        _ => None,
    }
}

fn parse_hex_bytes<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N * 2 {
        return None;
    }
    let mut bytes = [0u8; N];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = hex_nibble(chunk[0])?;
        let low = hex_nibble(chunk[1])?;
        bytes[index] = (high << 4) | low;
    }
    Some(bytes)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::node::NodeId;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{ResourceLogs, ScopeLogs};

    fn attr<'a>(log: &'a LogRecord, key: &str) -> Option<&'a AnyValue> {
        find_attr_value(&log.attributes, key)
    }

    fn test_processor_id() -> NodeId {
        NodeId {
            index: 0,
            name: "common_schema".into(),
        }
    }

    fn logs_payload(logs: LogsData) -> OtapPayload {
        let mut bytes = BytesMut::new();
        logs.encode(&mut bytes).expect("encode logs");
        OtlpProtoBytes::ExportLogsRequest(bytes.freeze()).into()
    }

    fn logs_arrow_payload(logs: LogsData) -> OtapPayload {
        let records: OtapArrowRecords = logs_payload(logs).try_into().expect("convert to arrow");
        records.into()
    }

    #[test]
    fn promotes_microsoft_common_schema_fields() {
        let mut log = LogRecord {
            time_unix_nano: 1,
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new(
                    "PartA.time",
                    AnyValue::new_string("2024-06-15T12:00:00+05:30"),
                ),
                KeyValue::new(
                    "PartA.ext_dt_traceId",
                    AnyValue::new_string("0102030405060708090a0b0c0d0e0f10"),
                ),
                KeyValue::new(
                    "PartA.ext_dt_spanId",
                    AnyValue::new_string("a1b2c3d4e5f60718"),
                ),
                KeyValue::new("PartA.ext_dt_traceFlags", AnyValue::new_int(1)),
                KeyValue::new("PartA.ext_cloud_role", AnyValue::new_string("checkout")),
                KeyValue::new("PartB.body", AnyValue::new_string("failed")),
                KeyValue::new("PartB.severityNumber", AnyValue::new_int(17)),
                KeyValue::new("PartB.severityText", AnyValue::new_string("ERROR")),
                KeyValue::new("PartB.name", AnyValue::new_string("CheckoutFailure")),
                KeyValue::new("PartC.status", AnyValue::new_int(500)),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut log));

        let expected_time = DateTime::parse_from_rfc3339("2024-06-15T12:00:00+05:30")
            .expect("valid time")
            .timestamp_nanos_opt()
            .expect("representable timestamp") as u64;
        assert_eq!(log.time_unix_nano, expected_time);
        assert_eq!(log.event_name, "CheckoutFailure");
        assert_eq!(log.severity_number, 17);
        assert_eq!(log.severity_text, "ERROR");
        assert_eq!(log.flags, 1);
        assert_eq!(
            log.trace_id,
            parse_hex_bytes::<16>("0102030405060708090a0b0c0d0e0f10")
                .expect("hex")
                .to_vec()
        );
        assert_eq!(
            log.span_id,
            parse_hex_bytes::<8>("a1b2c3d4e5f60718")
                .expect("hex")
                .to_vec()
        );
        assert_eq!(
            log.body.as_ref().and_then(any_str),
            Some("failed"),
            "PartB.body should become the typed body"
        );
        assert!(find_attr_value(&log.attributes, "__csver__").is_none());
        assert_eq!(
            attr(&log, "service.name").and_then(any_str),
            Some("checkout")
        );
        assert_eq!(attr(&log, "status").and_then(any_int), Some(500));
    }

    #[test]
    fn preserves_unknown_part_a_and_part_b_fields() {
        let mut log = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartA.futureField", AnyValue::new_string("part-a")),
                KeyValue::new("PartB.futureField", AnyValue::new_string("part-b")),
                KeyValue::new("PartB.body", AnyValue::new_string("body")),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut log));
        assert_eq!(
            attr(&log, "PartA.futureField").and_then(any_str),
            Some("part-a")
        );
        assert_eq!(
            attr(&log, "PartB.futureField").and_then(any_str),
            Some("part-b")
        );
        assert!(find_attr_value(&log.attributes, "__csver__").is_none());
        assert!(find_attr_value(&log.attributes, "PartB._typeName").is_none());
    }

    #[test]
    fn leaves_non_microsoft_common_schema_record_unchanged() {
        let original = vec![KeyValue::new("PartB.body", AnyValue::new_string("not cs"))];
        let mut log = LogRecord {
            attributes: original.clone(),
            ..Default::default()
        };

        assert!(!promote_microsoft_common_schema_log(&mut log));
        assert_eq!(log.attributes, original);
        assert!(log.body.is_none());
    }

    #[test]
    fn rejects_wrong_microsoft_common_schema_version() {
        let original = vec![
            KeyValue::new("__csver__", AnyValue::new_int(0x300)),
            KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
            KeyValue::new("PartB.body", AnyValue::new_string("body")),
        ];
        let mut log = LogRecord {
            attributes: original.clone(),
            ..Default::default()
        };

        assert!(!promote_microsoft_common_schema_log(&mut log));
        assert_eq!(log.attributes, original);
        assert!(log.body.is_none());
    }

    #[test]
    fn rejects_missing_or_non_log_type_name() {
        let mut missing_type = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB.body", AnyValue::new_string("body")),
            ],
            ..Default::default()
        };
        let mut wrong_type = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Span")),
                KeyValue::new("PartB.body", AnyValue::new_string("body")),
            ],
            ..Default::default()
        };

        assert!(!promote_microsoft_common_schema_log(&mut missing_type));
        assert!(!promote_microsoft_common_schema_log(&mut wrong_type));
        assert!(missing_type.body.is_none());
        assert!(wrong_type.body.is_none());
    }

    #[test]
    fn malformed_time_preserves_existing_timestamp() {
        let mut log = LogRecord {
            time_unix_nano: 123,
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartA.time", AnyValue::new_string("not-a-time")),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut log));
        assert_eq!(log.time_unix_nano, 123);
    }

    #[test]
    fn pre_epoch_time_preserves_existing_timestamp() {
        let mut log = LogRecord {
            time_unix_nano: 123,
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartA.time", AnyValue::new_string("1969-12-31T23:59:59Z")),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut log));
        assert_eq!(log.time_unix_nano, 123);
    }

    #[test]
    fn malformed_trace_and_span_ids_fall_back_to_attributes() {
        let mut log = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartA.ext_dt_traceId", AnyValue::new_string("short")),
                KeyValue::new("PartA.ext_dt_spanId", AnyValue::new_string("not-hex!")),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut log));
        assert!(log.trace_id.is_empty());
        assert!(log.span_id.is_empty());
        assert_eq!(attr(&log, "trace.id").and_then(any_str), Some("short"));
        assert_eq!(attr(&log, "span.id").and_then(any_str), Some("not-hex!"));
    }

    #[test]
    fn part_b_name_takes_precedence_over_part_a_name() {
        let mut log = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartB.name", AnyValue::new_string("PartBName")),
                KeyValue::new("PartA.name", AnyValue::new_string("PartAName")),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut log));
        assert_eq!(log.event_name, "PartBName");
    }

    #[test]
    fn preserves_part_b_event_id_with_common_schema_name() {
        let mut log = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartC.eventId", AnyValue::new_int(1)),
                KeyValue::new("PartB.eventId", AnyValue::new_int(2)),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut log));
        assert_eq!(attr(&log, "eventId").and_then(any_int), Some(1));
        assert_eq!(attr(&log, "PartB.eventId").and_then(any_int), Some(2));
    }

    #[test]
    fn positive_severity_number_is_clamped_to_otlp_range() {
        let mut high = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartB.severityNumber", AnyValue::new_int(99)),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut high));
        assert_eq!(high.severity_number, 24);
    }

    #[test]
    fn non_positive_severity_preserves_unspecified() {
        let mut zero = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartB.severityNumber", AnyValue::new_int(0)),
            ],
            ..Default::default()
        };
        let mut negative = LogRecord {
            attributes: vec![
                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                KeyValue::new("PartB.severityNumber", AnyValue::new_int(-10)),
            ],
            ..Default::default()
        };

        assert!(promote_microsoft_common_schema_log(&mut zero));
        assert!(promote_microsoft_common_schema_log(&mut negative));
        assert_eq!(zero.severity_number, 0);
        assert_eq!(negative.severity_number, 0);
    }

    #[test]
    fn promote_logs_reports_whether_any_record_changed() {
        let mut logs = LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![
                        LogRecord {
                            attributes: vec![KeyValue::new(
                                "PartB.body",
                                AnyValue::new_string("not cs"),
                            )],
                            ..Default::default()
                        },
                        LogRecord {
                            attributes: vec![
                                KeyValue::new("__csver__", AnyValue::new_int(0x400)),
                                KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                                KeyValue::new("PartB.body", AnyValue::new_string("cs")),
                            ],
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let stats = promote_microsoft_common_schema_logs(&mut logs);
        assert!(stats.promoted_any());
        assert_eq!(stats.records_seen, 2);
        assert_eq!(stats.records_promoted, 1);
        assert_eq!(stats.records_skipped_not_common_schema(), 1);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records[0].attributes[0].key, "PartB.body");
        assert_eq!(records[1].body.as_ref().and_then(any_str), Some("cs"));

        let mut non_cs = LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue::new(
                            "PartB.body",
                            AnyValue::new_string("not cs"),
                        )],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let stats = promote_microsoft_common_schema_logs(&mut non_cs);
        assert!(!stats.promoted_any());
        assert_eq!(stats.records_seen, 1);
        assert_eq!(stats.records_promoted, 0);
        assert_eq!(stats.records_skipped_not_common_schema(), 1);
    }

    #[test]
    fn arrow_payload_with_csver_but_no_promoted_records_stays_arrow() {
        let payload = logs_arrow_payload(LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        attributes: vec![
                            KeyValue::new("__csver__", AnyValue::new_int(0x300)),
                            KeyValue::new("PartB._typeName", AnyValue::new_string("Log")),
                            KeyValue::new("PartB.body", AnyValue::new_string("unsupported")),
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });

        let result = promote_microsoft_common_schema_payload(payload, test_processor_id())
            .expect("payload promotion");

        assert!(!result.promoted);
        assert!(!result.arrow_skipped_no_csver);
        assert_eq!(result.stats.records_seen, 1);
        assert_eq!(result.stats.records_promoted, 0);
        assert!(matches!(result.payload, OtapPayload::OtapArrowRecords(_)));
    }

    #[test]
    fn record_batch_key_probe_finds_plain_and_dictionary_keys() {
        use arrow::array::ArrayRef;
        use arrow::datatypes::DataType;
        use arrow::datatypes::Field;
        use arrow::datatypes::Schema;

        let plain = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ATTRIBUTE_KEY,
                DataType::Utf8,
                false,
            )])),
            vec![Arc::new(StringArray::from(vec!["other", "__csver__"])) as ArrayRef],
        )
        .expect("plain key batch");
        assert!(record_batch_has_attr_key(&plain, "__csver__"));
        assert!(!record_batch_has_attr_key(&plain, "missing"));

        let dict: DictionaryArray<UInt16Type> = vec!["other", "__csver__"].into_iter().collect();
        let dictionary = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ATTRIBUTE_KEY,
                dict.data_type().clone(),
                false,
            )])),
            vec![Arc::new(dict) as ArrayRef],
        )
        .expect("dictionary key batch");
        assert!(record_batch_has_attr_key(&dictionary, "__csver__"));
    }
}
