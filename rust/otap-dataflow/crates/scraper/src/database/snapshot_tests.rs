// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn snapshot_page(count: usize) -> QueryPage {
    QueryPage {
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
