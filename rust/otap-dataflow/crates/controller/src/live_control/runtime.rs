// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime-instance launch, shutdown, and exit reporting.
//!
//! This module owns the boundary between controller state and actual pipeline
//! threads. It registers launched instances, reconciles early exits, sends
//! shutdown control messages, waits for readiness/exit transitions, and exposes
//! global runtime shutdown/error helpers used by controller teardown. Automatic
//! recovery is scoped to one logical core: it fences one worker per core, gives
//! every replacement a pipeline-wide unique generation, and yields to explicit
//! rollout or shutdown operations.

use super::*;

enum RecoveryReadyError {
    Cancelled,
    Failed(String),
}

struct RuntimeRecoveryAttempt {
    attempt: usize,
    target_key: DeployedPipelineKey,
    resolved: ResolvedPipelineConfig,
    inherited_extensions: InheritedExtensionRegistrations,
    context_bindings: Arc<CompiledContextBindings>,
    placement: LivePipelinePlacement,
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

fn is_observability_instance(deployed_key: &DeployedPipelineKey) -> bool {
    deployed_key.pipeline_group_id.as_ref() == SYSTEM_PIPELINE_GROUP_ID
        && deployed_key.pipeline_id.as_ref() == SYSTEM_OBSERVABILITY_PIPELINE_ID
}

/// Computes capped exponential backoff for a one-based restart attempt.
///
/// Saturating arithmetic keeps even a corrupted or extreme attempt count from
/// wrapping into a short retry delay.
fn runtime_recovery_backoff(policy: &RuntimeRecoveryPolicy, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1).min(u32::MAX as usize) as u32;
    let multiplier = 2_u32.checked_pow(exponent).unwrap_or(u32::MAX);
    policy
        .initial_backoff
        .checked_mul(multiplier)
        .unwrap_or(policy.max_backoff)
        .min(policy.max_backoff)
}

