// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Self-metrics for the Windows performance-counter receiver.

use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge, HistogramNormal};
use otel_arrow_dfe_telemetry_macros::metric_set;

/// Scrape and counter metrics for one receiver node.
#[metric_set(name = "receiver.windows_perf_counters")]
#[derive(Debug, Default, Clone)]
pub(super) struct WindowsPerfCountersMetrics {
    /// Scrapes in which at least one counter's query collected.
    #[metric(unit = "{scrape}")]
    pub scrapes: Counter<u64>,
    /// Scrapes in which at least one counter's query failed to open or collect.
    #[metric(unit = "{scrape}")]
    pub scrape_failures: Counter<u64>,
    /// Ticks skipped because the previous scrape was still running.
    #[metric(unit = "{tick}")]
    pub scrape_overruns: Counter<u64>,
    /// Time from scrape request to report.
    #[metric(unit = "s")]
    pub scrape_duration: HistogramNormal,
    /// Failed counter adds, including retries.
    #[metric(unit = "{failure}")]
    pub counter_add_failures: Counter<u64>,
    /// Counters currently added to a query.
    #[metric(unit = "{counter}")]
    pub active_counters: Gauge<u64>,
    /// Batches dropped because downstream was full or closed.
    #[metric(unit = "{batch}")]
    pub batches_dropped: Counter<u64>,
}
