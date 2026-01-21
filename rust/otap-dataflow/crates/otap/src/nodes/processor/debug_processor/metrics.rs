// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP DebugProcessor node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP DebugProcessor
#[metric_set(name = "debug.processor.pdata.metrics")]
#[derive(Debug, Default, Clone)]
pub struct DebugPdataMetrics {
    /// Number of log signals consumed
    #[metric(unit = "{log}")]
    pub log_signals_consumed: Counter<u64>,
    /// Number of events (structured logs) consumed
    #[metric(unit = "{event}")]
    pub events_consumed: Counter<u64>,
    /// Number of span signals consumed
    #[metric(unit = "{span}")]
    pub span_signals_consumed: Counter<u64>,
    /// number of span links consumed
    #[metric(unit = "{link}")]
    pub span_links_consumed: Counter<u64>,
    /// number of span events (structured logs) consumed
    #[metric(unit = "{event}")]
    pub span_events_consumed: Counter<u64>,
    /// Number of metrics consumed
    #[metric(unit = "{metric}")]
    pub metric_signals_consumed: Counter<u64>,
    /// number of metric datapoints consumed
    #[metric(unit = "{datapoint}")]
    pub metric_datapoints_consumed: Counter<u64>,
    /// number of metrics (batches) consumed
    #[metric(unit = "{msg}")]
    pub metrics_consumed: Counter<u64>,
    /// number of logs (batches) consumed
    #[metric(unit = "{msg}")]
    pub logs_consumed: Counter<u64>,
    /// number of traces (batches) consumed
    #[metric(unit = "{msg}")]
    pub traces_consumed: Counter<u64>,
}
