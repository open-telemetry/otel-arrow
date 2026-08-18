// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Kafka receiver node.

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_otap::metrics::ReceiverMessageMetrics;
use otap_df_telemetry::common_attributes::{
    Outcome, OutcomeAttributes, ReceiverRejectionErrorType, SignalAttributes,
    SignalOutcomeAttributes,
};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge, ObserveUpDownCounter};
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use rdkafka::error::KafkaError;
use rdkafka::types::RDKafkaErrorCode;

/// Kafka-specific reason for rejecting a consumed message before pipeline admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum KafkaReceiverRejectionReason {
    /// The Kafka record contained no payload.
    EmptyPayload,
    /// The Kafka topic did not map to a configured signal.
    UnknownTopic,
    /// The payload could not be decoded using the configured signal encoding.
    Decode,
    /// The receiver exhausted the compact topic ID space used in acknowledgement routing.
    TopicIdExhausted,
    /// An unexpected receiver-local condition rejected the message.
    Internal,
}

/// Signal context for a Kafka message rejected before decoding completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum KafkaReceiverRejectionSignal {
    /// The rejected message was routed to the traces decoder.
    Traces,
    /// The rejected message was routed to the metrics decoder.
    Metrics,
    /// The rejected message was routed to the logs decoder.
    Logs,
    /// The signal could not be established before rejection.
    Unknown,
}

impl From<SignalType> for KafkaReceiverRejectionSignal {
    fn from(signal: SignalType) -> Self {
        match signal {
            SignalType::Traces => Self::Traces,
            SignalType::Metrics => Self::Metrics,
            SignalType::Logs => Self::Logs,
        }
    }
}

/// Signal, generic error category, and Kafka-specific reason for a rejected message.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct KafkaReceiverRejectionAttributes {
    /// Signal selected from topic routing, or `unknown` when routing did not complete.
    pub signal: KafkaReceiverRejectionSignal,
    /// Generic receiver rejection category shared with other receivers.
    #[attribute_key = "error.type"]
    pub error_type: ReceiverRejectionErrorType,
    /// Kafka-specific bounded reason used to choose an operator response.
    pub reason: KafkaReceiverRejectionReason,
}

/// Bounded category for a Kafka consumer transport error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum KafkaReceiverTransportErrorType {
    /// Broker connectivity, name resolution, or network transport failed.
    Transport,
    /// A consumer or broker request deadline expired.
    Timeout,
    /// Broker authentication or TLS authentication failed.
    Authentication,
    /// The principal lacks permission for the requested Kafka operation.
    Authorization,
    /// A subscribed topic or partition is unavailable.
    UnknownTopicOrPartition,
    /// The stored or requested consumer offset cannot be used.
    Offset,
    /// The consumer exceeded its configured maximum polling interval.
    PollExceeded,
    /// The transport failure does not fit another bounded category.
    Other,
}

