// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP DebugProcessor node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP DebugProcessor
#[metric_set(name = "debug.processor.pdata.metrics")]
#[derive(Debug, Default, Clone)]
pub struct DebugPdataMetrics {
    /// Number of logs seen
    #[metric(unit = "{log}")]
    pub logs: Counter<u64>,
    /// Number of events (structured logs) seen
    #[metric(unit = "{event}")]
    pub events: Counter<u64>,
    /// Number of spans seen
    #[metric(unit = "{span}")]
    pub spans: Counter<u64>,
    /// number of span links seen
    #[metric(unit = "{link}")]
    pub span_links: Counter<u64>,
    /// number of span events (structured logs) seen
    #[metric(unit = "{event}")]
    pub span_events: Counter<u64>,
    /// Number of metrics seen
    #[metric(unit = "{metric}")]
    pub metrics: Counter<u64>,
    /// number of metric datapoints seen
    #[metric(unit = "{datapoint}")]
    pub metric_datapoints: Counter<u64>,
}
