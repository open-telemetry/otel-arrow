// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in compute-duration timing for processors.
//!
//! Processors that perform meaningful synchronous compute can add a
//! [`ComputeDuration`] field and call [`ComputeDuration::timed`] to
//! measure the wall-clock duration of that work.  Timing is gated on
//! the `PROCESS_DURATION` interest at the normal metric level.
//!
//! The closure-based API structurally prevents timing from spanning
//! `.await` points.

use std::cell::Cell;

use crate::Interests;
use otap_df_telemetry::instrument::{Mmsc, Timer};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

use crate::context::PipelineContext;

/// Metric set containing a single compute-duration instrument.
#[metric_set(name = "processor.compute")]
#[derive(Debug, Default, Clone)]
pub struct ComputeDurationMetrics {
    /// Wall-clock duration of the processor compute section, in nanoseconds.
    #[metric(name = "compute.duration", unit = "ns")]
    pub compute_duration: Mmsc,
}

/// Wrapper providing interests-gated duration recording and reporting.
pub struct ComputeDuration {
    metrics: MetricSet<ComputeDurationMetrics>,
    /// Accumulator for duration samples recorded via [`timed`](Self::timed).
    /// Uses `Cell` for interior mutability so `timed` can take `&self`,
    /// allowing callers to hold disjoint `&mut` borrows of sibling fields.
    accumulator: Cell<Mmsc>,
}

impl ComputeDuration {
    /// Register a new compute-duration metric set on the given pipeline.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            metrics: pipeline_ctx.register_metrics::<ComputeDurationMetrics>(),
            accumulator: Cell::new(Mmsc::default()),
        }
    }

    /// Time a synchronous closure if interests includes
    /// `PROCESS_DURATION`, otherwise just call `f` directly.
    ///
    /// The closure-based API structurally prevents the timer from
    /// being held across `.await` — the closure is `FnOnce`, not
    /// async, so the compiler enforces that only synchronous work is
    /// measured.
    #[inline]
    pub fn timed<T>(&self, interests: Interests, f: impl FnOnce() -> T) -> T {
        if interests.contains(Interests::PROCESS_DURATION) {
            let timer = Timer::start();
            let result = f();
            let mut acc = self.accumulator.get();
            acc.record(timer.elapsed_nanos());
            self.accumulator.set(acc);
            result
        } else {
            f()
        }
    }

    /// Record an externally-managed [`Timer`] into the accumulator.
    ///
    /// Use this when the work spans `.await` points and cannot be
    /// wrapped in [`timed`](Self::timed).  The caller is responsible
    /// for starting the timer and passing it in after the work
    /// completes.
    #[inline]
    pub fn record_elapsed(&self, interests: Interests, timer: Timer) {
        if interests.contains(Interests::PROCESS_DURATION) {
            let mut acc = self.accumulator.get();
            acc.record(timer.elapsed_nanos());
            self.accumulator.set(acc);
        }
    }

    /// Report accumulated duration metrics to the collector.
    ///
    /// Drains the accumulator into the metric set, then reports
    /// and resets as usual.
    pub fn report(&mut self, reporter: &mut MetricsReporter) {
        let acc = self.accumulator.replace(Mmsc::default());
        self.metrics.compute_duration.merge(acc);
        let _ = reporter.report(&mut self.metrics);
    }
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use super::*;
    use crate::context::ControllerContext;
    use otap_df_config::node::NodeKind;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::collections::HashMap;

    fn test_pipeline_ctx() -> PipelineContext {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        controller
            .pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0)
            .with_node_context(
                "node".into(),
                "urn:test:processor:example".into(),
                NodeKind::Processor,
                HashMap::new(),
            )
    }

    /// End-to-end: Timer measures real time, timed() and record_elapsed()
    /// accumulate via Mmsc::merge, and the accumulator drains on report.
    #[test]
    fn timed_and_record_elapsed_accumulate() {
        let ctx = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        let active = Interests::PROCESS_DURATION;

        // Two timed() calls and one record_elapsed() → 3 samples.
        let _ = cd.timed(active, || std::hint::black_box(42));
        let _ = cd.timed(active, || std::hint::black_box(43));
        let timer = Timer::start();
        let _ = std::hint::black_box(0);
        cd.record_elapsed(active, timer);

        let snap = cd.accumulator.get().get();
        assert_eq!(snap.count, 3);
        assert!(snap.min >= 0.0);
        assert!(snap.sum >= snap.min);

        // With interests disabled, nothing is recorded.
        let cd2 = ComputeDuration::new(&ctx);
        let _ = cd2.timed(Interests::empty(), || 1);
        cd2.record_elapsed(Interests::empty(), Timer::start());
        assert_eq!(cd2.accumulator.get().get().count, 0);
    }
}
