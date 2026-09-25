// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Contract between the shared polling core and database-specific drivers.

use super::page::{CompositeCursor, QueryPage};
use super::query::CompiledQuery;
use super::row::ColumnMetadata;
use async_trait::async_trait;
use otel_arrow_dfe_engine::error::ReceiverErrorKind;
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
            Self::Oracle => "oracle.db",
        }
    }
}

/// Cancellation handle for one database operation running outside the local async core.
#[async_trait(?Send)]
pub trait DriverCancellation: Clone {
    /// Adapter error returned when cancellation cannot be requested.
    type Error: Error + 'static;

    /// Requests cancellation of the whole current operation, not just one native call.
    ///
    /// Set an operation-scoped cancellation flag before interrupting native work.
    /// Any potentially uninterruptible native cancellation call must run outside
    /// the pipeline's Tokio blocking pool so runtime teardown cannot join it indefinitely.
    async fn cancel(&self) -> Result<(), Self::Error>;
}

/// Database-specific query execution required by the shared receiver.
#[async_trait(?Send)]
pub trait DriverAdapter {
    /// Adapter-specific error; implementations must redact sensitive diagnostic data.
    type Error: Error + 'static;
    /// Cloneable handle used to interrupt one active native operation.
    type Cancellation: DriverCancellation<Error = Self::Error>;

    /// Returns the adapter's stable database system identity.
    fn system(&self) -> DatabaseSystem;

    /// Resets cancellation state before one native operation starts.
    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error>;

    /// Validates SQL and cursor metadata before polling starts.
    ///
    /// Each adapter must apply its dialect's rules before executing any SQL,
    /// including during preparation or metadata inspection:
    ///
    /// - Require one read-only SELECT; reject extra statements and row-locking
    ///   forms such as `SELECT ... FOR UPDATE`.
    /// - Verify both cursor binds are real parameters used by the full keyset
    ///   predicate, which must select only rows strictly after the supplied cursor.
    /// - Require deterministic ascending timestamp/tie-breaker ordering consistent
    ///   with the predicate and selected cursor columns.
    /// - Require present, non-null cursor columns compatible with the UTC timestamp
    ///   and signed `int64` tie-breaker contract without lossy conversion.
    ///
    /// Reject unsupported or ambiguous forms. [`CompiledQuery::compile`]'s SELECT
    /// prefix check and a read-only account do not replace this validation.
    /// Source commit ordering, uniqueness, immutability, and retention remain
    /// separate source-data requirements.
    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error>;

    /// Executes one compiled query strictly after the committed cursor.
    ///
    /// Implementations bind the cursor through named database parameters and
    /// return a bounded page whose rows each carry their own cursor.
    /// Callers must successfully validate this same query with
    /// [`Self::validate_query`] before its first execution. Substitute cursor
    /// values through parameter binding, never SQL string concatenation.
    ///
    /// Check operation cancellation before and after each native fetch and between
    /// native calls made during value conversion. A successful interruption of one
    /// call must not let the loop start another call for the same cancelled operation.
    /// Keep blocking driver work on an adapter-owned bounded worker, not on the
    /// pipeline thread or its runtime-owned blocking pool.
    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error>;

    /// Stops the worker and destroys native resources off the pipeline thread.
    ///
    /// Success must confirm that operation work and native cleanup have stopped.
    /// Dropping or timing out this future does not prove the worker stopped; the
    /// receiver retains ownership when cleanup cannot be confirmed.
    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Classifies a terminal adapter failure for receiver diagnostics.
    fn classify_error(_error: &Self::Error) -> ReceiverErrorKind {
        ReceiverErrorKind::Other
    }
}
