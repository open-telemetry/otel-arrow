// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry metrics for the Kafka exporter.
//!
//! These metrics are exposed via the OTAP telemetry system and can be queried
//! from the data-plane admin `/api/v1/metrics` endpoint. They follow the standard
//! `metric_set` pattern used by other OTAP nodes.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the Kafka exporter.
///
/// Tracks success and failure counts for each signal type (logs, metrics, traces).
///
/// Metric set name: `exporter.kafka`
///
/// Individual metric names (after field name translation `_` -> `.`):
/// - `logs.exported` / `logs.failed`
/// - `metrics.exported` / `metrics.failed`
/// - `traces.exported` / `traces.failed`
/// - `acks.received`
/// - `nacks.received`
#[metric_set(name = "exporter.kafka")]
#[derive(Debug, Default, Clone)]
pub struct KafkaExporterMetrics {
    /// Number of log records successfully exported to Kafka.
    #[metric(unit = "{log}")]
    pub logs_exported: Counter<u64>,
    /// Number of log records that failed to export to Kafka.
    #[metric(unit = "{log}")]
    pub logs_failed: Counter<u64>,
    /// Number of metric data points successfully exported to Kafka.
    #[metric(unit = "{datapoint}")]
    pub metrics_exported: Counter<u64>,
    /// Number of metric data points that failed to export to Kafka.
    #[metric(unit = "{datapoint}")]
    pub metrics_failed: Counter<u64>,
    /// Number of trace spans successfully exported to Kafka.
    #[metric(unit = "{span}")]
    pub traces_exported: Counter<u64>,
    /// Number of trace spans that failed to export to Kafka.
    #[metric(unit = "{span}")]
    pub traces_failed: Counter<u64>,
    /// Number of acks received from downstream.
    #[metric(unit = "{batch}")]
    pub acks_received: Counter<u64>,
    /// Number of nacks received from downstream.
    #[metric(unit = "{batch}")]
    pub nacks_received: Counter<u64>,
    /// Batches where topic was resolved from a transport header.
    #[metric(unit = "{batch}")]
    pub topic_from_header: Counter<u64>,
    /// Batches where topic was resolved from static per-signal config.
    #[metric(unit = "{batch}")]
    pub topic_from_static_config: Counter<u64>,
}

impl KafkaExporterMetrics {
    /// Increments the exported counter for the given signal type.
    pub fn inc_exported(&mut self, signal_type: otap_df_config::SignalType) {
        match signal_type {
            otap_df_config::SignalType::Logs => self.logs_exported.inc(),
            otap_df_config::SignalType::Metrics => self.metrics_exported.inc(),
            otap_df_config::SignalType::Traces => self.traces_exported.inc(),
        }
    }

    /// Increments the failed counter for the given signal type.
    pub fn inc_failed(&mut self, signal_type: otap_df_config::SignalType) {
        match signal_type {
            otap_df_config::SignalType::Logs => self.logs_failed.inc(),
            otap_df_config::SignalType::Metrics => self.metrics_failed.inc(),
            otap_df_config::SignalType::Traces => self.traces_failed.inc(),
        }
    }

    /// Increment ack counter when downstream confirms a batch.
    pub fn inc_ack(&mut self) {
        self.acks_received.inc();
    }

    /// Increment nack counter when downstream rejects a batch.
    pub fn inc_nack(&mut self) {
        self.nacks_received.inc();
    }

    /// Increment counter when topic was resolved from a transport header.
    pub fn inc_topic_from_header(&mut self) {
        self.topic_from_header.inc();
    }

    /// Increment counter when topic was resolved from static per-signal config.
    pub fn inc_topic_from_static_config(&mut self) {
        self.topic_from_static_config.inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::SignalType;

    #[test]
    fn inc_exported_traces() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_exported(SignalType::Traces);
        m.inc_exported(SignalType::Traces);
        assert_eq!(m.traces_exported.get(), 2);
        assert_eq!(m.logs_exported.get(), 0);
        assert_eq!(m.metrics_exported.get(), 0);
    }

    #[test]
    fn inc_exported_metrics() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_exported(SignalType::Metrics);
        assert_eq!(m.metrics_exported.get(), 1);
    }

    #[test]
    fn inc_exported_logs() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_exported(SignalType::Logs);
        assert_eq!(m.logs_exported.get(), 1);
    }

    #[test]
    fn inc_failed_traces() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_failed(SignalType::Traces);
        assert_eq!(m.traces_failed.get(), 1);
        assert_eq!(m.traces_exported.get(), 0);
    }

    #[test]
    fn inc_failed_metrics() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_failed(SignalType::Metrics);
        assert_eq!(m.metrics_failed.get(), 1);
    }

    #[test]
    fn inc_failed_logs() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_failed(SignalType::Logs);
        assert_eq!(m.logs_failed.get(), 1);
    }

    #[test]
    fn inc_ack_and_nack() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_ack();
        m.inc_ack();
        m.inc_nack();
        assert_eq!(m.acks_received.get(), 2);
        assert_eq!(m.nacks_received.get(), 1);
    }

    #[test]
    fn counters_are_independent() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_exported(SignalType::Traces);
        m.inc_exported(SignalType::Metrics);
        m.inc_exported(SignalType::Logs);
        m.inc_failed(SignalType::Traces);
        m.inc_ack();
        m.inc_nack();
        m.inc_topic_from_header();
        m.inc_topic_from_static_config();

        assert_eq!(m.traces_exported.get(), 1);
        assert_eq!(m.metrics_exported.get(), 1);
        assert_eq!(m.logs_exported.get(), 1);
        assert_eq!(m.traces_failed.get(), 1);
        assert_eq!(m.metrics_failed.get(), 0);
        assert_eq!(m.logs_failed.get(), 0);
        assert_eq!(m.acks_received.get(), 1);
        assert_eq!(m.nacks_received.get(), 1);
        assert_eq!(m.topic_from_header.get(), 1);
        assert_eq!(m.topic_from_static_config.get(), 1);
    }

    #[test]
    fn inc_topic_from_header() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_topic_from_header();
        m.inc_topic_from_header();
        assert_eq!(m.topic_from_header.get(), 2);
        assert_eq!(m.topic_from_static_config.get(), 0);
    }

    #[test]
    fn inc_topic_from_static_config() {
        let mut m = KafkaExporterMetrics::default();
        m.inc_topic_from_static_config();
        m.inc_topic_from_static_config();
        assert_eq!(m.topic_from_static_config.get(), 2);
        assert_eq!(m.topic_from_header.get(), 0);
    }
}
