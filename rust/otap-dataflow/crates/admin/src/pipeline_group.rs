// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline group endpoints.
//!
//! - GET `/api/v1/groups/{id}` - get the committed configuration of one group
//! - POST `/api/v1/groups/{id}` - create an empty group
//! - DELETE `/api/v1/groups/{id}` - gracefully drain and delete one group
//! - GET `/api/v1/groups/:id/pipelines` - list active pipelines and their status (ToDo)
//! - POST `/api/v1/groups/shutdown` - shutdown all pipelines in all groups
//!   - Query parameters:
//!     - `wait` (bool, default: false) - if true, block until all pipelines have stopped
//!     - `timeout_secs` (u64, default: 60) - maximum seconds to wait when `wait=true`
//!
//!   Example (fire-and-forget):
//!   ```sh
//!   curl -X POST http://localhost:8080/api/v1/groups/shutdown
//!   ```
//!   Example (wait for graceful shutdown with 30s timeout):
//!   ```sh
//!   curl -X POST "http://localhost:8080/api/v1/groups/shutdown?wait=true&timeout_secs=30"
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
use crate::convert::json_shape;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use otap_df_admin_types::{
    groups::{ShutdownResponse, ShutdownStatus, Status as GroupsStatus},
    operations::{DeleteOptions, OperationOptions},
};
use otap_df_config::pipeline_group::PipelineGroupConfig;
use otap_df_telemetry::otel_info;
use std::time::{Duration, Instant};

/// All the routes for pipeline groups.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        // Returns committed configuration for one pipeline group.
        .route(
            "/groups/{pipeline_group_id}",
            get(show_group).post(create_group).delete(delete_group),
        )
        // Returns a summary of all pipelines and their statuses.
        .route("/groups/status", get(show_status))
        // Shutdown all pipelines in all groups.
        .route("/groups/shutdown", post(shutdown_all_pipelines))
    // ToDo Global liveness and readiness probes.
}

/// Converts a typed control-plane rejection into the shared HTTP error shape.
fn operation_error_response(status: StatusCode, error: crate::ControlPlaneError) -> Response {
    (status, Json(error.as_operation_error())).into_response()
}

