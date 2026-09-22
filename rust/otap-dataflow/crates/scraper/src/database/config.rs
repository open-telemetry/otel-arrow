// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral polling, watermark, and checkpoint configuration.

use serde::Deserialize;
use std::fmt;
use std::time::Duration;

const MAX_ROWS_PER_POLL: usize = 10_000;
const MAX_FETCH_SIZE: usize = 10_000;
const MIN_INTERVAL: Duration = Duration::from_millis(1);
const MAX_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_NACK_BACKOFF: Duration = Duration::from_secs(5 * 60);
const MAX_BYTE_LIMIT: u64 = 256 * 1024 * 1024;
const MAX_CONSECUTIVE_FAILURES: u32 = 1_000;
const MAX_CATCH_UP_PAGES: usize = 1024;
const MAX_CATCH_UP_DURATION: Duration = Duration::from_secs(5 * 60);

/// Budgets for immediately fetching additional acknowledged pages.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CatchUpConfig {
    /// Maximum query-page fetches admitted in one cycle, including empty probes.
    pub max_pages: usize,
    /// Elapsed cycle budget that gates the next fetch, not an in-flight deadline.
    #[serde(with = "humantime_serde")]
    pub max_duration: Duration,
}

impl Default for CatchUpConfig {
    fn default() -> Self {
        Self {
            max_pages: 32,
            max_duration: Duration::from_secs(10),
        }
    }
}

/// Bounds and timing shared by every database receiver.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollingConfig {
    /// Delay after a poll cycle ends, not between its acknowledged pages.
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    /// Native database call timeout.
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// Hard row limit for one fetched page, including in a catch-up cycle.
    pub max_rows_per_poll: usize,
    /// Target number of rows fetched per native driver round trip.
    pub fetch_size: usize,
    /// Byte ceiling applied separately to normalized rows and serialized OTLP.
    pub max_batch_bytes: u64,
    /// Cycle budgets; omitted fields use defaults. Set max_pages to 1 for single-page cycles.
    #[serde(default)]
    pub catch_up: CatchUpConfig,
}

/// Watermark mode selected by the operator.
///
/// Only `composite` is implemented. `scalar` and `snapshot` are deliberately
/// absent from this enum so an operator configuring them receives a schema
/// error instead of silently inheriting composite behavior.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum WatermarkConfig {
    /// Ordered timestamp plus a unique non-null `int64` tie-breaker.
    Composite {
        /// Timestamp cursor component.
        timestamp: TimestampCursorConfig,
        /// Tie-breaker cursor component.
        tie_breaker: TieBreakerCursorConfig,
    },
}

/// Timestamp component of a composite watermark.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampCursorConfig {
    /// Result column holding the ordered timestamp.
    pub column: String,
    /// Named bind carrying the committed timestamp, without a leading colon.
    pub bind: String,
    /// Timestamp used before any checkpoint exists; redacted in debug output.
    pub initial: String,
    /// Cursor timezone. Only `UTC` is supported.
    pub timezone: String,
}

impl fmt::Debug for TimestampCursorConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TimestampCursorConfig")
            .field("column", &self.column)
            .field("bind", &self.bind)
            .field("initial", &"<redacted>")
            .field("timezone", &self.timezone)
            .finish()
    }
}

/// Tie-breaker component of a composite watermark.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TieBreakerCursorConfig {
    /// Result column holding the non-null `int64` tie-breaker.
    pub column: String,
    /// Named bind carrying the committed tie-breaker, without a leading colon.
    pub bind: String,
    /// Tie-breaker used before any checkpoint exists; redacted in debug output.
    pub initial: i64,
}

impl fmt::Debug for TieBreakerCursorConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TieBreakerCursorConfig")
            .field("column", &self.column)
            .field("bind", &self.bind)
            .field("initial", &"<redacted>")
            .finish()
    }
}

/// Behavior applied when a downstream node negatively acknowledges a page.
///
/// Only `rewind` is implemented. A terminal `fail` policy is deferred so an
/// operator cannot select a mode the receiver does not honor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OnNack {
    /// Retain the durable cursor and re-query the same page after a backoff.
    Rewind,
}

/// Durable checkpoint policy.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointConfig {
    /// Root directory for revisioned checkpoint files.
    pub directory: String,
    /// Behavior on a negative acknowledgement.
    pub on_nack: OnNack,
    /// Fixed delay before replaying a negatively acknowledged page.
    #[serde(with = "humantime_serde")]
    pub nack_backoff: Duration,
    /// Consecutive durable-write failures before the receiver terminates.
    pub max_consecutive_failures: u32,
}

/// Required output columns for the initial all-column body mapping.
#[derive(Clone, Debug, Default)]
pub struct OutputConfig {
    /// Optional result column used as the OTLP event timestamp.
    pub timestamp_column: Option<String>,
    /// Result columns that must exist even when not emitted specially.
    pub validation_columns: Vec<String>,
}

impl OutputConfig {
    /// Validates mappings that do not require live result metadata.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(column) = &self.timestamp_column {
            validate_name("query.output.timestamp_column", column)?;
        }
        for column in &self.validation_columns {
            validate_name("query.output.validation_columns", column)?;
        }
        Ok(())
    }
}

