// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The configuration for the dataflow engine.

mod io;
mod resolve;
mod validate;

use crate::PipelineGroupId;
use crate::health::HealthPolicy;
use crate::observed_state::ObservedStateSettings;
use crate::pipeline::telemetry::TelemetryConfig;
use crate::pipeline::{PipelineConfig, PipelineConnection, PipelineNodes};
use crate::pipeline_group::PipelineGroupConfig;
use crate::policy::{ChannelCapacityPolicy, Policies, ResourcesPolicy, TelemetryPolicy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use self::resolve::{
    OBSERVABILITY_INTERNAL_PIPELINE_GROUP_ID, OBSERVABILITY_INTERNAL_PIPELINE_ID,
    ResolvedOtelDataflowSpec, ResolvedPipelineConfig, ResolvedPipelineRole,
};

#[cfg(test)]
use crate::error::Error;

/// Current engine configuration schema version.
pub const ENGINE_CONFIG_VERSION_V1: &str = "otel_dataflow/v1";

/// Root configuration for the pipeline engine.
/// Contains engine-level settings and all pipeline groups.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtelDataflowSpec {
    /// Version of the engine configuration schema.
    pub version: String,

    /// Top-level policy set.
    #[serde(default)]
    pub policies: Policies,

    /// Engine-wide runtime declarations.
    #[serde(default)]
    pub engine: EngineConfig,

    /// All groups managed by this engine, keyed by group ID.
    pub groups: HashMap<PipelineGroupId, PipelineGroupConfig>,
}

/// Top-level engine configuration section.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct EngineConfig {
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
    /// Optional policy set for this observability pipeline.
    ///
    /// Note: resources policy is intentionally unsupported for observability for now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policies: Option<EngineObservabilityPolicies>,

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
        PipelineConfig::for_observability_pipeline(
            self.policies
                .map(EngineObservabilityPolicies::into_policies),
            self.nodes,
            self.connections,
        )
    }
}

/// Policy declarations allowed on the dedicated engine observability pipeline.
///
/// Note: `resources` is intentionally not supported yet for observability.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct EngineObservabilityPolicies {
    /// Channel capacity policy.
    #[serde(default)]
    pub channel_capacity: ChannelCapacityPolicy,
    /// Health policy used by observed-state liveness/readiness evaluation.
    #[serde(default)]
    pub health: HealthPolicy,
    /// Runtime telemetry policy controlling pipeline-local metric collection.
    #[serde(default)]
    pub telemetry: TelemetryPolicy,
}

impl EngineObservabilityPolicies {
    #[must_use]
    fn into_policies(self) -> Policies {
        Policies {
            channel_capacity: self.channel_capacity,
            health: self.health,
            telemetry: self.telemetry,
            resources: ResourcesPolicy::default(),
        }
    }

    #[must_use]
    pub(crate) fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        self.clone().into_policies().validation_errors(path_prefix)
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
    fn from_yaml_uses_default_top_level_channel_capacity_policy() {
        let yaml = valid_engine_yaml(ENGINE_CONFIG_VERSION_V1);
        let config = OtelDataflowSpec::from_yaml(&yaml).expect("should parse");
        assert_eq!(config.policies.channel_capacity.control.node, 256);
        assert_eq!(config.policies.channel_capacity.control.pipeline, 256);
        assert_eq!(config.policies.channel_capacity.pdata, 128);
        assert_eq!(config.policies.health, HealthPolicy::default());
        assert!(config.policies.telemetry.pipeline_metrics);
        assert!(config.policies.telemetry.tokio_metrics);
        assert!(config.policies.telemetry.channel_metrics);
        assert_eq!(
            config.policies.resources.core_allocation,
            crate::policy::CoreAllocation::AllCores
        );
    }

