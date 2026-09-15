// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validated, database-neutral query plans.

use super::config::{CheckpointConfig, ConfigError, OutputConfig, PollingConfig, WatermarkConfig};
use super::page::CompositeCursor;
use std::fmt;
use std::time::Duration;

const MAX_QUERY_BYTES: usize = 16 * 1024;

/// Cursor binds and columns required by composite watermark mode.
#[derive(Clone, Debug)]
pub struct CompositeWatermark {
    /// Result column holding the ordered timestamp.
    pub timestamp_column: String,
    /// Named bind carrying the committed timestamp, without a leading colon.
    pub timestamp_bind: String,
    /// Result column holding the non-null `int64` tie-breaker.
    pub tie_breaker_column: String,
    /// Named bind carrying the committed tie-breaker, without a leading colon.
    pub tie_breaker_bind: String,
    /// Cursor used before any durable checkpoint exists.
    pub initial: CompositeCursor,
}

/// Immutable query plan passed to a database adapter.
#[derive(Clone)]
pub struct CompiledQuery {
    sql: String,
    interval: Duration,
    timeout: Duration,
    fetch_size: usize,
    max_rows: usize,
    max_batch_bytes: u64,
    max_normalized_bytes: u64,
    watermark: CompositeWatermark,
    output: OutputConfig,
}

impl CompiledQuery {
    /// Validates and compiles one operator-authored composite watermark query.
    pub fn compile(
        sql: String,
        config: PollingConfig,
        watermark: &WatermarkConfig,
        checkpoint: &CheckpointConfig,
        output: OutputConfig,
    ) -> Result<Self, QueryError> {
        config.validate()?;
        watermark.validate()?;
        checkpoint.validate()?;
        output.validate()?;
        if sql.len() > MAX_QUERY_BYTES {
            return Err(QueryError::QueryTooLong {
                maximum: MAX_QUERY_BYTES,
            });
        }
        if !is_read_only(&sql) {
            return Err(QueryError::NotReadOnly);
        }
        let timestamp = watermark.timestamp();
        let tie_breaker = watermark.tie_breaker();
        Ok(Self {
            sql,
            interval: config.interval,
            timeout: config.timeout,
            fetch_size: config.fetch_size,
            max_rows: config.max_rows_per_poll,
            max_batch_bytes: config.max_batch_bytes,
            max_normalized_bytes: config.max_normalized_bytes,
            watermark: CompositeWatermark {
                timestamp_column: timestamp.column.clone(),
                timestamp_bind: timestamp.bind.clone(),
                tie_breaker_column: tie_breaker.column.clone(),
                tie_breaker_bind: tie_breaker.bind.clone(),
                initial: CompositeCursor::new(timestamp.initial.clone(), tie_breaker.initial),
            },
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

    /// Returns the requested initial row capacity for one driver page.
    ///
    /// Adapters may use a smaller native fetch array when result metadata is
    /// needed to prove that the allocation remains bounded.
    #[must_use]
    pub const fn fetch_size(&self) -> usize {
        self.fetch_size
    }

    /// Returns the hard row ceiling for one poll.
    #[must_use]
    pub const fn max_rows(&self) -> usize {
        self.max_rows
    }

    /// Returns the exact serialized OTLP ceiling for one emitted page.
    #[must_use]
    pub const fn max_batch_bytes(&self) -> u64 {
        self.max_batch_bytes
    }

    /// Returns the normalized in-memory ceiling for one poll before encoding.
    #[must_use]
    pub const fn max_normalized_bytes(&self) -> u64 {
        self.max_normalized_bytes
    }

    /// Returns the composite cursor binds and columns.
    #[must_use]
    pub const fn watermark(&self) -> &CompositeWatermark {
        &self.watermark
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
            .field("max_batch_bytes", &self.max_batch_bytes)
            .field("max_normalized_bytes", &self.max_normalized_bytes)
            .field("watermark", &self.watermark)
            .field("output", &self.output)
            .finish()
    }
}

fn is_read_only(sql: &str) -> bool {
    // Vendor adapters add token-level validation and a runtime read-only
    // transaction. This shared check only rejects obviously unsafe statements.
    let first = sql.split_whitespace().next();
    first.is_some_and(|keyword| keyword.eq_ignore_ascii_case("select"))
        && !sql.to_ascii_uppercase().contains("FOR UPDATE")
}

/// Failure while compiling a query plan.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum QueryError {
    /// Shared receiver configuration is invalid.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Only read-only SQL is accepted.
    #[error("query.statement must start with SELECT")]
    NotReadOnly,
    /// The statement exceeds the supported length.
    #[error("query.statement must be at most {maximum} bytes")]
    QueryTooLong {
        /// Largest supported statement length.
        maximum: usize,
    },
}
