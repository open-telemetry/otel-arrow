// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Self-metrics for the Windows performance-counter receiver.

use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Bounded operation on a Windows performance counter that can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum CounterOperation {
    /// Collecting query sample failed.
    Collect,
    /// Reading counter raw or formatted value failed.
    Read,
}

/// Attributes for counter operation failure metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct CounterOperationFailureAttributes {
    /// Operation that failed.
    #[attribute_key = "operation"]
    pub(super) operation: CounterOperation,
}

/// Counter operation failure metrics for one receiver node.
#[metric_set(
    name = "receiver.windows_perf_counters",
    measurement_attributes = CounterOperationFailureAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct CounterOperationFailureMetrics {
    /// Failed counter operations.
    #[metric(unit = "{failure}")]
    pub(super) counter_failures: Counter<u64>,
}

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
