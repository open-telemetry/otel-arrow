// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral cursor and bounded page contracts.
//!
//! Scalar and composite positions retain their exact values across delivery
//! and restart. Comparison is explicit and rejects incompatible cursor types.

use super::row::{ColumnMetadata, Row};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Ordered position of one database row under composite watermark mode.
///
/// The timestamp is retained as adapter-normalized text so no precision is
/// lost between the database, the durable checkpoint, and the next bind.
/// Equality compares the stored representation, not normalized instants.
/// Chronological comparison must validate and normalize timestamps explicitly.
///
/// Scenario: Equivalent timestamps have different fractional-second spellings.
/// Guarantees: Cursor ordering cannot accidentally use lexical string comparison.
///
/// ```compile_fail
/// use otel_arrow_dfe_scraper::database::CompositeCursor;
///
/// let first = CompositeCursor::new("2026-01-01 10:00:00.0 +00:00".into(), 1);
/// let next = CompositeCursor::new("2026-01-01 10:00:00 +00:00".into(), 2);
/// assert!(first < next);
/// ```
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompositeCursor {
    /// Ordered timestamp component using UTC semantics.
    pub timestamp: String,
    /// Non-null tie-breaker that is unique inside one timestamp group.
    pub tie_breaker: i64,
}

impl CompositeCursor {
    /// Creates a composite cursor from its ordered components.
    #[must_use]
    pub const fn new(timestamp: String, tie_breaker: i64) -> Self {
        Self {
            timestamp,
            tie_breaker,
        }
    }
}

impl fmt::Debug for CompositeCursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompositeCursor")
            .field("timestamp", &"<redacted>")
            .field("tie_breaker", &"<redacted>")
            .finish()
    }
}

/// One normalized row paired with the cursor it occupies in the ordered result.
#[derive(Clone, Debug, PartialEq)]
pub struct CursorRow {
    /// Ordered values matching the page's result metadata.
    pub row: Row,
    /// Position of this row in the query's required ascending ordering.
    pub cursor: Cursor,
}

/// One bounded page fetched after a committed cursor.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryPage {
    /// Result columns shared by every row.
    pub columns: Vec<ColumnMetadata>,
    /// Rows returned by this poll in the query's required ascending ordering.
    pub rows: Vec<CursorRow>,
}

impl QueryPage {
    /// Returns whether this page contains no rows to emit.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// A typed position passed unchanged from the driver through durable delivery.
///
/// The untagged composite representation preserves version-1 checkpoint bytes.
/// Scalar values have an explicit `type` tag and cannot be read as composites.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Cursor {
    /// Timestamp and signed tie-breaker position.
    Composite(CompositeCursor),
    /// Single-column, strictly ordered position.
    Scalar(super::scalar::ScalarValue),
}

impl Cursor {
    /// Creates a composite position.
    #[must_use]
    pub const fn composite(timestamp: String, tie_breaker: i64) -> Self {
        Self::Composite(CompositeCursor::new(timestamp, tie_breaker))
    }

    /// Returns composite components, if this is a composite position.
    #[must_use]
    pub const fn as_composite(&self) -> Option<&CompositeCursor> {
        match self {
            Self::Composite(value) => Some(value),
            Self::Scalar(_) => None,
        }
    }

    /// Checks that the position is valid without exposing its values in errors.
    pub fn validate(&self) -> Result<(), super::scalar::CursorError> {
        match self {
            Self::Composite(value) => super::otap::parse_utc_timestamp(&value.timestamp)
                .map(|_| ())
                .map_err(|_| super::scalar::CursorError::InvalidTimestamp),
            Self::Scalar(value) => value.validate(),
        }
    }

    /// Compares compatible positions using their declared ordering semantics.
    ///
    /// Unlike derived enum ordering, different modes or scalar types are errors.
    pub fn compare(&self, other: &Self) -> Result<std::cmp::Ordering, super::scalar::CursorError> {
        use super::scalar::CursorError;
        match (self, other) {
            (Self::Composite(left), Self::Composite(right)) => {
                let left_time = super::otap::parse_utc_timestamp(&left.timestamp)
                    .map_err(|_| CursorError::InvalidTimestamp)?;
                let right_time = super::otap::parse_utc_timestamp(&right.timestamp)
                    .map_err(|_| CursorError::InvalidTimestamp)?;
                Ok((left_time, left.tie_breaker).cmp(&(right_time, right.tie_breaker)))
            }
            (Self::Scalar(left), Self::Scalar(right)) => left.compare(right),
            _ => Err(CursorError::TypeMismatch),
        }
    }
}

impl From<CompositeCursor> for Cursor {
    fn from(value: CompositeCursor) -> Self {
        Self::Composite(value)
    }
}

impl From<super::scalar::ScalarValue> for Cursor {
    fn from(value: super::scalar::ScalarValue) -> Self {
        Self::Scalar(value)
    }
}
