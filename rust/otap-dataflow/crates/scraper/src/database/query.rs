// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validated, database-neutral query plans.

use super::config::{
    CatchUpConfig, CheckpointConfig, ConfigError, OnPermanentNack, OutputConfig, PollingConfig,
    WatermarkConfig,
};
use super::page::{CompositeCursor, Cursor};
use super::scalar::{CursorError, ScalarValue};
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

/// Validated binds and columns for a single-column keyset query.
#[derive(Clone, Debug)]
pub struct ScalarWatermark {
    /// Result column containing the ordered key.
    pub column: String,
    /// Named parameter bound without converting the value to SQL text.
    pub bind: String,
    /// Typed starting value used when no checkpoint exists.
    pub initial: ScalarValue,
}

/// Mode-specific contract that adapters must validate before execution.
#[derive(Clone, Debug)]
pub enum CompiledWatermark {
    /// One complete result per polling interval, with no source position.
    Snapshot,
    /// Timestamp plus tie-breaker keyset.
    Composite(CompositeWatermark),
    /// One typed, unique, strictly ordered key.
    Scalar(ScalarWatermark),
}

impl CompiledWatermark {
    /// Returns an owned initial position for the receiver state.
    #[must_use]
    pub fn initial(&self) -> Cursor {
        match self {
            Self::Snapshot => Cursor::Snapshot,
            Self::Composite(value) => value.initial.clone().into(),
            Self::Scalar(value) => value.initial.clone().into(),
        }
    }

    /// Returns composite bind metadata only in composite mode.
    #[must_use]
    pub const fn as_composite(&self) -> Option<&CompositeWatermark> {
        match self {
            Self::Composite(value) => Some(value),
            Self::Scalar(_) | Self::Snapshot => None,
        }
    }

    /// Rejects checkpoint or driver positions that do not match this query.
    pub fn validate_cursor(&self, cursor: &Cursor) -> Result<(), CursorError> {
        cursor.validate()?;
        match (self, cursor) {
            (Self::Snapshot, Cursor::Snapshot) => Ok(()),
            (Self::Composite(_), Cursor::Composite(_)) => Ok(()),
            (Self::Scalar(spec), Cursor::Scalar(value)) if spec.initial.same_type(value) => Ok(()),
            _ => Err(CursorError::TypeMismatch),
        }
    }
}

/// Immutable query plan passed to a database adapter.
#[derive(Clone)]
pub struct CompiledQuery {
    sql: String,
    interval: Duration,
    timeout: Duration,
    fetch_size_rows: usize,
    max_rows: usize,
    max_batch_bytes: u64,
    catch_up: CatchUpConfig,
    on_permanent_nack: OnPermanentNack,
    watermark: CompiledWatermark,
    output: OutputConfig,
}

impl CompiledQuery {
    /// Validates and compiles one operator-authored watermark query.
    ///
    /// SQL checks here cover length and a leading SELECT keyword only.
    /// Before execution, [`super::DriverAdapter::validate_query`] must validate
    /// the complete vendor statement, its read-only behavior, and cursor
    /// semantics. A compiled plan is not proof that SQL is safe to execute.
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
        let watermark = match watermark {
            WatermarkConfig::Snapshot {} => CompiledWatermark::Snapshot,
            WatermarkConfig::Composite {
                timestamp,
                tie_breaker,
            } => CompiledWatermark::Composite(CompositeWatermark {
                timestamp_column: timestamp.column.clone(),
                timestamp_bind: timestamp.bind.clone(),
                tie_breaker_column: tie_breaker.column.clone(),
                tie_breaker_bind: tie_breaker.bind.clone(),
                initial: CompositeCursor::new(timestamp.initial.clone(), tie_breaker.initial),
            }),
            WatermarkConfig::Scalar {
                column,
                bind,
                initial,
            } => CompiledWatermark::Scalar(ScalarWatermark {
                column: column.clone(),
                bind: bind.clone(),
                initial: initial.clone(),
            }),
        };
        Ok(Self {
            sql,
            interval: config.interval,
            timeout: config.timeout,
            fetch_size_rows: config.fetch_size_rows,
            max_rows: config.max_rows_per_poll,
            max_batch_bytes: config.max_batch_bytes,
            catch_up: config.catch_up,
            on_permanent_nack: checkpoint.on_permanent_nack,
            watermark,
            output,
        })
    }

    /// Returns the operator-authored SQL for execution only.
    #[must_use]
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Returns the delay applied after a poll cycle ends.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.interval
    }

    /// Returns the native database call timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Returns the hard row ceiling for one fetched page.
    #[must_use]
    pub const fn max_rows(&self) -> usize {
        self.max_rows
    }

    /// Returns the target number of rows per native driver fetch, not bytes.
    #[must_use]
    pub const fn fetch_size_rows(&self) -> usize {
        self.fetch_size_rows
    }

    /// Returns the exact serialized OTLP ceiling for one emitted page.
    #[must_use]
    pub const fn max_batch_bytes(&self) -> u64 {
        self.max_batch_bytes
    }

    /// Returns the cycle budgets without allocating or cloning the plan.
    #[must_use]
    pub const fn catch_up(&self) -> CatchUpConfig {
        self.catch_up
    }

    /// Returns the source-local policy for permanent downstream rejection.
    #[must_use]
    pub const fn on_permanent_nack(&self) -> OnPermanentNack {
        self.on_permanent_nack
    }

    /// Returns the normalized-row ceiling using the existing batch byte setting.
    ///
    /// Normalized storage and encoded payloads are checked separately against
    /// this value; it is not a combined process-memory ceiling.
    #[must_use]
    pub const fn max_normalized_bytes(&self) -> u64 {
        self.max_batch_bytes
    }

    /// Returns the mode-specific cursor binds and columns.
    #[must_use]
    pub const fn watermark(&self) -> &CompiledWatermark {
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
            .field("fetch_size_rows", &self.fetch_size_rows)
            .field("max_rows", &self.max_rows)
            .field("max_batch_bytes", &self.max_batch_bytes)
            .field("catch_up", &self.catch_up)
            .field("on_permanent_nack", &self.on_permanent_nack)
            .field("watermark", &self.watermark)
            .field("output", &self.output)
            .finish()
    }
}

fn is_read_only(sql: &str) -> bool {
    // This is only a prefix check. DriverAdapter::validate_query must enforce
    // single-statement, read-only SQL and cursor semantics before execution.
    sql.split_whitespace()
        .next()
        .is_some_and(|keyword| keyword.eq_ignore_ascii_case("select"))
}

/// Failure while compiling a query plan.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum QueryError {
    /// Shared receiver configuration is invalid.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// The statement does not start with the required SELECT keyword.
    #[error("query.statement must start with SELECT")]
    NotReadOnly,
    /// The statement exceeds the supported length.
    #[error("query.statement must be at most {maximum} bytes")]
    QueryTooLong {
        /// Largest supported statement length.
        maximum: usize,
    },
}
