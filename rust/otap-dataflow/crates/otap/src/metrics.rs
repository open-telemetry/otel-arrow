// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic metrics used in the OTAP pipeline.
//!
//! Note: We try as much as possible to follow the following
//! [RFC Pipeline Component Telemetry](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).

use otel_arrow_dfe_telemetry::common_attributes::{SignalAttributes, SignalOutcomeAttributes};
use otel_arrow_dfe_telemetry::instrument::{Counter, HistogramNormal};
use otel_arrow_dfe_telemetry_macros::metric_set;
use std::time::Duration;

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
    /// Encoded application payload size observed before receiver decoding.
    #[metric(name = "payload.size", unit = "By")]
    pub payload_size: Counter<u64>,
    /// Receiver-local time from observing the classified message through termination.
    /// Downstream processing and Ack/Nack completion are excluded.
    #[metric(unit = "s")]
    pub duration: HistogramNormal,
}

impl ReceiverReceivedMetrics {
    /// Records one classified external message when receiver-local handling terminates.
    #[inline]
    pub fn record(&mut self, duration: Duration, payload_size: u64) {
        self.messages.inc();
        self.payload_size.add(payload_size);
        self.duration.record(duration.as_secs_f64());
    }
}

/// Individual attempts to submit encoded application payloads from an exporter.
#[metric_set(
    name = "exporter.attempted",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ExporterAttemptedMetrics {
    /// Number of PData messages submitted across export attempts.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
    /// Time spent performing export attempts.
    #[metric(unit = "s")]
    pub duration: HistogramNormal,
}

impl ExporterAttemptedMetrics {
    /// Records one export attempt.
    #[inline]
    pub fn record(&mut self, duration: Duration) {
        self.messages.inc();
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
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::common_attributes::Outcome;
    use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

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

    /// Scenario: Shared received and attempted sets produce terminal snapshots.
    /// Guarantees: Metric namespaces, units, and bounded dimensions match the external-boundary contract.
    #[test]
    fn external_boundary_metric_descriptors_are_stable() {
        let mut received = new_received_metrics();
        received
            .with(SignalOutcomeAttributes {
                signal: SignalType::Metrics,
                outcome: Outcome::Success,
            })
            .record(Duration::from_millis(10), 64);
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
                .any(|metric| metric.name == "payload.size" && metric.unit == "By")
        );
        assert!(
            received_snapshot
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
            .record(Duration::from_millis(20));
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
            .record(Duration::from_millis(10));
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .record(Duration::from_millis(20));

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
