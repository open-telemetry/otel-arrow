// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the fake signal receiver
//!

use crate::fake_signal_receiver::fake_signal::{
    fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
};
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::AnyValue, logs::v1::LogsData, metrics::v1::MetricsData, trace::v1::TracesData,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use weaver_forge::registry::ResolvedRegistry;

/// Configuration should take a scenario to play out
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    // A ordered list of steps defining various signals to emit
    steps: Vec<ScenarioStep>,
    resolved_registry: ResolvedRegistry
}

impl Config {
    /// Create a new config given a name and a vector of scenario steps
    #[must_use]
    pub fn new(steps: Vec<ScenarioStep>, resolved_registry: ResolvedRegistry) -> Self {
        Self { steps, resolved_registry }
    }
    /// Provide a reference to the vector of scenario steps
    #[must_use]
    pub fn get_steps(&self) -> &Vec<ScenarioStep> {
        &self.steps
    }


    pub fn get_registry(&self) -> &Registry {
        &self.resolved_registry
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

// #[cfg(test)]
// mod tests {
//     use crate::fake_signal_receiver::config::*;

//     #[test]
//     fn test_config() {
//         let mut steps = vec![];
//         let metric_attributes = default_metric_attributes();
//         let default_attributes = default_attributes();
//         let datapoint_config = DatapointConfig::new(3, vec![DatapointType::Gauge, DatapointType::Histogram], metric_attributes, 50.0, 0.0);
//         let metric_config =
//             MetricConfig::new(1, 1, 1, datapoint_config);
//         let trace_config = SpanConfig::new(
//             1,
//             1,
//             1,
//             default_attributes.clone(),
//             default_span_names(),
//             default_event_config(),
//             default_link_config(),
//         );

//         let log_config = LogConfig::new(1, 1, 1, default_attributes.clone(), default_event_names());

//         steps.push(ScenarioStep::new(SignalConfig::Metric(metric_config), 1, 0));
//         steps.push(ScenarioStep::new(SignalConfig::Span(trace_config), 1, 0));
//         steps.push(ScenarioStep::new(SignalConfig::Log(log_config), 1, 0));

//         let config = Config::new(steps);
//         // Convert the Point to a JSON string.
//         let serialized = serde_json::to_string(&config).unwrap();
//         println!("{serialized}");
//     }
// }
