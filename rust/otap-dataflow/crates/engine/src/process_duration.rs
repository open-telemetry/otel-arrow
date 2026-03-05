// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in process-duration timing for processors.
//!
//! Processors that perform meaningful compute can add a
//! [`ProcessDuration`] field and use either:
//!
//! - [`ProcessDuration::guard`] – a drop-guard that records the
//!   elapsed time when it goes out of scope.  This keeps the
//!   original code structure intact (no closure / re-indentation).
//! - [`ProcessDuration::timed`] – a scoped closure that records
//!   duration on return.
//!
//! Both are gated on [`Interests::CONSUMER_METRICS`] (MetricLevel ≥ Normal).

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
}

impl ProcessDuration {
    /// Register a new process-duration metric set on the given pipeline.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            metrics: pipeline_ctx.register_metrics::<ProcessDurationMetrics>(),
        }
    }

    /// Start timing if `interests` includes [`Interests::CONSUMER_METRICS`].
    ///
    /// Returns a lightweight [`TimingGuard`] that captures the start
    /// instant.  Call [`TimingGuard::stop`] to record the elapsed
    /// duration, or let the guard drop without recording (e.g. on
    /// early-return error paths where you don't want the measurement).
    ///
    /// Usage:
    /// ```ignore
    /// let timing = self.process_duration.start(effect_handler.node_interests());
    /// // … existing processing code unchanged …
    /// timing.stop(&mut self.process_duration);
    /// ```
    pub fn start(interests: Interests) -> TimingGuard {
        let timer = if interests.contains(Interests::CONSUMER_METRICS) {
            Some(Timer::start())
        } else {
            None
        };
        TimingGuard { timer }
    }

    /// Time `f` if `interests` includes [`Interests::CONSUMER_METRICS`],
    /// otherwise just call `f` without overhead.
    pub fn timed<F, R>(&mut self, interests: Interests, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if interests.contains(Interests::CONSUMER_METRICS) {
            self.metrics.process_duration.timed(f)
        } else {
            f()
        }
    }

    /// Report accumulated duration metrics to the collector.
    pub fn report(&mut self, reporter: &mut MetricsReporter) {
        let _ = reporter.report(&mut self.metrics);
    }

    fn record(&mut self, nanos: f64) {
        self.metrics.process_duration.record(nanos);
    }
}

/// Lightweight timing token.  Call [`TimingGuard::stop`] to record
/// the elapsed duration into the originating [`ProcessDuration`].
#[must_use]
pub struct TimingGuard {
    timer: Option<Timer>,
}

impl TimingGuard {
    /// Record the elapsed time and consume the guard.
    pub fn stop(self, pd: &mut ProcessDuration) {
        if let Some(timer) = self.timer {
            pd.record(timer.elapsed_nanos());
        }
    }
}
