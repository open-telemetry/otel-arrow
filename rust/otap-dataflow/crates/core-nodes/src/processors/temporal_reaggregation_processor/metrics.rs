// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the temporal reaggregation processor.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the temporal reaggregation processor.
#[metric_set(name = "temporal_reaggregation.processor.pdata.metrics")]
#[derive(Debug, Default, Clone)]
pub struct TemporalReaggregationMetrics {
    /// Number of flushes triggered by the regular timer.
    #[metric(unit = "{flush}")]
    pub flushes_timer: Counter<u64>,

    /// Number of flushes triggered by exceeding the maximum stream count.
    #[metric(unit = "{flush}")]
    pub flushes_overflow: Counter<u64>,

    /// Number of incoming batches rejected because they individually exceed some
    /// specified limit or fail to be processed into a view.
    #[metric(unit = "{batch}")]
    pub batches_rejected: Counter<u64>,

    /// Number of batches passed through entirely (no aggregatable metrics).
    #[metric(unit = "{batch}")]
    pub full_passthrough_batches: Counter<u64>,

    /// Number of partial passthrough batches emitted (batch contained a mix of
    /// aggregatable and non-aggregatable metrics).
    #[metric(unit = "{batch}")]
    pub passthrough_batches: Counter<u64>,
}
