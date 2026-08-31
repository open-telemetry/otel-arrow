// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_pdata::PayloadData;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData};
use prost::Message;
use std::collections::BTreeMap;
use std::time::Duration;

fn column(name: &str, source_type: &str) -> ColumnMetadata {
    ColumnMetadata {
        name: name.to_owned(),
        source_type: source_type.to_owned(),
        nullable: true,
    }
}

fn decode(result: QueryResult, output: &OutputConfig) -> LogsData {
    let pdata = rows_to_pdata(result, DatabaseSystem::Oracle, "oracle-audit", output, 123)
        .expect("database rows should map");
    let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
        pdata.payload().into_data()
    else {
        panic!("expected OTLP logs bytes");
    };
    LogsData::decode(bytes).expect("OTLP logs should decode")
}

fn body_field<'a>(record: &'a LogRecord, key: &str) -> &'a KeyValue {
    let Some(any_value::Value::KvlistValue(body)) =
        record.body.as_ref().and_then(|body| body.value.as_ref())
    else {
        panic!("expected KeyValueList body");
    };
    body.values
        .iter()
        .find(|field| field.key == key)
        .expect("body field should exist")
}

fn field_value<'a>(record: &'a LogRecord, key: &str) -> &'a any_value::Value {
    body_field(record, key)
        .value
        .as_ref()
        .and_then(|value| value.value.as_ref())
        .expect("body field should have a value")
}

