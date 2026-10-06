// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Azure Data Explorer Exporter node.

use std::cell::RefCell;
use std::rc::Rc;

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::ExporterMetrics;
use otel_arrow_dfe_telemetry::common_attributes::{
    HttpResponse, HttpResponseAttributes, Outcome, SignalOutcomeAttributes,
};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;

/// Shared handle to the metrics tracker.
///
/// # Borrow Discipline
///
/// All `borrow_mut()` calls on this handle **must** be brief temporaries
/// (e.g. `self.metrics.borrow_mut().record_http_response(...)`). Never hold a
/// `RefMut` guard across an `.await` point or while calling other methods
/// that may also borrow, as this will panic at runtime.
pub type AzureDataExplorerExporterMetricsRc = Rc<RefCell<AzureDataExplorerExporterMetricsTracker>>;

/// Logical ADX batches partitioned by signal and terminal outcome.
#[metric_set(
    name = "exporter.azure_data_explorer",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AzureDataExplorerExporterBatchMetrics {
    /// Number of logical ADX batches reaching a terminal outcome.
    #[metric(unit = "{batch}")]
    pub batches: Counter<u64>,
}

/// ADX HTTP attempts partitioned by bounded response category.
#[metric_set(
    name = "exporter.azure_data_explorer.http",
    measurement_attributes = HttpResponseAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AzureDataExplorerExporterHttpMetrics {
    /// Number of HTTP attempts by response category.
    #[metric(unit = "{response}")]
    pub responses: Counter<u64>,
}

/// Full metrics tracker for the ADX exporter.
pub struct AzureDataExplorerExporterMetricsTracker {
    pub(super) boundary: ExporterMetrics,
    batch_metrics: MeasurementMetricSet<AzureDataExplorerExporterBatchMetrics>,
    http_metrics: MeasurementMetricSet<AzureDataExplorerExporterHttpMetrics>,
}

impl std::fmt::Debug for AzureDataExplorerExporterMetricsTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureDataExplorerExporterMetricsTracker")
            .finish()
    }
}

impl AzureDataExplorerExporterMetricsTracker {
    /// Register the shared exporter boundary and ADX-specific metric sets.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            boundary: ExporterMetrics::register(pipeline_ctx),
            batch_metrics: AzureDataExplorerExporterBatchMetrics::register(pipeline_ctx),
            http_metrics: AzureDataExplorerExporterHttpMetrics::register(pipeline_ctx),
        }
    }

    /// Report metrics to the telemetry system.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary
            .report(reporter)
            .and_then(|()| reporter.report_measurement(&mut self.batch_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.http_metrics))
    }

    /// Take snapshots of the shared boundary and ADX-specific metric sets.
    #[must_use]
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.batch_metrics.terminal_snapshots());
        snapshots.extend(self.http_metrics.terminal_snapshots());
        snapshots
    }

    /// Record one terminal logical ADX batch.
    #[inline]
    pub fn record_batch(&mut self, signal: SignalType, outcome: Outcome) {
        self.batch_metrics
            .with(SignalOutcomeAttributes { signal, outcome })
            .batches
            .inc();
    }

    /// Record one physical ADX HTTP attempt by bounded response category.
    #[inline]
    pub fn record_http_response(&mut self, response: HttpResponse) {
        self.http_metrics
            .with(HttpResponseAttributes { response })
            .responses
            .inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;

    fn test_tracker() -> AzureDataExplorerExporterMetricsTracker {
        let interests = Interests::NODE_INPUT_METRICS
            | Interests::NODE_LOCAL_DURATION
            | Interests::NODE_ITEM_COUNTS
            | Interests::NODE_SIZE;
        let (pipeline_ctx, _registry) = test_pipeline_ctx_with_interests(interests);
        AzureDataExplorerExporterMetricsTracker::register(&pipeline_ctx)
    }

    /// Scenario: ADX batches reach successful, failed, and refused terminal outcomes.
    /// Guarantees: One batches metric is partitioned by bounded signal and outcome attributes.
    #[test]
    fn batches_are_partitioned_by_signal_and_outcome() {
        let mut metrics = test_tracker();
        metrics.record_batch(SignalType::Logs, Outcome::Success);
        metrics.record_batch(SignalType::Logs, Outcome::Failure);
        metrics.record_batch(SignalType::Metrics, Outcome::Refused);

        assert_eq!(
            metrics
                .batch_metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .batches
                .get(),
            1
        );
        assert_eq!(
            metrics
                .batch_metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Failure,
                })
                .batches
                .get(),
            1
        );
        assert_eq!(
            metrics
                .batch_metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Metrics,
                    outcome: Outcome::Refused,
                })
                .batches
                .get(),
            1
        );
    }

    /// Scenario: ADX HTTP attempts receive successful, rejected, and transport outcomes.
    /// Guarantees: One responses metric is partitioned by a bounded response attribute.
    #[test]
    fn http_attempts_are_partitioned_by_response() {
        let mut metrics = test_tracker();
        metrics.record_http_response(HttpResponse::Http2xx);
        metrics.record_http_response(HttpResponse::Http2xx);
        metrics.record_http_response(HttpResponse::Http429);
        metrics.record_http_response(HttpResponse::NetworkError);

        assert_eq!(
            metrics
                .http_metrics
                .get(HttpResponseAttributes {
                    response: HttpResponse::Http2xx,
                })
                .responses
                .get(),
            2
        );
        assert_eq!(
            metrics
                .http_metrics
                .get(HttpResponseAttributes {
                    response: HttpResponse::Http429,
                })
                .responses
                .get(),
            1
        );
        assert_eq!(
            metrics
                .http_metrics
                .get(HttpResponseAttributes {
                    response: HttpResponse::NetworkError,
                })
                .responses
                .get(),
            1
        );
    }
}
