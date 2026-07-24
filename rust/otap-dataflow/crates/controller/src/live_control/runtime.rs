// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime-instance launch, shutdown, and exit reporting.
//!
//! This module owns the boundary between controller state and actual pipeline
//! threads. It registers launched instances, reconciles early exits, sends
//! shutdown control messages, waits for readiness/exit transitions, and exposes
//! global runtime shutdown/error helpers used by controller teardown.

use super::*;

enum RecoveryReadyError {
    Cancelled,
    Failed(String),
}

struct RuntimeRecoveryAttempt {
    attempt: usize,
    target_key: DeployedPipelineKey,
    resolved: ResolvedPipelineConfig,
    backoff: Duration,
}

enum RuntimeRecoveryAttemptDecision {
    Cancelled,
    Exhausted,
    Attempt(Box<RuntimeRecoveryAttempt>),
}

/// Formats a deployed instance compactly for aggregated operator errors.
fn deployed_instance_label(deployed_key: &DeployedPipelineKey) -> String {
    format!(
        "{}:{} core={} generation={}",
        deployed_key.pipeline_group_id.as_ref(),
        deployed_key.pipeline_id.as_ref(),
        deployed_key.core_id,
        deployed_key.deployment_generation
    )
}

fn runtime_recovery_backoff(policy: &RuntimeRecoveryPolicy, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1).min(u32::MAX as usize) as u32;
    let multiplier = 2_u32.checked_pow(exponent).unwrap_or(u32::MAX);
    policy
        .initial_backoff
        .checked_mul(multiplier)
        .unwrap_or(policy.max_backoff)
        .min(policy.max_backoff)
}

fn runtime_recovery_streak_expired(
    ready_since: Option<Instant>,
    reset_after: Duration,
    now: Instant,
) -> bool {
    ready_since.is_some_and(|ready_since| now.saturating_duration_since(ready_since) >= reset_after)
}

impl<
    PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable + FlowMetricHook,
