// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::collector::{run_test, TEST_TIMEOUT_SECONDS};
use super::service_type::{ServiceInputType, ServiceOutputType};

use super::error;
use snafu::ResultExt;

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
    I::Response: std::fmt::Debug + PartialEq + Default,
    F: FnOnce() -> I::Request,
{
    match run_single_round_trip::<I, O, F>(create_request, expected_error).await {
        Ok(_) => {}
        Err(err) => {
            panic!("Test failed: {:?}", err);
        }
    }
}

async fn run_single_round_trip<I, O, F>(
    create_request: F,
    expected_error: Option<&str>,
) -> error::Result<()>
where
    I: ServiceInputType,
    O: ServiceOutputType,
    I::Request: std::fmt::Debug + PartialEq + From<O::Request>,
    O::Request: std::fmt::Debug + PartialEq + From<I::Request>,
    I::Response: std::fmt::Debug + PartialEq + Default,
    F: FnOnce() -> I::Request,
{
    tokio::time::timeout(
        std::time::Duration::from_secs(TEST_TIMEOUT_SECONDS),
        run_test::<I, O, _, _>(|mut context| async move {
            // Create test data
            let request = create_request();

            // Keep a copy for comparison
            let expected_request = request.clone();

            // Send data to the collector
            match I::send_data(&mut context.client, request).await {
                Ok(_) => {
                    // Note: We do not test the response value.
                }
                Err(status) => {
                    if let Some(expected_msg) = expected_error {
                        let err_str = status.to_string();
                        // If we expect an error, check for ptatern
                        if !err_str.contains(expected_msg) {
                            // one with the expected message.
                            return (
                                context,
                                Err(error::Error::PatternNotFound {
                                    pattern: expected_msg.into(),
                                    input: err_str.into(),
                                }),
                            );
                        }
                        // Nothing more to test
                        return (context, Ok(()));
                    } else {
                        // Unexpected, fail.
                        return (context, Err(status));
                    }
                }
            };

            // The data data should have been received already
            let received_request = match context.request_rx.recv().await {
                Ok(req) => req,
                Err(e) => {
                    return (context, Err(e));
                }
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
    .context(error::TestTimeoutSnafu)?
}
