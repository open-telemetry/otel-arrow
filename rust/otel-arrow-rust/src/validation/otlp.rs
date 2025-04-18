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
use crate::proto::opentelemetry::experimental::arrow::v1::{
    arrow_metrics_service_server::{ArrowMetricsService, ArrowMetricsServiceServer},
    BatchArrowRecords, BatchStatus, StatusCode,
};

use super::service_type::{ServiceInputType, ServiceOutputType, TestReceiver};

use futures::Stream;
use std::pin::Pin;
use tokio_stream::wrappers::TcpListenerStream;
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

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        TraceServiceClient::connect(endpoint).await
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
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(TraceServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }
}

/// Legacy type for backwards compatibility
#[derive(Debug)]
pub struct TracesServiceType;

impl ServiceInputType for TracesServiceType {
    type Request = ExportTraceServiceRequest;
    type Response = ExportTraceServiceResponse;
    type Client = TraceServiceClient<Channel>;

    fn signal() -> &'static str {
        "traces"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        OTLPTracesInputType::connect_client(endpoint).await
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response, tonic::Status> {
        OTLPTracesInputType::send_data(client, request).await
    }
}

impl ServiceOutputType for TracesServiceType {
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
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        OTLPTracesOutputType::create_server(receiver, incoming)
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

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        MetricsServiceClient::connect(endpoint).await
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
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(MetricsServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }
}

/// OTAP metrics service type for testing
#[derive(Debug)]
pub struct OTAPMetricsOutputType;

/// The OTAP metrics service adapter that translates OTAP arrow data to OTLP metrics
pub struct OTAPMetricsAdapter {
    receiver: TestReceiver<ExportMetricsServiceRequest>,
}

impl OTAPMetricsAdapter {
    fn new(receiver: TestReceiver<ExportMetricsServiceRequest>) -> Self {
        Self { receiver }
    }
}

/// Stream type for ArrowMetricsService implementation
type BatchStatusStream =
    Pin<Box<dyn Stream<Item = std::result::Result<BatchStatus, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl ArrowMetricsService for OTAPMetricsAdapter {
    type ArrowMetricsStream = BatchStatusStream;

    async fn arrow_metrics(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowMetricsStream>, Status> {
        let mut input_stream = request.into_inner();
        let receiver = self.receiver.clone();
        
        // Create a channel to send batch statuses back to the client
        let (tx, rx) = tokio::sync::mpsc::channel(100); // TODO?
        
        // Spawn a task to process incoming arrow records and convert them to OTLP
        tokio::spawn(async move {
            let mut related_data = crate::otlp::related_data::RelatedData::default();
            
            loop {
		match input_stream.message().await {
                    Ok(Some(batch)) => {
			let batch_id = batch.batch_id;
                    
			// Process each arrow payload in the batch
			match process_arrow_batch(&batch, &mut related_data) {
                            Ok(metrics_request) => {
				// Send the converted OTLP data to the test receiver
				match receiver.process_export_request::<ExportMetricsServiceRequest>(Request::new(metrics_request), "metrics").await {
                                    Ok(_) => {
					// Send success status back to client
					if tx.send(Ok(BatchStatus {
                                            batch_id,
                                            status_code: StatusCode::Ok as i32,
                                            status_message: "Successfully processed".to_string(),
					})).await.is_err() {
                                            break; // Client disconnected
					}
                                    }
                                    Err(e) => {
					// Send error status back to client
					if tx.send(Ok(BatchStatus {
                                            batch_id,
                                            status_code: StatusCode::Internal as i32,
                                            status_message: format!("Failed to process: {}", e),
					})).await.is_err() {
                                            break; // Client disconnected
					}
                                    }
				}
                            }
                            Err(e) => {
				// Send error status back to client
				if tx.send(Ok(BatchStatus {
                                    batch_id,
                                    status_code: StatusCode::InvalidArgument as i32,
                                    status_message: format!("Failed to convert: {}", e),
				})).await.is_err() {
                                    break; // Client disconnected
				}
                            }
                        }
                    }
                    Ok(None) => {
			break;
		    }
                    Err(e) => {
			// Error receiving batch from client
			let _ = tx.send(Ok(BatchStatus {
                            batch_id: -1, // Unknown batch ID for protocol errors
                            status_code: StatusCode::Internal as i32,
                            status_message: format!("Failed to receive batch: {}", e),
			})).await;
			break;
                    }
		}
            }
	});
	    
	// Convert the mpsc receiver to a Stream
	let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
	Ok(Response::new(Box::pin(output_stream) as Self::ArrowMetricsStream))
    }
}

// Process an Arrow batch and convert it to OTLP metrics
fn process_arrow_batch(
    batch: &BatchArrowRecords,
    related_data: &mut crate::otlp::related_data::RelatedData,
) -> Result<ExportMetricsServiceRequest, String> {
    use crate::decode::record_message::RecordMessage;
    use arrow::ipc::reader::StreamReader;
    use std::io::Cursor;
    
    let mut metrics_request = ExportMetricsServiceRequest::default();
    
    // Process each arrow payload
    for payload in &batch.arrow_payloads {
        // Create a reader for the Arrow record
        let reader = StreamReader::try_new(Cursor::new(&payload.record), None)
            .map_err(|e| format!("Failed to create Arrow reader: {}", e))?;
        
        // Get the first (and only) batch
        let arrow_batch = reader
            .into_iter()
            .next()
            .ok_or_else(|| "Empty Arrow batch".to_string())?
            .map_err(|e| format!("Failed to read Arrow batch: {}", e))?;
        
        // Create a record message
        let record_message = RecordMessage {
            batch_id: batch.batch_id,
            schema_id: payload.schema_id.clone(),
            payload_type: crate::opentelemetry::ArrowPayloadType::try_from(payload.r#type)
                .map_err(|_| format!("Invalid payload type: {}", payload.r#type))?,
            record: arrow_batch,
        };
        
        // Convert Arrow payload to OTLP metrics
        let otlp_metrics = crate::otlp::metric::metrics_from(&record_message.record, related_data)
            .map_err(|e| format!("Failed to convert to OTLP: {:?}", e))?;
        
        // Merge with existing metrics
        merge_metrics(&mut metrics_request, otlp_metrics);
    }
    
    Ok(metrics_request)
}

// Merge two ExportMetricsServiceRequest objects
fn merge_metrics(
    target: &mut ExportMetricsServiceRequest,
    source: ExportMetricsServiceRequest,
) {
    target.resource_metrics.extend(source.resource_metrics);
}

impl ServiceOutputType for OTAPMetricsOutputType {
    type Request = ExportMetricsServiceRequest;
    type Server = ArrowMetricsServiceServer<OTAPMetricsAdapter>;

    fn signal() -> &'static str {
        "metrics"
    }

    fn protocol() -> &'static str {
        "otelarrow" // matches the exporter and receiver name
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            let adapter = OTAPMetricsAdapter::new(receiver);
            Server::builder()
                .add_service(ArrowMetricsServiceServer::new(adapter))
                .serve_with_incoming(incoming)
                .await
        })
    }
}

