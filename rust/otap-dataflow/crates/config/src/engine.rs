// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The configuration for the dataflow engine.

use crate::PipelineGroupId;
use crate::error::Error;
use crate::pipeline_group::PipelineGroupConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration for the pipeline engine.
/// Contains engine-level settings and all pipeline groups.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EngineConfig {
    /// Settings that apply to the entire engine instance.
    pub settings: EngineSettings,

    /// All pipeline group managed by this engine, keyed by pipeline group ID.
    pub pipeline_groups: HashMap<PipelineGroupId, PipelineGroupConfig>,
}

/// Global settings for the engine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EngineSettings {
    /// Optional HTTP admin server configuration.
    pub http_admin: Option<HttpAdminSettings>,

    /// Telemetry settings.
    pub telemetry: TelemetrySettings,
}

/// Configuration for the telemetry metrics system.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TelemetrySettings {
    /// The size of the reporting channel.
    #[serde(default = "default_reporting_channel_size")]
    pub reporting_channel_size: usize,
    /// The interval at which metrics are flushed and aggregated by the collector.
    #[serde(default = "default_reporting_interval")]
    pub flush_interval: std::time::Duration,
}

/// Configuration for the HTTP admin endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HttpAdminSettings {
    /// The address to bind the HTTP server to (e.g., "127.0.0.1:8080").
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
}

impl Default for HttpAdminSettings {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
        }
    }
}

const fn default_reporting_channel_size() -> usize {
    100
}

const fn default_reporting_interval() -> std::time::Duration {
    std::time::Duration::from_secs(1)
}

fn default_bind_address() -> String {
    "127.0.0.1:8080".into()
}

impl EngineConfig {
    /// Creates a new `EngineConfig` with the given JSON string.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let config: EngineConfig =
            serde_json::from_str(json).map_err(|e| Error::DeserializationError {
                context: Default::default(),
                format: "JSON".to_string(),
                details: e.to_string(),
            })?;
        config.validate()?;
        Ok(config)
    }

    /// Validates the engine configuration and returns a [`Error::InvalidConfiguration`] error
    /// containing all validation errors found in the pipeline groups.
    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = Vec::new();

        for (pipeline_group_id, pipeline_group) in &self.pipeline_groups {
            if let Err(e) = pipeline_group.validate(pipeline_group_id) {
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            Err(Error::InvalidConfiguration { errors })
        } else {
            Ok(())
        }
    }
}
