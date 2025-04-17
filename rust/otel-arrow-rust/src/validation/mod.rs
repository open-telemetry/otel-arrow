// OTLP validation framework for round-trip testing.
//
// This module provides facilities for testing round-trip fidelity of telemetry data
// through a Golang OTel Collector. It can create test data, send it to a collector,
// receive the exported data, and verify that the data matches what was sent.

mod collector_test;
mod otlp_validation;

// Re-export the validation test function for use in tests
pub use otlp_validation::test_otlp_round_trip;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_otlp_fidelity() {
        // Only run this test if the collector path is set
        let env = std::env::var("OTEL_COLLECTOR_PATH").unwrap_or("../../bin/otelarrowcol".to_string());

        match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            test_otlp_round_trip(env),
        )
        .await
        {
            Ok(result) => result.unwrap(),
            Err(_) => panic!("Test timed out after 10 seconds"),
        }
    }
}
