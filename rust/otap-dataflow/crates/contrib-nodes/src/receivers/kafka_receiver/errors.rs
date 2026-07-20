// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Kafka Receiver that add additional observability.

use otap_df_engine::error::Error as EngineError;

/// Decode error with signal context for per-signal metrics.
///
/// Wraps an [`EngineError`] with information about which signal type (or
/// pre-signal stage) failed, so the caller in `run_receive_loop` can
/// increment the correct per-signal unmarshal counter and emit a descriptive
/// error log.
pub(crate) enum DecodeError {
    /// Empty payload (no signal context yet).
    EmptyPayload(EngineError),
    /// Topic didn't match any configured signal.
    UnknownTopic(EngineError),
    /// Traces decode/unmarshal failed.
    Traces(EngineError),
    /// Metrics decode/unmarshal failed.
    Metrics(EngineError),
    /// Logs decode/unmarshal failed.
    Logs(EngineError),
}

impl DecodeError {
    /// Unwrap the inner [`EngineError`] for inspection.
    #[cfg(test)]
    pub(crate) fn inner(&self) -> &EngineError {
        match self {
            Self::EmptyPayload(e)
            | Self::UnknownTopic(e)
            | Self::Traces(e)
            | Self::Metrics(e)
            | Self::Logs(e) => e,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_error_inner_empty_payload() {
        let err = DecodeError::EmptyPayload(EngineError::PdataConversionError {
            error: "empty".to_string(),
        });
        let inner = err.inner();
        assert!(matches!(inner, EngineError::PdataConversionError { .. }));
    }

    #[test]
    fn decode_error_inner_unknown_topic() {
        let err = DecodeError::UnknownTopic(EngineError::PdataConversionError {
            error: "unknown".to_string(),
        });
        let inner = err.inner();
        assert!(matches!(inner, EngineError::PdataConversionError { .. }));
    }

    #[test]
    fn decode_error_inner_traces() {
        let err = DecodeError::Traces(EngineError::PdataConversionError {
            error: "traces".to_string(),
        });
        let inner = err.inner();
        assert!(matches!(inner, EngineError::PdataConversionError { .. }));
    }

    #[test]
    fn decode_error_inner_metrics() {
        let err = DecodeError::Metrics(EngineError::PdataConversionError {
            error: "metrics".to_string(),
        });
        let inner = err.inner();
        assert!(matches!(inner, EngineError::PdataConversionError { .. }));
    }

    #[test]
    fn decode_error_inner_logs() {
        let err = DecodeError::Logs(EngineError::PdataConversionError {
            error: "logs".to_string(),
        });
        let inner = err.inner();
        assert!(matches!(inner, EngineError::PdataConversionError { .. }));
    }
}
