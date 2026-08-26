// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP DebugProcessor node.

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry_macros::{attribute_set, metric_set};

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SignalAttributes {
    pub signal: SignalType,
}

/// Debug-specific nested item metrics not covered by universal channel metrics.
#[metric_set(name = "processor.debug", measurement_attributes = SignalAttributes)]
#[derive(Debug, Default, Clone)]
pub struct DebugMetrics {
    /// Named log events for logs, or span events for traces
    #[metric(name = "consumed.events", unit = "{event}")]
    pub consumed_events: Counter<u64>,
    /// Span links
    #[metric(name = "consumed.links", unit = "{link}")]
    pub consumed_links: Counter<u64>,
}
