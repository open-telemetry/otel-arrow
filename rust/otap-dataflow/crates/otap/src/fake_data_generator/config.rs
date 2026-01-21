// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the fake signal receiver
//!

use serde::{Deserialize, Serialize};

use weaver_common::result::WResult;
use weaver_common::vdir::VirtualDirectoryPath;
use weaver_forge::registry::ResolvedRegistry;
use weaver_resolver::SchemaResolver;
use weaver_semconv::registry::SemConvRegistry;
use weaver_semconv::registry_repo::RegistryRepo;

/// Source of telemetry data schema and attributes
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    /// Use OpenTelemetry semantic conventions registry via weaver
    /// - Fetches and parses semantic conventions from registry_path
    /// - Full attribute coverage based on spec
    /// - Requires network/file access at startup
    #[default]
    SemanticConventions,

    /// Use minimal static hardcoded signals
    /// - No external dependencies or network access
    /// - Fixed set of attributes (e.g., service.name, http.method, etc.)
    Static,
}

/// Strategy for generating telemetry batches
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStrategy {
    /// Generate fresh signals for every batch
    /// - New objects allocated per signal
    /// - Fresh timestamps and unique IDs per signal
    /// - Highest CPU/memory cost per batch
    #[default]
    Fresh,

    /// Pre-generate complete batches at startup, cycle through them unchanged
    /// - Zero per-batch allocation at runtime
    /// - Timestamps and IDs will repeat (stale)
    /// - Lowest CPU cost, maximum throughput
    PreGenerated,

    /// Pre-generate signal templates, clone and update timestamps/IDs per batch
    /// - Templates cloned per batch
    /// - Fresh timestamps and unique IDs
    /// - Moderate CPU cost, good balance
    ///
    /// TODO: Not yet implemented - currently behaves like Fresh
    Templates,
}

/// Configuration should take a scenario to play out
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Configuration of the traffic to generate
    traffic_config: TrafficConfig,

    /// Source of telemetry schema and attributes
    #[serde(default)]
    data_source: DataSource,

    /// Path to the semantic conventions registry (only used when data_source = semantic_conventions)
    #[serde(default = "default_registry_path")]
    registry_path: VirtualDirectoryPath,

    /// Strategy for generating telemetry batches
    #[serde(default)]
    generation_strategy: GenerationStrategy,

    /// Number of batches/templates to pre-generate (only used with PreGenerated or Templates strategy)
    #[serde(default = "default_pool_size")]
    pool_size: usize,
}

/// Configuration to describe the traffic being sent
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrafficConfig {
    #[serde(default = "default_signals_per_second")]
    signals_per_second: Option<usize>,
    #[serde(default = "default_max_signal")]
    max_signal_count: Option<u64>,
    #[serde(default = "default_max_batch_size")]
    max_batch_size: usize,
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
            data_source: DataSource::default(),
            generation_strategy: GenerationStrategy::default(),
            pool_size: default_pool_size(),
        }
    }

    /// Builder-style method to set data source
    #[must_use]
    pub fn with_data_source(mut self, data_source: DataSource) -> Self {
        self.data_source = data_source;
        self
    }

    /// Builder-style method to set generation strategy
    #[must_use]
    pub fn with_generation_strategy(
        mut self,
        generation_strategy: GenerationStrategy,
        pool_size: usize,
    ) -> Self {
        self.generation_strategy = generation_strategy;
        self.pool_size = pool_size;
        self
    }

    /// Get the data source
    #[must_use]
    pub fn data_source(&self) -> &DataSource {
        &self.data_source
    }

    /// Get the generation strategy
    #[must_use]
    pub fn generation_strategy(&self) -> &GenerationStrategy {
        &self.generation_strategy
    }

    /// Get the pool size for pre-generation
    #[must_use]
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }

    /// Provide a reference to the traffic config
    #[must_use]
    pub fn get_traffic_config(&self) -> &TrafficConfig {
        &self.traffic_config
    }

    /// Provide a reference to the ResolvedRegistry.
    /// Returns None if data_source is Static.
    pub fn get_registry(&self) -> Result<Option<ResolvedRegistry>, String> {
        match self.data_source {
            DataSource::Static => Ok(None),
            DataSource::SemanticConventions => {
                let registry_repo = RegistryRepo::try_new("main", &self.registry_path)
                    .map_err(|err| err.to_string())?;

                // Load the semantic convention specs
                let semconv_specs =
                    match SchemaResolver::load_semconv_specs(&registry_repo, true, false) {
                        WResult::Ok(semconv_specs) => semconv_specs,
                        WResult::OkWithNFEs(semconv_specs, _) => semconv_specs,
                        WResult::FatalErr(err) => return Err(err.to_string()),
                    };

                // Resolve the main registry
                let mut registry =
                    SemConvRegistry::from_semconv_specs(&registry_repo, semconv_specs)
                        .map_err(|err| err.to_string())?;
                // Resolve the semantic convention specifications.
                let resolved_schema =
                    match SchemaResolver::resolve_semantic_convention_registry(&mut registry, true)
                    {
                        WResult::Ok(resolved_schema) => resolved_schema,
                        WResult::OkWithNFEs(resolved_schema, _) => resolved_schema,
                        WResult::FatalErr(err) => return Err(err.to_string()),
                    };

                let resolved_registry = ResolvedRegistry::try_from_resolved_registry(
                    &resolved_schema.registry,
                    resolved_schema.catalog(),
                )
                .map_err(|err| err.to_string())?;

                Ok(Some(resolved_registry))
            }
        }
    }
}

