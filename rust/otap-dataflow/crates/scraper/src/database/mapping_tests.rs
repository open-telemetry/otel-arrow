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

/// Scenario: One encoder handles several pages and an empty poll while earlier batches stay alive.
/// Guarantees: Records, observation times, byte counts, and cursors belong only to their own page;
/// subsequent encoding does not overwrite payloads already handed downstream.
#[test]
fn reused_encoder_keeps_pages_and_downstream_payloads_independent() {
    let columns = vec![column("PAYLOAD", "VARCHAR2")];
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "oracle-audit".to_owned(),
        OutputConfig::default(),
        columns.clone(),
    )
    .expect("valid mapping");
    let mut batches = Vec::new();
    for (iteration, count) in [3, 1, 2].into_iter().enumerate() {
        let mut input = page(
            columns.clone(),
            (0..count)
                .map(|index| Row {
                    values: vec![CellValue::String(format!("page-{iteration}-row-{index}"))],
                })
                .collect(),
        );
        for (index, row) in input.rows.iter_mut().enumerate() {
            row.cursor = cursor((iteration * 10 + index) as i64);
        }
        let observed = 100 + iteration as u64;
        let encoded = encoder
            .encode_page(input, observed, UNLIMITED_BYTES)
            .expect("page encodes")
            .expect("nonempty page");
        assert_eq!(encoded.row_count, count);
        assert_eq!(encoded.deferred_rows, 0);
        assert_eq!(encoded.event_time_fallbacks, 0);
        assert_eq!(
            encoded.candidate,
            cursor((iteration * 10 + count - 1) as i64)
        );
        batches.push((encoded, observed, count));
        assert!(
            encoder
                .encode_page(page(columns.clone(), Vec::new()), 999, UNLIMITED_BYTES)
                .expect("empty poll encodes")
                .is_none()
        );
    }
    for (iteration, (encoded, observed, count)) in batches.into_iter().enumerate() {
        let encoded_bytes = encoded.encoded_bytes;
        let logs = decode(encoded);
        assert_eq!(encoded_bytes, logs.encoded_len());
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), count);
        for (index, record) in records.iter().enumerate() {
            assert_eq!(record.observed_time_unix_nano, observed);
            assert_eq!(record.time_unix_nano, observed);
            assert!(matches!(
                field_value(record, "PAYLOAD"),
                any_value::Value::StringValue(value)
                    if value == &format!("page-{iteration}-row-{index}")
            ));
        }
    }
}

/// Scenario: Live metadata reorders the event-time column and then renames a body column.
/// Guarantees: Recompiled plans use the current timestamp index and body keys, including when
/// the original schema returns after a different schema was cached.
#[test]
fn reused_encoder_recompiles_reordered_and_renamed_columns() {
    let original = vec![
        column("EVENT_TIME", "TIMESTAMP"),
        column("PAYLOAD", "VARCHAR2"),
    ];
    let output = OutputConfig {
        timestamp_column: Some("event_time".to_owned()),
        ..OutputConfig::default()
    };
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "oracle-audit".to_owned(),
        output,
        original.clone(),
    )
    .expect("valid mapping");
    for (columns, key, reversed) in [
        (original.clone(), "PAYLOAD", false),
        (
            vec![original[1].clone(), original[0].clone()],
            "PAYLOAD",
            true,
        ),
        (
            vec![column("DETAILS", "VARCHAR2"), original[0].clone()],
            "DETAILS",
            true,
        ),
        (original, "PAYLOAD", false),
    ] {
        let mut values = vec![
            CellValue::Timestamp("2026-08-28T00:00:00".to_owned()),
            CellValue::String("not a timestamp".to_owned()),
        ];
        if reversed {
            values.reverse();
        }
        let encoded = encoder
            .encode_page(page(columns, vec![Row { values }]), 123, UNLIMITED_BYTES)
            .expect("changed schema encodes")
            .expect("one row");
        let logs = decode(encoded);
        let record = &logs.resource_logs[0].scope_logs[0].log_records[0];
        assert_eq!(record.time_unix_nano, 1_787_875_200_000_000_000);
        assert!(matches!(
            field_value(record, key),
            any_value::Value::StringValue(value) if value == "not a timestamp"
        ));
        assert!(matches!(
            field_value(record, "EVENT_TIME"),
            any_value::Value::StringValue(value) if value == "2026-08-28T00:00:00"
        ));
        let Some(any_value::Value::KvlistValue(body)) =
            record.body.as_ref().and_then(|body| body.value.as_ref())
        else {
            panic!("expected body");
        };
        assert_eq!(body.values.len(), 2);
        assert_eq!(body.values[usize::from(!reversed)].key, key);
    }
}

