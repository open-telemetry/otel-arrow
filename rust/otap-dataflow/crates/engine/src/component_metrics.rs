// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component-level pipeline metrics for tracking data consumed and produced
//! by each node (receiver, processor, exporter).
//!
//! Metric sets are level-gated by [`MetricLevel`]:
//!
//! - **Basic+**: Request outcome counters (consumed success/failure/refused, produced success/refused).
//! - **Normal+**: (Reserved for future forward-path byte counters.)
//! - **Detailed**: (Reserved; consumed duration moved to channel receiver metrics.)
//!
//! At `MetricLevel::None`, no metric sets are registered.

use crate::control::MetricLevel;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Metric set definitions — grouped by level
// ---------------------------------------------------------------------------

// -- Basic+ : request outcome counts --

/// Consumer request outcome counts (Basic+).
#[metric_set(name = "component.consumed.requests")]
#[derive(Debug, Default, Clone)]
pub struct ConsumedRequestMetrics {
    /// Requests successfully consumed.
    #[metric(name = "success", unit = "{1}")]
    pub success: Counter<u64>,
    /// Requests that failed (transient / retryable).
    #[metric(name = "failure", unit = "{1}")]
    pub failure: Counter<u64>,
    /// Requests permanently refused.
    #[metric(name = "refused", unit = "{1}")]
    pub refused: Counter<u64>,
}

/// Producer request outcome counts (Basic+).
#[metric_set(name = "component.produced.requests")]
#[derive(Debug, Default, Clone)]
pub struct ProducedRequestMetrics {
    /// Requests acknowledged by downstream.
    #[metric(name = "success", unit = "{1}")]
    pub success: Counter<u64>,
    /// Requests that failed downstream (transient / retryable, i.e. non-permanent nack).
    #[metric(name = "failure", unit = "{1}")]
    pub failure: Counter<u64>,
    /// Requests permanently refused by downstream (permanent nack).
    #[metric(name = "refused", unit = "{1}")]
    pub refused: Counter<u64>,
}

// ---------------------------------------------------------------------------
// Aggregated state — level-gated enum
// ---------------------------------------------------------------------------

/// Basic-level component metrics: request outcome counters only.
pub struct BasicComponentMetrics {
    pub(crate) consumed_requests: MetricSet<ConsumedRequestMetrics>,
    pub(crate) produced_requests: MetricSet<ProducedRequestMetrics>,
}

/// Normal-level component metrics: request outcome counters.
pub struct NormalComponentMetrics {
    pub(crate) consumed_requests: MetricSet<ConsumedRequestMetrics>,
    pub(crate) produced_requests: MetricSet<ProducedRequestMetrics>,
}

/// Detailed-level component metrics: request counters.
pub struct DetailedComponentMetrics {
    pub(crate) consumed_requests: MetricSet<ConsumedRequestMetrics>,
    pub(crate) produced_requests: MetricSet<ProducedRequestMetrics>,
}

/// Level-gated component metrics state.
///
/// Each variant stores a flat struct with exactly the metric sets required
/// for that level — no `Option` or `Box` needed.  Only created when
/// `MetricLevel >= Basic`.
pub enum ComponentMetricsState {
    /// Basic level: request outcome counters only.
    Basic(BasicComponentMetrics),
    /// Normal level: request counters.
    Normal(NormalComponentMetrics),
    /// Detailed level: request counters.
    Detailed(DetailedComponentMetrics),
}

impl ComponentMetricsState {
    // -- Consumer-side recording --

    /// Record a successful consumption event (Basic+: count).
    #[inline]
    pub fn record_consumed_success(&mut self) {
        match self {
            Self::Basic(m) => m.consumed_requests.success.add(1),
            Self::Normal(m) => m.consumed_requests.success.add(1),
            Self::Detailed(m) => m.consumed_requests.success.add(1),
        }
    }

    /// Record a failed consumption event (Basic+: count).
    #[inline]
    pub fn record_consumed_failure(&mut self) {
        match self {
            Self::Basic(m) => m.consumed_requests.failure.add(1),
            Self::Normal(m) => m.consumed_requests.failure.add(1),
            Self::Detailed(m) => m.consumed_requests.failure.add(1),
        }
    }

    /// Record a permanently refused consumption event (Basic+: count).
    #[inline]
    pub fn record_consumed_refused(&mut self) {
        match self {
            Self::Basic(m) => m.consumed_requests.refused.add(1),
            Self::Normal(m) => m.consumed_requests.refused.add(1),
            Self::Detailed(m) => m.consumed_requests.refused.add(1),
        }
    }

    // -- Producer-side recording --

    /// Record a successful production event (Basic+: count).
    #[inline]
    pub fn record_produced_success(&mut self) {
        match self {
            Self::Basic(m) => m.produced_requests.success.add(1),
            Self::Normal(m) => m.produced_requests.success.add(1),
            Self::Detailed(m) => m.produced_requests.success.add(1),
        }
    }