/// Returns the committed configuration for one pipeline group.
pub async fn show_group(
    Path(pipeline_group_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<PipelineGroupConfig>, StatusCode> {
    match state.controller.group_details(&pipeline_group_id) {
        Ok(Some(group)) => Ok(Json(group)),
        Ok(None) | Err(crate::ControlPlaneError::GroupNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Creates one empty pipeline group.
pub async fn create_group(
    Path(pipeline_group_id): Path<String>,
    State(state): State<AppState>,
    Json(group): Json<PipelineGroupConfig>,
) -> impl IntoResponse {
    otel_info!(
        "group.create.requested",
        pipeline_group_id = pipeline_group_id.as_str()
    );

    match state.controller.create_group(&pipeline_group_id, group) {
        Ok(group) => (StatusCode::CREATED, Json(group)).into_response(),
        Err(error @ crate::ControlPlaneError::GroupAlreadyExists)
        | Err(error @ crate::ControlPlaneError::RolloutConflict) => {
            operation_error_response(StatusCode::CONFLICT, error)
        }
        Err(crate::ControlPlaneError::InvalidRequest { message }) => operation_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            crate::ControlPlaneError::InvalidRequest { message },
        ),
        Err(other) => operation_error_response(StatusCode::INTERNAL_SERVER_ERROR, other),
    }
}

/// Gracefully drains and removes one pipeline group from committed state.
pub async fn delete_group(
    Path(pipeline_group_id): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<DeleteOptions>,
) -> impl IntoResponse {
    otel_info!(
        "group.delete.requested",
        pipeline_group_id = pipeline_group_id.as_str(),
        timeout_secs = params.timeout_secs
    );

    match state
        .controller
        .delete_group(&pipeline_group_id, params.timeout_secs)
    {
        Ok(status) if status.state == "succeeded" => (StatusCode::OK, Json(status)).into_response(),
        Ok(status) => (StatusCode::CONFLICT, Json(status)).into_response(),
        Err(error @ crate::ControlPlaneError::GroupNotFound) => {
            operation_error_response(StatusCode::NOT_FOUND, error)
        }
        Err(crate::ControlPlaneError::RolloutConflict) => operation_error_response(
            StatusCode::CONFLICT,
            crate::ControlPlaneError::RolloutConflict,
        ),
        Err(crate::ControlPlaneError::InvalidRequest { message }) => operation_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            crate::ControlPlaneError::InvalidRequest { message },
        ),
        Err(other) => operation_error_response(StatusCode::INTERNAL_SERVER_ERROR, other),
    }
}

pub async fn show_status(State(state): State<AppState>) -> Result<Json<GroupsStatus>, StatusCode> {
    Ok(Json(GroupsStatus {
        generated_at: Utc::now().to_rfc3339(),
        pipelines: json_shape(&state.observed_state_store.snapshot()),
    }))
}

async fn shutdown_all_pipelines(
    State(state): State<AppState>,
    Query(params): Query<OperationOptions>,
) -> impl IntoResponse {
    let start_time = Instant::now();

    otel_info!(
        "shutdown.requested",
        wait = params.wait,
        timeout_secs = params.timeout_secs
    );

    if let Err(err) = state.controller.shutdown_all(params.timeout_secs) {
        otel_info!("shutdown.failed", error = ?err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ShutdownResponse {
                status: ShutdownStatus::Failed,
                errors: Some(vec![format!("{err:?}")]),
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
                status: ShutdownStatus::Accepted,
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
                    status: ShutdownStatus::Timeout,
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
                    status: ShutdownStatus::Completed,
                    errors: None,
                    duration_ms: Some(start_time.elapsed().as_millis() as u64),
                }),
            );
        }

        // Wait before polling again
        tokio::time::sleep(poll_interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ControlPlane, ControlPlaneError, GroupDeleteStatus, PipelineDetails, ReconfigureRequest,
        RolloutStatus, ShutdownStatus,
    };
    use axum::body::to_bytes;
    use otap_df_admin_types::operations::{OperationError, OperationErrorKind};
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_engine::memory_limiter::MemoryPressureState;
    use otap_df_state::store::ObservedStateStore;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::sync::Arc;

    #[derive(Clone)]
    struct StubControlPlane {
        group_details_result: Result<Option<PipelineGroupConfig>, ControlPlaneError>,
        create_group_result: Result<PipelineGroupConfig, ControlPlaneError>,
        delete_group_result: Result<GroupDeleteStatus, ControlPlaneError>,
    }

    impl ControlPlane for StubControlPlane {
        fn shutdown_all(&self, _timeout_secs: u64) -> Result<(), ControlPlaneError> {
            Ok(())
        }

        fn shutdown_pipeline(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _timeout_secs: u64,
        ) -> Result<ShutdownStatus, ControlPlaneError> {
            Err(ControlPlaneError::PipelineNotFound)
        }

        fn reconfigure_pipeline(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _request: ReconfigureRequest,
        ) -> Result<RolloutStatus, ControlPlaneError> {
            Err(ControlPlaneError::PipelineNotFound)
        }

        fn pipeline_details(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
        ) -> Result<Option<PipelineDetails>, ControlPlaneError> {
            Ok(None)
        }

        fn rollout_status(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _rollout_id: &str,
        ) -> Result<Option<RolloutStatus>, ControlPlaneError> {
            Ok(None)
        }

        fn shutdown_status(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _shutdown_id: &str,
        ) -> Result<Option<ShutdownStatus>, ControlPlaneError> {
            Ok(None)
        }

        fn group_details(
            &self,
            _pipeline_group_id: &str,
        ) -> Result<Option<PipelineGroupConfig>, ControlPlaneError> {
            self.group_details_result.clone()
        }

        fn create_group(
            &self,
            _pipeline_group_id: &str,
            _group: PipelineGroupConfig,
        ) -> Result<PipelineGroupConfig, ControlPlaneError> {
            self.create_group_result.clone()
        }

        fn delete_group(
            &self,
            _pipeline_group_id: &str,
            _timeout_secs: u64,
        ) -> Result<GroupDeleteStatus, ControlPlaneError> {
            self.delete_group_result.clone()
        }
    }

    fn test_app_state(controller: Arc<dyn ControlPlane>) -> AppState {
        let metrics_registry = TelemetryRegistryHandle::new();
        let observed_state_store =
            ObservedStateStore::new(&ObservedStateSettings::default(), metrics_registry.clone());

        AppState {
            observed_state_store: observed_state_store.handle(),
            metrics_registry,
            controller,
            log_tap: None,
            memory_pressure_state: MemoryPressureState::default(),
            target_info: Arc::from(""),
        }
    }

    fn delete_status(state: &str) -> GroupDeleteStatus {
        GroupDeleteStatus {
            pipeline_group_id: "default".to_string().into(),
            state: state.to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:01Z".to_string(),
            pipelines: Vec::new(),
            failure_reason: (state != "succeeded").then(|| "group delete failed".to_string()),
        }
    }

    fn stub(
        group_details_result: Result<Option<PipelineGroupConfig>, ControlPlaneError>,
        create_group_result: Result<PipelineGroupConfig, ControlPlaneError>,
        delete_group_result: Result<GroupDeleteStatus, ControlPlaneError>,
    ) -> Arc<dyn ControlPlane> {
        Arc::new(StubControlPlane {
            group_details_result,
            create_group_result,
            delete_group_result,
        })
    }

    /// Scenario: a caller asks for the committed configuration of an existing
    /// pipeline group.
    /// Guarantees: the handler returns the group configuration directly.
    #[tokio::test]
    async fn show_group_returns_committed_group_config() {
        let group = PipelineGroupConfig::new();
        let response = show_group(
            Path("default".to_string()),
            State(test_app_state(stub(
                Ok(Some(group.clone())),
                Ok(group.clone()),
                Ok(delete_status("succeeded")),
            ))),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let decoded: PipelineGroupConfig =
            serde_json::from_slice(&body).expect("group body should deserialize");
        assert_eq!(decoded, group);
    }

    /// Scenario: a caller creates an empty pipeline group.
    /// Guarantees: the handler returns HTTP 201 with the committed group
    /// configuration.
    #[tokio::test]
    async fn create_group_returns_created_group_config() {
        let group = PipelineGroupConfig::new();
        let response = create_group(
            Path("default".to_string()),
            State(test_app_state(stub(
                Ok(None),
                Ok(group.clone()),
                Ok(delete_status("succeeded")),
            ))),
            Json(group.clone()),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let decoded: PipelineGroupConfig =
            serde_json::from_slice(&body).expect("group body should deserialize");
        assert_eq!(decoded, group);
    }

    /// Scenario: the control plane rejects group creation as invalid.
    /// Guarantees: the handler returns a typed operation error with HTTP 422.
    #[tokio::test]
    async fn create_group_returns_operation_error_for_invalid_request() {
        let response = create_group(
            Path("default".to_string()),
            State(test_app_state(stub(
                Ok(None),
                Err(ControlPlaneError::InvalidRequest {
                    message: "pipeline group creation only supports empty groups".to_string(),
                }),
                Ok(delete_status("succeeded")),
            ))),
            Json(PipelineGroupConfig::new()),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let error: OperationError =
            serde_json::from_slice(&body).expect("error body should deserialize");
        assert_eq!(error.kind, OperationErrorKind::InvalidRequest);
        assert_eq!(
            error.message.as_deref(),
            Some("pipeline group creation only supports empty groups")
        );
    }

    /// Scenario: group deletion completes successfully.
    /// Guarantees: the handler returns HTTP 200 with the terminal delete
    /// status body.
    #[tokio::test]
    async fn delete_group_returns_terminal_success_status() {
        let group = PipelineGroupConfig::new();
        let response = delete_group(
            Path("default".to_string()),
            State(test_app_state(stub(
                Ok(Some(group.clone())),
                Ok(group),
                Ok(delete_status("succeeded")),
            ))),
            Query(DeleteOptions { timeout_secs: 30 }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let status: GroupDeleteStatus =
            serde_json::from_slice(&body).expect("delete body should deserialize");
        assert_eq!(status.pipeline_group_id.as_ref(), "default");
        assert_eq!(status.state, "succeeded");
    }

    /// Scenario: group deletion targets a missing group.
    /// Guarantees: the handler returns HTTP 404 with the typed operation error
    /// shape used by SDK callers.
    #[tokio::test]
    async fn delete_group_returns_not_found_for_missing_group() {
        let group = PipelineGroupConfig::new();
        let response = delete_group(
            Path("missing".to_string()),
            State(test_app_state(stub(
                Ok(None),
                Ok(group),
                Err(ControlPlaneError::GroupNotFound),
            ))),
            Query(DeleteOptions { timeout_secs: 30 }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let error: OperationError =
            serde_json::from_slice(&body).expect("error body should deserialize");
        assert_eq!(error.kind, OperationErrorKind::GroupNotFound);
    }

    /// Scenario: group deletion reaches a terminal failed state.
    /// Guarantees: the handler returns HTTP 409 with the delete status body.
    #[tokio::test]
    async fn delete_group_returns_conflict_with_failed_delete_status() {
        let group = PipelineGroupConfig::new();
        let response = delete_group(
            Path("default".to_string()),
            State(test_app_state(stub(
                Ok(Some(group.clone())),
                Ok(group),
                Ok(delete_status("failed")),
            ))),
            Query(DeleteOptions { timeout_secs: 30 }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let status: GroupDeleteStatus =
            serde_json::from_slice(&body).expect("delete body should deserialize");
        assert_eq!(status.state, "failed");
        assert_eq!(
            status.failure_reason.as_deref(),
            Some("group delete failed")
        );
    }
}