/// Scenario: A timestamp column changes only its source type to an unsupported numeric type.
/// Guarantees: Construction and schema refresh reject metadata before encoding even a valid
/// timestamp cell, and a rejected refresh leaves the last valid schema usable.
#[test]
fn reused_encoder_rejects_source_type_changes_before_row_conversion() {
    let columns = vec![column("EVENT_TIME", "TIMESTAMP")];
    let invalid = vec![column("EVENT_TIME", "NUMBER")];
    let output = OutputConfig {
        timestamp_column: Some("EVENT_TIME".to_owned()),
        ..OutputConfig::default()
    };
    assert!(matches!(
        OtlpPageEncoder::new(
            DatabaseSystem::Oracle,
            "oracle-audit".to_owned(),
            output.clone(),
            invalid.clone(),
        ),
        Err(OtlpMappingError::InvalidEventTimeMetadata { .. })
    ));
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "oracle-audit".to_owned(),
        output,
        columns.clone(),
    )
    .expect("valid mapping");
    let rows = vec![Row {
        values: vec![CellValue::Timestamp("2026-08-28T00:00:00".to_owned())],
    }];
    for bad_rows in [Vec::new(), rows.clone()] {
        assert!(matches!(
            encoder.encode_page(page(invalid.clone(), bad_rows), 123, UNLIMITED_BYTES),
            Err(OtlpMappingError::InvalidEventTimeMetadata { column, source_type })
                if column == "EVENT_TIME" && source_type == "NUMBER"
        ));
        let recovered = encoder
            .encode_page(page(columns.clone(), rows.clone()), 456, UNLIMITED_BYTES)
            .expect("original schema remains valid")
            .expect("one row");
        let logs = decode(recovered);
        assert_eq!(logs.resource_logs[0].scope_logs[0].log_records.len(), 1);
        assert_eq!(
            logs.resource_logs[0].scope_logs[0].log_records[0].time_unix_nano,
            1_787_875_200_000_000_000
        );
    }
}

/// Scenario: A refreshed schema duplicates ASCII-folded names or loses a configured column.
/// Guarantees: Both initial and refreshed schemas are validated, and rejected schemas cannot
/// poison a subsequent page using the original timestamp and validation columns.
#[test]
fn reused_encoder_recovers_after_duplicate_and_missing_columns() {
    let columns = vec![
        column("EVENT_TIME", "TIMESTAMP"),
        column("PAYLOAD", "VARCHAR2"),
    ];
    let output = OutputConfig {
        timestamp_column: Some("event_time".to_owned()),
        validation_columns: vec!["payload".to_owned()],
    };
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "oracle-audit".to_owned(),
        output.clone(),
        columns.clone(),
    )
    .expect("valid mapping");
    for (invalid, missing) in [
        (
            vec![
                columns[0].clone(),
                columns[1].clone(),
                column("payload", "VARCHAR2"),
            ],
            None,
        ),
        (vec![columns[0].clone()], Some("payload")),
        (vec![columns[1].clone()], Some("event_time")),
    ] {
        let startup = OtlpPageEncoder::new(
            DatabaseSystem::Oracle,
            "oracle-audit".to_owned(),
            output.clone(),
            invalid.clone(),
        )
        .err()
        .expect("invalid constructor metadata");
        let refresh = encoder
            .encode_page(page(invalid, Vec::new()), 123, UNLIMITED_BYTES)
            .expect_err("invalid refreshed metadata");
        for error in [startup, refresh] {
            match missing {
                Some(expected) => assert!(matches!(
                    error,
                    OtlpMappingError::UnknownColumn { name } if name == expected
                )),
                None => assert!(matches!(error, OtlpMappingError::DuplicateColumn { .. })),
            }
        }
        let recovered = encoder
            .encode_page(
                page(
                    columns.clone(),
                    vec![Row {
                        values: vec![
                            CellValue::Timestamp("2026-08-28T00:00:00".to_owned()),
                            CellValue::String("recovered".to_owned()),
                        ],
                    }],
                ),
                456,
                UNLIMITED_BYTES,
            )
            .expect("original schema survives")
            .expect("one row");
        let logs = decode(recovered);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].time_unix_nano, 1_787_875_200_000_000_000);
        assert!(matches!(
            field_value(&records[0], "PAYLOAD"),
            any_value::Value::StringValue(value) if value == "recovered"
        ));
    }
}

