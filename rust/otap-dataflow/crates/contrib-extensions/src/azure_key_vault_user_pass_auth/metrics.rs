// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Startup acquisition/publication telemetry, separate from database login.

use otel_arrow_dfe_engine::context::ExtensionContext;
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::registry::EntityKey;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;

use crate::common::background_refresh::{
    AuthExtensionAcquisitionMetrics, BackgroundProviderMetrics,
};

/// Operational metrics for Azure Key Vault user/pass authentication.
#[metric_set(name = "extension.azure_key_vault_user_pass_auth")]
#[derive(Debug, Default, Clone)]
pub struct AzureKeyVaultUserPassAuthOperationalMetrics {
    /// Complete credentials published to consumers.
    #[metric(unit = "{credential}")]
    pub publishes: Counter<u64>,
    /// Successful complete acquisition duration in milliseconds.
    #[metric(unit = "ms")]
    pub latency: Mmsc,
}

/// Lifecycle metrics recorded by the shared background provider.
pub struct AzureKeyVaultUserPassAuthMetrics {
    pub(crate) acquisition_metrics: MeasurementMetricSet<AuthExtensionAcquisitionMetrics>,
    pub(crate) operational_metrics: MetricSet<AzureKeyVaultUserPassAuthOperationalMetrics>,
}

impl std::fmt::Debug for AzureKeyVaultUserPassAuthMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureKeyVaultUserPassAuthMetrics").finish()
    }
}

impl AzureKeyVaultUserPassAuthMetrics {
    /// Registers the extension's metric sets with the engine extension context.
    #[must_use]
    pub fn register(ext_ctx: &ExtensionContext, entity_key: EntityKey) -> Self {
        Self {
            acquisition_metrics: ext_ctx
                .register_measurement_metric_set_for_entity::<AuthExtensionAcquisitionMetrics>(
                    entity_key,
                ),
            operational_metrics: ext_ctx
                .register_metric_set_for_entity::<AzureKeyVaultUserPassAuthOperationalMetrics>(
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
            operational_metrics: registry
                .register_metric_set::<AzureKeyVaultUserPassAuthOperationalMetrics>(
                    otel_arrow_dfe_telemetry::testing::EmptyAttributes(),
                ),
        }
    }
}

impl BackgroundProviderMetrics for AzureKeyVaultUserPassAuthMetrics {
    fn record_acquisition(&mut self, outcome: Outcome) {
        self.acquisition_metrics
            .with(OutcomeAttributes { outcome })
            .acquisitions
            .inc();
    }

    fn record_publish(&mut self) {
        self.operational_metrics.publishes.inc();
    }

    fn record_success_latency(&mut self, latency_ms: f64) {
        self.operational_metrics.latency.record(latency_ms);
    }

    fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.acquisition_metrics)?;
        reporter.report(&mut self.operational_metrics)
    }

    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.acquisition_metrics.terminal_snapshots();
        snapshots.extend(self.operational_metrics.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    #[test]
    fn acquisitions_are_partitioned_by_outcome() {
        let registry = TelemetryRegistryHandle::new();
        let mut metrics = AzureKeyVaultUserPassAuthMetrics::new_test(&registry);

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
    fn operational_metrics_record_publish_and_latency() {
        let registry = TelemetryRegistryHandle::new();
        let mut metrics = AzureKeyVaultUserPassAuthMetrics::new_test(&registry);

        metrics.record_publish();
        metrics.record_publish();
        metrics.record_success_latency(15.0);

        assert_eq!(metrics.operational_metrics.publishes.get(), 2);
        assert_eq!(metrics.operational_metrics.latency.count, 1);
        assert_eq!(metrics.operational_metrics.latency.sum, 15.0);
    }

    #[test]
    fn terminal_snapshots_and_reporting() {
        let registry = TelemetryRegistryHandle::new();
        let mut metrics = AzureKeyVaultUserPassAuthMetrics::new_test(&registry);

        metrics.record_acquisition(Outcome::Success);
        metrics.record_publish();

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);

        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(4);
        metrics.record_acquisition(Outcome::Failure);
        metrics.record_publish();
        metrics.report(&mut reporter).expect("report succeeds");

        assert_eq!(rx.try_iter().count(), 2);
    }
}
