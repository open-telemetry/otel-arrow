// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal Oracle OCI polling receiver.

use super::scraper::ScraperReceiver;
use linkme::distributed_slice;
use oracle_scraper::OracleScraper;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::validation::validate_typed_config;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

mod oracle_scraper;

otap_df_telemetry::otel_component_scope!(
    urn = ORACLE_RECEIVER_URN,
    target = "otel.receiver.oracle",
);

/// URN for the Oracle OCI receiver.
pub const ORACLE_RECEIVER_URN: &str = "urn:otel:receiver:oracle";

const DEFAULT_PASSWORD_ENV: &str = "ORACLE_PWD";
const DEFAULT_MAX_ROWS: usize = 100;
const MAX_ROWS_LIMIT: usize = 1_000;
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(30);
const DEFAULT_CALL_TIMEOUT: Duration = Duration::from_secs(10);

type OracleReceiver = ScraperReceiver<OracleScraper>;

fn default_password_env() -> String {
    DEFAULT_PASSWORD_ENV.to_owned()
}

const fn default_max_rows() -> usize {
    DEFAULT_MAX_ROWS
}

const fn default_poll_interval() -> Duration {
    DEFAULT_POLL_INTERVAL
}

const fn default_call_timeout() -> Duration {
    DEFAULT_CALL_TIMEOUT
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "OracleReceiverConfigBuilder")]
struct OracleReceiverConfig {
    collection_interval: Duration,
    scraper: OracleScraperConfig,
}

#[derive(Clone, Debug)]
pub(super) struct OracleScraperConfig {
    connect_string: String,
    username: String,
    password_env: String,
    query: String,
    call_timeout: Duration,
    max_rows: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleReceiverConfigBuilder {
    connect_string: String,
    username: String,
    #[serde(default = "default_password_env")]
    password_env: String,
    query: String,
    #[serde(default = "default_poll_interval", with = "humantime_serde")]
    poll_interval: Duration,
    #[serde(default = "default_call_timeout", with = "humantime_serde")]
    call_timeout: Duration,
    #[serde(default = "default_max_rows")]
    max_rows: usize,
}

impl TryFrom<OracleReceiverConfigBuilder> for OracleReceiverConfig {
    type Error = String;

    fn try_from(value: OracleReceiverConfigBuilder) -> Result<Self, Self::Error> {
        let connect_string = required_text("connect_string", value.connect_string)?;
        let username = required_text("username", value.username)?;
        let password_env = required_text("password_env", value.password_env)?;
        let query = required_text("query", value.query)?;
        if !is_read_only_query(&query) {
            return Err("query must start with SELECT or WITH".to_owned());
        }

        if value.poll_interval.is_zero() {
            return Err("poll_interval must be greater than zero".to_owned());
        }
        if value.call_timeout.is_zero() {
            return Err("call_timeout must be greater than zero".to_owned());
        }
        if !(1..=MAX_ROWS_LIMIT).contains(&value.max_rows) {
            return Err(format!("max_rows must be between 1 and {MAX_ROWS_LIMIT}"));
        }

        Ok(Self {
            collection_interval: value.poll_interval,
            scraper: OracleScraperConfig {
                connect_string,
                username,
                password_env,
                query,
                call_timeout: value.call_timeout,
                max_rows: value.max_rows,
            },
        })
    }
}

impl OracleReceiverConfig {
    fn create_receiver(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<OracleReceiver, ConfigError> {
        if pipeline.num_cores() != 1 {
            return Err(ConfigError::InvalidUserConfig {
                error: "the minimal Oracle receiver requires a single-core pipeline".to_owned(),
            });
        }

        let config: Self = serde_json::from_value(config.clone()).map_err(|error| {
            ConfigError::InvalidUserConfig {
                error: error.to_string(),
            }
        })?;
        Ok(ScraperReceiver::new(
            OracleScraper::new(config.scraper),
            config.collection_interval,
        ))
    }
}

fn required_text(name: &str, value: String) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        Err(format!("{name} must not be empty"))
    } else {
        Ok(value.to_owned())
    }
}

fn is_read_only_query(query: &str) -> bool {
    query.split_whitespace().next().is_some_and(|keyword| {
        keyword.eq_ignore_ascii_case("SELECT") || keyword.eq_ignore_ascii_case("WITH")
    })
}