/// Scenario: One row contains every value in the closed CellValue model.
/// Guarantees: OTLP preserves the exact scalar mapping, structured JSON body, resource and scope
/// identity, one-row-per-record cardinality, and configured event and observation timestamps.
#[test]
fn maps_the_complete_cell_value_contract_to_otlp() {
    let columns = vec![
        column("NULL_VALUE", "VARCHAR2"),
        column("BOOL_VALUE", "BOOLEAN"),
        column("INT_VALUE", "NUMBER"),
        column("UINT_VALUE", "NUMBER"),
        column("LARGE_UINT", "NUMBER"),
        column("DECIMAL_VALUE", "NUMBER"),
        column("FLOAT_VALUE", "BINARY_DOUBLE"),
        column("STRING_VALUE", "VARCHAR2"),
        column("BYTES_VALUE", "RAW"),
        column("DATE_VALUE", "DATE"),
        column("TIMESTAMP_VALUE", "TIMESTAMP"),
        column("TIMESTAMP_TZ_VALUE", "TIMESTAMP WITH TIME ZONE"),
        column("INTERVAL_VALUE", "INTERVAL DAY TO SECOND"),
        column("JSON_VALUE", "JSON"),
        column("UUID_VALUE", "VARCHAR2"),
    ];
    let result = QueryResult {
        columns,
        rows: vec![Row {
            values: vec![
                CellValue::Null,
                CellValue::Bool(true),
                CellValue::Int64(-42),
                CellValue::UInt64(42),
                CellValue::UInt64(u64::MAX),
                CellValue::Decimal("1.234567890123456789".to_owned()),
                CellValue::Float64(1.5),
                CellValue::String("text".to_owned()),
                CellValue::Bytes(vec![0, 1, 2]),
                CellValue::Date("2026-08-28".to_owned()),
                CellValue::Timestamp("2026-08-28T12:30:00".to_owned()),
                CellValue::TimestampTz("2026-08-28T12:30:00+00:00".to_owned()),
                CellValue::Interval("+01 02:03:04".to_owned()),
                CellValue::Json(r#"{"enabled":true}"#.to_owned()),
                CellValue::Uuid("123e4567-e89b-12d3-a456-426614174000".to_owned()),
            ],
        }],
    };
    let output = OutputConfig {
        attributes: BTreeMap::from([
            ("NULL_VALUE".to_owned(), "test.null".to_owned()),
            ("BOOL_VALUE".to_owned(), "test.bool".to_owned()),
            ("LARGE_UINT".to_owned(), "test.large_uint".to_owned()),
        ]),
        timestamp_column: Some("TIMESTAMP_TZ_VALUE".to_owned()),
        ..OutputConfig::default()
    };

    let logs = decode(result, &output);
    let resource_logs = &logs.resource_logs[0];
    let resource = resource_logs.resource.as_ref().expect("resource");
    assert!(resource.attributes.iter().any(|attribute| {
        attribute.key == "db.system.name"
            && matches!(
                attribute.value.as_ref().and_then(|value| value.value.as_ref()),
                Some(any_value::Value::StringValue(value)) if value == "oracle"
            )
    }));
    assert!(resource.attributes.iter().any(|attribute| {
        attribute.key == "receiver.database.source_id"
            && matches!(
                attribute.value.as_ref().and_then(|value| value.value.as_ref()),
                Some(any_value::Value::StringValue(value)) if value == "oracle-audit"
            )
    }));

    let scope_logs = &resource_logs.scope_logs[0];
    let scope = scope_logs.scope.as_ref().expect("scope");
    assert_eq!(scope.name, "otel-arrow.database_receiver");
    assert_eq!(scope.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(scope_logs.log_records.len(), 1);

    let record = &scope_logs.log_records[0];
    assert_eq!(record.observed_time_unix_nano, 123);
    assert_eq!(record.time_unix_nano, 1_787_920_200_000_000_000);
    assert!(
        body_field(record, "NULL_VALUE")
            .value
            .as_ref()
            .is_some_and(|value| value.value.is_none())
    );
    assert!(matches!(
        field_value(record, "BOOL_VALUE"),
        any_value::Value::BoolValue(true)
    ));
    assert!(matches!(
        field_value(record, "INT_VALUE"),
        any_value::Value::IntValue(-42)
    ));
    assert!(matches!(
        field_value(record, "UINT_VALUE"),
        any_value::Value::IntValue(42)
    ));
    assert!(matches!(
        field_value(record, "LARGE_UINT"),
        any_value::Value::StringValue(value) if value == &u64::MAX.to_string()
    ));
    assert!(matches!(
        field_value(record, "DECIMAL_VALUE"),
        any_value::Value::StringValue(value) if value == "1.234567890123456789"
    ));
    assert!(matches!(
        field_value(record, "FLOAT_VALUE"),
        any_value::Value::DoubleValue(value) if *value == 1.5
    ));
    assert!(matches!(
        field_value(record, "BYTES_VALUE"),
        any_value::Value::StringValue(value) if value == "AAEC"
    ));
    for (key, expected) in [
        ("STRING_VALUE", "text"),
        ("DATE_VALUE", "2026-08-28"),
        ("TIMESTAMP_VALUE", "2026-08-28T12:30:00"),
        ("TIMESTAMP_TZ_VALUE", "2026-08-28T12:30:00+00:00"),
        ("INTERVAL_VALUE", "+01 02:03:04"),
        ("UUID_VALUE", "123e4567-e89b-12d3-a456-426614174000"),
    ] {
        assert!(matches!(
            field_value(record, key),
            any_value::Value::StringValue(value) if value == expected
        ));
    }
    assert!(matches!(
        field_value(record, "JSON_VALUE"),
        any_value::Value::KvlistValue(value) if value.values[0].key == "enabled"
    ));
    assert!(
        !record
            .attributes
            .iter()
            .any(|attribute| attribute.key == "test.null")
    );
    assert!(record.attributes.iter().any(|attribute| {
        attribute.key == "test.bool"
            && matches!(
                attribute
                    .value
                    .as_ref()
                    .and_then(|value| value.value.as_ref()),
                Some(any_value::Value::BoolValue(true))
            )
    }));
    assert!(record.attributes.iter().any(|attribute| {
        attribute.key == "test.large_uint"
            && matches!(
                attribute.value.as_ref().and_then(|value| value.value.as_ref()),
                Some(any_value::Value::StringValue(value)) if value == &u64::MAX.to_string()
            )
    }));
}

/// Scenario: A normalized row contains a non-finite float.
/// Guarantees: Conversion fails the batch explicitly instead of emitting invalid OTLP data.
#[test]
fn rejects_non_finite_floats() {
    let result = QueryResult {
        columns: vec![column("VALUE", "BINARY_DOUBLE")],
        rows: vec![Row {
            values: vec![CellValue::Float64(f64::INFINITY)],
        }],
    };

    assert!(matches!(
        rows_to_pdata(
            result,
            DatabaseSystem::Oracle,
            "oracle-audit",
            &OutputConfig::default(),
            123
        ),
        Err(OtlpMappingError::NonFiniteFloat)
    ));
}

/// Scenario: Live metadata has a missing, duplicate, or non-temporal configured column.
/// Guarantees: Invalid query mappings fail startup validation before any row is ingested.
#[test]
fn rejects_invalid_live_metadata_mappings() {
    let columns = vec![
        column("AUDIT_ID", "NUMBER"),
        column("LAST_UPDATED", "TIMESTAMP WITH TIME ZONE"),
    ];
    let missing = OutputConfig {
        validation_columns: vec!["MISSING_ID".to_owned()],
        ..OutputConfig::default()
    };
    assert!(matches!(
        validate_mapping(&columns, &missing),
        Err(OtlpMappingError::UnknownColumn { name }) if name == "MISSING_ID"
    ));

    let non_temporal = OutputConfig {
        timestamp_column: Some("AUDIT_ID".to_owned()),
        ..OutputConfig::default()
    };
    assert!(matches!(
        validate_mapping(&columns, &non_temporal),
        Err(OtlpMappingError::InvalidEventTimeMetadata { column, .. })
            if column == "AUDIT_ID"
    ));

    let duplicate = vec![column("AUDIT_ID", "NUMBER"), column("audit_id", "NUMBER")];
    assert!(matches!(
        validate_mapping(&duplicate, &OutputConfig::default()),
        Err(OtlpMappingError::DuplicateColumn { .. })
    ));
}

/// Scenario: A date-only value is selected as the event-time column.
/// Guarantees: The receiver maps the date to midnight UTC instead of accepting metadata and then
/// failing every conversion batch.
#[test]
fn maps_date_only_event_time_to_midnight_utc() {
    let result = QueryResult {
        columns: vec![column("EVENT_DATE", "DATE")],
        rows: vec![Row {
            values: vec![CellValue::Date("2026-08-28".to_owned())],
        }],
    };
    let output = OutputConfig {
        timestamp_column: Some("EVENT_DATE".to_owned()),
        ..OutputConfig::default()
    };

    let logs = decode(result, &output);
    assert_eq!(
        logs.resource_logs[0].scope_logs[0].log_records[0].time_unix_nano,
        1_787_875_200_000_000_000
    );
}

/// Scenario: A query is modifying, locking, or not a directly validated SELECT.
/// Guarantees: The minimal foundation rejects unsafe SQL before connecting to a database.
#[test]
fn rejects_queries_outside_the_read_only_foundation() {
    let polling = || PollingConfig {
        interval: Duration::from_secs(1),
        timeout: Duration::from_secs(1),
        fetch_size: 10,
        max_rows_per_poll: 100,
    };
    for sql in [
        "DELETE FROM AUDIT_LOGS",
        "SELECT * FROM AUDIT_LOGS FOR UPDATE",
        "WITH rows AS (SELECT 1 FROM DUAL) SELECT * FROM rows",
    ] {
        assert!(matches!(
            CompiledQuery::compile(sql.to_owned(), polling(), OutputConfig::default()),
            Err(QueryError::NotReadOnly)
        ));
    }
}

/// Scenario: A driver page is configured larger than the complete poll ceiling.
/// Guarantees: Invalid fetch bounds fail configuration instead of defeating max_rows_per_poll.
#[test]
fn rejects_fetch_size_above_poll_limit() {
    let config = PollingConfig {
        interval: Duration::from_secs(1),
        timeout: Duration::from_secs(1),
        fetch_size: 101,
        max_rows_per_poll: 100,
    };

    assert_eq!(
        config.validate(),
        Err(ConfigError::FetchSizeExceedsRowLimit)
    );
}
