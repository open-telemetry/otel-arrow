use std::time::Duration;
use std::path::Path;

use tokio::time::timeout;

use crate::proto::opentelemetry::collector::trace::v1::{
    trace_service_client::TraceServiceClient,
    ExportTraceServiceRequest,
};

use crate::pdata::{TraceID, SpanID};

use crate::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, status::StatusCode,
};

use crate::proto::opentelemetry::common::v1::{
    AnyValue, KeyValue, InstrumentationScope,
};

use crate::proto::opentelemetry::resource::v1::{
    Resource,
};

use super::collector_test::{
    CollectorProcess, generate_otlp_to_otlp_config, start_test_receiver,
};

pub async fn test_otlp_round_trip<T: AsRef<Path>>(collector: T) -> Result<(), Box<dyn std::error::Error>> {
    // Port for the receiver (where the test sends data to the collector)
    const RECEIVER_PORT: u16 = 4317;
    // Port for the exporter (where the collector sends data back)
    const EXPORTER_PORT: u16 = 5317;
    
    // Start the test receiver server on the exporter port to receive the exported data
    let (server_handle, mut request_rx) = start_test_receiver(EXPORTER_PORT).await
        .map_err(|e| format!("Failed to start test receiver: {}", e))?;
    
    // Generate and start the collector with OTLP->OTLP config
    let collector_config = generate_otlp_to_otlp_config(RECEIVER_PORT, EXPORTER_PORT);
    let _collector = CollectorProcess::start(collector.as_ref(), &collector_config).await
        .map_err(|e| format!("Failed to start collector: {}", e))?;
    
    // Allow collector time to initialize
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create OTLP client to send test data
    let client_endpoint = format!("http://127.0.0.1:{}", RECEIVER_PORT);
    let mut client = TraceServiceClient::connect(client_endpoint).await?;
    
    // Create test span data
    let request = create_test_trace_request();
    
    // Keep a copy for comparison
    let expected_request = request.clone();
    
    // Send trace data to the collector
    client.export(request).await?;
    
    // Wait for the data to be received by our test receiver
    let received_request = match timeout(Duration::from_secs(5), request_rx.recv()).await {
        Ok(Some(req)) => req,
        Ok(None) => return Err("Channel closed".into()),
        Err(_) => return Err("Timeout waiting for exported data".into()),
    };
    
    // Compare the received data with what was sent
    assert_requests_equal(expected_request, received_request);
    
    // Gracefully shut down the server
    drop(request_rx);
    let _ = server_handle.await;
    
    Ok(())
}

/// Creates a test trace request with a simple span
fn create_test_trace_request() -> ExportTraceServiceRequest {
    let start_time = 1619712000000000000u64;
    let end_time = 1619712001000000000u64;
    let trace_id = TraceID::try_new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]).unwrap();
    let span_id = SpanID::try_new(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    // Create a simple span with some attributes
    let span = Span::build(trace_id, span_id, "test_span", start_time)
	.end_time_unix_nano(end_time)
        .attributes(vec![
            KeyValue::new("test.attribute", AnyValue::new_string("test value")),
        ])
        .status(Status::new("success", StatusCode::Ok))
	.finish();

    // Create a request with the span
    ExportTraceServiceRequest::new(
	vec![ResourceSpans::build(Resource::default())
	     .scope_spans(
		 vec![ScopeSpans::build(InstrumentationScope::default())
		      .spans(vec![span])
		      .finish()],
	     )
	     .finish()],
    )
}

/// Compare two trace requests to ensure they are equivalent
fn assert_requests_equal(expected: ExportTraceServiceRequest, actual: ExportTraceServiceRequest) {
    assert_eq!(
        expected,
        actual,
    );
}
