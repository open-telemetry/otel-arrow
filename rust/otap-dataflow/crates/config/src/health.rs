// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Health policy defining liveness and readiness probes.
//!
//! ## Liveness Probe
//!
//! Used to decide whether to restart the server/container. Typical use cases:
//! - Detect deadlocks or stuck event loops.
//! - Force a restart when the app can't recover by itself.
//! - Should be cheap and internal (not dependent on external systems).
//!
//! ## Readiness Probe
//!
//! Used to decide whether the Pod should receive traffic. Typical use cases:
//! - Gate traffic until startup work is done (pipeline deployed and running).
//! - Temporarily remove the Pod from load balancing when it can't serve correctly.
//! - Can check key dependencies, but avoid making it too fragile.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Policy controlling health checks for a pipeline instance.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HealthPolicy {
    /// Phases in which the system is considered alive.
    #[serde(default = "default_live_if")]
    pub live_if: Vec<PhaseKind>,
    /// Phases in which the system is considered ready.
    #[serde(default = "default_ready_if")]
    pub ready_if: Vec<PhaseKind>,

    /// Quorum for livez across cores.
    #[serde(default = "default_live_quorum")]
    pub live_quorum: Quorum,
    /// Quorum for readyz across cores.
    #[serde(default = "default_ready_quorum")]
    pub ready_quorum: Quorum,
}

impl Default for HealthPolicy {
    fn default() -> Self {
        Self {
            live_if: default_live_if(),
            ready_if: default_ready_if(),
            live_quorum: default_live_quorum(),
            ready_quorum: default_ready_quorum(),
        }
    }
}

const fn default_live_quorum() -> Quorum {
    Quorum::AtLeast(1)
}

const fn default_ready_quorum() -> Quorum {
    Quorum::All
}

/// Quorum expresses how many cores must satisfy a predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
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

/// Default policy:
/// - live in all states except `Deleted`
/// - ready in `Running` and `Updating` (could be optional).
fn default_live_if() -> Vec<PhaseKind> {
    vec![
        PhaseKind::Pending,
        PhaseKind::Starting,
        PhaseKind::Running,
        PhaseKind::Updating,
        PhaseKind::RollingBack,
        PhaseKind::Draining,
        PhaseKind::Stopped,
        PhaseKind::Rejected,
        PhaseKind::Failed,
        PhaseKind::Deleting,
    ]
}

fn default_ready_if() -> Vec<PhaseKind> {
    vec![PhaseKind::Running, PhaseKind::Updating]
}

impl HealthPolicy {
    /// Check if the given phase kind is considered live.
    #[inline]
    pub fn is_live<K: Into<PhaseKind>>(&self, k: K) -> bool {
        self.live_if.contains(&k.into())
    }

    /// Check if the given phase kind is considered ready.
    #[inline]
    pub fn is_ready<K: Into<PhaseKind>>(&self, k: K) -> bool {
        self.ready_if.contains(&k.into())
    }
}
