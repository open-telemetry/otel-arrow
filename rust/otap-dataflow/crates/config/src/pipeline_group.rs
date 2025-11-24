// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The configuration for a pipeline group.

use crate::error::Error;
use crate::pipeline::PipelineConfig;
use crate::{PipelineGroupId, PipelineId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

/// Configuration for a single pipeline group.
/// Contains group-specific settings and all its pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineGroupConfig {
    /// All pipelines belonging to this pipeline group, keyed by pipeline ID.
    pub pipelines: HashMap<PipelineId, PipelineConfig>,

    /// Quota for the pipeline group.
    #[serde(default)]
    pub quota: Quota,
}

impl PipelineGroupConfig {
    /// Creates a new empty pipeline group configuration.
    #[must_use]
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

impl Default for PipelineGroupConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline group quota configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct Quota {
    /// CPU core allocation strategy for this pipeline group.
    #[serde(default)]
    pub core_allocation: CoreAllocation,
}

/// Defines how CPU cores should be allocated for pipeline execution.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoreAllocation {
    /// Use all available CPU cores.
    #[default]
    AllCores,
    /// Use a specific number of CPU cores (starting from core 0).
    /// If the requested number exceeds available cores, use all available cores.
    CoreCount {
        /// Number of cores to use. If 0, uses all available cores.
        count: usize,
    },
    /// Defines a set of CPU cores should be allocated for pipeline execution.
    CoreSet {
        /// Core set defined as a set of ranges.
        set: Vec<CoreRange>,
    },
}

impl Display for CoreAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreAllocation::AllCores => write!(f, "*"),
            CoreAllocation::CoreCount { count } => write!(f, "[{count} cores]"),
            CoreAllocation::CoreSet { set } => {
                let mut first = true;
                for item in set {
                    if !first {
                        write!(f, ",")?
                    }
                    write!(f, "{item}")?;
                    first = false
                }
                Ok(())
            }
        }
    }
}

/// Defines a range of CPU cores should be allocated for pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct CoreRange {
    /// Start core ID (inclusive).
    pub start: usize,
    /// End core ID (inclusive).
    pub end: usize,
}

impl Display for CoreRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_allocation_display_all_cores() {
        let allocation = CoreAllocation::AllCores;
        assert_eq!(allocation.to_string(), "*");
    }

    #[test]
    fn test_core_allocation_display_core_count() {
        let allocation = CoreAllocation::CoreCount { count: 4 };
        assert_eq!(allocation.to_string(), "[4 cores]");
    }

    #[test]
    fn test_core_allocation_display_core_set_single_range() {
        let allocation = CoreAllocation::CoreSet {
            set: vec![CoreRange { start: 0, end: 3 }],
        };
        assert_eq!(allocation.to_string(), "0-3");
    }

    #[test]
    fn test_core_allocation_display_core_set_multiple_ranges() {
        let allocation = CoreAllocation::CoreSet {
            set: vec![
                CoreRange { start: 0, end: 3 },
                CoreRange { start: 8, end: 11 },
                CoreRange { start: 16, end: 16 },
            ],
        };
        assert_eq!(allocation.to_string(), "0-3,8-11,16");
    }
}
