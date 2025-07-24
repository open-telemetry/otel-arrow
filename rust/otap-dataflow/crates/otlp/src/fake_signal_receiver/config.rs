// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the fake signal receiver
//!

use crate::fake_signal_receiver::fake_signal::{
    fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
};

use crate::fake_signal_receiver::config_defaults::{
    default_attributes, default_event_config, default_event_names, default_link_config,
    default_metric_attributes, default_trace_states, default_span_names, default_datapoint_type, default_top_value, default_bottom_value, default_datapoint_config
};
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::AnyValue, logs::v1::LogsData, metrics::v1::MetricsData, trace::v1::TracesData,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration should take a scenario to play out
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    // A ordered list of steps defining various signals to emit
    steps: Vec<ScenarioStep>,
}

impl Config {
    /// Create a new config given a name and a vector of scenario steps
    #[must_use]
    pub fn new(steps: Vec<ScenarioStep>) -> Self {
        Self { steps }
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
    /// delay in ms
    #[serde(default = "default_delay_between_batches_ms")]
    delay_between_batches_ms: u64,
    #[serde(default = "default_batches_to_generate")]
    batches_to_generate: u64,
    config: SignalConfig,
}

fn default_delay_between_batches_ms() -> u64 {
    0
}

fn default_batches_to_generate() -> u64 {
    1
}

impl ScenarioStep {
    /// create a new step
    #[must_use]
    pub fn new(
        config: SignalConfig,
        batches_to_generate: u64,
        delay_between_batches_ms: u64,
    ) -> Self {
        Self {
            config,
            batches_to_generate,
            delay_between_batches_ms,
        }
    }
    /// return the configuration stored inside the scenario step
    #[must_use]
    pub fn get_config(&self) -> SignalConfig {
        self.config.clone()
    }

    /// return the number of batches to generate
    #[must_use]
    pub fn get_batches_to_generate(&self) -> u64 {
        self.batches_to_generate
    }

    /// return the delay in ms
    #[must_use]
    pub fn get_delay_between_batches_ms(&self) -> u64 {
        self.delay_between_batches_ms
    }
}

/// configs to describe the data being generated
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "signal_type")]
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
pub enum DatapointType {
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
#[derive(Clone, Deserialize, Serialize)]
pub struct MetricConfig {
    resource_count: usize,
    scope_count: usize,
    metric_count: usize,
    #[serde(default = "default_datapoint_config")]
    datapoints: DatapointConfig,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DatapointConfig {
    datapoint_count: usize,
    #[serde(default = "default_datapoint_type")]
    datapoint_type: Vec<DatapointType>,
            #[serde(default = "default_metric_attributes")]
    attributes: HashMap<String, Vec<AttributeValue>>, 
            #[serde(default = "default_top_value")]
    top_value: f64,
      #[serde(default = "default_bottom_value")]
    bottom_value: f64,
}


impl DatapointConfig {
    #[must_use]
    pub fn new(
        datapoint_count: usize,
        datapoint_type: Vec<DatapointType>,
        attributes: HashMap<String, Vec<AttributeValue>>,
        top_value: f64,
      
        bottom_value: f64,
    ) -> Self {
        Self {
            datapoint_count,
            datapoint_type,
            attributes,
            top_value,
            bottom_value,
        }
    }
}

/// configuration settings for a fake log signal
#[derive(Clone, Deserialize, Serialize)]
pub struct LogConfig {
    resource_count: usize,
    scope_count: usize,
    log_count: usize,
    #[serde(default = "default_attributes")]
    attributes: HashMap<String, Vec<AttributeValue>>,
    #[serde(default = "default_event_names")]
    event_names: Vec<String>,
}
/// configuration settings for a fake trace signal
#[derive(Clone, Deserialize, Serialize)]
pub struct SpanConfig {
    resource_count: usize,
    scope_count: usize,
    span_count: usize,
    #[serde(default = "default_attributes")]
    attributes: HashMap<String, Vec<AttributeValue>>,
    #[serde(default = "default_span_names")]
    span_names: Vec<String>,
    #[serde(default = "default_event_config")]
    events: EventConfig,
    #[serde(default = "default_link_config")]
    links: LinkConfig,
}

impl MetricConfig {
    /// create a new config
    #[must_use]
    pub fn new(
        resource_count: usize,
        scope_count: usize,
        metric_count: usize,
        datapoints: DatapointConfig,
    ) -> Self {
        Self {
            resource_count,
            scope_count,
            metric_count,
            datapoints,
        }
    }
    /// Take the metric config and generate the corresponding metric signal
    #[must_use]
    pub fn get_signal(&self) -> MetricsData {
        // check datapoint type
        let hash: HashMap<String, Vec<AttributeValue>> = HashMap::new();
        fake_otlp_metrics(
            self.resource_count,
            self.scope_count,
            self.metric_count,
            0,
            DatapointType::Gauge,
            &hash
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
        attributes: HashMap<String, Vec<AttributeValue>>,
        event_names: Vec<String>,
    ) -> Self {
        Self {
            resource_count,
            scope_count,
            log_count,
            attributes,
            event_names,
        }
    }
    /// Take the log config and generate the corresponding log signal
    #[must_use]
    pub fn get_signal(&self) -> LogsData {
        fake_otlp_logs(
            self.resource_count,
            self.scope_count,
            self.log_count,
            &self.attributes,
            &self.event_names,
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
        attributes: HashMap<String, Vec<AttributeValue>>,
        span_names: Vec<String>,
        events: EventConfig,
        links: LinkConfig,
    ) -> Self {
        Self {
            resource_count,
            scope_count,
            span_count,
            attributes,
            span_names,
            events,
            links,
        }
    }
    /// Take the traces config and generate the corresponding traces signal
    #[must_use]
    pub fn get_signal(&self) -> TracesData {
        fake_otlp_traces(
            self.resource_count,
            self.scope_count,
            self.span_count,
            &self.attributes,
            &self.span_names,
            &self.events,
            &self.links,
        )
    }
}

/// configuration settings for a
#[derive(Clone, Deserialize, Serialize)]
pub struct EventConfig {
    event_count: usize,
    #[serde(default = "default_event_names")]
    event_names: Vec<String>,
    #[serde(default = "default_attributes")]
    attributes: HashMap<String, Vec<AttributeValue>>,
}

/// configuration settings for a
#[derive(Clone, Deserialize, Serialize)]
pub struct LinkConfig {
    link_count: usize,
    #[serde(default = "default_trace_states")]
    trace_states: Vec<String>,
    #[serde(default = "default_attributes")]
    attributes: HashMap<String, Vec<AttributeValue>>,
}

/// ToDo: Define serializable AnyValue enum
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Int(i64),
    Double(f64),
}

impl AttributeValue {
    #[must_use]
    pub fn convert_anyvalue(&self) -> AnyValue {
        match &self {
            AttributeValue::String(value) => AnyValue::new_string(value),
            AttributeValue::Bool(value) => AnyValue::new_bool(*value),
            AttributeValue::Int(value) => AnyValue::new_int(*value),
            AttributeValue::Double(value) => AnyValue::new_double(*value),
        }
    }
}

impl LinkConfig {
    /// create a new config
    #[must_use]
    pub fn new(
        link_count: usize,
        trace_states: Vec<String>,
        attributes: HashMap<String, Vec<AttributeValue>>,
    ) -> Self {
        Self {
            link_count,
            trace_states,
            attributes,
        }
    }

