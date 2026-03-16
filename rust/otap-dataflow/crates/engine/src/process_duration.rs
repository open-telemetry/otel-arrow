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
    /// Accumulator for duration samples recorded via [`timed`](Self::timed)
    /// and [`record_elapsed`](Self::record_elapsed).
    accumulator: Mmsc,
}

impl ComputeDuration {
    /// Register a new compute-duration metric set on the given pipeline.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            metrics: pipeline_ctx.register_metrics::<ComputeDurationMetrics>(),
            accumulator: Mmsc::default(),
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
    pub fn timed<T>(&mut self, interests: Interests, f: impl FnOnce() -> T) -> T {
        if interests.contains(Interests::PROCESS_DURATION) {
            let timer = Timer::start();
            let result = f();
            self.accumulator.record(timer.elapsed_nanos());
            result
        } else {
            f()
        }
    }

    /// Start a timer if interests include `PROCESS_DURATION`.
    ///
    /// Returns `Some(Timer)` when timing is active, `None` otherwise.
    /// Pass the result to [`record_elapsed`](Self::record_elapsed)
    /// after the work completes.
    #[inline]
    #[must_use]
    pub fn start_timer(&self, interests: Interests) -> Option<Timer> {
        if interests.contains(Interests::PROCESS_DURATION) {
            Some(Timer::start())
        } else {
            None
        }
    }

    /// Record an externally-managed [`Timer`] into the accumulator.
    ///
    /// Use this when the work spans `.await` points and cannot be
    /// wrapped in [`timed`](Self::timed).  Pass the `Option<Timer>`
    /// returned by [`start_timer`](Self::start_timer); if `None`,
    /// this is a no-op.
    #[inline]
    pub fn record_elapsed(&mut self, timer: Option<Timer>) {
        if let Some(timer) = timer {
            self.accumulator.record(timer.elapsed_nanos());
        }
    }

    /// Report accumulated duration metrics to the collector.
    ///
    /// Drains the accumulator into the metric set, then reports
    /// and resets as usual.
    pub fn report(&mut self, reporter: &mut MetricsReporter) {
        let acc = std::mem::take(&mut self.accumulator);
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
        let mut cd = ComputeDuration::new(&ctx);
        let active = Interests::PROCESS_DURATION;

        // Two timed() calls and one start_timer/record_elapsed() → 3 samples.
        let _ = cd.timed(active, || std::hint::black_box(42));
        let _ = cd.timed(active, || std::hint::black_box(43));
        let timer = cd.start_timer(active);
        let _ = std::hint::black_box(0);
        cd.record_elapsed(timer);

        let snap = cd.accumulator.get();
        assert_eq!(snap.count, 3);
        assert!(snap.min >= 0.0);
        assert!(snap.sum >= snap.min);

        // With interests disabled, nothing is recorded.
        let mut cd2 = ComputeDuration::new(&ctx);
        let _ = cd2.timed(Interests::empty(), || 1);
        let timer = cd2.start_timer(Interests::empty());
        assert!(timer.is_none());
        cd2.record_elapsed(timer);
        assert_eq!(cd2.accumulator.get().count, 0);
    }
}
