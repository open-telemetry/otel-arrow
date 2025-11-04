// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics telemetry configuration specification.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use serde_json::Value;

/// Configuration of the metrics dispatcher.
#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MetricsDispatcherConfig {
    /// The interval at which metrics are flushed and aggregated by the collector.
    #[serde(default = "default_flush_interval")]
    pub flush_interval: std::time::Duration,

    /// List of metric processor configurations to be used by the dispatcher.
    #[serde(default = "default_processors")]
    pub processors: Vec<MetricProcessorConfig>,

    /// List of metric exporter configurations to be used by the dispatcher.
    #[serde(default = "default_exporters")]
    pub exporters: Vec<MetricExporterConfig>,
}

impl Default for MetricsDispatcherConfig {
    fn default() -> Self {
        Self {
            flush_interval: default_flush_interval(),
            processors: default_processors(),
            exporters: default_exporters(),
        }
    }
}

/// Default flush interval of 1 second.
fn default_flush_interval() -> std::time::Duration {
    std::time::Duration::from_secs(1)
}

fn default_exporters() -> Vec<MetricExporterConfig> {
    Vec::new()
}

fn default_processors() -> Vec<MetricProcessorConfig> {
    Vec::new()
}

/// Type of metric processor.
#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProcessorExporterType {
    /// Filters out metrics based on configuration.
    Filter,
}

/// Configuration for a specific metric processor.
#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MetricProcessorConfig {
    #[serde(rename = "type")]
    /// Type of the metric processor.
    pub processor_type: ProcessorExporterType,

    /// Processor specific configuration.
    /// This configuration is interpreted by the processor itself and is not interpreted and validated during load time.
    #[serde(default)]
    pub config: Value,
}


/// Type of metric exporter.
#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricExporterType {
    /// Exports metrics to standard output.
    Logging,
    /// Exports metrics to Prometheus.
    Prometheus,
}

/// Configuration for a specific metric exporter.
#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MetricExporterConfig {
    #[serde(rename = "type")]
    /// Type of the metric exporter.
    pub exporter_type: MetricExporterType,

    /// Exporter specific configuration.
    /// This configuration is interpreted by the exporter itself and is not interpreted and validated during load time.
    #[serde(default)]
    pub config: Value,
}
