// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry for the OAuth2 Client Auth extension.

use otel_arrow_dfe_engine::context::ExtensionContext;
use otel_arrow_dfe_telemetry::common_attributes::Outcome;
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::registry::EntityKey;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

use crate::common::background_refresh::{
    AuthExtensionAcquisitionMetrics, BackgroundProviderMetrics,
};

/// Telemetry metrics for the OAuth2 Client Auth extension.
pub struct OAuth2ClientAuthMetrics {
    pub(crate) acquisition_metrics: MeasurementMetricSet<AuthExtensionAcquisitionMetrics>,
}

impl std::fmt::Debug for OAuth2ClientAuthMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuth2ClientAuthMetrics").finish()
    }
}

impl OAuth2ClientAuthMetrics {
    /// Registers the extension's metric sets with the engine extension context.
    #[must_use]
    pub fn register(ext_ctx: &ExtensionContext, entity_key: EntityKey) -> Self {
        Self {
            acquisition_metrics: ext_ctx
                .register_measurement_metric_set_for_entity::<AuthExtensionAcquisitionMetrics>(
                    entity_key,
                ),
        }
    }

    /// Registers the extension's metric sets with a test registry handle.
    #[cfg(test)]
    #[must_use]
    pub fn new_test(
        registry: &otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle,
    ) -> Self {
        Self {
            acquisition_metrics: registry
                .register_metric_set_with_measurement_attributes::<AuthExtensionAcquisitionMetrics>(
                    otel_arrow_dfe_telemetry::testing::EmptyAttributes(),
                ),
        }
    }
}

impl BackgroundProviderMetrics for OAuth2ClientAuthMetrics {
    fn record_acquisition(&mut self, outcome: Outcome) {
        self.acquisition_metrics.record_acquisition(outcome);
    }

    fn record_publish(&mut self) {
        self.acquisition_metrics.record_publish();
    }

    fn record_success_latency(&mut self, latency_ms: f64) {
        self.acquisition_metrics.record_success_latency(latency_ms);
    }

    fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.acquisition_metrics.report(reporter)
    }

    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        self.acquisition_metrics.terminal_snapshots()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_telemetry::common_attributes::OutcomeAttributes;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    #[test]
    fn acquisitions_are_partitioned_by_outcome() {
        let registry = TelemetryRegistryHandle::new();
        let mut metrics = OAuth2ClientAuthMetrics::new_test(&registry);

        metrics.record_acquisition(Outcome::Success);
        metrics.record_acquisition(Outcome::Failure);
        metrics.record_acquisition(Outcome::Failure);

        assert_eq!(
            metrics
                .acquisition_metrics
                .get(OutcomeAttributes {
                    outcome: Outcome::Success
                })
                .acquisitions
                .get(),
            1
        );
        assert_eq!(
            metrics
                .acquisition_metrics
                .get(OutcomeAttributes {
                    outcome: Outcome::Failure
                })
                .acquisitions
                .get(),
            2
        );
    }

    #[test]
    fn shared_metrics_record_publish_and_duration() {
        let registry = TelemetryRegistryHandle::new();
        let mut metrics = OAuth2ClientAuthMetrics::new_test(&registry);

        metrics.record_publish();
        metrics.record_publish();
        metrics.record_success_latency(15.0);

        let success = metrics
            .acquisition_metrics
            .get(OutcomeAttributes {
                outcome: Outcome::Success,
            });
        assert_eq!(success.publishes.get(), 2);
        assert_eq!(success.duration.count, 1);
        assert_eq!(success.duration.sum, 15.0);
    }

    #[test]
    fn terminal_snapshots_and_reporting() {
        let registry = TelemetryRegistryHandle::new();
        let mut metrics = OAuth2ClientAuthMetrics::new_test(&registry);

        metrics.record_acquisition(Outcome::Success);
        metrics.record_publish();

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);

        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(4);
        metrics.record_acquisition(Outcome::Failure);
        metrics.record_publish();
        metrics.report(&mut reporter).expect("report succeeds");

        assert_eq!(rx.try_iter().count(), 2);
    }
}
