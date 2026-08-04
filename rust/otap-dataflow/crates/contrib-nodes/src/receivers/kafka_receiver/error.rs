// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Kafka Receiver.
//!
//! [`KafkaReceiverError`] is the single error type for the Kafka receiver
//! module. It consolidates both the per-message decode errors (previously
//! `DecodeError`) and the configuration validation errors (previously
//! `KafkaReceiverConfigError`).
//!
//! Decode variants carry signal context so the receive loop can increment the
//! correct per-signal unmarshal counter and emit a descriptive error log. They
//! wrap an [`EngineError`] as their source. Configuration variants are
//! structured with named fields so callers can inspect exactly which rule was
//! violated instead of matching on a free-form string.

use otap_df_engine::error::Error as EngineError;

/// Errors produced by the Kafka receiver.
#[derive(Debug, thiserror::Error)]
pub enum KafkaReceiverError {
    // ==================== Decode Errors ====================
    /// Empty payload (no signal context yet).
    #[error("empty kafka payload: {0}")]
    EmptyPayloadDecode(#[source] EngineError),

    /// Topic didn't match any configured signal.
    #[error("unknown kafka topic: {0}")]
    UnknownTopicDecode(#[source] EngineError),

    /// Traces decode/unmarshal failed.
    #[error("traces decode failed: {0}")]
    TracesDecode(#[source] EngineError),

    /// Metrics decode/unmarshal failed.
    #[error("metrics decode failed: {0}")]
    MetricsDecode(#[source] EngineError),

    /// Logs decode/unmarshal failed.
    #[error("logs decode failed: {0}")]
    LogsDecode(#[source] EngineError),

    // ==================== Configuration Errors ====================
    /// A required string field was left empty.
    #[error("invalid kafka receiver configuration: {field} can't be empty")]
    ConfigEmptyField {
        /// The name of the empty field (e.g. `brokers`, `client_id`).
        field: String,
    },

    /// `resource_attrs_from_headers` contained an empty header key.
    #[error(
        "invalid kafka receiver configuration: resource_attrs_from_headers contains an empty header key"
    )]
    ConfigEmptyHeaderKey,

    /// The extraction `key` for a header in `resource_attrs_from_headers` was empty.
    #[error(
        "invalid kafka receiver configuration: resource_attrs_from_headers['{header_key}'].key can't be empty"
    )]
    ConfigEmptyExtractionKey {
        /// The header key whose extraction `key` was empty.
        header_key: String,
    },

    /// No signal (traces, metrics, or logs) had any topics configured.
    #[error(
        "invalid kafka receiver configuration: at least one signal (traces, metrics, or logs) must have non-empty topics"
    )]
    ConfigNoSignalTopics,

    /// The same topic appeared under more than one signal.
    #[error("invalid kafka receiver configuration: kafka topics overlap across signals")]
    ConfigOverlappingTopics,

    /// A literal topic name failed Kafka topic-name validation.
    #[error("invalid kafka receiver configuration: {signal}.topics: {message}")]
    ConfigInvalidTopicName {
        /// The signal the topic belongs to (`traces`, `metrics`, or `logs`).
        signal: String,
        /// The underlying validation message.
        message: String,
    },

    /// A regex topic pattern (starting with `^`) failed to compile.
    #[error(
        "invalid kafka receiver configuration: invalid regex topic pattern in {signal}: '{topic}': {message}"
    )]
    ConfigInvalidTopicRegex {
        /// The signal the pattern belongs to.
        signal: String,
        /// The offending pattern.
        topic: String,
        /// The regex-compilation error message.
        message: String,
    },

    /// `exclude_topics` was set for a signal that has no regex topic pattern.
    #[error(
        "invalid kafka receiver configuration: {signal}.exclude_topics is only allowed when at least one topic is a regex pattern"
    )]
    ConfigExcludeTopicsRequiresRegex {
        /// The signal the constraint applies to.
        signal: String,
    },

    /// An `exclude_topics` entry was empty.
    #[error(
        "invalid kafka receiver configuration: {signal}.exclude_topics entries must be non-empty"
    )]
    ConfigEmptyExcludeTopic {
        /// The signal the empty entry belongs to.
        signal: String,
    },

    /// An `exclude_topics` regex pattern failed to compile.
    #[error(
        "invalid kafka receiver configuration: invalid regex in {signal}.exclude_topics: '{pattern}': {message}"
    )]
    ConfigInvalidExcludeRegex {
        /// The signal the pattern belongs to.
        signal: String,
        /// The offending pattern.
        pattern: String,
        /// The regex-compilation error message.
        message: String,
    },

    /// The `auth` sub-configuration failed validation.
    #[error("invalid kafka receiver configuration: auth: {message}")]
    ConfigInvalidAuth {
        /// The underlying auth validation message.
        message: String,
    },

    /// The `tls` sub-configuration failed validation.
    #[error("invalid kafka receiver configuration: tls: {message}")]
    ConfigInvalidTls {
        /// The underlying tls validation message.
        message: String,
    },

    /// `max_fetch_bytes` was smaller than `min_fetch_bytes`.
    #[error(
        "invalid kafka receiver configuration: max_fetch_bytes ({max}) must be >= min_fetch_bytes ({min})"
    )]
    ConfigInvalidFetchBytes {
        /// The configured `max_fetch_bytes`.
        max: i32,
        /// The configured `min_fetch_bytes`.
        min: i32,
    },

    /// A field that must be strictly positive was zero (or negative).
    #[error("invalid kafka receiver configuration: {field} must be > 0")]
    ConfigNonPositiveValue {
        /// The name of the offending field.
        field: String,
    },
}

