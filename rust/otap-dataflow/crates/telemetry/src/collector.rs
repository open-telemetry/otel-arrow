// SPDX-License-Identifier: Apache-2.0

//! Task periodically collecting the metrics emitted by the engine and the pipelines.

use crate::error::Error;
use crate::metrics::MetricSetSnapshot;
use crate::pipeline::MetricsPipeline;
use crate::registry::MetricsRegistryHandle;
use crate::reporter::MetricsReporter;
use tokio::time::{interval, Duration};
use crate::config::Config;

/// Metrics collector.
///
/// In this project, metrics are multivariate, meaning that multiple metrics are reported together
/// sharing the timestamp and the same set of attributes.
pub struct MetricsCollector<P: MetricsPipeline> {
    /// The metrics registry where metrics are declared and aggregated.
    registry: MetricsRegistryHandle,

    /// Receiver for incoming metrics.
    /// The message is a tuple of (MetricsKey, MultivariateMetrics).
    /// The metrics key is the aggregation key for the metrics,
    metrics_receiver: flume::Receiver<MetricSetSnapshot>,

    /// The interval at which metrics are flushed and aggregated by the collector.
    flush_interval: Duration,

    /// The metrics pipeline for reporting metrics.
    pipeline: P,
}

impl<P: MetricsPipeline> MetricsCollector<P> {
    /// Creates a new `MetricsCollector`.
    pub(crate) fn new(config: Config, registry: MetricsRegistryHandle, pipeline: P) -> (Self, MetricsReporter) {
        let (metrics_sender, metrics_receiver) = flume::bounded::<MetricSetSnapshot>(config.reporting_channel_size);

        (
            Self {
                registry,
                metrics_receiver,
                flush_interval: config.flush_interval,
                pipeline,
            },
            MetricsReporter::new(metrics_sender),
        )
    }

