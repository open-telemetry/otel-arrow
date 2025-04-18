// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

use super::collector::{generate_config, CollectorProcess, COLLECTOR_PATH};
use super::service_type::{
    start_test_receiver, LogsServiceType, MetricsServiceType,
    ServiceType, TracesServiceType,
};

pub const SHUTDOWN_TIMEOUT_SECONDS: u64 = 5;
pub const RECEIVER_TIMEOUT_SECONDS: u64 = 10;
pub const TEST_TIMEOUT_SECONDS: u64 = 20;

/// TestContext contains all the necessary components for running a test
pub struct TestContext<S: ServiceType> {
    pub client: S::Client,
    pub collector: CollectorProcess,
    pub request_rx: super::collector::TimeoutReceiver<S::Request>,
    pub server_handle: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
}

/// Generic test runner for telemetry signal tests
///
/// This function will:
/// 1. Start a generic test receiver server
/// 2. Start the OTel collector
/// 3. Create a test context with client and receiver
/// 4. Run the supplied test logic
/// 5. Perform cleanup
///
/// The service type parameter S determines which signal type to test (traces, metrics, or logs)
pub async fn run_test<S, T, F>(
    test_logic: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: ServiceType,
    S::Request: Debug + PartialEq,
    F: FnOnce(TestContext<S>) -> T,
    T: std::future::Future<Output = (TestContext<S>, Result<(), Box<dyn std::error::Error>>)>,
{
    // Generate random ports in the high u16 range to avoid conflicts
    let random_value = rand::random::<u16>();
    let receiver_port = 40000 + (random_value % 25000);

    // Start the test receiver server and wrap it with a timeout to avoid tests getting stuck
    let (server_handle, request_rx_raw, exporter_port) =
        start_test_receiver::<S>()
            .await
            .map_err(|e| format!("Failed to start test receiver: {}", e))?;
            
    // Create a timeout-wrapped version of the receiver
    let timeout_duration = std::time::Duration::from_secs(RECEIVER_TIMEOUT_SECONDS);
    let request_rx = super::collector::TimeoutReceiver {
        inner: request_rx_raw,
        timeout: timeout_duration,
    };

    // Generate and start the collector with OTLP->OTLP config using the dynamic ports
    let collector_config = generate_config("otlp", "otlp", S::name(), receiver_port, exporter_port);

    let collector = CollectorProcess::start(COLLECTOR_PATH.clone(), &collector_config)
        .await
        .map_err(|e| format!("Failed to start collector: {}", e))?;

    // Create OTLP client to send test data
    let client_endpoint = format!("http://127.0.0.1:{}", receiver_port);
    let client = S::connect_client(client_endpoint).await?;

    // Create the test context
    let context = TestContext {
        client,
        collector,
        request_rx,
        server_handle,
    };

    // Run the provided test logic, transferring ownership of the context
    // The test_logic now returns the context back along with the result
    let (mut context, result) = test_logic(context).await;

    // Cleanup: drop the client connection first
    drop(context.client);

    // Send a shutdown signal to the collector process.
    match context.collector.shutdown().await {
        Ok(status) => {
            if let Some(s) = status {
                eprintln!("Collector exited with status: {}", s);
            } else {
                eprintln!("Collector shutdown initiated");
            }
        }
        Err(e) => eprintln!("Error shutting down collector: {}", e),
    }

    drop(context.request_rx);

    // Wait for the server to shut down with timeout
    match tokio::time::timeout(std::time::Duration::from_secs(SHUTDOWN_TIMEOUT_SECONDS), context.server_handle).await {
        Ok(Ok(_)) => eprintln!("{} server shut down successfully", S::name()),
        Ok(Err(e)) => eprintln!("Error shutting down {} server: {}", S::name(), e),
        Err(_) => {
            eprintln!("Timed out waiting for {} server to shut down", S::name());
        }
    }

    // Return the result from the test logic
    result
}

/// Test a single round-trip of data through the collector
///
/// This function will:
/// 1. Create test data using the provided function
/// 2. Send data to the collector
/// 3. Wait for the data to be received
/// 4. Verify that the exported data matches what was sent
async fn run_single_round_trip_test<S: ServiceType, F>(create_request: F)
where
    S::Request: std::fmt::Debug + PartialEq,
    F: FnOnce() -> S::Request,
{
    match tokio::time::timeout(
        std::time::Duration::from_secs(TEST_TIMEOUT_SECONDS),
        run_test::<S, _, _>(|mut context| async move {
            // Create test data using the provided function
            let request = create_request();

            // Keep a copy for comparison
            let expected_request = request.clone();

            // Send data to the collector using the service type's send_data function
            S::send_data(&mut context.client, request).await?;

            // Wait for the data to be received by our test receiver
            let received_request = match context.request_rx.recv().await {
                Ok(req) => req,
                Err(e) => return (context, Err(format!("Error receiving data: {}", e).into())),
            };

            // Compare the received data with what was sent
            assert_eq!(expected_request, received_request);

            // Return the context and the result
            (context, Ok(()))
        }),
    )
        .await
    {
        Ok(result) => result.unwrap(),
        Err(_) => panic!("Test timed out after {} seconds", TEST_TIMEOUT_SECONDS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::testdata;

    // Test the trace signal
    #[tokio::test]
    async fn test_traces_single_request() {
        run_single_round_trip_test::<TracesServiceType, _>(testdata::traces::create_single_request).await;
    }

    // Test the metrics signal
    #[tokio::test]
    async fn test_metrics_single_request() {
        run_single_round_trip_test::<MetricsServiceType, _>(testdata::metrics::create_single_request).await;
    }

    // Test the logs signal
    #[tokio::test]
    async fn test_logs_single_request() {
        run_single_round_trip_test::<LogsServiceType, _>(testdata::logs::create_single_request).await;
    }
}
