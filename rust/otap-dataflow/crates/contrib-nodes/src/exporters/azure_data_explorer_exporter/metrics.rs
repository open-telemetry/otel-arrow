// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Azure Data Explorer Exporter node.

use std::cell::RefCell;
use std::rc::Rc;

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::{ExporterAttempt, ExporterMetrics};
use otel_arrow_dfe_telemetry::common_attributes::{
    HttpResponse, HttpResponseAttributes, Outcome, SignalAttributes, SignalOutcomeAttributes,
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

/// Preserves attempt accounting when preparation or an HTTP future is canceled.
pub(super) struct PendingAttempt {
    metrics: AzureDataExplorerExporterMetricsRc,
    attempt: Option<ExporterAttempt>,
    http: bool,
}

impl PendingAttempt {
    pub(super) fn new(
        metrics: AzureDataExplorerExporterMetricsRc,
        attempt: ExporterAttempt,
    ) -> Self {
        Self {
            metrics,
            attempt: Some(attempt),
            http: false,
        }
    }

    pub(super) fn for_http(mut self) -> Self {
        self.http = true;
        self
    }

    pub(super) fn take(&mut self) -> ExporterAttempt {
        self.attempt.take().expect("attempt is completed once")
    }
}

impl Drop for PendingAttempt {
    fn drop(&mut self) {
        if let Some(attempt) = self.attempt.take() {
            let error = attempt.failed(());
            let completed = attempt.complete::<(), _>(Err(error));
            let mut metrics = self.metrics.borrow_mut();
            let _ = metrics.boundary.record(completed);
            if self.http {
                metrics.record_http_response(HttpResponse::NetworkError);
            }
        }
    }
}

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

/// Input diagnostics partitioned by the bounded OTLP signal.
#[metric_set(
    name = "exporter.azure_data_explorer",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AzureDataExplorerExporterInputMetrics {
    /// Number of source messages refused because a serialized row exceeded its byte limit.
    #[metric(unit = "{message}", name = "row_too_large")]
    pub row_too_large: Counter<u64>,
}

/// Full metrics tracker for the ADX exporter.
pub struct AzureDataExplorerExporterMetricsTracker {
    pub(super) boundary: ExporterMetrics,
    batch_metrics: MeasurementMetricSet<AzureDataExplorerExporterBatchMetrics>,
    http_metrics: MeasurementMetricSet<AzureDataExplorerExporterHttpMetrics>,
    input_metrics: MeasurementMetricSet<AzureDataExplorerExporterInputMetrics>,
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
            input_metrics: AzureDataExplorerExporterInputMetrics::register(pipeline_ctx),
        }
    }

    /// Report metrics to the telemetry system.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary
            .report(reporter)
            .and_then(|()| reporter.report_measurement(&mut self.batch_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.http_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.input_metrics))
    }

    /// Take snapshots of the shared boundary and ADX-specific metric sets.
    #[must_use]
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.batch_metrics.terminal_snapshots());
        snapshots.extend(self.http_metrics.terminal_snapshots());
        snapshots.extend(self.input_metrics.terminal_snapshots());
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

    /// Record safe diagnostics for one refused source message, without attempt accounting.
    ///
    /// Payloads and nested decoder/view errors are never formatted. Non-input
    /// errors are ignored; their diagnostics belong to the export path.
    pub(super) fn record_input_error(&mut self, signal: SignalType, error: &super::error::Error) {
        use super::error::Error;

        match error {
            Error::RowTooLarge { actual, limit } => {
                self.input_metrics
                    .with(SignalAttributes { signal })
                    .row_too_large
                    .inc();
                otel_warn!(
                    "azure_data_explorer_exporter.row_too_large",
                    signal = ?signal,
                    actual_row_bytes = *actual,
                    max_row_bytes = *limit,
                );
            }
            Error::RequestTooLarge { actual, limit } => {
                otel_warn!(
                    "azure_data_explorer_exporter.request_too_large",
                    signal = ?signal,
                    actual_bytes = *actual,
                    max_bytes = *limit,
                );
            }
            Error::TooManyRows { actual, limit } => {
                otel_warn!(
                    "azure_data_explorer_exporter.request_too_large",
                    signal = ?signal,
                    actual_rows = *actual,
                    max_rows = *limit,
                );
            }
            Error::TransformationDepthLimitExceeded { actual, limit }
            | Error::TransformationWorkLimitExceeded { actual, limit } => {
                let limit_kind = match error {
                    Error::TransformationDepthLimitExceeded { .. } => "depth",
                    _ => "work",
                };
                otel_warn!(
                    "azure_data_explorer_exporter.transformation_limit_exceeded",
                    signal = ?signal,
                    limit_kind,
                    actual = *actual,
                    limit = *limit,
                );
            }
            Error::InvalidMetricData { reason } => {
                otel_warn!(
                    "azure_data_explorer_exporter.invalid_metric_data",
                    signal = ?signal,
                    reason = *reason,
                );
            }
            Error::PayloadView(_) => {
                otel_warn!(
                    "azure_data_explorer_exporter.extraction_failed",
                    signal = ?signal,
                );
            }
            Error::Decode(_) => {
                otel_warn!(
                    "azure_data_explorer_exporter.payload_refused",
                    signal = ?signal,
                );
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::error::Error;
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

    /// Scenario: Oversized rows are refused in logs, metrics, and traces.
    /// Guarantees: The dedicated row counter counts refused source messages separately per signal.
    #[test]
    fn oversized_rows_are_partitioned_by_signal() {
        let mut metrics = test_tracker();
        let error = Error::RowTooLarge {
            actual: 1025,
            limit: 1024,
        };
        metrics.record_input_error(SignalType::Logs, &error);
        metrics.record_input_error(SignalType::Logs, &error);
        metrics.record_input_error(SignalType::Metrics, &error);
        metrics.record_input_error(SignalType::Traces, &error);

        for (signal, expected) in [
            (SignalType::Logs, 2),
            (SignalType::Metrics, 1),
            (SignalType::Traces, 1),
        ] {
            assert_eq!(
                metrics
                    .input_metrics
                    .get(SignalAttributes { signal })
                    .row_too_large
                    .get(),
                expected,
            );
        }
    }

    /// Scenario: Non-row input refusals and export errors are recorded as input diagnostics.
    /// Guarantees: Other error categories neither increment nor activate the row-specific counter.
    #[test]
    fn other_errors_do_not_count_as_oversized_rows() {
        let mut metrics = test_tracker();
        let errors = [
            Error::RequestTooLarge {
                actual: 4097,
                limit: 4096,
            },
            Error::TooManyRows {
                actual: 11,
                limit: 10,
            },
            Error::TransformationDepthLimitExceeded {
                actual: 65,
                limit: 64,
            },
            Error::TransformationWorkLimitExceeded {
                actual: 65537,
                limit: 65536,
            },
            Error::InvalidMetricData {
                reason: "missing numeric value",
            },
            Error::PayloadView(Box::new(
                otel_arrow_dfe_pdata::error::Error::ColumnNotFound {
                    name: "sensitive-column".to_string(),
                },
            )),
            Error::Decode(Box::new(
                otel_arrow_dfe_pdata_codec::CodecError::InvalidBatch {
                    reason: "sensitive-payload".to_string(),
                },
            )),
            Error::Config("not an input error".to_string()),
            Error::UnexpectedStatus {
                status: http::StatusCode::BAD_REQUEST,
                body: "sensitive-response".to_string(),
            },
        ];
        for error in &errors {
            metrics.record_input_error(SignalType::Logs, error);
        }
        assert_eq!(
            metrics
                .input_metrics
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .row_too_large
                .get(),
            0,
        );
        assert!(metrics.input_metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: An oversized-row refusal is followed by terminal metric collection.
    /// Guarantees: Its count and signal are included exactly once without an outcome dimension.
    #[test]
    fn terminal_snapshots_include_oversized_rows() {
        let mut metrics = test_tracker();
        metrics.record_input_error(
            SignalType::Traces,
            &Error::RowTooLarge {
                actual: 1025,
                limit: 1024,
            },
        );

        let snapshots = metrics.terminal_snapshots();
        let snapshot = snapshots
            .iter()
            .find(|snapshot| {
                snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .any(|field| field.name == "row_too_large")
            })
            .expect("row diagnostics must be included in terminal snapshots");
        assert_eq!(snapshot.descriptor().name, "exporter.azure_data_explorer");
        assert_eq!(
            snapshot.measurement_attribute_value("signal"),
            Some("traces")
        );
        assert_eq!(snapshot.measurement_attribute_value("outcome"), None);
        assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), 1);
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: Periodic telemetry reporting follows an oversized-row refusal.
    /// Guarantees: The dedicated row counter is emitted through the normal reporting path.
    #[test]
    fn oversized_rows_are_reported() {
        let mut metrics = test_tracker();
        metrics.record_input_error(
            SignalType::Logs,
            &Error::RowTooLarge {
                actual: 1025,
                limit: 1024,
            },
        );
        let (receiver, mut reporter) = MetricsReporter::create_new_and_receiver(16);
        metrics
            .report(&mut reporter)
            .expect("report input diagnostics");
        assert_eq!(receiver.try_iter().count(), 1);
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
