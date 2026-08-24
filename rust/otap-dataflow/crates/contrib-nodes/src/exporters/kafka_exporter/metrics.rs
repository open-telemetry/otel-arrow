// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded-cardinality internal telemetry for the Kafka exporter.

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_otap::metrics::ExporterExportMetrics;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, HistogramNormal};
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use rdkafka::error::KafkaError;
use rdkafka::types::RDKafkaErrorCode;
use std::time::Duration;

/// Bounded reason that a Kafka export failed before or during delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum KafkaExporterErrorType {
    /// The incoming signal has no Kafka exporter configuration.
    UnconfiguredSignal,
    /// A dynamic topic supplied by a transport header was invalid.
    InvalidTopic,
    /// The pdata payload could not be encoded for Kafka.
    Encoding,
    /// The local librdkafka producer queue stayed full until the enqueue deadline.
    QueueFull,
    /// The enqueue, request, or broker delivery deadline expired.
    Timeout,
    /// The encoded Kafka record exceeded a client or broker size limit.
    MessageTooLarge,
    /// Broker authentication or TLS authentication failed.
    Authentication,
    /// The principal lacks permission for the requested Kafka operation.
    Authorization,
    /// The destination topic or partition is unavailable.
    UnknownTopicOrPartition,
    /// The broker could not meet the configured in-sync replica requirement.
    InsufficientReplicas,
    /// Broker connectivity, name resolution, or network transport failed.
    Transport,
    /// The delivery failure does not fit another bounded category.
    Other,
}

impl KafkaExporterErrorType {
    /// Classifies a librdkafka delivery error into an operator-actionable bounded category.
    #[must_use]
    pub fn from_kafka_error(error: &KafkaError) -> Self {
        match error.rdkafka_error_code() {
            Some(RDKafkaErrorCode::QueueFull) => Self::QueueFull,
            Some(
                RDKafkaErrorCode::MessageTimedOut
                | RDKafkaErrorCode::OperationTimedOut
                | RDKafkaErrorCode::TimedOutQueue
                | RDKafkaErrorCode::RequestTimedOut,
            ) => Self::Timeout,
            Some(
                RDKafkaErrorCode::InvalidMessageSize
                | RDKafkaErrorCode::MessageSizeTooLarge
                | RDKafkaErrorCode::MessageBatchTooLarge,
            ) => Self::MessageTooLarge,
            Some(
                RDKafkaErrorCode::Authentication
                | RDKafkaErrorCode::SSL
                | RDKafkaErrorCode::SaslAuthenticationFailed
                | RDKafkaErrorCode::UnsupportedSASLMechanism
                | RDKafkaErrorCode::IllegalSASLState,
            ) => Self::Authentication,
            Some(
                RDKafkaErrorCode::TopicAuthorizationFailed
                | RDKafkaErrorCode::GroupAuthorizationFailed
                | RDKafkaErrorCode::ClusterAuthorizationFailed
                | RDKafkaErrorCode::TransactionalIdAuthorizationFailed
                | RDKafkaErrorCode::DelegationTokenAuthorizationFailed,
            ) => Self::Authorization,
            Some(
                RDKafkaErrorCode::UnknownTopic
                | RDKafkaErrorCode::UnknownPartition
                | RDKafkaErrorCode::UnknownTopicOrPartition
                | RDKafkaErrorCode::InvalidTopic,
            ) => Self::UnknownTopicOrPartition,
            Some(
                RDKafkaErrorCode::ISRInsufficient
                | RDKafkaErrorCode::NotEnoughReplicas
                | RDKafkaErrorCode::NotEnoughReplicasAfterAppend,
            ) => Self::InsufficientReplicas,
            Some(
                RDKafkaErrorCode::BrokerDestroy
                | RDKafkaErrorCode::DestroyBroker
                | RDKafkaErrorCode::BrokerTransportFailure
                | RDKafkaErrorCode::Resolve
                | RDKafkaErrorCode::AllBrokersDown
                | RDKafkaErrorCode::UnknownBroker
                | RDKafkaErrorCode::BrokerNotAvailable
                | RDKafkaErrorCode::LeaderNotAvailable
                | RDKafkaErrorCode::NotLeaderForPartition
                | RDKafkaErrorCode::ReplicaNotAvailable
                | RDKafkaErrorCode::CoordinatorNotAvailable
                | RDKafkaErrorCode::NotCoordinator
                | RDKafkaErrorCode::ListenerNotFound
                | RDKafkaErrorCode::NetworkException,
            ) => Self::Transport,
            _ => Self::Other,
        }
    }
}

