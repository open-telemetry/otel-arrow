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
pub async fn run_single_round_trip_test<I, O, F>(create_request: F, expected_error: Option<&str>)
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
            let send_result = I::send_data(&mut context.client, request).await;
            
            // If we expect an error, check if we got one with the expected message
            if let Some(expected_err_msg) = expected_error {
                match send_result {
                    Ok(_) => return (context, Err(format!("Expected error containing '{}' but operation succeeded", expected_err_msg).into())),
                    Err(e) => {
                        let err_str = e.to_string();
                        if !err_str.contains(expected_err_msg) {
                            return (context, Err(format!("Expected error containing '{}' but got: {}", expected_err_msg, err_str).into()));
                        }
                        // We got the expected error, test passes
                        return (context, Ok(()));
                    }
                }
            } else {
                // We expect success, so any error is a failure
                if let Err(e) = send_result {
                    return (context, Err(format!("Error sending data: {}", e).into()));
                }
            }

            // Only proceed with verification if we don't expect an error
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
