use grpc_stubs::proto::collector::logs::v1::logs_service_server::LogsService;
use grpc_stubs::proto::collector::logs::v1::{ExportLogsServiceRequest, ExportLogsServiceResponse};
use grpc_stubs::proto::collector::metrics::v1::metrics_service_server::MetricsService;
use grpc_stubs::proto::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use grpc_stubs::proto::collector::trace::v1::trace_service_server::TraceService;
use grpc_stubs::proto::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use tonic::{Request, Response, Status};
use otap_df_engine::receiver::{EffectHandler, SendableMode};
use tokio::sync::mpsc::Sender;

pub struct LogsServiceMock {
    sender: Sender<OTLPRequest>
}

impl LogsServiceMock {
    pub fn new(sender: Sender<OTLPRequest>) -> Self {
        Self { sender }
    }
}


pub struct MetricsServiceMock {
    sender: Sender<OTLPRequest>
}

impl MetricsServiceMock {
    pub fn new(sender: Sender<OTLPRequest>) -> Self {
        Self { sender }
    }
}

pub struct TraceServiceMock {
    sender: Sender<OTLPRequest>
}

impl TraceServiceMock {
    pub fn new(sender: Sender<OTLPRequest>) -> Self {
        Self { sender }
    }
}


#[tonic::async_trait]
impl LogsService for LogsServiceMock {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        self.sender.send(OTLPRequest::Logs(request.into_inner())).await.expect("Logs failed to be sent through channel");
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceMock {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        self.sender.send(OTLPRequest::Metrics(request.into_inner())).await.expect("Metrics failed to be sent through channel");
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl TraceService for TraceServiceMock {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        self.sender.send(OTLPRequest::Trace(request.into_inner())).await.expect("Traces failed to be sent through channel");
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}


pub fn start_mock_server(sender: Sender, listening_addr: SocketAddr, shutdown_signal: ) -> Result<(), Box<dyn Error>> {
    let mock_logs_service = LogsServiceServer::new(LogsServiceMock::new(sender.clone()));
    let mock_metrics_service = MetricsServiceServer::new(MetricsServiceMock::new(sender.clone()));
    let mock_trace_service = TraceServiceServer::new(TraceServiceMock::new(sender.clone()));
    tokio::spawn(async move {
        Server::builder().add_service(mock_logs_service).add_service(mock_metrics_service).add_service(mock_trace__service).serve_with_shutdown(listening_addr, async {
            // Wait for the shutdown signal
            shutdown_signal.await.ok();
        }).await
    });
    // start server in the background 
}