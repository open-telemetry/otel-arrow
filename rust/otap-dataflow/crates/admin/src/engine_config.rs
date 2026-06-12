// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine configuration endpoints.
//!
//! - GET `/api/v1/config` - get the controller-owned engine configuration
//! - POST `/api/v1/config/reconcile` - reconcile the controller to a desired
//!   full engine configuration

use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use otap_df_telemetry::otel_info;

/// All routes for engine configuration.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/config", get(show_config))
        .route("/config/reconcile", post(reconcile_config))
}

/// Converts a typed control-plane rejection into the shared HTTP error shape.
fn operation_error_response(status: StatusCode, error: crate::ControlPlaneError) -> Response {
    (status, Json(error.as_operation_error())).into_response()
}

/// Returns the full controller-owned engine configuration.
pub async fn show_config(State(state): State<AppState>) -> impl IntoResponse {
    match state.controller.engine_config_snapshot() {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(error) => operation_error_response(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

/// Reconciles the running controller state to a desired full engine config.
pub async fn reconcile_config(
    State(state): State<AppState>,
    Json(request): Json<crate::EngineConfigReconcileRequest>,
) -> impl IntoResponse {
    otel_info!("config.reconcile.requested");

    match state.controller.reconcile_engine_config(request) {
        Ok(status) => match status.state {
            crate::EngineConfigReconcileState::Succeeded => {
                (StatusCode::OK, Json(status)).into_response()
            }
            crate::EngineConfigReconcileState::Failed => {
                (StatusCode::CONFLICT, Json(status)).into_response()
            }
            crate::EngineConfigReconcileState::Pending
            | crate::EngineConfigReconcileState::Running => {
                (StatusCode::ACCEPTED, Json(status)).into_response()
            }
        },
        Err(error @ crate::ControlPlaneError::RolloutConflict) => {
            operation_error_response(StatusCode::CONFLICT, error)
        }
        Err(crate::ControlPlaneError::InvalidRequest { message }) => operation_error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            crate::ControlPlaneError::InvalidRequest { message },
        ),
        Err(other) => operation_error_response(StatusCode::INTERNAL_SERVER_ERROR, other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ControlPlane, ControlPlaneError, EngineConfigReconcileState, EngineConfigReconcileStatus,
        GroupDeleteStatus, PipelineDeleteStatus, PipelineDetails, ReconfigureRequest,
        RolloutStatus, ShutdownStatus,
    };
    use axum::body::to_bytes;
    use otap_df_admin_types::operations::{OperationError, OperationErrorKind};
    use otap_df_config::engine::OtelDataflowSpec;
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_engine::memory_limiter::MemoryPressureState;
    use otap_df_state::store::ObservedStateStore;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::sync::Arc;

    #[derive(Clone)]
    struct StubControlPlane {
        snapshot_result: Result<OtelDataflowSpec, ControlPlaneError>,
        reconcile_result: Result<EngineConfigReconcileStatus, ControlPlaneError>,
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

        fn engine_config_snapshot(&self) -> Result<OtelDataflowSpec, ControlPlaneError> {
            self.snapshot_result.clone()
        }

        fn reconcile_engine_config(
            &self,
            _request: crate::EngineConfigReconcileRequest,
        ) -> Result<EngineConfigReconcileStatus, ControlPlaneError> {
            self.reconcile_result.clone()
        }

        fn delete_pipeline(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _timeout_secs: u64,
        ) -> Result<PipelineDeleteStatus, ControlPlaneError> {
            Err(ControlPlaneError::PipelineNotFound)
        }

        fn delete_group(
            &self,
            _pipeline_group_id: &str,
            _timeout_secs: u64,
        ) -> Result<GroupDeleteStatus, ControlPlaneError> {
            Err(ControlPlaneError::GroupNotFound)
        }
    }

    fn empty_engine_config() -> OtelDataflowSpec {
        OtelDataflowSpec::from_yaml("version: otel_dataflow/v1\n")
            .expect("empty engine config should parse")
    }

    fn reconcile_status(state: EngineConfigReconcileState) -> EngineConfigReconcileStatus {
        let mut status = EngineConfigReconcileStatus::new(
            "reconcile-1".to_owned(),
            state,
            None,
            "2026-01-01T00:00:00Z".to_owned(),
        );
        if state == EngineConfigReconcileState::Failed {
            status.failure_reason = Some("reconcile failed".to_owned());
        }
        status
    }

    fn reconcile_request() -> crate::EngineConfigReconcileRequest {
        crate::EngineConfigReconcileRequest {
            config: empty_engine_config(),
            step_timeout_secs: 30,
            drain_timeout_secs: 30,
            delete_timeout_secs: 30,
            delete_missing: true,
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

    fn stub(
        snapshot_result: Result<OtelDataflowSpec, ControlPlaneError>,
        reconcile_result: Result<EngineConfigReconcileStatus, ControlPlaneError>,
    ) -> Arc<dyn ControlPlane> {
        Arc::new(StubControlPlane {
            snapshot_result,
            reconcile_result,
        })
    }

    /// Scenario: a caller asks for the committed controller-owned engine
    /// configuration.
    /// Guarantees: the handler returns the config snapshot directly.
    #[tokio::test]
    async fn show_config_returns_engine_config_snapshot() {
        let config = empty_engine_config();
        let response = show_config(State(test_app_state(stub(
            Ok(config.clone()),
            Ok(reconcile_status(EngineConfigReconcileState::Succeeded)),
        ))))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let decoded: OtelDataflowSpec =
            serde_json::from_slice(&body).expect("config body should deserialize");
        assert_eq!(decoded, config);
    }

    /// Scenario: the active control plane does not support full config
    /// snapshots.
    /// Guarantees: the handler returns a typed internal operation error.
    #[tokio::test]
    async fn show_config_returns_operation_error_for_unsupported_control_plane() {
        let response = show_config(State(test_app_state(stub(
            Err(ControlPlaneError::Internal {
                message: "engine config snapshots are not supported".to_owned(),
            }),
            Ok(reconcile_status(EngineConfigReconcileState::Succeeded)),
        ))))
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let error: OperationError =
            serde_json::from_slice(&body).expect("error body should deserialize");
        assert_eq!(error.kind, OperationErrorKind::Internal);
    }

    /// Scenario: full-engine reconciliation succeeds.
    /// Guarantees: the handler returns HTTP 200 with the reconciliation status.
    #[tokio::test]
    async fn reconcile_config_returns_success_status() {
        let response = reconcile_config(
            State(test_app_state(stub(
                Ok(empty_engine_config()),
                Ok(reconcile_status(EngineConfigReconcileState::Succeeded)),
            ))),
            Json(reconcile_request()),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let status: EngineConfigReconcileStatus =
            serde_json::from_slice(&body).expect("status body should deserialize");
        assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    }

    /// Scenario: full-engine reconciliation is accepted but reaches a failed
    /// terminal state.
    /// Guarantees: the handler returns HTTP 409 with the terminal status body.
    #[tokio::test]
    async fn reconcile_config_returns_conflict_for_failed_status() {
        let response = reconcile_config(
            State(test_app_state(stub(
                Ok(empty_engine_config()),
                Ok(reconcile_status(EngineConfigReconcileState::Failed)),
            ))),
            Json(reconcile_request()),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        let status: EngineConfigReconcileStatus =
            serde_json::from_slice(&body).expect("status body should deserialize");
        assert_eq!(status.state, EngineConfigReconcileState::Failed);
        assert_eq!(status.failure_reason.as_deref(), Some("reconcile failed"));
    }

    /// Scenario: the control plane rejects reconciliation as invalid before
    /// starting work.
    /// Guarantees: the handler returns HTTP 422 with a typed operation error.
    #[tokio::test]
    async fn reconcile_config_returns_operation_error_for_invalid_request() {
        let response = reconcile_config(
            State(test_app_state(stub(
                Ok(empty_engine_config()),
                Err(ControlPlaneError::InvalidRequest {
                    message: "desired config is invalid".to_owned(),
                }),
            ))),
            Json(reconcile_request()),
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
        assert_eq!(error.message.as_deref(), Some("desired config is invalid"));
    }

    /// Scenario: reconciliation conflicts with another active control-plane
    /// operation.
    /// Guarantees: the handler returns HTTP 409 with a typed conflict error.
    #[tokio::test]
    async fn reconcile_config_returns_operation_error_for_conflict() {
        let response = reconcile_config(
            State(test_app_state(stub(
                Ok(empty_engine_config()),
                Err(ControlPlaneError::RolloutConflict),
            ))),
            Json(reconcile_request()),
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
    }
}
