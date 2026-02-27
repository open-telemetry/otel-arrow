// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component-level pipeline metrics for tracking data consumed and produced
//! by each node (receiver, processor, exporter).
//!
//! Metric sets are level-gated by [`MetricLevel`]:
//!
//! - **Basic+**: Request outcome counters (consumed success/failure/refused, produced success/refused).
//! - **Normal+**: Forward-path byte counters (consumed bytes, produced bytes).
//! - **Detailed**: Duration counters per outcome (consumed duration success/failure/refused).
//!
//! At `MetricLevel::None`, no metric sets are registered.

use crate::control::MetricLevel;
use otap_df_telemetry::instrument::{Counter, Mmsc};
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
    /// Requests refused by downstream (all nacks).
    #[metric(name = "refused", unit = "{1}")]
    pub refused: Counter<u64>,
}

// -- Normal+ : forward-path byte counts --

/// Consumer byte counter (Normal+). Recorded on the forward path when data
/// enters the node — no outcome breakdown.
#[metric_set(name = "component.consumed")]
#[derive(Debug, Default, Clone)]
pub struct ConsumedBytesMetrics {
    /// Cumulative bytes entering this node.
    #[metric(name = "bytes", unit = "{By}")]
    pub bytes: Counter<u64>,
}

/// Producer byte counter (Normal+). Recorded on the forward path when data
/// is sent — no outcome breakdown.
#[metric_set(name = "component.produced")]
#[derive(Debug, Default, Clone)]
pub struct ProducedBytesMetrics {
    /// Cumulative bytes sent by this node.
    #[metric(name = "bytes", unit = "{By}")]
    pub bytes: Counter<u64>,
}

// -- Detailed : duration per outcome --

/// Consumer duration summary (Detailed). Recorded at ack/nack time using the
/// entry frame's `time_ns`. Tracks min/max/sum/count across all outcomes.
#[metric_set(name = "component.consumed")]
#[derive(Debug, Default, Clone)]
pub struct ConsumedDurationMetrics {
    /// Duration (ns) of consumed requests (min/max/sum/count).
    #[metric(unit = "ns")]
    pub duration_ns: Mmsc,
}

// ---------------------------------------------------------------------------
// Aggregated state — level-gated enum
// ---------------------------------------------------------------------------

/// Basic-level component metrics: request outcome counters only.
pub struct BasicComponentMetrics {
    pub(crate) consumed_requests: MetricSet<ConsumedRequestMetrics>,
    pub(crate) produced_requests: MetricSet<ProducedRequestMetrics>,
}

/// Normal-level component metrics: request outcome counters + byte counters.
pub struct NormalComponentMetrics {
    pub(crate) consumed_requests: MetricSet<ConsumedRequestMetrics>,
    pub(crate) produced_requests: MetricSet<ProducedRequestMetrics>,
    pub(crate) consumed_bytes: MetricSet<ConsumedBytesMetrics>,
    pub(crate) produced_bytes: MetricSet<ProducedBytesMetrics>,
}

/// Detailed-level component metrics: request counters + byte counters + duration counters.
pub struct DetailedComponentMetrics {
    pub(crate) consumed_requests: MetricSet<ConsumedRequestMetrics>,
    pub(crate) produced_requests: MetricSet<ProducedRequestMetrics>,
    pub(crate) consumed_bytes: MetricSet<ConsumedBytesMetrics>,
    pub(crate) produced_bytes: MetricSet<ProducedBytesMetrics>,
    pub(crate) consumed_duration: MetricSet<ConsumedDurationMetrics>,
}

/// Level-gated component metrics state.
///
/// Each variant stores a flat struct with exactly the metric sets required
/// for that level — no `Option` or `Box` needed.  Only created when
/// `MetricLevel >= Basic`.
pub enum ComponentMetricsState {
    /// Basic level: request outcome counters only.
    Basic(BasicComponentMetrics),
    /// Normal level: request + byte counters.
    Normal(NormalComponentMetrics),
    /// Detailed level: request + byte + duration counters.
    Detailed(DetailedComponentMetrics),
}

impl ComponentMetricsState {
    // -- Consumer-side recording --

