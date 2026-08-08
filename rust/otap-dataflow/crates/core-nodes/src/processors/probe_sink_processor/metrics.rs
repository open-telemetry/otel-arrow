// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the probe-sink processor.

use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry_macros::metric_set;

/// Metrics for the probe-sink processor.
///
/// `pipeline_e2e_latency_ns` is a Min/Mean/Max/Sum-Count histogram-equivalent
/// over the per-probe latency observed at this sink. Pair it with
/// `probes_observed` to derive averages and rates.
#[metric_set(name = "processor.probe_sink")]
#[derive(Debug, Default, Clone)]
pub struct ProbeSinkMetrics {
    /// Total number of probe records observed at this sink.
    #[metric(unit = "{probe}")]
    pub probes_observed: Counter<u64>,
    /// Number of probe records dropped (forwarded only when `drop_probes` = false).
    #[metric(unit = "{probe}")]
    pub probes_dropped: Counter<u64>,
    /// Number of incoming log records that were not probes (passed through).
    #[metric(unit = "{log}")]
    pub non_probe_logs_forwarded: Counter<u64>,
    /// Number of probe records that were ill-formed (missing emitted_at
    /// attribute or wrong type).
    #[metric(unit = "{probe}")]
    pub probes_invalid: Counter<u64>,
    /// End-to-end latency of a probe, in nanoseconds, measured between the
    /// emitter's wall-clock stamp and the sink's wall-clock observation.
    #[metric(name = "pipeline.e2e.latency", unit = "ns")]
    pub pipeline_e2e_latency_ns: Mmsc,
}
