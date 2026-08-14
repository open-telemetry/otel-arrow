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
    /// Total number of log rows written successfully into clickhouse
    #[metric(unit = "{row}")]
    pub log_rows_written: Counter<u64>,

    /// Total number of trace rows written successfully into clickhouse
    #[metric(unit = "{row}")]
    pub trace_rows_written: Counter<u64>,

    /// Total number of OTAP log batches transformed by the specialized path.
    #[metric(unit = "{batch}")]
    pub log_fast_path_batches: Counter<u64>,

    /// Total number of log batches sent through the generic transform fallback.
    #[metric(unit = "{batch}")]
    pub log_transform_fallback_batches: Counter<u64>,

    /// Total number of raw OTLP log batches transformed directly to ClickHouse columns.
    #[metric(unit = "{batch}")]
    pub log_otlp_direct_path_batches: Counter<u64>,

    /// Total number of raw OTLP log batches sent through the legacy transform fallback.
    #[metric(unit = "{batch}")]
    pub log_otlp_transform_fallback_batches: Counter<u64>,
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

    /// Records one log batch transformed by the specialized path.
    pub fn record_log_fast_path(&mut self) {
        self.log_fast_path_batches.inc();
    }

    /// Records one log batch sent through the generic fallback path.
    pub fn record_log_transform_fallback(&mut self) {
        self.log_transform_fallback_batches.inc();
    }

    /// Records one raw OTLP log batch transformed directly to ClickHouse columns.
    pub fn record_log_otlp_direct_path(&mut self) {
        self.log_otlp_direct_path_batches.inc();
    }

    /// Records one raw OTLP log batch transformed by the legacy fallback path.
    pub fn record_log_otlp_transform_fallback(&mut self) {
        self.log_otlp_transform_fallback_batches.inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a successful log insert reports its row count.
    /// Guarantees: log rows increment only the log row counter.
    #[test]
    fn add_logs_increments_log_rows_counter() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(100, ArrowPayloadType::Logs);
        assert_eq!(m.log_rows_written.get(), 100);
        assert_eq!(m.trace_rows_written.get(), 0);
    }

    /// Scenario: a successful span insert reports its row count.
    /// Guarantees: span rows increment only the trace row counter.
    #[test]
    fn add_spans_increments_trace_rows_counter() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(7, ArrowPayloadType::Spans);
        assert_eq!(m.trace_rows_written.get(), 7);
        assert_eq!(m.log_rows_written.get(), 0);
    }

    /// Scenario: a non-signal Arrow payload reports written rows.
    /// Guarantees: unsupported payload types leave both signal row counters unchanged.
    #[test]
    fn add_unknown_payload_type_is_noop() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(99, ArrowPayloadType::UnivariateMetrics);
        assert_eq!(m.log_rows_written.get(), 0);
        assert_eq!(m.trace_rows_written.get(), 0);
    }

    /// Scenario: a successful insert contains zero rows.
    /// Guarantees: adding zero leaves the selected row counter unchanged.
    #[test]
    fn add_zero_rows_does_not_change_counter() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(0, ArrowPayloadType::Logs);
        assert_eq!(m.log_rows_written.get(), 0);
    }

    /// Scenario: several successful log inserts complete.
    /// Guarantees: row metrics accumulate every reported log row count.
    #[test]
    fn add_accumulates_across_multiple_calls() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(10, ArrowPayloadType::Logs);
        m.add(20, ArrowPayloadType::Logs);
        m.add(30, ArrowPayloadType::Logs);
        assert_eq!(m.log_rows_written.get(), 60);
    }

    /// Scenario: log and span inserts both complete successfully.
    /// Guarantees: each signal updates its independent row counter.
    #[test]
    fn counters_are_independent() {
        let mut m = ClickhouseExporterMetrics::default();
        m.add(1, ArrowPayloadType::Logs);
        m.add(2, ArrowPayloadType::Spans);
        assert_eq!(m.log_rows_written.get(), 1);
        assert_eq!(m.trace_rows_written.get(), 2);
    }

    /// Scenario: ClickHouse exporter metrics are newly registered.
    /// Guarantees: all written-row counters start at zero.
    #[test]
    fn default_counters_are_zero() {
        let m = ClickhouseExporterMetrics::default();
        assert_eq!(m.log_rows_written.get(), 0);
        assert_eq!(m.trace_rows_written.get(), 0);
    }

    /// Scenario: specialized and fallback log transforms are both observed.
    /// Guarantees: each transform path increments only its dedicated batch counter.
    #[test]
    fn transform_path_counters_are_independent() {
        let mut metrics = ClickhouseExporterMetrics::default();
        metrics.record_log_fast_path();
        metrics.record_log_transform_fallback();
        metrics.record_log_transform_fallback();

        assert_eq!(metrics.log_fast_path_batches.get(), 1);
        assert_eq!(metrics.log_transform_fallback_batches.get(), 2);
    }

    /// Scenario: direct and fallback raw OTLP log transforms are both observed.
    /// Guarantees: each raw OTLP transform path increments only its dedicated counter.
    #[test]
    fn otlp_transform_path_counters_are_independent() {
        let mut metrics = ClickhouseExporterMetrics::default();
        metrics.record_log_otlp_direct_path();
        metrics.record_log_otlp_direct_path();
        metrics.record_log_otlp_transform_fallback();

        assert_eq!(metrics.log_otlp_direct_path_batches.get(), 2);
        assert_eq!(metrics.log_otlp_transform_fallback_batches.get(), 1);
        assert_eq!(metrics.log_fast_path_batches.get(), 0);
        assert_eq!(metrics.log_transform_fallback_batches.get(), 0);
    }
}
