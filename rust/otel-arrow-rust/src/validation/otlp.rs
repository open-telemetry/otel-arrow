// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use crate::validation::testdata;
    use crate::validation::service_type::{LogsServiceType, MetricsServiceType, TracesServiceType};
    use crate::validation::scenarios::run_single_round_trip_test;

    #[tokio::test]
    async fn test_traces_single_request() {
        run_single_round_trip_test::<TracesServiceType, _>(testdata::traces::create_single_request)
            .await;
    }

    #[tokio::test]
    async fn test_metrics_single_request() {
        run_single_round_trip_test::<MetricsServiceType, _>(
            testdata::metrics::create_single_request,
        )
        .await;
    }

    #[tokio::test]
    async fn test_logs_single_request() {
        run_single_round_trip_test::<LogsServiceType, _>(testdata::logs::create_single_request)
            .await;
    }
}
