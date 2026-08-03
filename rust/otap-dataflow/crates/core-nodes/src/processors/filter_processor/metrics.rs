// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP FilterProcessor node.
use otap_df_telemetry::common_attributes::SignalAttributes;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP FilterProcessor
#[metric_set(
    name = "processor.filter.pdata",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct FilterPdataMetrics {
    /// Number of signal items (log records, spans, or metric data points) a
    /// decision node chose to drop.
    #[metric(name = "dropped.items", unit = "{item}")]
    pub dropped_items: Counter<u64>,
}
