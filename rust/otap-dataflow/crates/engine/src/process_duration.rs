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
