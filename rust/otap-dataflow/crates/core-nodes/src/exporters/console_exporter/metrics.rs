// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Console Exporter node.

use super::ConsoleOutputFormat;
use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_otap::metrics::ExporterExportMetrics;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::time::Duration;

/// Actionable category for a failed console export operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum ConsoleExportErrorType {
    /// An OTLP byte payload could not be exposed as a logs view.
    OtlpViewCreation,
    /// An OTAP Arrow payload could not be exposed as a logs view.
    OtapViewCreation,
    /// The exporter received a signal that it cannot render.
    UnsupportedSignal,
    /// The selected output formatter could not encode the logs payload.
    Formatting,
    /// The rendered output could not be written to stdout.
    Write,
}

/// Fixed output-format context for console exporter failure metrics.
#[attribute_set(item, registration)]
#[derive(Debug, Clone, Copy)]
struct ConsoleFormatAttributes {
    format: ConsoleOutputFormat,
}

/// Signal and error dimensions for failed console export operations.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct ConsoleFailureAttributes {
    signal: SignalType,
    #[attribute_key = "error.type"]
    error_type: ConsoleExportErrorType,
}

/// Failed console exports grouped by signal and actionable error type.
#[metric_set(
    name = "exporter.console.failures",
    registration_attributes = ConsoleFormatAttributes,
    measurement_attributes = ConsoleFailureAttributes
)]
#[derive(Debug, Default, Clone)]
struct ConsoleExporterFailureMetrics {
    /// Number of PData messages that failed for the classified reason.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

/// Metric sets emitted directly by a console exporter.
pub(super) struct ConsoleExporterMetrics {
    export_metrics: MeasurementMetricSet<ExporterExportMetrics>,
    failure_metrics: MeasurementMetricSet<ConsoleExporterFailureMetrics>,
}

impl ConsoleExporterMetrics {
    /// Registers console exporter metrics with the selected output format.
    pub(super) fn register(pipeline_ctx: &PipelineContext, format: ConsoleOutputFormat) -> Self {
        Self {
            export_metrics: ExporterExportMetrics::register(pipeline_ctx),
            failure_metrics: ConsoleExporterFailureMetrics::register(
                pipeline_ctx,
                &ConsoleFormatAttributes { format },
            ),
        }
    }

    /// Reports all console exporter metric sets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter
            .report_measurement(&mut self.export_metrics)
            .and_then(|()| reporter.report_measurement(&mut self.failure_metrics))
    }

    /// Takes terminal snapshots of every touched metric bucket.
    #[must_use]
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.export_metrics.terminal_snapshots();
        snapshots.extend(self.failure_metrics.terminal_snapshots());
        snapshots
    }

    /// Records one successfully rendered and written PData message.
    #[inline]
    pub(super) fn record_success(&mut self, signal: SignalType, duration: Duration) {
        self.export_metrics
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Success,
            })
            .record(duration);
    }

    /// Records one failed PData message and its diagnostic category.
    #[inline]
    pub(super) fn record_failure(
        &mut self,
        signal: SignalType,
        error_type: ConsoleExportErrorType,
        duration: Duration,
    ) {
        self.export_metrics
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Failure,
            })
            .record(duration);
        self.failure_metrics
            .with(ConsoleFailureAttributes { signal, error_type })
            .messages
            .inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics(format: ConsoleOutputFormat) -> ConsoleExporterMetrics {
        new_test_metrics_with_registry(format).1
    }

    fn new_test_metrics_with_registry(
        format: ConsoleOutputFormat,
    ) -> (TelemetryRegistryHandle, ConsoleExporterMetrics) {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry.clone());
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        (
            registry,
            ConsoleExporterMetrics::register(&pipeline_ctx, format),
        )
    }

    /// Scenario: Console exports succeed and fail for different telemetry signals.
    /// Guarantees: Outcome counts and durations stay paired while failure types use isolated buckets.
    #[test]
    fn export_outcomes_and_failures_are_bucketed_consistently() {
        let mut metrics = new_test_metrics(ConsoleOutputFormat::RecordJson);
        metrics.record_success(SignalType::Logs, Duration::from_millis(10));
        metrics.record_success(SignalType::Logs, Duration::from_millis(20));
        metrics.record_failure(
            SignalType::Logs,
            ConsoleExportErrorType::OtlpViewCreation,
            Duration::from_millis(30),
        );
        metrics.record_failure(
            SignalType::Metrics,
            ConsoleExportErrorType::UnsupportedSignal,
            Duration::from_millis(40),
        );

        assert_eq!(
            metrics
                .export_metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .messages
                .get(),
            2
        );
        assert_eq!(
            metrics
                .export_metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Failure,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .export_metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .duration_seconds
                .get()
                .count(),
            2
        );
        assert_eq!(
            metrics
                .export_metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Failure,
                })
                .duration_seconds
                .get()
                .count(),
            1
        );
        assert_eq!(
            metrics
                .failure_metrics
                .get(ConsoleFailureAttributes {
                    signal: SignalType::Logs,
                    error_type: ConsoleExportErrorType::OtlpViewCreation,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .failure_metrics
                .get(ConsoleFailureAttributes {
                    signal: SignalType::Metrics,
                    error_type: ConsoleExportErrorType::UnsupportedSignal,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .failure_metrics
                .get(ConsoleFailureAttributes {
                    signal: SignalType::Logs,
                    error_type: ConsoleExportErrorType::Write,
                })
                .messages
                .get(),
            0
        );
    }

    /// Scenario: A failed console export is handed off during terminal shutdown twice.
    /// Guarantees: Outcome and diagnostic buckets have stable attributes, emit once, and drain.
    #[test]
    fn terminal_snapshots_emit_touched_buckets_once() {
        let (registry, mut metrics) = new_test_metrics_with_registry(ConsoleOutputFormat::Pretty);
        metrics.record_failure(
            SignalType::Traces,
            ConsoleExportErrorType::UnsupportedSignal,
            Duration::from_millis(10),
        );

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.exports"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.measurement_attribute_value("outcome") == Some("failure")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.console.failures"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.measurement_attribute_value("error.type") == Some("unsupported_signal")
        }));

        for snapshot in &snapshots {
            registry.accumulate_metric_set_snapshot(
                snapshot.key(),
                snapshot.bucket(),
                snapshot.get_metrics(),
            );
        }
        let export_batch = registry.drain_metric_export_batch();
        let failure_export = export_batch
            .metric_sets
            .iter()
            .find(|metric_set| metric_set.descriptor.name == "exporter.console.failures")
            .expect("console failure metric set");
        assert_eq!(
            failure_export.item_attributes,
            [
                ("format".to_owned(), "pretty".to_owned()),
                ("signal".to_owned(), "traces".to_owned()),
                ("error.type".to_owned(), "unsupported_signal".to_owned()),
            ]
        );
        assert!(metrics.terminal_snapshots().is_empty());
    }
}
