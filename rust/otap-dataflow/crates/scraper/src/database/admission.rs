// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Capacity-aware admission of adapter-normalized rows.

use super::{Cursor, CursorRow, Row, ScalarValue};
use std::mem::size_of;

/// A normalized row could not be safely admitted to a bounded page.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RowAdmissionError {
    /// The candidate is not a strictly ordered successor of the retained prefix.
    #[error("invalid normalized page successor")]
    InvalidCursor,
    /// The first row, actual retained capacity, or allocation exceeds available resources.
    #[error("cannot retain normalized page requiring {normalized_bytes} bytes")]
    Limit {
        /// Required accounted storage; allocation can also fail below the byte ceiling.
        normalized_bytes: u64,
    },
}

/// Returns conservative row and cursor heap storage, excluding the `CursorRow` slot.
///
/// Charges retained capacities, not lengths. Inline row and cursor headers are
/// already charged by the outer vector's slots.
#[must_use]
pub fn row_heap_bytes(row: &CursorRow) -> u64 {
    let cursor = match &row.cursor {
        Cursor::Snapshot | Cursor::Scalar(ScalarValue::Int64(_) | ScalarValue::UInt64(_)) => 0,
        Cursor::Composite(value) => value.timestamp.capacity(),
        Cursor::Scalar(ScalarValue::String(value) | ScalarValue::Timestamp(value)) => {
            value.capacity()
        }
    };
    row.row
        .normalized_size()
        .saturating_sub(size_of::<Row>() as u64)
        .saturating_add(cursor as u64)
}

/// Appends a normalized row if the row and byte ceilings permit it.
///
/// `payload_bytes` tracks retained row/cursor heap storage plus any fixed
/// adapter-accounted overhead, excluding outer vector slots. Initialize it
/// before the first call and do not otherwise mutate it or `rows` between calls.
/// Spare vector capacity is charged, trimmed when necessary, and rechecked after
/// allocation. A candidate's cursor is validated even when it will be deferred;
/// consecutive snapshot markers are the only unordered case.
///
/// Returns `Ok(false)` for a fitting prefix followed by a valid successor that
/// does not fit. The candidate is dropped, without changing the prefix or its
/// payload counter. A first-row overflow is an error, never an empty page.
/// Adapters must translate deferral into failure for all-or-nothing snapshots,
/// validate the first cursor against the committed position, and own native
/// fetching, decoding, cancellation, and cleanup.
pub fn push_bounded_row(
    rows: &mut Vec<CursorRow>,
    payload_bytes: &mut u64,
    row: CursorRow,
    max_rows: usize,
    limit: u64,
) -> Result<bool, RowAdmissionError> {
    if let Some(previous) = rows.last()
        && !(matches!(previous.cursor, Cursor::Snapshot) && matches!(row.cursor, Cursor::Snapshot))
        && row
            .cursor
            .compare(&previous.cursor)
            .map_err(|_| RowAdmissionError::InvalidCursor)?
            != std::cmp::Ordering::Greater
    {
        return Err(RowAdmissionError::InvalidCursor);
    }

    let next_payload = payload_bytes.saturating_add(row_heap_bytes(&row));
    let slot = size_of::<CursorRow>() as u64;
    let minimum = next_payload.saturating_add((rows.len() as u64 + 1).saturating_mul(slot));
    if rows.len() >= max_rows || minimum > limit {
        return if rows.is_empty() {
            Err(RowAdmissionError::Limit {
                normalized_bytes: minimum,
            })
        } else {
            Ok(false)
        };
    }
    let available_slots = ((limit - next_payload) / slot).min(max_rows as u64) as usize;
    if rows.capacity() > available_slots {
        // Speculative spare slots must not reject a candidate that really fits.
        rows.shrink_to_fit();
    }
    if rows.len() == rows.capacity() {
        let target = rows
            .capacity()
            .saturating_mul(2)
            .max(4)
            .min(available_slots);
        rows.try_reserve_exact(target - rows.len())
            .map_err(|_| RowAdmissionError::Limit {
                normalized_bytes: minimum,
            })?;
    }
    let allocated = next_payload.saturating_add((rows.capacity() as u64).saturating_mul(slot));
    if allocated > limit {
        return Err(RowAdmissionError::Limit {
            normalized_bytes: allocated,
        });
    }
    *payload_bytes = next_payload;
    rows.push(row);
    Ok(true)
}