impl KafkaReceiverTransportErrorType {
    fn from_kafka_error(error: &KafkaError) -> Self {
        match error.rdkafka_error_code() {
            Some(
                RDKafkaErrorCode::MessageTimedOut
                | RDKafkaErrorCode::OperationTimedOut
                | RDKafkaErrorCode::TimedOutQueue
                | RDKafkaErrorCode::RequestTimedOut,
            ) => Self::Timeout,
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
                RDKafkaErrorCode::OffsetOutOfRange
                | RDKafkaErrorCode::NoOffset
                | RDKafkaErrorCode::AutoOffsetReset,
            ) => Self::Offset,
            Some(RDKafkaErrorCode::PollExceeded) => Self::PollExceeded,
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

/// Bounded error category for a Kafka consumer transport failure.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct KafkaReceiverTransportAttributes {
    /// Operator-actionable transport error category.
    #[attribute_key = "error.type"]
    pub error_type: KafkaReceiverTransportErrorType,
}

/// Downstream acknowledgement results for admitted Kafka messages.
#[metric_set(
    name = "receiver.kafka.acknowledgements",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaReceiverAcknowledgementMetrics {
    /// Number of downstream acknowledgement responses.
    #[metric(unit = "{response}")]
    pub responses: Counter<u64>,
}

/// Kafka messages rejected before pipeline admission.
#[metric_set(
    name = "receiver.kafka.rejections",
    measurement_attributes = KafkaReceiverRejectionAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaReceiverRejectionMetrics {
    /// Number of rejected Kafka messages.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

/// Broker offset commit results.
#[metric_set(
    name = "receiver.kafka.offset_commits",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaReceiverOffsetCommitMetrics {
    /// Number of offset commit results reported by librdkafka.
    #[metric(unit = "{commit}")]
    pub commits: Counter<u64>,
}

/// Kafka consumer health and consumer-group activity.
#[metric_set(name = "receiver.kafka.consumer")]
#[derive(Debug, Default, Clone)]
pub struct KafkaReceiverConsumerMetrics {
    /// Kafka records delivered by the consumer before filtering or decoding.
    #[metric(name = "records.received", unit = "{message}")]
    pub records_received: Counter<u64>,
    /// Kafka record payload bytes delivered before filtering or decoding.
    #[metric(name = "records.bytes", unit = "By")]
    pub record_bytes: Counter<u64>,
    /// Current number of delivered records awaiting acknowledgement and commit progress.
    #[metric(name = "records.inflight", unit = "{message}")]
    pub records_in_flight: ObserveUpDownCounter<u64>,
    /// Messages skipped because their offsets were already tracked in this ownership generation.
    #[metric(name = "records.duplicates", unit = "{message}")]
    pub duplicate_records: Counter<u64>,
    /// Consumer-group assignment events observed by this consumer.
    #[metric(name = "group.rebalances", unit = "{rebalance}")]
    pub rebalances: Counter<u64>,
    /// Current number of partitions owned by this consumer.
    #[metric(name = "group.partitions", unit = "{partition}")]
    pub partitions: ObserveUpDownCounter<u64>,
    /// Partitions newly acquired by this consumer across rebalances.
    #[metric(name = "group.partition.assignments", unit = "{partition}")]
    pub partition_assignments: Counter<u64>,
    /// Owned partitions revoked from this consumer across rebalances.
    #[metric(name = "group.partition.revocations", unit = "{partition}")]
    pub partition_revocations: Counter<u64>,
    /// Synchronous commit calls that failed while partitions were being revoked.
    #[metric(name = "group.rebalance.commit_failures", unit = "{error}")]
    pub rebalance_commit_failures: Counter<u64>,
    /// Mean broker-committed consumer-group lag across every owned partition.
    #[metric(name = "group.lag", unit = "{message}")]
    pub lag: Gauge<f64>,
    /// Ack or nack responses ignored because their partition ownership was stale.
    #[metric(name = "group.feedback.after_revocation", unit = "{response}")]
    pub feedback_after_revocation: Counter<u64>,
}

/// Transport-level Kafka receiver errors.
#[metric_set(
    name = "receiver.kafka.transport",
    measurement_attributes = KafkaReceiverTransportAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaReceiverTransportMetrics {
    /// Number of non-EOF errors returned by the Kafka consumer.
    #[metric(unit = "{error}")]
    pub errors: Counter<u64>,
}

/// Bounded-cardinality Kafka receiver metrics tracker.
pub struct KafkaReceiverMetrics {
    /// Admitted message lifecycle metrics.
    pub messages: MeasurementMetricSet<ReceiverMessageMetrics>,
    /// Downstream acknowledgement metrics.
    pub acknowledgements: MeasurementMetricSet<KafkaReceiverAcknowledgementMetrics>,
    /// Pre-admission rejection metrics.
    pub rejections: MeasurementMetricSet<KafkaReceiverRejectionMetrics>,
    /// Offset commit outcome metrics.
    pub offset_commits: MeasurementMetricSet<KafkaReceiverOffsetCommitMetrics>,
    /// Fixed consumer ingress and health metrics.
    pub consumer: MetricSet<KafkaReceiverConsumerMetrics>,
    /// Fixed transport metrics.
    pub transport: MeasurementMetricSet<KafkaReceiverTransportMetrics>,
}

impl KafkaReceiverMetrics {
    /// Registers all Kafka receiver metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            messages: ReceiverMessageMetrics::register(pipeline_ctx),
            acknowledgements: KafkaReceiverAcknowledgementMetrics::register(pipeline_ctx),
            rejections: KafkaReceiverRejectionMetrics::register(pipeline_ctx),
            offset_commits: KafkaReceiverOffsetCommitMetrics::register(pipeline_ctx),
            consumer: pipeline_ctx.register_metrics::<KafkaReceiverConsumerMetrics>(),
            transport: KafkaReceiverTransportMetrics::register(pipeline_ctx),
        }
    }

    /// Records one Kafka consumer delivery before filtering or decoding.
    pub fn record_consumed_record(&mut self, payload_bytes: u64) {
        self.consumer.records_received.inc();
        if payload_bytes > 0 {
            self.consumer.record_bytes.add(payload_bytes);
        }
    }

