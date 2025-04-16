use std::time::Duration;

use tokio::time::timeout;

use crate::proto::opentelemetry::collector::trace::v1::{
    trace_service_client::TraceServiceClient,
    ExportTraceServiceRequest,
};

use crate::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, status::StatusCode,
};

use crate::proto::opentelemetry::common::v1::{
    AnyValue, KeyValue,
};

use super::collector_test::{
    CollectorProcess, generate_otlp_to_otlp_config, start_test_receiver,
};

/// OTLP round-trip validation test
/// 
/// This function:
/// 1. Starts a test receiver server to receive exported data
/// 2. Starts an OTel collector process with OTLP->OTLP pipeline
/// 3. Creates an OTLP client and sends test data to the collector
/// 4. Verifies the received data matches what was sent
pub async fn test_otlp_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    // Port for the receiver (where the test sends data to the collector)
    const RECEIVER_PORT: u16 = 4317;
    // Port for the exporter (where the collector sends data back)
    const EXPORTER_PORT: u16 = 5317;
    
    // Start the test receiver server on the exporter port to receive the exported data
    let (server_handle, mut request_rx) = start_test_receiver(EXPORTER_PORT).await
        .map_err(|e| format!("Failed to start test receiver: {}", e))?;
    
    // Generate and start the collector with OTLP->OTLP config
    let collector_config = generate_otlp_to_otlp_config(RECEIVER_PORT, EXPORTER_PORT);
    let _collector = CollectorProcess::start(&collector_config).await
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
    let start_time = 1619712000000000000;
    let end_time = 1619712001000000000;
    let trace_id = TraceID(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanID(&[1, 2, 3, 4, 5, 6, 7, 8]);

    // Create a simple span with some attributes
    let span = Span::build(trace_id, span_id, "test_span", start_time)
	.end_time_unix_nano(end_time)
        .attributes(vec![
            KeyValue::new("test.attribute", AnyValue::new_string("test value")),
        ])
        .status(Status::new(StatusCode::Ok, "success"))
	.finish();

    // Create a request with the span
    ExportTraceServiceRequest::new(
	vec![ResourceSpans::new(
	    Resource::default(),
            vec![ScopeSpans::new(
		Scope::default(),
		vec![span],
            )],
        )],
    )
}

/// Compare two trace requests to ensure they are equivalent
fn assert_requests_equal(expected: ExportTraceServiceRequest, actual: ExportTraceServiceRequest) {
    assert_eq!(
        expected.resource_spans.len(),
        actual.resource_spans.len(),
        "Different number of resource spans"
    );
    
    for (i, (expected_rs, actual_rs)) in expected.resource_spans.iter().zip(actual.resource_spans.iter()).enumerate() {
        assert_eq!(
            expected_rs.scope_spans.len(),
            actual_rs.scope_spans.len(),
            "Different number of scope spans in resource span {}", i
        );
        
        for (j, (expected_ss, actual_ss)) in expected_rs.scope_spans.iter().zip(actual_rs.scope_spans.iter()).enumerate() {
            assert_eq!(
                expected_ss.spans.len(),
                actual_ss.spans.len(),
                "Different number of spans in scope span {}.{}", i, j
            );
            
            for (k, (expected_span, actual_span)) in expected_ss.spans.iter().zip(actual_ss.spans.iter()).enumerate() {
                assert_eq!(
                    expected_span.trace_id,
                    actual_span.trace_id,
                    "Different trace ID in span {}.{}.{}", i, j, k
                );
                assert_eq!(
                    expected_span.span_id,
                    actual_span.span_id,
                    "Different span ID in span {}.{}.{}", i, j, k
                );
                assert_eq!(
                    expected_span.name,
                    actual_span.name,
                    "Different name in span {}.{}.{}", i, j, k
                );
                // Add more detailed comparisons as needed
            }
        }
    }
}
