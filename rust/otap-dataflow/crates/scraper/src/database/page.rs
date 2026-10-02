// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral cursor and bounded page contracts.
//!
//! The first supported watermark mode is `composite`: an ordered timestamp
//! paired with a non-null `int64` tie-breaker that is unique within each
//! timestamp group. Scalar and snapshot modes require separate contract work.

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
    pub cursor: CompositeCursor,
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
