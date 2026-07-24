// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine and pipeline policy declarations.

use crate::byte_units;
use crate::health::HealthPolicy;
use crate::transport_headers_policy::TransportHeadersPolicy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    /// Controller-managed recovery policy for failed regular pipeline runtimes.
    ///
    /// When absent, a parent scope's runtime recovery policy or the built-in
    /// default applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) runtime_recovery: Option<RuntimeRecoveryPolicy>,
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
        let mut runtime_recovery = None;
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
            if runtime_recovery.is_none() {
                runtime_recovery = scope.runtime_recovery.as_ref();
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
            runtime_recovery: runtime_recovery.cloned().unwrap_or_default(),
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

        if let Some(resources) = &self.resources {
            if let Err(e) = resources.core_allocation.validate() {
                errors.push(format!("{path_prefix}.resources.core_allocation: {e}"));
            }
        }
        if let Some(runtime_recovery) = &self.runtime_recovery {
            errors.extend(
                runtime_recovery.validation_errors(&format!("{path_prefix}.runtime_recovery")),
            );
        }
        if let Some(telemetry) = &self.telemetry {
            errors.extend(telemetry.validation_errors(&format!("{path_prefix}.telemetry")));
        }
        if let Some(transport_headers) = &self.transport_headers {
            if let Err(e) = transport_headers.header_propagation.validate() {
                errors.push(format!(
                    "{path_prefix}.transport_headers.header_propagation.default.selector: {e}"
                ));
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
    /// Controller-managed runtime recovery policy.
    pub runtime_recovery: RuntimeRecoveryPolicy,
    /// Resources policy.
    pub resources: ResourcesPolicy,
    /// Transport headers policy. `None` when the feature is not configured
    /// (opt-in only -- no headers are captured or propagated by default).
    pub transport_headers: Option<TransportHeadersPolicy>,
}

impl ResolvedPolicies {
    /// Compares resolved policies while intentionally ignoring the resources
    /// policy, which controls placement and scaling rather than runtime shape.
    #[must_use]
    pub fn eq_ignoring_resources(&self, other: &Self) -> bool {
        let Self {
            channel_capacity: self_channel_capacity,
            health: self_health,
            telemetry: self_telemetry,
            runtime_recovery: self_runtime_recovery,
            resources: _,
            transport_headers: self_transport_headers,
        } = self;
        let Self {
            channel_capacity: other_channel_capacity,
            health: other_health,
            telemetry: other_telemetry,
            runtime_recovery: other_runtime_recovery,
            resources: _,
            transport_headers: other_transport_headers,
        } = other;

        self_channel_capacity == other_channel_capacity
            && self_health == other_health
            && self_telemetry == other_telemetry
            && self_runtime_recovery == other_runtime_recovery
            && self_transport_headers == other_transport_headers
    }
}

/// Controller-managed recovery policy for failed regular pipeline runtimes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRecoveryPolicy {
    /// Whether unexpected runtime failures are recovered in-process.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Maximum replacement launches allowed in one failure streak.
    #[serde(default = "default_runtime_recovery_max_restarts")]
    pub max_restarts: usize,
    /// Delay before the first replacement attempt.
    #[serde(
        default = "default_runtime_recovery_initial_backoff",
        with = "humantime_serde"
    )]
    #[schemars(with = "String")]
    pub initial_backoff: Duration,
    /// Upper bound for exponential replacement backoff.
    #[serde(
        default = "default_runtime_recovery_max_backoff",
        with = "humantime_serde"
    )]
    #[schemars(with = "String")]
    pub max_backoff: Duration,
    /// Maximum time a replacement may take to report admitted and ready.
    #[serde(
        default = "default_runtime_recovery_startup_timeout",
        with = "humantime_serde"
    )]
    #[schemars(with = "String")]
    pub startup_timeout: Duration,
    /// Ready-runtime duration after which the restart streak resets.
    #[serde(
        default = "default_runtime_recovery_reset_after",
        with = "humantime_serde"
    )]
    #[schemars(with = "String")]
    pub reset_after: Duration,
}

impl Default for RuntimeRecoveryPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            max_restarts: default_runtime_recovery_max_restarts(),
            initial_backoff: default_runtime_recovery_initial_backoff(),
            max_backoff: default_runtime_recovery_max_backoff(),
            startup_timeout: default_runtime_recovery_startup_timeout(),
            reset_after: default_runtime_recovery_reset_after(),
        }
    }
}

