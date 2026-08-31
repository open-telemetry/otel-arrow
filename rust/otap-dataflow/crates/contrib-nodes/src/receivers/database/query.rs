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
