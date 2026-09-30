// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded-cardinality metrics for the OTLP HTTP exporter.

use http::StatusCode;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::http_client_auth::HttpClientAuthProvider;
use otel_arrow_dfe_otap::metrics::ExporterMetrics;
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::borrow::Cow;

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

#[attribute_set(item, registration)]
#[derive(Debug, Clone)]
struct OtlpHttpAuthSourceAttributes {
    source: Cow<'static, str>,
}

/// Current authentication readiness for a bound provider.
#[metric_set(
    name = "exporter.otlp_http.authentication",
    registration_attributes = OtlpHttpAuthSourceAttributes,
)]
#[derive(Debug, Default, Clone)]
struct OtlpHttpExporterAuthMetrics {
    /// Whether authenticated progress is currently possible (0=false, 1=true).
    #[metric(unit = "{1}")]
    ready: Gauge<u64>,
}

/// Terminal outcome and failure metrics emitted by an OTLP HTTP exporter.
pub(super) struct OtlpHttpExporterMetrics {
    pub(super) boundary: ExporterMetrics,
    failures: MeasurementMetricSet<OtlpHttpExporterFailureMetrics>,
    auth: Option<MetricSet<OtlpHttpExporterAuthMetrics>>,
}

impl OtlpHttpExporterMetrics {
    /// Registers all OTLP HTTP exporter metric sets.
    #[must_use]
    pub(super) fn register(
        pipeline_ctx: &PipelineContext,
        auth: Option<&dyn HttpClientAuthProvider>,
    ) -> Self {
        Self {
            boundary: ExporterMetrics::register(pipeline_ctx),
            failures: OtlpHttpExporterFailureMetrics::register(pipeline_ctx),
            auth: auth.map(|a| {
                let mut metrics = OtlpHttpExporterAuthMetrics::register(
                    pipeline_ctx,
                    &OtlpHttpAuthSourceAttributes { source: a.name() },
                );
                metrics.ready.set(u64::from(a.is_ready()));
                metrics
            }),
        }
    }

    /// Records whether authenticated progress is currently possible.
    pub(super) fn record_auth_readiness(&mut self, ready: bool) {
        if let Some(auth) = self.auth.as_mut() {
            auth.ready.set(u64::from(ready));
        }
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
        reporter
            .report_measurement(&mut self.failures)
            .and_then(|()| {
                if let Some(auth) = self.auth.as_mut() {
                    reporter.report(auth)
                } else {
                    Ok(())
                }
            })
    }

    /// Takes terminal snapshots after synchronizing authentication readiness.
    #[must_use]
    pub(super) fn terminal_snapshots(
        &mut self,
        auth: Option<&dyn HttpClientAuthProvider>,
    ) -> Vec<MetricSetSnapshot> {
        self.record_auth_readiness(auth.is_none_or(HttpClientAuthProvider::is_ready));
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.failures.terminal_snapshots());
        if let Some(auth) = self.auth.as_mut() {
            snapshots.extend(auth.terminal_snapshots());
        }
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::poll_fn;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
    use otel_arrow_dfe_otap::http_client_auth::test_support::MockHttpClientAuthProvider;
    use otel_arrow_dfe_otap::metrics::ErrorWithOutcome;

    fn new_metrics() -> OtlpHttpExporterMetrics {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
        OtlpHttpExporterMetrics::register(&pipeline_ctx, None)
    }

    /// Scenario: a bound auth provider becomes ready and is then invalidated
    /// before the HTTP exporter's terminal snapshot.
    /// Guarantees: terminal snapshots resample the provider and report its
    /// current readiness rather than the last value observed by the main loop.
    #[test]
    fn terminal_snapshot_resamples_authentication_readiness() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
        let mut auth = MockHttpClientAuthProvider::new(
            http::header::AUTHORIZATION,
            vec![("Bearer token".into(), None)],
        );
        let mut metrics = OtlpHttpExporterMetrics::register(&pipeline_ctx, Some(&auth));

        futures::executor::block_on(poll_fn(|cx| {
            auth.poll_refresh(cx, &super::super::HTTP_AUTH_EVENTS)
        }));
        let snapshots = metrics.terminal_snapshots(Some(&auth));
        let auth_snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.descriptor().name == "exporter.otlp_http.authentication")
            .expect("bound auth must register its authentication metric set");
        assert_eq!(auth_snapshot.get_metrics()[0].to_u64_lossy(), 1);

        auth.invalidate(1);

        let snapshots = metrics.terminal_snapshots(Some(&auth));
        let auth_snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.descriptor().name == "exporter.otlp_http.authentication")
            .expect("bound auth must register its authentication metric set");
        assert_eq!(auth_snapshot.get_metrics()[0].to_u64_lossy(), 0);
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
}