    #[must_use]
    pub fn get_count(&self) -> usize {
        self.link_count
    }

    #[must_use]
    pub fn get_trace_states(&self) -> &Vec<String> {
        &self.trace_states
    }

    #[must_use]
    pub fn get_attributes(&self) -> &HashMap<String, Vec<AttributeValue>> {
        &self.attributes
    }
}

impl EventConfig {
    /// create a new config
    #[must_use]
    pub fn new(
        event_count: usize,
        event_names: Vec<String>,
        attributes: HashMap<String, Vec<AttributeValue>>,
    ) -> Self {
        Self {
            event_count,
            event_names,
            attributes,
        }
    }

    #[must_use]
    pub fn get_count(&self) -> usize {
        self.event_count
    }

    #[must_use]
    pub fn get_event_names(&self) -> &Vec<String> {
        &self.event_names
    }

    #[must_use]
    pub fn get_attributes(&self) -> &HashMap<String, Vec<AttributeValue>> {
        &self.attributes
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
        let metric_attributes = default_metric_attributes();
        let default_attributes = default_attributes();
        let datapoint_config = DatapointConfig::new(3, vec![DatapointType::Gauge, DatapointType::Histogram], metric_attributes, 50.0, 0.0);
        let metric_config =
            MetricConfig::new(1, 1, 1, datapoint_config);
        let trace_config = SpanConfig::new(
            1,
            1,
            1,
            default_attributes.clone(),
            default_span_names(),
            default_event_config(),
            default_link_config(),
        );

        let log_config = LogConfig::new(1, 1, 1, default_attributes.clone(), default_event_names());

        steps.push(ScenarioStep::new(SignalConfig::Metric(metric_config), 1, 0));
        steps.push(ScenarioStep::new(SignalConfig::Span(trace_config), 1, 0));
        steps.push(ScenarioStep::new(SignalConfig::Log(log_config), 1, 0));

        let config = Config::new(steps);
        // Convert the Point to a JSON string.
        let serialized = serde_json::to_string(&config).unwrap();
        println!("{serialized}");
    }
}