impl KafkaReceiverError {
    /// Unwrap the inner [`EngineError`] for decode variants.
    ///
    /// Returns `None` for configuration variants, which carry no
    /// [`EngineError`] source.
    #[cfg(test)]
    pub(crate) fn inner(&self) -> Option<&EngineError> {
        match self {
            Self::EmptyPayloadDecode(e)
            | Self::UnknownTopicDecode(e)
            | Self::TracesDecode(e)
            | Self::MetricsDecode(e)
            | Self::LogsDecode(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    // ==================== Decode Error Tests ====================

    /// Scenario: each decode variant is constructed with an inner EngineError.
    /// Guarantees: inner() returns the wrapped EngineError for decode variants.
    #[test]
    fn decode_error_inner_returns_engine_error() {
        let cases = [
            KafkaReceiverError::EmptyPayloadDecode(EngineError::PdataConversionError {
                error: "empty".to_string(),
            }),
            KafkaReceiverError::UnknownTopicDecode(EngineError::PdataConversionError {
                error: "unknown".to_string(),
            }),
            KafkaReceiverError::TracesDecode(EngineError::PdataConversionError {
                error: "traces".to_string(),
            }),
            KafkaReceiverError::MetricsDecode(EngineError::PdataConversionError {
                error: "metrics".to_string(),
            }),
            KafkaReceiverError::LogsDecode(EngineError::PdataConversionError {
                error: "logs".to_string(),
            }),
        ];
        for err in &cases {
            let inner = err.inner().expect("decode variant has inner EngineError");
            assert!(matches!(inner, EngineError::PdataConversionError { .. }));
            assert!(err.source().is_some());
        }
    }

    // ==================== Configuration Error Tests ====================

    /// Scenario: config variants carry no EngineError source.
    /// Guarantees: inner() returns None for configuration variants.
    #[test]
    fn config_error_has_no_inner_engine_error() {
        let err = KafkaReceiverError::ConfigEmptyField {
            field: "brokers".to_string(),
        };
        assert!(err.inner().is_none());
    }

    /// Scenario: an empty required field is reported.
    /// Guarantees: the Display string names the field and keeps the invalid prefix.
    #[test]
    fn config_empty_field_message() {
        let err = KafkaReceiverError::ConfigEmptyField {
            field: "brokers".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid kafka receiver configuration: brokers can't be empty"
        );
    }

    /// Scenario: topics overlap across signals.
    /// Guarantees: the Display string mentions overlap for operator diagnosis.
    #[test]
    fn config_overlapping_topics_message() {
        let err = KafkaReceiverError::ConfigOverlappingTopics;
        assert!(err.to_string().contains("overlap"));
    }

    /// Scenario: no signal has topics configured.
    /// Guarantees: the Display string mentions the at-least-one-signal rule.
    #[test]
    fn config_no_signal_topics_message() {
        let err = KafkaReceiverError::ConfigNoSignalTopics;
        assert!(err.to_string().contains("at least one signal"));
    }

    /// Scenario: max_fetch_bytes is smaller than min_fetch_bytes.
    /// Guarantees: the Display string reports both configured values.
    #[test]
    fn config_invalid_fetch_bytes_message() {
        let err = KafkaReceiverError::ConfigInvalidFetchBytes { max: 50, min: 100 };
        assert_eq!(
            err.to_string(),
            "invalid kafka receiver configuration: max_fetch_bytes (50) must be >= min_fetch_bytes (100)"
        );
    }

    /// Scenario: a positive-only field is set to zero.
    /// Guarantees: the Display string names the field and the > 0 rule.
    #[test]
    fn config_non_positive_value_message() {
        let err = KafkaReceiverError::ConfigNonPositiveValue {
            field: "max_partition_fetch_bytes".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid kafka receiver configuration: max_partition_fetch_bytes must be > 0"
        );
    }

    // ==================== Trait Tests ====================

    /// Scenario: KafkaReceiverError is used across Send and !Send contexts.
    /// Guarantees: the error type is Send + Sync.
    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<KafkaReceiverError>();
    }

    /// Scenario: the error is treated as a std error for source chaining.
    /// Guarantees: KafkaReceiverError implements std::error::Error.
    #[test]
    fn error_implements_std_error() {
        fn assert_std_error<T: StdError>() {}
        assert_std_error::<KafkaReceiverError>();
    }
}
