// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kafka header extraction and injection into telemetry payloads.
//!
//! Scans Kafka message headers once and builds format-specific representations
//! (`HeaderExtractions`) that downstream OTLP and OTAP paths can apply to
//! resource attributes for all signal types (traces, metrics, logs).

use super::config::{AttributeValueType, HeaderExtraction};
use bytes::Bytes;
use otap_df_engine::error::Error as EngineError;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::Consumer as PdataConsumer;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::otap::transform::{
    AttributesTransform, LiteralValue, UpsertTransform, apply_attribute_transform,
};
use otap_df_pdata::otap::{OtapArrowRecords, from_record_messages};
use otap_df_pdata::proto::opentelemetry::arrow::v1::{ArrowPayloadType, BatchArrowRecords};
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use otap_df_telemetry::otel_error;
use prost::Message;
use rdkafka::Message as _;
use rdkafka::message::{BorrowedMessage, Headers};
use std::collections::{BTreeMap, HashMap};

/// Pre-collected header values with format-specific attribute representations.
///
/// Built via [`HeaderExtractions::otlp`] or
/// [`HeaderExtractions::otap`] depending on the message format.
/// Each constructor pre-builds the attribute representation that its
/// downstream consumer needs, avoiding per-resource or per-message conversion.
pub(crate) struct HeaderExtractions {
    /// Pre-built protobuf `KeyValue` pairs for the OTLP path.
    otlp_attributes: Option<Vec<KeyValue>>,
    /// Pre-built Arrow attribute transform for the OTAP path.
    otap_attributes: Option<AttributesTransform>,
}

impl HeaderExtractions {
    /// Returns `true` if at least one extraction was found.
    pub(crate) fn has_any(&self) -> bool {
        self.otlp_attributes.is_some() || self.otap_attributes.is_some()
    }

    /// Build extractions for the OTLP protobuf path.
    ///
    /// Scans Kafka message headers once, building `KeyValue` pairs with
    /// `AnyValue` directly so they can be inserted into resource attributes
    /// without any intermediate conversion.
    pub(crate) fn otlp(
        kafka_message: &BorrowedMessage<'_>,
        extractors: &HashMap<String, HeaderExtraction>,
    ) -> Self {
        let mut kvs: Vec<KeyValue> = Vec::new();

        let Some(headers) = kafka_message.headers() else {
            return Self {
                otlp_attributes: None,
                otap_attributes: None,
            };
        };

        for header in headers.iter() {
            let Some(extraction) = extractors.get(header.key) else {
                continue;
            };
            let Some(raw) = header.value else { continue };

            if let Some(value) =
                parse_any_value(header.key, &extraction.key, raw, &extraction.value_type)
            {
                kvs.push(KeyValue {
                    key: extraction.key.clone(),
                    value: Some(AnyValue { value: Some(value) }),
                });
            }
        }

        let otlp_attributes = if kvs.is_empty() { None } else { Some(kvs) };
        Self {
            otlp_attributes,
            otap_attributes: None,
        }
    }

    /// Build extractions for the OTAP Arrow path.
    ///
    /// Scans Kafka message headers once, building an [`AttributesTransform`]
    /// containing an [`UpsertTransform`] so that `apply_attribute_transform`
    /// can be called directly without any per-message allocation.
    pub(crate) fn otap(
        kafka_message: &BorrowedMessage<'_>,
        extractors: &HashMap<String, HeaderExtraction>,
    ) -> Self {
        let mut entries: BTreeMap<String, LiteralValue> = BTreeMap::new();

        let Some(headers) = kafka_message.headers() else {
            return Self {
                otlp_attributes: None,
                otap_attributes: None,
            };
        };

        for header in headers.iter() {
            let Some(extraction) = extractors.get(header.key) else {
                continue;
            };
            let Some(raw) = header.value else { continue };

            if let Some(literal) =
                parse_literal_value(header.key, &extraction.key, raw, &extraction.value_type)
            {
                let _ = entries.insert(extraction.key.clone(), literal);
            }
        }

        let otap_attributes = if entries.is_empty() {
            None
        } else {
            Some(AttributesTransform::default().with_upsert(UpsertTransform::new(entries)))
        };
        Self {
            otlp_attributes: None,
            otap_attributes,
        }
    }

