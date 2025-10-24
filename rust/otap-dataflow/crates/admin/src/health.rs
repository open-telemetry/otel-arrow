// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Global health and status endpoints.
//!
//! - GET `/status` - list all pipelines and their status
//! - GET `/livez` - liveness probe
//! - GET `/readyz` - readiness probe

use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use otap_df_state::PipelineKey;
use otap_df_state::pipeline_status::PipelineStatus;
use serde::Serialize;
use std::collections::HashMap;

/// All the routes for health and status endpoints.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        // Returns a summary of all pipelines and their statuses.
        .route("/status", get(show_status))
        // Returns liveness status.
        .route("/livez", get(livez))
        // Returns readiness status.
        .route("/readyz", get(readyz))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    generated_at: String,
    pipelines: HashMap<PipelineKey, PipelineStatus>,
}

pub async fn show_status(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, StatusCode> {
    Ok(Json(StatusResponse {
        generated_at: Utc::now().to_rfc3339(),
        pipelines: state.observed_state_store.snapshot(),
    }))
}

pub async fn livez() -> StatusCode {
    // ToDo
    StatusCode::OK
}

pub async fn readyz() -> StatusCode {
    // ToDo
    StatusCode::OK
}