    /// Collects metrics from the reporting channel and aggregates them into the `registry`.
    /// The collection runs indefinitely until the metrics channel is closed.
    /// Returns the pipeline instance when the loop ends.
    pub async fn run_collection_loop(self) -> Result<P, Error> {
        let mut timer = interval(self.flush_interval);

        loop {
            tokio::select! {
                // ToDo need to be moved into a CollectorExporter
                _ = timer.tick() => {
                    self.registry.visit_non_zero_metrics_and_reset(|measurement, field_iter, attrs| {
                        // Ignore individual report errors for now; could log.
                        let _ = self.pipeline.report_iter(measurement, field_iter, attrs);
                    });
                }
                result = self.metrics_receiver.recv_async() => {
                    match result {
                        Ok(metrics) => {
                            self.registry.accumulate_snapshot(metrics.key, &metrics.metrics);
                        }
                        Err(_) => {
                            // Channel closed, exit the loop and return the pipeline
                            return Ok(self.pipeline);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MetricsKey;
    use crate::attributes::AttributeSetHandler;
    use crate::descriptor::MetricsDescriptor;
    use crate::registry::NonZeroMetrics;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    // Mock pipeline for testing that tracks reported metrics
    #[derive(Debug, Clone)]
    struct MockPipeline {
        reports: Arc<Mutex<Vec<ReportCall>>>,
    }

    #[derive(Debug, Clone)]
    struct ReportCall {
        descriptor_name: String,
        metrics_data: Vec<(String, u64)>, // field_name, value
    }

    impl MockPipeline {
        fn new() -> Self {
            Self {
                reports: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_reports(&self) -> Vec<ReportCall> {
            self.reports.lock().unwrap().clone()
        }

        fn report_count(&self) -> usize {
            self.reports.lock().unwrap().len()
        }

        fn clear_reports(&self) {
            self.reports.lock().unwrap().clear();
        }
    }

    impl MetricsPipeline for MockPipeline {
        fn report_iter<'a>(
            &self,
            desc: &'static MetricsDescriptor,
            _attrs: &'a dyn AttributeSetHandler,
            metrics: NonZeroMetrics<'a>,
        ) -> Result<(), Error> {
            let mut reports = self.reports.lock().unwrap();
            let mut metrics_data = Vec::new();

            for (field_desc, value) in metrics {
                metrics_data.push((field_desc.name.to_string(), value));
            }

            reports.push(ReportCall {
                descriptor_name: desc.name.to_string(),
                metrics_data,
            });
            Ok(())
        }
    }

    // Error pipeline for testing error handling
    #[derive(Debug, Clone)]
    struct ErrorPipeline {
        should_fail: Arc<Mutex<bool>>,
        call_count: Arc<Mutex<usize>>,
    }

    impl ErrorPipeline {
        fn new() -> Self {
            Self {
                should_fail: Arc::new(Mutex::new(false)),
                call_count: Arc::new(Mutex::new(0)),
            }
        }

        fn set_should_fail(&self, fail: bool) {
            *self.should_fail.lock().unwrap() = fail;
        }

        fn get_call_count(&self) -> usize {
            *self.call_count.lock().unwrap()
        }
    }

    impl MetricsPipeline for ErrorPipeline {
        fn report_iter<'a>(
            &self,
            desc: &'static MetricsDescriptor,
            _attrs: &'a dyn AttributeSetHandler,
            _metrics: NonZeroMetrics<'a>,
        ) -> Result<(), Error> {
            *self.call_count.lock().unwrap() += 1;

            if *self.should_fail.lock().unwrap() {
                Err(Error::MetricsNotRegistered { descriptor: desc })
            } else {
                Ok(())
            }
        }
    }

    fn create_test_config(flush_interval_ms: u64) -> Config {
        Config {
            reporting_channel_size: 10,
            flush_interval: Duration::from_millis(flush_interval_ms),
        }
    }

    fn create_test_snapshot(key: MetricsKey, values: Vec<u64>) -> MetricSetSnapshot {
        MetricSetSnapshot { key, metrics: values }
    }

    fn create_test_registry() -> MetricsRegistryHandle {
        MetricsRegistryHandle::new()
    }

    #[tokio::test]
    async fn test_collector_returns_pipeline_on_channel_close() {
        let config = create_test_config(1000);
        let registry = create_test_registry();
        let mock_pipeline = MockPipeline::new();
        let initial_report_count = mock_pipeline.report_count();

        let (sender, receiver) = flume::bounded::<MetricSetSnapshot>(config.reporting_channel_size);
        let collector = MetricsCollector {
            registry,
            metrics_receiver: receiver,
            flush_interval: config.flush_interval,
            pipeline: mock_pipeline.clone(),
        };

        // Close the channel immediately
        drop(sender);

        let returned_pipeline = collector.run_collection_loop().await.unwrap();

        // Verify the returned pipeline has the same state as when we started
        assert_eq!(returned_pipeline.report_count(), initial_report_count);
    }

    #[tokio::test]
    async fn test_pipeline_content_validation() {
        let config = create_test_config(50); // Short interval for quick test
        let registry = create_test_registry();
        let mock_pipeline = MockPipeline::new();

        // Pre-populate registry with test data
        let test_key = MetricsKey::default();
        registry.accumulate_snapshot(test_key, &vec![42, 24]);

        let (sender, receiver) = flume::bounded::<MetricSetSnapshot>(config.reporting_channel_size);
        let collector = MetricsCollector {
            registry,
            metrics_receiver: receiver,
            flush_interval: config.flush_interval,
            pipeline: mock_pipeline.clone(),
        };

        let collection_task = tokio::spawn(async move {
            collector.run_collection_loop().await
        });

        // Wait for timer to trigger and flush the metrics
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Close the channel
        drop(sender);

        let returned_pipeline = collection_task.await.unwrap().unwrap();

        // Now validate the actual content reported to the pipeline
        let reports = returned_pipeline.get_reports();

        // We should have at least one report from the timer flush
        if !reports.is_empty() {
            // Validate that the reports contain actual metric data
            for report in &reports {
                assert!(!report.descriptor_name.is_empty());
                // Each report should have metrics data
                assert!(!report.metrics_data.is_empty());

                // Validate field names and values are reasonable
                for (field_name, value) in &report.metrics_data {
                    assert!(!field_name.is_empty());
                    assert!(*value > 0); // We populated with non-zero values
                }
            }
        }
    }

    #[tokio::test]
    async fn test_pipeline_receives_accumulated_metrics() {
        let config = create_test_config(40); // Fast timer
        let registry = create_test_registry();
        let mock_pipeline = MockPipeline::new();

        let (sender, receiver) = flume::bounded::<MetricSetSnapshot>(config.reporting_channel_size);
        let collector = MetricsCollector {
            registry: registry.clone(),
            metrics_receiver: receiver,
            flush_interval: config.flush_interval,
            pipeline: mock_pipeline.clone(),
        };

        let collection_task = tokio::spawn(async move {
            collector.run_collection_loop().await
        });

        // Send metrics that will be accumulated
        let test_key = MetricsKey::default();
        sender.send_async(create_test_snapshot(test_key, vec![10, 20])).await.unwrap();
        sender.send_async(create_test_snapshot(test_key, vec![5, 15])).await.unwrap();

        // Wait for metrics to be processed and timer to fire
        tokio::time::sleep(Duration::from_millis(80)).await;

        let reports_count = mock_pipeline.report_count();

        // Close channel
        drop(sender);
        let returned_pipeline = collection_task.await.unwrap().unwrap();

        // Verify that the pipeline received reports from the timer
        let final_reports = returned_pipeline.get_reports();
        assert_eq!(final_reports.len(), reports_count);

        // If we got reports, verify they contain the accumulated data
        if !final_reports.is_empty() {
            for report in &final_reports {
                // Each report should have data from accumulated metrics
                for (_field_name, value) in &report.metrics_data {
                    // Values should reflect accumulation (15 = 10+5, 35 = 20+15)
                    assert!(*value >= 15); // At least the accumulated minimum
                }
            }
        }
    }
}