    /// Apply header extractions to an OTLP protobuf traces payload.
    ///
    /// Deserializes the `ExportTraceServiceRequest`, injects attributes into
    /// the resource attributes of each `ResourceSpans`, then re-serializes.
    pub(crate) fn apply_otlp_traces(&self, data: &[u8]) -> Result<OtapPdata, EngineError> {
        let mut request = ExportTraceServiceRequest::decode(data).map_err(|e| {
            EngineError::PdataConversionError {
                error: format!(
                    "Failed to decode ExportTraceServiceRequest for header extraction: {e}"
                ),
            }
        })?;

        if let Some(ref kvs) = self.otlp_attributes {
            for resource_spans in &mut request.resource_spans {
                upsert_resource_attributes(
                    resource_spans
                        .resource
                        .get_or_insert_with(Resource::default),
                    kvs,
                );
            }
        }

        // Re-encode to protobuf bytes
        let mut buf = Vec::with_capacity(request.encoded_len());
        request
            .encode(&mut buf)
            .map_err(|e| EngineError::PdataConversionError {
                error: format!(
                    "Failed to re-encode ExportTraceServiceRequest after header extraction: {e}"
                ),
            })?;

        Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportTracesRequest(Bytes::from(buf)).into(),
        ))
    }

    /// Apply header extractions to an OTLP protobuf metrics payload.
    ///
    /// Deserializes the `ExportMetricsServiceRequest`, injects attributes into
    /// the resource attributes of each `ResourceMetrics`, then re-serializes.
    pub(crate) fn apply_otlp_metrics(&self, data: &[u8]) -> Result<OtapPdata, EngineError> {
        let mut request = ExportMetricsServiceRequest::decode(data).map_err(|e| {
            EngineError::PdataConversionError {
                error: format!(
                    "Failed to decode ExportMetricsServiceRequest for header extraction: {e}"
                ),
            }
        })?;

        if let Some(ref kvs) = self.otlp_attributes {
            for resource_metrics in &mut request.resource_metrics {
                upsert_resource_attributes(
                    resource_metrics
                        .resource
                        .get_or_insert_with(Resource::default),
                    kvs,
                );
            }
        }

        // Re-encode to protobuf bytes
        let mut buf = Vec::with_capacity(request.encoded_len());
        request
            .encode(&mut buf)
            .map_err(|e| EngineError::PdataConversionError {
                error: format!(
                    "Failed to re-encode ExportMetricsServiceRequest after header extraction: {e}"
                ),
            })?;

        Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportMetricsRequest(Bytes::from(buf)).into(),
        ))
    }

    /// Apply header extractions to an OTLP protobuf logs payload.
    ///
    /// Deserializes the `ExportLogsServiceRequest`, injects attributes into
    /// the resource attributes of each `ResourceLogs`, then re-serializes.
    pub(crate) fn apply_otlp_logs(&self, data: &[u8]) -> Result<OtapPdata, EngineError> {
        let mut request = ExportLogsServiceRequest::decode(data).map_err(|e| {
            EngineError::PdataConversionError {
                error: format!(
                    "Failed to decode ExportLogsServiceRequest for header extraction: {e}"
                ),
            }
        })?;

        if let Some(ref kvs) = self.otlp_attributes {
            for resource_logs in &mut request.resource_logs {
                upsert_resource_attributes(
                    resource_logs.resource.get_or_insert_with(Resource::default),
                    kvs,
                );
            }
        }

        // Re-encode to protobuf bytes
        let mut buf = Vec::with_capacity(request.encoded_len());
        request
            .encode(&mut buf)
            .map_err(|e| EngineError::PdataConversionError {
                error: format!(
                    "Failed to re-encode ExportLogsServiceRequest after header extraction: {e}"
                ),
            })?;

        Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(Bytes::from(buf)).into(),
        ))
    }

    /// Apply header extractions to an OTAP Arrow traces payload.
    ///
    /// Injects attributes into `ResourceAttrs` for the traces payload.
    pub(crate) fn apply_otap_traces(&self, data: &[u8]) -> Result<OtapPdata, EngineError> {
        let arrow_records = decode_otap_traces(data)?;
        self.apply_otap_resource_attrs(arrow_records)
    }

    /// Apply header extractions to an OTAP Arrow metrics payload.
    ///
    /// Injects attributes into `ResourceAttrs` for the metrics payload.
    pub(crate) fn apply_otap_metrics(&self, data: &[u8]) -> Result<OtapPdata, EngineError> {
        let arrow_records = decode_otap_metrics(data)?;
        self.apply_otap_resource_attrs(arrow_records)
    }

    /// Apply header extractions to an OTAP Arrow logs payload.
    ///
    /// Injects attributes into `ResourceAttrs` for the logs payload.
    pub(crate) fn apply_otap_logs(&self, data: &[u8]) -> Result<OtapPdata, EngineError> {
        let arrow_records = decode_otap_logs(data)?;
        self.apply_otap_resource_attrs(arrow_records)
    }

    /// Shared OTAP logic: apply attribute transform to `ResourceAttrs`.
    fn apply_otap_resource_attrs(
        &self,
        mut arrow_records: OtapArrowRecords,
    ) -> Result<OtapPdata, EngineError> {
        if let Some(ref transform) = self.otap_attributes {
            let _ = apply_attribute_transform(
                &mut arrow_records,
                ArrowPayloadType::ResourceAttrs,
                transform,
                false,
            )
            .map_err(|e| EngineError::PdataConversionError {
                error: format!("Failed to transform resource attributes: {e}"),
            })?;
        }

        Ok(OtapPdata::new(Context::default(), arrow_records.into()))
    }
}

