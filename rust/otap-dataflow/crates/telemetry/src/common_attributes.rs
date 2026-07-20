// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable closed-set attributes for internal telemetry metrics.

use crate::attributes::AttributeEnum;
use otap_df_config::SignalType;
use otap_df_telemetry_macros::AttributeEnum;

// SignalType belongs to the configuration model because it describes pipeline
// data independently of telemetry. Implement the telemetry-specific trait here
// to reuse that canonical type without making config depend on telemetry.
// This avoids boilerplate conversion between config::SignalType and a separate
// telemetry::MetricSignalType.
//
// Other closed-set attributes use the `AttributeEnum` macro to implement the trait automatically.
impl AttributeEnum for SignalType {
    const CARDINALITY: usize = 3;
    const VARIANTS: &'static [&'static str] = &["traces", "metrics", "logs"];

    fn variant_index(self) -> usize {
        match self {
            Self::Traces => 0,
            Self::Metrics => 1,
            Self::Logs => 2,
        }
    }
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
    /// A bad-request response.
    #[attribute_value = "http_400"]
    Http400,
    /// An unauthorized response.
    #[attribute_value = "http_401"]
    Http401,
    /// A forbidden response.
    #[attribute_value = "http_403"]
    Http403,
    /// A not-found response.
    #[attribute_value = "http_404"]
    Http404,
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
    /// A response that is not otherwise classified.
    Other,
}
