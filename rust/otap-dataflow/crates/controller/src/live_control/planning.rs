use super::*;

impl<PData: 'static + Clone + Send + Sync + std::fmt::Debug + ReceivedAtNode + Unwindable>
    ControllerRuntime<PData>
{
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

    /// Compares two resolved pipelines while ignoring resource-policy differences.
    pub(super) fn runtime_shape_matches_ignoring_resources(
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
    pub(super) fn resolved_pipeline_matches(
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

    /// Classifies a reconfigure request and prepares the rollout state machine inputs.
    pub(super) fn prepare_rollout_plan(
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
    pub(super) fn insert_rollout(
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
        self.observed_state_store.set_pipeline_active_cores(
            plan.pipeline_key.clone(),
            plan.target_assigned_cores.iter().copied(),
        );
        self.observed_state_store
            .set_pipeline_active_generation(plan.pipeline_key.clone(), active_generation);
    }

    /// Selects the active instances targeted by a per-pipeline shutdown request.
    pub(super) fn prepare_shutdown_plan(
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
    pub(super) fn insert_shutdown(
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

    /// Records a rollout and launches its background execution worker.
    pub(super) fn spawn_rollout(
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
    pub(super) fn spawn_shutdown(
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
}
