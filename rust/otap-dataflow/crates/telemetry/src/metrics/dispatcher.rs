// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics dispatcher for processing and exporting internal telemetry metrics.

use std::{collections::HashMap, sync::Arc};

use otap_df_config::service::telemetry::metrics::{MetricExporterType, MetricsDispatcherConfig, ProcessorExporterType};
use serde::Serialize;
use tokio::time::{MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

use crate::metrics::exporters::prometheus_exporter::PrometheusMetricsExporter;
use crate::metrics::exporters::logging_exporter::LoggingMetricsExporter;
use crate::metrics::processors::filter_processor::FilterMetricsProcessor;
use crate::{attributes::AttributeValue, descriptor::MetricsField, registry::MetricsRegistryHandle};
use crate::error::Error;

/// Configurable metrics dispatcher.
pub trait MetricsExporter: Send + Sync {
    /// Get the name of the exporter.
    fn name(&self) -> &str;

    /// Export metrics method to be implemented by exporters.
    fn export_metrics(&self, metric_sets: Vec<MetricsEntrySnapshot>) -> Result<(), Error>;
}

/// Configurable metrics processor.
pub trait MetricsProcessor: Send + Sync {
    /// Get the name of the processor.
    fn name(&self) -> &str;

    /// Process metrics method to be implemented by processors.
    fn process_metrics(&self, metric_sets: Vec<MetricsEntrySnapshot>) -> Result<Vec<MetricsEntrySnapshot>, Error>;
}

/// The metrics dispatcher that handles metric processing and exporting.
pub struct MetricsDispatcher {
    metrics_handler: MetricsRegistryHandle,
    flush_interval: std::time::Duration,
    metrics_processors: Vec<Box<dyn MetricsProcessor + Send + Sync>>,
    metrics_exporters: Vec<Box<dyn MetricsExporter + Send + Sync>>,
}

impl MetricsDispatcher {
    /// Create a new metrics dispatcher with the given handler.
    pub fn new(metrics_handler: MetricsRegistryHandle, config: MetricsDispatcherConfig) -> Self {
        let flush_interval = config.flush_interval;
        let new_instance = Self {
            metrics_handler,
            flush_interval,
            metrics_processors: Vec::new(),
            metrics_exporters: Vec::new()
        };
        new_instance.configure(config)
    }

    fn configure(mut self, config: MetricsDispatcherConfig) -> Self {
        for processor_config in config.processors {
            match processor_config.processor_type {
                ProcessorExporterType::Filter => {
                    let processor_config = processor_config.config;
                    let processor = Box::new(FilterMetricsProcessor::new(processor_config));
                    self.metrics_processors.push(processor);
                }
            }
        }

        for exporter_config in config.exporters {
            match exporter_config.exporter_type {
                MetricExporterType::Logging => {
                    let exporter = Box::new(LoggingMetricsExporter::new());
                    self.metrics_exporters.push(exporter);
                }
                MetricExporterType::Prometheus => {
                    let exporter_config = exporter_config.config;
                    let exporter = Box::new(PrometheusMetricsExporter::new(exporter_config));
                    self.metrics_exporters.push(exporter);
                }
            }
        }
        self
    }

    /// Set the flush interval for the dispatcher.
    pub fn with_flush_interval(mut self, interval: std::time::Duration) -> Self {
        self.flush_interval = interval;
        self
    }

    /// Add a metrics exporter to the dispatcher.
    pub fn with_exporter(mut self, exporter: Box<dyn MetricsExporter + Send + Sync>) -> Self {
        self.metrics_exporters.push(exporter);
        self
    }

    /// Add a metrics processor to the dispatcher.
    pub fn with_processor(mut self, processor: Box<dyn MetricsProcessor + Send + Sync>) -> Self {
        self.metrics_processors.push(processor);
        self
    }

    /// Get the flush interval in milliseconds.
    pub fn set_flush_interval_seconds(mut self, seconds: u64) {
        self.flush_interval = std::time::Duration::from_secs(seconds);
    }

    /// Get the flush interval as a Duration.
    pub fn get_flush_interval(&self) -> std::time::Duration {
        self.flush_interval
    }

    /// Start the metrics dispatcher.
    pub async fn run_dispatch_loop(self: Arc<Self>, cancellation_token: CancellationToken) -> Result<(), Error> {
        let mut ticker = interval(self.get_flush_interval());
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    return Ok(())
                }
                _ = ticker.tick() => {
                    if let Err(e) = self.dispatch_metrics() {
                        return Err(e);
                    }
                }
            }
        }
    }

    fn dispatch_metrics(&self) -> Result<(), Error> {
        // Collect the metric sets to be exported
        let mut metrics_entry_snapshots: Vec<MetricsEntrySnapshot> = Vec::new();
        self.metrics_handler.visit_metrics_and_reset(|descriptor, attributes, metrics_iter| {
            
            let mut attributes_map = HashMap::new();
            for (key, value) in attributes.iter_attributes() {
                let _ = attributes_map.insert(key.to_string(), value.clone());
            }

            let mut metrics = Vec::new();
            for (field, value) in metrics_iter {
                let metric_data_point_snapshot = MetricDataPointSnapshot::new (
                    *field,
                    value,
                );
                metrics.push(metric_data_point_snapshot);
            }

            let descriptor_snapshot = MetricDescriptorSnapshot::new(
                descriptor.name.to_owned(),
            );

            let metrics_entry_snapshot = MetricsEntrySnapshot::new(
                descriptor_snapshot,
                attributes_map,
                metrics,
            );
            metrics_entry_snapshots.push(metrics_entry_snapshot);
        });


        if !metrics_entry_snapshots.is_empty() {
            // Process the collected metrics. Propagate errors if any.
            for processor in &self.metrics_processors {
                metrics_entry_snapshots = processor.process_metrics(metrics_entry_snapshots)?;
            }

            // Export the data after processing
            for exporter in &self.metrics_exporters {
                if let Err(e) = exporter.export_metrics(metrics_entry_snapshots.clone()) {
                    // Log the error but continue with other exporters
                    let exporter_name = exporter.name();
                    log::error!("Error exporting internal metrics with {exporter_name}: {e}");
                }
            }
        }

        Ok(())
    }

}

