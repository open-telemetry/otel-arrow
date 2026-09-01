// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle implementation of the shared database polling contracts.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = ORACLE_RECEIVER_URN,
    target = "otel.receiver.oracle",
);

mod adapter;
mod config;

use crate::receivers::database::{
    CheckpointStore, DatabaseReceiver, DatabaseReceiverMetrics, SourceLease,
};
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
use std::path::Path;
use std::sync::Arc;

pub use adapter::{OracleAdapter, OracleAdapterError};
pub use config::{OracleConfigError, OracleReceiverConfig};

/// Stable component identifier for the Oracle receiver.
pub const ORACLE_RECEIVER_URN: &str = "urn:otel:receiver:oracle";

type Receiver = DatabaseReceiver<OracleAdapter>;

fn parse(config: &Value) -> Result<OracleReceiverConfig, ConfigError> {
    serde_json::from_value(config.clone()).map_err(invalid_config)
}

/// Builds a receiver bound to one durable checkpoint source.
fn build(
    pipeline: &PipelineContext,
    receiver_name: &str,
    config: &Value,
) -> Result<Receiver, ConfigError> {
    let config = parse(config)?;
    let query = config.compile().map_err(invalid_config)?;
    let checkpoint = config.checkpoint();
    let store = CheckpointStore::new(
        Path::new(&checkpoint.directory),
        pipeline.pipeline_group_id().as_ref(),
        pipeline.pipeline_id().as_ref(),
        receiver_name,
        config.source_id(),
        config.config_fingerprint().to_owned(),
    );
    let lease = SourceLease::acquire(&store.lease_key()).map_err(invalid_config)?;
    let metrics = Some(pipeline.register_metrics::<DatabaseReceiverMetrics>());
    Ok(DatabaseReceiver::new(
        config.adapter(),
        query,
        store,
        lease,
        checkpoint.nack_backoff,
        checkpoint.max_consecutive_failures,
        config.source_id().to_owned(),
        metrics,
    ))
}

/// Validates configuration without acquiring a source lease.
fn validate(config: &Value) -> Result<(), ConfigError> {
    let config = parse(config)?;
    _ = config.compile().map_err(invalid_config)?;
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
            let receiver = build(
                &pipeline,
                receiver_config.name.as_ref(),
                &node_config.config,
            )?;
            Ok(ReceiverWrapper::local(
                receiver,
                node,
                node_config,
                receiver_config,
            ))
        },
    validate_config: validate,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

#[cfg(test)]
mod tests;
