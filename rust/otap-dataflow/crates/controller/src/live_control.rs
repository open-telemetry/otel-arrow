// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use chrono::Utc;
use otap_df_admin::{
    ControlPlane, ControlPlaneError, PipelineDetails,
    PipelineRolloutState as ApiPipelineRolloutState,
    PipelineRolloutSummary as ApiPipelineRolloutSummary, ReconfigureRequest, RolloutCoreStatus,
    RolloutStatus, ShutdownCoreStatus, ShutdownStatus,
};
use otap_df_state::conditions::ConditionStatus;
use otap_df_state::phase::PipelinePhase;
use otap_df_state::pipeline_status::{PipelineRolloutState, PipelineRolloutSummary};
use std::any::Any;
use std::backtrace::Backtrace;
use std::collections::VecDeque;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Condvar, Mutex, Weak};
use std::time::{Duration, Instant};

const TERMINAL_ROLLOUT_RETENTION_LIMIT: usize = 32;
const TERMINAL_SHUTDOWN_RETENTION_LIMIT: usize = 32;
const TERMINAL_OPERATION_RETENTION_TTL: Duration = Duration::from_secs(24 * 60 * 60);

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
struct PanicReport {
    kind: &'static str,
    payload_message: String,
    thread_name: Option<String>,
    thread_id: Option<usize>,
    core_id: Option<usize>,
    backtrace: String,
}

