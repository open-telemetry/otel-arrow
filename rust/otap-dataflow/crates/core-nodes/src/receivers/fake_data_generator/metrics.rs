// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTLP Fake Signal Receiver node.

use otap_df_telemetry::instrument::{Counter, Gauge, Mmsc};
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP PerfExporter.
#[metric_set(name = "fake_data_generator.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct FakeSignalReceiverMetrics {
    /// Number of logs generated.
    #[metric(unit = "{log}")]
    pub logs_produced: Counter<u64>,
    /// Number of spans generated.
    #[metric(unit = "{span}")]
    pub spans_produced: Counter<u64>,
    /// Number of metrics generated.
    #[metric(unit = "{metric}")]
    pub metrics_produced: Counter<u64>,
    /// Number of smooth-mode production runs started.
    #[metric(name = "smooth.runs.started", unit = "{run}")]
    pub smooth_runs_started: Counter<u64>,
    /// Number of smooth-mode production runs that completed before the next run tick.
    #[metric(name = "smooth.runs.completed", unit = "{run}")]
    pub smooth_runs_completed: Counter<u64>,
    /// Number of smooth-mode production runs that still had work at the next run tick.
    #[metric(name = "smooth.runs.behind", unit = "{run}")]
    pub smooth_runs_behind: Counter<u64>,
    /// Number of batches remaining when smooth mode detects that a run is behind.
    #[metric(name = "smooth.behind.remaining.batches", unit = "{batch}")]
    pub smooth_behind_remaining_batches: Mmsc,
    /// Number of signal items remaining when smooth mode detects that a run is behind.
    #[metric(name = "smooth.behind.remaining.items", unit = "{item}")]
    pub smooth_behind_remaining_items: Mmsc,
    /// Smooth-mode configured batches per one-second run.
    #[metric(name = "smooth.run.batches", unit = "{batch}")]
    pub smooth_run_batches: Gauge<u64>,
    /// Smooth-mode configured interval between batches.
    #[metric(name = "smooth.batch.interval", unit = "ns")]
    pub smooth_batch_interval_ns: Gauge<u64>,
    /// Lateness of smooth-mode batch ticks relative to their scheduled instant.
    #[metric(name = "smooth.batch.tick.lateness.duration", unit = "ns")]
    pub smooth_batch_tick_lateness_duration_ns: Mmsc,
    /// Wall-clock time spent generating or cloning one smooth-mode payload.
    #[metric(name = "smooth.payload.generate.duration", unit = "ns")]
    pub smooth_payload_generate_duration_ns: Mmsc,
    /// Wall-clock time spent sending one smooth-mode payload into the downstream channel.
    #[metric(name = "smooth.payload.send.duration", unit = "ns")]
    pub smooth_payload_send_duration_ns: Mmsc,
    /// Number of smooth-mode payload send attempts rejected because the downstream channel was full.
    #[metric(name = "smooth.payload.send.full", unit = "{attempt}")]
    pub smooth_payload_send_full: Counter<u64>,
    /// Number of smooth-mode payloads retried after a previous full-channel send.
    #[metric(name = "smooth.payload.send.retry", unit = "{payload}")]
    pub smooth_payload_send_retry: Counter<u64>,
}
