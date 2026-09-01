// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral polling, watermark, and checkpoint configuration.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

const MAX_ROWS_PER_POLL: usize = 10_000;
const MIN_INTERVAL: Duration = Duration::from_millis(1);
const MAX_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_NACK_BACKOFF: Duration = Duration::from_secs(5 * 60);
const MAX_BYTE_LIMIT: u64 = 256 * 1024 * 1024;
const MAX_CONSECUTIVE_FAILURES: u32 = 1_000;

/// Bounds and timing shared by every database receiver.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollingConfig {
    /// Delay between completed query executions.
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    /// Native database call timeout.
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// Initial row capacity reserved for one driver result.
    ///
    /// Adapters may apply a smaller native fetch size when vendor metadata is
    /// required to calculate a safe allocation.
    pub fetch_size: usize,
    /// Hard row limit for one poll.
    pub max_rows_per_poll: usize,
    /// Exact serialized OTLP payload ceiling for one emitted page.
    pub max_batch_bytes: u64,
    /// Hard normalized in-memory byte limit for one poll before encoding.
    pub max_normalized_bytes: u64,
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
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampCursorConfig {
    /// Result column holding the ordered timestamp.
    pub column: String,
    /// Named bind carrying the committed timestamp, without a leading colon.
    pub bind: String,
    /// Timestamp used before any checkpoint exists.
    pub initial: String,
    /// Cursor timezone. Only `UTC` is supported.
    pub timezone: String,
}

/// Tie-breaker component of a composite watermark.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TieBreakerCursorConfig {
    /// Result column holding the non-null `int64` tie-breaker.
    pub column: String,
    /// Named bind carrying the committed tie-breaker, without a leading colon.
    pub bind: String,
    /// Tie-breaker used before any checkpoint exists.
    pub initial: i64,
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

/// Database-row to OTLP log mapping selected before ingestion.
///
/// This models the generic database design's output contract. The initial
/// Oracle public schema does not expose these controls, so Oracle uses the
/// default all-column body and watermark-derived event time.
#[derive(Clone, Debug, Default)]
pub struct OutputConfig {
    /// Source columns included in the body. An empty list includes every column.
    pub include_columns: Vec<String>,
    /// Source-column to typed OTLP attribute-name mappings.
    pub attributes: BTreeMap<String, String>,
    /// Optional result column used as the OTLP event timestamp.
    pub timestamp_column: Option<String>,
    /// Result columns that must exist even when not emitted specially.
    pub validation_columns: Vec<String>,
}

impl OutputConfig {
    /// Validates mappings that do not require live result metadata.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_unique_names(
            "query.output.include_columns",
            self.include_columns.iter().map(String::as_str),
        )?;
        validate_mapping_names("query.output.attributes", &self.attributes)?;

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
            return Err(ConfigError::ZeroInterval);
        }
        if !(MIN_INTERVAL..=MAX_INTERVAL).contains(&self.interval) {
            return Err(ConfigError::IntervalRange {
                maximum_seconds: MAX_INTERVAL.as_secs(),
            });
        }
        if self.timeout.is_zero() {
            return Err(ConfigError::ZeroTimeout);
        }
        if self.fetch_size == 0 {
            return Err(ConfigError::ZeroFetchSize);
        }
        if self.max_rows_per_poll == 0 {
            return Err(ConfigError::ZeroRowLimit);
        }
        if self.max_batch_bytes == 0 {
            return Err(ConfigError::ZeroBatchByteLimit);
        }
        if self.max_normalized_bytes == 0 {
            return Err(ConfigError::ZeroNormalizedByteLimit);
        }
        if self.max_batch_bytes > MAX_BYTE_LIMIT {
            return Err(ConfigError::ByteLimit {
                field: "query.max_batch_bytes",
                maximum: MAX_BYTE_LIMIT,
            });
        }
        if self.max_normalized_bytes > MAX_BYTE_LIMIT {
            return Err(ConfigError::ByteLimit {
                field: "query.max_normalized_bytes",
                maximum: MAX_BYTE_LIMIT,
            });
        }
        if self.max_rows_per_poll > MAX_ROWS_PER_POLL {
            return Err(ConfigError::RowLimit {
                maximum: MAX_ROWS_PER_POLL,
            });
        }
        if self.fetch_size > self.max_rows_per_poll {
            return Err(ConfigError::FetchSizeExceedsRowLimit);
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
            return Err(ConfigError::EmptyName {
                field: "watermark.timestamp.initial",
            });
        }
        if !timestamp.timezone.eq_ignore_ascii_case("UTC") {
            return Err(ConfigError::UnsupportedTimezone);
        }
        if timestamp.bind.eq_ignore_ascii_case(&tie_breaker.bind) {
            return Err(ConfigError::DuplicateBind);
        }
        if timestamp.column.eq_ignore_ascii_case(&tie_breaker.column) {
            return Err(ConfigError::DuplicateCursorColumn);
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
            return Err(ConfigError::CheckpointTraversal);
        }
        if self.nack_backoff.is_zero() || self.nack_backoff > MAX_NACK_BACKOFF {
            return Err(ConfigError::NackBackoffRange {
                maximum_seconds: MAX_NACK_BACKOFF.as_secs(),
            });
        }
        if self.max_consecutive_failures == 0
            || self.max_consecutive_failures > MAX_CONSECUTIVE_FAILURES
        {
            return Err(ConfigError::CheckpointFailureRange {
                maximum: MAX_CONSECUTIVE_FAILURES,
            });
        }
        Ok(())
    }
}

