// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The configuration for the dataflow engine.

use crate::error::{Context, Error};
use crate::observed_state::ObservedStateSettings;
use crate::pipeline::telemetry::TelemetryConfig;
use crate::pipeline::{PipelineConfig, PipelineConnection, PipelineNodes};
use crate::pipeline_group::PipelineGroupConfig;
use crate::{PipelineGroupId, PipelineId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Current engine configuration schema version.
pub const ENGINE_CONFIG_VERSION_V1: &str = "otel_dataflow/v1";

/// Root configuration for the pipeline engine.
/// Contains engine-level settings and all pipeline groups.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtelDataflowSpec {
    /// Version of the engine configuration schema.
    pub version: String,

    /// Engine-wide runtime declarations.
    #[serde(default)]
    pub engine: EngineSectionConfig,

    /// All groups managed by this engine, keyed by group ID.
    pub groups: HashMap<PipelineGroupId, PipelineGroupConfig>,
}

/// Top-level engine configuration section.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct EngineSectionConfig {
    /// Optional HTTP admin server configuration.
    pub http_admin: Option<HttpAdminSettings>,

    /// Telemetry backend configuration shared across pipelines.
    #[serde(default)]
    pub telemetry: TelemetryConfig,

    /// Observed state store settings shared across pipelines.
    #[serde(default)]
    pub observed_state: ObservedStateSettings,

    /// Engine observability declarations.
    #[serde(default)]
    pub observability: EngineObservabilityConfig,
}

/// Engine observability declarations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct EngineObservabilityConfig {
    /// Optional dedicated observability pipeline for the engine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<EngineObservabilityPipelineConfig>,
}

/// Configuration for the dedicated engine observability pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct EngineObservabilityPipelineConfig {
    /// Nodes of the observability pipeline.
    #[serde(default)]
    pub nodes: PipelineNodes,

    /// Explicit graph connections for observability nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<PipelineConnection>,
}

