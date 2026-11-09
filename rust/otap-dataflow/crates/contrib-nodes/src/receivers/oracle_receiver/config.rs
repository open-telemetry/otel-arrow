// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle configuration using the shared native source-query contract.

use super::adapter::{
    OracleAdapter, OracleAdapterConfig, normalize_cursor_timestamp, validate_scalar_value,
};
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider;
use otel_arrow_dfe_scraper::database::{
    CatchUpConfig, CheckpointConfig, CompiledQuery, DatabaseSystem, OutputConfig, PollingConfig,
    ScalarValue, SourceQuery, WatermarkConfig,
};
use secrecy::zeroize::Zeroizing;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;

const MAX_SOURCE_ID_BYTES: usize = 256;
const MIN_ORACLE_INTERVAL: Duration = Duration::from_secs(60);
const MAX_ORACLE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MIN_ORACLE_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_ORACLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

const fn default_interval() -> Duration {
    Duration::from_secs(60)
}
const fn default_timeout() -> Duration {
    Duration::from_secs(30)
}
const fn default_fetch_size_rows() -> usize {
    300
}

/// Validated configuration for one Oracle snapshot or keyset query.
pub struct OracleReceiverConfig {
    source_id: String,
    connection: OracleConnectionConfig,
    query: CompiledQuery,
    checkpoint: CheckpointConfig,
    config_fingerprint: String,
}

impl<'de> Deserialize<'de> for OracleReceiverConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawOracleConfig::deserialize(deserializer).map_err(|_| {
            D::Error::custom("invalid Oracle receiver configuration: check required fields, field names, types, and enum values")
        })?;
        Self::try_from(raw).map_err(D::Error::custom)
    }
}

impl OracleReceiverConfig {
    /// Returns the stable source identifier attached to every emitted row.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Returns the durable checkpoint policy.
    #[must_use]
    pub const fn checkpoint(&self) -> &CheckpointConfig {
        &self.checkpoint
    }

    /// Returns the versioned semantic identity, excluding credentials and client paths.
    #[must_use]
    pub fn config_fingerprint(&self) -> &str {
        &self.config_fingerprint
    }

    /// Returns the shared query plan.
    #[must_use]
    pub fn query(&self) -> CompiledQuery {
        self.query.clone()
    }

    /// Builds the Oracle adapter for this configuration.
    #[must_use]
    pub fn adapter(&self, credentials: Box<dyn BasicAuthProvider>) -> OracleAdapter {
        OracleAdapter::new(
            OracleAdapterConfig {
                connect_string: self.connection.connect_string.clone(),
                instant_client_dir: self.connection.instant_client_dir.clone(),
            },
            credentials,
        )
    }
}

impl TryFrom<RawOracleConfig> for OracleReceiverConfig {
    type Error = OracleConfigError;

    fn try_from(mut config: RawOracleConfig) -> Result<Self, Self::Error> {
        required("source_id", &config.source_id)?;
        if config.source_id.len() > MAX_SOURCE_ID_BYTES {
            return Err(OracleConfigError::new(format!(
                "source_id must not exceed {MAX_SOURCE_ID_BYTES} bytes"
            )));
        }
        required(
            "connection.connect_string",
            &config.connection.connect_string,
        )?;
        required(
            "connection.instant_client_dir",
            &config.connection.instant_client_dir,
        )?;
        required("query.statement", &config.query.statement)?;
        if !(MIN_ORACLE_INTERVAL..=MAX_ORACLE_INTERVAL).contains(&config.query.interval)
            || config.query.interval.subsec_nanos() != 0
        {
            return Err(OracleConfigError::new(
                "query.interval must be between 1 minute and 24 hours in whole seconds",
            ));
        }
        if !(MIN_ORACLE_TIMEOUT..=MAX_ORACLE_TIMEOUT).contains(&config.query.timeout)
            || config.query.timeout.subsec_nanos() != 0
        {
            return Err(OracleConfigError::new(
                "query.timeout must be between 1s and 5m in whole seconds",
            ));
        }
        config.query.polling().validate()?;
        config.watermark.validate()?;
        config.checkpoint.validate()?;
        let output = match &mut config.watermark {
            WatermarkConfig::Snapshot {} => OutputConfig::default(),
            WatermarkConfig::Composite {
                timestamp,
                tie_breaker,
            } => {
                validate_result_name("watermark.timestamp.column", &timestamp.column)?;
                validate_result_name("watermark.tie_breaker.column", &tie_breaker.column)?;
                timestamp.initial = normalize_cursor_timestamp(&timestamp.initial)
                    .map_err(|_| OracleConfigError::new("watermark.timestamp.initial must be a valid Oracle timestamp with at most nine digits per numeric component"))?;
                OutputConfig {
                    timestamp_column: Some(timestamp.column.clone()),
                    validation_columns: vec![tie_breaker.column.clone()],
                }
            }
            WatermarkConfig::Scalar {
                column, initial, ..
            } => {
                validate_result_name("watermark.column", column)?;
                validate_scalar_value(initial).map_err(|_| OracleConfigError::new(
                    "invalid Oracle scalar initial value: strings must be nonempty without NUL; timestamps must have supported precision; text is bounded to 1024 bytes",
                ))?;
                let timestamp_column = if let ScalarValue::Timestamp(text) = initial {
                    *text = normalize_cursor_timestamp(text)
                        .map_err(|_| OracleConfigError::new("invalid Oracle scalar timestamp"))?;
                    Some(column.clone())
                } else {
                    None
                };
                OutputConfig {
                    timestamp_column,
                    validation_columns: vec![column.clone()],
                }
            }
        };
        let query = CompiledQuery::compile(
            config.query.statement.clone(),
            config.query.polling(),
            &config.watermark,
            &config.checkpoint,
            output,
        )?;
        let source = SourceQuery::compile(&query, DatabaseSystem::Oracle)?;
        let fingerprint = match &config.watermark {
            WatermarkConfig::Snapshot {} => serde_json::to_vec(&SnapshotFingerprintInput {
                source_id: &config.source_id,
                connect_string: &config.connection.connect_string,
                statement: source.fingerprint_sql(),
                mode: "snapshot",
            }),
            WatermarkConfig::Composite {
                timestamp,
                tie_breaker,
            } => serde_json::to_vec(&FingerprintInput {
                source_id: &config.source_id,
                connect_string: &config.connection.connect_string,
                statement: source.fingerprint_sql(),
                timestamp_column: &timestamp.column,
                timestamp_bind: &timestamp.bind,
                timestamp_initial: &timestamp.initial,
                tie_breaker_column: &tie_breaker.column,
                tie_breaker_bind: &tie_breaker.bind,
                tie_breaker_initial: tie_breaker.initial,
            }),
            WatermarkConfig::Scalar {
                column,
                bind,
                initial,
            } => serde_json::to_vec(&ScalarFingerprintInput {
                source_id: &config.source_id,
                connect_string: &config.connection.connect_string,
                statement: source.fingerprint_sql(),
                mode: "scalar",
                column,
                bind,
                initial,
            }),
        };
        let fingerprint_bytes =
            Zeroizing::new(fingerprint.map_err(|_| {
                OracleConfigError::new("failed to fingerprint Oracle configuration")
            })?);
        let config_fingerprint = blake3::hash(&fingerprint_bytes).to_hex().to_string();
        Ok(Self {
            source_id: config.source_id,
            connection: config.connection,
            query,
            checkpoint: config.checkpoint,
            config_fingerprint,
        })
    }
}

