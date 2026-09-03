// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics specific to the Parquet exporter IO lifecycle.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Lifecycle operations for Parquet exporter files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum FileOperation {
    /// A file was created.
    Created,
    /// A file was closed.
    Closed,
    /// A flush was scheduled because the max rows threshold was reached.
    FlushScheduledMaxRows,
    /// A flush was scheduled because the max age threshold was reached.
    FlushScheduledMaxAge,
    /// A flush attempt was made.
    FlushAttempts,
    /// A flush completed successfully.
    FlushSuccesses,
    /// A flush failed.
    FlushFailures,
}

/// Parquet exporter IO metrics attributes.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ParquetExporterFileAttributes {
    /// The file operation type.
    pub operation: FileOperation,
}

/// Parquet exporter file IO metrics.
/// Grouped under `otap.exporter.parquet.files`.
#[metric_set(
    name = "otap.exporter.parquet.files",
    measurement_attributes = ParquetExporterFileAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ParquetExporterFileMetrics {
    /// Number of Parquet files processed (across all payload types and partitions).
    #[metric(unit = "{file}")]
    pub count: Counter<u64>,
}

/// Parquet exporter row IO metrics.
/// Grouped under `otap.exporter.parquet.rows`.
#[metric_set(name = "otap.exporter.parquet.rows")]
#[derive(Debug, Default, Clone)]
pub struct ParquetExporterRowMetrics {
    /// Total number of rows written into Parquet writers (appended, not necessarily flushed yet).
    #[metric(unit = "{row}")]
    pub written: Counter<u64>,
}

/// Shared bounded-cardinality Parquet exporter metrics tracker.
pub struct ParquetExporterMetrics {
    /// File metrics.
    pub files: MeasurementMetricSet<ParquetExporterFileMetrics>,
    /// Row metrics.
    pub rows: MetricSet<ParquetExporterRowMetrics>,
}

