// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    CellValue, Cursor, CursorRow, Row, RowAdmissionError, ScalarValue, push_bounded_row,
    push_bounded_row_with_validation, row_heap_bytes,
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

/// Scenario: An adapter validates native positions instead of parsing normalized timestamp text.
/// Guarantees: Its validator is called exactly once for first, retained and deferred candidates,
/// and rejection precedes all budget checks without falling back to textual cursor comparison.
#[test]
fn adapter_validation_replaces_text_comparison_before_budget_checks() {
    let mut candidate = row(1);
    candidate.cursor = ScalarValue::Timestamp("native-position-only".into()).into();
    assert!(candidate.cursor.compare(&candidate.cursor).is_err());
    let one_row = row_heap_bytes(&candidate) + size_of::<CursorRow>() as u64;
    for (max_rows, limit) in [(1, 4096), (2, one_row), (2, 4096)] {
        let mut rows = Vec::new();
        let mut payload = 0;
        let mut calls = 0;
        assert!(
            push_bounded_row_with_validation(
                &mut rows,
                &mut payload,
                candidate.clone(),
                max_rows,
                limit,
                |previous, next| {
                    calls += 1;
                    assert!(previous.is_none());
                    assert_eq!(next, &candidate.cursor);
                    Ok(())
                },
            )
            .unwrap()
        );
        let retained = retained_bytes(&rows, payload);
        assert_eq!(
            push_bounded_row_with_validation(
                &mut rows,
                &mut payload,
                candidate.clone(),
                max_rows,
                limit,
                |previous, next| {
                    calls += 1;
                    assert_eq!(previous, Some(&candidate.cursor));
                    assert_eq!(next, &candidate.cursor);
                    Err(RowAdmissionError::InvalidCursor)
                },
            ),
            Err(RowAdmissionError::InvalidCursor)
        );
        assert_eq!(rows, vec![candidate.clone()]);
        assert_eq!(retained_bytes(&rows, payload), retained);
        assert_eq!(
            push_bounded_row_with_validation(
                &mut rows,
                &mut payload,
                candidate.clone(),
                max_rows,
                limit,
                |_, _| {
                    calls += 1;
                    Ok(())
                },
            )
            .unwrap(),
            max_rows == 2 && limit == 4096
        );
        assert_eq!(calls, 3);
    }
    let mut rows = Vec::new();
    let mut payload = 0;
    assert_eq!(
        push_bounded_row_with_validation(&mut rows, &mut payload, candidate, 0, 0, |_, _| {
            Err(RowAdmissionError::InvalidCursor)
        }),
        Err(RowAdmissionError::InvalidCursor)
    );
    assert_eq!(rows.capacity(), 0);
    assert_eq!(payload, 0);
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

fn payload_row(id: i64, heap: usize) -> CursorRow {
    let values = if heap == 0 {
        Vec::new()
    } else {
        vec![CellValue::Bytes(vec![0; heap - size_of::<CellValue>()])]
    };
    let candidate = CursorRow {
        row: Row { values },
        cursor: ScalarValue::Int64(id).into(),
    };
    assert_eq!(row_heap_bytes(&candidate), heap as u64);
    candidate
}

fn capacity_changes_for_page(name: &str, heap_slots: impl Fn(usize) -> usize) -> usize {
    let slot = size_of::<CursorRow>();
    let fitting_rows = 6000;
    let max_rows = 10_000;
    let limit = (fitting_rows * slot) as u64
        + (0..fitting_rows)
            .map(|index| row_heap_bytes(&payload_row(index as i64, heap_slots(index) * slot)))
            .sum::<u64>();
    let mut rows = Vec::new();
    let mut payload = 0;
    let mut growths = 0;
    let mut trims = 0;
    for index in 0..fitting_rows {
        let capacity = rows.capacity();
        assert!(
            push_bounded_row(
                &mut rows,
                &mut payload,
                payload_row(index as i64, heap_slots(index) * slot),
                max_rows,
                limit,
            )
            .expect("all rows in the fitting prefix must be admitted")
        );
        growths += usize::from(rows.capacity() > capacity);
        trims += usize::from(rows.capacity() < capacity);
        assert!(retained_bytes(&rows, payload) <= limit);
        assert!(rows.capacity() <= max_rows);
    }
    let capacity = rows.capacity();
    let retained = retained_bytes(&rows, payload);
    assert!(
        !push_bounded_row(
            &mut rows,
            &mut payload,
            payload_row(fitting_rows as i64, heap_slots(fitting_rows) * slot),
            max_rows,
            limit,
        )
        .expect("the next valid row must be deferred")
    );
    assert_eq!(rows.capacity(), capacity);
    assert_eq!(retained_bytes(&rows, payload), retained);
    assert_eq!(retained, limit);
    for (index, retained) in rows.iter().enumerate() {
        assert_eq!(
            *retained,
            payload_row(index as i64, heap_slots(index) * slot)
        );
    }
    assert_eq!(rows.len(), fitting_rows);
    println!(
        "{name}: rows={}, growths={growths}, trims={trims}",
        rows.len()
    );
    growths + trims
}

/// Scenario: A byte-limited 6000-row page has constant heap payload equal to one outer slot.
/// Guarantees: Capacity changes stay bounded rather than reallocating for each fitting successor.
#[test]
fn constant_payload_capacity_changes_are_bounded() {
    let changes = capacity_changes_for_page("constant", |_| 1);
    assert!(changes <= 96, "{changes} capacity changes for 6000 rows");
}

/// Scenario: Row heap payload changes from empty to wide and follows a deterministic sawtooth.
/// Guarantees: Growth and trimming retain the full ordered prefix without per-row capacity churn.
#[test]
fn varying_payload_capacity_changes_are_bounded() {
    let zero_then_wide =
        capacity_changes_for_page("zero-then-wide", |index| usize::from(index >= 3000) * 2);
    let sawtooth = capacity_changes_for_page("sawtooth", |index| (index / 512) % 4);
    assert!(
        zero_then_wide <= 96 && sawtooth <= 96,
        "capacity changes: zero-then-wide={zero_then_wide}, sawtooth={sawtooth}"
    );
}

/// Scenario: Fixed adapter overhead makes the first row's payload or slot accounting overflow u64.
/// Guarantees: Even an unlimited byte ceiling fails without allocating or mutating the counter.
#[test]
fn first_row_arithmetic_overflow_is_rejected() {
    for overhead in [u64::MAX, u64::MAX - row_heap_bytes(&row(1))] {
        let mut rows = Vec::new();
        let mut payload = overhead;
        assert_eq!(
            push_bounded_row(&mut rows, &mut payload, row(1), 2, u64::MAX),
            Err(RowAdmissionError::Limit {
                normalized_bytes: u64::MAX,
            })
        );
        assert_eq!(rows.capacity(), 0);
        assert!(rows.is_empty());
        assert_eq!(payload, overhead);
    }
}

/// Scenario: A valid prefix with near-u64::MAX fixed overhead cannot account for another row.
/// Guarantees: Arithmetic overflow defers the successor without panicking or changing the prefix.
#[test]
fn prefix_arithmetic_overflow_is_deferred() {
    let slot = size_of::<CursorRow>() as u64;
    let first_heap = row_heap_bytes(&row(1));
    for remaining in [0, first_heap, first_heap + slot - 1] {
        let overhead = u64::MAX - first_heap - slot - remaining;
        let mut rows = Vec::new();
        let mut payload = overhead;
        assert!(
            push_bounded_row(&mut rows, &mut payload, row(1), 1, u64::MAX)
                .expect("first row fits with fixed overhead")
        );
        let capacity = rows.capacity();
        let before = payload;
        assert_eq!(
            push_bounded_row(&mut rows, &mut payload, row(2), 2, u64::MAX),
            Ok(false)
        );
        assert_eq!(rows, vec![row(1)]);
        assert_eq!(rows.capacity(), capacity);
        assert_eq!(payload, before);
    }
}
