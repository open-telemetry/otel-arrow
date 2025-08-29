// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of structs describing the observed state of the controller.

use otap_df_config::{PipelineGroupId, PipelineId};
use serde::Serialize;
use slotmap::{SlotMap, new_key_type};

new_key_type! {
    /// Unique key for identifying pipeline groups.
    pub struct PipelineGroupKey;
}

new_key_type! {
    /// Unique key for identifying pipelineS.
    pub struct PipelineKey;
}

/// Observed state of the controller.
#[derive(Default, Serialize)]
pub struct ControllerState {
    pipeline_groups: SlotMap<PipelineGroupKey, PipelineGroupState>,
}

/// Observed state of a pipeline group.
#[derive(Serialize)]
pub struct PipelineGroupState {
    pipeline_group_id: PipelineGroupId,
    pipelines: SlotMap<PipelineKey, PipelineState>,
}

/// Observed state of a pipeline.
#[derive(Serialize)]
pub struct PipelineState {
    pipeline_id: PipelineId,
    core_id: usize,
    thread_id: usize,

    status: PipelineStatus,
}

/// Status of a pipeline.
#[derive(Serialize)]
pub enum PipelineStatus {
    /// The pipeline is currently running.
    Running,
    /// The pipeline is in the process of stopping.
    Stopping,
    /// The pipeline has been stopped.
    Stopped,
    /// The pipeline has encountered an error.
    Error(String),
}

impl ControllerState {
    /// Records a new pipeline group to the observed state.
    pub fn record_pipeline_group(&mut self, group_id: PipelineGroupId) -> PipelineGroupKey {
        self.pipeline_groups.insert(PipelineGroupState::new(group_id))
    }

    /// Records a pipeline under the specified pipeline group.
    pub fn record_pipeline(
        &mut self,
        group_key: PipelineGroupKey,
        pipeline_id: PipelineId,
        core_id: usize,
        thread_id: usize,
    ) -> Option<PipelineKey> {
        self.pipeline_groups
            .get_mut(group_key)
            .map(|group| group.add_pipeline(pipeline_id, core_id, thread_id))
    }
}

impl PipelineGroupState {
    fn new(group_id: PipelineGroupId) -> Self {
        Self {
            pipeline_group_id: group_id,
            pipelines: Default::default(),
        }
    }

    /// Adds a new pipeline to the pipeline group.
    fn add_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        core_id: usize,
        thread_id: usize,
    ) -> PipelineKey {
        self.pipelines.insert(PipelineState {
            pipeline_id,
            core_id,
            thread_id,
            status: PipelineStatus::Running,
        })
    }
}
