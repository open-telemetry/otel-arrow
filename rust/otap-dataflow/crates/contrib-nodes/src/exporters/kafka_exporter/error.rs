// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Kafka exporter.
//!
//! [`KafkaExporterError`] is the single error type for the Kafka exporter
//! module. It consolidates what were previously separate enums:
//!
//! - configuration and runtime errors (formerly the exporter's own
//!   `KafkaExporterError`),
//! - payload-encoding errors (formerly `encoder::EncodingError`), and
//! - dynamic-topic-routing errors (formerly `topic_router::TopicRoutingError`).
//!
//! Consolidating keeps a single `Result<_, KafkaExporterError>` across the
//! encode / route / send pipeline and lets the export loop classify each
//! failure (transient vs. permanent nack) by matching one enum.

use crate::common::kafka::sanitize_for_log;
use otap_df_config::SignalType;
use rdkafka::error::KafkaError;
use rdkafka::types::RDKafkaErrorCode;

/// Classifies a producer send failure as permanent (non-retryable) or transient
/// (retryable).
///
/// A Kafka send can fail either at enqueue time (librdkafka rejects the record
/// locally, e.g. the payload exceeds `message.max.bytes`) or at delivery time
/// (the broker's delivery callback resolves with an error). Both surface as a
/// [`KafkaError`] carrying an [`RDKafkaErrorCode`]. This function returns `true`
/// only for a conservative allowlist of codes that can never succeed on retry,
/// so the exporter can permanently nack such a batch and let an upstream
/// `processor:retry` drop it at the source instead of retrying an error that
/// will always fail.
///
/// The classification is deliberately conservative: any code not in the
/// permanent set -- and any error without an [`RDKafkaErrorCode`] (e.g. a
/// canceled delivery) -- is treated as transient so that a retryable or
/// unrecognized failure is retried rather than silently dropped.
#[must_use]
pub(crate) fn is_permanent_send_error(err: &KafkaError) -> bool {
    let Some(code) = err.rdkafka_error_code() else {
        // No underlying code (e.g. Canceled): prefer retry over drop.
        return false;
    };
    matches!(
        code,
        // The record itself is malformed or too large: it will never be
        // accepted no matter how many times it is retried.
        RDKafkaErrorCode::MessageSizeTooLarge
            | RDKafkaErrorCode::InvalidMessageSize
            | RDKafkaErrorCode::MessageBatchTooLarge
            | RDKafkaErrorCode::InvalidMessage
            | RDKafkaErrorCode::InvalidRecord
            // Fundamentally unsupported request shape for this broker/topic:
            // the same request will keep being rejected.
            | RDKafkaErrorCode::InvalidRequiredAcks
            | RDKafkaErrorCode::UnsupportedVersion
            | RDKafkaErrorCode::UnsupportedForMessageFormat
            // A locally-invalid argument (bad topic/partition/config for this
            // record) will not become valid on retry.
            | RDKafkaErrorCode::InvalidArgument
            // Serialization of the key/value failed: deterministic for the
            // same record, so retrying is pointless.
            | RDKafkaErrorCode::KeySerialization
            | RDKafkaErrorCode::ValueSerialization
    )
    // NOTE: transient/retryable codes (timeouts, broker/leader unavailable,
    // network failures, queue-full, coordinator churn, and any unlisted or
    // future code under `#[non_exhaustive]`) intentionally fall through to
    // `false` so they are retried rather than dropped.
}

/// Errors produced by the Kafka exporter.
#[derive(Debug, thiserror::Error)]
pub enum KafkaExporterError {
    // ==================== Configuration / runtime ====================
    /// Configuration error (invalid config or producer construction failure).
    #[error("Kafka exporter configuration error: {0}")]
    Configuration(String),

