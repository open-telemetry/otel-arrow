// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Self-metrics for the Windows performance-counter receiver.

use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge};
use otel_arrow_dfe_telemetry_macros::metric_set;

/// Receiver-specific diagnostics for one receiver node.
#[metric_set(name = "receiver.windows_perf_counters")]
#[derive(Debug, Default, Clone)]
pub(super) struct WindowsPerfCountersMetrics {
    /// Ticks skipped because the previous scrape was still running.
    #[metric(unit = "{tick}")]
    pub scrape_overruns: Counter<u64>,

    /// Failed counter adds, including retries.
    #[metric(unit = "{failure}")]
    pub counter_add_failures: Counter<u64>,

    /// Counters currently added, whether or not they produce values.
    #[metric(unit = "{counter}")]
    pub active_counters: Gauge<u64>,

    /// Batches dropped because downstream was full or closed.
    #[metric(unit = "{batch}")]
    pub batches_dropped: Counter<u64>,
}
