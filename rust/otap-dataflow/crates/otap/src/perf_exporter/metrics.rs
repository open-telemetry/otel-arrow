// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP PerfExporter node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP PerfExporter.
#[metric_set(name = "perf.exporter.pdata.metrics")]
#[derive(Debug, Default, Clone)]
pub struct PerfExporterPdataMetrics {
    /// Number of invalid pdata batches received.
    #[metric(unit = "{msg}")]
    pub invalid_batches: Counter<u64>,
    /// Number of Arrow records received.
    #[metric(unit = "{record}")]
    pub arrow_records: Counter<u64>,
    /// Number of logs received.
    #[metric(unit = "{log}")]
    pub logs: Counter<u64>,
    /// Number of spans received.
    #[metric(unit = "{span}")]
    pub spans: Counter<u64>,
    /// Number of metrics received.
    #[metric(unit = "{metric}")]
    pub metrics: Counter<u64>,
}
