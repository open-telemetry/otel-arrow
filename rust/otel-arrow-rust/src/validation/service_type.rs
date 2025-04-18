// Service type abstractions for validation testing
use std::fmt::Debug;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use tonic::transport::{Channel, Server};
use tokio_stream::wrappers::TcpListenerStream;

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



/// A trait that abstracts over the different OTLP service types
pub trait ServiceType: Debug + Send + Sync + 'static {
    /// The request type for this service
    type Request: Clone + PartialEq + Send + Sync + 'static;
    
    /// The response type for this service
    type Response: Default + Send + 'static;
    
    /// The client type for this service
    type Client;
    
    /// The server implementation type
    type Server;
    
    /// Server type to add to the tonic server
    type TonicServer;
    
    /// The name of this service type (for logging and identification)
    fn name() -> &'static str;
    
    /// Create a new client for this service
    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error>;
    
    /// Create test data for this service
    fn create_test_data(name: &str) -> Self::Request;
    
    /// Send data through the client
    async fn send_data(client: &mut Self::Client, request: Self::Request) -> Result<Self::Response, tonic::Status>;
    
    /// Start a service-specific receiver
    async fn start_receiver(
        timeout_secs: Option<u64>,
    ) -> Result<
        (
            tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
            super::collector_test::TimeoutReceiver<Self::Request>,
            u16, // actual port number that was assigned
        ),
        String,
    >;
    
    /// Create a test receiver service and start it on a randomly assigned port
    async fn create_service(timeout_secs: Option<u64>) -> Result<
        (
            tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
            super::collector_test::TimeoutReceiver<Self::Request>,
            u16, // actual port number that was assigned
        ),
        String,
    > where
        Self: Sized
    {
        Self::start_receiver(timeout_secs).await
    }
}

/// Generic test receiver that can be used for any OTLP service
#[derive(Debug)]
pub struct TestReceiver<T> {
    pub request_tx: mpsc::Sender<T>,
}

/// Helper function to create a TCP server with dynamically allocated port for traces service
async fn create_trace_server(
    timeout_secs: Option<u64>,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        super::collector_test::TimeoutReceiver<ExportTraceServiceRequest>,
        u16, // actual port number that was assigned
    ),
    String,
> {
    use super::collector_test::TimeoutReceiver;
    
    // Bind to a specific address with port 0 for dynamic port allocation
    let addr = "127.0.0.1:0";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind listener: {}", e))?;
    
    // Get the assigned port
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    
    // Create a channel for receiving data
    let (request_tx, request_rx) = mpsc::channel::<ExportTraceServiceRequest>(100);
    
    // Create a test receiver
    let receiver = TestReceiver { request_tx };
    
    // Convert the listener to a stream of connections
    let incoming = TcpListenerStream::new(listener);
    
    // Create our server
    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(TraceServiceServer::new(receiver))
            .serve_with_incoming(incoming)
            .await
    });
    
    // Create a timeout-wrapped version of the receiver
    let timeout_duration = std::time::Duration::from_secs(timeout_secs.unwrap_or(10));
    let request_rx = TimeoutReceiver {
        inner: request_rx,
        timeout: timeout_duration,
    };
    
    Ok((handle, request_rx, port))
}

/// Helper function to create a TCP server with dynamically allocated port for metrics service
async fn create_metrics_server(
    timeout_secs: Option<u64>,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        super::collector_test::TimeoutReceiver<ExportMetricsServiceRequest>,
        u16, // actual port number that was assigned
    ),
    String,
> {
    use super::collector_test::TimeoutReceiver;
    
    // Bind to a specific address with port 0 for dynamic port allocation
    let addr = "127.0.0.1:0";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind listener: {}", e))?;
    
    // Get the assigned port
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    
    // Create a channel for receiving data
    let (request_tx, request_rx) = mpsc::channel::<ExportMetricsServiceRequest>(100);
    
    // Create a test receiver
    let receiver = TestReceiver { request_tx };
    
    // Convert the listener to a stream of connections
    let incoming = TcpListenerStream::new(listener);
    
    // Create our server
    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(MetricsServiceServer::new(receiver))
            .serve_with_incoming(incoming)
            .await
    });
    
    // Create a timeout-wrapped version of the receiver
    let timeout_duration = std::time::Duration::from_secs(timeout_secs.unwrap_or(10));
    let request_rx = TimeoutReceiver {
        inner: request_rx,
        timeout: timeout_duration,
    };
    
    Ok((handle, request_rx, port))
}

