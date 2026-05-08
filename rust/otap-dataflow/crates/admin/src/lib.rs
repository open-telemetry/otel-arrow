// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP server for exposing admin endpoints.

mod convert;
mod dashboard;
pub mod error;
mod health;
mod pipeline;
mod pipeline_group;
mod telemetry;

use axum::Router;
use otap_df_admin_types::operations::{OperationError, OperationErrorKind};
pub use otap_df_admin_types::pipelines::{
    PipelineDetails, PipelineRolloutState, PipelineRolloutSummary, ReconfigureRequest,
    RolloutCoreStatus, RolloutStatus, ShutdownCoreStatus, ShutdownStatus,
};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;

use crate::error::Error;
use otap_df_config::engine::HttpAdminSettings;
use otap_df_engine::memory_limiter::MemoryPressureState;
use otap_df_state::store::ObservedStateHandle;
use otap_df_telemetry::log_tap::InternalLogTapHandle;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::{otel_info, otel_warn};

/// Control-plane error surfaced to admin handlers.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ControlPlaneError {
    /// The requested pipeline group does not exist.
    GroupNotFound,
    /// The requested pipeline does not exist.
    PipelineNotFound,
    /// Another incompatible live operation is active in the current consistency scope.
    RolloutConflict,
    /// Submitted pipeline configuration failed validation or violated a runtime boundary.
    InvalidRequest {
        /// Human-readable validation failure detail.
        message: String,
    },
    /// The requested rollout could not be found.
    RolloutNotFound,
    /// The requested shutdown could not be found.
    ShutdownNotFound,
    /// Unexpected internal failure while processing the request.
    Internal {
        /// Human-readable internal failure detail.
        message: String,
    },
}

impl ControlPlaneError {
    /// Converts a control-plane error into the public operation rejection model.
    #[must_use]
    pub fn as_operation_error(&self) -> OperationError {
        match self {
            Self::GroupNotFound => OperationError::new(OperationErrorKind::GroupNotFound),
            Self::PipelineNotFound => OperationError::new(OperationErrorKind::PipelineNotFound),
            Self::RolloutConflict => OperationError::new(OperationErrorKind::Conflict),
            Self::InvalidRequest { message } => {
                OperationError::new(OperationErrorKind::InvalidRequest)
                    .with_message(message.clone())
            }
            Self::RolloutNotFound => OperationError::new(OperationErrorKind::RolloutNotFound),
            Self::ShutdownNotFound => OperationError::new(OperationErrorKind::ShutdownNotFound),
            Self::Internal { message } => {
                OperationError::new(OperationErrorKind::Internal).with_message(message.clone())
            }
        }
    }
}

/// Control-plane interface implemented by the controller runtime.
pub trait ControlPlane: Send + Sync {
    /// Requests shutdown of all currently running runtime instances.
    fn shutdown_all(&self, timeout_secs: u64) -> Result<(), ControlPlaneError>;

    /// Requests shutdown of all currently running runtime instances for one logical pipeline.
    fn shutdown_pipeline(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<ShutdownStatus, ControlPlaneError>;

    /// Reconfigures a logical pipeline and returns the rollout job snapshot.
    fn reconfigure_pipeline(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: ReconfigureRequest,
    ) -> Result<RolloutStatus, ControlPlaneError>;

    /// Returns the live active config for a logical pipeline.
    fn pipeline_details(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<PipelineDetails>, ControlPlaneError>;

    /// Returns the detailed status for a rollout job.
    fn rollout_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        rollout_id: &str,
    ) -> Result<Option<RolloutStatus>, ControlPlaneError>;

    /// Returns the detailed status for a shutdown job.
    fn shutdown_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        shutdown_id: &str,
    ) -> Result<Option<ShutdownStatus>, ControlPlaneError>;
}

/// Shared state for the HTTP admin server.
#[derive(Clone)]
struct AppState {
    /// The observed state store for querying the current state of the entire system.
    observed_state_store: ObservedStateHandle,

    /// The metrics registry for querying current metrics.
    metrics_registry: TelemetryRegistryHandle,

    /// Resident controller control plane for runtime mutations.
    controller: Arc<dyn ControlPlane>,

    /// Optional internal log tap for querying retained internal logs.
    log_tap: Option<InternalLogTapHandle>,

    /// Shared process-wide memory pressure state.
    memory_pressure_state: MemoryPressureState,

    /// Pre-rendered Prometheus `target_info` block (HELP/TYPE/sample lines)
    /// derived once from the resource attributes at server startup. Empty
    /// when no resource attributes are supplied; written verbatim at the top
    /// of every Prometheus scrape so we don't re-sort and re-escape on each
    /// request.
    target_info: Arc<str>,
}

/// Run the admin HTTP server until shutdown is requested.
pub async fn run(
    config: HttpAdminSettings,
    observed_store: ObservedStateHandle,
    controller: Arc<dyn ControlPlane>,
    metrics_registry: TelemetryRegistryHandle,
    memory_pressure_state: MemoryPressureState,
    log_tap: Option<InternalLogTapHandle>,
    resource_attributes: HashMap<String, String>,
    cancel: CancellationToken,
) -> Result<(), Error> {
    let target_info: Arc<str> = telemetry::render_target_info(&resource_attributes).into();
    let app_state = AppState {
        observed_state_store: observed_store,
        metrics_registry,
        controller,
        log_tap,
        memory_pressure_state,
        target_info,
    };

    let api_routes = Router::new()
        .merge(health::routes())
        .merge(telemetry::routes())
        .merge(pipeline_group::routes())
        .merge(pipeline::routes());

    let app = Router::new()
        .nest("/api/v1", api_routes)
        .merge(dashboard::routes())
        .layer(ServiceBuilder::new())
        .with_state(app_state);

    // Parse the configured bind address.
    let addr = config.bind_address.parse::<SocketAddr>().map_err(|e| {
        let details = format!("{e}");
        otel_warn!(
            "endpoint.parse_address_failed",
            bind_address = config.bind_address.as_str(),
            error = details.as_str(),
            message = "Failed to parse admin server bind address"
        );
        Error::InvalidBindAddress {
            bind_address: config.bind_address.clone(),
            details,
        }
    })?;

    // Bind the TCP listener.
    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        let addr_str = addr.to_string();
        let details = format!("{e}");
        otel_warn!(
            "endpoint.bind_failed",
            bind_address = addr_str.as_str(),
            error = details.as_str(),
            message = "Failed to bind admin server"
        );
        Error::BindFailed {
            addr: addr_str,
            details,
        }
    })?;

    let addr_str = addr.to_string();
    otel_info!(
        "endpoint.start",
        bind_address = addr_str.as_str(),
        message = "Admin HTTP server listening"
    );

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
