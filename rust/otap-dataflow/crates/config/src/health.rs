// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Health policy configuration.

/// Policy controlling aggregate probes and using the per-core `ProbePolicy`.
#[derive(Debug, Clone)]
pub struct AggregationPolicy {
    /// Mapping of per-core phases to probes.
    pub core_probe: ProbePolicy,
    /// Quorum for livez across cores.
    pub live_quorum: Quorum,
    /// Quorum for readyz across cores.
    pub ready_quorum: Quorum,
}

/// Defaults: live if *any* core is live; ready when *all* non-deleted cores are ready.
pub const DEFAULT_AGGREGATION_POLICY: AggregationPolicy = AggregationPolicy {
    core_probe: DEFAULT_PROBE_POLICY,
    live_quorum: Quorum::AtLeast(1),
    ready_quorum: Quorum::All,
};

/// Quorum expresses how many cores must satisfy a predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(variant_size_differences)]
pub enum Quorum {
    /// All non-deleted cores must satisfy the predicate.
    All,
    /// At least an absolute number of non-deleted cores must satisfy the predicate.
    AtLeast(usize),
    /// At least this percentage (0..=100) of non-deleted cores must satisfy the predicate.
    Percent(u8),
}

/// Coarse discriminant for `Phase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseKind {
    /// Initial state, not yet started.
    Pending, 
    /// In the process of
    Starting, 
    /// Fully started and operational.
    Running, 
    /// In the process of
    Updating, 
    /// In the process of
    RollingBack, 
    /// In the process of
    Draining,
    /// Fully stopped (not running).
    Stopped, 
    /// Permanently rejected for configuration reasons.
    Rejected, 
    /// Failed due to a runtime error.
    Failed, 
    /// In the process of
    Deleting, 
    /// Fully deleted (not running, not recoverable).
    Deleted,
}

/// Declarative mapping of phases to Kubernetes probes.
/// - `live_if`: pipeline instance is considered alive (the pod/process is functional/manageable).
/// - `ready_if`: instance is ready to accept traffic/work.
#[derive(Debug, Clone)]
pub struct ProbePolicy {
    /// Phases in which the instance is considered alive.
    pub live_if:   &'static [PhaseKind],
    /// Phases in which the instance is considered ready.
    pub ready_if:  &'static [PhaseKind],
}

/// Default policy:
/// - live in all states except `Deleted`
/// - ready in `Running` (and optionally `Updating`).
pub const DEFAULT_LIVE_IF: &[PhaseKind] = &[
    PhaseKind::Pending, PhaseKind::Starting, PhaseKind::Running, PhaseKind::Updating,
    PhaseKind::RollingBack, PhaseKind::Draining, PhaseKind::Stopped, PhaseKind::Rejected,
    PhaseKind::Failed, PhaseKind::Deleting,
];

/// Flip `Updating` off here if you do not want readiness during updates.
pub const DEFAULT_READY_IF: &[PhaseKind] = &[
    PhaseKind::Running,
    PhaseKind::Updating,
];

const DEFAULT_PROBE_POLICY: ProbePolicy = ProbePolicy {
    live_if: DEFAULT_LIVE_IF,
    ready_if: DEFAULT_READY_IF,
};

impl ProbePolicy {
    /// Check if the given phase kind is considered live.
    #[inline]
    pub fn is_live<K: Into<PhaseKind>>(&self, k: K) -> bool { self.live_if.contains(&k.into()) }
    
    /// Check if the given phase kind is considered ready.
    #[inline]
    pub fn is_ready<K: Into<PhaseKind>>(&self, k: K) -> bool { self.ready_if.contains(&k.into()) }
}