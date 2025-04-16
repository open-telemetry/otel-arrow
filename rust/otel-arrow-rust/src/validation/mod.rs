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
        if std::env::var("OTEL_COLLECTOR_PATH").is_ok() {
            if let Err(e) = test_otlp_round_trip().await {
                panic!("Round-trip test failed: {}", e);
            }
        } else {
            println!("Skipping round-trip test: OTEL_COLLECTOR_PATH not set");
        }
    }
}