impl RuntimeRecoveryPolicy {
    /// Returns validation errors for the recovery policy.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if self.max_restarts == 0 {
            errors.push(format!("{path_prefix}.max_restarts must be greater than 0"));
        }
        if self.initial_backoff.is_zero() {
            errors.push(format!(
                "{path_prefix}.initial_backoff must be greater than 0"
            ));
        }
        if self.max_backoff.is_zero() {
            errors.push(format!("{path_prefix}.max_backoff must be greater than 0"));
        }
        if self.initial_backoff > self.max_backoff {
            errors.push(format!(
                "{path_prefix}.initial_backoff must not exceed {path_prefix}.max_backoff"
            ));
        }
        if self.startup_timeout.is_zero() {
            errors.push(format!(
                "{path_prefix}.startup_timeout must be greater than 0"
            ));
        }
        if self.reset_after.is_zero() {
            errors.push(format!("{path_prefix}.reset_after must be greater than 0"));
        }
        errors
    }
}

const fn default_runtime_recovery_max_restarts() -> usize {
    5
}

const fn default_runtime_recovery_initial_backoff() -> Duration {
    Duration::from_millis(250)
}

const fn default_runtime_recovery_max_backoff() -> Duration {
    Duration::from_secs(30)
}

const fn default_runtime_recovery_startup_timeout() -> Duration {
    Duration::from_secs(30)
}

const fn default_runtime_recovery_reset_after() -> Duration {
    Duration::from_secs(60)
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
    /// Distributed flow_metrics that sum per-message compute duration across
    /// a range of processor nodes.
    #[serde(default)]
    pub flow_metrics: Vec<FlowMetricConfig>,
}

/// Configuration for flow metrics across a contiguous range of processor nodes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FlowMetricConfig {
    /// User-facing identifier for this flow metric, used as a metric attribute.
    pub id: String,
    /// Processor node bounds for this flow metric.
    pub bounds: FlowBounds,
    /// Metrics to enable. Omitted means all metrics are enabled.
    #[serde(default)]
    pub metrics: Option<Vec<FlowMetric>>,
    /// Optional per-flow purpose differentiator, emitted as the `flow.purpose`
    /// scope attribute on every metric this flow produces. Lets OTel View
    /// selectors target distinct flavors of processor work (e.g. `filter`
    /// flows that keep/drop records vs `transform` flows that enrich and
    /// reshape them) even though all flows share the single `flow`
    /// instrumentation scope. When omitted, `flow.purpose` is still emitted
    /// but carries an empty value (no purpose differentiation).
    #[serde(default)]
    pub purpose: Option<String>,
}

impl FlowMetricConfig {
    /// Returns whether the given metric is enabled for this flow.
    #[must_use]
    pub fn has(&self, metric: FlowMetric) -> bool {
        match &self.metrics {
            None => true,
            Some(metrics) => metrics.contains(&metric),
        }
    }
}

/// Start/end node bounds for a flow metric.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FlowBounds {
    /// Processor node name where the flow metric range begins (inclusive).
    pub start_node: String,
    /// Processor node name where the flow metric range ends (inclusive).
    pub end_node: String,
}

/// Individual metrics that can be enabled for a flow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FlowMetric {
    /// Aggregate processor compute duration across the flow.
    ComputeDuration,
    /// Signal item count entering the flow.
    SignalsIncoming,
    /// Signal item count leaving the flow.
    SignalsOutgoing,
    /// Signal item count a decision node chose to drop.
    SignalsDropped,
}

impl TelemetryPolicy {
    /// Returns validation errors for the telemetry policy.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        let mut errors = Vec::new();
        for (idx, flow) in self.flow_metrics.iter().enumerate() {
            let path = format!("{path_prefix}.flow_metrics[{idx}].metrics");
            if let Some(metrics) = &flow.metrics {
                if metrics.is_empty() {
                    errors.push(format!(
                        "{path} must not be empty when explicitly configured"
                    ));
                }
                let mut seen = HashSet::new();
                for metric in metrics {
                    if !seen.insert(*metric) {
                        errors.push(format!("{path} must not contain duplicate entries"));
                        break;
                    }
                }
            }
        }
        errors
    }
}

