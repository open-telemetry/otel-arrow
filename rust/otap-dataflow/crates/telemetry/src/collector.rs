// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Task periodically collecting the internal signals emitted by the engine and the pipelines.

use std::sync::Arc;

use otap_df_config::pipeline::telemetry::TelemetryConfig;
use tokio_util::sync::CancellationToken;

use crate::error::Error;
use crate::metrics::MetricSetSnapshot;
use crate::registry::TelemetryRegistryHandle;
use crate::reporter::MetricsReporter;

/// Internal collector responsible for gathering internal telemetry signals (fow now only metric
/// sets or multivariate metrics).
pub struct InternalCollector {
    /// The registry where entities and metrics are declared and aggregated.
    registry: TelemetryRegistryHandle,

    /// Receiver for incoming metrics.
    /// The message is a combination of a MetricSetKey and collection of MetricValues.
    /// The metrics key is the aggregation key for the metrics,
    metrics_receiver: flume::Receiver<MetricSetSnapshot>,
}

impl InternalCollector {
    /// Creates a new `InternalCollector` with a pipeline.
    pub(crate) fn new(
        config: &TelemetryConfig,
        registry: TelemetryRegistryHandle,
    ) -> (Self, MetricsReporter) {
        let (metrics_sender, metrics_receiver) =
            flume::bounded::<MetricSetSnapshot>(config.reporting_channel_size);

        (
            Self {
                registry,
                metrics_receiver,
            },
            MetricsReporter::new(metrics_sender),
        )
    }

    /// Drains all pending metrics from the reporting channel and aggregates
    /// them into the `registry`.
    pub fn collect_pending(&self) {
        while let Ok(metrics) = self.metrics_receiver.try_recv() {
            self.registry
                .accumulate_metric_set_snapshot(metrics.key, &metrics.metrics);
        }
    }

    /// Collects metrics from the reporting channel and aggregates them into the `registry`.
    /// The collection runs indefinitely until the metrics channel is closed.
    /// Returns the pipeline instance when the loop ends (or None if no pipeline was configured).
    pub async fn run_collection_loop(self: Arc<Self>) -> Result<(), Error> {
        loop {
            match self.metrics_receiver.recv_async().await {
                Ok(metrics) => {
                    self.registry
                        .accumulate_metric_set_snapshot(metrics.key, &metrics.metrics);
                }
                Err(_) => {
                    // Channel closed, exit the loop
                    return Ok(());
                }
            }
        }
    }

    /// Runs the collection loop until cancellation is requested.
    ///
    /// This method starts the internal signal collection loop and listens for a shutdown signal.
    /// It returns when either the collection loop ends (Ok/Err) or the shutdown signal fires.
    pub async fn run(self: Arc<Self>, cancel: CancellationToken) -> Result<(), Error> {
        tokio::select! {
            biased;

            _ = cancel.cancelled() => {
                // Shutdown requested; cancel the collection loop by dropping its future.
                Ok(())
            }
            res = self.run_collection_loop() => {
                res
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use otap_df_config::pipeline::telemetry::metrics::MetricsConfig;
    use otap_df_config::settings::telemetry::logs::LogsConfig;

    use super::*;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, Instrument, MetricValueType,
        MetricsDescriptor, MetricsField, Temporality,
    };
    use crate::metrics::MetricSetHandler;
    use crate::metrics::MetricValue;
    use crate::registry::MetricSetKey;
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::time::Duration;

    // --- Test-only mock metric/attributes definitions (no pipeline required) ---

    #[derive(Debug)]
    struct MockMetricSet {
        values: Vec<MetricValue>,
    }

    impl MockMetricSet {
        fn new() -> Self {
            Self {
                values: vec![MetricValue::from(0u64), MetricValue::from(0u64)],
            }
        }
    }

    impl Default for MockMetricSet {
        fn default() -> Self {
            Self::new()
        }
    }

    static MOCK_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test_metrics",
        metrics: &[
            MetricsField {
                name: "counter1",
                unit: "1",
                brief: "Test counter 1",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "counter2",
                unit: "1",
                brief: "Test counter 2",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
        ],
    };

    static MOCK_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test_attributes",
        fields: &[AttributeField {
            key: "test_key",
            r#type: AttributeValueType::String,
            brief: "Test attribute",
        }],
    };

    impl MetricSetHandler for MockMetricSet {
        fn descriptor(&self) -> &'static MetricsDescriptor {
            &MOCK_METRICS_DESCRIPTOR
        }
        fn snapshot_values(&self) -> Vec<MetricValue> {
            self.values.clone()
        }
        fn clear_values(&mut self) {
            self.values.iter_mut().for_each(MetricValue::reset);
        }
        fn needs_flush(&self) -> bool {
            self.values.iter().any(|&v| !v.is_zero())
        }
    }