    /// Record a successful consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_success(&mut self, duration_ns: u64) {
        match self {
            Self::Basic(m) => m.consumed_requests.success.add(1),
            Self::Normal(m) => m.consumed_requests.success.add(1),
            Self::Detailed(m) => {
                m.consumed_requests.success.add(1);
                m.consumed_duration.duration_ns.record(duration_ns as f64);
            }
        }
    }

    /// Record a failed consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_failure(&mut self, duration_ns: u64) {
        match self {
            Self::Basic(m) => m.consumed_requests.failure.add(1),
            Self::Normal(m) => m.consumed_requests.failure.add(1),
            Self::Detailed(m) => {
                m.consumed_requests.failure.add(1);
                m.consumed_duration.duration_ns.record(duration_ns as f64);
            }
        }
    }

    /// Record a permanently refused consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_refused(&mut self, duration_ns: u64) {
        match self {
            Self::Basic(m) => m.consumed_requests.refused.add(1),
            Self::Normal(m) => m.consumed_requests.refused.add(1),
            Self::Detailed(m) => {
                m.consumed_requests.refused.add(1);
                m.consumed_duration.duration_ns.record(duration_ns as f64);
            }
        }
    }

    /// Record consumed bytes on the forward path (Normal+).
    #[inline]
    pub fn record_consumed_bytes(&mut self, bytes: u64) {
        match self {
            Self::Basic(_) => {}
            Self::Normal(m) => m.consumed_bytes.bytes.add(bytes),
            Self::Detailed(m) => m.consumed_bytes.bytes.add(bytes),
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

    /// Record a refused production event (Basic+: count).
    #[inline]
    pub fn record_produced_refused(&mut self) {
        match self {
            Self::Basic(m) => m.produced_requests.refused.add(1),
            Self::Normal(m) => m.produced_requests.refused.add(1),
            Self::Detailed(m) => m.produced_requests.refused.add(1),
        }
    }

    /// Record produced bytes on the forward path (Normal+).
    #[inline]
    pub fn record_produced_bytes(&mut self, bytes: u64) {
        match self {
            Self::Basic(_) => {}
            Self::Normal(m) => m.produced_bytes.bytes.add(bytes),
            Self::Detailed(m) => m.produced_bytes.bytes.add(bytes),
        }
    }

    // -- Accessors --

    /// Returns the configured metric level.
    #[inline]
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
                reporter.report(&mut m.consumed_bytes)?;
                reporter.report(&mut m.produced_bytes)?;
            }
            Self::Detailed(m) => {
                reporter.report(&mut m.consumed_requests)?;
                reporter.report(&mut m.produced_requests)?;
                reporter.report(&mut m.consumed_bytes)?;
                reporter.report(&mut m.produced_bytes)?;
                reporter.report(&mut m.consumed_duration)?;
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

    /// Record a successful consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_success(&self, duration_ns: u64) {
        self.with_state(|s| s.record_consumed_success(duration_ns));
    }

    /// Record a failed consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_failure(&self, duration_ns: u64) {
        self.with_state(|s| s.record_consumed_failure(duration_ns));
    }

    /// Record a permanently refused consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_refused(&self, duration_ns: u64) {
        self.with_state(|s| s.record_consumed_refused(duration_ns));
    }

    /// Record consumed bytes on the forward path (Normal+).
    #[inline]
    pub fn record_consumed_bytes(&self, bytes: u64) {
        self.with_state(|s| s.record_consumed_bytes(bytes));
    }

    /// Record a successful production event (Basic+: count).
    #[inline]
    pub fn record_produced_success(&self) {
        self.with_state(|s| s.record_produced_success());
    }

    /// Record a refused production event (Basic+: count).
    #[inline]
    pub fn record_produced_refused(&self) {
        self.with_state(|s| s.record_produced_refused());
    }

    /// Record produced bytes on the forward path (Normal+).
    #[inline]
    pub fn record_produced_bytes(&self, bytes: u64) {
        self.with_state(|s| s.record_produced_bytes(bytes));
    }

    /// Returns the configured metric level.
    #[inline]
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
                NodeControlMsg::Nack(_) => {
                    handle.record_produced_refused();
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
        h.record_consumed_success(100);
        h.record_consumed_failure(50);
        h.record_consumed_refused(25);
        h.record_produced_success();
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
            assert_eq!(m.produced_requests.refused.get(), 1);
        });
    }

    // -- Normal: + byte counters --

    #[test]
    fn normal_level_bytes_counters() {
        let h = handle_at_level(MetricLevel::Normal).expect("Normal should create a handle");
        assert_eq!(h.metric_level(), MetricLevel::Normal);
        h.record_consumed_bytes(1024);
        h.record_produced_bytes(512);
        h.record_consumed_success(0);

        h.with_state(|s| {
            let m = match s {
                ComponentMetricsState::Normal(m) => m,
                _ => panic!("expected Normal variant"),
            };
            assert_eq!(m.consumed_requests.success.get(), 1);
            assert_eq!(m.consumed_bytes.bytes.get(), 1024);
            assert_eq!(m.produced_bytes.bytes.get(), 512);
        });
    }

    // -- Detailed: + duration counters --

    #[test]
    fn detailed_level_duration_counters() {
        let h = handle_at_level(MetricLevel::Detailed).expect("Detailed should create a handle");
        assert_eq!(h.metric_level(), MetricLevel::Detailed);
        h.record_consumed_success(100);  // duration_ns = 100
        h.record_consumed_failure(50);
        h.record_consumed_refused(25);

        h.with_state(|s| {
            let m = match s {
                ComponentMetricsState::Detailed(m) => m,
                _ => panic!("expected Detailed variant"),
            };
            assert_eq!(m.consumed_requests.success.get(), 1);
            assert_eq!(m.consumed_requests.failure.get(), 1);
            assert_eq!(m.consumed_requests.refused.get(), 1);
            let snap = m.consumed_duration.duration_ns.get();
            assert_eq!(snap.count, 3);
            assert_eq!(snap.sum, 175.0); // 100 + 50 + 25
            assert_eq!(snap.min, 25.0);
            assert_eq!(snap.max, 100.0);
        });
    }
}