/// Helper function to create a TCP server with dynamically allocated port for logs service
async fn create_logs_server(
    timeout_secs: Option<u64>,
) -> Result<
    (
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
        super::collector_test::TimeoutReceiver<ExportLogsServiceRequest>,
        u16, // actual port number that was assigned
    ),
    String,
> {
    use super::collector_test::TimeoutReceiver;
    
    // Bind to a specific address with port 0 for dynamic port allocation
    let addr = "127.0.0.1:0";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind listener: {}", e))?;
    
    // Get the assigned port
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    
    // Create a channel for receiving data
    let (request_tx, request_rx) = mpsc::channel::<ExportLogsServiceRequest>(100);
    
    // Create a test receiver
    let receiver = TestReceiver { request_tx };
    
    // Convert the listener to a stream of connections
    let incoming = TcpListenerStream::new(listener);
    
    // Create our server
    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(LogsServiceServer::new(receiver))
            .serve_with_incoming(incoming)
            .await
    });
    
    // Create a timeout-wrapped version of the receiver
    let timeout_duration = std::time::Duration::from_secs(timeout_secs.unwrap_or(10));
    let request_rx = TimeoutReceiver {
        inner: request_rx,
        timeout: timeout_duration,
    };
    
    Ok((handle, request_rx, port))
}

/// Implementation of the Traces service type
#[derive(Debug)]
pub struct TracesServiceType;

impl ServiceType for TracesServiceType {
    type Request = ExportTraceServiceRequest;
    type Response = ExportTraceServiceResponse;
    type Client = TraceServiceClient<Channel>;
    type Server = TestReceiver<ExportTraceServiceRequest>;
    type TonicServer = TraceServiceServer<TestReceiver<ExportTraceServiceRequest>>;
    
    fn name() -> &'static str {
        "traces"
    }
    
    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        TraceServiceClient::connect(endpoint).await
    }
    
    fn create_test_data(name: &str) -> Self::Request {
        use crate::pdata::{SpanID, TraceID};
        use crate::proto::opentelemetry::trace::v1::{
            status::StatusCode, ResourceSpans, ScopeSpans, Span, Status,
        };
        use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
        use crate::proto::opentelemetry::resource::v1::Resource;
        
        // ... existing code ...
        let start_time = 1619712000000000000u64;
        let end_time = 1619712001000000000u64;
        let trace_id =
            TraceID::try_new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]).unwrap();
        let span_id = SpanID::try_new(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

        // Create a simple span with some attributes
        let span = Span::build(trace_id, span_id, name, start_time)
            .end_time_unix_nano(end_time)
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .status(Status::new("success", StatusCode::Ok))
            .finish();

        // Create a request with the span
        ExportTraceServiceRequest::new(vec![ResourceSpans::build(Resource::default())
            .scope_spans(vec![ScopeSpans::build(InstrumentationScope::default())
                .spans(vec![span])
                .finish()])
            .finish()])
    }
    
    async fn send_data(client: &mut Self::Client, request: Self::Request) -> Result<Self::Response, tonic::Status> {
        client.export(request).await.map(|response| response.into_inner())
    }
    
    async fn start_receiver(
        timeout_secs: Option<u64>,
    ) -> Result<
        (
            tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
            super::collector_test::TimeoutReceiver<Self::Request>,
            u16, // actual port number that was assigned
        ),
        String,
    > {
        create_trace_server(timeout_secs).await
    }
}

/// Implementation of the Metrics service type
#[derive(Debug)]
pub struct MetricsServiceType;

impl ServiceType for MetricsServiceType {
    type Request = ExportMetricsServiceRequest;
    type Response = ExportMetricsServiceResponse;
    type Client = MetricsServiceClient<Channel>;
    type Server = TestReceiver<ExportMetricsServiceRequest>;
    type TonicServer = MetricsServiceServer<TestReceiver<ExportMetricsServiceRequest>>;
    