impl PanicReport {
    fn capture(
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

    fn summary_message(&self) -> String {
        format!("{} panicked: {}", self.kind, self.payload_message)
    }

    fn detail_message(&self) -> String {
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

    fn error_summary(&self) -> ErrorSummary {
        ErrorSummary::Pipeline {
            error_kind: "panic".into(),
            message: self.summary_message(),
            source: Some(self.detail_message()),
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeInstanceError {
    error_kind: String,
    message: String,
    detail: Option<String>,
}

impl RuntimeInstanceError {
    fn runtime(message: String) -> Self {
        Self {
            error_kind: "runtime".into(),
            message,
            detail: None,
        }
    }

    fn from_panic(report: PanicReport) -> Self {
        Self {
            error_kind: "panic".into(),
            message: report.summary_message(),
            detail: Some(report.detail_message()),
        }
    }

    fn error_summary(&self) -> ErrorSummary {
        ErrorSummary::Pipeline {
            error_kind: self.error_kind.clone(),
            message: self.message.clone(),
            source: self.detail.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RolloutAction {
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
enum RolloutLifecycleState {
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

    const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::RollbackFailed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShutdownLifecycleState {
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

    const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }
}

#[derive(Debug, Clone)]
struct RolloutCoreProgress {
    core_id: usize,
    previous_generation: Option<u64>,
    target_generation: u64,
    state: String,
    updated_at: String,
    detail: Option<String>,
}

#[derive(Debug, Clone)]
struct RolloutRecord {
    rollout_id: String,
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
    action: RolloutAction,
    state: RolloutLifecycleState,
    target_generation: u64,
    previous_generation: Option<u64>,
    started_at: String,
    updated_at: String,
    failure_reason: Option<String>,
    cores: Vec<RolloutCoreProgress>,
    completed_at: Option<Instant>,
}

impl RolloutRecord {
    /// Creates the initial in-memory record for a rollout operation.
    fn new(
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
    fn summary(&self) -> PipelineRolloutSummary {
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
    fn api_summary(&self) -> ApiPipelineRolloutSummary {
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
    fn status(&self) -> RolloutStatus {
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
struct ShutdownCoreProgress {
    core_id: usize,
    deployment_generation: u64,
    state: String,
    updated_at: String,
    detail: Option<String>,
}

#[derive(Debug, Clone)]
struct ShutdownRecord {
    shutdown_id: String,
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
    state: ShutdownLifecycleState,
    started_at: String,
    updated_at: String,
    failure_reason: Option<String>,
    cores: Vec<ShutdownCoreProgress>,
    completed_at: Option<Instant>,
}

impl ShutdownRecord {
    /// Creates the initial in-memory record for a pipeline shutdown operation.
    fn new(
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
    fn status(&self) -> ShutdownStatus {
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

struct RuntimeInstanceRecord {
    // The controller drops this sender once shutdown is requested so the
    // pipeline control loop can observe channel closure after node tasks exit.
    control_sender: Option<Arc<dyn PipelineAdminSender>>,
    lifecycle: RuntimeInstanceLifecycle,
}

#[derive(Debug, Clone)]
enum RuntimeInstanceLifecycle {
    Active,
    Exited(RuntimeInstanceExit),
}

#[derive(Debug, Clone)]
enum RuntimeInstanceExit {
    Success,
    Error(RuntimeInstanceError),
}

#[derive(Debug, Clone)]
struct LogicalPipelineRecord {
    resolved: ResolvedPipelineConfig,
    active_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TopicRuntimeProfile {
    backend: TopicBackendKind,
    policies: otap_df_config::topic::TopicPolicies,
    selected_mode: InferredTopicMode,
}

struct ControllerRuntimeState {
    live_config: OtelDataflowSpec,
    logical_pipelines: HashMap<PipelineKey, LogicalPipelineRecord>,
    runtime_instances: HashMap<DeployedPipelineKey, RuntimeInstanceRecord>,
    rollouts: HashMap<String, RolloutRecord>,
    active_rollouts: HashMap<PipelineKey, String>,
    terminal_rollouts: HashMap<PipelineKey, VecDeque<String>>,
    shutdowns: HashMap<String, ShutdownRecord>,
    active_shutdowns: HashMap<PipelineKey, String>,
    terminal_shutdowns: HashMap<PipelineKey, VecDeque<String>>,
    generation_counters: HashMap<PipelineKey, u64>,
    active_instances: usize,
    next_rollout_id: u64,
    next_shutdown_id: u64,
    next_thread_id: usize,
    first_error: Option<String>,
}

pub(super) struct ControllerRuntime<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    pipeline_factory: &'static PipelineFactory<PData>,
    controller_context: ControllerContext,
    observed_state_store: ObservedStateStore,
    observed_state_handle: ObservedStateHandle,
    engine_event_reporter: ObservedEventReporter,
    metrics_reporter: MetricsReporter,
    declared_topics: DeclaredTopics<PData>,
    available_core_ids: Vec<CoreId>,
    engine_tracing_setup: TracingSetup,
    telemetry_reporting_interval: Duration,
    memory_pressure_tx: tokio::sync::watch::Sender<MemoryPressureChanged>,
    state: Mutex<ControllerRuntimeState>,
    state_changed: Condvar,
}

struct ControllerControlPlane<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    runtime: Arc<ControllerRuntime<PData>>,
}

pub(super) struct LaunchedPipelineThread<PData> {
    pub(super) thread_name: String,
    pub(super) thread_id: usize,
    pub(super) pipeline_key: DeployedPipelineKey,
    pub(super) control_sender: Arc<dyn PipelineAdminSender>,
    pub(super) join_handle: thread::JoinHandle<Result<Vec<()>, Error>>,
    pub(super) _marker: std::marker::PhantomData<PData>,
}

#[derive(Debug)]
struct CandidateRolloutPlan {
    pipeline_key: PipelineKey,
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
    action: RolloutAction,
    resolved_pipeline: ResolvedPipelineConfig,
    current_record: Option<LogicalPipelineRecord>,
    current_assigned_cores: Vec<usize>,
    target_assigned_cores: Vec<usize>,
    common_assigned_cores: Vec<usize>,
    added_assigned_cores: Vec<usize>,
    removed_assigned_cores: Vec<usize>,
    resize_start_cores: Vec<usize>,
    resize_stop_cores: Vec<usize>,
    target_generation: u64,
    rollout: RolloutRecord,
    step_timeout_secs: u64,
    drain_timeout_secs: u64,
}

#[derive(Debug)]
struct CandidateShutdownPlan {
    pipeline_key: PipelineKey,
    shutdown: ShutdownRecord,
    target_instances: Vec<DeployedPipelineKey>,
    timeout_secs: u64,
}

struct ActiveRuntimeCoreState {
    current_generation_cores: Vec<usize>,
    has_foreign_active_generations: bool,
}

/// Returns a fresh RFC3339 timestamp for externally visible status updates.
fn timestamp_now() -> String {
    Utc::now().to_rfc3339()
}

/// Returns whether a terminal operation snapshot has exceeded retention TTL.
fn is_expired(completed_at: Option<Instant>, now: Instant) -> bool {
    completed_at
        .and_then(|completed_at| now.checked_duration_since(completed_at))
        .is_some_and(|age| age >= TERMINAL_OPERATION_RETENTION_TTL)
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable>
    ControllerRuntime<PData>
{
    #[allow(clippy::too_many_arguments)]
    /// Builds the resident controller runtime used by live reconfiguration.
    pub(super) fn new(
        pipeline_factory: &'static PipelineFactory<PData>,
        controller_context: ControllerContext,
        observed_state_store: ObservedStateStore,
        observed_state_handle: ObservedStateHandle,
        engine_event_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        declared_topics: DeclaredTopics<PData>,
        available_core_ids: Vec<CoreId>,
        engine_tracing_setup: TracingSetup,
        telemetry_reporting_interval: Duration,
        memory_pressure_tx: tokio::sync::watch::Sender<MemoryPressureChanged>,
        live_config: OtelDataflowSpec,
    ) -> Self {
        Self {
            pipeline_factory,
            controller_context,
            observed_state_store,
            observed_state_handle,
            engine_event_reporter,
            metrics_reporter,
            declared_topics,
            available_core_ids,
            engine_tracing_setup,
            telemetry_reporting_interval,
            memory_pressure_tx,
            state: Mutex::new(ControllerRuntimeState {
                live_config,
                logical_pipelines: HashMap::new(),
                runtime_instances: HashMap::new(),
                rollouts: HashMap::new(),
                active_rollouts: HashMap::new(),
                terminal_rollouts: HashMap::new(),
                shutdowns: HashMap::new(),
                active_shutdowns: HashMap::new(),
                terminal_shutdowns: HashMap::new(),
                generation_counters: HashMap::new(),
                active_instances: 0,
                next_rollout_id: 0,
                next_shutdown_id: 0,
                next_thread_id: 1,
                first_error: None,
            }),
            state_changed: Condvar::new(),
        }
    }

    /// Seeds the runtime registry with a pipeline already committed at startup.
    pub(super) fn register_committed_pipeline(
        &self,
        resolved: ResolvedPipelineConfig,
        generation: u64,
    ) {
        let pipeline_key = PipelineKey::new(
            resolved.pipeline_group_id.clone(),
            resolved.pipeline_id.clone(),
        );
        self.observed_state_store
            .set_pipeline_active_generation(pipeline_key.clone(), generation);

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        _ = state
            .generation_counters
            .insert(pipeline_key.clone(), generation + 1);
        _ = state.logical_pipelines.insert(
            pipeline_key,
            LogicalPipelineRecord {
                resolved,
                active_generation: generation,
            },
        );
    }

    /// Allocates the next controller-local logical thread identifier.
    pub(super) fn next_thread_id(&self) -> usize {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let thread_id = state.next_thread_id;
        state.next_thread_id += 1;
        thread_id
    }

    /// Returns the declared-topic registry shared with launched pipelines.
    pub(super) fn declared_topics(&self) -> &DeclaredTopics<PData> {
        &self.declared_topics
    }

    /// Exposes the runtime as the admin control-plane trait object.
    pub(super) fn control_plane(self: &Arc<Self>) -> Arc<dyn ControlPlane> {
        Arc::new(ControllerControlPlane {
            runtime: Arc::clone(self),
        })
    }

    /// Checks whether a logical pipeline still has an active rollout or shutdown.
    fn pipeline_has_active_operation_locked(
        state: &ControllerRuntimeState,
        pipeline_key: &PipelineKey,
    ) -> bool {
        state.active_rollouts.contains_key(pipeline_key)
            || state.active_shutdowns.contains_key(pipeline_key)
    }

    /// Marks a rollout terminal and enqueues it for bounded retention.
    fn record_terminal_rollout_locked(
        state: &mut ControllerRuntimeState,
        pipeline_key: &PipelineKey,
        rollout_id: &str,
        now: Instant,
    ) {
        let mut enqueue = false;
        if let Some(rollout) = state.rollouts.get_mut(rollout_id) {
            if rollout.state.is_terminal() && rollout.completed_at.is_none() {
                rollout.completed_at = Some(now);
                enqueue = true;
            }
        }
        if enqueue {
            state
                .terminal_rollouts
                .entry(pipeline_key.clone())
                .or_default()
                .push_back(rollout_id.to_owned());
        }
        Self::prune_terminal_rollout_queue_locked(state, pipeline_key, now);
    }

    /// Evicts expired or over-cap terminal rollout snapshots for one pipeline.
    fn prune_terminal_rollout_queue_locked(
        state: &mut ControllerRuntimeState,
        pipeline_key: &PipelineKey,
        now: Instant,
    ) {
        loop {
            let Some((rollout_id, queue_len)) =
                state.terminal_rollouts.get(pipeline_key).and_then(|queue| {
                    queue
                        .front()
                        .cloned()
                        .map(|rollout_id| (rollout_id, queue.len()))
                })
            else {
                break;
            };

            let should_evict = queue_len > TERMINAL_ROLLOUT_RETENTION_LIMIT
                || state
                    .rollouts
                    .get(&rollout_id)
                    .is_none_or(|rollout| is_expired(rollout.completed_at, now));
            if !should_evict {
                break;
            }

            if let Some(evicted_id) = state
                .terminal_rollouts
                .get_mut(pipeline_key)
                .and_then(VecDeque::pop_front)
            {
                _ = state.rollouts.remove(&evicted_id);
            }
        }

        if state
            .terminal_rollouts
            .get(pipeline_key)
            .is_some_and(VecDeque::is_empty)
        {
            _ = state.terminal_rollouts.remove(pipeline_key);
        }
    }

    /// Marks a shutdown terminal and enqueues it for bounded retention.
    fn record_terminal_shutdown_locked(
        state: &mut ControllerRuntimeState,
        pipeline_key: &PipelineKey,
        shutdown_id: &str,
        now: Instant,
    ) {
        let mut enqueue = false;
        if let Some(shutdown) = state.shutdowns.get_mut(shutdown_id) {
            if shutdown.state.is_terminal() && shutdown.completed_at.is_none() {
                shutdown.completed_at = Some(now);
                enqueue = true;
            }
        }
        if enqueue {
            state
                .terminal_shutdowns
                .entry(pipeline_key.clone())
                .or_default()
                .push_back(shutdown_id.to_owned());
        }
        Self::prune_terminal_shutdown_queue_locked(state, pipeline_key, now);
    }

    /// Evicts expired or over-cap terminal shutdown snapshots for one pipeline.
    fn prune_terminal_shutdown_queue_locked(
        state: &mut ControllerRuntimeState,
        pipeline_key: &PipelineKey,
        now: Instant,
    ) {
        loop {
            let Some((shutdown_id, queue_len)) = state
                .terminal_shutdowns
                .get(pipeline_key)
                .and_then(|queue| {
                    queue
                        .front()
                        .cloned()
                        .map(|shutdown_id| (shutdown_id, queue.len()))
                })
            else {
                break;
            };

            let should_evict = queue_len > TERMINAL_SHUTDOWN_RETENTION_LIMIT
                || state
                    .shutdowns
                    .get(&shutdown_id)
                    .is_none_or(|shutdown| is_expired(shutdown.completed_at, now));
            if !should_evict {
                break;
            }

            if let Some(evicted_id) = state
                .terminal_shutdowns
                .get_mut(pipeline_key)
                .and_then(VecDeque::pop_front)
            {
                _ = state.shutdowns.remove(&evicted_id);
            }
        }

        if state
            .terminal_shutdowns
            .get(pipeline_key)
            .is_some_and(VecDeque::is_empty)
        {
            _ = state.terminal_shutdowns.remove(pipeline_key);
        }
    }

    /// Runs TTL/cap eviction across all retained terminal operation history.
    fn prune_terminal_operation_history_locked(state: &mut ControllerRuntimeState, now: Instant) {
        let rollout_keys: Vec<_> = state.terminal_rollouts.keys().cloned().collect();
        for pipeline_key in rollout_keys {
            Self::prune_terminal_rollout_queue_locked(state, &pipeline_key, now);
        }

        let shutdown_keys: Vec<_> = state.terminal_shutdowns.keys().cloned().collect();
        for pipeline_key in shutdown_keys {
            Self::prune_terminal_shutdown_queue_locked(state, &pipeline_key, now);
        }
    }

    /// Drops exited runtime instances once no active controller work still needs them.
    fn prune_exited_runtime_instances_for_pipeline_locked(
        state: &mut ControllerRuntimeState,
        pipeline_key: &PipelineKey,
    ) -> bool {
        if Self::pipeline_has_active_operation_locked(state, pipeline_key) {
            return false;
        }

        state.runtime_instances.retain(|deployed_key, instance| {
            if deployed_key.pipeline_group_id != *pipeline_key.pipeline_group_id()
                || deployed_key.pipeline_id != *pipeline_key.pipeline_id()
            {
                return true;
            }

            matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
        });
        true
    }

    /// Opportunistically trims retained rollout and shutdown history.
    fn prune_retained_operation_history(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::prune_terminal_operation_history_locked(&mut state, Instant::now());
    }

    /// Trims exited instances and terminal history for one logical pipeline.
    fn prune_pipeline_runtime_and_history(&self, pipeline_key: &PipelineKey) {
        let should_compact = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let should_compact =
                Self::prune_exited_runtime_instances_for_pipeline_locked(&mut state, pipeline_key);
            Self::prune_terminal_rollout_queue_locked(&mut state, pipeline_key, Instant::now());
            Self::prune_terminal_shutdown_queue_locked(&mut state, pipeline_key, Instant::now());
            should_compact
        };
        if should_compact {
            self.observed_state_store
                .compact_pipeline_instances(pipeline_key);
        }
    }

    /// Resolves the concrete core ids selected by a pipeline resource policy.
    fn assigned_cores_for_resolved(
        &self,
        resolved_pipeline: &ResolvedPipelineConfig,
    ) -> Result<Vec<usize>, ControlPlaneError> {
        Controller::<PData>::select_cores_for_allocation(
            self.available_core_ids.clone(),
            &resolved_pipeline.policies.resources.core_allocation,
        )
        .map(|cores| cores.into_iter().map(|core| core.id).collect())
        .map_err(|err| ControlPlaneError::InvalidRequest {
            message: err.to_string(),
        })
    }

    /// Reports which active cores still belong to the current committed generation.
    fn active_runtime_core_state(
        &self,
        pipeline_key: &PipelineKey,
        active_generation: u64,
    ) -> ActiveRuntimeCoreState {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut current_generation_cores = Vec::new();
        let mut has_foreign_active_generations = false;

        for (deployed_key, instance) in &state.runtime_instances {
            if deployed_key.pipeline_group_id != *pipeline_key.pipeline_group_id()
                || deployed_key.pipeline_id != *pipeline_key.pipeline_id()
                || !matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
            {
                continue;
            }

            if deployed_key.deployment_generation == active_generation {
                current_generation_cores.push(deployed_key.core_id);
            } else {
                has_foreign_active_generations = true;
            }
        }

        current_generation_cores.sort_unstable();
        ActiveRuntimeCoreState {
            current_generation_cores,
            has_foreign_active_generations,
        }
    }

    /// Compares two resolved pipelines while ignoring resource-policy differences.
    fn runtime_shape_matches_ignoring_resources(
        current: &ResolvedPipelineConfig,
        candidate: &ResolvedPipelineConfig,
    ) -> Result<bool, ControlPlaneError> {
        if current.role != candidate.role {
            return Ok(false);
        }

        let mut current_pipeline =
            serde_json::to_value(&current.pipeline).map_err(|err| ControlPlaneError::Internal {
                message: err.to_string(),
            })?;
        let mut candidate_pipeline = serde_json::to_value(&candidate.pipeline).map_err(|err| {
            ControlPlaneError::Internal {
                message: err.to_string(),
            }
        })?;
        if let serde_json::Value::Object(map) = &mut current_pipeline {
            let _ = map.remove("policies");
        }
        if let serde_json::Value::Object(map) = &mut candidate_pipeline {
            let _ = map.remove("policies");
        }

        Ok(current_pipeline == candidate_pipeline
            && current.policies.channel_capacity == candidate.policies.channel_capacity
            && current.policies.health == candidate.policies.health
            && current.policies.telemetry == candidate.policies.telemetry
            && current.policies.transport_headers == candidate.policies.transport_headers)
    }

    /// Compares two resolved pipelines for exact runtime equivalence.
    fn resolved_pipeline_matches(
        current: &ResolvedPipelineConfig,
        candidate: &ResolvedPipelineConfig,
    ) -> Result<bool, ControlPlaneError> {
        let current_pipeline =
            serde_json::to_value(&current.pipeline).map_err(|err| ControlPlaneError::Internal {
                message: err.to_string(),
            })?;
        let candidate_pipeline = serde_json::to_value(&candidate.pipeline).map_err(|err| {
            ControlPlaneError::Internal {
                message: err.to_string(),
            }
        })?;

        Ok(current.role == candidate.role
            && current_pipeline == candidate_pipeline
            && current.policies == candidate.policies)
    }

    /// Builds the effective runtime topic profile map used to reject broker mutations.
    fn pipeline_topic_profiles(
        config: &OtelDataflowSpec,
    ) -> Result<HashMap<TopicName, TopicRuntimeProfile>, ControlPlaneError> {
        let (global_names, group_names) =
            Controller::<PData>::build_declared_topic_name_maps(config).map_err(|err| {
                ControlPlaneError::InvalidRequest {
                    message: err.to_string(),
                }
            })?;
        Controller::<PData>::validate_topic_wiring_acyclic(config, &global_names, &group_names)
            .map_err(|err| ControlPlaneError::InvalidRequest {
                message: err.to_string(),
            })?;
        let (inferred_modes, _) =
            Controller::<PData>::infer_topic_modes(config, &global_names, &group_names).map_err(
                |err| ControlPlaneError::InvalidRequest {
                    message: err.to_string(),
                },
            )?;
        let default_selection_policy = config.engine.topics.impl_selection;

        let mut profiles = HashMap::new();
        for (topic_name, spec) in &config.topics {
            let declared_name = global_names
                .get(topic_name)
                .ok_or_else(|| ControlPlaneError::Internal {
                    message: format!(
                        "missing declared topic name for global topic `{}` while building runtime profiles",
                        topic_name.as_ref()
                    ),
                })?
                .clone();
            let topology_mode = inferred_modes
                .get(&declared_name)
                .copied()
                .unwrap_or(InferredTopicMode::Mixed);
            let selection_policy = spec.impl_selection.unwrap_or(default_selection_policy);
            let selected_mode = Controller::<PData>::apply_topic_impl_selection_policy(
                topology_mode,
                selection_policy,
            );
            _ = profiles.insert(
                declared_name,
                TopicRuntimeProfile {
                    backend: spec.backend,
                    policies: spec.policies.clone(),
                    selected_mode,
                },
            );
        }

        for (group_id, group_cfg) in &config.groups {
            for (topic_name, spec) in &group_cfg.topics {
                let declared_name = group_names
                    .get(&(group_id.clone(), topic_name.clone()))
                    .ok_or_else(|| ControlPlaneError::Internal {
                        message: format!(
                            "missing declared topic name for group `{}` topic `{}` while building runtime profiles",
                            group_id.as_ref(),
                            topic_name.as_ref()
                        ),
                    })?
                    .clone();
                let topology_mode = inferred_modes
                    .get(&declared_name)
                    .copied()
                    .unwrap_or(InferredTopicMode::Mixed);
                let selection_policy = spec.impl_selection.unwrap_or(default_selection_policy);
                let selected_mode = Controller::<PData>::apply_topic_impl_selection_policy(
                    topology_mode,
                    selection_policy,
                );
                _ = profiles.insert(
                    declared_name,
                    TopicRuntimeProfile {
                        backend: spec.backend,
                        policies: spec.policies.clone(),
                        selected_mode,
                    },
                );
            }
        }

        Ok(profiles)
    }

    /// Classifies a reconfigure request and prepares the rollout state machine inputs.
    fn prepare_rollout_plan(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: &ReconfigureRequest,
    ) -> Result<CandidateRolloutPlan, ControlPlaneError> {
        let pipeline_group_id: PipelineGroupId = pipeline_group_id.to_owned().into();
        let pipeline_id: PipelineId = pipeline_id.to_owned().into();
        let pipeline_key = PipelineKey::new(pipeline_group_id.clone(), pipeline_id.clone());

        let (live_config, current_record) = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !state.live_config.groups.contains_key(&pipeline_group_id) {
                return Err(ControlPlaneError::GroupNotFound);
            }
            if state.active_rollouts.contains_key(&pipeline_key)
                || state.active_shutdowns.contains_key(&pipeline_key)
            {
                return Err(ControlPlaneError::RolloutConflict);
            }
            (
                state.live_config.clone(),
                state.logical_pipelines.get(&pipeline_key).cloned(),
            )
        };

        let mut candidate_pipeline = request.pipeline.clone();
        candidate_pipeline
            .canonicalize_for_pipeline(&pipeline_group_id, &pipeline_id)
            .map_err(|err| ControlPlaneError::InvalidRequest {
                message: err.to_string(),
            })?;
        candidate_pipeline
            .validate(&pipeline_group_id, &pipeline_id)
            .map_err(|err| ControlPlaneError::InvalidRequest {
                message: err.to_string(),
            })?;

        let mut candidate_config = live_config.clone();
        let group_cfg = candidate_config
            .groups
            .get_mut(&pipeline_group_id)
            .ok_or_else(|| ControlPlaneError::Internal {
                message: format!(
                    "group `{}` disappeared while preparing rollout plan",
                    pipeline_group_id.as_ref()
                ),
            })?;
        _ = group_cfg
            .pipelines
            .insert(pipeline_id.clone(), candidate_pipeline.clone());

        candidate_config
            .validate()
            .map_err(|err| ControlPlaneError::InvalidRequest {
                message: err.to_string(),
            })?;
        Controller::<PData>::validate_engine_components_with_factory(
            self.pipeline_factory,
            &candidate_config,
        )
        .map_err(|message| ControlPlaneError::InvalidRequest { message })?;

        let current_profiles = Self::pipeline_topic_profiles(&live_config)?;
        let candidate_profiles = Self::pipeline_topic_profiles(&candidate_config)?;
        if current_profiles != candidate_profiles {
            return Err(ControlPlaneError::InvalidRequest {
                message: "request would require runtime topic broker mutation".to_owned(),
            });
        }

        let resolved_pipeline = candidate_config
            .resolve()
            .pipelines
            .into_iter()
            .find(|pipeline| {
                pipeline.role == ResolvedPipelineRole::Regular
                    && pipeline.pipeline_group_id == pipeline_group_id
                    && pipeline.pipeline_id == pipeline_id
            })
            .ok_or_else(|| ControlPlaneError::Internal {
                message: "candidate pipeline disappeared during resolution".to_owned(),
            })?;
        let current_assigned_cores = if let Some(record) = current_record.as_ref() {
            self.assigned_cores_for_resolved(&record.resolved)?
        } else {
            Vec::new()
        };
        let target_assigned_cores = self.assigned_cores_for_resolved(&resolved_pipeline)?;
        let current_core_set: HashSet<_> = current_assigned_cores.iter().copied().collect();
        let target_core_set: HashSet<_> = target_assigned_cores.iter().copied().collect();
        let active_runtime_state = current_record
            .as_ref()
            .map(|record| self.active_runtime_core_state(&pipeline_key, record.active_generation))
            .unwrap_or(ActiveRuntimeCoreState {
                current_generation_cores: Vec::new(),
                has_foreign_active_generations: false,
            });
        let active_core_set: HashSet<_> = active_runtime_state
            .current_generation_cores
            .iter()
            .copied()
            .collect();
        let common_assigned_cores: Vec<_> = target_assigned_cores
            .iter()
            .copied()
            .filter(|core_id| current_core_set.contains(core_id))
            .collect();
        let added_assigned_cores: Vec<_> = target_assigned_cores
            .iter()
            .copied()
            .filter(|core_id| !current_core_set.contains(core_id))
            .collect();
        let removed_assigned_cores: Vec<_> = current_assigned_cores
            .iter()
            .copied()
            .filter(|core_id| !target_core_set.contains(core_id))
            .collect();
        let resize_start_cores: Vec<_> = target_assigned_cores
            .iter()
            .copied()
            .filter(|core_id| !active_core_set.contains(core_id))
            .collect();
        let resize_stop_cores: Vec<_> = active_runtime_state
            .current_generation_cores
            .iter()
            .copied()
            .filter(|core_id| !target_core_set.contains(core_id))
            .collect();
        let action = if let Some(record) = current_record.as_ref() {
            let identical_update = current_assigned_cores == target_assigned_cores
                && active_runtime_state.current_generation_cores == target_assigned_cores
                && !active_runtime_state.has_foreign_active_generations
                && Self::resolved_pipeline_matches(&record.resolved, &resolved_pipeline)?;
            let resize_only = current_assigned_cores != target_assigned_cores
                && !active_runtime_state.has_foreign_active_generations
                && Self::runtime_shape_matches_ignoring_resources(
                    &record.resolved,
                    &resolved_pipeline,
                )?;
            if identical_update {
                RolloutAction::NoOp
            } else if resize_only {
                RolloutAction::Resize
            } else {
                RolloutAction::Replace
            }
        } else {
            RolloutAction::Create
        };
        let (resize_start_cores, resize_stop_cores) = match action {
            RolloutAction::Resize => (resize_start_cores, resize_stop_cores),
            RolloutAction::Create | RolloutAction::NoOp | RolloutAction::Replace => {
                (Vec::new(), Vec::new())
            }
        };
        let previous_generation = current_record
            .as_ref()
            .map(|record| record.active_generation);

        let (rollout_id, target_generation) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.active_rollouts.contains_key(&pipeline_key)
                || state.active_shutdowns.contains_key(&pipeline_key)
            {
                return Err(ControlPlaneError::RolloutConflict);
            }
            let rollout_id = format!("rollout-{}", state.next_rollout_id);
            state.next_rollout_id += 1;
            let target_generation = match action {
                RolloutAction::NoOp | RolloutAction::Resize => {
                    previous_generation.ok_or_else(|| ControlPlaneError::Internal {
                        message: format!(
                            "rollout planner produced {:?} for {}:{} without a current generation",
                            action,
                            pipeline_key.pipeline_group_id().as_ref(),
                            pipeline_key.pipeline_id().as_ref()
                        ),
                    })?
                }
                RolloutAction::Create | RolloutAction::Replace => {
                    let generation_counter = state
                        .generation_counters
                        .entry(pipeline_key.clone())
                        .or_insert(0);
                    let target_generation = *generation_counter;
                    *generation_counter += 1;
                    target_generation
                }
            };
            (rollout_id, target_generation)
        };

        let rollout_core_ids = match action {
            RolloutAction::NoOp => Vec::new(),
            RolloutAction::Resize => {
                let mut ids = resize_start_cores.clone();
                let additional_stop_cores: Vec<_> = resize_stop_cores
                    .iter()
                    .copied()
                    .filter(|core_id| !ids.contains(core_id))
                    .collect();
                ids.extend(additional_stop_cores);
                ids
            }
            RolloutAction::Create | RolloutAction::Replace => {
                let mut ids = target_assigned_cores.clone();
                ids.extend(removed_assigned_cores.iter().copied());
                ids
            }
        };
        let cores = rollout_core_ids
            .into_iter()
            .map(|core_id| RolloutCoreProgress {
                core_id,
                previous_generation: match action {
                    RolloutAction::Create => None,
                    RolloutAction::NoOp => active_core_set
                        .contains(&core_id)
                        .then_some(previous_generation)
                        .flatten(),
                    RolloutAction::Replace => current_core_set
                        .contains(&core_id)
                        .then_some(previous_generation)
                        .flatten(),
                    RolloutAction::Resize => active_core_set
                        .contains(&core_id)
                        .then_some(previous_generation)
                        .flatten(),
                },
                target_generation,
                state: "pending".to_owned(),
                updated_at: timestamp_now(),
                detail: None,
            })
            .collect();
        let rollout = RolloutRecord::new(
            rollout_id,
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            action,
            target_generation,
            current_record
                .as_ref()
                .map(|record| record.active_generation),
            cores,
        );

        Ok(CandidateRolloutPlan {
            pipeline_key,
            pipeline_group_id,
            pipeline_id,
            action,
            resolved_pipeline,
            current_record,
            current_assigned_cores,
            target_assigned_cores,
            common_assigned_cores,
            added_assigned_cores,
            removed_assigned_cores,
            resize_start_cores,
            resize_stop_cores,
            target_generation,
            rollout,
            step_timeout_secs: request.step_timeout_secs.max(1),
            drain_timeout_secs: request.drain_timeout_secs.max(1),
        })
    }

    /// Registers a newly accepted rollout and publishes its initial summary.
    fn insert_rollout(
        &self,
        pipeline_key: &PipelineKey,
        rollout: RolloutRecord,
    ) -> Result<(), ControlPlaneError> {
        self.prune_retained_operation_history();
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.active_rollouts.contains_key(pipeline_key)
                || state.active_shutdowns.contains_key(pipeline_key)
            {
                return Err(ControlPlaneError::RolloutConflict);
            }
            _ = state
                .active_rollouts
                .insert(pipeline_key.clone(), rollout.rollout_id.clone());
            _ = state
                .rollouts
                .insert(rollout.rollout_id.clone(), rollout.clone());
        }
        self.observed_state_store
            .set_pipeline_rollout_summary(pipeline_key.clone(), rollout.summary());
        Ok(())
    }

    /// Applies an in-place update to a rollout record and refreshes observed state.
    fn update_rollout<F>(&self, pipeline_key: &PipelineKey, rollout_id: &str, update: F)
    where
        F: FnOnce(&mut RolloutRecord),
    {
        let summary = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(rollout) = state.rollouts.get_mut(rollout_id) else {
                return;
            };
            update(rollout);
            rollout.updated_at = timestamp_now();
            let is_terminal = rollout.state.is_terminal();
            let summary = rollout.summary();
            if is_terminal {
                Self::record_terminal_rollout_locked(
                    &mut state,
                    pipeline_key,
                    rollout_id,
                    Instant::now(),
                );
            }
            summary
        };
        self.observed_state_store
            .set_pipeline_rollout_summary(pipeline_key.clone(), summary);
    }

    /// Updates the per-core progress entry for a rollout.
    fn update_rollout_core_state(
        &self,
        pipeline_key: &PipelineKey,
        rollout_id: &str,
        core_id: usize,
        state: &str,
        detail: Option<String>,
    ) {
        self.update_rollout(pipeline_key, rollout_id, |rollout| {
            if let Some(core) = rollout
                .cores
                .iter_mut()
                .find(|core| core.core_id == core_id)
            {
                core.state = state.to_owned();
                core.updated_at = timestamp_now();
                core.detail = detail;
            }
        });
    }

    /// Marks a rollout inactive and prunes any no-longer-needed retained state.
    fn finish_rollout(&self, pipeline_key: &PipelineKey, rollout_id: &str) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state
                .active_rollouts
                .get(pipeline_key)
                .is_some_and(|id| id == rollout_id)
            {
                let _ = state.active_rollouts.remove(pipeline_key);
            }
        }
        self.prune_pipeline_runtime_and_history(pipeline_key);
    }

    /// Emits a structured controller panic log without mutating observed instance state.
    fn report_controller_worker_panic(
        &self,
        pipeline_key: &PipelineKey,
        operation_kind: &'static str,
        operation_id: &str,
        report: &PanicReport,
    ) {
        let ErrorSummary::Pipeline {
            error_kind,
            message,
            source,
        } = report.error_summary()
        else {
            unreachable!("panic reports are always pipeline-level summaries");
        };

        otel_error!(
            "controller.worker_panic",
            pipeline_group_id = %pipeline_key.pipeline_group_id(),
            pipeline_id = %pipeline_key.pipeline_id(),
            operation_kind = operation_kind,
            operation_id = operation_id,
            error_kind = error_kind.as_str(),
            message = message.as_str(),
            source = source.as_deref().unwrap_or(""),
        );
    }

    /// Forces rollout terminal cleanup when the detached rollout worker panics.
    fn handle_rollout_worker_panic(
        &self,
        pipeline_key: &PipelineKey,
        rollout_id: &str,
        thread_name: String,
        panic: Box<dyn Any + Send>,
    ) {
        let report = PanicReport::capture("rollout worker", panic, Some(thread_name), None, None);
        let failure_reason = report.summary_message();
        self.update_rollout(pipeline_key, rollout_id, |rollout| {
            rollout.state = RolloutLifecycleState::Failed;
            rollout.failure_reason = Some(failure_reason.clone());
        });
        self.report_controller_worker_panic(pipeline_key, "rollout", rollout_id, &report);
        self.finish_rollout(pipeline_key, rollout_id);
    }

    /// Returns the latest rollout snapshot, evicting expired history first.
    fn rollout_status_snapshot(&self, rollout_id: &str) -> Option<RolloutStatus> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::prune_terminal_operation_history_locked(&mut state, Instant::now());
        state.rollouts.get(rollout_id).map(RolloutRecord::status)
    }

    /// Clears temporary serving-generation overrides after a rollout settles.
    fn clear_pipeline_serving_generations<I>(&self, pipeline_key: &PipelineKey, core_ids: I)
    where
        I: IntoIterator<Item = usize>,
    {
        for core_id in core_ids {
            self.observed_state_store
                .clear_pipeline_serving_generation(pipeline_key.clone(), core_id);
        }
    }

    /// Commits the winning pipeline config and active generation into runtime state.
    fn commit_pipeline_record(&self, plan: &CandidateRolloutPlan, active_generation: u64) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(group_cfg) = state.live_config.groups.get_mut(&plan.pipeline_group_id) {
                _ = group_cfg.pipelines.insert(
                    plan.pipeline_id.clone(),
                    plan.resolved_pipeline.pipeline.clone(),
                );
            }
            _ = state.logical_pipelines.insert(
                plan.pipeline_key.clone(),
                LogicalPipelineRecord {
                    resolved: plan.resolved_pipeline.clone(),
                    active_generation,
                },
            );
        }
        self.observed_state_store
            .set_pipeline_active_generation(plan.pipeline_key.clone(), active_generation);
    }

    /// Selects the active instances targeted by a per-pipeline shutdown request.
    fn prepare_shutdown_plan(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<CandidateShutdownPlan, ControlPlaneError> {
        let pipeline_group_id: PipelineGroupId = pipeline_group_id.to_owned().into();
        let pipeline_id: PipelineId = pipeline_id.to_owned().into();
        let pipeline_key = PipelineKey::new(pipeline_group_id.clone(), pipeline_id.clone());

        let target_instances = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !state.live_config.groups.contains_key(&pipeline_group_id) {
                return Err(ControlPlaneError::GroupNotFound);
            }
            if !state.logical_pipelines.contains_key(&pipeline_key) {
                return Err(ControlPlaneError::PipelineNotFound);
            }
            if state.active_rollouts.contains_key(&pipeline_key)
                || state.active_shutdowns.contains_key(&pipeline_key)
            {
                return Err(ControlPlaneError::RolloutConflict);
            }

            let targets: Vec<_> = state
                .runtime_instances
                .iter()
                .filter_map(|(deployed_key, instance)| {
                    if deployed_key.pipeline_group_id == pipeline_group_id
                        && deployed_key.pipeline_id == pipeline_id
                        && matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                    {
                        Some(deployed_key.clone())
                    } else {
                        None
                    }
                })
                .collect();
            if targets.is_empty() {
                return Err(ControlPlaneError::InvalidRequest {
                    message: format!(
                        "pipeline {}:{} is already stopped",
                        pipeline_group_id.as_ref(),
                        pipeline_id.as_ref()
                    ),
                });
            }
            targets
        };

        let shutdown_id = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.active_rollouts.contains_key(&pipeline_key)
                || state.active_shutdowns.contains_key(&pipeline_key)
            {
                return Err(ControlPlaneError::RolloutConflict);
            }
            let shutdown_id = format!("shutdown-{}", state.next_shutdown_id);
            state.next_shutdown_id += 1;
            shutdown_id
        };

        let shutdown = ShutdownRecord::new(
            shutdown_id,
            pipeline_group_id,
            pipeline_id,
            target_instances
                .iter()
                .map(|instance| ShutdownCoreProgress {
                    core_id: instance.core_id,
                    deployment_generation: instance.deployment_generation,
                    state: "pending".to_owned(),
                    updated_at: timestamp_now(),
                    detail: None,
                })
                .collect(),
        );

        Ok(CandidateShutdownPlan {
            pipeline_key,
            shutdown,
            target_instances,
            timeout_secs: timeout_secs.max(1),
        })
    }

    /// Registers a newly accepted shutdown operation.
    fn insert_shutdown(
        &self,
        pipeline_key: &PipelineKey,
        shutdown: ShutdownRecord,
    ) -> Result<(), ControlPlaneError> {
        self.prune_retained_operation_history();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.active_rollouts.contains_key(pipeline_key)
            || state.active_shutdowns.contains_key(pipeline_key)
        {
            return Err(ControlPlaneError::RolloutConflict);
        }
        _ = state
            .active_shutdowns
            .insert(pipeline_key.clone(), shutdown.shutdown_id.clone());
        _ = state
            .shutdowns
            .insert(shutdown.shutdown_id.clone(), shutdown);
        Ok(())
    }

    /// Applies an in-place update to a shutdown record and prunes on completion.
    fn update_shutdown<F>(&self, pipeline_key: &PipelineKey, shutdown_id: &str, update: F)
    where
        F: FnOnce(&mut ShutdownRecord),
    {
        let should_prune = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(shutdown) = state.shutdowns.get_mut(shutdown_id) else {
                return;
            };
            update(shutdown);
            shutdown.updated_at = timestamp_now();
            let is_terminal = shutdown.state.is_terminal();
            if is_terminal {
                Self::record_terminal_shutdown_locked(
                    &mut state,
                    pipeline_key,
                    shutdown_id,
                    Instant::now(),
                );
                let _ = state.active_shutdowns.remove(pipeline_key);
                true
            } else {
                false
            }
        };

        if should_prune {
            self.prune_pipeline_runtime_and_history(pipeline_key);
        }
    }

    /// Returns the latest shutdown snapshot, evicting expired history first.
    fn shutdown_status_snapshot(&self, shutdown_id: &str) -> Option<ShutdownStatus> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::prune_terminal_operation_history_locked(&mut state, Instant::now());
        state.shutdowns.get(shutdown_id).map(ShutdownRecord::status)
    }

    /// Forces shutdown terminal cleanup when the detached shutdown worker panics.
    fn handle_shutdown_worker_panic(
        &self,
        pipeline_key: &PipelineKey,
        shutdown_id: &str,
        thread_name: String,
        panic: Box<dyn Any + Send>,
    ) {
        let report = PanicReport::capture("shutdown worker", panic, Some(thread_name), None, None);
        let failure_reason = report.summary_message();
        self.update_shutdown(pipeline_key, shutdown_id, |shutdown| {
            shutdown.state = ShutdownLifecycleState::Failed;
            shutdown.failure_reason = Some(failure_reason.clone());
        });
        self.report_controller_worker_panic(pipeline_key, "shutdown", shutdown_id, &report);
    }

    /// Returns the committed pipeline config plus any currently active rollout summary.
    fn pipeline_details_snapshot(
        &self,
        pipeline_key: &PipelineKey,
    ) -> Result<Option<PipelineDetails>, ControlPlaneError> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(record) = state.logical_pipelines.get(pipeline_key) else {
            if !state
                .live_config
                .groups
                .contains_key(pipeline_key.pipeline_group_id())
            {
                return Err(ControlPlaneError::GroupNotFound);
            }
            return Ok(None);
        };
        let rollout = state
            .active_rollouts
            .get(pipeline_key)
            .and_then(|rollout_id| state.rollouts.get(rollout_id))
            .map(RolloutRecord::api_summary);
        Ok(Some(PipelineDetails {
            pipeline_group_id: pipeline_key.pipeline_group_id().clone(),
            pipeline_id: pipeline_key.pipeline_id().clone(),
            active_generation: Some(record.active_generation),
            pipeline: record.resolved.pipeline.clone(),
            rollout,
        }))
    }

    /// Records a rollout and launches its background execution worker.
    fn spawn_rollout(
        self: &Arc<Self>,
        plan: CandidateRolloutPlan,
    ) -> Result<RolloutStatus, ControlPlaneError> {
        let rollout_id = plan.rollout.rollout_id.clone();
        let pipeline_key = plan.pipeline_key.clone();
        self.insert_rollout(&pipeline_key, plan.rollout.clone())?;
        if matches!(plan.action, RolloutAction::NoOp) {
            self.commit_pipeline_record(&plan, plan.target_generation);
            self.update_rollout(&pipeline_key, &rollout_id, |rollout| {
                rollout.state = RolloutLifecycleState::Succeeded;
                rollout.failure_reason = None;
            });
            self.finish_rollout(&pipeline_key, &rollout_id);
            return self.rollout_status_snapshot(&rollout_id).ok_or_else(|| {
                ControlPlaneError::Internal {
                    message: format!("rollout {rollout_id} disappeared before response"),
                }
            });
        }

        let initial_status = plan.rollout.status();
        let runtime = Arc::clone(self);
        let rollout_runtime = Arc::clone(&runtime);
        let rollout_cleanup_runtime = Arc::clone(&runtime);
        let worker_pipeline_key = pipeline_key.clone();
        let worker_rollout_id = rollout_id.clone();
        let worker_thread_name = format!(
            "rollout-{}-{}",
            pipeline_key.pipeline_group_id().as_ref(),
            pipeline_key.pipeline_id().as_ref()
        );
        let _rollout_handle = thread::Builder::new()
            .name(worker_thread_name.clone())
            .spawn(move || {
                if let Err(panic) =
                    catch_unwind(AssertUnwindSafe(|| rollout_runtime.run_rollout(plan)))
                {
                    rollout_cleanup_runtime.handle_rollout_worker_panic(
                        &worker_pipeline_key,
                        &worker_rollout_id,
                        worker_thread_name,
                        panic,
                    );
                }
            })
            .map_err(|err| {
                runtime.finish_rollout(&pipeline_key, &rollout_id);
                ControlPlaneError::Internal {
                    message: err.to_string(),
                }
            })?;
        Ok(initial_status)
    }

    /// Records a shutdown and launches its background execution worker.
    fn spawn_shutdown(
        self: &Arc<Self>,
        plan: CandidateShutdownPlan,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        let shutdown_id = plan.shutdown.shutdown_id.clone();
        let pipeline_key = plan.pipeline_key.clone();
        let initial_status = plan.shutdown.status();
        self.insert_shutdown(&pipeline_key, plan.shutdown.clone())?;
        let runtime = Arc::clone(self);
        let shutdown_runtime = Arc::clone(&runtime);
        let shutdown_cleanup_runtime = Arc::clone(&runtime);
        let worker_pipeline_key = pipeline_key.clone();
        let worker_shutdown_id = shutdown_id.clone();
        let worker_thread_name = format!(
            "shutdown-{}-{}",
            pipeline_key.pipeline_group_id().as_ref(),
            pipeline_key.pipeline_id().as_ref()
        );
        let _shutdown_handle = thread::Builder::new()
            .name(worker_thread_name.clone())
            .spawn(move || {
                if let Err(panic) =
                    catch_unwind(AssertUnwindSafe(|| shutdown_runtime.run_shutdown(plan)))
                {
                    shutdown_cleanup_runtime.handle_shutdown_worker_panic(
                        &worker_pipeline_key,
                        &worker_shutdown_id,
                        worker_thread_name,
                        panic,
                    );
                }
            })
            .map_err(|err| {
                runtime.update_shutdown(&pipeline_key, &shutdown_id, |shutdown| {
                    shutdown.state = ShutdownLifecycleState::Failed;
                    shutdown.failure_reason = Some(err.to_string());
                });
                ControlPlaneError::Internal {
                    message: err.to_string(),
                }
            })?;
        Ok(initial_status)
    }

    /// Drives one rollout operation from running to a terminal state.
    fn run_rollout(self: Arc<Self>, plan: CandidateRolloutPlan) {
        self.update_rollout(&plan.pipeline_key, &plan.rollout.rollout_id, |rollout| {
            rollout.state = RolloutLifecycleState::Running;
        });

        let result = match plan.action {
            RolloutAction::Create => self.run_create_rollout(&plan),
            RolloutAction::NoOp => Ok(()),
            RolloutAction::Replace => self.run_replace_rollout(&plan),
            RolloutAction::Resize => self.run_resize_rollout(&plan),
        };

        match result {
            Ok(()) => {
                self.update_rollout(&plan.pipeline_key, &plan.rollout.rollout_id, |rollout| {
                    rollout.state = RolloutLifecycleState::Succeeded;
                    rollout.failure_reason = None;
                });
            }
            Err(RolloutExecutionError::Failed(reason)) => {
                self.update_rollout(&plan.pipeline_key, &plan.rollout.rollout_id, |rollout| {
                    rollout.state = RolloutLifecycleState::Failed;
                    rollout.failure_reason = Some(reason);
                });
            }
            Err(RolloutExecutionError::RollbackFailed(reason)) => {
                self.update_rollout(&plan.pipeline_key, &plan.rollout.rollout_id, |rollout| {
                    rollout.state = RolloutLifecycleState::RollbackFailed;
                    rollout.failure_reason = Some(reason);
                });
            }
        }

        self.finish_rollout(&plan.pipeline_key, &plan.rollout.rollout_id);
    }

    /// Drives one pipeline shutdown operation to completion or failure.
    fn run_shutdown(self: Arc<Self>, plan: CandidateShutdownPlan) {
        self.update_shutdown(&plan.pipeline_key, &plan.shutdown.shutdown_id, |shutdown| {
            shutdown.state = ShutdownLifecycleState::Running;
        });

        for deployed_key in &plan.target_instances {
            if let Err(message) =
                self.request_instance_shutdown(deployed_key, plan.timeout_secs, "pipeline shutdown")
            {
                self.update_shutdown(&plan.pipeline_key, &plan.shutdown.shutdown_id, |shutdown| {
                    shutdown.state = ShutdownLifecycleState::Failed;
                    shutdown.failure_reason = Some(message.clone());
                    if let Some(core) = shutdown.cores.iter_mut().find(|core| {
                        core.core_id == deployed_key.core_id
                            && core.deployment_generation == deployed_key.deployment_generation
                    }) {
                        core.state = "failed".to_owned();
                        core.updated_at = timestamp_now();
                        core.detail = Some(message.clone());
                    }
                });
                return;
            }

            self.update_shutdown(&plan.pipeline_key, &plan.shutdown.shutdown_id, |shutdown| {
                if let Some(core) = shutdown.cores.iter_mut().find(|core| {
                    core.core_id == deployed_key.core_id
                        && core.deployment_generation == deployed_key.deployment_generation
                }) {
                    core.state = "shutdown_requested".to_owned();
                    core.updated_at = timestamp_now();
                }
            });
        }

        let deadline = Instant::now() + Duration::from_secs(plan.timeout_secs);
        let mut remaining: HashSet<_> = plan.target_instances.iter().cloned().collect();
        while !remaining.is_empty() {
            let mut completed = Vec::new();
            for deployed_key in &remaining {
                match self.instance_exit(deployed_key) {
                    Some(RuntimeInstanceExit::Success) => {
                        completed.push(deployed_key.clone());
                    }
                    Some(RuntimeInstanceExit::Error(error)) => {
                        self.update_shutdown(
                            &plan.pipeline_key,
                            &plan.shutdown.shutdown_id,
                            |shutdown| {
                                shutdown.state = ShutdownLifecycleState::Failed;
                                shutdown.failure_reason = Some(error.message.clone());
                                if let Some(core) = shutdown.cores.iter_mut().find(|core| {
                                    core.core_id == deployed_key.core_id
                                        && core.deployment_generation
                                            == deployed_key.deployment_generation
                                }) {
                                    core.state = "failed".to_owned();
                                    core.updated_at = timestamp_now();
                                    core.detail = Some(error.message.clone());
                                }
                            },
                        );
                        return;
                    }
                    None => {}
                }
            }

            for deployed_key in completed {
                let _ = remaining.remove(&deployed_key);
                self.update_shutdown(&plan.pipeline_key, &plan.shutdown.shutdown_id, |shutdown| {
                    if let Some(core) = shutdown.cores.iter_mut().find(|core| {
                        core.core_id == deployed_key.core_id
                            && core.deployment_generation == deployed_key.deployment_generation
                    }) {
                        core.state = "exited".to_owned();
                        core.updated_at = timestamp_now();
                    }
                });
            }

            if remaining.is_empty() {
                break;
            }

            if Instant::now() >= deadline {
                let failure_reason = remaining
                    .iter()
                    .next()
                    .map(|deployed_key| {
                        format!(
                            "timed out waiting for pipeline {}:{} core={} generation={} to drain",
                            deployed_key.pipeline_group_id.as_ref(),
                            deployed_key.pipeline_id.as_ref(),
                            deployed_key.core_id,
                            deployed_key.deployment_generation
                        )
                    })
                    .unwrap_or_else(|| "shutdown timed out".to_owned());
                self.update_shutdown(&plan.pipeline_key, &plan.shutdown.shutdown_id, |shutdown| {
                    shutdown.state = ShutdownLifecycleState::Failed;
                    shutdown.failure_reason = Some(failure_reason.clone());
                    for deployed_key in &remaining {
                        if let Some(core) = shutdown.cores.iter_mut().find(|core| {
                            core.core_id == deployed_key.core_id
                                && core.deployment_generation == deployed_key.deployment_generation
                        }) {
                            core.state = "failed".to_owned();
                            core.updated_at = timestamp_now();
                            core.detail = Some(failure_reason.clone());
                        }
                    }
                });
                return;
            }

            thread::sleep(Duration::from_millis(50));
        }

        self.update_shutdown(&plan.pipeline_key, &plan.shutdown.shutdown_id, |shutdown| {
            shutdown.state = ShutdownLifecycleState::Succeeded;
        });
    }

    /// Creates a brand-new logical pipeline by launching all target instances.
    fn run_create_rollout(
        self: &Arc<Self>,
        plan: &CandidateRolloutPlan,
    ) -> Result<(), RolloutExecutionError> {
        let mut launched = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(plan.step_timeout_secs);
        for core_id in &plan.target_assigned_cores {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "starting",
                None,
            );
            let deployed_key = self
                .launch_regular_pipeline_instance(
                    &plan.resolved_pipeline,
                    *core_id,
                    plan.target_generation,
                )
                .map_err(|err| RolloutExecutionError::Failed(err.to_string()))?;
            launched.push(deployed_key);
        }

        for deployed_key in &launched {
            self.wait_for_pipeline_ready(deployed_key, deadline)
                .map_err(|reason| {
                    let _ = self.shutdown_instances(&launched, plan.drain_timeout_secs);
                    RolloutExecutionError::Failed(reason)
                })?;
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                deployed_key.core_id,
                "ready",
                None,
            );
        }

        self.commit_pipeline_record(plan, plan.target_generation);
        Ok(())
    }

    /// Resizes a pipeline when only core allocation changed and common cores stay untouched.
    fn run_resize_rollout(
        self: &Arc<Self>,
        plan: &CandidateRolloutPlan,
    ) -> Result<(), RolloutExecutionError> {
        let Some(current_record) = plan.current_record.as_ref() else {
            return Err(RolloutExecutionError::Failed(
                "internal error: resize rollout missing current record".to_owned(),
            ));
        };
        let active_generation = current_record.active_generation;
        let mut started_cores = Vec::new();
        let mut retired_cores = Vec::new();

        for core_id in &plan.resize_start_cores {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "starting",
                None,
            );

            let new_key = self
                .launch_regular_pipeline_instance(
                    &plan.resolved_pipeline,
                    *core_id,
                    active_generation,
                )
                .map_err(|err| RolloutExecutionError::Failed(err.to_string()))?;
            let ready_deadline = Instant::now() + Duration::from_secs(plan.step_timeout_secs);
            if let Err(reason) = self.wait_for_pipeline_ready(&new_key, ready_deadline) {
                let _ = self.shutdown_instances(&[new_key], plan.drain_timeout_secs);
                return self.rollback_resize_rollout(plan, &started_cores, &retired_cores, reason);
            }

            started_cores.push(*core_id);
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "started",
                None,
            );
        }

