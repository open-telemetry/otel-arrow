// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resolution phase for [`OtelDataflowSpec`].

use crate::engine::{EngineConfig, OtelDataflowSpec};
use crate::pipeline::PipelineConfig;
use crate::policy::{Policies, ResolvedPolicies, ResourcesPolicy};
use crate::topic::TopicSpec;
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
    /// Resolved policies after hierarchy resolution.
    pub policies: ResolvedPolicies,
    /// Pipeline role.
    pub role: ResolvedPipelineRole,
}

impl OtelDataflowSpec {
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

        let mut pipelines: Vec<ResolvedPipelineConfig> = pipeline_keys
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
                let scopes: Vec<&Policies> = [
                    pipeline.policies(),
                    pipeline_group.policies.as_ref(),
                    Some(&self.policies),
                ]
                .into_iter()
                .flatten()
                .collect();
                let policies = Policies::resolve(scopes);
                ResolvedPipelineConfig {
                    pipeline_group_id,
                    pipeline_id,
                    pipeline,
                    policies,
                    role: ResolvedPipelineRole::Regular,
                }
            })
            .collect();

        if let Some(obs_pipeline) = self.engine.observability.pipeline.clone() {
            let obs_as_policies = obs_pipeline
                .policies
                .as_ref()
                .map(|p| p.clone().into_policies())
                .unwrap_or_default();
            let mut policies = Policies::resolve([&obs_as_policies, &self.policies]);
            // Observability pipelines use default resources and do not
            // capture/propagate transport headers.
            policies.resources = ResourcesPolicy::default();
            policies.transport_headers = None;
            pipelines.push(ResolvedPipelineConfig {
                pipeline_group_id: SYSTEM_PIPELINE_GROUP_ID.into(),
                pipeline_id: SYSTEM_OBSERVABILITY_PIPELINE_ID.into(),
                pipeline: obs_pipeline.into_pipeline_config(),
                policies,
                role: ResolvedPipelineRole::ObservabilityInternal,
            });
        }

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
