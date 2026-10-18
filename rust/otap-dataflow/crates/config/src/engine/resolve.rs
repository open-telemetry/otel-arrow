// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resolution phase for [`OtelDataflowSpec`].

use crate::codecs::{CodecsConfig, ResolvedCodecsConfig};
use crate::context_policy::ContextEntryDeclaration;
use crate::engine::{EngineConfig, OtelDataflowSpec};
use crate::pipeline::PipelineConfig;
use crate::policy::{
    Policies, RateLimiterDeclarationScope, ResolvedPolicies, ResolvedResourcesPolicy,
};
use crate::topic::{TopicScope, TopicSpec};
use crate::{PipelineGroupId, PipelineId, TopicName};

/// System pipeline-group id used by the engine to group internal telemetry pipelines.
pub const SYSTEM_PIPELINE_GROUP_ID: &str = "system";
/// Synthetic pipeline id used by the engine observability pipeline.
pub const SYSTEM_OBSERVABILITY_PIPELINE_ID: &str = "observability";

/// Resolved policy snapshot for an engine configuration.
///
/// This is a deterministic snapshot computed from [`OtelDataflowSpec`]
/// using hierarchy precedence rules.
#[derive(Debug, Clone)]
pub struct ResolvedOtelDataflowSpec {
    /// Engine-wide runtime declarations.
    pub engine: EngineConfig,
    /// Resolved pipeline configurations.
    pub pipelines: Vec<ResolvedPipelineConfig>,
}

impl ResolvedOtelDataflowSpec {
    /// Splits resolved pipelines by role and returns the associated engine config.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        EngineConfig,
        Vec<ResolvedPipelineConfig>,
        Option<ResolvedPipelineConfig>,
    ) {
        let mut regular_pipelines = Vec::new();
        let mut observability_pipeline = None;

        for pipeline in self.pipelines {
            match pipeline.role {
                ResolvedPipelineRole::Regular => regular_pipelines.push(pipeline),
                ResolvedPipelineRole::ObservabilityInternal => {
                    observability_pipeline = Some(pipeline)
                }
            }
        }

        (self.engine, regular_pipelines, observability_pipeline)
    }
}

/// Pipeline role in the resolved snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedPipelineRole {
    /// A user-defined regular pipeline from `groups.<group>.pipelines`.
    Regular,
    /// The dedicated engine observability pipeline.
    ObservabilityInternal,
}

/// Resolved data for one pipeline.
#[derive(Debug, Clone)]
pub struct ResolvedPipelineConfig {
    /// Pipeline group identifier.
    pub pipeline_group_id: PipelineGroupId,
    /// Pipeline identifier.
    pub pipeline_id: PipelineId,
    /// Pipeline definition.
    pub pipeline: PipelineConfig,
    /// Resolved topic namespace visible to this pipeline.
    pub topic_scope: TopicScope,
    /// Resolved policies after hierarchy resolution.
    pub policies: ResolvedPolicies,
    /// Effective codec configuration after inheritance resolution.
    pub codecs: ResolvedCodecsConfig,
    /// Pipeline role.
    pub role: ResolvedPipelineRole,
}

impl ResolvedPipelineConfig {
    /// Compares two resolved pipelines for exact runtime equivalence.
    ///
    /// Logical identity is intentionally ignored here; callers compare two
    /// candidate snapshots for the same logical pipeline and only care whether
    /// runtime-relevant config, resolved policies, and codec settings match.
    #[must_use]
    pub fn runtime_matches(&self, other: &Self) -> bool {
        let Self {
            pipeline_group_id: _,
            pipeline_id: _,
            pipeline: self_pipeline,
            topic_scope: self_topic_scope,
            policies: self_policies,
            codecs: self_codecs,
            role: self_role,
        } = self;
        let Self {
            pipeline_group_id: _,
            pipeline_id: _,
            pipeline: other_pipeline,
            topic_scope: other_topic_scope,
            policies: other_policies,
            codecs: other_codecs,
            role: other_role,
        } = other;

        self_role == other_role
            && self_topic_scope == other_topic_scope
            // Inherited settings are compared through the resolved snapshot below.
            // Comparing their declaration placement here would redeploy for a
            // scope-only move whose effective runtime settings are unchanged.
            && self_pipeline.eq_ignoring_inherited_settings(other_pipeline)
            && self_policies == other_policies
            && self_codecs == other_codecs
    }

