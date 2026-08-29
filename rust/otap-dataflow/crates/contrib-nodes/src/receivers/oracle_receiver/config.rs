// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle receiver configuration.

use super::adapter::{OracleAdapter, OracleAdapterConfig};
use crate::receivers::database::{
    CompiledQuery, DatabaseReceiver, OutputConfig, PollingConfig, QueryError,
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
        let query = CompiledQuery::compile(
            self.query.statement,
            PollingConfig {
                interval: self.query.interval,
                timeout: self.query.timeout,
                fetch_size: self.query.fetch_size,
                max_rows_per_poll: self.query.max_rows_per_poll,
            },
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
    interval: std::time::Duration,
    fetch_size: usize,
    max_rows_per_poll: usize,
    #[serde(with = "humantime_serde")]
    timeout: std::time::Duration,
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

    /// Scenario: Configuration adds a query field that is not defined by the Oracle design.
    /// Guarantees: The public schema stays closed and rejects invented output or policy controls.
    #[test]
    fn rejects_undocumented_query_fields() {
        for (field, value) in [
            ("name", serde_json::json!("audit-query")),
            ("error_policy", serde_json::json!("fail_batch")),
            (
                "output",
                serde_json::json!({"include_columns": ["AUDIT_ID"]}),
            ),
        ] {
            let mut config = config();
            config["query"][field] = value;
            assert!(
                serde_json::from_value::<OracleReceiverConfig>(config).is_err(),
                "undocumented query field '{field}' must be rejected"
            );
        }
    }
}
