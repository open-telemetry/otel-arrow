// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! PostgreSQL 15 bounded composite-cursor receiver.

#[cfg(test)]
#[macro_use]
#[path = "test.rs"]
mod tests;

mod adapter;
mod config;
mod query;
mod value;
mod worker;

use linkme::distributed_slice;
use otel_arrow_dfe_config::{error::Error as ConfigError, node::NodeUserConfig};
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BasicAuthProvider;
use otel_arrow_dfe_engine::capability::registry::Capabilities;
use otel_arrow_dfe_engine::{
    ReceiverFactory, config::ReceiverConfig, context::PipelineContext,
    memory_limiter::LocalReceiverAdmissionState, node::NodeId, receiver::ReceiverWrapper,
};
use otel_arrow_dfe_otap::{OTAP_RECEIVER_FACTORIES, pdata::OtapPdata};
use otel_arrow_dfe_scraper::{
    CheckpointStore, DatabaseReceiver, DatabaseReceiverMetrics, LeaseError, SourceBinding,
};
use std::{path::Path, sync::Arc};

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = POSTGRESQL_RECEIVER_URN,
    target = "otel.receiver.postgresql",
);

/// PostgreSQL receiver registration identity.
pub const POSTGRESQL_RECEIVER_URN: &str = "urn:otel:receiver:postgresql";

#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Registers the opt-in local PostgreSQL receiver.
pub static POSTGRESQL_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: POSTGRESQL_RECEIVER_URN,
    create,
    validate_config: |value| {
        config::PostgreSqlReceiverConfig::parse(value)
            .and_then(config::PostgreSqlReceiverConfig::validate)
            .map(|_| ())
            .map_err(config_error)
    },
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

fn config_error(error: adapter::Error) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.to_string(),
    }
}

fn source_lease_error(error: LeaseError) -> ConfigError {
    let error = match error {
        LeaseError::AlreadyOwned => {
            "another database receiver already owns this PostgreSQL checkpoint source".into()
        }
        LeaseError::Unavailable => "PostgreSQL checkpoint lease registry is unavailable".into(),
        LeaseError::InvalidPath { .. } => "PostgreSQL checkpoint lease path is invalid".into(),
        LeaseError::InvalidGeneration { .. } => {
            "PostgreSQL checkpoint lease generation is invalid".into()
        }
        LeaseError::GenerationOverflow { .. } => {
            "PostgreSQL checkpoint lease generation overflowed".into()
        }
        LeaseError::Io { source, .. } => format!(
            "PostgreSQL checkpoint lease I/O failed (kind {:?}, OS {:?})",
            source.kind(),
            source.raw_os_error(),
        ),
    };
    ConfigError::InvalidUserConfig { error }
}

fn create(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    receiver_config: &ReceiverConfig,
    capabilities: &Capabilities,
) -> Result<ReceiverWrapper<OtapPdata>, ConfigError> {
    if pipeline.num_cores() != 1 {
        return Err(ConfigError::InvalidUserConfig {
            error: "postgresql requires a one-core source pipeline".into(),
        });
    }
    // Resolve required wiring before acquiring source ownership or starting a worker.
    let credentials = capabilities
        .require_local::<BasicAuthProvider>()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })?;
    let validated = config::PostgreSqlReceiverConfig::parse(&node_config.config)
        .and_then(config::PostgreSqlReceiverConfig::validate)
        .map_err(config_error)?;
    let store = CheckpointStore::new(
        Path::new(&validated.config.checkpoint.directory),
        pipeline.pipeline_group_id().as_ref(),
        pipeline.pipeline_id().as_ref(),
        node.name.as_ref(),
        &validated.config.source_id,
        validated.fingerprint.clone(),
    );
    // Acquire exclusive ownership before driver work starts; the shared receiver
    // retains the binding through ACK/checkpoint handling and confirmed shutdown.
    let binding = SourceBinding::acquire(store).map_err(source_lease_error)?;
    let query = validated.common.clone();
    let backoff = validated.config.checkpoint.nack_backoff;
    let failures = validated.config.checkpoint.max_consecutive_failures;
    let adapter = adapter::PostgreSqlAdapter::new(validated, credentials).map_err(config_error)?;
    let metrics = DatabaseReceiverMetrics::register(&pipeline);
    let admission =
        LocalReceiverAdmissionState::from_process_state(&pipeline.memory_pressure_state());
    let receiver = DatabaseReceiver::new(
        adapter,
        query,
        binding,
        backoff,
        failures,
        admission,
        Some(metrics),
    );
    Ok(ReceiverWrapper::local(
        receiver,
        node,
        node_config,
        receiver_config,
    ))
}
