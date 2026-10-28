// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry shared by every background provider extension.

use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{MetricSet, MetricSetHandler, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

/// The instruments a background provider metric set must expose.
pub trait BackgroundProviderMetrics: MetricSetHandler + Send + 'static {
    /// Counter of successful acquisitions.
    fn successes(&mut self) -> &mut Counter<u64>;
    /// Counter of failed acquisitions.
    fn failures(&mut self) -> &mut Counter<u64>;
    /// Counter of values published to consumers via the watch channel.
    fn publishes(&mut self) -> &mut Counter<u64>;
    /// Latency (ms) of successful acquisitions.
    fn success_latency(&mut self) -> &mut Mmsc;
}

/// Tracks and flushes a background provider's metric set.
pub struct BackgroundProviderMetricsTracker<M: BackgroundProviderMetrics> {
    metrics: MetricSet<M>,
}

impl<M: BackgroundProviderMetrics> std::fmt::Debug for BackgroundProviderMetricsTracker<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackgroundProviderMetricsTracker").finish()
    }
}

impl<M: BackgroundProviderMetrics> BackgroundProviderMetricsTracker<M> {
    /// Creates a new tracker wrapping a registered metric set.
    #[must_use]
    pub fn new(metrics: MetricSet<M>) -> Self {
        Self { metrics }
    }

    /// Flushes the metric set to the telemetry reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report(&mut self.metrics)
    }

    /// Returns a point-in-time snapshot of the metric set, e.g. to attach to
    /// the terminal state on shutdown.
    #[must_use]
    pub fn snapshot(&self) -> MetricSetSnapshot {
        self.metrics.snapshot()
    }

    /// Records a successful acquisition with its latency in milliseconds.
    pub fn record_success(&mut self, latency_ms: f64) {
        self.metrics.successes().inc();
        self.metrics.success_latency().record(latency_ms);
    }

    /// Records a failed acquisition.
    pub fn record_failure(&mut self) {
        self.metrics.failures().inc();
    }

    /// Records a token publication to consumers.
    pub fn record_publish(&mut self) {
        self.metrics.publishes().inc();
    }
}
