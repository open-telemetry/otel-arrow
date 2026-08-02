// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry for engine-owned rate admission.
//!
//! # Why the hot path does not touch a `MetricSet`
//!
//! `Counter<u64>` inside a `MetricSet` is a plain, non-atomic `u64` and requires
//! `&mut` to increment. A gate shared by many worker threads would therefore
//! need a mutex around the metric set, and that mutex would be acquired on every
//! admission decision -- turning an overloaded receiver's ingress path into a
//! single contended lock exactly when it is most loaded. Observe-only mode would
//! be the worst case: every request is over limit, so every request would
//! contend.
//!
//! Instead, gates increment lock-free `AtomicU64` staging counters and the
//! engine's periodic reporting task drains them into the real metric set. The
//! mutex is acquired once per reporting interval, never per request.
//!
//! # Why admissions are not counted here
//!
//! The staging counters cover refusals only (`would_throttle`, `throttle`,
//! `oversized`). Admitted requests are already counted by component telemetry
//! (`receiver.otlp.requests.started` and friends), so counting them again would
//! duplicate an existing series *and* add a second contended cache line beside
//! the bucket's atomic on the path that must stay cheapest -- the normal,
//! not-throttled one. Refusals are the signal operators actually need, and by
//! construction they only appear when something is already wrong.

use crate::admission::AdmissionDimension;
use crate::context::PipelineContext;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Why an admission point declined, or would have declined, a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum AdmissionRefusal {
    /// Over limit, but enforcement was inactive so the request was still admitted.
    WouldThrottle,
    /// Over limit and refused; the client may retry later.
    Throttle,
    /// Larger than total burst capacity; retrying cannot help.
    Oversized,
}

impl AdmissionRefusal {
    /// Stable index into the staging counter array.
    const fn index(self) -> usize {
        match self {
            Self::WouldThrottle => 0,
            Self::Throttle => 1,
            Self::Oversized => 2,
        }
    }

    /// Refusals in staging-index order.
    const ALL: [Self; REFUSAL_KINDS] = [Self::WouldThrottle, Self::Throttle, Self::Oversized];
}

const REFUSAL_KINDS: usize = 3;

/// Datapoint attributes for a rate admission refusal.
///
/// Cardinality is bounded by construction: two dimensions times three refusal
/// kinds, six series per receiver node at most.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct AdmissionRefusalAttributes {
    /// Weight dimension the limiter meters.
    pub dimension: AdmissionDimension,
    /// Why the request was declined.
    pub refusal: AdmissionRefusal,
}

/// Rate admission refusals, scoped to the receiver node entity.
#[metric_set(
    name = "admission.rate_limiter",
    measurement_attributes = AdmissionRefusalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct RateAdmissionMetrics {
    /// Admission attempts declined, or that would have been declined in observe-only mode.
    #[metric(unit = "{decision}")]
    pub refusals: Counter<u64>,
}

/// Lock-free staging counters written by admission gates.
///
/// One `AtomicU64` per refusal kind. Increments are `Relaxed` because the only
/// consumer is a periodic drain that needs the eventual total, not an ordering
/// relationship with any other memory.
#[derive(Debug, Default)]
pub struct RefusalCounters {
    counts: [AtomicU64; REFUSAL_KINDS],
}

impl RefusalCounters {
    /// Records one refusal.
    #[inline]
    pub fn record(&self, refusal: AdmissionRefusal) {
        let _ = self.counts[refusal.index()].fetch_add(1, Ordering::Relaxed);
    }

    /// Takes and clears the accumulated counts.
    pub(crate) fn drain(&self) -> [u64; REFUSAL_KINDS] {
        let mut drained = [0_u64; REFUSAL_KINDS];
        for (slot, counter) in drained.iter_mut().zip(self.counts.iter()) {
            *slot = counter.swap(0, Ordering::Relaxed);
        }
        drained
    }
}

/// Engine-owned reporting handle for one bound admission gate.
///
/// Components never see this type. The engine creates it when it binds a gate
/// and hands it to the periodic reporting task, so adopting admission control
/// costs a component exactly zero telemetry code.
#[derive(Clone)]
pub(crate) struct AdmissionMetricsHandle {
    dimension: AdmissionDimension,
    counters: Arc<RefusalCounters>,
    metrics: Arc<Mutex<MeasurementMetricSet<RateAdmissionMetrics>>>,
}

impl AdmissionMetricsHandle {
    /// Registers the metric set for the current node entity.
    pub(crate) fn new(
        pipeline_ctx: &PipelineContext,
        dimension: AdmissionDimension,
        counters: Arc<RefusalCounters>,
    ) -> Self {
        Self {
            dimension,
            counters,
            metrics: Arc::new(Mutex::new(RateAdmissionMetrics::register(pipeline_ctx))),
        }
    }

