// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared state types for the live-control runtime.
//!
//! This module intentionally contains mostly data and small conversion helpers.
//! Planning, execution, and runtime-instance management all mutate these
//! records through `ControllerRuntime` while holding the runtime mutex.

use super::*;

/// Maximum terminal rollout records retained per logical pipeline.
pub(super) const TERMINAL_ROLLOUT_RETENTION_LIMIT: usize = 32;
/// Maximum terminal shutdown records retained per logical pipeline.
pub(super) const TERMINAL_SHUTDOWN_RETENTION_LIMIT: usize = 32;
/// Maximum age for terminal rollout/shutdown records kept in memory.
pub(super) const TERMINAL_OPERATION_RETENTION_TTL: Duration = Duration::from_secs(24 * 60 * 60);

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

#[derive(Debug, Clone)]
/// Structured panic capture with public and diagnostic renderings.
///
/// Rollout/shutdown workers and runtime thread watchers use this type to keep
/// operator-visible failure messages concise while preserving thread context
/// and a forced backtrace for internal telemetry.
pub(crate) struct PanicReport {
    pub(super) kind: &'static str,
    pub(super) payload_message: String,
    pub(super) thread_name: Option<String>,
    pub(super) thread_id: Option<usize>,
    pub(super) core_id: Option<usize>,
    pub(super) backtrace: String,
}

impl PanicReport {
    /// Captures a panic payload plus best-effort worker/thread context.
    pub(crate) fn capture(
        kind: &'static str,
        panic: Box<dyn Any + Send>,
        thread_name: Option<String>,
        thread_id: Option<usize>,
        core_id: Option<usize>,
    ) -> Self {
        Self {
            kind,
            payload_message: panic_payload_message(&*panic),
            thread_name,
            thread_id,
            core_id,
            backtrace: Backtrace::force_capture().to_string(),
        }
    }

    /// Returns the short message stored in public rollout/shutdown status.
    pub(super) fn summary_message(&self) -> String {
        format!("{} panicked: {}", self.kind, self.payload_message)
    }

    /// Returns the diagnostic message used as internal error source detail.
    pub(super) fn detail_message(&self) -> String {
        let mut context = Vec::new();
        if let Some(thread_name) = &self.thread_name {
            context.push(format!("thread_name={thread_name}"));
        }
        if let Some(thread_id) = self.thread_id {
            context.push(format!("thread_id={thread_id}"));
        }
        if let Some(core_id) = self.core_id {
            context.push(format!("core_id={core_id}"));
        }

        let mut detail = self.summary_message();
        if !context.is_empty() {
            detail.push_str("\ncontext: ");
            detail.push_str(&context.join(", "));
        }
        detail.push_str("\nbacktrace:\n");
        detail.push_str(&self.backtrace);
        detail
    }

    /// Converts the panic report into the observed-state error payload.
    pub(super) fn error_summary(&self) -> ErrorSummary {
        ErrorSummary::Pipeline {
            error_kind: "panic".into(),
            message: self.summary_message(),
            source: Some(self.detail_message()),
        }
    }
}

#[derive(Debug, Clone)]
/// Error recorded when a deployed runtime instance exits unsuccessfully.
pub(crate) struct RuntimeInstanceError {
    pub(super) error_kind: String,
    pub(super) message: String,
    pub(super) detail: Option<String>,
}

impl RuntimeInstanceError {
    /// Builds a plain runtime error without panic diagnostics.
    pub(crate) fn runtime(message: String) -> Self {
        Self {
            error_kind: "runtime".into(),
            message,
            detail: None,
        }
    }

    /// Builds a runtime error from structured panic diagnostics.
    pub(crate) fn from_panic(report: PanicReport) -> Self {
        Self {
            error_kind: "panic".into(),
            message: report.summary_message(),
            detail: Some(report.detail_message()),
        }
    }

