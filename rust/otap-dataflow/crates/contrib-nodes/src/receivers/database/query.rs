// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validated, database-neutral query plans.

use super::config::{ConfigError, OutputConfig, PollingConfig};
use std::fmt;
use std::time::Duration;

const DEFAULT_MAX_NORMALIZED_BYTES: u64 = 8 * 1024 * 1024;

/// Immutable query plan passed to a database adapter.
#[derive(Clone)]
pub struct CompiledQuery {
    sql: String,
    interval: Duration,
    timeout: Duration,
    fetch_size: usize,
    max_rows: usize,
    max_normalized_bytes: u64,
    output: OutputConfig,
}

impl CompiledQuery {
    /// Validates and compiles one operator-authored snapshot query.
    pub fn compile(
        sql: String,
        config: PollingConfig,
        output: OutputConfig,
    ) -> Result<Self, QueryError> {
        config.validate()?;
        output.validate()?;
        if !is_read_only(&sql) {
            return Err(QueryError::NotReadOnly);
        }
        Ok(Self {
            sql,
            interval: config.interval,
            timeout: config.timeout,
            fetch_size: config.fetch_size,
            max_rows: config.max_rows_per_poll,
            max_normalized_bytes: DEFAULT_MAX_NORMALIZED_BYTES,
            output,
        })
    }

    /// Returns the operator-authored SQL for execution only.
    #[must_use]
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Returns the delay applied after a completed poll.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.interval
    }

    /// Returns the native database call timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Returns the maximum rows fetched in one native operation.
    #[must_use]
    pub const fn fetch_size(&self) -> usize {
        self.fetch_size
    }

    /// Returns the hard row ceiling for one poll.
    #[must_use]
    pub const fn max_rows(&self) -> usize {
        self.max_rows
    }

    /// Returns the hard normalized-byte ceiling for one poll.
    #[must_use]
    pub const fn max_normalized_bytes(&self) -> u64 {
        self.max_normalized_bytes
    }

    /// Returns the OTLP output mapping.
    #[must_use]
    pub const fn output(&self) -> &OutputConfig {
        &self.output
    }
}

impl fmt::Debug for CompiledQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledQuery")
            .field("sql", &"<redacted>")
            .field("interval", &self.interval)
            .field("timeout", &self.timeout)
            .field("fetch_size", &self.fetch_size)
            .field("max_rows", &self.max_rows)
            .field("max_normalized_bytes", &self.max_normalized_bytes)
            .field("output", &self.output)
            .finish()
    }
}

fn is_read_only(sql: &str) -> bool {
    // This foundation intentionally avoids pretending to be a SQL parser.
    // Runtime read-only transactions provide the final vendor-side guard.
    let first = sql.split_whitespace().next();
    first.is_some_and(|keyword| keyword.eq_ignore_ascii_case("select"))
        && !sql.to_ascii_uppercase().contains("FOR UPDATE")
}

/// Failure while compiling a query plan.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum QueryError {
    /// Shared polling configuration is invalid.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Only read-only SQL is accepted.
    #[error("query.statement must start with SELECT")]
    NotReadOnly,
}

#[cfg(test)]
mod tests {
    use super::{CompiledQuery, QueryError};
    use crate::receivers::database::{OutputConfig, PollingConfig};
    use std::time::Duration;

    fn polling_config() -> PollingConfig {
        PollingConfig {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            fetch_size: 100,
            max_rows_per_poll: 1_000,
        }
    }

    /// Scenario: Configuration contains a statement that can modify database state.
    /// Guarantees: Query compilation accepts only statements beginning with SELECT.
    #[test]
    fn rejects_modifying_statement() {
        let result = CompiledQuery::compile(
            "DELETE FROM audit_log".to_owned(),
            polling_config(),
            OutputConfig::default(),
        );
        assert_eq!(
            result.expect_err("DELETE must be rejected"),
            QueryError::NotReadOnly
        );
    }

    /// Scenario: A statement starts with WITH and its top-level operation is not parsed.
    /// Guarantees: The conservative first slice rejects CTE statements rather than risking DML.
    #[test]
    fn rejects_unparsed_with_statement() {
        let result = CompiledQuery::compile(
            "WITH rows AS (SELECT 1 AS id FROM dual) SELECT id FROM rows".to_owned(),
            polling_config(),
            OutputConfig::default(),
        );
        assert_eq!(
            result.expect_err("WITH is not safely classified yet"),
            QueryError::NotReadOnly
        );
    }

    /// Scenario: A SELECT statement requests locks through a FOR UPDATE clause.
    /// Guarantees: The read-only receiver rejects explicit row-locking queries.
    #[test]
    fn rejects_select_for_update() {
        let result = CompiledQuery::compile(
            "SELECT id FROM audit_log FOR UPDATE".to_owned(),
            polling_config(),
            OutputConfig::default(),
        );
        assert_eq!(
            result.expect_err("locking SELECT must be rejected"),
            QueryError::NotReadOnly
        );
    }

    /// Scenario: A compiled query is included in diagnostic output.
    /// Guarantees: SQL text is redacted while safe execution limits remain visible.
    #[test]
    fn debug_output_redacts_sql() {
        let query = CompiledQuery::compile(
            "SELECT secret FROM audit_log".to_owned(),
            polling_config(),
            OutputConfig::default(),
        )
        .expect("SELECT should compile");

        let debug = format!("{query:?}");
        assert!(!debug.contains("audit_log"));
        assert!(debug.contains("<redacted>"));
    }
}