fn validate_mapping_names(
    field: &'static str,
    mappings: &BTreeMap<String, String>,
) -> Result<(), ConfigError> {
    validate_unique_names(field, mappings.keys().map(String::as_str))?;
    validate_unique_names(field, mappings.values().map(String::as_str))
}

fn validate_unique_names<'a>(
    field: &'static str,
    names: impl IntoIterator<Item = &'a str>,
) -> Result<(), ConfigError> {
    let mut normalized = BTreeSet::new();
    for name in names {
        validate_name(field, name)?;
        if !normalized.insert(name.to_ascii_lowercase()) {
            return Err(ConfigError::DuplicateName { field });
        }
    }
    Ok(())
}

fn validate_name(field: &'static str, name: &str) -> Result<(), ConfigError> {
    if name.trim().is_empty() {
        Err(ConfigError::EmptyName { field })
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
        return Err(ConfigError::InvalidBind { field });
    }
    Ok(())
}

/// Invalid database receiver configuration.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ConfigError {
    /// Poll interval is zero.
    #[error("query.interval must be greater than zero")]
    ZeroInterval,
    /// Poll interval is outside the supported range.
    #[error("query.interval must be between 1ms and {maximum_seconds}s")]
    IntervalRange {
        /// Longest supported interval in seconds.
        maximum_seconds: u64,
    },
    /// Query timeout is zero.
    #[error("query.timeout must be greater than zero")]
    ZeroTimeout,
    /// Driver fetch size is zero.
    #[error("query.fetch_size must be greater than zero")]
    ZeroFetchSize,
    /// Per-poll row limit is zero.
    #[error("query.max_rows_per_poll must be greater than zero")]
    ZeroRowLimit,
    /// Per-page encoded byte limit is zero.
    #[error("query.max_batch_bytes must be greater than zero")]
    ZeroBatchByteLimit,
    /// Per-poll normalized byte limit is zero.
    #[error("query.max_normalized_bytes must be greater than zero")]
    ZeroNormalizedByteLimit,
    /// A configured byte limit exceeds the supported ceiling.
    #[error("{field} must not exceed {maximum} bytes")]
    ByteLimit {
        /// Invalid configuration field.
        field: &'static str,
        /// Largest supported byte value.
        maximum: u64,
    },
    /// Per-poll row limit exceeds the fixed receiver allocation ceiling.
    #[error("query.max_rows_per_poll must not exceed {maximum}")]
    RowLimit {
        /// Largest supported row count per poll.
        maximum: usize,
    },
    /// Fetch size exceeds the poll ceiling.
    #[error("query.fetch_size must not exceed query.max_rows_per_poll")]
    FetchSizeExceedsRowLimit,
    /// A configured mapping name is empty.
    #[error("{field} names must not be empty")]
    EmptyName {
        /// Invalid configuration field.
        field: &'static str,
    },
    /// Configured names collide under OTLP's case-insensitive matching policy.
    #[error("{field} contains duplicate names")]
    DuplicateName {
        /// Invalid configuration field.
        field: &'static str,
    },
    /// A bind name is not a plain identifier without a leading colon.
    #[error("{field} must omit ':' and contain only ASCII alphanumerics or '_'")]
    InvalidBind {
        /// Invalid configuration field.
        field: &'static str,
    },
    /// Both cursor components share one bind name.
    #[error("watermark bind names must be distinct")]
    DuplicateBind,
    /// Both cursor components reference the same result column.
    #[error("watermark cursor columns must be distinct")]
    DuplicateCursorColumn,
    /// Cursor semantics outside UTC are not supported.
    #[error("watermark.timestamp.timezone must be UTC")]
    UnsupportedTimezone,
    /// The checkpoint directory escapes its configured root.
    #[error("checkpoint.directory must not contain '..' components")]
    CheckpointTraversal,
    /// The negative-acknowledgement backoff is outside the supported range.
    #[error("checkpoint.nack_backoff must be between 1ms and {maximum_seconds}s")]
    NackBackoffRange {
        /// Longest supported backoff in seconds.
        maximum_seconds: u64,
    },
    /// The consecutive checkpoint failure budget is outside the supported range.
    #[error("checkpoint.max_consecutive_failures must be between 1 and {maximum}")]
    CheckpointFailureRange {
        /// Largest supported failure budget.
        maximum: u32,
    },
}
