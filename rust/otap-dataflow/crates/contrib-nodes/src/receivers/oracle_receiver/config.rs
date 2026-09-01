// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle receiver configuration.

use super::adapter::{OracleAdapter, OracleAdapterConfig};
use crate::receivers::database::{
    CompiledQuery, DatabaseReceiver, OutputConfig, PollingConfig, QueryError,
};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

const MAX_SOURCE_ID_BYTES: usize = 256;
const MIN_ORACLE_TIMEOUT: Duration = Duration::from_millis(1);
const MAX_ORACLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Validated configuration for one Oracle snapshot query.
///
/// Watermark and checkpoint sections are validated for forward-compatible
/// configuration, but this initial polling slice does not apply their state.
#[derive(Deserialize)]
#[serde(try_from = "RawOracleConfig")]
pub struct OracleReceiverConfig {
    source_id: String,
    connection: OracleConnectionConfig,
    authentication: OracleAuthenticationConfig,
    query: OracleQueryConfig,
    event_timestamp_column: Option<String>,
    validation_columns: Vec<String>,
}

impl OracleReceiverConfig {
    /// Returns the stable source identifier attached by later OTLP mapping.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Builds the shared receiver and Oracle adapter.
    pub fn build(self) -> Result<DatabaseReceiver<OracleAdapter>, QueryError> {
        let polling = self.query.polling();
        let query = CompiledQuery::compile(
            self.query.statement,
            polling,
            OutputConfig {
                timestamp_column: self.event_timestamp_column,
                validation_columns: self.validation_columns,
                ..OutputConfig::default()
            },
        )?;
        let adapter = OracleAdapter::new(OracleAdapterConfig {
            connect_string: self.connection.connect_string,
            instant_client_dir: self.connection.instant_client_dir,
            username_file: self.authentication.username_file,
            password_file: self.authentication.password_file,
        });
        Ok(DatabaseReceiver::new(adapter, query, self.source_id))
    }
}

impl TryFrom<RawOracleConfig> for OracleReceiverConfig {
    type Error = OracleConfigError;

    fn try_from(config: RawOracleConfig) -> Result<Self, Self::Error> {
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
        required(
            "authentication.username_file",
            &config.authentication.username_file,
        )?;
        required(
            "authentication.password_file",
            &config.authentication.password_file,
        )?;
        required("query.statement", &config.query.statement)?;
        if !(MIN_ORACLE_TIMEOUT..=MAX_ORACLE_TIMEOUT).contains(&config.query.timeout) {
            return Err(OracleConfigError::new(
                "query.timeout must be between 1ms and 5m",
            ));
        }
        config.query.polling().validate()?;
        let (event_timestamp_column, validation_columns) = if let Some(watermark) = config.watermark
        {
            required("watermark.timestamp_column", &watermark.timestamp_column)?;
            required(
                "watermark.tie_breaker_column",
                &watermark.tie_breaker_column,
            )?;
            if watermark.timezone != "UTC" {
                return Err(OracleConfigError::new("watermark.timezone must be UTC"));
            }
            _ = watermark.start_at;
            // Watermark progression is intentionally absent, but these columns
            // still define event time and must be checked against live metadata.
            (
                Some(watermark.timestamp_column),
                vec![watermark.tie_breaker_column],
            )
        } else {
            (None, Vec::new())
        };
        if let Some(checkpoint) = config.checkpoint {
            required("checkpoint.directory", &checkpoint.directory)?;
            if checkpoint.max_consecutive_failures == 0 {
                return Err(OracleConfigError::new(
                    "checkpoint.max_consecutive_failures must be greater than zero",
                ));
            }
            _ = checkpoint.on_nack;
        }

        Ok(Self {
            source_id: config.source_id,
            connection: config.connection,
            authentication: config.authentication,
            query: config.query,
            event_timestamp_column,
            validation_columns,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOracleConfig {
    source_id: String,
    connection: OracleConnectionConfig,
    authentication: OracleAuthenticationConfig,
    query: OracleQueryConfig,
    #[serde(default)]
    watermark: Option<OracleWatermarkConfig>,
    #[serde(default)]
    checkpoint: Option<OracleCheckpointConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleConnectionConfig {
    connect_string: String,
    instant_client_dir: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleAuthenticationConfig {
    username_file: String,
    password_file: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleQueryConfig {
    statement: String,
    #[serde(with = "humantime_serde")]
    interval: Duration,
    fetch_size: usize,
    max_rows_per_poll: usize,
    #[serde(deserialize_with = "deserialize_byte_size")]
    max_batch_bytes: u64,
    #[serde(with = "humantime_serde")]
    timeout: Duration,
}

impl OracleQueryConfig {
    fn polling(&self) -> PollingConfig {
        PollingConfig {
            interval: self.interval,
            timeout: self.timeout,
            fetch_size: self.fetch_size,
            max_rows_per_poll: self.max_rows_per_poll,
            max_batch_bytes: self.max_batch_bytes,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleWatermarkConfig {
    timestamp_column: String,
    tie_breaker_column: String,
    timezone: String,
    start_at: WatermarkStart,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum WatermarkStart {
    Beginning,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleCheckpointConfig {
    directory: String,
    on_nack: OnNack,
    max_consecutive_failures: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum OnNack {
    Rewind,
    Fail,
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

impl From<crate::receivers::database::ConfigError> for OracleConfigError {
    fn from(error: crate::receivers::database::ConfigError) -> Self {
        Self::new(error.to_string())
    }
}
