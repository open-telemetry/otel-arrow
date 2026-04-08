// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine and pipeline policy declarations.

use crate::byte_units;
use crate::health::HealthPolicy;
use crate::transport_headers_policy::TransportHeadersPolicy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::Duration;

/// Top-level policy set.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Policies {
    /// Channel capacity policy.
    ///
    /// When absent, a parent scope's channel capacity policy or the built-in
    /// default applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) channel_capacity: Option<ChannelCapacityPolicy>,
    /// Health policy used by observed-state liveness/readiness evaluation.
    ///
    /// When absent, a parent scope's health policy or the built-in default
    /// applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) health: Option<HealthPolicy>,
    /// Runtime telemetry policy controlling pipeline-local metric collection.
    ///
    /// When absent, a parent scope's telemetry policy or the built-in default
    /// applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) telemetry: Option<TelemetryPolicy>,
    /// Resources policy controlling runtime core allocation.
    ///
    /// When absent, a parent scope's resources policy or the built-in default
    /// applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resources: Option<ResourcesPolicy>,
    /// Transport headers policy controlling header capture at receivers
    /// and propagation at exporters.
    ///
    /// When absent, transport headers are not captured or propagated
    /// (the feature is entirely opt-in).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) transport_headers: Option<TransportHeadersPolicy>,
}

impl Policies {
    /// Override the resources policy.
    pub fn set_resources(&mut self, resources: ResourcesPolicy) {
        self.resources = Some(resources);
    }

    /// Returns the explicitly configured resources policy, if any.
    #[must_use]
    pub fn resources(&self) -> Option<&ResourcesPolicy> {
        self.resources.as_ref()
    }

    /// Resolves a fully-populated policy set from scopes ordered by precedence.
    #[must_use]
    pub fn resolve<'a>(scopes: impl IntoIterator<Item = &'a Policies>) -> ResolvedPolicies {
        let mut channel_capacity = None;
        let mut health = None;
        let mut telemetry = None;
        let mut resources = None;
        let mut transport_headers = None;
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
            if transport_headers.is_none() {
                transport_headers = scope.transport_headers.as_ref();
            }
        }
        ResolvedPolicies {
            channel_capacity: channel_capacity.cloned().unwrap_or_default(),
            health: health.cloned().unwrap_or_default(),
            telemetry: telemetry.cloned().unwrap_or_default(),
            resources: resources.cloned().unwrap_or_default(),
            transport_headers: transport_headers.cloned(),
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
            if channel_capacity.control.completion == 0 {
                errors.push(format!(
                    "{path_prefix}.channel_capacity.control.completion must be greater than 0"
                ));
            }
            if channel_capacity.pdata == 0 {
                errors.push(format!(
                    "{path_prefix}.channel_capacity.pdata must be greater than 0"
                ));
            }
        }
        if let Some(memory_limiter) = self
            .resources
            .as_ref()
            .and_then(|resources| resources.memory_limiter.as_ref())
        {
            let limiter_path = format!("{path_prefix}.resources.memory_limiter");
            if memory_limiter.check_interval < Duration::from_millis(100) {
                errors.push(format!(
                    "{limiter_path}.check_interval must be at least 100ms"
                ));
            }
            if memory_limiter.retry_after_secs == 0 {
                errors.push(format!(
                    "{limiter_path}.retry_after_secs must be greater than 0"
                ));
            }
            if memory_limiter.purge_on_hard && memory_limiter.purge_min_interval.is_zero() {
                errors.push(format!(
                    "{limiter_path}.purge_min_interval must be greater than 0"
                ));
            }
            match (memory_limiter.soft_limit, memory_limiter.hard_limit) {
                (Some(soft_limit), Some(hard_limit)) => {
                    if soft_limit == 0 {
                        errors.push(format!(
                            "{limiter_path}.soft_limit must be greater than 0"
                        ));
                    }
                    if hard_limit <= soft_limit {
                        errors.push(format!(
                            "{limiter_path}.hard_limit must be greater than {limiter_path}.soft_limit"
                        ));
                    }
                    if let Some(hysteresis) = memory_limiter.hysteresis
                        && hysteresis >= soft_limit
                    {
                        errors.push(format!(
                            "{limiter_path}.hysteresis must be smaller than {limiter_path}.soft_limit"
                        ));
                    }
                }
                (None, None) => {
                    if memory_limiter.source != MemoryLimiterSource::Auto {
                        errors.push(format!(
                            "{limiter_path}.soft_limit and {limiter_path}.hard_limit must be set when {limiter_path}.source is not auto"
                        ));
                    }
                }
                _ => errors.push(format!(
                    "{limiter_path}.soft_limit and {limiter_path}.hard_limit must either both be set or both be omitted"
                )),
            }
        }
        errors
    }
}

