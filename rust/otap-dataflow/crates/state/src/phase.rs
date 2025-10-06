// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Definition of all states/phases that a pipeline can be in.

use otap_df_config::health::PhaseKind;
use serde::Serialize;
use std::fmt;
use std::fmt::Display;

/// States/Phases that a pipeline instance (bound to a CPU core) can be in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
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
    /// Admission or configuration was rejected before (or during) startup.
    /// Examples include:
    /// - for admission: quota exceeded, resource limits, ...
    /// - for configuration: invalid or incompatible config, ...
    Rejected(RejectReason),
    /// A new spec/version is being applied (rolling or otherwise) while the pipeline
    /// remains under control.
    Updating,
    /// The system is reverting to the last known-good state after an update failure.
    RollingBack,
    /// Teardown is in progress; `DeletionMode` indicates graceful (after drain) or forced.
    Deleting(DeletionMode),
    /// All resources for this pipeline have been removed; terminal state.
    Deleted,
}

/// Monitoring-friendly aggregate phase for a logical pipeline spanning many cores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PipelineAggPhase {
    /// All cores are deleted; the pipeline has been fully removed.
    Deleted,
    /// Teardown is in progress.
    Deleting {
        /// `forced=true` if any core uses forced deletion
        forced: bool,
        /// How many pipeline core instances are still being deleted
        remaining: usize,
    },
    /// One or more cores are in a failed state
    Failed {
        /// How many pipeline instance cores are in `Failed` state
        failed: usize,
        /// How many pipeline instance cores are currently serving
        running: usize,
        /// The most common reason for failure among the failed cores
        top_reason: Option<FailReason>,
    },
    /// One or more cores are rejected.
    /// Counts plus most common rejection reason.
    Rejected {
        /// How many cores are currently rejected
        rejected: usize,
        /// How many cores are currently serving
        running: usize,
        /// The most common reason for rejection among the rejected cores
        top_reason: Option<RejectReason>,
    },
    /// A rollback is underway on at least one core
    RollingBack {
        /// How many cores are currently rolling back
        rolling_back: usize,
        /// How many cores are currently serving
        running: usize,
    },
    /// An update is applying on at least one core
    Updating {
        /// How many cores are currently updating
        updating: usize,
        /// How many cores are currently serving
        running: usize,
    },
    /// Graceful shutdown in progress on some cores
    Draining {
        /// How many cores are currently in `Draining` state
        draining: usize,
        /// How many cores are currently serving
        running: usize,
    },
    /// All non-deleted cores are running (full capacity).
    RunningAll,
    /// Some cores are running while others are not (partial capacity).
    RunningDegraded {
        /// How many cores are currently running
        running: usize,
        /// How many non-deleted cores are there in total
        total_active: usize,
    },
    /// All non-deleted cores are stopped (safe but not serving)
    StoppedAll {
        /// How many cores are currently stopped
        stopped: usize,
    },
    /// A mix where some non-deleted cores are stopped (includes how many are active overall)
    StoppedPartial {
        /// How many cores are currently stopped
        stopped: usize,
        /// How many non-deleted cores are there in total
        total_active: usize,
    },
    /// Early lifecycle: at least one core is pending/starting
    Starting {
        /// How many cores are currently in `Pending` state
        pending: usize,
        /// How many cores are currently in `Starting` state
        starting: usize,
    },
    /// The state has not been determined yet or is inconsistent.
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
            PipelinePhase::Rejected(_) => "Rejected",
            PipelinePhase::Starting => "Starting",
            PipelinePhase::Updating => "Updating",
            PipelinePhase::RollingBack => "RollingBack",
            PipelinePhase::Deleting(_) => "Deleting",
            PipelinePhase::Deleted => "Deleted",
        };
        write!(f, "{label}")
    }
}

impl PipelinePhase {
    /// Returns the `PhaseKind` corresponding to this `PipelinePhase` (i.e. without details).
    #[must_use]
    pub fn kind(&self) -> PhaseKind {
        match self {
            PipelinePhase::Pending => PhaseKind::Pending,
            PipelinePhase::Starting => PhaseKind::Starting,
            PipelinePhase::Running => PhaseKind::Running,
            PipelinePhase::Updating => PhaseKind::Updating,
            PipelinePhase::RollingBack => PhaseKind::RollingBack,
            PipelinePhase::Draining => PhaseKind::Draining,
            PipelinePhase::Stopped => PhaseKind::Stopped,
            PipelinePhase::Failed(_) => PhaseKind::Failed,
            PipelinePhase::Deleting(_) => PhaseKind::Deleting,
            PipelinePhase::Deleted => PhaseKind::Deleted,
            PipelinePhase::Rejected(_) => PhaseKind::Rejected,
        }
    }
}

/// Why admission/config were rejected (distinct from runtime failures).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum RejectReason {
    /// Admission refused the pipeline (policy/quotas/validation).
    AdmissionError,
    /// Startup aborted due to invalid or incompatible configuration.
    ConfigRejected,
}

/// How a deletion was initiated and executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletionMode {
    /// Deletion preceded by a graceful drain of work/traffic.
    Graceful,
    /// Immediate deletion that preempts draining and can drop in‑flight work.
    Forced,
}

/// Categorized reasons for entering `PipelinePhase::Failed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
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
