// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics specific to the Clickhouse lifecycle.

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_telemetry::common_attributes::SignalAttributes;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Transformation path used to convert log batches for ClickHouse export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum ClickhouseTransformPath {
    /// Specialized zero-copy transformation path for canonical OTAP log records.
    FastPath,
    /// Generic transformation plan fallback for OTAP records that cannot use the fast path.
    GenericFallback,
    /// Direct transformation path from raw serialized OTLP protobuf log requests into ClickHouse columns.
    OtlpDirect,
    /// Legacy fallback transformation path used when direct raw OTLP log conversion fails.
    OtlpLegacyFallback,
}

/// Attributes for transform batches metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ClickhouseTransformAttributes {
    /// Transformation path used for the log batch.
    #[attribute_key = "path"]
    pub path: ClickhouseTransformPath,
}

/// Rows written metrics for ClickHouse.
#[metric_set(
    name = "exporter.clickhouse.rows",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ClickhouseRowMetrics {
    /// Total number of rows written successfully into clickhouse
    #[metric(unit = "{row}")]
    pub written: Counter<u64>,
}

/// Transform batch metrics for ClickHouse.
#[metric_set(
    name = "exporter.clickhouse.batches",
    measurement_attributes = ClickhouseTransformAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ClickhouseBatchMetrics {
    /// Total number of log batches transformed by the given path.
    #[metric(unit = "{batch}")]
    pub transformed: Counter<u64>,
}

/// Clickhouse exporter metrics.
pub struct ClickhouseExporterMetrics {
    row_metrics: MeasurementMetricSet<ClickhouseRowMetrics>,
    batch_metrics: MeasurementMetricSet<ClickhouseBatchMetrics>,
}

