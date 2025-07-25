// SPDX-License-Identifier: Apache-2.0

//! The configuration for a pipeline group.

use crate::error::Error;
use crate::pipeline::PipelineConfig;
use crate::{Description, PipelineGroupId, PipelineId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single pipeline group.
/// Contains group-specific settings and all its pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineGroupConfig {
    /// An optional description of the pipeline group.
    pub description: Option<Description>,

    /// Settings that apply to this pipeline group only.
    pub settings: PipelineGroupSettings,

    /// All pipelines belonging to this pipeline group, keyed by pipeline ID.
    pub pipelines: HashMap<PipelineId, PipelineConfig>,
    // ToDo: Add resource quota support.
}

/// Pipeline group-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineGroupSettings {
    /// If false, the pipeline group is disabled and cannot process data.
    pub enabled: bool,
}

impl PipelineGroupConfig {
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
