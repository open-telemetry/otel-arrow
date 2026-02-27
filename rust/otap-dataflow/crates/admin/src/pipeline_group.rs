// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline group endpoints.
//!
//! - GET `/pipeline-groups/:id/pipelines` - list active pipelines and their status (ToDo)
//! - POST `/pipeline-groups/shutdown` - shutdown all pipelines in all groups
//!   - Query parameters:
//!     - `wait` (bool, default: false) - if true, block until all pipelines have stopped
//!     - `timeout_secs` (u64, default: 60) - maximum seconds to wait when `wait=true`
//!
//!   Example (fire-and-forget):
//!   ```sh
//!   curl -X POST http://localhost:8080/pipeline-groups/shutdown
//!   ```
//!   Example (wait for graceful shutdown with 30s timeout):
//!   ```sh
//!   curl -X POST "http://localhost:8080/pipeline-groups/shutdown?wait=true&timeout_secs=30"
//!   ```
//!
//!   - 200 OK if `wait=true` and all pipelines stopped successfully
//!   - 202 Accepted if the stop request was accepted and is being processed (async operation)
//!   - 400 Bad Request if the pipeline is already stopped (ToDo)
//!   - 404 Not Found if the pipeline does not exist (ToDo)
//!   - 500 Internal Server Error if the stop request could not be processed
//!   - 504 Gateway Timeout if `wait=true` and pipelines did not stop within timeout
//!
//! ToDo Probably a more long term alternative -> avoid verb-y subpaths and support PATCH /.../pipelines/{pipelineId} with a body like {"status":"stopped"}. Use 409 if already stopping/stopped.
//! ToDo Other pipeline group operations will be added in the future.

use crate::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use otap_df_config::PipelineKey;
use otap_df_state::pipeline_status::PipelineStatus;
use otap_df_telemetry::otel_info;
use serde::{Deserialize, Serialize};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
}

/// Query parameters for the shutdown endpoint.
#[derive(Deserialize)]
struct ShutdownParams {
    /// If true, block until all pipelines have stopped draining.
    #[serde(default)]
    wait: bool,
    /// Maximum seconds to wait when `wait=true`. Defaults to 60.
    #[serde(default = "default_timeout_secs")]
    timeout_secs: u64,
}

const fn default_timeout_secs() -> u64 {
    60
}

pub async fn show_status(
    State(state): State<AppState>,
) -> Result<Json<PipelineGroupsStatusResponse>, StatusCode> {
    Ok(Json(PipelineGroupsStatusResponse {
        generated_at: Utc::now().to_rfc3339(),
        pipelines: state.observed_state_store.snapshot(),
    }))
}

async fn shutdown_all_pipelines(
    State(state): State<AppState>,
    Query(params): Query<ShutdownParams>,
) -> impl IntoResponse {
    let start_time = Instant::now();

    otel_info!(
        "shutdown.requested",
        wait = params.wait,
        timeout_secs = params.timeout_secs
    );

    // Send shutdown message to all pipelines
    let errors: Vec<_> = state
        .ctrl_msg_senders
        .iter()
        .filter_map(|sender| {
            // Use the timeout from params for the shutdown deadline
            let deadline = Instant::now() + Duration::from_secs(params.timeout_secs);
            sender
                .try_send_shutdown(
                    deadline,
                    "Shutdown requested via the `/pipeline-groups/shutdown` endpoint.".to_owned(),
                )
                .err()
        })
        .map(|e| e.to_string())
        .collect();

    // If there were errors sending shutdown messages, return immediately
    if !errors.is_empty() {
        otel_info!("shutdown.failed", error_count = errors.len());
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ShutdownResponse {
                status: "failed",
                errors: Some(errors),
                duration_ms: Some(start_time.elapsed().as_millis() as u64),
            }),
        );
    }

    // If wait=false, return immediately with 202 Accepted
    if !params.wait {
        otel_info!("shutdown.accepted", blocking = false);
        return (
            StatusCode::ACCEPTED,
            Json(ShutdownResponse {
                status: "accepted",
                errors: None,
                duration_ms: None,
            }),
        );
    }

    // wait=true: Poll until all pipelines reach terminal state or timeout
    otel_info!("shutdown.blocking_wait", timeout_secs = params.timeout_secs);
    let timeout = Duration::from_secs(params.timeout_secs);
    let poll_interval = Duration::from_millis(100);

    loop {
        // Check if we've exceeded the timeout
        if start_time.elapsed() > timeout {
            otel_info!(
                "shutdown.timeout",
                timeout_secs = params.timeout_secs,
                elapsed_ms = start_time.elapsed().as_millis() as u64
            );
            return (
                StatusCode::GATEWAY_TIMEOUT,
                Json(ShutdownResponse {
                    status: "timeout",
                    errors: Some(vec![format!(
                        "Shutdown did not complete within {} seconds",
                        params.timeout_secs
                    )]),
                    duration_ms: Some(start_time.elapsed().as_millis() as u64),
                }),
            );
        }

        // Check if all pipelines have terminated
        let snapshot = state.observed_state_store.snapshot();
        let all_terminated =
            !snapshot.is_empty() && snapshot.values().all(|status| status.is_terminated());

        if all_terminated {
            otel_info!(
                "shutdown.completed",
                elapsed_ms = start_time.elapsed().as_millis() as u64
            );
            return (
                StatusCode::OK,
                Json(ShutdownResponse {
                    status: "completed",
                    errors: None,
                    duration_ms: Some(start_time.elapsed().as_millis() as u64),
                }),
            );
        }

        // Wait before polling again
        tokio::time::sleep(poll_interval).await;
    }
}
