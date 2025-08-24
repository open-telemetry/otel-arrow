// SPDX-License-Identifier: Apache-2.0

//! Task periodically collecting the metrics emitted by the engine and the pipelines.

use crate::error::Error;
use crate::metrics::MetricSetSnapshot;
use crate::registry::MetricsRegistryHandle;
use crate::reporter::MetricsReporter;
use tokio::time::{interval, Duration};
use crate::config::Config;

/// Metrics collector.
///
/// In this project, metrics are multivariate, meaning that multiple metrics are reported together
/// sharing the timestamp and the same set of attributes.
pub struct MetricsCollector {
    /// The metrics registry where metrics are declared and aggregated.
    registry: MetricsRegistryHandle,

    /// Receiver for incoming metrics.
    /// The message is a tuple of (MetricsKey, MultivariateMetrics).
    /// The metrics key is the aggregation key for the metrics,
    metrics_receiver: flume::Receiver<MetricSetSnapshot>,

    /// The interval at which metrics are flushed and aggregated by the collector.
    flush_interval: Duration,
}

impl MetricsCollector {
    /// Creates a new `MetricsCollector` with a pipeline.
    pub(crate) fn new(config: Config, registry: MetricsRegistryHandle) -> (Self, MetricsReporter) {
        let (metrics_sender, metrics_receiver) = flume::bounded::<MetricSetSnapshot>(config.reporting_channel_size);

        (
            Self {
                registry,
                metrics_receiver,
                flush_interval: config.flush_interval,
            },
            MetricsReporter::new(metrics_sender),
        )
    }

