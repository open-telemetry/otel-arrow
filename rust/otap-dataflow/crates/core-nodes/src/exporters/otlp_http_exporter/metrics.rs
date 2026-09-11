// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded-cardinality metrics for the OTLP HTTP exporter.

use http::StatusCode;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::ExporterMetrics;
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

use super::agent_fed_auth::AgentFedAuthErrorType;

/// Actionable category for a failed OTLP HTTP export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum OtlpHttpExporterErrorType {
    /// An OTAP Arrow payload could not be encoded as OTLP protobuf.
    Encoding,
    /// The request body could not be compressed.
    Compression,
    /// Authentication credentials were absent, unusable, or rejected.
    Authentication,
    /// The destination denied the authenticated principal.
    Authorization,
    /// The request exceeded a client, server, or gateway deadline.
    Timeout,
    /// The destination refused the request because capacity was exhausted.
    Throttled,
    /// The destination or gateway was temporarily unavailable.
    Unavailable,
    /// The destination permanently rejected the request.
    Rejected,
    /// The destination reported a server-side failure.
    ServerError,
    /// The HTTP transport failed without a response status.
    Transport,
    /// The successful response exceeded the configured body-size limit.
    ResponseTooLarge,
    /// The successful response body was not valid OTLP protobuf.
    ResponseDecode,
    /// The destination accepted only part of the export request.
    PartialRejection,
    /// The failure did not fit another bounded category.
    Other,
}

impl OtlpHttpExporterErrorType {
    /// Classifies an HTTP error status by the operator action it suggests.
    #[must_use]
    pub(super) fn from_status(status: StatusCode) -> Self {
        match status {
            StatusCode::UNAUTHORIZED => Self::Authentication,
            StatusCode::FORBIDDEN => Self::Authorization,
            StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => Self::Timeout,
            StatusCode::TOO_MANY_REQUESTS => Self::Throttled,
            StatusCode::BAD_GATEWAY | StatusCode::SERVICE_UNAVAILABLE => Self::Unavailable,
            status if status.is_client_error() => Self::Rejected,
            status if status.is_server_error() => Self::ServerError,
            _ => Self::Other,
        }
    }

    /// Returns whether the destination refused the attempt without an exporter failure.
    #[must_use]
    pub(super) const fn is_refusal(self) -> bool {
        matches!(
            self,
            Self::Authentication
                | Self::Authorization
                | Self::Throttled
                | Self::Rejected
                | Self::PartialRejection
        )
    }
}

/// Signal and error dimensions for failed OTLP HTTP exports.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct OtlpHttpFailureAttributes {
    /// Pipeline signal associated with the PData message.
    signal: SignalType,
    /// Bounded category describing the terminal export failure.
    #[attribute_key = "error.type"]
    error_type: OtlpHttpExporterErrorType,
}

/// Failed OTLP HTTP exports grouped by signal and actionable error type.
#[metric_set(
    name = "exporter.otlp_http.failures",
    measurement_attributes = OtlpHttpFailureAttributes
)]
#[derive(Debug, Default, Clone)]
struct OtlpHttpExporterFailureMetrics {
    /// Number of PData messages that failed for the classified error type.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

/// Failure reason for an agent-fed authentication lookup.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct OtlpHttpAuthFailureAttributes {
    /// Bounded reason the snapshot could not authenticate another request.
    #[attribute_key = "error.type"]
    error_type: AgentFedAuthErrorType,
}

/// Agent-fed authentication failures, including failures before data admission.
#[metric_set(
    name = "exporter.otlp_http.authentication",
    measurement_attributes = OtlpHttpAuthFailureAttributes
)]
#[derive(Debug, Default, Clone)]
struct OtlpHttpExporterAuthMetrics {
    /// Number of credential checks that failed to produce a usable snapshot.
    #[metric(unit = "{attempt}")]
    failures: Counter<u64>,
}

/// Terminal outcome and failure metrics emitted by an OTLP HTTP exporter.
pub(super) struct OtlpHttpExporterMetrics {
    pub(super) boundary: ExporterMetrics,
    failures: MeasurementMetricSet<OtlpHttpExporterFailureMetrics>,
    auth: MeasurementMetricSet<OtlpHttpExporterAuthMetrics>,
}