impl Default for TelemetryPolicy {
    fn default() -> Self {
        Self {
            pipeline_metrics: true,
            tokio_metrics: true,
            runtime_metrics: MetricLevel::Basic,
            flow_metrics: Vec::new(),
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct CoreAllocation {
    /// Allocation strategy: "all_cores", "core_count", or "core_set"
    #[serde(default = "default_strategy", alias = "type")]
    pub strategy: CoreAllocationStrategy,

    /// Number of cores to use (only valid when strategy is "core_count").
    /// If 0, uses all available cores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,

    /// Core set defined as a set of ranges (only valid when strategy is "core_set").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set: Option<Vec<CoreRange>>,
}

/// Defines how CPU cores should be allocated for pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CoreAllocationStrategy {
    /// Use all available CPU cores.
    #[default]
    AllCores,
    /// Use a specific number of CPU cores (starting from core 0).
    /// If the requested number exceeds available cores, use all available cores.
    CoreCount,
    /// Defines a set of CPU cores should be allocated for pipeline execution.
    CoreSet,
}

fn default_strategy() -> CoreAllocationStrategy {
    CoreAllocationStrategy::AllCores
}

impl Default for CoreAllocation {
    fn default() -> Self {
        CoreAllocation {
            strategy: CoreAllocationStrategy::AllCores,
            count: None,
            set: None,
        }
    }
}

impl CoreAllocation {
    /// Creates an `AllCores` allocation (use all available CPU cores).
    #[must_use]
    pub fn all_cores() -> Self {
        Self::default()
    }

    /// Creates a `CoreCount` allocation with the given number of cores.
    #[must_use]
    pub fn core_count(count: usize) -> Self {
        Self {
            strategy: CoreAllocationStrategy::CoreCount,
            count: Some(count),
            set: None,
        }
    }

    /// Creates a `CoreSet` allocation with the given core ranges.
    #[must_use]
    pub fn core_set(set: Vec<CoreRange>) -> Self {
        Self {
            strategy: CoreAllocationStrategy::CoreSet,
            count: None,
            set: Some(set),
        }
    }

    /// Validates that the fields are consistent with the selected strategy.
    ///
    /// - `all_cores`: `count` and `set` must both be `None`.
    /// - `core_count`: `count` must be `Some`, `set` must be `None`.
    /// - `core_set`: `set` must be `Some` and non-empty, `count` must be `None`.
    pub fn validate(&self) -> Result<(), String> {
        match self.strategy {
            CoreAllocationStrategy::AllCores => {
                if self.count.is_some() {
                    return Err("'count' must not be set when strategy is 'all_cores'".to_string());
                }
                if self.set.is_some() {
                    return Err("'set' must not be set when strategy is 'all_cores'".to_string());
                }
            }
            CoreAllocationStrategy::CoreCount => {
                if self.count.is_none() {
                    return Err("'count' is required when strategy is 'core_count'".to_string());
                }
                if self.set.is_some() {
                    return Err("'set' must not be set when strategy is 'core_count'".to_string());
                }
            }
            CoreAllocationStrategy::CoreSet => {
                if self.count.is_some() {
                    return Err("'count' must not be set when strategy is 'core_set'".to_string());
                }
                match &self.set {
                    None => {
                        return Err("'set' is required when strategy is 'core_set'".to_string());
                    }
                    Some(set) if set.is_empty() => {
                        return Err(
                            "'set' must not be empty when strategy is 'core_set'".to_string()
                        );
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Display for CoreAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.strategy {
            CoreAllocationStrategy::AllCores => write!(f, "*"),
            CoreAllocationStrategy::CoreCount => {
                let count = self.count.unwrap_or(0);
                write!(f, "[{count} cores]")
            }
            CoreAllocationStrategy::CoreSet => {
                let mut first = true;
                if let Some(set) = &self.set {
                    for item in set {
                        if !first {
                            write!(f, ",")?;
                        }
                        write!(f, "{item}")?;
                        first = false;
                    }
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
    use super::{
        MemoryLimiterMode, MemoryLimiterPolicy, MemoryLimiterSource, Policies,
        RuntimeRecoveryPolicy,
    };
    use std::time::Duration;

    #[test]
    fn resolved_policies_eq_ignoring_resources_ignores_resource_only_changes() {
        let current = super::ResolvedPolicies {
            resources: super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::core_count(1),
                memory_limiter: None,
            },
            ..super::ResolvedPolicies::default()
        };
        let candidate = super::ResolvedPolicies {
            resources: super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::core_count(2),
                memory_limiter: None,
            },
            ..super::ResolvedPolicies::default()
        };

        assert_ne!(current, candidate);
        assert!(current.eq_ignoring_resources(&candidate));
    }

    #[test]
    fn resolved_policies_eq_ignoring_resources_detects_runtime_policy_change() {
        let current = super::ResolvedPolicies::default();
        let candidate = super::ResolvedPolicies {
            telemetry: super::TelemetryPolicy {
                pipeline_metrics: false,
                ..super::TelemetryPolicy::default()
            },
            ..super::ResolvedPolicies::default()
        };

        assert!(!current.eq_ignoring_resources(&candidate));
    }

    /// Scenario: all policy families are omitted from configuration.
    /// Guarantees: runtime recovery defaults to five enabled attempts with balanced timing.
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
            super::CoreAllocation::all_cores()
        );
        assert_eq!(defaults.health, crate::health::HealthPolicy::default());
        assert_eq!(
            defaults.runtime_recovery,
            RuntimeRecoveryPolicy {
                enabled: true,
                max_restarts: 5,
                initial_backoff: Duration::from_millis(250),
                max_backoff: Duration::from_secs(30),
                startup_timeout: Duration::from_secs(30),
                reset_after: Duration::from_secs(60),
            }
        );
    }

    /// Scenario: pipeline and parent scopes specify different runtime recovery policies.
    /// Guarantees: policy resolution selects the complete lower-scope policy family.
    #[test]
    fn runtime_recovery_policy_resolves_by_scope_precedence() {
        let parent = Policies {
            runtime_recovery: Some(RuntimeRecoveryPolicy::default()),
            ..Policies::default()
        };
        let child_policy = RuntimeRecoveryPolicy {
            enabled: false,
            max_restarts: 2,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(2),
            startup_timeout: Duration::from_secs(3),
            reset_after: Duration::from_secs(4),
        };
        let child = Policies {
            runtime_recovery: Some(child_policy.clone()),
            ..Policies::default()
        };

        let resolved = Policies::resolve([&child, &parent]);

        assert_eq!(resolved.runtime_recovery, child_policy);
    }

    /// Scenario: a runtime recovery policy uses human-readable duration values.
    /// Guarantees: YAML parsing accepts every public recovery field with exact values.
    #[test]
    fn runtime_recovery_policy_parses_human_readable_durations() {
        let policy: RuntimeRecoveryPolicy = serde_yaml::from_str(
            r#"
enabled: false
max_restarts: 7
initial_backoff: 125ms
max_backoff: 12s
startup_timeout: 45s
reset_after: 2m
"#,
        )
        .expect("runtime recovery policy should parse");

        assert!(!policy.enabled);
        assert_eq!(policy.max_restarts, 7);
        assert_eq!(policy.initial_backoff, Duration::from_millis(125));
        assert_eq!(policy.max_backoff, Duration::from_secs(12));
        assert_eq!(policy.startup_timeout, Duration::from_secs(45));
        assert_eq!(policy.reset_after, Duration::from_secs(120));
    }

    /// Scenario: a runtime recovery policy contains an unsupported field.
    /// Guarantees: strict deserialization rejects typos instead of silently ignoring them.
    #[test]
    fn runtime_recovery_policy_rejects_unknown_fields() {
        let error = serde_yaml::from_str::<RuntimeRecoveryPolicy>(
            r#"
max_restarts: 5
unknown_recovery_option: true
"#,
        )
        .expect_err("unknown runtime recovery fields should be rejected");

        assert!(error.to_string().contains("unknown_recovery_option"));
    }

    /// Scenario: every bounded recovery duration and count is invalid.
    /// Guarantees: validation reports all unsafe zero values and inverted backoff bounds.
    #[test]
    fn runtime_recovery_policy_validates_bounds() {
        let policies = Policies {
            runtime_recovery: Some(RuntimeRecoveryPolicy {
                enabled: true,
                max_restarts: 0,
                initial_backoff: Duration::from_secs(2),
                max_backoff: Duration::ZERO,
                startup_timeout: Duration::ZERO,
                reset_after: Duration::ZERO,
            }),
            ..Policies::default()
        };

        let errors = policies.validation_errors("policies");

        assert_eq!(errors.len(), 5);
        assert!(errors.iter().any(|error| error.contains("max_restarts")));
        assert!(errors.iter().any(|error| error.contains("max_backoff")));
        assert!(errors.iter().any(|error| error.contains("must not exceed")));
        assert!(errors.iter().any(|error| error.contains("startup_timeout")));
        assert!(errors.iter().any(|error| error.contains("reset_after")));
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
        assert_eq!(super::CoreAllocation::all_cores().to_string(), "*");
    }

    #[test]
    fn core_allocation_display_core_count() {
        assert_eq!(
            super::CoreAllocation::core_count(4).to_string(),
            "[4 cores]"
        );
    }

    #[test]
    fn core_allocation_display_core_set_single_range() {
        assert_eq!(
            super::CoreAllocation::core_set(vec![super::CoreRange { start: 0, end: 3 }])
                .to_string(),
            "0-3"
        );
    }

    #[test]
    fn core_allocation_display_core_set_multiple_ranges() {
        assert_eq!(
            super::CoreAllocation::core_set(vec![
                super::CoreRange { start: 0, end: 3 },
                super::CoreRange { start: 8, end: 11 },
                super::CoreRange { start: 16, end: 16 },
            ])
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
    fn flow_metrics_omitted_metrics_enable_all() {
        let yaml = r#"
            flow_metrics:
              - id: flow1
                bounds: { start_node: a, end_node: b }
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        let flow = &policy.flow_metrics[0];
        assert!(flow.metrics.is_none());
        assert!(flow.has(super::FlowMetric::ComputeDuration));
        assert!(flow.has(super::FlowMetric::SignalsIncoming));
        assert!(flow.has(super::FlowMetric::SignalsOutgoing));
        assert!(flow.has(super::FlowMetric::SignalsDropped));
    }

    #[test]
    fn flow_metrics_explicit_subset_is_honored() {
        let yaml = r#"
            flow_metrics:
              - id: flow1
                bounds: { start_node: a, end_node: b }
                metrics: [compute_duration]
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        let flow = &policy.flow_metrics[0];
        assert!(flow.has(super::FlowMetric::ComputeDuration));
        assert!(!flow.has(super::FlowMetric::SignalsIncoming));
        assert!(!flow.has(super::FlowMetric::SignalsOutgoing));
        assert!(!flow.has(super::FlowMetric::SignalsDropped));
    }

    #[test]
    fn flow_metrics_dropped_is_parsed() {
        let yaml = r#"
            flow_metrics:
              - id: flow1
                bounds: { start_node: a, end_node: b }
                metrics: [signals_dropped]
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        let flow = &policy.flow_metrics[0];
        assert!(flow.has(super::FlowMetric::SignalsDropped));
        assert!(!flow.has(super::FlowMetric::ComputeDuration));
    }

    #[test]
    fn flow_metrics_purpose_defaults_to_none() {
        let yaml = r#"
            flow_metrics:
              - id: flow1
                bounds: { start_node: a, end_node: b }
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.flow_metrics[0].purpose, None);
    }

    #[test]
    fn flow_metrics_purpose_is_parsed() {
        let yaml = r#"
            flow_metrics:
              - id: flow1
                bounds: { start_node: a, end_node: b }
                purpose: receiver
        "#;
        let policy: super::TelemetryPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.flow_metrics[0].purpose.as_deref(), Some("receiver"));
    }

    #[test]
    fn flow_metrics_rejects_empty_metrics() {
        let policies = Policies {
            telemetry: Some(super::TelemetryPolicy {
                flow_metrics: vec![super::FlowMetricConfig {
                    id: "flow1".to_string(),
                    bounds: super::FlowBounds {
                        start_node: "a".to_string(),
                        end_node: "b".to_string(),
                    },
                    metrics: Some(vec![]),
                    purpose: None,
                }],
                ..super::TelemetryPolicy::default()
            }),
            ..Default::default()
        };
        let errors = policies.validation_errors("policies");
        assert!(
            errors
                .iter()
                .any(|error| error.contains("must not be empty"))
        );
    }

    #[test]
    fn flow_metrics_rejects_duplicate_metrics() {
        let policies = Policies {
            telemetry: Some(super::TelemetryPolicy {
                flow_metrics: vec![super::FlowMetricConfig {
                    id: "flow1".to_string(),
                    bounds: super::FlowBounds {
                        start_node: "a".to_string(),
                        end_node: "b".to_string(),
                    },
                    metrics: Some(vec![
                        super::FlowMetric::ComputeDuration,
                        super::FlowMetric::ComputeDuration,
                    ]),
                    purpose: None,
                }],
                ..super::TelemetryPolicy::default()
            }),
            ..Default::default()
        };
        let errors = policies.validation_errors("policies");
        assert!(errors.iter().any(|error| error.contains("duplicate")));
    }

    #[test]
    fn validates_memory_limiter_settings() {
        let policies = Policies {
            resources: Some(super::ResourcesPolicy {
                core_allocation: super::CoreAllocation::all_cores(),
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
                core_allocation: super::CoreAllocation::all_cores(),
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
                core_allocation: super::CoreAllocation::all_cores(),
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
                core_allocation: super::CoreAllocation::all_cores(),
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
                core_allocation: super::CoreAllocation::all_cores(),
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
                core_allocation: super::CoreAllocation::all_cores(),
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

    #[test]
    fn validates_transport_headers_selector() {
        use crate::transport_headers_policy::{
            HeaderPropagationPolicy, PropagationDefault, PropagationSelector,
            PropagationSelectorType, TransportHeadersPolicy,
        };

        let policies = Policies {
            transport_headers: Some(TransportHeadersPolicy {
                header_propagation: HeaderPropagationPolicy {
                    default: PropagationDefault {
                        selector: PropagationSelector {
                            selector_type: PropagationSelectorType::Named,
                            named: None, // Invalid: named type requires named list
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("transport_headers.header_propagation.default.selector"));
        assert!(errors[0].contains("'named' list is required"));
    }

    #[test]
    fn core_allocation_validate_all_cores_valid() {
        assert!(super::CoreAllocation::all_cores().validate().is_ok());
    }

    #[test]
    fn core_allocation_validate_core_count_valid() {
        assert!(super::CoreAllocation::core_count(4).validate().is_ok());
    }

    #[test]
    fn core_allocation_validate_core_set_valid() {
        assert!(
            super::CoreAllocation::core_set(vec![super::CoreRange { start: 0, end: 3 }])
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn core_allocation_validate_all_cores_with_count() {
        let alloc = super::CoreAllocation {
            strategy: super::CoreAllocationStrategy::AllCores,
            count: Some(4),
            set: None,
        };
        let err = alloc.validate().unwrap_err();
        assert!(err.contains("'count' must not be set"));
    }

    #[test]
    fn core_allocation_validate_all_cores_with_set() {
        let alloc = super::CoreAllocation {
            strategy: super::CoreAllocationStrategy::AllCores,
            count: None,
            set: Some(vec![super::CoreRange { start: 0, end: 3 }]),
        };
        let err = alloc.validate().unwrap_err();
        assert!(err.contains("'set' must not be set"));
    }

    #[test]
    fn core_allocation_validate_core_count_missing_count() {
        let alloc = super::CoreAllocation {
            strategy: super::CoreAllocationStrategy::CoreCount,
            count: None,
            set: None,
        };
        let err = alloc.validate().unwrap_err();
        assert!(err.contains("'count' is required"));
    }

    #[test]
    fn core_allocation_validate_core_count_with_set() {
        let alloc = super::CoreAllocation {
            strategy: super::CoreAllocationStrategy::CoreCount,
            count: Some(4),
            set: Some(vec![super::CoreRange { start: 0, end: 3 }]),
        };
        let err = alloc.validate().unwrap_err();
        assert!(err.contains("'set' must not be set"));
    }

    #[test]
    fn core_allocation_validate_core_set_missing_set() {
        let alloc = super::CoreAllocation {
            strategy: super::CoreAllocationStrategy::CoreSet,
            count: None,
            set: None,
        };
        let err = alloc.validate().unwrap_err();
        assert!(err.contains("'set' is required"));
    }

    #[test]
    fn core_allocation_validate_core_set_empty_set() {
        let alloc = super::CoreAllocation {
            strategy: super::CoreAllocationStrategy::CoreSet,
            count: None,
            set: Some(vec![]),
        };
        let err = alloc.validate().unwrap_err();
        assert!(err.contains("'set' must not be empty"));
    }

    #[test]
    fn core_allocation_validate_core_set_with_count() {
        let alloc = super::CoreAllocation {
            strategy: super::CoreAllocationStrategy::CoreSet,
            count: Some(4),
            set: Some(vec![super::CoreRange { start: 0, end: 3 }]),
        };
        let err = alloc.validate().unwrap_err();
        assert!(err.contains("'count' must not be set"));
    }

    #[test]
    fn validates_core_allocation_in_policies() {
        let policies = Policies {
            resources: Some(super::ResourcesPolicy {
                core_allocation: super::CoreAllocation {
                    strategy: super::CoreAllocationStrategy::CoreCount,
                    count: None,
                    set: None,
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let errors = policies.validation_errors("policies");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("resources.core_allocation"));
        assert!(errors[0].contains("'count' is required"));
    }
}