    /// A dynamic-routing allowlist regex pattern failed to compile.
    #[error(
        "invalid kafka exporter configuration: invalid regex in {signal:?}.allowed_topics_regex: '{pattern}': {message}"
    )]
    ConfigInvalidTopicRegex {
        /// The signal the pattern belongs to.
        signal: SignalType,
        /// The offending pattern.
        pattern: String,
        /// The regex-compilation error message.
        message: String,
    },

    /// Missing topic configuration for a signal type.
    #[error("No topic configured for signal type: {0:?}")]
    MissingTopic(SignalType),

    /// Kafka client error.
    #[error("Kafka client error: {0}")]
    KafkaError(#[from] KafkaError),

    /// The delivery-result notification was canceled before it resolved: the
    /// producer was dropped after the record was enqueued but before its
    /// delivery callback fired. Treated as a transient send failure so the
    /// batch can be retried rather than crashing the exporter.
    #[error("Kafka delivery notification canceled before completion")]
    DeliveryCanceled,

    // ==================== Encoding ====================
    /// Failed to convert payload to OTLP bytes.
    #[error("Failed to convert payload to OTLP bytes: {0}")]
    OtlpConversion(String),

    /// Failed to convert payload to OtapArrowRecords.
    #[error("Failed to convert payload to OtapArrowRecords: {0}")]
    OtapArrowRecordsConversion(String),

    /// Failed to convert OtapArrowRecords to BatchArrowRecord bytes.
    #[error("Failed to convert OtapArrowRecords to BatchArrowRecord bytes: {0}")]
    BatchArrowRecordConversion(String),

    // ==================== Dynamic topic routing ====================
    /// A topic was supplied via a transport header but it failed Kafka topic
    /// validation. Non-retryable: the same header will always be invalid, so
    /// the exporter permanently nacks the batch rather than silently rerouting
    /// it to the static topic.
    #[error("invalid Kafka topic '{topic}' from transport header: {reason}")]
    InvalidHeaderTopic {
        /// The offending topic value (already sanitized/bounded for safe
        /// rendering).
        topic: String,
        /// Human-readable reason the topic failed validation
        reason: String,
    },

    /// A topic was supplied via a transport header and is a syntactically valid
    /// Kafka topic, but it is not permitted by the signal's operator-configured
    /// dynamic-routing allowlist (exact or regex). Non-retryable: the batch is
    /// permanently nacked rather than routed to a disallowed destination or the
    /// static topic.
    #[error("Kafka topic '{topic}' from transport header is not allowed by the routing policy")]
    DisallowedHeaderTopic {
        /// The disallowed topic value (already sanitized/bounded for safe
        /// rendering).
        topic: String,
    },
}

impl KafkaExporterError {
    /// Builds a [`KafkaExporterError::InvalidHeaderTopic`] and emits the routing
    /// warning once, so all "header present but unusable as a topic" cases
    /// (non-UTF-8 value or failed Kafka topic validation) share a single
    /// construction and log site. Both the topic value and the reason are
    /// sanitized/bounded before they are logged or stored.
    pub(crate) fn invalid_header_topic(topic: impl AsRef<str>, reason: impl Into<String>) -> Self {
        let topic = sanitize_for_log(topic.as_ref());
        let reason = sanitize_for_log(&reason.into());
        otel_warn!(
            "kafka.exporter.topic.invalid_header",
            header_topic = %topic,
            %reason,
            "invalid Kafka topic from transport header, permanently nacking batch"
        );
        Self::InvalidHeaderTopic { topic, reason }
    }

