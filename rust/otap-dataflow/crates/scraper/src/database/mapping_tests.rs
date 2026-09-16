// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_pdata::PayloadData;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData};
use prost::Message;

const UNLIMITED_BYTES: u64 = 64 * 1024 * 1024;

fn column(name: &str, source_type: &str) -> ColumnMetadata {
    ColumnMetadata {
        name: name.to_owned(),
        source_type: source_type.to_owned(),
        nullable: true,
    }
}

fn cursor(tie_breaker: i64) -> CompositeCursor {
    CompositeCursor::new("2026-08-28 12:30:00".to_owned(), tie_breaker)
}

fn page(columns: Vec<ColumnMetadata>, rows: Vec<Row>) -> QueryPage {
    QueryPage {
        columns,
        rows: rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| CursorRow {
                row,
                cursor: cursor(index as i64),
            })
            .collect(),
    }
}

fn encode(page: QueryPage, output: &OutputConfig, max_batch_bytes: u64) -> EncodedPage {
    encode_page(
        page,
        DatabaseSystem::Oracle,
        "oracle-audit",
        output,
        123,
        max_batch_bytes,
    )
    .expect("database page should encode")
    .expect("page should contain rows")
}

fn decode(encoded: EncodedPage) -> LogsData {
    let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
        encoded.pdata.payload().into_data()
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
        column("TIMESTAMP_VALUE", "TIMESTAMP"),
        column("TIMESTAMP_TZ_VALUE", "TIMESTAMP WITH TIME ZONE"),
        column("INTERVAL_VALUE", "INTERVAL DAY TO SECOND"),
    ];
    let rows = vec![Row {
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
            CellValue::Timestamp("2026-08-28T12:30:00".to_owned()),
            CellValue::TimestampTz("2026-08-28T12:30:00+00:00".to_owned()),
            CellValue::Interval("+01 02:03:04".to_owned()),
        ],
    }];
    let output = OutputConfig {
        timestamp_column: Some("TIMESTAMP_TZ_VALUE".to_owned()),
        ..OutputConfig::default()
    };

    let encoded = encode(page(columns, rows), &output, UNLIMITED_BYTES);
    let logs = decode(encoded);
    let resource_logs = &logs.resource_logs[0];
    let resource = resource_logs.resource.as_ref().expect("resource");
    assert!(resource.attributes.iter().any(|attribute| {
        attribute.key == "db.system.name"
            && matches!(
                attribute.value.as_ref().and_then(|value| value.value.as_ref()),
                Some(any_value::Value::StringValue(value)) if value == "oracle.db"
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
        any_value::Value::BytesValue(value) if value == &[0, 1, 2]
    ));
    for (key, expected) in [
        ("STRING_VALUE", "text"),
        ("TIMESTAMP_VALUE", "2026-08-28T12:30:00"),
        ("TIMESTAMP_TZ_VALUE", "2026-08-28T12:30:00+00:00"),
        ("INTERVAL_VALUE", "+01 02:03:04"),
    ] {
        assert!(matches!(
            field_value(record, key),
            any_value::Value::StringValue(value) if value == expected
        ));
    }
}

/// Scenario: a page of rows is encoded with a generous byte ceiling.
/// Guarantees: the reported encoded size equals the exact serialized OTLP payload length and the
/// candidate is the cursor of the final encoded row, so byte accounting is never an estimate.
#[test]
fn reports_exact_encoded_size_and_final_candidate() {
    let rows = (0..4)
        .map(|index| Row {
            values: vec![CellValue::String(format!("row-{index}"))],
        })
        .collect();
    let columns = vec![column("PAYLOAD", "VARCHAR2")];

    let encoded = encode(
        page(columns, rows),
        &OutputConfig::default(),
        UNLIMITED_BYTES,
    );
    let row_count = encoded.row_count;
    let deferred_rows = encoded.deferred_rows;
    let encoded_bytes = encoded.encoded_bytes;
    let candidate = encoded.candidate.clone();
    let logs = decode(encoded);

    assert_eq!(row_count, 4);
    assert_eq!(deferred_rows, 0);
    assert_eq!(encoded_bytes, logs.encoded_len());
    assert_eq!(candidate, cursor(3));
}

/// Scenario: only part of a fetched page fits the configured encoded-byte ceiling.
/// Guarantees: the largest non-empty prefix is emitted, the candidate comes from the last row
/// actually emitted, and the remaining rows are deferred rather than dropped.
#[test]
fn emits_largest_fitting_prefix_and_defers_the_rest() {
    let rows: Vec<Row> = (0..10)
        .map(|index| Row {
            values: vec![CellValue::String(format!("row-{index:04}"))],
        })
        .collect();
    let columns = vec![column("PAYLOAD", "VARCHAR2")];
    let full = encode(
        page(columns.clone(), rows.clone()),
        &OutputConfig::default(),
        UNLIMITED_BYTES,
    );
    // Choose a ceiling strictly between the three-row and full-page sizes.
    let three_rows = encode(
        page(columns.clone(), rows[..3].to_vec()),
        &OutputConfig::default(),
        UNLIMITED_BYTES,
    );
    assert!(three_rows.encoded_bytes < full.encoded_bytes);

    let limited = encode(
        page(columns, rows),
        &OutputConfig::default(),
        three_rows.encoded_bytes as u64,
    );

    assert_eq!(limited.row_count, 3);
    assert_eq!(limited.deferred_rows, 7);
    assert_eq!(limited.encoded_bytes, three_rows.encoded_bytes);
    assert!(limited.encoded_bytes as u64 <= three_rows.encoded_bytes as u64);
    assert_eq!(limited.candidate, cursor(2));
}

/// Scenario: the first row of a page alone exceeds the encoded-byte ceiling.
/// Guarantees: encoding fails explicitly instead of emitting an empty page or silently skipping
/// the row, which would strand the cursor and lose data.
#[test]
fn oversized_first_row_fails_instead_of_being_skipped() {
    let rows = vec![Row {
        values: vec![CellValue::String("x".repeat(4096))],
    }];
    let columns = vec![column("PAYLOAD", "VARCHAR2")];

    assert!(matches!(
        encode_page(
            page(columns, rows),
            DatabaseSystem::Oracle,
            "oracle-audit",
            &OutputConfig::default(),
            123,
            64,
        ),
        Err(OtlpMappingError::OversizedFirstRow { .. })
    ));
}

/// Scenario: a poll returns no rows after the committed cursor.
/// Guarantees: encoding reports no page rather than an empty payload, so the receiver neither
/// emits an empty batch downstream nor produces a candidate cursor.
#[test]
fn empty_page_produces_no_batch() {
    let encoded = encode_page(
        page(vec![column("PAYLOAD", "VARCHAR2")], Vec::new()),
        DatabaseSystem::Oracle,
        "oracle-audit",
        &OutputConfig::default(),
        123,
        UNLIMITED_BYTES,
    )
    .expect("empty page should encode");

    assert!(encoded.is_none());
}

/// Scenario: A normalized row contains a non-finite float.
/// Guarantees: Conversion fails the batch explicitly instead of emitting invalid OTLP data.
#[test]
fn rejects_non_finite_floats() {
    let rows = vec![Row {
        values: vec![CellValue::Float64(f64::INFINITY)],
    }];

    assert!(matches!(
        encode_page(
            page(vec![column("VALUE", "BINARY_DOUBLE")], rows),
            DatabaseSystem::Oracle,
            "oracle-audit",
            &OutputConfig::default(),
            123,
            UNLIMITED_BYTES,
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

/// Scenario: A timestamp value is selected as the event-time column.
/// Guarantees: The receiver maps the value as UTC without depending on the host timezone.
#[test]
fn maps_timestamp_event_time_to_utc() {
    let rows = vec![Row {
        values: vec![CellValue::Timestamp("2026-08-28T00:00:00".to_owned())],
    }];
    let output = OutputConfig {
        timestamp_column: Some("EVENT_TIME".to_owned()),
        ..OutputConfig::default()
    };

    let encoded = encode(
        page(vec![column("EVENT_TIME", "TIMESTAMP")], rows),
        &output,
        UNLIMITED_BYTES,
    );
    let logs = decode(encoded);

    assert_eq!(
        logs.resource_logs[0].scope_logs[0].log_records[0].time_unix_nano,
        1_787_875_200_000_000_000
    );
}

/// Scenario: Native timestamp text includes a numeric offset and nanosecond precision.
/// Guarantees: Cursor ordering and event-time conversion use the same UTC instant without lexical comparison.
#[test]
fn native_timestamp_offsets_preserve_nanosecond_instants() {
    let expected = parse_utc_timestamp("2026-08-28T10:30:00.123456789Z").expect("UTC timestamp");
    for text in [
        "2026-08-28 12:30:00.123456789 +02:00",
        "2026-08-28T12:30:00.123456789+02:00",
        "2026-08-28 10:30:00.123456789",
    ] {
        assert_eq!(
            parse_utc_timestamp(text).expect("equivalent timestamp"),
            expected
        );
    }
}

/// Scenario: Oracle returns a valid timestamp outside OTLP's unsigned nanosecond range.
/// Guarantees: The raw timestamp remains in the structured body and the record uses observation
/// time instead of permanently poisoning composite-watermark progress.
#[test]
fn falls_back_to_observation_time_for_unrepresentable_event_time() {
    let rows = vec![Row {
        values: vec![CellValue::Timestamp(
            "9999-12-31T23:59:59.999999999".to_owned(),
        )],
    }];
    let output = OutputConfig {
        timestamp_column: Some("EVENT_TS".to_owned()),
        ..OutputConfig::default()
    };

    let encoded = encode(
        page(vec![column("EVENT_TS", "TIMESTAMP")], rows),
        &output,
        UNLIMITED_BYTES,
    );
    assert_eq!(encoded.event_time_fallbacks, 1);
    let logs = decode(encoded);
    let record = &logs.resource_logs[0].scope_logs[0].log_records[0];
    assert_eq!(record.time_unix_nano, 123);
    assert!(matches!(
        field_value(record, "EVENT_TS"),
        any_value::Value::StringValue(value)
            if value == "9999-12-31T23:59:59.999999999"
    ));
}