    fn name() -> &'static str {
        "metrics"
    }
    
    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        MetricsServiceClient::connect(endpoint).await
    }
    
    fn create_test_data(name: &str) -> Self::Request {
        use crate::proto::opentelemetry::metrics::v1::{
            Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics,
        };
        use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
        use crate::proto::opentelemetry::resource::v1::Resource;
        
        // ... existing code ...
        let timestamp = 1619712000000000000u64;
        
        // Create a simple gauge metric with a single data point
        let data_point = NumberDataPoint::build_double(timestamp + 1000000000, 42.0)
            .start_time_unix_nano(timestamp)
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .finish();
            
        let metric = Metric::build_gauge(name, Gauge::new(vec![data_point]))
            .description(format!("Test metric {}", name))
            .unit("count")
            .finish();
            
        // Create a metrics request
        ExportMetricsServiceRequest::new(vec![ResourceMetrics::build(Resource::default())
            .scope_metrics(vec![ScopeMetrics::build(InstrumentationScope::default())
                .metrics(vec![metric])
                .finish()])
            .finish()])
    }
    
    async fn send_data(client: &mut Self::Client, request: Self::Request) -> Result<Self::Response, tonic::Status> {
        client.export(request).await.map(|response| response.into_inner())
    }
    
    async fn start_receiver(
        timeout_secs: Option<u64>,
    ) -> Result<
        (
            tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
            super::collector_test::TimeoutReceiver<Self::Request>,
            u16, // actual port number that was assigned
        ),
        String,
    > {
        create_metrics_server(timeout_secs).await
    }
}

/// Implementation of the Logs service type
#[derive(Debug)]
pub struct LogsServiceType;

impl ServiceType for LogsServiceType {
    type Request = ExportLogsServiceRequest;
    type Response = ExportLogsServiceResponse;
    type Client = LogsServiceClient<Channel>;
    type Server = TestReceiver<ExportLogsServiceRequest>;
    type TonicServer = LogsServiceServer<TestReceiver<ExportLogsServiceRequest>>;
    
    fn name() -> &'static str {
        "logs"
    }
    
    async fn connect_client(endpoint: String) -> Result<Self::Client, tonic::transport::Error> {
        LogsServiceClient::connect(endpoint).await
    }
    
    fn create_test_data(name: &str) -> Self::Request {
        use crate::proto::opentelemetry::logs::v1::{
            LogRecord, ResourceLogs, ScopeLogs, SeverityNumber,
        };
        use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
        use crate::proto::opentelemetry::resource::v1::Resource;
        
        // ... existing code ...
        let timestamp = 1619712000000000000u64;
        
        // Create a simple log record
        let log_record = LogRecord::build(timestamp, SeverityNumber::Info, "important")
            .severity_text("INFO")
            .body(AnyValue::new_string(format!("Test log message: {}", name)))
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .finish();
            
        // Create a logs request
        ExportLogsServiceRequest::new(vec![ResourceLogs::build(Resource::default())
            .scope_logs(vec![ScopeLogs::build(InstrumentationScope::default())
                .log_records(vec![log_record])
                .finish()])
            .finish()])
    }
    
    async fn send_data(client: &mut Self::Client, request: Self::Request) -> Result<Self::Response, tonic::Status> {
        client.export(request).await.map(|response| response.into_inner())
    }
    
    async fn start_receiver(
        timeout_secs: Option<u64>,
    ) -> Result<
        (
            tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
            super::collector_test::TimeoutReceiver<Self::Request>,
            u16, // actual port number that was assigned
        ),
        String,
    > {
        create_logs_server(timeout_secs).await
    }
}

// Implementations for the TestReceiver for each service type
#[tonic::async_trait]
impl TraceService for TestReceiver<ExportTraceServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let request_inner = request.into_inner();

        // Forward the received request to the test channel
        if let Err(err) = self.request_tx.send(request_inner).await {
            return Err(Status::internal(format!(
                "Failed to send trace data to test channel: {}",
                err
            )));
        }

        // Return success response
        Ok(Response::new(ExportTraceServiceResponse::default()))
    }
}

#[tonic::async_trait]
impl MetricsService for TestReceiver<ExportMetricsServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let request_inner = request.into_inner();

        // Forward the received request to the test channel
        if let Err(err) = self.request_tx.send(request_inner).await {
            return Err(Status::internal(format!(
                "Failed to send metrics data to test channel: {}",
                err
            )));
        }

        // Return success response
        Ok(Response::new(ExportMetricsServiceResponse::default()))
    }
}

#[tonic::async_trait]
impl LogsService for TestReceiver<ExportLogsServiceRequest> {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let request_inner = request.into_inner();

        // Forward the received request to the test channel
        if let Err(err) = self.request_tx.send(request_inner).await {
            return Err(Status::internal(format!(
                "Failed to send logs data to test channel: {}",
                err
            )));
        }

        // Return success response
        Ok(Response::new(ExportLogsServiceResponse::default()))
    }
}
