// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP Dataflow Engine Controller
//!
//! This controller is responsible for deploying, managing, and monitoring pipeline groups
//! within the current process.
//!
//! Each pipeline configuration declares its CPU requirements through
//! `policies.resources.core_allocation`.
//! Based on this policy, the controller allocates CPU cores and spawns one dedicated
//! thread per assigned core. Threads are pinned to distinct CPU cores, following a
//! strict thread-per-core model.
//!
//! A pipeline deployed on `n` cores results in `n` worker threads. Hot data paths are
//! fully contained within each thread to maximize CPU cache locality and minimize
//! cross-thread contention. Inter-thread communication is restricted to control
//! messages and internal telemetry only.
//!
//! By default, pipelines are expected to run on dedicated CPU cores. It is possible
//! to deploy multiple pipeline configurations on the same cores, primarily for
//! consolidation, testing, or transitional deployments. This comes at the cost of
//! reduced efficiency, especially cache locality. Even in this mode, pipeline
//! instances run in independent threads and do not share mutable data structures.
//!
//! Pipelines do not perform implicit work stealing, dynamic scheduling, or automatic
//! load balancing across threads. Any form of cross-pipeline or cross-thread data
//! exchange must be explicitly modeled.
//!
//! In the future, controller-managed named channels will be introduced as the
//! recommended mechanism to implement explicit load balancing and routing schemes
//! within the engine. These channels will complement the existing SO_REUSEPORT-based
//! load balancing mechanism already supported at the receiver level on operating
//! systems that provide it.
//!
//! Pipelines can be gracefully shut down by sending control messages through their
//! control channels.
//!
//! Future work includes:
//! - TODO: Complete status and health checks for pipelines
//! - TODO: Auto-restart threads in case of panic
//! - TODO: Live pipeline updates
//! - TODO: Better resource control

use crate::error::Error;
use crate::thread_task::spawn_thread_local_task;
use chrono::Utc;
use core_affinity::CoreId;
use otap_df_admin::{
    ControlPlane, ControlPlaneError, PipelineDetails, PipelineRolloutStatus,
    PipelineShutdownStatus, ReplacePipelineRequest, RolloutCoreStatus, ShutdownCoreStatus,
};
use otap_df_config::engine::{
    OtelDataflowSpec, ResolvedPipelineConfig, ResolvedPipelineRole,
    SYSTEM_OBSERVABILITY_PIPELINE_ID, SYSTEM_PIPELINE_GROUP_ID,
};
use otap_df_config::node::{NodeKind, NodeUserConfig};
use otap_df_config::policy::{ChannelCapacityPolicy, CoreAllocation, TelemetryPolicy};
use otap_df_config::topic::{
    TopicAckPropagationMode, TopicBackendKind, TopicBroadcastOnLagPolicy, TopicImplSelectionPolicy,
    TopicSpec,
};
use otap_df_config::{
    DeployedPipelineKey, PipelineGroupId, PipelineId, PipelineKey, SubscriptionGroupName,
    TopicName, pipeline::PipelineConfig,
};
use otap_df_engine::PipelineFactory;
use otap_df_engine::ReceivedAtNode;
use otap_df_engine::Unwindable;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    PipelineAdminSender, PipelineCtrlMsgReceiver, PipelineCtrlMsgSender, pipeline_ctrl_msg_channel,
};
use otap_df_engine::entity_context::{
    node_entity_key, pipeline_entity_key, set_pipeline_entity_key,
};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::topic::{
    InMemoryBackend, PipelineTopicBinding, TopicBroker, TopicOptions, TopicPublishOutcomeConfig,
    TopicSet,
};
use otap_df_state::conditions::ConditionStatus;
use otap_df_state::phase::PipelinePhase;
use otap_df_state::pipeline_status::{PipelineRolloutState, PipelineRolloutSummary};
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::event::{EngineEvent, ErrorSummary, ObservedEventReporter};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry::{
    InternalTelemetrySettings, InternalTelemetrySystem, TracingSetup, otel_error, otel_info,
    otel_info_span, otel_warn, self_tracing::LogContext,
};
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

/// Error types and helpers for the controller module.
pub mod error;
/// Utilities to spawn async tasks on dedicated threads with graceful shutdown.
pub mod thread_task;

/// Controller for managing pipelines in a thread-per-core model.
///
/// # Thread Safety
/// This struct is designed to be used in multi-threaded contexts. Each pipeline is run on a
/// dedicated thread pinned to a CPU core.
/// Intended for use as a long-lived process controller.
pub struct Controller<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    /// The pipeline factory used to build runtime pipelines.
    pipeline_factory: &'static PipelineFactory<PData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    ParkMainThread,
    ShutdownWhenDone,
}

struct DeclaredTopics<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    broker: TopicBroker<PData>,
    global_names: HashMap<TopicName, TopicName>,
    group_names: HashMap<(PipelineGroupId, TopicName), TopicName>,
    inferred_mode_reports: Vec<InferredTopicModeReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InferredTopicMode {
    Mixed,
    BalancedOnly,
    BroadcastOnly,
}

impl InferredTopicMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::BalancedOnly => "balanced_only",
            Self::BroadcastOnly => "broadcast_only",
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
    const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::RollingBack => "rolling_back",
            Self::RollbackFailed => "rollback_failed",
        }
    }

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
}

impl RolloutRecord {
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
        }
    }

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

    fn status(&self) -> PipelineRolloutStatus {
        PipelineRolloutStatus {
            rollout_id: self.rollout_id.clone(),
            pipeline_group_id: self.pipeline_group_id.clone(),
            pipeline_id: self.pipeline_id.clone(),
            action: self.action.as_str().to_owned(),
            state: self.state.as_str().to_owned(),
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
}

impl ShutdownRecord {
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
        }
    }

    fn status(&self) -> PipelineShutdownStatus {
        PipelineShutdownStatus {
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
    Error(String),
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
    shutdowns: HashMap<String, ShutdownRecord>,
    active_shutdowns: HashMap<PipelineKey, String>,
    generation_counters: HashMap<PipelineKey, u64>,
    active_instances: usize,
    next_rollout_id: u64,
    next_shutdown_id: u64,
    next_thread_id: usize,
    first_error: Option<String>,
}

struct ControllerRuntime<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    pipeline_factory: &'static PipelineFactory<PData>,
    controller_context: ControllerContext,
    observed_state_store: ObservedStateStore,
    observed_state_handle: otap_df_state::store::ObservedStateHandle,
    engine_event_reporter: ObservedEventReporter,
    metrics_reporter: MetricsReporter,
    declared_topics: DeclaredTopics<PData>,
    available_core_ids: Vec<CoreId>,
    engine_tracing_setup: TracingSetup,
    state: Mutex<ControllerRuntimeState>,
    state_changed: Condvar,
}

struct ControllerControlPlane<PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    runtime: Arc<ControllerRuntime<PData>>,
}

struct LaunchedPipelineThread<PData> {
    thread_name: String,
    thread_id: usize,
    pipeline_key: DeployedPipelineKey,
    control_sender: Arc<dyn PipelineAdminSender>,
    join_handle: thread::JoinHandle<Result<Vec<()>, Error>>,
    _marker: std::marker::PhantomData<PData>,
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

fn timestamp_now() -> String {
    Utc::now().to_rfc3339()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TopicBackendCapabilities {
    supports_balanced_only: bool,
    supports_broadcast_only: bool,
    supports_mixed: bool,
    supports_broadcast_on_lag_drop_oldest: bool,
    supports_broadcast_on_lag_disconnect: bool,
    supports_ack_propagation_disabled: bool,
    supports_ack_propagation_auto: bool,
}

impl TopicBackendCapabilities {
    const fn supports_mode(self, mode: InferredTopicMode) -> bool {
        match mode {
            InferredTopicMode::BalancedOnly => self.supports_balanced_only,
            InferredTopicMode::BroadcastOnly => self.supports_broadcast_only,
            InferredTopicMode::Mixed => self.supports_mixed,
        }
    }

    const fn supports_broadcast_on_lag(self, policy: TopicBroadcastOnLagPolicy) -> bool {
        match policy {
            TopicBroadcastOnLagPolicy::DropOldest => self.supports_broadcast_on_lag_drop_oldest,
            TopicBroadcastOnLagPolicy::Disconnect => self.supports_broadcast_on_lag_disconnect,
        }
    }

    const fn supports_ack_propagation(self, policy: TopicAckPropagationMode) -> bool {
        match policy {
            TopicAckPropagationMode::Disabled => self.supports_ack_propagation_disabled,
            TopicAckPropagationMode::Auto => self.supports_ack_propagation_auto,
        }
    }
}

const fn broadcast_on_lag_policy_value(policy: TopicBroadcastOnLagPolicy) -> &'static str {
    match policy {
        TopicBroadcastOnLagPolicy::DropOldest => "drop_oldest",
        TopicBroadcastOnLagPolicy::Disconnect => "disconnect",
    }
}

const fn ack_propagation_policy_value(policy: TopicAckPropagationMode) -> &'static str {
    match policy {
        TopicAckPropagationMode::Disabled => "disabled",
        TopicAckPropagationMode::Auto => "auto",
    }
}

#[derive(Debug, Default)]
struct TopicUsageSummary {
    receiver_refs: usize,
    exporter_refs: usize,
    has_broadcast_receivers: bool,
    balanced_groups: HashSet<SubscriptionGroupName>,
    has_unknown_receiver_mode: bool,
}

#[derive(Debug)]
enum TopicReceiverMode {
    Broadcast,
    Balanced(SubscriptionGroupName),
    Unknown,
}

