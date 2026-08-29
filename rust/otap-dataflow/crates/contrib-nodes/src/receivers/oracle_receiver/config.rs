// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle receiver configuration.

use super::adapter::{OracleAdapter, OracleAdapterConfig};
use crate::receivers::database::{
    CompiledQuery, DatabaseReceiver, ErrorPolicy, OutputConfig, PollingConfig, QueryError,
};
use serde::Deserialize;

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
    query_name: String,
}

impl OracleReceiverConfig {
    /// Returns the stable source identifier attached by later OTLP mapping.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Builds the shared receiver and Oracle adapter.
    pub fn build(self) -> Result<DatabaseReceiver<OracleAdapter>, QueryError> {
        let query = CompiledQuery::compile(
            self.query.statement,
            PollingConfig {
                interval: self.query.interval,
                timeout: self.query.timeout,
                fetch_size: self.query.fetch_size,
                max_rows_per_poll: self.query.max_rows_per_poll,
            },
            self.query.output,
            self.query.error_policy,
        )?;
        let adapter = OracleAdapter::new(OracleAdapterConfig {
            connect_string: self.connection.connect_string,
            instant_client_dir: self.connection.instant_client_dir,
            username_file: self.authentication.username_file,
            password_file: self.authentication.password_file,
        });
        Ok(DatabaseReceiver::new(
            adapter,
            query,
            self.source_id,
            self.query_name,
        ))
    }
}

impl TryFrom<RawOracleConfig> for OracleReceiverConfig {
    type Error = OracleConfigError;

    fn try_from(mut config: RawOracleConfig) -> Result<Self, Self::Error> {
        required("source_id", &config.source_id)?;
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
        config.query.polling().validate()?;
        let query_name = config
            .query
            .name
            .clone()
            .unwrap_or_else(|| config.source_id.clone());
        required("query.name", &query_name)?;

        if let Some(watermark) = config.watermark {
            required("watermark.timestamp_column", &watermark.timestamp_column)?;
            required(
                "watermark.tie_breaker_column",
                &watermark.tie_breaker_column,
            )?;
            if watermark.timezone != "UTC" {
                return Err(OracleConfigError::new("watermark.timezone must be UTC"));
            }
            _ = watermark.start_at;
            if let Some(configured) = &config.query.output.timestamp_column
                && !configured.eq_ignore_ascii_case(&watermark.timestamp_column)
            {
                return Err(OracleConfigError::new(
                    "query.output.timestamp_column must match watermark.timestamp_column",
                ));
            }
            // Watermark progression is intentionally absent, but these columns
            // still define event time and must be checked against live metadata.
            config.query.output.timestamp_column = Some(watermark.timestamp_column);
            config
                .query
                .output
                .validation_columns
                .push(watermark.tie_breaker_column);
        }
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
            query_name,
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
    /// Stable query identity emitted on every log record.
    #[serde(default)]
    name: Option<String>,
    statement: String,
    #[serde(with = "humantime_serde")]
    interval: std::time::Duration,
    fetch_size: usize,
    max_rows_per_poll: usize,
    #[serde(with = "humantime_serde")]
    timeout: std::time::Duration,
    /// Database-neutral OTLP body, attribute, and resource mapping.
    #[serde(default)]
    output: OutputConfig,
    /// Permanent conversion failure scope; batch failure is the safe default.
    #[serde(default)]
    error_policy: ErrorPolicy,
}

impl OracleQueryConfig {
    fn polling(&self) -> PollingConfig {
        PollingConfig {
            interval: self.interval,
            timeout: self.timeout,
            fetch_size: self.fetch_size,
            max_rows_per_poll: self.max_rows_per_poll,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::OracleReceiverConfig;

    fn config() -> serde_json::Value {
        serde_json::json!({
            "source_id": "oracle-audit",
            "connection": {
                "connect_string": "database.contoso.com:1521/ORCL",
                "instant_client_dir": "/opt/oracle/instantclient"
            },
            "authentication": {
                "username_file": "/var/run/secrets/oracle/oracle-audit/username",
                "password_file": "/var/run/secrets/oracle/oracle-audit/password"
            },
            "query": {
                "statement": "SELECT AUDIT_ID, LAST_UPDATED FROM AUDIT_LOGS",
                "interval": "5m",
                "fetch_size": 1000,
                "max_rows_per_poll": 10000,
                "timeout": "2m"
            },
            "watermark": {
                "timestamp_column": "LAST_UPDATED",
                "tie_breaker_column": "AUDIT_ID",
                "timezone": "UTC",
                "start_at": "beginning"
            },
            "checkpoint": {
                "directory": "${engine.state_dir}/oracle",
                "on_nack": "rewind",
                "max_consecutive_failures": 5
            }
        })
    }

    /// Scenario: The documented Oracle configuration is loaded before runtime wiring.
    /// Guarantees: The complete stable shape validates and builds a shared receiver.
    #[test]
    fn accepts_documented_configuration() {
        let config: OracleReceiverConfig =
            serde_json::from_value(config()).expect("configuration should deserialize");
        assert_eq!(config.source_id(), "oracle-audit");
        _ = config.build().expect("configuration should build");
    }

    /// Scenario: A snapshot-only receiver omits inactive watermark and checkpoint sections.
    /// Guarantees: The foundation remains usable without implying stateful progress behavior.
    #[test]
    fn accepts_snapshot_config_without_state_sections() {
        let mut value = config();
        _ = value
            .as_object_mut()
            .expect("config object")
            .remove("watermark");
        _ = value
            .as_object_mut()
            .expect("config object")
            .remove("checkpoint");

        let config: OracleReceiverConfig =
            serde_json::from_value(value).expect("snapshot config should deserialize");
        _ = config.build().expect("snapshot config should build");
    }

    /// Scenario: The configured Oracle query could modify database state.
    /// Guarantees: Receiver construction rejects it before credentials or a connection are used.
    #[test]
    fn rejects_modifying_query_before_connect() {
        let mut value = config();
        value["query"]["statement"] = serde_json::json!("DELETE FROM AUDIT_LOGS");
        let config: OracleReceiverConfig =
            serde_json::from_value(value).expect("shape should deserialize");

        assert!(config.build().is_err());
    }
}
