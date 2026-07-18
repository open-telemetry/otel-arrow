// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime-instance launch, shutdown, and exit reporting.
//!
//! This module owns the boundary between controller state and actual pipeline
//! threads. It registers launched instances, reconciles early exits, sends
//! shutdown control messages, waits for readiness/exit transitions, and exposes
//! global runtime shutdown/error helpers used by controller teardown.

use super::*;

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
        let should_compact = {
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
            should_compact
        };

        if should_compact {
            let logical_pipeline_key = PipelineKey::new(
                launched.pipeline_key.pipeline_group_id.clone(),
                launched.pipeline_key.pipeline_id.clone(),
            );
            self.observed_state_store
                .compact_pipeline_instances(&logical_pipeline_key);
        }
    }

    /// Records a pipeline instance exit and closes the registration-before/after-exit race.
    ///
    /// If the instance is already visible in runtime_instances, the exit is applied immediately.
    /// Otherwise we store it in pending_instance_exits so register_launched_instance() can
    /// reconcile it as soon as registration becomes visible.
    pub(crate) fn note_instance_exit(
        &self,
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

        let should_compact = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.runtime_instances.contains_key(&pipeline_key) {
                Self::apply_instance_exit_locked(&mut state, &pipeline_key, &exit)
            } else {
                _ = state
                    .pending_instance_exits
                    .insert(pipeline_key.clone(), exit.clone());
                false
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
