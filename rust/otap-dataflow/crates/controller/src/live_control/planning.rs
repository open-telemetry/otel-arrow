// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Request planning, operation recording, and worker spawning.
//!
//! Planning converts admin requests into explicit candidate plans while holding
//! no long-running runtime resources. It also owns operation-record insertion
//! and status snapshot materialization because those steps are tightly coupled
//! to conflict detection and bounded history retention.

use super::*;

struct EngineOperationGuard<'a, PData: 'static + Clone + Send + Sync + std::fmt::Debug> {
    runtime: &'a ControllerRuntime<PData>,
    operation_id: String,
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug> EngineOperationGuard<'_, PData> {
    fn operation_id(&self) -> &str {
        &self.operation_id
    }
}

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug> Drop
    for EngineOperationGuard<'_, PData>
{
    fn drop(&mut self) {
        let mut state = self
            .runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state
            .active_engine_operation
            .as_deref()
            .is_some_and(|operation_id| operation_id == self.operation_id)
        {
            state.active_engine_operation = None;
        }
    }
}

impl<
    PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable + FlowMetricHook,
> ControllerRuntime<PData>
{
    fn engine_operation_allows(state: &ControllerRuntimeState, operation_id: Option<&str>) -> bool {
        match (state.active_engine_operation.as_deref(), operation_id) {
            (None, None) => true,
            (Some(active), Some(operation_id)) if active == operation_id => true,
            _ => false,
        }
    }

    fn begin_named_engine_operation(
        &self,
        operation_id: String,
    ) -> Result<EngineOperationGuard<'_, PData>, ControlPlaneError> {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.active_engine_operation.is_some() {
                return Err(ControlPlaneError::RolloutConflict);
            }
            state.active_engine_operation = Some(operation_id.clone());
        }
        Ok(EngineOperationGuard {
            runtime: self,
            operation_id,
        })
    }

    fn begin_reconcile_operation(
        &self,
    ) -> Result<(String, EngineOperationGuard<'_, PData>), ControlPlaneError> {
        let reconcile_id = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.active_engine_operation.is_some() {
                return Err(ControlPlaneError::RolloutConflict);
            }
            let reconcile_id = format!("reconcile-{}", state.next_reconcile_id);
            state.next_reconcile_id += 1;
            state.active_engine_operation = Some(reconcile_id.clone());
            reconcile_id
        };
        Ok((
            reconcile_id.clone(),
            EngineOperationGuard {
                runtime: self,
                operation_id: reconcile_id,
            },
        ))
    }

    /// Resolves the concrete core ids selected by a pipeline resource policy.
    pub(super) fn assigned_cores_for_resolved(
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
    pub(super) fn active_runtime_core_state(
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

    /// Builds the effective runtime topic profile map used to reject broker mutations.
    pub(super) fn pipeline_topic_profiles(
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

    // The process-wide memory limiter owns a runtime pressure-monitoring task
    // that is created during controller startup. Live reconciliation can replace
    // pipelines, but it does not currently restart or reconfigure that task, so
    // keep the pressure source immutable for live updates.
    fn validate_live_memory_limiter_unchanged(
        current_config: &OtelDataflowSpec,
        desired_config: &OtelDataflowSpec,
    ) -> Result<(), ControlPlaneError> {
        let current = current_config
            .policies
            .resources()
            .and_then(|resources| resources.memory_limiter.as_ref());
        let desired = desired_config
            .policies
            .resources()
            .and_then(|resources| resources.memory_limiter.as_ref());
        if current != desired {
            return Err(ControlPlaneError::InvalidRequest {
                message: "request would require runtime memory_limiter mutation".to_owned(),
            });
        }
        Ok(())
    }

    /// Classifies a reconfigure request and prepares the rollout state machine inputs.
    pub(super) fn prepare_rollout_plan(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: &ReconfigureRequest,
    ) -> Result<CandidateRolloutPlan, ControlPlaneError> {
        self.prepare_rollout_plan_for_engine_operation(
            pipeline_group_id,
            pipeline_id,
            request,
            None,
            None,
        )
    }

    fn prepare_rollout_plan_for_engine_operation(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: &ReconfigureRequest,
        planning_config: Option<&OtelDataflowSpec>,
        engine_operation_id: Option<&str>,
    ) -> Result<CandidateRolloutPlan, ControlPlaneError> {
        let pipeline_group_id: PipelineGroupId = pipeline_group_id.to_owned().into();
        let pipeline_id: PipelineId = pipeline_id.to_owned().into();
        let pipeline_key = PipelineKey::new(pipeline_group_id.clone(), pipeline_id.clone());

        let (live_config, current_record) = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !Self::engine_operation_allows(&state, engine_operation_id) {
                return Err(ControlPlaneError::RolloutConflict);
            }
            let plan_config = planning_config.unwrap_or(&state.live_config);
            if !plan_config.groups.contains_key(&pipeline_group_id) {
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

        let candidate_pipeline = request.pipeline.clone();
        candidate_pipeline
            .validate(&pipeline_group_id, &pipeline_id)
            .map_err(|err| ControlPlaneError::InvalidRequest {
                message: err.to_string(),
            })?;

        let mut candidate_config = planning_config
            .cloned()
            .unwrap_or_else(|| live_config.clone());
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
        startup::validate_engine_components(&candidate_config, self.pipeline_factory).map_err(
            |error| ControlPlaneError::InvalidRequest {
                message: error.to_string(),
            },
        )?;

        let current_profiles = Self::pipeline_topic_profiles(&live_config)?;
        let candidate_profiles = Self::pipeline_topic_profiles(&candidate_config)?;
        if current_profiles != candidate_profiles {
            return Err(ControlPlaneError::InvalidRequest {
                message: "request would require runtime topic broker mutation".to_owned(),
            });
        }
        Self::validate_live_memory_limiter_unchanged(&live_config, &candidate_config)?;

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
                && record.resolved.runtime_matches(&resolved_pipeline);
            let resize_only = current_assigned_cores != target_assigned_cores
                && !active_runtime_state.has_foreign_active_generations
                && record
                    .resolved
                    .runtime_shape_matches_ignoring_resources(&resolved_pipeline);
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
        let step_timeout_secs = request.step_timeout_secs.max(1);
        let drain_timeout_secs = request.drain_timeout_secs.max(1);
        let rollout = RolloutRecord::new(
            rollout_id,
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            action,
            target_generation,
            current_record
                .as_ref()
                .map(|record| record.active_generation),
            drain_timeout_secs,
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
            step_timeout_secs,
            drain_timeout_secs,
        })
    }

    /// Registers a newly accepted rollout and publishes its initial summary.
    #[cfg(test)]
    pub(super) fn insert_rollout(
        &self,
        pipeline_key: &PipelineKey,
        rollout: RolloutRecord,
    ) -> Result<(), ControlPlaneError> {
        self.insert_rollout_for_engine_operation(pipeline_key, rollout, None)
    }

    fn insert_rollout_for_engine_operation(
        &self,
        pipeline_key: &PipelineKey,
        rollout: RolloutRecord,
        engine_operation_id: Option<&str>,
    ) -> Result<(), ControlPlaneError> {
        self.prune_retained_operation_history();
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !Self::engine_operation_allows(&state, engine_operation_id) {
                return Err(ControlPlaneError::RolloutConflict);
            }
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
    pub(super) fn update_rollout<F>(&self, pipeline_key: &PipelineKey, rollout_id: &str, update: F)
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
    pub(super) fn update_rollout_core_state(
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
    pub(super) fn finish_rollout(&self, pipeline_key: &PipelineKey, rollout_id: &str) {
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

    /// Returns the latest rollout snapshot, evicting expired history first.
    pub(super) fn rollout_status_snapshot(&self, rollout_id: &str) -> Option<RolloutStatus> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::prune_terminal_operation_history_locked(&mut state, Instant::now());
        state.rollouts.get(rollout_id).map(RolloutRecord::status)
    }

    /// Clears temporary serving-generation overrides after a rollout settles.
    pub(super) fn clear_pipeline_serving_generations<I>(
        &self,
        pipeline_key: &PipelineKey,
        core_ids: I,
    ) where
        I: IntoIterator<Item = usize>,
    {
        for core_id in core_ids {
            self.observed_state_store
                .clear_pipeline_serving_generation(pipeline_key.clone(), core_id);
        }
    }

    /// Commits the winning pipeline config and active generation into runtime state.
    pub(super) fn commit_pipeline_record(
        &self,
        plan: &CandidateRolloutPlan,
        active_generation: u64,
    ) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            _ = state
                .live_config
                .groups
                .entry(plan.pipeline_group_id.clone())
                .or_default()
                .pipelines
                .insert(
                    plan.pipeline_id.clone(),
                    plan.resolved_pipeline.pipeline.clone(),
                );
            _ = state.logical_pipelines.insert(
                plan.pipeline_key.clone(),
                LogicalPipelineRecord {
                    resolved: plan.resolved_pipeline.clone(),
                    active_generation,
                },
            );
        }
        self.observed_state_store.set_pipeline_active_cores(
            plan.pipeline_key.clone(),
            plan.target_assigned_cores.iter().copied(),
        );
        self.observed_state_store
            .set_pipeline_active_generation(plan.pipeline_key.clone(), active_generation);
    }

    /// Selects the active instances targeted by a per-pipeline shutdown request.
    #[cfg(test)]
    pub(super) fn prepare_shutdown_plan(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<CandidateShutdownPlan, ControlPlaneError> {
        self.prepare_shutdown_plan_for_engine_operation(
            pipeline_group_id,
            pipeline_id,
            timeout_secs,
            None,
        )
    }

    pub(super) fn prepare_shutdown_plan_for_engine_operation(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
        engine_operation_id: Option<&str>,
    ) -> Result<CandidateShutdownPlan, ControlPlaneError> {
        let pipeline_group_id: PipelineGroupId = pipeline_group_id.to_owned().into();
        let pipeline_id: PipelineId = pipeline_id.to_owned().into();
        let pipeline_key = PipelineKey::new(pipeline_group_id.clone(), pipeline_id.clone());

        let target_instances = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !Self::engine_operation_allows(&state, engine_operation_id) {
                return Err(ControlPlaneError::RolloutConflict);
            }
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
            if !Self::engine_operation_allows(&state, engine_operation_id) {
                return Err(ControlPlaneError::RolloutConflict);
            }
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
    #[cfg(test)]
    pub(super) fn insert_shutdown(
        &self,
        pipeline_key: &PipelineKey,
        shutdown: ShutdownRecord,
    ) -> Result<(), ControlPlaneError> {
        self.insert_shutdown_for_engine_operation(pipeline_key, shutdown, None)
    }

    fn insert_shutdown_for_engine_operation(
        &self,
        pipeline_key: &PipelineKey,
        shutdown: ShutdownRecord,
        engine_operation_id: Option<&str>,
    ) -> Result<(), ControlPlaneError> {
        self.prune_retained_operation_history();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !Self::engine_operation_allows(&state, engine_operation_id) {
            return Err(ControlPlaneError::RolloutConflict);
        }
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
    pub(super) fn update_shutdown<F>(
        &self,
        pipeline_key: &PipelineKey,
        shutdown_id: &str,
        update: F,
    ) where
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
    pub(super) fn shutdown_status_snapshot(&self, shutdown_id: &str) -> Option<ShutdownStatus> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::prune_terminal_operation_history_locked(&mut state, Instant::now());
        state.shutdowns.get(shutdown_id).map(ShutdownRecord::status)
    }

    /// Returns committed pipeline details plus any active rollout summary.
    pub(super) fn pipeline_details_snapshot(
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

    /// Returns the committed configuration for one pipeline group.
    pub(super) fn group_details_snapshot(
        &self,
        pipeline_group_id: &PipelineGroupId,
    ) -> Option<PipelineGroupConfig> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.live_config.groups.get(pipeline_group_id).cloned()
    }

    /// Creates an empty pipeline group in the controller-owned live config.
    pub(super) fn create_group(
        &self,
        pipeline_group_id: &str,
        group: PipelineGroupConfig,
    ) -> Result<PipelineGroupConfig, ControlPlaneError> {
        let guard =
            self.begin_named_engine_operation(format!("create-group:{pipeline_group_id}"))?;
        let pipeline_group_id: PipelineGroupId = pipeline_group_id.to_owned().into();

        if !group.pipelines.is_empty() {
            return Err(ControlPlaneError::InvalidRequest {
                message: "pipeline group creation only supports empty groups".to_owned(),
            });
        }
        if !group.topics.is_empty() {
            return Err(ControlPlaneError::InvalidRequest {
                message: "pipeline group creation with topic declarations is not supported yet"
                    .to_owned(),
            });
        }
        group
            .validate(&pipeline_group_id)
            .map_err(|err| ControlPlaneError::InvalidRequest {
                message: err.to_string(),
            })?;

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !Self::engine_operation_allows(&state, Some(guard.operation_id())) {
            return Err(ControlPlaneError::RolloutConflict);
        }
        if state.live_config.groups.contains_key(&pipeline_group_id) {
            return Err(ControlPlaneError::GroupAlreadyExists);
        }

        let mut candidate_config = state.live_config.clone();
        _ = candidate_config
            .groups
            .insert(pipeline_group_id.clone(), group.clone());
        candidate_config
            .validate()
            .map_err(|err| ControlPlaneError::InvalidRequest {
                message: err.to_string(),
            })?;
        state.live_config = candidate_config;
        Ok(group)
    }

    /// Returns a clone of the current controller-owned engine configuration.
    pub(super) fn engine_config_snapshot(&self) -> OtelDataflowSpec {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.live_config.clone()
    }

    fn apply_reconcile_success(&self, desired_config: &OtelDataflowSpec, delete_missing: bool) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if delete_missing {
            state.live_config = desired_config.clone();
            return;
        }

        state.live_config.version = desired_config.version.clone();
        state.live_config.policies = desired_config.policies.clone();
        state.live_config.topics = desired_config.topics.clone();
        state.live_config.engine = desired_config.engine.clone();
        for (pipeline_group_id, desired_group) in &desired_config.groups {
            let group = state
                .live_config
                .groups
                .entry(pipeline_group_id.clone())
                .or_default();
            group.policies = desired_group.policies.clone();
            group.topics = desired_group.topics.clone();
            for (pipeline_id, pipeline) in &desired_group.pipelines {
                _ = group
                    .pipelines
                    .insert(pipeline_id.clone(), pipeline.clone());
            }
        }
    }

    fn live_pipeline_keys(&self) -> Vec<PipelineKey> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut keys: Vec<_> = state
            .live_config
            .groups
            .iter()
            .flat_map(|(pipeline_group_id, group)| {
                group.pipelines.keys().map(|pipeline_id| {
                    PipelineKey::new(pipeline_group_id.clone(), pipeline_id.clone())
                })
            })
            .collect();
        for pipeline_key in state.logical_pipelines.keys() {
            if !keys.contains(pipeline_key) {
                keys.push(pipeline_key.clone());
            }
        }
        keys.sort_by(|left, right| {
            left.pipeline_group_id()
                .as_ref()
                .cmp(right.pipeline_group_id().as_ref())
                .then_with(|| {
                    left.pipeline_id()
                        .as_ref()
                        .cmp(right.pipeline_id().as_ref())
                })
        });
        keys
    }

    fn live_group_ids(&self) -> Vec<PipelineGroupId> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut ids: Vec<_> = state.live_config.groups.keys().cloned().collect();
        ids.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));
        ids
    }

    fn remove_pipeline_record_for_engine_operation(
        &self,
        pipeline_key: &PipelineKey,
        engine_operation_id: Option<&str>,
    ) -> Result<(), ControlPlaneError> {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !Self::engine_operation_allows(&state, engine_operation_id) {
                return Err(ControlPlaneError::RolloutConflict);
            }
            if state.active_rollouts.contains_key(pipeline_key)
                || state.active_shutdowns.contains_key(pipeline_key)
            {
                return Err(ControlPlaneError::RolloutConflict);
            }

            for (deployed_key, instance) in &state.runtime_instances {
                if deployed_key.pipeline_group_id == *pipeline_key.pipeline_group_id()
                    && deployed_key.pipeline_id == *pipeline_key.pipeline_id()
                    && matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                {
                    return Err(ControlPlaneError::InvalidRequest {
                        message: format!(
                            "pipeline {}:{} still has active runtime instances",
                            pipeline_key.pipeline_group_id().as_ref(),
                            pipeline_key.pipeline_id().as_ref()
                        ),
                    });
                }
            }

            if let Some(group) = state
                .live_config
                .groups
                .get_mut(pipeline_key.pipeline_group_id())
            {
                let _ = group.pipelines.remove(pipeline_key.pipeline_id());
            }
            let _ = state.logical_pipelines.remove(pipeline_key);
            let _ = state.generation_counters.remove(pipeline_key);
            state.runtime_instances.retain(|deployed_key, _| {
                deployed_key.pipeline_group_id != *pipeline_key.pipeline_group_id()
                    || deployed_key.pipeline_id != *pipeline_key.pipeline_id()
            });
            state.pending_instance_exits.retain(|deployed_key, _| {
                deployed_key.pipeline_group_id != *pipeline_key.pipeline_group_id()
                    || deployed_key.pipeline_id != *pipeline_key.pipeline_id()
            });
            if let Some(ids) = state.terminal_rollouts.remove(pipeline_key) {
                for rollout_id in ids {
                    let _ = state.rollouts.remove(&rollout_id);
                }
            }
            if let Some(ids) = state.terminal_shutdowns.remove(pipeline_key) {
                for shutdown_id in ids {
                    let _ = state.shutdowns.remove(&shutdown_id);
                }
            }
        }

        self.observed_state_store.remove_pipeline(pipeline_key);
        Ok(())
    }

    fn rollout_change_action(action: RolloutAction) -> ConfigChangeAction {
        match action {
            RolloutAction::Create => ConfigChangeAction::Create,
            RolloutAction::NoOp => ConfigChangeAction::Noop,
            RolloutAction::Replace => ConfigChangeAction::Replace,
            RolloutAction::Resize => ConfigChangeAction::Resize,
        }
    }

    fn rollout_terminal(status: &RolloutStatus) -> bool {
        matches!(
            status.state,
            ApiPipelineRolloutState::Succeeded
                | ApiPipelineRolloutState::Failed
                | ApiPipelineRolloutState::RollbackFailed
        )
    }

    fn rollout_succeeded(status: &RolloutStatus) -> bool {
        status.state == ApiPipelineRolloutState::Succeeded
    }

    fn wait_for_rollout_terminal(&self, initial_status: RolloutStatus) -> RolloutStatus {
        let mut status = initial_status;
        while !Self::rollout_terminal(&status) {
            thread::sleep(Duration::from_millis(50));
            let Some(next_status) = self.rollout_status_snapshot(&status.rollout_id) else {
                return status;
            };
            status = next_status;
        }
        status
    }

    fn wait_for_shutdown_terminal(&self, initial_status: ShutdownStatus) -> ShutdownStatus {
        let mut status = initial_status;
        while status.state != "succeeded" && status.state != "failed" {
            thread::sleep(Duration::from_millis(50));
            let Some(next_status) = self.shutdown_status_snapshot(&status.shutdown_id) else {
                return status;
            };
            status = next_status;
        }
        status
    }

    /// Gracefully drains and removes one pipeline from live controller state.
    pub(super) fn request_delete_pipeline(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
    ) -> Result<PipelineDeleteStatus, ControlPlaneError> {
        let operation_id = format!("delete-pipeline:{pipeline_group_id}:{pipeline_id}");
        let guard = self.begin_named_engine_operation(operation_id)?;
        self.request_delete_pipeline_for_engine_operation(
            pipeline_group_id,
            pipeline_id,
            timeout_secs,
            Some(guard.operation_id()),
        )
    }

    fn request_delete_pipeline_for_engine_operation(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        pipeline_id: &str,
        timeout_secs: u64,
        engine_operation_id: Option<&str>,
    ) -> Result<PipelineDeleteStatus, ControlPlaneError> {
        let started_at = timestamp_now();
        let pipeline_group_id: PipelineGroupId = pipeline_group_id.to_owned().into();
        let pipeline_id: PipelineId = pipeline_id.to_owned().into();
        let pipeline_key = PipelineKey::new(pipeline_group_id.clone(), pipeline_id.clone());
        let has_active_runtime = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !Self::engine_operation_allows(&state, engine_operation_id) {
                return Err(ControlPlaneError::RolloutConflict);
            }
            let Some(group) = state.live_config.groups.get(&pipeline_group_id) else {
                return Err(ControlPlaneError::GroupNotFound);
            };
            if !group.pipelines.contains_key(&pipeline_id)
                && !state.logical_pipelines.contains_key(&pipeline_key)
            {
                return Err(ControlPlaneError::PipelineNotFound);
            }
            if state.active_rollouts.contains_key(&pipeline_key)
                || state.active_shutdowns.contains_key(&pipeline_key)
            {
                return Err(ControlPlaneError::RolloutConflict);
            }
            state
                .runtime_instances
                .iter()
                .any(|(deployed_key, instance)| {
                    deployed_key.pipeline_group_id == pipeline_group_id
                        && deployed_key.pipeline_id == pipeline_id
                        && matches!(instance.lifecycle, RuntimeInstanceLifecycle::Active)
                })
        };

        let shutdown = if has_active_runtime {
            match self.request_shutdown_pipeline_for_engine_operation(
                &pipeline_group_id,
                &pipeline_id,
                timeout_secs,
                engine_operation_id,
            ) {
                Ok(initial_shutdown) => {
                    let terminal_shutdown = self.wait_for_shutdown_terminal(initial_shutdown);
                    if terminal_shutdown.state != "succeeded" {
                        return Ok(PipelineDeleteStatus {
                            pipeline_group_id,
                            pipeline_id,
                            state: "failed".to_owned(),
                            started_at,
                            updated_at: timestamp_now(),
                            shutdown: Some(terminal_shutdown.clone()),
                            failure_reason: terminal_shutdown.failure_reason.clone().or_else(
                                || {
                                    Some(
                                        "pipeline shutdown did not complete successfully"
                                            .to_owned(),
                                    )
                                },
                            ),
                        });
                    }
                    Some(terminal_shutdown)
                }
                // The pipeline may have stopped between the has_active_runtime
                // check and the shutdown request. Treat this as already-stopped
                // and proceed directly to record removal.
                Err(ControlPlaneError::InvalidRequest { ref message })
                    if message.contains("already stopped") =>
                {
                    None
                }
                Err(err) => return Err(err),
            }
        } else {
            None
        };

        self.remove_pipeline_record_for_engine_operation(&pipeline_key, engine_operation_id)?;
        Ok(PipelineDeleteStatus {
            pipeline_group_id,
            pipeline_id,
            state: "succeeded".to_owned(),
            started_at,
            updated_at: timestamp_now(),
            shutdown,
            failure_reason: None,
        })
    }

    /// Gracefully drains and removes one group from live controller state.
    pub(super) fn request_delete_group(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        timeout_secs: u64,
    ) -> Result<GroupDeleteStatus, ControlPlaneError> {
        let operation_id = format!("delete-group:{pipeline_group_id}");
        let guard = self.begin_named_engine_operation(operation_id)?;
        self.request_delete_group_for_engine_operation(
            pipeline_group_id,
            timeout_secs,
            Some(guard.operation_id()),
        )
    }

    fn request_delete_group_for_engine_operation(
        self: &Arc<Self>,
        pipeline_group_id: &str,
        timeout_secs: u64,
        engine_operation_id: Option<&str>,
    ) -> Result<GroupDeleteStatus, ControlPlaneError> {
        let started_at = timestamp_now();
        let pipeline_group_id: PipelineGroupId = pipeline_group_id.to_owned().into();
        let pipeline_ids = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !Self::engine_operation_allows(&state, engine_operation_id) {
                return Err(ControlPlaneError::RolloutConflict);
            }
            let Some(group) = state.live_config.groups.get(&pipeline_group_id) else {
                return Err(ControlPlaneError::GroupNotFound);
            };
            let mut ids: Vec<_> = group.pipelines.keys().cloned().collect();
            for pipeline_key in state.logical_pipelines.keys() {
                if pipeline_key.pipeline_group_id() == &pipeline_group_id
                    && !ids.contains(pipeline_key.pipeline_id())
                {
                    ids.push(pipeline_key.pipeline_id().clone());
                }
            }
            ids.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));
            ids
        };

        let mut pipelines = Vec::new();
        for pipeline_id in pipeline_ids {
            let delete = self.request_delete_pipeline_for_engine_operation(
                &pipeline_group_id,
                pipeline_id.as_ref(),
                timeout_secs,
                engine_operation_id,
            )?;
            let failed = delete.state != "succeeded";
            pipelines.push(delete);
            if failed {
                let failure_reason = pipelines
                    .last()
                    .and_then(|status| status.failure_reason.clone())
                    .or_else(|| Some("pipeline deletion failed".to_owned()));
                return Ok(GroupDeleteStatus {
                    pipeline_group_id,
                    state: "failed".to_owned(),
                    started_at,
                    updated_at: timestamp_now(),
                    pipelines,
                    failure_reason,
                });
            }
        }

        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !Self::engine_operation_allows(&state, engine_operation_id) {
                return Err(ControlPlaneError::RolloutConflict);
            }
            let Some(group) = state.live_config.groups.get(&pipeline_group_id) else {
                return Err(ControlPlaneError::GroupNotFound);
            };
            if !group.pipelines.is_empty() {
                return Err(ControlPlaneError::InvalidRequest {
                    message: format!(
                        "group `{}` still contains pipelines after delete",
                        pipeline_group_id.as_ref()
                    ),
                });
            }
            let _ = state.live_config.groups.remove(&pipeline_group_id);
        }

        Ok(GroupDeleteStatus {
            pipeline_group_id,
            state: "succeeded".to_owned(),
            started_at,
            updated_at: timestamp_now(),
            pipelines,
            failure_reason: None,
        })
    }

    /// Reconciles live controller state to a complete desired engine config.
    pub(super) fn reconcile_engine_config(
        self: &Arc<Self>,
        request: EngineConfigReconcileRequest,
    ) -> Result<EngineConfigReconcileStatus, ControlPlaneError> {
        let (reconcile_id, guard) = self.begin_reconcile_operation()?;
        let started_at = timestamp_now();
        let mut status = EngineConfigReconcileStatus::new(
            reconcile_id,
            EngineConfigReconcileState::Running,
            None,
            started_at,
        );

        let live_config = self.engine_config_snapshot();
        let desired_config = request.config;
        desired_config
            .validate()
            .map_err(|err| ControlPlaneError::InvalidRequest {
                message: err.to_string(),
            })?;
        startup::validate_engine_components(&desired_config, self.pipeline_factory).map_err(
            |error| ControlPlaneError::InvalidRequest {
                message: error.to_string(),
            },
        )?;

        let current_profiles = Self::pipeline_topic_profiles(&live_config)?;
        let desired_profiles = Self::pipeline_topic_profiles(&desired_config)?;
        if current_profiles != desired_profiles {
            return Err(ControlPlaneError::InvalidRequest {
                message: "desired config would require runtime topic broker mutation".to_owned(),
            });
        }
        Self::validate_live_memory_limiter_unchanged(&live_config, &desired_config)?;

        let mut desired_keys = Vec::new();
        for (pipeline_group_id, group) in &desired_config.groups {
            for (pipeline_id, pipeline) in &group.pipelines {
                desired_keys.push((
                    PipelineKey::new(pipeline_group_id.clone(), pipeline_id.clone()),
                    pipeline.clone(),
                ));
            }
        }
        desired_keys.sort_by(|(left, _), (right, _)| {
            left.pipeline_group_id()
                .as_ref()
                .cmp(right.pipeline_group_id().as_ref())
                .then_with(|| {
                    left.pipeline_id()
                        .as_ref()
                        .cmp(right.pipeline_id().as_ref())
                })
        });
        let desired_key_set: HashSet<_> = desired_keys
            .iter()
            .map(|(pipeline_key, _)| pipeline_key.clone())
            .collect();

        for (pipeline_key, pipeline) in desired_keys {
            let request = ReconfigureRequest {
                pipeline,
                step_timeout_secs: request.step_timeout_secs,
                drain_timeout_secs: request.drain_timeout_secs,
            };
            let plan = self.prepare_rollout_plan_for_engine_operation(
                pipeline_key.pipeline_group_id(),
                pipeline_key.pipeline_id(),
                &request,
                Some(&desired_config),
                Some(guard.operation_id()),
            )?;
            let action = Self::rollout_change_action(plan.action);
            let initial_rollout =
                self.spawn_rollout_for_engine_operation(plan, Some(guard.operation_id()))?;
            let terminal_rollout = self.wait_for_rollout_terminal(initial_rollout);
            let succeeded = Self::rollout_succeeded(&terminal_rollout);
            let failure_reason = terminal_rollout.failure_reason.clone();
            status.changes.push(ConfigChangeStatus {
                pipeline_group_id: Some(pipeline_key.pipeline_group_id().clone()),
                pipeline_id: Some(pipeline_key.pipeline_id().clone()),
                action,
                state: if succeeded {
                    "succeeded".to_owned()
                } else {
                    "failed".to_owned()
                },
                rollout: Some(terminal_rollout),
                shutdown: None,
                detail: failure_reason.clone(),
            });
            status.updated_at = timestamp_now();
            if !succeeded {
                status.state = EngineConfigReconcileState::Failed;
                status.failure_reason = failure_reason
                    .or_else(|| Some("pipeline rollout did not complete successfully".to_owned()));
                return Ok(status);
            }
        }

        if request.delete_missing {
            for pipeline_key in self.live_pipeline_keys() {
                if desired_key_set.contains(&pipeline_key) {
                    continue;
                }
                let delete = self.request_delete_pipeline_for_engine_operation(
                    pipeline_key.pipeline_group_id(),
                    pipeline_key.pipeline_id(),
                    request.delete_timeout_secs,
                    Some(guard.operation_id()),
                )?;
                let succeeded = delete.state == "succeeded";
                let failure_reason = delete.failure_reason.clone();
                status.changes.push(ConfigChangeStatus {
                    pipeline_group_id: Some(pipeline_key.pipeline_group_id().clone()),
                    pipeline_id: Some(pipeline_key.pipeline_id().clone()),
                    action: ConfigChangeAction::Delete,
                    state: delete.state.clone(),
                    rollout: None,
                    shutdown: delete.shutdown.clone(),
                    detail: failure_reason.clone(),
                });
                status.updated_at = timestamp_now();
                if !succeeded {
                    status.state = EngineConfigReconcileState::Failed;
                    status.failure_reason =
                        failure_reason.or_else(|| Some("pipeline deletion failed".to_owned()));
                    return Ok(status);
                }
            }

            let desired_group_ids: HashSet<_> = desired_config.groups.keys().cloned().collect();
            for pipeline_group_id in self.live_group_ids() {
                if desired_group_ids.contains(&pipeline_group_id) {
                    continue;
                }
                let delete = self.request_delete_group_for_engine_operation(
                    &pipeline_group_id,
                    request.delete_timeout_secs,
                    Some(guard.operation_id()),
                )?;
                let succeeded = delete.state == "succeeded";
                let failure_reason = delete.failure_reason.clone();
                status.changes.push(ConfigChangeStatus {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: None,
                    action: ConfigChangeAction::Delete,
                    state: delete.state,
                    rollout: None,
                    shutdown: None,
                    detail: failure_reason.clone(),
                });
                status.updated_at = timestamp_now();
                if !succeeded {
                    status.state = EngineConfigReconcileState::Failed;
                    status.failure_reason =
                        failure_reason.or_else(|| Some("group deletion failed".to_owned()));
                    return Ok(status);
                }
            }
        }

        self.apply_reconcile_success(&desired_config, request.delete_missing);
        status.state = EngineConfigReconcileState::Succeeded;
        status.updated_at = timestamp_now();
        Ok(status)
    }

    /// Records a rollout and launches its background execution worker.
    pub(super) fn spawn_rollout(
        self: &Arc<Self>,
        plan: CandidateRolloutPlan,
    ) -> Result<RolloutStatus, ControlPlaneError> {
        self.spawn_rollout_for_engine_operation(plan, None)
    }

    fn spawn_rollout_for_engine_operation(
        self: &Arc<Self>,
        plan: CandidateRolloutPlan,
        engine_operation_id: Option<&str>,
    ) -> Result<RolloutStatus, ControlPlaneError> {
        let rollout_id = plan.rollout.rollout_id.clone();
        let pipeline_key = plan.pipeline_key.clone();
        self.insert_rollout_for_engine_operation(
            &pipeline_key,
            plan.rollout.clone(),
            engine_operation_id,
        )?;
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

    pub(super) fn spawn_shutdown_for_engine_operation(
        self: &Arc<Self>,
        plan: CandidateShutdownPlan,
        engine_operation_id: Option<&str>,
    ) -> Result<ShutdownStatus, ControlPlaneError> {
        let shutdown_id = plan.shutdown.shutdown_id.clone();
        let pipeline_key = plan.pipeline_key.clone();
        let initial_status = plan.shutdown.status();
        self.insert_shutdown_for_engine_operation(
            &pipeline_key,
            plan.shutdown.clone(),
            engine_operation_id,
        )?;
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
}
