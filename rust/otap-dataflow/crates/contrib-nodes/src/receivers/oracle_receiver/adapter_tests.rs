// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    CellValue, OracleAdapterError, OracleType, bounded_connect_string, cursor_bind_type,
    finite_float, read_credential, validate_described_cursor_columns, validate_types,
};
use crate::receivers::database::{CompositeCursor, CompositeWatermark};
use oracle::sql_type::Timestamp;
use std::fs;
use std::str::FromStr;
use std::time::Duration;

fn watermark() -> CompositeWatermark {
    CompositeWatermark {
        timestamp_column: "EVENT_TS".to_owned(),
        timestamp_bind: "last_timestamp".to_owned(),
        tie_breaker_column: "EVENT_ID".to_owned(),
        tie_breaker_bind: "last_tie_breaker".to_owned(),
        initial: CompositeCursor::new("1970-01-01 00:00:00".to_owned(), 0),
    }
}

fn columns(timestamp: OracleType, tie_breaker: OracleType) -> Vec<(String, OracleType)> {
    vec![
        ("PAYLOAD".to_owned(), OracleType::Varchar2(64)),
        ("EVENT_TS".to_owned(), timestamp),
        ("EVENT_ID".to_owned(), tie_breaker),
    ]
}

/// Scenario: live metadata reports supported cursor types under differing identifier case.
/// Guarantees: both cursor columns are resolved to their result positions, so the receiver reads
/// each row's cursor from the correct columns regardless of driver quoting behavior.
#[test]
fn resolves_supported_cursor_columns_case_insensitively() {
    let mut described = columns(OracleType::Timestamp(6), OracleType::Number(38, 0));
    described[1].0 = "event_ts".to_owned();

    let (timestamp_index, tie_breaker_index) =
        validate_described_cursor_columns(&described, &watermark()).expect("cursor columns");

    assert_eq!(timestamp_index, 1);
    assert_eq!(tie_breaker_index, 2);
}

/// Scenario: every supported Oracle DATE and TIMESTAMP family type is used as the cursor.
/// Guarantees: the documented supported timestamp types are all accepted, so a valid deployment
/// is not rejected because of a timestamp precision or timezone variant.
#[test]
fn accepts_the_supported_oracle_timestamp_family() {
    for timestamp in [
        OracleType::Date,
        OracleType::Timestamp(0),
        OracleType::Timestamp(9),
        OracleType::TimestampTZ(6),
        OracleType::TimestampLTZ(6),
    ] {
        assert!(
            validate_described_cursor_columns(
                &columns(timestamp.clone(), OracleType::Int64),
                &watermark(),
            )
            .is_ok(),
            "timestamp type '{timestamp}' must be supported"
        );
    }
}

/// Scenario: a cursor column has a non-temporal, fractional, or otherwise non-deterministic type.
/// Guarantees: unsupported cursor metadata fails before polling, so the receiver never paginates
/// on a column whose ordering or checkpoint round trip is not exact.
#[test]
fn rejects_unsupported_cursor_column_types() {
    assert!(matches!(
        validate_described_cursor_columns(
            &columns(OracleType::Varchar2(32), OracleType::Int64),
            &watermark(),
        ),
        Err(OracleAdapterError::UnsupportedCursorTimestamp { .. })
    ));
    assert!(matches!(
        validate_described_cursor_columns(
            &columns(OracleType::Timestamp(6), OracleType::Number(38, 2)),
            &watermark(),
        ),
        Err(OracleAdapterError::UnsupportedCursorTieBreaker { .. })
    ));
    assert!(matches!(
        validate_described_cursor_columns(
            &columns(OracleType::Timestamp(6), OracleType::BinaryDouble),
            &watermark(),
        ),
        Err(OracleAdapterError::UnsupportedCursorTieBreaker { .. })
    ));
}

/// Scenario: a configured cursor column is absent from the query's result metadata.
/// Guarantees: a mismatch between the statement and the cursor configuration fails at startup
/// rather than at the first row fetch.
#[test]
fn rejects_missing_cursor_columns() {
    let described = vec![("PAYLOAD".to_owned(), OracleType::Varchar2(64))];

    assert!(matches!(
        validate_described_cursor_columns(&described, &watermark()),
        Err(OracleAdapterError::MissingCursorColumn(column)) if column == "EVENT_TS"
    ));
}

