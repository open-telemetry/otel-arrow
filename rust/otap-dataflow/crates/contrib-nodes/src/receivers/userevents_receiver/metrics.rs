// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Linux userevents receiver.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Internal telemetry for the Linux userevents receiver.
#[metric_set(name = "userevents.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub(super) struct UsereventsReceiverMetrics {
    /// Number of perf samples received from the kernel ring.
    #[metric(unit = "{item}")]
    pub received_samples: Counter<u64>,
    /// Number of perf samples forwarded downstream.
    #[metric(unit = "{item}")]
    pub forwarded_samples: Counter<u64>,
    /// Number of samples dropped because the downstream channel was full.
    #[metric(unit = "{item}")]
    pub dropped_downstream_full: Counter<u64>,
    /// Number of samples dropped due to process-wide memory pressure.
    #[metric(unit = "{item}")]
    pub dropped_memory_pressure: Counter<u64>,
    /// Number of samples dropped because no matching subscription was found.
    #[metric(unit = "{item}")]
    pub dropped_no_subscription: Counter<u64>,
    /// Number of lost samples reported by the perf ring.
    #[metric(unit = "{item}")]
    pub lost_perf_samples: Counter<u64>,
    /// Number of late-registration retries attempted while waiting for tracepoints.
    #[metric(unit = "{event}")]
    pub late_registration_retries: Counter<u64>,
    /// Number of receiver sessions successfully started.
    #[metric(unit = "{event}")]
    pub sessions_started: Counter<u64>,
    /// Number of Arrow batches flushed downstream.
    #[metric(unit = "{event}")]
    pub flushed_batches: Counter<u64>,
}
