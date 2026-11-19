// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    CellValue, Cursor, CursorRow, Row, RowAdmissionError, ScalarValue, push_bounded_row,
    row_heap_bytes,
};
use std::mem::size_of;

fn row(id: i64) -> CursorRow {
    CursorRow {
        row: Row {
            values: vec![CellValue::Int64(id)],
        },
        cursor: ScalarValue::Int64(id).into(),
    }
}

fn retained_bytes(rows: &Vec<CursorRow>, payload: u64) -> u64 {
    payload + (rows.capacity() * size_of::<CursorRow>()) as u64
}

/// Scenario: The first row exceeds the byte ceiling by one byte, including fixed overhead.
/// Guarantees: Admission fails without allocating a row vector or changing its payload counter.
#[test]
fn first_row_overflow_is_not_an_empty_page() {
    let overhead = 37;
    let minimum = overhead + row_heap_bytes(&row(1)) + size_of::<CursorRow>() as u64;
    let mut rows = Vec::new();
    let mut payload = overhead;
    assert_eq!(
        push_bounded_row(&mut rows, &mut payload, row(1), 2, minimum - 1),
        Err(RowAdmissionError::Limit {
            normalized_bytes: minimum,
        })
    );
    assert!(rows.is_empty());
    assert_eq!(rows.capacity(), 0);
    assert_eq!(payload, overhead);
}

/// Scenario: A successor exceeds the remaining byte ceiling after one row fits exactly.
/// Guarantees: Deferral preserves the fitting prefix, retained capacity, and payload counter.
#[test]
fn byte_ceiling_returns_only_the_fitting_prefix() {
    let limit = row_heap_bytes(&row(1)) + size_of::<CursorRow>() as u64;
    let mut rows = Vec::new();
    let mut payload = 0;
    assert!(push_bounded_row(&mut rows, &mut payload, row(1), 2, limit).expect("first row fits"));
    let before = retained_bytes(&rows, payload);
    assert_eq!(before, limit);
    assert!(!push_bounded_row(&mut rows, &mut payload, row(2), 2, limit).expect("valid successor"));
    assert_eq!(rows, vec![row(1)]);
    assert_eq!(retained_bytes(&rows, payload), before);
}

/// Scenario: Successive rows fill exact byte budgets on both sides of vector growth boundaries.
/// Guarantees: Every fitting row is retained, actual spare capacity is charged, and row ceilings win.
#[test]
fn exact_row_and_byte_ceilings_include_allocated_slots() {
    let overhead = 19;
    let per_row = row_heap_bytes(&row(1)) + size_of::<CursorRow>() as u64;
    for max_rows in 1..=9 {
        let limit = overhead + max_rows as u64 * per_row;
        let mut rows = Vec::new();
        let mut payload = overhead;
        for id in 1..=max_rows {
            assert!(
                push_bounded_row(&mut rows, &mut payload, row(id as i64), max_rows, limit)
                    .expect("exact row budget")
            );
            assert!(retained_bytes(&rows, payload) <= limit);
            assert!(rows.capacity() <= max_rows);
        }
        assert_eq!(retained_bytes(&rows, payload), limit);
        assert!(
            !push_bounded_row(
                &mut rows,
                &mut payload,
                row(max_rows as i64 + 1),
                max_rows,
                u64::MAX,
            )
            .expect("valid row-limit successor")
        );
        assert_eq!(rows.len(), max_rows);
    }
}

/// Scenario: A wide second row fits only after speculative spare slots are released.
/// Guarantees: Retained capacity cannot prematurely cut a fitting prefix or exceed the byte ceiling.
#[test]
fn spare_capacity_is_trimmed_for_a_fitting_row() {
    let mut rows = Vec::with_capacity(8);
    let mut payload = 0;
    let generous = 16_384;
    assert!(
        push_bounded_row(&mut rows, &mut payload, row(1), 8, generous).expect("first row fits")
    );
    let original_capacity = rows.capacity();
    let mut wide = row(2);
    wide.row.values.push(CellValue::Bytes(vec![0; 1024]));
    let limit = payload + row_heap_bytes(&wide) + 2 * size_of::<CursorRow>() as u64;
    assert!(
        push_bounded_row(&mut rows, &mut payload, wide, 8, limit).expect("wide successor fits")
    );
    assert_eq!(rows.len(), 2);
    assert!(rows.capacity() < original_capacity);
    assert_eq!(retained_bytes(&rows, payload), limit);
}