impl OtlpHttpExporterMetrics {
    /// Registers all OTLP HTTP exporter metric sets.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            boundary: ExporterMetrics::register(pipeline_ctx),
            failures: OtlpHttpExporterFailureMetrics::register(pipeline_ctx),
            auth: OtlpHttpExporterAuthMetrics::register(pipeline_ctx),
        }
    }

    /// Records one agent-fed credential check failure.
    pub(super) fn record_auth_failure(&mut self, error_type: AgentFedAuthErrorType) {
        self.auth
            .with(OtlpHttpAuthFailureAttributes { error_type })
            .failures
            .inc();
    }

    /// Records one failed terminal export diagnostic category.
    pub(super) fn record_failure(
        &mut self,
        signal: SignalType,
        error_type: OtlpHttpExporterErrorType,
    ) {
        self.failures
            .with(OtlpHttpFailureAttributes { signal, error_type })
            .messages
            .inc();
    }

    /// Reports all touched OTLP HTTP exporter metric buckets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.failures)?;
        reporter.report_measurement(&mut self.auth)
    }

    /// Takes terminal snapshots of all touched metric buckets.
    #[must_use]
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.failures.terminal_snapshots());
        snapshots.extend(self.auth.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_otap::metrics::ErrorWithOutcome;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

    fn new_metrics() -> OtlpHttpExporterMetrics {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        OtlpHttpExporterMetrics::register(&pipeline_ctx)
    }

    /// Scenario: Representative HTTP error statuses are classified by operator action.
    /// Guarantees: Authentication, retryable capacity, rejection, and server failures remain distinct.
    #[test]
    fn http_statuses_map_to_actionable_error_types() {
        let cases = [
            (
                StatusCode::UNAUTHORIZED,
                OtlpHttpExporterErrorType::Authentication,
            ),
            (
                StatusCode::FORBIDDEN,
                OtlpHttpExporterErrorType::Authorization,
            ),
            (
                StatusCode::REQUEST_TIMEOUT,
                OtlpHttpExporterErrorType::Timeout,
            ),
            (
                StatusCode::GATEWAY_TIMEOUT,
                OtlpHttpExporterErrorType::Timeout,
            ),
            (
                StatusCode::TOO_MANY_REQUESTS,
                OtlpHttpExporterErrorType::Throttled,
            ),
            (
                StatusCode::BAD_GATEWAY,
                OtlpHttpExporterErrorType::Unavailable,
            ),
            (
                StatusCode::SERVICE_UNAVAILABLE,
                OtlpHttpExporterErrorType::Unavailable,
            ),
            (StatusCode::BAD_REQUEST, OtlpHttpExporterErrorType::Rejected),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                OtlpHttpExporterErrorType::ServerError,
            ),
            (StatusCode::OK, OtlpHttpExporterErrorType::Other),
        ];

        for (status, expected) in cases {
            assert_eq!(OtlpHttpExporterErrorType::from_status(status), expected);
        }
    }

    /// Scenario: One successful and one failed OTLP HTTP export are recorded.
    /// Guarantees: The failure has one matching error bucket while success has none.
    #[test]
    fn failure_classification_is_paired_with_the_terminal_outcome() {
        let mut metrics = new_metrics();
        let completed = futures::executor::block_on(
            metrics
                .boundary
                .attempt(SignalType::Metrics)
                .run(async |_| Ok::<_, ErrorWithOutcome<OtlpHttpExporterErrorType>>(())),
        );
        metrics.boundary.record(completed).unwrap();
        let completed =
            futures::executor::block_on(metrics.boundary.attempt(SignalType::Metrics).run(
                async |attempt| Err::<(), _>(attempt.refused(OtlpHttpExporterErrorType::Throttled)),
            ));
        let error_type = metrics.boundary.record(completed).unwrap_err();
        metrics.record_failure(SignalType::Metrics, error_type);

        let snapshots = metrics.boundary.terminal_snapshots();
        for outcome in ["success", "refused"] {
            assert!(snapshots.iter().any(|snapshot| {
                snapshot.descriptor().name == "exporter.attempted"
                    && snapshot.measurement_attribute_value("signal") == Some("metrics")
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
                .get(OtlpHttpFailureAttributes {
                    signal: SignalType::Metrics,
                    error_type: OtlpHttpExporterErrorType::Throttled,
                })
                .messages
                .get(),
            1
        );
    }

    /// Scenario: Agent-fed checks fail for every bounded credential error category.
    /// Guarantees: Each failure is counted independently without requiring a signal batch.
    #[test]
    fn agent_fed_auth_failures_are_counted_by_bounded_reason() {
        let mut metrics = new_metrics();
        let error_types = [
            AgentFedAuthErrorType::CredentialUnavailable,
            AgentFedAuthErrorType::LookupTimeout,
            AgentFedAuthErrorType::EmptyToken,
            AgentFedAuthErrorType::TokenNearExpiry,
            AgentFedAuthErrorType::InvalidToken,
            AgentFedAuthErrorType::RejectedCredentialUnchanged,
        ];

        for error_type in error_types {
            metrics.record_auth_failure(error_type);
            assert_eq!(
                metrics
                    .auth
                    .get(OtlpHttpAuthFailureAttributes { error_type })
                    .failures
                    .get(),
                1
            );
        }
    }

    /// Scenario: Authentication metrics are handed to a periodic metrics reporter.
    /// Guarantees: The authentication metric set participates in normal reporting.
    #[test]
    fn reports_agent_fed_auth_metrics() {
        let mut metrics = new_metrics();
        metrics.record_auth_failure(AgentFedAuthErrorType::LookupTimeout);
        let (_receiver, mut reporter) = MetricsReporter::create_new_and_receiver(3);

        metrics.report(&mut reporter).unwrap();
    }
}
