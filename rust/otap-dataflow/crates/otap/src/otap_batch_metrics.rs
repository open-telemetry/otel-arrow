// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP batch processor.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Minimal, essential metrics for the OTAP batch processor.
///
/// Naming is flat to minimize descriptor size and overhead.
#[metric_set(name = "otap.processor.batch")]
#[derive(Debug, Default, Clone)]
pub struct OtapBatchProcessorMetrics {
    // Volume per signal (rows)
    #[metric(unit = "{row}")]
    pub received_rows_logs: Counter<u64>,
    #[metric(unit = "{row}")]
    pub received_rows_metrics: Counter<u64>,
    #[metric(unit = "{row}")]
    pub received_rows_traces: Counter<u64>,

    // Flush reasons (aggregated across signals)
    #[metric(unit = "{flush}")]
    pub flushes_size: Counter<u64>,
    #[metric(unit = "{flush}")]
    pub flushes_timer: Counter<u64>,

    // Errors/drops (aggregated)
    #[metric(unit = "{msg}")]
    pub dropped_conversion: Counter<u64>,
    #[metric(unit = "{error}")]
    pub batching_errors: Counter<u64>,
    #[metric(unit = "{msg}")]
    pub dropped_empty_records: Counter<u64>,

    // Splitting (aggregated)
    #[metric(unit = "{event}")]
    pub split_requests: Counter<u64>,
}
