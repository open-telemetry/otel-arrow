// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn snapshot_page(count: usize) -> QueryPage {
    QueryPage {
        completion: QueryCompletion::Complete,
        columns: vec![ColumnMetadata {
            name: "VALUE".into(),
            source_type: "VARCHAR2".into(),
            nullable: true,
        }],
        rows: (0..count)
            .map(|_| CursorRow {
                row: Row {
                    values: vec![CellValue::Null],
                },
                cursor: Cursor::Snapshot,
            })
            .collect(),
    }
}

fn encode(page: QueryPage, limit: u64) -> Result<Option<EncodedPage>, OtlpMappingError> {
    encode_page(
        page,
        DatabaseSystem::Oracle,
        "snapshot",
        &OutputConfig::default(),
        1,
        limit,
    )
}

/// Scenario: A snapshot watermark is configured without any result-column metadata.
/// Guarantees: Only the no-position cursor is accepted and keyset fields are rejected.
#[test]
fn snapshot_config_has_no_column_contract() {
    let config: WatermarkConfig = serde_json::from_str(r#"{"mode":"snapshot"}"#).expect("config");
    config.validate().expect("valid snapshot");
    assert!(config.timestamp().is_none());
    assert!(config.tie_breaker().is_none());
    for field in ["column", "bind", "initial", "timestamp", "tie_breaker"] {
        let value = serde_json::json!({"mode":"snapshot", field:"unused"});
        assert!(
            serde_json::from_value::<WatermarkConfig>(value).is_err(),
            "{field}"
        );
    }
    let watermark = CompiledWatermark::Snapshot;
    assert_eq!(watermark.initial(), Cursor::Snapshot);
    assert!(watermark.as_composite().is_none());
    watermark
        .validate_cursor(&Cursor::Snapshot)
        .expect("no position");
    assert!(
        watermark
            .validate_cursor(&Cursor::Scalar(ScalarValue::Int64(0)))
            .is_err()
    );
    assert!(
        watermark
            .validate_cursor(&Cursor::composite("2026-01-01T00:00:00Z".into(), 0))
            .is_err()
    );
}

/// Scenario: Snapshot progress is serialized alongside the existing untagged cursor formats.
/// Guarantees: JSON null means no source position and never implies an ordered row offset.
#[test]
fn snapshot_cursor_round_trips_without_a_column_value() {
    assert_eq!(
        serde_json::to_string(&Cursor::Snapshot).expect("serialize"),
        "null"
    );
    assert_eq!(
        serde_json::from_str::<Cursor>("null").expect("deserialize"),
        Cursor::Snapshot
    );
    assert!(Cursor::Snapshot.compare(&Cursor::Snapshot).is_err());
    for invalid in ["{}", r#"{"snapshot":1}"#, "1", r#""snapshot""#] {
        assert!(serde_json::from_str::<Cursor>(invalid).is_err());
    }
}

/// Scenario: A complete snapshot contains repeated rows and null values with no unique key.
/// Guarantees: All rows are encoded and the candidate carries no source-column state.
#[test]
fn snapshot_encodes_duplicates_and_nulls() {
    let encoded = encode(snapshot_page(3), 4096)
        .expect("encode")
        .expect("nonempty");
    assert_eq!(encoded.row_count, 3);
    assert_eq!(encoded.deferred_rows, 0);
    assert_eq!(encoded.candidate, Cursor::Snapshot);
    assert!(
        encode(snapshot_page(0), 4096)
            .expect("empty snapshot")
            .is_none()
    );
}

/// Scenario: The byte limit fits a row prefix, but not the entire snapshot.
/// Guarantees: No partial payload or checkpoint candidate escapes; an exact fit succeeds.
#[test]
fn snapshot_encoding_is_all_or_nothing() {
    let size = encode(snapshot_page(2), 4096)
        .expect("size")
        .expect("nonempty")
        .encoded_bytes as u64;
    assert!(encode(snapshot_page(2), size).is_ok());
    assert!(matches!(
        encode(snapshot_page(2), size - 1),
        Err(OtlpMappingError::SnapshotByteLimit { .. })
    ));
    assert!(matches!(
        encode(snapshot_page(1), 1),
        Err(OtlpMappingError::SnapshotByteLimit { .. })
    ));
}

/// Scenario: A bounded snapshot prefix is marked incomplete even though its rows fit.
/// Guarantees: Both standalone and reusable encoders reject it without returning a checkpoint candidate.
#[test]
fn incomplete_snapshot_cannot_be_encoded() {
    let mut page = snapshot_page(2);
    page.completion = QueryCompletion::Incomplete;
    assert!(matches!(
        encode(page.clone(), 4096),
        Err(OtlpMappingError::IncompleteSnapshot)
    ));
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "snapshot".into(),
        OutputConfig::default(),
        page.columns.clone(),
    )
    .expect("encoder");
    assert!(matches!(
        encoder.encode_page(page, 1, 4096),
        Err(OtlpMappingError::IncompleteSnapshot)
    ));
    assert_eq!(
        encoder
            .encode_page(snapshot_page(2), 1, 4096)
            .expect("complete result after rejection")
            .expect("batch")
            .row_count,
        2
    );
}