/// Engine-wide metric level controlling channel, node, and shared control-plane
/// Fully-resolved policy snapshot where every field is populated.
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
    /// Transport headers policy. `None` when the feature is not configured
    /// (opt-in only -- no headers are captured or propagated by default).
    pub transport_headers: Option<TransportHeadersPolicy>,
}
/// instrumentation overhead.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MetricLevel {
    /// No instrumentation.
    #[default]
    None,
    /// Channel transport metrics plus shared control-plane state gauges.
    Basic,
    /// Adds per-node produced/consumed outcome metrics (success, failure,
    /// refused) and shared control-plane message/phase counters.
    Normal,
    /// Adds pipeline latency measurement (entry timestamps), shared drain
    /// durations, and completion unwind-depth summaries.
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
    /// Runtime metric detail level for channel transport, node outcomes, and
    /// shared control-plane telemetry.
    #[serde(default = "default_metric_level_basic")]
    pub runtime_metrics: MetricLevel,
}

impl Default for TelemetryPolicy {
    fn default() -> Self {
        Self {
            pipeline_metrics: true,
            tokio_metrics: true,
            runtime_metrics: MetricLevel::Basic,
        }
    }
}

const fn default_metric_level_basic() -> MetricLevel {
    MetricLevel::Basic
}

const fn default_true() -> bool {
    true
}

const fn default_false() -> bool {
    false
}

/// Resource-related policy declarations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ResourcesPolicy {
    /// CPU core allocation strategy for this pipeline.
    #[serde(default)]
    pub core_allocation: CoreAllocation,
    /// Optional process-wide memory limiter configuration.
    ///
    /// This is currently supported only at the top-level `policies.resources`
    /// scope. Group and pipeline overrides are rejected during engine validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limiter: Option<MemoryLimiterPolicy>,
}

/// Process-wide memory limiter declarations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MemoryLimiterPolicy {
    /// Runtime behavior applied when the limiter classifies `Hard` pressure.
    pub mode: MemoryLimiterMode,
    /// Preferred memory source used by the limiter.
    #[serde(default)]
    pub source: MemoryLimiterSource,
    /// Period between memory samples.
    #[serde(
        default = "default_memory_limiter_check_interval",
        with = "humantime_serde"
    )]
    #[schemars(with = "String")]
    pub check_interval: Duration,
    /// Soft limit in bytes. When omitted with `source: auto`, the runtime derives a value
    /// from the detected cgroup memory limit.
    #[serde(default, deserialize_with = "byte_units::deserialize_u64")]
    #[schemars(with = "Option<String>")]
    pub soft_limit: Option<u64>,
    /// Hard limit in bytes. When omitted with `source: auto`, the runtime derives a value
    /// from the detected cgroup memory limit.
    #[serde(default, deserialize_with = "byte_units::deserialize_u64")]
    #[schemars(with = "Option<String>")]
    pub hard_limit: Option<u64>,
    /// Bytes below the soft limit required to leave `Soft` pressure.
    #[serde(default, deserialize_with = "byte_units::deserialize_u64")]
    #[schemars(with = "Option<String>")]
    pub hysteresis: Option<u64>,
    /// Retry-After header value returned by HTTP receivers while shedding ingress in
    /// `enforce` mode.
    #[serde(default = "default_memory_limiter_retry_after_secs")]
    pub retry_after_secs: u32,
    /// Whether the admin readiness endpoint should fail while in `Hard` pressure in
    /// `enforce` mode.
    #[serde(default = "default_true")]
    pub fail_readiness_on_hard: bool,
    /// Whether the limiter should force a jemalloc purge when a tick's pre-purge sample
    /// classifies as `Hard`.
    #[serde(default = "default_false")]
    pub purge_on_hard: bool,
    /// Minimum interval between forced jemalloc purges.
    #[serde(
        default = "default_memory_limiter_purge_min_interval",
        with = "humantime_serde"
    )]
    #[schemars(with = "String")]
    pub purge_min_interval: Duration,
}

