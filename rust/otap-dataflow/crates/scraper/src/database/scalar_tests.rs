// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::cmp::Ordering;
use std::time::Duration;

fn values() -> Vec<ScalarValue> {
    vec![
        ScalarValue::Int64(i64::MIN),
        ScalarValue::UInt64(u64::MAX),
        ScalarValue::String("private-key-713".into()),
        ScalarValue::Timestamp("2026-09-29T01:02:03.123456789Z".into()),
    ]
}

fn compile(initial: ScalarValue) -> CompiledQuery {
    CompiledQuery::compile(
        "SELECT KEY FROM EVENTS WHERE KEY > :last_key ORDER BY KEY ASC".into(),
        PollingConfig {
            interval: Duration::from_secs(1),
            timeout: Duration::from_secs(1),
            max_rows_per_poll: 10,
            fetch_size_rows: 10,
            max_batch_bytes: 4096,
            catch_up: CatchUpConfig::default(),
        },
        &WatermarkConfig::Scalar {
            column: "KEY".into(),
            bind: "last_key".into(),
            initial,
        },
        &CheckpointConfig {
            directory: "./state".into(),
            on_nack: OnNack::Rewind,
            on_permanent_nack: OnPermanentNack::Pause,
            nack_backoff: Duration::from_secs(1),
            max_consecutive_failures: 3,
        },
        OutputConfig::default(),
    )
    .expect("scalar query compiles")
}

/// Scenario: Every scalar type is configured and compiled into an adapter query.
/// Guarantees: One bind and column retain the exact declared type and initial value.
#[test]
fn scalar_configuration_preserves_types() {
    for value in values() {
        let json = serde_json::json!({
            "mode": "scalar", "column": "KEY", "bind": "last_key", "initial": value,
        });
        let config: WatermarkConfig = serde_json::from_value(json).expect("scalar schema");
        config.validate().expect("valid scalar configuration");
        assert!(config.timestamp().is_none());
        assert!(config.tie_breaker().is_none());
        let query = compile(value.clone());
        let CompiledWatermark::Scalar(spec) = query.watermark() else {
            panic!("scalar mode")
        };
        assert_eq!(spec.column, "KEY");
        assert_eq!(spec.bind, "last_key");
        assert_eq!(spec.initial, value);
        assert_eq!(query.watermark().initial(), Cursor::Scalar(value));
    }
}

/// Scenario: Scalar values cross signed and unsigned numeric boundaries or use string ordering.
/// Guarantees: Integer comparisons are exact and strings use explicit binary, not numeric, ordering.
#[test]
fn scalar_ordering_is_type_specific() {
    for (left, right) in [
        (ScalarValue::Int64(i64::MIN), ScalarValue::Int64(i64::MAX)),
        (
            ScalarValue::UInt64(i64::MAX as u64 + 1),
            ScalarValue::UInt64(u64::MAX),
        ),
        (
            ScalarValue::String("10".into()),
            ScalarValue::String("2".into()),
        ),
        (
            ScalarValue::String("A".into()),
            ScalarValue::String("a".into()),
        ),
        (
            ScalarValue::Timestamp("2026-01-01T00:00:00.000000001Z".into()),
            ScalarValue::Timestamp("2026-01-01T00:00:00.000000002Z".into()),
        ),
    ] {
        assert_eq!(left.compare(&right), Ok(Ordering::Less));
        assert_eq!(right.compare(&left), Ok(Ordering::Greater));
        assert_eq!(left.compare(&left), Ok(Ordering::Equal));
    }
}

/// Scenario: Timestamp spellings differ in precision and timezone offset.
/// Guarantees: Equal instants cannot advance a scalar watermark through lexical differences.
#[test]
fn scalar_timestamps_compare_instants() {
    let left = ScalarValue::Timestamp("2026-01-01T01:00:00.000+01:00".into());
    let right = ScalarValue::Timestamp("2026-01-01 00:00:00".into());
    assert_eq!(left.compare(&right), Ok(Ordering::Equal));
    assert_eq!(
        right.compare(&ScalarValue::Timestamp("invalid".into())),
        Err(CursorError::InvalidTimestamp)
    );
}

/// Scenario: A restart or adapter changes a scalar type or watermark mode.
/// Guarantees: Different kinds are rejected, never coerced or ordered by enum discriminant.
#[test]
fn incompatible_positions_fail_closed() {
    let composite = Cursor::composite("2026-01-01".into(), 0);
    for left in values() {
        let query = compile(left.clone());
        assert_eq!(
            query.watermark().validate_cursor(&composite),
            Err(CursorError::TypeMismatch)
        );
        for right in values() {
            if left.same_type(&right) {
                continue;
            }
            let right = Cursor::Scalar(right);
            assert_eq!(
                query.watermark().validate_cursor(&right),
                Err(CursorError::TypeMismatch)
            );
            assert_eq!(
                Cursor::Scalar(left.clone()).compare(&right),
                Err(CursorError::TypeMismatch)
            );
        }
    }
}

