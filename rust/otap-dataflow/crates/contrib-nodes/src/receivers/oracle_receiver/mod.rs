// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle implementation of the shared database polling contracts.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = ORACLE_RECEIVER_URN,
    target = "otel.receiver.oracle",
);

mod adapter;
mod config;

use crate::receivers::database::DatabaseReceiver;
use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::ReceiverFactory;
use otel_arrow_dfe_engine::config::ReceiverConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use serde_json::Value;
use std::sync::Arc;

pub use adapter::{OracleAdapter, OracleAdapterError};
pub use config::{OracleConfigError, OracleReceiverConfig};

/// Stable component identifier for the Oracle receiver.
pub const ORACLE_RECEIVER_URN: &str = "urn:otel:receiver:oracle";

type Receiver = DatabaseReceiver<OracleAdapter>;

fn build(config: &Value) -> Result<Receiver, ConfigError> {
    let config: OracleReceiverConfig =
        serde_json::from_value(config.clone()).map_err(invalid_config)?;
    config.build().map_err(invalid_config)
}

fn validate(config: &Value) -> Result<(), ConfigError> {
    _ = build(config)?;
    Ok(())
}

fn invalid_config(error: impl std::fmt::Display) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.to_string(),
    }
}

/// Registers the Oracle receiver as a local OTAP component.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static ORACLE_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: ORACLE_RECEIVER_URN,
    create:
        |pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         receiver_config: &ReceiverConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            if pipeline.num_cores() != 1 {
                return Err(ConfigError::InvalidUserConfig {
                    error: "the Oracle receiver requires a single-core pipeline".to_owned(),
                });
            }
            Ok(ReceiverWrapper::local(
                build(&node_config.config)?,
                node,
                node_config,
                receiver_config,
            ))
        },
    validate_config: validate,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
    use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node};
    use std::time::{Duration, Instant};

    /// Scenario: A live Oracle database is explicitly configured for an end-to-end smoke test.
    /// Guarantees: The registered receiver validates metadata, polls Oracle, and emits OTLP logs.
    #[test]
    fn emits_oracle_rows_when_live_test_is_enabled() {
        if std::env::var_os("OTAP_ORACLE_RECEIVER_E2E").is_none() {
            return;
        }
        let config = serde_json::json!({
            "source_id": "current-time",
            "connection": {
                "connect_string": std::env::var("ORACLE_CONNECT_STRING")
                    .unwrap_or_else(|_| "//localhost:1521/FREEPDB1".to_owned()),
                "instant_client_dir": std::env::var("ORACLE_INSTANT_CLIENT_DIR")
                    .unwrap_or_else(|_| "C:\\oracle\\instantclient".to_owned())
            },
            "authentication": {
                "username_file": std::env::var("ORACLE_USERNAME_FILE")
                    .unwrap_or_else(|_| "C:\\secrets\\oracle-username".to_owned()),
                "password_file": std::env::var("ORACLE_PASSWORD_FILE")
                    .unwrap_or_else(|_| "C:\\secrets\\oracle-password".to_owned())
            },
            "query": {
                "statement": "SELECT 1 AS ID, SYSTIMESTAMP AS CURRENT_TIME FROM DUAL",
                "interval": "100ms",
                "fetch_size": 10,
                "max_rows_per_poll": 10,
                "timeout": "10s"
            },
            "watermark": {
                "timestamp_column": "CURRENT_TIME",
                "tie_breaker_column": "ID",
                "timezone": "UTC",
                "start_at": "beginning"
            },
            "checkpoint": {
                "directory": "${engine.state_dir}/oracle",
                "on_nack": "rewind",
                "max_consecutive_failures": 5
            }
        });
        let receiver = build(&config).expect("receiver config should build");
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
                let mut pdata = ctx.recv().await.expect("receiver should emit pdata");
                assert_eq!(pdata.num_items(), 1);
            });
    }
}