/// Timed phase of one Kafka export attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum KafkaExporterOperation {
    /// Convert pdata into the configured Kafka wire encoding.
    Encoding,
    /// Enqueue the encoded record and wait for its broker delivery result.
    Delivery,
}

/// Source used to resolve a Kafka destination topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum KafkaTopicSource {
    /// A captured transport header supplied the destination topic.
    Header,
    /// Static per-signal configuration supplied the destination topic.
    StaticConfig,
}

/// Signal and failure reason dimensions for failed Kafka exports.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct KafkaExporterFailureAttributes {
    /// Signal carried by the failed export.
    pub signal: SignalType,
    /// Bounded Kafka export failure category.
    #[attribute_key = "error.type"]
    pub error_type: KafkaExporterErrorType,
}

/// Signal, operation, and outcome dimensions for Kafka export phase latency.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct KafkaExporterOperationAttributes {
    /// Signal carried by the export attempt.
    pub signal: SignalType,
    /// Timed exporter phase.
    pub operation: KafkaExporterOperation,
    /// Terminal outcome of the phase.
    pub outcome: Outcome,
}

/// Signal and bounded topic-source dimensions for successful routing decisions.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct KafkaExporterRoutingAttributes {
    /// Signal whose destination was resolved.
    pub signal: SignalType,
    /// Bounded source of the resolved topic.
    #[attribute_key = "topic.source"]
    pub source: KafkaTopicSource,
}

/// Encoded Kafka export payload measurements.
#[metric_set(
    name = "exporter.kafka.exports",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaExporterExportMetrics {
    /// Encoded Kafka payload bytes for attempts that reached the producer.
    #[metric(unit = "By")]
    pub bytes: HistogramNormal,
}

/// Kafka export phase latency.
#[metric_set(
    name = "exporter.kafka.operations",
    measurement_attributes = KafkaExporterOperationAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaExporterOperationMetrics {
    /// Time spent in the selected exporter phase.
    #[metric(unit = "s")]
    pub duration: HistogramNormal,
}

