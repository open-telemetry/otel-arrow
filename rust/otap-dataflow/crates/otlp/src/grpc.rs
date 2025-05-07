use crate::grpc_stubs::proto::collector::logs::v1::ExportLogsServiceRequest;
use crate::grpc_stubs::proto::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::grpc_stubs::proto::collector::trace::v1::ExportTraceServiceRequest;

/// Expose the OTLP gRPC services.

// Enum to represent received OTLP requests.
#[derive(Debug, Clone)]
pub enum OTLPRequest {
    /// Logs Data
    Logs(ExportLogsServiceRequest),
    /// Metrics Data
    Metrics(ExportMetricsServiceRequest),
    /// Traces/Span Data
    Traces(ExportTraceServiceRequest),
}