    /// Collects metrics from the reporting channel and aggregates them into the `registry`.
    /// The collection runs indefinitely until the metrics channel is closed.
    /// Returns the pipeline instance when the loop ends (or None if no pipeline was configured).
    pub async fn run_collection_loop(self) -> Result<(), Error> {
        loop {
            match self.metrics_receiver.recv_async().await {
                Ok(metrics) => {
                    self.registry.accumulate_snapshot(metrics.key, &metrics.metrics);
                }
                Err(_) => {
                    // Channel closed, exit the loop
                    return Ok(());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MetricsKey;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{MetricsDescriptor, MetricsField, AttributesDescriptor, AttributeField, AttributeValueType, Instrument};
    use crate::metrics::MetricSetHandler;
    use std::fmt::Debug;
    use std::time::Duration;

    // --- Test-only mock metric/attributes definitions (no pipeline required) ---

    #[derive(Debug)]
    struct MockMetricSet {
        values: Vec<u64>,
    }

    impl MockMetricSet {
        fn new() -> Self {
            Self { values: vec![0, 0] }
        }
    }

    impl Default for MockMetricSet {
        fn default() -> Self { Self::new() }
    }

    static MOCK_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test_metrics",
        metrics: &[
            MetricsField { name: "counter1", unit: "1", brief: "Test counter 1", instrument: Instrument::Counter },
            MetricsField { name: "counter2", unit: "1", brief: "Test counter 2", instrument: Instrument::Counter },
        ],
    };

    static MOCK_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test_attributes",
        fields: &[AttributeField { key: "test_key", r#type: AttributeValueType::String, brief: "Test attribute" }],
    };

    impl MetricSetHandler for MockMetricSet {
        fn descriptor(&self) -> &'static MetricsDescriptor { &MOCK_METRICS_DESCRIPTOR }
        fn snapshot_values(&self) -> Vec<u64> { self.values.clone() }
        fn clear_values(&mut self) { self.values.iter_mut().for_each(|v| *v = 0); }
        fn needs_flush(&self) -> bool { self.values.iter().any(|&v| v != 0) }
    }

    #[derive(Debug)]
    struct MockAttributeSet {
        value: String,
        attribute_values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(value: impl Into<String>) -> Self {
            let v = value.into();
            Self { value: v.clone(), attribute_values: vec![AttributeValue::String(v)] }
        }
    }

    impl AttributeSetHandler for MockAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor { &MOCK_ATTRIBUTES_DESCRIPTOR }
        fn iter_attributes<'a>(&'a self) -> crate::attributes::AttributeIterator<'a> {
            crate::attributes::AttributeIterator::new(&MOCK_ATTRIBUTES_DESCRIPTOR.fields, &self.attribute_values)
        }
        fn attribute_values(&self) -> &[AttributeValue] { &self.attribute_values }
    }

    fn create_test_config(_flush_interval_ms: u64) -> Config {
        // Flush interval is irrelevant when no pipeline is configured; keep field for completeness.
        Config { reporting_channel_size: 10, flush_interval: Duration::from_millis(_flush_interval_ms) }
    }

    fn create_test_snapshot(key: MetricsKey, values: Vec<u64>) -> MetricSetSnapshot {
        MetricSetSnapshot { key, metrics: values }
    }

    fn create_test_registry() -> MetricsRegistryHandle { MetricsRegistryHandle::new() }

    // --- Tests without any pipeline, asserting on the registry state ---

    #[tokio::test]
    async fn test_collector_without_pipeline_returns_none_on_channel_close() {
        let config = create_test_config(100);
        let registry = create_test_registry();
        let (collector, _reporter) = MetricsCollector::new(config, registry);

        // Close immediately
        drop(_reporter);
        let result = collector.run_collection_loop().await.unwrap();
    }

    #[tokio::test]
    async fn test_accumulates_snapshots_into_registry() {
        let config = create_test_config(10);
        let registry = create_test_registry();

        // Register a metric set to get a valid key
        let metric_set: crate::metrics::MetricSet<MockMetricSet> = registry.register(MockAttributeSet::new("attr"));
        let key = metric_set.key;

        let (collector, reporter) = MetricsCollector::new(config, registry.clone());

        let handle = tokio::spawn(async move { collector.run_collection_loop().await });

        // Send two snapshots that should be accumulated: [10,20] + [5,15] => [15,35]
        reporter.report_snapshot(create_test_snapshot(key, vec![10, 20])).await.unwrap();
        reporter.report_snapshot(create_test_snapshot(key, vec![5, 15])).await.unwrap();

        // Give the collector a brief moment to process
        tokio::time::sleep(Duration::from_millis(30)).await;

        // Inspect current metrics without resetting
        let mut collected = Vec::new();
        registry.visit_current_metrics(|_desc, _attrs, iter| {
            for (field, value) in iter { collected.push((field.name, value)); }
        });

        assert_eq!(collected.len(), 2);
        // Order follows descriptor order
        assert_eq!(collected[0].0, "counter1");
        assert_eq!(collected[0].1, 15);
        assert_eq!(collected[1].0, "counter2");
        assert_eq!(collected[1].1, 35);

        // Close the channel and ensure loop ends returning None
        drop(reporter);
        let res = handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_visit_non_zero_then_reset_via_registry_api() {
        let config = create_test_config(10);
        let registry = create_test_registry();
        let metric_set: crate::metrics::MetricSet<MockMetricSet> = registry.register(MockAttributeSet::new("attr"));
        let key = metric_set.key;

        let (collector, reporter) = MetricsCollector::new(config, registry.clone());
        let handle = tokio::spawn(async move { collector.run_collection_loop().await });

        reporter.report_snapshot(create_test_snapshot(key, vec![7, 0])).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;

        // First visit should see the non-zero and then reset
        let mut first = Vec::new();
        registry.visit_non_zero_metrics_and_reset(|_d, _a, iter| {
            for (f, v) in iter { first.push((f.name, v)); }
        });
        assert_eq!(first, vec![("counter1", 7)]);

        // Second visit should see nothing
        let mut count = 0;
        registry.visit_non_zero_metrics_and_reset(|_, _, _| { count += 1; });
        assert_eq!(count, 0);

        drop(reporter);
        let res = handle.await.unwrap().unwrap();
    }
}
