// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared typed database-row to OTLP log conversion.

use super::{CellValue, ColumnMetadata, DatabaseSystem, OutputConfig, QueryResult};
use base64::Engine;
use chrono::{DateTime, NaiveDateTime, Utc};
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
        require_column(&available, column)?;
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

/// Converts one bounded result into one OTLP logs payload.
pub fn rows_to_pdata(
    result: QueryResult,
    system: DatabaseSystem,
    source_id: &str,
    output: &OutputConfig,
    observed_time_unix_nano: u64,
) -> Result<OtapPdata, OtlpMappingError> {
    validate_mapping(&result.columns, output)?;
    let included = normalized_names(&output.include_columns);
    let include_all = included.is_empty();
    let records = result
        .rows
        .iter()
        .map(|row| {
            if row.values.len() != result.columns.len() {
                return Err(OtlpMappingError::ColumnCount);
            }
            let mut body = Vec::with_capacity(result.columns.len());
            let mut attributes = vec![
                KeyValue {
                    key: SOURCE_ID_ATTRIBUTE.to_owned(),
                    value: Some(string_value(source_id)),
                },
                KeyValue {
                    key: QUERY_NAME_ATTRIBUTE.to_owned(),
                    // The Oracle foundation has one query per source, so its
                    // stable source_id is also the unambiguous query identity.
                    value: Some(string_value(source_id)),
                },
            ];
            let mut event_time = None;
            for (column, value) in result.columns.iter().zip(&row.values) {
                if include_all || included.contains(&column.name.to_ascii_lowercase()) {
                    body.push(KeyValue {
                        key: column.name.clone(),
                        value: Some(cell_to_body(value)?),
                    });
                }
                if let Some(attribute_name) = configured_value(&output.attributes, &column.name)
                    && !matches!(value, CellValue::Null)
                {
                    // Null attributes are omitted by policy; body fields retain
                    // an empty AnyValue so the row shape remains observable.
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
                    event_time = Some(parse_event_time(value, &column.name)?);
                }
            }
            Ok(LogRecord {
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
            })
        })
        .collect::<Result<Vec<_>, OtlpMappingError>>()?;
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
    let logs = LogsData {
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
    };
    let payload: OtapPayload =
        OtlpProtoBytes::ExportLogsRequest(logs.encode_to_vec().into()).into();
    Ok(OtapPdata::new_todo_context(payload))
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

fn parse_event_time(value: &CellValue, column: &str) -> Result<u64, OtlpMappingError> {
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
        .map_err(|source| OtlpMappingError::InvalidEventTime {
            column: column.to_owned(),
            source,
        })?
        .and_utc();
    timestamp_to_nanos(timestamp, column)
}

fn timestamp_to_nanos(timestamp: DateTime<Utc>, column: &str) -> Result<u64, OtlpMappingError> {
    let nanos =
        timestamp
            .timestamp_nanos_opt()
            .ok_or_else(|| OtlpMappingError::EventTimeOutOfRange {
                column: column.to_owned(),
            })?;
    u64::try_from(nanos).map_err(|_| OtlpMappingError::EventTimeOutOfRange {
        column: column.to_owned(),
    })
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
    /// Event time cannot fit the OTLP unsigned nanosecond field.
    #[error("event-time column '{column}' is outside the supported OTLP time range")]
    EventTimeOutOfRange {
        /// Configured result column.
        column: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::database::Row;
    use otel_arrow_dfe_pdata::PayloadData;

    fn columns() -> Vec<ColumnMetadata> {
        vec![
            ColumnMetadata {
                name: "AUDIT_ID".to_owned(),
                source_type: "NUMBER".to_owned(),
                nullable: false,
            },
            ColumnMetadata {
                name: "LAST_UPDATED".to_owned(),
                source_type: "TIMESTAMP WITH TIME ZONE".to_owned(),
                nullable: false,
            },
            ColumnMetadata {
                name: "USER_NAME".to_owned(),
                source_type: "VARCHAR2".to_owned(),
                nullable: true,
            },
        ]
    }

    /// Scenario: Oracle audit values are converted into the shared OTLP logs contract.
    /// Guarantees: One row becomes one typed LogRecord with database resource identity,
    /// component scope, configured event time, observation time, and a KeyValueList body.
    #[test]
    fn maps_database_row_to_typed_otlp_log() {
        let result = QueryResult {
            columns: columns(),
            rows: vec![Row {
                values: vec![
                    CellValue::Decimal("42".to_owned()),
                    CellValue::TimestampTz("2026-08-28T12:30:00.000000000+00:00".to_owned()),
                    CellValue::String("alice".to_owned()),
                ],
            }],
            normalized_bytes: 48,
        };
        let output = OutputConfig {
            timestamp_column: Some("LAST_UPDATED".to_owned()),
            validation_columns: vec!["AUDIT_ID".to_owned()],
            ..OutputConfig::default()
        };

        let pdata = rows_to_pdata(result, DatabaseSystem::Oracle, "oracle-audit", &output, 123)
            .expect("row should map");
        let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
            pdata.payload().into_data()
        else {
            panic!("expected OTLP logs bytes");
        };
        let logs = LogsData::decode(bytes).expect("logs should decode");
        let resource_logs = &logs.resource_logs[0];
        let resource = resource_logs.resource.as_ref().expect("resource");
        assert_eq!(resource.attributes[0].key, "db.system.name");
        assert!(matches!(
            resource.attributes[0]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(any_value::Value::StringValue(value)) if value == "oracle"
        ));
        let scope_logs = &resource_logs.scope_logs[0];
        assert_eq!(
            scope_logs.scope.as_ref().expect("scope").name,
            DATABASE_SCOPE
        );
        assert_eq!(
            scope_logs.scope.as_ref().expect("scope").version,
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(scope_logs.log_records.len(), 1);
        let record = &scope_logs.log_records[0];
        assert_eq!(record.observed_time_unix_nano, 123);
        assert_eq!(record.time_unix_nano, 1_787_920_200_000_000_000);
        assert!(matches!(
            record.body.as_ref().and_then(|body| body.value.as_ref()),
            Some(any_value::Value::KvlistValue(body)) if body.values.len() == 3
        ));
        assert!(record.attributes.iter().any(|attribute| {
            attribute.key == QUERY_NAME_ATTRIBUTE
                && matches!(
                    attribute.value.as_ref().and_then(|value| value.value.as_ref()),
                    Some(any_value::Value::StringValue(value)) if value == "oracle-audit"
                )
        }));
    }

    /// Scenario: The configured tie-breaker column is absent from live query metadata.
    /// Guarantees: Validation fails before any database row can be emitted.
    #[test]
    fn rejects_missing_required_column() {
        let output = OutputConfig {
            timestamp_column: Some("LAST_UPDATED".to_owned()),
            validation_columns: vec!["MISSING_ID".to_owned()],
            ..OutputConfig::default()
        };

        assert!(matches!(
            validate_mapping(&columns(), &output),
            Err(OtlpMappingError::UnknownColumn { name }) if name == "MISSING_ID"
        ));
    }

    /// Scenario: The configured event-time column resolves to numeric live metadata.
    /// Guarantees: Startup validation rejects the mapping before ingestion begins.
    #[test]
    fn rejects_non_temporal_event_time_metadata() {
        let output = OutputConfig {
            timestamp_column: Some("AUDIT_ID".to_owned()),
            validation_columns: Vec::new(),
            ..OutputConfig::default()
        };

        assert!(matches!(
            validate_mapping(&columns(), &output),
            Err(OtlpMappingError::InvalidEventTimeMetadata { column, .. })
                if column == "AUDIT_ID"
        ));
    }

    /// Scenario: Operators select body fields and promote typed attributes while one selected
    /// attribute contains SQL NULL.
    /// Guarantees: Mapping is case-insensitive, scalar types are preserved, null attributes are
    /// omitted, and the existing source identifier supplies safe resource and query identity.
    #[test]
    fn applies_configured_output_mapping() {
        let result = QueryResult {
            columns: columns(),
            rows: vec![Row {
                values: vec![
                    CellValue::Int64(42),
                    CellValue::TimestampTz("2026-08-28T12:30:00+00:00".to_owned()),
                    CellValue::Null,
                ],
            }],
            normalized_bytes: 34,
        };
        let output = OutputConfig {
            include_columns: vec!["audit_id".to_owned(), "USER_NAME".to_owned()],
            attributes: BTreeMap::from([
                ("audit_id".to_owned(), "audit.id".to_owned()),
                ("user_name".to_owned(), "user.name".to_owned()),
            ]),
            ..OutputConfig::default()
        };

        let pdata = rows_to_pdata(result, DatabaseSystem::Oracle, "oracle-audit", &output, 123)
            .expect("configured mapping should succeed");
        let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
            pdata.payload().into_data()
        else {
            panic!("expected OTLP logs bytes");
        };
        let logs = LogsData::decode(bytes).expect("logs should decode");
        let resource_logs = &logs.resource_logs[0];
        assert!(
            resource_logs
                .resource
                .as_ref()
                .expect("resource")
                .attributes
                .iter()
                .any(|attribute| attribute.key == SOURCE_ID_ATTRIBUTE)
        );

        let record = &resource_logs.scope_logs[0].log_records[0];
        assert_eq!(record.time_unix_nano, 123);
        let Some(any_value::Value::KvlistValue(body)) =
            record.body.as_ref().and_then(|body| body.value.as_ref())
        else {
            panic!("expected object body");
        };
        assert_eq!(body.values.len(), 2);
        assert!(body.values.iter().any(|field| field.key == "USER_NAME"));
        assert!(record.attributes.iter().any(|attribute| {
            attribute.key == "audit.id"
                && matches!(
                    attribute
                        .value
                        .as_ref()
                        .and_then(|value| value.value.as_ref()),
                    Some(any_value::Value::IntValue(42))
                )
        }));
        assert!(
            !record
                .attributes
                .iter()
                .any(|attribute| attribute.key == "user.name")
        );
    }

    /// Scenario: Every closed CellValue variant is converted to its required OTLP scalar form.
    /// Guarantees: Precision-sensitive values remain strings, bytes use base64, safe integers stay
    /// typed, overflowing unsigned integers become strings, and non-finite floats fail explicitly.
    #[test]
    fn preserves_cell_value_conversion_contract() {
        assert!(cell_to_any(&CellValue::Null).expect("null").value.is_none());
        assert!(matches!(
            cell_to_any(&CellValue::Bool(true)).expect("bool").value,
            Some(any_value::Value::BoolValue(true))
        ));
        assert!(matches!(
            cell_to_any(&CellValue::Int64(-42)).expect("int").value,
            Some(any_value::Value::IntValue(-42))
        ));
        assert!(matches!(
            cell_to_any(&CellValue::UInt64(42))
                .expect("safe uint")
                .value,
            Some(any_value::Value::IntValue(42))
        ));
        assert!(matches!(
            cell_to_any(&CellValue::UInt64(u64::MAX))
                .expect("large uint")
                .value,
            Some(any_value::Value::StringValue(value)) if value == u64::MAX.to_string()
        ));
        assert!(matches!(
            cell_to_any(&CellValue::Decimal("1.234567890123456789".to_owned()))
                .expect("decimal")
                .value,
            Some(any_value::Value::StringValue(value))
                if value == "1.234567890123456789"
        ));
        assert!(matches!(
            cell_to_any(&CellValue::Float64(1.5))
                .expect("finite float")
                .value,
            Some(any_value::Value::DoubleValue(value)) if value == 1.5
        ));
        assert!(matches!(
            cell_to_any(&CellValue::Bytes(vec![0, 1, 2]))
                .expect("bytes")
                .value,
            Some(any_value::Value::StringValue(value)) if value == "AAEC"
        ));
        for (cell, expected) in [
            (CellValue::String("text".to_owned()), "text"),
            (CellValue::Date("2026-08-28".to_owned()), "2026-08-28"),
            (
                CellValue::Timestamp("2026-08-28T12:30:00".to_owned()),
                "2026-08-28T12:30:00",
            ),
            (
                CellValue::TimestampTz("2026-08-28T12:30:00+00:00".to_owned()),
                "2026-08-28T12:30:00+00:00",
            ),
            (
                CellValue::Interval("+01 02:03:04".to_owned()),
                "+01 02:03:04",
            ),
            (
                CellValue::Json(r#"{"enabled":true}"#.to_owned()),
                r#"{"enabled":true}"#,
            ),
            (
                CellValue::Uuid("123e4567-e89b-12d3-a456-426614174000".to_owned()),
                "123e4567-e89b-12d3-a456-426614174000",
            ),
        ] {
            assert!(matches!(
                cell_to_any(&cell).expect("string-like value").value,
                Some(any_value::Value::StringValue(value)) if value == expected
            ));
        }
        assert!(matches!(
            cell_to_body(&CellValue::Json(r#"{"enabled":true}"#.to_owned()))
                .expect("JSON")
                .value,
            Some(any_value::Value::KvlistValue(_))
        ));
        assert!(matches!(
            cell_to_any(&CellValue::Float64(f64::INFINITY)),
            Err(OtlpMappingError::NonFiniteFloat)
        ));
    }

    /// Scenario: Live metadata contains two column names that differ only by case.
    /// Guarantees: The identity body mapping rejects duplicate names before ingestion.
    #[test]
    fn rejects_duplicate_result_column_names() {
        let columns = vec![
            ColumnMetadata {
                name: "AUDIT_ID".to_owned(),
                source_type: "NUMBER".to_owned(),
                nullable: false,
            },
            ColumnMetadata {
                name: "audit_id".to_owned(),
                source_type: "NUMBER".to_owned(),
                nullable: false,
            },
        ];

        assert!(matches!(
            validate_mapping(&columns, &OutputConfig::default()),
            Err(OtlpMappingError::DuplicateColumn { .. })
        ));
    }

    /// Scenario: A configured event-time column contains malformed timestamp text.
    /// Guarantees: Conversion returns an explicit row error instead of using observation time.
    #[test]
    fn rejects_malformed_event_time() {
        assert!(matches!(
            parse_event_time(
                &CellValue::Timestamp("not-a-timestamp".to_owned()),
                "LAST_UPDATED"
            ),
            Err(OtlpMappingError::InvalidEventTime { .. })
        ));
    }
}