/// Scenario: An adapter returns an empty incomplete snapshot with no row cursor identifying its mode.
/// Guarantees: Both encoder entry points reject it rather than report successful query exhaustion.
#[test]
fn empty_incomplete_snapshot_is_rejected_by_both_encoders() {
    let mut page = snapshot_page(0);
    page.completion = QueryCompletion::Incomplete;
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "snapshot".into(),
        OutputConfig::default(),
        page.columns.clone(),
    )
    .expect("encoder");
    let standalone = encode(page.clone(), 4096);
    let reusable = encoder.encode_page(page, 1, 4096);
    assert!(matches!(
        standalone,
        Err(OtlpMappingError::EmptyIncompletePage)
    ));
    assert!(matches!(
        reusable,
        Err(OtlpMappingError::EmptyIncompletePage)
    ));
}

/// Scenario: An exhausted snapshot has no rows to encode.
/// Guarantees: Both encoder entry points accept a complete empty page without inventing a candidate.
#[test]
fn complete_empty_pages_are_accepted_by_both_encoders() {
    let page = snapshot_page(0);
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "snapshot".into(),
        OutputConfig::default(),
        page.columns.clone(),
    )
    .expect("encoder");
    assert!(
        encode(page.clone(), 4096)
            .expect("complete empty result")
            .is_none()
    );
    assert!(
        encoder
            .encode_page(page, 1, 4096)
            .expect("complete empty result")
            .is_none()
    );
}

/// Scenario: An incomplete scalar or composite page retains a resumable nonempty prefix.
/// Guarantees: Both encoders emit its rows and return the exact retained cursor as the candidate.
#[test]
fn incomplete_keyset_prefixes_remain_encodable() {
    for cursor in [
        Cursor::Scalar(ScalarValue::Int64(1)),
        Cursor::composite("2026-01-01T00:00:00Z".into(), 1),
    ] {
        let mut page = snapshot_page(1);
        page.completion = QueryCompletion::Incomplete;
        page.rows[0].cursor = cursor.clone();
        let mut encoder = OtlpPageEncoder::new(
            DatabaseSystem::Oracle,
            "keyset".into(),
            OutputConfig::default(),
            page.columns.clone(),
        )
        .expect("encoder");
        for encoded in [
            encode(page.clone(), 4096),
            encoder.encode_page(page, 1, 4096),
        ] {
            let encoded = encoded.expect("resumable prefix").expect("nonempty page");
            assert_eq!(encoded.row_count, 1);
            assert_eq!(encoded.candidate, cursor);
        }
    }
}

/// Scenario: An adapter mixes snapshot markers and scalar positions in either order.
/// Guarantees: The standalone encoder cannot accidentally apply keyset prefix semantics to a snapshot.
#[test]
fn mixed_snapshot_pages_are_rejected() {
    for index in [0, 1] {
        let mut page = snapshot_page(2);
        page.rows[index].cursor = Cursor::Scalar(ScalarValue::Int64(1));
        assert!(matches!(
            encode(page, 4096),
            Err(OtlpMappingError::MixedSnapshotCursors)
        ));
    }
}
