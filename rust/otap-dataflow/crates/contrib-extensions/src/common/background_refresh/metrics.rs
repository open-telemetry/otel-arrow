// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry shared by every background provider extension.

use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;

/// Shared credential or token acquisition metrics partitioned by outcome.
#[metric_set(
    name = "extension.auth",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AuthExtensionAcquisitionMetrics {
    /// Number of credential or token acquisitions partitioned by outcome.
    #[metric(unit = "{acquisition}")]
    pub acquisitions: Counter<u64>,
}

/// The metrics operations a background provider metric set must expose.
pub trait BackgroundProviderMetrics: Send + 'static {
    /// Records an acquisition outcome (Success or Failure).
    fn record_acquisition(&mut self, outcome: Outcome);
    /// Records a credential or token publication to consumers.
    fn record_publish(&mut self);
    /// Records the latency of a successful acquisition in milliseconds.
    fn record_success_latency(&mut self, latency_ms: f64);
    /// Flushes metric sets to the telemetry reporter.
    fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError>;
    /// Takes terminal snapshots of all metric sets.
    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot>;
}

/// Tracks and flushes a background provider's metric set.
pub struct BackgroundProviderMetricsTracker<M: BackgroundProviderMetrics> {
    metrics: M,
}

impl<M: BackgroundProviderMetrics> std::fmt::Debug for BackgroundProviderMetricsTracker<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackgroundProviderMetricsTracker").finish()
    }
}

impl<M: BackgroundProviderMetrics> BackgroundProviderMetricsTracker<M> {
    /// Creates a new tracker wrapping a registered metric set.
    #[must_use]
    pub fn new(metrics: M) -> Self {
        Self { metrics }
    }

    /// Flushes the metric set to the telemetry reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.metrics.report(reporter)
    }

    /// Returns point-in-time snapshots of the metric sets, e.g. to attach to
    /// the terminal state on shutdown.
    #[must_use]
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        self.metrics.terminal_snapshots()
    }

    /// Records a successful acquisition with its latency in milliseconds.
    pub fn record_success(&mut self, latency_ms: f64) {
        self.metrics.record_acquisition(Outcome::Success);
        self.metrics.record_success_latency(latency_ms);
    }

    /// Records a failed acquisition.
    pub fn record_failure(&mut self) {
        self.metrics.record_acquisition(Outcome::Failure);
    }

    /// Records a token publication to consumers.
    pub fn record_publish(&mut self) {
        self.metrics.record_publish();
    }
}
