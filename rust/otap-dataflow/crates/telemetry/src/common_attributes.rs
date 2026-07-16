// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable closed-set attributes for internal telemetry metrics.

use otap_df_telemetry_macros::AttributeEnum;

/// Signal carried by a telemetry datapoint.
///
/// Components that support multiple OpenTelemetry signals should use this
/// closed-set attribute instead of defining a component-local equivalent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum MetricSignal {
    /// Log records.
    Logs,
    /// Metric data points.
    Metrics,
    /// Spans.
    Traces,
}

/// Outcome of a pipeline component operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum Outcome {
    /// The component completed the operation successfully.
    Success,
    /// The component itself failed the operation.
    Failure,
    /// A downstream component refused the operation.
    Refused,
}

/// Result category for an HTTP request attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum HttpResponse {
    /// A successful 2xx response.
    #[attribute_value = "http_2xx"]
    Http2xx,
    /// An unauthorized response.
    #[attribute_value = "http_401"]
    Http401,
    /// A forbidden response.
    #[attribute_value = "http_403"]
    Http403,
    /// A payload-too-large response.
    #[attribute_value = "http_413"]
    Http413,
    /// A rate-limited response.
    #[attribute_value = "http_429"]
    Http429,
    /// A server-error response.
    #[attribute_value = "http_5xx"]
    Http5xx,
    /// A transport failure before a response was received.
    NetworkError,
}