impl EngineObservabilityPipelineConfig {
    /// Converts this config into a runtime [`PipelineConfig`].
    #[must_use]
    pub fn into_pipeline_config(self) -> PipelineConfig {
        PipelineConfig::for_observability_pipeline(self.nodes, self.connections)
    }
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
        engine: EngineSectionConfig,
    ) -> Result<Self, Error> {
        let mut pipeline_group = PipelineGroupConfig::new();
        pipeline_group.add_pipeline(pipeline_id, pipeline)?;
        let mut groups = HashMap::new();
        let _ = groups.insert(pipeline_group_id, pipeline_group);
        let config = OtelDataflowSpec {
            version: ENGINE_CONFIG_VERSION_V1.to_string(),
            engine,
            groups,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validates the engine configuration and returns a [`Error::InvalidConfiguration`] error
    /// containing all validation errors found in the pipeline groups.
    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = Vec::new();

        if self.version != ENGINE_CONFIG_VERSION_V1 {
            errors.push(Error::InvalidUserConfig {
                error: format!(
                    "unsupported engine config version `{}`; expected `{}`",
                    self.version, ENGINE_CONFIG_VERSION_V1
                ),
            });
        }

        if let Some(observability_pipeline) = self.engine.observability.pipeline.clone() {
            if observability_pipeline.nodes.is_empty() {
                errors.push(Error::InvalidUserConfig {
                    error: "engine.observability.pipeline.nodes must not be empty".to_owned(),
                });
            } else {
                let pipeline_cfg = observability_pipeline.into_pipeline_config();
                if let Err(e) = pipeline_cfg.validate(&"engine".into(), &"observability".into()) {
                    errors.push(e);
                }
            }
        }

        for (pipeline_group_id, pipeline_group) in &self.groups {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn valid_engine_yaml(version: &str) -> String {
        format!(
            r#"
version: {version}
engine: {{}}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:example:receiver"
            config: null
          exporter:
            type: "urn:test:example:exporter"
            config: null
        connections:
          - from: receiver
            to: exporter
"#
        )
    }

    fn write_temp_file(ext: &str, contents: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "otap-df-config-engine-tests-{}-{}.{}",
            std::process::id(),
            suffix,
            ext
        ));
        fs::write(&path, contents).expect("failed to write temporary test file");
        path
    }

    #[test]
    fn from_yaml_requires_version_field() {
        let yaml = r#"
engine: {}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:example:receiver"
            config: null
          exporter:
            type: "urn:test:example:exporter"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let err = OtelDataflowSpec::from_yaml(yaml).unwrap_err();
        match err {
            Error::DeserializationError { details, .. } => {
                assert!(details.contains("missing field `version`"));
            }
            other => panic!("expected deserialization error, got: {other:?}"),
        }
    }

    #[test]
    fn from_yaml_accepts_supported_version() {
        let yaml = valid_engine_yaml(ENGINE_CONFIG_VERSION_V1);
        let config = OtelDataflowSpec::from_yaml(&yaml).expect("v1 config should be accepted");
        assert_eq!(config.version, ENGINE_CONFIG_VERSION_V1);
    }

    #[test]
    fn from_yaml_rejects_unsupported_version() {
        let yaml = valid_engine_yaml("otel_dataflow/v2");
        let err = OtelDataflowSpec::from_yaml(&yaml).unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported engine config version `otel_dataflow/v2`")
        );
    }

    #[test]
    fn from_yaml_accepts_observability_pipeline() {
        let yaml = r#"
version: otel_dataflow/v1
engine:
  observability:
    pipeline:
      nodes:
        itr:
          type: "urn:otel:internal_telemetry:receiver"
          config: {}
        sink:
          type: "urn:otel:console:exporter"
          config: {}
      connections:
        - from: itr
          to: sink
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:example:receiver"
            config: null
          exporter:
            type: "urn:test:example:exporter"
            config: null
        connections:
          - from: receiver
            to: exporter
"#;

        let config = OtelDataflowSpec::from_yaml(yaml).expect("should parse");
        assert!(config.engine.observability.pipeline.is_some());
    }

    #[test]
    fn from_json_file_nonexistent_file() {
        let result = OtelDataflowSpec::from_json_file("/nonexistent/path/spec.json");

        assert!(result.is_err());
        match result {
            Err(Error::FileReadError { .. }) => {}
            other => panic!("Expected FileReadError, got {other:?}"),
        }
    }

    #[test]
    fn from_yaml_file_nonexistent_file() {
        let result = OtelDataflowSpec::from_yaml_file("/nonexistent/path/spec.yaml");

        assert!(result.is_err());
        match result {
            Err(Error::FileReadError { .. }) => {}
            other => panic!("Expected FileReadError, got {other:?}"),
        }
    }

    #[test]
    fn from_file_yml_extension() {
        let result = OtelDataflowSpec::from_file("/nonexistent/spec.yml");

        assert!(result.is_err());
        // Should be a file read error (nonexistent), not an extension error.
        match result {
            Err(Error::FileReadError { details, .. }) => {
                assert!(!details.contains("Unsupported file extension"));
            }
            other => panic!("Expected FileReadError, got {other:?}"),
        }
    }

    #[test]
    fn from_file_unsupported_extension() {
        let result = OtelDataflowSpec::from_file("/some/path/spec.txt");

        assert!(result.is_err());
        match result {
            Err(Error::FileReadError { details, .. }) => {
                assert!(details.contains("Unsupported file extension"));
                assert!(details.contains("txt"));
                assert!(details.contains(".json, .yaml, .yml"));
            }
            other => panic!("Expected FileReadError with unsupported extension, got {other:?}"),
        }
    }

    #[test]
    fn from_file_no_extension() {
        let result = OtelDataflowSpec::from_file("/some/path/spec");

        assert!(result.is_err());
        match result {
            Err(Error::FileReadError { details, .. }) => {
                assert!(details.contains("Unsupported file extension"));
                assert!(details.contains("<none>"));
                assert!(details.contains(".json, .yaml, .yml"));
            }
            other => panic!("Expected FileReadError with no extension, got {other:?}"),
        }
    }

    #[test]
    fn from_json_file_reads_valid_spec() {
        let yaml = valid_engine_yaml(ENGINE_CONFIG_VERSION_V1);
        let model = OtelDataflowSpec::from_yaml(&yaml).expect("fixture yaml should parse");
        let json = serde_json::to_string(&model).expect("fixture should serialize to json");
        let path = write_temp_file("json", &json);

        let result = OtelDataflowSpec::from_json_file(&path);
        let _ = fs::remove_file(&path);

        assert!(result.is_ok());
        let parsed = result.expect("json file should parse");
        assert_eq!(parsed.version, ENGINE_CONFIG_VERSION_V1);
        assert!(parsed.groups.contains_key("default"));
    }

    #[test]
    fn from_file_json_extension() {
        let yaml = valid_engine_yaml(ENGINE_CONFIG_VERSION_V1);
        let model = OtelDataflowSpec::from_yaml(&yaml).expect("fixture yaml should parse");
        let json = serde_json::to_string(&model).expect("fixture should serialize to json");
        let path = write_temp_file("json", &json);

        let result = OtelDataflowSpec::from_file(&path);
        let _ = fs::remove_file(&path);

        assert!(result.is_ok());
        let parsed = result.expect("json file should parse");
        assert_eq!(parsed.version, ENGINE_CONFIG_VERSION_V1);
        assert!(parsed.groups.contains_key("default"));
    }

    #[test]
    fn bundled_configs_parse_as_engine_configs() {
        let mut dirs = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs")];
        while let Some(dir) = dirs.pop() {
            for entry in fs::read_dir(&dir).unwrap_or_else(|e| {
                panic!("failed to read configs directory {}: {e}", dir.display())
            }) {
                let path = entry.expect("failed to read dir entry").path();
                if path.is_dir() {
                    dirs.push(path);
                    continue;
                }

                let is_yaml = matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yaml" | "yml")
                );
                if !is_yaml {
                    continue;
                }

                let parsed = OtelDataflowSpec::from_file(&path);
                assert!(
                    parsed.is_ok(),
                    "failed to parse engine config {}: {parsed:?}",
                    path.display()
                );
            }
        }
    }
}
