// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Narrow database boundary for SQL polling receivers.

use async_trait::async_trait;
use std::error::Error as StdError;

/// Credentials supplied when an adapter opens a database session.
///
/// This type intentionally does not implement `Debug` so passwords cannot be
/// accidentally included in diagnostic output.
pub(crate) struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    /// Creates username/password credentials.
    pub(crate) const fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    /// Consumes the credentials into values accepted by a database driver.
    pub(crate) fn into_parts(self) -> (String, String) {
        (self.username, self.password)
    }
}

/// Stable position used by paged SQL polling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompoundWatermark {
    /// Ordered timestamp component.
    pub(crate) timestamp: String,
    /// Unique tie-breaker for rows sharing the timestamp.
    pub(crate) tie_breaker: String,
}

/// One bounded page request sent to a SQL adapter.
pub(crate) struct PageRequest {
    /// Last acknowledged position, or `None` for an initial/stateless read.
    pub(crate) watermark: Option<CompoundWatermark>,
    /// Maximum number of rows the adapter may return.
    pub(crate) limit: usize,
}

/// One database-neutral string value extracted from a SQL row.
pub(crate) struct SqlColumn {
    /// Database column name.
    pub(crate) name: String,
    /// String value, or `None` for SQL `NULL`.
    pub(crate) value: Option<String>,
}

/// One database row extracted by an adapter.
pub(crate) struct SqlRow {
    /// Ordered columns in the row.
    pub(crate) columns: Vec<SqlColumn>,
}

/// One bounded page returned by an adapter.
pub(crate) struct Page {
    /// Extracted rows.
    pub(crate) rows: Vec<SqlRow>,
}

/// Database-neutral error category returned by an adapter classifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ErrorClass {
    /// Invalid polling request or unsupported configuration.
    Configuration,
    /// Failure while opening a database session.
    Connection,
    /// Failure while executing or reading a query.
    Query,
    /// Failure in the adapter's execution boundary.
    Internal,
}

/// Database-specific operations required by the SQL polling lifecycle.
///
/// The associated session and error types prevent the shared boundary from
/// exposing Oracle client types or pretending all SQL engines behave alike.
#[async_trait(?Send)]
pub(crate) trait SqlPollingAdapter {
    /// Driver-specific database session or pool.
    type Session;
    /// Driver-specific error.
    type Error: StdError + 'static;

    /// Opens a database session using externally loaded credentials.
    async fn connect(&self, credentials: Credentials) -> Result<Self::Session, Self::Error>;

    /// Fetches one bounded page after the supplied watermark.
    async fn fetch_page(
        &self,
        session: &mut Self::Session,
        request: PageRequest,
    ) -> Result<Page, Self::Error>;

    /// Maps a driver-specific error to the shared polling taxonomy.
    fn classify_error(&self, error: &Self::Error) -> ErrorClass;
}
