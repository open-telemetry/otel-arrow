// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in process-duration timing for processors.
//!
//! Processors that perform meaningful compute can add a
//! [`ProcessDuration`] field and call [`ProcessDuration::timed`] to
//! record wall-clock duration of their processing section.  Recording
//! is gated on [`Interests::CONSUMER_METRICS`] (MetricLevel ≥ Normal).

use crate::Interests;
use otap_df_telemetry::instrument::Mmsc;
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
}
