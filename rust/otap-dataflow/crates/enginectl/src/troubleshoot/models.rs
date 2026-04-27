// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Stable troubleshooting report, filter, diagnosis, and bundle data models.
//!
//! These structs and enums form the boundary between internal admin SDK
//! responses and `dfctl`'s human-readable, JSON, NDJSON, and agent-oriented
//! outputs. Keeping the shapes in one module makes it clear which fields are
//! command-facing contracts and lets describe, filter, diagnose, render, and
//! TUI code share the same vocabulary.

use otap_df_admin_api::{groups, pipelines, telemetry};
use serde::Serialize;
use serde_json::Value;

/// Normalized event categories used by event filters and watch output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedEventKind {
    Request,
    Success,
    Error,
    Log,
}

/// Event shape derived from engine lifecycle events or retained logs.
///
/// This model gives commands one sortable/filterable event representation even
/// though the admin API exposes different raw event variants.
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
    /// Builds a stable de-duplication key used by watch commands.
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

/// Event filter model shared by event snapshot and watch commands.
#[derive(Debug, Clone, Default)]
pub struct EventFilters {
    pub kinds: Vec<NormalizedEventKind>,
    pub pipeline_group_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub contains: Option<String>,
}

/// Log filter model shared by log commands and troubleshooting flows.
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

/// Metrics filter model shared by metrics commands and troubleshooting flows.
#[derive(Debug, Clone, Default)]
pub struct MetricsFilters {
    pub metric_sets: Vec<String>,
    pub metric_names: Vec<String>,
    pub pipeline_group_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub core_id: Option<usize>,
    pub node_id: Option<String>,
}

/// Derived group-level status report emitted by `dfctl groups describe`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsDescribeReport {
    pub status: groups::Status,
    pub summary: GroupsSummary,
    pub recent_events: Vec<NormalizedEvent>,
}

/// Aggregated group status counts and problem pipeline lists.
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

/// Derived pipeline status report emitted by `dfctl pipelines describe`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDescribeReport {
    pub details: pipelines::PipelineDetails,
    pub status: pipelines::Status,
    pub livez: pipelines::ProbeResult,
    pub readyz: pipelines::ProbeResult,
    pub recent_events: Vec<NormalizedEvent>,
}

/// Coarse health classification assigned to a diagnosis report.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisStatus {
    Healthy,
    InProgress,
    Blocked,
    Failed,
    Unknown,
}

/// Severity assigned to one diagnosis finding.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
}

/// Small source excerpt that supports a diagnosis finding.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceExcerpt {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    pub message: String,
}

/// One actionable problem or observation produced by diagnosis heuristics.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisFinding {
    pub code: String,
    pub severity: FindingSeverity,
    pub summary: String,
    pub evidence: Vec<EvidenceExcerpt>,
}

/// Complete diagnosis result for group or pipeline troubleshooting.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisReport {
    pub scope: String,
    pub status: DiagnosisStatus,
    pub summary: String,
    pub findings: Vec<DiagnosisFinding>,
    pub recommended_next_steps: Vec<String>,
}

/// Per-pipeline row included in group shutdown watch snapshots.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupShutdownWatchPipeline {
    pub pipeline: String,
    pub running_cores: usize,
    pub total_cores: usize,
    pub terminal: bool,
    pub phases: Vec<String>,
}

/// Point-in-time group shutdown progress snapshot for human and NDJSON watch output.
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

/// Controls how much metric detail a support bundle carries.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleMetricsShape {
    /// Compact metrics are smaller and optimized for quick diagnosis.
    Compact,
    /// Full metrics preserve the verbose admin API metrics payload.
    Full,
}

/// Metadata describing how and when a support bundle was collected.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleMetadata {
    /// RFC 3339 timestamp recorded by the CLI at collection time.
    pub collected_at: String,
    /// Maximum number of retained log entries requested for the bundle.
    pub logs_limit: usize,
    /// Metrics detail level included in the bundle.
    pub metrics_shape: BundleMetricsShape,
}

/// Metrics payload embedded in a support bundle.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "shape", content = "data", rename_all = "snake_case")]
pub enum BundleMetrics {
    /// Compact metrics response from the admin API.
    Compact(telemetry::CompactMetricsResponse),
    /// Full metrics response from the admin API.
    Full(telemetry::MetricsResponse),
}

/// Fleet-wide troubleshooting package emitted by `dfctl groups bundle`.
///
/// A group bundle captures the evidence needed to investigate group-level
/// readiness or shutdown issues without issuing several separate commands. It
/// is a point-in-time CLI artifact and may contain sensitive telemetry or
/// operational data.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsBundle {
    /// Collection timestamp and capture options.
    pub metadata: BundleMetadata,
    /// Group status, summary, and recent events.
    pub describe: GroupsDescribeReport,
    /// Diagnosis derived from status, logs, and metrics.
    pub diagnosis: DiagnosisReport,
    /// Retained logs collected from the admin API.
    pub logs: telemetry::LogsResponse,
    /// Metrics collected at the requested detail level.
    pub metrics: BundleMetrics,
}

/// Pipeline-scoped troubleshooting package emitted by `dfctl pipelines bundle`.
///
/// A pipeline bundle captures describe output, diagnosis, logs, metrics, and
/// optional operation status for one pipeline. It is intended for incident
/// handoff, offline inspection, and agent-assisted troubleshooting.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineBundle {
    /// Collection timestamp and capture options.
    pub metadata: BundleMetadata,
    /// Pipeline details, status, probes, and recent events.
    pub describe: PipelineDescribeReport,
    /// Diagnosis derived from pipeline evidence.
    pub diagnosis: DiagnosisReport,
    /// Optional rollout status included when the command targets a rollout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollout_status: Option<pipelines::RolloutStatus>,
    /// Optional shutdown status included when the command targets a shutdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutdown_status: Option<pipelines::ShutdownStatus>,
    /// Retained logs collected from the admin API.
    pub logs: telemetry::LogsResponse,
    /// Metrics collected at the requested detail level.
    pub metrics: BundleMetrics,
}
