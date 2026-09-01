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
            Self::Oracle => "oracle",
        }
    }
}

/// Cancellation handle for one database operation running outside the local async core.
#[async_trait(?Send)]
pub trait DriverCancellation: Clone {
    /// Adapter error returned when cancellation cannot be requested.
    type Error: Error + 'static;

    /// Requests cancellation of the current native database operation.
    async fn cancel(&self) -> Result<(), Self::Error>;
}

/// Database-specific query execution required by the shared receiver.
#[async_trait(?Send)]
pub trait DriverAdapter {
    /// Adapter-specific error with its diagnostic source chain intact.
    type Error: Error + 'static;
    /// Cloneable handle used to interrupt one active native operation.
    type Cancellation: DriverCancellation<Error = Self::Error>;

    /// Returns the adapter's stable database system identity.
    fn system(&self) -> DatabaseSystem;

    /// Resets cancellation state before one native operation starts.
    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error>;

    /// Inspects live result metadata and validates cursor columns.
    ///
    /// Implementations must reject cursor columns whose vendor types cannot
    /// produce a deterministic, non-null composite cursor.
    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error>;

    /// Executes one compiled query strictly after the committed cursor.
    ///
    /// Implementations bind the cursor through named database parameters and
    /// return a bounded page whose rows each carry their own cursor.
    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error>;

    /// Classifies a terminal adapter failure for receiver diagnostics.
    fn classify_error(_error: &Self::Error) -> ReceiverErrorKind {
        ReceiverErrorKind::Other
    }
}