/// Enforcement behavior for the process-wide limiter.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLimiterMode {
    /// Update metrics/logs and reject ingress at `Hard`.
    Enforce,
    /// Update metrics/logs only; `Hard` remains advisory.
    ObserveOnly,
}

const fn default_memory_limiter_check_interval() -> Duration {
    Duration::from_secs(1)
}

const fn default_memory_limiter_retry_after_secs() -> u32 {
    1
}

const fn default_memory_limiter_purge_min_interval() -> Duration {
    Duration::from_secs(5)
}

/// Preferred memory source for the process-wide limiter.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLimiterSource {
    /// Prefer cgroup memory if available, otherwise fall back to RSS and then jemalloc resident.
    #[default]
    Auto,
    /// Use cgroup memory accounting only.
    Cgroup,
    /// Use process RSS only.
    Rss,
    /// Use jemalloc resident bytes only.
    JemallocResident,
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
    /// Capacity used for the shared pipeline-runtime orchestration control channel.
    #[serde(default = "default_pipeline_control_channel_capacity")]
    pub pipeline: usize,
    /// Capacity used for the shared Ack/Nack completion control channel.
    #[serde(default = "default_completion_control_channel_capacity")]
    pub completion: usize,
}

impl Default for ControlChannelCapacityPolicy {
    fn default() -> Self {
        Self {
            node: default_node_control_channel_capacity(),
            pipeline: default_pipeline_control_channel_capacity(),
            completion: default_completion_control_channel_capacity(),
        }
    }
}

const fn default_node_control_channel_capacity() -> usize {
    256
}

const fn default_pipeline_control_channel_capacity() -> usize {
    256
}

const fn default_completion_control_channel_capacity() -> usize {
    512
}

const fn default_pdata_channel_capacity() -> usize {
    128
}

#[cfg(test)]
mod tests {
    use super::{MemoryLimiterMode, MemoryLimiterPolicy, MemoryLimiterSource, Policies};
    use std::time::Duration;