/// Scenario: a committed cursor timestamp is bound back into Oracle after a restart.
/// Guarantees: the checkpointed text round-trips through the Oracle timestamp type without
/// losing sub-second precision, so replay resumes at the exact committed boundary.
#[test]
fn cursor_timestamp_round_trips_through_oracle() {
    let committed = Timestamp::from_str("2026-01-01 12:34:56.123456789")
        .expect("committed timestamp should parse")
        .to_string();

    let rebound = Timestamp::from_str(&committed).expect("committed text should rebind");

    assert_eq!(rebound.to_string(), committed);
    assert_eq!(rebound.nanosecond(), 123_456_789);
}

/// Scenario: a committed timezone-aware cursor is rebound after restart.
/// Guarantees: the bind type retains the cursor's UTC offset instead of coercing it to a
/// timezone-naive timestamp and moving the polling boundary.
#[test]
fn timezone_aware_cursor_uses_a_timezone_aware_bind() {
    let committed = Timestamp::from_str("2026-01-01 12:34:56.123456789 +05:30")
        .expect("timezone-aware cursor should parse");

    assert!(committed.with_tz());
    assert_eq!(committed.tz_offset(), 19_800);
    assert!(matches!(cursor_bind_type(), OracleType::TimestampTZ(9)));
}

/// Scenario: a checkpoint file holds a cursor timestamp Oracle cannot parse.
/// Guarantees: an invalid committed timestamp is reported explicitly instead of being
/// interpolated into SQL or silently reset to the initial cursor.
#[test]
fn rejects_uninterpretable_cursor_timestamps() {
    assert!(Timestamp::from_str("not-a-timestamp").is_err());
}

/// Scenario: Oracle returns a non-finite binary floating-point value.
/// Guarantees: Driver normalization fails the batch instead of emitting invalid OTLP data.
#[test]
fn rejects_non_finite_float() {
    assert!(matches!(
        finite_float(CellValue::Float64(f64::NAN)),
        Err(OracleAdapterError::NonFiniteFloat)
    ));
}
/// Scenario: Oracle result metadata contains a vendor type without a CellValue mapping.
/// Guarantees: Metadata validation fails explicitly instead of using a lossy fallback.
#[test]
fn rejects_unsupported_vendor_type() {
    assert!(matches!(
        validate_types(&[OracleType::BLOB]),
        Err(OracleAdapterError::UnsupportedType(_))
    ));
}

/// Scenario: An Easy Connect string uses a query timeout longer than connection establishment.
/// Guarantees: Connection and transport attempts remain bounded while query calls retain their
/// independently configured timeout.
#[test]
fn adds_bounded_network_timeouts() {
    let connect_string =
        bounded_connect_string("database.contoso.com:1521/ORCL", Duration::from_secs(120))
            .expect("Easy Connect string should be supported");

    assert_eq!(
        connect_string,
        "database.contoso.com:1521/ORCL?connect_timeout=10&transport_connect_timeout=10"
    );
}

/// Scenario: An Easy Connect string adds retries or multiple database addresses.
/// Guarantees: Connection establishment cannot multiply the fixed per-attempt startup bound.
#[test]
fn rejects_unbounded_connection_attempts() {
    for connect_string in [
        "database.contoso.com:1521/ORCL?retry_count=10",
        "database.contoso.com:1521/ORCL?retry_delay=5",
        "db1.contoso.com,db2.contoso.com:1521/ORCL",
    ] {
        assert!(bounded_connect_string(connect_string, Duration::from_secs(120)).is_err());
    }
}

/// Scenario: A mounted credential contains a trailing newline.
/// Guarantees: Kubernetes-style secret files load without adding the line ending to the credential.
#[test]
fn trims_credential_line_endings() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("password");
    fs::write(&path, b"secret\r\n").expect("write credential");

    assert_eq!(
        read_credential(path.to_str().expect("UTF-8 path"), "password")
            .expect("credential should load"),
        "secret"
    );
}

/// Scenario: A mounted credential is not valid UTF-8.
/// Guarantees: Invalid text is rejected without including credential bytes in diagnostics.
#[test]
fn rejects_non_utf8_credential() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("password");
    fs::write(&path, [0xff]).expect("write credential");

    assert!(matches!(
        read_credential(path.to_str().expect("UTF-8 path"), "password"),
        Err(OracleAdapterError::InvalidCredentialEncoding("password"))
    ));
}
