// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The configuration for a pipeline group.

use crate::error::Error;
use crate::pipeline::PipelineConfig;
use crate::policy::Policies;
use crate::{PipelineGroupId, PipelineId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single pipeline group.
/// Contains group-specific policies and all its pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineGroupConfig {
    /// Optional policy set for this pipeline group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policies: Option<Policies>,

    /// All pipelines belonging to this pipeline group, keyed by pipeline ID.
    pub pipelines: HashMap<PipelineId, PipelineConfig>,
}

impl PipelineGroupConfig {
    /// Creates a new empty pipeline group configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            policies: None,
            pipelines: HashMap::new(),
        }
    }

    /// Adds a pipeline to the pipeline group.
    pub fn add_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        pipeline: PipelineConfig,
    ) -> Result<(), Error> {
        if self.pipelines.contains_key(&pipeline_id) {
            return Err(Error::DuplicatePipeline { pipeline_id });
        }
        _ = self.pipelines.insert(pipeline_id.clone(), pipeline);
        Ok(())
    }

    /// Validates the pipeline group configuration.
    pub fn validate(&self, pipeline_group_id: &PipelineGroupId) -> Result<(), Error> {
        let mut errors = Vec::new();

        if let Some(policies) = &self.policies {
            let path = format!("groups.{pipeline_group_id}.policies");
            errors.extend(
                policies
                    .validation_errors(&path)
                    .into_iter()
                    .map(|error| Error::InvalidUserConfig { error }),
            );
        }

        for (pipeline_id, pipeline) in &self.pipelines {
            if let Some(policies) = pipeline.policies() {
                let path = format!("groups.{pipeline_group_id}.pipelines.{pipeline_id}.policies");
                errors.extend(
                    policies
                        .validation_errors(&path)
                        .into_iter()
                        .map(|error| Error::InvalidUserConfig { error }),
                );
            }
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

impl Default for PipelineGroupConfig {
    fn default() -> Self {
        Self::new()
    }
}