/// Legacy type for backwards compatibility
#[derive(Debug)]
pub struct MetricsServiceType;

impl ServiceInputType for MetricsServiceType {
    type Request = ExportMetricsServiceRequest;
    type Response = ExportMetricsServiceResponse;
    type Client = MetricsServiceClient<Channel>;

    fn signal() -> &'static str {
        "metrics"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        OTLPMetricsInputType::connect_client(endpoint).await
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response, tonic::Status> {
        OTLPMetricsInputType::send_data(client, request).await
    }
}

impl ServiceOutputType for MetricsServiceType {
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
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        OTLPMetricsOutputType::create_server(receiver, incoming)
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

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        LogsServiceClient::connect(endpoint).await
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
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        tokio::spawn(async move {
            Server::builder()
                .add_service(LogsServiceServer::new(receiver))
                .serve_with_incoming(incoming)
                .await
        })
    }
}

/// Legacy type for backwards compatibility
#[derive(Debug)]
pub struct LogsServiceType;

impl ServiceInputType for LogsServiceType {
    type Request = ExportLogsServiceRequest;
    type Response = ExportLogsServiceResponse;
    type Client = LogsServiceClient<Channel>;

    fn signal() -> &'static str {
        "logs"
    }

    fn protocol() -> &'static str {
        "otlp"
    }

    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        OTLPLogsInputType::connect_client(endpoint).await
    }

    async fn send_data(
        client: &mut Self::Client,
        request: Self::Request,
    ) -> Result<Self::Response, tonic::Status> {
        OTLPLogsInputType::send_data(client, request).await
    }
}

impl ServiceOutputType for LogsServiceType {
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
        incoming: TcpListenerStream,
    ) -> tokio::task::JoinHandle<Result<(), tonic::transport::Error>> {
        OTLPLogsOutputType::create_server(receiver, incoming)
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
    async fn test_otlp_traces_single_request() {
        run_single_round_trip_test::<OTLPTracesInputType, OTLPTracesOutputType, _>(
            testdata::traces::create_single_request
        ).await;
    }

    #[tokio::test]
    async fn test_otlp_metrics_single_request() {
        run_single_round_trip_test::<OTLPMetricsInputType, OTLPMetricsOutputType, _>(
            testdata::metrics::create_single_request
        ).await;
    }

    #[tokio::test]
    async fn test_otlp_logs_single_request() {
        run_single_round_trip_test::<OTLPLogsInputType, OTLPLogsOutputType, _>(
            testdata::logs::create_single_request
        ).await;
    }

    #[tokio::test]
    async fn test_otap_metrics_single_request() {
        run_single_round_trip_test::<OTLPMetricsInputType, OTAPMetricsOutputType, _>(
            testdata::metrics::create_single_request
        ).await;
    }
}
