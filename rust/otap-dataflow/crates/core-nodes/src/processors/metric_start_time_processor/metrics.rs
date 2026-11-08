// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry metrics for the OTAP MetricStartTimeProcessor node.

use otel_arrow_dfe_telemetry::common_attributes::SignalAttributes;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry_macros::metric_set;

/// Telemetry metrics recorded by the metric start time processor.
#[metric_set(
    name = "processor.metric_start_time",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct MetricStartTimeMetrics {
    /// Number of initial cumulative data points dropped while seeding per-series reference state.
    #[metric(name = "dropped.items", unit = "{item}")]
    pub dropped_items: Counter<u64>,

    /// Number of cumulative data points whose start time and value were adjusted.
    #[metric(name = "adjusted.data_points", unit = "{datapoint}")]
    pub adjusted_data_points: Counter<u64>,

    /// Number of cumulative counter, summary, or histogram resets detected.
    #[metric(name = "resets.detected", unit = "{reset}")]
    pub resets_detected: Counter<u64>,
}
