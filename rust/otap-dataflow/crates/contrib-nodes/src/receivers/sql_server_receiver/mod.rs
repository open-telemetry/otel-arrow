// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental SQL Server polling receiver using the shared database scraper.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = SQL_SERVER_RECEIVER_URN,
    target = "otel.receiver.sql_server",
);

mod adapter;
mod config;
mod connector;
mod data;
mod error;
mod session;
mod sql;

pub use adapter::{SqlServerAdapter, SqlServerCancellation, SqlServerConnector};
pub use config::{SqlServerConfigError, SqlServerConnectionConfig, SqlServerReceiverConfig};
pub use connector::TcpConnector;
pub use error::SqlServerError;
pub use sql::{SqlServerParameter, SqlServerStatement};

#[cfg(test)]
mod adapter_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod live_tests;

#[cfg(test)]
mod network_tests;

use linkme::distributed_slice;
use otel_arrow_dfe_config::{error::Error as ConfigError, node::NodeUserConfig};
use otel_arrow_dfe_engine::{
    ReceiverFactory, capability::auth::basic_auth_provider::BasicAuthProvider,
    config::ReceiverConfig, context::PipelineContext, memory_limiter::LocalReceiverAdmissionState,
    node::NodeId, receiver::ReceiverWrapper,
};
use otel_arrow_dfe_otap::{OTAP_RECEIVER_FACTORIES, pdata::OtapPdata};
use otel_arrow_dfe_scraper::{
    CheckpointStore, DatabaseReceiver, DatabaseReceiverMetrics, LeaseError, SourceBinding,
};
use otel_arrow_dfe_telemetry::metrics::MetricSetRegistrar;
use serde::Deserialize;
use std::{path::Path, sync::Arc};

/// Stable SQL Server receiver identifier.
pub const SQL_SERVER_RECEIVER_URN: &str = "urn:otel:receiver:sql_server";

/// Registers the independently enabled, single-core SQL Server receiver.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static SQL_SERVER_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: SQL_SERVER_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             capabilities| {
        if pipeline.num_cores() != 1 {
            return Err(invalid(
                "the SQL Server receiver requires a single-core pipeline",
            ));
        }
        let credentials = capabilities
            .require_local::<BasicAuthProvider>()
            .map_err(invalid)?;
        let config = SqlServerReceiverConfig::deserialize(&node_config.config).map_err(invalid)?;
        if !config.connection().tls_enabled() {
            otel_arrow_dfe_telemetry::otel_warn!(
                "sql_server.tls_not_required",
                message = "SQL Server TLS is not required by this receiver. Credentials and rows may be unencrypted; use only for isolated testing. Server policy may still require encryption."
            );
        } else if config.connection().trust_server_certificate() {
            otel_arrow_dfe_telemetry::otel_warn!(
                "sql_server.certificate_verification_disabled",
                message = "SQL Server certificate-chain and hostname validation are disabled. The server is not authenticated; use only for isolated testing."
            );
        }
        let query = config.query().clone();
        let checkpoint = config.checkpoint();
        let source = SourceBinding::acquire(CheckpointStore::new(
            Path::new(&checkpoint.directory),
            pipeline.pipeline_group_id().as_ref(),
            pipeline.pipeline_id().as_ref(),
            receiver_config.name.as_ref(),
            config.source_id(),
            config.config_fingerprint().to_owned(),
        ))
        .map_err(lease_error)?;
        let backoff = checkpoint.nack_backoff;
        let failures = checkpoint.max_consecutive_failures;
        let metrics = Some(pipeline.register_metric_set::<DatabaseReceiverMetrics>());
        let receiver = DatabaseReceiver::new(
            SqlServerAdapter::new(config, TcpConnector::default(), credentials),
            query,
            source,
            backoff,
            failures,
            LocalReceiverAdmissionState::from_process_state(&pipeline.memory_pressure_state()),
            metrics,
        );
        Ok(ReceiverWrapper::local(
            receiver,
            node,
            node_config,
            receiver_config,
        ))
    },
    validate_config: |config| {
        SqlServerReceiverConfig::deserialize(config)
            .map(|_| ())
            .map_err(invalid)
    },
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

fn invalid(error: impl std::fmt::Display) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.to_string(),
    }
}

fn lease_error(error: LeaseError) -> ConfigError {
    match error {
        LeaseError::AlreadyOwned => invalid("another receiver already owns this checkpoint source"),
        LeaseError::Io { source, .. } => invalid(format!(
            "SQL Server checkpoint lease I/O failed (kind {:?}, OS {:?})",
            source.kind(),
            source.raw_os_error(),
        )),
        _ => invalid("SQL Server checkpoint ownership is unavailable or invalid"),
    }
}