    /// Records a decoded message after it is admitted to the pipeline send path.
    pub fn record_message_admitted(&mut self, signal: SignalType, payload_bytes: u64) {
        let messages = self.messages.with(SignalAttributes { signal });
        messages.started.inc();
        if payload_bytes > 0 {
            messages.bytes.add(payload_bytes);
        }
    }

    /// Records termination of receiver work for an admitted message.
    pub fn record_message_completed(&mut self, signal: SignalType) {
        self.messages
            .with(SignalAttributes { signal })
            .completed
            .inc();
    }

    /// Records an ack, nack, or invalid downstream acknowledgement response.
    pub fn record_acknowledgement(&mut self, signal: SignalType, outcome: Outcome) {
        self.acknowledgements
            .with(SignalOutcomeAttributes { signal, outcome })
            .responses
            .inc();
    }

    /// Records a message rejected before admission to the pipeline send path.
    pub fn record_rejection(
        &mut self,
        signal: Option<SignalType>,
        error_type: ReceiverRejectionErrorType,
        reason: KafkaReceiverRejectionReason,
    ) {
        self.rejections
            .with(KafkaReceiverRejectionAttributes {
                signal: signal.map_or(KafkaReceiverRejectionSignal::Unknown, Into::into),
                error_type,
                reason,
            })
            .messages
            .inc();
    }

    /// Records broker-reported offset commit outcomes.
    pub fn record_offset_commits(&mut self, outcome: Outcome, count: u64) {
        if count > 0 {
            self.offset_commits
                .with(OutcomeAttributes { outcome })
                .commits
                .add(count);
        }
    }

    /// Returns an offset commit bucket for inspection without marking it for export.
    #[must_use]
    pub fn offset_commits_for(&self, outcome: Outcome) -> &KafkaReceiverOffsetCommitMetrics {
        self.offset_commits.get(OutcomeAttributes { outcome })
    }

    /// Records a Kafka consumer error under an operator-actionable bounded category.
    pub fn record_transport_error(&mut self, error: &KafkaError) {
        self.transport
            .with(KafkaReceiverTransportAttributes {
                error_type: KafkaReceiverTransportErrorType::from_kafka_error(error),
            })
            .errors
            .inc();
    }

