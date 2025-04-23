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

use super::service_type::{ServiceInputType, ServiceOutputType, TestReceiver};
use super::tcp_stream::ShutdownableTcpListenerStream;

use super::error::{Result, TonicStatusSnafu, TonicTransportSnafu};
use std::result::Result as StdResult;
use snafu::ResultExt;

use tonic::transport::{Channel, Server};
use tonic::{Request, Response, Status};

/// OTLP traces service type for testing
#[derive(Debug)]
pub struct OTLPTracesInputType;

impl ServiceInputType for OTLPTracesInputType {
    type Request = ExportTraceServiceRequest;
    type Response = ExportTraceServiceResponse;
    type Client = TraceServiceClient<Channel>;

    fn signal() -> &'static str {
        "traces"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client> {
        TraceServiceClient::connect(endpoint)
	    .await
	    .context(TonicTransportSnafu)
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response> {
        client
            .export(Request::new(request))
            .await
            .map(|response| response.into_inner())
	    .context(TonicStatusSnafu)
    }
}

#[derive(Debug)]
pub struct OTLPTracesOutputType;

impl ServiceOutputType for OTLPTracesOutputType {
    type Request = ExportTraceServiceRequest;
    type Server = TraceServiceServer<TestReceiver<ExportTraceServiceRequest>>;

    fn signal() -> &'static str {
        "traces"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: ShutdownableTcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(TraceServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }
}

/// OTLP metrics service type for testing
#[derive(Debug)]
pub struct OTLPMetricsInputType;

impl ServiceInputType for OTLPMetricsInputType {
    type Request = ExportMetricsServiceRequest;
    type Response = ExportMetricsServiceResponse;
    type Client = MetricsServiceClient<Channel>;

    fn signal() -> &'static str {
        "metrics"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client> {
        MetricsServiceClient::connect(endpoint)
	    .await
	    .context(TonicTransportSnafu)
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response> {
        client
            .export(Request::new(request))
            .await
            .map(|response| response.into_inner())
	    .context(TonicStatusSnafu)
    }
}

#[derive(Debug)]
pub struct OTLPMetricsOutputType;

impl ServiceOutputType for OTLPMetricsOutputType {
    type Request = ExportMetricsServiceRequest;
    type Server = MetricsServiceServer<TestReceiver<ExportMetricsServiceRequest>>;

    fn signal() -> &'static str {
        "metrics"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: ShutdownableTcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(MetricsServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }
}

/// OTLP logs service type for testing
#[derive(Debug)]
pub struct OTLPLogsInputType;

impl ServiceInputType for OTLPLogsInputType {
    type Request = ExportLogsServiceRequest;
    type Response = ExportLogsServiceResponse;
    type Client = LogsServiceClient<Channel>;

    fn signal() -> &'static str {
        "logs"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client> {
        LogsServiceClient::connect(endpoint)
	    .await
	    .context(TonicTransportSnafu)
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response> {
        client
            .export(Request::new(request))
            .await
            .map(|response| response.into_inner())
	    .context(TonicStatusSnafu)
    }
}

#[derive(Debug)]
pub struct OTLPLogsOutputType;

impl ServiceOutputType for OTLPLogsOutputType {
    type Request = ExportLogsServiceRequest;
    type Server = LogsServiceServer<TestReceiver<ExportLogsServiceRequest>>;

    fn signal() -> &'static str {
        "logs"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: ShutdownableTcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(LogsServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }
}

// Implementations for the TestReceiver for each OTLP service type

#[tonic::async_trait]
impl TraceService for TestReceiver<ExportTraceServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> StdResult<Response<ExportTraceServiceResponse>, Status> {
        self.process_export_request(request, "trace").await
    }
}

#[tonic::async_trait]
impl MetricsService for TestReceiver<ExportMetricsServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> StdResult<Response<ExportMetricsServiceResponse>, Status> {
        self.process_export_request(request, "metrics").await
    }
}

#[tonic::async_trait]
impl LogsService for TestReceiver<ExportLogsServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> StdResult<Response<ExportLogsServiceResponse>, Status> {
        self.process_export_request(request, "logs").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::otap::*;
    use crate::validation::scenarios::run_single_round_trip_test;
    use crate::validation::testdata;

    #[tokio::test]
    async fn test_otlp_traces_single_request() {
        run_single_round_trip_test::<OTLPTracesInputType, OTLPTracesOutputType, _>(
            testdata::traces::create_single_request,
            None, // Expect success
        )
        .await;
    }

    #[tokio::test]
    async fn test_otlp_metrics_single_request() {
        run_single_round_trip_test::<OTLPMetricsInputType, OTLPMetricsOutputType, _>(
            testdata::metrics::create_single_request,
            None, // Expect success
        )
        .await;
    }

    #[tokio::test]
    async fn test_otlp_logs_single_request() {
        run_single_round_trip_test::<OTLPLogsInputType, OTLPLogsOutputType, _>(
            testdata::logs::create_single_request,
            None, // Expect success
        )
        .await;
    }

    #[tokio::test]
    async fn test_otap_metrics_single_request() {
        run_single_round_trip_test::<OTLPMetricsInputType, OTAPMetricsOutputType, _>(
            testdata::metrics::create_single_request,
	    // This test expects a specific error due to disagreements
	    // between the Rust and Golang implementations about OTAP
	    // metrics encoding.
            Some("ColumnDataTypeMismatch"),
        )
        .await;
    }
}
