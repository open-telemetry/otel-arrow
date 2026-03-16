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
    /// allowing callers to hold shared borrows of sibling fields in the
    /// closure.
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

    /// timed() accumulates samples when PROCESS_DURATION is active,
    /// and is a no-op when interests are disabled.
    #[test]
    fn timed_accumulates_when_active() {
        let ctx = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        let active = Interests::PROCESS_DURATION;

        // Three timed() calls → 3 samples.
        let _ = cd.timed(active, || std::hint::black_box(42));
        let _ = cd.timed(active, || std::hint::black_box(43));
        let _ = cd.timed(active, || std::hint::black_box(44));

        let snap = cd.accumulator.get().get();
        assert_eq!(snap.count, 3);
        assert!(snap.min >= 0.0);
        assert!(snap.sum >= snap.min);

        // With interests disabled, nothing is recorded.
        let cd2 = ComputeDuration::new(&ctx);
        let _ = cd2.timed(Interests::empty(), || 1);
        assert_eq!(cd2.accumulator.get().get().count, 0);
    }
}
