// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics specific to the Clickhouse lifecycle.

use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Clickhouse exporter metrics.
/// Grouped under `otap.exporter.clickhouse`.
#[metric_set(name = "otap.exporter.clickhouse")]
#[derive(Debug, Default, Clone)]
pub struct ClickhouseExporterMetrics {
    /// Total number of log rows written into clickhouse
    #[metric(unit = "{row}")]
    pub log_rows_written: Counter<u64>,

    /// Total number of trace rows written into clickhouse
    #[metric(unit = "{row}")]
    pub trace_rows_written: Counter<u64>,
}

impl ClickhouseExporterMetrics {
    /// Increments the row counter for the given payload type.
    pub fn add(&mut self, rows: u64, payload_type: ArrowPayloadType) {
        match payload_type {
            ArrowPayloadType::Logs => self.log_rows_written.add(rows),
            ArrowPayloadType::Spans => self.trace_rows_written.add(rows),
            _ => {}
        }
    }
}
