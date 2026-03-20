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
    /// Channel capacity policy.
    ///
    /// When absent, the parent scope's channel capacity policy or the built-in
    /// default applies.  Serde leaves this `None` when the key is omitted from
    /// the YAML/JSON, so a `policies:` block that only sets (e.g.) `telemetry`
    /// does **not** implicitly reset channel capacities and shadow a top-level
    /// override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) channel_capacity: Option<ChannelCapacityPolicy>,
    /// Health policy used by observed-state liveness/readiness evaluation.
    ///
    /// When absent, the parent scope's health policy or the built-in default
    /// applies.  Serde leaves this `None` when the key is omitted from the
    /// YAML/JSON, so a `policies:` block that only sets (e.g.) `telemetry`
    /// does **not** implicitly reset health criteria and shadow a top-level
    /// override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) health: Option<HealthPolicy>,
    /// Runtime telemetry policy controlling pipeline-local metric collection.
    ///
    /// When absent, the parent scope's telemetry policy or the built-in default
    /// applies.  Serde leaves this `None` when the key is omitted from the
    /// YAML/JSON, so a `policies:` block that only sets (e.g.)
    /// `channel_capacity` does **not** implicitly reset `channel_metrics`
    /// to `Basic` and shadow a top-level override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) telemetry: Option<TelemetryPolicy>,
    /// Resources policy controlling runtime core allocation.
    ///
    /// When absent, the parent scope's resources policy or the built-in default
    /// (`core_allocation: all_cores`) applies.  Serde leaves this `None` when
    /// the key is omitted from the YAML/JSON, so a `policies:` block that only
    /// sets (e.g.) `channel_capacity` does **not** implicitly pin `core_allocation`
    /// to `AllCores` and shadow a `--num-cores` / `--core-id-range` CLI flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resources: Option<ResourcesPolicy>,
}

impl Policies {
    /// Sets the resources policy (used by CLI overrides).
    pub fn set_resources(&mut self, resources: ResourcesPolicy) {
        self.resources = Some(resources);
    }

    /// Resolves a fully-populated policy set from scopes in precedence
    /// order (most specific first).  For each field the first `Some`
    /// value wins; fields absent at every level use built-in defaults.
    #[must_use]
    pub fn resolve<'a>(scopes: impl IntoIterator<Item = &'a Policies>) -> ResolvedPolicies {
        let mut channel_capacity = None;
        let mut health = None;
        let mut telemetry = None;
        let mut resources = None;
        for scope in scopes {
            if channel_capacity.is_none() {
                channel_capacity = scope.channel_capacity.as_ref();
            }
            if health.is_none() {
                health = scope.health.as_ref();
            }
            if telemetry.is_none() {
                telemetry = scope.telemetry.as_ref();
            }
            if resources.is_none() {
                resources = scope.resources.as_ref();
            }
        }
        ResolvedPolicies {
            channel_capacity: channel_capacity.cloned().unwrap_or_default(),
            health: health.cloned().unwrap_or_default(),
            telemetry: telemetry.cloned().unwrap_or_default(),
            resources: resources.cloned().unwrap_or_default(),
        }
    }

    /// Returns validation errors for explicitly configured fields.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(channel_capacity) = &self.channel_capacity {
            if channel_capacity.control.node == 0 {
                errors.push(format!(
                    "{path_prefix}.channel_capacity.control.node must be greater than 0"
                ));
            }
            if channel_capacity.control.pipeline == 0 {
                errors.push(format!(
                    "{path_prefix}.channel_capacity.control.pipeline must be greater than 0"
                ));
            }
            if channel_capacity.pdata == 0 {
                errors.push(format!(
                    "{path_prefix}.channel_capacity.pdata must be greater than 0"
                ));
            }
        }
        errors
    }
}

/// Fully-resolved policy snapshot where every field is populated.
///
/// Produced by [`Policies::resolve`] after walking the scope hierarchy
/// (pipeline → group → top-level → built-in defaults).  All `Option`
/// fields from [`Policies`] are collapsed to concrete values, so
/// consumers can access fields directly without fallback logic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedPolicies {
    /// Channel capacity policy.
    pub channel_capacity: ChannelCapacityPolicy,
    /// Health policy.
    pub health: HealthPolicy,
    /// Runtime telemetry policy.
    pub telemetry: TelemetryPolicy,
    /// Resources policy.
    pub resources: ResourcesPolicy,
}

/// Engine-wide metric level controlling per-channel and per-node
/// instrumentation overhead.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MetricLevel {
    /// No instrumentation.
    #[default]
    None,
    /// Channel transport metrics (send/recv counts, capacity).
    Basic,
    /// Adds per-node produced/consumed outcome metrics
    /// (success, failure, refused).
    Normal,
    /// Adds pipeline latency measurement (entry timestamps).
    Detailed,
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
    /// Channel and component metric detail level.
    #[serde(default = "default_metric_level_basic")]
    pub channel_metrics: MetricLevel,
}

impl Default for TelemetryPolicy {
    fn default() -> Self {
        Self {
            pipeline_metrics: true,
            tokio_metrics: true,
            channel_metrics: MetricLevel::Basic,
        }
    }
}

const fn default_metric_level_basic() -> MetricLevel {
    MetricLevel::Basic
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
        let defaults = Policies::resolve([&Policies::default()]);
        assert_eq!(defaults.channel_capacity.control.node, 256);
        assert_eq!(defaults.channel_capacity.control.pipeline, 256);
        assert_eq!(defaults.channel_capacity.pdata, 128);
        assert!(defaults.telemetry.pipeline_metrics);
        assert!(defaults.telemetry.tokio_metrics);
        assert_eq!(
            defaults.telemetry.channel_metrics,
            super::MetricLevel::Basic
        );
        assert_eq!(
            defaults.resources.core_allocation,
            super::CoreAllocation::AllCores
        );
        assert_eq!(defaults.health, crate::health::HealthPolicy::default());
    }

    #[test]
    fn validates_non_zero_capacities() {
        let policies = Policies {
            channel_capacity: Some(super::ChannelCapacityPolicy {
                control: super::ControlChannelCapacityPolicy {
                    node: 0,
                    pipeline: 0,
                },
                pdata: 0,
            }),
            ..Default::default()
        };

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

    #[test]
    fn metric_level_ordering() {
        use super::MetricLevel;
        assert!(MetricLevel::None < MetricLevel::Basic);
        assert!(MetricLevel::Basic < MetricLevel::Normal);
        assert!(MetricLevel::Normal < MetricLevel::Detailed);
        assert!(MetricLevel::Detailed >= MetricLevel::Basic);
    }

    #[test]
    fn metric_level_serde_roundtrip() {
        use super::MetricLevel;
        for (level, expected_str) in [
            (MetricLevel::None, "\"none\""),
            (MetricLevel::Basic, "\"basic\""),
            (MetricLevel::Normal, "\"normal\""),
            (MetricLevel::Detailed, "\"detailed\""),
        ] {
            let json = serde_json::to_string(&level).expect("serialize");
            assert_eq!(json, expected_str);
            let back: MetricLevel = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, level);
        }
    }

    #[test]
    fn telemetry_policy_with_channel_metrics_level() {
        let yaml = r#"
            pipeline_metrics: true
            tokio_metrics: false
            channel_metrics: detailed
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.channel_metrics, super::MetricLevel::Detailed);
        assert!(!policy.tokio_metrics);
    }

    #[test]
    fn telemetry_policy_defaults_channel_metrics_to_basic() {
        let yaml = r#"
            pipeline_metrics: true
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.channel_metrics, super::MetricLevel::Basic);
    }
}
