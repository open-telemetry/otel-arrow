use std::path::Path;
use std::fmt::Debug;

use super::service_type::{ServiceType, start_test_receiver};
use super::collector_test::{generate_otlp_to_otlp_config, CollectorProcess};

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
    // Generate random ports in the high u16 range to avoid conflicts
    // Use test_name in hash calculation to ensure different tests get different ports
    let name_hash = test_name.chars().fold(0u16, |acc, c| acc.wrapping_add(c as u16));
    let random_value = rand::random::<u16>();
    let combined_value = random_value.wrapping_add(name_hash);
    let receiver_port = 40000 + (combined_value % 25000);
    
    // Print to stdout to ensure we see the debug info in test output
    println!("PORT DEBUG: test_name='{}', name_hash={}, random_value={}, combined_value={}, receiver_port={}", 
              test_name, name_hash, random_value, combined_value, receiver_port);

    // Start the test receiver server with a 10-second timeout to avoid tests getting stuck
    let (server_handle, mut request_rx, exporter_port) = start_test_receiver::<S>(Some(10))
        .await
        .map_err(|e| format!("Failed to start test receiver: {}", e))?;

    // Generate and start the collector with OTLP->OTLP config using the dynamic ports
    let collector_config = generate_otlp_to_otlp_config(S::name(), receiver_port, exporter_port);

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

    // Proper cleanup sequence to avoid port conflicts in subsequent tests:
    
    // 1. Drop the client connection first
    drop(client);
    
    // 2. Send SIGTERM to the collector process to initiate graceful shutdown
    // This ensures the collector stops sending to our exporter
    match collector.shutdown().await {
        Ok(status) => {
            if let Some(s) = status {
                eprintln!("Collector exited with status: {}", s);
            } else {
                eprintln!("Collector shutdown initiated");
            }
        },
        Err(e) => eprintln!("Error shutting down collector: {}", e),
    }
    
    // 3. Give the collector a moment to cleanly disconnect
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    // 4. Now drop the request receiver 
    drop(request_rx);
    
    // 5. Wait for the server to shut down with timeout
    match tokio::time::timeout(std::time::Duration::from_secs(5), server_handle).await {
        Ok(Ok(_)) => eprintln!("{} server shut down successfully", S::name()),
        Ok(Err(e)) => eprintln!("Error shutting down {} server: {}", S::name(), e),
        Err(_) => {
            eprintln!("Timed out waiting for {} server to shut down", S::name());
            // We could force-abort the server here if needed
        }
    }
    
    // 6. Wait a moment before returning to ensure all sockets are properly closed
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    Ok(())
}
