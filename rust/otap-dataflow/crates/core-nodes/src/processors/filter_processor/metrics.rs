// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP FilterProcessor node.
use otel_arrow_dfe_telemetry::common_attributes::SignalAttributes;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry_macros::metric_set;

/// Item-drop metrics for the filter processor.
#[metric_set(
    name = "processor.filter",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct FilterDropMetrics {
    /// Number of signal items (log records, spans, or metric data points) a
    /// decision node chose to drop.
    #[metric(name = "dropped.items", unit = "{item}")]
    pub dropped_items: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics() -> MeasurementMetricSet<FilterDropMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        FilterDropMetrics::register(&pipeline_ctx)
    }

    /// Scenario: A filter batch drops items for one telemetry signal.
    /// Guarantees: The dropped-item counter retains the existing signal dimension and value.
    #[test]
    fn dropped_items_are_recorded_by_signal() {
        let mut metrics = new_test_metrics();
        metrics
            .with(SignalAttributes {
                signal: SignalType::Traces,
            })
            .dropped_items
            .add(4);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(
            snapshots[0].measurement_attribute_value("signal"),
            Some("traces")
        );
        assert_eq!(snapshots[0].get_metrics()[0].to_u64_lossy(), 4);
    }
}