/// Scenario: Value vectors, text, bytes, and textual cursors retain much more capacity than length.
/// Guarantees: Admission charges every retained heap allocation, not just visible data lengths.
#[test]
fn nested_value_and_cursor_capacity_cannot_bypass_the_budget() {
    let mut text = String::with_capacity(128);
    text.push('x');
    let mut binary = Vec::with_capacity(256);
    binary.push(7);
    let mut values = Vec::with_capacity(8);
    let heap = values.capacity() * size_of::<CellValue>() + text.capacity() + binary.capacity();
    values.push(CellValue::String(text));
    values.push(CellValue::Bytes(binary));

    let mut cursor_text = String::with_capacity(64);
    cursor_text.push('z');
    let cursor_capacity = cursor_text.capacity();
    let candidate = CursorRow {
        row: Row { values },
        cursor: ScalarValue::String(cursor_text).into(),
    };
    let expected_heap = (heap + cursor_capacity) as u64;
    assert_eq!(row_heap_bytes(&candidate), expected_heap);
    let minimum = expected_heap + size_of::<CursorRow>() as u64;
    let mut rows = Vec::new();
    let mut payload = 0;
    assert_eq!(
        push_bounded_row(&mut rows, &mut payload, candidate, 1, minimum - 1),
        Err(RowAdmissionError::Limit {
            normalized_bytes: minimum,
        })
    );
    assert!(rows.is_empty());
}

/// Scenario: Composite and scalar timestamp cursors retain spare text capacity.
/// Guarantees: Cursor heap storage is counted once in addition to the row and outer slots.
#[test]
fn timestamp_cursor_capacity_is_charged() {
    for composite in [false, true] {
        let mut timestamp = String::with_capacity(128);
        timestamp.push_str("2026-01-01T00:00:00Z");
        let capacity = timestamp.capacity() as u64;
        let mut candidate = row(1);
        let row_heap = row_heap_bytes(&candidate);
        candidate.cursor = if composite {
            Cursor::composite(timestamp, 1)
        } else {
            ScalarValue::Timestamp(timestamp).into()
        };
        assert_eq!(row_heap_bytes(&candidate), row_heap + capacity);
        let limit = row_heap + capacity + size_of::<CursorRow>() as u64;
        let mut rows = Vec::new();
        let mut payload = 0;
        assert!(
            push_bounded_row(&mut rows, &mut payload, candidate, 1, limit)
                .expect("timestamp cursor fits")
        );
        assert_eq!(retained_bytes(&rows, payload), limit);
    }
}

/// Scenario: A duplicate, backwards, or incompatible cursor appears before or at either ceiling.
/// Guarantees: Invalid successors fail before deferral and never change the accepted prefix.
#[test]
fn invalid_successors_are_rejected_even_when_deferred() {
    let one_row = row_heap_bytes(&row(1)) + size_of::<CursorRow>() as u64;
    for (max_rows, limit) in [(1, 4096), (2, one_row), (2, 4096)] {
        for cursor in [
            ScalarValue::Int64(1).into(),
            ScalarValue::Int64(0).into(),
            ScalarValue::UInt64(2).into(),
            Cursor::Snapshot,
        ] {
            let mut rows = Vec::new();
            let mut payload = 0;
            assert!(
                push_bounded_row(&mut rows, &mut payload, row(1), max_rows, limit)
                    .expect("first row fits")
            );
            let before = retained_bytes(&rows, payload);
            let mut next = row(2);
            next.cursor = cursor;
            assert_eq!(
                push_bounded_row(&mut rows, &mut payload, next, max_rows, limit),
                Err(RowAdmissionError::InvalidCursor)
            );
            assert_eq!(rows, vec![row(1)]);
            assert_eq!(retained_bytes(&rows, payload), before);
        }
    }
}

/// Scenario: Multiple snapshot rows use the same no-position marker and reach a row ceiling.
/// Guarantees: Snapshots are unordered, but overflow still signals deferral for adapter rejection.
#[test]
fn snapshot_markers_do_not_require_ordering() {
    let mut rows = Vec::new();
    let mut payload = 0;
    for id in 1..=3 {
        let mut candidate = row(id);
        candidate.cursor = Cursor::Snapshot;
        assert_eq!(
            push_bounded_row(&mut rows, &mut payload, candidate, 2, 4096)
                .expect("unordered snapshot row"),
            id <= 2
        );
    }
    assert_eq!(rows.len(), 2);
}
