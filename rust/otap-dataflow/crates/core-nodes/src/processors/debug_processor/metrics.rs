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
    /// Incoming OTLP requests handled by the debug processor
    #[metric(name = "consumed.requests", unit = "{request}")]
    pub consumed_requests: Counter<u64>,
    /// Primary signal items: log records, metric datapoints, or spans
    #[metric(name = "consumed.items", unit = "{item}")]
    pub consumed_items: Counter<u64>,
    /// Named log events for logs, or span events for traces
    #[metric(name = "consumed.events", unit = "{event}")]
    pub consumed_events: Counter<u64>,
    /// Span links
    #[metric(name = "consumed.links", unit = "{link}")]
    pub consumed_links: Counter<u64>,
}