/// Declares the Oracle OCI receiver as a local receiver factory.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static ORACLE_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: ORACLE_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ReceiverWrapper::local(
            OracleReceiverConfig::create_receiver(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
    validate_config: validate_typed_config::<OracleReceiverConfig>,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{receiver::TestRuntime, test_node, test_pipeline_ctx};
    use std::time::Instant;

    fn test_config() -> OracleReceiverConfig {
        serde_json::from_value(serde_json::json!({
            "connect_string": "//localhost:1521/FREEPDB1",
            "username": "PDBADMIN",
            "query": "SELECT SYSDATE AS CURRENT_TIME FROM DUAL"
        }))
        .expect("test config should deserialize")
    }

    /// Scenario: a minimal Oracle receiver configuration omits optional polling settings.
    /// Guarantees: safe bounded defaults are applied for credentials, timing, and row count.
    #[test]
    fn config_applies_bounded_defaults() {
        let config = test_config();

        assert_eq!(config.scraper.password_env, "ORACLE_PWD");
        assert_eq!(config.collection_interval, Duration::from_secs(30));
        assert_eq!(config.scraper.call_timeout, Duration::from_secs(10));
        assert_eq!(config.scraper.max_rows, 100);
    }

    /// Scenario: an Oracle receiver configuration requests an unbounded or empty row batch.
    /// Guarantees: row limits outside the supported bounded range fail configuration parsing.
    #[test]
    fn config_rejects_invalid_row_limits() {
        for max_rows in [0, MAX_ROWS_LIMIT + 1] {
            let result = serde_json::from_value::<OracleReceiverConfig>(serde_json::json!({
                "connect_string": "//localhost:1521/FREEPDB1",
                "username": "PDBADMIN",
                "query": "SELECT 1 FROM DUAL",
                "max_rows": max_rows
            }));
            assert!(result.is_err());
        }
    }

    /// Scenario: an Oracle receiver query attempts to execute a non-query SQL statement.
    /// Guarantees: configuration accepts SELECT/WITH polling and rejects an accidental DELETE.
    #[test]
    fn config_rejects_non_query_sql() {
        let result = serde_json::from_value::<OracleReceiverConfig>(serde_json::json!({
            "connect_string": "//localhost:1521/FREEPDB1",
            "username": "PDBADMIN",
            "query": "DELETE FROM telemetry_events"
        }));

        assert!(result.is_err());
        assert!(is_read_only_query("SELECT 1 FROM DUAL"));
        assert!(is_read_only_query(
            "with rows as (select 1 value from dual) select * from rows"
        ));
    }

    /// Scenario: local Oracle credentials opt in to a live receiver test through environment variables.
    /// Guarantees: the receiver polls through OCI and sends the returned row into the OTAP pipeline.
    #[test]
    fn oracle_receiver_emits_rows_when_configured() {
        if std::env::var_os("OTAP_ORACLE_RECEIVER_E2E").is_none() {
            return;
        }

        let config = serde_json::json!({
            "connect_string": std::env::var("ORACLE_CONNECT_STRING")
                .unwrap_or_else(|_| "//localhost:1521/FREEPDB1".to_owned()),
            "username": std::env::var("ORACLE_USERNAME")
                .unwrap_or_else(|_| "PDBADMIN".to_owned()),
            "query": "SELECT SYSDATE AS CURRENT_TIME FROM DUAL",
            "poll_interval": "100ms",
            "call_timeout": "10s",
            "max_rows": 10
        });
        let (pipeline_ctx, _registry) = test_pipeline_ctx();
        let receiver =
            OracleReceiverConfig::create_receiver(pipeline_ctx, &config).expect("receiver config");
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(ORACLE_RECEIVER_URN));
        let receiver_wrapper = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver_wrapper)
            .run_test(|ctx| async move {
                ctx.sleep(Duration::from_millis(250)).await;
                ctx.send_shutdown(Instant::now(), "Oracle receiver E2E complete")
                    .await
                    .expect("shutdown should enqueue");
            })
            .run_validation(|mut ctx| async move {
                let pdata = ctx.recv().await.expect("receiver should emit pdata");
                assert_eq!(pdata.num_items(), 1);
            });
    }
}
