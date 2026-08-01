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
    /// Number of signals consumed
    #[metric(unit = "{signal}")]
    pub signals_consumed: Counter<u64>,

    /// Number of signals filtered
    #[metric(unit = "{signal}")]
    pub signals_filtered: Counter<u64>,
}