/// Upsert `KeyValue` pairs into a `Resource`'s attributes.
///
/// For each key-value pair, if an attribute with the same key already exists
/// its value is replaced; otherwise the pair is appended.
fn upsert_resource_attributes(resource: &mut Resource, kvs: &[KeyValue]) {
    for kv in kvs {
        if let Some(existing) = resource.attributes.iter_mut().find(|a| a.key == kv.key) {
            existing.value.clone_from(&kv.value);
        } else {
            resource.attributes.push(kv.clone());
        }
    }
}

/// Decode OTAP Arrow bytes into `OtapArrowRecords::Traces`.
fn decode_otap_traces(data: &[u8]) -> Result<OtapArrowRecords, EngineError> {
    let mut bar =
        BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
            error: e.to_string(),
        })?;
    let mut pdc = PdataConsumer::default();
    let record_messages = pdc.consume_bar(&mut bar)?;
    Ok(OtapArrowRecords::Traces(
        from_record_messages(record_messages).map_err(|e| EngineError::PdataConversionError {
            error: e.to_string(),
        })?,
    ))
}

/// Decode OTAP Arrow bytes into `OtapArrowRecords::Metrics`.
fn decode_otap_metrics(data: &[u8]) -> Result<OtapArrowRecords, EngineError> {
    let mut bar =
        BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
            error: e.to_string(),
        })?;
    let mut pdc = PdataConsumer::default();
    let record_messages = pdc.consume_bar(&mut bar)?;
    Ok(OtapArrowRecords::Metrics(
        from_record_messages(record_messages).map_err(|e| EngineError::PdataConversionError {
            error: e.to_string(),
        })?,
    ))
}

/// Decode OTAP Arrow bytes into `OtapArrowRecords::Logs`.
fn decode_otap_logs(data: &[u8]) -> Result<OtapArrowRecords, EngineError> {
    let mut bar =
        BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
            error: e.to_string(),
        })?;
    let mut pdc = PdataConsumer::default();
    let record_messages = pdc.consume_bar(&mut bar)?;
    Ok(OtapArrowRecords::Logs(
        from_record_messages(record_messages).map_err(|e| EngineError::PdataConversionError {
            error: e.to_string(),
        })?,
    ))
}

