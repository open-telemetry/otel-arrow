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
use otap_df_state::store::{
    Condition, ConditionStatus, ConditionType, NodeErrorSummary, PipelineEvent, PipelineEventKind,
    PipelinePhase, PipelineStatus,
};
use serde::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// All the routes for pipeline groups.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/pipeline-groups/status", get(show_status))
        .route("/pipeline-groups/shutdown", post(shutdown_all_pipelines))
}

#[derive(Serialize)]
pub struct PipelineGroupsStatusResponse {
    generated_at: String,
    summary: PipelinesSummary,
    pipelines: HashMap<String, PipelineStatusView>,
}

#[derive(Default, Serialize)]
struct PipelinesSummary {
    total: usize,
    phase_counts: HashMap<String, usize>,
    ready_true: usize,
    healthy_true: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    recent_failures: Vec<RecentFailure>,
}

#[derive(Serialize)]
struct RecentFailure {
    pipeline: String,
    condition_type: ConditionType,
    status: ConditionStatus,
    reason: String,
    message: String,
    #[serde(serialize_with = "ts_to_rfc3339")]
    last_transition_time: SystemTime,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    node_errors: Vec<NodeErrorSummary>,
}

#[derive(Serialize)]
struct PipelineStatusView {
    phase: PipelinePhase,
    #[serde(serialize_with = "ts_to_rfc3339")]
    phase_since: SystemTime,
    conditions: Vec<Condition>,
    per_core: HashMap<usize, otap_df_state::store::CoreStatus>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    recent_events: Vec<PipelineEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    derived: Option<DerivedPipelineInfo>,
}

#[derive(Serialize)]
struct DerivedPipelineInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    last_ready_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_healthy_time: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    latest_node_errors: Vec<NodeErrorSummary>,
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
            sender
                .try_send_shutdown("admin requested shutdown".to_owned()) // ToDo we probably need to codify reasons in the future
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
    pipelines: HashMap<PipelineKey, PipelineStatus>,
) -> PipelineGroupsStatusResponse {
    let mut summary = PipelinesSummary::default();
    let mut pipeline_views = HashMap::new();
    let mut recent_failures = Vec::new();

    for (key, status) in pipelines {
        let name = key.as_string();
        let view = PipelineStatusView::from_status(&status);

        summary.total += 1;
        *summary
            .phase_counts
            .entry(status.phase().to_string())
            .or_default() += 1;

        if is_condition_true(status.conditions(), ConditionType::Ready) {
            summary.ready_true += 1;
        }
        if is_condition_true(status.conditions(), ConditionType::Healthy) {
            summary.healthy_true += 1;
        }

        if let Some(failure) = build_recent_failure(&name, status.conditions()) {
            recent_failures.push(failure);
        }

        _ = pipeline_views.insert(name, view);
    }

    recent_failures.sort_by(|a, b| b.last_transition_time.cmp(&a.last_transition_time));
    summary.recent_failures = recent_failures.into_iter().take(10).collect();

    PipelineGroupsStatusResponse {
        generated_at: Utc::now().to_rfc3339(),
        summary,
        pipelines: pipeline_views,
    }
}

impl PipelineStatusView {
    fn from_status(status: &PipelineStatus) -> Self {
        let recent_events: Vec<PipelineEvent> = status.recent_events().iter().cloned().collect();

        let mut last_ready_time: Option<SystemTime> = None;
        let mut last_healthy_time: Option<SystemTime> = None;

        for event in status.recent_events().iter().rev() {
            if let PipelineEventKind::Condition {
                r#type,
                status: cond_status,
                ..
            } = event.kind()
            {
                if *r#type == ConditionType::Ready && *cond_status == ConditionStatus::True {
                    last_ready_time = Some(event.timestamp());
                }
                if *r#type == ConditionType::Healthy && *cond_status == ConditionStatus::True {
                    last_healthy_time = Some(event.timestamp());
                }
                if last_ready_time.is_some() && last_healthy_time.is_some() {
                    break;
                }
            }
        }

        let latest_node_errors = collect_latest_node_errors(status);
        let derived = if last_ready_time.is_none()
            && last_healthy_time.is_none()
            && latest_node_errors.is_empty()
        {
            None
        } else {
            Some(DerivedPipelineInfo {
                last_ready_time: format_optional_ts(last_ready_time),
                last_healthy_time: format_optional_ts(last_healthy_time),
                latest_node_errors,
            })
        };

        Self {
            phase: status.phase(),
            phase_since: status.phase_since(),
            conditions: status.conditions().cloned().collect(),
            per_core: status.per_core().clone(),
            recent_events,
            derived,
        }
    }
}

fn collect_latest_node_errors(status: &PipelineStatus) -> Vec<NodeErrorSummary> {
    let mut seen = HashSet::new();
    let mut errors = Vec::new();

    for condition in status.conditions() {
        if let Some(details) = condition.details.as_ref() {
            let is_failure = match condition.r#type {
                ConditionType::StartError => condition.status == ConditionStatus::True,
                ConditionType::Healthy | ConditionType::Ready => {
                    condition.status != ConditionStatus::True
                }
                _ => false,
            };

            if !is_failure {
                continue;
            }

            for node_error in &details.node_errors {
                let key = (
                    node_error.node.clone(),
                    node_error.error_kind.clone(),
                    node_error.message.clone(),
                );
                if seen.insert(key) {
                    errors.push(node_error.clone());
                }
            }
        }
    }

    errors
}

fn build_recent_failure<'a, I>(pipeline: &str, conditions: I) -> Option<RecentFailure>
where
    I: IntoIterator<Item = &'a Condition>,
{
    conditions
        .into_iter()
        .filter(|condition| match condition.r#type {
            ConditionType::StartError => condition.status == ConditionStatus::True,
            ConditionType::Healthy | ConditionType::Ready => {
                condition.status != ConditionStatus::True
            }
            _ => false,
        })
        .max_by_key(|condition| condition.last_transition_time)
        .map(|condition| RecentFailure {
            pipeline: pipeline.to_owned(),
            condition_type: condition.r#type.clone(),
            status: condition.status,
            reason: condition.reason.clone(),
            message: condition.message.clone(),
            last_transition_time: condition.last_transition_time,
            node_errors: condition
                .details
                .as_ref()
                .map(|d| d.node_errors.clone())
                .unwrap_or_default(),
        })
}

fn is_condition_true<'a, I>(conditions: I, kind: ConditionType) -> bool
where
    I: IntoIterator<Item = &'a Condition>,
{
    conditions
        .into_iter()
        .filter(|c| c.r#type == kind)
        .any(|c| c.status == ConditionStatus::True)
}

fn format_optional_ts(ts: Option<SystemTime>) -> Option<String> {
    ts.map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339())
}

fn ts_to_rfc3339<S>(t: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let dt: chrono::DateTime<Utc> = (*t).into();
    serializer.serialize_str(&dt.to_rfc3339())
}
