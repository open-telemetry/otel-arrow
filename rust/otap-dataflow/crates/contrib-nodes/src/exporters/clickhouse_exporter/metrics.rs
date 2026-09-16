// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics specific to the Clickhouse lifecycle.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_telemetry::common_attributes::SignalAttributes;
use otel_arrow_dfe_telemetry::config::SignalType;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Clickhouse transform path variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum ClickhouseTransformPath {
    FastPath,
    GenericFallback,
    OtlpDirect,
    OtlpLegacyFallback,
}

/// Attributes for transform batches metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ClickhouseTransformAttributes {
    #[attribute_key = "path"]
    pub path: ClickhouseTransformPath,
}

/// Rows written metrics for ClickHouse.
#[metric_set(
    name = "otap.exporter.clickhouse.rows",
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
    name = "otap.exporter.clickhouse.batches",
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
}