    /// Builds a [`KafkaExporterError::DisallowedHeaderTopic`] and emits the
    /// routing warning once. The topic value is sanitized/bounded before it is
    /// logged or stored, since it is client-controlled.
    pub(crate) fn disallowed_header_topic(topic: impl AsRef<str>) -> Self {
        let topic = sanitize_for_log(topic.as_ref());
        otel_warn!(
            "kafka.exporter.topic.disallowed_header",
            header_topic = %topic,
            "Kafka topic from transport header is not permitted by the routing policy, \
             permanently nacking batch"
        );
        Self::DisallowedHeaderTopic { topic }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    /// Scenario: a configuration error is formatted.
    /// Guarantees: the Display string names the failure and the config prefix.
    #[test]
    fn config_error_message() {
        let err = KafkaExporterError::Configuration("bad config".to_string());
        let s = err.to_string();
        assert!(s.contains("bad config"), "got: {s}");
        assert!(s.contains("configuration error"), "got: {s}");
    }

    /// Scenario: a missing-topic error names the signal type.
    /// Guarantees: the Display string mentions the offending signal type.
    #[test]
    fn missing_topic_message() {
        let err = KafkaExporterError::MissingTopic(SignalType::Logs);
        assert!(err.to_string().contains("Logs"), "got: {}", err);
    }

    /// Scenario: the delivery-canceled variant is formatted.
    /// Guarantees: the Display string mentions cancellation so the transient
    /// nack it drives carries a meaningful reason.
    #[test]
    fn delivery_canceled_message() {
        let err = KafkaExporterError::DeliveryCanceled;
        assert!(err.to_string().contains("canceled"), "got: {}", err);
    }

    /// Scenario: an invalid dynamic-routing regex is reported.
    /// Guarantees: the Display string names the signal (rendered from the
    /// `SignalType`), the pattern, and the allowed_topics_regex field so an
    /// operator can fix the config.
    #[test]
    fn invalid_topic_regex_message() {
        let err = KafkaExporterError::ConfigInvalidTopicRegex {
            signal: SignalType::Logs,
            pattern: "[".to_string(),
            message: "unclosed character class".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("Logs.allowed_topics_regex"), "got: {s}");
        assert!(s.contains('['), "got: {s}");
    }

    /// Scenario: the invalid-header-topic constructor sanitizes the value.
    /// Guarantees: a control character in the client-supplied topic is escaped
    /// in the stored/logged topic value (no raw newline survives).
    #[test]
    fn invalid_header_topic_sanitizes_value() {
        let err = KafkaExporterError::invalid_header_topic("evil\ntopic", "bad");
        match err {
            KafkaExporterError::InvalidHeaderTopic { topic, .. } => {
                assert!(
                    !topic.contains('\n'),
                    "raw newline must be escaped: {topic:?}"
                );
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    /// Scenario: the invalid-header-topic constructor sanitizes the reason,
    /// using a reason produced by the real `validate_kafka_topic` for a topic
    /// containing a raw newline (which embeds that client-supplied character
    /// into the reason string).
    /// Guarantees: no raw control character from the client-supplied topic
    /// survives in the stored/logged reason field (log-injection defense also
    /// covers the reason, not just the topic value).
    #[test]
    fn invalid_header_topic_sanitizes_reason() {
        use crate::common::kafka::validate_kafka_topic;
        // The offending topic carries a control char that validation reflects
        // verbatim into its error string.
        let bad_topic = "evil\ntopic";
        let reason = validate_kafka_topic(bad_topic)
            .expect_err("a topic with a newline must fail validation");
        assert!(
            reason.contains('\n'),
            "precondition: the raw validation reason must embed the newline: {reason:?}"
        );

        let err = KafkaExporterError::invalid_header_topic(bad_topic, reason);
        match err {
            KafkaExporterError::InvalidHeaderTopic { reason, .. } => {
                assert!(
                    !reason.contains('\n'),
                    "raw newline in reason must be escaped: {reason:?}"
                );
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    /// Scenario: the disallowed-header-topic constructor sanitizes the value.
    /// Guarantees: a control character in the client-supplied topic is escaped
    /// in the stored/logged topic value.
    #[test]
    fn disallowed_header_topic_sanitizes_value() {
        let err = KafkaExporterError::disallowed_header_topic("evil\ttopic");
        match err {
            KafkaExporterError::DisallowedHeaderTopic { topic } => {
                assert!(!topic.contains('\t'), "raw tab must be escaped: {topic:?}");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    /// Scenario: the error is shared across Send/Sync contexts and used as a
    /// std error.
    /// Guarantees: KafkaExporterError is Send + Sync and implements
    /// std::error::Error.
    #[test]
    fn error_is_send_sync_std_error() {
        fn assert_bounds<T: Send + Sync + StdError>() {}
        assert_bounds::<KafkaExporterError>();
    }

    // ---- Send-error classification ----

    /// Scenario: a produce failure carries a Kafka error code that can never
    /// succeed on retry (an oversized record, an authorization failure, or a
    /// deterministic malformed/serialization error).
    /// Guarantees: `is_permanent_send_error` classifies each such code as
    /// permanent, so the exporter permanently nacks it (dropped at the source)
    /// instead of retrying an error that will always fail.
    #[test]
    fn permanent_send_error_codes_are_permanent() {
        for code in [
            RDKafkaErrorCode::MessageSizeTooLarge,
            RDKafkaErrorCode::InvalidMessageSize,
            RDKafkaErrorCode::MessageBatchTooLarge,
            RDKafkaErrorCode::InvalidMessage,
            RDKafkaErrorCode::InvalidRecord,
            RDKafkaErrorCode::InvalidRequiredAcks,
            RDKafkaErrorCode::UnsupportedVersion,
            RDKafkaErrorCode::UnsupportedForMessageFormat,
            RDKafkaErrorCode::InvalidArgument,
            RDKafkaErrorCode::KeySerialization,
            RDKafkaErrorCode::ValueSerialization,
        ] {
            assert!(
                is_permanent_send_error(&KafkaError::MessageProduction(code)),
                "{code:?} should be classified as a permanent send error"
            );
        }
    }

    /// Scenario: a produce failure carries a Kafka error code that may resolve
    /// on retry (a timeout, a broker/leader that is temporarily unavailable, a
    /// network failure, or a full local queue).
    /// Guarantees: `is_permanent_send_error` classifies each such code as
    /// transient, so the exporter emits a retryable nack and an upstream
    /// `processor:retry` can retry the batch.
    #[test]
    fn transient_send_error_codes_are_transient() {
        for code in [
            RDKafkaErrorCode::RequestTimedOut,
            RDKafkaErrorCode::MessageTimedOut,
            RDKafkaErrorCode::BrokerNotAvailable,
            RDKafkaErrorCode::LeaderNotAvailable,
            RDKafkaErrorCode::NotLeaderForPartition,
            RDKafkaErrorCode::NotEnoughReplicas,
            RDKafkaErrorCode::NetworkException,
            RDKafkaErrorCode::BrokerTransportFailure,
            RDKafkaErrorCode::AllBrokersDown,
            RDKafkaErrorCode::QueueFull,
        ] {
            assert!(
                !is_permanent_send_error(&KafkaError::MessageProduction(code)),
                "{code:?} should be classified as a transient (retryable) send error"
            );
        }
    }

    /// Scenario: a send failure has no underlying `RDKafkaErrorCode` (e.g. the
    /// delivery notification was canceled).
    /// Guarantees: `is_permanent_send_error` defaults such an error to transient,
    /// so an unclassified failure is retried rather than silently dropped.
    #[test]
    fn send_error_without_code_defaults_to_transient() {
        assert!(
            !is_permanent_send_error(&KafkaError::Canceled),
            "an error without an rdkafka code must default to transient (retryable)"
        );
    }
}
