// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Contract between the shared polling core and database-specific drivers.

use super::{ColumnMetadata, CompiledQuery, Row};
use async_trait::async_trait;
use std::error::Error;

/// Stable OpenTelemetry database system identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseSystem {
    /// Oracle Database.
    Oracle,
}

impl DatabaseSystem {
    /// Returns the semantic-convention value for `db.system.name`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Oracle => "oracle",
        }
    }
}

/// One bounded, normalized query result.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryResult {
    /// Result columns shared by every row.
    pub columns: Vec<ColumnMetadata>,
    /// Rows returned by this poll.
    pub rows: Vec<Row>,
    /// Total normalized value bytes.
    pub normalized_bytes: u64,
}

/// Database-specific query execution required by the shared receiver.
#[async_trait(?Send)]
pub trait DriverAdapter {
    /// Adapter-specific error with its diagnostic source chain intact.
    type Error: Error + 'static;

    /// Returns the adapter's stable database system identity.
    fn system(&self) -> DatabaseSystem;

    /// Inspects live result metadata without returning rows.
    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error>;

    /// Executes one compiled query and returns a bounded normalized result.
    async fn execute(&mut self, query: &CompiledQuery) -> Result<QueryResult, Self::Error>;

    /// Returns whether this failure should discard only the current result batch.
    fn is_batch_error(&self, _error: &Self::Error) -> bool {
        false
    }
}
