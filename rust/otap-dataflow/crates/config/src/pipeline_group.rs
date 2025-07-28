// SPDX-License-Identifier: Apache-2.0

//! The configuration for a pipeline group.

use crate::error::Error;
use crate::pipeline::PipelineConfig;
use crate::{PipelineGroupId, PipelineId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single pipeline group.
/// Contains group-specific settings and all its pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineGroupConfig {
    /// All pipelines belonging to this pipeline group, keyed by pipeline ID.
    pub pipelines: HashMap<PipelineId, PipelineConfig>,

    /// Quota for the pipeline group.
    #[serde(default)]
    pub quota: Quota,
}

impl PipelineGroupConfig {
    /// Creates a new empty pipeline group configuration.
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            quota: Default::default(),
        }
    }
    
    /// Sets the quota for the pipeline group.
    pub fn set_quota(&mut self, quota: Quota) {
        self.quota = quota;
    }
    
    /// Adds a pipeline to the pipeline group.
    pub fn add_pipeline(&mut self, pipeline_id: PipelineId, pipeline: PipelineConfig) -> Result<(), Error> {
        let prev_pipeline_grp = self.pipelines.insert(pipeline_id.clone(), pipeline);
        if prev_pipeline_grp.is_some() {
            return Err(Error::DuplicatePipeline { pipeline_id });
        }
        Ok(())
    }
    
    /// Validates the pipeline group configuration.
    pub fn validate(&self, pipeline_group_id: &PipelineGroupId) -> Result<(), Error> {
        let mut errors = Vec::new();

        for (pipeline_id, pipeline) in &self.pipelines {
            if let Err(e) = pipeline.validate(pipeline_group_id, pipeline_id) {
                errors.push(e);
            }
        }
        if !errors.is_empty() {
            Err(Error::InvalidConfiguration { errors })
        } else {
            Ok(())
        }
    }
}

/// Pipeline group quota configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct Quota {
    /// Number of CPU cores to use for this pipeline group.
    /// If set to 0, it will use all available cores.
    /// If set to a value greater than 0, it will use that many cores.
    #[serde(default = "default_num_cores")]
    pub num_cores: usize,
}

fn default_num_cores() -> usize {
    0 // Default to using all available cores
}
