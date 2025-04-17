// OTLP validation framework for round-trip testing.
//

// This module provides facilities for testing round-trip fidelity of telemetry data
// through a Golang OTel Collector. It can create test data, send it to a collector,
// receive the exported data, and verify that the data matches what was sent.

mod collector_test;
mod otlp_validation;
mod service_type;

// Re-export the validation test functions for use in tests
pub use otlp_validation::{test_otlp_round_trip, test_signal_round_trip};
pub use service_type::{LogsServiceType, MetricsServiceType, ServiceType, TracesServiceType};

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to run a test with the given service type
    async fn run_test_with_service<S: ServiceType>(test_name: &str)
    where
        S::Request: std::fmt::Debug + PartialEq,
    {
        // Only run this test if the collector path is set
        let env = std::env::var("OTEL_COLLECTOR_PATH").unwrap_or("../../bin/otelarrowcol".to_string());

        match tokio::time::timeout(
            std::time::Duration::from_secs(20), // Increased timeout for testing all signals
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
    async fn test_traces_fidelity() {
        run_test_with_service::<TracesServiceType>("test_traces").await;
    }

    // Test the metrics signal
    #[tokio::test]
    async fn test_metrics_fidelity() {
        run_test_with_service::<MetricsServiceType>("test_metrics").await;
    }

    // Test the logs signal
    #[tokio::test]
    async fn test_logs_fidelity() {
        run_test_with_service::<LogsServiceType>("test_logs").await;
    }

    // Backward compatibility test using the old function
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
