// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Client-side describe, filter, and diagnosis helpers used by the CLI and TUI.

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
