// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral cursor and bounded page contracts.
//!
//! The first supported watermark mode is `composite`: an ordered timestamp
//! paired with a non-null `int64` tie-breaker that is unique within each
//! timestamp group. Keeping the cursor in its own type lets future scalar and
//! snapshot modes be added without changing the driver contract's shape.

use super::row::{ColumnMetadata, Row};
use serde::{Deserialize, Serialize};

/// Ordered position of one database row under composite watermark mode.
///
/// The timestamp is retained as adapter-normalized text so no precision is
/// lost between the database, the durable checkpoint, and the next bind.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;
