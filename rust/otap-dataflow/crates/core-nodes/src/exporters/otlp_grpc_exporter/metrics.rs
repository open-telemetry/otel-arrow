// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded-cardinality metrics for the OTLP gRPC exporter.

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
use tonic::{Code, Status};

/// Actionable category for a failed OTLP gRPC export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum OtlpGrpcExporterErrorType {
    /// An OTAP Arrow payload could not be encoded as OTLP protobuf.
    Encoding,
    /// Authentication credentials were absent, unusable, or rejected.
    Authentication,
    /// The destination denied the authenticated principal.
    Authorization,
    /// The request was cancelled or exceeded its deadline.
    Timeout,
    /// The destination refused the request because capacity was exhausted.
    Throttled,
    /// The destination or operation was temporarily unavailable.
    Unavailable,
    /// The destination permanently rejected the request.
    Rejected,
    /// The destination reported an internal failure or data loss.
    ServerError,
    /// The gRPC transport failed without a more specific status.
    Transport,
    /// The failure did not fit another bounded category.
    Other,
}

impl OtlpGrpcExporterErrorType {
    /// Classifies a terminal gRPC status by the operator action it suggests.
    #[must_use]
    pub(super) fn from_status(status: &Status) -> Self {
        match status.code() {
            Code::Unauthenticated => Self::Authentication,
            Code::PermissionDenied => Self::Authorization,
            Code::Cancelled | Code::DeadlineExceeded => Self::Timeout,
            Code::ResourceExhausted => Self::Throttled,
            Code::Aborted | Code::Unavailable => Self::Unavailable,
            Code::InvalidArgument
            | Code::NotFound
            | Code::AlreadyExists
            | Code::FailedPrecondition
            | Code::OutOfRange
            | Code::Unimplemented => Self::Rejected,
            Code::Internal | Code::DataLoss => Self::ServerError,
            Code::Unknown => Self::Transport,
            Code::Ok => Self::Other,
        }
    }
}

/// Signal and error dimensions for failed OTLP gRPC exports.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct OtlpGrpcFailureAttributes {
    /// Pipeline signal associated with the PData message.
    signal: SignalType,
    /// Bounded category describing the terminal export failure.
    #[attribute_key = "error.type"]
    error_type: OtlpGrpcExporterErrorType,
}

/// Failed OTLP gRPC exports grouped by signal and actionable error type.
#[metric_set(
    name = "exporter.otlp_grpc.failures",
    measurement_attributes = OtlpGrpcFailureAttributes
)]
#[derive(Debug, Default, Clone)]
struct OtlpGrpcExporterFailureMetrics {
    /// Number of PData messages that failed for the classified error type.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

/// Terminal outcome and failure metrics emitted by an OTLP gRPC exporter.
pub(super) struct OtlpGrpcExporterMetrics {
    pub(super) exports: MeasurementMetricSet<ExporterExportMetrics>,
    failures: MeasurementMetricSet<OtlpGrpcExporterFailureMetrics>,
}

impl OtlpGrpcExporterMetrics {
    /// Registers all OTLP gRPC exporter metric sets.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            exports: ExporterExportMetrics::register(pipeline_ctx),
            failures: OtlpGrpcExporterFailureMetrics::register(pipeline_ctx),
        }
    }

    /// Records one successful terminal export.
    pub(super) fn record_success(&mut self, signal: SignalType, duration: Duration) {
        self.exports
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Success,
            })
            .record(duration);
    }

    /// Records one failed terminal export and exactly one diagnostic category.
    pub(super) fn record_failure(
        &mut self,
        signal: SignalType,
        error_type: OtlpGrpcExporterErrorType,
        duration: Duration,
    ) {
        self.exports
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Failure,
            })
            .record(duration);
        self.failures
            .with(OtlpGrpcFailureAttributes { signal, error_type })
            .messages
            .inc();
    }

    /// Reports all touched OTLP gRPC exporter metric buckets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter
            .report_measurement(&mut self.exports)
            .and_then(|()| reporter.report_measurement(&mut self.failures))
    }

    /// Takes terminal snapshots of all touched metric buckets.
    #[must_use]
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.exports.terminal_snapshots();
        snapshots.extend(self.failures.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_metrics() -> OtlpGrpcExporterMetrics {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        OtlpGrpcExporterMetrics::register(&pipeline_ctx)
    }

    /// Scenario: Every gRPC status is classified into a bounded actionable category.
    /// Guarantees: Status classification is exhaustive and stable for exporter failure telemetry.
    #[test]
    fn grpc_statuses_map_to_actionable_error_types() {
        let cases = [
            (
                Code::Unauthenticated,
                OtlpGrpcExporterErrorType::Authentication,
            ),
            (
                Code::PermissionDenied,
                OtlpGrpcExporterErrorType::Authorization,
            ),
            (Code::Cancelled, OtlpGrpcExporterErrorType::Timeout),
            (Code::DeadlineExceeded, OtlpGrpcExporterErrorType::Timeout),
            (
                Code::ResourceExhausted,
                OtlpGrpcExporterErrorType::Throttled,
            ),
            (Code::Aborted, OtlpGrpcExporterErrorType::Unavailable),
            (Code::Unavailable, OtlpGrpcExporterErrorType::Unavailable),
            (Code::InvalidArgument, OtlpGrpcExporterErrorType::Rejected),
            (Code::NotFound, OtlpGrpcExporterErrorType::Rejected),
            (Code::AlreadyExists, OtlpGrpcExporterErrorType::Rejected),
            (
                Code::FailedPrecondition,
                OtlpGrpcExporterErrorType::Rejected,
            ),
            (Code::OutOfRange, OtlpGrpcExporterErrorType::Rejected),
            (Code::Unimplemented, OtlpGrpcExporterErrorType::Rejected),
            (Code::Internal, OtlpGrpcExporterErrorType::ServerError),
            (Code::DataLoss, OtlpGrpcExporterErrorType::ServerError),
            (Code::Unknown, OtlpGrpcExporterErrorType::Transport),
            (Code::Ok, OtlpGrpcExporterErrorType::Other),
        ];

        for (code, expected) in cases {
            assert_eq!(
                OtlpGrpcExporterErrorType::from_status(&Status::new(code, "test")),
                expected
            );
        }
    }

    /// Scenario: One successful and one failed OTLP gRPC export are recorded.
    /// Guarantees: The failure has one matching error bucket while success has none.
    #[test]
    fn failure_classification_is_paired_with_the_terminal_outcome() {
        let mut metrics = new_metrics();
        metrics.record_success(SignalType::Logs, Duration::from_millis(10));
        metrics.record_failure(
            SignalType::Logs,
            OtlpGrpcExporterErrorType::Unavailable,
            Duration::from_millis(20),
        );

        assert_eq!(
            metrics
                .exports
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .exports
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
                .failures
                .get(OtlpGrpcFailureAttributes {
                    signal: SignalType::Logs,
                    error_type: OtlpGrpcExporterErrorType::Unavailable,
                })
                .messages
                .get(),
            1
        );
    }
}
