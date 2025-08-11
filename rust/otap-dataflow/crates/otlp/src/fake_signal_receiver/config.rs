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
    // Configuration of the traffic to generate
    traffic_config: TrafficConfig,

    resolved_registry: ResolvedRegistry,
}

/// Configuration to describe the traffic being sent
#[derive(Clone, Deserialize, Serialize)]
pub struct TrafficConfig {
    #[serde(default = "default_messages_per_second")]
    messages_per_second: usize,
    #[serde(default = "default_load")]
    metric_load: Option<LoadConfig>,
    #[serde(default = "default_load")]
    trace_load: Option<LoadConfig>,
    #[serde(default = "default_load")]
    log_load: Option<LoadConfig>,
}

/// Configuration to describe how large the data should be
#[derive(Clone, Deserialize, Serialize)]
pub struct LoadConfig {
    // number of messages produced
    #[serde(default = "default_message_count")]
    messages_per_scope: usize,
    // number of scope produced
    #[serde(default = "default_scope_count")]
    scopes_per_resource: usize,
    // number of resources produced
    #[serde(default = "default_resource_count")]
    resources_per_request: usize,
}

impl Config {
    /// Create a new config given a name and a vector of scenario steps
    #[must_use]
    pub fn new(traffic_config: TrafficConfig, resolved_registry: ResolvedRegistry) -> Self {
        Self {
            traffic_config,
            resolved_registry,
        }
    }
    /// Provide a reference to the vector of scenario steps
    #[must_use]
    pub fn get_traffic_config(&self) -> &TrafficConfig {
        &self.traffic_config
    }
    /// Provide a reference to the ResolvedRegistry
    #[must_use]
    pub fn get_registry(&self) -> &ResolvedRegistry {
        &self.resolved_registry
    }
}

impl TrafficConfig {
    /// create a new traffic config which describes the output traffic of the receiver
    #[must_use]
    pub fn new(
        messages_per_second: usize,
        metric_load: Option<LoadConfig>,
        trace_load: Option<LoadConfig>,
        log_load: Option<LoadConfig>,
    ) -> Self {
        Self {
            messages_per_second,
            metric_load,
            trace_load,
            log_load,
        }
    }

    /// return the specified message rate
    #[must_use]
    pub fn get_message_rate(&self) -> usize {
        self.messages_per_second
    }

    /// get the config describing how big the metric signal is
    #[must_use]
    pub fn get_metric_load(&self) -> Option<LoadConfig> {
        self.metric_load.clone()
    }

    /// get the config describing how big the trace signal is
    #[must_use]
    pub fn get_trace_load(&self) -> Option<LoadConfig> {
        self.trace_load.clone()
    }

    /// get the config describing how big the log signal is
    #[must_use]
    pub fn get_log_load(&self) -> Option<LoadConfig> {
        self.log_load.clone()
    }

    /// calculate the total message load from all signals
    #[must_use]
    pub fn get_total_message_size(&self) -> usize {
        let mut total_message = 0;

        if let Some(load) = &self.metric_load {
            total_message += load.calculate_total_message_size();
        }

        if let Some(load) = &self.trace_load {
            total_message += load.calculate_total_message_size();
        }

        if let Some(load) = &self.log_load {
            total_message += load.calculate_total_message_size();
        }

        total_message
    }
}

impl LoadConfig {
    /// create a new confg defining the size of the request load
    #[must_use]
    pub fn new(
        resources_per_request: usize,
        scopes_per_resource: usize,
        messages_per_scope: usize,
    ) -> Self {
        Self {
            resources_per_request,
            scopes_per_resource,
            messages_per_scope,
        }
    }

    /// get the number of resources to generate
    #[must_use]
    pub fn get_resources(&self) -> usize {
        self.resources_per_request
    }

    /// get the number of scopes to generate per resource
    #[must_use]
    pub fn get_scopes(&self) -> usize {
        self.scopes_per_resource
    }

    /// get the number of messages to generate per scope
    #[must_use]
    pub fn get_messages(&self) -> usize {
        self.messages_per_scope
    }

    /// calculate the total messages that will be sent with a signal load
    #[must_use]
    pub fn calculate_total_message_size(&self) -> usize {
        self.resources_per_request * self.scopes_per_resource * self.messages_per_scope
    }
}

fn default_messages_per_second() -> usize {
    30
}

fn default_message_count() -> usize {
    10
}

fn default_resource_count() -> usize {
    1
}

fn default_scope_count() -> usize {
    1
}

fn default_load() -> Option<LoadConfig> {
    None
}