/// Snapshot of a metrics entry at a point in time.
#[derive(Serialize, Clone)]
pub struct MetricsEntrySnapshot {
    /// Descriptor of the metric metadata
    pub descriptor: MetricDescriptorSnapshot,

    /// Attributes associated with this metric set.
    pub attributes: HashMap<String, AttributeValue>,

    /// Individual metrics within this set.
    pub metrics: Vec<MetricDataPointSnapshot>,
}

impl MetricsEntrySnapshot {
    /// Create a new metrics entry snapshot.
    pub fn new(
        descriptor: MetricDescriptorSnapshot,
        attributes: HashMap<String, AttributeValue>,
        metrics: Vec<MetricDataPointSnapshot>,
    ) -> Self {
        Self {
            descriptor,
            attributes,
            metrics,
        }
    }
}

/// Metric data point snapshot.
#[derive(Serialize, Clone)]
pub struct MetricDataPointSnapshot {
    /// Descriptor for retrieving metric metadata
    pub metadata: MetricsField,
    /// Current value.
    pub value: u64,
}

impl MetricDataPointSnapshot {
    /// Create a new metric data point snapshot.
    pub fn new(metadata: MetricsField, value: u64) -> Self {
        Self {
            metadata,
            value
        }
    }
}

/// Metric descriptor snapshot.
#[derive(Serialize, Clone)]
pub struct MetricDescriptorSnapshot {
    /// Human-friendly group name.
    pub name: String,
}

impl MetricDescriptorSnapshot {
    /// Create a new metric descriptor snapshot.
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;

    use crate::{attributes::AttributeSetHandler, descriptor::{AttributeField, AttributeValueType, AttributesDescriptor, Instrument, MetricsDescriptor}, metrics::{MetricSet, MetricSetHandler}};

    use super::*;
    use otap_df_config::service::telemetry::metrics::{MetricExporterConfig, MetricProcessorConfig, ProcessorExporterType};
    use tokio::time::Duration;

    // Mock exporter for testing
    struct MockExporter {
        counter: Arc<AtomicUsize>,
    }

    impl MockExporter {
        fn new(counter: Arc<AtomicUsize>) -> Self {
            Self {
                counter
            }
        }
    }

    impl MetricsExporter for MockExporter {
        fn name(&self) -> &str {
            "MockExporter"
        }

        fn export_metrics(&self, _metric_sets: Vec<MetricsEntrySnapshot>) -> Result<(), Error> {
            // Increment the counter each time export_metrics is called
            _ = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    struct MockProcessor {
        counter: Arc<AtomicUsize>,
    }

    impl MockProcessor {
        fn new(counter: Arc<AtomicUsize>) -> Self {
            Self {
                counter
            }
        }
    }

    impl MetricsProcessor for MockProcessor {
        fn name(&self) -> &str {
            "MockProcessor"
        }

        /// A simple processor that increments a counter and modifies metric names.
        fn process_metrics(&self, metric_sets: Vec<MetricsEntrySnapshot>) -> Result<Vec<MetricsEntrySnapshot>, Error> {
            _ = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            let mut metric_sets_to_iterate = metric_sets;
            for metric_set in metric_sets_to_iterate.iter_mut() {
                let mut descriptor = metric_set.descriptor.clone();
                let new_name = format!("processed_{}", descriptor.name);
                descriptor.name = new_name;
                metric_set.descriptor = descriptor;
            }
            Ok(metric_sets_to_iterate)
        }
    }

    static MOCK_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test_attributes",
        fields: &[AttributeField {
            key: "test_key",
            r#type: AttributeValueType::String,
            brief: "Test attribute",
        }],
    };

