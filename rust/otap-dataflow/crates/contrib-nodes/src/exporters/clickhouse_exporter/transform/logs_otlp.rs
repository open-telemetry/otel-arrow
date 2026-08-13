// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct transformation of serialized OTLP logs into ClickHouse Arrow columns.

use std::sync::Arc;

use arrow::array::{
    ArrayRef, MapBuilder, RecordBatch, StringBuilder, TimestampNanosecondBuilder, UInt8Builder,
};
use arrow::datatypes::{DataType, Field, Schema};
use base64::Engine;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, ValueType,
};
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use serde::Serialize;
use serde::ser::{Error as SerError, SerializeMap, SerializeSeq, Serializer};

use crate::exporters::clickhouse_exporter::consts as ch_consts;
use crate::exporters::clickhouse_exporter::error::ClickhouseExporterError;

const SERVICE_NAME_KEY: &[u8] = b"service.name";
const INITIAL_STRING_BYTES_PER_ROW: usize = 16;
const MAX_PREALLOCATED_ROWS: usize = 16 * 1024;

fn bounded_row_capacity(row_count: usize) -> usize {
    row_count.min(MAX_PREALLOCATED_ROWS)
}

/// Direct OTLP logs transformer with reusable sizing and serialization state.
#[derive(Default)]
pub(crate) struct OtlpLogsTransformer {
    cached_schema: Option<(Vec<(&'static str, DataType)>, Arc<Schema>)>,
    json_scratch: Vec<u8>,
    previous_row_count: usize,
}

impl OtlpLogsTransformer {
    /// Transform serialized `ExportLogsServiceRequest` bytes directly to a ClickHouse batch.
    pub(crate) fn transform(
        &mut self,
        request: &[u8],
    ) -> Result<Option<RecordBatch>, ClickhouseExporterError> {
        let logs = RawLogsData::try_new(request)?;
        let mut builders = LogsBuilders::new(self.previous_row_count);
        let mut presence = ColumnPresence::default();
        let mut row_count = 0;

        for resource_logs in logs.resources() {
            let resource_schema_url = optional_utf8(resource_logs.schema_url())?;
            let (resource_attributes, service_name) = match resource_logs.resource() {
                Some(resource) => collect_resource_attributes(resource, &mut self.json_scratch)?,
                None => (Vec::new(), String::new()),
            };
            presence.resource_schema_url |= resource_schema_url.is_some();
            presence.resource_attributes |= !resource_attributes.is_empty();

            for scope_logs in resource_logs.scopes() {
                let (scope_name, scope_version, scope_attributes) = match scope_logs.scope() {
                    Some(scope) => {
                        let name = optional_utf8(scope.name())?;
                        let version = optional_utf8(scope.version())?;
                        let attributes =
                            collect_attributes(scope.attributes(), &mut self.json_scratch)?;
                        (name, version, attributes)
                    }
                    None => (None, None, Vec::new()),
                };
                presence.scope_name |= scope_name.is_some();
                presence.scope_version |= scope_version.is_some();
                presence.scope_attributes |= !scope_attributes.is_empty();

                for log in scope_logs.log_records() {
                    row_count += 1;
                    builders
                        .timestamp
                        .append_value(log.time_unix_nano().unwrap_or(0) as i64);
                    append_optional_string(
                        &mut builders.resource_schema_url,
                        resource_schema_url.as_deref(),
                    );
                    append_attribute_map(&mut builders.resource_attributes, &resource_attributes)?;
                    builders.service_name.append_value(&service_name);
                    append_optional_string(&mut builders.scope_name, scope_name.as_deref());
                    append_optional_string(&mut builders.scope_version, scope_version.as_deref());
                    append_attribute_map(&mut builders.scope_attributes, &scope_attributes)?;

                    let mut has_log_attributes = false;
                    for attribute in log.attributes() {
                        append_attribute(
                            &mut builders.log_attributes,
                            attribute,
                            &mut self.json_scratch,
                        )?;
                        has_log_attributes = true;
                    }
                    builders.log_attributes.append(true)?;
                    presence.log_attributes |= has_log_attributes;

                    match log.body() {
                        Some(body) if body.value_type() != ValueType::Empty => {
                            stringify_any_value(body, &mut self.json_scratch)?;
                            builders.body.append_value(utf8(&self.json_scratch)?);
                            presence.body = true;
                        }
                        _ => builders.body.append_null(),
                    }

                    append_optional_bytes_as_string(
                        &mut builders.event_name,
                        log.event_name(),
                        &mut presence.event_name,
                    )?;
                    append_optional_bytes_as_string(
                        &mut builders.severity_text,
                        log.severity_text(),
                        &mut presence.severity_text,
                    )?;
                    match log.severity_number() {
                        Some(value) => {
                            let value = u8::try_from(value).map_err(|_| {
                                ClickhouseExporterError::CoercionError {
                                    error: format!(
                                        "OTLP log severity number {value} cannot be represented as UInt8"
                                    ),
                                }
                            })?;
                            builders.severity_number.append_value(value);
                            presence.severity_number = true;
                        }
                        None => builders.severity_number.append_null(),
                    }

                    append_optional_hex(
                        &mut builders.span_id,
                        log.span_id().map(|id| id.as_slice()),
                        &mut presence.span_id,
                        &mut self.json_scratch,
                    );
                    append_optional_hex(
                        &mut builders.trace_id,
                        log.trace_id().map(|id| id.as_slice()),
                        &mut presence.trace_id,
                        &mut self.json_scratch,
                    );
                }
            }
        }

        self.previous_row_count = row_count;
        if row_count == 0 {
            return Ok(None);
        }

        let mut columns = builders.finish(presence);
        columns.sort_unstable_by_key(|(name, _)| *name);
        let schema = self.schema_for(&columns);
        let arrays = columns.into_iter().map(|(_, array)| array).collect();
        Ok(Some(RecordBatch::try_new(schema, arrays)?))
    }

    fn schema_for(&mut self, columns: &[(&'static str, ArrayRef)]) -> Arc<Schema> {
        let key = columns
            .iter()
            .map(|(name, array)| (*name, array.data_type().clone()))
            .collect::<Vec<_>>();
        if let Some((cached_key, schema)) = &self.cached_schema
            && cached_key == &key
        {
            return schema.clone();
        }

        let fields = key
            .iter()
            .map(|(name, data_type)| Field::new(*name, data_type.clone(), true))
            .collect::<Vec<_>>();
        let schema = Arc::new(Schema::new(fields));
        self.cached_schema = Some((key, schema.clone()));
        schema
    }
}

#[derive(Default)]
struct ColumnPresence {
    body: bool,
    event_name: bool,
    log_attributes: bool,
    resource_attributes: bool,
    resource_schema_url: bool,
    scope_attributes: bool,
    scope_name: bool,
    scope_version: bool,
    severity_number: bool,
    severity_text: bool,
    span_id: bool,
    trace_id: bool,
}

struct LogsBuilders {
    body: StringBuilder,
    event_name: StringBuilder,
    log_attributes: MapBuilder<StringBuilder, StringBuilder>,
    resource_attributes: MapBuilder<StringBuilder, StringBuilder>,
    resource_schema_url: StringBuilder,
    scope_attributes: MapBuilder<StringBuilder, StringBuilder>,
    scope_name: StringBuilder,
    scope_version: StringBuilder,
    service_name: StringBuilder,
    severity_number: UInt8Builder,
    severity_text: StringBuilder,
    span_id: StringBuilder,
    timestamp: TimestampNanosecondBuilder,
    trace_id: StringBuilder,
}

impl LogsBuilders {
    fn new(row_capacity: usize) -> Self {
        let row_capacity = bounded_row_capacity(row_capacity);
        let string_capacity = row_capacity.saturating_mul(INITIAL_STRING_BYTES_PER_ROW);
        let string_builder = || StringBuilder::with_capacity(row_capacity, string_capacity);
        let map_builder = || {
            MapBuilder::with_capacity(
                None,
                StringBuilder::new(),
                StringBuilder::new(),
                row_capacity,
            )
        };
        Self {
            body: string_builder(),
            event_name: string_builder(),
            log_attributes: map_builder(),
            resource_attributes: map_builder(),
            resource_schema_url: string_builder(),
            scope_attributes: map_builder(),
            scope_name: string_builder(),
            scope_version: string_builder(),
            service_name: string_builder(),
            severity_number: UInt8Builder::with_capacity(row_capacity),
            severity_text: string_builder(),
            span_id: string_builder(),
            timestamp: TimestampNanosecondBuilder::with_capacity(row_capacity),
            trace_id: string_builder(),
        }
    }

    fn finish(mut self, presence: ColumnPresence) -> Vec<(&'static str, ArrayRef)> {
        let mut columns = Vec::with_capacity(14);
        if presence.body {
            columns.push((ch_consts::CH_BODY, Arc::new(self.body.finish()) as ArrayRef));
        }
        if presence.event_name {
            columns.push((
                ch_consts::CH_EVENT_NAME,
                Arc::new(self.event_name.finish()) as ArrayRef,
            ));
        }
        if presence.log_attributes {
            columns.push((
                ch_consts::CH_LOG_ATTRIBUTES,
                Arc::new(self.log_attributes.finish()) as ArrayRef,
            ));
        }
        if presence.resource_attributes {
            columns.push((
                ch_consts::CH_RESOURCE_ATTRIBUTES,
                Arc::new(self.resource_attributes.finish()) as ArrayRef,
            ));
            columns.push((
                ch_consts::CH_SERVICE_NAME,
                Arc::new(self.service_name.finish()) as ArrayRef,
            ));
        }
        if presence.resource_schema_url {
            columns.push((
                ch_consts::CH_RESOURCE_SCHEMA_URL,
                Arc::new(self.resource_schema_url.finish()) as ArrayRef,
            ));
        }
        if presence.scope_attributes {
            columns.push((
                ch_consts::CH_SCOPE_ATTRIBUTES,
                Arc::new(self.scope_attributes.finish()) as ArrayRef,
            ));
        }
        if presence.scope_name {
            columns.push((
                ch_consts::CH_SCOPE_NAME,
                Arc::new(self.scope_name.finish()) as ArrayRef,
            ));
        }
        if presence.scope_version {
            columns.push((
                ch_consts::CH_SCOPE_VERSION,
                Arc::new(self.scope_version.finish()) as ArrayRef,
            ));
        }
        if presence.severity_number {
            columns.push((
                ch_consts::CH_SEVERITY_NUMBER,
                Arc::new(self.severity_number.finish()) as ArrayRef,
            ));
        }
        if presence.severity_text {
            columns.push((
                ch_consts::CH_SEVERITY_TEXT,
                Arc::new(self.severity_text.finish()) as ArrayRef,
            ));
        }
        if presence.span_id {
            columns.push((
                ch_consts::CH_SPAN_ID,
                Arc::new(self.span_id.finish()) as ArrayRef,
            ));
        }
        columns.push((
            ch_consts::CH_TIMESTAMP,
            Arc::new(self.timestamp.finish()) as ArrayRef,
        ));
        if presence.trace_id {
            columns.push((
                ch_consts::CH_TRACE_ID,
                Arc::new(self.trace_id.finish()) as ArrayRef,
            ));
        }
        columns
    }
}

fn collect_resource_attributes<R: ResourceView>(
    resource: R,
    scratch: &mut Vec<u8>,
) -> Result<(Vec<(String, String)>, String), ClickhouseExporterError> {
    let mut attributes = Vec::new();
    let mut service_name = None;
    for attribute in resource.attributes() {
        let key_bytes = attribute.key();
        let key = utf8(key_bytes)?.to_owned();
        stringify_optional_any_value(attribute.value(), scratch)?;
        let value = utf8(scratch)?.to_owned();
        if service_name.is_none() && key_bytes == SERVICE_NAME_KEY {
            service_name = Some(value.clone());
        }
        attributes.push((key, value));
    }
    Ok((attributes, service_name.unwrap_or_default()))
}

fn collect_attributes<I, A>(
    attributes: I,
    scratch: &mut Vec<u8>,
) -> Result<Vec<(String, String)>, ClickhouseExporterError>
where
    I: Iterator<Item = A>,
    A: AttributeView,
{
    let mut collected = Vec::new();
    for attribute in attributes {
        let key = utf8(attribute.key())?.to_owned();
        stringify_optional_any_value(attribute.value(), scratch)?;
        collected.push((key, utf8(scratch)?.to_owned()));
    }
    Ok(collected)
}

fn append_attribute<A: AttributeView>(
    builder: &mut MapBuilder<StringBuilder, StringBuilder>,
    attribute: A,
    scratch: &mut Vec<u8>,
) -> Result<(), ClickhouseExporterError> {
    builder.keys().append_value(utf8(attribute.key())?);
    stringify_optional_any_value(attribute.value(), scratch)?;
    builder.values().append_value(utf8(scratch)?);
    Ok(())
}

fn append_attribute_map(
    builder: &mut MapBuilder<StringBuilder, StringBuilder>,
    attributes: &[(String, String)],
) -> Result<(), ClickhouseExporterError> {
    for (key, value) in attributes {
        builder.keys().append_value(key);
        builder.values().append_value(value);
    }
    builder.append(true)?;
    Ok(())
}

fn stringify_optional_any_value<'a, V: AnyValueView<'a> + 'a>(
    value: Option<V>,
    scratch: &mut Vec<u8>,
) -> Result<(), ClickhouseExporterError> {
    scratch.clear();
    if let Some(value) = value {
        stringify_any_value(value, scratch)?;
    }
    Ok(())
}

fn stringify_any_value<'a, V: AnyValueView<'a> + 'a>(
    value: V,
    scratch: &mut Vec<u8>,
) -> Result<(), ClickhouseExporterError> {
    scratch.clear();
    match value.value_type() {
        ValueType::Empty => {}
        ValueType::String => scratch.extend_from_slice(
            value
                .as_string()
                .ok_or_else(|| invalid_any_value("string"))?,
        ),
        ValueType::Bool => scratch.extend_from_slice(
            if value.as_bool().ok_or_else(|| invalid_any_value("bool"))? {
                b"true"
            } else {
                b"false"
            },
        ),
        ValueType::Int64 => {
            let mut buffer = itoa::Buffer::new();
            scratch.extend_from_slice(
                buffer
                    .format(value.as_int64().ok_or_else(|| invalid_any_value("int64"))?)
                    .as_bytes(),
            );
        }
        ValueType::Double => {
            let mut buffer = ryu::Buffer::new();
            scratch.extend_from_slice(
                buffer
                    .format(
                        value
                            .as_double()
                            .ok_or_else(|| invalid_any_value("double"))?,
                    )
                    .as_bytes(),
            );
        }
        ValueType::Bytes => {
            let encoded = base64::engine::general_purpose::STANDARD
                .encode(value.as_bytes().ok_or_else(|| invalid_any_value("bytes"))?);
            scratch.extend_from_slice(encoded.as_bytes());
        }
        ValueType::Array | ValueType::KeyValueList => {
            serde_json::to_writer(&mut *scratch, &AnyValueSerializerWrapper(value)).map_err(
                |error| ClickhouseExporterError::CoercionError {
                    error: format!("failed to serialize OTLP AnyValue as JSON: {error}"),
                },
            )?;
        }
    }
    Ok(())
}

fn invalid_any_value(expected: &str) -> ClickhouseExporterError {
    ClickhouseExporterError::CoercionError {
        error: format!("OTLP AnyValue declared {expected} but did not contain a valid value"),
    }
}

struct AnyValueSerializerWrapper<T>(T);

impl<'a, T> Serialize for AnyValueSerializerWrapper<T>
where
    T: AnyValueView<'a> + 'a,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_any_value(&self.0, serializer)
    }
}

