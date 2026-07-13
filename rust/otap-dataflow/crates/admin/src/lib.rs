// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP server for exposing admin endpoints.

mod convert;
mod dashboard;
mod engine_config;
pub mod error;
mod health;
mod pipeline;
mod pipeline_group;
mod telemetry;

use axum::Router;
use axum::response::Response;
pub use otap_df_admin_types::engine::{
    ConfigChangeAction, ConfigChangeStatus, EngineConfigReconcileRequest,
    EngineConfigReconcileState, EngineConfigReconcileStatus, GroupDeleteStatus,
    PipelineDeleteStatus,
};
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
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;

use crate::error::Error;
use otap_df_config::engine::{HttpAdminSettings, OtelDataflowSpec};
use otap_df_config::pipeline::telemetry::AttributeValue as ResourceAttributeValue;
use otap_df_config::pipeline_group::PipelineGroupConfig;
use otap_df_engine::memory_limiter::MemoryPressureState;
use otap_df_state::store::ObservedStateHandle;
use otap_df_telemetry::log_tap::InternalLogTapHandle;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::{otel_info, otel_warn};

const TERMINAL_CONTROL_PLANE_PERMITS: usize = 1;

/// Control-plane error surfaced to admin handlers.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ControlPlaneError {
    /// The requested pipeline group does not exist.
    GroupNotFound,
    /// The requested pipeline group already exists.
    GroupAlreadyExists,
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
            Self::GroupAlreadyExists => OperationError::new(OperationErrorKind::Conflict),
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

    /// Returns the committed configuration for one pipeline group.
    fn group_details(
        &self,
        _pipeline_group_id: &str,
    ) -> Result<Option<PipelineGroupConfig>, ControlPlaneError> {
        let _ = self;
        Err(ControlPlaneError::Internal {
            message: "pipeline group details are not supported by this control plane".to_owned(),
        })
    }

    /// Creates one empty pipeline group in the controller-owned configuration.
    fn create_group(
        &self,
        _pipeline_group_id: &str,
        _group: PipelineGroupConfig,
    ) -> Result<PipelineGroupConfig, ControlPlaneError> {
        let _ = self;
        Err(ControlPlaneError::Internal {
            message: "pipeline group creation is not supported by this control plane".to_owned(),
        })
    }

    /// Returns the full current engine configuration known to the controller.
    fn engine_config_snapshot(&self) -> Result<OtelDataflowSpec, ControlPlaneError> {
        let _ = self;
        Err(ControlPlaneError::Internal {
            message: "engine config snapshots are not supported by this control plane".to_owned(),
        })
    }

    /// Reconciles the controller runtime to the supplied full desired configuration.
    fn reconcile_engine_config(
        &self,
        _request: EngineConfigReconcileRequest,
    ) -> Result<EngineConfigReconcileStatus, ControlPlaneError> {
        let _ = self;
        Err(ControlPlaneError::Internal {
            message: "full engine config reconciliation is not supported by this control plane"
                .to_owned(),
        })
    }

    /// Gracefully drains and removes one logical pipeline from the controller state.
    fn delete_pipeline(
        &self,
        _pipeline_group_id: &str,
        _pipeline_id: &str,
        _timeout_secs: u64,
    ) -> Result<PipelineDeleteStatus, ControlPlaneError> {
        let _ = self;
        Err(ControlPlaneError::Internal {
            message: "pipeline deletion is not supported by this control plane".to_owned(),
        })
    }

    /// Gracefully drains and removes one logical pipeline group from the controller state.
    fn delete_group(
        &self,
        _pipeline_group_id: &str,
        _timeout_secs: u64,
    ) -> Result<GroupDeleteStatus, ControlPlaneError> {
        let _ = self;
        Err(ControlPlaneError::Internal {
            message: "pipeline group deletion is not supported by this control plane".to_owned(),
        })
    }
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

    /// Bounds long synchronous terminal control-plane operations dispatched
    /// from async HTTP handlers.
    terminal_control_plane_permits: Arc<Semaphore>,

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

impl AppState {
    async fn run_terminal_control_plane<T, F>(&self, operation: F) -> Result<T, ControlPlaneError>
    where
        T: Send + 'static,
        F: FnOnce(Arc<dyn ControlPlane>) -> Result<T, ControlPlaneError> + Send + 'static,
    {
        let permit = self
            .terminal_control_plane_permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| ControlPlaneError::RolloutConflict)?;
        let controller = Arc::clone(&self.controller);
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            operation(controller)
        })
        .await
        .map_err(|err| ControlPlaneError::Internal {
            message: format!("terminal control-plane operation failed to join: {err}"),
        })?
    }
}

/// Attaches hardened security headers to every `/api/v1/*` response.
///
/// Mirrors the non-CSP subset of the UI/static header set from `dashboard.rs`:
/// `Cache-Control`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`.
/// Content-Security-Policy is intentionally omitted — it is only meaningful for
/// HTML responses served by the UI, not for JSON API endpoints.
/// Centralising them as a layer on `api_routes` means new endpoints inherit them.
async fn attach_api_security_headers(mut response: Response) -> Response {
    use axum::http::{HeaderName, HeaderValue, header};
    let h = response.headers_mut();
    let _ = h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    let _ = h.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    let _ = h.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    let _ = h.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    response
}

/// Run the admin HTTP server until shutdown is requested.
pub async fn run(
    config: HttpAdminSettings,
    observed_store: ObservedStateHandle,
    controller: Arc<dyn ControlPlane>,
    metrics_registry: TelemetryRegistryHandle,
    memory_pressure_state: MemoryPressureState,
    log_tap: Option<InternalLogTapHandle>,
    resource_attributes: HashMap<String, ResourceAttributeValue>,
    cancel: CancellationToken,
) -> Result<(), Error> {
    let target_info: Arc<str> = telemetry::render_target_info(&resource_attributes).into();
    let app_state = AppState {
        observed_state_store: observed_store,
        metrics_registry,
        controller,
        terminal_control_plane_permits: Arc::new(Semaphore::new(TERMINAL_CONTROL_PLANE_PERMITS)),
        log_tap,
        memory_pressure_state,
        target_info,
    };

    let api_routes = Router::new()
        .merge(health::routes())
        .merge(telemetry::routes())
        .merge(engine_config::routes())
        .merge(pipeline_group::routes())
        .merge(pipeline::routes())
        .layer(axum::middleware::map_response(attach_api_security_headers));

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
