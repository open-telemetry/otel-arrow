// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Console Exporter node.

use super::ConsoleOutputFormat;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::{ExporterAttempt, ExporterMetrics};
#[cfg(test)]
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Actionable category for a failed console export operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum ConsoleExportErrorType {
    /// An OTLP byte payload could not be exposed as a signal view.
    OtlpViewCreation,
    /// An OTAP Arrow payload could not be exposed as a signal view.
    OtapViewCreation,
    /// The exporter received a signal that it cannot render.
    UnsupportedSignal,
    /// The selected output formatter could not encode the payload.
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
    shared: ExporterMetrics,
    failure_metrics: MeasurementMetricSet<ConsoleExporterFailureMetrics>,
}

impl ConsoleExporterMetrics {
    /// Registers console exporter metrics with the selected output format.
    pub(super) fn register(pipeline_ctx: &PipelineContext, format: ConsoleOutputFormat) -> Self {
        Self {
            shared: ExporterMetrics::register(pipeline_ctx),
            failure_metrics: ConsoleExporterFailureMetrics::register(
                pipeline_ctx,
                &ConsoleFormatAttributes { format },
            ),
        }
    }

    /// Starts one console export attempt.
    pub(super) fn start_attempt(&self, item_count: impl FnOnce() -> u64) -> ExporterAttempt {
        self.shared.start_attempt(item_count)
    }

    /// Reports all console exporter metric sets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.shared
            .report(reporter)
            .and_then(|()| reporter.report_measurement(&mut self.failure_metrics))
    }

    /// Takes terminal snapshots of every touched metric bucket.
    #[must_use]
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.shared.terminal_snapshots();
        snapshots.extend(self.failure_metrics.terminal_snapshots());
        snapshots
    }

    /// Records one terminal console export and its optional failure category.
    #[inline]
    pub(super) fn record_attempt(
        &mut self,
        signal: SignalType,
        result: &Result<(), ConsoleExportErrorType>,
        payload_size: Option<usize>,
        attempt: ExporterAttempt,
    ) {
        self.shared
            .record_attempt(signal, result, payload_size, attempt);
    }

    /// Records one bounded console-specific export error category.
    #[inline]
    pub(super) fn record_error(&mut self, signal: SignalType, error_type: ConsoleExportErrorType) {
        self.failure_metrics
            .with(ConsoleFailureAttributes { signal, error_type })
            .messages
            .inc();
    }

    #[cfg(test)]
    fn attempted_for(
        &self,
        attributes: SignalOutcomeAttributes,
    ) -> &otel_arrow_dfe_otap::metrics::ExporterAttemptedMetrics {
        self.shared.attempted_for(attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

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
        let attempt = metrics.start_attempt(|| 0);
        metrics.record_attempt(SignalType::Logs, &Ok(()), None, attempt);
        let attempt = metrics.start_attempt(|| 0);
        metrics.record_attempt(SignalType::Logs, &Ok(()), None, attempt);
        let attempt = metrics.start_attempt(|| 0);
        let result = Err(ConsoleExportErrorType::OtlpViewCreation);
        metrics.record_attempt(SignalType::Logs, &result, None, attempt);
        metrics.record_error(SignalType::Logs, ConsoleExportErrorType::OtlpViewCreation);
        let attempt = metrics.start_attempt(|| 0);
        let result = Err(ConsoleExportErrorType::UnsupportedSignal);
        metrics.record_attempt(SignalType::Metrics, &result, None, attempt);
        metrics.record_error(
            SignalType::Metrics,
            ConsoleExportErrorType::UnsupportedSignal,
        );

        assert_eq!(
            metrics
                .attempted_for(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .messages
                .get(),
            2
        );
        assert_eq!(
            metrics
                .attempted_for(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Failure,
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
        let attempt = metrics.start_attempt(|| 0);
        let result = Err(ConsoleExportErrorType::UnsupportedSignal);
        metrics.record_attempt(SignalType::Traces, &result, None, attempt);
        metrics.record_error(
            SignalType::Traces,
            ConsoleExportErrorType::UnsupportedSignal,
        );

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.attempted"
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
