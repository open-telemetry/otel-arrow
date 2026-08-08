// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the probe-emitter receiver.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the probe-emitter receiver.
#[metric_set(name = "receiver.probe_emitter")]
#[derive(Debug, Default, Clone)]
pub struct ProbeEmitterMetrics {
    /// Number of probe records emitted into the pipeline.
    #[metric(unit = "{probe}")]
    pub probes_emitted: Counter<u64>,
    /// Number of probes that failed to send (channel full / closed).
    #[metric(unit = "{probe}")]
    pub probes_send_failed: Counter<u64>,
}
