// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline endpoints.
//!
//! - GET `/groups/{pipeline_group_id}/pipelines/{pipeline_id}`
//!   Get the configuration of the specified pipeline.
//! - GET `/groups/{pipeline_group_id}/pipelines/{pipeline_id}/status`
//!   Get the status of the specified pipeline.
//! - GET `/groups/{pipeline_group_id}/pipelines/{pipeline_id}/rollouts/{rollout_id}`
//!   Get the status of a specific rollout job for the logical pipeline.
//! - GET `/groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdowns/{shutdown_id}`
//!   Get the status of a specific shutdown job for the logical pipeline.
//! - PUT `/groups/{pipeline_group_id}/pipelines/{pipeline_id}`
//!   Create or replace a pipeline and return a rollout job status snapshot.
//! - POST `/groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdown`
//!   Shutdown a specific logical pipeline and return a shutdown job status snapshot.
//!   - Query parameters:
//!     - `wait` (bool, default: false) - if true, block until the pipeline stops
//!     - `timeout_secs` (u64, default: 60) - maximum seconds to wait when `wait=true`
//!   - 200 OK if `wait=true` and the pipeline stopped successfully
//!   - 202 Accepted if the stop request was accepted and is being processed (async operation)
//!   - 400 Bad Request if the pipeline is already stopped
//!   - 404 Not Found if the group or pipeline does not exist
//!   - 409 Conflict if a rollout or shutdown is active for the pipeline, or if a waited
//!     shutdown fails
//!   - 500 Internal Server Error if the stop request could not be processed
//!   - 504 Gateway Timeout if `wait=true` and the pipeline did not stop within timeout
//!
//! ToDo Alternative -> avoid verb-y subpaths and support PATCH /.../pipelines/{pipelineId} with a body like {"status":"stopped"}. Use 409 if already stopping/stopped.

use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use otap_df_config::PipelineKey;
use otap_df_state::pipeline_status::PipelineStatus;
use otap_df_telemetry::otel_info;
use serde::Deserialize;
use std::time::{Duration, Instant};

/// All the routes for pipelines.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/groups/{pipeline_group_id}/pipelines/{pipeline_id}",
            get(show_pipeline).put(put_pipeline),
        )
        // Returns the status of a specific pipeline.
        .route(
            "/groups/{pipeline_group_id}/pipelines/{pipeline_id}/status",
            get(show_status),
        )
        .route(
            "/groups/{pipeline_group_id}/pipelines/{pipeline_id}/rollouts/{rollout_id}",
            get(show_rollout),
        )
        .route(
            "/groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdowns/{shutdown_id}",
            get(show_shutdown),
        )
        .route(
            "/groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdown",
            post(shutdown_pipeline),
        )
        // liveness and readiness probes.
        .route(
            "/groups/{pipeline_group_id}/pipelines/{pipeline_id}/livez",
            get(liveness),
        )
        .route(
            "/groups/{pipeline_group_id}/pipelines/{pipeline_id}/readyz",
            get(readiness),
        )
}

#[derive(Deserialize)]
pub(crate) struct WaitParams {
    #[serde(default)]
    wait: bool,
    #[serde(default = "default_timeout_secs")]
    timeout_secs: u64,
}

const fn default_timeout_secs() -> u64 {
    60
}

fn rollout_is_terminal(state: &str) -> bool {
    matches!(state, "succeeded" | "failed" | "rollback_failed")
}

fn rollout_is_success(state: &str) -> bool {
    state == "succeeded"
}

fn shutdown_is_terminal(state: &str) -> bool {
    matches!(state, "succeeded" | "failed")
}

fn shutdown_is_success(state: &str) -> bool {
    state == "succeeded"
}