#[derive(Serialize)]
struct SnapshotFingerprintInput<'a> {
    source_id: &'a str,
    connect_string: &'a str,
    statement: &'a str,
    mode: &'static str,
}

#[derive(Serialize)]
struct ScalarFingerprintInput<'a> {
    source_id: &'a str,
    connect_string: &'a str,
    statement: &'a str,
    mode: &'static str,
    column: &'a str,
    bind: &'a str,
    initial: &'a ScalarValue,
}

#[derive(Serialize)]
struct FingerprintInput<'a> {
    source_id: &'a str,
    connect_string: &'a str,
    statement: &'a str,
    timestamp_column: &'a str,
    timestamp_bind: &'a str,
    timestamp_initial: &'a str,
    tie_breaker_column: &'a str,
    tie_breaker_bind: &'a str,
    tie_breaker_initial: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOracleConfig {
    source_id: String,
    connection: OracleConnectionConfig,
    query: OracleQueryConfig,
    watermark: WatermarkConfig,
    checkpoint: CheckpointConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleConnectionConfig {
    connect_string: String,
    instant_client_dir: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleQueryConfig {
    statement: String,
    #[serde(default = "default_interval", with = "humantime_serde")]
    interval: Duration,
    #[serde(default = "default_fetch_size_rows")]
    fetch_size_rows: usize,
    max_rows_per_poll: usize,
    #[serde(deserialize_with = "deserialize_byte_size")]
    max_batch_bytes: u64,
    #[serde(default = "default_timeout", with = "humantime_serde")]
    timeout: Duration,
    #[serde(default)]
    catch_up: CatchUpConfig,
}

impl OracleQueryConfig {
    fn polling(&self) -> PollingConfig {
        PollingConfig {
            interval: self.interval,
            timeout: self.timeout,
            fetch_size_rows: self.fetch_size_rows,
            max_rows_per_poll: self.max_rows_per_poll,
            max_batch_bytes: self.max_batch_bytes,
            catch_up: self.catch_up,
        }
    }
}

fn deserialize_byte_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    otel_arrow_dfe_config::byte_units::deserialize_u64(deserializer)?
        .ok_or_else(|| DeError::custom("byte size must not be null"))
}

fn validate_result_name(field: &'static str, name: &str) -> Result<(), OracleConfigError> {
    if name.trim().is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
        return Err(OracleConfigError::new(format!(
            "{field} must name a result column using 1..128 bytes without control characters"
        )));
    }
    Ok(())
}

fn required(field: &'static str, value: &str) -> Result<(), OracleConfigError> {
    if value.trim().is_empty() {
        Err(OracleConfigError::new(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

/// Invalid Oracle receiver configuration.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct OracleConfigError {
    message: String,
}

impl OracleConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<otel_arrow_dfe_scraper::database::ConfigError> for OracleConfigError {
    fn from(error: otel_arrow_dfe_scraper::database::ConfigError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<otel_arrow_dfe_scraper::database::QueryError> for OracleConfigError {
    fn from(error: otel_arrow_dfe_scraper::database::QueryError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<otel_arrow_dfe_scraper::database::SourceQueryError> for OracleConfigError {
    fn from(error: otel_arrow_dfe_scraper::database::SourceQueryError) -> Self {
        Self::new(error.to_string())
    }
}
