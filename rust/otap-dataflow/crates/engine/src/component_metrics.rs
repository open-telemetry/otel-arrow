// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component-level pipeline metrics for tracking data consumed and produced
//! by each node (receiver, processor, exporter).
//!
//! Five metric sets:
//! - consumed.success  — data successfully processed by this node
//! - consumed.failure  — data that failed processing (transient / retryable)
//! - consumed.refused  — data permanently rejected
//! - produced.success  — data acknowledged by a downstream node
//! - produced.refused  — data refused/nacked by a downstream node

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Metric set definitions
// ---------------------------------------------------------------------------

/// Metrics for data successfully consumed (processed) by a component.
#[metric_set(name = "component.consumed.success")]
#[derive(Debug, Default, Clone)]
pub struct ConsumedSuccessMetrics {
    /// Cumulative bytes consumed successfully.
    #[metric(name = "bytes", unit = "{By}")]
    pub bytes: Counter<u64>,
    /// Cumulative processing duration in nanoseconds.
    #[metric(name = "duration_ns", unit = "ns")]
    pub duration_ns: Counter<u64>,
}

/// Metrics for data that failed processing (transient / retryable).
#[metric_set(name = "component.consumed.failure")]
#[derive(Debug, Default, Clone)]
pub struct ConsumedFailureMetrics {
    /// Cumulative bytes that failed processing.
    #[metric(name = "bytes", unit = "{By}")]
    pub bytes: Counter<u64>,
    /// Cumulative duration before failure was reported.
    #[metric(name = "duration_ns", unit = "ns")]
    pub duration_ns: Counter<u64>,
}

/// Metrics for data permanently refused by a component.
#[metric_set(name = "component.consumed.refused")]
#[derive(Debug, Default, Clone)]
pub struct ConsumedRefusedMetrics {
    /// Cumulative bytes permanently refused.
    #[metric(name = "bytes", unit = "{By}")]
    pub bytes: Counter<u64>,
    /// Cumulative duration before refusal was reported.
    #[metric(name = "duration_ns", unit = "ns")]
    pub duration_ns: Counter<u64>,
}

/// Metrics for data acknowledged by a downstream component.
#[metric_set(name = "component.produced.success")]
#[derive(Debug, Default, Clone)]
pub struct ProducedSuccessMetrics {
    /// Cumulative bytes acknowledged by downstream.
    #[metric(name = "bytes", unit = "{By}")]
    pub bytes: Counter<u64>,
}

/// Metrics for data refused by a downstream component.
#[metric_set(name = "component.produced.refused")]
#[derive(Debug, Default, Clone)]
pub struct ProducedRefusedMetrics {
    /// Cumulative bytes refused by downstream.
    #[metric(name = "bytes", unit = "{By}")]
    pub bytes: Counter<u64>,
}

// ---------------------------------------------------------------------------
// Aggregated state
// ---------------------------------------------------------------------------

/// Aggregated state holding all five component metric sets.
pub struct ComponentMetricsState {
    consumed_success: MetricSet<ConsumedSuccessMetrics>,
    consumed_failure: MetricSet<ConsumedFailureMetrics>,
    consumed_refused: MetricSet<ConsumedRefusedMetrics>,
    produced_success: MetricSet<ProducedSuccessMetrics>,
    produced_refused: MetricSet<ProducedRefusedMetrics>,
}

impl ComponentMetricsState {
    pub(crate) const fn new(
        consumed_success: MetricSet<ConsumedSuccessMetrics>,
        consumed_failure: MetricSet<ConsumedFailureMetrics>,
        consumed_refused: MetricSet<ConsumedRefusedMetrics>,
        produced_success: MetricSet<ProducedSuccessMetrics>,
        produced_refused: MetricSet<ProducedRefusedMetrics>,
    ) -> Self {
        Self {
            consumed_success,
            consumed_failure,
            consumed_refused,
            produced_success,
            produced_refused,
        }
    }

    // -- Consumer-side recording --

    /// Record a successful consumption event.
    #[inline]
    pub fn record_consumed_success(&mut self, bytes: u64, duration_ns: u64) {
        self.consumed_success.bytes.add(bytes);
        self.consumed_success.duration_ns.add(duration_ns);
    }

    /// Record a failed consumption event (transient / retryable).
    #[inline]
    pub fn record_consumed_failure(&mut self, bytes: u64, duration_ns: u64) {
        self.consumed_failure.bytes.add(bytes);
        self.consumed_failure.duration_ns.add(duration_ns);
    }

    /// Record a permanently refused consumption event.
    #[inline]
    pub fn record_consumed_refused(&mut self, bytes: u64, duration_ns: u64) {
        self.consumed_refused.bytes.add(bytes);
        self.consumed_refused.duration_ns.add(duration_ns);
    }

    // -- Producer-side recording --

    /// Record a successful production event (downstream ack).
    #[inline]
    pub fn record_produced_success(&mut self, bytes: u64) {
        self.produced_success.bytes.add(bytes);
    }

    /// Record a refused production event (downstream nack).
    #[inline]
    pub fn record_produced_refused(&mut self, bytes: u64) {
        self.produced_refused.bytes.add(bytes);
    }

    // -- Reporting --

    /// Flush all five metric sets to the reporter.
    pub fn report(
        &mut self,
        reporter: &mut MetricsReporter,
    ) -> Result<(), otap_df_telemetry::error::Error> {
        reporter.report(&mut self.consumed_success)?;
        reporter.report(&mut self.consumed_failure)?;
        reporter.report(&mut self.consumed_refused)?;
        reporter.report(&mut self.produced_success)?;
        reporter.report(&mut self.produced_refused)?;
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

    /// Record a successful consumption event.
    #[inline]
    pub fn record_consumed_success(&self, bytes: u64, duration_ns: u64) {
        self.with_state(|s| s.record_consumed_success(bytes, duration_ns));
    }

    /// Record a failed consumption event (transient / retryable).
    #[inline]
    pub fn record_consumed_failure(&self, bytes: u64, duration_ns: u64) {
        self.with_state(|s| s.record_consumed_failure(bytes, duration_ns));
    }

    /// Record a permanently refused consumption event.
    #[inline]
    pub fn record_consumed_refused(&self, bytes: u64, duration_ns: u64) {
        self.with_state(|s| s.record_consumed_refused(bytes, duration_ns));
    }

    /// Record a successful production event (downstream ack).
    #[inline]
    pub fn record_produced_success(&self, bytes: u64) {
        self.with_state(|s| s.record_produced_success(bytes));
    }

    /// Record a refused production event (downstream nack).
    #[inline]
    pub fn record_produced_refused(&self, bytes: u64) {
        self.with_state(|s| s.record_produced_refused(bytes));
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

/// Record produced metrics for an incoming control message.
///
/// Call this in node run loops when a control message is received. Records
/// `produced.success` for Ack messages and `produced.refused` for Nack messages,
/// using `calldata.req_bytes` from the message.
pub(crate) fn record_produced_for_control_msg<PData>(
    msg: &crate::control::NodeControlMsg<PData>,
) {
    use crate::control::NodeControlMsg;
    use crate::entity_context::current_component_metrics;

    if let Some(handle) = current_component_metrics() {
        match msg {
            NodeControlMsg::Ack(ack) => {
                handle.record_produced_success(ack.calldata.req_bytes);
            }
            NodeControlMsg::Nack(nack) => {
                handle.record_produced_refused(nack.calldata.req_bytes);
            }
            _ => {}
        }
    }
}
