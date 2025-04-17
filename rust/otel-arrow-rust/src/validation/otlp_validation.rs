use std::path::Path;

use crate::proto::opentelemetry::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};

use crate::pdata::{SpanID, TraceID};

use crate::proto::opentelemetry::trace::v1::{
    status::StatusCode, ResourceSpans, ScopeSpans, Span, Status,
};

use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};

use crate::proto::opentelemetry::resource::v1::Resource;

use super::collector_test::{generate_otlp_to_otlp_config, start_test_receiver, CollectorProcess};

pub async fn test_otlp_round_trip<T: AsRef<Path>>(
    collector: T,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate a random port in the high u16 range for the receiver
    let receiver_port = 40000 + (rand::random::<u16>() % 25000);

    // Start the test receiver server on a dynamically allocated port to receive the exported data
    // Start the test receiver server with a 10-second timeout to avoid tests getting stuck
    let (server_handle, mut request_rx, exporter_port) = start_test_receiver(Some(10))
        .await
        .map_err(|e| format!("Failed to start test receiver: {}", e))?;

    // Generate and start the collector with OTLP->OTLP config using the dynamic port
    let collector_config = generate_otlp_to_otlp_config(receiver_port, exporter_port);

    let mut collector = CollectorProcess::start(collector.as_ref(), &collector_config)
        .await
        .map_err(|e| format!("Failed to start collector: {}", e))?;

    // Create OTLP client to send test data
    let client_endpoint = format!("http://127.0.0.1:{}", receiver_port);
    let mut client = TraceServiceClient::connect(client_endpoint).await?;

    // Create test span data
    let request = create_test_trace_request("test1");

    // Keep a copy for comparison
    let expected_request = request.clone();

    // Send trace data to the collector
    client.export(request).await?;

    // Wait for the data to be received by our test receiver
    let received_request = match request_rx.recv().await {
        Ok(req) => req,
        Err(e) => return Err(format!("Error receiving data: {}", e).into()),
    };

    // Compare the received data with what was sent
    assert_requests_equal(expected_request, received_request);

    // Drop the client connection first
    drop(client);

    // Now drop the request receiver and gracefully shut down the server
    drop(request_rx);

    // Send SIGTERM to the collector process to initiate graceful shutdown
    collector.shutdown().await?;

    match tokio::time::timeout(std::time::Duration::from_secs(5), server_handle).await {
        Ok(Ok(_)) => eprintln!("Server shut down successfully"),
        Ok(Err(e)) => eprintln!("Error shutting down server: {}", e),
        Err(_) => eprintln!("Timed out waiting for server to shut down"),
    }

    Ok(())
}

/// Creates a test trace request with a simple span
fn create_test_trace_request<T: AsRef<str>>(name: T) -> ExportTraceServiceRequest {
    let start_time = 1619712000000000000u64;
    let end_time = 1619712001000000000u64;
    let trace_id =
        TraceID::try_new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]).unwrap();
    let span_id = SpanID::try_new(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    // Create a simple span with some attributes
    let span = Span::build(trace_id, span_id, name.as_ref(), start_time)
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

/// Compare two trace requests to ensure they are equivalent
fn assert_requests_equal(expected: ExportTraceServiceRequest, actual: ExportTraceServiceRequest) {
    assert_eq!(expected, actual);
}
