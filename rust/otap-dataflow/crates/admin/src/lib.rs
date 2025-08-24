// SPDX-License-Identifier: Apache-2.0

//! HTTP server for exposing admin endpoints.

pub mod error;
mod telemetry;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::thread;
use std::thread::JoinHandle;
use tokio::net::TcpListener;
use tokio::runtime::Builder;
use tower::ServiceBuilder;

use otap_df_config::engine::HttpAdminSettings;
use otap_df_telemetry::registry::MetricsRegistryHandle;
use crate::error::Error;

/// Shared state for the HTTP admin server.
#[derive(Clone)]
struct AppState {
    metrics_registry: MetricsRegistryHandle,
}

/// Starts the HTTP admin server thread with the given configuration and metrics registry.
pub fn start_thread(
    config: HttpAdminSettings,
    metrics_registry: MetricsRegistryHandle,
) -> Result<JoinHandle<()>, Error> {
    let addr = config.bind_address.clone();

    Ok(thread::Builder::new()
        .name("http-admin".to_owned())
        .spawn(move || {
            let rt = Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime");
            let local_tasks = tokio::task::LocalSet::new();

            let _task: tokio::task::JoinHandle<Result<_, Error>> = local_tasks.spawn_local(async move {
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

                // Start serving requests.
                axum::serve(listener, app)
                    .await
                    .map_err(|e| Error::ServerError {
                        addr: addr.to_string(),
                        details: format!("{e}"),
                    })?;
                Ok(())
            });
            rt.block_on(async {
                local_tasks.await;
            });
        })
        .map_err(|e| Error::ServerError {
            addr,
            details: format!("Failed to spawn HTTP admin thread: {e}"),
        })?)
}
