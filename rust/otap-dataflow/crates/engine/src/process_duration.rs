// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in process-duration timing for processors.
//!
//! Processors that perform meaningful compute can add a
//! [`ProcessDuration`] field.  Calling [`ProcessDuration::start`]
//! returns a [`TimingGuard`] that automatically records the elapsed
//! wall-clock duration when it is dropped (RAII).
//!
//! ```ignore
//! let _timing = self.process_duration.start(interests);
//! // … processing code …
//! // duration is recorded when `_timing` drops at scope exit
//! ```
//!
//! Timing is gated on [`Interests::PROCESS_DURATION`] (MetricLevel ≥ Normal).
//!
//! The guard is `#[must_use]`; combined with the workspace-level
//! `unused_results = "deny"` lint this makes forgetting to bind the
//! return value a **compile error**.

use std::cell::Cell;
use std::rc::Rc;

use crate::Interests;
use otap_df_telemetry::instrument::{Mmsc, Timer};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

use crate::context::PipelineContext;

/// Metric set containing a single process-duration histogram.
#[metric_set(name = "processor.process.duration")]
#[derive(Debug, Default, Clone)]
pub struct ProcessDurationMetrics {
    /// Wall-clock duration of the processor compute section, in nanoseconds.
    #[metric(name = "process.duration", unit = "ns")]
    pub process_duration: Mmsc,
}

/// Wrapper providing interests-gated duration recording and reporting.
pub struct ProcessDuration {
    metrics: MetricSet<ProcessDurationMetrics>,
    /// Shared accumulator written by [`TimingGuard`] on drop.
    accumulator: Rc<Cell<Mmsc>>,
}

impl ProcessDuration {
    /// Register a new process-duration metric set on the given pipeline.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            metrics: pipeline_ctx.register_metrics::<ProcessDurationMetrics>(),
            accumulator: Rc::new(Cell::new(Mmsc::default())),
        }
    }

    /// Start timing if `interests` includes [`Interests::PROCESS_DURATION`].
    ///
    /// Returns a [`TimingGuard`] that records the elapsed duration
    /// into this [`ProcessDuration`] when it is dropped.  If interests
    /// do not include [`Interests::PROCESS_DURATION`], the returned
    /// guard is inert (no overhead).
    ///
    /// # Compile-time safety
    ///
    /// The guard is `#[must_use]`.  With the workspace lint
    /// `unused_results = "deny"`, failing to bind the return value is
    /// a compile error.  Because recording happens on [`Drop`], there
    /// is no `stop()` method to forget.
    ///
    /// ```ignore
    /// let _timing = self.process_duration.start(effect_handler.node_interests());
    /// // … existing processing code unchanged …
    /// // duration recorded automatically when _timing drops
    /// ```
    #[must_use = "the timing guard records on drop; bind it with `let _timing = …`"]
    pub fn start(&self, interests: Interests) -> TimingGuard {
        let timer = if interests.contains(Interests::PROCESS_DURATION) {
            Some(Timer::start())
        } else {
            None
        };
        TimingGuard {
            timer,
            accumulator: Rc::clone(&self.accumulator),
        }
    }

    /// Report accumulated duration metrics to the collector.
    ///
    /// Drains the shared accumulator into the metric set, then reports
    /// and resets as usual.
    pub fn report(&mut self, reporter: &mut MetricsReporter) {
        // Drain the accumulator written to by dropped TimingGuards.
        let acc = self.accumulator.replace(Mmsc::default());
        self.metrics.process_duration.merge(acc);
        let _ = reporter.report(&mut self.metrics);
    }
}

/// RAII timing guard.  Records the elapsed duration into the
/// originating [`ProcessDuration`] when dropped.
#[must_use = "the timing guard records on drop; bind it with `let _timing = …`"]
pub struct TimingGuard {
    timer: Option<Timer>,
    accumulator: Rc<Cell<Mmsc>>,
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        if let Some(timer) = self.timer.take() {
            let nanos = timer.elapsed_nanos();
            let mut mmsc = self.accumulator.get();
            mmsc.record(nanos);
            self.accumulator.set(mmsc);
        }
    }
}
