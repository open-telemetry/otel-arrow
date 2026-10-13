// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_scraper::database::ScalarWatermark;

fn spec(initial: ScalarValue) -> CompiledWatermark {
    CompiledWatermark::Scalar(ScalarWatermark {
        column: "KEY".into(),
        bind: "last_key".into(),
        initial,
    })
}

/// Scenario: Described Oracle columns use each supported scalar native type.
/// Guarantees: The cached decode plan selects exactly the configured scalar kind without timestamp coercion.
#[test]
fn scalar_metadata_selects_exact_decoder() {
    for (value, data_type, kind) in [
        (
            ScalarValue::Int64(i64::MIN),
            OracleType::Number(19, 0),
            ScalarKind::Int64,
        ),
        (
            ScalarValue::UInt64(u64::MAX),
            OracleType::Number(20, 0),
            ScalarKind::UInt64,
        ),
        (ScalarValue::Int64(0), OracleType::Int64, ScalarKind::Int64),
        (
            ScalarValue::UInt64(0),
            OracleType::UInt64,
            ScalarKind::UInt64,
        ),
        (
            ScalarValue::String("!".into()),
            OracleType::Varchar2(1024),
            ScalarKind::String,
        ),
        (
            ScalarValue::Timestamp("2026-01-01".into()),
            OracleType::TimestampTZ(9),
            ScalarKind::Timestamp,
        ),
        (
            ScalarValue::Timestamp("2026-01-01".into()),
            OracleType::Date,
            ScalarKind::Timestamp,
        ),
    ] {
        assert_eq!(
            validate_described_watermark(&[("key".into(), data_type)], &spec(value))
                .expect("metadata"),
            CursorColumns::Scalar { index: 0, kind }
        );
    }
}

/// Scenario: Scalar metadata is missing, fractional, padded, national-character, or the wrong type.
/// Guarantees: The adapter rejects columns requiring coercion or incompatible string ordering.
#[test]
fn scalar_metadata_rejects_lossy_or_ambiguous_types() {
    for (value, data_type) in [
        (ScalarValue::Int64(0), OracleType::Number(19, 1)),
        (ScalarValue::Int64(0), OracleType::Number(20, 0)),
        (ScalarValue::UInt64(0), OracleType::Number(21, 0)),
        (ScalarValue::Int64(0), OracleType::BinaryDouble),
        (ScalarValue::UInt64(0), OracleType::Varchar2(20)),
        (ScalarValue::String("!".into()), OracleType::Char(20)),
        (ScalarValue::String("!".into()), OracleType::NVarchar2(20)),
        (ScalarValue::String("!".into()), OracleType::Number(10, 0)),
        (
            ScalarValue::Timestamp("2026-01-01".into()),
            OracleType::Varchar2(30),
        ),
    ] {
        assert!(matches!(
            validate_described_watermark(&[("KEY".into(), data_type)], &spec(value)),
            Err(OracleAdapterError::UnsupportedScalarType)
        ));
    }
    assert!(matches!(
        validate_described_watermark(&[], &spec(ScalarValue::Int64(0))),
        Err(OracleAdapterError::MissingCursorColumn)
    ));
}

/// Scenario: Native scalar parameters include integer extremes and nanosecond UTC timestamps.
/// Guarantees: Preflight retains numeric values and rejects Oracle NULL-equivalent or truncated text.
#[test]
fn scalar_native_preflight_is_lossless_and_bounded() {
    for value in [
        ScalarValue::Int64(i64::MIN),
        ScalarValue::Int64(i64::MAX),
        ScalarValue::UInt64(u64::MAX),
        ScalarValue::String("key-\u{1f680}".into()),
        ScalarValue::Timestamp("2026-01-01T00:00:00.123456789Z".into()),
    ] {
        validate_scalar_value(&value).expect("valid native scalar");
    }
    for value in [
        ScalarValue::String(String::new()),
        ScalarValue::String("bad\0key".into()),
        ScalarValue::String("x".repeat(1025)),
        ScalarValue::Timestamp("2026-01-01T00:00:00.1234567890Z".into()),
    ] {
        assert!(validate_scalar_value(&value).is_err());
    }
}

/// Scenario: Oracle exposes a database character set different from shared UTF-8 ordering.
/// Guarantees: Only AL32UTF8 is admitted for explicit-BINARY VARCHAR2 scalar queries.
#[test]
fn scalar_string_charset_fails_closed() {
    assert!(validate_scalar_charset("AL32UTF8").is_ok());
    for name in ["UTF8", "WE8MSWIN1252", "AL16UTF16", "", "private-charset"] {
        let error = validate_scalar_charset(name).expect_err("incompatible charset");
        assert!(!error.to_string().contains("private"));
    }
}

/// Scenario: Scalar cursor text reserves capacity greater than its length.
/// Guarantees: Page admission accounts for allocated key storage, while integers require no heap bytes.
#[test]
fn scalar_cursor_heap_is_charged_to_page_budget() {
    let mut key = String::with_capacity(1024);
    key.push('k');
    let cursor = Cursor::Scalar(ScalarValue::String(key));
    assert_eq!(cursor_heap_bytes(&cursor), 1024);
    assert_eq!(
        cursor_heap_bytes(&Cursor::Scalar(ScalarValue::UInt64(u64::MAX))),
        0
    );
    let mut rows = Vec::new();
    let mut bytes = 0;
    let row = CursorRow {
        row: Row { values: vec![] },
        cursor,
    };
    assert!(matches!(
        push_bounded_row(&mut rows, &mut bytes, row, 10, 512),
        Err(OracleAdapterError::NormalizedByteLimit { .. })
    ));
    assert!(rows.is_empty());
}

