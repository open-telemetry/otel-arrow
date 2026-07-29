// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the OTAP PerfExporter node.

use otap_df_telemetry::common_attributes::SignalAttributes;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP PerfExporter.
#[metric_set(
    name = "exporter.perf.pdata",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct PerfExporterPdataMetrics {
    /// Number of log records, metric data points, or spans received by the performance exporter.
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::SignalType;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::metrics::MeasurementMetricSet;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics() -> MeasurementMetricSet<PerfExporterPdataMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        PerfExporterPdataMetrics::register(&pipeline_ctx)
    }

    /// Scenario: The performance exporter receives items from all supported signal types.
    /// Guarantees: Item counts are exported in isolated bounded signal buckets.
    #[test]
    fn items_are_bucketed_by_signal() {
        let mut metrics = new_test_metrics();
        metrics
            .with(SignalAttributes {
                signal: SignalType::Logs,
            })
            .items
            .add(2);
        metrics
            .with(SignalAttributes {
                signal: SignalType::Metrics,
            })
            .items
            .add(3);
        metrics
            .with(SignalAttributes {
                signal: SignalType::Traces,
            })
            .items
            .add(4);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 3);
        assert!(snapshots.iter().all(|snapshot| {
            snapshot.descriptor().name == "exporter.perf.pdata"
                && snapshot.descriptor().metrics[0].name == "items"
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.measurement_attribute_value("signal") == Some("logs")
                && snapshot.get_metrics()[0].to_u64_lossy() == 2
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.measurement_attribute_value("signal") == Some("metrics")
                && snapshot.get_metrics()[0].to_u64_lossy() == 3
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.get_metrics()[0].to_u64_lossy() == 4
        }));
    }
}
