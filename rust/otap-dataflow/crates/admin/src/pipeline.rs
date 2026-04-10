// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline endpoints.
//!
//! - GET `/api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}`
//!   Get the configuration of the specified pipeline.
//! - GET `/api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/status`
//!   Get the status of the specified pipeline.
//! - GET `/api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/rollouts/{rollout_id}`
//!   Get the status of a specific rollout job for the logical pipeline.
//!   Older rollout ids may return `404 Not Found` after bounded in-memory
//!   retention evicts terminal history.
//! - GET `/api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdowns/{shutdown_id}`
//!   Get the status of a specific shutdown job for the logical pipeline.
//!   Older shutdown ids may return `404 Not Found` after bounded in-memory
//!   retention evicts terminal history.
//! - PUT `/api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}`
//!   Create or replace a pipeline and return a rollout job status snapshot.
//! - POST `/api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdown`
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
use crate::convert::json_shape;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use otap_df_admin_types::pipelines::{PipelineRolloutState, Status as ApiPipelineStatus};
use otap_df_config::PipelineKey;
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

/// Converts a typed control-plane rejection into the shared HTTP error shape.
fn operation_error_response(status: StatusCode, error: crate::ControlPlaneError) -> Response {
    (status, Json(error.as_operation_error())).into_response()
}

/// Returns whether a rollout status is already in a terminal state.
fn rollout_is_terminal(state: PipelineRolloutState) -> bool {
    matches!(
        state,
        PipelineRolloutState::Succeeded
            | PipelineRolloutState::Failed
            | PipelineRolloutState::RollbackFailed
    )
}

/// Returns whether a terminal rollout finished successfully.
fn rollout_is_success(state: PipelineRolloutState) -> bool {
    state == PipelineRolloutState::Succeeded
}

/// Returns whether a shutdown status string represents a terminal state.
fn shutdown_is_terminal(state: &str) -> bool {
    matches!(state, "succeeded" | "failed")
}

/// Returns whether a terminal shutdown finished successfully.
fn shutdown_is_success(state: &str) -> bool {
    state == "succeeded"
}

/// Returns committed configuration details for one logical pipeline.
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