fn serialize_any_value<'a, T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AnyValueView<'a> + 'a,
    S: Serializer,
{
    match value.value_type() {
        ValueType::Empty => serializer.serialize_none(),
        ValueType::String => serializer.serialize_str(
            std::str::from_utf8(
                value
                    .as_string()
                    .ok_or_else(|| S::Error::custom("missing OTLP string value"))?,
            )
            .map_err(S::Error::custom)?,
        ),
        ValueType::Bool => serializer.serialize_bool(
            value
                .as_bool()
                .ok_or_else(|| S::Error::custom("missing OTLP bool value"))?,
        ),
        ValueType::Int64 => serializer.serialize_i64(
            value
                .as_int64()
                .ok_or_else(|| S::Error::custom("missing OTLP int64 value"))?,
        ),
        ValueType::Double => serializer.serialize_f64(
            value
                .as_double()
                .ok_or_else(|| S::Error::custom("missing OTLP double value"))?,
        ),
        ValueType::Bytes => serializer.serialize_bytes(
            value
                .as_bytes()
                .ok_or_else(|| S::Error::custom("missing OTLP bytes value"))?,
        ),
        ValueType::Array => {
            let mut sequence = serializer.serialize_seq(None)?;
            for child in value
                .as_array()
                .ok_or_else(|| S::Error::custom("missing OTLP array value"))?
            {
                sequence.serialize_element(&AnyValueSerializerWrapper(child))?;
            }
            sequence.end()
        }
        ValueType::KeyValueList => {
            let mut map = serializer.serialize_map(None)?;
            for attribute in value
                .as_kvlist()
                .ok_or_else(|| S::Error::custom("missing OTLP map value"))?
            {
                let key = std::str::from_utf8(attribute.key()).map_err(S::Error::custom)?;
                match attribute.value() {
                    Some(child) => {
                        map.serialize_entry(key, &AnyValueSerializerWrapper(child))?;
                    }
                    None => map.serialize_entry(key, &Option::<()>::None)?,
                }
            }
            map.end()
        }
    }
}

