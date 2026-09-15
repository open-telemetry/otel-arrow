// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic metrics used in the OTAP pipeline.
//!
//! Note: We try as much as possible to follow the following
//! [RFC Pipeline Component Telemetry](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::{
    Outcome, SignalAttributes, SignalOutcomeAttributes,
};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::{Counter, HistogramNormal};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;
use std::ops::AsyncFnOnce;
use std::time::{Duration, Instant};

/// Receiver-local handling of classified messages received at the external boundary.
#[metric_set(
    name = "receiver.received",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
struct ReceiverReceivedMetrics {
    /// Number of classified external messages whose receiver-local handling terminated.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

impl ReceiverReceivedMetrics {
    /// Records one classified external message when receiver-local handling terminates.
    #[inline]
    pub fn record(&mut self) {
        self.messages.inc();
    }
}

/// Optional payload-size accounting for classified receiver messages.
#[metric_set(
    name = "receiver.received",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
struct ReceiverReceivedPayloadMetrics {
    /// Encoded application payload size observed before receiver decoding.
    #[metric(name = "payload.size", unit = "By")]
    payload_size: Counter<u64>,
}

impl ReceiverReceivedPayloadMetrics {
    /// Records the encoded application payload size for one received message.
    #[inline]
    pub fn record(&mut self, payload_size: u64) {
        self.payload_size.add(payload_size);
    }
}

/// Receiver-defined local processing of classified external messages.
#[metric_set(
    name = "receiver.processing",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
struct ReceiverProcessingMetrics {
    /// Component-defined receiver-local processing time.
    ///
    /// Each receiver documents its stable start and end boundary. Downstream
    /// processing, batching wait, handoff wait, and Ack/Nack completion are
    /// excluded.
    #[metric(unit = "s")]
    duration: HistogramNormal,
}

impl ReceiverProcessingMetrics {
    /// Records one receiver-local processing operation.
    #[inline]
    pub fn record(&mut self, duration: Duration) {
        self.duration.record(duration.as_secs_f64());
    }
}

/// Receiver-local processing state captured for enabled shared metrics.
#[derive(Debug)]
pub struct ReceiverProcessing {
    measure_duration: bool,
    payload_size: Option<u64>,
    accepts_payload_size: bool,
}

/// Operation error paired with the outcome and optional receiver signal it represents.
#[derive(Debug)]
pub struct ErrorWithOutcome<E> {
    signal: Option<SignalType>,
    outcome: Outcome,
    error: E,
}

impl<E> From<E> for ErrorWithOutcome<E> {
    fn from(error: E) -> Self {
        Self {
            signal: None,
            outcome: Outcome::Failure,
            error,
        }
    }
}

/// Completed receiver processing ready to be recorded.
#[derive(Debug)]
#[must_use = "completed receiver processing must be recorded"]
pub struct CompletedReceiverProcessing<T, E> {
    signal: Option<SignalType>,
    outcome: Outcome,
    duration: Option<Duration>,
    payload_size: Option<u64>,
    result: Result<T, E>,
}

/// Shared receiver metrics with node-interest-gated processing duration.
#[derive(Debug)]
pub struct ReceiverMetrics {
    received: MeasurementMetricSet<ReceiverReceivedMetrics>,
    payload: MeasurementMetricSet<ReceiverReceivedPayloadMetrics>,
    processing: MeasurementMetricSet<ReceiverProcessingMetrics>,
    interests: Interests,
}

impl ReceiverMetrics {
    /// Registers the shared receiver metric sets.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            received: ReceiverReceivedMetrics::register(pipeline_ctx),
            payload: ReceiverReceivedPayloadMetrics::register(pipeline_ctx),
            processing: ReceiverProcessingMetrics::register(pipeline_ctx),
            interests: pipeline_ctx.node_interests(),
        }
    }

    /// Creates receiver-local processing instrumentation for one external message.
    #[must_use]
    pub fn processing(&self) -> ReceiverProcessing {
        ReceiverProcessing {
            measure_duration: self.interests.contains(Interests::NODE_LOCAL_DURATION),
            payload_size: None,
            accepts_payload_size: self.interests.contains(Interests::NODE_SIZE),
        }
    }

    /// Records one completed receiver processing observation.
    pub fn record<T, E>(&mut self, completed: CompletedReceiverProcessing<T, E>) -> Result<T, E> {
        let Some(signal) = completed.signal else {
            return completed.result;
        };
        if let Some(duration) = completed.duration {
            self.processing
                .with(SignalAttributes { signal })
                .record(duration);
        }
        let attributes = SignalOutcomeAttributes {
            signal,
            outcome: completed.outcome,
        };
        if self.interests.contains(Interests::NODE_OUTPUT_METRICS) {
            self.received.with(attributes).record();
        }
        if let Some(payload_size) = completed.payload_size {
            self.payload.with(attributes).record(payload_size);
        }
        completed.result
    }

    /// Reports every touched shared receiver metric bucket.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.received)?;
        reporter.report_measurement(&mut self.payload)?;
        reporter.report_measurement(&mut self.processing)
    }

    /// Takes every touched shared receiver metric bucket for terminal handoff.
    #[must_use]
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.received.terminal_snapshots();
        snapshots.extend(self.payload.terminal_snapshots());
        snapshots.extend(self.processing.terminal_snapshots());
        snapshots
    }
}

