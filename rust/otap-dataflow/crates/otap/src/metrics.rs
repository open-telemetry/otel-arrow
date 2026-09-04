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
use std::time::{Duration, Instant};

/// Receiver-local handling of classified messages received at the external boundary.
#[metric_set(
    name = "receiver.received",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ReceiverReceivedMetrics {
    /// Number of classified external messages whose receiver-local handling terminated.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
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
pub struct ReceiverReceivedPayloadMetrics {
    /// Encoded application payload size observed before receiver decoding.
    #[metric(name = "payload.size", unit = "By")]
    pub payload_size: Counter<u64>,
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
pub struct ReceiverProcessingMetrics {
    /// Component-defined receiver-local processing time.
    ///
    /// Each receiver documents its stable start and end boundary. Downstream
    /// processing, batching wait, handoff wait, and Ack/Nack completion are
    /// excluded.
    #[metric(unit = "s")]
    pub duration: HistogramNormal,
}

impl ReceiverProcessingMetrics {
    /// Records one receiver-local processing operation.
    #[inline]
    pub fn record(&mut self, duration: Duration) {
        self.duration.record(duration.as_secs_f64());
    }
}

/// Per-message state captured for shared receiver metrics.
#[derive(Debug)]
pub struct ReceiverOperation {
    started_at: Option<Instant>,
    payload_size: Option<u64>,
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

    /// Starts receiver-local processing for one classified external message.
    #[must_use]
    pub fn start_operation(&self, payload_size: usize) -> ReceiverOperation {
        ReceiverOperation {
            started_at: self
                .interests
                .contains(Interests::COMPONENT_DURATION)
                .then(Instant::now),
            payload_size: self
                .interests
                .contains(Interests::PRODUCED_CONSUMED_SIZE)
                .then(|| u64::try_from(payload_size).unwrap_or(u64::MAX)),
        }
    }

    /// Records terminal receiver handling and ends local processing before handoff.
    pub fn record_operation<T, E>(
        &mut self,
        signal: SignalType,
        result: &Result<T, E>,
        operation: ReceiverOperation,
    ) {
        let outcome = if result.is_ok() {
            Outcome::Success
        } else {
            Outcome::Failure
        };
        if let Some(started_at) = operation.started_at {
            self.processing
                .with(SignalAttributes { signal })
                .record(started_at.elapsed());
        }
        let attributes = SignalOutcomeAttributes { signal, outcome };
        self.received.with(attributes).record();
        if let Some(payload_size) = operation.payload_size {
            self.payload.with(attributes).record(payload_size);
        }
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

    /// Returns a received bucket for component tests.
    #[must_use]
    pub fn received_for(&self, attributes: SignalOutcomeAttributes) -> &ReceiverReceivedMetrics {
        self.received.get(attributes)
    }
}

/// Individual attempts to submit encoded application payloads from an exporter.
#[metric_set(
    name = "exporter.attempted",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ExporterAttemptedMetrics {
    /// Number of backend submission attempts.
    ///
    /// Retries count again. This differs from `node.input.messages`, which
    /// counts PData messages entering the exporter.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
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
pub struct ExporterAttemptedDurationMetrics {
    /// Time spent performing export attempts, including backend latency.
    #[metric(unit = "s")]
    pub duration: HistogramNormal,
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
pub struct ExporterAttemptedPayloadMetrics {
    /// Encoded application payload size submitted across export attempts.
    #[metric(name = "payload.size", unit = "By")]
    pub payload_size: Counter<u64>,
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
pub struct ExporterAttemptedItemsMetrics {
    /// Number of signal items submitted across export attempts.
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
}

impl ExporterAttemptedItemsMetrics {
    /// Records the item count for one export attempt.
    #[inline]
    pub fn record(&mut self, items: u64) {
        self.items.add(items);
    }
}

/// Per-message state captured only for enabled shared exporter metrics.
#[derive(Debug)]
pub struct ExporterAttempt {
    started_at: Option<Instant>,
    items: Option<u64>,
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

    /// Starts one attempt without paying optional measurement costs when disabled.
    pub fn start_attempt(&self, item_count: impl FnOnce() -> u64) -> ExporterAttempt {
        ExporterAttempt {
            started_at: self
                .interests
                .contains(Interests::COMPONENT_DURATION)
                .then(Instant::now),
            items: self
                .interests
                .contains(Interests::PRODUCED_CONSUMED_ITEM_COUNTS)
                .then(item_count),
        }
    }

    /// Records the terminal outcome of one exporter attempt.
    pub fn record_attempt<T, E>(
        &mut self,
        signal: SignalType,
        result: &Result<T, E>,
        payload_size: Option<usize>,
        attempt: ExporterAttempt,
    ) {
        let outcome = if result.is_ok() {
            Outcome::Success
        } else {
            Outcome::Failure
        };
        let attributes = SignalOutcomeAttributes { signal, outcome };
        self.attempted.with(attributes).record();
        if let Some(started_at) = attempt.started_at {
            self.duration.with(attributes).record(started_at.elapsed());
        }
        if self.interests.contains(Interests::PRODUCED_CONSUMED_SIZE)
            && let Some(payload_size) = payload_size
        {
            self.payload
                .with(attributes)
                .record(u64::try_from(payload_size).unwrap_or(u64::MAX));
        }
        if let Some(items) = attempt.items {
            self.items.with(attributes).record(items);
        }
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

    /// Returns an attempt bucket for component tests.
    #[must_use]
    pub fn attempted_for(&self, attributes: SignalOutcomeAttributes) -> &ExporterAttemptedMetrics {
        self.attempted.get(attributes)
    }
}

/// Completed export operations.
///
/// This set will be deprecated after exporters migrate to
/// [`ExporterAttemptedMetrics`] and node-consumer terminal accounting.
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
/// This set will be deprecated after receivers migrate to [`ReceiverReceivedMetrics`].
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

    /// Scenario: Optional exporter measurements are disabled for one node.
    /// Guarantees: Item inspection, clock timing, and payload-size snapshots are skipped while the attempt message is recorded.
    #[test]
    fn exporter_helper_skips_disabled_optional_measurements() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
        let mut metrics = ExporterMetrics::register(&pipeline_ctx);
        let item_count_called = Cell::new(false);

        let attempt = metrics.start_attempt(|| {
            item_count_called.set(true);
            5
        });
        metrics.record_attempt(SignalType::Logs, &Ok::<(), ()>(()), Some(128), attempt);

        assert!(!item_count_called.get());
        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].descriptor().name, "exporter.attempted");
        assert!(
            snapshots[0]
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "messages")
        );
    }

    /// Scenario: All optional exporter measurements are enabled for one node.
    /// Guarantees: One attempt records duration, encoded payload size, and lazily counted items under the same signal and outcome.
    #[test]
    fn exporter_helper_records_enabled_optional_measurements() {
        let interests = Interests::COMPONENT_DURATION
            | Interests::PRODUCED_CONSUMED_ITEM_COUNTS
            | Interests::PRODUCED_CONSUMED_SIZE;
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
        let mut metrics = ExporterMetrics::register(&pipeline_ctx);
        let item_count_called = Cell::new(false);

        let attempt = metrics.start_attempt(|| {
            item_count_called.set(true);
            5
        });
        metrics.record_attempt(SignalType::Metrics, &Err::<(), ()>(()), Some(128), attempt);

        assert!(item_count_called.get());
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

    /// Scenario: Optional receiver measurements are disabled for one node.
    /// Guarantees: Terminal message accounting emits without a duration or zero-valued payload-size snapshot.
    #[test]
    fn receiver_helper_skips_disabled_optional_measurements() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
        let mut metrics = ReceiverMetrics::register(&pipeline_ctx);

        let operation = metrics.start_operation(128);
        metrics.record_operation(SignalType::Logs, &Ok::<(), ()>(()), operation);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].descriptor().name, "receiver.received");
        assert!(
            snapshots[0]
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "messages")
        );
    }

    /// Scenario: Receiver duration and payload-size measurements are enabled for one node.
    /// Guarantees: Processing and terminal received metrics remain separate and preserve their intended attributes.
    #[test]
    fn receiver_helper_records_enabled_optional_measurements() {
        let interests = Interests::COMPONENT_DURATION | Interests::PRODUCED_CONSUMED_SIZE;
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
        let mut metrics = ReceiverMetrics::register(&pipeline_ctx);

        let operation = metrics.start_operation(128);
        metrics.record_operation(SignalType::Traces, &Err::<(), ()>(()), operation);

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
