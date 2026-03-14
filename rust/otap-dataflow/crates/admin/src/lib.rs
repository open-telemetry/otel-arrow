// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP server for exposing admin endpoints.

mod dashboard;
pub mod error;
mod health;
mod pipeline;
mod pipeline_group;
mod telemetry;

use axum::Router;
use otap_df_config::{PipelineGroupId, PipelineId, pipeline::PipelineConfig};
use otap_df_state::pipeline_status::PipelineRolloutSummary;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;

use crate::error::Error;
use otap_df_config::engine::HttpAdminSettings;
use otap_df_state::store::ObservedStateHandle;
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
    /// Another rollout is already active for this logical pipeline.
    RolloutConflict,
    /// Submitted pipeline configuration failed validation or violated a v1 boundary.
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

/// Body for `PUT /pipeline-groups/{group}/pipelines/{id}`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplacePipelineRequest {
    /// Candidate pipeline configuration to create or roll out.
    pub pipeline: PipelineConfig,
    /// Per-core admission/ready timeout in seconds.
    #[serde(default = "default_rollout_timeout_secs")]
    pub step_timeout_secs: u64,
    /// Graceful drain timeout in seconds when shutting down the old generation.
    #[serde(default = "default_rollout_timeout_secs")]
    pub drain_timeout_secs: u64,
}

/// Detailed per-core rollout progress.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RolloutCoreStatus {
    /// Target core for this step.
    pub core_id: usize,
    /// Previously serving generation on this core, if any.
    pub previous_generation: Option<u64>,
    /// Candidate generation being launched for this core.
    pub target_generation: u64,
    /// Current lifecycle state for this core step.
    pub state: String,
    /// RFC3339 timestamp for the latest step transition.
    pub updated_at: String,
    /// Optional human-readable detail for failures or waits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Detailed rollout status returned by rollout endpoints.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineRolloutStatus {
    /// Controller-assigned rollout identifier.
    pub rollout_id: String,
    /// Logical target pipeline group id.
    pub pipeline_group_id: PipelineGroupId,
    /// Logical target pipeline id.
    pub pipeline_id: PipelineId,
    /// `create` when this was a new pipeline, `replace` when rolling an existing one.
    pub action: String,
    /// Current rollout lifecycle state.
    pub state: String,
    /// Candidate generation targeted by this rollout.
    pub target_generation: u64,
    /// Previously committed generation, if any.
    pub previous_generation: Option<u64>,
    /// RFC3339 timestamp for rollout creation.
    pub started_at: String,
    /// RFC3339 timestamp for the latest rollout transition.
    pub updated_at: String,
    /// Optional failure or rollback reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    /// Per-core rollout progress entries.
    pub cores: Vec<RolloutCoreStatus>,
}

/// Detailed per-core shutdown progress.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownCoreStatus {
    /// Target core being drained.
    pub core_id: usize,
    /// Deployment generation targeted for shutdown on this core.
    pub deployment_generation: u64,
    /// Current lifecycle state for this core shutdown step.
    pub state: String,
    /// RFC3339 timestamp for the latest step transition.
    pub updated_at: String,
    /// Optional human-readable detail for failures or waits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Detailed shutdown status returned by shutdown endpoints.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineShutdownStatus {
    /// Controller-assigned shutdown identifier.
    pub shutdown_id: String,
    /// Logical target pipeline group id.
    pub pipeline_group_id: PipelineGroupId,
    /// Logical target pipeline id.
    pub pipeline_id: PipelineId,
    /// Current shutdown lifecycle state.
    pub state: String,
    /// RFC3339 timestamp for shutdown creation.
    pub started_at: String,
    /// RFC3339 timestamp for the latest shutdown transition.
    pub updated_at: String,
    /// Optional failure reason when shutdown does not complete cleanly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    /// Per-core shutdown progress entries.
    pub cores: Vec<ShutdownCoreStatus>,
}

/// Live logical pipeline view returned by `GET /pipeline-groups/{group}/pipelines/{id}`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDetails {
    /// Logical pipeline group id.
    pub pipeline_group_id: PipelineGroupId,
    /// Logical pipeline id.
    pub pipeline_id: PipelineId,
    /// Last committed active generation, if known.
    pub active_generation: Option<u64>,
    /// Current live pipeline configuration.
    pub pipeline: PipelineConfig,
    /// Optional rollout summary mirrored into `/status`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollout: Option<PipelineRolloutSummary>,
}

const fn default_rollout_timeout_secs() -> u64 {
    60
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
    ) -> Result<PipelineShutdownStatus, ControlPlaneError>;

    /// Creates or replaces a logical pipeline and returns the rollout job snapshot.
    fn replace_pipeline(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: ReplacePipelineRequest,
    ) -> Result<PipelineRolloutStatus, ControlPlaneError>;

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
    ) -> Result<Option<PipelineRolloutStatus>, ControlPlaneError>;

    /// Returns the detailed status for a shutdown job.
    fn shutdown_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        shutdown_id: &str,
    ) -> Result<Option<PipelineShutdownStatus>, ControlPlaneError>;
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
}

/// Run the admin HTTP server until shutdown is requested.
pub async fn run(
    config: HttpAdminSettings,
    observed_store: ObservedStateHandle,
    controller: Arc<dyn ControlPlane>,
    metrics_registry: TelemetryRegistryHandle,
    cancel: CancellationToken,
) -> Result<(), Error> {
    let app_state = AppState {
        observed_state_store: observed_store,
        metrics_registry,
        controller,
    };

    let app = Router::new()
        .merge(health::routes())
        .merge(telemetry::routes())
        .merge(pipeline_group::routes())
        .merge(pipeline::routes())
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