#[derive(Debug, Clone)]
struct InferredTopicModeReport {
    topic: TopicName,
    topology_mode: InferredTopicMode,
    selected_mode: InferredTopicMode,
    selection_policy: TopicImplSelectionPolicy,
    receiver_refs: usize,
    exporter_refs: usize,
    balanced_group_count: usize,
    has_broadcast_receivers: bool,
    has_unknown_receiver_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TopicWiringVertex {
    PipelineNode {
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        node_id: otap_df_config::NodeId,
    },
    Topic {
        declared_name: TopicName,
    },
}

impl TopicWiringVertex {
    fn label(&self) -> String {
        match self {
            Self::PipelineNode {
                pipeline_group_id,
                pipeline_id,
                node_id,
            } => format!(
                "pipeline:{}/{}/{}",
                pipeline_group_id.as_ref(),
                pipeline_id.as_ref(),
                node_id.as_ref()
            ),
            Self::Topic { declared_name } => format!("topic:{}", declared_name.as_ref()),
        }
    }
}

/// Returns the set of entity keys relevant to this context.
fn engine_context() -> LogContext {
    if let Some(node) = node_entity_key() {
        smallvec![node]
    } else if let Some(pipeline) = pipeline_entity_key() {
        smallvec![pipeline]
    } else {
        smallvec![]
    }
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable>
    ControllerRuntime<PData>
{
    #[allow(clippy::too_many_arguments)]
    fn new(
        pipeline_factory: &'static PipelineFactory<PData>,
        controller_context: ControllerContext,
        observed_state_store: ObservedStateStore,
        observed_state_handle: otap_df_state::store::ObservedStateHandle,
        engine_event_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        declared_topics: DeclaredTopics<PData>,
        available_core_ids: Vec<CoreId>,
        engine_tracing_setup: TracingSetup,
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
            state: Mutex::new(ControllerRuntimeState {
                live_config,
                logical_pipelines: HashMap::new(),
                runtime_instances: HashMap::new(),
                rollouts: HashMap::new(),
                active_rollouts: HashMap::new(),
                shutdowns: HashMap::new(),
                active_shutdowns: HashMap::new(),
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

    fn register_committed_pipeline(&self, resolved: ResolvedPipelineConfig, generation: u64) {
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

    fn next_thread_id(&self) -> usize {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let thread_id = state.next_thread_id;
        state.next_thread_id += 1;
        thread_id
    }

    fn assigned_cores_for_resolved(
        &self,
        resolved_pipeline: &ResolvedPipelineConfig,
    ) -> Result<Vec<usize>, ControlPlaneError> {
        Controller::<PData>::select_cores_for_allocation(
            self.available_core_ids.clone(),
            &resolved_pipeline
                .policies
                .effective_resources()
                .core_allocation,
        )
        .map(|cores| cores.into_iter().map(|core| core.id).collect())
        .map_err(|err| ControlPlaneError::InvalidRequest {
            message: err.to_string(),
        })
    }

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

        let mut current_policies =
            serde_json::to_value(&current.policies).map_err(|err| ControlPlaneError::Internal {
                message: err.to_string(),
            })?;
        let mut candidate_policies = serde_json::to_value(&candidate.policies).map_err(|err| {
            ControlPlaneError::Internal {
                message: err.to_string(),
            }
        })?;
        if let serde_json::Value::Object(map) = &mut current_policies {
            let _ = map.remove("resources");
        }
        if let serde_json::Value::Object(map) = &mut candidate_policies {
            let _ = map.remove("resources");
        }

        Ok(current_pipeline == candidate_pipeline && current_policies == candidate_policies)
    }

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
            Controller::<PData>::infer_topic_modes(config, &global_names, &group_names);
        let default_selection_policy = config.engine.topics.impl_selection;

        let mut profiles = HashMap::new();
        for (topic_name, spec) in &config.topics {
            let declared_name = global_names
                .get(topic_name)
                .expect("global topic declaration must resolve")
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
                    .expect("group topic declaration must resolve")
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

    fn prepare_rollout_plan(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: &ReplacePipelineRequest,
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
            .expect("group existence checked above");
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
                RolloutAction::NoOp | RolloutAction::Resize => previous_generation
                    .expect("noop and resize rollouts always have a current generation"),
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

    fn insert_rollout(
        &self,
        pipeline_key: &PipelineKey,
        rollout: RolloutRecord,
    ) -> Result<(), ControlPlaneError> {
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
            rollout.summary()
        };
        self.observed_state_store
            .set_pipeline_rollout_summary(pipeline_key.clone(), summary);
    }

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

    fn finish_rollout(&self, pipeline_key: &PipelineKey, rollout_id: &str) {
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

    fn rollout_status_snapshot(&self, rollout_id: &str) -> Option<PipelineRolloutStatus> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.rollouts.get(rollout_id).map(RolloutRecord::status)
    }

    fn clear_pipeline_serving_generations<I>(&self, pipeline_key: &PipelineKey, core_ids: I)
    where
        I: IntoIterator<Item = usize>,
    {
        for core_id in core_ids {
            self.observed_state_store
                .clear_pipeline_serving_generation(pipeline_key.clone(), core_id);
        }
    }

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

    fn insert_shutdown(
        &self,
        pipeline_key: &PipelineKey,
        shutdown: ShutdownRecord,
    ) -> Result<(), ControlPlaneError> {
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

    fn update_shutdown<F>(&self, pipeline_key: &PipelineKey, shutdown_id: &str, update: F)
    where
        F: FnOnce(&mut ShutdownRecord),
    {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(shutdown) = state.shutdowns.get_mut(shutdown_id) else {
            return;
        };
        update(shutdown);
        shutdown.updated_at = timestamp_now();
        if matches!(
            shutdown.state,
            ShutdownLifecycleState::Succeeded | ShutdownLifecycleState::Failed
        ) {
            let _ = state.active_shutdowns.remove(pipeline_key);
        }
    }

    fn shutdown_status_snapshot(&self, shutdown_id: &str) -> Option<PipelineShutdownStatus> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.shutdowns.get(shutdown_id).map(ShutdownRecord::status)
    }

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
            .map(RolloutRecord::summary);
        Ok(Some(PipelineDetails {
            pipeline_group_id: pipeline_key.pipeline_group_id().clone(),
            pipeline_id: pipeline_key.pipeline_id().clone(),
            active_generation: Some(record.active_generation),
            pipeline: record.resolved.pipeline.clone(),
            rollout,
        }))
    }

    fn spawn_rollout(
        self: &Arc<Self>,
        plan: CandidateRolloutPlan,
    ) -> Result<PipelineRolloutStatus, ControlPlaneError> {
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
        let _rollout_handle = thread::Builder::new()
            .name(format!(
                "rollout-{}-{}",
                pipeline_key.pipeline_group_id().as_ref(),
                pipeline_key.pipeline_id().as_ref()
            ))
            .spawn(move || {
                rollout_runtime.run_rollout(plan);
            })
            .map_err(|err| {
                runtime.finish_rollout(&pipeline_key, &rollout_id);
                ControlPlaneError::Internal {
                    message: err.to_string(),
                }
            })?;
        Ok(initial_status)
    }

    fn spawn_shutdown(
        self: &Arc<Self>,
        plan: CandidateShutdownPlan,
    ) -> Result<PipelineShutdownStatus, ControlPlaneError> {
        let shutdown_id = plan.shutdown.shutdown_id.clone();
        let pipeline_key = plan.pipeline_key.clone();
        let initial_status = plan.shutdown.status();
        self.insert_shutdown(&pipeline_key, plan.shutdown.clone())?;
        let runtime = Arc::clone(self);
        let shutdown_runtime = Arc::clone(&runtime);
        let _shutdown_handle = thread::Builder::new()
            .name(format!(
                "shutdown-{}-{}",
                pipeline_key.pipeline_group_id().as_ref(),
                pipeline_key.pipeline_id().as_ref()
            ))
            .spawn(move || {
                shutdown_runtime.run_shutdown(plan);
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
                    Some(RuntimeInstanceExit::Error(message)) => {
                        self.update_shutdown(
                            &plan.pipeline_key,
                            &plan.shutdown.shutdown_id,
                            |shutdown| {
                                shutdown.state = ShutdownLifecycleState::Failed;
                                shutdown.failure_reason = Some(message.clone());
                                if let Some(core) = shutdown.cores.iter_mut().find(|core| {
                                    core.core_id == deployed_key.core_id
                                        && core.deployment_generation
                                            == deployed_key.deployment_generation
                                }) {
                                    core.state = "failed".to_owned();
                                    core.updated_at = timestamp_now();
                                    core.detail = Some(message.clone());
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

    fn run_resize_rollout(
        self: &Arc<Self>,
        plan: &CandidateRolloutPlan,
    ) -> Result<(), RolloutExecutionError> {
        let active_generation = plan
            .current_record
            .as_ref()
            .expect("resize rollouts always have a current record")
            .active_generation;
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

    fn run_replace_rollout(
        self: &Arc<Self>,
        plan: &CandidateRolloutPlan,
    ) -> Result<(), RolloutExecutionError> {
        let previous = plan
            .current_record
            .as_ref()
            .expect("replace rollouts always have a current record");
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
        let previous = plan
            .current_record
            .as_ref()
            .expect("resize rollouts always have a current record");
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
        let previous = plan
            .current_record
            .as_ref()
            .expect("replace rollouts always have a current record");
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
            self.controller_context.clone(),
            self.metrics_reporter.clone(),
            self.engine_event_reporter.clone(),
            self.engine_tracing_setup.clone(),
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

    fn register_launched_instance(self: &Arc<Self>, launched: LaunchedPipelineThread<PData>) {
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
                    Ok(Err(err)) => RuntimeInstanceExit::Error(err.to_string()),
                    Err(panic) => RuntimeInstanceExit::Error(format!(
                        "thread {} (id {}) panicked: {panic:?}",
                        launched.thread_name, launched.thread_id
                    )),
                };
                if let Some(runtime) = Weak::upgrade(&runtime) {
                    runtime.note_instance_exit(launched.pipeline_key, exit);
                }
            });
    }

    fn note_instance_exit(&self, pipeline_key: DeployedPipelineKey, exit: RuntimeInstanceExit) {
        match &exit {
            RuntimeInstanceExit::Success => {
                self.engine_event_reporter
                    .report(EngineEvent::drained(pipeline_key.clone(), None));
            }
            RuntimeInstanceExit::Error(message) => {
                let err_summary = ErrorSummary::Pipeline {
                    error_kind: "runtime".into(),
                    message: message.clone(),
                    source: None,
                };
                self.engine_event_reporter
                    .report(EngineEvent::pipeline_runtime_error(
                        pipeline_key.clone(),
                        "Pipeline encountered a runtime error.",
                        err_summary,
                    ));
            }
        }

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(instance) = state.runtime_instances.get_mut(&pipeline_key) {
            instance.lifecycle = RuntimeInstanceLifecycle::Exited(exit.clone());
        }
        state.active_instances = state.active_instances.saturating_sub(1);
        if let RuntimeInstanceExit::Error(message) = &exit {
            if state.first_error.is_none() {
                state.first_error = Some(message.clone());
            }
        }
        self.state_changed.notify_all();
    }

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
                    RuntimeInstanceExit::Error(message) => Err(message),
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
                RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Error(message)) => {
                    return Err(message.clone());
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
                Some(RuntimeInstanceExit::Error(message)) => Err(message),
                None => Err(err.to_string()),
            };
        }
        self.release_instance_control_sender(deployed_key);
        Ok(())
    }

    fn wait_for_instance_exit(
        &self,
        deployed_key: &DeployedPipelineKey,
        deadline: Instant,
    ) -> Result<(), String> {
        loop {
            if let Some(exit) = self.instance_exit(deployed_key) {
                return match exit {
                    RuntimeInstanceExit::Success => Ok(()),
                    RuntimeInstanceExit::Error(message) => Err(message),
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

    fn release_instance_control_sender(&self, deployed_key: &DeployedPipelineKey) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(instance) = state.runtime_instances.get_mut(deployed_key) {
            instance.control_sender = None;
        }
    }

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

    fn request_shutdown_pipeline(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<PipelineShutdownStatus, ControlPlaneError> {
        let plan = self.prepare_shutdown_plan(pipeline_group_id, pipeline_id, timeout_secs)?;
        self.spawn_shutdown(plan)
    }

    fn wait_until_all_instances_exit(&self) {
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

    fn take_runtime_error(&self) -> Option<Error> {
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
    ) -> Result<PipelineShutdownStatus, ControlPlaneError> {
        self.runtime
            .request_shutdown_pipeline(pipeline_group_id, pipeline_id, timeout_secs)
    }

    fn replace_pipeline(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: ReplacePipelineRequest,
    ) -> Result<PipelineRolloutStatus, ControlPlaneError> {
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
    ) -> Result<Option<PipelineRolloutStatus>, ControlPlaneError> {
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
    ) -> Result<Option<PipelineShutdownStatus>, ControlPlaneError> {
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

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable>
    Controller<PData>
{
    /// Creates a new controller with the given pipeline factory.
    pub const fn new(pipeline_factory: &'static PipelineFactory<PData>) -> Self {
        Self { pipeline_factory }
    }

    fn validate_pipeline_components_with_factory(
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        pipeline_cfg: &PipelineConfig,
    ) -> Result<(), String> {
        for (node_id, node_cfg) in pipeline_cfg.node_iter() {
            let urn_str = node_cfg.r#type.as_str();
            let validate_config_fn = match node_cfg.kind() {
                NodeKind::Receiver => pipeline_factory
                    .get_receiver_factory_map()
                    .get(urn_str)
                    .map(|factory| factory.validate_config),
                NodeKind::Processor | NodeKind::ProcessorChain => pipeline_factory
                    .get_processor_factory_map()
                    .get(urn_str)
                    .map(|factory| factory.validate_config),
                NodeKind::Exporter => pipeline_factory
                    .get_exporter_factory_map()
                    .get(urn_str)
                    .map(|factory| factory.validate_config),
            };

            let Some(validate_fn) = validate_config_fn else {
                let kind_name = match node_cfg.kind() {
                    NodeKind::Receiver => "receiver",
                    NodeKind::Processor | NodeKind::ProcessorChain => "processor",
                    NodeKind::Exporter => "exporter",
                };
                return Err(format!(
                    "Unknown {} component `{}` in pipeline_group={} pipeline={} node={}",
                    kind_name,
                    urn_str,
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    node_id.as_ref()
                ));
            };

            validate_fn(&node_cfg.config).map_err(|err| {
                format!(
                    "Invalid config for component `{}` in pipeline_group={} pipeline={} node={}: {}",
                    urn_str,
                    pipeline_group_id.as_ref(),
                    pipeline_id.as_ref(),
                    node_id.as_ref(),
                    err
                )
            })?;
        }
        Ok(())
    }

    fn validate_engine_components_with_factory(
        pipeline_factory: &'static PipelineFactory<PData>,
        engine_cfg: &OtelDataflowSpec,
    ) -> Result<(), String> {
        for (pipeline_group_id, pipeline_group) in &engine_cfg.groups {
            for (pipeline_id, pipeline_cfg) in &pipeline_group.pipelines {
                Self::validate_pipeline_components_with_factory(
                    pipeline_factory,
                    pipeline_group_id,
                    pipeline_id,
                    pipeline_cfg,
                )?;
            }
        }

        if let Some(obs_pipeline) = &engine_cfg.engine.observability.pipeline {
            let obs_group_id: PipelineGroupId = SYSTEM_PIPELINE_GROUP_ID.into();
            let obs_pipeline_id: PipelineId = SYSTEM_OBSERVABILITY_PIPELINE_ID.into();
            let obs_pipeline_config = obs_pipeline.clone().into_pipeline_config();
            Self::validate_pipeline_components_with_factory(
                pipeline_factory,
                &obs_group_id,
                &obs_pipeline_id,
                &obs_pipeline_config,
            )?;
        }

        Ok(())
    }

    /// Validates that every configured node resolves to a registered component and that the
    /// static component-specific configuration validates.
    pub fn validate_engine_components(&self, engine_cfg: &OtelDataflowSpec) -> Result<(), String> {
        Self::validate_engine_components_with_factory(self.pipeline_factory, engine_cfg)
    }

    /// Starts the controller with the given engine configurations.
    pub fn run_forever(&self, engine_config: OtelDataflowSpec) -> Result<(), Error> {
        self.run_with_mode(engine_config, RunMode::ParkMainThread)
    }

    /// Starts the controller with the given engine configurations.
    ///
    /// Runs until pipelines are shut down, then closes telemetry/admin services.
    pub fn run_till_shutdown(&self, engine_config: OtelDataflowSpec) -> Result<(), Error> {
        self.run_with_mode(engine_config, RunMode::ShutdownWhenDone)
    }

    fn map_topic_spec_to_options(
        spec: &TopicSpec,
        inferred_mode: InferredTopicMode,
    ) -> TopicOptions {
        let balanced_capacity = spec.policies.balanced.queue_capacity.max(1);
        let broadcast_capacity = spec.policies.broadcast.queue_capacity.max(1);
        let broadcast_on_lag = spec.policies.broadcast.on_lag;
        match inferred_mode {
            InferredTopicMode::Mixed => TopicOptions::Mixed {
                balanced_capacity,
                broadcast_capacity,
                on_lag: broadcast_on_lag,
            },
            InferredTopicMode::BalancedOnly => TopicOptions::BalancedOnly {
                capacity: balanced_capacity,
            },
            InferredTopicMode::BroadcastOnly => TopicOptions::BroadcastOnly {
                capacity: broadcast_capacity,
                on_lag: broadcast_on_lag,
            },
        }
    }

    fn map_topic_spec_to_publish_outcome_config(spec: &TopicSpec) -> TopicPublishOutcomeConfig {
        TopicPublishOutcomeConfig {
            max_in_flight: spec.policies.ack_propagation.max_in_flight.max(1),
            timeout: spec.policies.ack_propagation.timeout,
        }
    }

    fn build_declared_topic_name_maps(
        config: &OtelDataflowSpec,
    ) -> Result<
        (
            HashMap<TopicName, TopicName>,
            HashMap<(PipelineGroupId, TopicName), TopicName>,
        ),
        Error,
    > {
        let mut global_names = HashMap::new();
        let mut group_names = HashMap::new();

        for topic_name in config.topics.keys() {
            let declared_name =
                Self::parse_topic_name(&format!("global::{}", topic_name.as_ref()))?;
            _ = global_names.insert(topic_name.clone(), declared_name);
        }

        for (group_id, group_cfg) in &config.groups {
            for topic_name in group_cfg.topics.keys() {
                let declared_name = Self::parse_topic_name(&format!(
                    "group::{}::{}",
                    group_id.as_ref(),
                    topic_name.as_ref()
                ))?;
                _ = group_names.insert((group_id.clone(), topic_name.clone()), declared_name);
            }
        }

        Ok((global_names, group_names))
    }

    fn resolve_declared_topic_name(
        pipeline_group_id: &PipelineGroupId,
        topic_name: &TopicName,
        global_names: &HashMap<TopicName, TopicName>,
        group_names: &HashMap<(PipelineGroupId, TopicName), TopicName>,
    ) -> Option<TopicName> {
        group_names
            .get(&(pipeline_group_id.clone(), topic_name.clone()))
            .cloned()
            .or_else(|| global_names.get(topic_name).cloned())
    }

    fn parse_topic_name_from_node_config(node_config: &NodeUserConfig) -> Option<TopicName> {
        let raw_topic = node_config.config.get("topic")?.as_str()?;
        TopicName::parse(raw_topic).ok()
    }

    fn parse_topic_receiver_mode(node_config: &NodeUserConfig) -> TopicReceiverMode {
        let Some(subscription) = node_config.config.get("subscription") else {
            return TopicReceiverMode::Broadcast;
        };
        let Some(subscription) = subscription.as_object() else {
            return TopicReceiverMode::Unknown;
        };
        let Some(mode) = subscription.get("mode").and_then(|value| value.as_str()) else {
            return TopicReceiverMode::Unknown;
        };

        match mode {
            "broadcast" => TopicReceiverMode::Broadcast,
            "balanced" => {
                let Some(raw_group) = subscription.get("group").and_then(|value| value.as_str())
                else {
                    return TopicReceiverMode::Unknown;
                };
                match SubscriptionGroupName::parse(raw_group) {
                    Ok(group) => TopicReceiverMode::Balanced(group),
                    Err(_) => TopicReceiverMode::Unknown,
                }
            }
            _ => TopicReceiverMode::Unknown,
        }
    }

    fn infer_topic_mode(summary: &TopicUsageSummary) -> InferredTopicMode {
        if summary.has_unknown_receiver_mode {
            return InferredTopicMode::Mixed;
        }
        if summary.receiver_refs == 0 {
            return InferredTopicMode::Mixed;
        }
        if summary.has_broadcast_receivers && summary.balanced_groups.is_empty() {
            return InferredTopicMode::BroadcastOnly;
        }
        if !summary.has_broadcast_receivers && summary.balanced_groups.len() == 1 {
            return InferredTopicMode::BalancedOnly;
        }
        InferredTopicMode::Mixed
    }

    fn infer_topic_modes(
        config: &OtelDataflowSpec,
        global_names: &HashMap<TopicName, TopicName>,
        group_names: &HashMap<(PipelineGroupId, TopicName), TopicName>,
    ) -> (
        HashMap<TopicName, InferredTopicMode>,
        Vec<InferredTopicModeReport>,
    ) {
        let mut usage_by_declared_topic = HashMap::<TopicName, TopicUsageSummary>::new();
        for declared_name in global_names.values().chain(group_names.values()) {
            _ = usage_by_declared_topic.insert(declared_name.clone(), TopicUsageSummary::default());
        }

        let mut visit_topic_node =
            |pipeline_group_id: &PipelineGroupId, node_config: &NodeUserConfig| {
                let topic_name = match Self::parse_topic_name_from_node_config(node_config) {
                    Some(topic_name) => topic_name,
                    None => return,
                };
                let Some(declared_topic_name) = Self::resolve_declared_topic_name(
                    pipeline_group_id,
                    &topic_name,
                    global_names,
                    group_names,
                ) else {
                    return;
                };
                let Some(summary) = usage_by_declared_topic.get_mut(&declared_topic_name) else {
                    return;
                };

                match node_config.kind() {
                    NodeKind::Receiver => {
                        summary.receiver_refs += 1;
                        match Self::parse_topic_receiver_mode(node_config) {
                            TopicReceiverMode::Broadcast => {
                                summary.has_broadcast_receivers = true;
                            }
                            TopicReceiverMode::Balanced(group) => {
                                _ = summary.balanced_groups.insert(group);
                            }
                            TopicReceiverMode::Unknown => {
                                summary.has_unknown_receiver_mode = true;
                            }
                        }
                    }
                    NodeKind::Exporter => {
                        summary.exporter_refs += 1;
                    }
                    _ => {}
                }
            };

        for (group_id, group_cfg) in &config.groups {
            for pipeline_cfg in group_cfg.pipelines.values() {
                for (_node_id, node_cfg) in pipeline_cfg.node_iter() {
                    if node_cfg.r#type.id() != "topic" {
                        continue;
                    }
                    visit_topic_node(group_id, node_cfg.as_ref());
                }
            }
        }

        if let Some(observability_pipeline) = config.engine.observability.pipeline.as_ref() {
            let system_group_id: PipelineGroupId = SYSTEM_PIPELINE_GROUP_ID.into();
            for (_node_id, node_cfg) in observability_pipeline.nodes.iter() {
                if node_cfg.r#type.id() != "topic" {
                    continue;
                }
                visit_topic_node(&system_group_id, node_cfg.as_ref());
            }
        }

        let mut inferred_modes = HashMap::with_capacity(usage_by_declared_topic.len());
        let mut inferred_mode_reports = Vec::with_capacity(usage_by_declared_topic.len());
        let mut declared_topics: Vec<_> = usage_by_declared_topic.keys().cloned().collect();
        declared_topics.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));

        for declared_topic in declared_topics {
            let summary = usage_by_declared_topic
                .get(&declared_topic)
                .expect("declared topic must have a usage summary");
            let topology_mode = Self::infer_topic_mode(summary);
            inferred_mode_reports.push(InferredTopicModeReport {
                topic: declared_topic.clone(),
                topology_mode,
                selected_mode: topology_mode,
                selection_policy: TopicImplSelectionPolicy::Auto,
                receiver_refs: summary.receiver_refs,
                exporter_refs: summary.exporter_refs,
                balanced_group_count: summary.balanced_groups.len(),
                has_broadcast_receivers: summary.has_broadcast_receivers,
                has_unknown_receiver_mode: summary.has_unknown_receiver_mode,
            });
            _ = inferred_modes.insert(declared_topic, topology_mode);
        }

        (inferred_modes, inferred_mode_reports)
    }

    fn add_topic_wiring_edge(
        adjacency: &mut HashMap<TopicWiringVertex, Vec<TopicWiringVertex>>,
        from: TopicWiringVertex,
        to: TopicWiringVertex,
    ) {
        adjacency.entry(from.clone()).or_default().push(to.clone());
        let _ = adjacency.entry(to).or_default();
    }

    fn collect_topic_wiring_edges_for_pipeline(
        adjacency: &mut HashMap<TopicWiringVertex, Vec<TopicWiringVertex>>,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        pipeline: &PipelineConfig,
        global_names: &HashMap<TopicName, TopicName>,
        group_names: &HashMap<(PipelineGroupId, TopicName), TopicName>,
    ) {
        for connection in pipeline.connection_iter() {
            let targets = connection.to_nodes();
            for source in connection.from_sources() {
                let source_vertex = TopicWiringVertex::PipelineNode {
                    pipeline_group_id: pipeline_group_id.clone(),
                    pipeline_id: pipeline_id.clone(),
                    node_id: source.node_id().clone(),
                };
                for target in &targets {
                    let target_vertex = TopicWiringVertex::PipelineNode {
                        pipeline_group_id: pipeline_group_id.clone(),
                        pipeline_id: pipeline_id.clone(),
                        node_id: target.clone(),
                    };
                    Self::add_topic_wiring_edge(adjacency, source_vertex.clone(), target_vertex);
                }
            }
        }

        let mut topic_nodes = pipeline.node_iter().collect::<Vec<_>>();
        topic_nodes.sort_by(|(left, _), (right, _)| left.as_ref().cmp(right.as_ref()));
        for (node_id, node_config) in topic_nodes {
            if node_config.r#type.id() != "topic" {
                continue;
            }
            let Some(topic_name) = Self::parse_topic_name_from_node_config(node_config) else {
                continue;
            };
            let Some(declared_name) = Self::resolve_declared_topic_name(
                pipeline_group_id,
                &topic_name,
                global_names,
                group_names,
            ) else {
                continue;
            };
            let node_vertex = TopicWiringVertex::PipelineNode {
                pipeline_group_id: pipeline_group_id.clone(),
                pipeline_id: pipeline_id.clone(),
                node_id: node_id.clone(),
            };
            let topic_vertex = TopicWiringVertex::Topic { declared_name };
            match node_config.kind() {
                NodeKind::Exporter => {
                    Self::add_topic_wiring_edge(adjacency, node_vertex, topic_vertex);
                }
                NodeKind::Receiver => {
                    Self::add_topic_wiring_edge(adjacency, topic_vertex, node_vertex);
                }
                _ => {}
            }
        }
    }

    fn detect_topic_wiring_cycles(
        adjacency: &HashMap<TopicWiringVertex, Vec<TopicWiringVertex>>,
    ) -> Vec<Vec<TopicWiringVertex>> {
        fn visit(
            node: &TopicWiringVertex,
            adjacency: &HashMap<TopicWiringVertex, Vec<TopicWiringVertex>>,
            visiting: &mut HashSet<TopicWiringVertex>,
            visited: &mut HashSet<TopicWiringVertex>,
            current_path: &mut Vec<TopicWiringVertex>,
            cycles: &mut Vec<Vec<TopicWiringVertex>>,
        ) {
            if visited.contains(node) {
                return;
            }
            if visiting.contains(node) {
                if let Some(pos) = current_path.iter().position(|candidate| candidate == node) {
                    cycles.push(current_path[pos..].to_vec());
                }
                return;
            }

            let _ = visiting.insert(node.clone());
            current_path.push(node.clone());

            if let Some(targets) = adjacency.get(node) {
                for target in targets {
                    visit(target, adjacency, visiting, visited, current_path, cycles);
                }
            }

            let _ = visiting.remove(node);
            let _ = visited.insert(node.clone());
            let _ = current_path.pop();
        }

        let mut nodes = adjacency.keys().cloned().collect::<Vec<_>>();
        nodes.sort_by_key(TopicWiringVertex::label);

        let mut cycles = Vec::new();
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        for node in nodes {
            visit(
                &node,
                adjacency,
                &mut visiting,
                &mut visited,
                &mut current_path,
                &mut cycles,
            );
        }

        cycles
    }

    fn validate_topic_wiring_acyclic(
        config: &OtelDataflowSpec,
        global_names: &HashMap<TopicName, TopicName>,
        group_names: &HashMap<(PipelineGroupId, TopicName), TopicName>,
    ) -> Result<(), Error> {
        let mut adjacency = HashMap::<TopicWiringVertex, Vec<TopicWiringVertex>>::new();

        let mut group_ids = config.groups.keys().cloned().collect::<Vec<_>>();
        group_ids.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));
        for group_id in group_ids {
            let group_cfg = config
                .groups
                .get(&group_id)
                .expect("group collected from config must still exist");
            let mut pipeline_ids = group_cfg.pipelines.keys().cloned().collect::<Vec<_>>();
            pipeline_ids.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));
            for pipeline_id in pipeline_ids {
                let pipeline_cfg = group_cfg
                    .pipelines
                    .get(&pipeline_id)
                    .expect("pipeline collected from config must still exist");
                Self::collect_topic_wiring_edges_for_pipeline(
                    &mut adjacency,
                    &group_id,
                    &pipeline_id,
                    pipeline_cfg,
                    global_names,
                    group_names,
                );
            }
        }

        if let Some(observability_pipeline) = config.engine.observability.pipeline.as_ref() {
            let system_group_id: PipelineGroupId = SYSTEM_PIPELINE_GROUP_ID.into();
            let observability_pipeline_id: PipelineId = SYSTEM_OBSERVABILITY_PIPELINE_ID.into();
            let pipeline_cfg = observability_pipeline.clone().into_pipeline_config();
            Self::collect_topic_wiring_edges_for_pipeline(
                &mut adjacency,
                &system_group_id,
                &observability_pipeline_id,
                &pipeline_cfg,
                global_names,
                group_names,
            );
        }

        if let Some(cycle) = Self::detect_topic_wiring_cycles(&adjacency)
            .into_iter()
            .next()
        {
            let mut cycle_labels = cycle
                .iter()
                .map(TopicWiringVertex::label)
                .collect::<Vec<_>>();
            if let Some(first) = cycle.first() {
                cycle_labels.push(first.label());
            }
            return Err(Error::TopicWiringCycleDetected {
                cycle: cycle_labels,
            });
        }

        Ok(())
    }

    fn emit_topic_mode_reports(reports: &[InferredTopicModeReport]) {
        for report in reports {
            otel_info!(
                "controller.topic_mode_inferred",
                topic = report.topic.as_ref(),
                topology_mode = report.topology_mode.as_str(),
                selected_mode = report.selected_mode.as_str(),
                selection_policy = report.selection_policy.to_string(),
                receiver_refs = report.receiver_refs as u64,
                exporter_refs = report.exporter_refs as u64,
                balanced_group_count = report.balanced_group_count as u64,
                has_broadcast_receivers = report.has_broadcast_receivers,
                has_unknown_receiver_mode = report.has_unknown_receiver_mode,
                message = "Resolved topic mode from topology inference and config selection policy"
            );
        }
    }

    fn apply_topic_impl_selection_policy(
        topology_mode: InferredTopicMode,
        policy: TopicImplSelectionPolicy,
    ) -> InferredTopicMode {
        match policy {
            TopicImplSelectionPolicy::Auto => topology_mode,
            TopicImplSelectionPolicy::ForceMixed => InferredTopicMode::Mixed,
        }
    }

    fn update_topic_mode_report(
        reports: &mut [InferredTopicModeReport],
        topic: &TopicName,
        selection_policy: TopicImplSelectionPolicy,
        selected_mode: InferredTopicMode,
    ) {
        if let Some(report) = reports.iter_mut().find(|report| &report.topic == topic) {
            report.selection_policy = selection_policy;
            report.selected_mode = selected_mode;
        }
    }

    fn topic_backend_capabilities(backend: TopicBackendKind) -> Option<TopicBackendCapabilities> {
        match backend {
            TopicBackendKind::InMemory => Some(TopicBackendCapabilities {
                supports_balanced_only: true,
                supports_broadcast_only: true,
                supports_mixed: true,
                supports_broadcast_on_lag_drop_oldest: true,
                supports_broadcast_on_lag_disconnect: true,
                supports_ack_propagation_disabled: true,
                supports_ack_propagation_auto: true,
            }),
            TopicBackendKind::Quiver => None,
        }
    }

    fn validate_topic_runtime_support_with_capabilities(
        topic: &TopicName,
        backend: TopicBackendKind,
        policies: &otap_df_config::topic::TopicPolicies,
        selected_mode: InferredTopicMode,
        capabilities: TopicBackendCapabilities,
    ) -> Result<(), Error> {
        if !capabilities.supports_mode(selected_mode) {
            return Err(Error::UnsupportedTopicMode {
                topic: topic.clone(),
                backend,
                mode: selected_mode.as_str().to_owned(),
            });
        }

        if matches!(
            selected_mode,
            InferredTopicMode::BroadcastOnly | InferredTopicMode::Mixed
        ) && !capabilities.supports_broadcast_on_lag(policies.broadcast.on_lag)
        {
            return Err(Error::UnsupportedTopicPolicy {
                topic: topic.clone(),
                backend,
                policy: "broadcast.on_lag",
                value: broadcast_on_lag_policy_value(policies.broadcast.on_lag).to_owned(),
            });
        }

        if !capabilities.supports_ack_propagation(policies.ack_propagation.mode) {
            return Err(Error::UnsupportedTopicPolicy {
                topic: topic.clone(),
                backend,
                policy: "ack_propagation",
                value: ack_propagation_policy_value(policies.ack_propagation.mode).to_owned(),
            });
        }

        Ok(())
    }

    fn validate_topic_runtime_support(
        topic: &TopicName,
        spec: &TopicSpec,
        selected_mode: InferredTopicMode,
    ) -> Result<(), Error> {
        let Some(capabilities) = Self::topic_backend_capabilities(spec.backend) else {
            return Err(Error::UnsupportedTopicBackend {
                topic: topic.clone(),
                backend: spec.backend,
            });
        };
        Self::validate_topic_runtime_support_with_capabilities(
            topic,
            spec.backend,
            &spec.policies,
            selected_mode,
            capabilities,
        )
    }

    fn declare_topic(
        broker: &TopicBroker<PData>,
        name: TopicName,
        spec: &TopicSpec,
        inferred_mode: InferredTopicMode,
    ) -> Result<(), Error> {
        Self::validate_topic_runtime_support(&name, spec, inferred_mode)?;
        let opts = Self::map_topic_spec_to_options(spec, inferred_mode);
        match spec.backend {
            TopicBackendKind::InMemory => {
                _ = broker
                    .create_topic(name, opts, InMemoryBackend)
                    .map_err(|e| Error::PipelineRuntimeError {
                        source: Box::new(e),
                    })?;
                Ok(())
            }
            TopicBackendKind::Quiver => unreachable!("unsupported backend must be rejected above"),
        }
    }

    fn parse_topic_name(raw: &str) -> Result<TopicName, Error> {
        TopicName::parse(raw).map_err(|e| Error::PipelineRuntimeError {
            source: Box::new(EngineError::InternalError {
                message: format!("invalid topic name `{raw}`: {e}"),
            }),
        })
    }

    fn declare_topics(config: &OtelDataflowSpec) -> Result<DeclaredTopics<PData>, Error> {
        let broker = TopicBroker::<PData>::new();
        let (global_names, group_names) = Self::build_declared_topic_name_maps(config)?;
        Self::validate_topic_wiring_acyclic(config, &global_names, &group_names)?;
        let (inferred_modes, mut inferred_mode_reports) =
            Self::infer_topic_modes(config, &global_names, &group_names);
        let default_selection_policy = config.engine.topics.impl_selection;

        for (topic_name, spec) in &config.topics {
            let declared_name = global_names
                .get(topic_name)
                .expect("global topic declaration must resolve to a declared topic name")
                .clone();
            let topology_mode = inferred_modes
                .get(&declared_name)
                .copied()
                .unwrap_or(InferredTopicMode::Mixed);
            let selection_policy = spec.impl_selection.unwrap_or(default_selection_policy);
            let selected_mode =
                Self::apply_topic_impl_selection_policy(topology_mode, selection_policy);
            Self::update_topic_mode_report(
                &mut inferred_mode_reports,
                &declared_name,
                selection_policy,
                selected_mode,
            );
            Self::declare_topic(&broker, declared_name, spec, selected_mode)?;
        }

        for (group_id, group_cfg) in &config.groups {
            for (topic_name, spec) in &group_cfg.topics {
                let declared_name = group_names
                    .get(&(group_id.clone(), topic_name.clone()))
                    .expect("group topic declaration must resolve to a declared topic name")
                    .clone();
                let topology_mode = inferred_modes
                    .get(&declared_name)
                    .copied()
                    .unwrap_or(InferredTopicMode::Mixed);
                let selection_policy = spec.impl_selection.unwrap_or(default_selection_policy);
                let selected_mode =
                    Self::apply_topic_impl_selection_policy(topology_mode, selection_policy);
                Self::update_topic_mode_report(
                    &mut inferred_mode_reports,
                    &declared_name,
                    selection_policy,
                    selected_mode,
                );
                Self::declare_topic(&broker, declared_name, spec, selected_mode)?;
            }
        }

        Ok(DeclaredTopics {
            broker,
            global_names,
            group_names,
            inferred_mode_reports,
        })
    }

    fn build_pipeline_topic_set(
        config: &OtelDataflowSpec,
        declared: &DeclaredTopics<PData>,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
        core_id: usize,
    ) -> Result<TopicSet<PData>, Error> {
        let set_name = format!(
            "{}::{}::core-{}",
            pipeline_group_id.as_ref(),
            pipeline_id.as_ref(),
            core_id
        );
        let set = TopicSet::new(set_name);

        for (global_topic_name, topic_spec) in &config.topics {
            if let Some(declared_name) = declared.global_names.get(global_topic_name) {
                let handle = declared
                    .broker
                    .get_topic_required(declared_name)
                    .map_err(|e| Error::PipelineRuntimeError {
                        source: Box::new(e),
                    })?;
                let handle = handle.with_default_publish_outcome_config(
                    Self::map_topic_spec_to_publish_outcome_config(topic_spec),
                );
                let binding = PipelineTopicBinding::from(handle)
                    .with_default_queue_on_full(topic_spec.policies.balanced.on_full.clone())
                    .with_default_ack_propagation_mode(topic_spec.policies.ack_propagation.mode);
                _ = set.insert(global_topic_name.clone(), binding);
            }
        }

        if let Some(group_cfg) = config.groups.get(pipeline_group_id) {
            for (group_topic_name, topic_spec) in &group_cfg.topics {
                if let Some(declared_name) = declared
                    .group_names
                    .get(&(pipeline_group_id.clone(), group_topic_name.clone()))
                {
                    let handle =
                        declared
                            .broker
                            .get_topic_required(declared_name)
                            .map_err(|e| Error::PipelineRuntimeError {
                                source: Box::new(e),
                            })?;
                    let handle = handle.with_default_publish_outcome_config(
                        Self::map_topic_spec_to_publish_outcome_config(topic_spec),
                    );
                    let binding = PipelineTopicBinding::from(handle)
                        .with_default_queue_on_full(topic_spec.policies.balanced.on_full.clone())
                        .with_default_ack_propagation_mode(
                            topic_spec.policies.ack_propagation.mode,
                        );
                    // Group-local declarations override globals with the same local name.
                    _ = set.insert(group_topic_name.clone(), binding);
                }
            }
        }

        Ok(set)
    }

    fn run_with_mode(
        &self,
        engine_config: OtelDataflowSpec,
        run_mode: RunMode,
    ) -> Result<(), Error> {
        let num_pipeline_groups = engine_config.groups.len();
        let resolved_config = engine_config.resolve();
        let (engine, pipelines, observability_pipeline) = resolved_config.into_parts();
        let num_pipelines = pipelines.len();
        let admin_settings = engine.http_admin.clone().unwrap_or_default();
        // Initialize metrics system and observed event store.
        // ToDo A hierarchical metrics system will be implemented to better support hardware with multiple NUMA nodes.
        let telemetry_config = &engine.telemetry;
        otel_info!(
            "controller.start",
            num_pipeline_groups = num_pipeline_groups,
            num_pipelines = num_pipelines
        );

        // Create the shared telemetry registry first - it will be used by both
        // the observed state store and the internal telemetry system.
        let telemetry_registry = TelemetryRegistryHandle::new();

        // Create the observed state store for the telemetry system.
        let obs_state_store =
            ObservedStateStore::new(&engine.observed_state, telemetry_registry.clone());
        let obs_state_handle = obs_state_store.handle();
        let engine_evt_reporter = obs_state_store.reporter(engine.observed_state.engine_events);
        let console_async_reporter = telemetry_config
            .logs
            .providers
            .uses_console_async_provider()
            .then(|| obs_state_store.reporter(engine.observed_state.logging_events));

        // Create the telemetry system. The console_async_reporter is passed when any
        // providers use ConsoleAsync. The its_logs_receiver is passed when any
        // providers use the ITS mode.
        let telemetry_system = InternalTelemetrySystem::new(
            telemetry_config,
            telemetry_registry.clone(),
            console_async_reporter,
            engine_context,
        )?;

        let admin_tracing_setup = telemetry_system.admin_tracing_setup();
        let internal_tracing_setup = telemetry_system.internal_tracing_setup();

        let metrics_dispatcher = telemetry_system.dispatcher();
        let metrics_reporter = telemetry_system.reporter();
        let controller_ctx = ControllerContext::new(telemetry_system.registry());
        // Declare all topics up front before any pipeline thread starts.
        let declared_topics = Self::declare_topics(&engine_config)?;

        for pipeline_entry in &pipelines {
            let pipeline_key = PipelineKey::new(
                pipeline_entry.pipeline_group_id.clone(),
                pipeline_entry.pipeline_id.clone(),
            );
            obs_state_store.register_pipeline_health_policy(
                pipeline_key,
                pipeline_entry.policies.health.clone(),
            );
        }

        let all_cores =
            core_affinity::get_core_ids().ok_or_else(|| Error::CoreDetectionUnavailable)?;
        let its_core = *all_cores.first().expect("a cpu core");
        let its_key = Self::internal_pipeline_key(its_core);
        if let Some(pipeline) = observability_pipeline.as_ref() {
            obs_state_store.register_pipeline_health_policy(
                PipelineKey::new(
                    its_key.pipeline_group_id.clone(),
                    its_key.pipeline_id.clone(),
                ),
                pipeline.policies.health.clone(),
            );
        }
        let available_core_ids = all_cores;
        let planned_core_assignments =
            Self::preflight_pipeline_core_allocations(&pipelines, &available_core_ids)?;

        let internal_pipeline_handle = Self::spawn_internal_pipeline_if_configured(
            its_key.clone(),
            its_core,
            observability_pipeline,
            &engine_config,
            &declared_topics,
            &telemetry_system,
            self.pipeline_factory,
            &controller_ctx,
            &engine_evt_reporter,
            &metrics_reporter,
            internal_tracing_setup,
        )?;

        // TODO: This should be validated somewhere, that engine observability pipeline is
        // defined when ITS is requested. Possibly we could fill in a default.
        let has_internal_pipeline = internal_pipeline_handle.is_some();
        match (
            has_internal_pipeline,
            telemetry_config.logs.providers.uses_its_provider(),
        ) {
            (false, true) => {
                otel_warn!(
                    "ITS provider requested yet engine.observability.pipeline is not defined"
                )
            }
            (true, false) => {
                otel_warn!(
                    "engine.observability.pipeline is defined yet ITS provider is not requested"
                )
            }
            _ => {}
        };

        // Initialize the global subscriber AFTER the internal pipeline has signaled
        // successful startup. This ensures the channel receiver is being consumed
        // before we start sending logs.
        telemetry_system.init_global_subscriber();
        Self::emit_topic_mode_reports(&declared_topics.inferred_mode_reports);

        let internal_collector = telemetry_system.collector();
        let metrics_agg_handle = spawn_thread_local_task(
            "metrics-aggregator",
            admin_tracing_setup.clone(),
            move |cancellation_token| internal_collector.run(cancellation_token),
        )?;

        // Start the metrics dispatcher only if there are metric readers configured.
        let metrics_dispatcher_handle = if telemetry_config.metrics.has_readers() {
            Some(spawn_thread_local_task(
                "metrics-dispatcher",
                admin_tracing_setup.clone(),
                move |cancellation_token| metrics_dispatcher.run_dispatch_loop(cancellation_token),
            )?)
        } else {
            None
        };

        // Start the observed state store background task
        let obs_state_store_runtime = obs_state_store.clone();
        let obs_state_join_handle = spawn_thread_local_task(
            "observed-state-store",
            admin_tracing_setup.clone(),
            move |cancellation_token| obs_state_store_runtime.run(cancellation_token),
        )?;

        // Start the engine-wide metrics collection task.
        // This samples engine-level metrics (e.g. RSS) on a fixed interval and
        // reports them once per engine, rather than duplicating across pipelines.
        let engine_entity_key = controller_ctx.register_engine_entity();
        let engine_registry = controller_ctx.telemetry_registry();
        let engine_reporter = metrics_reporter.clone();
        let engine_metrics_handle = spawn_thread_local_task(
            "engine-metrics",
            admin_tracing_setup.clone(),
            move |cancellation_token| async move {
                use otap_df_engine::engine_metrics::EngineMetricsMonitor;
                use std::time::Duration;
                use tokio::time::{MissedTickBehavior, interval};

                // TODO: Make this interval configurable via engine config.
                const ENGINE_METRICS_INTERVAL: Duration = Duration::from_secs(5);

                let mut monitor =
                    EngineMetricsMonitor::new(engine_registry, engine_entity_key, engine_reporter);

                let mut ticker = interval(ENGINE_METRICS_INTERVAL);
                ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            return Ok::<(), otap_df_telemetry::error::Error>(());
                        }
                        _ = ticker.tick() => {
                            monitor.update();
                            if let Err(err) = monitor.report() {
                                otel_warn!(
                                    "engine.metrics.reporting.fail",
                                    error = err.to_string()
                                );
                            }
                        }
                    }
                }
            },
        )?;

        let runtime = Arc::new(ControllerRuntime::new(
            self.pipeline_factory,
            controller_ctx.clone(),
            obs_state_store.clone(),
            obs_state_handle.clone(),
            engine_evt_reporter.clone(),
            metrics_reporter.clone(),
            declared_topics,
            available_core_ids.clone(),
            telemetry_system.engine_tracing_setup(),
            engine_config.clone(),
        ));

        if let Some(launched) = internal_pipeline_handle {
            runtime.register_launched_instance(launched);
        }

        for (pipeline_entry, requested_cores) in pipelines.iter().zip(planned_core_assignments) {
            runtime.register_committed_pipeline(pipeline_entry.clone(), 0);
            let num_cores = requested_cores.len();

            let core_allocation = pipeline_entry
                .policies
                .effective_resources()
                .core_allocation
                .to_string();
            otel_info!(
                "pipeline.core_allocation",
                pipeline_group_id = pipeline_entry.pipeline_group_id.as_ref(),
                pipeline_id = pipeline_entry.pipeline_id.as_ref(),
                num_cores = num_cores,
                core_allocation = core_allocation
            );

            for core_id in &requested_cores {
                let launched = Self::launch_pipeline_thread(
                    self.pipeline_factory,
                    DeployedPipelineKey {
                        pipeline_group_id: pipeline_entry.pipeline_group_id.clone(),
                        pipeline_id: pipeline_entry.pipeline_id.clone(),
                        core_id: core_id.id,
                        deployment_generation: 0,
                    },
                    *core_id,
                    num_cores,
                    pipeline_entry.pipeline.clone(),
                    pipeline_entry.policies.channel_capacity.clone(),
                    pipeline_entry.policies.telemetry.clone(),
                    controller_ctx.clone(),
                    metrics_reporter.clone(),
                    engine_evt_reporter.clone(),
                    telemetry_system.engine_tracing_setup(),
                    &engine_config,
                    &runtime.declared_topics,
                    runtime.next_thread_id(),
                    None,
                )?;
                runtime.register_launched_instance(launched);
            }
        }

        drop(metrics_reporter);

        let control_plane: Arc<dyn ControlPlane> = Arc::new(ControllerControlPlane {
            runtime: Arc::clone(&runtime),
        });
        let admin_server_handle = spawn_thread_local_task(
            "http-admin",
            admin_tracing_setup,
            move |cancellation_token| {
                otap_df_admin::run(
                    admin_settings,
                    obs_state_handle,
                    control_plane,
                    telemetry_registry,
                    cancellation_token,
                )
            },
        )?;

        if run_mode == RunMode::ShutdownWhenDone {
            runtime.wait_until_all_instances_exit();
        }

        // In standard engine mode we keep the main thread parked after startup.
        if run_mode == RunMode::ParkMainThread {
            thread::park();
        }

        // All pipelines have finished; shut down the admin HTTP server and metric aggregator gracefully.
        engine_metrics_handle.shutdown_and_join()?;
        admin_server_handle.shutdown_and_join()?;
        metrics_agg_handle.shutdown_and_join()?;
        if let Some(handle) = metrics_dispatcher_handle {
            handle.shutdown_and_join()?;
        }
        obs_state_join_handle.shutdown_and_join()?;
        telemetry_system.shutdown_otel()?;

        if let Some(err) = runtime.take_runtime_error() {
            return Err(err);
        }

        Ok(())
    }

    /// Selects which CPU cores to use based on the given allocation.
    fn select_cores_for_allocation(
        mut available_core_ids: Vec<CoreId>,
        core_allocation: &CoreAllocation,
    ) -> Result<Vec<CoreId>, Error> {
        available_core_ids.sort_by_key(|c| c.id);

        let max_core_id = available_core_ids.iter().map(|c| c.id).max().unwrap_or(0);
        let num_cores = available_core_ids.len();

        match core_allocation {
            CoreAllocation::AllCores => Ok(available_core_ids),
            CoreAllocation::CoreCount { count } => {
                if *count == 0 {
                    Ok(available_core_ids)
                } else if *count > num_cores {
                    Err(Error::InvalidCoreAllocation {
                        alloc: core_allocation.clone(),
                        message: format!(
                            "Requested {} cores but only {} cores available on this system",
                            count, num_cores
                        ),
                        available: available_core_ids.iter().map(|c| c.id).collect(),
                    })
                } else {
                    Ok(available_core_ids.into_iter().take(*count).collect())
                }
            }
            CoreAllocation::CoreSet { set } => {
                // Validate all ranges first
                for r in set.iter() {
                    if r.start > r.end {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Invalid core range: start ({}) is greater than end ({})",
                                r.start, r.end
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                    if r.start > max_core_id {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Core ID {} exceeds available cores (system has cores 0-{})",
                                r.start, max_core_id
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                    if r.end > max_core_id {
                        return Err(Error::InvalidCoreAllocation {
                            alloc: core_allocation.clone(),
                            message: format!(
                                "Core ID {} exceeds available cores (system has cores 0-{})",
                                r.end, max_core_id
                            ),
                            available: available_core_ids.iter().map(|c| c.id).collect(),
                        });
                    }
                }

                // Check for overlapping ranges
                for (i, r1) in set.iter().enumerate() {
                    for r2 in set.iter().skip(i + 1) {
                        // Two ranges overlap if they share any common cores
                        if r1.start <= r2.end && r2.start <= r1.end {
                            let overlap_start = r1.start.max(r2.start);
                            let overlap_end = r1.end.min(r2.end);
                            return Err(Error::InvalidCoreAllocation {
                                alloc: core_allocation.clone(),
                                message: format!(
                                    "Core ranges overlap: {}-{} and {}-{} share cores {}-{}",
                                    r1.start, r1.end, r2.start, r2.end, overlap_start, overlap_end
                                ),
                                available: available_core_ids.iter().map(|c| c.id).collect(),
                            });
                        }
                    }
                }

                // Filter cores in range
                let selected: Vec<_> = available_core_ids
                    .into_iter()
                    // Naively check if each interval contains the point
                    // This problem is known as the "Interval Stabbing Problem"
                    // and has more efficient but more complex solutions
                    .filter(|c| set.iter().any(|r| r.start <= c.id && c.id <= r.end))
                    .collect();

                if selected.is_empty() {
                    return Err(Error::InvalidCoreAllocation {
                        alloc: core_allocation.clone(),
                        message: "No available cores in the specified ranges".to_owned(),
                        available: core_affinity::get_core_ids()
                            .unwrap_or_default()
                            .iter()
                            .map(|c| c.id)
                            .collect(),
                    });
                }

                Ok(selected)
            }
        }
    }

    /// Pre-resolves core assignments for all regular pipelines.
    ///
    /// This validates the full pipeline set before any pipeline thread is spawned.
    fn preflight_pipeline_core_allocations(
        pipelines: &[ResolvedPipelineConfig],
        available_core_ids: &[CoreId],
    ) -> Result<Vec<Vec<CoreId>>, Error> {
        pipelines
            .iter()
            .map(|pipeline_entry| {
                Self::select_cores_for_allocation(
                    available_core_ids.to_vec(),
                    &pipeline_entry
                        .policies
                        .effective_resources()
                        .core_allocation,
                )
            })
            .collect()
    }

    fn internal_pipeline_key(core_id: CoreId) -> DeployedPipelineKey {
        DeployedPipelineKey {
            pipeline_group_id: SYSTEM_PIPELINE_GROUP_ID.into(),
            pipeline_id: SYSTEM_OBSERVABILITY_PIPELINE_ID.into(),
            core_id: core_id.id,
            deployment_generation: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn launch_pipeline_thread(
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_key: DeployedPipelineKey,
        core_id: CoreId,
        num_cores: usize,
        pipeline_config: PipelineConfig,
        channel_capacity_policy: ChannelCapacityPolicy,
        telemetry_policy: TelemetryPolicy,
        controller_ctx: ControllerContext,
        metrics_reporter: MetricsReporter,
        engine_evt_reporter: ObservedEventReporter,
        tracing_setup: TracingSetup,
        config: &OtelDataflowSpec,
        declared_topics: &DeclaredTopics<PData>,
        thread_id: usize,
        internal_telemetry: Option<(
            InternalTelemetrySettings,
            std_mpsc::SyncSender<Result<(), EngineError>>,
        )>,
    ) -> Result<LaunchedPipelineThread<PData>, Error> {
        let mut pipeline_ctx = controller_ctx.pipeline_context_with_generation(
            pipeline_key.pipeline_group_id.clone(),
            pipeline_key.pipeline_id.clone(),
            pipeline_key.core_id,
            num_cores,
            thread_id,
            pipeline_key.deployment_generation,
        );
        let topic_set = Self::build_pipeline_topic_set(
            config,
            declared_topics,
            &pipeline_key.pipeline_group_id,
            &pipeline_key.pipeline_id,
            pipeline_key.core_id,
        )?;
        pipeline_ctx.set_topic_set(topic_set);
        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) =
            pipeline_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
        let control_sender: Arc<dyn PipelineAdminSender> = Arc::new(pipeline_ctrl_msg_tx.clone());
        let thread_name = format!(
            "pipeline-{}-{}-core-{}-gen-{}",
            pipeline_key.pipeline_group_id.as_ref(),
            pipeline_key.pipeline_id.as_ref(),
            pipeline_key.core_id,
            pipeline_key.deployment_generation
        );
        let run_key = pipeline_key.clone();
        let handle = thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                Self::run_pipeline_thread(
                    run_key,
                    core_id,
                    pipeline_config,
                    channel_capacity_policy,
                    telemetry_policy,
                    pipeline_factory,
                    pipeline_ctx,
                    engine_evt_reporter,
                    metrics_reporter,
                    pipeline_ctrl_msg_tx,
                    pipeline_ctrl_msg_rx,
                    tracing_setup,
                    internal_telemetry,
                )
            })
            .map_err(|e| Error::ThreadSpawnError {
                thread_name: thread_name.clone(),
                source: e,
            })?;

        Ok(LaunchedPipelineThread {
            thread_name,
            thread_id,
            pipeline_key,
            control_sender,
            join_handle: handle,
            _marker: std::marker::PhantomData,
        })
    }

    /// Spawns the internal telemetry pipeline if engine observability config provides one.
    ///
    /// Returns the thread handle if an internal pipeline was spawned
    /// and waits for it to start, or None.
    #[allow(clippy::too_many_arguments)]
    fn spawn_internal_pipeline_if_configured(
        its_key: DeployedPipelineKey,
        its_core: CoreId,
        observability_pipeline: Option<ResolvedPipelineConfig>,
        config: &OtelDataflowSpec,
        declared_topics: &DeclaredTopics<PData>,
        telemetry_system: &InternalTelemetrySystem,
        pipeline_factory: &'static PipelineFactory<PData>,
        controller_ctx: &ControllerContext,
        engine_evt_reporter: &ObservedEventReporter,
        metrics_reporter: &MetricsReporter,
        tracing_setup: TracingSetup,
    ) -> Result<Option<LaunchedPipelineThread<PData>>, Error> {
        let (internal_config, channel_capacity_policy, telemetry_policy): (
            PipelineConfig,
            ChannelCapacityPolicy,
            TelemetryPolicy,
        ) = match observability_pipeline {
            Some(config) if config.role == ResolvedPipelineRole::ObservabilityInternal => {
                let channel_capacity_policy = config.policies.channel_capacity;
                let telemetry_policy = config.policies.telemetry;
                (config.pipeline, channel_capacity_policy, telemetry_policy)
            }
            Some(_) => {
                // Note: This path is internal-only and should be filtered by caller.
                return Ok(None);
            }
            _ => {
                // Note: Inconsistent configurations are checked elsewhere.
                // This method is "_if_configured()" for lifetime reasons,
                // so a silent return.
                return Ok(None);
            }
        };

        let its_settings = match telemetry_system.internal_telemetry_settings() {
            None => {
                // Note: An inconsistency warning will be logged by the
                // calling function.
                return Ok(None);
            }
            Some(its_settings) => its_settings,
        };

        // Create a channel to signal startup success/failure
        let (startup_tx, startup_rx) = std_mpsc::sync_channel::<Result<(), EngineError>>(1);
        let launched = Self::launch_pipeline_thread(
            pipeline_factory,
            its_key,
            its_core,
            1,
            internal_config,
            channel_capacity_policy,
            telemetry_policy,
            controller_ctx.clone(),
            metrics_reporter.clone(),
            engine_evt_reporter.clone(),
            tracing_setup,
            config,
            declared_topics,
            0,
            Some((its_settings, startup_tx)),
        )?;

        // Wait for the internal pipeline to signal successful startup
        match startup_rx.recv() {
            Ok(Ok(())) => {
                otel_info!(
                    "internal_pipeline.started",
                    message = "Internal telemetry pipeline started successfully"
                );
            }
            Ok(Err(e)) => {
                // Internal pipeline failed to build - propagate the error
                return Err(Error::PipelineRuntimeError {
                    source: Box::new(e),
                });
            }
            Err(err) => {
                // Channel closed unexpectedly - thread may have panicked
                return Err(Error::PipelineRuntimeError {
                    source: Box::new(err),
                });
            }
        }

        Ok(Some(launched))
    }

    /// Runs a single pipeline in the current thread.
    fn run_pipeline_thread(
        pipeline_key: DeployedPipelineKey,
        core_id: CoreId,
        pipeline_config: PipelineConfig,
        channel_capacity_policy: ChannelCapacityPolicy,
        telemetry_policy: TelemetryPolicy,
        pipeline_factory: &'static PipelineFactory<PData>,
        pipeline_context: PipelineContext,
        obs_evt_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<PData>,
        pipeline_ctrl_msg_rx: PipelineCtrlMsgReceiver<PData>,
        tracing_setup: TracingSetup,
        internal_telemetry: Option<(
            InternalTelemetrySettings,
            std_mpsc::SyncSender<Result<(), EngineError>>,
        )>,
    ) -> Result<Vec<()>, Error> {
        // Pin thread to specific core. As much as possible, we pin
        // before allocating memory.
        if !core_affinity::set_for_current(core_id) {
            // Continue execution even if pinning fails.
            // This is acceptable because the OS will still schedule the thread, but performance may be less predictable.
            otel_warn!(
                "core_affinity.set_failed",
                message = "Failed to set core affinity for pipeline thread. Performance may be less predictable."
            );
        }

        // Run the pipeline with thread-local tracing subscriber active.
        tracing_setup.with_subscriber(|| {
            // Create a tracing span for this pipeline thread
            // so that all logs within this scope include pipeline context.
            let span = otel_info_span!("pipeline_thread", core.id = core_id.id);
            let _guard = span.enter();

            // The controller creates a pipeline instance into a dedicated thread. The corresponding
            // entity is registered here for proper context tracking and set into thread-local storage
            // in order to be accessible by all components within this thread.
            let pipeline_entity_key = pipeline_context.register_pipeline_entity();
            let _pipeline_entity_guard =
                set_pipeline_entity_key(pipeline_context.metrics_registry(), pipeline_entity_key);

            obs_evt_reporter.report(EngineEvent::admitted(
                pipeline_key.clone(),
                Some("Pipeline admission successful.".to_owned()),
            ));

            // Build the runtime pipeline from the configuration
            let its_settings = internal_telemetry.as_ref().map(|(s, _)| s).cloned();
            let runtime_pipeline = pipeline_factory
                .build(
                    pipeline_context.clone(),
                    pipeline_config.clone(),
                    channel_capacity_policy,
                    telemetry_policy,
                    its_settings,
                )
                .map_err(|e| {
                    if let Some((_, startup_tx)) = internal_telemetry.as_ref() {
                        let _ = startup_tx.send(Err(EngineError::InternalError {
                            message: e.to_string(),
                        }));
                    }
                    otel_error!(
                        "controller.pipeline_build_failed",
                        pipeline_group_id = pipeline_key.pipeline_group_id.as_ref(),
                        pipeline_id = pipeline_key.pipeline_id.as_ref(),
                        core_id = core_id.id,
                        error = %e,
                        message = "Failed to build runtime pipeline from configuration"
                    );
                    Error::PipelineRuntimeError {
                        source: Box::new(e),
                    }
                })?;

            obs_evt_reporter.report(EngineEvent::ready(
                pipeline_key.clone(),
                Some("Pipeline initialization successful.".to_owned()),
            ));

            if let Some((_, startup_tx)) = internal_telemetry.as_ref() {
                let _ = startup_tx.send(Ok(()));
            }

            // Start the pipeline (this will use the current thread's Tokio runtime)
            runtime_pipeline
                .run_forever(
                    pipeline_key,
                    pipeline_context,
                    obs_evt_reporter,
                    metrics_reporter,
                    pipeline_ctrl_msg_tx,
                    pipeline_ctrl_msg_rx,
                )
                .map_err(|e| {
                    otel_error!(
                        "controller.pipeline_runtime_failed",
                        core_id = core_id.id,
                        error = %e,
                        message = "Pipeline terminated with a runtime error"
                    );
                    Error::PipelineRuntimeError {
                        source: Box::new(e),
                    }
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::engine::{ResolvedPipelineConfig, ResolvedPipelineRole};
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_config::policy::{CoreRange, Policies, ResourcesPolicy};
    use otap_df_config::settings::telemetry::logs::LogLevel;
    use otap_df_config::topic::{TopicAckPropagationMode, TopicBroadcastOnLagPolicy};
    use otap_df_engine::ExporterFactory;
    use otap_df_engine::ReceiverFactory;
    use otap_df_engine::config::{ExporterConfig, ReceiverConfig};
    use otap_df_engine::control::{PipelineControlMsg, pipeline_ctrl_msg_channel};
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::wiring_contract::WiringContract;
    use otap_df_telemetry::TracingSetup;
    use otap_df_telemetry::tracing_init::ProviderSetup;

    fn available_core_ids() -> Vec<CoreId> {
        vec![
            CoreId { id: 0 },
            CoreId { id: 1 },
            CoreId { id: 2 },
            CoreId { id: 3 },
            CoreId { id: 4 },
            CoreId { id: 5 },
            CoreId { id: 6 },
            CoreId { id: 7 },
        ]
    }

    fn to_ids(v: &[CoreId]) -> Vec<usize> {
        v.iter().map(|c| c.id).collect()
    }

    fn minimal_pipeline_config() -> PipelineConfig {
        PipelineConfig::from_yaml(
            "g".into(),
            "p".into(),
            r#"
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
        )
        .expect("minimal test pipeline config should parse")
    }

    fn resolved_pipeline_with_core_allocation(
        pipeline_group_id: &str,
        pipeline_id: &str,
        core_allocation: CoreAllocation,
    ) -> ResolvedPipelineConfig {
        let policies = Policies {
            resources: Some(ResourcesPolicy { core_allocation }),
            ..Default::default()
        };
        ResolvedPipelineConfig {
            pipeline_group_id: pipeline_group_id.to_string().into(),
            pipeline_id: pipeline_id.to_string().into(),
            pipeline: minimal_pipeline_config(),
            policies,
            role: ResolvedPipelineRole::Regular,
        }
    }

    fn global_topic_handle(
        declared: &DeclaredTopics<()>,
        topic_name: &str,
    ) -> otap_df_engine::topic::TopicHandle<()> {
        let declared_name = declared
            .global_names
            .get(topic_name)
            .expect("global topic must be declared");
        declared
            .broker
            .get_topic_required(declared_name)
            .expect("declared topic must exist in broker")
    }

    fn group_topic_handle(
        declared: &DeclaredTopics<()>,
        group_id: &str,
        topic_name: &str,
    ) -> otap_df_engine::topic::TopicHandle<()> {
        let key = (
            PipelineGroupId::from(group_id.to_owned()),
            TopicName::parse(topic_name).expect("topic name must parse"),
        );
        let declared_name = declared
            .group_names
            .get(&key)
            .expect("group topic must be declared");
        declared
            .broker
            .get_topic_required(declared_name)
            .expect("declared topic must exist in broker")
    }

    fn test_validate_config(
        _config: &serde_json::Value,
    ) -> Result<(), otap_df_config::error::Error> {
        Ok(())
    }

    fn test_receiver_create(
        _pipeline_ctx: PipelineContext,
        _node: otap_df_engine::node::NodeId,
        _node_config: Arc<NodeUserConfig>,
        _receiver_config: &ReceiverConfig,
    ) -> Result<ReceiverWrapper<()>, otap_df_config::error::Error> {
        panic!("test receiver factory should not be constructed")
    }

    fn test_exporter_create(
        _pipeline_ctx: PipelineContext,
        _node: otap_df_engine::node::NodeId,
        _node_config: Arc<NodeUserConfig>,
        _exporter_config: &ExporterConfig,
    ) -> Result<ExporterWrapper<()>, otap_df_config::error::Error> {
        panic!("test exporter factory should not be constructed")
    }

    static TEST_RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[
        ReceiverFactory {
            name: "urn:test:receiver:example",
            create: test_receiver_create,
            wiring_contract: WiringContract::UNRESTRICTED,
            validate_config: test_validate_config,
        },
        ReceiverFactory {
            name: "urn:otel:receiver:topic",
            create: test_receiver_create,
            wiring_contract: WiringContract::UNRESTRICTED,
            validate_config: test_validate_config,
        },
    ];

    static TEST_EXPORTER_FACTORIES: &[ExporterFactory<()>] = &[
        ExporterFactory {
            name: "urn:test:exporter:example",
            create: test_exporter_create,
            wiring_contract: WiringContract::UNRESTRICTED,
            validate_config: test_validate_config,
        },
        ExporterFactory {
            name: "urn:otel:exporter:topic",
            create: test_exporter_create,
            wiring_contract: WiringContract::UNRESTRICTED,
            validate_config: test_validate_config,
        },
    ];

    static TEST_PIPELINE_FACTORY: PipelineFactory<()> =
        PipelineFactory::new(TEST_RECEIVER_FACTORIES, &[], TEST_EXPORTER_FACTORIES);

    fn test_runtime(config: &OtelDataflowSpec) -> Arc<ControllerRuntime<()>> {
        let registry = TelemetryRegistryHandle::new();
        let observed_state_store =
            ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
        let observed_state_handle = observed_state_store.handle();
        let engine_event_reporter = observed_state_store.reporter(Default::default());
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(8);
        let declared_topics =
            Controller::<()>::declare_topics(config).expect("declared topics should be valid");

        Arc::new(ControllerRuntime::new(
            &TEST_PIPELINE_FACTORY,
            ControllerContext::new(registry),
            observed_state_store,
            observed_state_handle,
            engine_event_reporter,
            metrics_reporter,
            declared_topics,
            available_core_ids(),
            TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
            config.clone(),
        ))
    }

    fn engine_config_with_pipeline(pipeline_yaml: &str) -> OtelDataflowSpec {
        OtelDataflowSpec::from_yaml(&format!(
            r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
{pipeline_yaml}
"#
        ))
        .expect("engine config should parse")
    }

    fn register_existing_pipeline(runtime: &ControllerRuntime<()>, config: &OtelDataflowSpec) {
        register_pipeline(runtime, config, "g1", "p1");
    }

    fn register_pipeline(
        runtime: &ControllerRuntime<()>,
        config: &OtelDataflowSpec,
        group_id: &str,
        pipeline_id: &str,
    ) {
        let resolved = config
            .resolve()
            .pipelines
            .into_iter()
            .find(|pipeline| {
                pipeline.role == ResolvedPipelineRole::Regular
                    && pipeline.pipeline_group_id.as_ref() == group_id
                    && pipeline.pipeline_id.as_ref() == pipeline_id
            })
            .expect("resolved pipeline should exist");
        runtime.register_committed_pipeline(resolved, 0);
    }

    fn register_runtime_instance(
        runtime: &ControllerRuntime<()>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        core_id: usize,
        generation: u64,
        lifecycle: RuntimeInstanceLifecycle,
    ) -> PipelineCtrlMsgReceiver<()> {
        let (tx, rx) = pipeline_ctrl_msg_channel(4);
        let control_sender: Arc<dyn PipelineAdminSender> = Arc::new(tx.clone());
        let is_active = matches!(&lifecycle, RuntimeInstanceLifecycle::Active);
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        _ = state.runtime_instances.insert(
            DeployedPipelineKey {
                pipeline_group_id: pipeline_group_id.to_owned().into(),
                pipeline_id: pipeline_id.to_owned().into(),
                core_id,
                deployment_generation: generation,
            },
            RuntimeInstanceRecord {
                control_sender: Some(control_sender),
                lifecycle,
            },
        );
        if is_active {
            state.active_instances += 1;
        }
        rx
    }

    fn wait_for_shutdown_state(
        runtime: &ControllerRuntime<()>,
        shutdown_id: &str,
        expected_state: &str,
    ) -> PipelineShutdownStatus {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let status = runtime
                .shutdown_status_snapshot(shutdown_id)
                .expect("shutdown should exist");
            if status.state == expected_state {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for shutdown {shutdown_id} to reach state {expected_state}, current state: {}",
                status.state
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn wait_for_shutdown_message(
        receiver: &mut PipelineCtrlMsgReceiver<()>,
    ) -> PipelineControlMsg<()> {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Ok(message) = receiver.try_recv() {
                return message;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for shutdown control message"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    #[test]
    fn prepare_rollout_plan_accepts_core_allocation_scale_up() {
        let config = engine_config_with_pipeline(
            r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);
        let _receiver =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
        )
        .expect("replacement should parse");

        let plan = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement,
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect("core allocation changes should be planned");

        assert_eq!(plan.action, RolloutAction::Resize);
        assert_eq!(plan.current_assigned_cores, vec![0]);
        assert_eq!(plan.target_assigned_cores, vec![0, 1]);
        assert_eq!(plan.common_assigned_cores, vec![0]);
        assert_eq!(plan.added_assigned_cores, vec![1]);
        assert!(plan.removed_assigned_cores.is_empty());
        assert_eq!(plan.resize_start_cores, vec![1]);
        assert!(plan.resize_stop_cores.is_empty());
        assert_eq!(plan.target_generation, 0);
        assert_eq!(
            plan.rollout
                .cores
                .iter()
                .map(|core| core.core_id)
                .collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn prepare_rollout_plan_accepts_core_allocation_scale_down() {
        let config = engine_config_with_pipeline(
            r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);
        let _receiver0 =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
        let _receiver1 =
            register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);

        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
        )
        .expect("replacement should parse");

        let plan = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement,
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect("core allocation changes should be planned");

        assert_eq!(plan.action, RolloutAction::Resize);
        assert_eq!(plan.current_assigned_cores, vec![0, 1]);
        assert_eq!(plan.target_assigned_cores, vec![0]);
        assert_eq!(plan.common_assigned_cores, vec![0]);
        assert!(plan.added_assigned_cores.is_empty());
        assert_eq!(plan.removed_assigned_cores, vec![1]);
        assert!(plan.resize_start_cores.is_empty());
        assert_eq!(plan.resize_stop_cores, vec![1]);
        assert_eq!(plan.target_generation, 0);
        assert_eq!(
            plan.rollout
                .cores
                .iter()
                .map(|core| core.core_id)
                .collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn prepare_rollout_plan_returns_noop_for_identical_active_pipeline() {
        let config = engine_config_with_pipeline(
            r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);
        let _receiver =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
        )
        .expect("replacement should parse");

        let plan = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement,
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect("identical updates should be planned");

        assert_eq!(plan.action, RolloutAction::NoOp);
        assert_eq!(plan.target_generation, 0);
        assert!(plan.rollout.cores.is_empty());
        assert!(plan.resize_start_cores.is_empty());
        assert!(plan.resize_stop_cores.is_empty());
    }

    #[test]
    fn spawn_rollout_returns_immediate_success_for_noop() {
        let config = engine_config_with_pipeline(
            r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);
        let _receiver =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
        )
        .expect("replacement should parse");

        let plan = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement,
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect("identical updates should be planned");

        let status = runtime
            .spawn_rollout(plan)
            .expect("noop rollout should succeed");

        assert_eq!(status.action, "noop");
        assert_eq!(status.state, "succeeded");
        assert_eq!(status.target_generation, 0);
        assert!(status.cores.is_empty());

        let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
        let details = runtime
            .pipeline_details_snapshot(&pipeline_key)
            .expect("group should exist")
            .expect("pipeline should exist");
        assert_eq!(details.active_generation, Some(0));
        assert!(details.rollout.is_none());

        let rollout = runtime
            .rollout_status_snapshot(&status.rollout_id)
            .expect("completed rollout should remain queryable");
        assert_eq!(rollout.state, "succeeded");
    }

    #[test]
    fn prepare_rollout_plan_keeps_replace_when_runtime_shape_changes() {
        let config = engine_config_with_pipeline(
            r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);
        let _receiver =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
        )
        .expect("replacement should parse");

        let plan = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement,
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect("runtime shape changes should still be planned");

        assert_eq!(plan.action, RolloutAction::Replace);
        assert_eq!(plan.target_generation, 1);
        assert_eq!(plan.common_assigned_cores, vec![0]);
        assert_eq!(plan.added_assigned_cores, vec![1]);
        assert!(plan.resize_start_cores.is_empty());
        assert!(plan.resize_stop_cores.is_empty());
        assert_eq!(
            plan.rollout
                .cores
                .iter()
                .map(|core| core.core_id)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[test]
    fn prepare_rollout_plan_rejects_topic_runtime_mutation() {
        let config = OtelDataflowSpec::from_yaml(
            r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          to_topic:
            type: "urn:otel:exporter:topic"
            config:
              topic: shared
        connections:
          - from: receiver
            to: to_topic
"#,
        )
        .expect("config should parse");
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);

        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  from_topic:
    type: "urn:otel:receiver:topic"
    config:
      topic: shared
      subscription:
        mode: balanced
        group: workers
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: from_topic
    to: exporter
"#,
        )
        .expect("replacement should parse");

        let err = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement,
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect_err("topic runtime changes should be rejected");

        match err {
            ControlPlaneError::InvalidRequest { message } => {
                assert!(message.contains("topic broker mutation"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn prepare_rollout_plan_rejects_concurrent_rollout_for_same_pipeline() {
        let config = engine_config_with_pipeline(
            r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);

        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
        )
        .expect("replacement should parse");
        let plan = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement.clone(),
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect("first rollout plan should be accepted");
        runtime
            .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
            .expect("rollout should register");

        let err = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement,
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect_err("second rollout should conflict");

        assert_eq!(err, ControlPlaneError::RolloutConflict);
    }

    #[test]
    fn pipeline_details_returns_committed_config_while_rollout_is_pending() {
        let config = engine_config_with_pipeline(
            r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);

        let replacement = PipelineConfig::from_yaml(
            "g1".into(),
            "p1".into(),
            r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
        )
        .expect("replacement should parse");
        let plan = runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReplacePipelineRequest {
                    pipeline: replacement.clone(),
                    step_timeout_secs: 60,
                    drain_timeout_secs: 60,
                },
            )
            .expect("rollout plan should be accepted");
        runtime
            .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
            .expect("rollout should register");

        let details = runtime
            .pipeline_details_snapshot(&PipelineKey::new("g1".into(), "p1".into()))
            .expect("group should exist")
            .expect("pipeline details should exist");

        let mut committed_nodes = details
            .pipeline
            .node_iter()
            .map(|(node_id, _)| node_id.as_ref().to_owned())
            .collect::<Vec<_>>();
        committed_nodes.sort();
        assert_eq!(
            committed_nodes,
            vec!["exporter".to_owned(), "receiver".to_owned()]
        );
        assert_eq!(details.active_generation, Some(0));
        assert_eq!(
            details
                .rollout
                .expect("pending rollout summary should be present")
                .target_generation,
            1
        );
    }

    #[test]
    fn request_shutdown_pipeline_rejects_missing_group() {
        let config = engine_config_with_pipeline(
            r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);

        let err = runtime
            .request_shutdown_pipeline("missing", "p1", 5)
            .expect_err("missing group should be rejected");

        assert_eq!(err, ControlPlaneError::GroupNotFound);
    }

    #[test]
    fn request_shutdown_pipeline_rejects_missing_pipeline() {
        let config = engine_config_with_pipeline(
            r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);

        let err = runtime
            .request_shutdown_pipeline("g1", "missing", 5)
            .expect_err("missing pipeline should be rejected");

        assert_eq!(err, ControlPlaneError::PipelineNotFound);
    }

    #[test]
    fn request_shutdown_pipeline_rejects_active_rollout() {
        let config = engine_config_with_pipeline(
            r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);

        let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        _ = state
            .active_rollouts
            .insert(pipeline_key, "rollout-42".to_owned());
        drop(state);

        let err = runtime
            .request_shutdown_pipeline("g1", "p1", 5)
            .expect_err("active rollout should conflict");

        assert_eq!(err, ControlPlaneError::RolloutConflict);
    }

    #[test]
    fn request_shutdown_pipeline_rejects_active_shutdown() {
        let config = engine_config_with_pipeline(
            r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);
        let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
        let shutdown = ShutdownRecord::new(
            "shutdown-0".to_owned(),
            "g1".into(),
            "p1".into(),
            vec![ShutdownCoreProgress {
                core_id: 0,
                deployment_generation: 0,
                state: "pending".to_owned(),
                updated_at: timestamp_now(),
                detail: None,
            }],
        );
        runtime
            .insert_shutdown(&pipeline_key, shutdown)
            .expect("shutdown should register");

        let err = runtime
            .request_shutdown_pipeline("g1", "p1", 5)
            .expect_err("active shutdown should conflict");

        assert_eq!(err, ControlPlaneError::RolloutConflict);
    }

    #[test]
    fn request_shutdown_pipeline_rejects_already_stopped_pipeline() {
        let config = engine_config_with_pipeline(
            r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);

        let err = runtime
            .request_shutdown_pipeline("g1", "p1", 5)
            .expect_err("already stopped pipeline should be rejected");

        match err {
            ControlPlaneError::InvalidRequest { message } => {
                assert!(message.contains("already stopped"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn request_shutdown_pipeline_targets_only_active_instances_for_pipeline() {
        let config = OtelDataflowSpec::from_yaml(
            r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        )
        .expect("config should parse");
        let runtime = test_runtime(&config);
        register_pipeline(&runtime, &config, "g1", "p1");
        register_pipeline(&runtime, &config, "g1", "p2");

        let mut p1_core0 =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
        let mut p1_core1 =
            register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);
        let mut p1_exited = register_runtime_instance(
            &runtime,
            "g1",
            "p1",
            2,
            0,
            RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Success),
        );
        let mut p2_core0 =
            register_runtime_instance(&runtime, "g1", "p2", 3, 0, RuntimeInstanceLifecycle::Active);

        let _shutdown = runtime
            .request_shutdown_pipeline("g1", "p1", 5)
            .expect("shutdown request should be accepted");

        assert!(matches!(
            wait_for_shutdown_message(&mut p1_core0),
            PipelineControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
        ));
        assert!(matches!(
            wait_for_shutdown_message(&mut p1_core1),
            PipelineControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
        ));
        assert!(
            p1_exited.try_recv().is_err(),
            "exited runtime should not receive shutdown"
        );
        assert!(
            p2_core0.try_recv().is_err(),
            "other pipelines must not receive shutdown"
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let state = runtime
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let p1_core0_released = state
                .runtime_instances
                .get(&DeployedPipelineKey {
                    pipeline_group_id: "g1".into(),
                    pipeline_id: "p1".into(),
                    core_id: 0,
                    deployment_generation: 0,
                })
                .and_then(|instance| instance.control_sender.as_ref())
                .is_none();
            let p1_core1_released = state
                .runtime_instances
                .get(&DeployedPipelineKey {
                    pipeline_group_id: "g1".into(),
                    pipeline_id: "p1".into(),
                    core_id: 1,
                    deployment_generation: 0,
                })
                .and_then(|instance| instance.control_sender.as_ref())
                .is_none();
            let p2_core0_retained = state
                .runtime_instances
                .get(&DeployedPipelineKey {
                    pipeline_group_id: "g1".into(),
                    pipeline_id: "p2".into(),
                    core_id: 3,
                    deployment_generation: 0,
                })
                .and_then(|instance| instance.control_sender.as_ref())
                .is_some();
            drop(state);

            if p1_core0_released && p1_core1_released && p2_core0_retained {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for targeted control senders to be released"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    #[test]
    fn request_shutdown_pipeline_tracks_completion() {
        let config = engine_config_with_pipeline(
            r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);

        let mut core0 =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
        let mut core1 =
            register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);

        let shutdown = runtime
            .request_shutdown_pipeline("g1", "p1", 5)
            .expect("shutdown request should be accepted");
        assert_eq!(shutdown.state, "pending");

        assert!(matches!(
            wait_for_shutdown_message(&mut core0),
            PipelineControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
        ));
        assert!(matches!(
            wait_for_shutdown_message(&mut core1),
            PipelineControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
        ));

        runtime.note_instance_exit(
            DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p1".into(),
                core_id: 0,
                deployment_generation: 0,
            },
            RuntimeInstanceExit::Success,
        );
        runtime.note_instance_exit(
            DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p1".into(),
                core_id: 1,
                deployment_generation: 0,
            },
            RuntimeInstanceExit::Success,
        );

        let status = wait_for_shutdown_state(&runtime, &shutdown.shutdown_id, "succeeded");
        assert_eq!(status.cores.len(), 2);
        assert!(status.cores.iter().all(|core| core.state == "exited"));

        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            !state
                .active_shutdowns
                .contains_key(&PipelineKey::new("g1".into(), "p1".into()))
        );
    }

    #[test]
    fn request_shutdown_pipeline_tracks_timeout_failure() {
        let config = engine_config_with_pipeline(
            r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
        );
        let runtime = test_runtime(&config);
        register_existing_pipeline(&runtime, &config);

        let mut core0 =
            register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

        let shutdown = runtime
            .request_shutdown_pipeline("g1", "p1", 1)
            .expect("shutdown request should be accepted");
        assert!(matches!(
            wait_for_shutdown_message(&mut core0),
            PipelineControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
        ));

        let status = wait_for_shutdown_state(&runtime, &shutdown.shutdown_id, "failed");
        assert!(
            status
                .failure_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("timed out waiting"))
        );
        assert_eq!(status.cores.len(), 1);
        assert_eq!(status.cores[0].state, "failed");
    }

    #[test]
    fn select_all_cores_by_default() {
        let core_allocation = CoreAllocation::AllCores;
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }

    #[test]
    fn select_limited_by_num_cores() {
        let core_allocation = CoreAllocation::CoreCount { count: 4 };
        let available_core_ids = available_core_ids();
        let result = Controller::<()>::select_cores_for_allocation(
            available_core_ids.clone(),
            &core_allocation,
        )
        .unwrap();
        assert_eq!(result.len(), 4);
        let expected_ids: Vec<usize> = available_core_ids
            .into_iter()
            .take(4)
            .map(|c| c.id)
            .collect();
        assert_eq!(to_ids(&result), expected_ids);
    }

    #[test]
    fn select_with_valid_single_core_range() {
        let available_core_ids = available_core_ids();
        let first_id = available_core_ids[0].id;
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange {
                start: first_id,
                end: first_id,
            }],
        };
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![first_id]);
    }

    #[test]
    fn select_with_valid_multi_core_range() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 5 },
                CoreRange { start: 6, end: 6 },
            ],
        };
        let available_core_ids = available_core_ids();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn select_with_inverted_range_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange { start: 2, end: 1 }],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, .. } => {
                assert_eq!(alloc, core_allocation);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_out_of_bounds_range_errors() {
        let start = 100;
        let end = 110;
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange { start, end }],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, .. } => {
                assert_eq!(alloc, core_allocation);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_zero_count_uses_all_cores() {
        let core_allocation = CoreAllocation::CoreCount { count: 0 };
        let available_core_ids = available_core_ids();
        let expected_core_ids = available_core_ids.clone();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), to_ids(&expected_core_ids));
    }

    #[test]
    fn select_with_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 5 },
                CoreRange { start: 4, end: 7 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_fully_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 6 },
                CoreRange { start: 3, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_identical_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 3, end: 5 },
                CoreRange { start: 3, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn select_with_adjacent_ranges_succeeds() {
        // Adjacent but non-overlapping ranges should work
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 2, end: 3 },
                CoreRange { start: 4, end: 5 },
            ],
        };
        let available_core_ids = available_core_ids();
        let result =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap();
        assert_eq!(to_ids(&result), vec![2, 3, 4, 5]);
    }

    #[test]
    fn select_with_multiple_overlapping_ranges_errors() {
        let core_allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 1, end: 3 },
                CoreRange { start: 2, end: 4 },
                CoreRange { start: 5, end: 6 },
            ],
        };
        let available_core_ids = available_core_ids();
        let err =
            Controller::<()>::select_cores_for_allocation(available_core_ids, &core_allocation)
                .unwrap_err();
        match err {
            Error::InvalidCoreAllocation { alloc, message, .. } => {
                assert_eq!(alloc, core_allocation);
                assert!(
                    message.contains("overlap"),
                    "Expected overlap error message, got: {}",
                    message
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn preflight_fails_fast_when_later_pipeline_allocation_is_invalid() {
        let pipelines = vec![
            resolved_pipeline_with_core_allocation(
                "g1",
                "p1",
                CoreAllocation::CoreCount { count: 2 },
            ),
            resolved_pipeline_with_core_allocation(
                "g1",
                "p2",
                CoreAllocation::CoreSet {
                    set: vec![CoreRange {
                        start: 999,
                        end: 999,
                    }],
                },
            ),
        ];

        let err = Controller::<()>::preflight_pipeline_core_allocations(
            &pipelines,
            &available_core_ids(),
        )
        .expect_err("preflight should fail");
        match err {
            Error::InvalidCoreAllocation { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn preflight_succeeds_and_allows_cross_pipeline_core_overlap() {
        let pipelines = vec![
            resolved_pipeline_with_core_allocation(
                "g1",
                "p1",
                CoreAllocation::CoreSet {
                    set: vec![CoreRange { start: 1, end: 2 }],
                },
            ),
            resolved_pipeline_with_core_allocation(
                "g1",
                "p2",
                CoreAllocation::CoreSet {
                    set: vec![CoreRange { start: 2, end: 3 }],
                },
            ),
        ];

        let assignments = Controller::<()>::preflight_pipeline_core_allocations(
            &pipelines,
            &available_core_ids(),
        )
        .expect("preflight should succeed");

        assert_eq!(assignments.len(), 2);
        assert_eq!(to_ids(&assignments[0]), vec![1, 2]);
        assert_eq!(to_ids(&assignments[1]), vec![2, 3]);
    }

    #[test]
    fn declare_topics_accepts_default_and_explicit_in_memory_backend() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  global_default: {}
  global_mem:
    backend: in_memory
groups:
  g1:
    topics:
      local_default: {}
      local_mem:
        backend: in_memory
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");

        assert_eq!(declared.broker.topic_names().len(), 4);
    }

    #[test]
    fn declare_topics_rejects_unimplemented_backend_kind() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  global_quiver:
    backend: quiver
groups:
  g1:
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        match Controller::<()>::declare_topics(&config) {
            Err(Error::UnsupportedTopicBackend { topic, backend }) => {
                assert_eq!(topic.as_ref(), "global::global_quiver");
                assert_eq!(backend, TopicBackendKind::Quiver);
            }
            Ok(_) => panic!("quiver backend should be rejected"),
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_topic_runtime_support_rejects_unsupported_mode_for_backend_capabilities() {
        let topic = TopicName::parse("test_topic").expect("topic name should parse");
        let capabilities = TopicBackendCapabilities {
            supports_balanced_only: true,
            supports_broadcast_only: false,
            supports_mixed: false,
            supports_broadcast_on_lag_drop_oldest: true,
            supports_broadcast_on_lag_disconnect: false,
            supports_ack_propagation_disabled: true,
            supports_ack_propagation_auto: true,
        };

        let err = Controller::<()>::validate_topic_runtime_support_with_capabilities(
            &topic,
            TopicBackendKind::InMemory,
            &TopicSpec::default().policies,
            InferredTopicMode::BroadcastOnly,
            capabilities,
        )
        .expect_err("broadcast_only should be rejected");

        match err {
            Error::UnsupportedTopicMode {
                topic,
                backend,
                mode,
            } => {
                assert_eq!(topic.as_ref(), "test_topic");
                assert_eq!(backend, TopicBackendKind::InMemory);
                assert_eq!(mode, "broadcast_only");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_topic_runtime_support_rejects_unsupported_broadcast_lag_policy() {
        let topic = TopicName::parse("test_topic").expect("topic name should parse");
        let capabilities = TopicBackendCapabilities {
            supports_balanced_only: true,
            supports_broadcast_only: true,
            supports_mixed: true,
            supports_broadcast_on_lag_drop_oldest: true,
            supports_broadcast_on_lag_disconnect: false,
            supports_ack_propagation_disabled: true,
            supports_ack_propagation_auto: true,
        };
        let mut spec = TopicSpec::default();
        spec.policies.broadcast.on_lag = TopicBroadcastOnLagPolicy::Disconnect;

        let err = Controller::<()>::validate_topic_runtime_support_with_capabilities(
            &topic,
            TopicBackendKind::InMemory,
            &spec.policies,
            InferredTopicMode::BroadcastOnly,
            capabilities,
        )
        .expect_err("disconnect lag policy should be rejected");

        match err {
            Error::UnsupportedTopicPolicy {
                topic,
                backend,
                policy,
                value,
            } => {
                assert_eq!(topic.as_ref(), "test_topic");
                assert_eq!(backend, TopicBackendKind::InMemory);
                assert_eq!(policy, "broadcast.on_lag");
                assert_eq!(value, "disconnect");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_topic_runtime_support_rejects_unsupported_ack_propagation_policy() {
        let topic = TopicName::parse("test_topic").expect("topic name should parse");
        let capabilities = TopicBackendCapabilities {
            supports_balanced_only: true,
            supports_broadcast_only: true,
            supports_mixed: true,
            supports_broadcast_on_lag_drop_oldest: true,
            supports_broadcast_on_lag_disconnect: true,
            supports_ack_propagation_disabled: true,
            supports_ack_propagation_auto: false,
        };
        let mut spec = TopicSpec::default();
        spec.policies.ack_propagation.mode = TopicAckPropagationMode::Auto;

        let err = Controller::<()>::validate_topic_runtime_support_with_capabilities(
            &topic,
            TopicBackendKind::InMemory,
            &spec.policies,
            InferredTopicMode::BalancedOnly,
            capabilities,
        )
        .expect_err("ack auto should be rejected");

        match err {
            Error::UnsupportedTopicPolicy {
                topic,
                backend,
                policy,
                value,
            } => {
                assert_eq!(topic.as_ref(), "test_topic");
                assert_eq!(backend, TopicBackendKind::InMemory);
                assert_eq!(policy, "ack_propagation");
                assert_eq!(value, "auto");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn declare_topics_rejects_same_pipeline_topic_wiring_cycle() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  loop: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          loop_receiver:
            type: "urn:otel:receiver:topic"
            config:
              topic: loop
          loop_exporter:
            type: "urn:otel:exporter:topic"
            config:
              topic: loop
        connections:
          - from: loop_receiver
            to: loop_exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let err = Controller::<()>::declare_topics(&config)
            .err()
            .expect("same-pipeline topic feedback loop should be rejected");
        match err {
            Error::TopicWiringCycleDetected { cycle } => {
                assert!(cycle.len() >= 4, "unexpected cycle path: {cycle:?}");
                assert_eq!(cycle.first(), cycle.last());
                assert!(cycle.contains(&"topic:global::loop".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p1/loop_receiver".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p1/loop_exporter".to_owned()));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn declare_topics_rejects_cross_pipeline_topic_wiring_cycle() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  topic_a: {}
  topic_b: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          from_topic_a:
            type: "urn:otel:receiver:topic"
            config:
              topic: topic_a
          to_topic_b:
            type: "urn:otel:exporter:topic"
            config:
              topic: topic_b
        connections:
          - from: from_topic_a
            to: to_topic_b
      p2:
        nodes:
          from_topic_b:
            type: "urn:otel:receiver:topic"
            config:
              topic: topic_b
          to_topic_a:
            type: "urn:otel:exporter:topic"
            config:
              topic: topic_a
        connections:
          - from: from_topic_b
            to: to_topic_a
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let err = Controller::<()>::declare_topics(&config)
            .err()
            .expect("cross-pipeline topic cycle should be rejected");
        match err {
            Error::TopicWiringCycleDetected { cycle } => {
                assert!(cycle.len() >= 6, "unexpected cycle path: {cycle:?}");
                assert_eq!(cycle.first(), cycle.last());
                assert!(cycle.contains(&"topic:global::topic_a".to_owned()));
                assert!(cycle.contains(&"topic:global::topic_b".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p1/from_topic_a".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p1/to_topic_b".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p2/from_topic_b".to_owned()));
                assert!(cycle.contains(&"pipeline:g1/p2/to_topic_a".to_owned()));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn declare_topics_infers_balanced_only_for_single_consumer_group() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  balanced_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: balanced_topic
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "balanced_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(matches!(
            topic.subscribe(
                otap_df_engine::topic::SubscriptionMode::Broadcast,
                otap_df_engine::topic::SubscriberOptions::default(),
            ),
            Err(otap_df_engine::error::Error::SubscribeBroadcastNotSupported)
        ));
    }

    #[test]
    fn declare_topics_infers_broadcast_only_when_only_broadcast_receivers_exist() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  broadcast_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: broadcast_topic
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "broadcast_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(matches!(
            topic.subscribe(
                otap_df_engine::topic::SubscriptionMode::Balanced { group: "g1".into() },
                otap_df_engine::topic::SubscriberOptions::default(),
            ),
            Err(otap_df_engine::error::Error::SubscribeBalancedNotSupported)
        ));
    }

    #[test]
    fn declare_topics_keeps_mixed_for_multiple_balanced_groups() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  mixed_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: mixed_topic
              subscription:
                mode: balanced
                group: g1
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
      p2:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: mixed_topic
              subscription:
                mode: balanced
                group: g2
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "mixed_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced { group: "g3".into() },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
    }

    #[test]
    fn declare_topics_defaults_to_mixed_when_topic_has_no_receivers() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  idle_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "idle_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced { group: "g1".into() },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
    }

    #[test]
    fn declare_topics_inference_respects_group_local_topic_shadowing() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    topics:
      shared: {}
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: shared
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let global_topic = global_topic_handle(&declared, "shared");
        let group_topic = group_topic_handle(&declared, "g1", "shared");

        assert!(
            global_topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(matches!(
            group_topic.subscribe(
                otap_df_engine::topic::SubscriptionMode::Broadcast,
                otap_df_engine::topic::SubscriberOptions::default(),
            ),
            Err(otap_df_engine::error::Error::SubscribeBroadcastNotSupported)
        ));
    }

    #[test]
    fn declare_topics_engine_default_force_mixed_disables_optimization() {
        let yaml = r#"
version: otel_dataflow/v1
engine:
  topics:
    impl_selection: force_mixed
topics:
  balanced_topic: {}
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: balanced_topic
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "balanced_topic");

        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Broadcast,
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
    }

    #[test]
    fn declare_topics_topic_override_auto_wins_over_engine_force_mixed() {
        let yaml = r#"
version: otel_dataflow/v1
engine:
  topics:
    impl_selection: force_mixed
topics:
  balanced_topic:
    impl_selection: auto
groups:
  g1:
    pipelines:
      p1:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: balanced_topic
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "balanced_topic");

        assert!(matches!(
            topic.subscribe(
                otap_df_engine::topic::SubscriptionMode::Broadcast,
                otap_df_engine::topic::SubscriberOptions::default(),
            ),
            Err(otap_df_engine::error::Error::SubscribeBroadcastNotSupported)
        ));
        assert!(
            topic
                .subscribe(
                    otap_df_engine::topic::SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    otap_df_engine::topic::SubscriberOptions::default(),
                )
                .is_ok()
        );
    }

    #[test]
    fn build_pipeline_topic_set_wires_topic_queue_on_full_policy() {
        let yaml = r#"
version: otel_dataflow/v1
topics:
  global_drop:
    policies:
      balanced:
        queue_capacity: 8
        on_full: drop_newest
      broadcast:
        queue_capacity: 8
        on_lag: disconnect
      ack_propagation:
        mode: auto
        max_in_flight: 21
        timeout: 45s
groups:
  g1:
    topics:
      local_block:
        policies:
          balanced:
            queue_capacity: 8
            on_full: block
          broadcast:
            queue_capacity: 8
            on_lag: drop_oldest
          ack_propagation:
            mode: disabled
            max_in_flight: 22
            timeout: 46s
      # Same local alias as global to verify group-local override path.
      global_drop:
        policies:
          balanced:
            queue_capacity: 8
            on_full: block
          broadcast:
            queue_capacity: 8
            on_lag: drop_oldest
          ack_propagation:
            mode: disabled
            max_in_flight: 23
            timeout: 47s
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let group_id: PipelineGroupId = "g1".into();
        let pipeline_id: PipelineId = "p1".into();
        let set = Controller::<()>::build_pipeline_topic_set(
            &config,
            &declared,
            &group_id,
            &pipeline_id,
            0,
        )
        .expect("topic set should build");

        let local_block = set
            .get_required(TopicName::from("local_block"))
            .expect("local_block topic must exist");
        assert_eq!(
            local_block.default_queue_on_full(),
            otap_df_config::topic::TopicQueueOnFullPolicy::Block
        );
        assert_eq!(
            local_block.default_ack_propagation_mode(),
            otap_df_config::topic::TopicAckPropagationMode::Disabled
        );
        assert_eq!(
            local_block.broadcast_on_lag_policy(),
            otap_df_config::topic::TopicBroadcastOnLagPolicy::DropOldest
        );
        assert_eq!(
            local_block.default_publish_outcome_config().max_in_flight,
            22
        );
        assert_eq!(
            local_block.default_publish_outcome_config().timeout,
            Duration::from_secs(46)
        );

        // group-local declaration must override global policy for same local name
        let overridden = set
            .get_required(TopicName::from("global_drop"))
            .expect("overridden topic must exist");
        assert_eq!(
            overridden.default_queue_on_full(),
            otap_df_config::topic::TopicQueueOnFullPolicy::Block
        );
        assert_eq!(
            overridden.default_ack_propagation_mode(),
            otap_df_config::topic::TopicAckPropagationMode::Disabled
        );
        assert_eq!(
            overridden.broadcast_on_lag_policy(),
            otap_df_config::topic::TopicBroadcastOnLagPolicy::DropOldest
        );
        assert_eq!(
            overridden.default_publish_outcome_config().max_in_flight,
            23
        );
        assert_eq!(
            overridden.default_publish_outcome_config().timeout,
            Duration::from_secs(47)
        );
    }

    #[tokio::test]
    async fn declare_topics_preserves_separate_balanced_and_broadcast_capacities() {
        let yaml = r#"
version: otel_dataflow/v1
engine:
  topics:
    impl_selection: force_mixed
topics:
  mixed_topic:
    policies:
      balanced:
        queue_capacity: 1
      broadcast:
        queue_capacity: 3
        on_lag: disconnect
groups:
  g1:
    pipelines:
      balanced_consumer:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: mixed_topic
              subscription:
                mode: balanced
                group: workers
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
      broadcast_consumer:
        nodes:
          recv:
            type: "urn:otel:receiver:topic"
            config:
              topic: mixed_topic
              subscription:
                mode: broadcast
          sink:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: recv
            to: sink
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("test config should parse");
        let declared = Controller::<()>::declare_topics(&config).expect("topics should declare");
        let topic = global_topic_handle(&declared, "mixed_topic");

        let mut balanced = topic
            .subscribe(
                otap_df_engine::topic::SubscriptionMode::Balanced {
                    group: "workers".into(),
                },
                otap_df_engine::topic::SubscriberOptions::default(),
            )
            .expect("balanced subscription should succeed");
        let mut broadcast = topic
            .subscribe(
                otap_df_engine::topic::SubscriptionMode::Broadcast,
                otap_df_engine::topic::SubscriberOptions::default(),
            )
            .expect("broadcast subscription should succeed");

        assert_eq!(
            topic
                .try_publish(Arc::new(()))
                .expect("publish should succeed"),
            otap_df_engine::topic::PublishOutcome::Published
        );
        assert_eq!(
            topic
                .try_publish(Arc::new(()))
                .expect("publish should still reach broadcast"),
            otap_df_engine::topic::PublishOutcome::DroppedOnFull
        );
        assert_eq!(
            topic
                .try_publish(Arc::new(()))
                .expect("publish should still reach broadcast"),
            otap_df_engine::topic::PublishOutcome::DroppedOnFull
        );
        assert_eq!(
            topic.broadcast_on_lag_policy(),
            otap_df_config::topic::TopicBroadcastOnLagPolicy::Disconnect
        );
        topic.close();

        let mut balanced_messages = 0usize;
        while let Ok(item) = balanced.recv().await {
            match item {
                otap_df_engine::topic::RecvItem::Message(_) => balanced_messages += 1,
                otap_df_engine::topic::RecvItem::Lagged { missed } => {
                    panic!("unexpected lag for balanced subscription: missed={missed}");
                }
            }
        }
        assert_eq!(balanced_messages, 1);

        let mut broadcast_messages = 0usize;
        while let Ok(item) = broadcast.recv().await {
            match item {
                otap_df_engine::topic::RecvItem::Message(_) => broadcast_messages += 1,
                otap_df_engine::topic::RecvItem::Lagged { missed } => {
                    panic!("unexpected lag with broadcast capacity 3: missed={missed}");
                }
            }
        }
        assert_eq!(broadcast_messages, 3);
    }
}