/// Parse raw header bytes into an [`any_value::Value`] for the OTLP protobuf path.
/// Returns `None` (logging an error) on invalid UTF-8 or parse failure.
fn parse_any_value(
    header_key: &str,
    attribute_key: &str,
    raw: &[u8],
    value_type: &AttributeValueType,
) -> Option<any_value::Value> {
    let utf8 = match std::str::from_utf8(raw) {
        Ok(s) => s,
        Err(_) => {
            otel_error!(
                "kafka.header.attribute.invalid_utf8",
                header_key = header_key,
                attribute_key = attribute_key,
            );
            return None;
        }
    };

    match value_type {
        AttributeValueType::String => Some(any_value::Value::StringValue(utf8.to_string())),
        AttributeValueType::Int => match utf8.parse::<i64>() {
            Ok(v) => Some(any_value::Value::IntValue(v)),
            Err(_) => {
                otel_error!(
                    "kafka.header.attribute.parse_int_failed",
                    header_key = header_key,
                    attribute_key = attribute_key,
                    raw_value = utf8,
                );
                None
            }
        },
        AttributeValueType::Float => match utf8.parse::<f64>() {
            Ok(v) => Some(any_value::Value::DoubleValue(v)),
            Err(_) => {
                otel_error!(
                    "kafka.header.attribute.parse_float_failed",
                    header_key = header_key,
                    attribute_key = attribute_key,
                    raw_value = utf8,
                );
                None
            }
        },
        AttributeValueType::Bool => match utf8.parse::<bool>() {
            Ok(v) => Some(any_value::Value::BoolValue(v)),
            Err(_) => {
                otel_error!(
                    "kafka.header.attribute.parse_bool_failed",
                    header_key = header_key,
                    attribute_key = attribute_key,
                    raw_value = utf8,
                );
                None
            }
        },
    }
}