    /// Drains staged counts into the metric set and reports it.
    ///
    /// Skips silently when the metric set is momentarily locked, matching the
    /// channel-metrics contract: a reporting tick must never block ingress, and
    /// counts are cumulative so the next tick reports what this one missed.
    pub(crate) fn report(
        &self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        let Ok(mut metrics) = self.metrics.try_lock() else {
            return Ok(());
        };
        self.apply_drain(&mut metrics);
        metrics_reporter.report_measurement(&mut metrics)
    }

    /// Drains staged counts and returns final snapshots for shutdown reporting.
    pub(crate) fn terminal_snapshots(&self) -> Vec<MetricSetSnapshot> {
        // Shutdown has no later reporting tick that can recover skipped counts,
        // so the terminal path waits instead of using periodic best effort.
        let mut metrics = self
            .metrics
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        self.apply_drain(&mut metrics);
        metrics.terminal_snapshots()
    }

    fn apply_drain(&self, metrics: &mut MeasurementMetricSet<RateAdmissionMetrics>) {
        let drained = self.counters.drain();
        for (refusal, count) in AdmissionRefusal::ALL.into_iter().zip(drained) {
            if count == 0 {
                continue;
            }
            metrics
                .with(AdmissionRefusalAttributes {
                    dimension: self.dimension,
                    refusal,
                })
                .refusals
                .add(count);
        }
    }
}

/// Collects admission metric handles during pipeline construction.
///
/// Mirrors `ChannelMetricsRegistry`: handles are gathered while nodes are being
/// built and then moved into the runtime's periodic reporting task in one go.
#[derive(Default)]
pub(crate) struct AdmissionMetricsRegistry {
    handles: Vec<AdmissionMetricsHandle>,
}

impl AdmissionMetricsRegistry {
    pub(crate) fn register(&mut self, handle: AdmissionMetricsHandle) {
        self.handles.push(handle);
    }

    pub(crate) fn into_handles(self) -> Vec<AdmissionMetricsHandle> {
        self.handles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: refusals of every kind are staged and then drained twice.
    /// Guarantees: a drain returns exactly what was staged since the previous drain and
    /// resets the counters, so periodic reporting neither loses nor double-counts.
    #[test]
    fn draining_returns_staged_counts_once() {
        let counters = RefusalCounters::default();

        counters.record(AdmissionRefusal::WouldThrottle);
        counters.record(AdmissionRefusal::WouldThrottle);
        counters.record(AdmissionRefusal::Throttle);
        counters.record(AdmissionRefusal::Oversized);

        assert_eq!(counters.drain(), [2, 1, 1]);
        assert_eq!(counters.drain(), [0, 0, 0]);
    }

    /// Scenario: many threads stage refusals against one shared counter block.
    /// Guarantees: no increment is lost under contention, which is what allows the
    /// hot path to stay lock-free instead of serialising on a metrics mutex.
    #[test]
    fn concurrent_staging_loses_no_increments() {
        let counters = Arc::new(RefusalCounters::default());

        std::thread::scope(|scope| {
            for _ in 0..8 {
                let counters = Arc::clone(&counters);
                let _ = scope.spawn(move || {
                    for _ in 0..1_000 {
                        counters.record(AdmissionRefusal::Throttle);
                    }
                });
            }
        });

        assert_eq!(counters.drain(), [0, 8_000, 0]);
    }

    /// Scenario: the refusal indices are used to address the staging array.
    /// Guarantees: each refusal kind maps to a distinct, in-bounds slot, so adding a
    /// variant without extending the array cannot silently alias another counter.
    #[test]
    fn refusal_indices_are_distinct_and_in_bounds() {
        let indices: Vec<usize> = AdmissionRefusal::ALL
            .into_iter()
            .map(AdmissionRefusal::index)
            .collect();

        assert_eq!(indices, vec![0, 1, 2]);
        assert_eq!(indices.len(), REFUSAL_KINDS);
    }

    /// Scenario: a bound gate stages a refusal before the runtime's terminal flush.
    /// Guarantees: the engine-owned handle publishes the common metric name with
    /// bounded dimension and refusal attributes, then drains it exactly once.
    #[test]
    fn terminal_snapshot_uses_common_bounded_vocabulary_once() {
        let (pipeline_ctx, _registry) = crate::testing::test_pipeline_ctx();
        let counters = Arc::new(RefusalCounters::default());
        let handle = AdmissionMetricsHandle::new(
            &pipeline_ctx,
            AdmissionDimension::Messages,
            Arc::clone(&counters),
        );
        counters.record(AdmissionRefusal::Throttle);

        let snapshots = handle.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].descriptor().name, "admission.rate_limiter");
        assert_eq!(
            snapshots[0].measurement_attribute_value("dimension"),
            Some("messages")
        );
        assert_eq!(
            snapshots[0].measurement_attribute_value("refusal"),
            Some("throttle")
        );
        assert!(handle.terminal_snapshots().is_empty());
    }
}
