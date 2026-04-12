use super::*;

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable>
    ControllerRuntime<PData>
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

    /// Drops the retained admin sender so draining can observe channel closure.
    pub(super) fn release_instance_control_sender(&self, deployed_key: &DeployedPipelineKey) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(instance) = state.runtime_instances.get_mut(deployed_key) {
            instance.control_sender = None;
        }
    }

    /// Broadcasts shutdown to every currently active runtime instance.
    pub(super) fn request_shutdown_all(&self, timeout_secs: u64) -> Result<(), ControlPlaneError> {
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
    pub(super) fn request_shutdown_pipeline(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        let plan = self.prepare_shutdown_plan(pipeline_group_id, pipeline_id, timeout_secs)?;
        self.spawn_shutdown(plan)
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
