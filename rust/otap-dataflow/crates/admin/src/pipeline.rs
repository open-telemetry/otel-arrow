// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline endpoints.
//!
//! - GET `/pipeline-groups/:id/pipelines` - list active pipelines and their status
//! - GET `/pipeline-groups/:id/pipelines/:id` - get details of a specific pipeline
//! - POST `/pipeline-groups/:id/pipelines/:id`/stop - stop a specific pipeline
//!   - 202 Accepted if the stop request was accepted and is being processed (async operation)
//!   - 400 Bad Request if the pipeline is already stopped
//!   - 404 Not Found if the pipeline does not exist
//!
//! ToDo Alternative -> avoid verb-y subpaths and support PATCH /.../pipelines/{pipelineId} with a body like {"status":"stopped"}. Use 409 if already stopping/stopped.

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::post;
use serde::Serialize;
use crate::AppState;

/// All the routes for pipelines.
pub(crate) fn routes() -> Router<AppState> {
    Router::new().route(
        "/pipeline-groups/{group_id}/pipelines/{pipeline_id}/stop",
        post(stop_pipeline),
    )
}

/// Result of attempting to stop a pipeline.
#[derive(Debug, Clone, Copy)]
pub enum StopResult {
    /// Stop request accepted and will be processed asynchronously.
    Accepted,
    /// Pipeline is already stopped or stopping.
    AlreadyStopped,
    /// Pipeline or group does not exist.
    NotFound,
}

// Response body.
#[derive(Serialize)]
struct StopResponse {
    status: &'static str,
}

async fn stop_pipeline(
    State(_state): State<AppState>,
    Path((group_id, pipeline_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // let result = state.controller.initiate_stop(&group_id, &pipeline_id);
    let result = StopResult::NotFound; // Placeholder until controller logic is implemented.

    println!("Received stop request for pipeline '{pipeline_id}' in group '{group_id}'");

    match result {
        StopResult::Accepted => (
            StatusCode::ACCEPTED,
            Json(StopResponse { status: "accepted" }),
        ),
        StopResult::AlreadyStopped => (
            StatusCode::BAD_REQUEST,
            Json(StopResponse {
                status: "already_stopped",
            }),
        ),
        StopResult::NotFound => (
            StatusCode::NOT_FOUND,
            Json(StopResponse {
                status: "not_found",
            }),
        ),
    }
}