// SPDX-License-Identifier: Apache-2.0

//! HTTP server for exposing admin endpoints.

pub mod error;
mod telemetry;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::thread::JoinHandle;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use otap_df_config::engine::HttpAdminSettings;
use otap_df_telemetry::registry::MetricsRegistryHandle;
use crate::error::Error;

/// Shared state for the HTTP admin server.
#[derive(Clone)]
struct AppState {
    metrics_registry: MetricsRegistryHandle,
}

/// Run the admin HTTP server until shutdown is requested.
pub async fn run(
    config: HttpAdminSettings,
    metrics_registry: MetricsRegistryHandle,
    cancel: CancellationToken,
) -> Result<(), Error> {
    let app_state = AppState { metrics_registry };

    let app = Router::new()
        .route("/telemetry/live-schema", get(telemetry::get_live_schema))
        .route("/telemetry/metrics", get(telemetry::get_metrics))
        .route("/telemetry/metrics/aggregate", get(telemetry::get_metrics_aggregate))
        .layer(ServiceBuilder::new())
        .with_state(app_state);

    // Parse the configured bind address.
    let addr = config
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

/// Handle to the running HTTP admin server thread.
///
/// Allows signaling a graceful shutdown and waiting for the thread to exit.
pub struct ServerHandle {
    addr: String,
    join_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl ServerHandle {
    /// Signal the server to shut down gracefully and wait for the thread to finish.
    pub fn shutdown(&mut self) -> Result<(), Error> {
        let addr = self.addr.clone();
        // Signal shutdown; if receiver is already dropped the server has exited.
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.join_handle.take() {
            handle.join().map_err(|e| Error::ServerError {
                addr,
                details: format!("HTTP admin thread join failed: {e:?}"),
            })?
        }
        Ok(())
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        // Best-effort shutdown signal on drop; ignore if already sent.
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // Do not join in Drop to avoid blocking unwinding or shutdown paths.
    }
}
