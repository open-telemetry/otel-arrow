use super::*;

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable>
    ControllerRuntime<PData>
{
    pub(super) fn report_controller_worker_panic(
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
    pub(super) fn handle_rollout_worker_panic(
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

    pub(super) fn handle_shutdown_worker_panic(
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

    pub(super) fn run_rollout(self: Arc<Self>, plan: CandidateRolloutPlan) {
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
    pub(super) fn run_shutdown(self: Arc<Self>, plan: CandidateShutdownPlan) {
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
    pub(super) fn run_create_rollout(
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
    pub(super) fn run_resize_rollout(
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
    pub(super) fn run_replace_rollout(
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
    pub(super) fn rollback_resize_rollout(
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
    pub(super) fn rollback_replace_rollout(
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
    pub(super) fn shutdown_instances(
        self: &Arc<Self>,
        keys: &[DeployedPipelineKey],
        timeout_secs: u64,
    ) -> Result<(), String> {
        for key in keys {
            self.shutdown_instance(key, timeout_secs, "candidate cleanup")?;
        }
        Ok(())
    }
}
