use crate::grpc_stubs::proto::collector::{
logs::v1::{logs_service_server::LogsService, ExportLogsServiceRequest, ExportLogsServiceResponse}, 
metrics::v1::{metrics_service_server::MetricsService, ExportMetricsServiceRequest, ExportMetricsServiceResponse}, 
trace::v1::{trace_service_server::TraceService, ExportTraceServiceRequest, ExportTraceServiceResponse}};

use otap_df_engine::receiver::{EffectHandlerTrait, SendEffectHandler};
use tonic::{Request, Response, Status};


/// struct that implements the Log Service trait
pub struct LogsServiceImpl {
    effect_handler: SendEffectHandler<OTLPRequest>,
}

impl LogsServiceImpl {
    /// Create a LogsServiceImpl with a sendable Effect Handler
    pub fn new(effect_handler: SendEffectHandler<OTLPRequest>) -> Self {
        Self { effect_handler }
    }
}

/// struct that implements the Metric Service trait
pub struct MetricsServiceImpl {
    effect_handler: SendEffectHandler<OTLPRequest>,
}

impl MetricsServiceImpl {
    /// Create a MetricsServiceImpl with a sendable Effect Handler
    pub fn new(effect_handler: SendEffectHandler<OTLPRequest>) -> Self {
        Self { effect_handler }
    }
}

/// struct that implements the Trace Service trait
pub struct TraceServiceImpl {
    effect_handler: SendEffectHandler<OTLPRequest>,
}

impl TraceServiceImpl {
    /// Create a TraceServiceImpl with a sendable Effect Handler
    pub fn new(effect_handler: SendEffectHandler<OTLPRequest>) -> Self {
        Self { effect_handler }
    }
}

#[tonic::async_trait]
impl LogsService for LogsServiceImpl {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let effect_handler_clone = self.effect_handler.clone();
        _ = tokio::task::spawn_local( async move {
            _ = effect_handler_clone.send_message(OTLPRequest::Logs(request.into_inner())).await;
        });
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceImpl {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let effect_handler_clone = self.effect_handler.clone();
        _ = tokio::task::spawn_local( async move {
            _ = effect_handler_clone.send_message(OTLPRequest::Metrics(request.into_inner())).await;
        });
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl TraceService for TraceServiceImpl {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let effect_handler_clone = self.effect_handler.clone();
        _ = tokio::task::spawn_local( async move {
            _ = effect_handler_clone.send_message(OTLPRequest::Traces(request.into_inner())).await;
        });
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

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


#[cfg(test)]
mod tests {
    use crate::grpc::OTLPRequest;
    use otap_df_engine::error::Error;
    use otap_df_channel::error::SendError;

    fn assert_send<T: Send>() {}
    #[test]
    fn test() {
        _ = assert_send::<Error<SendError<OTLPRequest>>>();
    }

}