impl ClickhouseExporterMetrics {
    /// Creates and registers a new ClickhouseExporterMetrics instance.
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            row_metrics: ClickhouseRowMetrics::register(pipeline_ctx),
            batch_metrics: ClickhouseBatchMetrics::register(pipeline_ctx),
        }
    }

    /// Reports measurements to the metrics reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter
            .report_measurement(&mut self.row_metrics)
            .and_then(|()| reporter.report_measurement(&mut self.batch_metrics))
    }

    /// Increments the row counter for the given payload type.
    pub fn add(&mut self, rows: u64, payload_type: ArrowPayloadType) {
        let signal = match payload_type {
            ArrowPayloadType::Logs => SignalType::Logs,
            ArrowPayloadType::Spans => SignalType::Traces,
            _ => return,
        };
        self.row_metrics
            .with(SignalAttributes { signal })
            .written
            .add(rows);
    }

    /// Records one log batch transformed by the specialized path.
    pub fn record_log_fast_path(&mut self) {
        self.batch_metrics
            .with(ClickhouseTransformAttributes {
                path: ClickhouseTransformPath::FastPath,
            })
            .transformed
            .inc();
    }

    /// Records one log batch sent through the generic fallback path.
    pub fn record_log_transform_fallback(&mut self) {
        self.batch_metrics
            .with(ClickhouseTransformAttributes {
                path: ClickhouseTransformPath::GenericFallback,
            })
            .transformed
            .inc();
    }

    /// Records one raw OTLP log batch transformed directly to ClickHouse columns.
    pub fn record_log_otlp_direct_path(&mut self) {
        self.batch_metrics
            .with(ClickhouseTransformAttributes {
                path: ClickhouseTransformPath::OtlpDirect,
            })
            .transformed
            .inc();
    }

    /// Records one raw OTLP log batch transformed by the legacy fallback path.
    pub fn record_log_otlp_transform_fallback(&mut self) {
        self.batch_metrics
            .with(ClickhouseTransformAttributes {
                path: ClickhouseTransformPath::OtlpLegacyFallback,
            })
            .transformed
            .inc();
    }

    /// Returns the terminal snapshots of the metrics.
    pub fn terminal_snapshots(
        &mut self,
    ) -> Vec<otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot> {
        let mut snapshots = Vec::new();
        snapshots.extend(self.row_metrics.terminal_snapshots());
        snapshots.extend(self.batch_metrics.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::attributes::AttributeEnum;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    fn pipeline_context() -> PipelineContext {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0)
    }

    /// Scenario: ClickHouse transform path enum values are rendered into metrics.
    /// Guarantees: Every transform path variant has a stable lowercase snake_case telemetry value.
    #[test]
    fn clickhouse_transform_path_attribute_values_are_stable() {
        assert_eq!(ClickhouseTransformPath::FastPath.as_str(), "fast_path");
        assert_eq!(
            ClickhouseTransformPath::GenericFallback.as_str(),
            "generic_fallback"
        );
        assert_eq!(ClickhouseTransformPath::OtlpDirect.as_str(), "otlp_direct");
        assert_eq!(
            ClickhouseTransformPath::OtlpLegacyFallback.as_str(),
            "otlp_legacy_fallback"
        );
        assert_eq!(ClickhouseTransformPath::CARDINALITY, 4);
        assert_eq!(
            ClickhouseTransformPath::VARIANTS,
            &[
                "fast_path",
                "generic_fallback",
                "otlp_direct",
                "otlp_legacy_fallback"
            ]
        );
    }

    /// Scenario: ClickHouse transform attributes wrap a transform path.
    /// Guarantees: The path attribute is preserved without mutation.
    #[test]
    fn test_clickhouse_transform_attributes() {
        let attr = ClickhouseTransformAttributes {
            path: ClickhouseTransformPath::FastPath,
        };
        assert_eq!(attr.path, ClickhouseTransformPath::FastPath);
    }

    /// Scenario: ClickHouse row metrics counter is incremented directly.
    /// Guarantees: The written counter records the added row count.
    #[test]
    fn test_clickhouse_row_metrics() {
        let mut m = ClickhouseRowMetrics::default();
        m.written.add(10);
        assert_eq!(m.written.get(), 10);
    }

    /// Scenario: ClickHouse batch metrics counter is incremented directly.
    /// Guarantees: The transformed counter records batch increments.
    #[test]
    fn test_clickhouse_batch_metrics() {
        let mut m = ClickhouseBatchMetrics::default();
        m.transformed.inc();
        assert_eq!(m.transformed.get(), 1);
    }

    /// Scenario: a successful log insert reports its row count.
    /// Guarantees: log rows increment only the log row counter.
    #[test]
    fn add_logs_increments_log_rows_counter() {
        let pipeline_ctx = pipeline_context();
        let mut m = ClickhouseExporterMetrics::new(&pipeline_ctx);
        m.add(100, ArrowPayloadType::Logs);
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .written
                .get(),
            100
        );
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Traces,
                })
                .written
                .get(),
            0
        );
    }

    /// Scenario: a successful span insert reports its row count.
    /// Guarantees: span rows increment only the trace row counter.
    #[test]
    fn add_spans_increments_trace_rows_counter() {
        let pipeline_ctx = pipeline_context();
        let mut m = ClickhouseExporterMetrics::new(&pipeline_ctx);
        m.add(7, ArrowPayloadType::Spans);
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Traces,
                })
                .written
                .get(),
            7
        );
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .written
                .get(),
            0
        );
    }

    /// Scenario: a non-signal Arrow payload reports written rows.
    /// Guarantees: unsupported payload types leave both signal row counters unchanged.
    #[test]
    fn add_unknown_payload_type_is_noop() {
        let pipeline_ctx = pipeline_context();
        let mut m = ClickhouseExporterMetrics::new(&pipeline_ctx);
        m.add(99, ArrowPayloadType::UnivariateMetrics);
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .written
                .get(),
            0
        );
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Traces,
                })
                .written
                .get(),
            0
        );
    }

    /// Scenario: a successful insert contains zero rows.
    /// Guarantees: adding zero leaves the selected row counter unchanged.
    #[test]
    fn add_zero_rows_does_not_change_counter() {
        let pipeline_ctx = pipeline_context();
        let mut m = ClickhouseExporterMetrics::new(&pipeline_ctx);
        m.add(0, ArrowPayloadType::Logs);
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .written
                .get(),
            0
        );
    }

    /// Scenario: several successful log inserts complete.
    /// Guarantees: row metrics accumulate every reported log row count.
    #[test]
    fn add_accumulates_across_multiple_calls() {
        let pipeline_ctx = pipeline_context();
        let mut m = ClickhouseExporterMetrics::new(&pipeline_ctx);
        m.add(10, ArrowPayloadType::Logs);
        m.add(20, ArrowPayloadType::Logs);
        m.add(30, ArrowPayloadType::Logs);
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .written
                .get(),
            60
        );
    }

    /// Scenario: log and span inserts both complete successfully.
    /// Guarantees: each signal updates its independent row counter.
    #[test]
    fn counters_are_independent() {
        let pipeline_ctx = pipeline_context();
        let mut m = ClickhouseExporterMetrics::new(&pipeline_ctx);
        m.add(1, ArrowPayloadType::Logs);
        m.add(2, ArrowPayloadType::Spans);
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .written
                .get(),
            1
        );
        assert_eq!(
            m.row_metrics
                .get(SignalAttributes {
                    signal: SignalType::Traces,
                })
                .written
                .get(),
            2
        );
    }

    /// Scenario: specialized and fallback log transforms are both observed.
    /// Guarantees: each transform path increments only its dedicated batch counter.
    #[test]
    fn transform_path_counters_are_independent() {
        let pipeline_ctx = pipeline_context();
        let mut metrics = ClickhouseExporterMetrics::new(&pipeline_ctx);
        metrics.record_log_fast_path();
        metrics.record_log_transform_fallback();
        metrics.record_log_transform_fallback();

        assert_eq!(
            metrics
                .batch_metrics
                .get(ClickhouseTransformAttributes {
                    path: ClickhouseTransformPath::FastPath,
                })
                .transformed
                .get(),
            1
        );
        assert_eq!(
            metrics
                .batch_metrics
                .get(ClickhouseTransformAttributes {
                    path: ClickhouseTransformPath::GenericFallback,
                })
                .transformed
                .get(),
            2
        );
        assert_eq!(
            metrics
                .batch_metrics
                .get(ClickhouseTransformAttributes {
                    path: ClickhouseTransformPath::OtlpDirect,
                })
                .transformed
                .get(),
            0
        );
        assert_eq!(
            metrics
                .batch_metrics
                .get(ClickhouseTransformAttributes {
                    path: ClickhouseTransformPath::OtlpLegacyFallback,
                })
                .transformed
                .get(),
            0
        );
    }

    /// Scenario: direct and fallback raw OTLP log transforms are both observed.
    /// Guarantees: each raw OTLP transform path increments only its dedicated counter.
    #[test]
    fn otlp_transform_path_counters_are_independent() {
        let pipeline_ctx = pipeline_context();
        let mut metrics = ClickhouseExporterMetrics::new(&pipeline_ctx);
        metrics.record_log_otlp_direct_path();
        metrics.record_log_otlp_direct_path();
        metrics.record_log_otlp_transform_fallback();

        assert_eq!(
            metrics
                .batch_metrics
                .get(ClickhouseTransformAttributes {
                    path: ClickhouseTransformPath::OtlpDirect,
                })
                .transformed
                .get(),
            2
        );
        assert_eq!(
            metrics
                .batch_metrics
                .get(ClickhouseTransformAttributes {
                    path: ClickhouseTransformPath::OtlpLegacyFallback,
                })
                .transformed
                .get(),
            1
        );
        assert_eq!(
            metrics
                .batch_metrics
                .get(ClickhouseTransformAttributes {
                    path: ClickhouseTransformPath::FastPath,
                })
                .transformed
                .get(),
            0
        );
        assert_eq!(
            metrics
                .batch_metrics
                .get(ClickhouseTransformAttributes {
                    path: ClickhouseTransformPath::GenericFallback,
                })
                .transformed
                .get(),
            0
        );
    }

    /// Scenario: terminal snapshots expose touched row and batch metric buckets.
    /// Guarantees: snapshots contain the correct descriptors, dimensions, and are cleared after handoff.
    #[test]
    fn terminal_snapshots_expose_clickhouse_schema_and_clear() {
        let pipeline_ctx = pipeline_context();
        let mut metrics = ClickhouseExporterMetrics::new(&pipeline_ctx);
        metrics.add(42, ArrowPayloadType::Logs);
        metrics.record_log_fast_path();

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);

        let row_snapshot = snapshots
            .iter()
            .find(|s| s.descriptor().name == "exporter.clickhouse.rows")
            .expect("expected row metrics snapshot");
        assert_eq!(row_snapshot.descriptor().metrics[0].name, "written");
        assert_eq!(row_snapshot.descriptor().metrics[0].unit, "{row}");
        assert_eq!(
            row_snapshot.measurement_attribute_value("signal"),
            Some("logs")
        );

        let batch_snapshot = snapshots
            .iter()
            .find(|s| s.descriptor().name == "exporter.clickhouse.batches")
            .expect("expected batch metrics snapshot");
        assert_eq!(batch_snapshot.descriptor().metrics[0].name, "transformed");
        assert_eq!(batch_snapshot.descriptor().metrics[0].unit, "{batch}");
        assert_eq!(
            batch_snapshot.measurement_attribute_value("path"),
            Some("fast_path")
        );

        // Second call should return empty since buckets were already taken.
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: multiple signals and transformation paths record metrics.
    /// Guarantees: terminal snapshots contain dimensioned attributes for every reported signal and path.
    #[test]
    fn terminal_snapshots_dimensioned_attributes() {
        let pipeline_ctx = pipeline_context();
        let mut metrics = ClickhouseExporterMetrics::new(&pipeline_ctx);
        metrics.add(10, ArrowPayloadType::Logs);
        metrics.add(20, ArrowPayloadType::Spans);
        metrics.record_log_fast_path();
        metrics.record_log_transform_fallback();
        metrics.record_log_otlp_direct_path();
        metrics.record_log_otlp_transform_fallback();

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 6);

        let row_signals: Vec<_> = snapshots
            .iter()
            .filter(|s| s.descriptor().name == "exporter.clickhouse.rows")
            .map(|s| s.measurement_attribute_value("signal"))
            .collect();
        assert!(row_signals.contains(&Some("logs")));
        assert!(row_signals.contains(&Some("traces")));

        let batch_paths: Vec<_> = snapshots
            .iter()
            .filter(|s| s.descriptor().name == "exporter.clickhouse.batches")
            .map(|s| s.measurement_attribute_value("path"))
            .collect();
        assert!(batch_paths.contains(&Some("fast_path")));
        assert!(batch_paths.contains(&Some("generic_fallback")));
        assert!(batch_paths.contains(&Some("otlp_direct")));
        assert!(batch_paths.contains(&Some("otlp_legacy_fallback")));
    }

    /// Scenario: measurements are reported to metrics reporter.
    /// Guarantees: report successfully writes all touched measurement metric sets.
    #[test]
    fn report_emits_measurements() {
        let pipeline_ctx = pipeline_context();
        let mut metrics = ClickhouseExporterMetrics::new(&pipeline_ctx);
        let (receiver, mut reporter) = MetricsReporter::create_new_and_receiver(16);
        metrics.add(10, ArrowPayloadType::Logs);
        metrics.record_log_fast_path();

        metrics.report(&mut reporter).unwrap();
        assert_eq!(receiver.try_iter().count(), 2);
    }
}
