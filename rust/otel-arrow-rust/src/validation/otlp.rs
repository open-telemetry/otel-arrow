// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;
use std::path::Path;

use super::collector::{generate_config, CollectorProcess};
use super::service_type::{
    start_test_receiver, LogsServiceType, MetricsServiceType, ServiceType, TracesServiceType,
};

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
    let name_hash = test_name
        .chars()
        .fold(0u16, |acc, c| acc.wrapping_add(c as u16));
    let random_value = rand::random::<u16>();
    let combined_value = random_value.wrapping_add(name_hash);
    let receiver_port = 40000 + (combined_value % 25000);

    // Start the test receiver server with a 10-second timeout to avoid tests getting stuck
    let (server_handle, mut request_rx, exporter_port) =
        start_test_receiver::<S>(Some(10))
            .await
            .map_err(|e| format!("Failed to start test receiver: {}", e))?;

    // Generate and start the collector with OTLP->OTLP config using the dynamic ports
    let collector_config = generate_config("otlp", "otlp", S::name(), receiver_port, exporter_port);

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

    // Send a shutdown signal to the collector process.
    match collector.shutdown().await {
        Ok(status) => {
            if let Some(s) = status {
                eprintln!("Collector exited with status: {}", s);
            } else {
                eprintln!("Collector shutdown initiated");
            }
        }
        Err(e) => eprintln!("Error shutting down collector: {}", e),
    }

    drop(request_rx);

    // Wait for the server to shut down with timeout
    match tokio::time::timeout(std::time::Duration::from_secs(5), server_handle).await {
        Ok(Ok(_)) => eprintln!("{} server shut down successfully", S::name()),
        Ok(Err(e)) => eprintln!("Error shutting down {} server: {}", S::name(), e),
        Err(_) => {
            eprintln!("Timed out waiting for {} server to shut down", S::name());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to run a test with the given service type
    async fn run_test_with_service<S: ServiceType>(test_name: &str)
    where
        S::Request: std::fmt::Debug + PartialEq,
    {
        let env =
            std::env::var("OTEL_COLLECTOR_PATH").unwrap_or("../../bin/otelarrowcol".to_string());

        match tokio::time::timeout(
            std::time::Duration::from_secs(20),
            test_signal_round_trip::<S, _>(env, test_name),
        )
        .await
        {
            Ok(result) => result.unwrap(),
            Err(_) => panic!("Test timed out after 20 seconds"),
        }
    }

    // Test the trace signal
    #[tokio::test]
    async fn test_traces_single_request() {
        run_test_with_service::<TracesServiceType>("test_span").await;
    }

    // Test the metrics signal
    #[tokio::test]
    async fn test_metrics_single_request() {
        run_test_with_service::<MetricsServiceType>("test_metric").await;
    }

    // Test the logs signal
    #[tokio::test]
    async fn test_logs_single_request() {
        run_test_with_service::<LogsServiceType>("test_log").await;
    }
}
