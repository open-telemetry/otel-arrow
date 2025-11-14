// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for common.rs optional resource and scope handling.
//!
//! This module tests the round-trip encoding/decoding of OTLP data
//! with optional resource, scope, and attributes fields. The changes
//! in common.rs make ResourceArrays::id and ScopeArrays fields optional,
//! allowing OTAP batches to omit these columns when not present.

#[cfg(test)]
mod tests {
    use crate::testing::fixtures::*;
    use crate::testing::round_trip::*;

    // ============================================================================
    // Logs Tests
    // ============================================================================

    #[test]
    fn test_logs_with_full_resource_and_scope() {
        test_logs_round_trip(logs_with_full_resource_and_scope());
    }

    #[test]
    fn test_logs_with_no_resource() {
        test_logs_round_trip(logs_with_no_resource());
    }

    #[test]
    fn test_logs_with_no_scope() {
        test_logs_round_trip(logs_with_no_scope());
    }

    #[test]
    fn test_logs_with_no_attributes() {
        test_logs_round_trip(logs_with_no_attributes());
    }

    #[test]
    fn test_logs_with_no_resource_no_scope() {
        test_logs_round_trip(logs_with_no_resource_no_scope());
    }

    #[test]
    fn test_logs_multiple_records_no_resource() {
        test_logs_round_trip(logs_multiple_records_no_resource());
    }

    #[test]
    fn test_logs_multiple_scopes_no_resource() {
        test_logs_round_trip(logs_multiple_scopes_no_resource());
    }

    #[test]
    fn test_logs_multiple_resources_mixed_content() {
        test_logs_round_trip(logs_multiple_resources_mixed_content());
    }

    #[test]
    fn test_logs_with_empty_log_records() {
        test_logs_round_trip(logs_with_empty_log_records());
    }

    // ============================================================================
    // Traces Tests
    // ============================================================================

    #[test]
    fn test_traces_with_full_resource_and_scope() {
        test_traces_round_trip(traces_with_full_resource_and_scope());
    }

    #[test]
    fn test_traces_with_no_resource() {
        test_traces_round_trip(traces_with_no_resource());
    }

    #[test]
    fn test_traces_with_no_scope() {
        test_traces_round_trip(traces_with_no_scope());
    }

    #[test]
    fn test_traces_with_no_attributes() {
        test_traces_round_trip(traces_with_no_attributes());
    }

    #[test]
    fn test_traces_with_no_resource_no_scope() {
        test_traces_round_trip(traces_with_no_resource_no_scope());
    }

    #[test]
    fn test_traces_multiple_spans_no_resource() {
        test_traces_round_trip(traces_multiple_spans_no_resource());
    }

    #[test]
    fn test_traces_multiple_scopes_no_resource() {
        test_traces_round_trip(traces_multiple_scopes_no_resource());
    }

    #[test]
    fn test_traces_multiple_resources_mixed_content() {
        test_traces_round_trip(traces_multiple_resources_mixed_content());
    }

    // ============================================================================
    // Metrics Tests
    // ============================================================================

    #[test]
    fn test_metrics_sum_with_full_resource_and_scope() {
        test_metrics_round_trip(metrics_sum_with_full_resource_and_scope());
    }

    #[test]
    fn test_metrics_sum_with_no_resource() {
        test_metrics_round_trip(metrics_sum_with_no_resource());
    }

    #[test]
    fn test_metrics_sum_with_no_scope() {
        test_metrics_round_trip(metrics_sum_with_no_scope());
    }

    #[test]
    fn test_metrics_sum_with_no_resource_no_scope() {
        test_metrics_round_trip(metrics_sum_with_no_resource_no_scope());
    }

    #[test]
    fn test_metrics_sum_with_no_data_points() {
        test_metrics_round_trip(metrics_sum_with_no_data_points());
    }

    #[test]
    fn test_empty_logs() {
        let encoded = encode_logs(&empty_logs());
        assert_eq!(
            encoded.batch_length(),
            0,
            "Empty logs should have batch_length == 0"
        );
    }

    #[test]
    fn test_logs_with_empty_scope_logs() {
        let encoded = encode_logs(&logs_with_empty_scope_logs());
        assert_eq!(
            encoded.batch_length(),
            0,
            "Logs with no records should have batch_length == 0"
        );
    }

    #[test]
    fn test_empty_traces() {
        let encoded = encode_traces(&empty_traces());
        assert_eq!(
            encoded.batch_length(),
            0,
            "Empty traces should have batch_length == 0"
        );
    }

    #[test]
    fn test_traces_with_empty_scope_spans() {
        let encoded = encode_traces(&traces_with_empty_scope_spans());
        assert_eq!(
            encoded.batch_length(),
            0,
            "Traces with no spans should have batch_length == 0"
        );
    }

    #[test]
    fn test_traces_with_empty_spans() {
        let encoded = encode_traces(&traces_with_empty_spans());
        assert_eq!(
            encoded.batch_length(),
            0,
            "Traces with no spans should have batch_length == 0"
        );
    }

    #[test]
    fn test_empty_metrics() {
        let encoded = encode_metrics(&empty_metrics());
        assert_eq!(
            encoded.batch_length(),
            0,
            "Empty metrics should have batch_length == 0"
        );
    }

    #[test]
    fn test_metrics_with_no_scope_metrics() {
        let encoded = encode_metrics(&metrics_with_no_scope_metrics());
        assert_eq!(
            encoded.batch_length(),
            0,
            "Metrics with no scope metrics should have batch_length == 0"
        );
    }

    #[test]
    fn test_metrics_with_no_metrics() {
        let encoded = encode_metrics(&metrics_with_no_metrics());
        assert_eq!(
            encoded.batch_length(),
            0,
            "Metrics with no metrics should have batch_length == 0"
        );
    }

    #[test]
    fn test_metrics_multiple_sums_no_resource() {
        test_metrics_round_trip(metrics_multiple_sums_no_resource());
    }
}
