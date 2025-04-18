// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::collector::{run_test, TEST_TIMEOUT_SECONDS};
use super::service_type::{LogsServiceType, MetricsServiceType, ServiceType, TracesServiceType};

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
            match S::send_data(&mut context.client, request).await {
                Ok(_) => {}
                Err(e) => return (context, Err(format!("Error sending data: {}", e).into())),
            };

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
        run_single_round_trip_test::<TracesServiceType, _>(testdata::traces::create_single_request)
            .await;
    }

    // Test the metrics signal
    #[tokio::test]
    async fn test_metrics_single_request() {
        run_single_round_trip_test::<MetricsServiceType, _>(
            testdata::metrics::create_single_request,
        )
        .await;
    }

    // Test the logs signal
    #[tokio::test]
    async fn test_logs_single_request() {
        run_single_round_trip_test::<LogsServiceType, _>(testdata::logs::create_single_request)
            .await;
    }
}
