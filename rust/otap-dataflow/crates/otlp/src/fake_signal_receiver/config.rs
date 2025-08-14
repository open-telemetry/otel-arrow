// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the fake signal receiver
//!

use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::LogsData, metrics::v1::MetricsData, trace::v1::TracesData,
};
use serde::{Deserialize, Serialize};

use weaver_common::result::WResult;
use weaver_common::vdir::VirtualDirectoryPath;
use weaver_forge::registry::ResolvedRegistry;
use weaver_resolver::SchemaResolver;
use weaver_semconv::registry::SemConvRegistry;
use weaver_semconv::registry_repo::RegistryRepo;

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
    #[serde(default = "default_registry_path")]
    registry_path: VirtualDirectoryPath,
}

/// Configuration to describe the traffic being sent
#[derive(Clone, Deserialize, Serialize)]
pub struct TrafficConfig {
    #[serde(default = "default_signals_per_second")]
    signals_per_second: usize,
    #[serde(default = "default_weight")]
    metric_weight: u32,
    #[serde(default = "default_weight")]
    trace_weight: u32,
    #[serde(default = "default_weight")]
    log_weight: u32,
}

impl Config {
    /// Create a new config given a name and a vector of scenario steps
    #[must_use]
    pub fn new(traffic_config: TrafficConfig, registry_path: VirtualDirectoryPath) -> Self {
        Self {
            traffic_config,
            registry_path,
        }
    }
    /// Provide a reference to the vector of scenario steps
    #[must_use]
    pub fn get_traffic_config(&self) -> &TrafficConfig {
        &self.traffic_config
    }
    /// Provide a reference to the ResolvedRegistry
    #[must_use]
    pub fn get_registry(&self) -> Result<ResolvedRegistry, String> {
        let registry_repo =
            RegistryRepo::try_new("main", &self.registry_path).map_err(|err| err.to_string())?;

        // Load the semantic convention specs
        let semconv_specs = match SchemaResolver::load_semconv_specs(&registry_repo, true, false) {
            WResult::Ok(semconv_specs) => semconv_specs,
            WResult::OkWithNFEs(semconv_specs, _) => semconv_specs,
            WResult::FatalErr(err) => return Err(err.to_string()),
        };

        // Resolve the main registry
        let mut registry = SemConvRegistry::from_semconv_specs(&registry_repo, semconv_specs)
            .map_err(|err| err.to_string())?;
        // Resolve the semantic convention specifications.
        // If there are any resolution errors, they should be captured into the ongoing list of
        // diagnostic messages and returned immediately because there is no point in continuing
        // as the resolution is a prerequisite for the next stages.
        let resolved_schema =
            match SchemaResolver::resolve_semantic_convention_registry(&mut registry, true) {
                WResult::Ok(resolved_schema) => resolved_schema,
                WResult::OkWithNFEs(resolved_schema, _) => resolved_schema,
                WResult::FatalErr(err) => return Err(err.to_string()),
            };

        let resolved_registry = ResolvedRegistry::try_from_resolved_registry(
            &resolved_schema.registry,
            resolved_schema.catalog(),
        )
        .map_err(|err| err.to_string())?;

        Ok(resolved_registry)
    }
}

impl TrafficConfig {
    /// create a new traffic config which describes the output traffic of the receiver
    #[must_use]
    pub fn new(
        signals_per_second: usize,
        metric_weight: u32,
        trace_weight: u32,
        log_weight: u32,
    ) -> Self {
        Self {
            signals_per_second,
            metric_weight,
            trace_weight,
            log_weight,
        }
    }

    /// return the specified message rate
    #[must_use]
    pub fn get_message_rate(&self) -> usize {
        self.signals_per_second
    }

    /// get the config describing how big the metric signal is
    #[must_use]
    pub fn calculate_signal_count(&self) -> (usize, usize, usize) {
        // ToDo: Handle case where the total signal count don't add up the the signals being sent per second
        let total_weight: f32 = (self.trace_weight + self.metric_weight + self.log_weight) as f32;

        let metric_percent: f32 = self.metric_weight as f32 / total_weight;
        let trace_percent: f32 = self.trace_weight as f32 / total_weight;
        let log_percent: f32 = self.log_weight as f32 / total_weight;

        let metric_count: usize = (metric_percent * self.signals_per_second as f32) as usize;
        let trace_count: usize = (trace_percent * self.signals_per_second as f32) as usize;
        let log_count: usize = (log_percent * self.signals_per_second as f32) as usize;
        (metric_count, trace_count, log_count)
    }
}

fn default_signals_per_second() -> usize {
    30
}

fn default_weight() -> u32 {
    0
}

fn default_registry_path() -> VirtualDirectoryPath {
    VirtualDirectoryPath::GitRepo {
        url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
        sub_folder: Some("model".to_owned()),
        refspec: None,
    }
}
