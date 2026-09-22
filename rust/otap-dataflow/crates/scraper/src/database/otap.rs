// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared typed database-row to OTLP log conversion.

use super::page::{CompositeCursor, CursorRow, QueryPage};
use super::{CellValue, ColumnMetadata, DatabaseSystem, OutputConfig, Row};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::OtapPayload;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue, KeyValueList, any_value,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
use prost::Message;
use std::collections::HashMap;
use std::mem::size_of;

const DATABASE_SCOPE: &str = "otel-arrow.database_receiver";
const SOURCE_ID_ATTRIBUTE: &str = "receiver.database.source_id";
const QUERY_NAME_ATTRIBUTE: &str = "receiver.database.query.name";
const MAX_RETAINED_RECORD_BYTES: usize = 4 * 1024 * 1024;

/// Validates configured columns against live result metadata.
pub fn validate_mapping(
    columns: &[ColumnMetadata],
    output: &OutputConfig,
) -> Result<(), OtlpMappingError> {
    mapping_timestamp_index(columns, output).map(|_| ())
}

fn mapping_timestamp_index(
    columns: &[ColumnMetadata],
    output: &OutputConfig,
) -> Result<Option<usize>, OtlpMappingError> {
    // Column matching is case-insensitive because database drivers can change
    // identifier case based on quoting and vendor defaults.
    let mut indices = HashMap::with_capacity(columns.len());
    for (index, column) in columns.iter().enumerate() {
        if indices
            .insert(column.name.to_ascii_lowercase(), index)
            .is_some()
        {
            return Err(OtlpMappingError::DuplicateColumn {
                name: column.name.clone(),
            });
        }
    }
    let timestamp_index = if let Some(column) = &output.timestamp_column {
        let index = indices
            .get(&column.to_ascii_lowercase())
            .copied()
            .ok_or_else(|| OtlpMappingError::UnknownColumn {
                name: column.clone(),
            })?;
        let metadata = &columns[index];
        if !supports_event_time(&metadata.source_type) {
            return Err(OtlpMappingError::InvalidEventTimeMetadata {
                column: column.clone(),
                source_type: metadata.source_type.clone(),
            });
        }
        Some(index)
    } else {
        None
    };
    for column in &output.validation_columns {
        if !indices.contains_key(&column.to_ascii_lowercase()) {
            return Err(OtlpMappingError::UnknownColumn {
                name: column.clone(),
            });
        }
    }
    Ok(timestamp_index)
}

fn supports_event_time(source_type: &str) -> bool {
    // Adapters expose stable vendor type names, while row conversion provides
    // the final value-level check and timestamp parser.
    let source_type = source_type.to_ascii_uppercase();
    source_type.starts_with("DATE")
        || source_type.starts_with("TIMESTAMP")
        || source_type.starts_with("CHAR")
        || source_type.starts_with("NCHAR")
        || source_type.starts_with("VARCHAR")
        || source_type.starts_with("NVARCHAR")
        || source_type == "TEXT"
}

struct MappingPlan {
    columns: Vec<ColumnMetadata>,
    timestamp_index: Option<usize>,
}

impl MappingPlan {
    fn compile(
        columns: Vec<ColumnMetadata>,
        output: &OutputConfig,
    ) -> Result<Self, OtlpMappingError> {
        let timestamp_index = mapping_timestamp_index(&columns, output)?;
        Ok(Self {
            columns,
            timestamp_index,
        })
    }
}

/// Single-owner encoder with one cached schema and reusable envelope metadata.
///
/// The receiver moves this value into its blocking encoding job and back again;
/// no shared mutable cache or synchronization is needed on the encoding path.
pub(crate) struct OtlpPageEncoder {
    mapping: MappingPlan,
    output: OutputConfig,
    source_id: String,
    logs: LogsData,
    resource_static_bytes: usize,
    scope_static_bytes: usize,
}

impl OtlpPageEncoder {
    /// Releases reusable empty record storage while ingress is memory-pressure paused.
    pub(crate) fn release_scratch(&mut self) {
        let records = &mut self.logs.resource_logs[0].scope_logs[0].log_records;
        debug_assert!(records.is_empty());
        *records = Vec::new();
    }