impl ReceiverProcessing {
    /// Classifies and returns an error from the processing closure as failed.
    pub fn failed<E>(&self, signal: SignalType, error: E) -> ErrorWithOutcome<E> {
        ErrorWithOutcome {
            signal: Some(signal),
            outcome: Outcome::Failure,
            error,
        }
    }

    /// Classifies and returns an error from the processing closure as refused.
    ///
    /// Pass the returned error directly to `Err`.
    pub fn refused<E>(&self, signal: SignalType, error: E) -> ErrorWithOutcome<E> {
        ErrorWithOutcome {
            signal: Some(signal),
            outcome: Outcome::Refused,
            error,
        }
    }

    /// Sets the encoded application payload size without evaluating it when disabled.
    pub fn set_payload_size_with(&mut self, payload_size: impl FnOnce() -> usize) {
        if self.accepts_payload_size {
            self.payload_size = Some(u64::try_from(payload_size()).unwrap_or(u64::MAX));
        }
    }

    /// Runs receiver-local processing and captures its terminal result.
    ///
    /// `Ok((signal, value))` records success. Return errors through
    /// [`Self::failed`] or [`Self::refused`] after signal classification.
    /// An unclassified `Err(error)` emits no shared receiver metric.
    #[must_use = "the completed receiver processing observation must be recorded"]
    pub fn run<T, E>(
        mut self,
        work: impl FnOnce(&mut ReceiverProcessing) -> Result<(SignalType, T), ErrorWithOutcome<E>>,
    ) -> CompletedReceiverProcessing<T, E> {
        let started_at = self.measure_duration.then(Instant::now);
        let result = work(&mut self);
        let (signal, outcome, result) = match result {
            Ok((signal, value)) => (Some(signal), Outcome::Success, Ok(value)),
            Err(ErrorWithOutcome {
                signal,
                outcome,
                error,
            }) => (signal, outcome, Err(error)),
        };
        CompletedReceiverProcessing {
            signal,
            outcome,
            duration: started_at.map(|started_at| started_at.elapsed()),
            payload_size: self.payload_size,
            result,
        }
    }
}

/// Individual node-local delivery attempts from an exporter.
#[metric_set(
    name = "exporter.attempted",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
