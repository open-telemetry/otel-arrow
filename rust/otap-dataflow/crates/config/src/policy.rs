// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine and pipeline policy declarations.

use crate::health::HealthPolicy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Top-level policy set.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Policies {
    /// Flow-related policies.
    #[serde(default)]
    pub flow: FlowPolicy,
    /// Health policy used by observed-state liveness/readiness evaluation.
    #[serde(default)]
    pub health: HealthPolicy,
    /// Runtime telemetry policy controlling pipeline-local metric collection.
    #[serde(default)]
    pub telemetry: TelemetryPolicy,
    /// Resources policy controlling runtime core allocation.
    #[serde(default)]
    pub resources: ResourcesPolicy,
}

impl Policies {
    /// Returns validation errors for this policy set.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        let mut errors = Vec::new();
        let channel_capacity = &self.flow.channel_capacity;
        if channel_capacity.control.node == 0 {
            errors.push(format!(
                "{path_prefix}.flow.channel_capacity.control.node must be greater than 0"
            ));
        }
        if channel_capacity.control.pipeline == 0 {
            errors.push(format!(
                "{path_prefix}.flow.channel_capacity.control.pipeline must be greater than 0"
            ));
        }
        if channel_capacity.pdata == 0 {
            errors.push(format!(
                "{path_prefix}.flow.channel_capacity.pdata must be greater than 0"
            ));
        }
        errors
    }
}

/// Runtime telemetry policy declarations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TelemetryPolicy {
    /// Enable capture of per-pipeline internal metrics.
    #[serde(default = "default_true")]
    pub pipeline_metrics: bool,
    /// Enable capture of Tokio runtime internal metrics.
    #[serde(default = "default_true")]
    pub tokio_metrics: bool,
    /// Enable capture of channel-level metrics.
    #[serde(default = "default_true")]
    pub channel_metrics: bool,
}

impl Default for TelemetryPolicy {
    fn default() -> Self {
        Self {
            pipeline_metrics: true,
            tokio_metrics: true,
            channel_metrics: true,
        }
    }
}

const fn default_true() -> bool {
    true
}

/// Resource-related policy declarations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ResourcesPolicy {
    /// CPU core allocation strategy for this pipeline.
    #[serde(default)]
    pub core_allocation: CoreAllocation,
}

/// Defines how CPU cores should be allocated for pipeline execution.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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

/// Flow-related policy declarations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct FlowPolicy {
    /// Channel capacity policy.
    #[serde(default)]
    pub channel_capacity: ChannelCapacityPolicy,
}

/// Channel capacities used by control and pdata channels.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChannelCapacityPolicy {
    /// Capacities for control channels.
    #[serde(default)]
    pub control: ControlChannelCapacityPolicy,
    /// Capacity for pdata channels.
    #[serde(default = "default_pdata_channel_capacity")]
    pub pdata: usize,
}

impl Default for ChannelCapacityPolicy {
    fn default() -> Self {
        Self {
            control: ControlChannelCapacityPolicy::default(),
            pdata: default_pdata_channel_capacity(),
        }
    }
}

/// Control channel capacities.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ControlChannelCapacityPolicy {
    /// Capacity used for node control channels.
    #[serde(default = "default_node_control_channel_capacity")]
    pub node: usize,
    /// Capacity used for pipeline control channels.
    #[serde(default = "default_pipeline_control_channel_capacity")]
    pub pipeline: usize,
}

impl Default for ControlChannelCapacityPolicy {
    fn default() -> Self {
        Self {
            node: default_node_control_channel_capacity(),
            pipeline: default_pipeline_control_channel_capacity(),
        }
    }
}

const fn default_node_control_channel_capacity() -> usize {
    256
}

const fn default_pipeline_control_channel_capacity() -> usize {
    256
}

const fn default_pdata_channel_capacity() -> usize {
    128
}

#[cfg(test)]
mod tests {
    use super::Policies;

    #[test]
    fn defaults_match_expected_values() {
        let policies = Policies::default();
        assert_eq!(policies.flow.channel_capacity.control.node, 256);
        assert_eq!(policies.flow.channel_capacity.control.pipeline, 256);
        assert_eq!(policies.flow.channel_capacity.pdata, 128);
        assert!(policies.telemetry.pipeline_metrics);
        assert!(policies.telemetry.tokio_metrics);
        assert!(policies.telemetry.channel_metrics);
        assert_eq!(
            policies.resources.core_allocation,
            super::CoreAllocation::AllCores
        );
    }

    #[test]
    fn validates_non_zero_capacities() {
        let mut policies = Policies::default();
        policies.flow.channel_capacity.control.node = 0;
        policies.flow.channel_capacity.control.pipeline = 0;
        policies.flow.channel_capacity.pdata = 0;

        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 3);
        assert!(errors.iter().any(|e| e.contains("control.node")));
        assert!(errors.iter().any(|e| e.contains("control.pipeline")));
        assert!(errors.iter().any(|e| e.contains(".pdata")));
    }

    #[test]
    fn core_allocation_display_all_cores() {
        assert_eq!(super::CoreAllocation::AllCores.to_string(), "*");
    }

    #[test]
    fn core_allocation_display_core_count() {
        assert_eq!(
            super::CoreAllocation::CoreCount { count: 4 }.to_string(),
            "[4 cores]"
        );
    }

    #[test]
    fn core_allocation_display_core_set_single_range() {
        assert_eq!(
            super::CoreAllocation::CoreSet {
                set: vec![super::CoreRange { start: 0, end: 3 }]
            }
            .to_string(),
            "0-3"
        );
    }

    #[test]
    fn core_allocation_display_core_set_multiple_ranges() {
        assert_eq!(
            super::CoreAllocation::CoreSet {
                set: vec![
                    super::CoreRange { start: 0, end: 3 },
                    super::CoreRange { start: 8, end: 11 },
                    super::CoreRange { start: 16, end: 16 },
                ]
            }
            .to_string(),
            "0-3,8-11,16"
        );
    }
}
