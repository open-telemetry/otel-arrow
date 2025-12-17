// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the TransformProcessor node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the TransformProcessor node.
#[metric_set(name = "transform.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    /// Telemetry items (log records, spans, or metrics) received by this processor.
    #[metric(unit = "{item}")]
    pub items_received: Counter<u64>,

    /// Telemetry items sent to the next processor in the pipeline.
    #[metric(unit = "{item}")]
    pub items_sent: Counter<u64>,

    /// Telemetry items successfully transformed.
    #[metric(unit = "{item}")]
    pub items_transformed: Counter<u64>,

    /// Telemetry items that failed during transformation.
    #[metric(unit = "{item}")]
    pub items_failed: Counter<u64>,
}
