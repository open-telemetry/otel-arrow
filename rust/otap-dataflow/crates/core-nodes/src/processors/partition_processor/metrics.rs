// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the PartitionProcessor node.

use otap_df_telemetry::common_attributes::OutcomeAttributes;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Partition operations grouped by outcome.
#[metric_set(
    name = "processor.partition",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    /// Number of partitioning operations attempted by this processor.
    #[metric(unit = "{operation}")]
    pub operations: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::common_attributes::Outcome;
    use otap_df_telemetry::metrics::MeasurementMetricSet;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics() -> MeasurementMetricSet<Metrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        Metrics::register(&pipeline_ctx)
    }

    /// Scenario: Partition operations complete with successful and failed outcomes.
    /// Guarantees: Operation counts are exported as isolated bounded outcome buckets.
    #[test]
    fn partition_operations_are_bucketed_by_outcome() {
        let mut metrics = new_test_metrics();
        metrics
            .with(OutcomeAttributes {
                outcome: Outcome::Success,
            })
            .operations
            .add(2);
        metrics
            .with(OutcomeAttributes {
                outcome: Outcome::Failure,
            })
            .operations
            .inc();

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.iter().all(|snapshot| {
            snapshot.descriptor().name == "processor.partition"
                && snapshot.descriptor().metrics[0].name == "operations"
                && snapshot.descriptor().metrics[0].unit == "{operation}"
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.measurement_attribute_value("outcome") == Some("success")
                && snapshot.get_metrics()[0].to_u64_lossy() == 2
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.measurement_attribute_value("outcome") == Some("failure")
                && snapshot.get_metrics()[0].to_u64_lossy() == 1
        }));
    }
}
