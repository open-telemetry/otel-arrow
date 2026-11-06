// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry shared by every background provider extension.

use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;

/// Shared credential or token acquisition metrics partitioned by outcome.
#[metric_set(name = "extension.auth", measurement_attributes = OutcomeAttributes)]
#[derive(Debug, Default, Clone)]
pub struct AuthExtensionAcquisitionMetrics {
    /// Number of credential or token acquisitions partitioned by outcome.
    #[metric(unit = "{acquisition}")]
    pub acquisitions: Counter<u64>,
    /// Number of credentials or tokens published to consumers via the watch channel.
    #[metric(unit = "{publication}")]
    pub publishes: Counter<u64>,
    /// Acquisition duration in seconds.
    #[metric(unit = "s")]
    pub duration: Mmsc,
}

/// Type alias for [`AuthExtensionAcquisitionMetrics`].
pub type AuthExtensionMetrics = AuthExtensionAcquisitionMetrics;

/// The metrics operations a background provider metric set must expose.
pub trait BackgroundProviderMetrics: Send + 'static {
    /// Records an acquisition outcome with its elapsed duration in seconds.
    fn record_acquisition(&mut self, outcome: Outcome, duration_secs: f64);
    /// Records a credential or token publication to consumers.
    fn record_publish(&mut self);
    /// Flushes metric sets to the telemetry reporter.
    fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError>;
    /// Takes terminal snapshots of all metric sets.
    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot>;
}

impl BackgroundProviderMetrics for MeasurementMetricSet<AuthExtensionAcquisitionMetrics> {
    fn record_acquisition(&mut self, outcome: Outcome, duration_secs: f64) {
        let metrics = self.with(OutcomeAttributes { outcome });
        metrics.acquisitions.inc();
        metrics.duration.record(duration_secs);
    }

    fn record_publish(&mut self) {
        self.with(OutcomeAttributes {
            outcome: Outcome::Success,
        })
        .publishes
        .inc();
    }

    fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(self)
    }

    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        MeasurementMetricSet::terminal_snapshots(self)
    }
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

    /// Records an acquisition outcome with its elapsed duration in seconds.
    pub fn record_acquisition(&mut self, outcome: Outcome, duration_secs: f64) {
        self.metrics.record_acquisition(outcome, duration_secs);
    }

    /// Records a successful acquisition with its elapsed duration in seconds.
    pub fn record_success(&mut self, duration_secs: f64) {
        self.metrics
            .record_acquisition(Outcome::Success, duration_secs);
    }

    /// Records a failed acquisition with its elapsed duration in seconds.
    pub fn record_failure(&mut self, duration_secs: f64) {
        self.metrics
            .record_acquisition(Outcome::Failure, duration_secs);
    }

    /// Records a token publication to consumers.
    pub fn record_publish(&mut self) {
        self.metrics.record_publish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    #[test]
    fn shared_auth_metrics_partitioned_by_outcome() {
        let registry = TelemetryRegistryHandle::new();
        let mut metrics = registry
            .register_metric_set_with_measurement_attributes::<AuthExtensionAcquisitionMetrics>(
                otel_arrow_dfe_telemetry::testing::EmptyAttributes(),
            );

        metrics.record_acquisition(Outcome::Success, 0.0125);
        metrics.record_publish();
        metrics.record_acquisition(Outcome::Failure, 0.005);

        let success = metrics.get(OutcomeAttributes {
            outcome: Outcome::Success,
        });
        assert_eq!(success.acquisitions.get(), 1);
        assert_eq!(success.publishes.get(), 1);
        assert_eq!(success.duration.count, 1);
        assert_eq!(success.duration.sum, 0.0125);

        let failure = metrics.get(OutcomeAttributes {
            outcome: Outcome::Failure,
        });
        assert_eq!(failure.acquisitions.get(), 1);
        assert_eq!(failure.publishes.get(), 0);
        assert_eq!(failure.duration.count, 1);
        assert_eq!(failure.duration.sum, 0.005);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn tracker_lifecycle_and_snapshots() {
        let registry = TelemetryRegistryHandle::new();
        let metrics = registry
            .register_metric_set_with_measurement_attributes::<AuthExtensionAcquisitionMetrics>(
                otel_arrow_dfe_telemetry::testing::EmptyAttributes(),
            );
        let mut tracker = BackgroundProviderMetricsTracker::new(metrics);

        tracker.record_success(0.025);
        tracker.record_publish();
        tracker.record_failure(0.01);

        let snapshots = tracker.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);

        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(4);
        tracker.record_success(0.01);
        tracker.record_failure(0.005);
        tracker.record_publish();
        tracker.report(&mut reporter).expect("report succeeds");
        assert_eq!(rx.try_iter().count(), 2);
    }
}