    #[test]
    fn resolve_channel_capacity_policy_respects_scope_precedence() {
        let yaml = r#"
version: otel_dataflow/v1
policies:
  channel_capacity:
      control:
        node: 200
        pipeline: 201
      pdata: 202
  health:
    ready_if: [Running]
  telemetry:
    channel_metrics: false
  resources:
    core_allocation:
      type: core_count
      count: 9
engine: {}
groups:
  g1:
    policies:
      channel_capacity:
          control:
            node: 150
            pipeline: 151
          pdata: 152
      health:
        ready_if: [Running, Updating]
      telemetry:
        channel_metrics: true
      resources:
        core_allocation:
          type: core_count
          count: 5
    pipelines:
      p1:
        policies:
          channel_capacity:
              control:
                node: 50
                pipeline: 51
              pdata: 52
          health:
            ready_if: [Failed]
          telemetry:
            channel_metrics: false
          resources:
            core_allocation:
              type: core_count
              count: 2
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
      p2:
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
  g2:
    pipelines:
      p3:
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

        let resolved = config.resolve();
        assert_eq!(resolved.pipelines.len(), 3);
        assert!(
            resolved
                .pipelines
                .iter()
                .all(|p| p.role == ResolvedPipelineRole::Regular)
        );
        let resolved_ids: Vec<(String, String)> = resolved
            .pipelines
            .iter()
            .map(|p| (p.pipeline_group_id.to_string(), p.pipeline_id.to_string()))
            .collect();
        assert_eq!(
            resolved_ids,
            vec![
                ("g1".to_string(), "p1".to_string()),
                ("g1".to_string(), "p2".to_string()),
                ("g2".to_string(), "p3".to_string()),
            ]
        );

        let p1_resolved = resolved
            .pipelines
            .iter()
            .find(|p| p.pipeline_group_id.as_ref() == "g1" && p.pipeline_id.as_ref() == "p1")
            .expect("g1/p1 should be resolved");
        assert_eq!(p1_resolved.policies.channel_capacity.control.node, 50);
        assert_eq!(p1_resolved.policies.channel_capacity.control.pipeline, 51);
        assert_eq!(p1_resolved.policies.channel_capacity.pdata, 52);
        assert_eq!(
            p1_resolved.policies.resources.core_allocation,
            crate::policy::CoreAllocation::CoreCount { count: 2 }
        );
        assert_eq!(
            p1_resolved.policies.health.ready_if,
            vec![crate::health::PhaseKind::Failed]
        );
        assert!(!p1_resolved.policies.telemetry.channel_metrics);

        let p2_resolved = resolved
            .pipelines
            .iter()
            .find(|p| p.pipeline_group_id.as_ref() == "g1" && p.pipeline_id.as_ref() == "p2")
            .expect("g1/p2 should be resolved");
        assert_eq!(p2_resolved.policies.channel_capacity.control.node, 150);
        assert_eq!(p2_resolved.policies.channel_capacity.control.pipeline, 151);
        assert_eq!(p2_resolved.policies.channel_capacity.pdata, 152);
        assert_eq!(
            p2_resolved.policies.health.ready_if,
            vec![
                crate::health::PhaseKind::Running,
                crate::health::PhaseKind::Updating,
            ]
        );
        assert!(p2_resolved.policies.telemetry.channel_metrics);
        assert_eq!(
            p2_resolved.policies.resources.core_allocation,
            crate::policy::CoreAllocation::CoreCount { count: 5 }
        );

        let p3_resolved = resolved
            .pipelines
            .iter()
            .find(|p| p.pipeline_group_id.as_ref() == "g2" && p.pipeline_id.as_ref() == "p3")
            .expect("g2/p3 should be resolved");
        assert_eq!(p3_resolved.policies.channel_capacity.control.node, 200);
        assert_eq!(p3_resolved.policies.channel_capacity.control.pipeline, 201);
        assert_eq!(p3_resolved.policies.channel_capacity.pdata, 202);
        assert_eq!(
            p3_resolved.policies.health.ready_if,
            vec![crate::health::PhaseKind::Running]
        );
        assert!(!p3_resolved.policies.telemetry.channel_metrics);
        assert_eq!(
            p3_resolved.policies.resources.core_allocation,
            crate::policy::CoreAllocation::CoreCount { count: 9 }
        );
    }

    #[test]
    fn resolve_observability_channel_capacity_policy_overrides_top_level() {
        let yaml = r#"
version: otel_dataflow/v1
policies:
  channel_capacity:
      control:
        node: 200
        pipeline: 201
      pdata: 202
  health:
    ready_if: [Running]
  telemetry:
    channel_metrics: false
engine:
  observability:
    pipeline:
      policies:
        channel_capacity:
            control:
              node: 10
              pipeline: 11
            pdata: 12
        health:
          ready_if: [Failed]
        telemetry:
          channel_metrics: true
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
        let resolved = config.resolve();
        let obs = resolved
            .pipelines
            .iter()
            .find(|p| p.role == ResolvedPipelineRole::ObservabilityInternal)
            .expect("observability pipeline should be resolved");
        assert_eq!(obs.pipeline_group_id.as_ref(), "internal");
        assert_eq!(obs.pipeline_id.as_ref(), "internal");
        assert_eq!(obs.policies.channel_capacity.control.node, 10);
        assert_eq!(obs.policies.channel_capacity.control.pipeline, 11);
        assert_eq!(obs.policies.channel_capacity.pdata, 12);
        assert_eq!(
            obs.policies.health.ready_if,
            vec![crate::health::PhaseKind::Failed]
        );
        assert!(obs.policies.telemetry.channel_metrics);
        assert_eq!(
            obs.policies.resources.core_allocation,
            crate::policy::CoreAllocation::AllCores
        );
        assert_eq!(
            resolved
                .pipelines
                .iter()
                .filter(|p| p.role == ResolvedPipelineRole::ObservabilityInternal)
                .count(),
            1
        );
        let main = resolved
            .pipelines
            .iter()
            .find(|p| p.role == ResolvedPipelineRole::Regular)
            .expect("main pipeline should be resolved");
        assert_eq!(main.pipeline_group_id.as_ref(), "default");
        assert_eq!(main.pipeline_id.as_ref(), "main");
    }

    #[test]
    fn from_yaml_rejects_observability_resources_policy() {
        let yaml = r#"
version: otel_dataflow/v1
engine:
  observability:
    pipeline:
      policies:
        resources:
          core_allocation:
            type: core_count
            count: 2
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

        let err = OtelDataflowSpec::from_yaml(yaml).expect_err("should reject resources");
        match err {
            Error::DeserializationError { details, .. } => {
                assert!(details.contains("unknown field `resources`"));
            }
            other => panic!("expected deserialization error, got: {other:?}"),
        }
    }

    #[test]
    fn from_yaml_rejects_zero_policy_capacities() {
        let yaml = r#"
version: otel_dataflow/v1
policies:
  channel_capacity:
      control:
        node: 0
        pipeline: 0
      pdata: 0
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

        let err = OtelDataflowSpec::from_yaml(yaml).expect_err("zero capacities should fail");
        let rendered = err.to_string();
        assert!(rendered.contains("channel_capacity.control.node"));
        assert!(rendered.contains("channel_capacity.control.pipeline"));
        assert!(rendered.contains("channel_capacity.pdata"));
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
