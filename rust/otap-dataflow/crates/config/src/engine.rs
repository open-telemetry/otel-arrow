// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The configuration for the dataflow engine.

use crate::error::{Context, Error};
use crate::health::HealthPolicy;
use crate::observed_state::ObservedStateSettings;
use crate::pipeline::telemetry::TelemetryConfig;
use crate::pipeline::{PipelineConfig, PipelineConnection, PipelineNodes};
use crate::pipeline_group::PipelineGroupConfig;
use crate::policy::{FlowPolicy, Policies, ResourcesPolicy, TelemetryPolicy};
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
    /// Flow-related policies.
    #[serde(default)]
    pub flow: FlowPolicy,
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
            flow: self.flow,
            health: self.health,
            telemetry: self.telemetry,
            resources: ResourcesPolicy::default(),
        }
    }

    #[must_use]
    fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        self.clone().into_policies().validation_errors(path_prefix)
    }
}

/// Resolved policy snapshot for an engine configuration.
///
/// This is a deterministic snapshot computed from [`OtelDataflowSpec`]
/// using hierarchy precedence rules.
#[derive(Debug, Clone)]
pub struct ResolvedOtelDataflowSpec {
    /// Engine-wide runtime declarations.
    pub engine: EngineConfig,
    /// Resolved pipeline configurations for all regular pipelines.
    pub pipelines: Vec<ResolvedPipelineConfig>,
    /// Resolved observability pipeline when configured.
    pub observability_pipeline: Option<ResolvedObservabilityPipeline>,
}

/// Resolved data for one regular pipeline.
#[derive(Debug, Clone)]
pub struct ResolvedPipelineConfig {
    /// Pipeline group identifier.
    pub pipeline_group_id: PipelineGroupId,
    /// Pipeline identifier.
    pub pipeline_id: PipelineId,
    /// Pipeline definition.
    pub pipeline: PipelineConfig,
    /// Resolved policies after hierarchy resolution.
    pub policies: Policies,
}

