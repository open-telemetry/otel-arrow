use super::*;

pub(super) const TERMINAL_ROLLOUT_RETENTION_LIMIT: usize = 32;
pub(super) const TERMINAL_SHUTDOWN_RETENTION_LIMIT: usize = 32;
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
pub(crate) struct PanicReport {
    pub(super) kind: &'static str,
    pub(super) payload_message: String,
    pub(super) thread_name: Option<String>,
    pub(super) thread_id: Option<usize>,
    pub(super) core_id: Option<usize>,
    pub(super) backtrace: String,
}

impl PanicReport {
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

    pub(super) fn summary_message(&self) -> String {
        format!("{} panicked: {}", self.kind, self.payload_message)
    }

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

    pub(super) fn error_summary(&self) -> ErrorSummary {
        ErrorSummary::Pipeline {
            error_kind: "panic".into(),
            message: self.summary_message(),
            source: Some(self.detail_message()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeInstanceError {
    pub(super) error_kind: String,
    pub(super) message: String,
    pub(super) detail: Option<String>,
}

impl RuntimeInstanceError {
    pub(crate) fn runtime(message: String) -> Self {
        Self {
            error_kind: "runtime".into(),
            message,
            detail: None,
        }
    }

    pub(crate) fn from_panic(report: PanicReport) -> Self {
        Self {
            error_kind: "panic".into(),
            message: report.summary_message(),
            detail: Some(report.detail_message()),
        }
    }

    pub(super) fn error_summary(&self) -> ErrorSummary {
        ErrorSummary::Pipeline {
            error_kind: self.error_kind.clone(),
            message: self.message.clone(),
            source: self.detail.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub(super) struct RolloutCoreProgress {
    pub(super) core_id: usize,
    pub(super) previous_generation: Option<u64>,
    pub(super) target_generation: u64,
    pub(super) state: String,
    pub(super) updated_at: String,
    pub(super) detail: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct RolloutRecord {
    pub(super) rollout_id: String,
    pub(super) pipeline_group_id: PipelineGroupId,
    pub(super) pipeline_id: PipelineId,
    pub(super) action: RolloutAction,
    pub(super) state: RolloutLifecycleState,
    pub(super) target_generation: u64,
    pub(super) previous_generation: Option<u64>,
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
pub(super) struct ShutdownCoreProgress {
    pub(super) core_id: usize,
    pub(super) deployment_generation: u64,
    pub(super) state: String,
    pub(super) updated_at: String,
    pub(super) detail: Option<String>,
}

#[derive(Debug, Clone)]
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

pub(super) struct RuntimeInstanceRecord {
    // The controller drops this sender once shutdown is requested so the
    // pipeline control loop can observe channel closure after node tasks exit.
    pub(super) control_sender: Option<Arc<dyn PipelineAdminSender>>,
    pub(super) lifecycle: RuntimeInstanceLifecycle,
}

#[derive(Debug, Clone)]
pub(super) enum RuntimeInstanceLifecycle {
    Active,
    Exited(RuntimeInstanceExit),
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeInstanceExit {
    Success,
    Error(RuntimeInstanceError),
}

#[derive(Debug, Clone)]
pub(super) struct LogicalPipelineRecord {
    pub(super) resolved: ResolvedPipelineConfig,
    pub(super) active_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TopicRuntimeProfile {
    pub(super) backend: TopicBackendKind,
    pub(super) policies: otap_df_config::topic::TopicPolicies,
    pub(super) selected_mode: InferredTopicMode,
}

pub(super) struct ControllerRuntimeState {
    pub(super) live_config: OtelDataflowSpec,
    pub(super) logical_pipelines: HashMap<PipelineKey, LogicalPipelineRecord>,
    pub(super) runtime_instances: HashMap<DeployedPipelineKey, RuntimeInstanceRecord>,
    // A pipeline thread can finish before register_launched_instance() publishes it as Active.
    // We park that exit here and reconcile it during registration instead of leaving stale
    // liveness behind.
    pub(super) pending_instance_exits: HashMap<DeployedPipelineKey, RuntimeInstanceExit>,
    pub(super) rollouts: HashMap<String, RolloutRecord>,
    pub(super) active_rollouts: HashMap<PipelineKey, String>,
    pub(super) terminal_rollouts: HashMap<PipelineKey, VecDeque<String>>,
    pub(super) shutdowns: HashMap<String, ShutdownRecord>,
    pub(super) active_shutdowns: HashMap<PipelineKey, String>,
    pub(super) terminal_shutdowns: HashMap<PipelineKey, VecDeque<String>>,
    pub(super) generation_counters: HashMap<PipelineKey, u64>,
    pub(super) active_instances: usize,
    pub(super) next_rollout_id: u64,
    pub(super) next_shutdown_id: u64,
    pub(super) next_thread_id: usize,
    pub(super) first_error: Option<String>,
}

#[derive(Debug)]
pub(super) struct CandidateRolloutPlan {
    pub(super) pipeline_key: PipelineKey,
    pub(super) pipeline_group_id: PipelineGroupId,
    pub(super) pipeline_id: PipelineId,
    pub(super) action: RolloutAction,
    pub(super) resolved_pipeline: ResolvedPipelineConfig,
    pub(super) current_record: Option<LogicalPipelineRecord>,
    pub(super) current_assigned_cores: Vec<usize>,
    pub(super) target_assigned_cores: Vec<usize>,
    pub(super) common_assigned_cores: Vec<usize>,
    pub(super) added_assigned_cores: Vec<usize>,
    pub(super) removed_assigned_cores: Vec<usize>,
    pub(super) resize_start_cores: Vec<usize>,
    pub(super) resize_stop_cores: Vec<usize>,
    pub(super) target_generation: u64,
    pub(super) rollout: RolloutRecord,
    pub(super) step_timeout_secs: u64,
    pub(super) drain_timeout_secs: u64,
}

#[derive(Debug)]
pub(super) struct CandidateShutdownPlan {
    pub(super) pipeline_key: PipelineKey,
    pub(super) shutdown: ShutdownRecord,
    pub(super) target_instances: Vec<DeployedPipelineKey>,
    pub(super) timeout_secs: u64,
}

pub(super) struct ActiveRuntimeCoreState {
    pub(super) current_generation_cores: Vec<usize>,
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
pub(super) enum RolloutExecutionError {
    Failed(String),
    RollbackFailed(String),
}
