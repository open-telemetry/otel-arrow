// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Oracle implementation of the shared database polling contracts.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = ORACLE_RECEIVER_URN,
    target = "otel.receiver.oracle",
);

#[cfg(test)]
#[macro_use]
#[path = "test.rs"]
mod tests;

mod adapter;
mod config;
mod worker;

use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::ReceiverFactory;
use otel_arrow_dfe_engine::config::ReceiverConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::memory_limiter::LocalReceiverAdmissionState;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_scraper::{
    CheckpointStore, DatabaseReceiver, DatabaseReceiverMetrics, LeaseError, SourceBinding,
};
use otel_arrow_dfe_telemetry::metrics::MetricSetRegistrar;
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

pub use adapter::{OracleAdapter, OracleAdapterError};
pub use config::{OracleConfigError, OracleReceiverConfig};

/// Stable component identifier for the Oracle receiver.
pub const ORACLE_RECEIVER_URN: &str = "urn:otel:receiver:oracle";

type Receiver = DatabaseReceiver<OracleAdapter>;

fn parse(config: &Value) -> Result<OracleReceiverConfig, ConfigError> {
    OracleReceiverConfig::deserialize(config).map_err(invalid_config)
}

/// Builds a receiver bound to one durable checkpoint source.
fn build(
    pipeline: &PipelineContext,
    receiver_name: &str,
    config: &Value,
) -> Result<Receiver, ConfigError> {
    let config = parse(config)?;
    let query = config.query();
    let checkpoint = config.checkpoint();
    let store = CheckpointStore::new(
        Path::new(&checkpoint.directory),
        pipeline.pipeline_group_id().as_ref(),
        pipeline.pipeline_id().as_ref(),
        receiver_name,
        config.source_id(),
        config.config_fingerprint().to_owned(),
    );
    let source = SourceBinding::acquire(store).map_err(invalid_source_lease)?;
    let metrics = Some(pipeline.register_metric_set::<DatabaseReceiverMetrics>());
    Ok(DatabaseReceiver::new(
        config.adapter(),
        query,
        source,
        checkpoint.nack_backoff,
        checkpoint.max_consecutive_failures,
        LocalReceiverAdmissionState::from_process_state(&pipeline.memory_pressure_state()),
        metrics,
    ))
}

/// Validates configuration without acquiring a source lease.
fn validate(config: &Value) -> Result<(), ConfigError> {
    parse(config).map(|_| ())
}

fn invalid_config(error: impl std::fmt::Display) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.to_string(),
    }
}

fn invalid_source_lease(error: LeaseError) -> ConfigError {
    match error {
        LeaseError::AlreadyOwned => {
            invalid_config("another database receiver already owns this checkpoint source")
        }
        LeaseError::Unavailable => {
            invalid_config("Oracle checkpoint lease registry is unavailable")
        }
        LeaseError::InvalidPath { .. } => invalid_config("Oracle checkpoint lease path is invalid"),
        LeaseError::InvalidGeneration { .. } => {
            invalid_config("Oracle checkpoint lease generation is invalid")
        }
        LeaseError::GenerationOverflow { .. } => {
            invalid_config("Oracle checkpoint lease generation overflowed")
        }
        LeaseError::Io { source, .. } => invalid_config(format!(
            "Oracle checkpoint lease I/O failed (kind {:?}, OS {:?})",
            source.kind(),
            source.raw_os_error(),
        )),
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
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
};