/// Parse raw header bytes into a [`LiteralValue`] for the OTAP Arrow path.
/// Returns `None` (logging an error) on invalid UTF-8 or parse failure.
fn parse_literal_value(
    header_key: &str,
    attribute_key: &str,
    raw: &[u8],
    value_type: &AttributeValueType,
) -> Option<LiteralValue> {
    let utf8 = match std::str::from_utf8(raw) {
        Ok(s) => s,
        Err(_) => {
            otel_error!(
                "kafka.header.attribute.invalid_utf8",
                header_key = header_key,
                attribute_key = attribute_key,
            );
            return None;
        }
    };

    match value_type {
        AttributeValueType::String => Some(LiteralValue::Str(utf8.to_string())),
        AttributeValueType::Int => match utf8.parse::<i64>() {
            Ok(v) => Some(LiteralValue::Int(v)),
            Err(_) => {
                otel_error!(
                    "kafka.header.attribute.parse_int_failed",
                    header_key = header_key,
                    attribute_key = attribute_key,
                    raw_value = utf8,
                );
                None
            }
        },
        AttributeValueType::Float => match utf8.parse::<f64>() {
            Ok(v) => Some(LiteralValue::Double(v)),
            Err(_) => {
                otel_error!(
                    "kafka.header.attribute.parse_float_failed",
                    header_key = header_key,
                    attribute_key = attribute_key,
                    raw_value = utf8,
                );
                None
            }
        },
        AttributeValueType::Bool => match utf8.parse::<bool>() {
            Ok(v) => Some(LiteralValue::Bool(v)),
            Err(_) => {
                otel_error!(
                    "kafka.header.attribute.parse_bool_failed",
                    header_key = header_key,
                    attribute_key = attribute_key,
                    raw_value = utf8,
                );
                None
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use otap_df_pdata::otap::transform::UpsertTransform;
    use otap_df_pdata::otap::{OtapArrowRecords, Traces};
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_pdata::proto::opentelemetry::common::v1::{InstrumentationScope, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        Metric, ResourceMetrics, ScopeMetrics, Sum,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use otap_df_pdata::{OtapPayload, OtlpProtoBytes, Producer, TryIntoWithOptions};
    use prost::Message;
    use std::collections::BTreeMap;

    /// Build a string-valued `KeyValue` for test assertions.
    fn string_kv(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
        }
    }

    /// Helper to create a trace request with spans and an existing resource attribute.
    fn create_traces_with_spans() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![string_kv("service.name", "my-service")],
                    ..Default::default()
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope::default()),
                    spans: vec![
                        Span {
                            trace_id: vec![1u8; 16],
                            span_id: vec![1u8; 8],
                            name: "span-1".to_string(),
                            attributes: vec![string_kv("existing", "original")],
                            ..Default::default()
                        },
                        Span {
                            trace_id: vec![2u8; 16],
                            span_id: vec![2u8; 8],
                            name: "span-2".to_string(),
                            attributes: vec![string_kv("existing-2", "original-2")],
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    /// Helper to create a metrics request with an existing resource attribute.
    fn create_metrics_request() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![string_kv("service.name", "metrics-service")],
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope::default()),
                    metrics: vec![Metric {
                        name: "test.counter".to_string(),
                        data: Some(
                            otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data::Sum(
                                Sum::default(),
                            ),
                        ),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    /// Helper to create a logs request with an existing resource attribute.
    fn create_logs_request() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![string_kv("service.name", "logs-service")],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        time_unix_nano: 1,
                        attributes: vec![string_kv("log.attr", "value")],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    /// Create OTAP Arrow wire bytes from the `create_traces_with_spans()` helper.
    fn create_traces_otap_bytes() -> Vec<u8> {
        let request = create_traces_with_spans();
        let mut buf = Vec::new();
        request.encode(&mut buf).expect("encode OTLP request");

        let payload: OtapPayload = OtlpProtoBytes::ExportTracesRequest(Bytes::from(buf)).into();
        let mut otap_records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("convert OTLP to OTAP Arrow");

        arrow_records_to_bytes(&mut otap_records)
    }

    fn arrow_records_to_bytes(arrow_records: &mut OtapArrowRecords) -> Vec<u8> {
        let mut producer = Producer::new();
        let bar = producer
            .produce_bar(arrow_records)
            .expect("failed to get batch arrow records");
        let mut bytes = vec![];
        bar.encode(&mut bytes).expect("failed to encode");
        bytes
    }

    /// Convert an `OtapPdata` (containing OTAP Arrow records) back to an OTLP
    /// `ExportTraceServiceRequest` so tests can assert against familiar protobuf
    /// structs instead of Arrow column internals.
    fn otap_pdata_to_traces(pdata: &mut OtapPdata) -> ExportTraceServiceRequest {
        let otlp: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("OTAP -> OTLP conversion");
        ExportTraceServiceRequest::decode(otlp.as_bytes()).expect("decode OTLP traces")
    }

    // ---- OTLP traces tests ----

    #[test]
    fn apply_otlp_traces_attribute_added_to_resource() {
        let request = create_traces_with_spans();
        let mut bytes = Vec::new();
        request.encode(&mut bytes).unwrap();

        let extractions = HeaderExtractions {
            otlp_attributes: Some(vec![string_kv("tenant.id", "acme-corp")]),
            otap_attributes: None,
        };

        let mut pdata = extractions
            .apply_otlp_traces(&bytes)
            .expect("should succeed");

        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("to OtlpProtoBytes");
        let result = ExportTraceServiceRequest::decode(proto.as_bytes()).expect("decode result");

        for rs in &result.resource_spans {
            let resource = rs.resource.as_ref().expect("should have resource");
            let tenant_attr = resource
                .attributes
                .iter()
                .find(|kv| kv.key == "tenant.id")
                .expect("resource should have tenant.id attribute");
            let value = tenant_attr
                .value
                .as_ref()
                .expect("should have value")
                .value
                .as_ref()
                .expect("should have inner value");
            assert!(
                matches!(value, any_value::Value::StringValue(s) if s == "acme-corp"),
                "resource attribute value should be 'acme-corp'"
            );

            // Original resource attribute preserved
            assert!(
                resource
                    .attributes
                    .iter()
                    .any(|kv| kv.key == "service.name"),
                "original service.name resource attribute should be preserved"
            );
        }

        // Span attributes should be untouched
        let span1 = &result.resource_spans[0].scope_spans[0].spans[0];
        assert!(
            !span1.attributes.iter().any(|kv| kv.key == "tenant.id"),
            "span should NOT have tenant.id attribute"
        );
        assert!(
            span1.attributes.iter().any(|kv| kv.key == "existing"),
            "original span attributes should be preserved"
        );
    }

    #[test]
    fn apply_otlp_traces_multiple_attributes() {
        let request = create_traces_with_spans();
        let mut bytes = Vec::new();
        request.encode(&mut bytes).unwrap();

        let extractions = HeaderExtractions {
            otlp_attributes: Some(vec![
                string_kv("tenant.id", "acme"),
                string_kv("env", "prod"),
            ]),
            otap_attributes: None,
        };

        let mut pdata = extractions
            .apply_otlp_traces(&bytes)
            .expect("should succeed");

        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("to OtlpProtoBytes");
        let result = ExportTraceServiceRequest::decode(proto.as_bytes()).expect("decode result");

        for rs in &result.resource_spans {
            let resource = rs.resource.as_ref().expect("should have resource");
            assert!(resource.attributes.iter().any(|kv| kv.key == "tenant.id"));
            assert!(resource.attributes.iter().any(|kv| kv.key == "env"));
        }
    }

    #[test]
    fn apply_otlp_traces_upserts_existing_resource_attribute() {
        let request = create_traces_with_spans();
        let mut bytes = Vec::new();
        request.encode(&mut bytes).unwrap();

        // Upsert service.name which already exists on the resource
        let extractions = HeaderExtractions {
            otlp_attributes: Some(vec![string_kv("service.name", "overridden-service")]),
            otap_attributes: None,
        };

        let mut pdata = extractions
            .apply_otlp_traces(&bytes)
            .expect("should succeed");

        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("to OtlpProtoBytes");
        let result = ExportTraceServiceRequest::decode(proto.as_bytes()).expect("decode result");

        let resource = result.resource_spans[0]
            .resource
            .as_ref()
            .expect("should have resource");
        let svc_attrs: Vec<_> = resource
            .attributes
            .iter()
            .filter(|kv| kv.key == "service.name")
            .collect();
        assert_eq!(svc_attrs.len(), 1, "should not duplicate the key");
        let value = svc_attrs[0].value.as_ref().unwrap().value.as_ref().unwrap();
        assert!(
            matches!(value, any_value::Value::StringValue(s) if s == "overridden-service"),
            "service.name should be overridden"
        );
    }

    #[test]
    fn apply_otlp_traces_empty_request_does_not_fail() {
        let request = ExportTraceServiceRequest {
            resource_spans: vec![],
        };
        let mut bytes = Vec::new();
        request.encode(&mut bytes).unwrap();

        let extractions = HeaderExtractions {
            otlp_attributes: Some(vec![string_kv("key", "value")]),
            otap_attributes: None,
        };

        let result = extractions.apply_otlp_traces(&bytes);
        assert!(result.is_ok(), "should succeed even with empty request");
    }

    // ---- OTLP metrics tests ----

    #[test]
    fn apply_otlp_metrics_attribute_added_to_resource() {
        let request = create_metrics_request();
        let mut bytes = Vec::new();
        request.encode(&mut bytes).unwrap();

        let extractions = HeaderExtractions {
            otlp_attributes: Some(vec![string_kv("tenant.id", "acme-corp")]),
            otap_attributes: None,
        };

        let mut pdata = extractions
            .apply_otlp_metrics(&bytes)
            .expect("should succeed");

        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("to OtlpProtoBytes");
        let result = ExportMetricsServiceRequest::decode(proto.as_bytes()).expect("decode result");

        for rm in &result.resource_metrics {
            let resource = rm.resource.as_ref().expect("should have resource");
            assert!(
                resource.attributes.iter().any(|kv| kv.key == "tenant.id"),
                "resource should have tenant.id attribute"
            );
            assert!(
                resource
                    .attributes
                    .iter()
                    .any(|kv| kv.key == "service.name"),
                "original service.name should be preserved"
            );
        }
    }

    // ---- OTLP logs tests ----

    #[test]
    fn apply_otlp_logs_attribute_added_to_resource() {
        let request = create_logs_request();
        let mut bytes = Vec::new();
        request.encode(&mut bytes).unwrap();

        let extractions = HeaderExtractions {
            otlp_attributes: Some(vec![string_kv("tenant.id", "acme-corp")]),
            otap_attributes: None,
        };

        let mut pdata = extractions.apply_otlp_logs(&bytes).expect("should succeed");

        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("to OtlpProtoBytes");
        let result = ExportLogsServiceRequest::decode(proto.as_bytes()).expect("decode result");

        for rl in &result.resource_logs {
            let resource = rl.resource.as_ref().expect("should have resource");
            assert!(
                resource.attributes.iter().any(|kv| kv.key == "tenant.id"),
                "resource should have tenant.id attribute"
            );
            assert!(
                resource
                    .attributes
                    .iter()
                    .any(|kv| kv.key == "service.name"),
                "original service.name should be preserved"
            );

            // Log record attributes should be untouched
            for sl in &rl.scope_logs {
                for lr in &sl.log_records {
                    assert!(
                        !lr.attributes.iter().any(|kv| kv.key == "tenant.id"),
                        "log record should NOT have tenant.id attribute"
                    );
                }
            }
        }
    }

    // ---- has_any tests ----

    #[test]
    fn header_extractions_has_any() {
        let empty = HeaderExtractions {
            otlp_attributes: None,
            otap_attributes: None,
        };
        assert!(!empty.has_any());

        let with_otlp_attrs = HeaderExtractions {
            otlp_attributes: Some(vec![string_kv("k", "v")]),
            otap_attributes: None,
        };
        assert!(with_otlp_attrs.has_any());

        let with_otap_attributes = HeaderExtractions {
            otlp_attributes: None,
            otap_attributes: Some(AttributesTransform::default().with_upsert(
                UpsertTransform::new(BTreeMap::from([(
                    "k".to_string(),
                    LiteralValue::Str("v".to_string()),
                )])),
            )),
        };
        assert!(with_otap_attributes.has_any());
    }

    // ---- OTAP Arrow traces tests ----

    #[test]
    fn apply_otap_traces_empty_does_not_fail() {
        let otap_bytes = arrow_records_to_bytes(&mut OtapArrowRecords::Traces(Traces::default()));

        let extractions = HeaderExtractions {
            otlp_attributes: None,
            otap_attributes: Some(AttributesTransform::default().with_upsert(
                UpsertTransform::new(BTreeMap::from([(
                    "tenant.id".to_string(),
                    LiteralValue::Str("acme".to_string()),
                )])),
            )),
        };

        let result = extractions.apply_otap_traces(&otap_bytes);
        assert!(result.is_ok(), "should handle empty OTAP traces gracefully");
    }

    #[test]
    fn apply_otap_traces_attribute_added_to_resource() {
        let otap_bytes = create_traces_otap_bytes();

        let extractions = HeaderExtractions {
            otlp_attributes: None,
            otap_attributes: Some(AttributesTransform::default().with_upsert(
                UpsertTransform::new(BTreeMap::from([(
                    "tenant.id".to_string(),
                    LiteralValue::Str("acme-corp".to_string()),
                )])),
            )),
        };

        let mut pdata = extractions
            .apply_otap_traces(&otap_bytes)
            .expect("should succeed");

        let result = otap_pdata_to_traces(&mut pdata);

        for rs in &result.resource_spans {
            let resource = rs.resource.as_ref().expect("should have resource");
            let tenant_attr = resource
                .attributes
                .iter()
                .find(|kv| kv.key == "tenant.id")
                .expect("resource should have tenant.id attribute");
            let value = tenant_attr
                .value
                .as_ref()
                .expect("should have value")
                .value
                .as_ref()
                .expect("should have inner value");
            assert!(
                matches!(value, any_value::Value::StringValue(s) if s == "acme-corp"),
                "resource tenant.id should be 'acme-corp'",
            );
        }

        // Original span attributes should be preserved
        let span1 = &result.resource_spans[0].scope_spans[0].spans[0];
        assert!(
            span1.attributes.iter().any(|kv| kv.key == "existing"),
            "original span 'existing' attribute should be preserved"
        );
    }

    #[test]
    fn apply_otap_traces_multiple_attributes() {
        let otap_bytes = create_traces_otap_bytes();

        let extractions = HeaderExtractions {
            otlp_attributes: None,
            otap_attributes: Some(AttributesTransform::default().with_upsert(
                UpsertTransform::new(BTreeMap::from([
                    (
                        "tenant.id".to_string(),
                        LiteralValue::Str("acme".to_string()),
                    ),
                    ("env".to_string(), LiteralValue::Str("prod".to_string())),
                ])),
            )),
        };

        let mut pdata = extractions
            .apply_otap_traces(&otap_bytes)
            .expect("should succeed");

        let result = otap_pdata_to_traces(&mut pdata);

        for rs in &result.resource_spans {
            let resource = rs.resource.as_ref().expect("should have resource");
            assert!(
                resource.attributes.iter().any(|kv| kv.key == "tenant.id"),
                "resource should have tenant.id attribute",
            );
            assert!(
                resource.attributes.iter().any(|kv| kv.key == "env"),
                "resource should have env attribute",
            );
        }
    }
}
