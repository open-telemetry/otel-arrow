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

/// Receiver-local handling of classified messages at the external ingress boundary.
#[metric_set(
    name = "receiver.ingress",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ReceiverIngressMetrics {
    /// Number of classified external messages whose receiver-local handling terminated.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
    /// Encoded application payload bytes observed at the receiver ingress boundary.
    #[metric(name = "wire_bytes", unit = "By")]
    pub wire_bytes: Counter<u64>,
    /// Receiver-local time from observing the classified message through termination.
    /// Downstream processing and Ack/Nack completion are excluded.
    #[metric(unit = "s")]
    pub duration: HistogramNormal,
}

impl ReceiverIngressMetrics {
    /// Records one classified external message when receiver-local handling terminates.
    #[inline]
    pub fn record(&mut self, duration: Duration, wire_bytes: u64) {
        self.messages.inc();
        self.wire_bytes.add(wire_bytes);
        self.duration.record(duration.as_secs_f64());
    }
}

/// Terminal external results observed at an exporter egress boundary.
#[metric_set(
    name = "exporter.egress",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ExporterEgressMetrics {
    /// Number of PData messages whose export reached a terminal external result.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
    /// Number of signal items whose export reached the terminal external result.
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
    /// Encoded application payload bytes submitted across all export attempts.
    #[metric(name = "wire_bytes", unit = "By")]
    pub wire_bytes: Counter<u64>,
    /// Time from dequeuing PData through its terminal local or backend export result.
    /// Ack/Nack notification time is excluded.
    #[metric(unit = "s")]
    pub duration: HistogramNormal,
}

impl ExporterEgressMetrics {
    /// Records one terminal export result.
    #[inline]
    pub fn record(&mut self, duration: Duration, items: u64, wire_bytes: u64) {
        self.messages.inc();
        self.items.add(items);
        self.wire_bytes.add(wire_bytes);
        self.duration.record(duration.as_secs_f64());
    }
}

/// Completed export operations.
///
/// This set will be deprecated after exporters migrate to [`ExporterEgressMetrics`].
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
/// This set will be deprecated after receivers migrate to [`ReceiverIngressMetrics`].
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

    fn new_egress_metrics() -> MeasurementMetricSet<ExporterEgressMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ExporterEgressMetrics::register(&pipeline_ctx)
    }

    fn new_ingress_metrics() -> MeasurementMetricSet<ReceiverIngressMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ReceiverIngressMetrics::register(&pipeline_ctx)
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

    /// Scenario: Shared ingress and egress sets produce terminal snapshots.
    /// Guarantees: Metric namespaces, units, and bounded dimensions match the external-boundary contract.
    #[test]
    fn external_boundary_metric_descriptors_are_stable() {
        let mut ingress = new_ingress_metrics();
        ingress
            .with(SignalOutcomeAttributes {
                signal: SignalType::Metrics,
                outcome: Outcome::Success,
            })
            .record(Duration::from_millis(10), 64);
        let ingress_snapshot = ingress
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("ingress snapshot");
        assert_eq!(ingress_snapshot.descriptor().name, "receiver.ingress");
        assert_eq!(
            ingress_snapshot.measurement_attribute_value("signal"),
            Some("metrics")
        );
        assert_eq!(
            ingress_snapshot.measurement_attribute_value("outcome"),
            Some("success")
        );
        assert!(
            ingress_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "messages" && metric.unit == "{message}")
        );
        assert!(
            ingress_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "wire_bytes" && metric.unit == "By")
        );
        assert!(
            ingress_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "duration" && metric.unit == "s")
        );

        let mut egress = new_egress_metrics();
        egress
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .record(Duration::from_millis(20), 2, 96);
        let egress_snapshot = egress
            .terminal_snapshots()
            .into_iter()
            .next()
            .expect("egress snapshot");
        assert_eq!(egress_snapshot.descriptor().name, "exporter.egress");
        assert_eq!(
            egress_snapshot.measurement_attribute_value("signal"),
            Some("logs")
        );
        assert_eq!(
            egress_snapshot.measurement_attribute_value("outcome"),
            Some("success")
        );
        assert!(
            egress_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "items" && metric.unit == "{item}")
        );
        assert!(
            egress_snapshot
                .descriptor()
                .metrics
                .iter()
                .any(|metric| metric.name == "wire_bytes" && metric.unit == "By")
        );
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