/// Starts a pipeline reconfiguration and optionally waits for its terminal result.
pub async fn put_pipeline(
    Path((pipeline_group_id, pipeline_id)): Path<(String, String)>,
    Query(params): Query<WaitParams>,
    State(state): State<AppState>,
    Json(request): Json<crate::ReconfigureRequest>,
) -> impl IntoResponse {
    let rollout =
        match state
            .controller
            .reconfigure_pipeline(&pipeline_group_id, &pipeline_id, request)
        {
            Ok(rollout) => rollout,
            Err(crate::ControlPlaneError::GroupNotFound) => {
                return operation_error_response(
                    StatusCode::NOT_FOUND,
                    crate::ControlPlaneError::GroupNotFound,
                );
            }
            Err(crate::ControlPlaneError::RolloutConflict) => {
                return operation_error_response(
                    StatusCode::CONFLICT,
                    crate::ControlPlaneError::RolloutConflict,
                );
            }
            Err(crate::ControlPlaneError::InvalidRequest { message }) => {
                return operation_error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    crate::ControlPlaneError::InvalidRequest { message },
                );
            }
            Err(other) => {
                return operation_error_response(StatusCode::INTERNAL_SERVER_ERROR, other);
            }
        };

    if !params.wait {
        let status = if rollout_is_terminal(rollout.state) {
            if rollout_is_success(rollout.state) {
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
    let mut last_status = Some(rollout);
    loop {
        let Some(rollout_id) = last_status.as_ref().map(|status| status.rollout_id.clone()) else {
            return operation_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                crate::ControlPlaneError::Internal {
                    message: "initial rollout status disappeared while waiting".to_string(),
                },
            );
        };
        match state
            .controller
            .rollout_status(&pipeline_group_id, &pipeline_id, &rollout_id)
        {
            Ok(Some(current)) if rollout_is_terminal(current.state) => {
                let status = if rollout_is_success(current.state) {
                    StatusCode::OK
                } else {
                    StatusCode::CONFLICT
                };
                return (status, Json(current)).into_response();
            }
            Ok(Some(current)) => {
                last_status = Some(current);
            }
            Ok(None) | Err(crate::ControlPlaneError::RolloutNotFound) => {
                return operation_error_response(
                    StatusCode::NOT_FOUND,
                    crate::ControlPlaneError::RolloutNotFound,
                );
            }
            Err(other) => {
                return operation_error_response(StatusCode::INTERNAL_SERVER_ERROR, other);
            }
        }

        if Instant::now() >= deadline {
            return match last_status {
                Some(status) => (StatusCode::GATEWAY_TIMEOUT, Json(status)).into_response(),
                None => operation_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    crate::ControlPlaneError::Internal {
                        message: "rollout status disappeared before timeout response".to_string(),
                    },
                ),
            };
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Returns the latest snapshot for one rollout operation id.
pub async fn show_rollout(
    Path((pipeline_group_id, pipeline_id, rollout_id)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Result<Json<crate::RolloutStatus>, StatusCode> {
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

/// Returns the latest snapshot for one shutdown operation id.
pub async fn show_shutdown(
    Path((pipeline_group_id, pipeline_id, shutdown_id)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Result<Json<crate::ShutdownStatus>, StatusCode> {
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

/// Starts a tracked shutdown for one logical pipeline and optionally waits.
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
                let Some(shutdown_id) = last_status
                    .as_ref()
                    .map(|status| status.shutdown_id.clone())
                else {
                    return operation_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        crate::ControlPlaneError::Internal {
                            message: "initial shutdown status disappeared while waiting"
                                .to_string(),
                        },
                    );
                };
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
                        return operation_error_response(
                            StatusCode::NOT_FOUND,
                            crate::ControlPlaneError::ShutdownNotFound,
                        );
                    }
                    Err(other) => {
                        return operation_error_response(StatusCode::INTERNAL_SERVER_ERROR, other);
                    }
                }

                if Instant::now() >= deadline {
                    return match last_status {
                        Some(status) => (StatusCode::GATEWAY_TIMEOUT, Json(status)).into_response(),
                        None => operation_error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            crate::ControlPlaneError::Internal {
                                message: "shutdown status disappeared before timeout response"
                                    .to_string(),
                            },
                        ),
                    };
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        Err(error @ crate::ControlPlaneError::GroupNotFound)
        | Err(error @ crate::ControlPlaneError::PipelineNotFound) => {
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

/// Returns aggregated runtime status for one logical pipeline.
pub async fn show_status(
    Path((pipeline_group_id, pipeline_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<Option<ApiPipelineStatus>>, StatusCode> {
    let key = PipelineKey::new(pipeline_group_id.into(), pipeline_id.into());
    let pipeline_status = state.observed_state_store.pipeline_status(&key);
    Ok(Json(
        pipeline_status
            .as_ref()
            .map(json_shape::<_, ApiPipelineStatus>),
    ))
}

/// Used by the kubelet livenessProbe to decide whether to restart the container.
///
/// Typical use cases:
/// - Detect deadlocks or stuck event loops.
/// - Force a restart when the app can't recover by itself.
/// - Should be cheap and internal (not dependent on external systems).
///
/// ToDo Implement heartbeat checks.
///
/// Serves the liveness probe for one logical pipeline.
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
///
/// Serves the readiness probe for one logical pipeline.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ControlPlane, ControlPlaneError, PipelineDetails, RolloutStatus, ShutdownStatus};
    use axum::body::to_bytes;
    use otap_df_admin_types::operations::{OperationError, OperationErrorKind};
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_engine::memory_limiter::MemoryPressureState;
    use otap_df_state::store::ObservedStateStore;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;
    use std::sync::Arc;

    #[derive(Clone)]
    struct StubControlPlane {
        replace_result: Result<RolloutStatus, ControlPlaneError>,
        rollout_status_result: Result<Option<RolloutStatus>, ControlPlaneError>,
        shutdown_result: Result<ShutdownStatus, ControlPlaneError>,
        shutdown_status_result: Result<Option<ShutdownStatus>, ControlPlaneError>,
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
            self.shutdown_result.clone()
        }

        fn reconfigure_pipeline(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _request: crate::ReconfigureRequest,
        ) -> Result<RolloutStatus, ControlPlaneError> {
            self.replace_result.clone()
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
            self.rollout_status_result.clone()
        }

        fn shutdown_status(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _shutdown_id: &str,
        ) -> Result<Option<ShutdownStatus>, ControlPlaneError> {
            self.shutdown_status_result.clone()
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
        }
    }

    fn request() -> crate::ReconfigureRequest {
        crate::ReconfigureRequest {
            pipeline: serde_json::from_value(json!({
                "type": "otap",
                "nodes": {
                    "recv": {
                        "type": "receiver:fake",
                        "config": {}
                    }
                }
            }))
            .expect("fixture pipeline should deserialize"),
            step_timeout_secs: 60,
            drain_timeout_secs: 60,
        }
    }

    fn rollout_status(state: PipelineRolloutState) -> RolloutStatus {
        serde_json::from_value(json!({
            "rolloutId": "rollout-1",
            "pipelineGroupId": "default",
            "pipelineId": "main",
            "action": "replace",
            "state": state,
            "targetGeneration": 1,
            "previousGeneration": 0,
            "startedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:01Z",
            "cores": []
        }))
        .expect("fixture rollout status should deserialize")
    }

    fn shutdown_status(state: &str) -> ShutdownStatus {
        serde_json::from_value(json!({
            "shutdownId": "shutdown-1",
            "pipelineGroupId": "default",
            "pipelineId": "main",
            "state": state,
            "startedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:01Z",
            "cores": []
        }))
        .expect("fixture shutdown status should deserialize")
    }

    /// Scenario: the control plane rejects a pipeline reconfigure request
    /// before rollout work starts.
    /// Guarantees: the admin handler converts that rejection into a structured
    /// operation-error body with the expected HTTP status.
    #[tokio::test]
    async fn put_pipeline_returns_operation_error_body_on_invalid_request() {
        let response = put_pipeline(
            Path(("default".to_string(), "main".to_string())),
            Query(WaitParams {
                wait: false,
                timeout_secs: 60,
            }),
            State(test_app_state(Arc::new(StubControlPlane {
                replace_result: Err(ControlPlaneError::InvalidRequest {
                    message: "invalid candidate".to_string(),
                }),
                rollout_status_result: Ok(None),
                shutdown_result: Ok(shutdown_status("succeeded")),
                shutdown_status_result: Ok(None),
            }))),
            Json(request()),
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
        assert_eq!(error.message.as_deref(), Some("invalid candidate"));
    }

    /// Scenario: a waited pipeline reconfigure request times out and the
    /// control plane can still report the latest rollout snapshot.
    /// Guarantees: the admin handler returns HTTP 504 with that rollout status
    /// body instead of dropping the operation context.
    #[tokio::test]
    async fn put_pipeline_timeout_returns_latest_rollout_status_snapshot() {
        let response = put_pipeline(
            Path(("default".to_string(), "main".to_string())),
            Query(WaitParams {
                wait: true,
                timeout_secs: 0,
            }),
            State(test_app_state(Arc::new(StubControlPlane {
                replace_result: Ok(rollout_status(PipelineRolloutState::Running)),
                rollout_status_result: Ok(Some(rollout_status(PipelineRolloutState::Running))),
                shutdown_result: Ok(shutdown_status("succeeded")),
                shutdown_status_result: Ok(None),
            }))),
            Json(request()),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let status: RolloutStatus =
            serde_json::from_slice(&body).expect("timeout body should deserialize");
        assert_eq!(status.rollout_id, "rollout-1");
        assert_eq!(status.state, PipelineRolloutState::Running);
    }

    /// Scenario: a pipeline shutdown request collides with an active rollout
    /// for the same logical pipeline.
    /// Guarantees: the admin handler returns a typed conflict body so callers
    /// can distinguish request rejection from shutdown progress.
    #[tokio::test]
    async fn shutdown_pipeline_returns_operation_error_body_on_conflict() {
        let response = shutdown_pipeline(
            Path(("default".to_string(), "main".to_string())),
            Query(WaitParams {
                wait: false,
                timeout_secs: 60,
            }),
            State(test_app_state(Arc::new(StubControlPlane {
                replace_result: Ok(rollout_status(PipelineRolloutState::Succeeded)),
                rollout_status_result: Ok(None),
                shutdown_result: Err(ControlPlaneError::RolloutConflict),
                shutdown_status_result: Ok(None),
            }))),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let error: OperationError =
            serde_json::from_slice(&body).expect("error body should deserialize");
        assert_eq!(error.kind, OperationErrorKind::Conflict);
        assert_eq!(error.message, None);
    }

    /// Scenario: a waited pipeline shutdown request times out while the control
    /// plane still has a current shutdown snapshot.
    /// Guarantees: the admin handler responds with HTTP 504 and the latest
    /// shutdown status body for follow-up polling.
    #[tokio::test]
    async fn shutdown_pipeline_timeout_returns_latest_status_snapshot() {
        let response = shutdown_pipeline(
            Path(("default".to_string(), "main".to_string())),
            Query(WaitParams {
                wait: true,
                timeout_secs: 0,
            }),
            State(test_app_state(Arc::new(StubControlPlane {
                replace_result: Ok(rollout_status(PipelineRolloutState::Succeeded)),
                rollout_status_result: Ok(None),
                shutdown_result: Ok(shutdown_status("running")),
                shutdown_status_result: Ok(Some(shutdown_status("running"))),
            }))),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let status: ShutdownStatus =
            serde_json::from_slice(&body).expect("timeout body should deserialize");
        assert_eq!(status.shutdown_id, "shutdown-1");
        assert_eq!(status.state, "running");
    }

    /// Scenario: a caller asks for a rollout status id that is no longer
    /// available from the control plane.
    /// Guarantees: the admin handler returns HTTP 404 so evicted rollout
    /// history is observable as not found.
    #[tokio::test]
    async fn show_rollout_returns_not_found_when_status_is_missing() {
        let response = show_rollout(
            Path((
                "default".to_string(),
                "main".to_string(),
                "rollout-1".to_string(),
            )),
            State(test_app_state(Arc::new(StubControlPlane {
                replace_result: Ok(rollout_status(PipelineRolloutState::Succeeded)),
                rollout_status_result: Ok(None),
                shutdown_result: Ok(shutdown_status("succeeded")),
                shutdown_status_result: Ok(None),
            }))),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Scenario: a caller asks for a shutdown status id that is no longer
    /// available from the control plane.
    /// Guarantees: the admin handler returns HTTP 404 so evicted shutdown
    /// history is observable as not found.
    #[tokio::test]
    async fn show_shutdown_returns_not_found_when_status_is_missing() {
        let response = show_shutdown(
            Path((
                "default".to_string(),
                "main".to_string(),
                "shutdown-1".to_string(),
            )),
            State(test_app_state(Arc::new(StubControlPlane {
                replace_result: Ok(rollout_status(PipelineRolloutState::Succeeded)),
                rollout_status_result: Ok(None),
                shutdown_result: Ok(shutdown_status("succeeded")),
                shutdown_status_result: Ok(None),
            }))),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
