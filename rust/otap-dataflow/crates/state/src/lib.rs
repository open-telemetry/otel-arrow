// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! State stores

use otap_df_config::{PipelineGroupId, PipelineId};
use serde::{Serialize, Serializer};

mod config;
pub mod error;
pub mod reporter;
pub mod store;

type CoreId = usize;

/// Unique key for identifying a pipeline within a pipeline group.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PipelineKey {
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
}

impl PipelineKey {
    /// Construct a new PipelineKey from group and pipeline ids.
    #[must_use]
    pub fn new(pipeline_group_id: PipelineGroupId, pipeline_id: PipelineId) -> Self {
        Self {
            pipeline_group_id,
            pipeline_id,
        }
    }
}

impl Serialize for PipelineKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}:{}", self.pipeline_group_id, self.pipeline_id);
        serializer.serialize_str(&s)
    }
}

/// Unique key for identifying a pipeline running on a specific core.
#[derive(Debug, Clone)]
pub struct DeployedPipelineKey {
    /// The unique ID of the pipeline group the pipeline belongs to.
    pub pipeline_group_id: PipelineGroupId,

    /// The unique ID of the pipeline within its group.
    pub pipeline_id: PipelineId,

    /// The CPU core ID the pipeline is pinned to.
    /// Note: Not using core_affinity::CoreId directly to avoid dependency leakage in this public API
    pub core_id: usize,
}
