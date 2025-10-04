// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Definition of all states/phases that a pipeline can be in.

use std::fmt;
use std::fmt::Display;
use serde::Serialize;

/// High-level lifecycle of a pipeline as seen by the controller.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub enum PipelinePhase {
    /// The pipeline exists but has not been admitted to start yet, awaiting a decision (e.g.
    /// admission controller, quota checks, start_request, ...).
    Pending,
    /// The pipeline is in the process of starting up (initialization of all nodes and connections
    /// based on the pipeline spec).
    Starting,
    /// The pipeline is currently running and actively processing telemetry data.
    Running,
    /// A graceful stop has been requested. Ingress is quiescing and in-flight
    /// data is draining, possibly with a deadline.
    Draining,
    /// The pipeline has been stopped.
    Stopped,
    /// Entered a terminal error state (e.g. unrecoverable apply error). The
    /// controller may attempt retries based on policy, but phase reflects the
    /// current failure.
    Failed(FailReason),
    /// The pipeline spec was invalid or could not be applied (e.g. validation
    /// error, quota exceeded, resource limits, ...).
    Rejected,
    /// A new spec/version is being applied (rolling or otherwise) while the pipeline
    /// remains under control.
    Updating,
    /// The system is reverting to the last known-good state after an update failure.
    RollingBack,
    /// Teardown is in progress; `DeletionMode` indicates graceful (after drain) or forced.
    Deleting(DeletionMode),
    /// All resources for this pipeline have been removed; terminal state.
    Deleted,

    /// The controller cannot currently determine the state (e.g. missing
    /// heartbeats past the freshness window).
    Unknown,
}

impl Display for PipelinePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            PipelinePhase::Pending => "Pending",
            PipelinePhase::Running => "Running",
            PipelinePhase::Draining => "Draining",
            PipelinePhase::Stopped => "Stopped",
            PipelinePhase::Failed(_) => "Failed",
            PipelinePhase::Rejected => "Rejected",
            PipelinePhase::Starting => "Starting",
            PipelinePhase::Updating => "Updating",
            PipelinePhase::RollingBack => "RollingBack",
            PipelinePhase::Deleting(_) => "Deleting",
            PipelinePhase::Deleted => "Deleted",
            PipelinePhase::Unknown => "Unknown",
        };
        write!(f, "{label}")
    }
}

/// How a deletion was initiated and executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DeletionMode {
    /// Deletion preceded by a graceful drain of work/traffic.
    Graceful,
    /// Immediate deletion that preempts draining and can drop in‑flight work.
    Forced,
}

/// Categorized reasons for entering `PipelinePhase::Failed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FailReason {
    /// Admission rejected the pipeline before startup.
    AdmissionError,
    /// Startup failed due to invalid or incompatible configuration.
    ConfigRejected,
    /// A fatal runtime fault occurred while the pipeline was active.
    RuntimeError,
    /// Draining could not complete within policy or hit an unrecoverable condition.
    DrainError,
    /// The rollback process itself failed to complete successfully.
    RollbackFailed,
    /// Teardown failed while deleting resources.
    DeleteError,
}