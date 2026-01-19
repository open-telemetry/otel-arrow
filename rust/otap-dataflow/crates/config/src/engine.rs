// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The configuration for the dataflow engine.

use crate::error::{Context, Error};
use crate::observed_state::ObservedStateSettings;
use crate::pipeline::PipelineConfig;
use crate::pipeline::service::telemetry::TelemetryConfig;
use crate::pipeline_group::PipelineGroupConfig;
use crate::{PipelineGroupId, PipelineId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root configuration for the pipeline engine.
/// Contains engine-level settings and all pipeline groups.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EngineConfig {
    /// Settings that apply to the entire engine instance.
    pub settings: EngineSettings,

    /// All pipeline group managed by this engine, keyed by pipeline group ID.
    pub pipeline_groups: HashMap<PipelineGroupId, PipelineGroupConfig>,
}

/// Global settings for the engine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct EngineSettings {
    /// Optional HTTP admin server configuration.
    pub http_admin: Option<HttpAdminSettings>,

    /// Telemetry backend configuration shared across pipelines.
    #[serde(default)]
    pub telemetry: TelemetryConfig,

    /// Observed state store settings shared across pipelines.
    #[serde(default)]
    pub observed_state: ObservedStateSettings,
}

/// Configuration for the HTTP admin endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
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

    /// Creates a new `EngineConfig` with the given YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, Error> {
        let config: EngineConfig =
            serde_yaml::from_str(yaml).map_err(|e| Error::DeserializationError {
                context: Context::default(),
                format: "YAML".to_string(),
                details: e.to_string(),
            })?;
        config.validate()?;
        Ok(config)
    }

    /// Load an [`EngineConfig`] from a JSON file.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: Context::default(),
            details: e.to_string(),
        })?;
        Self::from_json(&contents)
    }

    /// Load an [`EngineConfig`] from a YAML file.
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: Context::default(),
            details: e.to_string(),
        })?;
        Self::from_yaml(&contents)
    }

    /// Load an [`EngineConfig`] from a file, automatically detecting the format based on file extension.
    ///
    /// Supports:
    /// - JSON files: `.json`
    /// - YAML files: `.yaml`, `.yml`
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("json") => Self::from_json_file(path),
            Some("yaml") | Some("yml") => Self::from_yaml_file(path),
            _ => {
                let details = format!(
                    "Unsupported file extension: {}. Supported extensions are: .json, .yaml, .yml",
                    extension.unwrap_or_else(|| "<none>".to_string())
                );
                Err(Error::FileReadError {
                    context: Context::default(),
                    details,
                })
            }
        }
    }

    /// Creates a new `EngineConfig` from a single pipeline definition.
    pub fn from_pipeline(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        pipeline: PipelineConfig,
        settings: EngineSettings,
    ) -> Result<Self, Error> {
        let mut pipeline_group = PipelineGroupConfig::new();
        pipeline_group.add_pipeline(pipeline_id, pipeline)?;
        let mut pipeline_groups = HashMap::new();
        let _ = pipeline_groups.insert(pipeline_group_id, pipeline_group);
        let config = EngineConfig {
            settings,
            pipeline_groups,
        };
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
