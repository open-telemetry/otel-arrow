// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Client-side troubleshooting primitives shared by the CLI command handlers and the TUI.
//!
//! The troubleshoot layer turns raw admin API responses into higher-level
//! evidence that is easier for humans, shell scripts, and AI agents to consume.
//! The submodules have intentionally narrow responsibilities: `models` defines
//! the stable report and bundle shapes, `describe` derives summaries and
//! normalized events from status payloads, `filter` applies client-side
//! scoping to logs/events/metrics, and `diagnose` converts the collected
//! evidence into actionable findings.

#[path = "troubleshoot/describe.rs"]
mod describe;
#[path = "troubleshoot/diagnose.rs"]
mod diagnose;
#[path = "troubleshoot/filter.rs"]
mod filter;
#[path = "troubleshoot/models.rs"]
mod models;

pub use describe::{
    describe_groups, describe_pipeline, extract_events_from_group_status,
    extract_events_from_pipeline_status, group_shutdown_snapshot, tail_events,
};
pub use diagnose::{
    diagnose_group_shutdown, diagnose_pipeline_rollout, diagnose_pipeline_shutdown,
};
pub use filter::{filter_logs, filter_metrics_compact, filter_metrics_full};
pub use models::{
    BundleMetadata, BundleMetrics, BundleMetricsShape, DiagnosisFinding, DiagnosisReport,
    DiagnosisStatus, EventFilters, EvidenceExcerpt, FindingSeverity, GroupShutdownWatchPipeline,
    GroupShutdownWatchSnapshot, GroupsBundle, GroupsDescribeReport, LogFilters, MetricsFilters,
    NormalizedEvent, NormalizedEventKind, PipelineBundle, PipelineDescribeReport,
};

#[cfg(test)]
#[path = "troubleshoot/tests.rs"]
mod tests;
