// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded-cardinality terminal metrics for the OTAP exporter.

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_otap::metrics::ExporterExportMetrics;
use otap_df_pdata::proto::opentelemetry::arrow::v1::StatusCode;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::time::Duration;
use tonic::{Code, Status};

/// Actionable category for a failed OTAP export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum OtapExporterErrorType {
    /// The incoming PData payload could not be converted to OTAP Arrow records.
    PayloadConversion,
    /// OTAP Arrow records could not be encoded into outbound batch records.
    Encoding,
    /// Authentication credentials were rejected by the destination.
    Authentication,
    /// The destination denied the authenticated principal.
    Authorization,
    /// The stream or batch exceeded its deadline or was cancelled.
    Timeout,
    /// The destination refused the batch because capacity was exhausted.
    Throttled,
    /// The destination or stream operation was temporarily unavailable.
    Unavailable,
    /// The destination permanently rejected the batch.
    Rejected,
    /// The destination reported an internal failure or data loss.
    ServerError,
    /// The streaming transport ended without a more specific status.
    Transport,
    /// An internal stream-coordination channel failed.
    Internal,
    /// The export was abandoned during exporter shutdown.
    Shutdown,
    /// The failure did not fit another bounded category.
    Other,
}

impl OtapExporterErrorType {
    /// Classifies a terminal gRPC status by the operator action it suggests.
    #[must_use]
    pub(super) fn from_grpc_status(status: &Status) -> Self {
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

    /// Classifies a non-OK OTAP batch status by the operator action it suggests.
    #[must_use]
    pub(super) fn from_batch_status(status_code: i32) -> Self {
        match StatusCode::try_from(status_code) {
            Ok(StatusCode::Canceled | StatusCode::DeadlineExceeded) => Self::Timeout,
            Ok(StatusCode::PermissionDenied) => Self::Authorization,
            Ok(StatusCode::ResourceExhausted) => Self::Throttled,
            Ok(StatusCode::Aborted | StatusCode::Unavailable) => Self::Unavailable,
            Ok(StatusCode::InvalidArgument) => Self::Rejected,
            Ok(StatusCode::Internal) => Self::ServerError,
            Ok(StatusCode::Unauthenticated) => Self::Authentication,
            Ok(StatusCode::Ok) | Err(_) => Self::Other,
        }
    }
}

/// Signal and error dimensions for failed OTAP exports.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct OtapFailureAttributes {
    /// Pipeline signal associated with the PData message.
    signal: SignalType,
    /// Bounded category describing the terminal export failure.
    #[attribute_key = "error.type"]
    error_type: OtapExporterErrorType,
}

/// Failed OTAP exports grouped by signal and actionable error type.
#[metric_set(
    name = "exporter.otap.failures",
    measurement_attributes = OtapFailureAttributes
)]
#[derive(Debug, Default, Clone)]
struct OtapExporterFailureMetrics {
    /// Number of PData messages that failed for the classified error type.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

/// Terminal outcome and failure metrics emitted by an OTAP exporter.
pub(super) struct OtapExporterMetrics {
    pub(super) exports: MeasurementMetricSet<ExporterExportMetrics>,
    failures: MeasurementMetricSet<OtapExporterFailureMetrics>,
}

impl OtapExporterMetrics {
    /// Registers all OTAP exporter terminal metric sets.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            exports: ExporterExportMetrics::register(pipeline_ctx),
            failures: OtapExporterFailureMetrics::register(pipeline_ctx),
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
        error_type: OtapExporterErrorType,
        duration: Duration,
    ) {
        self.exports
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Failure,
            })
            .record(duration);
        self.failures
            .with(OtapFailureAttributes { signal, error_type })
            .messages
            .inc();
    }

    /// Reports all touched OTAP exporter terminal metric buckets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter
            .report_measurement(&mut self.exports)
            .and_then(|()| reporter.report_measurement(&mut self.failures))
    }

    /// Takes terminal snapshots of all touched terminal metric buckets.
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

    fn new_metrics() -> OtapExporterMetrics {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        OtapExporterMetrics::register(&pipeline_ctx)
    }

    /// Scenario: OTAP gRPC and batch statuses represent equivalent backend failures.
    /// Guarantees: Both protocols map equivalent statuses to the same actionable categories.
    #[test]
    fn grpc_and_batch_statuses_share_actionable_categories() {
        let cases = [
            (StatusCode::Canceled, OtapExporterErrorType::Timeout),
            (StatusCode::DeadlineExceeded, OtapExporterErrorType::Timeout),
            (
                StatusCode::PermissionDenied,
                OtapExporterErrorType::Authorization,
            ),
            (
                StatusCode::ResourceExhausted,
                OtapExporterErrorType::Throttled,
            ),
            (StatusCode::Aborted, OtapExporterErrorType::Unavailable),
            (StatusCode::Unavailable, OtapExporterErrorType::Unavailable),
            (StatusCode::InvalidArgument, OtapExporterErrorType::Rejected),
            (StatusCode::Internal, OtapExporterErrorType::ServerError),
            (
                StatusCode::Unauthenticated,
                OtapExporterErrorType::Authentication,
            ),
            (StatusCode::Ok, OtapExporterErrorType::Other),
        ];

        for (status, expected) in cases {
            assert_eq!(
                OtapExporterErrorType::from_batch_status(status as i32),
                expected
            );
        }
        assert_eq!(
            OtapExporterErrorType::from_grpc_status(&Status::resource_exhausted("busy")),
            OtapExporterErrorType::Throttled
        );
        assert_eq!(
            OtapExporterErrorType::from_batch_status(i32::MAX),
            OtapExporterErrorType::Other
        );
    }

    /// Scenario: One successful and one failed OTAP export are recorded.
    /// Guarantees: The failure has one matching error bucket while success has none.
    #[test]
    fn failure_classification_is_paired_with_the_terminal_outcome() {
        let mut metrics = new_metrics();
        metrics.record_success(SignalType::Traces, Duration::from_millis(10));
        metrics.record_failure(
            SignalType::Traces,
            OtapExporterErrorType::Encoding,
            Duration::from_millis(20),
        );

        assert_eq!(
            metrics
                .exports
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Traces,
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
                    signal: SignalType::Traces,
                    outcome: Outcome::Failure,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .failures
                .get(OtapFailureAttributes {
                    signal: SignalType::Traces,
                    error_type: OtapExporterErrorType::Encoding,
                })
                .messages
                .get(),
            1
        );
    }
}
