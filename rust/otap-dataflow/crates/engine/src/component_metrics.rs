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

/// Consumer duration counters (Detailed). Recorded at ack/nack time using the
/// entry frame's `time_ns`.
/// TODO: Replace Counter<u64> with Histogram<f64> once otap-df-telemetry supports histograms.
#[metric_set(name = "component.consumed.duration")]
#[derive(Debug, Default, Clone)]
pub struct ConsumedDurationMetrics {
    /// Cumulative duration (ns) for successful consumption.
    #[metric(name = "success_ns", unit = "ns")]
    pub success_ns: Counter<u64>,
    /// Cumulative duration (ns) for failed consumption.
    #[metric(name = "failure_ns", unit = "ns")]
    pub failure_ns: Counter<u64>,
    /// Cumulative duration (ns) for refused consumption.
    #[metric(name = "refused_ns", unit = "ns")]
    pub refused_ns: Counter<u64>,
}

// ---------------------------------------------------------------------------
// Aggregated state — level-gated
// ---------------------------------------------------------------------------

/// Aggregated state holding level-gated component metric sets.
/// Only the metric sets appropriate for the configured `MetricLevel` are populated.
pub struct ComponentMetricsState {
    pub(crate) metric_level: MetricLevel,
    // Basic+ (always present when metric_level >= Basic)
    pub(crate) consumed_requests: Option<MetricSet<ConsumedRequestMetrics>>,
    pub(crate) produced_requests: Option<MetricSet<ProducedRequestMetrics>>,
    // Normal+
    pub(crate) consumed_bytes: Option<MetricSet<ConsumedBytesMetrics>>,
    pub(crate) produced_bytes: Option<MetricSet<ProducedBytesMetrics>>,
    // Detailed
    pub(crate) consumed_duration: Option<MetricSet<ConsumedDurationMetrics>>,
}

impl ComponentMetricsState {
    // -- Consumer-side recording --

    /// Record a successful consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_success(&mut self, duration_ns: u64) {
        if let Some(ref mut m) = self.consumed_requests {
            m.success.add(1);
        }
        if let Some(ref mut m) = self.consumed_duration {
            m.success_ns.add(duration_ns);
        }
    }

    /// Record a failed consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_failure(&mut self, duration_ns: u64) {
        if let Some(ref mut m) = self.consumed_requests {
            m.failure.add(1);
        }
        if let Some(ref mut m) = self.consumed_duration {
            m.failure_ns.add(duration_ns);
        }
    }

    /// Record a permanently refused consumption event (Basic+: count, Detailed: + duration).
    #[inline]
    pub fn record_consumed_refused(&mut self, duration_ns: u64) {
        if let Some(ref mut m) = self.consumed_requests {
            m.refused.add(1);
        }
        if let Some(ref mut m) = self.consumed_duration {
            m.refused_ns.add(duration_ns);
        }
    }

    /// Record consumed bytes on the forward path (Normal+).
    #[inline]
    pub fn record_consumed_bytes(&mut self, bytes: u64) {
        if let Some(ref mut m) = self.consumed_bytes {
            m.bytes.add(bytes);
        }
    }

    // -- Producer-side recording --

    /// Record a successful production event (Basic+: count).
    #[inline]
    pub fn record_produced_success(&mut self) {
        if let Some(ref mut m) = self.produced_requests {
            m.success.add(1);
        }
    }

    /// Record a refused production event (Basic+: count).
    #[inline]
    pub fn record_produced_refused(&mut self) {
        if let Some(ref mut m) = self.produced_requests {
            m.refused.add(1);
        }
    }

    /// Record produced bytes on the forward path (Normal+).
    #[inline]
    pub fn record_produced_bytes(&mut self, bytes: u64) {
        if let Some(ref mut m) = self.produced_bytes {
            m.bytes.add(bytes);
        }
    }

    // -- Accessors --

    /// Returns the configured metric level.
    #[inline]
    pub fn metric_level(&self) -> MetricLevel {
        self.metric_level
    }

    // -- Reporting --

    /// Flush all registered metric sets to the reporter.
    pub fn report(
        &mut self,
        reporter: &mut MetricsReporter,
    ) -> Result<(), otap_df_telemetry::error::Error> {
        if let Some(ref mut m) = self.consumed_requests {
            reporter.report(m)?;
        }
        if let Some(ref mut m) = self.produced_requests {
            reporter.report(m)?;
        }
        if let Some(ref mut m) = self.consumed_bytes {
            reporter.report(m)?;
        }
        if let Some(ref mut m) = self.produced_bytes {
            reporter.report(m)?;
        }
        if let Some(ref mut m) = self.consumed_duration {
            reporter.report(m)?;
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
