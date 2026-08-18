// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded telemetry instruments for file exporter outcomes and filesystem operations.
//!
//! Metrics use closed signal and operation attributes and deliberately omit destination paths so
//! component observability cannot introduce unbounded cardinality or expose sensitive locations.

use otap_df_config::SignalType;
use otap_df_telemetry::common_attributes::{SignalAttributes, SignalOutcomeAttributes};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Terminal file export outcomes, partitioned by signal and outcome.
#[metric_set(
    name = "exporter.file.exports",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct FileExporterExportMetrics {
    /// Telemetry messages whose export reached a terminal outcome.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

/// Bounded file operation associated with an I/O failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum FileOperation {
    /// File acquisition, directory creation, open, or tail validation.
    Open,
    /// Frame write or flush.
    Write,
    /// Data synchronization.
    Sync,
    /// Failed-write rollback.
    Rollback,
}

/// Signal and operation dimensions for file failures.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SignalOperationAttributes {
    /// Signal associated with the file writer.
    pub signal: SignalType,
    /// Failed file operation.
    pub operation: FileOperation,
}

/// Successful writes and append-tail recovery, partitioned by signal.
#[metric_set(name = "exporter.file", measurement_attributes = SignalAttributes)]
#[derive(Debug, Default, Clone)]
pub struct FileSignalMetrics {
    /// Signal items in frames successfully written before ACK routing.
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
    /// Successfully written frame bytes, including newline delimiters.
    #[metric(unit = "By")]
    pub bytes: Counter<u64>,
    /// Append-mode partial tails successfully repaired.
    #[metric(unit = "{recovery}")]
    pub tail_recoveries: Counter<u64>,
    /// Bytes removed by successful append-tail recovery.
    #[metric(unit = "By")]
    pub tail_recovered_bytes: Counter<u64>,
}

/// File I/O failures, partitioned by signal and bounded operation.
#[metric_set(
    name = "exporter.file",
    measurement_attributes = SignalOperationAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct FileFailureMetrics {
    /// Open, write, sync, or rollback failures.
    #[metric(unit = "{failure}")]
    pub failures: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::attributes::AttributeEnum as _;
    use otap_df_telemetry::common_attributes::Outcome;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn pipeline_context() -> otap_df_engine::context::PipelineContext {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0)
    }

    /// Scenario: File exports complete with different outcomes for multiple signals.
    /// Guarantees: `exporter.file.exports.messages` isolates counts by signal and outcome.
    #[test]
    fn export_metrics_are_partitioned_by_signal_and_outcome() {
        let mut metrics = FileExporterExportMetrics::register(&pipeline_context());
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

    /// Scenario: Export and file-operation metrics are handed off at terminal shutdown.
    /// Guarantees: Snapshots expose the exact component-specific schema and clear touched buckets.
    #[test]
    fn terminal_snapshots_expose_file_exporter_schema_once() {
        let pipeline = pipeline_context();
        let mut exports = FileExporterExportMetrics::register(&pipeline);
        let mut failures = FileFailureMetrics::register(&pipeline);
        exports
            .with(SignalOutcomeAttributes {
                signal: SignalType::Traces,
                outcome: Outcome::Success,
            })
            .messages
            .inc();
        failures
            .with(SignalOperationAttributes {
                signal: SignalType::Traces,
                operation: FileOperation::Sync,
            })
            .failures
            .inc();

        let export_snapshots = exports.terminal_snapshots();
        let failure_snapshots = failures.terminal_snapshots();
        assert_eq!(export_snapshots.len(), 1);
        assert_eq!(failure_snapshots.len(), 1);

        let export = &export_snapshots[0];
        assert_eq!(export.descriptor().name, "exporter.file.exports");
        assert_eq!(export.descriptor().metrics[0].name, "messages");
        assert_eq!(export.descriptor().metrics[0].unit, "{message}");
        assert_eq!(export.measurement_attribute_value("signal"), Some("traces"));
        assert_eq!(
            export.measurement_attribute_value("outcome"),
            Some("success")
        );

        let failure = &failure_snapshots[0];
        assert_eq!(failure.descriptor().name, "exporter.file");
        assert_eq!(failure.descriptor().metrics[0].name, "failures");
        assert_eq!(failure.descriptor().metrics[0].unit, "{failure}");
        assert_eq!(
            failure.measurement_attribute_value("signal"),
            Some("traces")
        );
        assert_eq!(
            failure.measurement_attribute_value("operation"),
            Some("sync")
        );

        assert!(exports.terminal_snapshots().is_empty());
        assert!(failures.terminal_snapshots().is_empty());
    }

    /// Scenario: File-operation enum values are rendered into metrics and events.
    /// Guarantees: Every operation has a stable lowercase telemetry value.
    #[test]
    fn file_operation_attribute_values_are_stable() {
        assert_eq!(FileOperation::Open.as_str(), "open");
        assert_eq!(FileOperation::Write.as_str(), "write");
        assert_eq!(FileOperation::Sync.as_str(), "sync");
        assert_eq!(FileOperation::Rollback.as_str(), "rollback");
    }
}
