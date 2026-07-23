// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic metrics used in the OTAP pipeline.
//!
//! Note: We try as much as possible to follow the following
//! [RFC Pipeline Component Telemetry](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).

use otap_df_telemetry::common_attributes::SignalOutcomeAttributes;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Completed PData export operations.
#[metric_set(
    name = "exporter.pdata.exports",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ExporterPDataExportMetrics {
    /// Number of PData messages whose export reached a terminal outcome.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::SignalType;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::common_attributes::Outcome;
    use otap_df_telemetry::metrics::MeasurementMetricSet;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics() -> MeasurementMetricSet<ExporterPDataExportMetrics> {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        ExporterPDataExportMetrics::register(&pipeline_ctx)
    }

    /// Scenario: An exporter completes successful and failed exports for multiple signals.
    /// Guarantees: Terminal export counts are isolated by signal and outcome.
    #[test]
    fn exporter_metrics_are_partitioned_by_signal_and_outcome() {
        let mut metrics = new_test_metrics();
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .messages
            .add(2);
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Failure,
            })
            .messages
            .inc();

        assert_eq!(
            metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .messages
                .get(),
            2
        );
        assert_eq!(
            metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Failure,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Metrics,
                    outcome: Outcome::Success,
                })
                .messages
                .get(),
            0
        );
    }

    /// Scenario: Export outcome metrics are handed off during terminal shutdown twice.
    /// Guarantees: Only touched buckets are emitted and each bucket is cleared after handoff.
    #[test]
    fn terminal_snapshots_emit_touched_buckets_once() {
        let mut metrics = new_test_metrics();
        metrics
            .with(SignalOutcomeAttributes {
                signal: SignalType::Traces,
                outcome: Outcome::Success,
            })
            .messages
            .inc();

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.pdata.exports"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
        }));
        assert!(metrics.terminal_snapshots().is_empty());
    }
}
