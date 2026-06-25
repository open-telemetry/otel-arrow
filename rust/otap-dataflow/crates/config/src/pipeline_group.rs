// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The configuration for a pipeline group.

use crate::error::Error;
use crate::pipeline::PipelineConfig;
use crate::policy::Policies;
use crate::topic::TopicSpec;
use crate::{PipelineGroupId, PipelineId, TopicName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single pipeline group.
/// Contains group-specific policies and all its pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PipelineGroupConfig {
    /// Optional policy set for this pipeline group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policies: Option<Policies>,

    /// Group-local topic declarations visible only to pipelines in this group.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub topics: HashMap<TopicName, TopicSpec>,

    /// All pipelines belonging to this pipeline group, keyed by pipeline ID.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub pipelines: HashMap<PipelineId, PipelineConfig>,
}

impl PipelineGroupConfig {
    /// Creates a new empty pipeline group configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            policies: None,
            topics: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }

    /// Returns a clone of this pipeline group with every node's and
    /// extension's credential header values redacted, for safe exposure
    /// through the admin/config snapshot APIs. See
    /// [`PipelineConfig::redacted_for_snapshot`](crate::pipeline::PipelineConfig::redacted_for_snapshot).
    /// The stored config is left unchanged.
    #[must_use]
    pub fn redacted_for_snapshot(&self) -> PipelineGroupConfig {
        let mut redacted = self.clone();
        for pipeline in redacted.pipelines.values_mut() {
            *pipeline = pipeline.redacted_for_snapshot();
        }
        redacted
    }

    /// Adds a pipeline to the pipeline group.
    pub fn add_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        pipeline: PipelineConfig,
    ) -> Result<(), Error> {
        if self.pipelines.contains_key(&pipeline_id) {
            return Err(Error::DuplicatePipeline { pipeline_id });
        }
        _ = self.pipelines.insert(pipeline_id.clone(), pipeline);
        Ok(())
    }

    /// Validates the pipeline group configuration.
    pub fn validate(&self, pipeline_group_id: &PipelineGroupId) -> Result<(), Error> {
        let mut errors = Vec::new();

        if let Some(policies) = &self.policies {
            let path = format!("groups.{pipeline_group_id}.policies");
            errors.extend(
                policies
                    .validation_errors(&path)
                    .into_iter()
                    .map(|error| Error::InvalidUserConfig { error }),
            );
        }

        for (topic_name, topic) in &self.topics {
            let path = format!("groups.{pipeline_group_id}.topics.{topic_name}");
            errors.extend(
                topic
                    .validation_errors(&path)
                    .into_iter()
                    .map(|error| Error::InvalidUserConfig { error }),
            );
        }

        for (pipeline_id, pipeline) in &self.pipelines {
            if let Some(policies) = pipeline.policies() {
                let path = format!("groups.{pipeline_group_id}.pipelines.{pipeline_id}.policies");
                errors.extend(
                    policies
                        .validation_errors(&path)
                        .into_iter()
                        .map(|error| Error::InvalidUserConfig { error }),
                );
            }
            if let Err(e) = pipeline.validate(pipeline_group_id, pipeline_id) {
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

impl Default for PipelineGroupConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacted_for_snapshot_masks_node_and_extension_headers() {
        // A pipeline group whose pipeline carries credential headers on BOTH a
        // node and an extension. `redacted_for_snapshot()` must scrub both
        // before the config is exposed through the admin/config snapshot APIs.
        // Deserialize directly (no validation) to keep the test focused on
        // redaction.
        let yaml = r#"
            pipelines:
              main:
                nodes:
                  exporter:
                    type: "urn:otel:exporter:otlp"
                    config:
                      headers:
                        authorization: "Bearer node-super-secret"
                extensions:
                  auth:
                    type: "urn:otap:extension:headers_setter"
                    config:
                      headers:
                        authorization: "Bearer ext-super-secret"
        "#;
        let group: PipelineGroupConfig =
            serde_yaml::from_str(yaml).expect("pipeline group should deserialize");
        let redacted = group.redacted_for_snapshot();

        let redacted_json = serde_json::to_string(&redacted).expect("redacted serializes");
        assert!(
            !redacted_json.contains("node-super-secret"),
            "node credential must not survive redaction: {redacted_json}"
        );
        assert!(
            !redacted_json.contains("ext-super-secret"),
            "extension credential must not survive redaction: {redacted_json}"
        );
        assert!(
            redacted_json.contains(crate::node::REDACTED_HEADER_VALUE),
            "redaction placeholder should be present: {redacted_json}"
        );

        // Redaction returns a copy; the original group keeps the cleartext.
        let original_json = serde_json::to_string(&group).expect("original serializes");
        assert!(original_json.contains("node-super-secret"));
        assert!(original_json.contains("ext-super-secret"));
    }
}
