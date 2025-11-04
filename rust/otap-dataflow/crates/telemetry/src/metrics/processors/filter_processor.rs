// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry Filter Metrics processor.

use std::collections::HashSet;

use serde::Deserialize;
use serde_json::Value;

use crate::metrics::dispatcher::{MetricsEntrySnapshot, MetricsProcessor};
use crate::error::Error;

/// Filter metrics processor that filters out metrics based on configuration.
pub struct FilterMetricsProcessor {
    exclude_metrics: HashSet<String>,
}

impl MetricsProcessor for FilterMetricsProcessor {
    fn name(&self) -> &str {
        "FilterMetricsProcessor"
    }

    fn process_metrics(&self, metric_sets: Vec<MetricsEntrySnapshot>) -> Result<Vec<MetricsEntrySnapshot>, Error> {
        let mut metric_sets_clonned = Vec::new();
        for metric_set in metric_sets {
            let mut filtered_metrics = Vec::new();
            for metric in metric_set.metrics {
                if !self.exclude_metrics.contains(metric.metadata.name) {
                    filtered_metrics.push(metric);
                }
            }
            if filtered_metrics.len() > 0 {
                let new_metric_set = MetricsEntrySnapshot {
                    descriptor: metric_set.descriptor,
                    attributes: metric_set.attributes,
                    metrics: filtered_metrics,
                };
                metric_sets_clonned.push(new_metric_set);
            }
        }
        Ok(metric_sets_clonned)
    }
}

impl FilterMetricsProcessor {
    /// Creates a new instance of the FilterMetricsProcessor.
    pub fn new(config: Value) -> Self {
        let filter_metrics_processor_config: FilterMetricsProcessorConfig = serde_json::from_value(config)
            .expect("Failed to parse FilterMetricsProcessor configuration");
        let exclude = filter_metrics_processor_config.exclude.metric_names.into_iter().collect();
        FilterMetricsProcessor { exclude_metrics: exclude}
    }
}

/// Configuration for the FilterMetricsProcessor.
#[derive(Default, Debug, Deserialize)]
pub struct FilterMetricsProcessorConfig {
    /// Configuration for excluding metrics.
    pub exclude: ExcludeConfig,
}

/// Configuration for excluding metrics.
#[derive(Default, Debug, Deserialize)]
pub struct ExcludeConfig {
    /// List of metric names to exclude.
    pub metric_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{descriptor::MetricsField, metrics::dispatcher::{MetricDataPointSnapshot, MetricDescriptorSnapshot}};

    use super::*;

    #[test]
    fn test_new_filter_metrics_processor() {
        let config = serde_json::json!({
            "exclude": {
                "metric_names": ["metric_to_exclude"]
            }
        });
        let processor = FilterMetricsProcessor::new(config);
        assert_eq!(processor.exclude_metrics.len(), 1);
        assert!(processor.exclude_metrics.contains("metric_to_exclude"));
    }

    #[test]
    fn test_process_metrics() {
        let config = serde_json::json!({
            "exclude": {
                "metric_names": ["metric_to_exclude"]
            }
        });
        let processor = FilterMetricsProcessor::new(config);
        let metrics_field1 = MetricsField {
            name: "metric_to_exclude",
            unit: "unit",
            brief: "brief of metric to exclude",
            instrument: crate::descriptor::Instrument::Counter,
        };
        let metrics_field2 = MetricsField {
            name: "metric_that_remains",
            unit: "unit",
            brief: "brief of metric that remains",
            instrument: crate::descriptor::Instrument::Counter,
        };
        let metric_sets = vec![
            MetricsEntrySnapshot {
                descriptor: MetricDescriptorSnapshot::new("metric_to_exclude".to_string()),
                attributes: HashMap::new(),
                metrics: vec![MetricDataPointSnapshot::new(metrics_field1, 1)],
            },
            MetricsEntrySnapshot {
                descriptor: MetricDescriptorSnapshot::new("metric_that_remains".to_string()),
                attributes: HashMap::new(),
                metrics: vec![MetricDataPointSnapshot::new(metrics_field2, 2)],
            },
        ];
        let processed = processor.process_metrics(metric_sets).unwrap();
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].descriptor.name, "metric_that_remains");
    }
}
