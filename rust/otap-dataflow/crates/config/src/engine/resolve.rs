// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resolution phase for [`OtelDataflowSpec`].

use crate::engine::{EngineConfig, OtelDataflowSpec};
use crate::health::HealthPolicy;
use crate::pipeline::PipelineConfig;
use crate::policy::{ChannelCapacityPolicy, Policies, ResourcesPolicy, TelemetryPolicy};
use crate::{PipelineGroupId, PipelineId};

/// System pipeline-group id used by the engine to group internal telemetry pipelines.
pub const SYSTEM_PIPELINE_GROUP_ID: &str = "system";
/// Synthetic pipeline id used by the engine observability pipeline.
pub const OBSERVABILITY_INTERNAL_PIPELINE_ID: &str = "internal";

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
    pub policies: Policies,
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
                let channel_capacity_policy = self
                    .resolve_channel_capacity_policy(&pipeline_group_id, &pipeline_id)
                    .expect("effective channel capacity policy must resolve for existing pipeline");
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
                        channel_capacity: channel_capacity_policy,
                        health: health_policy,
                        telemetry: telemetry_policy,
                        resources: resources_policy,
                    },
                    role: ResolvedPipelineRole::Regular,
                }
            })
            .collect();

        if let Some(pipeline) = self.engine.observability.pipeline.clone() {
            let channel_capacity_policy = self.resolve_observability_channel_capacity_policy();
            let health_policy = self.resolve_observability_health_policy();
            let telemetry_policy = self.resolve_observability_telemetry_policy();
            pipelines.push(ResolvedPipelineConfig {
                pipeline_group_id: SYSTEM_PIPELINE_GROUP_ID.into(),
                pipeline_id: OBSERVABILITY_INTERNAL_PIPELINE_ID.into(),
                pipeline: pipeline.into_pipeline_config(),
                policies: Policies {
                    channel_capacity: channel_capacity_policy,
                    health: health_policy,
                    telemetry: telemetry_policy,
                    resources: ResourcesPolicy::default(),
                },
                role: ResolvedPipelineRole::ObservabilityInternal,
            });
        }

        ResolvedOtelDataflowSpec {
            engine: self.engine.clone(),
            pipelines,
        }
    }

    /// Resolves the effective channel capacity policy for a pipeline.
    ///
    /// Precedence:
    /// 1. pipeline-level policies
    /// 2. group-level policies
    /// 3. top-level policies
    #[must_use]
    fn resolve_channel_capacity_policy(
        &self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> Option<ChannelCapacityPolicy> {
        let pipeline_group = self.groups.get(pipeline_group_id)?;
        let pipeline = pipeline_group.pipelines.get(pipeline_id)?;

        pipeline
            .policies()
            .map(|p| p.channel_capacity.clone())
            .or_else(|| {
                pipeline_group
                    .policies
                    .as_ref()
                    .map(|p| p.channel_capacity.clone())
            })
            .or_else(|| Some(self.policies.channel_capacity.clone()))
    }

    /// Resolves the effective health policy for a pipeline.
    ///
    /// Precedence:
    /// 1. pipeline-level policies
    /// 2. group-level policies
    /// 3. top-level policies
    #[must_use]
    fn resolve_health_policy(
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
    fn resolve_telemetry_policy(
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
    fn resolve_resources_policy(
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

    /// Resolves the effective channel capacity policy for the engine observability pipeline.
    ///
    /// Precedence:
    /// 1. `engine.observability.pipeline.policies`
    /// 2. top-level policies
    #[must_use]
    fn resolve_observability_channel_capacity_policy(&self) -> ChannelCapacityPolicy {
        self.engine
            .observability
            .pipeline
            .as_ref()
            .and_then(|p| p.policies.as_ref())
            .map_or_else(
                || self.policies.channel_capacity.clone(),
                |p| p.channel_capacity.clone(),
            )
    }

    /// Resolves the effective health policy for the engine observability pipeline.
    ///
    /// Precedence:
    /// 1. `engine.observability.pipeline.policies`
    /// 2. top-level policies
    #[must_use]
    fn resolve_observability_health_policy(&self) -> HealthPolicy {
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
    fn resolve_observability_telemetry_policy(&self) -> TelemetryPolicy {
        self.engine
            .observability
            .pipeline
            .as_ref()
            .and_then(|p| p.policies.as_ref())
            .map_or_else(|| self.policies.telemetry.clone(), |p| p.telemetry.clone())
    }
}
