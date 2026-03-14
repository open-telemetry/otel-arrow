// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in compute-duration timing for processors.
//!
//! Processors that perform meaningful synchronous compute can add a
//! [`ComputeDuration`] field and call [`ComputeDuration::timed`] to
//! measure the wall-clock duration of that work.  Timing is gated on
//! the `PROCESS_DURATION` interest at the normal metric level.
//!
//! The closure-based [`ComputeDuration::timed`] API structurally
//! prevents timing from spanning `.await` points — the closure is
//! `FnOnce` (not async), so the compiler enforces that only
//! synchronous work is measured:
//!
//! ```ignore
//! let result = self.compute_duration.timed(interests, || {
//!     synchronous_compute(&mut records)
//! });
//! // Timer has already stopped before we reach any .await
//! effect_handler.send_message(result?).await?;
//! ```

use std::cell::Cell;
use std::rc::Rc;

use crate::Interests;
use otap_df_telemetry::instrument::{Mmsc, Timer};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

use crate::context::PipelineContext;

/// Metric set containing a single compute-duration instrument.
#[metric_set(name = "processor.compute.duration")]
#[derive(Debug, Default, Clone)]
pub struct ComputeDurationMetrics {
    /// Wall-clock duration of the processor compute section, in nanoseconds.
    #[metric(name = "compute.duration", unit = "ns")]
    pub compute_duration: Mmsc,
}

/// Wrapper providing interests-gated duration recording and reporting.
pub struct ComputeDuration {
    metrics: MetricSet<ComputeDurationMetrics>,
    /// Shared accumulator written by `TimingGuard` on drop.
    accumulator: Rc<Cell<Mmsc>>,
}

impl Default for ComputeDuration {
    /// Creates an unregistered no-op instance.
    ///
    /// Useful when a processor is constructed without a pipeline
    /// context (e.g. config-only parsing).  Calling `report` on a
    /// default instance is safe but does nothing.  Call
    /// [`ComputeDuration::new`] to get a registered, functional
    /// instance.
    fn default() -> Self {
        Self {
            metrics: MetricSet::default(),
            accumulator: Rc::new(Cell::new(Mmsc::default())),
        }
    }
}

impl ComputeDuration {
    /// Register a new compute-duration metric set on the given pipeline.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            metrics: pipeline_ctx.register_metrics::<ComputeDurationMetrics>(),
            accumulator: Rc::new(Cell::new(Mmsc::default())),
        }
    }

    /// Time a synchronous closure if interests includes
    /// `PROCESS_DURATION`.
    ///
    /// The closure-based API structurally prevents the timer from
    /// being held across `.await` — the closure is `FnOnce`, not
    /// async, so the compiler enforces that only synchronous work is
    /// measured.
    ///
    /// When `PROCESS_DURATION` is not in `interests`, the closure
    /// still runs but no timing overhead is incurred.
    #[inline]
    pub fn timed<T>(&self, interests: Interests, f: impl FnOnce() -> T) -> T {
        let _guard = self.start(interests);
        f()
    }

    /// Report accumulated duration metrics to the collector.
    ///
    /// Drains the shared accumulator into the metric set, then reports
    /// and resets as usual.
    pub fn report(&mut self, reporter: &mut MetricsReporter) {
        // Drain the accumulator written to by dropped TimingGuards.
        let acc = self.accumulator.replace(Mmsc::default());
        self.metrics.compute_duration.merge(acc);
        let _ = reporter.report(&mut self.metrics);
    }

    /// Start timing if interests includes `PROCESS_DURATION`.
    fn start(&self, interests: Interests) -> TimingGuard {
        if interests.contains(Interests::PROCESS_DURATION) {
            TimingGuard::Active {
                timer: Some(Timer::start()),
                accumulator: Rc::clone(&self.accumulator),
            }
        } else {
            TimingGuard::Disabled
        }
    }
}

/// Records the elapsed duration into the originating
/// `ComputeDuration` when dropped.
///
/// The `Disabled` variant carries no state and is zero-cost on drop.
///
/// Prefer [`ComputeDuration::timed`] which structurally prevents
/// holding the guard across `.await`.
#[must_use]
enum TimingGuard {
    /// Actively timing; records elapsed nanoseconds on drop.
    Active {
        /// The running timer, taken on drop.
        timer: Option<Timer>,
        /// Shared accumulator for recording elapsed time.
        accumulator: Rc<Cell<Mmsc>>,
    },
    /// Timing is disabled; drop is a no-op.
    Disabled,
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        if let TimingGuard::Active { timer, accumulator } = self {
            if let Some(timer) = timer.take() {
                let nanos = timer.elapsed_nanos();
                let mut mmsc = accumulator.get();
                mmsc.record(nanos);
                accumulator.set(mmsc);
            }
        }
    }
}
