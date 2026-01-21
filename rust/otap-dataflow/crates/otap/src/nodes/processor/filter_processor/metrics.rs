// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP FilterProcessor node.
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP FilterProcessor
#[metric_set(name = "filter.processor.pdata.metrics")]
#[derive(Debug, Default, Clone)]
pub struct FilterPdataMetrics {
    /// Number of log signals consumed
    #[metric(unit = "{log}")]
    pub log_signals_consumed: Counter<u64>,
    /// Number of span signals consumed
    #[metric(unit = "{span}")]
    pub span_signals_consumed: Counter<u64>,

    /// Number of log signals filtered
    #[metric(unit = "{log}")]
    pub log_signals_filtered: Counter<u64>,
    /// Number of span signals filtered
    #[metric(unit = "{span}")]
    pub span_signals_filtered: Counter<u64>,
}
