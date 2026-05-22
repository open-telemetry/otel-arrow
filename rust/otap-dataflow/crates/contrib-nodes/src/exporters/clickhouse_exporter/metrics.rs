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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_logs_increments_log_rows_counter() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(100, ArrowPayloadType::Logs);
        assert_eq!(m.log_rows_written.get(), 100);
        assert_eq!(m.trace_rows_written.get(), 0);
    }

    #[test]
    fn add_spans_increments_trace_rows_counter() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(7, ArrowPayloadType::Spans);
        assert_eq!(m.trace_rows_written.get(), 7);
        assert_eq!(m.log_rows_written.get(), 0);
    }

    #[test]
    fn add_unknown_payload_type_is_noop() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(99, ArrowPayloadType::UnivariateMetrics);
        assert_eq!(m.log_rows_written.get(), 0);
        assert_eq!(m.trace_rows_written.get(), 0);
    }

    #[test]
    fn add_zero_rows_does_not_change_counter() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(0, ArrowPayloadType::Logs);
        assert_eq!(m.log_rows_written.get(), 0);
    }

    #[test]
    fn add_accumulates_across_multiple_calls() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(10, ArrowPayloadType::Logs);
        m.add(20, ArrowPayloadType::Logs);
        m.add(30, ArrowPayloadType::Logs);
        assert_eq!(m.log_rows_written.get(), 60);
    }

    #[test]
    fn counters_are_independent() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(1, ArrowPayloadType::Logs);
        m.add(2, ArrowPayloadType::Spans);
        assert_eq!(m.log_rows_written.get(), 1);
        assert_eq!(m.trace_rows_written.get(), 2);
    }

    #[test]
    fn default_counters_are_zero() {
        let m = ClickhouseExporterMetrics::default();
        assert_eq!(m.log_rows_written.get(), 0);
        assert_eq!(m.trace_rows_written.get(), 0);
    }
}
