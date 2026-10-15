// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Errors exposed by the Azure Data Explorer exporter.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// A caller supplied an unsafe transformation limit.
    #[error("invalid transformation limit `{name}`: expected 1..={maximum}, got {actual}")]
    InvalidTransformationLimit {
        /// Name of the invalid limit.
        name: &'static str,
        /// Supplied limit.
        actual: usize,
        /// Largest supported limit.
        maximum: usize,
    },

    /// A serialized row exceeded the configured safety limit.
    #[error("serialized ADX row is {actual} bytes, exceeding the {limit}-byte limit")]
    RowTooLarge {
        /// Serialized row size including the JSON Lines separator.
        actual: usize,
        /// Configured row-size limit.
        limit: usize,
    },

    /// A transformed request exceeded the configured byte limit.
    #[error("serialized ADX request is {actual} bytes, exceeding the {limit}-byte limit")]
    RequestTooLarge {
        /// Serialized JSON Lines request size.
        actual: usize,
        /// Configured request-size limit.
        limit: usize,
    },

    /// A transformed request exceeded the configured row limit.
    #[error("serialized ADX request has {actual} rows, exceeding the {limit}-row limit")]
    TooManyRows {
        /// Number of transformed rows.
        actual: usize,
        /// Configured row-count limit.
        limit: usize,
    },

    /// Recursive OTLP values exceeded the supported nesting depth.
    #[error(
        "OTLP value nesting depth is {actual}, exceeding the {limit}-level transformation limit"
    )]
    TransformationDepthLimitExceeded {
        /// Observed nesting depth.
        actual: usize,
        /// Maximum supported nesting depth.
        limit: usize,
    },

    /// OTLP value traversal exceeded the per-message work limit.
    #[error(
        "OTLP value traversal visited {actual} values, exceeding the {limit}-value transformation limit"
    )]
    TransformationWorkLimitExceeded {
        /// Number of visited values.
        actual: usize,
        /// Maximum values visited per source message.
        limit: usize,
    },

    /// Metric data cannot be represented without silent data loss.
    #[error("invalid metric data: {reason}")]
    InvalidMetricData {
        /// Stable reason for refusing the source message.
        reason: &'static str,
    },

    /// The exporter runtime will be added in a follow-up change.
    #[error("Azure Data Explorer exporter is not implemented")]
    NotImplemented,
}
