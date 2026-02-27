// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! I/O and construction entry points for [`OtelDataflowSpec`].

use crate::engine::{ENGINE_CONFIG_VERSION_V1, EngineConfig, OtelDataflowSpec};
use crate::error::{Context, Error};
use crate::pipeline::PipelineConfig;
use crate::pipeline_group::PipelineGroupConfig;
use crate::policy::Policies;
use crate::{PipelineGroupId, PipelineId};
use std::collections::HashMap;
use std::path::Path;

impl OtelDataflowSpec {
    /// Creates a new [`OtelDataflowSpec`] with the given JSON string.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let config: OtelDataflowSpec =
            serde_json::from_str(json).map_err(|e| Error::DeserializationError {
                context: Default::default(),
                format: "JSON".to_string(),
                details: e.to_string(),
            })?;
        config.validate()?;
        Ok(config)
    }

    /// Creates a new [`OtelDataflowSpec`] with the given YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, Error> {
        let config: OtelDataflowSpec =
            serde_yaml::from_str(yaml).map_err(|e| Error::DeserializationError {
                context: Context::default(),
                format: "YAML".to_string(),
                details: e.to_string(),
            })?;
        config.validate()?;
        Ok(config)
    }

    /// Load an [`OtelDataflowSpec`] from a JSON file.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: Context::default(),
            details: e.to_string(),
        })?;
        Self::from_json(&contents)
    }

    /// Load an [`OtelDataflowSpec`] from a YAML file.
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
            context: Context::default(),
            details: e.to_string(),
        })?;
        Self::from_yaml(&contents)
    }

    /// Load an [`OtelDataflowSpec`] from a file, automatically detecting the format based on file extension.
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

    /// Creates a new [`OtelDataflowSpec`] from a single pipeline definition.
    pub fn from_pipeline(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        pipeline: PipelineConfig,
        engine: EngineConfig,
    ) -> Result<Self, Error> {
        let mut pipeline_group = PipelineGroupConfig::new();
        pipeline_group.add_pipeline(pipeline_id, pipeline)?;
        let mut groups = HashMap::new();
        let _ = groups.insert(pipeline_group_id, pipeline_group);
        let config = OtelDataflowSpec {
            version: ENGINE_CONFIG_VERSION_V1.to_string(),
            policies: Policies::default(),
            topics: HashMap::new(),
            engine,
            groups,
        };
        config.validate()?;
        Ok(config)
    }
}
