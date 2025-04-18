// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::opentelemetry::collector::logs::v1::{
    logs_service_client::LogsServiceClient,
    logs_service_server::{LogsService, LogsServiceServer},
    ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use crate::proto::opentelemetry::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient,
    metrics_service_server::{MetricsService, MetricsServiceServer},
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use crate::proto::opentelemetry::collector::trace::v1::{
    trace_service_client::TraceServiceClient,
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};

use super::service_type::{ServiceType, TestReceiver};

use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};
use tonic::{Request, Response, Status};

/// OTLP traces service type for testing
#[derive(Debug)]
pub struct TracesServiceType;

impl ServiceType for TracesServiceType {
    type Request = ExportTraceServiceRequest;
    type Response = ExportTraceServiceResponse;
    type Client = TraceServiceClient<Channel>;
    type Server = TraceServiceServer<TestReceiver<ExportTraceServiceRequest>>;

    fn name() -> &'static str {
        "traces"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        TraceServiceClient::connect(endpoint).await
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(TraceServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response, tonic::Status> {
        client
            .export(Request::new(request))
            .await
            .map(|response| response.into_inner())
    }
}

/// OTLP metrics service type for testing
#[derive(Debug)]
pub struct MetricsServiceType;

impl ServiceType for MetricsServiceType {
    type Request = ExportMetricsServiceRequest;
    type Response = ExportMetricsServiceResponse;
    type Client = MetricsServiceClient<Channel>;
    type Server = MetricsServiceServer<TestReceiver<ExportMetricsServiceRequest>>;

    fn name() -> &'static str {
        "metrics"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        MetricsServiceClient::connect(endpoint).await
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(MetricsServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response, tonic::Status> {
        client
            .export(Request::new(request))
            .await
            .map(|response| response.into_inner())
    }
}

/// OTLP logs service type for testing
#[derive(Debug)]
pub struct LogsServiceType;

impl ServiceType for LogsServiceType {
    type Request = ExportLogsServiceRequest;
    type Response = ExportLogsServiceResponse;
    type Client = LogsServiceClient<Channel>;
    type Server = LogsServiceServer<TestReceiver<ExportLogsServiceRequest>>;

    fn name() -> &'static str {
        "logs"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        LogsServiceClient::connect(endpoint).await
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(LogsServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response, tonic::Status> {
        client
            .export(Request::new(request))
            .await
            .map(|response| response.into_inner())
    }
}

// Implementations for the TestReceiver for each OTLP service type

#[tonic::async_trait]
impl TraceService for TestReceiver<ExportTraceServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        self.process_export_request(request, "trace").await
    }
}

#[tonic::async_trait]
impl MetricsService for TestReceiver<ExportMetricsServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        self.process_export_request(request, "metrics").await
    }
}

#[tonic::async_trait]
impl LogsService for TestReceiver<ExportLogsServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        self.process_export_request(request, "logs").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::testdata;
    use crate::validation::scenarios::run_single_round_trip_test;

    #[tokio::test]
    async fn test_traces_single_request() {
        run_single_round_trip_test::<TracesServiceType, _>(testdata::traces::create_single_request)
            .await;
    }

    #[tokio::test]
    async fn test_metrics_single_request() {
        run_single_round_trip_test::<MetricsServiceType, _>(
            testdata::metrics::create_single_request,
        )
        .await;
    }

    #[tokio::test]
    async fn test_logs_single_request() {
        run_single_round_trip_test::<LogsServiceType, _>(testdata::logs::create_single_request)
            .await;
    }
}
