// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded-cardinality internal telemetry for the Geneva exporter.

use geneva_uploader::client::UploadError;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::{ExporterAttempt, ExporterMetrics};
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::{Counter, HistogramNormal};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::time::Instant;

/// Bounded reason that Geneva exporter processing failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum GenevaExporterErrorType {
    /// OTAP transport-optimized identifiers could not be decoded.
    TransportDecoding,
    /// OTAP data could not be converted to the OTLP representation.
    Conversion,
    /// OTLP protobuf data could not be decoded.
    ProtobufDecoding,
    /// Signal data could not be encoded and compressed for Geneva.
    Encoding,
    /// Geneva throttled the upload.
    Throttled,
    /// Geneva rejected the upload with a non-throttling client error.
    Client,
    /// Geneva failed the upload with a server error.
    Server,
    /// Network transport failed before a terminal Geneva response.
    Transport,
    /// The configured account group could not be resolved.
    AccountRouting,
    /// The uploader failure did not fit another bounded category.
    Other,
    /// The exporter received a signal that Geneva does not support.
    UnsupportedSignal,
}

impl GenevaExporterErrorType {
    /// Classifies an uploader error into a bounded operator-facing category.
    #[must_use]
    pub(super) fn from_upload_error(error: &UploadError) -> Self {
        match error {
            UploadError::HttpStatus { status: 429, .. } => Self::Throttled,
            UploadError::HttpStatus { status, .. } if (400..500).contains(status) => Self::Client,
            UploadError::HttpStatus { status, .. } if (500..600).contains(status) => Self::Server,
            UploadError::Transport(_) => Self::Transport,
            UploadError::AccountGroupNotResolved { .. } => Self::AccountRouting,
            UploadError::HttpStatus { .. } | UploadError::Other(_) => Self::Other,
        }
    }

    /// Returns whether the failure explicitly rejected the attempted payload.
    #[must_use]
    pub(super) const fn is_refusal(self) -> bool {
        matches!(self, Self::Throttled | Self::Client)
    }
}

/// Bounded reason that a Geneva PData message did not require an upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum GenevaExporterSkipReason {
    /// The PData payload contained no telemetry.
    EmptyPayload,
}

/// Signal and failure dimensions for Geneva exporter processing errors.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct GenevaFailureAttributes {
    /// Signal carried by the failed operation.
    signal: SignalType,
    /// Bounded failure category.
    #[attribute_key = "error.type"]
    error_type: GenevaExporterErrorType,
}

/// Signal and reason dimensions for Geneva exporter skips.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct GenevaSkipAttributes {
    /// Signal carried by the skipped message.
    signal: SignalType,
    /// Bounded reason no upload was required.
    reason: GenevaExporterSkipReason,
}

/// Geneva encoding operations grouped by signal and outcome.
#[metric_set(
    name = "exporter.geneva.encoding",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
struct GenevaEncodingMetrics {
    /// Time spent encoding and compressing signal data.
    #[metric(unit = "s")]
    duration: HistogramNormal,
}

