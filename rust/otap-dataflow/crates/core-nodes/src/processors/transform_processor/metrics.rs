// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the TransformProcessor node.

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MeasurementMetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Query language configured for the transform processor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum TransformLanguage {
    /// Kusto Query Language.
    Kql,
    /// OpenTelemetry Pipeline Language.
    Opl,
    /// OpenTelemetry Transformation Language.
    Ottl,
}

/// Fixed query-language context for transform processor metrics.
#[attribute_set(item, registration)]
#[derive(Debug, Clone, Copy)]
struct TransformLanguageAttributes {
    language: TransformLanguage,
}

/// Actionable category for a failed transform operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum TransformErrorType {
    /// The input PData payload could not be converted to OTAP Arrow records.
    PayloadConversion,
    /// Transport-optimized identifiers could not be decoded.
    IdDecode,
    /// The configured query pipeline failed while executing.
    QueryExecution,
    /// A query referenced an output route that is not configured.
    RouteNotConfigured,
    /// No configured inbound request slot was available.
    InboundCapacity,
    /// No configured outbound request slot was available.
    OutboundCapacity,
    /// An immediate default or routed output send failed.
    OutputSend,
    /// An internal transform processor invariant failed.
    Internal,
}

/// Signal and error dimensions for failed transform operations.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct TransformFailureAttributes {
    signal: SignalType,
    #[attribute_key = "error.type"]
    error_type: TransformErrorType,
}

/// Transform operations grouped by signal and terminal local outcome.
#[metric_set(
    name = "processor.transform",
    registration_attributes = TransformLanguageAttributes,
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
struct TransformOperationMetrics {
    /// Number of matching input messages whose local transform operation terminated.
    #[metric(unit = "{operation}")]
    operations: Counter<u64>,
}

/// Failed transform operations grouped by signal and actionable error type.
#[metric_set(
    name = "processor.transform",
    registration_attributes = TransformLanguageAttributes,
    measurement_attributes = TransformFailureAttributes
)]
#[derive(Debug, Default, Clone)]
struct TransformFailureMetrics {
    /// Number of failed transform operations.
    #[metric(unit = "{operation}")]
    failures: Counter<u64>,
}

/// Metric sets emitted directly by a transform processor.
#[derive(Debug)]
pub(super) struct TransformMetrics {
    operations: MeasurementMetricSet<TransformOperationMetrics>,
    failures: MeasurementMetricSet<TransformFailureMetrics>,
}

impl TransformMetrics {
    /// Registers transform metrics with a fixed query-language attribute.
    pub(super) fn register(pipeline_ctx: &PipelineContext, language: TransformLanguage) -> Self {
        let language_attributes = TransformLanguageAttributes { language };
        Self {
            operations: TransformOperationMetrics::register(pipeline_ctx, &language_attributes),
            failures: TransformFailureMetrics::register(pipeline_ctx, &language_attributes),
        }
    }

    /// Reports all transform metric sets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter
            .report_measurement(&mut self.operations)
            .and_then(|()| reporter.report_measurement(&mut self.failures))
    }

    /// Records one locally successful transform operation.
    pub(super) fn record_success(&mut self, signal: SignalType) {
        self.operations
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Success,
            })
            .operations
            .inc();
    }

    /// Records one locally failed transform operation and its diagnostic category.
    pub(super) fn record_failure(&mut self, signal: SignalType, error_type: TransformErrorType) {
        self.operations
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Failure,
            })
            .operations
            .inc();
        self.failures
            .with(TransformFailureAttributes { signal, error_type })
            .failures
            .inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics(language: TransformLanguage) -> TransformMetrics {
        let registry = TelemetryRegistryHandle::new();
        let controller = otap_df_engine::context::ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        TransformMetrics::register(&pipeline_ctx, language)
    }

    /// Scenario: Transform operations succeed and fail for different telemetry signals.
    /// Guarantees: Outcome and error buckets remain isolated and every failure has one diagnostic.
    #[test]
    fn operation_outcomes_and_failures_are_bucketed_consistently() {
        let mut metrics = new_test_metrics(TransformLanguage::Kql);
        metrics.record_success(SignalType::Logs);
        metrics.record_success(SignalType::Logs);
        metrics.record_failure(SignalType::Metrics, TransformErrorType::QueryExecution);

        assert_eq!(
            metrics
                .operations
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .operations
                .get(),
            2
        );
        assert_eq!(
            metrics
                .operations
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Metrics,
                    outcome: Outcome::Failure,
                })
                .operations
                .get(),
            1
        );
        assert_eq!(
            metrics
                .failures
                .get(TransformFailureAttributes {
                    signal: SignalType::Metrics,
                    error_type: TransformErrorType::QueryExecution,
                })
                .failures
                .get(),
            1
        );
        assert_eq!(
            metrics
                .failures
                .get(TransformFailureAttributes {
                    signal: SignalType::Logs,
                    error_type: TransformErrorType::QueryExecution,
                })
                .failures
                .get(),
            0
        );
    }

    /// Scenario: Only one signal and error combination is recorded before terminal handoff.
    /// Guarantees: Only touched buckets are emitted and terminal snapshots drain them once.
    #[test]
    fn terminal_snapshots_emit_only_touched_buckets_once() {
        let mut metrics = new_test_metrics(TransformLanguage::Opl);
        metrics.record_failure(SignalType::Traces, TransformErrorType::OutputSend);

        let operation_snapshots = metrics.operations.terminal_snapshots();
        let failure_snapshots = metrics.failures.terminal_snapshots();
        assert_eq!(operation_snapshots.len(), 1);
        assert_eq!(failure_snapshots.len(), 1);
        assert_eq!(
            operation_snapshots[0].measurement_attribute_value("signal"),
            Some("traces")
        );
        assert_eq!(
            operation_snapshots[0].measurement_attribute_value("outcome"),
            Some("failure")
        );
        assert_eq!(
            failure_snapshots[0].measurement_attribute_value("signal"),
            Some("traces")
        );
        assert_eq!(
            failure_snapshots[0].measurement_attribute_value("error.type"),
            Some("output_send")
        );
        assert!(metrics.operations.terminal_snapshots().is_empty());
        assert!(metrics.failures.terminal_snapshots().is_empty());
    }
}