/// Scenario: A column changes only nullability, and null event times alternate with valid times.
/// Guarantees: Nullable schema refreshes succeed, null timestamps use the current observation
/// time without counting as out-of-range fallbacks, and event-time state never leaks across pages.
#[test]
fn reused_encoder_accepts_nullable_changes_and_null_timestamp_fallback() {
    let mut columns = vec![column("EVENT_TIME", "TIMESTAMP")];
    columns[0].nullable = false;
    let output = OutputConfig {
        timestamp_column: Some("EVENT_TIME".to_owned()),
        ..OutputConfig::default()
    };
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "oracle-audit".to_owned(),
        output,
        columns.clone(),
    )
    .expect("valid mapping");
    for (nullable, observed, value, expected, fallbacks) in [
        (
            false,
            100,
            CellValue::Timestamp("9999-12-31T23:59:59.999999999".to_owned()),
            100,
            1,
        ),
        (true, 200, CellValue::Null, 200, 0),
        (
            false,
            300,
            CellValue::Timestamp("2026-08-28T00:00:00".to_owned()),
            1_787_875_200_000_000_000,
            0,
        ),
        (true, 400, CellValue::Null, 400, 0),
    ] {
        columns[0].nullable = nullable;
        let is_null = matches!(value, CellValue::Null);
        let encoded = encoder
            .encode_page(
                page(
                    columns.clone(),
                    vec![Row {
                        values: vec![value],
                    }],
                ),
                observed,
                UNLIMITED_BYTES,
            )
            .expect("nullability refresh encodes")
            .expect("one row");
        assert_eq!(encoded.event_time_fallbacks, fallbacks);
        let logs = decode(encoded);
        let record = &logs.resource_logs[0].scope_logs[0].log_records[0];
        assert_eq!(record.time_unix_nano, expected);
        assert_eq!(record.observed_time_unix_nano, observed);
        assert_eq!(
            body_field(record, "EVENT_TIME")
                .value
                .as_ref()
                .expect("AnyValue")
                .value
                .is_none(),
            is_null
        );
    }
}

/// Scenario: A valid row precedes a width error or non-finite float, or the first row is oversized.
/// Guarantees: Every failed call discards partially mapped records, and the next call has only
/// its own record, cursor, timestamps, and exact serialized size.
#[test]
fn reused_encoder_discards_partial_rows_after_mapping_and_size_errors() {
    let columns = vec![column("VALUE", "BINARY_DOUBLE")];
    let output = OutputConfig::default();
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "oracle-audit".to_owned(),
        output.clone(),
        columns.clone(),
    )
    .expect("valid mapping");
    let good = Row {
        values: vec![CellValue::Float64(1.5)],
    };
    for bad in [
        Row { values: Vec::new() },
        Row {
            values: vec![CellValue::Float64(1.0), CellValue::Float64(2.0)],
        },
        Row {
            values: vec![CellValue::Float64(f64::NAN)],
        },
        Row {
            values: vec![CellValue::Float64(f64::INFINITY)],
        },
        Row {
            values: vec![CellValue::Float64(f64::NEG_INFINITY)],
        },
    ] {
        let width_error = bad.values.len() != columns.len();
        let error = encoder
            .encode_page(
                page(columns.clone(), vec![good.clone(), bad]),
                100,
                UNLIMITED_BYTES,
            )
            .expect_err("invalid second row fails the whole page");
        if width_error {
            assert!(matches!(error, OtlpMappingError::ColumnCount));
        } else {
            assert!(matches!(error, OtlpMappingError::NonFiniteFloat));
        }
        let recovered = encoder
            .encode_page(
                page(columns.clone(), vec![good.clone()]),
                123,
                UNLIMITED_BYTES,
            )
            .expect("next page recovers")
            .expect("one row");
        assert_eq!(recovered.row_count, 1);
        assert_eq!(recovered.candidate, cursor(0));
        let bytes = recovered.encoded_bytes;
        let expected = decode(encode(
            page(columns.clone(), vec![good.clone()]),
            &output,
            UNLIMITED_BYTES,
        ));
        let actual = decode(recovered);
        assert_eq!(bytes, actual.encoded_len());
        assert_eq!(actual, expected);
    }
    let expected = encode(
        page(columns.clone(), vec![good.clone()]),
        &output,
        UNLIMITED_BYTES,
    );
    let limit = expected.encoded_bytes as u64 - 1;
    assert!(matches!(
        encoder.encode_page(page(columns.clone(), vec![good.clone()]), 123, limit),
        Err(OtlpMappingError::OversizedFirstRow { encoded_bytes, limit: actual_limit })
            if encoded_bytes == expected.encoded_bytes && actual_limit == limit
    ));
    let recovered = encoder
        .encode_page(page(columns, vec![good]), 123, UNLIMITED_BYTES)
        .expect("oversized failure does not poison next page")
        .expect("one row");
    assert_eq!(recovered.row_count, 1);
    assert_eq!(recovered.deferred_rows, 0);
    assert_eq!(recovered.candidate, cursor(0));
    assert_eq!(recovered.encoded_bytes, expected.encoded_bytes);
    assert_eq!(decode(recovered), decode(expected));
}

