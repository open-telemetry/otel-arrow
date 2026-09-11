// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded-cardinality metrics for the OTLP gRPC exporter.

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::ExporterMetrics;
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
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

    /// Returns whether the destination refused the attempt without an exporter failure.
    #[must_use]
    pub(super) const fn is_refusal(self) -> bool {
        matches!(
            self,
            Self::Authentication | Self::Authorization | Self::Throttled | Self::Rejected
        )
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
    pub(super) boundary: ExporterMetrics,
    failures: MeasurementMetricSet<OtlpGrpcExporterFailureMetrics>,
}

impl OtlpGrpcExporterMetrics {
    /// Registers all OTLP gRPC exporter metric sets.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            boundary: ExporterMetrics::register(pipeline_ctx),
            failures: OtlpGrpcExporterFailureMetrics::register(pipeline_ctx),
        }
    }

    /// Records one failed terminal export diagnostic category.
    pub(super) fn record_failure(
        &mut self,
        signal: SignalType,
        error_type: OtlpGrpcExporterErrorType,
    ) {
        self.failures
            .with(OtlpGrpcFailureAttributes { signal, error_type })
            .messages
            .inc();
    }

    /// Reports all touched OTLP gRPC exporter metric buckets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.failures)
    }

    /// Takes terminal snapshots of all touched metric buckets.
    #[must_use]
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.failures.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_otap::metrics::ErrorWithOutcome;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

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
        let completed = futures::executor::block_on(
            metrics
                .boundary
                .attempt(SignalType::Logs)
                .run(async |_| Ok::<_, ErrorWithOutcome<OtlpGrpcExporterErrorType>>(())),
        );
        metrics.boundary.record(completed).unwrap();
        let completed =
            futures::executor::block_on(metrics.boundary.attempt(SignalType::Logs).run(
                async |attempt| {
                    Err::<(), _>(attempt.failed(OtlpGrpcExporterErrorType::Unavailable))
                },
            ));
        let error_type = metrics.boundary.record(completed).unwrap_err();
        metrics.record_failure(SignalType::Logs, error_type);

        let snapshots = metrics.boundary.terminal_snapshots();
        for outcome in ["success", "failure"] {
            assert!(snapshots.iter().any(|snapshot| {
                snapshot.descriptor().name == "exporter.attempted"
                    && snapshot.measurement_attribute_value("signal") == Some("logs")
                    && snapshot.measurement_attribute_value("outcome") == Some(outcome)
                    && snapshot
                        .descriptor()
                        .metrics
                        .iter()
                        .position(|metric| metric.name == "messages")
                        .is_some_and(|index| snapshot.get_metrics()[index].to_u64_lossy() == 1)
            }));
        }
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
