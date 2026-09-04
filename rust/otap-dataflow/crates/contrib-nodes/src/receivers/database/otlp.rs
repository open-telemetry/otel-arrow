// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared typed database-row to OTLP log conversion.

use super::page::{CompositeCursor, QueryPage};
use super::{CellValue, ColumnMetadata, DatabaseSystem, OutputConfig, Row};
use base64::Engine;
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
use std::collections::{BTreeMap, BTreeSet};

const DATABASE_SCOPE: &str = "otel-arrow.database_receiver";
const SOURCE_ID_ATTRIBUTE: &str = "receiver.database.source_id";
const QUERY_NAME_ATTRIBUTE: &str = "receiver.database.query.name";

/// Validates configured columns against live result metadata.
pub fn validate_mapping(
    columns: &[ColumnMetadata],
    output: &OutputConfig,
) -> Result<(), OtlpMappingError> {
    // Column matching is case-insensitive because database drivers can change
    // identifier case based on quoting and vendor defaults.
    let mut available = BTreeSet::new();
    for column in columns {
        if !available.insert(column.name.to_ascii_lowercase()) {
            return Err(OtlpMappingError::DuplicateColumn {
                name: column.name.clone(),
            });
        }
    }

    for column in &output.include_columns {
        require_column(&available, column)?;
    }
    for column in output.attributes.keys() {
        require_column(&available, column)?;
    }
    if let Some(column) = &output.timestamp_column {
        let metadata = columns
            .iter()
            .find(|metadata| metadata.name.eq_ignore_ascii_case(column))
            .ok_or_else(|| OtlpMappingError::UnknownColumn {
                name: column.clone(),
            })?;
        if !supports_event_time(&metadata.source_type) {
            return Err(OtlpMappingError::InvalidEventTimeMetadata {
                column: column.clone(),
                source_type: metadata.source_type.clone(),
            });
        }
    }
    for column in &output.validation_columns {
        require_column(&available, column)?;
    }
    Ok(())
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

fn require_column(available: &BTreeSet<String>, name: &str) -> Result<(), OtlpMappingError> {
    if available.contains(&name.to_ascii_lowercase()) {
        Ok(())
    } else {
        Err(OtlpMappingError::UnknownColumn {
            name: name.to_owned(),
        })
    }
}

fn normalized_names(names: &[String]) -> BTreeSet<String> {
    names.iter().map(|name| name.to_ascii_lowercase()).collect()
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
    validate_mapping(&page.columns, output)?;
    if page.rows.is_empty() {
        return Ok(None);
    }
    let included = normalized_names(&output.include_columns);
    let include_all = included.is_empty();

    // Prost encodes nested messages as tag + length-delimited payload. Building
    // the empty envelope once lets each candidate prefix be sized exactly
    // without re-encoding the whole payload for every row.
    let empty = logs_envelope(system, source_id, Vec::new());
    let empty_resource = empty
        .resource_logs
        .first()
        .ok_or(OtlpMappingError::EnvelopeShape)?;
    let empty_scope = empty_resource
        .scope_logs
        .first()
        .ok_or(OtlpMappingError::EnvelopeShape)?;
    let scope_static_bytes = empty_scope.encoded_len();
    let mut resource_without_scopes = empty_resource.clone();
    resource_without_scopes.scope_logs.clear();
    let resource_static_bytes = resource_without_scopes.encoded_len();

    let mut records = Vec::with_capacity(page.rows.len());
    let mut candidate = None;
    let mut encoded_bytes = 0;
    let mut records_wire_bytes = 0_usize;
    let mut event_time_fallbacks = 0_usize;
    let total_rows = page.rows.len();

    for cursor_row in &page.rows {
        let (record, used_event_time_fallback) = row_to_record(
            &cursor_row.row,
            &page.columns,
            source_id,
            output,
            include_all,
            &included,
            observed_time_unix_nano,
        )?;
        let record_bytes = record.encoded_len();
        let next_records_wire_bytes = records_wire_bytes
            .saturating_add(1)
            .saturating_add(prost::encoding::encoded_len_varint(record_bytes as u64))
            .saturating_add(record_bytes);
        let scope_bytes = scope_static_bytes.saturating_add(next_records_wire_bytes);
        let resource_bytes = resource_static_bytes
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
        candidate = Some(cursor_row.cursor.clone());
    }

    let candidate = candidate.ok_or(OtlpMappingError::MissingCandidate)?;
    let row_count = records.len();
    let logs = logs_envelope(system, source_id, records);
    debug_assert_eq!(encoded_bytes, logs.encoded_len());
    let payload: OtapPayload =
        OtlpProtoBytes::ExportLogsRequest(logs.encode_to_vec().into()).into();
    Ok(Some(EncodedPage {
        pdata: OtapPdata::new_todo_context(payload),
        candidate,
        row_count,
        encoded_bytes,
        deferred_rows: total_rows.saturating_sub(row_count),
        event_time_fallbacks,
    }))
}

#[allow(clippy::too_many_arguments)]
fn row_to_record(
    row: &Row,
    columns: &[ColumnMetadata],
    source_id: &str,
    output: &OutputConfig,
    include_all: bool,
    included: &BTreeSet<String>,
    observed_time_unix_nano: u64,
) -> Result<(LogRecord, bool), OtlpMappingError> {
    if row.values.len() != columns.len() {
        return Err(OtlpMappingError::ColumnCount);
    }
    let mut body = Vec::with_capacity(columns.len());
    let mut attributes = vec![
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
    for (column, value) in columns.iter().zip(&row.values) {
        if include_all || included.contains(&column.name.to_ascii_lowercase()) {
            body.push(KeyValue {
                key: column.name.clone(),
                value: Some(cell_to_body(value)?),
            });
        }
        if let Some(attribute_name) = configured_value(&output.attributes, &column.name)
            && !matches!(value, CellValue::Null)
        {
            // Null attributes are omitted by policy; body fields retain an
            // empty AnyValue so the row shape remains observable.
            attributes.push(KeyValue {
                key: attribute_name.clone(),
                value: Some(cell_to_any(value)?),
            });
        }
        if output
            .timestamp_column
            .as_ref()
            .is_some_and(|name| name.eq_ignore_ascii_case(&column.name))
            && !matches!(value, CellValue::Null)
        {
            match parse_event_time(value, &column.name)? {
                Some(timestamp) => event_time = Some(timestamp),
                None => used_event_time_fallback = true,
            }
        }
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

fn logs_envelope(system: DatabaseSystem, source_id: &str, records: Vec<LogRecord>) -> LogsData {
    let resource_attributes = vec![
        KeyValue {
            key: "db.system.name".to_owned(),
            value: Some(string_value(system.as_str())),
        },
        KeyValue {
            // source_id is already operator-authored and contains no endpoint
            // or credential data, making it safe database source identity.
            key: SOURCE_ID_ATTRIBUTE.to_owned(),
            value: Some(string_value(source_id)),
        },
    ];
    LogsData {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: resource_attributes,
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                scope: Some(InstrumentationScope {
                    name: DATABASE_SCOPE.to_owned(),
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                    ..Default::default()
                }),
                log_records: records,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn configured_value<'a>(
    mappings: &'a BTreeMap<String, String>,
    source: &str,
) -> Option<&'a String> {
    mappings
        .iter()
        .find_map(|(column, target)| column.eq_ignore_ascii_case(source).then_some(target))
}

fn cell_to_body(value: &CellValue) -> Result<AnyValue, OtlpMappingError> {
    // OTLP KeyValueList is the typed representation of the required JSON
    // object body. JSON cells can therefore remain structured in the body.
    if let CellValue::Json(value) = value {
        return json_to_any(&serde_json::from_str(value)?);
    }
    cell_to_any(value)
}

fn cell_to_any(value: &CellValue) -> Result<AnyValue, OtlpMappingError> {
    // Precision-sensitive SQL values remain text unless OTLP has an exact
    // scalar representation. In particular, OTLP has no unsigned integer or
    // decimal attribute type.
    Ok(match value {
        CellValue::Null => AnyValue::default(),
        CellValue::Bool(value) => AnyValue {
            value: Some(any_value::Value::BoolValue(*value)),
        },
        CellValue::Int64(value) => int_value(*value),
        CellValue::UInt64(value) => i64::try_from(*value)
            .map(int_value)
            .unwrap_or_else(|_| string_value(value.to_string())),
        CellValue::Float64(value) if value.is_finite() => AnyValue {
            value: Some(any_value::Value::DoubleValue(*value)),
        },
        CellValue::Float64(_) => return Err(OtlpMappingError::NonFiniteFloat),
        CellValue::Bytes(value) => {
            string_value(base64::engine::general_purpose::STANDARD.encode(value))
        }
        CellValue::Decimal(value)
        | CellValue::String(value)
        | CellValue::Date(value)
        | CellValue::Timestamp(value)
        | CellValue::TimestampTz(value)
        | CellValue::Interval(value)
        | CellValue::Json(value)
        | CellValue::Uuid(value) => string_value(value),
    })
}

fn json_to_any(value: &serde_json::Value) -> Result<AnyValue, OtlpMappingError> {
    Ok(match value {
        serde_json::Value::Null => AnyValue::default(),
        serde_json::Value::Bool(value) => AnyValue {
            value: Some(any_value::Value::BoolValue(*value)),
        },
        serde_json::Value::String(value) => string_value(value),
        serde_json::Value::Number(value) if value.is_i64() => {
            int_value(value.as_i64().ok_or(OtlpMappingError::InvalidJsonNumber)?)
        }
        serde_json::Value::Number(value) if value.is_u64() => {
            let value = value.as_u64().ok_or(OtlpMappingError::InvalidJsonNumber)?;
            i64::try_from(value)
                .map(int_value)
                .unwrap_or_else(|_| string_value(value.to_string()))
        }
        serde_json::Value::Number(value) => AnyValue {
            value: Some(any_value::Value::DoubleValue(
                value
                    .as_f64()
                    .filter(|value| value.is_finite())
                    .ok_or(OtlpMappingError::InvalidJsonNumber)?,
            )),
        },
        serde_json::Value::Array(values) => AnyValue {
            value: Some(any_value::Value::ArrayValue(
                values
                    .iter()
                    .map(json_to_any)
                    .collect::<Result<Vec<_>, _>>()?
                    .into(),
            )),
        },
        serde_json::Value::Object(values) => AnyValue {
            value: Some(any_value::Value::KvlistValue(KeyValueList {
                values: values
                    .iter()
                    .map(|(key, value)| {
                        Ok(KeyValue {
                            key: key.clone(),
                            value: Some(json_to_any(value)?),
                        })
                    })
                    .collect::<Result<Vec<_>, OtlpMappingError>>()?,
            })),
        },
    })
}

fn parse_event_time(value: &CellValue, column: &str) -> Result<Option<u64>, OtlpMappingError> {
    // Timezone-aware text is normalized by chrono. Naive values are interpreted
    // as UTC because Oracle sessions are configured to UTC by the adapter.
    let text = match value {
        CellValue::Date(value)
        | CellValue::Timestamp(value)
        | CellValue::TimestampTz(value)
        | CellValue::String(value) => value,
        _ => {
            return Err(OtlpMappingError::InvalidEventTimeType {
                column: column.to_owned(),
            });
        }
    };
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(text) {
        return timestamp_to_nanos(timestamp.with_timezone(&Utc), column);
    }
    let timestamp = NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f"))
        .or_else(|_| {
            NaiveDate::parse_from_str(text, "%Y-%m-%d").map(|date| date.and_time(NaiveTime::MIN))
        })
        .map_err(|source| OtlpMappingError::InvalidEventTime {
            column: column.to_owned(),
            source,
        })?
        .and_utc();
    timestamp_to_nanos(timestamp, column)
}

fn timestamp_to_nanos(
    timestamp: DateTime<Utc>,
    _column: &str,
) -> Result<Option<u64>, OtlpMappingError> {
    // Oracle supports a much wider year range than OTLP's unsigned nanosecond
    // timestamp. Preserve the source timestamp in the body and fall back to
    // observation time instead of making one valid row poison the checkpoint.
    Ok(timestamp
        .timestamp_nanos_opt()
        .and_then(|nanos| u64::try_from(nanos).ok()))
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
    /// JSON source text is invalid.
    #[error("JSON row mapping failed")]
    Json(#[from] serde_json::Error),
    /// A JSON number cannot be represented.
    #[error("JSON number is outside the supported OTLP range")]
    InvalidJsonNumber,
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
