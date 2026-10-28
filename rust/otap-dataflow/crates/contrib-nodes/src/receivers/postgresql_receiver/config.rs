// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Strict native configuration and restart compatibility identity.

use super::{
    adapter::{Error, Result},
    query::Plan,
};
use otel_arrow_dfe_scraper::database::{
    CatchUpConfig, CheckpointConfig, CompiledQuery, OnNack, OnPermanentNack, OutputConfig,
    PollingConfig, ScalarValue, TieBreakerCursorConfig, TimestampCursorConfig, WatermarkConfig,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use std::{net::IpAddr, path::Path, time::Duration};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PostgreSqlReceiverConfig {
    pub source_id: String,
    pub connection: Connection,
    pub query: Query,
    pub watermark: Watermark,
    pub checkpoint: Checkpoint,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Connection {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database: String,
    #[serde(default)]
    tls: Tls,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Tls {
    #[serde(default = "default_tls_enabled")]
    enabled: bool,
    #[serde(default)]
    ca_file: Option<String>,
}

impl Default for Tls {
    fn default() -> Self {
        Self {
            enabled: default_tls_enabled(),
            ca_file: None,
        }
    }
}

impl Connection {
    pub(crate) const fn tls_enabled(&self) -> bool {
        self.tls.enabled
    }

    pub(crate) fn ca_file(&self) -> Option<&Path> {
        self.tls.ca_file.as_deref().map(Path::new)
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Query {
    pub statement: String,
    #[serde(default = "default_interval", with = "humantime_serde")]
    pub interval: Duration,
    #[serde(default = "default_fetch_size_rows")]
    pub fetch_size_rows: usize,
    pub max_rows_per_poll: usize,
    #[serde(deserialize_with = "deserialize_byte_size")]
    pub max_batch_bytes: u64,
    #[serde(default)]
    pub catch_up: CatchUpConfig,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Watermark {
    Snapshot {},
    Scalar {
        column: String,
        bind: String,
        initial: ScalarValue,
    },
    Composite {
        timestamp: Timestamp,
        tie_breaker: Tie,
    },
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Timestamp {
    pub column: String,
    #[serde(default = "default_timestamp_bind")]
    pub bind: String,
    pub initial: String,
    pub timezone: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Tie {
    pub column: String,
    #[serde(default = "default_tie_breaker_bind")]
    pub bind: String,
    pub initial: i64,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Checkpoint {
    pub directory: String,
    #[serde(default = "default_on_nack")]
    pub on_nack: OnNack,
    #[serde(default)]
    pub on_permanent_nack: OnPermanentNack,
    #[serde(default = "default_nack_backoff", with = "humantime_serde")]
    pub nack_backoff: Duration,
    #[serde(default = "default_max_consecutive_failures")]
    pub max_consecutive_failures: u32,
}

const fn default_port() -> u16 {
    5432
}
const fn default_tls_enabled() -> bool {
    true
}
const fn default_interval() -> Duration {
    Duration::from_secs(60)
}
const fn default_fetch_size_rows() -> usize {
    300
}
const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

fn deserialize_byte_size<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    otel_arrow_dfe_config::byte_units::deserialize_u64(deserializer)?
        .ok_or_else(|| DeError::custom("byte size must not be null"))
}

const fn default_nack_backoff() -> Duration {
    Duration::from_secs(1)
}
const fn default_max_consecutive_failures() -> u32 {
    5
}
const fn default_on_nack() -> OnNack {
    OnNack::Rewind
}
fn default_timestamp_bind() -> String {
    "last_timestamp".into()
}
fn default_tie_breaker_bind() -> String {
    "last_tie_breaker".into()
}

pub(crate) struct Validated {
    pub config: PostgreSqlReceiverConfig,
    pub common: CompiledQuery,
    pub plan: Plan,
    pub fingerprint: String,
}

pub(crate) fn validate_name(value: &str, maximum: usize) -> Result<()> {
    if value.is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
        return Err(Error::Config);
    }
    Ok(())
}

impl PostgreSqlReceiverConfig {
    pub fn parse(value: &serde_json::Value) -> Result<Self> {
        // Omission selects defaults; explicit null must not silently disable an
        // option. Check nested values too, before serde applies field defaults.
        fn has_null(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Null => true,
                serde_json::Value::Array(values) => values.iter().any(has_null),
                serde_json::Value::Object(values) => values.values().any(has_null),
                _ => false,
            }
        }
        if has_null(value) {
            return Err(Error::Config);
        }
        serde_json::from_value(value.clone()).map_err(|_| Error::Config)
    }

    pub fn validate(mut self) -> Result<Validated> {
        validate_name(&self.source_id, 128)?;
        validate_name(&self.connection.database, 63)?;
        let connection = &mut self.connection;
        validate_name(&connection.host, 253)?;
        if connection.port == 0 {
            return Err(Error::Config);
        }
        if connection.host.parse::<IpAddr>().is_err() {
            if !connection.host.is_ascii()
                || connection.host.split('.').any(|label| {
                    label.is_empty()
                        || label.len() > 63
                        || label.starts_with('-')
                        || label.ends_with('-')
                        || !label
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                })
            {
                return Err(Error::Config);
            }
            connection.host.make_ascii_lowercase();
        }
        if connection
            .tls
            .ca_file
            .as_ref()
            .is_some_and(|path| path.trim().is_empty() || path.contains('\0'))
        {
            return Err(Error::Config);
        }
        if !connection.tls.enabled && connection.tls.ca_file.is_some() {
            return Err(Error::Config);
        }
        let query = &self.query;
        if !(60..=86400).contains(&query.interval.as_secs()) || query.interval.subsec_nanos() != 0 {
            return Err(Error::Config);
        }
        let watermark = match &mut self.watermark {
            Watermark::Snapshot {} => WatermarkConfig::Snapshot {},
            Watermark::Scalar {
                column,
                bind,
                initial,
            } => {
                validate_name(column, 63)?;
                match initial {
                    ScalarValue::Int64(_) => {}
                    ScalarValue::Timestamp(text) => {
                        *text =
                            super::value::normalize_cursor_time(text).map_err(|_| Error::Config)?;
                    }
                    ScalarValue::UInt64(_) | ScalarValue::String(_) => {
                        return Err(Error::Config);
                    }
                }
                WatermarkConfig::Scalar {
                    column: column.clone(),
                    bind: bind.clone(),
                    initial: initial.clone(),
                }
            }
            Watermark::Composite {
                timestamp,
                tie_breaker,
            } => {
                validate_name(&timestamp.column, 63)?;
                validate_name(&tie_breaker.column, 63)?;
                if !timestamp.timezone.eq_ignore_ascii_case("UTC") {
                    return Err(Error::Config);
                }
                timestamp.timezone = "UTC".into();
                timestamp.initial = super::value::normalize_cursor_time(&timestamp.initial)
                    .map_err(|_| Error::Config)?;
                WatermarkConfig::Composite {
                    timestamp: TimestampCursorConfig {
                        column: timestamp.column.clone(),
                        bind: timestamp.bind.clone(),
                        initial: timestamp.initial.clone(),
                        timezone: timestamp.timezone.clone(),
                    },
                    tie_breaker: TieBreakerCursorConfig {
                        column: tie_breaker.column.clone(),
                        bind: tie_breaker.bind.clone(),
                        initial: tie_breaker.initial,
                    },
                }
            }
        };
        let checkpoint = CheckpointConfig {
            directory: self.checkpoint.directory.clone(),
            on_nack: self.checkpoint.on_nack,
            on_permanent_nack: self.checkpoint.on_permanent_nack,
            nack_backoff: self.checkpoint.nack_backoff,
            max_consecutive_failures: self.checkpoint.max_consecutive_failures,
        };
        // Shared compilation validates the configured page/fetch/byte bounds
        // and catch-up settings, using the same contract as the Oracle receiver.
        let common = CompiledQuery::compile(
            query.statement.clone(),
            PollingConfig {
                interval: query.interval,
                timeout: OPERATION_TIMEOUT,
                max_rows_per_poll: query.max_rows_per_poll,
                fetch_size_rows: query.fetch_size_rows,
                max_batch_bytes: query.max_batch_bytes,
                catch_up: query.catch_up,
            },
            &watermark,
            &checkpoint,
            OutputConfig {
                timestamp_column: match &watermark {
                    WatermarkConfig::Composite { timestamp, .. } => Some(timestamp.column.clone()),
                    WatermarkConfig::Scalar {
                        column,
                        initial: ScalarValue::Timestamp(_),
                        ..
                    } => Some(column.clone()),
                    WatermarkConfig::Snapshot {}
                    | WatermarkConfig::Scalar {
                        initial: ScalarValue::Int64(_),
                        ..
                    }
                    | WatermarkConfig::Scalar {
                        initial: ScalarValue::UInt64(_) | ScalarValue::String(_),
                        ..
                    } => None,
                },
                validation_columns: vec![],
            },
        )
        .map_err(|_| Error::Config)?;
        let plan = Plan::compile(&common)?;
        // Hash the authored query and value contract, not the optimized SQL.
        // Scheduling, resource budgets, catch-up, and checkpoint policy are excluded
        // so operational tuning does not invalidate a durable cursor.
        // Discovery never hashes transient catalog state.
        let identity = serde_json::to_vec(&(
            "postgresql/v5-watermark-time",
            "postgresql",
            &self.connection.host,
            self.connection.port,
            &self.connection.database,
            &query.statement,
            "named-parameters/v2",
            &self.watermark,
            "UTC/microseconds/native-v2",
        ))
        .map_err(|_| Error::Config)?;
        let fingerprint = blake3::hash(&identity).to_hex().to_string();
        Ok(Validated {
            config: self,
            common,
            plan,
            fingerprint,
        })
    }
}
