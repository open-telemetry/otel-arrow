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
use otap_df_state::conditions::{Condition, ConditionKind, ConditionReason, ConditionStatus};
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
pub(crate) struct StatusResponse {
    generated_at: String,
    pipelines: HashMap<PipelineKey, PipelineStatus>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProbeResponse {
    probe: &'static str,
    status: &'static str,
    generated_at: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failing: Vec<PipelineConditionFailure>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PipelineConditionFailure {
    pipeline: PipelineKey,
    condition: Condition,
}

pub async fn show_status(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, StatusCode> {
    Ok(Json(StatusResponse {
        generated_at: Utc::now().to_rfc3339(),
        pipelines: state.observed_state_store.snapshot(),
    }))
}

pub(crate) async fn livez(State(state): State<AppState>) -> (StatusCode, Json<ProbeResponse>) {
    let snapshot = state.observed_state_store.snapshot();
    let failing = collect_condition_failures(
        &snapshot,
        ConditionKind::Accepted,
        skip_pipelines_without_runtimes,
        acceptance_failure,
    );

    if failing.is_empty() {
        (StatusCode::OK, Json(ProbeResponse::ok("livez")))
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProbeResponse::fail("livez", failing)),
        )
    }
}

pub(crate) async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ProbeResponse>) {
    let snapshot = state.observed_state_store.snapshot();
    let failing = collect_condition_failures(
        &snapshot,
        ConditionKind::Ready,
        skip_pipelines_without_runtimes,
        |cond| cond.status != ConditionStatus::True,
    );

    if failing.is_empty() {
        (StatusCode::OK, Json(ProbeResponse::ok("readyz")))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ProbeResponse::fail("readyz", failing)),
        )
    }
}

fn collect_condition_failures<FSkip, FFail>(
    pipelines: &HashMap<PipelineKey, PipelineStatus>,
    condition_kind: ConditionKind,
    skip: FSkip,
    failure_predicate: FFail,
) -> Vec<PipelineConditionFailure>
where
    FSkip: Fn(&PipelineStatus) -> bool,
    FFail: Fn(&Condition) -> bool,
{
    pipelines
        .iter()
        .filter(|(_, status)| !skip(status))
        .filter_map(|(key, status)| {
            let condition = status
                .conditions()
                .into_iter()
                .find(|c| c.kind == condition_kind)?;
            failure_predicate(&condition).then(|| PipelineConditionFailure {
                pipeline: key.clone(),
                condition,
            })
        })
        .collect()
}

fn acceptance_failure(condition: &Condition) -> bool {
    match condition.status {
        ConditionStatus::True => false,
        ConditionStatus::Unknown => {
            !matches!(condition.reason, Some(ConditionReason::NoPipelineRuntime))
        }
        ConditionStatus::False => {
            let benign = matches!(
                condition.reason,
                Some(
                    ConditionReason::Pending
                        | ConditionReason::StartRequested
                        | ConditionReason::Deleting
                        | ConditionReason::ForceDeleting
                        | ConditionReason::Deleted
                        | ConditionReason::NoPipelineRuntime
                )
            );
            !benign
        }
    }
}

fn skip_pipelines_without_runtimes(status: &PipelineStatus) -> bool {
    status.total_cores() == 0
}

impl ProbeResponse {
    fn ok(probe: &'static str) -> Self {
        Self {
            probe,
            status: "ok",
            generated_at: Utc::now().to_rfc3339(),
            failing: Vec::new(),
        }
    }

    fn fail(probe: &'static str, failing: Vec<PipelineConditionFailure>) -> Self {
        Self {
            probe,
            status: "failed",
            generated_at: Utc::now().to_rfc3339(),
            failing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cond(status: ConditionStatus, reason: Option<ConditionReason>) -> Condition {
        Condition {
            kind: ConditionKind::Accepted,
            status,
            reason,
            message: None,
            last_transition_time: None,
        }
    }

    #[test]
    fn acceptance_failure_ignores_benign_reasons() {
        assert!(!acceptance_failure(&cond(
            ConditionStatus::False,
            Some(ConditionReason::Pending)
        )));
        assert!(!acceptance_failure(&cond(
            ConditionStatus::False,
            Some(ConditionReason::Deleted)
        )));
        assert!(!acceptance_failure(&cond(
            ConditionStatus::Unknown,
            Some(ConditionReason::NoPipelineRuntime)
        )));
    }

    #[test]
    fn acceptance_failure_flags_errors() {
        assert!(acceptance_failure(&cond(
            ConditionStatus::False,
            Some(ConditionReason::AdmissionError)
        )));
        assert!(acceptance_failure(&cond(ConditionStatus::Unknown, None)));
    }
}
