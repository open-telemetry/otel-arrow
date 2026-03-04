// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-processor metrics for the OTAP engine.
//!
//! Captures wall-clock duration of each `process()` call, giving operators
//! visibility into per-processor compute cost without requiring individual
//! processors to self-instrument.

use otap_df_telemetry::instrument::Mmsc;
use otap_df_telemetry_macros::metric_set;

/// Metrics automatically recorded by the engine for every processor node.
#[metric_set(name = "node.processor")]
#[derive(Debug, Default, Clone)]
pub struct ProcessorMetrics {
    /// Wall-clock duration of each `process()` invocation, in nanoseconds.
    #[metric(name = "process.duration", unit = "ns")]
    pub process_duration_ns: Mmsc,
}