impl TrafficConfig {
    /// create a new traffic config which describes the output traffic of the receiver
    #[must_use]
    pub fn new(
        signals_per_second: Option<usize>,
        max_signal_count: Option<u64>,
        max_batch_size: usize,
        metric_weight: u32,
        trace_weight: u32,
        log_weight: u32,
    ) -> Self {
        Self {
            signals_per_second,
            max_signal_count,
            max_batch_size,
            metric_weight,
            trace_weight,
            log_weight,
        }
    }

    /// return the specified message rate
    #[must_use]
    pub fn get_signal_rate(&self) -> Option<usize> {
        self.signals_per_second
    }

    /// get the config describing how big the metric signal is
    #[must_use]
    pub fn calculate_signal_count(&self) -> (usize, usize, usize) {
        if let Some(rate_limit) = self.signals_per_second {
            // ToDo: Handle case where the total signal count don't add up the signals being sent per second
            let total_weight: f32 =
                (self.trace_weight + self.metric_weight + self.log_weight) as f32;

            let metric_percent: f32 = self.metric_weight as f32 / total_weight;
            let trace_percent: f32 = self.trace_weight as f32 / total_weight;
            let log_percent: f32 = self.log_weight as f32 / total_weight;

            let metric_count: usize = (metric_percent * rate_limit as f32) as usize;
            let trace_count: usize = (trace_percent * rate_limit as f32) as usize;
            let log_count: usize = (log_percent * rate_limit as f32) as usize;

            let _remaining_count = rate_limit - (metric_count + trace_count + log_count);
            // ToDo: Update signal count using by distributing the remaining count
            // if remaining_count > 0 {
            //     // we need to add to the remaining signal counts here to the counts

            // }

            (metric_count, trace_count, log_count)
        } else {
            // if no rate limit is set, use max_batch_size distributed by weights
            let total_weight: f32 =
                (self.trace_weight + self.metric_weight + self.log_weight) as f32;

            if total_weight == 0.0 {
                return (0, 0, 0);
            }

            let metric_percent: f32 = self.metric_weight as f32 / total_weight;
            let trace_percent: f32 = self.trace_weight as f32 / total_weight;
            let log_percent: f32 = self.log_weight as f32 / total_weight;

            let metric_count: usize = (metric_percent * self.max_batch_size as f32) as usize;
            let trace_count: usize = (trace_percent * self.max_batch_size as f32) as usize;
            let log_count: usize = (log_percent * self.max_batch_size as f32) as usize;

            (metric_count, trace_count, log_count)
        }
    }

    /// returns the max amounts of signals that should be sent
    #[must_use]
    pub fn get_max_signal_count(&self) -> Option<u64> {
        self.max_signal_count
    }

    /// returns the max batch size per message
    #[must_use]
    pub fn get_max_batch_size(&self) -> usize {
        self.max_batch_size
    }
}

fn default_pool_size() -> usize {
    10
}

fn default_signals_per_second() -> Option<usize> {
    Some(30)
}

fn default_max_signal() -> Option<u64> {
    None
}

fn default_max_batch_size() -> usize {
    1000
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