    #[derive(Debug)]
    struct MockAttributeSet {
        _value: String,
        attribute_values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(value: impl Into<String>) -> Self {
            let v = value.into();
            Self {
                _value: v.clone(),
                attribute_values: vec![AttributeValue::String(v)],
            }
        }
    }

    impl AttributeSetHandler for MockAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &MOCK_ATTRIBUTES_DESCRIPTOR
        }
        fn iter_attributes<'a>(&'a self) -> crate::attributes::AttributeIterator<'a> {
            crate::attributes::AttributeIterator::new(
                MOCK_ATTRIBUTES_DESCRIPTOR.fields,
                &self.attribute_values,
            )
        }
        fn attribute_values(&self) -> &[AttributeValue] {
            &self.attribute_values
        }
    }

    fn create_test_config(reporting_interval_ms: u64) -> TelemetryConfig {
        // Flush interval is irrelevant when no pipeline is configured; keep field for completeness.
        TelemetryConfig {
            reporting_channel_size: 10,
            reporting_interval: Duration::from_millis(reporting_interval_ms),
            metrics: MetricsConfig::default(),
            logs: LogsConfig::default(),
            resource: HashMap::new(),
        }
    }

    fn create_test_snapshot(key: MetricSetKey, values: Vec<MetricValue>) -> MetricSetSnapshot {
        MetricSetSnapshot {
            key,
            metrics: values,
        }
    }

    fn create_test_registry() -> TelemetryRegistryHandle {
        TelemetryRegistryHandle::new()
    }

    // --- Tests without any pipeline, asserting on the registry state ---

    #[tokio::test]
    async fn test_collector_without_pipeline_returns_none_on_channel_close() {
        let config = create_test_config(100);
        let telemetry_registry = create_test_registry();
        let (collector, _reporter) = InternalCollector::new(&config, telemetry_registry);

        // Close immediately
        drop(_reporter);
        Arc::new(collector).run_collection_loop().await.unwrap();
    }

    #[tokio::test]
    async fn test_accumulates_snapshots_into_registry() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();

        // Register a metric set to get a valid key
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;

        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());

        let handle = tokio::spawn(async move { Arc::new(collector).run_collection_loop().await });

        // Send two snapshots that should be accumulated: [10,20] + [5,15] => [15,35]
        reporter
            .report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(10u64), MetricValue::from(20u64)],
            ))
            .await
            .unwrap();
        reporter
            .report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(5u64), MetricValue::from(15u64)],
            ))
            .await
            .unwrap();

        // Give the collector a brief moment to process
        tokio::time::sleep(Duration::from_millis(30)).await;

        // Inspect current metrics without resetting
        let mut collected = Vec::new();
        telemetry_registry.visit_current_metrics(|_desc, _attrs, iter| {
            for (field, value) in iter {
                collected.push((field.name, value));
            }
        });

        assert_eq!(collected.len(), 2);
        // Order follows descriptor order
        assert_eq!(collected[0].0, "counter1");
        assert_eq!(collected[0].1, MetricValue::from(15u64));
        assert_eq!(collected[1].0, "counter2");
        assert_eq!(collected[1].1, MetricValue::from(35u64));

        // Close the channel and ensure loop ends returning None
        drop(reporter);
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_visit_then_reset_via_registry_api() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;

        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());
        let handle = tokio::spawn(async move { Arc::new(collector).run_collection_loop().await });

        reporter
            .report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(7u64), MetricValue::from(0u64)],
            ))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;

        // First visit should see the non-zero and then reset
        let mut first = Vec::new();
        telemetry_registry.visit_metrics_and_reset(|_d, _a, iter| {
            for (f, v) in iter {
                first.push((f.name, v));
            }
        });
        assert_eq!(
            first,
            vec![
                ("counter1", MetricValue::from(7u64)),
                ("counter2", MetricValue::from(0u64))
            ]
        );

        // Second visit should see nothing
        let mut count = 0;
        telemetry_registry.visit_metrics_and_reset(|_, _, _| {
            count += 1;
        });
        assert_eq!(count, 0);

        drop(reporter);
        handle.await.unwrap().unwrap();
    }
}