impl PollingConfig {
    /// Validates timing, row, and byte bounds.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.interval.is_zero() {
            return Err(ConfigError::new("query.interval must be greater than zero"));
        }
        if !(MIN_INTERVAL..=MAX_INTERVAL).contains(&self.interval) {
            return Err(ConfigError::new(format!(
                "query.interval must be between 1ms and {}s",
                MAX_INTERVAL.as_secs()
            )));
        }
        if self.timeout.is_zero() {
            return Err(ConfigError::new("query.timeout must be greater than zero"));
        }
        if self.max_rows_per_poll == 0 {
            return Err(ConfigError::new(
                "query.max_rows_per_poll must be greater than zero",
            ));
        }
        if self.fetch_size == 0 {
            return Err(ConfigError::new(
                "query.fetch_size must be greater than zero",
            ));
        }
        if self.max_batch_bytes == 0 {
            return Err(ConfigError::new(
                "query.max_batch_bytes must be greater than zero",
            ));
        }
        if self.max_batch_bytes > MAX_BYTE_LIMIT {
            return Err(ConfigError::new(format!(
                "query.max_batch_bytes must not exceed {MAX_BYTE_LIMIT} bytes"
            )));
        }
        if self.fetch_size > MAX_FETCH_SIZE {
            return Err(ConfigError::new(format!(
                "query.fetch_size must not exceed {MAX_FETCH_SIZE}"
            )));
        }
        if self.fetch_size > self.max_rows_per_poll {
            return Err(ConfigError::new(
                "query.fetch_size must not exceed query.max_rows_per_poll",
            ));
        }
        if self.max_rows_per_poll > MAX_ROWS_PER_POLL {
            return Err(ConfigError::new(format!(
                "query.max_rows_per_poll must not exceed {MAX_ROWS_PER_POLL}"
            )));
        }
        self.catch_up.validate()?;
        Ok(())
    }
}

impl CatchUpConfig {
    /// Validates the explicit page and elapsed-time budgets.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(1..=MAX_CATCH_UP_PAGES).contains(&self.max_pages) {
            return Err(ConfigError::new(format!(
                "query.catch_up.max_pages must be between 1 and {MAX_CATCH_UP_PAGES}"
            )));
        }
        if !(MIN_INTERVAL..=MAX_CATCH_UP_DURATION).contains(&self.max_duration) {
            return Err(ConfigError::new(
                "query.catch_up.max_duration must be between 1ms and 5min",
            ));
        }
        Ok(())
    }
}

impl WatermarkConfig {
    /// Validates cursor identifiers, bind names, and timezone semantics.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let Self::Composite {
            timestamp,
            tie_breaker,
        } = self;
        validate_name("watermark.timestamp.column", &timestamp.column)?;
        validate_name("watermark.tie_breaker.column", &tie_breaker.column)?;
        validate_bind("watermark.timestamp.bind", &timestamp.bind)?;
        validate_bind("watermark.tie_breaker.bind", &tie_breaker.bind)?;
        if timestamp.initial.trim().is_empty() {
            return Err(ConfigError::new(
                "watermark.timestamp.initial must not be empty",
            ));
        }
        if !timestamp.timezone.eq_ignore_ascii_case("UTC") {
            return Err(ConfigError::new("watermark.timestamp.timezone must be UTC"));
        }
        if timestamp.bind.eq_ignore_ascii_case(&tie_breaker.bind) {
            return Err(ConfigError::new("watermark bind names must be distinct"));
        }
        if timestamp.column.eq_ignore_ascii_case(&tie_breaker.column) {
            return Err(ConfigError::new(
                "watermark cursor columns must be distinct",
            ));
        }
        Ok(())
    }

    /// Returns the composite timestamp cursor component.
    #[must_use]
    pub const fn timestamp(&self) -> &TimestampCursorConfig {
        let Self::Composite { timestamp, .. } = self;
        timestamp
    }

    /// Returns the composite tie-breaker cursor component.
    #[must_use]
    pub const fn tie_breaker(&self) -> &TieBreakerCursorConfig {
        let Self::Composite { tie_breaker, .. } = self;
        tie_breaker
    }
}

impl CheckpointConfig {
    /// Validates checkpoint location, backoff, and failure bounds.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_name("checkpoint.directory", &self.directory)?;
        if std::path::Path::new(&self.directory)
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(ConfigError::new(
                "checkpoint.directory must not contain '..' components",
            ));
        }
        if self.nack_backoff < MIN_INTERVAL || self.nack_backoff > MAX_NACK_BACKOFF {
            return Err(ConfigError::new(format!(
                "checkpoint.nack_backoff must be between 1ms and {}s",
                MAX_NACK_BACKOFF.as_secs()
            )));
        }
        if self.max_consecutive_failures == 0
            || self.max_consecutive_failures > MAX_CONSECUTIVE_FAILURES
        {
            return Err(ConfigError::new(format!(
                "checkpoint.max_consecutive_failures must be between 1 and {MAX_CONSECUTIVE_FAILURES}"
            )));
        }
        Ok(())
    }
}

fn validate_name(field: &'static str, name: &str) -> Result<(), ConfigError> {
    if name.trim().is_empty() {
        Err(ConfigError::new(format!("{field} names must not be empty")))
    } else {
        Ok(())
    }
}

/// Validates a named bind so it can never be confused with inline SQL text.
fn validate_bind(field: &'static str, name: &str) -> Result<(), ConfigError> {
    validate_name(field, name)?;
    let mut bytes = name.bytes();
    let first = bytes.next().unwrap_or(b'0');
    if !(first.is_ascii_alphabetic() || first == b'_')
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(ConfigError::new(format!(
            "{field} must omit ':' and contain only ASCII alphanumerics or '_'"
        )));
    }
    Ok(())
}

/// Invalid database receiver configuration.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("{0}")]
pub struct ConfigError(String);

impl ConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}
