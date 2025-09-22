// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP batch processor.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Minimal, essential metrics for the OTAP batch processor, plus observability for
/// dirty-flag behavior and timer flush decisions.
#[metric_set(name = "otap.processor.batch")]
#[derive(Debug, Default, Clone)]
pub struct OtapBatchProcessorMetrics {
    /// Total items consumed for logs signal
    #[metric(unit = "{item}")]
    pub consumed_items_logs: Counter<u64>,
    /// Total items consumed for metrics signal
    #[metric(unit = "{item}")]
    pub consumed_items_metrics: Counter<u64>,
    /// Total items consumed for traces signal
    #[metric(unit = "{item}")]
    pub consumed_items_traces: Counter<u64>,

    /// Number of flushes triggered by size threshold (all signals)
    #[metric(unit = "{flush}")]
    pub flushes_size: Counter<u64>,
    /// Number of flushes triggered by timer (all signals)
    #[metric(unit = "{flush}")]
    pub flushes_timer: Counter<u64>,
    /// Number of flushes triggered by shutdown (all signals)
    #[metric(unit = "{flush}")]
    pub flushes_shutdown: Counter<u64>,

    /// Number of messages dropped due to conversion failures
    #[metric(unit = "{msg}")]
    pub dropped_conversion: Counter<u64>,
    /// Number of batching errors encountered
    #[metric(unit = "{error}")]
    pub batching_errors: Counter<u64>,
    /// Number of empty records dropped
    #[metric(unit = "{msg}")]
    pub dropped_empty_records: Counter<u64>,

    /// Number of split requests issued to upstream batching
    #[metric(unit = "{event}")]
    pub split_requests: Counter<u64>,

    /// Dirty flag set events for logs
    #[metric(unit = "{event}")]
    pub dirty_set_logs: Counter<u64>,
    /// Dirty flag set events for metrics
    #[metric(unit = "{event}")]
    pub dirty_set_metrics: Counter<u64>,
    /// Dirty flag set events for traces
    #[metric(unit = "{event}")]
    pub dirty_set_traces: Counter<u64>,

    /// Dirty flag cleared events for logs
    #[metric(unit = "{event}")]
    pub dirty_cleared_logs: Counter<u64>,
    /// Dirty flag cleared events for metrics
    #[metric(unit = "{event}")]
    pub dirty_cleared_metrics: Counter<u64>,
    /// Dirty flag cleared events for traces
    #[metric(unit = "{event}")]
    pub dirty_cleared_traces: Counter<u64>,

    /// Timer-triggered flushes that were performed for logs
    #[metric(unit = "{flush}")]
    pub timer_flush_performed_logs: Counter<u64>,
    /// Timer-triggered flushes that were performed for metrics
    #[metric(unit = "{flush}")]
    pub timer_flush_performed_metrics: Counter<u64>,
    /// Timer-triggered flushes that were performed for traces
    #[metric(unit = "{flush}")]
    pub timer_flush_performed_traces: Counter<u64>,

    /// Timer-triggered flushes that were skipped for logs (not dirty)
    #[metric(unit = "{flush}")]
    pub timer_flush_skipped_logs: Counter<u64>,
    /// Timer-triggered flushes that were skipped for metrics (not dirty)
    #[metric(unit = "{flush}")]
    pub timer_flush_skipped_metrics: Counter<u64>,
    /// Timer-triggered flushes that were skipped for traces (not dirty)
    #[metric(unit = "{flush}")]
    pub timer_flush_skipped_traces: Counter<u64>,
}
