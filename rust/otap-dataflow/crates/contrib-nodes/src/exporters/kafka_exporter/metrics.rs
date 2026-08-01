// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry metrics for the Kafka exporter.
//!
//! These metrics are exposed via the OTAP telemetry system and can be queried
//! from the data-plane admin `/api/v1/metrics` endpoint. They follow the standard
//! `metric_set` pattern used by other OTAP nodes.

use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

/// Signal-specific export completion metrics.
#[metric_set(
    name = "exporter.kafka.exports",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct KafkaExporterExportMetrics {
    /// Number of exported items partitioned by `signal` and `outcome` (`success` or `failure`).
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
}

/// Operational metrics for the Kafka exporter.
#[metric_set(name = "exporter.kafka")]
#[derive(Debug, Default, Clone)]
pub struct KafkaExporterOperationalMetrics {
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

pub struct KafkaExporterMetrics {
    pub export_metrics: MeasurementMetricSet<KafkaExporterExportMetrics>,
    pub operational_metrics: MetricSet<KafkaExporterOperationalMetrics>,
}

impl KafkaExporterMetrics {
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            export_metrics: KafkaExporterExportMetrics::register(pipeline_ctx),
            operational_metrics: KafkaExporterOperationalMetrics::register(pipeline_ctx),
        }
    }

    pub fn report(
        &mut self,
        reporter: &mut MetricsReporter,
    ) -> Result<(), otap_df_telemetry::error::Error> {
        reporter
            .report(&mut self.operational_metrics)
            .and_then(|()| reporter.report_measurement(&mut self.export_metrics))
    }

    pub fn terminal_snapshots(&mut self) -> Vec<otap_df_telemetry::metrics::MetricSetSnapshot> {
        let mut snapshots = self.operational_metrics.terminal_snapshots();
        snapshots.extend(self.export_metrics.terminal_snapshots());
        snapshots
    }

    pub fn inc_exported(&mut self, signal: otap_df_config::SignalType) {
        self.export_metrics
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Success,
            })
            .items
            .inc();
    }

    pub fn inc_failed(&mut self, signal: otap_df_config::SignalType) {
        self.export_metrics
            .with(SignalOutcomeAttributes {
                signal,
                outcome: Outcome::Failure,
            })
            .items
            .inc();
    }

    pub fn inc_ack(&mut self) {
        self.operational_metrics.acks_received.inc();
    }

    pub fn inc_nack(&mut self) {
        self.operational_metrics.nacks_received.inc();
    }

    pub fn inc_topic_from_header(&mut self) {
        self.operational_metrics.topic_from_header.inc();
    }

    pub fn inc_topic_from_static_config(&mut self) {
        self.operational_metrics.topic_from_static_config.inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporters::kafka_exporter::exporter::test_support::pipeline_context;
    use otap_df_config::SignalType;

    fn new_metrics() -> KafkaExporterMetrics {
        KafkaExporterMetrics::register(&pipeline_context())
    }

    /// Scenario: Traces are exported successfully.
    /// Guarantees: The traces success counter is incremented.
    #[test]
    fn inc_exported_traces() {
        let mut m = new_metrics();
        m.inc_exported(SignalType::Traces);
        m.inc_exported(SignalType::Traces);
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Traces,
                    outcome: Outcome::Success
                })
                .items
                .get(),
            2
        );
    }

    /// Scenario: Metrics are exported successfully.
    /// Guarantees: The metrics success counter is incremented.
    #[test]
    fn inc_exported_metrics() {
        let mut m = new_metrics();
        m.inc_exported(SignalType::Metrics);
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Metrics,
                    outcome: Outcome::Success
                })
                .items
                .get(),
            1
        );
    }

    /// Scenario: Logs are exported successfully.
    /// Guarantees: The logs success counter is incremented.
    #[test]
    fn inc_exported_logs() {
        let mut m = new_metrics();
        m.inc_exported(SignalType::Logs);
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success
                })
                .items
                .get(),
            1
        );
    }

    /// Scenario: Traces export fails.
    /// Guarantees: The traces failure counter is incremented.
    #[test]
    fn inc_failed_traces() {
        let mut m = new_metrics();
        m.inc_failed(SignalType::Traces);
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Traces,
                    outcome: Outcome::Failure
                })
                .items
                .get(),
            1
        );
    }

    /// Scenario: Metrics export fails.
    /// Guarantees: The metrics failure counter is incremented.
    #[test]
    fn inc_failed_metrics() {
        let mut m = new_metrics();
        m.inc_failed(SignalType::Metrics);
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Metrics,
                    outcome: Outcome::Failure
                })
                .items
                .get(),
            1
        );
    }

    /// Scenario: Logs export fails.
    /// Guarantees: The logs failure counter is incremented.
    #[test]
    fn inc_failed_logs() {
        let mut m = new_metrics();
        m.inc_failed(SignalType::Logs);
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Failure
                })
                .items
                .get(),
            1
        );
    }

    /// Scenario: Acks and nacks are received.
    /// Guarantees: Operational counters for acks and nacks are incremented correctly.
    #[test]
    fn inc_ack_and_nack() {
        let mut m = new_metrics();
        m.inc_ack();
        m.inc_ack();
        m.inc_nack();
        assert_eq!(m.operational_metrics.acks_received.get(), 2);
        assert_eq!(m.operational_metrics.nacks_received.get(), 1);
    }

    /// Scenario: Export, ACK, NACK, and topic routing events occur simultaneously.
    /// Guarantees: All counters increment independently without interfering with each other.
    #[test]
    fn counters_are_independent() {
        let mut m = new_metrics();
        m.inc_exported(SignalType::Traces);
        m.inc_exported(SignalType::Metrics);
        m.inc_exported(SignalType::Logs);
        m.inc_failed(SignalType::Traces);
        m.inc_ack();
        m.inc_nack();
        m.inc_topic_from_header();
        m.inc_topic_from_static_config();

        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Traces,
                    outcome: Outcome::Success
                })
                .items
                .get(),
            1
        );
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Metrics,
                    outcome: Outcome::Success
                })
                .items
                .get(),
            1
        );
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success
                })
                .items
                .get(),
            1
        );
        assert_eq!(
            m.export_metrics
                .with(SignalOutcomeAttributes {
                    signal: SignalType::Traces,
                    outcome: Outcome::Failure
                })
                .items
                .get(),
            1
        );

        assert_eq!(m.operational_metrics.acks_received.get(), 1);
        assert_eq!(m.operational_metrics.nacks_received.get(), 1);
        assert_eq!(m.operational_metrics.topic_from_header.get(), 1);
        assert_eq!(m.operational_metrics.topic_from_static_config.get(), 1);
    }

    /// Scenario: Topic is resolved from a header.
    /// Guarantees: The corresponding operational counter is incremented.
    #[test]
    fn inc_topic_from_header() {
        let mut m = new_metrics();
        m.inc_topic_from_header();
        m.inc_topic_from_header();
        assert_eq!(m.operational_metrics.topic_from_header.get(), 2);
        assert_eq!(m.operational_metrics.topic_from_static_config.get(), 0);
    }

    /// Scenario: Topic is resolved from static config.
    /// Guarantees: The corresponding operational counter is incremented.
    #[test]
    fn inc_topic_from_static_config() {
        let mut m = new_metrics();
        m.inc_topic_from_static_config();
        m.inc_topic_from_static_config();
        assert_eq!(m.operational_metrics.topic_from_static_config.get(), 2);
        assert_eq!(m.operational_metrics.topic_from_header.get(), 0);
    }
}