/// Resolved data for the observability pipeline.
#[derive(Debug, Clone)]
pub struct ResolvedObservabilityPipeline {
    /// Pipeline declaration.
    pub pipeline: EngineObservabilityPipelineConfig,
    /// Resolved flow policy after hierarchy resolution.
    pub flow_policy: FlowPolicy,
    /// Resolved health policy after hierarchy resolution.
    pub health_policy: HealthPolicy,
    /// Resolved telemetry policy after hierarchy resolution.
    pub telemetry_policy: TelemetryPolicy,
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
        engine: EngineConfig,
    ) -> Result<Self, Error> {
        let mut pipeline_group = PipelineGroupConfig::new();
        pipeline_group.add_pipeline(pipeline_id, pipeline)?;
        let mut groups = HashMap::new();
        let _ = groups.insert(pipeline_group_id, pipeline_group);
        let config = OtelDataflowSpec {
            version: ENGINE_CONFIG_VERSION_V1.to_string(),
            policies: Policies::default(),
            engine,
            groups,
        };
        config.validate()?;
        Ok(config)
    }

    /// Resolves and materializes policies once for all pipelines.
    ///
    /// The returned snapshot is deterministic: pipelines are ordered by
    /// `(group_id, pipeline_id)` lexicographically.
    #[must_use]
    pub fn resolve(&self) -> ResolvedOtelDataflowSpec {
        let mut pipeline_keys = Vec::new();
        for (pipeline_group_id, pipeline_group) in &self.groups {
            for pipeline_id in pipeline_group.pipelines.keys() {
                pipeline_keys.push((pipeline_group_id.clone(), pipeline_id.clone()));
            }
        }
        pipeline_keys.sort_by(|(g1, p1), (g2, p2)| {
            g1.as_ref()
                .cmp(g2.as_ref())
                .then_with(|| p1.as_ref().cmp(p2.as_ref()))
        });

        let pipelines = pipeline_keys
            .into_iter()
            .map(|(pipeline_group_id, pipeline_id)| {
                let pipeline_group = self
                    .groups
                    .get(&pipeline_group_id)
                    .expect("pipeline group collected during resolve must still exist in map");
                let pipeline = pipeline_group
                    .pipelines
                    .get(&pipeline_id)
                    .expect("pipeline collected during resolve must still exist in group map")
                    .clone();
                let flow_policy = self
                    .resolve_flow_policy(&pipeline_group_id, &pipeline_id)
                    .expect("effective flow policy must resolve for existing pipeline");
                let health_policy = self
                    .resolve_health_policy(&pipeline_group_id, &pipeline_id)
                    .expect("effective health policy must resolve for existing pipeline");
                let telemetry_policy = self
                    .resolve_telemetry_policy(&pipeline_group_id, &pipeline_id)
                    .expect("effective telemetry policy must resolve for existing pipeline");
                let resources_policy = self
                    .resolve_resources_policy(&pipeline_group_id, &pipeline_id)
                    .expect("effective resources policy must resolve for existing pipeline");
                ResolvedPipelineConfig {
                    pipeline_group_id,
                    pipeline_id,
                    pipeline,
                    policies: Policies {
                        flow: flow_policy,
                        health: health_policy,
                        telemetry: telemetry_policy,
                        resources: resources_policy,
                    },
                }
            })
            .collect();

        let observability_pipeline = self.engine.observability.pipeline.clone().map(|pipeline| {
            ResolvedObservabilityPipeline {
                pipeline,
                flow_policy: self.resolve_observability_flow_policy(),
                health_policy: self.resolve_observability_health_policy(),
                telemetry_policy: self.resolve_observability_telemetry_policy(),
            }
        });

        ResolvedOtelDataflowSpec {
            engine: self.engine.clone(),
            pipelines,
            observability_pipeline,
        }
    }

    /// Resolves the effective flow policy for a pipeline.
    ///
    /// Precedence:
    /// 1. pipeline-level policies
    /// 2. group-level policies
    /// 3. top-level policies
    #[must_use]
    pub fn resolve_flow_policy(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Option<FlowPolicy> {
        let pipeline_group = self.groups.get(pipeline_group_id)?;
        let pipeline = pipeline_group.pipelines.get(pipeline_id)?;

        pipeline
            .policies()
            .map(|p| p.flow.clone())
            .or_else(|| pipeline_group.policies.as_ref().map(|p| p.flow.clone()))
            .or_else(|| Some(self.policies.flow.clone()))
    }

    /// Resolves the effective health policy for a pipeline.
    ///
    /// Precedence:
    /// 1. pipeline-level policies
    /// 2. group-level policies
    /// 3. top-level policies
    #[must_use]
    pub fn resolve_health_policy(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Option<HealthPolicy> {
        let pipeline_group = self.groups.get(pipeline_group_id)?;
        let pipeline = pipeline_group.pipelines.get(pipeline_id)?;

        pipeline
            .policies()
            .map(|p| p.health.clone())
            .or_else(|| pipeline_group.policies.as_ref().map(|p| p.health.clone()))
            .or_else(|| Some(self.policies.health.clone()))
    }

    /// Resolves the effective runtime telemetry policy for a pipeline.
    ///
    /// Precedence:
    /// 1. pipeline-level policies
    /// 2. group-level policies
    /// 3. top-level policies
    #[must_use]
    pub fn resolve_telemetry_policy(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Option<TelemetryPolicy> {
        let pipeline_group = self.groups.get(pipeline_group_id)?;
        let pipeline = pipeline_group.pipelines.get(pipeline_id)?;

        pipeline
            .policies()
            .map(|p| p.telemetry.clone())
            .or_else(|| {
                pipeline_group
                    .policies
                    .as_ref()
                    .map(|p| p.telemetry.clone())
            })
            .or_else(|| Some(self.policies.telemetry.clone()))
    }

    /// Resolves the effective resources policy for a pipeline.
    ///
    /// Precedence:
    /// 1. pipeline-level policies
    /// 2. group-level policies
    /// 3. top-level policies
    #[must_use]
    pub fn resolve_resources_policy(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Option<ResourcesPolicy> {
        let pipeline_group = self.groups.get(pipeline_group_id)?;
        let pipeline = pipeline_group.pipelines.get(pipeline_id)?;

        pipeline
            .policies()
            .map(|p| p.resources.clone())
            .or_else(|| {
                pipeline_group
                    .policies
                    .as_ref()
                    .map(|p| p.resources.clone())
            })
            .or_else(|| Some(self.policies.resources.clone()))
    }

    /// Resolves the effective flow policy for the engine observability pipeline.
    ///
    /// Precedence:
    /// 1. `engine.observability.pipeline.policies`
    /// 2. top-level policies
    #[must_use]
    pub fn resolve_observability_flow_policy(&self) -> FlowPolicy {
        self.engine
            .observability
            .pipeline
            .as_ref()
            .and_then(|p| p.policies.as_ref())
            .map_or_else(|| self.policies.flow.clone(), |p| p.flow.clone())
    }

    /// Resolves the effective health policy for the engine observability pipeline.
    ///
    /// Precedence:
    /// 1. `engine.observability.pipeline.policies`
    /// 2. top-level policies
    #[must_use]
    pub fn resolve_observability_health_policy(&self) -> HealthPolicy {
        self.engine
            .observability
            .pipeline
            .as_ref()
            .and_then(|p| p.policies.as_ref())
            .map_or_else(|| self.policies.health.clone(), |p| p.health.clone())
    }

    /// Resolves the effective runtime telemetry policy for the engine observability pipeline.
    ///
    /// Precedence:
    /// 1. `engine.observability.pipeline.policies`
    /// 2. top-level policies
    #[must_use]
    pub fn resolve_observability_telemetry_policy(&self) -> TelemetryPolicy {
        self.engine
            .observability
            .pipeline
            .as_ref()
            .and_then(|p| p.policies.as_ref())
            .map_or_else(|| self.policies.telemetry.clone(), |p| p.telemetry.clone())
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

        errors.extend(
            self.policies
                .validation_errors("policies")
                .into_iter()
                .map(|error| Error::InvalidUserConfig { error }),
        );

        if let Some(observability_pipeline) = self.engine.observability.pipeline.clone() {
            if let Some(policies) = &observability_pipeline.policies {
                errors.extend(
                    policies
                        .validation_errors("engine.observability.pipeline.policies")
                        .into_iter()
                        .map(|error| Error::InvalidUserConfig { error }),
                );
            }
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
    fn from_yaml_uses_default_top_level_flow_policy() {
        let yaml = valid_engine_yaml(ENGINE_CONFIG_VERSION_V1);
        let config = OtelDataflowSpec::from_yaml(&yaml).expect("should parse");
        assert_eq!(config.policies.flow.channel_capacity.control.node, 256);
        assert_eq!(config.policies.flow.channel_capacity.control.pipeline, 256);
        assert_eq!(config.policies.flow.channel_capacity.pdata, 128);
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
    fn resolve_flow_policy_respects_scope_precedence() {
        let yaml = r#"
version: otel_dataflow/v1
policies:
  flow:
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
      flow:
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
          flow:
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

        let p1 = config
            .resolve_flow_policy(&"g1".into(), &"p1".into())
            .expect("p1 should resolve");
        assert_eq!(p1.channel_capacity.control.node, 50);
        assert_eq!(p1.channel_capacity.control.pipeline, 51);
        assert_eq!(p1.channel_capacity.pdata, 52);

        let p2 = config
            .resolve_flow_policy(&"g1".into(), &"p2".into())
            .expect("p2 should resolve");
        assert_eq!(p2.channel_capacity.control.node, 150);
        assert_eq!(p2.channel_capacity.control.pipeline, 151);
        assert_eq!(p2.channel_capacity.pdata, 152);

        let p3 = config
            .resolve_flow_policy(&"g2".into(), &"p3".into())
            .expect("p3 should resolve");
        assert_eq!(p3.channel_capacity.control.node, 200);
        assert_eq!(p3.channel_capacity.control.pipeline, 201);
        assert_eq!(p3.channel_capacity.pdata, 202);

        let h1 = config
            .resolve_health_policy(&"g1".into(), &"p1".into())
            .expect("p1 health should resolve");
        assert_eq!(h1.ready_if, vec![crate::health::PhaseKind::Failed]);

        let h2 = config
            .resolve_health_policy(&"g1".into(), &"p2".into())
            .expect("p2 health should resolve");
        assert_eq!(
            h2.ready_if,
            vec![
                crate::health::PhaseKind::Running,
                crate::health::PhaseKind::Updating,
            ]
        );

        let h3 = config
            .resolve_health_policy(&"g2".into(), &"p3".into())
            .expect("p3 health should resolve");
        assert_eq!(h3.ready_if, vec![crate::health::PhaseKind::Running]);

        let t1 = config
            .resolve_telemetry_policy(&"g1".into(), &"p1".into())
            .expect("p1 telemetry should resolve");
        assert!(!t1.channel_metrics);

        let t2 = config
            .resolve_telemetry_policy(&"g1".into(), &"p2".into())
            .expect("p2 telemetry should resolve");
        assert!(t2.channel_metrics);

        let t3 = config
            .resolve_telemetry_policy(&"g2".into(), &"p3".into())
            .expect("p3 telemetry should resolve");
        assert!(!t3.channel_metrics);

        let r1 = config
            .resolve_resources_policy(&"g1".into(), &"p1".into())
            .expect("p1 resources should resolve");
        assert_eq!(
            r1.core_allocation,
            crate::policy::CoreAllocation::CoreCount { count: 2 }
        );

        let r2 = config
            .resolve_resources_policy(&"g1".into(), &"p2".into())
            .expect("p2 resources should resolve");
        assert_eq!(
            r2.core_allocation,
            crate::policy::CoreAllocation::CoreCount { count: 5 }
        );

        let r3 = config
            .resolve_resources_policy(&"g2".into(), &"p3".into())
            .expect("p3 resources should resolve");
        assert_eq!(
            r3.core_allocation,
            crate::policy::CoreAllocation::CoreCount { count: 9 }
        );

        let resolved = config.resolve();
        assert_eq!(resolved.pipelines.len(), 3);
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

        let p1_resolved = &resolved.pipelines[0];
        assert_eq!(p1_resolved.policies.flow.channel_capacity.control.node, 50);
        assert_eq!(
            p1_resolved.policies.resources.core_allocation,
            crate::policy::CoreAllocation::CoreCount { count: 2 }
        );
        assert_eq!(
            p1_resolved.policies.health.ready_if,
            vec![crate::health::PhaseKind::Failed]
        );

        let p2_resolved = &resolved.pipelines[1];
        assert_eq!(p2_resolved.policies.flow.channel_capacity.control.node, 150);
        assert_eq!(
            p2_resolved.policies.resources.core_allocation,
            crate::policy::CoreAllocation::CoreCount { count: 5 }
        );
    }

    #[test]
    fn resolve_observability_flow_policy_overrides_top_level() {
        let yaml = r#"
version: otel_dataflow/v1
policies:
  flow:
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
        flow:
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
        let flow = config.resolve_observability_flow_policy();
        assert_eq!(flow.channel_capacity.control.node, 10);
        assert_eq!(flow.channel_capacity.control.pipeline, 11);
        assert_eq!(flow.channel_capacity.pdata, 12);

        let health = config.resolve_observability_health_policy();
        assert_eq!(health.ready_if, vec![crate::health::PhaseKind::Failed]);

        let telemetry = config.resolve_observability_telemetry_policy();
        assert!(telemetry.channel_metrics);

        let resolved = config.resolve();
        let obs = resolved
            .observability_pipeline
            .expect("observability pipeline should be resolved");
        assert_eq!(obs.flow_policy.channel_capacity.control.node, 10);
        assert_eq!(obs.flow_policy.channel_capacity.control.pipeline, 11);
        assert_eq!(obs.flow_policy.channel_capacity.pdata, 12);
        assert_eq!(
            obs.health_policy.ready_if,
            vec![crate::health::PhaseKind::Failed]
        );
        assert!(obs.telemetry_policy.channel_metrics);
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
  flow:
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
        assert!(rendered.contains("flow.channel_capacity.control.node"));
        assert!(rendered.contains("flow.channel_capacity.control.pipeline"));
        assert!(rendered.contains("flow.channel_capacity.pdata"));
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
