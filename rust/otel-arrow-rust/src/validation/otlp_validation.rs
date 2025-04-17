use std::path::Path;
use std::fmt::Debug;

use crate::validation::service_type::ServiceType;

use super::collector_test::{generate_otlp_to_otlp_config, start_test_receiver, CollectorProcess};

/// Generic function for testing round-trip fidelity of any telemetry signal type
/// 
/// This function will:
/// 1. Start a generic test receiver server
/// 2. Start the OTel collector
/// 3. Send test data to the collector
/// 4. Verify that the exported data matches what was sent
/// 
/// The service type parameter S determines which signal type to test (traces, metrics, or logs)
pub async fn test_signal_round_trip<S, T>(
    collector: T,
    test_name: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: ServiceType,
    S::Request: Debug + PartialEq,
    T: AsRef<Path>,
{
    // Generate a random port in the high u16 range for the receiver
    let receiver_port = 40000 + (rand::random::<u16>() % 25000);

    // Start the test receiver server with a 10-second timeout to avoid tests getting stuck
    let (server_handle, mut request_rx, exporter_port) = start_test_receiver::<S>(Some(10))
        .await
        .map_err(|e| format!("Failed to start test receiver: {}", e))?;

    // Generate and start the collector with OTLP->OTLP config using the dynamic port
    let collector_config = generate_otlp_to_otlp_config(receiver_port, exporter_port);

    let mut collector = CollectorProcess::start(collector.as_ref(), &collector_config)
        .await
        .map_err(|e| format!("Failed to start collector: {}", e))?;

    // Create OTLP client to send test data
    let client_endpoint = format!("http://127.0.0.1:{}", receiver_port);
    let mut client = S::connect_client(client_endpoint).await?;

    // Create test data using the service type's create_test_data function
    let request = S::create_test_data(test_name);

    // Keep a copy for comparison
    let expected_request = request.clone();

    // Send data to the collector using the service type's send_data function
    S::send_data(&mut client, request).await?;

    // Wait for the data to be received by our test receiver
    let received_request = match request_rx.recv().await {
        Ok(req) => req,
        Err(e) => return Err(format!("Error receiving data: {}", e).into()),
    };

    // Compare the received data with what was sent
    assert_eq!(expected_request, received_request);

    // Drop the client connection first
    drop(client);

    // Now drop the request receiver and gracefully shut down the server
    drop(request_rx);

    // Send SIGTERM to the collector process to initiate graceful shutdown
    collector.shutdown().await?;

    match tokio::time::timeout(std::time::Duration::from_secs(5), server_handle).await {
        Ok(Ok(_)) => eprintln!("{} server shut down successfully", S::name()),
        Ok(Err(e)) => eprintln!("Error shutting down {} server: {}", S::name(), e),
        Err(_) => eprintln!("Timed out waiting for {} server to shut down", S::name()),
    }

    Ok(())
}

/// Test round-trip fidelity of trace data through the collector
/// 
/// This is a convenience wrapper around test_signal_round_trip for traces only.
pub async fn test_otlp_round_trip<T: AsRef<Path>>(
    collector: T,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::validation::service_type::TracesServiceType;
    
    test_signal_round_trip::<TracesServiceType, _>(collector, "test1").await
}