struct ExporterAttemptedMetrics {
    /// Number of node-local delivery attempts.
    ///
    /// Retries count again. This differs from `node.input.messages`, which
    /// counts PData messages entering the exporter.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

impl ExporterAttemptedMetrics {
    /// Records one export attempt.
    #[inline]
    pub fn record(&mut self) {
        self.messages.inc();
    }
}

/// Optional duration accounting for individual exporter attempts.
#[metric_set(
    name = "exporter.attempted",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
struct ExporterAttemptedDurationMetrics {
    /// Time spent performing export attempts, including backend latency.
    #[metric(unit = "s")]
    duration: HistogramNormal,
}

impl ExporterAttemptedDurationMetrics {
    /// Records the duration of one export attempt.
    #[inline]
    pub fn record(&mut self, duration: Duration) {
        self.duration.record(duration.as_secs_f64());
    }
}

/// Optional payload-size accounting for individual exporter attempts.
#[metric_set(
    name = "exporter.attempted",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
struct ExporterAttemptedPayloadMetrics {
    /// Encoded application payload size produced or submitted across export attempts.
    #[metric(name = "payload.size", unit = "By")]
    payload_size: Counter<u64>,
}

impl ExporterAttemptedPayloadMetrics {
    /// Records the encoded application payload size for one export attempt.
    #[inline]
    pub fn record(&mut self, payload_size: u64) {
        self.payload_size.add(payload_size);
    }
}

/// Optional item accounting for individual exporter attempts.
#[metric_set(
    name = "exporter.attempted",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
struct ExporterAttemptedItemsMetrics {
    /// Number of signal items handled across export attempts.
    #[metric(unit = "{item}")]
    items: Counter<u64>,
}

impl ExporterAttemptedItemsMetrics {
    /// Records the item count for one export attempt.
    #[inline]
    pub fn record(&mut self, items: u64) {
        self.items.add(items);
    }
}

/// Prepared instrumentation for one node-local exporter attempt.
#[derive(Debug)]
pub struct ExporterAttempt {
    signal: SignalType,
    started_at: Option<Instant>,
    items: Option<u64>,
    accepts_item_count: bool,
    payload_size: Option<u64>,
    accepts_payload_size: bool,
}

/// Completed exporter attempt ready to be recorded.
#[derive(Debug)]
#[must_use = "completed exporter attempts must be recorded"]
pub struct CompletedExporterAttempt<T, E> {
    signal: SignalType,
    outcome: Outcome,
    duration: Option<Duration>,
    payload_size: Option<u64>,
    items: Option<u64>,
    result: Result<T, E>,
}

/// Shared exporter attempt metrics with node-interest-gated optional measurements.
#[derive(Debug)]
pub struct ExporterMetrics {
    attempted: MeasurementMetricSet<ExporterAttemptedMetrics>,
    duration: MeasurementMetricSet<ExporterAttemptedDurationMetrics>,
    payload: MeasurementMetricSet<ExporterAttemptedPayloadMetrics>,
    items: MeasurementMetricSet<ExporterAttemptedItemsMetrics>,
    interests: Interests,
}

impl ExporterMetrics {
    /// Registers the shared exporter metric sets.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            attempted: ExporterAttemptedMetrics::register(pipeline_ctx),
            duration: ExporterAttemptedDurationMetrics::register(pipeline_ctx),
            payload: ExporterAttemptedPayloadMetrics::register(pipeline_ctx),
            items: ExporterAttemptedItemsMetrics::register(pipeline_ctx),
            interests: pipeline_ctx.node_interests(),
        }
    }

    /// Starts instrumentation for one node-local exporter attempt.
    #[must_use]
    pub fn attempt(&self, signal: SignalType) -> ExporterAttempt {
        ExporterAttempt {
            signal,
            started_at: self
                .interests
                .contains(Interests::NODE_LOCAL_DURATION)
                .then(Instant::now),
            items: None,
            accepts_item_count: self.interests.contains(Interests::NODE_ITEM_COUNTS),
            payload_size: None,
            accepts_payload_size: self.interests.contains(Interests::NODE_SIZE),
        }
    }

    /// Records one completed exporter attempt.
    pub fn record<T, E>(&mut self, completed: CompletedExporterAttempt<T, E>) -> Result<T, E> {
        let attributes = SignalOutcomeAttributes {
            signal: completed.signal,
            outcome: completed.outcome,
        };
        if self.interests.contains(Interests::NODE_INPUT_METRICS) {
            self.attempted.with(attributes).record();
        }
        if let Some(duration) = completed.duration {
            self.duration.with(attributes).record(duration);
        }
        if let Some(payload_size) = completed.payload_size {
            self.payload.with(attributes).record(payload_size);
        }
        if let Some(items) = completed.items {
            self.items.with(attributes).record(items);
        }
        completed.result
    }

    /// Reports every touched shared exporter metric bucket.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.attempted)?;
        reporter.report_measurement(&mut self.duration)?;
        reporter.report_measurement(&mut self.payload)?;
        reporter.report_measurement(&mut self.items)
    }

    /// Takes every touched shared exporter metric bucket for terminal handoff.
    #[must_use]
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.attempted.terminal_snapshots();
        snapshots.extend(self.duration.terminal_snapshots());
        snapshots.extend(self.payload.terminal_snapshots());
        snapshots.extend(self.items.terminal_snapshots());
        snapshots
    }
}

