// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP server for exposing admin endpoints.

pub mod error;
mod pipeline;
mod pipeline_group;
mod telemetry;

use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;

use crate::error::Error;
use otap_df_config::engine::HttpAdminSettings;
use otap_df_engine::control::PipelineCtrlMsgSender;
use otap_df_state::store::ObservedStateHandle;
use otap_df_telemetry::registry::MetricsRegistryHandle;

/// Shared state for the HTTP admin server.
#[derive(Clone)]
struct AppState {
    /// The observed state store for querying the current state of the entire system.
    observed_state_store: ObservedStateHandle,

    /// The metrics registry for querying current metrics.
    metrics_registry: MetricsRegistryHandle,

    /// The control message senders for controlling pipelines.
    ctrl_msg_senders: Vec<PipelineCtrlMsgSender>,
}

/// Run the admin HTTP server until shutdown is requested.
pub async fn run(
    config: HttpAdminSettings,
    observed_store: ObservedStateHandle,
    ctrl_msg_senders: Vec<PipelineCtrlMsgSender>,
    metrics_registry: MetricsRegistryHandle,
    cancel: CancellationToken,
) -> Result<(), Error> {
    let app_state = AppState {
        observed_state_store: observed_store,
        metrics_registry,
        ctrl_msg_senders,
    };

    let app = Router::new()
        .merge(telemetry::routes())
        .merge(pipeline_group::routes())
        .merge(pipeline::routes())
        .layer(ServiceBuilder::new())
        .with_state(app_state);

    // Parse the configured bind address.
    let addr =
        config
            .bind_address
            .parse::<SocketAddr>()
            .map_err(|e| Error::InvalidBindAddress {
                bind_address: config.bind_address.clone(),
                details: format!("{e}"),
            })?;

    // Bind the TCP listener.
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| Error::BindFailed {
            addr: addr.to_string(),
            details: format!("{e}"),
        })?;

    // Start serving requests, with graceful shutdown on signal.
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancel.cancelled().await;
        })
        .await
        .map_err(|e| Error::ServerError {
            addr: addr.to_string(),
            details: format!("{e}"),
        })
}
