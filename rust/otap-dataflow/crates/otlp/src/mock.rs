use crate::proto::opentelemetry::collector::logs::v1::logs_service_server::LogsService;
use crate::proto::opentelemetry::collector::logs::v1::{ExportLogsServiceRequest, ExportLogsServiceResponse};
use crate::proto::opentelemetry::collector::metrics::v1::metrics_service_server::MetricsService;
use crate::proto::opentelemetry::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use crate::proto::opentelemetry::collector::trace::v1::trace_service_server::TraceService;
use crate::proto::opentelemetry::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};

use crate::proto::opentelemetry::collector::{logs::v1::logs_service_server::LogsServiceServer,
    metrics::v1::metrics_service_server::MetricsServiceServer,
    trace::v1::trace_service_server::TraceServiceServer};
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;
use std::error::Error;
use std::net::SocketAddr;
use crate::grpc::OTLPRequest;
use tokio::task::JoinHandle;

/// struct that implements the Log Service trait
pub struct LogsServiceMock {
    sender: Sender<OTLPRequest>
}

impl LogsServiceMock {
    /// creates a new mock logs service
    pub fn new(sender: Sender<OTLPRequest>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Metrics Service trait
pub struct MetricsServiceMock {
    sender: Sender<OTLPRequest>
}

impl MetricsServiceMock {
    /// creates a new mock metrics service
    pub fn new(sender: Sender<OTLPRequest>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Trace Service trait
pub struct TraceServiceMock {
    sender: Sender<OTLPRequest>
}

impl TraceServiceMock {
    /// creates a new mock trace service
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
        self.sender.send(OTLPRequest::Traces(request.into_inner())).await.expect("Traces failed to be sent through channel");
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

/// Starts a OTLP server in the background on a given address and a shutdown signal channel

/// # Arguments
/// * `sender` - A sender for OTLP requests to be sent through the channel
/// * `listening_addr` - The address to listen on
/// * `shutdown_signal` - A receiver for the shutdown signal

pub fn start_mock_server<T: Send + 'static>(sender: Sender<OTLPRequest>, listening_addr: SocketAddr, shutdown_signal: Receiver<T>) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>{
    let mock_logs_service = LogsServiceServer::new(LogsServiceMock::new(sender.clone()));
    let mock_metrics_service = MetricsServiceServer::new(MetricsServiceMock::new(sender.clone()));
    let mock_trace_service = TraceServiceServer::new(TraceServiceMock::new(sender.clone()));
    tokio::spawn(async move {
        Server::builder().add_service(mock_logs_service).add_service(mock_metrics_service).add_service(mock_trace_service).serve_with_shutdown(listening_addr, async {
            // Wait for the shutdown signal
            drop(shutdown_signal.await.ok());
        }).await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    })
    // start server in the background 
}