impl ExporterAttempt {
    /// Classifies and returns an error from the attempt closure as failed.
    pub fn failed<E>(&self, error: E) -> ErrorWithOutcome<E> {
        ErrorWithOutcome {
            signal: None,
            outcome: Outcome::Failure,
            error,
        }
    }

    /// Classifies and returns an error from the attempt closure as refused.
    ///
    /// Pass the returned error directly to `Err`. Other errors are classified
    /// as failures, while successful results are classified as successes.
    pub fn refused<E>(&self, error: E) -> ErrorWithOutcome<E> {
        ErrorWithOutcome {
            signal: None,
            outcome: Outcome::Refused,
            error,
        }
    }

    /// Sets the signal item count without evaluating it when disabled.
    pub fn set_item_count_with(&mut self, item_count: impl FnOnce() -> u64) {
        if self.accepts_item_count {
            self.items = Some(item_count());
        }
    }

    /// Sets the encoded application payload size without evaluating it when disabled.
    pub fn set_payload_size_with(&mut self, payload_size: impl FnOnce() -> usize) {
        if self.accepts_payload_size {
            self.payload_size = Some(u64::try_from(payload_size()).unwrap_or(u64::MAX));
        }
    }

    /// Runs one exporter attempt and captures its terminal result.
    ///
    /// `Ok(value)` records success. Return errors through [`Self::failed`] or
    /// [`Self::refused`] to classify their terminal outcome.
    #[must_use = "the completed exporter attempt must be recorded"]
    pub async fn run<T, E>(
        mut self,
        work: impl AsyncFnOnce(&mut ExporterAttempt) -> Result<T, ErrorWithOutcome<E>>,
    ) -> CompletedExporterAttempt<T, E> {
        let (outcome, result) = match work(&mut self).await {
            Ok(value) => (Outcome::Success, Ok(value)),
            Err(ErrorWithOutcome { outcome, error, .. }) => (outcome, Err(error)),
        };
        CompletedExporterAttempt {
            signal: self.signal,
            outcome,
            duration: self.started_at.map(|started_at| started_at.elapsed()),
            payload_size: self.payload_size,
            items: self.items,
            result,
        }
    }
}

/// Completed export operations.
///
/// This set will be deprecated after exporters migrate to
/// the shared exporter attempt metrics and node-input terminal accounting.
#[metric_set(
    name = "exporter.exports",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ExporterExportMetrics {
    /// Number of messages whose export reached a terminal outcome.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
    /// Time from dequeuing PData through its terminal local or backend export result.
    /// Ack/Nack notification time is excluded.
    #[metric(name = "duration", unit = "s")]
    pub duration_seconds: HistogramNormal,
}

impl ExporterExportMetrics {
    /// Records one terminal export outcome and its end-to-end duration.
    #[inline]
    pub fn record(&mut self, duration: Duration) {
        self.messages.inc();
        self.duration_seconds.record(duration.as_secs_f64());
    }
}

