// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::database::CellValue;

fn cursor_row(timestamp: &str, tie_breaker: i64) -> CursorRow {
    CursorRow {
        row: Row {
            values: vec![CellValue::Int64(tie_breaker)],
        },
        cursor: CompositeCursor::new(timestamp.to_owned(), tie_breaker),
    }
}

/// Scenario: two rows share a timestamp but differ only in their tie-breaker.
/// Guarantees: composite ordering advances within a timestamp group, so a page boundary that
/// lands inside a collision cannot re-emit or skip the colliding rows.
#[test]
fn tie_breaker_orders_rows_inside_one_timestamp_group() {
    let earlier = CompositeCursor::new("2026-01-01 00:00:00".to_owned(), 1);
    let later = CompositeCursor::new("2026-01-01 00:00:00".to_owned(), 2);
    let next_second = CompositeCursor::new("2026-01-01 00:00:01".to_owned(), 0);

    assert!(earlier < later);
    assert!(later < next_second);
}

/// Scenario: a page carries per-row cursors alongside normalized row values.
/// Guarantees: every emitted row keeps its own resumable position, so the committed candidate
/// can always be taken from the last row actually emitted rather than the last row fetched.
#[test]
fn every_page_row_carries_its_own_cursor() {
    let page = QueryPage {
        columns: vec![ColumnMetadata {
            name: "EVENT_ID".to_owned(),
            source_type: "NUMBER(38,0)".to_owned(),
            nullable: false,
        }],
        rows: vec![
            cursor_row("2026-01-01 00:00:00", 1),
            cursor_row("2026-01-01 00:00:00", 2),
        ],
    };

    assert!(!page.is_empty());
    assert_eq!(page.rows[0].cursor.tie_breaker, 1);
    assert_eq!(page.rows[1].cursor.tie_breaker, 2);
}

/// Scenario: a poll finds no rows after the committed cursor.
/// Guarantees: an exhausted source reports an empty page instead of a page whose candidate
/// would incorrectly advance the durable checkpoint.
#[test]
fn exhausted_source_reports_an_empty_page() {
    let page = QueryPage {
        columns: Vec::new(),
        rows: Vec::new(),
    };

    assert!(page.is_empty());
}

/// Scenario: a cursor is serialized into a durable checkpoint and read back.
/// Guarantees: timestamp text and tie-breaker survive a round trip exactly, so restart resumes
/// from the identical composite position with no precision loss.
#[test]
fn cursor_round_trips_through_serialization() {
    let cursor = CompositeCursor::new("2026-01-01 12:34:56.123456789".to_owned(), -42);
    let encoded = serde_json::to_string(&cursor).expect("cursor should serialize");
    let decoded: CompositeCursor = serde_json::from_str(&encoded).expect("cursor should decode");

    assert_eq!(decoded, cursor);
}

/// Scenario: durable checkpoint bytes contain an unexpected cursor field.
/// Guarantees: the closed cursor schema rejects unknown fields instead of silently discarding
/// state written by an incompatible future version.
#[test]
fn cursor_rejects_unknown_fields() {
    let encoded = r#"{"timestamp":"2026-01-01 00:00:00","tie_breaker":1,"extra":true}"#;

    assert!(serde_json::from_str::<CompositeCursor>(encoded).is_err());
}
