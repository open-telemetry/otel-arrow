// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Troubleshooting data models shared across describe, filter, and diagnosis flows.

use otap_df_admin_api::{groups, pipelines, telemetry};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedEventKind {
    Request,
    Success,
    Error,
    Log,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedEvent {
    pub time: String,
    pub kind: NormalizedEventKind,
    pub name: String,
    pub pipeline_group_id: String,
    pub pipeline_id: String,
    pub core_id: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record: Option<Value>,
}

impl NormalizedEvent {
    pub fn identity_key(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                "{}|{:?}|{}|{}/{}|{}|{:?}|{:?}|{:?}|{:?}",
                self.time,
                self.kind,
                self.name,
                self.pipeline_group_id,
                self.pipeline_id,
                self.core_id,
                self.deployment_generation,
                self.node_id,
                self.message,
                self.detail
            )
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventFilters {
    pub kinds: Vec<NormalizedEventKind>,
    pub pipeline_group_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub contains: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LogFilters {
    pub level: Option<String>,
    pub target: Option<String>,
    pub event: Option<String>,
    pub pipeline_group_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub contains: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MetricsFilters {
    pub metric_sets: Vec<String>,
    pub metric_names: Vec<String>,
    pub pipeline_group_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub core_id: Option<usize>,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsDescribeReport {
    pub status: groups::Status,
    pub summary: GroupsSummary,
    pub recent_events: Vec<NormalizedEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsSummary {
    pub total_pipelines: usize,
    pub running_pipelines: usize,
    pub ready_pipelines: usize,
    pub terminal_pipelines: usize,
    pub non_ready_pipelines: Vec<String>,
    pub non_terminal_pipelines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDescribeReport {
    pub details: pipelines::PipelineDetails,
    pub status: pipelines::Status,
    pub livez: pipelines::ProbeResult,
    pub readyz: pipelines::ProbeResult,
    pub recent_events: Vec<NormalizedEvent>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisStatus {
    Healthy,
    InProgress,
    Blocked,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceExcerpt {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisFinding {
    pub code: String,
    pub severity: FindingSeverity,
    pub summary: String,
    pub evidence: Vec<EvidenceExcerpt>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisReport {
    pub scope: String,
    pub status: DiagnosisStatus,
    pub summary: String,
    pub findings: Vec<DiagnosisFinding>,
    pub recommended_next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupShutdownWatchPipeline {
    pub pipeline: String,
    pub running_cores: usize,
    pub total_cores: usize,
    pub terminal: bool,
    pub phases: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupShutdownWatchSnapshot {
    pub started_at: String,
    pub generated_at: String,
    pub request_status: String,
    pub elapsed_ms: u64,
    pub total_pipelines: usize,
    pub terminal_pipelines: usize,
    pub all_terminal: bool,
    pub pipelines: Vec<GroupShutdownWatchPipeline>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleMetricsShape {
    Compact,
    Full,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleMetadata {
    pub collected_at: String,
    pub logs_limit: usize,
    pub metrics_shape: BundleMetricsShape,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "shape", content = "data", rename_all = "snake_case")]
pub enum BundleMetrics {
    Compact(telemetry::CompactMetricsResponse),
    Full(telemetry::MetricsResponse),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsBundle {
    pub metadata: BundleMetadata,
    pub describe: GroupsDescribeReport,
    pub diagnosis: DiagnosisReport,
    pub logs: telemetry::LogsResponse,
    pub metrics: BundleMetrics,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineBundle {
    pub metadata: BundleMetadata,
    pub describe: PipelineDescribeReport,
    pub diagnosis: DiagnosisReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollout_status: Option<pipelines::RolloutStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutdown_status: Option<pipelines::ShutdownStatus>,
    pub logs: telemetry::LogsResponse,
    pub metrics: BundleMetrics,
}