    /// Compares two resolved pipelines while ignoring resource-only policy
    /// differences used by resize classification.
    #[must_use]
    pub fn runtime_shape_matches_ignoring_resources(&self, other: &Self) -> bool {
        let Self {
            pipeline_group_id: _,
            pipeline_id: _,
            pipeline: self_pipeline,
            topic_scope: self_topic_scope,
            policies: self_policies,
            codecs: self_codecs,
            role: self_role,
        } = self;
        let Self {
            pipeline_group_id: _,
            pipeline_id: _,
            pipeline: other_pipeline,
            topic_scope: other_topic_scope,
            policies: other_policies,
            codecs: other_codecs,
            role: other_role,
        } = other;

        self_role == other_role
            && self_topic_scope == other_topic_scope
            && self_pipeline.eq_ignoring_inherited_settings(other_pipeline)
            && self_policies.eq_ignoring_resources(other_policies)
            && self_codecs == other_codecs
    }
}

impl OtelDataflowSpec {
    /// Resolves the topic scope visible to pipelines in a group.
    #[must_use]
    pub fn resolve_topic_scope(&self, pipeline_group_id: &PipelineGroupId) -> Option<TopicScope> {
        // Control-plane validation ensures that non-empty group topics still match the
        // declarations for that group ID in the startup configuration. Supporting dynamically
        // created topic scopes will require explicit scope allocation.
        self.groups.get(pipeline_group_id).map(|pipeline_group| {
            if pipeline_group.topics.is_empty() {
                TopicScope::GLOBAL
            } else {
                TopicScope::startup_group(pipeline_group_id.clone())
            }
        })
    }

    /// Resolves inherited policies and codec settings once for all pipelines.
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

        let mut pipelines: Vec<ResolvedPipelineConfig> = pipeline_keys
            .into_iter()
            .map(|(pipeline_group_id, pipeline_id)| {
                let pipeline_group = self
                    .groups
                    .get(&pipeline_group_id)
                    .expect("pipeline group collected during resolve must still exist in map");
                let topic_scope = self
                    .resolve_topic_scope(&pipeline_group_id)
                    .expect("pipeline group collected during resolve must still resolve");
                let pipeline = pipeline_group
                    .pipelines
                    .get(&pipeline_id)
                    .expect("pipeline collected during resolve must still exist in group map")
                    .clone();
                let scopes: Vec<&Policies> = [
                    pipeline.policies(),
                    pipeline_group.policies.as_ref(),
                    Some(&self.policies),
                ]
                .into_iter()
                .flatten()
                .collect();
                let mut policies = Policies::resolve(scopes);
                policies.context = self
                    .context_policy_layers(&pipeline_group_id, Some(&pipeline_id))
                    .into_iter()
                    .flat_map(|layer| {
                        let scope = layer.scope;
                        layer.context.into_iter().flat_map(move |context| {
                            let scope = scope.clone();
                            context.entries.iter().map(move |(name, definition)| {
                                ContextEntryDeclaration {
                                    scope: scope.clone(),
                                    name: name.clone(),
                                    definition: definition.clone(),
                                }
                            })
                        })
                    })
                    .collect();
                policies.rate_limiter_scope = if pipeline
                    .policies()
                    .and_then(Policies::resources)
                    .and_then(|resources| resources.rate_limiters.as_ref())
                    .is_some()
                {
                    Some(RateLimiterDeclarationScope::Pipeline(
                        pipeline_group_id.clone(),
                        pipeline_id.clone(),
                    ))
                } else if pipeline_group
                    .policies
                    .as_ref()
                    .and_then(Policies::resources)
                    .and_then(|resources| resources.rate_limiters.as_ref())
                    .is_some()
                {
                    Some(RateLimiterDeclarationScope::PipelineGroup(
                        pipeline_group_id.clone(),
                    ))
                } else if self
                    .policies
                    .resources()
                    .and_then(|resources| resources.rate_limiters.as_ref())
                    .is_some()
                {
                    Some(RateLimiterDeclarationScope::Engine)
                } else {
                    None
                };
                let codecs = CodecsConfig::resolve(
                    [
                        pipeline.codecs(),
                        pipeline_group.codecs.as_ref(),
                        self.codecs.as_ref(),
                    ]
                    .into_iter()
                    .flatten(),
                );
                ResolvedPipelineConfig {
                    topic_scope,
                    pipeline_group_id,
                    pipeline_id,
                    pipeline,
                    policies,
                    codecs,
                    role: ResolvedPipelineRole::Regular,
                }
            })
            .collect();