    /// Record a failed production event (Basic+: count). Transient / retryable (non-permanent nack).
    #[inline]
    pub fn record_produced_failure(&mut self) {
        match self {
            Self::Basic(m) => m.produced_requests.failure.add(1),
            Self::Normal(m) => m.produced_requests.failure.add(1),
            Self::Detailed(m) => m.produced_requests.failure.add(1),
        }
    }

    /// Record a refused production event (Basic+: count). Permanent nack.
    #[inline]
    pub fn record_produced_refused(&mut self) {
        match self {
            Self::Basic(m) => m.produced_requests.refused.add(1),
            Self::Normal(m) => m.produced_requests.refused.add(1),
            Self::Detailed(m) => m.produced_requests.refused.add(1),
        }
    }

    // -- Accessors --

    /// Returns the configured metric level.
    #[inline]
    #[must_use]
    pub fn metric_level(&self) -> MetricLevel {
        match self {
            Self::Basic(_) => MetricLevel::Basic,
            Self::Normal(_) => MetricLevel::Normal,
            Self::Detailed(_) => MetricLevel::Detailed,
        }
    }

    // -- Reporting --

    /// Flush all registered metric sets to the reporter.
    pub fn report(
        &mut self,
        reporter: &mut MetricsReporter,
    ) -> Result<(), otap_df_telemetry::error::Error> {
        match self {
            Self::Basic(m) => {
                reporter.report(&mut m.consumed_requests)?;
                reporter.report(&mut m.produced_requests)?;
            }
            Self::Normal(m) => {
                reporter.report(&mut m.consumed_requests)?;
                reporter.report(&mut m.produced_requests)?;
            }
            Self::Detailed(m) => {
                reporter.report(&mut m.consumed_requests)?;
                reporter.report(&mut m.produced_requests)?;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Handle types (local / shared)
// ---------------------------------------------------------------------------

/// Local (single-threaded) handle to component metrics.
pub type LocalComponentMetricsHandle = Rc<RefCell<ComponentMetricsState>>;
/// Shared (thread-safe) handle to component metrics.
pub type SharedComponentMetricsHandle = Arc<Mutex<ComponentMetricsState>>;

/// Handle to component metrics, supporting both local and shared modes.
#[derive(Clone)]
pub enum ComponentMetricsHandle {
    /// Local (single-threaded) variant.
    Local(LocalComponentMetricsHandle),
    /// Shared (thread-safe) variant.
    Shared(SharedComponentMetricsHandle),
}

impl fmt::Debug for ComponentMetricsHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentMetricsHandle::Local(_) => f.debug_tuple("Local").field(&"...").finish(),
            ComponentMetricsHandle::Shared(_) => f.debug_tuple("Shared").field(&"...").finish(),
        }
    }
}

impl ComponentMetricsHandle {
    /// Report all component metrics to the given reporter.
    #[inline]
    pub fn report(
        &self,
        reporter: &mut MetricsReporter,
    ) -> Result<(), otap_df_telemetry::error::Error> {
        match self {
            ComponentMetricsHandle::Local(h) => match h.try_borrow_mut() {
                Ok(mut state) => state.report(reporter),
                Err(_) => Ok(()),
            },
            ComponentMetricsHandle::Shared(h) => match h.try_lock() {
                Ok(mut state) => state.report(reporter),
                Err(_) => Ok(()),
            },
        }
    }

    /// Record a successful consumption event (Basic+: count).
    #[inline]
    pub fn record_consumed_success(&self) {
        self.with_state(|s| s.record_consumed_success());
    }

    /// Record a failed consumption event (Basic+: count).
    #[inline]
    pub fn record_consumed_failure(&self) {
        self.with_state(|s| s.record_consumed_failure());
    }

    /// Record a permanently refused consumption event (Basic+: count).
    #[inline]
    pub fn record_consumed_refused(&self) {
        self.with_state(|s| s.record_consumed_refused());
    }

    /// Record a successful production event (Basic+: count).
    #[inline]
    pub fn record_produced_success(&self) {
        self.with_state(|s| s.record_produced_success());
    }

    /// Record a failed production event (Basic+: count). Transient / retryable (non-permanent nack).
    #[inline]
    pub fn record_produced_failure(&self) {
        self.with_state(|s| s.record_produced_failure());
    }

    /// Record a refused production event (Basic+: count). Permanent nack.
    #[inline]
    pub fn record_produced_refused(&self) {
        self.with_state(|s| s.record_produced_refused());
    }