> ControllerRuntime<PData>
{
    /// Launches one regular pipeline instance on a specific core and generation.
    pub(super) fn launch_regular_pipeline_instance(
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
            Arc::downgrade(self),
            thread_id,
            None,
        )?;
        self.register_launched_instance(launched);
        Ok(deployed_key)
    }

    /// Registers a launched instance and reconciles the race where the thread exited first.
    ///
    /// The launch path inserts the instance as Active here, while the runtime thread reports its
    /// terminal exit independently through note_instance_exit(). If that exit arrived first, it
    /// was parked in pending_instance_exits and is applied immediately during registration.
    pub(crate) fn register_launched_instance(
        self: &Arc<Self>,
        launched: LaunchedPipelineThread<PData>,
    ) {
        let (should_compact, pending_exit) = {
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
            let pending_exit = state.pending_instance_exits.remove(&launched.pipeline_key);
            let should_compact = if let Some(exit) = pending_exit.as_ref() {
                Self::apply_instance_exit_locked(&mut state, &launched.pipeline_key, exit)
            } else {
                false
            };
            self.state_changed.notify_all();
            (should_compact, pending_exit)
        };

        if should_compact {
            let logical_pipeline_key = PipelineKey::new(
                launched.pipeline_key.pipeline_group_id.clone(),
                launched.pipeline_key.pipeline_id.clone(),
            );
            self.observed_state_store
                .compact_pipeline_instances(&logical_pipeline_key);
        }
        if let Some(RuntimeInstanceExit::Error(error)) = pending_exit {
            self.schedule_runtime_recovery(launched.pipeline_key, error);
        }
    }

    /// Records a pipeline instance exit and closes the registration-before/after-exit race.
    ///
    /// If the instance is already visible in runtime_instances, the exit is applied immediately.
    /// Otherwise we store it in pending_instance_exits so register_launched_instance() can
    /// reconcile it as soon as registration becomes visible.
    pub(crate) fn note_instance_exit(
        self: &Arc<Self>,
        pipeline_key: DeployedPipelineKey,
        exit: RuntimeInstanceExit,
    ) {
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

        let (should_compact, exit_was_applied) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.runtime_instances.contains_key(&pipeline_key) {
                (
                    Self::apply_instance_exit_locked(&mut state, &pipeline_key, &exit),
                    true,
                )
            } else {
                _ = state
                    .pending_instance_exits
                    .insert(pipeline_key.clone(), exit.clone());
                (false, false)
            }
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
        if exit_was_applied && let RuntimeInstanceExit::Error(error) = exit {
            self.schedule_runtime_recovery(pipeline_key, error);
        }
    }

    /// Starts one supervised recovery worker for a failed serving core.
    fn schedule_runtime_recovery(
        self: &Arc<Self>,
        failed_key: DeployedPipelineKey,
        error: RuntimeInstanceError,
    ) {
        let pipeline_key = PipelineKey::new(
            failed_key.pipeline_group_id.clone(),
            failed_key.pipeline_id.clone(),
        );
        let current_record = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.global_shutdown_requested || state.active_shutdowns.contains_key(&pipeline_key)
            {
                if state.first_error.is_none() {
                    state.first_error = Some(error.message);
                }
                return;
            }
            if state.active_rollouts.contains_key(&pipeline_key)
                || state.active_engine_operation.is_some()
            {
                return;
            }
            state.logical_pipelines.get(&pipeline_key).cloned()
        };

        let Some(current_record) = current_record else {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.first_error.is_none() {
                state.first_error = Some(error.message);
            }
            return;
        };
        let policy = current_record.resolved.policies.runtime_recovery.clone();
        let assigned_cores = match self.assigned_cores_for_resolved(&current_record.resolved) {
            Ok(assigned_cores) => assigned_cores,
            Err(err) => {
                self.fail_runtime_recovery(
                    &pipeline_key,
                    failed_key.core_id,
                    failed_key.deployment_generation,
                    0,
                    policy.max_restarts,
                    format!("failed to resolve recovery core allocation: {err:?}"),
                );
                return;
            }
        };
        if !assigned_cores.contains(&failed_key.core_id) {
            return;
        }

        let (worker_id, restart_count) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.global_shutdown_requested
                || state.active_rollouts.contains_key(&pipeline_key)
                || state.active_shutdowns.contains_key(&pipeline_key)
                || state.active_engine_operation.is_some()
            {
                return;
            }

            let recovery_key = (pipeline_key.clone(), failed_key.core_id);
            let recovery = state
                .runtime_recoveries
                .entry(recovery_key)
                .or_insert_with(|| RuntimeRecoveryState {
                    serving_generation: current_record.active_generation,
                    restart_count: 0,
                    ready_since: None,
                    worker_id: None,
                    candidate_generation: None,
                    cancel_requested: false,
                });
            if recovery.worker_id.is_some() {
                self.state_changed.notify_all();
                return;
            }
            if recovery.serving_generation != failed_key.deployment_generation {
                return;
            }
            if runtime_recovery_streak_expired(
                recovery.ready_since,
                policy.reset_after,
                Instant::now(),
            ) {
                recovery.restart_count = 0;
            }
            recovery.ready_since = None;
            recovery.cancel_requested = false;

            let restart_count = recovery.restart_count;
            let worker_id = state.next_recovery_id;
            state.next_recovery_id += 1;
            state
                .runtime_recoveries
                .get_mut(&(pipeline_key.clone(), failed_key.core_id))
                .expect("runtime recovery was inserted above")
                .worker_id = Some(worker_id);
            (worker_id, restart_count)
        };

        if !policy.enabled {
            self.clear_runtime_recovery_worker(&pipeline_key, failed_key.core_id, worker_id);
            self.fail_runtime_recovery(
                &pipeline_key,
                failed_key.core_id,
                failed_key.deployment_generation,
                restart_count,
                policy.max_restarts,
                format!("runtime recovery is disabled: {}", error.message),
            );
            return;
        }
        if restart_count >= policy.max_restarts {
            self.clear_runtime_recovery_worker(&pipeline_key, failed_key.core_id, worker_id);
            self.fail_runtime_recovery(
                &pipeline_key,
                failed_key.core_id,
                failed_key.deployment_generation,
                restart_count,
                policy.max_restarts,
                error.message,
            );
            return;
        }

        let runtime = Arc::clone(self);
        let worker_pipeline_key = pipeline_key.clone();
        let worker_policy = policy.clone();
        let initial_error = error.message;
        let worker_name = format!(
            "pipeline-recovery-{}-{}-core-{}",
            pipeline_key.pipeline_group_id().as_ref(),
            pipeline_key.pipeline_id().as_ref(),
            failed_key.core_id
        );
        let spawn_result = thread::Builder::new()
            .name(worker_name.clone())
            .spawn(move || {
                let run_result = catch_unwind(AssertUnwindSafe(|| {
                    runtime.run_runtime_recovery(
                        worker_pipeline_key.clone(),
                        failed_key.core_id,
                        failed_key.deployment_generation,
                        worker_id,
                        worker_policy.clone(),
                        initial_error,
                    );
                }));
                if let Err(panic) = run_result {
                    let report = PanicReport::capture(
                        "runtime recovery worker",
                        panic,
                        Some(worker_name),
                        None,
                        Some(failed_key.core_id),
                    );
                    runtime.clear_runtime_recovery_worker(
                        &worker_pipeline_key,
                        failed_key.core_id,
                        worker_id,
                    );
                    runtime.fail_runtime_recovery(
                        &worker_pipeline_key,
                        failed_key.core_id,
                        failed_key.deployment_generation,
                        restart_count,
                        worker_policy.max_restarts,
                        report.detail_message(),
                    );
                }
            });
        if let Err(spawn_error) = spawn_result {
            self.clear_runtime_recovery_worker(&pipeline_key, failed_key.core_id, worker_id);
            self.fail_runtime_recovery(
                &pipeline_key,
                failed_key.core_id,
                failed_key.deployment_generation,
                restart_count,
                policy.max_restarts,
                format!("failed to spawn runtime recovery worker: {spawn_error}"),
            );
        }
    }

    /// Runs bounded recovery attempts for one failed logical core.
    fn run_runtime_recovery(
        self: &Arc<Self>,
        pipeline_key: PipelineKey,
        core_id: usize,
        failed_generation: u64,
        worker_id: u64,
        policy: RuntimeRecoveryPolicy,
        mut last_error: String,
    ) {
        loop {
            let attempt =
                self.prepare_runtime_recovery_attempt(&pipeline_key, core_id, worker_id, &policy);
            let attempt = match attempt {
                RuntimeRecoveryAttemptDecision::Cancelled => return,
                RuntimeRecoveryAttemptDecision::Exhausted => {
                    let restart_count = self
                        .runtime_recovery_restart_count(&pipeline_key, core_id)
                        .unwrap_or(policy.max_restarts);
                    self.clear_runtime_recovery_worker(&pipeline_key, core_id, worker_id);
                    self.fail_runtime_recovery(
                        &pipeline_key,
                        core_id,
                        failed_generation,
                        restart_count,
                        policy.max_restarts,
                        last_error,
                    );
                    return;
                }
                RuntimeRecoveryAttemptDecision::Attempt(attempt) => attempt,
            };

            otel_warn!(
                "otelcol.pipeline.recovery.retry",
                pipeline_group_id = %pipeline_key.pipeline_group_id(),
                pipeline_id = %pipeline_key.pipeline_id(),
                core_id = core_id,
                from_generation = failed_generation,
                target_generation = attempt.target_key.deployment_generation,
                attempt = attempt.attempt,
                max_restarts = policy.max_restarts,
                backoff_ms = attempt.backoff.as_millis() as u64,
                error = last_error.as_str(),
            );

            if !self.wait_for_runtime_recovery_delay(
                &pipeline_key,
                core_id,
                worker_id,
                attempt.backoff,
            ) {
                self.clear_runtime_recovery_worker(&pipeline_key, core_id, worker_id);
                return;
            }

            let target_key = match self.launch_regular_pipeline_instance(
                &attempt.resolved,
                core_id,
                attempt.target_key.deployment_generation,
            ) {
                Ok(target_key) => target_key,
                Err(error) => {
                    last_error = error.to_string();
                    self.clear_runtime_recovery_candidate(&pipeline_key, core_id, worker_id);
                    continue;
                }
            };

            let ready_deadline = Instant::now() + policy.startup_timeout;
            match self.wait_for_runtime_recovery_ready(
                &pipeline_key,
                core_id,
                worker_id,
                &target_key,
                ready_deadline,
            ) {
                Ok(()) => {
                    let promoted = {
                        let mut state = self
                            .state
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let target_active =
                            state
                                .runtime_instances
                                .get(&target_key)
                                .is_some_and(|instance| {
                                    matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                                });
                        let Some(recovery) = state
                            .runtime_recoveries
                            .get_mut(&(pipeline_key.clone(), core_id))
                        else {
                            return;
                        };
                        if recovery.worker_id != Some(worker_id)
                            || recovery.cancel_requested
                            || !target_active
                        {
                            false
                        } else {
                            recovery.serving_generation = target_key.deployment_generation;
                            recovery.ready_since = Some(Instant::now());
                            recovery.worker_id = None;
                            recovery.candidate_generation = None;
                            true
                        }
                    };
                    self.state_changed.notify_all();
                    if !promoted {
                        last_error = self.instance_exit(&target_key).map_or_else(
                            || "replacement stopped before promotion".to_owned(),
                            |exit| match exit {
                                RuntimeInstanceExit::Success => {
                                    "replacement exited before promotion".to_owned()
                                }
                                RuntimeInstanceExit::Error(error) => error.message,
                            },
                        );
                        let cancelled = {
                            let state = self
                                .state
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            state
                                .runtime_recoveries
                                .get(&(pipeline_key.clone(), core_id))
                                .is_none_or(|recovery| {
                                    recovery.worker_id != Some(worker_id)
                                        || recovery.cancel_requested
                                })
                                || state.global_shutdown_requested
                                || state.active_rollouts.contains_key(&pipeline_key)
                                || state.active_shutdowns.contains_key(&pipeline_key)
                                || state.active_engine_operation.is_some()
                        };
                        let _ = self.shutdown_instance(
                            &target_key,
                            policy.startup_timeout.as_secs().max(1),
                            "runtime recovery replacement was not promoted",
                        );
                        if cancelled {
                            self.clear_runtime_recovery_worker(&pipeline_key, core_id, worker_id);
                            return;
                        }
                        self.clear_runtime_recovery_candidate(&pipeline_key, core_id, worker_id);
                        continue;
                    }

                    self.observed_state_store.set_pipeline_serving_generation(
                        pipeline_key.clone(),
                        core_id,
                        target_key.deployment_generation,
                    );
                    self.prune_pipeline_runtime_and_history(&pipeline_key);
                    otel_info!(
                        "otelcol.pipeline.recovery.complete",
                        pipeline_group_id = %pipeline_key.pipeline_group_id(),
                        pipeline_id = %pipeline_key.pipeline_id(),
                        core_id = core_id,
                        from_generation = failed_generation,
                        target_generation = target_key.deployment_generation,
                        attempt = attempt.attempt,
                    );
                    return;
                }
                Err(RecoveryReadyError::Cancelled) => {
                    let _ = self.shutdown_instance(
                        &target_key,
                        policy.startup_timeout.as_secs().max(1),
                        "runtime recovery cancelled",
                    );
                    self.clear_runtime_recovery_worker(&pipeline_key, core_id, worker_id);
                    return;
                }
                Err(RecoveryReadyError::Failed(error)) => {
                    last_error = error;
                    let _ = self.shutdown_instance(
                        &target_key,
                        policy.startup_timeout.as_secs().max(1),
                        "runtime recovery attempt failed",
                    );
                    self.clear_runtime_recovery_candidate(&pipeline_key, core_id, worker_id);
                }
            }
        }
    }

    /// Allocates the next generation and restart ordinal for a recovery worker.
    fn prepare_runtime_recovery_attempt(
        &self,
        pipeline_key: &PipelineKey,
        core_id: usize,
        worker_id: u64,
        policy: &RuntimeRecoveryPolicy,
    ) -> RuntimeRecoveryAttemptDecision {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let recovery_key = (pipeline_key.clone(), core_id);
        let Some(recovery) = state.runtime_recoveries.get(&recovery_key) else {
            return RuntimeRecoveryAttemptDecision::Cancelled;
        };
        if recovery.worker_id != Some(worker_id)
            || recovery.cancel_requested
            || state.global_shutdown_requested
            || state.active_rollouts.contains_key(pipeline_key)
            || state.active_shutdowns.contains_key(pipeline_key)
            || state.active_engine_operation.is_some()
        {
            if let Some(recovery) = state.runtime_recoveries.get_mut(&recovery_key)
                && recovery.worker_id == Some(worker_id)
            {
                recovery.worker_id = None;
                recovery.candidate_generation = None;
            }
            self.state_changed.notify_all();
            return RuntimeRecoveryAttemptDecision::Cancelled;
        }
        if recovery.restart_count >= policy.max_restarts {
            return RuntimeRecoveryAttemptDecision::Exhausted;
        }

        let Some(resolved) = state
            .logical_pipelines
            .get(pipeline_key)
            .map(|record| record.resolved.clone())
        else {
            return RuntimeRecoveryAttemptDecision::Exhausted;
        };
        let attempt = {
            let recovery = state
                .runtime_recoveries
                .get_mut(&recovery_key)
                .expect("runtime recovery exists above");
            recovery.restart_count += 1;
            recovery.restart_count
        };
        let target_generation = {
            let counter = state
                .generation_counters
                .entry(pipeline_key.clone())
                .or_insert(0);
            let generation = *counter;
            *counter += 1;
            generation
        };
        state
            .runtime_recoveries
            .get_mut(&recovery_key)
            .expect("runtime recovery exists above")
            .candidate_generation = Some(target_generation);

        RuntimeRecoveryAttemptDecision::Attempt(Box::new(RuntimeRecoveryAttempt {
            attempt,
            target_key: DeployedPipelineKey {
                pipeline_group_id: pipeline_key.pipeline_group_id().clone(),
                pipeline_id: pipeline_key.pipeline_id().clone(),
                core_id,
                deployment_generation: target_generation,
            },
            resolved,
            backoff: runtime_recovery_backoff(policy, attempt),
        }))
    }

    /// Waits for a recovery backoff while remaining cancellable.
    fn wait_for_runtime_recovery_delay(
        &self,
        pipeline_key: &PipelineKey,
        core_id: usize,
        worker_id: u64,
        delay: Duration,
    ) -> bool {
        let deadline = Instant::now() + delay;
        let recovery_key = (pipeline_key.clone(), core_id);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loop {
            let active = state
                .runtime_recoveries
                .get(&recovery_key)
                .is_some_and(|recovery| {
                    recovery.worker_id == Some(worker_id) && !recovery.cancel_requested
                })
                && !state.global_shutdown_requested
                && !state.active_rollouts.contains_key(pipeline_key)
                && !state.active_shutdowns.contains_key(pipeline_key)
                && state.active_engine_operation.is_none();
            if !active {
                return false;
            }
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return true;
            };
            let (next_state, timeout) = self
                .state_changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if timeout.timed_out() {
                return true;
            }
        }
    }

    /// Waits for a replacement generation to become ready or be cancelled.
    fn wait_for_runtime_recovery_ready(
        &self,
        pipeline_key: &PipelineKey,
        core_id: usize,
        worker_id: u64,
        target_key: &DeployedPipelineKey,
        deadline: Instant,
    ) -> Result<(), RecoveryReadyError> {
        loop {
            let cancelled = {
                let state = self
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                state
                    .runtime_recoveries
                    .get(&(pipeline_key.clone(), core_id))
                    .is_none_or(|recovery| {
                        recovery.worker_id != Some(worker_id) || recovery.cancel_requested
                    })
                    || state.global_shutdown_requested
                    || state.active_rollouts.contains_key(pipeline_key)
                    || state.active_shutdowns.contains_key(pipeline_key)
                    || state.active_engine_operation.is_some()
            };
            if cancelled {
                return Err(RecoveryReadyError::Cancelled);
            }

            if let Some(status) = self.observed_state_handle.pipeline_status(pipeline_key)
                && let Some(instance) =
                    status.instance_status(target_key.core_id, target_key.deployment_generation)
            {
                let accepted = instance.accepted_condition().status == ConditionStatus::True;
                let ready = instance.ready_condition().status == ConditionStatus::True;
                if accepted && ready {
                    return Ok(());
                }
                if matches!(
                    instance.phase(),
                    PipelinePhase::Failed(_)
                        | PipelinePhase::Rejected(_)
                        | PipelinePhase::Deleted
                        | PipelinePhase::Stopped
                ) {
                    return Err(RecoveryReadyError::Failed(format!(
                        "replacement failed to become ready on core {} (generation {})",
                        target_key.core_id, target_key.deployment_generation
                    )));
                }
            }

            if let Some(exit) = self.instance_exit(target_key) {
                return Err(RecoveryReadyError::Failed(match exit {
                    RuntimeInstanceExit::Success => format!(
                        "replacement exited before reporting ready on core {} (generation {})",
                        target_key.core_id, target_key.deployment_generation
                    ),
                    RuntimeInstanceExit::Error(error) => error.message,
                }));
            }
            if Instant::now() >= deadline {
                return Err(RecoveryReadyError::Failed(format!(
                    "timed out waiting for replacement on core {} (generation {})",
                    target_key.core_id, target_key.deployment_generation
                )));
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Cancels and joins active recovery workers for one logical pipeline.
    pub(super) fn cancel_runtime_recoveries_for_pipeline(&self, pipeline_key: &PipelineKey) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for ((key, _), recovery) in &mut state.runtime_recoveries {
            if key == pipeline_key && recovery.worker_id.is_some() {
                recovery.cancel_requested = true;
            }
        }
        self.state_changed.notify_all();
        while state
            .runtime_recoveries
            .iter()
            .any(|((key, _), recovery)| key == pipeline_key && recovery.worker_id.is_some())
        {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Cancels and joins every active runtime recovery worker.
    fn cancel_all_runtime_recoveries(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for recovery in state.runtime_recoveries.values_mut() {
            if recovery.worker_id.is_some() {
                recovery.cancel_requested = true;
            }
        }
        self.state_changed.notify_all();
        while state
            .runtime_recoveries
            .values()
            .any(|recovery| recovery.worker_id.is_some())
        {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Clears one worker's candidate generation while preserving its streak.
    fn clear_runtime_recovery_candidate(
        &self,
        pipeline_key: &PipelineKey,
        core_id: usize,
        worker_id: u64,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(recovery) = state
            .runtime_recoveries
            .get_mut(&(pipeline_key.clone(), core_id))
            && recovery.worker_id == Some(worker_id)
        {
            recovery.candidate_generation = None;
        }
        self.state_changed.notify_all();
    }

    /// Releases ownership of one per-core recovery worker.
    fn clear_runtime_recovery_worker(
        &self,
        pipeline_key: &PipelineKey,
        core_id: usize,
        worker_id: u64,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(recovery) = state
            .runtime_recoveries
            .get_mut(&(pipeline_key.clone(), core_id))
            && recovery.worker_id == Some(worker_id)
        {
            recovery.worker_id = None;
            recovery.candidate_generation = None;
        }
        self.state_changed.notify_all();
    }

    /// Returns the current replacement count for one logical core.
    fn runtime_recovery_restart_count(
        &self,
        pipeline_key: &PipelineKey,
        core_id: usize,
    ) -> Option<usize> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .runtime_recoveries
            .get(&(pipeline_key.clone(), core_id))
            .map(|recovery| recovery.restart_count)
    }

    /// Converts an unrecoverable core failure into coordinated engine failure.
    fn fail_runtime_recovery(
        self: &Arc<Self>,
        pipeline_key: &PipelineKey,
        core_id: usize,
        failed_generation: u64,
        restart_count: usize,
        max_restarts: usize,
        error: String,
    ) {
        let fatal_message = format!(
            "runtime recovery failed for {}:{} core={} generation={} after {} restart attempt(s): {}",
            pipeline_key.pipeline_group_id(),
            pipeline_key.pipeline_id(),
            core_id,
            failed_generation,
            restart_count,
            error
        );
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.first_error.is_none() {
                state.first_error = Some(fatal_message.clone());
            }
        }
        otel_error!(
            "otelcol.pipeline.recovery.abort",
            pipeline_group_id = %pipeline_key.pipeline_group_id(),
            pipeline_id = %pipeline_key.pipeline_id(),
            core_id = core_id,
            failed_generation = failed_generation,
            restart_count = restart_count,
            max_restarts = max_restarts,
            error = error.as_str(),
            message = fatal_message.as_str(),
        );
        if let Err(shutdown_error) = self.request_shutdown_all(10) {
            otel_error!(
                "otelcol.pipeline.recovery.abort",
                pipeline_group_id = %pipeline_key.pipeline_group_id(),
                pipeline_id = %pipeline_key.pipeline_id(),
                core_id = core_id,
                failed_generation = failed_generation,
                restart_count = restart_count,
                max_restarts = max_restarts,
                error = ?shutdown_error,
                message = "Runtime recovery was fatal and coordinated shutdown dispatch failed.",
            );
        }
        self.controller_thread.unpark();
    }

    /// Waits for a specific deployed instance to report admitted plus ready.
    pub(super) fn wait_for_pipeline_ready(
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
    pub(super) fn instance_exit(
        &self,
        deployed_key: &DeployedPipelineKey,
    ) -> Option<RuntimeInstanceExit> {
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
    pub(super) fn request_instance_shutdown(
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
    pub(super) fn wait_for_instance_exit(
        &self,
        deployed_key: &DeployedPipelineKey,
        deadline: Instant,
    ) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loop {
            if let Some(instance) = state.runtime_instances.get(deployed_key) {
                match &instance.lifecycle {
                    RuntimeInstanceLifecycle::Active => {}
                    RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Success) => {
                        return Ok(());
                    }
                    RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Error(error)) => {
                        return Err(error.message.clone());
                    }
                }
            }

            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return Err(format!(
                    "timed out waiting for pipeline {}:{} core={} generation={} to drain",
                    deployed_key.pipeline_group_id.as_ref(),
                    deployed_key.pipeline_id.as_ref(),
                    deployed_key.core_id,
                    deployed_key.deployment_generation
                ));
            };

            // Runtime registration and exit reporting both publish through this
            // mutex/condvar pair, so exit waits can sleep until real controller
            // state changes instead of polling every 50ms.
            let (next_state, _) = self
                .state_changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
        }
    }

    /// Waits for a global-shutdown producer to exit, including instances whose
    /// terminal record was immediately compacted because no tracked per-pipeline
    /// operation retained it.
    fn wait_for_global_shutdown_exit(
        &self,
        deployed_key: &DeployedPipelineKey,
        deadline: Instant,
    ) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loop {
            match state.runtime_instances.get(deployed_key) {
                None
                | Some(RuntimeInstanceRecord {
                    lifecycle: RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Success),
                    ..
                }) => return Ok(()),
                Some(RuntimeInstanceRecord {
                    lifecycle: RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Error(error)),
                    ..
                }) => return Err(error.message.clone()),
                Some(RuntimeInstanceRecord {
                    lifecycle: RuntimeInstanceLifecycle::Active,
                    ..
                }) => {}
            }

            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return Err(format!(
                    "timed out waiting for pipeline {} to drain before system observability shutdown",
                    deployed_instance_label(deployed_key)
                ));
            };
            let (next_state, _) = self
                .state_changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
        }
    }

    /// Requests shutdown for one instance and waits until it exits.
    pub(super) fn shutdown_instance(
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

    /// Drops the retained admin sender after shutdown has been accepted.
    ///
    /// The retained sender is the controller's "not yet signaled" marker for
    /// an active instance. Releasing it makes shutdown dispatch idempotent for
    /// that instance and lets the pipeline control loop observe channel closure
    /// once node tasks have exited.
    pub(super) fn release_instance_control_sender(&self, deployed_key: &DeployedPipelineKey) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(instance) = state.runtime_instances.get_mut(deployed_key) {
            instance.control_sender = None;
        }
    }

    /// Requests shutdown for every currently active runtime instance.
    ///
    /// This is best-effort across the snapshot: one failed send must not prevent
    /// later instances from receiving shutdown. It is also idempotent at the
    /// dispatch boundary: instances that already accepted shutdown have released
    /// their retained control sender and are skipped by later calls. When the
    /// system observability pipeline is active, producer pipelines are signaled
    /// and awaited first so their final telemetry reaches the internal receiver
    /// before that receiver is shut down.
    pub(super) fn request_shutdown_all(
        self: &Arc<Self>,
        timeout_secs: u64,
    ) -> Result<(), ControlPlaneError> {
        self.cancel_all_runtime_recoveries();
        let shutdown_timeout = Duration::from_secs(timeout_secs.max(1));
        let deadline = Instant::now() + shutdown_timeout;

        // Snapshot under the state lock, then send outside the lock so runtime
        // callbacks can report exits while shutdown dispatch is in progress.
        // Producer keys include active instances whose sender was already
        // released by an earlier request because they still need to exit before
        // observability can be stopped.
        let (
            mut producer_keys,
            mut producer_senders,
            mut observability_senders,
            coordinator_reserved,
        ) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let coordinator_active = state.global_shutdown_coordinators > 0;
            let mut producer_keys = Vec::new();
            let mut producer_senders = Vec::new();
            let mut observability_senders = Vec::new();

            for (deployed_key, instance) in &mut state.runtime_instances {
                if !matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active) {
                    continue;
                }

                let is_observability = deployed_key.pipeline_group_id.as_ref()
                    == SYSTEM_PIPELINE_GROUP_ID
                    && deployed_key.pipeline_id.as_ref() == SYSTEM_OBSERVABILITY_PIPELINE_ID;
                if is_observability {
                    if !coordinator_active && let Some(sender) = instance.control_sender.take() {
                        // Taking the sender is the idempotence marker for the
                        // asynchronous observability-shutdown coordinator.
                        observability_senders.push((deployed_key.clone(), sender));
                    }
                } else {
                    producer_keys.push(deployed_key.clone());
                    if let Some(sender) = &instance.control_sender {
                        producer_senders.push((deployed_key.clone(), sender.clone()));
                    }
                }
            }

            let coordinator_reserved = !coordinator_active
                && (!producer_keys.is_empty() || !observability_senders.is_empty());
            state.global_shutdown_requested = true;
            if coordinator_reserved {
                state.global_shutdown_coordinators += 1;
            }
            self.state_changed.notify_all();

            (
                producer_keys,
                producer_senders,
                observability_senders,
                coordinator_reserved,
            )
        };

        let sort_key = |deployed_key: &DeployedPipelineKey| {
            (
                deployed_key.pipeline_group_id.as_ref().to_owned(),
                deployed_key.pipeline_id.as_ref().to_owned(),
                deployed_key.core_id,
                deployed_key.deployment_generation,
            )
        };
        producer_keys.sort_by_key(&sort_key);
        producer_senders.sort_by_key(|(deployed_key, _)| sort_key(deployed_key));
        observability_senders.sort_by_key(|(deployed_key, _)| sort_key(deployed_key));

        let mut failures: HashMap<DeployedPipelineKey, String> = HashMap::new();
        let dispatch =
            |senders: Vec<(DeployedPipelineKey, Arc<dyn PipelineAdminSender>)>,
             failures: &mut HashMap<DeployedPipelineKey, String>| {
                for (deployed_key, sender) in senders {
                    if let Err(err) =
                        sender.try_send_shutdown(deadline, "global shutdown".to_owned())
                    {
                        // A failed send can race with the runtime thread exiting after
                        // the snapshot was taken. Treat clean exit as success, report a
                        // terminal runtime error if one was recorded, and otherwise keep
                        // the retained sender so a later shutdown-all can retry it.
                        match self.instance_exit(&deployed_key) {
                            Some(RuntimeInstanceExit::Success) => {
                                self.release_instance_control_sender(&deployed_key);
                            }
                            Some(RuntimeInstanceExit::Error(error)) => {
                                _ = failures.insert(deployed_key, error.message);
                            }
                            None => {
                                _ = failures.insert(deployed_key, err.to_string());
                            }
                        }
                    } else {
                        // After a successful send, the controller should not send
                        // another shutdown message to this same active instance.
                        self.release_instance_control_sender(&deployed_key);
                    }
                }
            };

        dispatch(producer_senders, &mut failures);

        if coordinator_reserved {
            let restore_senders = observability_senders.clone();
            let runtime = Arc::clone(self);
            if let Err(error) = thread::Builder::new()
                .name("global-observability-shutdown".to_owned())
                .spawn(move || {
                    let completion = catch_unwind(AssertUnwindSafe(|| {
                        runtime.complete_global_observability_shutdown(
                            producer_keys,
                            observability_senders,
                            deadline,
                            shutdown_timeout,
                        );
                    }));
                    runtime.finish_global_shutdown_coordinator();
                    if let Err(payload) = completion {
                        std::panic::resume_unwind(payload);
                    }
                })
            {
                self.restore_observability_senders(&restore_senders);
                self.finish_global_shutdown_coordinator();
                return Err(ControlPlaneError::Internal {
                    message: format!(
                        "failed to start global observability shutdown coordinator: {error}"
                    ),
                });
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            // Report all failures together after every eligible instance has
            // been attempted, preserving best-effort shutdown semantics.
            let mut failures: Vec<_> = failures.into_iter().collect();
            failures.sort_by_key(|(deployed_key, _)| sort_key(deployed_key));
            Err(ControlPlaneError::Internal {
                message: format!(
                    "global shutdown failed for {} runtime instance(s): {}",
                    failures.len(),
                    failures
                        .into_iter()
                        .map(|(deployed_key, error)| format!(
                            "{}: {error}",
                            deployed_instance_label(&deployed_key)
                        ))
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            })
        }
    }

    /// Completes the asynchronous second phase of global shutdown.
    fn complete_global_observability_shutdown(
        &self,
        producer_keys: Vec<DeployedPipelineKey>,
        observability_senders: Vec<(DeployedPipelineKey, Arc<dyn PipelineAdminSender>)>,
        producer_deadline: Instant,
        shutdown_timeout: Duration,
    ) {
        let mut wait_failures = Vec::new();
        for deployed_key in &producer_keys {
            if let Err(error) = self.wait_for_global_shutdown_exit(deployed_key, producer_deadline)
            {
                wait_failures.push(error);
            }
        }
        if !wait_failures.is_empty() {
            self.record_async_global_shutdown_failure(format!(
                "producer drain failed before system observability shutdown: {}",
                wait_failures.join("; ")
            ));
        }

        let active_producers = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            producer_keys
                .iter()
                .filter(|deployed_key| {
                    matches!(
                        state.runtime_instances.get(*deployed_key),
                        Some(RuntimeInstanceRecord {
                            lifecycle: RuntimeInstanceLifecycle::Active,
                            ..
                        })
                    )
                })
                .map(deployed_instance_label)
                .collect::<Vec<_>>()
        };
        if !active_producers.is_empty() {
            self.restore_observability_senders(&observability_senders);
            self.record_async_global_shutdown_failure(format!(
                "system observability remains active because producer shutdown timed out: {}",
                active_producers.join("; ")
            ));
            return;
        }

        // Observability is a distinct shutdown phase. Give it the same budget
        // selected by the caller instead of collapsing that phase to one second
        // after producers have consumed their own drain budget.
        let observability_deadline = Instant::now() + shutdown_timeout;
        let mut observability_keys = Vec::new();
        for (deployed_key, sender) in observability_senders {
            let final_error = loop {
                match sender.try_send_shutdown(observability_deadline, "global shutdown".to_owned())
                {
                    Ok(()) => break None,
                    Err(error) => match self.instance_exit(&deployed_key) {
                        Some(RuntimeInstanceExit::Success) => break None,
                        Some(RuntimeInstanceExit::Error(exit)) => break Some(exit.message),
                        None if Instant::now() < observability_deadline => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        None => break Some(error.to_string()),
                    },
                }
            };

            if let Some(error) = final_error {
                let restored = {
                    let mut state = self
                        .state
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if let Some(instance) = state.runtime_instances.get_mut(&deployed_key)
                        && matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                        && instance.control_sender.is_none()
                    {
                        instance.control_sender = Some(sender);
                        true
                    } else {
                        false
                    }
                };
                if restored {
                    self.record_async_global_shutdown_failure(format!(
                        "failed to send global shutdown to {} after retrying: {error}",
                        deployed_instance_label(&deployed_key)
                    ));
                }
            } else {
                observability_keys.push(deployed_key);
            }
        }

        for deployed_key in observability_keys {
            if let Err(error) =
                self.wait_for_global_shutdown_exit(&deployed_key, observability_deadline)
            {
                self.record_async_global_shutdown_failure(format!(
                    "system observability shutdown did not complete: {error}"
                ));
            }
        }
    }

    /// Marks one finite phased-shutdown coordinator complete and wakes teardown.
    fn finish_global_shutdown_coordinator(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.global_shutdown_coordinators = state.global_shutdown_coordinators.saturating_sub(1);
        self.state_changed.notify_all();
    }

    /// Waits for all phased global-shutdown coordinators to exhaust their finite budgets.
    ///
    /// Returns `true` when an engine-wide shutdown request was observed. A `false`
    /// result lets controller teardown retain its fallback for unrelated wakeups.
    pub(crate) fn wait_for_global_shutdown_completion(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.global_shutdown_coordinators > 0 {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        state.global_shutdown_requested
    }

    /// Returns whether every registered runtime instance has exited.
    pub(crate) fn all_instances_exited(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_instances
            == 0
    }

    /// Restores system observability senders if the coordinator could not start.
    fn restore_observability_senders(
        &self,
        senders: &[(DeployedPipelineKey, Arc<dyn PipelineAdminSender>)],
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for (deployed_key, sender) in senders {
            if let Some(instance) = state.runtime_instances.get_mut(deployed_key)
                && matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                && instance.control_sender.is_none()
            {
                instance.control_sender = Some(sender.clone());
            }
        }
    }

    /// Records a delayed global-shutdown failure for final controller teardown.
    fn record_async_global_shutdown_failure(&self, message: String) {
        otel_warn!(
            "controller.global_shutdown.async_phase_failed",
            error = message.as_str()
        );
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.first_error.is_none() {
            state.first_error = Some(message);
        }
    }

    /// Starts a tracked shutdown operation for one logical pipeline.
    pub(super) fn request_shutdown_pipeline(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        self.request_shutdown_pipeline_for_engine_operation(
            pipeline_group_id,
            pipeline_id,
            timeout_secs,
            None,
        )
    }

    pub(super) fn request_shutdown_pipeline_for_engine_operation(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
        engine_operation_id: Option<&str>,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        self.cancel_runtime_recoveries_for_pipeline(&PipelineKey::new(
            pipeline_group_id.to_owned().into(),
            pipeline_id.to_owned().into(),
        ));
        let plan = self.prepare_shutdown_plan_for_engine_operation(
            pipeline_group_id,
            pipeline_id,
            timeout_secs,
            engine_operation_id,
        )?;
        self.spawn_shutdown_for_engine_operation(plan, engine_operation_id)
    }

    /// Blocks until all active runtime instances have exited.
    pub(crate) fn wait_until_all_instances_exit(&self) {
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

    /// Blocks until all non-observability runtime instances have exited.
    ///
    /// The system observability pipeline is deliberately excluded because it
    /// must remain alive to consume terminal telemetry from those producers.
    pub(crate) fn wait_until_all_producer_instances_exit(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.runtime_instances.iter().any(|(key, instance)| {
            matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                && !(key.pipeline_group_id.as_ref() == SYSTEM_PIPELINE_GROUP_ID
                    && key.pipeline_id.as_ref() == SYSTEM_OBSERVABILITY_PIPELINE_ID)
        }) {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Blocks until every runtime instance exits or the timeout elapses.
    pub(crate) fn wait_until_all_instances_exit_for(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.active_instances > 0 {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            let (next_state, wait_result) = self
                .state_changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if wait_result.timed_out() && state.active_instances > 0 {
                return false;
            }
        }
        true
    }

    /// Returns the first runtime error observed by any watched pipeline thread.
    pub(crate) fn take_runtime_error(&self) -> Option<Error> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn recovery_policy() -> RuntimeRecoveryPolicy {
        RuntimeRecoveryPolicy {
            enabled: true,
            max_restarts: 10,
            initial_backoff: Duration::from_millis(250),
            max_backoff: Duration::from_secs(30),
            startup_timeout: Duration::from_secs(30),
            reset_after: Duration::from_secs(60),
        }
    }

    /// Scenario: repeated failures consume increasingly delayed recovery attempts.
    /// Guarantees: exponential backoff starts at 250 ms and saturates at 30 seconds.
    #[test]
    fn runtime_recovery_backoff_grows_and_caps() {
        let policy = recovery_policy();

        assert_eq!(
            runtime_recovery_backoff(&policy, 1),
            Duration::from_millis(250)
        );
        assert_eq!(
            runtime_recovery_backoff(&policy, 2),
            Duration::from_millis(500)
        );
        assert_eq!(
            runtime_recovery_backoff(&policy, 8),
            Duration::from_secs(30)
        );
        assert_eq!(
            runtime_recovery_backoff(&policy, usize::MAX),
            Duration::from_secs(30)
        );
    }

    /// Scenario: a recovered runtime fails before and after its reset window.
    /// Guarantees: only a failure at least 60 seconds after readiness resets the streak.
    #[test]
    fn runtime_recovery_streak_resets_after_ready_window() {
        let now = Instant::now();

        assert!(!runtime_recovery_streak_expired(
            Some(now - Duration::from_secs(59)),
            Duration::from_secs(60),
            now,
        ));
        assert!(runtime_recovery_streak_expired(
            Some(now - Duration::from_secs(60)),
            Duration::from_secs(60),
            now,
        ));
        assert!(!runtime_recovery_streak_expired(
            None,
            Duration::from_secs(60),
            now,
        ));
    }
}
