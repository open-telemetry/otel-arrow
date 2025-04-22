// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::collector::{run_test, TEST_TIMEOUT_SECONDS};
use super::service_type::{ServiceInputType, ServiceOutputType};

/// Test a single round-trip of data through the collector
///
/// This function will:
/// 1. Create test data using the provided function
/// 2. Send data to the collector
/// 3. Wait for the data to be received
/// 4. Verify that the exported data matches what was sent
pub async fn run_single_round_trip_test<I, O, F>(create_request: F)
where
    I: ServiceInputType,
    O: ServiceOutputType,
    I::Request: std::fmt::Debug + PartialEq + From<O::Request>,
    O::Request: std::fmt::Debug + PartialEq + From<I::Request>,
    F: FnOnce() -> I::Request,
{
    match tokio::time::timeout(
        std::time::Duration::from_secs(TEST_TIMEOUT_SECONDS),
        run_test::<I, O, _, _>(|mut context| async move {
            // Create test data using the provided function
            let request = create_request();

            // Keep a copy for comparison
            let expected_request = request.clone();

            // Send data to the collector using the service type's send_data function
            match I::send_data(&mut context.client, request).await {
                Ok(_) => {}
                Err(e) => return (context, Err(format!("Error sending data: {}", e).into())),
            };

            // Wait for the data to be received by our test receiver
            let received_request = match context.request_rx.recv().await {
                Ok(req) => req,
                Err(e) => return (context, Err(format!("Error receiving data: {}", e).into())),
            };

            // Compare the received data with what was sent
            // We need to convert the expected request to the output type for comparison
            let expected_output_request = O::Request::from(expected_request);
            assert_eq!(expected_output_request, received_request);

            // Return the context and the result
            (context, Ok(()))
        }),
    )
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => panic!("Test error {}", err),
        Err(err) => panic!(
            "Test timed out after {} seconds: {}",
            TEST_TIMEOUT_SECONDS, err
        ),
    }
}