/// Returns whether a ready replacement ran long enough to earn a fresh budget.
///
/// The reset is evaluated lazily on the next failure, avoiding a timer whose
/// only purpose would be to mutate an otherwise idle recovery record.
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
    /// Final drain window, independent of producer and extension-scope shutdown.
    pub(crate) const OBSERVABILITY_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

    /// Full observability drain and runtime-completion budget.
    pub(crate) const OBSERVABILITY_SHUTDOWN_COMPLETION_TIMEOUT: Duration =
        Self::OBSERVABILITY_SHUTDOWN_TIMEOUT.saturating_add(PIPELINE_SHUTDOWN_COMPLETION_GRACE);

    /// Launches one regular pipeline instance on a specific core and generation.
    pub(super) fn launch_regular_pipeline_instance(
        self: &Arc<Self>,
        resolved_pipeline: &ResolvedPipelineConfig,
        inherited_extensions: &InheritedExtensionRegistrations,
        context_bindings: Arc<CompiledContextBindings>,
        placement: &LivePipelinePlacement,
        core_id: usize,
        deployment_generation: u64,
    ) -> Result<DeployedPipelineKey, Error> {
        let thread_id = self.next_thread_id();
        let core_placement =
            placement
                .core(core_id)
                .ok_or_else(|| Error::PipelineRuntimeError {
                    source: Box::new(io::Error::other(format!(
                        "core {core_id} is not present in resolved placement for {}:{}",
                        resolved_pipeline.pipeline_group_id.as_ref(),
                        resolved_pipeline.pipeline_id.as_ref()
                    ))),
                })?;
        let num_cores = placement.placement.core_count();
        let live_config = self.engine_config_snapshot();
        let deployed_key = DeployedPipelineKey {
            pipeline_group_id: resolved_pipeline.pipeline_group_id.clone(),
            pipeline_id: resolved_pipeline.pipeline_id.clone(),
            core_id,
            deployment_generation,
        };
        Controller::<PData>::launch_pipeline_thread(
            self.pipeline_factory,
            deployed_key.clone(),
            CoreId { id: core_id },
            core_placement.numa_node_id,
            Arc::clone(&placement.listener_group_snapshot),
            context_bindings,
            num_cores,
            resolved_pipeline.pipeline.clone(),
            resolved_pipeline.policies.channel_capacity.clone(),
            resolved_pipeline.policies.telemetry.clone(),
            resolved_pipeline.policies.rate_limiters.clone(),
            resolved_pipeline.policies.rate_limiter_scope.clone(),
            inherited_extensions.clone(),
            self.controller_context.clone(),
            self.metrics_reporter.clone(),
            self.engine_event_reporter.clone(),
            self.engine_tracing_setup.clone(),
            self.telemetry_reporting_interval,
            self.memory_pressure_tx.clone(),
            &live_config,
            &self.declared_topics,
            self,
            thread_id,
            None,
        )?;
        Ok(deployed_key)
    }

    /// Reserves one deployed key before its pipeline OS thread is spawned.
    pub(crate) fn reserve_instance_launch(
        &self,
        pipeline_key: &DeployedPipelineKey,
        context_bindings: Arc<CompiledContextBindings>,
    ) -> Result<(), Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.launches_closed || state.global_shutdown_requested {
            let message = state.first_error.as_ref().map_or_else(
                || {
                    format!(
                        "pipeline launch admission is closed for {}",
                        deployed_instance_label(pipeline_key)
                    )
                },
                |error| {
                    format!(
                        "pipeline launch admission is closed for {} after fatal runtime failure: {error}",
                        deployed_instance_label(pipeline_key)
                    )
                },
            );
            return Err(Error::PipelineRuntimeError {
                source: Box::new(io::Error::other(message)),
            });
        }
        let already_live = state.launching_instances.contains_key(pipeline_key)
            || state
                .runtime_instances
                .get(pipeline_key)
                .is_some_and(|instance| {
                    matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                });
        if already_live {
            return Err(Error::PipelineRuntimeError {
                source: Box::new(io::Error::other(format!(
                    "pipeline instance is already launching or active: {}",
                    deployed_instance_label(pipeline_key)
                ))),
            });
        }
        let _ = state
            .launching_instances
            .insert(pipeline_key.clone(), context_bindings);
        state.active_instances += 1;
        self.state_changed.notify_all();
        Ok(())
    }

    /// Rolls back a launch reservation when OS-thread creation fails.
    pub(crate) fn abort_instance_launch(&self, pipeline_key: &DeployedPipelineKey) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.launching_instances.remove(pipeline_key).is_some() {
            state.active_instances = state.active_instances.saturating_sub(1);
            self.state_changed.notify_all();
        }
    }

    /// Publishes the control sender for a successfully spawned pipeline thread.
    ///
    /// Returns a deadline when shutdown raced with the spawn and must be sent
    /// to the newly active instance.
    pub(crate) fn activate_instance_launch(
        &self,
        pipeline_key: DeployedPipelineKey,
        control_sender: Arc<dyn PipelineAdminSender>,
    ) -> Option<Instant> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let context_bindings = state.launching_instances.remove(&pipeline_key)?;
        let is_observability = is_observability_instance(&pipeline_key);
        let _ = state.runtime_instances.insert(
            pipeline_key,
            RuntimeInstanceRecord {
                control_sender: Some(control_sender),
                context_bindings,
                lifecycle: RuntimeInstanceLifecycle::Active,
            },
        );
        let shutdown_deadline = if state.launches_closed || state.global_shutdown_requested {
            if is_observability {
                if state.extension_scope_hosts_stopped
                    && !Self::has_live_producer_instances_locked(&state)
                {
                    Some(
                        *state
                            .observability_shutdown_deadline
                            .get_or_insert_with(|| {
                                Instant::now() + Self::OBSERVABILITY_SHUTDOWN_TIMEOUT
                            }),
                    )
                } else {
                    None
                }
            } else {
                Some(state.global_shutdown_deadline.unwrap_or_else(Instant::now))
            }
        } else {
            None
        };
        self.state_changed.notify_all();
        shutdown_deadline
    }

    /// Publishes a spawned instance and immediately stops it when shutdown won the race.
    pub(crate) fn complete_instance_launch(
        &self,
        pipeline_key: DeployedPipelineKey,
        control_sender: Arc<dyn PipelineAdminSender>,
    ) {
        if let Some(deadline) =
            self.activate_instance_launch(pipeline_key.clone(), control_sender.clone())
        {
            if let Err(error) =
                control_sender.try_send_shutdown(deadline, "global shutdown".to_owned())
            {
                match self.instance_exit(&pipeline_key) {
                    Some(_) => self.release_instance_control_sender(&pipeline_key),
                    None => {
                        self.record_fatal_runtime_error(format!(
                            "failed to stop pipeline launched during global shutdown ({}): {error}",
                            deployed_instance_label(&pipeline_key)
                        ));
                    }
                }
            } else {
                self.release_instance_control_sender(&pipeline_key);
            }
        }
    }

    /// Registers a synthetic launched instance used by controller state tests.
    #[cfg(test)]
    pub(crate) fn register_launched_instance(
        self: &Arc<Self>,
        launched: LaunchedPipelineThread<PData>,
    ) {
        let pending_exit = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.pending_instance_exits.remove(&launched.pipeline_key)
        };
        if let Some(exit) = pending_exit {
            let should_compact = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let _ = state.runtime_instances.insert(
                    launched.pipeline_key.clone(),
                    RuntimeInstanceRecord {
                        control_sender: None,
                        context_bindings: Arc::clone(&launched.context_bindings),
                        lifecycle: RuntimeInstanceLifecycle::Exited(exit.clone()),
                    },
                );
                let logical_pipeline_key = PipelineKey::new(
                    launched.pipeline_key.pipeline_group_id.clone(),
                    launched.pipeline_key.pipeline_id.clone(),
                );
                Self::prune_exited_runtime_instances_for_pipeline_locked(
                    &mut state,
                    &logical_pipeline_key,
                )
            };
            if should_compact {
                let logical_pipeline_key = PipelineKey::new(
                    launched.pipeline_key.pipeline_group_id.clone(),
                    launched.pipeline_key.pipeline_id.clone(),
                );
                self.observed_state_store
                    .compact_pipeline_instances(&logical_pipeline_key);
            }
            self.state_changed.notify_all();
            if let RuntimeInstanceExit::Error(error) = exit {
                self.schedule_runtime_recovery(
                    launched.pipeline_key,
                    launched.context_bindings,
                    error,
                );
            }
            return;
        }
        self.reserve_instance_launch(&launched.pipeline_key, launched.context_bindings)
            .expect("synthetic runtime instance should reserve");
        let _ = self.activate_instance_launch(launched.pipeline_key, launched.control_sender);
    }

    /// Closes pipeline launch admission for the remainder of this controller run.
    #[cfg(test)]
    pub(crate) fn close_pipeline_launches(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.launches_closed = true;
        self.state_changed.notify_all();
    }

    /// Opens the final shutdown phase for the system observability pipeline.
    pub(crate) fn mark_extension_scope_hosts_stopped(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.extension_scope_hosts_stopped = true;
        self.state_changed.notify_all();
    }

    /// Completes teardown on the scope-host thread after its LocalSet is gone.
    ///
    /// The controller's join remains bounded. If it times out, this thread
    /// retains telemetry support while waiting for actual descendant and
    /// observability exits, not merely the fatal-shutdown release latch.
    pub(crate) fn finish_shutdown_after_extension_scopes(
        self: &Arc<Self>,
        shutdown_requested: &CancellationToken,
    ) {
        let deadline = self.global_shutdown_deadline_or_insert(Duration::from_secs(30));
        if let Err(error) = self.request_shutdown_all_until(deadline) {
            self.record_async_global_shutdown_failure(format!(
                "failed to drain pipelines after extension scope shutdown: {error:?}"
            ));
        }

        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            while Self::has_live_producer_instances_locked(&state)
                || state
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

        // An early return or panic can reach cleanup before controller-owned
        // metric producers stop. The guard cancels only after those producers
        // are gone; keep observability open until that handoff occurs.
        while !shutdown_requested.is_cancelled() {
            thread::sleep(Duration::from_millis(10));
        }
        self.mark_extension_scope_hosts_stopped();
        // A producer-only coordinator may have owned dispatch when the hosts
        // stopped. Wait for it before opening observability with its own budget.
        _ = self.wait_for_global_shutdown_completion();
        if let Err(error) = self.request_shutdown_all_until(deadline) {
            self.record_async_global_shutdown_failure(format!(
                "failed to stop system observability after extension scopes: {error:?}"
            ));
        }

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.active_instances > 0 || state.global_shutdown_coordinators > 0 {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Records a pipeline instance exit and closes the registration-before/after-exit race.
    ///
    /// A reserved or active instance is completed immediately. Synthetic test
    /// exits without a reservation remain pending until test registration.
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

        let (should_compact, exit_was_applied, context_bindings) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(context_bindings) = state.launching_instances.remove(&pipeline_key) {
                let _ = state.runtime_instances.insert(
                    pipeline_key.clone(),
                    RuntimeInstanceRecord {
                        control_sender: None,
                        context_bindings: Arc::clone(&context_bindings),
                        lifecycle: RuntimeInstanceLifecycle::Exited(exit.clone()),
                    },
                );
                state.active_instances = state.active_instances.saturating_sub(1);
                let logical_pipeline_key = PipelineKey::new(
                    pipeline_key.pipeline_group_id.clone(),
                    pipeline_key.pipeline_id.clone(),
                );
                (
                    Self::prune_exited_runtime_instances_for_pipeline_locked(
                        &mut state,
                        &logical_pipeline_key,
                    ),
                    true,
                    Some(context_bindings),
                )
            } else if let Some(instance) = state.runtime_instances.get(&pipeline_key) {
                let exit_was_applied =
                    matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active);
                let context_bindings = Arc::clone(&instance.context_bindings);
                (
                    Self::apply_instance_exit_locked(&mut state, &pipeline_key, &exit),
                    exit_was_applied,
                    Some(context_bindings),
                )
            } else {
                _ = state
                    .pending_instance_exits
                    .insert(pipeline_key.clone(), exit.clone());
                (false, false, None)
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
            self.schedule_runtime_recovery(
                pipeline_key,
                context_bindings.expect("exit context"),
                error,
            );
        }
    }

    /// Retains a runtime failure until an explicit lifecycle owner releases it.
    fn defer_runtime_recovery_locked(
        state: &mut ControllerRuntimeState,
        failed_key: DeployedPipelineKey,
        context_bindings: Arc<CompiledContextBindings>,
        error: RuntimeInstanceError,
    ) {
        // Deployed keys are unique and operation overlap is bounded by assigned
        // cores plus rollout candidates, so this queue cannot grow independently
        // of controller-owned runtime state.
        let _ = state
            .deferred_runtime_recoveries
            .insert(failed_key, (context_bindings, error));
    }

    /// Restarts failures deferred for one pipeline after ownership handoff.
    pub(super) fn resume_deferred_runtime_recoveries_for_pipeline(
        self: &Arc<Self>,
        pipeline_key: &PipelineKey,
    ) {
        let mut deferred = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.recovery_preempted(pipeline_key) {
                return;
            }

            let keys = state
                .deferred_runtime_recoveries
                .keys()
                .filter(|deployed_key| {
                    deployed_key.pipeline_group_id == *pipeline_key.pipeline_group_id()
                        && deployed_key.pipeline_id == *pipeline_key.pipeline_id()
                })
                .cloned()
                .collect::<Vec<_>>();
            keys.into_iter()
                .filter_map(|deployed_key| {
                    state
                        .deferred_runtime_recoveries
                        .remove(&deployed_key)
                        .map(|(context_bindings, error)| (deployed_key, context_bindings, error))
                })
                .collect::<Vec<_>>()
        };
        deferred.sort_by_key(|(deployed_key, _, _)| {
            (deployed_key.core_id, deployed_key.deployment_generation)
        });
        for (deployed_key, context_bindings, error) in deferred {
            // schedule_runtime_recovery revalidates the committed serving
            // generation, so failures for candidates retired by the operation
            // are ignored while failures for its winner are restarted.
            self.schedule_runtime_recovery(deployed_key, context_bindings, error);
        }
    }

    /// Restarts every failure deferred by a completed engine-wide operation.
    pub(super) fn resume_all_deferred_runtime_recoveries(self: &Arc<Self>) {
        let pipeline_keys = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.global_shutdown_requested || state.active_engine_operation.is_some() {
                return;
            }
            let mut pipeline_keys = Vec::new();
            for deployed_key in state.deferred_runtime_recoveries.keys() {
                let pipeline_key = PipelineKey::new(
                    deployed_key.pipeline_group_id.clone(),
                    deployed_key.pipeline_id.clone(),
                );
                if !pipeline_keys.contains(&pipeline_key) {
                    pipeline_keys.push(pipeline_key);
                }
            }
            pipeline_keys
        };
        for pipeline_key in pipeline_keys {
            self.resume_deferred_runtime_recoveries_for_pipeline(&pipeline_key);
        }
    }

    /// Starts one supervised recovery worker for a failed serving core.
    fn schedule_runtime_recovery(
        self: &Arc<Self>,
        failed_key: DeployedPipelineKey,
        context_bindings: Arc<CompiledContextBindings>,
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
            if state.recovery_in_shutdown_context(&pipeline_key) {
                if state.first_error.is_none() {
                    state.first_error = Some(error.message);
                }
                return;
            }
            if state.recovery_preempted(&pipeline_key) {
                // Explicit operations own the generation transition, but a core
                // can fail after its one-time readiness check. Retain the exit so
                // ownership release can recover whichever generation ultimately
                // remains serving.
                Self::defer_runtime_recovery_locked(
                    &mut state,
                    failed_key,
                    context_bindings,
                    error,
                );
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
        let assigned_cores: Vec<_> = current_record
            .placement
            .cores
            .iter()
            .map(|core| core.core_id.id)
            .collect();
        if !assigned_cores.contains(&failed_key.core_id) {
            return;
        }

        let (worker_id, restart_count) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.recovery_in_shutdown_context(&pipeline_key) {
                if state.first_error.is_none() {
                    state.first_error = Some(error.message);
                }
                return;
            }
            if state.recovery_preempted(&pipeline_key) {
                Self::defer_runtime_recovery_locked(
                    &mut state,
                    failed_key,
                    Arc::clone(&context_bindings),
                    error,
                );
                return;
            }

            let recovery_key = (pipeline_key.clone(), failed_key.core_id);
            let recovery = state
                .runtime_recoveries
                .entry(recovery_key)
                .or_insert_with(|| RuntimeRecoveryState {
                    serving_generation: current_record.active_generation,
                    context_bindings: Arc::clone(&context_bindings),
                    restart_count: 0,
                    ready_since: None,
                    worker_id: None,
                    candidate_generation: None,
                    cancel_requested: false,
                });
            if recovery.worker_id.is_some() {
                // Multiple exit notifications must not create concurrent workers
                // that spend the same core's restart budget independently.
                self.state_changed.notify_all();
                return;
            }
            if recovery.serving_generation != failed_key.deployment_generation {
                // This is a late exit from an instance that has already been
                // superseded. Only the selected serving generation may recover.
                return;
            }
            recovery.context_bindings = Arc::clone(&context_bindings);
            if runtime_recovery_streak_expired(
                recovery.ready_since,
                policy.reset_after,
                Instant::now(),
            ) {
                recovery.restart_count = 0;
            }
            // Clear readiness when the serving runtime fails. A failed candidate
            // must never inherit time accrued by its predecessor.
            recovery.ready_since = None;
            recovery.cancel_requested = false;

            let restart_count = recovery.restart_count;
            // worker_id is a fencing token. Cleanup from an older worker is only
            // allowed while it still owns this core's recovery record.
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
            // Reserve a unique generation and compute the tentative attempt
            // before backoff, but do not charge the restart budget until this
            // worker is still eligible immediately before launch.
            let attempt =
                self.prepare_runtime_recovery_attempt(&pipeline_key, core_id, worker_id, &policy);
            let attempt = match attempt {
                RuntimeRecoveryAttemptDecision::Cancelled => {
                    self.release_cancelled_runtime_recovery_worker(
                        &pipeline_key,
                        core_id,
                        failed_generation,
                        worker_id,
                        last_error,
                    );
                    return;
                }
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
                self.release_cancelled_runtime_recovery_worker(
                    &pipeline_key,
                    core_id,
                    failed_generation,
                    worker_id,
                    last_error,
                );
                return;
            }

            if !self.commit_runtime_recovery_launch(
                &pipeline_key,
                core_id,
                worker_id,
                attempt.attempt,
                attempt.target_key.deployment_generation,
            ) {
                self.release_cancelled_runtime_recovery_worker(
                    &pipeline_key,
                    core_id,
                    failed_generation,
                    worker_id,
                    last_error,
                );
                return;
            }

            let target_key = match self.launch_regular_pipeline_instance(
                &attempt.resolved,
                &attempt.inherited_extensions,
                Arc::clone(&attempt.context_bindings),
                &attempt.placement,
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
                    // Readiness is observed outside the controller mutex and can
                    // race an exit or cancellation. Promotion therefore rechecks
                    // worker ownership and runtime liveness atomically.
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
                        let preempted = state.recovery_preempted(&pipeline_key);
                        let Some(recovery) = state
                            .runtime_recoveries
                            .get_mut(&(pipeline_key.clone(), core_id))
                        else {
                            return;
                        };
                        if recovery.worker_id != Some(worker_id)
                            || recovery.cancel_requested
                            || preempted
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
                                || state.recovery_preempted(&pipeline_key)
                        };
                        let _ = self.shutdown_instance(
                            &target_key,
                            policy.startup_timeout.as_secs().max(1),
                            "runtime recovery replacement was not promoted",
                        );
                        if cancelled {
                            self.release_cancelled_runtime_recovery_worker(
                                &pipeline_key,
                                core_id,
                                failed_generation,
                                worker_id,
                                last_error,
                            );
                            return;
                        }
                        self.clear_runtime_recovery_candidate(&pipeline_key, core_id, worker_id);
                        continue;
                    }

                    // Publish the serving overlay only after controller state has
                    // accepted the candidate. Until then, status continues to
                    // select the failed generation rather than an unready runtime.
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
                    self.release_cancelled_runtime_recovery_worker(
                        &pipeline_key,
                        core_id,
                        failed_generation,
                        worker_id,
                        last_error,
                    );
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

    /// Reserves the next generation and tentative restart ordinal.
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
        // Explicit operations always outrank automatic recovery. The worker
        // releases its token in run_runtime_recovery so it can preserve the
        // failed serving generation for post-operation handoff first.
        if recovery.worker_id != Some(worker_id)
            || recovery.cancel_requested
            || state.recovery_preempted(pipeline_key)
        {
            return RuntimeRecoveryAttemptDecision::Cancelled;
        }
        if recovery.restart_count >= policy.max_restarts {
            return RuntimeRecoveryAttemptDecision::Exhausted;
        }

        let Some((resolved, inherited_extensions, placement, placement_generation)) =
            state.logical_pipelines.get(pipeline_key).map(|record| {
                (
                    record.resolved.clone(),
                    record.inherited_extensions.clone(),
                    record.placement.clone(),
                    record.placement_generation,
                )
            })
        else {
            return RuntimeRecoveryAttemptDecision::Exhausted;
        };
        let context_bindings = Arc::clone(&recovery.context_bindings);
        let placement =
            self.live_pipeline_placement_from(&resolved, placement, placement_generation);
        let attempt = recovery.restart_count + 1;
        let target_generation = {
            // Recovery and rollouts share this counter. Generations must remain
            // unique across core-local replacements and full-pipeline changes
            // because both are keyed by DeployedPipelineKey.
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
            inherited_extensions,
            context_bindings,
            placement,
            backoff: runtime_recovery_backoff(policy, attempt),
        }))
    }

    /// Charges one restart immediately before its replacement launch.
    fn commit_runtime_recovery_launch(
        &self,
        pipeline_key: &PipelineKey,
        core_id: usize,
        worker_id: u64,
        attempt: usize,
        target_generation: u64,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.recovery_preempted(pipeline_key) {
            return false;
        }
        let Some(recovery) = state
            .runtime_recoveries
            .get_mut(&(pipeline_key.clone(), core_id))
        else {
            return false;
        };
        if recovery.worker_id != Some(worker_id)
            || recovery.cancel_requested
            || recovery.candidate_generation != Some(target_generation)
            || recovery.restart_count.checked_add(1) != Some(attempt)
        {
            return false;
        }
        recovery.restart_count = attempt;
        true
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
            // Use the condition variable instead of sleeping so rollout, shutdown,
            // or engine reconciliation can interrupt a long recovery backoff.
            let worker_active =
                state
                    .runtime_recoveries
                    .get(&recovery_key)
                    .is_some_and(|recovery| {
                        recovery.worker_id == Some(worker_id) && !recovery.cancel_requested
                    });
            if !worker_active || state.recovery_preempted(pipeline_key) {
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
                    || state.recovery_preempted(pipeline_key)
            };
            if cancelled {
                return Err(RecoveryReadyError::Cancelled);
            }

            if let Some(status) = self.observed_state_handle.pipeline_status(pipeline_key)
                && let Some(instance) =
                    status.instance_status(target_key.core_id, target_key.deployment_generation)
            {
                // Ready and accepted are necessary but not sufficient for
                // promotion. run_runtime_recovery rechecks controller liveness
                // under the state lock after this observation succeeds.
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
    ///
    /// Waiting for each worker to release its fencing token prevents an explicit
    /// rollout or shutdown from overlapping a recovery candidate.
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

    /// Requests cancellation for every active runtime recovery worker.
    ///
    /// Global shutdown closes launch admission before calling this method, so
    /// workers can be allowed to release their fencing tokens asynchronously
    /// without creating a candidate after the shutdown snapshot.
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

    /// Releases a cancelled worker and defers its failed serving generation.
    fn release_cancelled_runtime_recovery_worker(
        &self,
        pipeline_key: &PipelineKey,
        core_id: usize,
        failed_generation: u64,
        worker_id: u64,
        error: String,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let recovery_key = (pipeline_key.clone(), core_id);
        let owns_worker = state
            .runtime_recoveries
            .get(&recovery_key)
            .is_some_and(|recovery| recovery.worker_id == Some(worker_id));
        if !owns_worker {
            return;
        }

        let should_defer = state.recovery_preempted(pipeline_key)
            && !state.recovery_in_shutdown_context(pipeline_key);
        if should_defer {
            let context_bindings = state
                .runtime_recoveries
                .get(&recovery_key)
                .map(|recovery| Arc::clone(&recovery.context_bindings))
                .expect("recovery worker ownership requires recovery state");
            Self::defer_runtime_recovery_locked(
                &mut state,
                DeployedPipelineKey {
                    pipeline_group_id: pipeline_key.pipeline_group_id().clone(),
                    pipeline_id: pipeline_key.pipeline_id().clone(),
                    core_id,
                    deployment_generation: failed_generation,
                },
                context_bindings,
                RuntimeInstanceError::runtime(error),
            );
        }

        if let Some(recovery) = state.runtime_recoveries.get_mut(&recovery_key)
            && recovery.worker_id == Some(worker_id)
        {
            recovery.worker_id = None;
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
            // Preserve the first fatal cause before requesting asynchronous
            // shutdown; controller teardown uses it as the process-level error.
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
        self.release_instance_wait();
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
            if let Some(status) = self.observed_state_handle.pipeline_status(&pipeline_key)
                && let Some(instance) =
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
    pub(crate) fn instance_exit(
        &self,
        deployed_key: &DeployedPipelineKey,
    ) -> Option<RuntimeInstanceExit> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // A reused key may still retain the previous launch's terminal record.
        if state.launching_instances.contains_key(deployed_key) {
            return None;
        }
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
        let drain_deadline = Instant::now() + Duration::from_secs(timeout_secs.max(1));
        self.request_instance_shutdown_until(deployed_key, drain_deadline, reason)
    }

    /// Sends shutdown with an absolute drain deadline.
    pub(super) fn request_instance_shutdown_until(
        &self,
        deployed_key: &DeployedPipelineKey,
        drain_deadline: Instant,
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

        if let Err(err) = sender.try_send_shutdown(drain_deadline, reason.to_owned()) {
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
                    "timed out waiting for pipeline {}:{} core={} generation={} to shut down",
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
            if state.launching_instances.contains_key(deployed_key) {
                let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                    return Err(format!(
                        "timed out waiting for pipeline {} to finish launching and drain before system observability shutdown",
                        deployed_instance_label(deployed_key)
                    ));
                };
                let (next_state, _) = self
                    .state_changed
                    .wait_timeout(state, remaining)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                state = next_state;
                continue;
            }
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
                    "timed out waiting for pipeline {} to shut down before system observability shutdown",
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
        let drain_deadline = Instant::now() + Duration::from_secs(timeout_secs.max(1));
        self.request_instance_shutdown_until(deployed_key, drain_deadline, reason)?;
        self.wait_for_instance_exit(
            deployed_key,
            pipeline_shutdown_completion_deadline(drain_deadline),
        )
    }

    /// Drops the retained admin sender after shutdown has been accepted.
    ///
    /// The retained sender is the controller's "not yet signaled" marker for
    /// an active instance. Releasing it makes shutdown dispatch idempotent for
    /// that instance and lets the pipeline control loop observe channel closure
    /// once node tasks have exited.
    pub(crate) fn release_instance_control_sender(&self, deployed_key: &DeployedPipelineKey) {
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
    /// and awaited first. Observability remains active until the controller marks
    /// extension scope hosts stopped, then a later call starts its final phase.
    /// Repeated requests retain the first producer deadline. Observability gets
    /// its own fixed window when its dependencies have stopped.
    pub(super) fn request_shutdown_all(
        self: &Arc<Self>,
        timeout_secs: u64,
    ) -> Result<(), ControlPlaneError> {
        self.request_shutdown_all_until(Instant::now() + Duration::from_secs(timeout_secs.max(1)))
    }

    /// Returns the established producer deadline, creating it only once.
    pub(crate) fn global_shutdown_deadline_or_insert(&self, timeout: Duration) -> Instant {
        let candidate = Instant::now() + timeout;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *state.global_shutdown_deadline.get_or_insert(candidate)
    }

    /// Returns the fixed deadline for the final observability phase.
    ///
    /// Call only after producer pipelines and extension scope hosts stop.
    pub(crate) fn observability_shutdown_deadline_or_insert(&self) -> Instant {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *state
            .observability_shutdown_deadline
            .get_or_insert_with(|| Instant::now() + Self::OBSERVABILITY_SHUTDOWN_TIMEOUT)
    }

    /// Requests global shutdown with an absolute deadline for producer draining.
    pub(crate) fn request_shutdown_all_until(
        self: &Arc<Self>,
        requested_deadline: Instant,
    ) -> Result<(), ControlPlaneError> {
        let deadline = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let deadline = state.global_shutdown_deadline.unwrap_or(requested_deadline);
            state.launches_closed = true;
            state.global_shutdown_requested = true;
            state.global_shutdown_deadline = Some(deadline);
            self.state_changed.notify_all();
            deadline
        };
        self.cancel_all_runtime_recoveries();

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
            let extension_scope_hosts_stopped = state.extension_scope_hosts_stopped;
            let mut producer_keys = Vec::new();
            let mut producer_senders = Vec::new();
            let mut observability_senders = Vec::new();

            for (deployed_key, instance) in &mut state.runtime_instances {
                if !matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active) {
                    continue;
                }

                let is_observability = is_observability_instance(deployed_key);
                if is_observability {
                    if extension_scope_hosts_stopped
                        && !coordinator_active
                        && let Some(sender) = instance.control_sender.take()
                    {
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
            producer_keys.extend(
                state
                    .launching_instances
                    .keys()
                    .filter(|deployed_key| !is_observability_instance(deployed_key))
                    .cloned(),
            );

            let coordinator_reserved = !coordinator_active
                && (!producer_keys.is_empty() || !observability_senders.is_empty());
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
    ) {
        let mut wait_failures = Vec::new();
        let producer_completion_deadline = pipeline_shutdown_completion_deadline(producer_deadline);
        for deployed_key in &producer_keys {
            if let Err(error) =
                self.wait_for_global_shutdown_exit(deployed_key, producer_completion_deadline)
            {
                wait_failures.push(error);
            }
        }
        if !wait_failures.is_empty() {
            self.record_async_global_shutdown_failure(format!(
                "producer shutdown failed before system observability shutdown: {}",
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
                    state.launching_instances.contains_key(*deployed_key)
                        || matches!(
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

        if observability_senders.is_empty() {
            return;
        }
        let observability_deadline = self.observability_shutdown_deadline_or_insert();
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

        let observability_completion_deadline =
            pipeline_shutdown_completion_deadline(observability_deadline);
        for deployed_key in observability_keys {
            if let Err(error) =
                self.wait_for_global_shutdown_exit(&deployed_key, observability_completion_deadline)
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

    /// Waits up to `timeout` for every phased global-shutdown coordinator.
    pub(crate) fn wait_for_global_shutdown_completion_for(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.global_shutdown_coordinators > 0 {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            let (next_state, wait_result) = self
                .state_changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if wait_result.timed_out() && state.global_shutdown_coordinators > 0 {
                return false;
            }
        }
        true
    }

    /// Returns whether every registered runtime instance has exited.
    #[cfg(test)]
    pub(crate) fn all_instances_exited(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_instances
            == 0
    }

    /// Records a fatal controller-owned runtime error for final teardown.
    pub(crate) fn record_fatal_runtime_error(&self, message: String) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.first_error.is_none() {
            state.first_error = Some(message);
        }
        state.instance_wait_released = true;
        self.state_changed.notify_all();
    }

    /// Returns whether teardown has a fatal runtime error to surface.
    pub(crate) fn has_fatal_runtime_error(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .first_error
            .is_some()
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
        state.instance_wait_released = true;
        self.state_changed.notify_all();
    }

    /// Starts a tracked shutdown operation for one logical pipeline.
    #[cfg(test)]
    pub(super) fn request_shutdown_pipeline(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        self.request_shutdown_pipeline_for_initiator(
            pipeline_group_id,
            pipeline_id,
            timeout_secs,
            None,
        )
    }

    /// Starts a tracked shutdown with its external initiator.
    pub(super) fn request_shutdown_pipeline_with_initiator(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
        initiator: PipelineShutdownInitiator,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        self.request_shutdown_pipeline_for_initiator(
            pipeline_group_id,
            pipeline_id,
            timeout_secs,
            Some(initiator),
        )
    }

    fn request_shutdown_pipeline_for_initiator(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
        initiator: Option<PipelineShutdownInitiator>,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        // Keep the reservation alive until active_shutdowns contains the
        // accepted operation. A failure in the cancel-to-insert window is then
        // treated as part of shutdown instead of starting a replacement.
        let _reservation = self.begin_pipeline_operation_reservation(
            PipelineKey::new(
                pipeline_group_id.to_owned().into(),
                pipeline_id.to_owned().into(),
            ),
            PipelineOperationKind::Shutdown,
        )?;
        self.request_shutdown_pipeline_for_engine_operation(
            pipeline_group_id,
            pipeline_id,
            timeout_secs,
            None,
            initiator,
        )
    }

    pub(super) fn request_shutdown_pipeline_for_engine_operation(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
        engine_operation_id: Option<&str>,
        initiator: Option<PipelineShutdownInitiator>,
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
            initiator,
        )?;
        self.spawn_shutdown_for_engine_operation(plan, engine_operation_id)
    }

    /// Blocks until all active runtime instances have exited, or until the wait
    /// is released via [`release_instance_wait`](Self::release_instance_wait).
    #[cfg(test)]
    pub(crate) fn wait_until_all_instances_exit(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.active_instances > 0 && !state.instance_wait_released {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Blocks until an explicit global shutdown has been requested and every
    /// runtime instance has exited, or until the wait is released after a fatal
    /// controller failure.
    ///
    /// Unlike [`wait_until_all_instances_exit`](Self::wait_until_all_instances_exit),
    /// this does not return merely because there are currently no active
    /// instances. Standard engine mode must remain alive for live-control
    /// operations even when it starts empty or the last pipeline stops.
    pub(crate) fn wait_until_global_shutdown_drains_or_released(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !state.instance_wait_released
            && (!state.global_shutdown_requested
                || Self::has_live_producer_instances_locked(&state))
        {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Releases controller lifecycle waits even if runtime instances are still active.
    ///
    /// This is a fatal-shutdown escape hatch: when a controller extension fails
    /// or runtime recovery is exhausted, the engine tears down regardless of
    /// whether the graceful drain of pipeline instances completes. The main
    /// controller thread must not block forever if that drain stalls. The latch
    /// is one-way for the current run, after which the controller proceeds to
    /// teardown.
    pub(crate) fn release_instance_wait(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.instance_wait_released = true;
        self.state_changed.notify_all();
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
        while !state.instance_wait_released && Self::has_live_producer_instances_locked(&state) {
            state = self
                .state_changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Returns whether every non-observability runtime instance has exited.
    pub(crate) fn all_producer_instances_exited(&self) -> bool {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        !Self::has_live_producer_instances_locked(&state)
    }

    /// Waits for non-observability runtime instances without honoring the fatal-release latch.
    pub(crate) fn wait_until_all_producer_instances_exit_for(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while Self::has_live_producer_instances_locked(&state) {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            let (next_state, wait_result) = self
                .state_changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if wait_result.timed_out() && Self::has_live_producer_instances_locked(&state) {
                return false;
            }
        }
        true
    }

    /// Waits for every recovery worker to release its launch fencing token.
    pub(crate) fn wait_for_runtime_recoveries_for(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state
            .runtime_recoveries
            .values()
            .any(|recovery| recovery.worker_id.is_some())
        {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            let (next_state, wait_result) = self
                .state_changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if wait_result.timed_out()
                && state
                    .runtime_recoveries
                    .values()
                    .any(|recovery| recovery.worker_id.is_some())
            {
                return false;
            }
        }
        true
    }

    fn has_live_producer_instances_locked(state: &ControllerRuntimeState) -> bool {
        state
            .launching_instances
            .keys()
            .any(|key| !is_observability_instance(key))
            || state.runtime_instances.iter().any(|(key, instance)| {
                matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                    && !is_observability_instance(key)
            })
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
