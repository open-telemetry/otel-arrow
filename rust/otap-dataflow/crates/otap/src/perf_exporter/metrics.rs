// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP PerfExporter node.

use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP PerfExporter.
#[metric_set(name = "perf.exporter.pdata.metrics")]
#[derive(Debug, Default, Clone)]
pub struct PerfExporterPdataMetrics {
    /// Number of invalid pdata batches received.
    #[metric(unit = "{msg}")]
    pub invalid_batches: Counter<u64>,
    /// Number of logs received.
    #[metric(unit = "{log}")]
    pub logs: Counter<u64>,
    /// Number of spans received.
    #[metric(unit = "{span}")]
    pub spans: Counter<u64>,
    /// Number of metrics received.
    #[metric(unit = "{metric}")]
    pub metrics: Counter<u64>,
    /// Nanoseconds of user CPU time consumed by this thread since the last telemetry tick.
    #[metric(unit = "{ns}")]
    pub thread_cpu_user_ns: Counter<u64>,
    /// Nanoseconds of system CPU time consumed by this thread since the last telemetry tick.
    #[metric(unit = "{ns}")]
    pub thread_cpu_system_ns: Counter<u64>,
    /// Nanoseconds of total CPU time consumed by this thread since the last telemetry tick.
    #[metric(unit = "{ns}")]
    pub thread_cpu_total_ns: Counter<u64>,
    /// Resident set size of this thread in bytes at the last telemetry tick.
    #[metric(unit = "{byte}")]
    pub thread_rss_bytes: Gauge<u64>,
    /// Absolute change in RSS for this thread since the previous telemetry tick (bytes).
    #[metric(unit = "{byte}")]
    pub thread_rss_delta_bytes: Counter<u64>,
    /// Bytes accounted for by minor page faults for this thread since the last telemetry tick.
    #[metric(unit = "{byte}")]
    pub thread_minor_fault_bytes: Counter<u64>,
    /// Bytes accounted for by major page faults for this thread since the last telemetry tick.
    #[metric(unit = "{byte}")]
    pub thread_major_fault_bytes: Counter<u64>,
    /// Total bytes touched by page faults for this thread since the last telemetry tick.
    #[metric(unit = "{byte}")]
    pub thread_fault_bytes: Counter<u64>,
    /// Bytes allocated by this thread since the last telemetry tick (jemalloc).
    #[metric(unit = "{byte}")]
    pub thread_alloc_bytes: Counter<u64>,
    /// Bytes deallocated by this thread since the last telemetry tick (jemalloc).
    #[metric(unit = "{byte}")]
    pub thread_dealloc_bytes: Counter<u64>,
}