        let obs_pipeline = self.engine.observability.pipeline.clone();
        let codecs = CodecsConfig::resolve(
            [obs_pipeline.codecs.as_ref(), self.codecs.as_ref()]
                .into_iter()
                .flatten(),
        );
        let obs_as_policies = obs_pipeline
            .policies
            .as_ref()
            .map(|p| p.clone().into_policies())
            .unwrap_or_default();
        let mut policies = Policies::resolve([&obs_as_policies, &self.policies]);
        // Observability pipelines use default resources and do not consume
        // user-facing receiver policies.
        policies.resources = ResolvedResourcesPolicy::default();
        policies.transport_headers = None;
        policies.authorized_identity = None;
        policies.context.clear();
        policies.rate_limiters.clear();
        policies.rate_limiter_scope = None;
        pipelines.push(ResolvedPipelineConfig {
            pipeline_group_id: SYSTEM_PIPELINE_GROUP_ID.into(),
            pipeline_id: SYSTEM_OBSERVABILITY_PIPELINE_ID.into(),
            pipeline: obs_pipeline.into_pipeline_config(),
            topic_scope: TopicScope::GLOBAL,
            policies,
            codecs,
            role: ResolvedPipelineRole::ObservabilityInternal,
        });

        ResolvedOtelDataflowSpec {
            engine: self.engine.clone(),
            pipelines,
        }
    }

    /// Resolves a topic specification visible from a pipeline group.
    ///
    /// Precedence:
    /// 1. `groups.<group>.topics.<name>`
    /// 2. top-level `topics.<name>`
    #[must_use]
    pub fn resolve_topic_spec(
        &self,
        pipeline_group_id: &PipelineGroupId,
        topic_name: &TopicName,
    ) -> Option<TopicSpec> {
        if let Some(group_topic) = self
            .groups
            .get(pipeline_group_id)
            .and_then(|group| group.topics.get(topic_name))
        {
            return Some(group_topic.clone());
        }
        self.topics.get(topic_name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolvedPipelineConfig, ResolvedPipelineRole};
    use crate::pipeline::PipelineConfig;
    use crate::policy::{
        CoreAllocation, ResolvedPolicies, ResolvedResourcesPolicy, TelemetryPolicy,
    };
    use crate::topic::TopicScope;

    fn codec_test_spec() -> serde_json::Value {
        serde_json::json!({
            "version": "otel_dataflow/v1",
            "groups": { "g": { "pipelines": { "p": {
                "nodes": {
                    "source": { "type": "urn:test:receiver:example", "config": {} },
                    "sink": { "type": "urn:test:exporter:example", "config": {} }
                },
                "connections": [{ "from": "source", "to": "sink" }]
            } } } }
        })
    }

    fn codec_declaration(validation: &str) -> serde_json::Value {
        serde_json::json!({ "defaults": { "decode": { "validation": validation } } })
    }

    fn resolve_codec_test_spec(
        spec: &serde_json::Value,
    ) -> (ResolvedPipelineConfig, ResolvedPipelineConfig) {
        let config = crate::engine::OtelDataflowSpec::from_json(&spec.to_string()).unwrap();
        let (_, mut regular, observability) = config.resolve().into_parts();
        (regular.remove(0), observability.unwrap())
    }

    /// Scenario: Codec defaults are declared at top-level, group, pipeline, and observability scope.
    /// Guarantees: The nearest explicit value wins; empty sections inherit, and observability bypasses groups.
    #[test]
    fn codec_config_inherits_across_pipeline_scopes() {
        use crate::codecs::DecodeValidation::{BestEffort, Strict};
        let mut spec = codec_test_spec();
        let (pipeline, observability) = resolve_codec_test_spec(&spec);
        assert_eq!(pipeline.codecs.decode_validation, BestEffort);
        assert_eq!(observability.codecs.decode_validation, BestEffort);

        spec["codecs"] = codec_declaration("strict");
        spec["groups"]["g"]["pipelines"]["p"]["codecs"] =
            serde_json::json!({"defaults": {"decode": {}}});
        let (pipeline, observability) = resolve_codec_test_spec(&spec);
        assert_eq!(pipeline.codecs.decode_validation, Strict);
        assert_eq!(observability.codecs.decode_validation, Strict);

        spec["groups"]["g"]["codecs"] = codec_declaration("best_effort");
        let (pipeline, observability) = resolve_codec_test_spec(&spec);
        assert_eq!(pipeline.codecs.decode_validation, BestEffort);
        assert_eq!(observability.codecs.decode_validation, Strict);

        spec["groups"]["g"]["pipelines"]["p"]["codecs"] = codec_declaration("strict");
        spec["engine"] = serde_json::json!({ "observability": { "pipeline":
            serde_json::to_value(crate::engine::EngineObservabilityPipelineConfig::default()).unwrap()
        } });
        spec["engine"]["observability"]["pipeline"]["codecs"] = codec_declaration("best_effort");
        let (pipeline, observability) = resolve_codec_test_spec(&spec);
        assert_eq!(pipeline.codecs.decode_validation, Strict);
        assert_eq!(observability.codecs.decode_validation, BestEffort);
        assert_eq!(
            pipeline
                .pipeline
                .codecs()
                .unwrap()
                .defaults
                .decode
                .validation,
            Some(Strict)
        );
        assert_eq!(
            observability
                .pipeline
                .codecs()
                .unwrap()
                .defaults
                .decode
                .validation,
            Some(BestEffort)
        );
    }

    /// Scenario: Only codec validation changes, then the same effective value moves between scopes.
    /// Guarantees: Effective changes require redeployment, but declaration-only changes do not restart a pipeline.
    #[test]
    fn codec_config_changes_redeploy_only_when_effective_settings_change() {
        let mut spec = codec_test_spec();
        let (current, _) = resolve_codec_test_spec(&spec);
        spec["codecs"] = codec_declaration("strict");
        let (strict, _) = resolve_codec_test_spec(&spec);
        assert!(!current.runtime_matches(&strict));
        assert!(!current.runtime_shape_matches_ignoring_resources(&strict));

        _ = spec.as_object_mut().unwrap().remove("codecs");
        spec["groups"]["g"]["pipelines"]["p"]["codecs"] = codec_declaration("strict");
        let (moved, _) = resolve_codec_test_spec(&spec);
        assert!(strict.runtime_matches(&moved));
        assert!(strict.runtime_shape_matches_ignoring_resources(&moved));
    }

    /// Scenario: A user puts codec settings in a node or the former pdata policy namespace.
    /// Guarantees: Unsupported locations fail configuration parsing instead of silently ignoring validation requests.
    #[test]
    fn codec_config_rejects_node_and_policy_locations() {
        let mut spec = codec_test_spec();
        spec["groups"]["g"]["pipelines"]["p"]["nodes"]["source"]["codecs"] =
            codec_declaration("strict");
        let error = crate::engine::OtelDataflowSpec::from_json(&spec.to_string()).unwrap_err();
        assert!(error.to_string().contains("unknown field `codecs`"));

        let mut spec = codec_test_spec();
        spec["policies"] = serde_json::json!({"pdata": {"decode_validation": "strict"}});
        let error = crate::engine::OtelDataflowSpec::from_json(&spec.to_string()).unwrap_err();
        assert!(error.to_string().contains("unknown field `pdata`"));
    }

    /// Scenario: An operator uses the complete YAML example in the codec guide.
    /// Guarantees: The documented configuration parses and its pipeline override leaves observability strict.
    #[test]
    fn codec_guide_example_matches_documented_settings() {
        use crate::codecs::DecodeValidation::{BestEffort, Strict};
        let guide = include_str!("../../../../docs/codecs.md");
        let example = guide
            .split("```yaml")
            .map(str::trim_start)
            .find(|part| part.starts_with("version:"))
            .unwrap()
            .split("```")
            .next()
            .unwrap();
        let config = crate::engine::OtelDataflowSpec::from_yaml(example).unwrap();
        let (_, regular, observability) = config.resolve().into_parts();
        assert_eq!(regular.len(), 1);
        assert_eq!(regular[0].codecs.decode_validation, BestEffort);
        assert_eq!(observability.unwrap().codecs.decode_validation, Strict);
    }

    #[test]
    fn runtime_shape_matches_ignoring_resources_ignores_resource_only_changes() {
        let current = ResolvedPipelineConfig {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            pipeline: PipelineConfig::from_yaml(
                "g1".into(),
                "p1".into(),
                r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
            )
            .expect("current pipeline should parse"),
            topic_scope: TopicScope::GLOBAL,
            policies: ResolvedPolicies {
                resources: ResolvedResourcesPolicy {
                    core_allocation: CoreAllocation::core_count(1),
                    memory_limiter: None,
                },
                ..ResolvedPolicies::default()
            },
            codecs: Default::default(),
            role: ResolvedPipelineRole::Regular,
        };
        let candidate = ResolvedPipelineConfig {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            pipeline: PipelineConfig::from_yaml(
                "g1".into(),
                "p1".into(),
                r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
            )
            .expect("candidate pipeline should parse"),
            topic_scope: TopicScope::GLOBAL,
            policies: ResolvedPolicies {
                resources: ResolvedResourcesPolicy {
                    core_allocation: CoreAllocation::core_count(2),
                    memory_limiter: None,
                },
                ..ResolvedPolicies::default()
            },
            codecs: Default::default(),
            role: ResolvedPipelineRole::Regular,
        };

        assert!(!current.runtime_matches(&candidate));
        assert!(current.runtime_shape_matches_ignoring_resources(&candidate));

        let mut different_scope = current.clone();
        different_scope.topic_scope = TopicScope::startup_group("g1".into());
        assert!(!current.runtime_matches(&different_scope));
        assert!(!current.runtime_shape_matches_ignoring_resources(&different_scope));
    }

    #[test]
    fn runtime_shape_matches_ignoring_resources_detects_runtime_policy_change() {
        let current = ResolvedPipelineConfig {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            pipeline: PipelineConfig::from_yaml(
                "g1".into(),
                "p1".into(),
                r#"
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
            )
            .expect("current pipeline should parse"),
            topic_scope: TopicScope::GLOBAL,
            policies: ResolvedPolicies::default(),
            codecs: Default::default(),
            role: ResolvedPipelineRole::Regular,
        };
        let candidate = ResolvedPipelineConfig {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            pipeline: current.pipeline.clone(),
            topic_scope: TopicScope::GLOBAL,
            policies: ResolvedPolicies {
                telemetry: TelemetryPolicy {
                    pipeline_metrics: false,
                    ..TelemetryPolicy::default()
                },
                ..ResolvedPolicies::default()
            },
            codecs: Default::default(),
            role: ResolvedPipelineRole::Regular,
        };

        assert!(!current.runtime_shape_matches_ignoring_resources(&candidate));
    }
}