        for core_id in &plan.resize_stop_cores {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "draining_old",
                None,
            );

            let old_key = DeployedPipelineKey {
                pipeline_group_id: plan.pipeline_group_id.clone(),
                pipeline_id: plan.pipeline_id.clone(),
                core_id: *core_id,
                deployment_generation: active_generation,
            };
            if let Err(reason) =
                self.shutdown_instance(&old_key, plan.drain_timeout_secs, "resize rollout drain")
            {
                return self.rollback_resize_rollout(plan, &started_cores, &retired_cores, reason);
            }

            retired_cores.push(*core_id);
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "retired",
                None,
            );
        }

        self.commit_pipeline_record(plan, active_generation);
        self.clear_pipeline_serving_generations(
            &plan.pipeline_key,
            plan.current_assigned_cores
                .iter()
                .chain(plan.target_assigned_cores.iter())
                .copied(),
        );
        Ok(())
    }

    /// Performs the serial rolling cutover used for topology or config changes.
    fn run_replace_rollout(
        self: &Arc<Self>,
        plan: &CandidateRolloutPlan,
    ) -> Result<(), RolloutExecutionError> {
        let Some(previous) = plan.current_record.as_ref() else {
            return Err(RolloutExecutionError::Failed(
                "internal error: replace rollout missing current record".to_owned(),
            ));
        };
        let previous_generation = previous.active_generation;
        for core_id in &plan.current_assigned_cores {
            self.observed_state_store.set_pipeline_serving_generation(
                plan.pipeline_key.clone(),
                *core_id,
                previous_generation,
            );
        }

        let mut activated_added_cores = Vec::new();
        let mut switched_common_cores = Vec::new();
        let mut retired_removed_cores = Vec::new();

        for core_id in &plan.added_assigned_cores {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "starting",
                None,
            );

            let new_key = self
                .launch_regular_pipeline_instance(
                    &plan.resolved_pipeline,
                    *core_id,
                    plan.target_generation,
                )
                .map_err(|err| RolloutExecutionError::Failed(err.to_string()))?;
            let ready_deadline = Instant::now() + Duration::from_secs(plan.step_timeout_secs);
            if let Err(reason) = self.wait_for_pipeline_ready(&new_key, ready_deadline) {
                let _ = self.shutdown_instances(&[new_key], plan.drain_timeout_secs);
                return self.rollback_replace_rollout(
                    plan,
                    &switched_common_cores,
                    &activated_added_cores,
                    &retired_removed_cores,
                    reason,
                );
            }

            self.observed_state_store.set_pipeline_serving_generation(
                plan.pipeline_key.clone(),
                *core_id,
                plan.target_generation,
            );
            activated_added_cores.push(*core_id);
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "switched",
                None,
            );
        }

        for core_id in &plan.common_assigned_cores {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "starting",
                None,
            );

            let new_key = self
                .launch_regular_pipeline_instance(
                    &plan.resolved_pipeline,
                    *core_id,
                    plan.target_generation,
                )
                .map_err(|err| RolloutExecutionError::Failed(err.to_string()))?;
            let ready_deadline = Instant::now() + Duration::from_secs(plan.step_timeout_secs);
            if let Err(reason) = self.wait_for_pipeline_ready(&new_key, ready_deadline) {
                let _ = self.shutdown_instances(&[new_key], plan.drain_timeout_secs);
                return self.rollback_replace_rollout(
                    plan,
                    &switched_common_cores,
                    &activated_added_cores,
                    &retired_removed_cores,
                    reason,
                );
            }

            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "draining_old",
                None,
            );

            let old_key = DeployedPipelineKey {
                pipeline_group_id: plan.pipeline_group_id.clone(),
                pipeline_id: plan.pipeline_id.clone(),
                core_id: *core_id,
                deployment_generation: previous_generation,
            };
            if let Err(reason) =
                self.shutdown_instance(&old_key, plan.drain_timeout_secs, "rolling cutover drain")
            {
                let _ = self.shutdown_instances(&[new_key], plan.drain_timeout_secs);
                return self.rollback_replace_rollout(
                    plan,
                    &switched_common_cores,
                    &activated_added_cores,
                    &retired_removed_cores,
                    reason,
                );
            }

            self.observed_state_store.set_pipeline_serving_generation(
                plan.pipeline_key.clone(),
                *core_id,
                plan.target_generation,
            );
            switched_common_cores.push(*core_id);
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "switched",
                None,
            );
        }

        for core_id in &plan.removed_assigned_cores {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "draining_old",
                None,
            );

            let old_key = DeployedPipelineKey {
                pipeline_group_id: plan.pipeline_group_id.clone(),
                pipeline_id: plan.pipeline_id.clone(),
                core_id: *core_id,
                deployment_generation: previous_generation,
            };
            if let Err(reason) = self.shutdown_instance(
                &old_key,
                plan.drain_timeout_secs,
                "resource policy rollout drain",
            ) {
                return self.rollback_replace_rollout(
                    plan,
                    &switched_common_cores,
                    &activated_added_cores,
                    &retired_removed_cores,
                    reason,
                );
            }

            self.observed_state_store
                .clear_pipeline_serving_generation(plan.pipeline_key.clone(), *core_id);
            retired_removed_cores.push(*core_id);
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "retired",
                None,
            );
        }

        self.commit_pipeline_record(plan, plan.target_generation);
        self.clear_pipeline_serving_generations(
            &plan.pipeline_key,
            plan.current_assigned_cores
                .iter()
                .chain(plan.target_assigned_cores.iter())
                .copied(),
        );
        Ok(())
    }

    /// Restores the prior core footprint after a resize rollout fails.
    fn rollback_resize_rollout(
        self: &Arc<Self>,
        plan: &CandidateRolloutPlan,
        started_cores: &[usize],
        retired_cores: &[usize],
        failure_reason: String,
    ) -> Result<(), RolloutExecutionError> {
        self.update_rollout(&plan.pipeline_key, &plan.rollout.rollout_id, |rollout| {
            rollout.state = RolloutLifecycleState::RollingBack;
            rollout.failure_reason = Some(failure_reason.clone());
        });
        let Some(previous) = plan.current_record.as_ref() else {
            return Err(RolloutExecutionError::RollbackFailed(
                "internal error: resize rollback missing current record".to_owned(),
            ));
        };
        let previous_generation = previous.active_generation;

        for core_id in retired_cores.iter().rev() {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rollback_starting",
                None,
            );

            let old_key = self
                .launch_regular_pipeline_instance(&previous.resolved, *core_id, previous_generation)
                .map_err(|err| RolloutExecutionError::RollbackFailed(err.to_string()))?;
            let ready_deadline = Instant::now() + Duration::from_secs(plan.step_timeout_secs);
            self.wait_for_pipeline_ready(&old_key, ready_deadline)
                .map_err(RolloutExecutionError::RollbackFailed)?;
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rolled_back",
                None,
            );
        }

        for core_id in started_cores.iter().rev() {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rollback_starting",
                None,
            );

            let new_key = DeployedPipelineKey {
                pipeline_group_id: plan.pipeline_group_id.clone(),
                pipeline_id: plan.pipeline_id.clone(),
                core_id: *core_id,
                deployment_generation: previous_generation,
            };
            self.shutdown_instance(&new_key, plan.drain_timeout_secs, "rollback cleanup")
                .map_err(RolloutExecutionError::RollbackFailed)?;
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rolled_back",
                None,
            );
        }

        self.clear_pipeline_serving_generations(
            &plan.pipeline_key,
            plan.current_assigned_cores
                .iter()
                .chain(plan.target_assigned_cores.iter())
                .copied(),
        );
        Err(RolloutExecutionError::Failed(failure_reason))
    }

    /// Restores the previous serving generation after a replace rollout fails.
    fn rollback_replace_rollout(
        self: &Arc<Self>,
        plan: &CandidateRolloutPlan,
        switched_common_cores: &[usize],
        activated_added_cores: &[usize],
        retired_removed_cores: &[usize],
        failure_reason: String,
    ) -> Result<(), RolloutExecutionError> {
        self.update_rollout(&plan.pipeline_key, &plan.rollout.rollout_id, |rollout| {
            rollout.state = RolloutLifecycleState::RollingBack;
            rollout.failure_reason = Some(failure_reason.clone());
        });
        let Some(previous) = plan.current_record.as_ref() else {
            return Err(RolloutExecutionError::RollbackFailed(
                "internal error: replace rollback missing current record".to_owned(),
            ));
        };
        let previous_generation = previous.active_generation;

        for core_id in retired_removed_cores.iter().rev() {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rollback_starting",
                None,
            );

            let old_key = self
                .launch_regular_pipeline_instance(&previous.resolved, *core_id, previous_generation)
                .map_err(|err| RolloutExecutionError::RollbackFailed(err.to_string()))?;
            let ready_deadline = Instant::now() + Duration::from_secs(plan.step_timeout_secs);
            self.wait_for_pipeline_ready(&old_key, ready_deadline)
                .map_err(RolloutExecutionError::RollbackFailed)?;
            self.observed_state_store.set_pipeline_serving_generation(
                plan.pipeline_key.clone(),
                *core_id,
                previous_generation,
            );
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rolled_back",
                None,
            );
        }

        for core_id in switched_common_cores.iter().rev() {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rollback_starting",
                None,
            );

            let old_key = self
                .launch_regular_pipeline_instance(&previous.resolved, *core_id, previous_generation)
                .map_err(|err| RolloutExecutionError::RollbackFailed(err.to_string()))?;
            let ready_deadline = Instant::now() + Duration::from_secs(plan.step_timeout_secs);
            self.wait_for_pipeline_ready(&old_key, ready_deadline)
                .map_err(RolloutExecutionError::RollbackFailed)?;

            let new_key = DeployedPipelineKey {
                pipeline_group_id: plan.pipeline_group_id.clone(),
                pipeline_id: plan.pipeline_id.clone(),
                core_id: *core_id,
                deployment_generation: plan.target_generation,
            };
            self.shutdown_instance(&new_key, plan.drain_timeout_secs, "rollback drain")
                .map_err(RolloutExecutionError::RollbackFailed)?;
            self.observed_state_store.set_pipeline_serving_generation(
                plan.pipeline_key.clone(),
                *core_id,
                previous_generation,
            );
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rolled_back",
                None,
            );
        }

        for core_id in activated_added_cores.iter().rev() {
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rollback_starting",
                None,
            );

            let new_key = DeployedPipelineKey {
                pipeline_group_id: plan.pipeline_group_id.clone(),
                pipeline_id: plan.pipeline_id.clone(),
                core_id: *core_id,
                deployment_generation: plan.target_generation,
            };
            self.shutdown_instance(&new_key, plan.drain_timeout_secs, "rollback cleanup")
                .map_err(RolloutExecutionError::RollbackFailed)?;
            self.observed_state_store
                .clear_pipeline_serving_generation(plan.pipeline_key.clone(), *core_id);
            self.update_rollout_core_state(
                &plan.pipeline_key,
                &plan.rollout.rollout_id,
                *core_id,
                "rolled_back",
                None,
            );
        }
        self.clear_pipeline_serving_generations(
            &plan.pipeline_key,
            plan.current_assigned_cores
                .iter()
                .chain(plan.target_assigned_cores.iter())
                .copied(),
        );
        Err(RolloutExecutionError::Failed(failure_reason))
    }

    /// Best-effort cleanup helper for batches of launched candidate instances.
    fn shutdown_instances(
        self: &Arc<Self>,
        keys: &[DeployedPipelineKey],
        timeout_secs: u64,
    ) -> Result<(), String> {
        for key in keys {
            self.shutdown_instance(key, timeout_secs, "candidate cleanup")?;
        }
        Ok(())
    }

    /// Launches one regular pipeline instance on a specific core and generation.
    fn launch_regular_pipeline_instance(
        self: &Arc<Self>,
        resolved_pipeline: &ResolvedPipelineConfig,
        core_id: usize,
        deployment_generation: u64,
    ) -> Result<DeployedPipelineKey, Error> {
        let thread_id = self.next_thread_id();
        let num_cores = self
            .assigned_cores_for_resolved(resolved_pipeline)
            .map_err(|err| Error::PipelineRuntimeError {
                source: Box::new(io::Error::other(format!("{err:?}"))),
            })?
            .len();
        let deployed_key = DeployedPipelineKey {
            pipeline_group_id: resolved_pipeline.pipeline_group_id.clone(),
            pipeline_id: resolved_pipeline.pipeline_id.clone(),
            core_id,
            deployment_generation,
        };
        let launched = Controller::<PData>::launch_pipeline_thread(
            self.pipeline_factory,
            deployed_key.clone(),
            CoreId { id: core_id },
            num_cores,
            resolved_pipeline.pipeline.clone(),
            resolved_pipeline.policies.channel_capacity.clone(),
            resolved_pipeline.policies.telemetry.clone(),
            resolved_pipeline.policies.transport_headers.clone(),
            self.controller_context.clone(),
            self.metrics_reporter.clone(),
            self.engine_event_reporter.clone(),
            self.engine_tracing_setup.clone(),
            self.telemetry_reporting_interval,
            self.memory_pressure_tx.clone(),
            &self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .live_config,
            &self.declared_topics,
            thread_id,
            None,
        )?;
        self.register_launched_instance(launched);
        Ok(deployed_key)
    }

    /// Registers a launched instance and starts a watcher for its terminal exit.
    pub(super) fn register_launched_instance(
        self: &Arc<Self>,
        launched: LaunchedPipelineThread<PData>,
    ) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            _ = state.runtime_instances.insert(
                launched.pipeline_key.clone(),
                RuntimeInstanceRecord {
                    control_sender: Some(launched.control_sender.clone()),
                    lifecycle: RuntimeInstanceLifecycle::Active,
                },
            );
            state.active_instances += 1;
            self.state_changed.notify_all();
        }

        let runtime = Arc::downgrade(self);
        let _ = thread::Builder::new()
            .name(format!("watcher-{}", launched.thread_name))
            .spawn(move || {
                let exit = match launched.join_handle.join() {
                    Ok(Ok(_)) => RuntimeInstanceExit::Success,
                    Ok(Err(err)) => {
                        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime(err.to_string()))
                    }
                    Err(panic) => RuntimeInstanceExit::Error(RuntimeInstanceError::from_panic(
                        PanicReport::capture(
                            "runtime thread",
                            panic,
                            Some(launched.thread_name.clone()),
                            Some(launched.thread_id),
                            Some(launched.pipeline_key.core_id),
                        ),
                    )),
                };
                if let Some(runtime) = Weak::upgrade(&runtime) {
                    runtime.note_instance_exit(launched.pipeline_key, exit);
                }
            });
    }

    /// Records a pipeline instance exit and wakes any waiters observing it.
    fn note_instance_exit(&self, pipeline_key: DeployedPipelineKey, exit: RuntimeInstanceExit) {
        match &exit {
            RuntimeInstanceExit::Success => {
                self.engine_event_reporter
                    .report(EngineEvent::drained(pipeline_key.clone(), None));
            }
            RuntimeInstanceExit::Error(error) => {
                self.engine_event_reporter
                    .report(EngineEvent::pipeline_runtime_error(
                        pipeline_key.clone(),
                        "Pipeline encountered a runtime error.",
                        error.error_summary(),
                    ));
            }
        }

        let should_compact = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(instance) = state.runtime_instances.get_mut(&pipeline_key) {
                instance.lifecycle = RuntimeInstanceLifecycle::Exited(exit.clone());
            }
            state.active_instances = state.active_instances.saturating_sub(1);
            if let RuntimeInstanceExit::Error(error) = &exit {
                if state.first_error.is_none() {
                    state.first_error = Some(error.message.clone());
                }
            }
            let logical_pipeline_key = PipelineKey::new(
                pipeline_key.pipeline_group_id.clone(),
                pipeline_key.pipeline_id.clone(),
            );
            Self::prune_exited_runtime_instances_for_pipeline_locked(
                &mut state,
                &logical_pipeline_key,
            )
        };
        if should_compact {
            let logical_pipeline_key = PipelineKey::new(
                pipeline_key.pipeline_group_id.clone(),
                pipeline_key.pipeline_id.clone(),
            );
            self.observed_state_store
                .compact_pipeline_instances(&logical_pipeline_key);
        }
        self.state_changed.notify_all();
    }

    /// Waits for a specific deployed instance to report admitted plus ready.
    fn wait_for_pipeline_ready(
        &self,
        deployed_key: &DeployedPipelineKey,
        deadline: Instant,
    ) -> Result<(), String> {
        let pipeline_key = PipelineKey::new(
            deployed_key.pipeline_group_id.clone(),
            deployed_key.pipeline_id.clone(),
        );
        loop {
            if let Some(status) = self.observed_state_handle.pipeline_status(&pipeline_key) {
                if let Some(instance) =
                    status.instance_status(deployed_key.core_id, deployed_key.deployment_generation)
                {
                    let accepted = instance.accepted_condition().status == ConditionStatus::True;
                    let ready = instance.ready_condition().status == ConditionStatus::True;
                    if accepted && ready {
                        return Ok(());
                    }
                    match instance.phase() {
                        PipelinePhase::Failed(_)
                        | PipelinePhase::Rejected(_)
                        | PipelinePhase::Deleted
                        | PipelinePhase::Stopped => {
                            return Err(format!(
                                "pipeline failed to become ready on core {} (generation {})",
                                deployed_key.core_id, deployed_key.deployment_generation
                            ));
                        }
                        _ => {}
                    }
                }
            }

            if let Some(exit) = self.instance_exit(deployed_key) {
                return match exit {
                    RuntimeInstanceExit::Success => Err(format!(
                        "pipeline exited before reporting ready on core {} (generation {})",
                        deployed_key.core_id, deployed_key.deployment_generation
                    )),
                    RuntimeInstanceExit::Error(error) => Err(error.message),
                };
            }

            if Instant::now() >= deadline {
                return Err(format!(
                    "timed out waiting for admitted+ready on core {} (generation {})",
                    deployed_key.core_id, deployed_key.deployment_generation
                ));
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Returns the terminal exit result for one deployed instance, if any.
    fn instance_exit(&self, deployed_key: &DeployedPipelineKey) -> Option<RuntimeInstanceExit> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state
            .runtime_instances
            .get(deployed_key)
            .and_then(|instance| match &instance.lifecycle {
                RuntimeInstanceLifecycle::Active => None,
                RuntimeInstanceLifecycle::Exited(exit) => Some(exit.clone()),
            })
    }

    /// Sends shutdown to one instance and releases the retained control sender.
    fn request_instance_shutdown(
        &self,
        deployed_key: &DeployedPipelineKey,
        timeout_secs: u64,
        reason: &str,
    ) -> Result<(), String> {
        let sender = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(instance) = state.runtime_instances.get(deployed_key) else {
                return Err(format!(
                    "pipeline instance {}:{} core={} generation={} is not registered",
                    deployed_key.pipeline_group_id.as_ref(),
                    deployed_key.pipeline_id.as_ref(),
                    deployed_key.core_id,
                    deployed_key.deployment_generation
                ));
            };

            match &instance.lifecycle {
                RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Success) => return Ok(()),
                RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Error(error)) => {
                    return Err(error.message.clone());
                }
                RuntimeInstanceLifecycle::Active => {}
            }

            instance.control_sender.clone().ok_or_else(|| {
                format!(
                    "shutdown already requested for pipeline {}:{} core={} generation={}",
                    deployed_key.pipeline_group_id.as_ref(),
                    deployed_key.pipeline_id.as_ref(),
                    deployed_key.core_id,
                    deployed_key.deployment_generation
                )
            })?
        };

        if let Err(err) = sender.try_send_shutdown(
            Instant::now() + Duration::from_secs(timeout_secs.max(1)),
            reason.to_owned(),
        ) {
            return match self.instance_exit(deployed_key) {
                Some(RuntimeInstanceExit::Success) => Ok(()),
                Some(RuntimeInstanceExit::Error(error)) => Err(error.message),
                None => Err(err.to_string()),
            };
        }
        self.release_instance_control_sender(deployed_key);
        Ok(())
    }

    /// Waits until a specific deployed instance exits or the deadline expires.
    fn wait_for_instance_exit(
        &self,
        deployed_key: &DeployedPipelineKey,
        deadline: Instant,
    ) -> Result<(), String> {
        loop {
            if let Some(exit) = self.instance_exit(deployed_key) {
                return match exit {
                    RuntimeInstanceExit::Success => Ok(()),
                    RuntimeInstanceExit::Error(error) => Err(error.message),
                };
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "timed out waiting for pipeline {}:{} core={} generation={} to drain",
                    deployed_key.pipeline_group_id.as_ref(),
                    deployed_key.pipeline_id.as_ref(),
                    deployed_key.core_id,
                    deployed_key.deployment_generation
                ));
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Requests shutdown for one instance and waits until it exits.
    fn shutdown_instance(
        &self,
        deployed_key: &DeployedPipelineKey,
        timeout_secs: u64,
        reason: &str,
    ) -> Result<(), String> {
        self.request_instance_shutdown(deployed_key, timeout_secs, reason)?;
        self.wait_for_instance_exit(
            deployed_key,
            Instant::now() + Duration::from_secs(timeout_secs.max(1)),
        )
    }

    /// Drops the retained admin sender so draining can observe channel closure.
    fn release_instance_control_sender(&self, deployed_key: &DeployedPipelineKey) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(instance) = state.runtime_instances.get_mut(deployed_key) {
            instance.control_sender = None;
        }
    }

    /// Broadcasts shutdown to every currently active runtime instance.
    fn request_shutdown_all(&self, timeout_secs: u64) -> Result<(), ControlPlaneError> {
        let senders: Vec<_> = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state
                .runtime_instances
                .iter()
                .filter_map(|(deployed_key, instance)| match instance.lifecycle {
                    RuntimeInstanceLifecycle::Active => instance
                        .control_sender
                        .as_ref()
                        .map(|sender| (deployed_key.clone(), sender.clone())),
                    RuntimeInstanceLifecycle::Exited(_) => None,
                })
                .collect()
        };

        for (deployed_key, sender) in senders {
            sender
                .try_send_shutdown(
                    Instant::now() + Duration::from_secs(timeout_secs.max(1)),
                    "global shutdown".to_owned(),
                )
                .map_err(|err| ControlPlaneError::Internal {
                    message: err.to_string(),
                })?;
            self.release_instance_control_sender(&deployed_key);
        }
        Ok(())
    }

    /// Starts a tracked shutdown operation for one logical pipeline.
    fn request_shutdown_pipeline(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        let plan = self.prepare_shutdown_plan(pipeline_group_id, pipeline_id, timeout_secs)?;
        self.spawn_shutdown(plan)
    }

    /// Blocks until all active runtime instances have exited.
    pub(super) fn wait_until_all_instances_exit(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.active_instances > 0 {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Returns the first runtime error observed by any watched pipeline thread.
    pub(super) fn take_runtime_error(&self) -> Option<Error> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state
            .first_error
            .as_ref()
            .map(|message| Error::PipelineRuntimeError {
                source: Box::new(io::Error::other(message.clone())),
            })
    }
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable>
    ControlPlane for ControllerControlPlane<PData>
{
    fn shutdown_all(&self, timeout_secs: u64) -> Result<(), ControlPlaneError> {
        self.runtime.request_shutdown_all(timeout_secs)
    }

    fn shutdown_pipeline(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        self.runtime
            .request_shutdown_pipeline(pipeline_group_id, pipeline_id, timeout_secs)
    }

    fn reconfigure_pipeline(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: ReconfigureRequest,
    ) -> Result<RolloutStatus, ControlPlaneError> {
        let plan = self
            .runtime
            .prepare_rollout_plan(pipeline_group_id, pipeline_id, &request)?;
        self.runtime.spawn_rollout(plan)
    }

    fn pipeline_details(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<PipelineDetails>, ControlPlaneError> {
        self.runtime.pipeline_details_snapshot(&PipelineKey::new(
            pipeline_group_id.to_owned().into(),
            pipeline_id.to_owned().into(),
        ))
    }

    fn rollout_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        rollout_id: &str,
    ) -> Result<Option<RolloutStatus>, ControlPlaneError> {
        let expected_key = PipelineKey::new(
            pipeline_group_id.to_owned().into(),
            pipeline_id.to_owned().into(),
        );
        let Some(status) = self.runtime.rollout_status_snapshot(rollout_id) else {
            return Ok(None);
        };
        let actual_key =
            PipelineKey::new(status.pipeline_group_id.clone(), status.pipeline_id.clone());
        if actual_key != expected_key {
            return Err(ControlPlaneError::RolloutNotFound);
        }
        Ok(Some(status))
    }

    fn shutdown_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        shutdown_id: &str,
    ) -> Result<Option<ShutdownStatus>, ControlPlaneError> {
        let expected_key = PipelineKey::new(
            pipeline_group_id.to_owned().into(),
            pipeline_id.to_owned().into(),
        );
        let Some(status) = self.runtime.shutdown_status_snapshot(shutdown_id) else {
            return Ok(None);
        };
        let actual_key =
            PipelineKey::new(status.pipeline_group_id.clone(), status.pipeline_id.clone());
        if actual_key != expected_key {
            return Err(ControlPlaneError::ShutdownNotFound);
        }
        Ok(Some(status))
    }
}

#[derive(Debug)]
enum RolloutExecutionError {
    Failed(String),
    RollbackFailed(String),
}

#[cfg(test)]
#[path = "live_control_tests.rs"]
mod tests;