pub async fn show_pipeline(
    Path((pipeline_group_id, pipeline_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<crate::PipelineDetails>, StatusCode> {
    match state
        .controller
        .pipeline_details(&pipeline_group_id, &pipeline_id)
    {
        Ok(Some(details)) => Ok(Json(details)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(
            crate::ControlPlaneError::PipelineNotFound | crate::ControlPlaneError::GroupNotFound,
        ) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn put_pipeline(
    Path((pipeline_group_id, pipeline_id)): Path<(String, String)>,
    Query(params): Query<WaitParams>,
    State(state): State<AppState>,
    Json(request): Json<crate::ReplacePipelineRequest>,
) -> impl IntoResponse {
    let rollout = match state
        .controller
        .replace_pipeline(&pipeline_group_id, &pipeline_id, request)
    {
        Ok(rollout) => rollout,
        Err(crate::ControlPlaneError::GroupNotFound) => {
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(crate::ControlPlaneError::RolloutConflict) => {
            return StatusCode::CONFLICT.into_response();
        }
        Err(crate::ControlPlaneError::InvalidRequest { message }) => {
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(message)).into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if !params.wait {
        let status = if rollout_is_terminal(&rollout.state) {
            if rollout_is_success(&rollout.state) {
                StatusCode::OK
            } else {
                StatusCode::CONFLICT
            }
        } else {
            StatusCode::ACCEPTED
        };
        return (status, Json(rollout)).into_response();
    }

    let deadline = Instant::now() + Duration::from_secs(params.timeout_secs);
    loop {
        match state
            .controller
            .rollout_status(&pipeline_group_id, &pipeline_id, &rollout.rollout_id)
        {
            Ok(Some(current)) if rollout_is_terminal(&current.state) => {
                let status = if rollout_is_success(&current.state) {
                    StatusCode::OK
                } else {
                    StatusCode::CONFLICT
                };
                return (status, Json(current)).into_response();
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(crate::ControlPlaneError::RolloutNotFound) => {
                return StatusCode::NOT_FOUND.into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }

        if Instant::now() >= deadline {
            return StatusCode::GATEWAY_TIMEOUT.into_response();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn show_rollout(
    Path((pipeline_group_id, pipeline_id, rollout_id)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Result<Json<crate::PipelineRolloutStatus>, StatusCode> {
    match state
        .controller
        .rollout_status(&pipeline_group_id, &pipeline_id, &rollout_id)
    {
        Ok(Some(status)) => Ok(Json(status)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(crate::ControlPlaneError::RolloutNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn show_shutdown(
    Path((pipeline_group_id, pipeline_id, shutdown_id)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Result<Json<crate::PipelineShutdownStatus>, StatusCode> {
    match state
        .controller
        .shutdown_status(&pipeline_group_id, &pipeline_id, &shutdown_id)
    {
        Ok(Some(status)) => Ok(Json(status)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(crate::ControlPlaneError::ShutdownNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn shutdown_pipeline(
    Path((pipeline_group_id, pipeline_id)): Path<(String, String)>,
    Query(params): Query<WaitParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    otel_info!(
        "pipeline.shutdown.requested",
        pipeline_group_id = pipeline_group_id.as_str(),
        pipeline_id = pipeline_id.as_str(),
        wait = params.wait,
        timeout_secs = params.timeout_secs
    );

    match state
        .controller
        .shutdown_pipeline(&pipeline_group_id, &pipeline_id, params.timeout_secs)
    {
        Ok(shutdown) => {
            if !params.wait {
                return (StatusCode::ACCEPTED, Json(shutdown)).into_response();
            }

            let deadline = Instant::now() + Duration::from_secs(params.timeout_secs);
            let mut last_status = Some(shutdown);
            loop {
                let shutdown_id = last_status
                    .as_ref()
                    .expect("initial shutdown status should be present")
                    .shutdown_id
                    .clone();
                match state.controller.shutdown_status(
                    &pipeline_group_id,
                    &pipeline_id,
                    &shutdown_id,
                ) {
                    Ok(Some(current)) if shutdown_is_terminal(&current.state) => {
                        let status = if shutdown_is_success(&current.state) {
                            StatusCode::OK
                        } else {
                            StatusCode::CONFLICT
                        };
                        return (status, Json(current)).into_response();
                    }
                    Ok(Some(current)) => {
                        last_status = Some(current);
                    }
                    Ok(None) | Err(crate::ControlPlaneError::ShutdownNotFound) => {
                        return StatusCode::NOT_FOUND.into_response();
                    }
                    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }

                if Instant::now() >= deadline {
                    return match last_status {
                        Some(status) => (StatusCode::GATEWAY_TIMEOUT, Json(status)).into_response(),
                        None => StatusCode::GATEWAY_TIMEOUT.into_response(),
                    };
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        Err(
            crate::ControlPlaneError::GroupNotFound | crate::ControlPlaneError::PipelineNotFound,
        ) => StatusCode::NOT_FOUND.into_response(),
        Err(crate::ControlPlaneError::RolloutConflict) => StatusCode::CONFLICT.into_response(),
        Err(crate::ControlPlaneError::InvalidRequest { message }) => {
            (StatusCode::BAD_REQUEST, Json(message)).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn show_status(
    Path((pipeline_group_id, pipeline_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Option<PipelineStatus>>, StatusCode> {
    let key = PipelineKey::new(pipeline_group_id.into(), pipeline_id.into());
    let pipeline_status = state.observed_state_store.pipeline_status(&key);
    Ok(Json(pipeline_status))
}

/// Used by the kubelet livenessProbe to decide whether to restart the container.
///
/// Typical use cases:
/// - Detect deadlocks or stuck event loops.
/// - Force a restart when the app can't recover by itself.
/// - Should be cheap and internal (not dependent on external systems).
///
/// ToDo Implement heartbeat checks.
async fn liveness(
    Path((pipeline_group_id, pipeline_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let key = PipelineKey::new(pipeline_group_id.into(), pipeline_id.into());
    let liveness = state.observed_state_store.liveness(&key);
    if liveness {
        (StatusCode::OK, "OK")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "NOT OK")
    }
}

/// Used by the kubelet readinessProbe (and by Services) to decide whether the Pod should receive
/// traffic.
///
/// Typical use cases:
/// - Gate traffic until startup work is done (pipeline deployed and running).
/// - Temporarily remove the Pod from load balancing when it can't serve correctly.
/// - Can check key dependencies, but avoid making it too fragile.
async fn readiness(
    Path((pipeline_group_id, pipeline_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let key = PipelineKey::new(pipeline_group_id.into(), pipeline_id.into());
    let readiness = state.observed_state_store.readiness(&key);
    if readiness {
        (StatusCode::OK, "OK")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "NOT OK")
    }
}
