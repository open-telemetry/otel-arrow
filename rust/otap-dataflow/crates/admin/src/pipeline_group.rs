// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline group endpoints.
//!
//! - GET `/pipeline-groups/:id/pipelines` - list active pipelines and their status
//! - POST `/pipeline-groups/shutdown` - shutdown all pipelines in all groups
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

/// All the routes for pipeline groups.
pub(crate) fn routes() -> Router<AppState> {
    Router::new().route(
        "/pipeline-groups/shutdown",
        post(shutdown_all_pipelines),
    )
}

/// Result of attempting to shut down one or several pipeline groups.
#[derive(Debug, Clone, Copy)]
pub enum ShutdownResult {
    /// Stop request accepted and will be processed asynchronously.
    Accepted,
    /// Pipeline is already stopped or stopping.
    AlreadyStopped,
    /// Pipeline or group does not exist.
    NotFound,
}

// Response body.
#[derive(Serialize)]
struct ShutdownResponse {
    status: &'static str,
}

async fn shutdown_all_pipelines(
    State(_state): State<AppState>
) -> impl IntoResponse {
    // let result = state.controller.initiate_stop(&group_id, &pipeline_id);
    let result = ShutdownResult::NotFound; // Placeholder until controller logic is implemented.

    match result {
        ShutdownResult::Accepted => (
            StatusCode::ACCEPTED,
            Json(ShutdownResponse { status: "accepted" }),
        ),
        ShutdownResult::AlreadyStopped => (
            StatusCode::BAD_REQUEST,
            Json(ShutdownResponse {
                status: "already_stopped",
            }),
        ),
        ShutdownResult::NotFound => (
            StatusCode::NOT_FOUND,
            Json(ShutdownResponse {
                status: "not_found",
            }),
        ),
    }
}