    /// Returns the configured metric level.
    #[inline]
    #[must_use]
    pub fn metric_level(&self) -> MetricLevel {
        match self {
            ComponentMetricsHandle::Local(h) => {
                h.borrow().metric_level()
            }
            ComponentMetricsHandle::Shared(h) => {
                h.lock().map(|s| s.metric_level()).unwrap_or_default()
            }
        }
    }

    #[inline]
    fn with_state(&self, f: impl FnOnce(&mut ComponentMetricsState)) {
        match self {
            ComponentMetricsHandle::Local(h) => {
                if let Ok(mut state) = h.try_borrow_mut() {
                    f(&mut state);
                }
            }
            ComponentMetricsHandle::Shared(h) => {
                if let Ok(mut state) = h.try_lock() {
                    f(&mut state);
                }
            }
        }
    }
}

/// Record produced metrics for an incoming control message (Basic+).
///
/// Call this in node run loops when a control message is received. Records
/// `produced.requests{outcome=success}` for Ack and `produced.requests{outcome=refused}` for Nack.
/// Skipped entirely at `MetricLevel::None`.
pub(crate) fn record_produced_for_control_msg<PData>(
    msg: &crate::control::NodeControlMsg<PData>,
) {
    use crate::control::NodeControlMsg;
    use crate::entity_context::{current_component_metrics, current_metric_level};

    if current_metric_level() >= MetricLevel::Basic {
        if let Some(handle) = current_component_metrics() {
            match msg {
                NodeControlMsg::Ack(_) => {
                    handle.record_produced_success();
                }
                NodeControlMsg::Nack(nack) => {
                    if nack.permanent {
                        handle.record_produced_refused();
                    } else {
                        handle.record_produced_failure();
                    }
                }
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// W13 — Unit tests for level-gated component metrics
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::attributes::NodeAttributeSet;
    use crate::entity_context::NodeTelemetryHandle;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    /// Helper: create an `Option<ComponentMetricsHandle>` at the given level using the real registry path.
    fn handle_at_level(level: MetricLevel) -> Option<ComponentMetricsHandle> {
        let registry = TelemetryRegistryHandle::new();
        let entity_key = registry.register_entity(NodeAttributeSet::default());
        let handle = NodeTelemetryHandle::new(registry, entity_key);
        handle.register_component_metrics(level)
    }

    // -- None: nothing is registered --

    #[test]
    fn none_level_no_metric_sets() {
        let h = handle_at_level(MetricLevel::None);
        assert!(h.is_none(), "None level should not create a handle");
    }

    // -- Basic: outcome counters present, no bytes/duration --

    #[test]
    fn basic_level_outcome_counters() {
        let h = handle_at_level(MetricLevel::Basic).expect("Basic should create a handle");
        assert_eq!(h.metric_level(), MetricLevel::Basic);
        h.record_consumed_success();
        h.record_consumed_failure();
        h.record_consumed_refused();
        h.record_produced_success();
        h.record_produced_failure();
        h.record_produced_refused();

        h.with_state(|s| {
            let m = match s {
                ComponentMetricsState::Basic(m) => m,
                _ => panic!("expected Basic variant"),
            };
            assert_eq!(m.consumed_requests.success.get(), 1);
            assert_eq!(m.consumed_requests.failure.get(), 1);
            assert_eq!(m.consumed_requests.refused.get(), 1);
            assert_eq!(m.produced_requests.success.get(), 1);
            assert_eq!(m.produced_requests.failure.get(), 1);
            assert_eq!(m.produced_requests.refused.get(), 1);
        });
    }

    // -- Normal: same as Basic (byte counters removed) --

    #[test]
    fn normal_level_outcome_counters() {
        let h = handle_at_level(MetricLevel::Normal).expect("Normal should create a handle");
        assert_eq!(h.metric_level(), MetricLevel::Normal);
        h.record_consumed_success();
        h.record_produced_success();

        h.with_state(|s| {
            let m = match s {
                ComponentMetricsState::Normal(m) => m,
                _ => panic!("expected Normal variant"),
            };
            assert_eq!(m.consumed_requests.success.get(), 1);
            assert_eq!(m.produced_requests.success.get(), 1);
        });
    }

    // -- Detailed: same counters as Basic/Normal (duration moved to channel metrics) --

    #[test]
    fn detailed_level_outcome_counters() {
        let h = handle_at_level(MetricLevel::Detailed).expect("Detailed should create a handle");
        assert_eq!(h.metric_level(), MetricLevel::Detailed);
        h.record_consumed_success();
        h.record_consumed_failure();
        h.record_consumed_refused();

        h.with_state(|s| {
            let m = match s {
                ComponentMetricsState::Detailed(m) => m,
                _ => panic!("expected Detailed variant"),
            };
            assert_eq!(m.consumed_requests.success.get(), 1);
            assert_eq!(m.consumed_requests.failure.get(), 1);
            assert_eq!(m.consumed_requests.refused.get(), 1);
        });
    }
}
