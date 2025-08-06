// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the fake signal receiver
//!

use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::LogsData, metrics::v1::MetricsData, trace::v1::TracesData,
};
use serde::{Deserialize, Serialize};

use weaver_forge::registry::ResolvedRegistry;

/// Temp pdata
#[derive(Clone, Debug)]
pub enum OTLPSignal {
    /// metrics pdata
    Metrics(MetricsData),
    /// traces pdata
    Traces(TracesData),
    /// log pdata
    Logs(LogsData),
}
/// Configuration should take a scenario to play out
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    // A ordered list of steps defining various signals to emit
    steps: Vec<ScenarioStep>,

    resolved_registry: ResolvedRegistry,
}

impl Config {
    /// Create a new config given a name and a vector of scenario steps
    #[must_use]
    pub fn new(steps: Vec<ScenarioStep>, resolved_registry: ResolvedRegistry) -> Self {
        Self {
            steps,
            resolved_registry,
        }
    }
    /// Provide a reference to the vector of scenario steps
    #[must_use]
    pub fn get_steps(&self) -> &Vec<ScenarioStep> {
        &self.steps
    }
    /// Provide a reference to the ResolvedRegistry
    #[must_use]
    pub fn get_registry(&self) -> &ResolvedRegistry {
        &self.resolved_registry
    }
}

/// A scenario step will contain a configuration
#[derive(Clone, Deserialize, Serialize)]
pub struct ScenarioStep {
    /// delay in ms
    #[serde(default = "default_messages_per_second")]
    messages_per_second: u64,
    #[serde(default = "default_batches_to_generate")]
    batches_to_generate: u64,
    signal_type: SignalType,
}

fn default_messages_per_second() -> u64 {
    10
}

fn default_batches_to_generate() -> u64 {
    1
}

impl ScenarioStep {
    /// create a new step
    #[must_use]
    pub fn new(
        signal_type: SignalType,
        batches_to_generate: u64,
        messages_per_second: u64,
    ) -> Self {
        Self {
            signal_type,
            batches_to_generate,
            messages_per_second,
        }
    }
    /// return the signal type stored inside the scenario step
    #[must_use]
    pub fn get_signal_type(&self) -> &SignalType {
        &self.signal_type
    }

    /// return the number of batches to generate
    #[must_use]
    pub fn get_batches_to_generate(&self) -> u64 {
        self.batches_to_generate
    }

    /// return the messages per second
    #[must_use]
    pub fn get_messages_per_second(&self) -> u64 {
        self.messages_per_second
    }
}

/// Struct to describe how large the signal request should be
#[derive(Clone, Deserialize, Serialize)]
pub struct Load {
    resource_count: usize,
    scope_count: usize,
}

impl Load {
    /// create new Load struct
    #[must_use]
    pub fn new(resource_count: usize, scope_count: usize) -> Self {
        Self {
            resource_count,
            scope_count,
        }
    }

    /// Provide a reference to the vector of scenario steps
    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.resource_count
    }
    /// Provide a reference to the vector of scenario steps
    #[must_use]
    pub fn scope_count(&self) -> usize {
        self.scope_count
    }
}

/// Describes what signals to generate and the signal size
#[derive(Clone, Deserialize, Serialize)]
pub enum SignalType {
    /// metrics signals
    Metrics(Load),
    /// logs signals
    Logs(Load),
    /// traces signals
    Traces(Load),
}