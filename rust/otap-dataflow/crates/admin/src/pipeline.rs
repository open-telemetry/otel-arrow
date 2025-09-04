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
use axum::{Json, Router};
use otap_df_state::PipelineKey;
use otap_df_state::store::PipelineStatus;

/// All the routes for pipelines.
pub(crate) fn routes() -> Router<AppState> {
    Router::new().route(
        "/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/status",
        axum::routing::get(show_status),
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