    /// Reports every touched Kafka receiver metric bucket.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.messages)?;
        reporter.report_measurement(&mut self.acknowledgements)?;
        reporter.report_measurement(&mut self.rejections)?;
        reporter.report_measurement(&mut self.offset_commits)?;
        reporter.report(&mut self.consumer)?;
        reporter.report_measurement(&mut self.transport)
    }

    /// Takes every touched Kafka receiver metric bucket for terminal handoff.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.messages.terminal_snapshots();
        snapshots.extend(self.acknowledgements.terminal_snapshots());
        snapshots.extend(self.rejections.terminal_snapshots());
        snapshots.extend(self.offset_commits.terminal_snapshots());
        if !self.consumer.is_empty() {
            snapshots.extend(self.consumer.terminal_snapshots());
        }
        snapshots.extend(self.transport.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics() -> KafkaReceiverMetrics {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        KafkaReceiverMetrics::register(&pipeline_ctx)
    }

    /// Scenario: Kafka consumer failures cover each operator-actionable transport category.
    /// Guarantees: Error codes map to bounded timeout, auth, topic, offset, poll, transport, and fallback values.
    #[test]
    fn transport_errors_are_classified_into_bounded_categories() {
        let cases = [
            (
                RDKafkaErrorCode::RequestTimedOut,
                KafkaReceiverTransportErrorType::Timeout,
            ),
            (
                RDKafkaErrorCode::Authentication,
                KafkaReceiverTransportErrorType::Authentication,
            ),
            (
                RDKafkaErrorCode::GroupAuthorizationFailed,
                KafkaReceiverTransportErrorType::Authorization,
            ),
            (
                RDKafkaErrorCode::UnknownTopicOrPartition,
                KafkaReceiverTransportErrorType::UnknownTopicOrPartition,
            ),
            (
                RDKafkaErrorCode::OffsetOutOfRange,
                KafkaReceiverTransportErrorType::Offset,
            ),
            (
                RDKafkaErrorCode::PollExceeded,
                KafkaReceiverTransportErrorType::PollExceeded,
            ),
            (
                RDKafkaErrorCode::AllBrokersDown,
                KafkaReceiverTransportErrorType::Transport,
            ),
            (
                RDKafkaErrorCode::InvalidArgument,
                KafkaReceiverTransportErrorType::Other,
            ),
        ];

        for (code, expected) in cases {
            let error = KafkaError::MessageConsumption(code);
            assert_eq!(
                KafkaReceiverTransportErrorType::from_kafka_error(&error),
                expected
            );
        }
    }

    /// Scenario: Kafka message lifecycle, acknowledgements, rejections, and commits span contexts.
    /// Guarantees: Counters remain isolated by their bounded signal, outcome, and error attributes.
    #[test]
    fn receiver_metrics_are_partitioned_by_context() {
        let mut metrics = new_test_metrics();
        metrics.record_consumed_record(42);
        metrics.record_message_admitted(SignalType::Logs, 42);
        metrics.record_message_completed(SignalType::Logs);
        metrics.record_acknowledgement(SignalType::Logs, Outcome::Refused);
        metrics.record_rejection(
            Some(SignalType::Logs),
            ReceiverRejectionErrorType::InvalidRequest,
            KafkaReceiverRejectionReason::Decode,
        );
        metrics.record_rejection(
            None,
            ReceiverRejectionErrorType::InvalidRequest,
            KafkaReceiverRejectionReason::UnknownTopic,
        );
        metrics.record_offset_commits(Outcome::Success, 2);
        metrics.record_offset_commits(Outcome::Failure, 1);

        let messages = metrics.messages.get(SignalAttributes {
            signal: SignalType::Logs,
        });
        assert_eq!(messages.started.get(), 1);
        assert_eq!(messages.completed.get(), 1);
        assert_eq!(messages.bytes.get(), 42);
        assert_eq!(metrics.consumer.records_received.get(), 1);
        assert_eq!(metrics.consumer.record_bytes.get(), 42);
        assert_eq!(
            metrics
                .messages
                .get(SignalAttributes {
                    signal: SignalType::Metrics,
                })
                .started
                .get(),
            0
        );
        assert_eq!(
            metrics
                .acknowledgements
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Refused,
                })
                .responses
                .get(),
            1
        );
        assert_eq!(
            metrics
                .rejections
                .get(KafkaReceiverRejectionAttributes {
                    signal: KafkaReceiverRejectionSignal::Logs,
                    error_type: ReceiverRejectionErrorType::InvalidRequest,
                    reason: KafkaReceiverRejectionReason::Decode,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .rejections
                .get(KafkaReceiverRejectionAttributes {
                    signal: KafkaReceiverRejectionSignal::Unknown,
                    error_type: ReceiverRejectionErrorType::InvalidRequest,
                    reason: KafkaReceiverRejectionReason::UnknownTopic,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .offset_commits
                .get(OutcomeAttributes {
                    outcome: Outcome::Success,
                })
                .commits
                .get(),
            2
        );
        assert_eq!(
            metrics
                .offset_commits
                .get(OutcomeAttributes {
                    outcome: Outcome::Failure,
                })
                .commits
                .get(),
            1
        );
    }

    /// Scenario: Kafka delivers a record that has not yet passed filtering or decoding.
    /// Guarantees: Transport ingress is counted without marking the record as pipeline-admitted.
    #[test]
    fn consumed_records_are_distinct_from_admitted_messages() {
        let mut metrics = new_test_metrics();

        metrics.record_consumed_record(128);

        assert_eq!(metrics.consumer.records_received.get(), 1);
        assert_eq!(metrics.consumer.record_bytes.get(), 128);
        assert_eq!(
            metrics
                .messages
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .started
                .get(),
            0
        );
    }

    /// Scenario: Kafka receiver metrics are transferred into terminal snapshots twice.
    /// Guarantees: Touched enum buckets carry wire values once; only the fixed consumer set may repeat.
    #[test]
    fn terminal_snapshots_preserve_enum_attribute_values_once() {
        let mut metrics = new_test_metrics();
        metrics.record_message_admitted(SignalType::Traces, 64);
        metrics.record_message_completed(SignalType::Traces);
        metrics.record_acknowledgement(SignalType::Traces, Outcome::Success);
        metrics.record_rejection(
            Some(SignalType::Traces),
            ReceiverRejectionErrorType::Internal,
            KafkaReceiverRejectionReason::TopicIdExhausted,
        );

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 4);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.messages"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.kafka.acknowledgements"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.kafka.rejections"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.measurement_attribute_value("error.type") == Some("internal")
                && snapshot.measurement_attribute_value("reason") == Some("topic_id_exhausted")
        }));
        let second = metrics.terminal_snapshots();
        assert!(
            second
                .iter()
                .all(|snapshot| snapshot.descriptor().name == "receiver.kafka.consumer")
        );
    }
}