/// Scenario: Cursor JSON is persisted and reloaded, including full-width unsigned values.
/// Guarantees: Values, precision, and types round-trip without i64 or f64 conversion.
#[test]
fn scalar_cursor_round_trips_losslessly() {
    for value in values() {
        let cursor = Cursor::Scalar(value);
        let bytes = serde_json::to_vec(&cursor).expect("encode");
        assert_eq!(
            serde_json::from_slice::<Cursor>(&bytes).expect("decode"),
            cursor
        );
    }
}

/// Scenario: An existing version-1 checkpoint contains an untagged composite cursor.
/// Guarantees: Decoding and re-encoding leave its exact JSON representation unchanged for checksum verification.
#[test]
fn legacy_composite_cursor_bytes_remain_stable() {
    let bytes = br#"{"timestamp":"2026-01-01 00:00:00","tie_breaker":42}"#;
    let cursor: Cursor = serde_json::from_slice(bytes).expect("legacy cursor");
    assert!(matches!(cursor, Cursor::Composite(_)));
    assert_eq!(serde_json::to_vec(&cursor).expect("encode"), bytes);
}

/// Scenario: Configuration supplies absent, unknown, null, overflowing, or mismatched scalar values.
/// Guarantees: Schema validation rejects unsupported or ambiguous scalar types.
#[test]
fn scalar_schema_rejects_invalid_values() {
    for value in [
        serde_json::json!({"type":"float64", "value":1.5}),
        serde_json::json!({"type":"bool", "value":true}),
        serde_json::json!({"type":"int64", "value":null}),
        serde_json::json!({"type":"int64", "value":"42"}),
        serde_json::json!({"type":"int64", "value":u64::MAX}),
        serde_json::json!({"type":"uint64", "value":-1}),
        serde_json::json!({"type":"string", "value":42}),
        serde_json::json!({"value":42}),
        serde_json::json!({"type":"int64", "value":42,"extra":true}),
        serde_json::json!({"type":"int64", "value":42,"timestamp":"2026-01-01","tie_breaker":1}),
    ] {
        assert!(serde_json::from_value::<Cursor>(value).is_err());
    }
}

/// Scenario: Scalar names, binds, timestamp text, or text lengths violate the shared contract.
/// Guarantees: Invalid configuration fails without opening a database connection.
#[test]
fn scalar_configuration_validates_bounds() {
    for (column, bind, initial) in [
        (" ", "last_key", ScalarValue::Int64(0)),
        ("KEY", ":last_key", ScalarValue::Int64(0)),
        ("KEY", "key;select", ScalarValue::Int64(0)),
        ("KEY", "", ScalarValue::Int64(0)),
        ("KEY", "last_key", ScalarValue::Timestamp("invalid".into())),
        ("KEY", "last_key", ScalarValue::String("x".repeat(1025))),
    ] {
        assert!(
            WatermarkConfig::Scalar {
                column: column.into(),
                bind: bind.into(),
                initial
            }
            .validate()
            .is_err()
        );
    }
    ScalarValue::String(String::new())
        .validate()
        .expect("empty starting key is valid");
    ScalarValue::String("x".repeat(1024))
        .validate()
        .expect("inclusive bound");
}

/// Scenario: Customer cursor values appear in direct and nested diagnostics.
/// Guarantees: Debug shows only the scalar type and a redaction marker.
#[test]
fn scalar_debug_redacts_values() {
    for value in values() {
        let sentinel = match &value {
            ScalarValue::Int64(value) => value.to_string(),
            ScalarValue::UInt64(value) => value.to_string(),
            ScalarValue::String(value) | ScalarValue::Timestamp(value) => value.clone(),
        };
        let config = WatermarkConfig::Scalar {
            column: "KEY".into(),
            bind: "last_key".into(),
            initial: value.clone(),
        };
        for debug in [
            format!("{value:?}"),
            format!("{config:#?}"),
            format!("{:?}", compile(value.clone())),
            format!("{:?}", Cursor::Scalar(value)),
        ] {
            assert!(!debug.contains(&sentinel));
            assert!(debug.contains("<redacted>"));
        }
    }
}

/// Scenario: A scalar page exceeds the OTLP byte ceiling after its first row.
/// Guarantees: The candidate remains the last emitted scalar key, not a deferred row's key.
#[test]
fn scalar_encoding_commits_only_the_emitted_prefix() {
    for initial in values() {
        let first = Cursor::Scalar(initial);
        let second = Cursor::Scalar(ScalarValue::Int64(123));
        let row = Row {
            values: vec![CellValue::String("body".repeat(30))],
        };
        let page = QueryPage {
            columns: vec![ColumnMetadata {
                name: "BODY".into(),
                source_type: "VARCHAR".into(),
                nullable: false,
            }],
            rows: vec![CursorRow {
                row: row.clone(),
                cursor: first.clone(),
            }],
        };
        let encode = |page, limit| {
            encode_page(
                page,
                DatabaseSystem::Oracle,
                "source",
                &OutputConfig::default(),
                1,
                limit,
            )
            .expect("valid page")
            .expect("nonempty page")
        };
        let limit = encode(page.clone(), u64::MAX).encoded_bytes as u64;
        let mut full = page;
        full.rows.push(CursorRow {
            row,
            cursor: second,
        });
        let encoded = encode(full, limit);
        assert_eq!(encoded.row_count, 1);
        assert_eq!(encoded.deferred_rows, 1);
        assert_eq!(encoded.candidate, first);
    }
}