/// Scenario: Payload lengths straddle protobuf varint boundaries on repeatedly reused encoders.
/// Guarantees: Prost-decoded lengths define exact inclusive byte ceilings; one byte less rejects
/// the first row or emits precisely the preceding prefix with its own cursor and deferred count.
#[test]
fn reused_encoder_preserves_exact_varint_boundary_prefixes() {
    let columns = vec![column("PAYLOAD", "VARCHAR2")];
    let output = OutputConfig::default();
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "oracle-audit".to_owned(),
        output.clone(),
        columns.clone(),
    )
    .expect("valid mapping");
    for payload_len in [127, 128, 16_383, 16_384] {
        let mut input = page(
            columns.clone(),
            (0..3)
                .map(|index| Row {
                    values: vec![CellValue::String(
                        char::from(b'a' + index).to_string().repeat(payload_len),
                    )],
                })
                .collect(),
        );
        for (index, row) in input.rows.iter_mut().enumerate() {
            row.cursor = cursor((payload_len + index) as i64);
        }
        let mut expected = Vec::new();
        for count in 1..=3 {
            let mut prefix = input.clone();
            prefix.rows.truncate(count);
            let encoded = encode(prefix, &output, UNLIMITED_BYTES);
            let encoded_bytes = encoded.encoded_bytes;
            let logs = decode(encoded);
            assert_eq!(encoded_bytes, logs.encoded_len());
            expected.push(logs);
        }
        for count in [3, 1, 2, 3] {
            let exact = expected[count - 1].encoded_len() as u64;
            let encoded = encoder
                .encode_page(input.clone(), 123, exact)
                .expect("exact ceiling accepts prefix")
                .expect("nonempty prefix");
            assert_eq!(encoded.row_count, count);
            assert_eq!(encoded.deferred_rows, 3 - count);
            assert_eq!(encoded.encoded_bytes as u64, exact);
            assert_eq!(encoded.candidate, input.rows[count - 1].cursor);
            assert_eq!(decode(encoded), expected[count - 1]);
            let below = encoder.encode_page(input.clone(), 123, exact - 1);
            if count == 1 {
                assert!(matches!(
                    below,
                    Err(OtlpMappingError::OversizedFirstRow { encoded_bytes, limit })
                        if encoded_bytes as u64 == exact && limit == exact - 1
                ));
            } else {
                let encoded = below
                    .expect("smaller prefix fits")
                    .expect("nonempty smaller prefix");
                assert_eq!(encoded.row_count, count - 1);
                assert_eq!(encoded.deferred_rows, 4 - count);
                assert_eq!(encoded.encoded_bytes, expected[count - 2].encoded_len());
                assert_eq!(encoded.candidate, input.rows[count - 2].cursor);
                assert_eq!(decode(encoded), expected[count - 2]);
            }
        }
    }
}

