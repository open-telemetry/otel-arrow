// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the fake signal receiver
//!


use crate::fake_signal_receiver::fake_signal::{
    fake_otlp_logs,
    fake_otlp_metrics,
    fake_otlp_traces, 
};
use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::LogsData,
    metrics::v1::MetricsData,
    trace::v1::TracesData,
};
use serde::{Deserialize, Serialize};

/// Configuration should take a scenario to play out
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    name: String,
    steps: Vec<ScenarioStep>,
}

impl Config {
    /// Create a new config given a name and a vector of scenario steps
    #[must_use]
    pub fn new(name: String, steps: Vec<ScenarioStep>) -> Self {
        Self { name, steps }
    }
    /// Provide a reference to the vector of scenario steps
    #[must_use]
    pub fn get_steps(&self) -> &Vec<ScenarioStep> {
        &self.steps
    }
}

/// A scenario step will contain a configuration
#[derive(Clone, Deserialize, Serialize)]
pub struct ScenarioStep {
    // name: String,
    /// delay in ms
    #[serde(default = "default_delay_between_batches")]
    delay_between_batch: u64,
    #[serde(default = "default_batches")]
    batches: u64,
    config: SignalConfig,
}

fn default_delay_between_batches() -> u64 {
    0
}

fn default_batches() -> u64 {
    1
}

impl ScenarioStep {
    /// create a new step
    #[must_use]
    pub fn new(config: SignalConfig, batches: u64, delay_between_batch: u64) -> Self {
        Self {
            config,
            batches,
            delay_between_batch,
        }
    }
    /// return the configuration stored inside the scenario step
    #[must_use]
    pub fn get_config(&self) -> SignalConfig {
        self.config
    }

    /// return the number of batches to generate
    #[must_use]
    pub fn get_batches(&self) -> u64 {
        self.batches
    }

    /// return the delay in ms
    #[must_use]
    pub fn get_delay_between_batch(&self) -> u64 {
        self.delay_between_batch
    }
}

/// configs to describe the data being generated
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(tag = "signal_type")]
pub enum SignalConfig {
    /// metric config
    Metric(MetricConfig),
    /// log config
    Log(LogConfig),
    /// span config
    Span(SpanConfig),
}
/// Specify the datapoint type for a metric
#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum MetricType {
    /// gauge datapoints
    Gauge,
    /// sum datapoints
    Sum,
    /// histogram datapoints
    Histogram,
    /// exponetial histogram datapoints
    ExponentialHistogram,
    /// summary datapoints
    Summary,
}

// ToDo: Need to be able to have multiple different datapoint types without all the metrics in the resource being the same
/// configuration settings for a fake metric signal
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct MetricConfig {
    resource_count: usize,
    scope_count: usize,
    metric_count: usize,
    datapoint_count: usize,
    datapoint_type: MetricType,
    attribute_count: usize,
}

/// configuration settings for a fake log signal
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct LogConfig {
    resource_count: usize,
    scope_count: usize,
    log_count: usize,
    attribute_count: usize,
}
/// configuration settings for a fake trace signal
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct SpanConfig {
    resource_count: usize,
    scope_count: usize,
    span_count: usize,
    event_count: usize,
    link_count: usize,
    attribute_count: usize,
}

impl MetricConfig {
    /// create a new config
    #[must_use]
    pub fn new(
        resource_count: usize,
        scope_count: usize,
        metric_count: usize,
        datapoint_count: usize,
        datapoint_type: MetricType,
        attribute_count: usize,
    ) -> Self {
        Self {
            resource_count,
            scope_count,
            metric_count,
            datapoint_count,
            datapoint_type,
            attribute_count,
        }
    }
    /// Take the metric config and generate the corresponding metric signal
    #[must_use]
    pub fn get_signal(&self) -> MetricsData {
        // check datapoint type
        fake_otlp_metrics(
            self.resource_count,
            self.scope_count,
            self.metric_count,
            self.datapoint_count,
            self.datapoint_type,
            self.attribute_count,
        )
    }
}

impl LogConfig {
    /// create a new config
    #[must_use]
    pub fn new(
        resource_count: usize,
        scope_count: usize,
        log_count: usize,
        attribute_count: usize,
    ) -> Self {
        Self {
            resource_count,
            scope_count,
            log_count,
            attribute_count,
        }
    }
    /// Take the log config and generate the corresponding log signal
    #[must_use]
    pub fn get_signal(&self) -> LogsData {
        fake_otlp_logs(
            self.resource_count,
            self.scope_count,
            self.log_count,
            self.attribute_count,
        )
    }
}

impl SpanConfig {
    /// create a new config
    #[must_use]
    pub fn new(
        resource_count: usize,
        scope_count: usize,
        span_count: usize,
        event_count: usize,
        link_count: usize,
        attribute_count: usize,
    ) -> Self {
        Self {
            resource_count,
            scope_count,
            span_count,
            event_count,
            link_count,
            attribute_count,
        }
    }
    /// Take the traces config and generate the corresponding traces signal
    #[must_use]
    pub fn get_signal(&self) -> TracesData {
        fake_otlp_traces(
            self.resource_count,
            self.scope_count,
            self.span_count,
            self.event_count,
            self.link_count,
            self.attribute_count,
        )
    }
}

/// Enum to represent the OTLP data being sent through the pipeline
#[derive(Debug, Clone)]
pub enum OTLPSignal {
    /// Logs signal
    Log(LogsData),
    /// Metrics signal
    Metric(MetricsData),
    /// Traces signal
    Span(TracesData),
}

#[cfg(test)]
mod tests {
    use crate::fake_signal_receiver::config::*;

    #[test]
    fn test_config() {
        let mut steps = vec![];
        let metric_config = MetricConfig::new(1, 1, 1, 1, MetricType::Gauge, 1);
        let trace_config = SpanConfig::new(1, 1, 1, 1, 1, 1);

        let log_config = LogConfig::new(1, 1, 1, 1);

        steps.push(ScenarioStep::new(SignalConfig::Metric(metric_config), 1, 0));
        steps.push(ScenarioStep::new(SignalConfig::Span(trace_config), 1, 0));
        steps.push(ScenarioStep::new(SignalConfig::Log(log_config), 1, 0));

        let config = Config::new("config".to_string(), steps);
        // Convert the Point to a JSON string.
        let serialized = serde_json::to_string(&config).unwrap();
        println!("{}", serialized);
    }
}