impl ParquetExporterMetrics {
    /// Registers Parquet exporter metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            files: ParquetExporterFileMetrics::register(pipeline_ctx),
            rows: pipeline_ctx.register_metrics::<ParquetExporterRowMetrics>(),
        }
    }

    /// Reports touched metric buckets.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report_measurement(&mut self.files)?;
        reporter.report(&mut self.rows)?;
        Ok(())
    }

    /// Takes every touched metric bucket for terminal handoff.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.files.terminal_snapshots();
        if self.rows.needs_flush() {
            snapshots.push(self.rows.snapshot());
        }
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::attributes::AttributeEnum as _;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    fn pipeline_context() -> PipelineContext {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0)
    }

    /// Scenario: File metrics are partitioned by operation type.
    /// Guarantees: Each `FileOperation` variant isolates its counter independently.
    #[test]
    fn file_metrics_are_partitioned_by_operation() {
        let pipeline = pipeline_context();
        let mut metrics = ParquetExporterMetrics::register(&pipeline);

        metrics
            .files
            .with(ParquetExporterFileAttributes {
                operation: FileOperation::Created,
            })
            .count
            .add(3);
        metrics
            .files
            .with(ParquetExporterFileAttributes {
                operation: FileOperation::Closed,
            })
            .count
            .add(2);
        metrics
            .files
            .with(ParquetExporterFileAttributes {
                operation: FileOperation::FlushAttempts,
            })
            .count
            .add(5);
        metrics
            .files
            .with(ParquetExporterFileAttributes {
                operation: FileOperation::FlushSuccesses,
            })
            .count
            .add(4);
        metrics
            .files
            .with(ParquetExporterFileAttributes {
                operation: FileOperation::FlushFailures,
            })
            .count
            .inc();
        metrics
            .files
            .with(ParquetExporterFileAttributes {
                operation: FileOperation::FlushScheduledMaxRows,
            })
            .count
            .add(6);
        metrics
            .files
            .with(ParquetExporterFileAttributes {
                operation: FileOperation::FlushScheduledMaxAge,
            })
            .count
            .add(7);

        assert_eq!(
            metrics
                .files
                .get(ParquetExporterFileAttributes {
                    operation: FileOperation::Created,
                })
                .count
                .get(),
            3
        );
        assert_eq!(
            metrics
                .files
                .get(ParquetExporterFileAttributes {
                    operation: FileOperation::Closed,
                })
                .count
                .get(),
            2
        );
        assert_eq!(
            metrics
                .files
                .get(ParquetExporterFileAttributes {
                    operation: FileOperation::FlushAttempts,
                })
                .count
                .get(),
            5
        );
        assert_eq!(
            metrics
                .files
                .get(ParquetExporterFileAttributes {
                    operation: FileOperation::FlushSuccesses,
                })
                .count
                .get(),
            4
        );
        assert_eq!(
            metrics
                .files
                .get(ParquetExporterFileAttributes {
                    operation: FileOperation::FlushFailures,
                })
                .count
                .get(),
            1
        );
        assert_eq!(
            metrics
                .files
                .get(ParquetExporterFileAttributes {
                    operation: FileOperation::FlushScheduledMaxRows,
                })
                .count
                .get(),
            6
        );
        assert_eq!(
            metrics
                .files
                .get(ParquetExporterFileAttributes {
                    operation: FileOperation::FlushScheduledMaxAge,
                })
                .count
                .get(),
            7
        );
    }

    /// Scenario: Row metrics track written rows independently of file operations.
    /// Guarantees: `rows.written` accumulates correctly.
    #[test]
    fn row_metrics_track_written_rows() {
        let pipeline = pipeline_context();
        let mut metrics = ParquetExporterMetrics::register(&pipeline);

        metrics.rows.written.add(100);
        metrics.rows.written.add(50);

        assert_eq!(metrics.rows.written.get(), 150);
    }

    /// Scenario: Terminal snapshots expose touched file and row metric buckets.
    /// Guarantees: Snapshots contain the correct descriptors and are cleared after handoff.
    #[test]
    fn terminal_snapshots_expose_parquet_schema_and_clear() {
        let pipeline = pipeline_context();
        let mut metrics = ParquetExporterMetrics::register(&pipeline);

        metrics
            .files
            .with(ParquetExporterFileAttributes {
                operation: FileOperation::Created,
            })
            .count
            .inc();
        metrics.rows.written.add(42);

        let snapshots = metrics.terminal_snapshots();
        // Should have at least one file snapshot and one row snapshot.
        assert!(snapshots.len() >= 2);

        let file_snapshot = snapshots
            .iter()
            .find(|s| s.descriptor().name == "otap.exporter.parquet.files")
            .expect("expected file metrics snapshot");
        assert_eq!(file_snapshot.descriptor().metrics[0].name, "count");
        assert_eq!(file_snapshot.descriptor().metrics[0].unit, "{file}");
        assert_eq!(
            file_snapshot.measurement_attribute_value("operation"),
            Some("created")
        );

        let row_snapshot = snapshots
            .iter()
            .find(|s| s.descriptor().name == "otap.exporter.parquet.rows")
            .expect("expected row metrics snapshot");
        assert_eq!(row_snapshot.descriptor().metrics[0].name, "written");
        assert_eq!(row_snapshot.descriptor().metrics[0].unit, "{row}");

        // Second call should return empty since buckets were already taken.
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: FileOperation enum values are rendered for telemetry consumption.
    /// Guarantees: Every variant has a stable lowercase telemetry value.
    #[test]
    fn file_operation_attribute_values_are_stable() {
        assert_eq!(FileOperation::Created.as_str(), "created");
        assert_eq!(FileOperation::Closed.as_str(), "closed");
        assert_eq!(
            FileOperation::FlushScheduledMaxRows.as_str(),
            "flush_scheduled_max_rows"
        );
        assert_eq!(
            FileOperation::FlushScheduledMaxAge.as_str(),
            "flush_scheduled_max_age"
        );
        assert_eq!(FileOperation::FlushAttempts.as_str(), "flush_attempts");
        assert_eq!(FileOperation::FlushSuccesses.as_str(), "flush_successes");
        assert_eq!(FileOperation::FlushFailures.as_str(), "flush_failures");
    }
}
