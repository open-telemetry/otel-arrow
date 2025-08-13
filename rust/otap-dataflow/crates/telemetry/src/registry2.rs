// SPDX-License-Identifier: Apache-2.0

//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: This module will be entirely generated from a telemetry schema and Weaver in the future.

use std::fmt::Debug;
use crate::metrics::{PerfExporterMetrics, ReceiverMetrics};

#[derive(Debug)]
pub struct MetricsRegistry {
    receiver_metrics: ReceiverMetrics,
    perf_exporter_metrics: PerfExporterMetrics,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            receiver_metrics: ReceiverMetrics::default(),
            perf_exporter_metrics: PerfExporterMetrics::default(),
        }
    }

    /// Adds a new set of receiver metrics to the aggregator.
    pub fn add_receiver_metrics(&mut self, metrics: &ReceiverMetrics) {
        self.receiver_metrics.bytes_received.add(metrics.bytes_received.get());
        self.receiver_metrics.messages_received.add(metrics.messages_received.get());
    }

    /// Adds a new set of perf exporter metrics to the aggregator.
    pub fn add_perf_exporter_metrics(&mut self, metrics: &PerfExporterMetrics) {
        self.perf_exporter_metrics.bytes_total.add(metrics.bytes_total.get());
        self.perf_exporter_metrics.pdata_msgs.add(metrics.pdata_msgs.get());
        self.perf_exporter_metrics.invalid_pdata_msgs.add(metrics.invalid_pdata_msgs.get());
        self.perf_exporter_metrics.logs.add(metrics.logs.get());
        self.perf_exporter_metrics.spans.add(metrics.spans.get());
        self.perf_exporter_metrics.metrics.add(metrics.metrics.get());
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::{MultivariateMetrics, ReceiverMetrics};

    #[test]
    fn test_multivariate_metrics_aggregator() {
        let mut aggregator = super::MetricsRegistry::new();
        let receiver1 = ReceiverMetrics {
            bytes_received: 100.into(),
            messages_received: 10.into(),
        };
        let receiver2 = ReceiverMetrics {
            bytes_received: 200.into(),
            messages_received: 20.into(),
        };
        let perf_exporter1 = crate::metrics::PerfExporterMetrics {
            bytes_total: 500.into(),
            pdata_msgs: 50.into(),
            invalid_pdata_msgs: 40.into(),
            logs: 60.into(),
            spans: 70.into(),
            metrics: 80.into(),
        };
        let perf_exporter2 = crate::metrics::PerfExporterMetrics {
            bytes_total: 600.into(),
            pdata_msgs: 70.into(),
            invalid_pdata_msgs: 60.into(),
            logs: 80.into(),
            spans: 90.into(),
            metrics: 100.into(),
        };

        receiver1.aggregate_into(&mut aggregator);
        receiver2.aggregate_into(&mut aggregator);
        perf_exporter1.aggregate_into(&mut aggregator);
        perf_exporter2.aggregate_into(&mut aggregator);

        assert_eq!(aggregator.receiver_metrics.bytes_received.get(), 300);
        assert_eq!(aggregator.receiver_metrics.messages_received.get(), 30);

        assert_eq!(aggregator.perf_exporter_metrics.bytes_total.get(), 1100);
        assert_eq!(aggregator.perf_exporter_metrics.pdata_msgs.get(), 120);
        assert_eq!(aggregator.perf_exporter_metrics.invalid_pdata_msgs.get(), 100);
        assert_eq!(aggregator.perf_exporter_metrics.logs.get(), 140);
        assert_eq!(aggregator.perf_exporter_metrics.spans.get(), 160);
        assert_eq!(aggregator.perf_exporter_metrics.metrics.get(), 180);
    }
}