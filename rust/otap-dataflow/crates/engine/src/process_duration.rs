// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in process-duration timing for processors.
//!
//! Processors that perform meaningful compute can add a
//! [`ProcessDuration`] field and use [`ProcessDuration::start`] to
//! begin timing.  The returned [`TimingGuard`] is a lightweight token;
//! call [`TimingGuard::stop`] to record the elapsed duration into the
//! metric set.  If the guard is dropped without calling `stop`
//! (e.g. on early-return error paths) no measurement is recorded.
//!
//! Timing is gated on [`Interests::PROCESS_DURATION`] (MetricLevel ≥ Normal).

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

    /// Start timing if `interests` includes [`Interests::PROCESS_DURATION`].
    ///
    /// Returns a lightweight [`TimingGuard`] that captures the start
    /// instant.  Call [`TimingGuard::stop`] to record the elapsed
    /// duration, or let the guard drop without recording (e.g. on
    /// early-return error paths where you don't want the measurement).
    ///
    /// Usage:
    /// ```ignore
    /// let timing = ProcessDuration::start(effect_handler.node_interests());
    /// // … existing processing code unchanged …
    /// timing.stop(&mut self.process_duration);
    /// ```
    pub fn start(interests: Interests) -> TimingGuard {
        let timer = if interests.contains(Interests::PROCESS_DURATION) {
            Some(Timer::start())
        } else {
            None
        };
        TimingGuard { timer }
    }

    /// Report accumulated duration metrics to the collector.
    pub fn report(&mut self, reporter: &mut MetricsReporter) {
        let _ = reporter.report(&mut self.metrics);
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
            pd.metrics.process_duration.record_timer(timer);
        }
    }
}