/// Lifecycle and wire bytes for messages admitted by a receiver.
///
/// This set will be deprecated after receivers migrate to the shared receiver metrics.
#[metric_set(
    name = "receiver.messages",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ReceiverMessageMetrics {
    /// Number of decoded messages admitted to the pipeline send path.
    #[metric(unit = "{message}")]
    pub started: Counter<u64>,
    /// Number of admitted messages whose receiver work terminated.
    #[metric(unit = "{message}")]
    pub completed: Counter<u64>,
    /// Encoded transport payload bytes admitted to the pipeline send path.
    #[metric(unit = "By")]
    pub bytes: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use std::cell::Cell;

    fn new_attempted_metrics() -> MeasurementMetricSet<ExporterAttemptedMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ExporterAttemptedMetrics::register(&pipeline_ctx)
    }

    fn new_attempted_items_metrics() -> MeasurementMetricSet<ExporterAttemptedItemsMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ExporterAttemptedItemsMetrics::register(&pipeline_ctx)
    }

    fn new_attempted_duration_metrics() -> MeasurementMetricSet<ExporterAttemptedDurationMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ExporterAttemptedDurationMetrics::register(&pipeline_ctx)
    }

    fn new_attempted_payload_metrics() -> MeasurementMetricSet<ExporterAttemptedPayloadMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ExporterAttemptedPayloadMetrics::register(&pipeline_ctx)
    }

    fn new_received_metrics() -> MeasurementMetricSet<ReceiverReceivedMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ReceiverReceivedMetrics::register(&pipeline_ctx)
    }

    fn new_received_payload_metrics() -> MeasurementMetricSet<ReceiverReceivedPayloadMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ReceiverReceivedPayloadMetrics::register(&pipeline_ctx)
    }

    fn new_processing_metrics() -> MeasurementMetricSet<ReceiverProcessingMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ReceiverProcessingMetrics::register(&pipeline_ctx)
    }

    fn new_export_metrics() -> MeasurementMetricSet<ExporterExportMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ExporterExportMetrics::register(&pipeline_ctx)
    }

    fn new_receiver_metrics() -> MeasurementMetricSet<ReceiverMessageMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ReceiverMessageMetrics::register(&pipeline_ctx)
    }

    /// Scenario: Shared receiver and exporter sets produce boundary snapshots.
    /// Guarantees: Metric namespaces, units, and bounded dimensions match the external-boundary contract.
    #[test]
    fn external_boundary_metric_descriptors_are_stable() {
        let mut received = new_received_metrics();
        received
            .with(SignalOutcomeAttributes {
                signal: SignalType::Metrics,
                outcome: Outcome::Success,
            })
            .record();
        let received_snapshot = received
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("received snapshot");
        assert_eq!(received_snapshot.descriptor().name, "receiver.received");
        assert_eq!(
            received_snapshot.measurement_attribute_value("signal"),
            Some("metrics")
        );
        assert_eq!(
            received_snapshot.measurement_attribute_value("outcome"),
            Some("success")
        );
        assert!(
            received_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "messages" && metric.unit == "{message}")
        );
        assert!(
            received_snapshot
                .descriptor()
                .metrics
                .iter()
                .all(|metric| metric.name != "payload.size")
        );
        assert!(
            received_snapshot
                .descriptor()
                .metrics
                .iter()
                .all(|metric| metric.name != "duration")
        );

        let mut received_payload = new_received_payload_metrics();
        received_payload
            .with(SignalOutcomeAttributes {
                signal: SignalType::Metrics,
                outcome: Outcome::Success,
            })
            .record(64);
        let received_payload_snapshot = received_payload
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("received payload snapshot");
        assert_eq!(
            received_payload_snapshot.descriptor().name,
            "receiver.received"
        );
        assert!(
            received_payload_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "payload.size" && metric.unit == "By")
        );

        let mut processing = new_processing_metrics();
        processing
            .with(SignalAttributes {
                signal: SignalType::Metrics,
            })
            .record(Duration::from_millis(10));
        let processing_snapshot = processing
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("processing snapshot");
        assert_eq!(processing_snapshot.descriptor().name, "receiver.processing");
        assert_eq!(
            processing_snapshot.measurement_attribute_value("signal"),
            Some("metrics")
        );
        assert_eq!(
            processing_snapshot.measurement_attribute_value("outcome"),
            None
        );
        assert!(
            processing_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "duration" && metric.unit == "s")
        );

        let mut attempted = new_attempted_metrics();
        attempted
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .record();
        let attempted_snapshot = attempted
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("attempted snapshot");
        assert_eq!(attempted_snapshot.descriptor().name, "exporter.attempted");
        assert_eq!(
            attempted_snapshot.measurement_attribute_value("signal"),
            Some("logs")
        );
        assert_eq!(
            attempted_snapshot.measurement_attribute_value("outcome"),
            Some("success")
        );
        assert!(
            attempted_snapshot
                .descriptor()
                .metrics
                .iter()
                .all(|metric| metric.name != "items")
        );
        assert!(
            attempted_snapshot
                .descriptor()
                .metrics
                .iter()
                .all(|metric| metric.name != "payload.size")
        );
        assert!(
            attempted_snapshot
                .descriptor()
                .metrics
                .iter()
                .all(|metric| metric.name != "duration")
        );

        let mut attempted_duration = new_attempted_duration_metrics();
        attempted_duration
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .record(Duration::from_millis(20));
        let attempted_duration_snapshot = attempted_duration
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("attempted duration snapshot");
        assert_eq!(
            attempted_duration_snapshot.descriptor().name,
            "exporter.attempted"
        );
        assert!(
            attempted_duration_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "duration" && metric.unit == "s")
        );

        let mut attempted_payload = new_attempted_payload_metrics();
        attempted_payload
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .record(96);
        let attempted_payload_snapshot = attempted_payload
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("attempted payload snapshot");
        assert_eq!(
            attempted_payload_snapshot.descriptor().name,
            "exporter.attempted"
        );
        assert!(
            attempted_payload_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "payload.size" && metric.unit == "By")
        );

        let mut attempted_items = new_attempted_items_metrics();
        attempted_items
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .record(2);
        let attempted_items_snapshot = attempted_items
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("attempted items snapshot");
        assert_eq!(
            attempted_items_snapshot.descriptor().name,
            "exporter.attempted"
        );
        assert!(
            attempted_items_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "items" && metric.unit == "{item}")
        );
    }

    /// Scenario: all exporter measurements are disabled for one node.
    /// Guarantees: item inspection, message counting, clock timing, and payload-size snapshots are skipped.
    #[tokio::test]
    async fn exporter_helper_skips_disabled_optional_measurements() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
        let mut metrics = ExporterMetrics::register(&pipeline_ctx);
        let item_count_called = Cell::new(false);
        let payload_size_called = Cell::new(false);

        let completed = metrics
            .attempt(SignalType::Logs)
            .run(async |attempt| {
                attempt.set_item_count_with(|| {
                    item_count_called.set(true);
                    5
                });
                attempt.set_payload_size_with(|| {
                    payload_size_called.set(true);
                    128
                });
                Ok::<(), ErrorWithOutcome<()>>(())
            })
            .await;
        metrics.record(completed).expect("attempt succeeds");

        assert!(!item_count_called.get());
        assert!(!payload_size_called.get());
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: All optional exporter measurements are enabled for one node.
    /// Guarantees: One attempt records duration, encoded payload size, and lazily counted items under the same signal and outcome.
    #[tokio::test]
    async fn exporter_helper_records_enabled_optional_measurements() {
        let interests = Interests::NODE_INPUT_METRICS
            | Interests::NODE_LOCAL_DURATION
            | Interests::NODE_ITEM_COUNTS
            | Interests::NODE_SIZE;
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
        let mut metrics = ExporterMetrics::register(&pipeline_ctx);
        let item_count_called = Cell::new(false);
        let payload_size_called = Cell::new(false);

        let completed = metrics
            .attempt(SignalType::Metrics)
            .run(async |attempt| {
                attempt.set_item_count_with(|| {
                    item_count_called.set(true);
                    5
                });
                attempt.set_payload_size_with(|| {
                    payload_size_called.set(true);
                    128
                });
                Err::<(), _>(attempt.failed("export failed"))
            })
            .await;
        assert!(metrics.record(completed).is_err());

        assert!(item_count_called.get());
        assert!(payload_size_called.get());
        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 4);
        for metric_name in ["messages", "duration", "payload.size", "items"] {
            assert!(snapshots.iter().any(|snapshot| {
                snapshot.descriptor().name == "exporter.attempted"
                    && snapshot.measurement_attribute_value("signal") == Some("metrics")
                    && snapshot.measurement_attribute_value("outcome") == Some("failure")
                    && snapshot
                        .descriptor()
                        .metrics
                        .iter()
                        .any(|metric| metric.name == metric_name)
            }));
        }
    }

    /// Scenario: all receiver measurements are disabled for one node.
    /// Guarantees: message counting, clock timing, and payload-size snapshots are skipped.
    #[test]
    fn receiver_helper_skips_disabled_optional_measurements() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
        let mut metrics = ReceiverMetrics::register(&pipeline_ctx);
        let payload_size_called = Cell::new(false);

        let completed = metrics.processing().run(|processing| {
            processing.set_payload_size_with(|| {
                payload_size_called.set(true);
                128
            });
            Ok::<_, ErrorWithOutcome<()>>((SignalType::Logs, ()))
        });
        metrics.record(completed).expect("processing succeeds");

        assert!(!payload_size_called.get());
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: Receiver duration and payload-size measurements are enabled for one node.
    /// Guarantees: Processing and terminal received metrics remain separate and preserve their intended attributes.
    #[test]
    fn receiver_helper_records_enabled_optional_measurements() {
        let interests =
            Interests::NODE_OUTPUT_METRICS | Interests::NODE_LOCAL_DURATION | Interests::NODE_SIZE;
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
        let mut metrics = ReceiverMetrics::register(&pipeline_ctx);
        let payload_size_called = Cell::new(false);

        let completed = metrics.processing().run(|processing| {
            processing.set_payload_size_with(|| {
                payload_size_called.set(true);
                128
            });
            Err::<(SignalType, ()), _>(processing.failed(SignalType::Traces, "processing failed"))
        });
        assert!(metrics.record(completed).is_err());

        assert!(payload_size_called.get());
        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 3);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.processing"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.measurement_attribute_value("outcome").is_none()
        }));
        for metric_name in ["messages", "payload.size"] {
            assert!(snapshots.iter().any(|snapshot| {
                snapshot.descriptor().name == "receiver.received"
                    && snapshot.measurement_attribute_value("signal") == Some("traces")
                    && snapshot.measurement_attribute_value("outcome") == Some("failure")
                    && snapshot
                        .descriptor()
                        .metrics
                        .iter()
                        .any(|metric| metric.name == metric_name)
            }));
        }
    }

    /// Scenario: Receiver payload-size measurement is enabled but the boundary size is unavailable.
    /// Guarantees: The received message and processing duration are recorded without a synthetic zero-byte payload observation.
    #[test]
    fn receiver_helper_omits_unavailable_payload_size() {
        let interests =
            Interests::NODE_OUTPUT_METRICS | Interests::NODE_LOCAL_DURATION | Interests::NODE_SIZE;
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
        let mut metrics = ReceiverMetrics::register(&pipeline_ctx);

        let completed = metrics
            .processing()
            .run(|_| Ok::<_, ErrorWithOutcome<()>>((SignalType::Logs, ())));
        metrics.record(completed).expect("processing succeeds");

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);
        assert!(
            snapshots
                .iter()
                .any(|snapshot| { snapshot.descriptor().name == "receiver.processing" })
        );
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.received"
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .any(|metric| metric.name == "messages")
        }));
        assert!(snapshots.iter().all(|snapshot| {
            snapshot
                .descriptor()
                .metrics
                .iter()
                .all(|metric| metric.name != "payload.size")
        }));
    }

    /// Scenario: Receiver processing fails before the message signal can be classified.
    /// Guarantees: The original error is returned without emitting incorrectly attributed shared metrics.
    #[test]
    fn receiver_helper_allows_failure_before_signal_classification() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::all());
        let mut metrics = ReceiverMetrics::register(&pipeline_ctx);

        let completed = metrics
            .processing()
            .run(|_| Err::<(SignalType, ()), ErrorWithOutcome<_>>("invalid envelope".into()));
        assert_eq!(metrics.record(completed), Err("invalid envelope"));
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: A classified receiver operation is refused by local policy.
    /// Guarantees: The error and optional measurements are recorded with the refused outcome.
    #[test]
    fn receiver_helper_records_explicit_refused_outcome() {
        let interests =
            Interests::NODE_OUTPUT_METRICS | Interests::NODE_LOCAL_DURATION | Interests::NODE_SIZE;
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
        let mut metrics = ReceiverMetrics::register(&pipeline_ctx);

        let completed = metrics.processing().run(|processing| {
            processing.set_payload_size_with(|| 128);
            Err::<(SignalType, ()), _>(processing.refused(SignalType::Logs, "capacity"))
        });
        assert_eq!(metrics.record(completed), Err("capacity"));

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 3);
        assert!(snapshots.iter().all(|snapshot| {
            snapshot.measurement_attribute_value("signal") == Some("logs")
                && snapshot.measurement_attribute_value("outcome") != Some("failure")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.received"
                && snapshot.measurement_attribute_value("outcome") == Some("refused")
        }));
    }

    /// Scenario: An exporter attempt is refused by local policy before delivery.
    /// Guarantees: The original error is returned and the attempt is recorded as refused.
    #[tokio::test]
    async fn exporter_helper_records_explicit_refused_outcome() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
        let mut metrics = ExporterMetrics::register(&pipeline_ctx);

        let completed = metrics
            .attempt(SignalType::Metrics)
            .run(async |attempt| Err::<(), _>(attempt.refused("policy")))
            .await;
        assert_eq!(metrics.record(completed), Err("policy"));
        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        let snapshot = &snapshots[0];
        assert_eq!(snapshot.descriptor().name, "exporter.attempted");
        assert_eq!(
            snapshot.measurement_attribute_value("signal"),
            Some("metrics")
        );
        assert_eq!(
            snapshot.measurement_attribute_value("outcome"),
            Some("refused")
        );
        let messages = snapshot
            .descriptor()
            .metrics
            .iter()
            .position(|metric| metric.name == "messages")
            .expect("messages metric");
        assert_eq!(snapshot.get_metrics()[messages].to_u64_lossy(), 1);
    }

    /// Scenario: An exporter handles a refused primary error before a fallback fails.
    /// Guarantees: The discarded refusal cannot classify the returned fallback error as refused.
    #[tokio::test]
    async fn exporter_helper_keeps_outcome_attached_to_returned_error() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
        let mut metrics = ExporterMetrics::register(&pipeline_ctx);

        let completed = metrics
            .attempt(SignalType::Logs)
            .run(async |attempt| {
                let primary = Err::<(), _>(attempt.refused("primary refused"));
                assert!(primary.is_err());
                Err::<(), _>(attempt.failed("fallback failed"))
            })
            .await;
        assert_eq!(metrics.record(completed), Err("fallback failed"));

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(
            snapshots[0].measurement_attribute_value("outcome"),
            Some("failure")
        );
    }

    /// Scenario: One logical export requires a failed attempt followed by a successful retry.
    /// Guarantees: Every attempt records its own message under its attempt outcome.
    #[test]
    fn exporter_attempts_are_recorded_independently() {
        let mut metrics = new_attempted_metrics();
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Failure,
            })
            .record();
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .record();

        let failed = metrics.get(SignalOutcomeAttributes {
            signal: SignalType::Logs,
            outcome: Outcome::Failure,
        });
        assert_eq!(failed.messages.get(), 1);

        let succeeded = metrics.get(SignalOutcomeAttributes {
            signal: SignalType::Logs,
            outcome: Outcome::Success,
        });
        assert_eq!(succeeded.messages.get(), 1);
    }

    /// Scenario: An exporter completes successful and failed exports for multiple signals.
    /// Guarantees: Terminal counts and durations are recorded together and isolated by signal and outcome.
    #[test]
    fn exporter_metrics_are_partitioned_by_signal_and_outcome() {
        let mut metrics = new_export_metrics();
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .record(Duration::from_millis(250));
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Failure,
            })
            .record(Duration::from_millis(500));

        assert_eq!(
            metrics
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
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Metrics,
                    outcome: Outcome::Success,
                })
                .messages
                .get(),
            0
        );
        assert_eq!(
            metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .duration_seconds
                .get()
                .count(),
            1
        );
    }

    /// Scenario: Export outcome metrics are handed off during terminal shutdown twice.
    /// Guarantees: Only touched buckets are emitted and each bucket is cleared after handoff.
    #[test]
    fn terminal_snapshots_emit_touched_buckets_once() {
        let mut metrics = new_export_metrics();
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Traces,
                outcome: Outcome::Success,
            })
            .record(Duration::from_millis(250));

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.exports"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
        }));
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: A receiver admits and completes a logs message with an encoded payload.
    /// Guarantees: Lifecycle and wire-byte counters share one signal-isolated receiver bucket.
    #[test]
    fn receiver_message_metrics_track_lifecycle_and_wire_bytes() {
        let mut metrics = new_receiver_metrics();
        let messages = metrics.with(SignalAttributes {
            signal: SignalType::Logs,
        });
        messages.started.inc();
        messages.completed.inc();
        messages.bytes.add(42);

        let messages = metrics.get(SignalAttributes {
            signal: SignalType::Logs,
        });
        assert_eq!(messages.started.get(), 1);
        assert_eq!(messages.completed.get(), 1);
        assert_eq!(messages.bytes.get(), 42);
        assert_eq!(
            metrics
                .get(SignalAttributes {
                    signal: SignalType::Metrics,
                })
                .started
                .get(),
            0
        );
    }
}