/// Scenario: A manual run compares warmed persistent encoding to the same optimized one-shot API.
/// Guarantees: Bounded narrow/wide fixtures produce equal payloads and counts; debug timings
/// isolate cache reuse, not old-branch performance or production throughput, with no speed gate.
#[test]
#[ignore = "manual cache-reuse microbenchmark; debug timings are not production throughput"]
fn compare_warmed_encoder_with_optimized_one_shot() {
    use std::hint::black_box;
    use std::time::Instant;

    const ITERATIONS: usize = 32;
    const ROWS: usize = 16;
    for width in [16, 256] {
        let columns: Vec<_> = (0..width)
            .map(|index| column(&format!("COLUMN_{index}"), "VARCHAR2"))
            .collect();
        let input = page(
            columns.clone(),
            (0..ROWS)
                .map(|_| Row {
                    values: (0..width)
                        .map(|_| CellValue::String("bounded payload".to_owned()))
                        .collect(),
                })
                .collect(),
        );
        let output = OutputConfig::default();
        let mut encoder = OtlpPageEncoder::new(
            DatabaseSystem::Oracle,
            "oracle-audit".to_owned(),
            output.clone(),
            columns,
        )
        .expect("valid mapping");
        let warmed = encoder
            .encode_page(input.clone(), 123, UNLIMITED_BYTES)
            .expect("warmup")
            .expect("rows");
        let baseline = encode(input.clone(), &output, UNLIMITED_BYTES);
        assert_eq!(warmed.row_count, baseline.row_count);
        assert_eq!(warmed.encoded_bytes, baseline.encoded_bytes);
        assert_eq!(decode(warmed), decode(baseline));

        let cached_pages = vec![input.clone(); ITERATIONS];
        let one_shot_pages = vec![input; ITERATIONS];
        let mut cached_counts = (0, 0);
        let start = Instant::now();
        for page in cached_pages {
            let encoded = black_box(
                encoder
                    .encode_page(black_box(page), 123, UNLIMITED_BYTES)
                    .expect("cached page")
                    .expect("rows"),
            );
            cached_counts.0 += encoded.row_count;
            cached_counts.1 += encoded.encoded_bytes;
        }

        let cached_elapsed = start.elapsed();
        let mut one_shot_counts = (0, 0);
        let start = Instant::now();
        for page in one_shot_pages {
            let encoded = black_box(encode(black_box(page), &output, UNLIMITED_BYTES));
            one_shot_counts.0 += encoded.row_count;
            one_shot_counts.1 += encoded.encoded_bytes;
        }
        let one_shot_elapsed = start.elapsed();
        assert_eq!(cached_counts, one_shot_counts);
        assert_eq!(cached_counts.0, ITERATIONS * ROWS);
        eprintln!(
            "cache reuse only: columns={width}, rows={ROWS}, iterations={ITERATIONS}, \
             cached={cached_elapsed:?}, optimized_one_shot={one_shot_elapsed:?}; \
             debug measurements are not production throughput"
        );
    }
}

/// Scenario: An isolated manual run encodes 10,000 large owned rows near the maximum page byte limit.
/// Guarantees: All rows fit the separate normalized/OTLP bounds, their final cursor is preserved,
/// and phase markers let an external sampler measure process peak memory without retaining a cloned input page.
#[test]
#[ignore = "manual large-page memory profile; run alone in a fresh process"]
fn profile_large_owned_page_memory() {
    use std::hint::black_box;
    use std::io::Write;
    use std::time::{Duration, Instant};

    const ROWS: usize = 10_000;
    const VALUE_BYTES: usize = 24 * 1024;
    const LIMIT: u64 = 256 * 1024 * 1024;

    println!("memory_profile phase=baseline rows={ROWS} value_bytes={VALUE_BYTES} limit={LIMIT}");
    std::io::stdout().flush().expect("flush baseline marker");
    std::thread::sleep(Duration::from_millis(200));
    let input = page(
        vec![column("PAYLOAD", "VARCHAR2")],
        (0..ROWS)
            .map(|_| Row {
                values: vec![CellValue::String("x".repeat(VALUE_BYTES))],
            })
            .collect(),
    );
    let normalized_bytes: u64 = input.rows.iter().map(|row| row.row.normalized_size()).sum();
    assert!(normalized_bytes <= LIMIT);
    println!("memory_profile phase=input_ready normalized_bytes={normalized_bytes}");
    std::io::stdout().flush().expect("flush input marker");
    std::thread::sleep(Duration::from_millis(200));
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "profile-source".to_owned(),
        OutputConfig::default(),
        input.columns.clone(),
    )
    .expect("profile encoder");
    let started = Instant::now();
    let encoded = encoder
        .encode_page(input, 123, LIMIT)
        .expect("bounded large page")
        .expect("nonempty large page");
    assert_eq!(encoded.row_count, ROWS);
    assert_eq!(encoded.deferred_rows, 0);
    assert_eq!(encoded.candidate, cursor(ROWS as i64 - 1));
    assert!(encoded.encoded_bytes as u64 <= LIMIT);
    println!(
        "memory_profile phase=encoded rows={} encoded_bytes={} elapsed_ms={}",
        encoded.row_count,
        encoded.encoded_bytes,
        started.elapsed().as_millis(),
    );
    std::io::stdout().flush().expect("flush encoded marker");
    // Keep the output live long enough for an external process-memory sample.
    std::thread::sleep(Duration::from_millis(200));
    drop(black_box(encoded));
}