/// Scenario: Scalar cursor values are extracted from the row already normalized for output.
/// Guarantees: Every scalar kind preserves exact values without native re-decoding or changing the output row.
#[test]
fn scalar_cursor_reuses_normalized_cells() {
    for (kind, cell, expected) in [
        (
            ScalarKind::Int64,
            CellValue::Decimal(i64::MIN.to_string()),
            ScalarValue::Int64(i64::MIN),
        ),
        (
            ScalarKind::Int64,
            CellValue::Int64(i64::MAX),
            ScalarValue::Int64(i64::MAX),
        ),
        (
            ScalarKind::UInt64,
            CellValue::Decimal(u64::MAX.to_string()),
            ScalarValue::UInt64(u64::MAX),
        ),
        (
            ScalarKind::UInt64,
            CellValue::UInt64(u64::MAX),
            ScalarValue::UInt64(u64::MAX),
        ),
        (
            ScalarKind::String,
            CellValue::String("a ".into()),
            ScalarValue::String("a ".into()),
        ),
        (
            ScalarKind::Timestamp,
            CellValue::Timestamp("2026-01-01T00:00:00.123456000".into()),
            ScalarValue::Timestamp("2026-01-01T00:00:00.123456000".into()),
        ),
        (
            ScalarKind::Timestamp,
            CellValue::TimestampTz("2026-01-01T00:00:00.123456789+00:00".into()),
            ScalarValue::Timestamp("2026-01-01T00:00:00.123456789+00:00".into()),
        ),
    ] {
        let row = Row {
            values: vec![CellValue::String("payload".into()), cell],
        };
        let before = row.clone();
        assert_eq!(
            extract_normalized_watermark(&row, CursorColumns::Scalar { index: 1, kind })
                .expect("scalar cursor"),
            Cursor::Scalar(expected)
        );
        assert_eq!(row, before);
    }
}

/// Scenario: Normalized scalar cells are missing, null, malformed, fractional, overflowing, or incorrectly typed.
/// Guarantees: No lossy coercion or fabricated progress replaces a failed cursor extraction.
#[test]
fn invalid_normalized_scalar_cells_fail_closed() {
    for (kind, cell) in [
        (
            ScalarKind::Int64,
            CellValue::Decimal("9223372036854775808".into()),
        ),
        (
            ScalarKind::UInt64,
            CellValue::Decimal("18446744073709551616".into()),
        ),
        (ScalarKind::UInt64, CellValue::Decimal("-1".into())),
        (ScalarKind::Int64, CellValue::Decimal("1.5".into())),
        (ScalarKind::Int64, CellValue::Float64(1.0)),
        (ScalarKind::UInt64, CellValue::Int64(1)),
        (ScalarKind::String, CellValue::String(String::new())),
        (ScalarKind::String, CellValue::String("x".repeat(1025))),
        (
            ScalarKind::Timestamp,
            CellValue::String("2026-01-01".into()),
        ),
        (
            ScalarKind::Timestamp,
            CellValue::Timestamp("invalid".into()),
        ),
        (ScalarKind::Timestamp, CellValue::Null),
    ] {
        assert!(
            extract_normalized_watermark(
                &Row { values: vec![cell] },
                CursorColumns::Scalar { index: 0, kind }
            )
            .is_err()
        );
    }
    assert!(
        extract_normalized_watermark(
            &Row { values: vec![] },
            CursorColumns::Scalar {
                index: 0,
                kind: ScalarKind::Int64
            }
        )
        .is_err()
    );
}

/// Scenario: Metadata selects DATE or a specific timestamp family/precision for a scalar temporal key.
/// Guarantees: Temporal scalar binding uses the exact upstream type; non-temporal keys need no timestamp bind.
#[test]
fn scalar_timestamp_binding_uses_discovered_type() {
    for data_type in [
        OracleType::Date,
        OracleType::Timestamp(6),
        OracleType::TimestampTZ(9),
        OracleType::TimestampLTZ(3),
    ] {
        let plan = CursorColumns::Scalar {
            index: 0,
            kind: ScalarKind::Timestamp,
        };
        assert_eq!(
            timestamp_type_for_cursor(plan, std::slice::from_ref(&data_type)).expect("bind type"),
            Some(data_type)
        );
    }
    for kind in [ScalarKind::Int64, ScalarKind::UInt64, ScalarKind::String] {
        assert_eq!(
            timestamp_type_for_cursor(CursorColumns::Scalar { index: 0, kind }, &[])
                .expect("no timestamp"),
            None
        );
    }
    assert!(
        timestamp_type_for_cursor(
            CursorColumns::Scalar {
                index: 0,
                kind: ScalarKind::Timestamp
            },
            &[OracleType::Timestamp(10)]
        )
        .is_err()
    );
}

/// Scenario: A scalar timestamp checkpoint crosses a UTC date boundary and targets TIMESTAMP(6).
/// Guarantees: The scalar path retains upstream UTC and precision normalization instead of using a universal zoned bind.
#[test]
fn scalar_timestamp_binding_preserves_upstream_normalization() {
    let target = timestamp_type_for_cursor(
        CursorColumns::Scalar {
            index: 0,
            kind: ScalarKind::Timestamp,
        },
        &[OracleType::Timestamp(6)],
    )
    .expect("type")
    .expect("temporal");
    let source = parse_cursor_timestamp("2026-01-01 00:15:30.123456 +05:30").expect("timestamp");
    let bound = cursor_bind_timestamp(source, &target).expect("bind");
    assert_eq!(bound.to_string(), "2025-12-31 18:45:30.123456");
    assert!(!bound.with_tz());
}
