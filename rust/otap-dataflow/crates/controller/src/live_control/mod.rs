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
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

mod execution;
mod planning;
mod runtime;
mod state;

#[cfg(test)]
use self::state::TERMINAL_OPERATION_RETENTION_TTL;
use self::state::{
    ActiveRuntimeCoreState, CandidateRolloutPlan, CandidateShutdownPlan, ControllerRuntimeState,
    LogicalPipelineRecord, RolloutAction, RolloutCoreProgress, RolloutExecutionError,
    RolloutLifecycleState, RolloutRecord, RuntimeInstanceLifecycle, RuntimeInstanceRecord,
    ShutdownCoreProgress, ShutdownLifecycleState, ShutdownRecord, TERMINAL_ROLLOUT_RETENTION_LIMIT,
    TERMINAL_SHUTDOWN_RETENTION_LIMIT, TopicRuntimeProfile, is_expired, timestamp_now,
};
pub(crate) use self::state::{PanicReport, RuntimeInstanceError, RuntimeInstanceExit};

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
    pub(super) pipeline_key: DeployedPipelineKey,
    pub(super) control_sender: Arc<dyn PipelineAdminSender>,
    pub(super) _marker: std::marker::PhantomData<PData>,
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
                pending_instance_exits: HashMap::new(),
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
        if let Ok(active_cores) = self.assigned_cores_for_resolved(&resolved) {
            self.observed_state_store
                .set_pipeline_active_cores(pipeline_key.clone(), active_cores);
        }
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

    /// Applies a terminal instance exit to controller state after the instance
    /// has been registered as active.
    fn apply_instance_exit_locked(
        state: &mut ControllerRuntimeState,
        pipeline_key: &DeployedPipelineKey,
        exit: &RuntimeInstanceExit,
    ) -> bool {
        if let Some(instance) = state.runtime_instances.get_mut(pipeline_key) {
            instance.lifecycle = RuntimeInstanceLifecycle::Exited(exit.clone());
        }
        state.active_instances = state.active_instances.saturating_sub(1);
        if let RuntimeInstanceExit::Error(error) = exit {
            if state.first_error.is_none() {
                state.first_error = Some(error.message.clone());
            }
        }
        let logical_pipeline_key = PipelineKey::new(
            pipeline_key.pipeline_group_id.clone(),
            pipeline_key.pipeline_id.clone(),
        );
        Self::prune_exited_runtime_instances_for_pipeline_locked(state, &logical_pipeline_key)
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

#[cfg(test)]
#[path = "../live_control_tests.rs"]
mod tests;