    #[cfg(test)]
    pub(crate) fn retained_record_bytes(&self) -> usize {
        self.logs.resource_logs[0].scope_logs[0]
            .log_records
            .capacity()
            .saturating_mul(size_of::<LogRecord>())
    }

    /// Validates the initial schema and prepares the constant OTLP envelope.
    pub(crate) fn new(
        system: DatabaseSystem,
        source_id: String,
        output: OutputConfig,
        columns: Vec<ColumnMetadata>,
    ) -> Result<Self, OtlpMappingError> {
        let mapping = MappingPlan::compile(columns, &output)?;
        let scope = ScopeLogs {
            scope: Some(InstrumentationScope {
                name: DATABASE_SCOPE.to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let scope_static_bytes = scope.encoded_len();
        let mut resource = ResourceLogs {
            resource: Some(Resource {
                attributes: vec![
                    KeyValue {
                        key: "db.system.name".to_owned(),
                        value: Some(string_value(system.as_str())),
                    },
                    KeyValue {
                        key: SOURCE_ID_ATTRIBUTE.to_owned(),
                        value: Some(string_value(source_id.clone())),
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        };
        let resource_static_bytes = resource.encoded_len();
        resource.scope_logs = vec![scope];
        Ok(Self {
            mapping,
            output,
            source_id,
            logs: LogsData {
                resource_logs: vec![resource],
            },
            resource_static_bytes,
            scope_static_bytes,
        })
    }

    /// Encodes a page, recompiling the plan only when its full metadata changes.
    pub(crate) fn encode_page(
        &mut self,
        page: QueryPage,
        observed_time_unix_nano: u64,
        max_batch_bytes: u64,
    ) -> Result<Option<EncodedPage>, OtlpMappingError> {
        let QueryPage { columns, rows } = page;
        if columns != self.mapping.columns {
            // Validate before replacement so a rejected schema cannot poison
            // the last valid plan. Names, order, source types and nullability matter.
            self.mapping = MappingPlan::compile(columns, &self.output)?;
        } else {
            // Release the adapter's duplicate metadata before allocating output records.
            drop(columns);
        }
        self.encode_rows(rows, observed_time_unix_nano, max_batch_bytes)
    }

    fn encode_rows(
        &mut self,
        rows: Vec<CursorRow>,
        observed_time_unix_nano: u64,
        max_batch_bytes: u64,
    ) -> Result<Option<EncodedPage>, OtlpMappingError> {
        let result = self.encode_rows_inner(rows, observed_time_unix_nano, max_batch_bytes);
        // Drop row payloads even on errors. Only an empty, bounded vector and
        // schema/envelope metadata survive between pages, never customer row values.
        let records = &mut self.logs.resource_logs[0].scope_logs[0].log_records;
        records.clear();
        if records.capacity().saturating_mul(size_of::<LogRecord>()) > MAX_RETAINED_RECORD_BYTES {
            self.release_scratch();
        }
        result
    }

    fn encode_rows_inner(
        &mut self,
        rows: Vec<CursorRow>,
        observed_time_unix_nano: u64,
        max_batch_bytes: u64,
    ) -> Result<Option<EncodedPage>, OtlpMappingError> {
        if rows.is_empty() {
            return Ok(None);
        }
        // The private constructor always creates exactly one resource and scope.
        let records = &mut self.logs.resource_logs[0].scope_logs[0].log_records;
        records.reserve(rows.len());
        let mut candidate = None;
        let mut encoded_bytes = 0;
        let mut records_wire_bytes = 0_usize;
        let mut event_time_fallbacks = 0_usize;
        let total_rows = rows.len();

        for cursor_row in rows {
            let (record, used_event_time_fallback) = row_to_record(
                cursor_row.row,
                &self.mapping,
                &self.source_id,
                observed_time_unix_nano,
            )?;
            let record_bytes = record.encoded_len();
            let next_records_wire_bytes = records_wire_bytes
                .saturating_add(1)
                .saturating_add(prost::encoding::encoded_len_varint(record_bytes as u64))
                .saturating_add(record_bytes);
            let scope_bytes = self
                .scope_static_bytes
                .saturating_add(next_records_wire_bytes);
            let resource_bytes = self
                .resource_static_bytes
                .saturating_add(1)
                .saturating_add(prost::encoding::encoded_len_varint(scope_bytes as u64))
                .saturating_add(scope_bytes);
            let candidate_size = 1_usize
                .saturating_add(prost::encoding::encoded_len_varint(resource_bytes as u64))
                .saturating_add(resource_bytes);
            if u64::try_from(candidate_size).unwrap_or(u64::MAX) > max_batch_bytes {
                if records.is_empty() {
                    return Err(OtlpMappingError::OversizedFirstRow {
                        encoded_bytes: candidate_size,
                        limit: max_batch_bytes,
                    });
                }
                break;
            }
            records.push(record);
            event_time_fallbacks =
                event_time_fallbacks.saturating_add(usize::from(used_event_time_fallback));
            records_wire_bytes = next_records_wire_bytes;
            encoded_bytes = candidate_size;
            candidate = Some(cursor_row.cursor);
        }

        let candidate = candidate.ok_or(OtlpMappingError::MissingCandidate)?;
        let row_count = records.len();
        debug_assert_eq!(encoded_bytes, self.logs.encoded_len());
        // The output buffer is transferred to downstream ownership. Reusing a
        // staging byte buffer here would require another copy or buffer-return protocol.
        // Prefix admission already computed the exact length. Use Prost's
        // generated encoder without another full length walk for allocation.
        let mut bytes = Vec::with_capacity(encoded_bytes);
        self.logs.encode_raw(&mut bytes);
        debug_assert_eq!(bytes.len(), encoded_bytes);
        let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(bytes.into()).into();
        Ok(Some(EncodedPage {
            pdata: OtapPdata::new_todo_context(payload),
            candidate,
            row_count,
            encoded_bytes,
            deferred_rows: total_rows.saturating_sub(row_count),
            event_time_fallbacks,
        }))
    }
}

/// One encoded OTLP page plus the cursor of its last emitted row.
#[derive(Debug)]
pub struct EncodedPage {
    /// Serialized OTLP logs payload.
    pub pdata: OtapPdata,
    /// Cursor of the last row actually included in `pdata`.
    pub candidate: CompositeCursor,
    /// Number of database rows encoded into `pdata`.
    pub row_count: usize,
    /// Exact serialized size of `pdata`.
    pub encoded_bytes: usize,
    /// Number of fetched rows deferred to the next poll by the byte ceiling.
    pub deferred_rows: usize,
    /// Records that used observation time because source event time could not
    /// fit OTLP's unsigned nanosecond range.
    pub event_time_fallbacks: usize,
}

/// Encodes the largest non-empty row prefix that fits `max_batch_bytes`.
///
/// Rows beyond the ceiling are deferred to the next poll rather than dropped,
/// and the returned candidate always comes from the last row actually encoded.
/// An empty page returns `None`; a first row that alone exceeds the ceiling is
/// an explicit error so no row is silently skipped.
pub fn encode_page(
    page: QueryPage,
    system: DatabaseSystem,
    source_id: &str,
    output: &OutputConfig,
    observed_time_unix_nano: u64,
    max_batch_bytes: u64,
) -> Result<Option<EncodedPage>, OtlpMappingError> {
    let QueryPage { columns, rows } = page;
    if rows.is_empty() {
        validate_mapping(&columns, output)?;
        return Ok(None);
    }
    OtlpPageEncoder::new(system, source_id.to_owned(), output.clone(), columns)?.encode_rows(
        rows,
        observed_time_unix_nano,
        max_batch_bytes,
    )
}

fn row_to_record(
    row: Row,
    mapping: &MappingPlan,
    source_id: &str,
    observed_time_unix_nano: u64,
) -> Result<(LogRecord, bool), OtlpMappingError> {
    let columns = &mapping.columns;
    if row.values.len() != columns.len() {
        return Err(OtlpMappingError::ColumnCount);
    }
    let mut body = Vec::with_capacity(columns.len());
    let attributes = vec![
        KeyValue {
            key: SOURCE_ID_ATTRIBUTE.to_owned(),
            value: Some(string_value(source_id)),
        },
        KeyValue {
            key: QUERY_NAME_ATTRIBUTE.to_owned(),
            // The Oracle receiver has one query per source, so its stable
            // source_id is also the unambiguous query identity.
            value: Some(string_value(source_id)),
        },
    ];
    let mut event_time = None;
    let mut used_event_time_fallback = false;
    if let Some(index) = mapping.timestamp_index {
        let value = &row.values[index];
        if !matches!(value, CellValue::Null) {
            event_time = parse_event_time(value, &columns[index].name)?;
            used_event_time_fallback = event_time.is_none();
        }
    }
    for (column, value) in columns.iter().zip(row.values) {
        body.push(KeyValue {
            key: column.name.clone(),
            value: Some(cell_to_any(value)?),
        });
    }
    Ok((
        LogRecord {
            time_unix_nano: event_time.unwrap_or(observed_time_unix_nano),
            observed_time_unix_nano,
            severity_number: SeverityNumber::Info as i32,
            severity_text: "INFO".to_owned(),
            body: Some(AnyValue {
                value: Some(any_value::Value::KvlistValue(KeyValueList { values: body })),
            }),
            attributes,
            event_name: "database.query.row".to_owned(),
            ..Default::default()
        },
        used_event_time_fallback,
    ))
}

fn cell_to_any(value: CellValue) -> Result<AnyValue, OtlpMappingError> {
    // Precision-sensitive SQL values remain text unless OTLP has an exact
    // scalar representation. In particular, OTLP has no unsigned integer or
    // decimal attribute type.
    Ok(match value {
        CellValue::Null => AnyValue::default(),
        CellValue::Bool(value) => AnyValue {
            value: Some(any_value::Value::BoolValue(value)),
        },
        CellValue::Int64(value) => int_value(value),
        CellValue::UInt64(value) => i64::try_from(value)
            .map(int_value)
            .unwrap_or_else(|_| string_value(value.to_string())),
        CellValue::Float64(value) if value.is_finite() => AnyValue {
            value: Some(any_value::Value::DoubleValue(value)),
        },
        CellValue::Float64(_) => return Err(OtlpMappingError::NonFiniteFloat),
        CellValue::Bytes(value) => AnyValue {
            value: Some(any_value::Value::BytesValue(value)),
        },
        CellValue::Decimal(value)
        | CellValue::String(value)
        | CellValue::Timestamp(value)
        | CellValue::TimestampTz(value)
        | CellValue::Interval(value) => string_value(value),
    })
}

fn parse_event_time(value: &CellValue, column: &str) -> Result<Option<u64>, OtlpMappingError> {
    let text = match value {
        CellValue::Timestamp(value) | CellValue::TimestampTz(value) | CellValue::String(value) => {
            value
        }
        _ => {
            return Err(OtlpMappingError::InvalidEventTimeType {
                column: column.to_owned(),
            });
        }
    };
    let timestamp =
        parse_utc_timestamp(text).map_err(|source| OtlpMappingError::InvalidEventTime {
            column: column.to_owned(),
            source,
        })?;
    Ok(timestamp_to_nanos(timestamp))
}

/// Parses supported timestamp representations for both progress and event time.
pub(crate) fn parse_utc_timestamp(text: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(text) {
        return Ok(timestamp.with_timezone(&Utc));
    }
    if let Ok(timestamp) = DateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f %:z") {
        return Ok(timestamp.with_timezone(&Utc));
    }
    // The adapter must establish the UTC meaning of timezone-less values;
    // interpreting them must never depend on the collector host's timezone.
    NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f"))
        .or_else(|_| {
            NaiveDate::parse_from_str(text, "%Y-%m-%d").map(|date| date.and_time(NaiveTime::MIN))
        })
        .map(|timestamp| timestamp.and_utc())
}

fn timestamp_to_nanos(timestamp: DateTime<Utc>) -> Option<u64> {
    // Oracle supports a much wider year range than OTLP's unsigned nanosecond
    // timestamp. Preserve the source timestamp in the body and fall back to
    // observation time instead of making one valid row poison the checkpoint.
    u64::try_from(timestamp.timestamp())
        .ok()
        .and_then(|seconds| seconds.checked_mul(1_000_000_000))
        .and_then(|nanos| nanos.checked_add(u64::from(timestamp.timestamp_subsec_nanos())))
}

fn string_value(value: impl Into<String>) -> AnyValue {
    AnyValue {
        value: Some(any_value::Value::StringValue(value.into())),
    }
}

fn int_value(value: i64) -> AnyValue {
    AnyValue {
        value: Some(any_value::Value::IntValue(value)),
    }
}

/// Database-row to OTLP conversion failure.
#[derive(Debug, thiserror::Error)]
pub enum OtlpMappingError {
    /// Live metadata contains duplicate normalized names.
    #[error("result metadata contains duplicate column '{name}'")]
    DuplicateColumn {
        /// Duplicate result column.
        name: String,
    },
    /// Output configuration references an absent result column.
    #[error("configured output column '{name}' is not present in result metadata")]
    UnknownColumn {
        /// Missing result column.
        name: String,
    },
    /// Row width does not match the inspected result metadata.
    #[error("database row value count does not match its result metadata")]
    ColumnCount,
    /// A float cannot be represented in OTLP.
    #[error("non-finite floating-point values are not supported")]
    NonFiniteFloat,
    /// Configured event-time column has an unsupported type.
    #[error("event-time column '{column}' is not a date, timestamp, or string")]
    InvalidEventTimeType {
        /// Configured result column.
        column: String,
    },
    /// Live metadata reports an unsupported event-time source type.
    #[error("event-time column '{column}' has unsupported source type '{source_type}'")]
    InvalidEventTimeMetadata {
        /// Configured result column.
        column: String,
        /// Adapter-reported source type.
        source_type: String,
    },
    /// Configured event-time text is invalid.
    #[error("event-time column '{column}' contains an invalid timestamp")]
    InvalidEventTime {
        /// Configured result column.
        column: String,
        /// Timestamp parsing failure.
        #[source]
        source: chrono::ParseError,
    },
    /// The first row alone exceeds the configured encoded-byte ceiling.
    #[error(
        "the first database row encodes to {encoded_bytes} bytes, exceeding the {limit}-byte query.max_batch_bytes limit"
    )]
    OversizedFirstRow {
        /// Exact serialized size of the single-row payload.
        encoded_bytes: usize,
        /// Configured encoded-byte ceiling.
        limit: u64,
    },
    /// A non-empty page produced no candidate cursor.
    #[error("a non-empty database page produced no candidate cursor")]
    MissingCandidate,
    /// The constructed OTLP envelope did not have its expected shape.
    #[error("the OTLP logs envelope did not have its expected resource and scope shape")]
    EnvelopeShape,
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use otel_arrow_dfe_pdata::PayloadData;

    fn columns() -> Vec<ColumnMetadata> {
        vec![ColumnMetadata {
            name: "PAYLOAD".to_owned(),
            source_type: "VARCHAR2".to_owned(),
            nullable: true,
        }]
    }

    fn encoder() -> OtlpPageEncoder {
        OtlpPageEncoder::new(
            DatabaseSystem::Oracle,
            "source".to_owned(),
            OutputConfig::default(),
            columns(),
        )
        .expect("encoder")
    }

    fn page(values: Vec<CellValue>) -> QueryPage {
        QueryPage {
            columns: columns(),
            rows: values
                .into_iter()
                .map(|value| CursorRow {
                    row: Row {
                        values: vec![value],
                    },
                    cursor: CompositeCursor::new("2026-01-01T00:00:00Z".to_owned(), 1),
                })
                .collect(),
        }
    }

    /// Scenario: Successive pages share the same complete result metadata.
    /// Guarantees: The schema plan and bounded record-vector allocation are reused, with no rows retained.
    #[test]
    fn stable_schema_reuses_plan_and_record_storage() {
        let mut encoder = encoder();
        let schema = encoder.mapping.columns.as_ptr();
        _ = encoder
            .encode_page(page(vec![CellValue::String("first".to_owned())]), 1, 4096)
            .expect("first page");
        let storage = encoder.logs.resource_logs[0].scope_logs[0]
            .log_records
            .as_ptr();
        _ = encoder
            .encode_page(page(vec![CellValue::String("second".to_owned())]), 2, 4096)
            .expect("second page");
        assert!(std::ptr::eq(schema, encoder.mapping.columns.as_ptr()));
        let records = &encoder.logs.resource_logs[0].scope_logs[0].log_records;
        assert!(std::ptr::eq(storage, records.as_ptr()));
        assert!(records.is_empty());
    }

    /// Scenario: Conversion fails after a prior row has been admitted into the envelope.
    /// Guarantees: No accepted-prefix payload remains cached after the error.
    #[test]
    fn conversion_error_clears_cached_row_payloads() {
        let mut encoder = encoder();
        assert!(matches!(
            encoder.encode_page(
                page(vec![
                    CellValue::String("customer payload".to_owned()),
                    CellValue::Float64(f64::NAN),
                ]),
                1,
                4096,
            ),
            Err(OtlpMappingError::NonFiniteFloat)
        ));
        assert!(
            encoder.logs.resource_logs[0].scope_logs[0]
                .log_records
                .is_empty()
        );
    }

    /// Scenario: A one-off page leaves an unusually large record-vector allocation.
    /// Guarantees: The encoder discards scratch capacity above its fixed retention bound.
    #[test]
    fn oversized_record_storage_is_not_retained() {
        let mut encoder = encoder();
        encoder.logs.resource_logs[0].scope_logs[0]
            .log_records
            .reserve(MAX_RETAINED_RECORD_BYTES / size_of::<LogRecord>() + 1);
        assert!(
            encoder
                .encode_page(page(Vec::new()), 1, 4096)
                .expect("empty page")
                .is_none()
        );
        assert_eq!(
            encoder.logs.resource_logs[0].scope_logs[0]
                .log_records
                .capacity(),
            0
        );
    }

    /// Scenario: Owned string and binary cell values are mapped into protobuf body fields.
    /// Guarantees: Both existing payload allocations are transferred, not deep-copied.
    #[test]
    fn owned_cell_payloads_keep_their_allocations() {
        let text = "payload".repeat(128);
        let bytes = vec![7; 1024];
        let text_ptr = text.as_ptr();
        let bytes_ptr = bytes.as_ptr();
        let mut columns = columns();
        columns.push(ColumnMetadata {
            name: "BYTES".to_owned(),
            source_type: "RAW".to_owned(),
            nullable: false,
        });
        let mapping = MappingPlan::compile(columns, &OutputConfig::default()).expect("mapping");
        let (record, _) = row_to_record(
            Row {
                values: vec![CellValue::String(text), CellValue::Bytes(bytes)],
            },
            &mapping,
            "source",
            1,
        )
        .expect("mapped row");
        let Some(any_value::Value::KvlistValue(body)) = record.body.and_then(|body| body.value)
        else {
            panic!("key-value body");
        };
        let Some(any_value::Value::StringValue(text)) = body.values[0]
            .value
            .as_ref()
            .and_then(|value| value.value.as_ref())
        else {
            panic!("string field");
        };
        let Some(any_value::Value::BytesValue(bytes)) = body.values[1]
            .value
            .as_ref()
            .and_then(|value| value.value.as_ref())
        else {
            panic!("binary field");
        };
        assert!(std::ptr::eq(text_ptr, text.as_ptr()));
        assert!(std::ptr::eq(bytes_ptr, bytes.as_ptr()));
    }

    /// Scenario: Payload lengths cross protobuf length-prefix boundaries.
    /// Guarantees: Cached envelope sizing and direct generated encoding match Prost's canonical output byte-for-byte.
    #[test]
    fn cached_sizes_match_generated_protobuf_encoding() {
        for size in [0, 1, 127, 128, 16_383, 16_384] {
            let mut encoder = encoder();
            let page = page(vec![CellValue::String("x".repeat(size))]);
            let encoded = encoder
                .encode_rows_inner(page.rows, 1, u64::MAX)
                .expect("encode")
                .expect("one row");
            let expected = encoder.logs.encode_to_vec();
            let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(actual)) =
                encoded.pdata.payload().into_data()
            else {
                panic!("OTLP logs bytes");
            };
            assert_eq!(encoded.encoded_bytes, expected.len());
            assert_eq!(actual.as_ref(), expected.as_slice());
        }
    }
}
