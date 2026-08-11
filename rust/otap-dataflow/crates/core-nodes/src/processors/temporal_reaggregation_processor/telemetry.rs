// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry definitions for the temporal reaggregation processor.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Emitted when creating a view fails so we cannot process the data
pub const VIEW_CREATION_FAILED_EVENT: &str = "temporal_reaggregation.view.creation_failed";

/// Emitted when encoding one or more attributes fails. This is mostly a concern for CBOR
/// encoded data.
pub const ATTRIBUTE_ENCODE_FAILED_EVENT: &str = "temporal_reaggregation.attribute.encode_failed";

/// Emitted when calldata returned to this processor is invalid in some way
pub const INVALID_CALLDATA_EVENT: &str = "temporal_reaggregation.calldata.invalid";

/// Emitted when there is an erroneous ack/nack event
pub const ERRONEOUS_ACK_EVENT: &str = "temporal_reaggregation.ack.erroneous";

/// Metrics for the temporal reaggregation processor.
#[metric_set(name = "processor.temporal_reaggregation")]
#[derive(Debug, Default, Clone)]
pub struct TemporalReaggregationMetrics {
    /// Number of flushes triggered by the regular timer.
    #[metric(unit = "{flush}")]
    pub flushes_timer: Counter<u64>,

    /// Number of flushes triggered by exceeding the maximum stream count.
    #[metric(unit = "{flush}")]
    pub flushes_overflow: Counter<u64>,

    /// Incoming items dropped because they exceed some limit or fail to be processed.
    #[metric(name = "dropped.items", unit = "{item}")]
    pub dropped_items: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_telemetry::metrics::MetricsRegistry;

    #[test]
    fn test_temporal_reaggregation_metrics() {
        let registry = MetricsRegistry::new();
        let metrics = TemporalReaggregationMetrics::new(&registry);
        
        metrics.flushes_timer.add(1);
        metrics.flushes_overflow.add(1);
        metrics.dropped_items.add(1);
    }
}