    struct MockAttributeSet {
        // Store the attribute values as owned data that we can return references to
        attribute_values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(value: String) -> Self {
            let attribute_values = vec![AttributeValue::String(value.clone())];
            Self { attribute_values }
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

    static MOCK_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test_metrics",
        metrics: &[
            MetricsField {
                name: "counter1",
                unit: "1",
                brief: "Test counter 1",
                instrument: Instrument::Counter,
            },
            MetricsField {
                name: "counter2",
                unit: "1",
                brief: "Test counter 2",
                instrument: Instrument::Counter,
            },
        ],
    };

    #[derive(Debug)]
    struct MockMetricSet {
        values: Vec<u64>,
    }

    impl MockMetricSet {
        fn new() -> Self {
            Self {
                values: vec![0, 0], // Initialize with 2 values to match MOCK_METRICS_DESCRIPTOR
            }
        }
    }

    impl Default for MockMetricSet {
        fn default() -> Self {
            Self::new()
        }
    }


    impl MetricSetHandler for MockMetricSet {
        fn descriptor(&self) -> &'static MetricsDescriptor {
            &MOCK_METRICS_DESCRIPTOR
        }

        fn snapshot_values(&self) -> Vec<u64> {
            self.values.clone()
        }

        fn clear_values(&mut self) {
            self.values.iter_mut().for_each(|v| *v = 0);
        }

        fn needs_flush(&self) -> bool {
            self.values.iter().any(|&v| v != 0)
        }
    }

    #[test]
    fn test_metrics_dispatcher_creation() {
        let registry = MetricsRegistryHandle::new();
        let metrics_dispacher_config = MetricsDispatcherConfig::default();
        let dispatcher = MetricsDispatcher::new(registry, metrics_dispacher_config);
        
        assert_eq!(dispatcher.get_flush_interval(), Duration::from_secs(1));
    }

    #[test]
    fn test_register_exporter() {
        let registry = MetricsRegistryHandle::new();
        let metrics_dispacher_config = MetricsDispatcherConfig::default();

        let exporter_counter = Arc::new(AtomicUsize::new(0));
        let mock_exporter = MockExporter::new(exporter_counter.clone());

        let dispatcher = MetricsDispatcher::new(registry, metrics_dispacher_config)
            .with_exporter(Box::new(mock_exporter));

        assert_eq!(dispatcher.metrics_exporters.len(), 1);
    }

    #[test]
    fn test_register_exporter_through_conf() {
        let registry = MetricsRegistryHandle::new();
        let metrics_dispacher_config = MetricsDispatcherConfig {
            flush_interval: Duration::from_secs(2),
            processors: vec![
                MetricProcessorConfig {
                    processor_type: ProcessorExporterType::Filter,
                    config: serde_json::json!({"exclude": { "metric_names": ["metric_to_exclude"] }}),
                },
            ],
            exporters: vec![
                MetricExporterConfig {
                    exporter_type: MetricExporterType::Logging,
                    config: serde_json::json!({}),
                },
            ],
        };
        let dispatcher = MetricsDispatcher::new(registry, metrics_dispacher_config);
                
        assert_eq!(dispatcher.metrics_exporters.len(), 1);
    }

    #[test]
    fn test_dispatch_metrics_calls_exporters() -> Result<(), Error> {
        let registry = MetricsRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());
        let metric_set: MetricSet<MockMetricSet> = registry.register(attrs);

        let metrics_key = metric_set.key;

        registry.accumulate_snapshot(metrics_key, &[10, 20]);
        registry.accumulate_snapshot(metrics_key, &[5, 15]);

        let exporter_counter = Arc::new(AtomicUsize::new(0));
        let mock_exporter = MockExporter::new(exporter_counter.clone());

        let processor_counter = Arc::new(AtomicUsize::new(0));
        let mock_processor = MockProcessor::new(processor_counter.clone());

        let metrics_dispacher_config = MetricsDispatcherConfig::default();
        let dispatcher = MetricsDispatcher::new(registry, metrics_dispacher_config)
            .with_processor(Box::new(mock_processor))
            .with_exporter(Box::new(mock_exporter));

        // Call dispatch_metrics directly
        dispatcher.dispatch_metrics()?;

        let processor_invocation_count = processor_counter.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(processor_invocation_count, 1);

        let exporter_invocation_count = exporter_counter.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(exporter_invocation_count, 1);

        Ok(())
    }
}
