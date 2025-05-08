use crate::grpc_stubs::proto::collector::logs::v1::logs_service_server::LogsService;
use crate::grpc_stubs::proto::collector::logs::v1::{ExportLogsServiceRequest, ExportLogsServiceResponse};
use crate::grpc_stubs::proto::collector::metrics::v1::metrics_service_server::MetricsService;
use crate::grpc_stubs::proto::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use crate::grpc_stubs::proto::collector::trace::v1::trace_service_server::TraceService;
use crate::grpc_stubs::proto::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use tonic::{Request, Response, Status};
use otap_df_engine::receiver::{EffectHandler, SendableMode};


pub struct LogsServiceImpl {
    effect_handler: EffectHandler<OTLPRequest, SendableMode>,
}

impl LogsServiceImpl {
    pub fn new(effect_handler: EffectHandler<OTLPRequest, SendableMode>) -> Self {
        Self { effect_handler }
    }
}
pub struct MetricsServiceImpl {
    effect_handler: EffectHandler<OTLPRequest, SendableMode>,
}

impl MetricsServiceImpl {
    pub fn new(effect_handler: EffectHandler<OTLPRequest, SendableMode>) -> Self {
        Self { effect_handler }
    }
}
pub struct TraceServiceImpl {
    effect_handler: EffectHandler<OTLPRequest, SendableMode>,
}

impl TraceServiceImpl {
    pub fn new(effect_handler: EffectHandler<OTLPRequest, SendableMode>) -> Self {
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
        effect_handler_clone.send_message(OTLPRequest::Logs(request.into_inner())).await;

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
        effect_handler_clone.send_message(OTLPRequest::Metrics(request.into_inner())).await;
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
        effect_handler_clone.send_message(OTLPRequest::Traces(request.into_inner())).await;
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}


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
