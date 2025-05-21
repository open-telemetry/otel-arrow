// SPDX-License-Identifier: Apache-2.0

//! Common types used throughout the OTLP implementation.

use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;

/// Enum to represent received OTLP requests.
#[derive(Debug, Clone)]
pub enum OTLPRequest {
    /// Logs Data
    Logs(ExportLogsServiceRequest),
    /// Metrics Data
    Metrics(ExportMetricsServiceRequest),
    /// Traces/Span Data
    Traces(ExportTraceServiceRequest),
}
