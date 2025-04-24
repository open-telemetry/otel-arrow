// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::collector::{run_test, TEST_TIMEOUT_SECONDS, RECEIVER_TIMEOUT_SECONDS};
use super::service_type::{ServiceInputType, ServiceOutputType};

use super::error;
use snafu::ResultExt;

/// Test a single round-trip of data through the collector
///
/// This function will:
/// 1. Create test data using the provided function
/// 2. Send data to the collector
/// 3. Wait for the data to be received
/// 4. Verify that the exported data matches what was sent or that
///    the expected error message occurs.
pub async fn run_single_round_trip_test<I, O, F>(create_request: F, expected_error: Option<&'static str>)
where
    I: ServiceInputType,
    O: ServiceOutputType,
    I::Request: std::fmt::Debug + PartialEq + From<O::Request>,
    O::Request: std::fmt::Debug + PartialEq + From<I::Request>,
    I::Response: std::fmt::Debug + PartialEq + Default,
    F: FnOnce() -> I::Request + 'static,
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
    expected_error: Option<&'static str>,
) -> error::Result<()>
where
    I: ServiceInputType,
    O: ServiceOutputType,
    I::Request: std::fmt::Debug + PartialEq + From<O::Request>,
    O::Request: std::fmt::Debug + PartialEq + From<I::Request>,
    I::Response: std::fmt::Debug + PartialEq + Default,
    F: FnOnce() -> I::Request + 'static,
{
    tokio::time::timeout(
        std::time::Duration::from_secs(TEST_TIMEOUT_SECONDS),
        run_test::<I, O, _>(|context| Box::pin(async move {
            // Create test data, make a copy
            let request = create_request();
            let expected_request = request.clone();

            // Note: We do not test the response value.  There is
	    // a point of confusion. I would expect this to work:
	    //
	    //   assert_eq!(resp, I::Response::default());
	    //
	    // but it doesn't because of an empty PartialSuccess on
	    // one side, None on the other.
            return match I::send_data(&mut context.client, request).await {
                Ok(_) => {
		    // The data should have been received already.
		    let timeout_duration = std::time::Duration::from_secs(RECEIVER_TIMEOUT_SECONDS);
		    let received_request =
			tokio::time::timeout(timeout_duration, context.request_rx.recv())
			.await
			.context(error::ReceiverTimeoutSnafu)?
			.ok_or(error::Error::NoResponse{})?;

		    // Compare the received data with what was sent.
		    let expected_output_request = O::Request::from(expected_request);
		    assert_eq!(expected_output_request, received_request);
		    
		    Ok(())
                }
                Err(status) => {
                    if let Some(expected_msg) = expected_error {
			// Expected, check for a match.
                        status.to_string()
                            .contains(expected_msg)
                            .then_some(())
                            .ok_or_else(|| error::Error::PatternNotFound {
                                pattern: expected_msg.into(),
                                input: status.to_string().into(),
                            })
                    } else {
                        // Unexpected, fail.
                        Err(status)
                    }
                }
            };
        })),
    )
    .await
    .context(error::TestTimeoutSnafu)?
}