fn optional_utf8(value: Option<&[u8]>) -> Result<Option<String>, ClickhouseExporterError> {
    value
        .map(|value| utf8(value).map(ToOwned::to_owned))
        .transpose()
}

fn utf8(value: &[u8]) -> Result<&str, ClickhouseExporterError> {
    std::str::from_utf8(value).map_err(|error| ClickhouseExporterError::CoercionError {
        error: format!("invalid UTF-8 in OTLP string: {error}"),
    })
}

fn append_optional_string(builder: &mut StringBuilder, value: Option<&str>) {
    match value {
        Some(value) => builder.append_value(value),
        None => builder.append_null(),
    }
}

fn append_optional_bytes_as_string(
    builder: &mut StringBuilder,
    value: Option<&[u8]>,
    present: &mut bool,
) -> Result<(), ClickhouseExporterError> {
    match value {
        Some(value) => {
            builder.append_value(utf8(value)?);
            *present = true;
        }
        None => builder.append_null(),
    }
    Ok(())
}

fn append_optional_hex(
    builder: &mut StringBuilder,
    value: Option<&[u8]>,
    present: &mut bool,
    scratch: &mut Vec<u8>,
) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    match value {
        Some(value) => {
            scratch.clear();
            scratch.reserve(value.len() * 2);
            for byte in value {
                scratch.push(HEX[(byte >> 4) as usize]);
                scratch.push(HEX[(byte & 0x0f) as usize]);
            }
            builder.append_value(
                std::str::from_utf8(scratch).expect("lowercase hexadecimal is valid UTF-8"),
            );
            *present = true;
        }
        None => builder.append_null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use arrow::array::Array;
    use arrow::util::display::array_value_to_string;
    use bytes::Bytes;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::fixtures;
    use otap_df_pdata::{OtapArrowRecords, OtapPayload, OtlpProtoBytes, TryIntoWithOptions};
    use prost::Message;

    use crate::exporters::clickhouse_exporter::transform::transform_batch::BatchTransformer;
    use crate::exporters::clickhouse_exporter::{
        config::Config,
        writer::{ClickHouseWriter, build_client},
    };

    fn request_bytes(logs: LogsData) -> Bytes {
        Bytes::from(
            ExportLogsServiceRequest {
                resource_logs: logs.resource_logs,
            }
            .encode_to_vec(),
        )
    }

    fn legacy_batch(bytes: &Bytes) -> Option<RecordBatch> {
        let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(bytes.clone()).into();
        let mut records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("convert raw OTLP logs through OTAP Arrow");
        records
            .decode_transport_optimized_ids()
            .expect("decode transport optimized IDs");
        BatchTransformer::new()
            .apply_plan(records)
            .expect("apply legacy ClickHouse transform")
            .remove(&ArrowPayloadType::Logs)
    }

    fn logical_values(array: &dyn Array) -> Vec<Option<String>> {
        if matches!(array.data_type(), DataType::Binary)
            || matches!(
                array.data_type(),
                DataType::Dictionary(_, value) if **value == DataType::Binary
            )
        {
            let binary = arrow::compute::cast(array, &DataType::Binary)
                .expect("cast dictionary-encoded binary values")
                .as_any()
                .downcast_ref::<arrow::array::BinaryArray>()
                .expect("binary array")
                .clone();
            return (0..binary.len())
                .map(|row| {
                    (!binary.is_null(row)).then(|| {
                        std::str::from_utf8(binary.value(row))
                            .expect("ClickHouse string bytes are UTF-8")
                            .to_owned()
                    })
                })
                .collect();
        }

        (0..array.len())
            .map(|row| {
                (!array.is_null(row)).then(|| {
                    array_value_to_string(array, row).expect("format Arrow value for comparison")
                })
            })
            .collect()
    }

    fn assert_matches_legacy(logs: LogsData) {
        let bytes = request_bytes(logs);
        let expected = legacy_batch(&bytes);
        let actual = OtlpLogsTransformer::default()
            .transform(&bytes)
            .expect("transform raw OTLP logs directly");

        match (actual, expected) {
            (Some(actual), Some(expected)) => {
                let actual_names = actual
                    .schema()
                    .fields()
                    .iter()
                    .map(|field| field.name().clone())
                    .collect::<Vec<_>>();
                let expected_names = expected
                    .schema()
                    .fields()
                    .iter()
                    .map(|field| field.name().clone())
                    .collect::<Vec<_>>();
                assert_eq!(actual_names, expected_names);
                assert_eq!(actual.num_rows(), expected.num_rows());
                for (actual, expected) in actual.columns().iter().zip(expected.columns()) {
                    assert_eq!(
                        logical_values(actual.as_ref()),
                        logical_values(expected.as_ref())
                    );
                }
            }
            (None, None) => {}
            (actual, expected) => panic!(
                "direct and legacy transforms disagree about output presence: direct={}, legacy={}",
                actual.is_some(),
                expected.is_some()
            ),
        }
    }

    fn logs_with_all_value_types() -> LogsData {
        let nested = AnyValue::new_kvlist(vec![
            KeyValue::new("nested.string", AnyValue::new_string("value")),
            KeyValue::new(
                "nested.array",
                AnyValue::new_array(vec![
                    AnyValue::new_bool(false),
                    AnyValue::new_int(-7),
                    AnyValue::new_bytes([1_u8, 2, 3]),
                ]),
            ),
        ]);
        LogsData::new(vec![ResourceLogs {
            resource: Some(
                Resource::build()
                    .attributes(vec![
                        KeyValue::new("service.name", AnyValue::new_string("first")),
                        KeyValue::new("service.name", AnyValue::new_string("second")),
                        KeyValue::new("resource.map", nested.clone()),
                    ])
                    .finish(),
            ),
            scope_logs: vec![ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("raw-scope")
                        .version("1.0")
                        .attributes(vec![KeyValue::new(
                            "scope.array",
                            AnyValue::new_array(vec![AnyValue::new_double(1.5)]),
                        )])
                        .finish(),
                ),
                log_records: vec![
                    LogRecord::build()
                        .time_unix_nano(123_u64)
                        .severity_number(SeverityNumber::Error)
                        .severity_text("ERROR")
                        .body(nested)
                        .attributes(vec![
                            KeyValue::new("empty", AnyValue::default()),
                            KeyValue::new("string", AnyValue::new_string("text")),
                            KeyValue::new("int", AnyValue::new_int(-42)),
                            KeyValue::new("double", AnyValue::new_double(3.25)),
                            KeyValue::new("bool", AnyValue::new_bool(true)),
                        ])
                        .trace_id([0x11; 16])
                        .span_id([0x22; 8])
                        .event_name("raw-event")
                        .finish(),
                ],
                schema_url: "https://scope.example/v1".to_string(),
            }],
            schema_url: "https://resource.example/v1".to_string(),
        }])
    }

    /// Scenario: raw OTLP logs contain resource, scope, and log attributes plus scalar fields.
    /// Guarantees: the direct path preserves the legacy transform's columns, row order, and values.
    #[test]
    fn full_logs_match_legacy_transform() {
        assert_matches_legacy(fixtures::logs_with_full_resource_and_scope());
    }

    /// Scenario: raw OTLP logs omit every resource, scope, and log attribute.
    /// Guarantees: the direct path omits the same attribute columns as the legacy transform.
    #[test]
    fn logs_without_attributes_match_legacy_transform() {
        assert_matches_legacy(fixtures::logs_with_no_attributes());
    }

    /// Scenario: raw OTLP logs span resources and scopes with mixed optional content.
    /// Guarantees: direct traversal preserves legacy row order, nulls, and conditional columns.
    #[test]
    fn mixed_resources_and_scopes_match_legacy_transform() {
        assert_matches_legacy(fixtures::logs_multiple_resources_mixed_content());
    }

    /// Scenario: raw OTLP logs exercise scalar, nested, ID, and schema fields.
    /// Guarantees: direct JSON and scalar conversion exactly preserve legacy logical values.
    #[test]
    fn nested_value_types_match_legacy_transform() {
        assert_matches_legacy(logs_with_all_value_types());
    }

    /// Scenario: a raw OTLP log attribute contains an arbitrary byte sequence.
    /// Guarantees: the direct path stores the attribute using standard base64 encoding.
    #[test]
    fn bytes_attribute_is_base64_encoded() {
        let logs = LogsData::new(vec![ResourceLogs::new(
            Resource::default(),
            vec![ScopeLogs::new(
                InstrumentationScope::default(),
                vec![
                    LogRecord::build()
                        .attributes(vec![KeyValue::new(
                            "bytes",
                            AnyValue::new_bytes([0_u8, 1, 2, 255]),
                        )])
                        .finish(),
                ],
            )],
        )]);
        let bytes = request_bytes(logs);
        let batch = OtlpLogsTransformer::default()
            .transform(&bytes)
            .expect("transform byte attribute")
            .expect("logs batch");
        let attributes = batch
            .column_by_name(ch_consts::CH_LOG_ATTRIBUTES)
            .expect("log attributes")
            .as_any()
            .downcast_ref::<arrow::array::MapArray>()
            .expect("map column");
        let keys = attributes
            .keys()
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .expect("map keys");
        let values = attributes
            .values()
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .expect("map values");

        assert_eq!(keys.value(0), "bytes");
        assert_eq!(values.value(0), "AAEC/w==");
    }

    /// Scenario: an OTLP export request contains no log records.
    /// Guarantees: the direct path produces no ClickHouse write batch, matching legacy behavior.
    #[test]
    fn empty_logs_produce_no_batch() {
        assert_matches_legacy(fixtures::empty_logs());
        assert_matches_legacy(fixtures::logs_with_empty_log_records());
    }

    /// Scenario: a raw OTLP request has invalid top-level protobuf framing.
    /// Guarantees: the direct path returns an error so exporter routing can reject the request.
    #[test]
    fn malformed_request_is_rejected() {
        let error = OtlpLogsTransformer::default()
            .transform(b"\xff")
            .expect_err("malformed protobuf must be rejected");

        assert!(matches!(error, ClickhouseExporterError::Child(_)));
    }

    /// Scenario: a nested OTLP field declares a length beyond its enclosing message.
    /// Guarantees: the direct transformer returns an error before traversing malformed ranges.
    #[test]
    fn malformed_nested_request_is_rejected() {
        let error = OtlpLogsTransformer::default()
            .transform(&[0x0a, 0x03, 0x1a, 0x05, 0x00])
            .expect_err("malformed nested protobuf must be rejected");

        assert!(matches!(error, ClickhouseExporterError::Child(_)));
    }

    /// Scenario: the previous OTLP request reports an arbitrarily large row count.
    /// Guarantees: speculative Arrow builder capacity remains capped at a bounded row count.
    #[test]
    fn reusable_row_preallocation_is_bounded() {
        assert_eq!(bounded_row_capacity(8_192), 8_192);
        assert_eq!(
            bounded_row_capacity(MAX_PREALLOCATED_ROWS),
            MAX_PREALLOCATED_ROWS
        );
        assert_eq!(bounded_row_capacity(usize::MAX), MAX_PREALLOCATED_ROWS);
    }

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct RawLogSummary {
        row_count: u64,
        service_name: String,
        body: String,
    }

    /// Scenario: a direct raw OTLP batch is inserted into a live ClickHouse logs table.
    /// Guarantees: emitted Arrow types bind by name and preserve rows, service extraction, and body JSON.
    #[tokio::test]
    #[ignore = "requires a live ClickHouse; run with --ignored e2e"]
    async fn e2e_raw_otlp_logs_insert_roundtrips_through_clickhouse_schema() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let endpoint =
            std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".into());
        let username = std::env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".into());
        let password = std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| "test".into());
        let patch = serde_json::from_value(serde_json::json!({
            "endpoint": endpoint,
            "database": "otap_e2e_raw_otlp_logs",
            "username": username,
            "password": password,
            "async_insert": false,
        }))
        .expect("valid ClickHouse config");
        let config = Config::from_patch(patch);
        build_client(&config, "default")
            .query("DROP DATABASE IF EXISTS otap_e2e_raw_otlp_logs")
            .execute()
            .await
            .expect("drop pre-existing test database");

        let bytes = request_bytes(logs_with_all_value_types());
        let batch = OtlpLogsTransformer::default()
            .transform(&bytes)
            .expect("transform raw OTLP logs")
            .expect("logs batch");
        let batches = HashMap::from([(ArrowPayloadType::Logs, batch)]);
        let writer = ClickHouseWriter::new(&config)
            .await
            .expect("create ClickHouse schema");
        _ = writer
            .write_batches(&batches)
            .await
            .expect("insert raw OTLP logs batch");

        let summary = build_client(&config, &config.database)
            .query(
                "SELECT count() AS row_count, CAST(any(ServiceName) AS String) AS service_name, \
                 any(Body) AS body FROM otap_e2e_raw_otlp_logs.otel_logs",
            )
            .fetch_one::<RawLogSummary>()
            .await
            .expect("read raw OTLP logs back");
        assert_eq!(summary.row_count, 1);
        assert_eq!(summary.service_name, "first");
        assert_eq!(
            summary.body,
            "{\"nested.string\":\"value\",\"nested.array\":[false,-7,[1,2,3]]}"
        );
    }
}
