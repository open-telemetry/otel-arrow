// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline group endpoints.
//!
//! - GET `/pipeline-groups/:id/pipelines` - list active pipelines and their status (ToDo)
//! - POST `/pipeline-groups/shutdown` - shutdown all pipelines in all groups
//!   - 202 Accepted if the stop request was accepted and is being processed (async operation)
//!   - 400 Bad Request if the pipeline is already stopped (ToDo)
//!   - 404 Not Found if the pipeline does not exist (ToDo)
//!   - 500 Internal Server Error if the stop request could not be processed
//!
//! ToDo Probably a more long term alternative -> avoid verb-y subpaths and support PATCH /.../pipelines/{pipelineId} with a body like {"status":"stopped"}. Use 409 if already stopping/stopped.
//! ToDo Other pipeline group operations will be added in the future.

use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use otap_df_state::PipelineKey;
use otap_df_state::pipeline_status::PipelineStatus;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// All the routes for pipeline groups.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        // Returns a summary of all pipelines and their statuses.
        .route("/pipeline-groups/status", get(show_status))
        // Shutdown all pipelines in all groups.
        .route("/pipeline-groups/shutdown", post(shutdown_all_pipelines))
    // ToDo Global liveness and readiness probes.
}

#[derive(Serialize)]
pub struct PipelineGroupsStatusResponse {
    generated_at: String,
    pipelines: HashMap<PipelineKey, PipelineStatus>,
}

/// Response body.
#[derive(Serialize)]
struct ShutdownResponse {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<String>>,
}

pub async fn show_status(
    State(state): State<AppState>,
) -> Result<Json<PipelineGroupsStatusResponse>, StatusCode> {
    let snapshot = state.observed_state_store.snapshot();
    let response = build_status_response(snapshot);
    Ok(Json(response))
}

async fn shutdown_all_pipelines(State(state): State<AppState>) -> impl IntoResponse {
    let errors: Vec<_> = state
        .ctrl_msg_senders
        .iter()
        .filter_map(|sender| {
            // ToDo configurable shutdown timeout
            let deadline = Instant::now() + Duration::from_secs(10);
            sender
                .try_send_shutdown(deadline, "admin requested shutdown".to_owned()) // ToDo we probably need to codify reasons in the future
                .err()
        })
        .map(|e| e.to_string())
        .collect();

    if errors.is_empty() {
        (
            StatusCode::ACCEPTED,
            Json(ShutdownResponse {
                status: "accepted",
                errors: None,
            }),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ShutdownResponse {
                status: "failed",
                errors: Some(errors),
            }),
        )
    }
}

fn build_status_response(
    mut pipelines: HashMap<PipelineKey, PipelineStatus>,
) -> PipelineGroupsStatusResponse {
    // Aggregated phase are computed on-demand.
    for pipeline_status in pipelines.values_mut() {
        pipeline_status.infer_agg_phase();
    }

    PipelineGroupsStatusResponse {
        generated_at: Utc::now().to_rfc3339(),
        pipelines,
    }
}
