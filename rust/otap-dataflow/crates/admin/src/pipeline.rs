// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline endpoints.
//! Status: Not implemented.
//!
//! - GET `/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}`
//!   Get the configuration of the specified pipeline.
//! - GET `/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/status`
//!   Get the status of the specified pipeline.
//! - POST `/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdown`
//!   Shutdown a specific pipeline
//!   - 202 Accepted if the stop request was accepted and is being processed (async operation)
//!   - 400 Bad Request if the pipeline is already stopped
//!   - 404 Not Found if the pipeline does not exist
//!
//! ToDo Alternative -> avoid verb-y subpaths and support PATCH /.../pipelines/{pipelineId} with a body like {"status":"stopped"}. Use 409 if already stopping/stopped.

use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use otap_df_state::PipelineKey;
use otap_df_state::pipeline_status::PipelineStatus;

/// All the routes for pipelines.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        // Returns the status of a specific pipeline.
        .route(
            "/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/status",
            get(show_status),
        )
        // liveness and readiness probes.
        .route(
            "/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/livez",
            get(liveness),
        )
        .route(
            "/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/readyz",
            get(readiness),
        )
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
