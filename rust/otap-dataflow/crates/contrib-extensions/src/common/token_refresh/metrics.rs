// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry shared by every bearer-token provider extension.
//!
//! Each extension declares its own `#[metric_set]` struct (so its metrics carry
//! its own set name) and implements [`TokenProviderMetrics`] to expose the four
//! instruments the refresh loop records. [`TokenProviderMetricsTracker`] then
//! provides the recording and flushing logic once for all of them.

use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;

/// The instruments a bearer-token provider metric set must expose.
pub trait TokenProviderMetrics: MetricSetHandler + Send + 'static {
    /// Counter of successful token acquisitions.
    fn successes(&mut self) -> &mut Counter<u64>;
    /// Counter of failed token acquisitions.
    fn failures(&mut self) -> &mut Counter<u64>;
    /// Counter of tokens published to consumers via the watch channel.
    fn publishes(&mut self) -> &mut Counter<u64>;
    /// Latency (ms) of successful acquisitions.
    fn success_latency(&mut self) -> &mut Mmsc;
}

/// Tracks and flushes a bearer-token provider's metric set.
pub struct TokenProviderMetricsTracker<M: TokenProviderMetrics> {
    metrics: MetricSet<M>,
}

impl<M: TokenProviderMetrics> std::fmt::Debug for TokenProviderMetricsTracker<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenProviderMetricsTracker").finish()
    }
}

impl<M: TokenProviderMetrics> TokenProviderMetricsTracker<M> {
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