    /// Converts the runtime error into the observed-state error payload.
    pub(super) fn error_summary(&self) -> ErrorSummary {
        ErrorSummary::Pipeline {
            error_kind: self.error_kind.clone(),
            message: self.message.clone(),
            source: self.detail.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Execution strategy selected for a rollout request.
pub(super) enum RolloutAction {
    Create,
    NoOp,
    Replace,
    Resize,
}

impl RolloutAction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::NoOp => "noop",
            Self::Replace => "replace",
            Self::Resize => "resize",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Internal lifecycle for one rollout operation.
pub(super) enum RolloutLifecycleState {
    Pending,
    Running,
    Succeeded,
    Failed,
    RollingBack,
    RollbackFailed,
}

impl RolloutLifecycleState {
    const fn as_pipeline_rollout_state(self) -> PipelineRolloutState {
        match self {
            Self::Pending => PipelineRolloutState::Pending,
            Self::Running => PipelineRolloutState::Running,
            Self::Succeeded => PipelineRolloutState::Succeeded,
            Self::Failed => PipelineRolloutState::Failed,
            Self::RollingBack => PipelineRolloutState::RollingBack,
            Self::RollbackFailed => PipelineRolloutState::RollbackFailed,
        }
    }

    const fn as_api_pipeline_rollout_state(self) -> ApiPipelineRolloutState {
        match self {
            Self::Pending => ApiPipelineRolloutState::Pending,
            Self::Running => ApiPipelineRolloutState::Running,
            Self::Succeeded => ApiPipelineRolloutState::Succeeded,
            Self::Failed => ApiPipelineRolloutState::Failed,
            Self::RollingBack => ApiPipelineRolloutState::RollingBack,
            Self::RollbackFailed => ApiPipelineRolloutState::RollbackFailed,
        }
    }

    pub(super) const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::RollbackFailed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Internal lifecycle for one pipeline shutdown operation.
pub(super) enum ShutdownLifecycleState {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl ShutdownLifecycleState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    pub(super) const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }
}

#[derive(Debug, Clone)]
/// Per-core progress row within a rollout operation.
pub(super) struct RolloutCoreProgress {
    pub(super) core_id: usize,
    pub(super) previous_generation: Option<u64>,
    pub(super) target_generation: u64,
    pub(super) state: String,
    pub(super) updated_at: String,
    pub(super) detail: Option<String>,
}

#[derive(Debug, Clone)]
/// In-memory rollout record retained for active and recent terminal lookups.
pub(super) struct RolloutRecord {
    pub(super) rollout_id: String,
    pub(super) pipeline_group_id: PipelineGroupId,
    pub(super) pipeline_id: PipelineId,
    pub(super) action: RolloutAction,
    pub(super) state: RolloutLifecycleState,
    pub(super) target_generation: u64,
    pub(super) previous_generation: Option<u64>,
    /// Drain timeout requested with the rollout, reused for panic cleanup.
    pub(super) drain_timeout_secs: u64,
    pub(super) started_at: String,
    pub(super) updated_at: String,
    pub(super) failure_reason: Option<String>,
    pub(super) cores: Vec<RolloutCoreProgress>,
    pub(super) completed_at: Option<Instant>,
}

impl RolloutRecord {
    /// Creates the initial in-memory record for a rollout operation.
    pub(super) fn new(
        rollout_id: String,
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        action: RolloutAction,
        target_generation: u64,
        previous_generation: Option<u64>,
        drain_timeout_secs: u64,
        cores: Vec<RolloutCoreProgress>,
    ) -> Self {
        let now = timestamp_now();
        Self {
            rollout_id,
            pipeline_group_id,
            pipeline_id,
            action,
            state: RolloutLifecycleState::Pending,
            target_generation,
            previous_generation,
            drain_timeout_secs,
            started_at: now.clone(),
            updated_at: now,
            failure_reason: None,
            cores,
            completed_at: None,
        }
    }

    /// Builds the compact rollout summary exposed through observed state.
    pub(super) fn summary(&self) -> PipelineRolloutSummary {
        PipelineRolloutSummary {
            rollout_id: self.rollout_id.clone(),
            state: self.state.as_pipeline_rollout_state(),
            target_generation: self.target_generation,
            started_at: self.started_at.clone(),
            updated_at: self.updated_at.clone(),
            failure_reason: self.failure_reason.clone(),
        }
    }

    /// Builds the admin-facing rollout summary embedded in pipeline details.
    pub(super) fn api_summary(&self) -> ApiPipelineRolloutSummary {
        ApiPipelineRolloutSummary {
            rollout_id: self.rollout_id.clone(),
            state: self.state.as_api_pipeline_rollout_state(),
            target_generation: self.target_generation,
            started_at: self.started_at.clone(),
            updated_at: self.updated_at.clone(),
            failure_reason: self.failure_reason.clone(),
        }
    }

    /// Materializes the full rollout status returned by the control plane.
    pub(super) fn status(&self) -> RolloutStatus {
        RolloutStatus {
            rollout_id: self.rollout_id.clone(),
            pipeline_group_id: self.pipeline_group_id.clone(),
            pipeline_id: self.pipeline_id.clone(),
            action: self.action.as_str().to_owned(),
            state: self.state.as_api_pipeline_rollout_state(),
            target_generation: self.target_generation,
            previous_generation: self.previous_generation,
            started_at: self.started_at.clone(),
            updated_at: self.updated_at.clone(),
            failure_reason: self.failure_reason.clone(),
            cores: self
                .cores
                .iter()
                .map(|core| RolloutCoreStatus {
                    core_id: core.core_id,
                    previous_generation: core.previous_generation,
                    target_generation: core.target_generation,
                    state: core.state.clone(),
                    updated_at: core.updated_at.clone(),
                    detail: core.detail.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
/// Per-instance progress row within a shutdown operation.
pub(super) struct ShutdownCoreProgress {
    pub(super) core_id: usize,
    pub(super) deployment_generation: u64,
    pub(super) state: String,
    pub(super) updated_at: String,
    pub(super) detail: Option<String>,
}

#[derive(Debug, Clone)]
/// In-memory shutdown record retained for active and recent terminal lookups.
pub(super) struct ShutdownRecord {
    pub(super) shutdown_id: String,
    pub(super) pipeline_group_id: PipelineGroupId,
    pub(super) pipeline_id: PipelineId,
    pub(super) state: ShutdownLifecycleState,
    pub(super) started_at: String,
    pub(super) updated_at: String,
    pub(super) failure_reason: Option<String>,
    pub(super) cores: Vec<ShutdownCoreProgress>,
    pub(super) completed_at: Option<Instant>,
}

impl ShutdownRecord {
    /// Creates the initial in-memory record for a pipeline shutdown operation.
    pub(super) fn new(
        shutdown_id: String,
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        cores: Vec<ShutdownCoreProgress>,
    ) -> Self {
        let now = timestamp_now();
        Self {
            shutdown_id,
            pipeline_group_id,
            pipeline_id,
            state: ShutdownLifecycleState::Pending,
            started_at: now.clone(),
            updated_at: now,
            failure_reason: None,
            cores,
            completed_at: None,
        }
    }

    /// Materializes the full shutdown status returned by the control plane.
    pub(super) fn status(&self) -> ShutdownStatus {
        ShutdownStatus {
            shutdown_id: self.shutdown_id.clone(),
            pipeline_group_id: self.pipeline_group_id.clone(),
            pipeline_id: self.pipeline_id.clone(),
            state: self.state.as_str().to_owned(),
            started_at: self.started_at.clone(),
            updated_at: self.updated_at.clone(),
            failure_reason: self.failure_reason.clone(),
            cores: self
                .cores
                .iter()
                .map(|core| ShutdownCoreStatus {
                    core_id: core.core_id,
                    deployment_generation: core.deployment_generation,
                    state: core.state.clone(),
                    updated_at: core.updated_at.clone(),
                    detail: core.detail.clone(),
                })
                .collect(),
        }
    }
}

/// Controller-owned record for one deployed runtime instance.
pub(super) struct RuntimeInstanceRecord {
    // The controller drops this sender once shutdown is requested so the
    // pipeline control loop can observe channel closure after node tasks exit.
    pub(super) control_sender: Option<Arc<dyn PipelineAdminSender>>,
    pub(super) lifecycle: RuntimeInstanceLifecycle,
}

#[derive(Debug, Clone)]
/// Runtime-instance liveness as understood by the controller.
pub(super) enum RuntimeInstanceLifecycle {
    /// The pipeline thread is still expected to be running.
    Active,
    /// The pipeline thread reported a terminal exit.
    Exited(RuntimeInstanceExit),
}

#[derive(Debug, Clone)]
/// Terminal result reported by a deployed pipeline runtime thread.
pub(crate) enum RuntimeInstanceExit {
    /// The runtime exited normally after drain/shutdown.
    Success,
    /// The runtime exited due to a pipeline error or panic.
    Error(RuntimeInstanceError),
}

#[derive(Debug, Clone)]
/// Committed logical pipeline config plus the active deployment generation.
pub(super) struct LogicalPipelineRecord {
    pub(super) resolved: ResolvedPipelineConfig,
    pub(super) active_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Topic runtime properties that cannot be mutated by live rollout.
pub(super) struct TopicRuntimeProfile {
    pub(super) backend: TopicBackendKind,
    pub(super) policies: otap_df_config::topic::TopicPolicies,
    pub(super) selected_mode: InferredTopicMode,
}

/// Complete mutable state protected by `ControllerRuntime::state`.
///
/// Keep this type as plain data: methods that enforce lifecycle invariants
/// should live on `ControllerRuntime` so mutations can update observed state
/// and wake condition variables consistently.
pub(super) struct ControllerRuntimeState {
    /// Latest accepted full engine config, including committed live changes.
    pub(super) live_config: OtelDataflowSpec,
    /// Committed logical pipelines keyed by group/pipeline id.
    pub(super) logical_pipelines: HashMap<PipelineKey, LogicalPipelineRecord>,
    /// Deployed runtime instances keyed by group/pipeline/core/generation.
    pub(super) runtime_instances: HashMap<DeployedPipelineKey, RuntimeInstanceRecord>,
    // A pipeline thread can finish before register_launched_instance() publishes it as Active.
    // We park that exit here and reconcile it during registration instead of leaving stale
    // liveness behind.
    pub(super) pending_instance_exits: HashMap<DeployedPipelineKey, RuntimeInstanceExit>,
    /// Rollout snapshots retained for active and recent terminal lookups.
    pub(super) rollouts: HashMap<String, RolloutRecord>,
    /// Active rollout id per logical pipeline; presence causes operation conflict.
    pub(super) active_rollouts: HashMap<PipelineKey, String>,
    /// FIFO terminal rollout ids per logical pipeline for cap/TTL eviction.
    pub(super) terminal_rollouts: HashMap<PipelineKey, VecDeque<String>>,
    /// Shutdown snapshots retained for active and recent terminal lookups.
    pub(super) shutdowns: HashMap<String, ShutdownRecord>,
    /// Active shutdown id per logical pipeline; presence causes operation conflict.
    pub(super) active_shutdowns: HashMap<PipelineKey, String>,
    /// FIFO terminal shutdown ids per logical pipeline for cap/TTL eviction.
    pub(super) terminal_shutdowns: HashMap<PipelineKey, VecDeque<String>>,
    /// Next deployment generation to assign for each logical pipeline.
    pub(super) generation_counters: HashMap<PipelineKey, u64>,
    /// Count of runtime instances still considered active by the controller.
    pub(super) active_instances: usize,
    /// Monotonic rollout id suffix.
    pub(super) next_rollout_id: u64,
    /// Monotonic shutdown id suffix.
    pub(super) next_shutdown_id: u64,
    /// Monotonic logical runtime-thread id used for diagnostics.
    pub(super) next_thread_id: usize,
    /// First runtime failure surfaced to global controller shutdown handling.
    pub(super) first_error: Option<String>,
}

#[derive(Debug)]
/// Fully validated rollout plan ready for background execution.
///
/// The planner precomputes generation ids, target core sets, resize deltas,
/// operation records, and timeouts so the worker can execute without
/// reinterpreting the admin request.
pub(super) struct CandidateRolloutPlan {
    /// Logical pipeline targeted by the rollout.
    pub(super) pipeline_key: PipelineKey,
    pub(super) pipeline_group_id: PipelineGroupId,
    pub(super) pipeline_id: PipelineId,
    /// Execution strategy selected by request classification.
    pub(super) action: RolloutAction,
    /// Resolved target pipeline config after applying the request.
    pub(super) resolved_pipeline: ResolvedPipelineConfig,
    /// Current committed record, absent for create rollouts.
    pub(super) current_record: Option<LogicalPipelineRecord>,
    /// Core allocation from the committed record.
    pub(super) current_assigned_cores: Vec<usize>,
    /// Core allocation requested by the candidate config.
    pub(super) target_assigned_cores: Vec<usize>,
    /// Cores present in both current and target assignments.
    pub(super) common_assigned_cores: Vec<usize>,
    /// Cores present only in the target assignment.
    pub(super) added_assigned_cores: Vec<usize>,
    /// Cores present only in the current assignment.
    pub(super) removed_assigned_cores: Vec<usize>,
    /// Cores to launch for resize-only rollouts.
    pub(super) resize_start_cores: Vec<usize>,
    /// Cores to drain for resize-only rollouts.
    pub(super) resize_stop_cores: Vec<usize>,
    /// Deployment generation assigned to the target runtime instances.
    pub(super) target_generation: u64,
    /// Initial rollout status record to insert before spawning a worker.
    pub(super) rollout: RolloutRecord,
    /// Per-step readiness timeout in seconds.
    pub(super) step_timeout_secs: u64,
    /// Drain timeout in seconds for old instances.
    pub(super) drain_timeout_secs: u64,
}

#[derive(Debug)]
/// Fully validated shutdown plan ready for background execution.
pub(super) struct CandidateShutdownPlan {
    /// Logical pipeline targeted by the shutdown.
    pub(super) pipeline_key: PipelineKey,
    /// Initial shutdown status record to insert before spawning a worker.
    pub(super) shutdown: ShutdownRecord,
    /// Active deployed instances that must exit for shutdown success.
    pub(super) target_instances: Vec<DeployedPipelineKey>,
    /// Per-instance shutdown timeout in seconds.
    pub(super) timeout_secs: u64,
}

/// Snapshot of active cores for the current committed generation.
pub(super) struct ActiveRuntimeCoreState {
    /// Active cores still running the committed generation.
    pub(super) current_generation_cores: Vec<usize>,
    /// Whether another active generation exists for the same logical pipeline.
    pub(super) has_foreign_active_generations: bool,
}

/// Returns a fresh RFC3339 timestamp for externally visible status updates.
pub(super) fn timestamp_now() -> String {
    Utc::now().to_rfc3339()
}

/// Returns whether a terminal operation snapshot has exceeded retention TTL.
pub(super) fn is_expired(completed_at: Option<Instant>, now: Instant) -> bool {
    completed_at
        .and_then(|completed_at| now.checked_duration_since(completed_at))
        .is_some_and(|age| age >= TERMINAL_OPERATION_RETENTION_TTL)
}

#[derive(Debug)]
/// Rollout worker failure category used to distinguish rollback failures.
pub(super) enum RolloutExecutionError {
    /// The rollout failed before or outside rollback handling.
    Failed(String),
    /// Rollback was attempted but did not restore the previous runtime shape.
    RollbackFailed(String),
}