/// Failed Kafka export attempts classified by actionable reason.
#[metric_set(
    name = "exporter.kafka.failures",
    measurement_attributes = KafkaExporterFailureAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaExporterFailureMetrics {
    /// Number of failed export attempts.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

/// Kafka topic-routing decisions without recording unbounded topic names.
#[metric_set(
    name = "exporter.kafka.routing",
    measurement_attributes = KafkaExporterRoutingAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaExporterRoutingMetrics {
    /// Number of messages routed using the selected bounded source.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

/// Composite metrics for the Kafka exporter.
#[derive(Debug)]
pub struct KafkaExporterMetrics {
    /// Generic terminal export counts shared by all aligned exporters.
    pub exports: MeasurementMetricSet<ExporterExportMetrics>,
    /// Kafka-specific encoded payload measurements.
    pub kafka_exports: MeasurementMetricSet<KafkaExporterExportMetrics>,
    /// Kafka exporter phase latency.
    pub operations: MeasurementMetricSet<KafkaExporterOperationMetrics>,
    /// Kafka-specific failure classifications.
    pub failures: MeasurementMetricSet<KafkaExporterFailureMetrics>,
    /// Bounded topic-routing decisions.
    pub routing: MeasurementMetricSet<KafkaExporterRoutingMetrics>,
}

impl KafkaExporterMetrics {
    /// Registers all Kafka exporter metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            exports: ExporterExportMetrics::register(pipeline_ctx),
            kafka_exports: KafkaExporterExportMetrics::register(pipeline_ctx),
            operations: KafkaExporterOperationMetrics::register(pipeline_ctx),
            failures: KafkaExporterFailureMetrics::register(pipeline_ctx),
            routing: KafkaExporterRoutingMetrics::register(pipeline_ctx),
        }
    }

    /// Records the terminal outcome, duration, and optional encoded bytes of one export.
    fn record_export(
        &mut self,
        signal: SignalType,
        outcome: Outcome,
        duration: Duration,
        payload_bytes: Option<usize>,
    ) {
        let attributes = SignalOutcomeAttributes { signal, outcome };
        self.exports.with(attributes).record(duration);
        if let Some(payload_bytes) = payload_bytes {
            self.kafka_exports
                .with(attributes)
                .bytes
                .record(payload_bytes as f64);
        }
    }

    /// Records one successful terminal Kafka export.
    pub fn record_success(&mut self, signal: SignalType, duration: Duration, payload_bytes: usize) {
        self.record_export(signal, Outcome::Success, duration, Some(payload_bytes));
    }

    /// Records the latency and outcome of one bounded export phase.
    pub fn record_operation(
        &mut self,
        signal: SignalType,
        operation: KafkaExporterOperation,
        outcome: Outcome,
        duration_seconds: f64,
    ) {
        self.operations
            .with(KafkaExporterOperationAttributes {
                signal,
                operation,
                outcome,
            })
            .duration
            .record(duration_seconds);
    }

    /// Records one failed terminal Kafka export and exactly one diagnostic category.
    pub fn record_failure(
        &mut self,
        signal: SignalType,
        error_type: KafkaExporterErrorType,
        duration: Duration,
        payload_bytes: Option<usize>,
    ) {
        self.record_export(signal, Outcome::Failure, duration, payload_bytes);
        self.failures
            .with(KafkaExporterFailureAttributes { signal, error_type })
            .messages
            .inc();
    }

    /// Records every operational observation for a failed Kafka delivery.
    pub fn record_delivery_failure(
        &mut self,
        signal: SignalType,
        error: &KafkaError,
        delivery_duration_seconds: f64,
        export_duration: Duration,
        payload_bytes: usize,
    ) {
        self.record_operation(
            signal,
            KafkaExporterOperation::Delivery,
            Outcome::Failure,
            delivery_duration_seconds,
        );
        self.record_failure(
            signal,
            KafkaExporterErrorType::from_kafka_error(error),
            export_duration,
            Some(payload_bytes),
        );
    }

    /// Records the bounded source of a successful topic-routing decision.
    pub fn record_routing(&mut self, signal: SignalType, source: KafkaTopicSource) {
        self.routing
            .with(KafkaExporterRoutingAttributes { signal, source })
            .messages
            .inc();
    }

    /// Returns a routing bucket for inspection without marking it for export.
    #[must_use]
    pub fn routing_for(
        &self,
        signal: SignalType,
        source: KafkaTopicSource,
    ) -> &KafkaExporterRoutingMetrics {
        self.routing
            .get(KafkaExporterRoutingAttributes { signal, source })
    }

    /// Reports every touched Kafka exporter metric bucket.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.exports)?;
        reporter.report_measurement(&mut self.kafka_exports)?;
        reporter.report_measurement(&mut self.operations)?;
        reporter.report_measurement(&mut self.failures)?;
        reporter.report_measurement(&mut self.routing)
    }

    /// Takes every touched Kafka exporter metric bucket for terminal handoff.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.exports.terminal_snapshots();
        snapshots.extend(self.kafka_exports.terminal_snapshots());
        snapshots.extend(self.operations.terminal_snapshots());
        snapshots.extend(self.failures.terminal_snapshots());
        snapshots.extend(self.routing.terminal_snapshots());
        snapshots
    }

    #[cfg(test)]
    fn kafka_exports_for(
        &self,
        signal: SignalType,
        outcome: Outcome,
    ) -> &KafkaExporterExportMetrics {
        self.kafka_exports
            .get(SignalOutcomeAttributes { signal, outcome })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporters::kafka_exporter::exporter::test_support::pipeline_context;

    fn new_metrics() -> KafkaExporterMetrics {
        KafkaExporterMetrics::register(&pipeline_context())
    }

    /// Scenario: librdkafka delivery failures cover each operator-actionable category.
    /// Guarantees: Error codes map to bounded queue, timeout, size, auth, topic, replica, transport, and fallback values.
    #[test]
    fn delivery_errors_are_classified_into_bounded_categories() {
        let cases = [
            (
                RDKafkaErrorCode::QueueFull,
                KafkaExporterErrorType::QueueFull,
            ),
            (
                RDKafkaErrorCode::MessageTimedOut,
                KafkaExporterErrorType::Timeout,
            ),
            (
                RDKafkaErrorCode::MessageSizeTooLarge,
                KafkaExporterErrorType::MessageTooLarge,
            ),
            (
                RDKafkaErrorCode::Authentication,
                KafkaExporterErrorType::Authentication,
            ),
            (
                RDKafkaErrorCode::TopicAuthorizationFailed,
                KafkaExporterErrorType::Authorization,
            ),
            (
                RDKafkaErrorCode::UnknownTopicOrPartition,
                KafkaExporterErrorType::UnknownTopicOrPartition,
            ),
            (
                RDKafkaErrorCode::NotEnoughReplicas,
                KafkaExporterErrorType::InsufficientReplicas,
            ),
            (
                RDKafkaErrorCode::BrokerTransportFailure,
                KafkaExporterErrorType::Transport,
            ),
            (
                RDKafkaErrorCode::InvalidArgument,
                KafkaExporterErrorType::Other,
            ),
        ];

        for (code, expected) in cases {
            let error = KafkaError::MessageProduction(code);
            assert_eq!(KafkaExporterErrorType::from_kafka_error(&error), expected);
        }
    }

    /// Scenario: Successful and failed exports span signals, phases, failures, and routing sources.
    /// Guarantees: Terminal failures are paired with one diagnostic category and every measurement remains isolated by its bounded attributes.
    #[test]
    fn exporter_metrics_are_partitioned_by_context() {
        let mut metrics = new_metrics();
        metrics.record_success(SignalType::Logs, Duration::from_millis(250), 128);
        metrics.record_failure(
            SignalType::Traces,
            KafkaExporterErrorType::Transport,
            Duration::from_millis(500),
            None,
        );
        metrics.record_operation(
            SignalType::Logs,
            KafkaExporterOperation::Encoding,
            Outcome::Success,
            0.01,
        );
        metrics.record_routing(SignalType::Logs, KafkaTopicSource::Header);

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
                    signal: SignalType::Traces,
                    outcome: Outcome::Failure,
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
                    outcome: Outcome::Success,
                })
                .duration_seconds
                .get()
                .count(),
            1
        );
        assert_eq!(
            metrics
                .kafka_exports_for(SignalType::Logs, Outcome::Success)
                .bytes
                .get()
                .count(),
            1
        );
        assert_eq!(
            metrics
                .operations
                .get(KafkaExporterOperationAttributes {
                    signal: SignalType::Logs,
                    operation: KafkaExporterOperation::Encoding,
                    outcome: Outcome::Success,
                })
                .duration
                .get()
                .count(),
            1
        );
        assert_eq!(
            metrics
                .failures
                .get(KafkaExporterFailureAttributes {
                    signal: SignalType::Traces,
                    error_type: KafkaExporterErrorType::Transport,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .routing
                .get(KafkaExporterRoutingAttributes {
                    signal: SignalType::Logs,
                    source: KafkaTopicSource::Header,
                })
                .messages
                .get(),
            1
        );
    }

    /// Scenario: Kafka reports a timed-out delivery after encoding a logs payload.
    /// Guarantees: One helper call records the delivery phase, classified failure, terminal outcome, duration, and payload bytes.
    #[test]
    fn delivery_failure_records_the_complete_operational_context() {
        let mut metrics = new_metrics();
        let error = KafkaError::MessageProduction(RDKafkaErrorCode::MessageTimedOut);

        metrics.record_delivery_failure(
            SignalType::Logs,
            &error,
            0.25,
            Duration::from_millis(500),
            128,
        );

        assert_eq!(
            metrics
                .operations
                .get(KafkaExporterOperationAttributes {
                    signal: SignalType::Logs,
                    operation: KafkaExporterOperation::Delivery,
                    outcome: Outcome::Failure,
                })
                .duration
                .get()
                .count(),
            1
        );
        assert_eq!(
            metrics
                .failures
                .get(KafkaExporterFailureAttributes {
                    signal: SignalType::Logs,
                    error_type: KafkaExporterErrorType::Timeout,
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
                .exports
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Failure,
                })
                .duration_seconds
                .get()
                .count(),
            1
        );
        assert_eq!(
            metrics
                .kafka_exports_for(SignalType::Logs, Outcome::Failure)
                .bytes
                .get()
                .count(),
            1
        );
    }

    /// Scenario: Kafka exporter enum metrics are transferred into terminal snapshots twice.
    /// Guarantees: Wire values are preserved on first handoff and all touched buckets then clear.
    #[test]
    fn terminal_snapshots_preserve_enum_attribute_values_once() {
        let mut metrics = new_metrics();
        metrics.record_failure(
            SignalType::Metrics,
            KafkaExporterErrorType::Encoding,
            Duration::from_millis(500),
            Some(64),
        );
        metrics.record_routing(SignalType::Metrics, KafkaTopicSource::StaticConfig);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 4);
        assert_eq!(
            snapshots
                .iter()
                .filter(|snapshot| snapshot.descriptor().name == "exporter.exports")
                .count(),
            1
        );
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.exports"
                && snapshot.measurement_attribute_value("signal") == Some("metrics")
                && snapshot.measurement_attribute_value("outcome") == Some("failure")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .any(|field| field.name == "messages")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.exports"
                && snapshot.measurement_attribute_value("signal") == Some("metrics")
                && snapshot.measurement_attribute_value("outcome") == Some("failure")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .any(|field| field.name == "duration")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.kafka.exports"
                && snapshot.measurement_attribute_value("signal") == Some("metrics")
                && snapshot.measurement_attribute_value("outcome") == Some("failure")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .any(|field| field.name == "bytes")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.kafka.failures"
                && snapshot.measurement_attribute_value("error.type") == Some("encoding")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.kafka.routing"
                && snapshot.measurement_attribute_value("topic.source") == Some("static_config")
        }));
        assert!(metrics.terminal_snapshots().is_empty());
    }
}
