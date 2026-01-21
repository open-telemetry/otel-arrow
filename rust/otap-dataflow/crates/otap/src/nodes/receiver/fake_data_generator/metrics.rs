// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTLP Fake Signal Receiver node.

use otap_df_telemetry::instrument::Counter;
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
}
