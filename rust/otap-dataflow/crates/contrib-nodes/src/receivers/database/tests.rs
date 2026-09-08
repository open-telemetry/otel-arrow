// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_pdata::PayloadData;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData};
use prost::Message;
use std::collections::BTreeMap;
use std::mem::size_of;
use std::time::Duration;

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

fn polling() -> PollingConfig {
    PollingConfig {
        interval: Duration::from_secs(1),
        timeout: Duration::from_secs(1),
        fetch_size: 10,
        max_rows_per_poll: 100,
        max_batch_bytes: 10 * 1024 * 1024,
        max_normalized_bytes: 10 * 1024 * 1024,
    }
}

fn watermark() -> WatermarkConfig {
    WatermarkConfig::Composite {
        timestamp: TimestampCursorConfig {
            column: "EVENT_TS".to_owned(),
            bind: "last_timestamp".to_owned(),
            initial: "1970-01-01 00:00:00".to_owned(),
            timezone: "UTC".to_owned(),
        },
        tie_breaker: TieBreakerCursorConfig {
            column: "EVENT_ID".to_owned(),
            bind: "last_tie_breaker".to_owned(),
            initial: 0,
        },
    }
}

fn checkpoint_config() -> CheckpointConfig {
    CheckpointConfig {
        directory: "${engine.state_dir}/oracle".to_owned(),
        on_nack: OnNack::Rewind,
        nack_backoff: Duration::from_secs(1),
        max_consecutive_failures: 5,
    }
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
            CellValue::Date("2026-08-28".to_owned()),
            CellValue::Timestamp("2026-08-28T12:30:00".to_owned()),
            CellValue::TimestampTz("2026-08-28T12:30:00+00:00".to_owned()),
            CellValue::Interval("+01 02:03:04".to_owned()),
            CellValue::Json(r#"{"enabled":true}"#.to_owned()),
            CellValue::Uuid("123e4567-e89b-12d3-a456-426614174000".to_owned()),
        ],
    }];
    let output = OutputConfig {
        attributes: BTreeMap::from([
            ("NULL_VALUE".to_owned(), "test.null".to_owned()),
            ("BOOL_VALUE".to_owned(), "test.bool".to_owned()),
            ("LARGE_UINT".to_owned(), "test.large_uint".to_owned()),
        ]),
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

/// Scenario: A date-only value is selected as the event-time column.
/// Guarantees: The receiver maps the date to midnight UTC instead of accepting metadata and then
/// failing every conversion batch.
#[test]
fn maps_date_only_event_time_to_midnight_utc() {
    let rows = vec![Row {
        values: vec![CellValue::Date("2026-08-28".to_owned())],
    }];
    let output = OutputConfig {
        timestamp_column: Some("EVENT_DATE".to_owned()),
        ..OutputConfig::default()
    };

    let encoded = encode(
        page(vec![column("EVENT_DATE", "DATE")], rows),
        &output,
        UNLIMITED_BYTES,
    );
    let logs = decode(encoded);

    assert_eq!(
        logs.resource_logs[0].scope_logs[0].log_records[0].time_unix_nano,
        1_787_875_200_000_000_000
    );
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

/// Scenario: A query is modifying, locking, or not a directly validated SELECT.
/// Guarantees: Unsafe SQL is rejected before any database connection is opened.
#[test]
fn rejects_queries_outside_the_read_only_contract() {
    for sql in [
        "DELETE FROM AUDIT_LOGS",
        "SELECT * FROM AUDIT_LOGS FOR UPDATE",
        "WITH rows AS (SELECT 1 FROM DUAL) SELECT * FROM rows",
    ] {
        assert!(matches!(
            CompiledQuery::compile(
                sql.to_owned(),
                polling(),
                &watermark(),
                &checkpoint_config(),
                OutputConfig::default(),
            ),
            Err(QueryError::NotReadOnly)
        ));
    }
}

/// Scenario: A driver page is configured larger than the complete poll ceiling.
/// Guarantees: Invalid fetch bounds fail configuration instead of defeating max_rows_per_poll.
#[test]
fn rejects_fetch_size_above_poll_limit() {
    let config = PollingConfig {
        fetch_size: 101,
        max_rows_per_poll: 100,
        ..polling()
    };

    assert_eq!(
        config.validate(),
        Err(ConfigError::FetchSizeExceedsRowLimit)
    );
}

/// Scenario: A poll is configured without a positive encoded or normalized byte budget.
/// Guarantees: Both the exact OTLP payload ceiling and the in-memory row ceiling are explicit and
/// non-zero, so neither bound can be silently disabled.
#[test]
fn requires_both_byte_limits() {
    assert_eq!(
        PollingConfig {
            max_batch_bytes: 0,
            ..polling()
        }
        .validate(),
        Err(ConfigError::ZeroBatchByteLimit)
    );
    assert_eq!(
        PollingConfig {
            max_normalized_bytes: 0,
            ..polling()
        }
        .validate(),
        Err(ConfigError::ZeroNormalizedByteLimit)
    );
}

/// Scenario: A polling limit exceeds the fixed result-row ceiling.
/// Guarantees: Misconfiguration cannot request an unbounded receiver allocation.
#[test]
fn rejects_excessive_polling_limit() {
    let config = PollingConfig {
        fetch_size: 10_000,
        max_rows_per_poll: 10_001,
        ..polling()
    };

    assert!(matches!(
        config.validate(),
        Err(ConfigError::RowLimit { maximum: 10_000 })
    ));
}

/// Scenario: A poll interval or byte ceiling is configured outside its supported range.
/// Guarantees: Operational bounds stay explicit and finite, so a single receiver cannot stall
/// indefinitely or request an arbitrarily large batch.
#[test]
fn rejects_out_of_range_operational_bounds() {
    assert!(matches!(
        PollingConfig {
            interval: Duration::from_secs(48 * 60 * 60),
            ..polling()
        }
        .validate(),
        Err(ConfigError::IntervalRange { .. })
    ));
    assert!(matches!(
        PollingConfig {
            max_batch_bytes: 512 * 1024 * 1024,
            ..polling()
        }
        .validate(),
        Err(ConfigError::ByteLimit {
            field: "query.max_batch_bytes",
            ..
        })
    ));
}

/// Scenario: watermark mode is configured as scalar or snapshot.
/// Guarantees: unimplemented modes are rejected by the schema instead of silently inheriting
/// composite behavior that they do not actually describe.
#[test]
fn rejects_unsupported_watermark_modes() {
    for mode in ["scalar", "snapshot"] {
        let value = serde_json::json!({
            "mode": mode,
            "timestamp": {
                "column": "EVENT_TS",
                "bind": "last_timestamp",
                "initial": "1970-01-01 00:00:00",
                "timezone": "UTC"
            },
            "tie_breaker": {
                "column": "EVENT_ID",
                "bind": "last_tie_breaker",
                "initial": 0
            }
        });

        assert!(
            serde_json::from_value::<WatermarkConfig>(value).is_err(),
            "watermark mode '{mode}' must be rejected"
        );
    }
}

/// Scenario: composite cursor fields are inconsistent or use unsupported semantics.
/// Guarantees: shared cursor bounds, distinct binds and columns, and UTC-only semantics are all
/// enforced before a database connection is opened.
#[test]
fn rejects_invalid_composite_watermarks() {
    let non_utc = WatermarkConfig::Composite {
        timestamp: TimestampCursorConfig {
            timezone: "America/New_York".to_owned(),
            ..watermark().timestamp().clone()
        },
        tie_breaker: watermark().tie_breaker().clone(),
    };
    assert_eq!(non_utc.validate(), Err(ConfigError::UnsupportedTimezone));

    let duplicate_bind = WatermarkConfig::Composite {
        timestamp: watermark().timestamp().clone(),
        tie_breaker: TieBreakerCursorConfig {
            bind: "last_timestamp".to_owned(),
            ..watermark().tie_breaker().clone()
        },
    };
    assert_eq!(duplicate_bind.validate(), Err(ConfigError::DuplicateBind));

    let colon_bind = WatermarkConfig::Composite {
        timestamp: TimestampCursorConfig {
            bind: ":last_timestamp".to_owned(),
            ..watermark().timestamp().clone()
        },
        tie_breaker: watermark().tie_breaker().clone(),
    };
    assert_eq!(
        colon_bind.validate(),
        Err(ConfigError::InvalidBind {
            field: "watermark.timestamp.bind"
        })
    );
}

/// Scenario: a NACK policy or checkpoint bound outside the supported contract is configured.
/// Guarantees: only the implemented rewind policy is accepted, and backoff and failure budgets
/// remain explicit and finite so a receiver cannot retry forever.
#[test]
fn rejects_unsupported_checkpoint_policy_and_bounds() {
    assert!(serde_json::from_value::<OnNack>(serde_json::json!("fail")).is_err());
    assert!(serde_json::from_value::<OnNack>(serde_json::json!("rewind")).is_ok());

    assert!(matches!(
        CheckpointConfig {
            nack_backoff: Duration::ZERO,
            ..checkpoint_config()
        }
        .validate(),
        Err(ConfigError::NackBackoffRange { .. })
    ));
    assert!(matches!(
        CheckpointConfig {
            max_consecutive_failures: 0,
            ..checkpoint_config()
        }
        .validate(),
        Err(ConfigError::CheckpointFailureRange { .. })
    ));
    assert!(matches!(
        CheckpointConfig {
            directory: "state/../../escape".to_owned(),
            ..checkpoint_config()
        }
        .validate(),
        Err(ConfigError::CheckpointTraversal)
    ));
}

/// Scenario: a valid composite configuration is compiled into a query plan.
/// Guarantees: cursor binds, columns, the initial cursor, and both byte ceilings are carried into
/// the plan the adapter executes, and SQL text is redacted from diagnostics.
#[test]
fn compiles_a_composite_query_plan() {
    let query = CompiledQuery::compile(
        "SELECT EVENT_TS, EVENT_ID FROM EVENTS ORDER BY EVENT_TS ASC, EVENT_ID ASC".to_owned(),
        polling(),
        &watermark(),
        &checkpoint_config(),
        OutputConfig::default(),
    )
    .expect("composite query should compile");

    assert_eq!(query.watermark().timestamp_bind, "last_timestamp");
    assert_eq!(query.watermark().tie_breaker_bind, "last_tie_breaker");
    assert_eq!(query.watermark().initial.tie_breaker, 0);
    assert_eq!(query.max_batch_bytes(), 10 * 1024 * 1024);
    assert_eq!(query.max_normalized_bytes(), 10 * 1024 * 1024);
    assert!(format!("{query:?}").contains("<redacted>"));
    assert!(!format!("{query:?}").contains("EVENT_TS ASC"));
}

/// Scenario: A result contains many NULL values with no dynamic payload bytes.
/// Guarantees: Memory accounting includes row and CellValue allocations, not only scalar payloads.
#[test]
fn normalized_size_includes_structural_allocations() {
    let row = Row {
        values: vec![CellValue::Null; 100],
    };

    assert!(row.normalized_size() >= (size_of::<Row>() + 100 * size_of::<CellValue>()) as u64);
}
