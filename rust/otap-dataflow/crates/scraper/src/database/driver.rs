// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Contract between the shared polling core and database-specific drivers.

use super::page::{Cursor, QueryPage};
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
    ///
    /// This must also work while disconnected so reconnect can be cancelled.
    /// An error must leave no operation running.
    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error>;

    /// Returns whether a failed operation can be retried after reconnecting.
    ///
    /// Only classify transient availability failures as retryable, never invalid
    /// SQL, incompatible metadata, authorization failures, or unconfirmed worker
    /// cleanup. A retryable error must confirm that the failed operation stopped
    /// and released partial results. The controller retains ownership and retries
    /// indefinitely with bounded backoff; it does not restart the pipeline.
    fn is_retryable(error: &Self::Error) -> bool;

    /// Returns whether a terminal operation failure should pause only this source.
    ///
    /// A paused source emits and checkpoints nothing, remains responsive to
    /// lifecycle control, and resumes only after receiver restart or
    /// reconfiguration. Use this for deterministic source-local conditions
    /// whose pipeline-wide failure would unnecessarily stop unrelated sources.
    fn should_pause_source(_error: &Self::Error) -> bool {
        false
    }

    /// Replaces failed connection/session state in one bounded, cancellable attempt.
    ///
    /// The controller calls `begin_operation` first, then this method, then
    /// revalidates the query before executing it again. Stop old connection and
    /// statement work before replacement; never overlap sessions with unjoined
    /// native work. Keep blocking cleanup/connect calls on the adapter-owned
    /// worker and honor cancellation and `query.timeout()` for native calls.
    /// Do not retry internally: return errors for shared retry classification.
    ///
    /// Factories must defer transient connection work to `validate_query` or this
    /// method so startup outages can also be recovered inside the receiver.
    async fn reconnect(&mut self, query: &CompiledQuery) -> Result<(), Self::Error>;

    /// Validates SQL and cursor metadata before polling starts.
    ///
    /// Each adapter must apply its dialect's rules before executing any SQL,
    /// including during preparation or metadata inspection:
    ///
    /// - Require one read-only SELECT; reject extra statements and row-locking
    ///   forms such as `SELECT ... FOR UPDATE`.
    /// - In snapshot mode, require no cursor binds or columns; ordering and
    ///   uniqueness are unnecessary. Return the entire result or an error when
    ///   row or normalized-byte bounds would truncate it.
    /// - In keyset modes, verify all configured cursor binds are real parameters used by the full keyset
    ///   predicate, which must select only rows strictly after the supplied cursor.
    /// - For keysets, require deterministic ascending ordering on the scalar key, or on the
    ///   composite timestamp and tie-breaker, consistent with the predicate.
    /// - For keysets, require present, non-null cursor columns compatible with the declared
    ///   types without rounding, truncation, or signed/unsigned coercion.
    /// - Scalar keys must be unique across the result, not merely within a page.
    ///   String keys require binary UTF-8 ordering without case folding, locale
    ///   collation, or trailing-space equivalence. Reject incompatible collations.
    /// - Timestamp keys use UTC semantics; preserve precision and normalize
    ///   timezone-less values in the adapter, never using the host timezone.
    ///
    /// Match every [`super::CompiledWatermark`] variant explicitly. An adapter
    /// may reject unsupported scalar types but must never reinterpret them as
    /// a timestamp or silently fall back to composite behavior.
    ///
    /// Reject unsupported or ambiguous forms. [`CompiledQuery::compile`]'s SELECT
    /// prefix check and a read-only account do not replace this validation.
    /// Source commit ordering, uniqueness, immutability, and retention remain
    /// separate source-data requirements.
    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error>;

    /// Executes a complete snapshot or a keyset page after the committed cursor.
    ///
    /// Snapshot implementations bind no cursor parameters and must return every
    /// row with `Cursor::Snapshot`. Detect overflow with a bounded extra-row
    /// probe and return an error instead of a partial result.
    ///
    /// Keyset implementations bind the cursor through named database parameters and
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
        cursor: &Cursor,
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