/// Geneva processing failures grouped by signal and actionable error type.
#[metric_set(
    name = "exporter.geneva.failures",
    measurement_attributes = GenevaFailureAttributes
)]
#[derive(Debug, Default, Clone)]
struct GenevaFailureMetrics {
    /// Number of failed Geneva processing operations.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

/// Geneva messages skipped before an upload attempt.
#[metric_set(
    name = "exporter.geneva.skipped",
    measurement_attributes = GenevaSkipAttributes
)]
#[derive(Debug, Default, Clone)]
struct GenevaSkippedMetrics {
    /// Number of skipped PData messages.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

/// Composite metrics emitted by a Geneva exporter.
#[derive(Debug)]
pub(super) struct GenevaExporterMetrics {
    /// Shared per-batch external export attempt metrics.
    pub(super) boundary: ExporterMetrics,
    encoding: MeasurementMetricSet<GenevaEncodingMetrics>,
    failures: MeasurementMetricSet<GenevaFailureMetrics>,
    skipped: MeasurementMetricSet<GenevaSkippedMetrics>,
    measure_duration: bool,
}

impl GenevaExporterMetrics {
    /// Registers all Geneva exporter metric sets.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            boundary: ExporterMetrics::register(pipeline_ctx),
            encoding: GenevaEncodingMetrics::register(pipeline_ctx),
            failures: GenevaFailureMetrics::register(pipeline_ctx),
            skipped: GenevaSkippedMetrics::register(pipeline_ctx),
            measure_duration: pipeline_ctx
                .node_interests()
                .contains(Interests::NODE_LOCAL_DURATION),
        }
    }

    /// Starts an encoding timer only when duration telemetry is enabled.
    pub(super) fn start_encoding(&self) -> Option<Instant> {
        self.measure_duration.then(Instant::now)
    }

    /// Records one completed Geneva encoding operation.
    pub(super) fn record_encoding(
        &mut self,
        signal: SignalType,
        outcome: Outcome,
        started_at: Option<Instant>,
    ) {
        if let Some(started_at) = started_at {
            self.encoding
                .with(SignalOutcomeAttributes { signal, outcome })
                .duration
                .record(started_at.elapsed().as_secs_f64());
        }
    }

    /// Records one bounded Geneva-specific failure category.
    pub(super) fn record_failure(
        &mut self,
        signal: SignalType,
        error_type: GenevaExporterErrorType,
    ) {
        self.failures
            .with(GenevaFailureAttributes { signal, error_type })
            .messages
            .inc();
    }

    /// Records one skipped Geneva input message.
    pub(super) fn record_skip(&mut self, signal: SignalType, reason: GenevaExporterSkipReason) {
        self.skipped
            .with(GenevaSkipAttributes { signal, reason })
            .messages
            .inc();
    }

    /// Records an attempt that terminated before producing an uploadable batch.
    pub(super) async fn record_unsubmitted_attempt(
        &mut self,
        attempt: ExporterAttempt,
        outcome: Outcome,
    ) {
        let completed = attempt
            .run(async |attempt| match outcome {
                Outcome::Success => Ok(()),
                Outcome::Failure => Err(attempt.failed(())),
                Outcome::Refused => Err(attempt.refused(())),
            })
            .await;
        let result = self.boundary.record(completed);
        debug_assert_eq!(result.is_ok(), outcome == Outcome::Success);
    }

    /// Reports every touched Geneva exporter metric bucket.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.encoding)?;
        reporter.report_measurement(&mut self.failures)?;
        reporter.report_measurement(&mut self.skipped)
    }

    /// Takes every touched Geneva exporter metric bucket for terminal handoff.
    #[must_use]
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.encoding.terminal_snapshots());
        snapshots.extend(self.failures.terminal_snapshots());
        snapshots.extend(self.skipped.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
    use otel_arrow_dfe_otap::metrics::ErrorWithOutcome;

    fn metric_value(
        snapshots: &[MetricSetSnapshot],
        set_name: &str,
        metric_name: &str,
        attributes: &[(&str, &str)],
    ) -> u64 {
        let snapshot = snapshots
            .iter()
            .find(|snapshot| {
                snapshot.descriptor().name == set_name
                    && attributes.iter().all(|(key, value)| {
                        snapshot.measurement_attribute_value(key) == Some(*value)
                    })
                    && snapshot
                        .descriptor()
                        .metrics
                        .iter()
                        .any(|metric| metric.name == metric_name)
            })
            .expect("metric snapshot");
        let metric = snapshot
            .descriptor()
            .metrics
            .iter()
            .position(|metric| metric.name == metric_name)
            .expect("metric descriptor");
        snapshot.get_metrics()[metric].to_u64_lossy()
    }

    /// Scenario: Geneva encoding and upload attempts cover successful and failed log batches.
    /// Guarantees: Shared per-batch values and Geneva-specific diagnostics remain partitioned by signal and outcome.
    #[tokio::test]
    async fn geneva_metrics_partition_attempts_and_diagnostics() {
        let interests = Interests::NODE_INPUT_METRICS
            | Interests::NODE_LOCAL_DURATION
            | Interests::NODE_ITEM_COUNTS
            | Interests::NODE_SIZE;
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
        let mut metrics = GenevaExporterMetrics::register(&pipeline_ctx);

        let encoding_started_at = metrics.start_encoding();
        metrics.record_encoding(SignalType::Logs, Outcome::Success, encoding_started_at);
        let completed = metrics
            .boundary
            .attempt(SignalType::Logs)
            .run(async |attempt| {
                attempt.set_item_count_with(|| 3);
                attempt.set_payload_size_with(|| 128);
                Ok::<(), ErrorWithOutcome<()>>(())
            })
            .await;
        metrics
            .boundary
            .record(completed)
            .expect("attempt succeeds");

        let completed = metrics
            .boundary
            .attempt(SignalType::Logs)
            .run(async |attempt| {
                attempt.set_item_count_with(|| 5);
                attempt.set_payload_size_with(|| 256);
                Err::<(), _>(attempt.failed(()))
            })
            .await;
        assert!(metrics.boundary.record(completed).is_err());
        metrics.record_failure(SignalType::Logs, GenevaExporterErrorType::Transport);
        metrics.record_skip(SignalType::Traces, GenevaExporterSkipReason::EmptyPayload);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(
            metric_value(
                &snapshots,
                "exporter.attempted",
                "messages",
                &[("signal", "logs"), ("outcome", "success")],
            ),
            1
        );
        assert_eq!(
            metric_value(
                &snapshots,
                "exporter.attempted",
                "items",
                &[("signal", "logs"), ("outcome", "failure")],
            ),
            5
        );
        assert_eq!(
            metric_value(
                &snapshots,
                "exporter.attempted",
                "payload.size",
                &[("signal", "logs"), ("outcome", "success")],
            ),
            128
        );
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.geneva.encoding"
                && snapshot.measurement_attribute_value("signal") == Some("logs")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .any(|metric| metric.name == "duration")
        }));
        assert_eq!(
            metric_value(
                &snapshots,
                "exporter.geneva.failures",
                "messages",
                &[("signal", "logs"), ("error.type", "transport")],
            ),
            1
        );
        assert_eq!(
            metric_value(
                &snapshots,
                "exporter.geneva.skipped",
                "messages",
                &[("signal", "traces"), ("reason", "empty_payload")],
            ),
            1
        );
    }

    /// Scenario: Geneva processing terminates before producing an uploadable batch.
    /// Guarantees: Success, failure, and refusal still emit one shared attempted message with the matching outcome.
    #[tokio::test]
    async fn unsubmitted_attempts_record_their_terminal_outcome() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
        let mut metrics = GenevaExporterMetrics::register(&pipeline_ctx);

        for outcome in [Outcome::Success, Outcome::Failure, Outcome::Refused] {
            let attempt = metrics.boundary.attempt(SignalType::Logs);
            metrics.record_unsubmitted_attempt(attempt, outcome).await;
        }

        let snapshots = metrics.terminal_snapshots();
        for outcome in ["success", "failure", "refused"] {
            assert_eq!(
                metric_value(
                    &snapshots,
                    "exporter.attempted",
                    "messages",
                    &[("signal", "logs"), ("outcome", outcome)],
                ),
                1
            );
        }
    }

    /// Scenario: Geneva encoding duration telemetry is disabled for a node.
    /// Guarantees: Starting an encoding operation does not read the clock or emit a duration.
    #[test]
    fn encoding_timer_respects_duration_interest() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
        let mut metrics = GenevaExporterMetrics::register(&pipeline_ctx);

        let started_at = metrics.start_encoding();
        assert!(started_at.is_none());
        metrics.record_encoding(SignalType::Logs, Outcome::Success, started_at);

        assert!(
            metrics
                .terminal_snapshots()
                .iter()
                .all(|snapshot| snapshot.descriptor().name != "exporter.geneva.encoding")
        );
    }

    /// Scenario: Geneva uploader failures span HTTP, transport, routing, and fallback variants.
    /// Guarantees: Every uploader error maps to a stable bounded telemetry category.
    #[test]
    fn upload_errors_are_classified_into_bounded_categories() {
        let cases = [
            (
                UploadError::HttpStatus {
                    status: 429,
                    retry_after: None,
                    message: "throttled".to_owned(),
                },
                GenevaExporterErrorType::Throttled,
            ),
            (
                UploadError::HttpStatus {
                    status: 400,
                    retry_after: None,
                    message: "bad request".to_owned(),
                },
                GenevaExporterErrorType::Client,
            ),
            (
                UploadError::HttpStatus {
                    status: 503,
                    retry_after: None,
                    message: "unavailable".to_owned(),
                },
                GenevaExporterErrorType::Server,
            ),
            (
                UploadError::Transport("network".to_owned()),
                GenevaExporterErrorType::Transport,
            ),
            (
                UploadError::AccountGroupNotResolved {
                    requested: "missing".to_owned(),
                    available: vec!["known".to_owned()],
                },
                GenevaExporterErrorType::AccountRouting,
            ),
            (
                UploadError::Other("internal".to_owned()),
                GenevaExporterErrorType::Other,
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(GenevaExporterErrorType::from_upload_error(&error), expected);
        }
    }

    /// Scenario: Geneva uploader failures are assigned a shared exporter outcome.
    /// Guarantees: Capacity and client rejections are refused while routing, transport, and server errors are failures.
    #[test]
    fn upload_error_outcomes_match_shared_attempt_semantics() {
        for error_type in [
            GenevaExporterErrorType::Throttled,
            GenevaExporterErrorType::Client,
        ] {
            assert!(error_type.is_refusal());
        }
        for error_type in [
            GenevaExporterErrorType::AccountRouting,
            GenevaExporterErrorType::Server,
            GenevaExporterErrorType::Transport,
            GenevaExporterErrorType::Other,
        ] {
            assert!(!error_type.is_refusal());
        }
    }
}
