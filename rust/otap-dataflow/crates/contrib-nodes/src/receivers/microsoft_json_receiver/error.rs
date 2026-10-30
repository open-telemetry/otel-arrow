// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_config::data_limits::DataLimitError;

/// Errors exposed by the Microsoft JSON receiver framework.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// The receiver runtime will be added in a follow-up change.
    #[error("Microsoft JSON receiver is not implemented")]
    NotImplemented,
}

/// A failure never returns a partially mapped log.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum MappingError {
    #[error("invalid endpoint configuration: {0}")]
    Config(String),
    #[error("endpoint file error: {0}")]
    File(String),
    #[error("event rejected: {0}")]
    Rejected(String),
    #[error("event rejected by instrumentation key origin policy")]
    Unauthorized,
    #[error("request payload rejected by event override")]
    RejectedPayload,
    #[error("mapping limit exceeded: {0}")]
    Limit(&'static str),
}

impl From<DataLimitError> for MappingError {
    fn from(error: DataLimitError) -> Self {
        match error {
            DataLimitError::Config(message) => Self::Config(message),
            DataLimitError::File(message) => Self::File(message),
            DataLimitError::Limit(limit) => Self::Limit(limit),
        }
    }
}
