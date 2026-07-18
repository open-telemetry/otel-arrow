// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP DebugProcessor node.

use otap_df_config::SignalType;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::{attribute_set, metric_set};

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SignalAttributes {
    pub signal: SignalType,
}

/// Pdata-oriented metrics for the OTAP DebugProcessor
#[metric_set(name = "processor.debug.pdata", measurement_attributes = SignalAttributes)]
#[derive(Debug, Default, Clone)]
pub struct DebugPdataMetrics {
    /// Number of items consumed
    #[metric(unit = "{item}")]
    pub items_consumed: Counter<u64>,
    /// Number of batches (messages) consumed
    #[metric(unit = "{msg}")]
    pub batches_consumed: Counter<u64>,
    /// Number of events (structured logs) consumed
    #[metric(unit = "{event}")]
    pub events_consumed: Counter<u64>,
    /// number of span links consumed
    #[metric(unit = "{link}")]
    pub span_links_consumed: Counter<u64>,
    /// number of span events (structured logs) consumed
    #[metric(unit = "{event}")]
    pub span_events_consumed: Counter<u64>,
    /// number of metric datapoints consumed
    #[metric(unit = "{datapoint}")]
    pub metric_datapoints_consumed: Counter<u64>,
}