    #[test]
    fn defaults_match_expected_values() {
        let defaults = Policies::resolve([&Policies::default()]);
        assert_eq!(defaults.channel_capacity.control.node, 256);
        assert_eq!(defaults.channel_capacity.control.pipeline, 256);
        assert_eq!(defaults.channel_capacity.control.completion, 512);
        assert_eq!(defaults.channel_capacity.pdata, 128);
        assert!(defaults.telemetry.pipeline_metrics);
        assert!(defaults.telemetry.tokio_metrics);
        assert_eq!(
            defaults.telemetry.runtime_metrics,
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
                    completion: 0,
                },
                pdata: 0,
            }),
            ..Default::default()
        };

        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 4);
        assert!(errors.iter().any(|e| e.contains("control.node")));
        assert!(errors.iter().any(|e| e.contains("control.pipeline")));
        assert!(errors.iter().any(|e| e.contains("control.completion")));
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
    fn telemetry_policy_with_runtime_metrics_level() {
        let yaml = r#"
            pipeline_metrics: true
            tokio_metrics: false
            runtime_metrics: detailed
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.runtime_metrics, super::MetricLevel::Detailed);
        assert!(!policy.tokio_metrics);
    }

    #[test]
    fn telemetry_policy_defaults_runtime_metrics_to_basic() {
        let yaml = r#"
            pipeline_metrics: true
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.runtime_metrics, super::MetricLevel::Basic);
    }

    #[test]
    fn validates_memory_limiter_settings() {
        let policies = Policies {
            resources: Some(super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::AllCores,
                memory_limiter: Some(MemoryLimiterPolicy {
                    mode: MemoryLimiterMode::Enforce,
                    source: MemoryLimiterSource::Auto,
                    check_interval: Duration::from_millis(50),
                    soft_limit: Some(200),
                    hard_limit: Some(100),
                    hysteresis: Some(200),
                    retry_after_secs: 1,
                    fail_readiness_on_hard: true,
                    purge_on_hard: false,
                    purge_min_interval: Duration::from_secs(5),
                }),
            }),
            ..Policies::default()
        };

        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 3);
        assert!(errors.iter().any(|error| error.contains("check_interval")));
        assert!(errors.iter().any(|error| error.contains("hard_limit")));
        assert!(errors.iter().any(|error| error.contains("hysteresis")));
    }

    #[test]
    fn validates_memory_limiter_requires_both_limits_when_explicit() {
        let policies = Policies {
            resources: Some(super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::AllCores,
                memory_limiter: Some(MemoryLimiterPolicy {
                    mode: MemoryLimiterMode::Enforce,
                    source: MemoryLimiterSource::Rss,
                    check_interval: Duration::from_secs(1),
                    soft_limit: Some(100),
                    hard_limit: None,
                    hysteresis: None,
                    retry_after_secs: 1,
                    fail_readiness_on_hard: true,
                    purge_on_hard: false,
                    purge_min_interval: Duration::from_secs(5),
                }),
            }),
            ..Policies::default()
        };

        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must either both be set or both be omitted"));
    }

    #[test]
    fn validates_memory_limiter_rejects_zero_soft_limit() {
        let policies = Policies {
            resources: Some(super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::AllCores,
                memory_limiter: Some(MemoryLimiterPolicy {
                    mode: MemoryLimiterMode::Enforce,
                    source: MemoryLimiterSource::Rss,
                    check_interval: Duration::from_secs(1),
                    soft_limit: Some(0),
                    hard_limit: Some(100),
                    hysteresis: None,
                    retry_after_secs: 1,
                    fail_readiness_on_hard: true,
                    purge_on_hard: false,
                    purge_min_interval: Duration::from_secs(5),
                }),
            }),
            ..Policies::default()
        };

        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("soft_limit must be greater than 0"));
    }

    #[test]
    fn validates_memory_limiter_requires_limits_for_non_auto_sources() {
        let policies = Policies {
            resources: Some(super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::AllCores,
                memory_limiter: Some(MemoryLimiterPolicy {
                    mode: MemoryLimiterMode::Enforce,
                    source: MemoryLimiterSource::Rss,
                    check_interval: Duration::from_secs(1),
                    soft_limit: None,
                    hard_limit: None,
                    hysteresis: None,
                    retry_after_secs: 1,
                    fail_readiness_on_hard: true,
                    purge_on_hard: false,
                    purge_min_interval: Duration::from_secs(5),
                }),
            }),
            ..Policies::default()
        };

        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("source is not auto"));
    }

    #[test]
    fn validates_memory_limiter_rejects_zero_retry_after_secs() {
        let policies = Policies {
            resources: Some(super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::AllCores,
                memory_limiter: Some(MemoryLimiterPolicy {
                    mode: MemoryLimiterMode::Enforce,
                    source: MemoryLimiterSource::Auto,
                    check_interval: Duration::from_secs(1),
                    soft_limit: Some(100),
                    hard_limit: Some(200),
                    hysteresis: None,
                    retry_after_secs: 0,
                    fail_readiness_on_hard: true,
                    purge_on_hard: false,
                    purge_min_interval: Duration::from_secs(5),
                }),
            }),
            ..Policies::default()
        };

        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("retry_after_secs must be greater than 0"));
    }

    #[test]
    fn validates_memory_limiter_rejects_zero_purge_min_interval() {
        let policies = Policies {
            resources: Some(super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::AllCores,
                memory_limiter: Some(MemoryLimiterPolicy {
                    mode: MemoryLimiterMode::Enforce,
                    source: MemoryLimiterSource::Auto,
                    check_interval: Duration::from_secs(1),
                    soft_limit: Some(100),
                    hard_limit: Some(200),
                    hysteresis: None,
                    retry_after_secs: 1,
                    fail_readiness_on_hard: true,
                    purge_on_hard: true,
                    purge_min_interval: Duration::ZERO,
                }),
            }),
            ..Policies::default()
        };

        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("purge_min_interval must be greater than 0"));
    }
}
