// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation phase for [`OtelDataflowSpec`].

use crate::engine::{
    ENGINE_CONFIG_VERSION_V1, OtelDataflowSpec, SYSTEM_OBSERVABILITY_PIPELINE_ID,
    SYSTEM_PIPELINE_GROUP_ID,
};
use crate::error::Error;

const INTERNAL_TELEMETRY_RECEIVER_URN: &str = "urn:otel:receiver:internal_telemetry";

/// Detects receiver-level settings that actually enable or transform metrics.
/// Empty `metrics: {}` and `views: []` blocks remain compatible with logs-only
/// ITS configurations, matching the receiver's typed `MetricsConfig::is_empty`
/// semantics without introducing a dependency on the core-nodes crate.
fn has_internal_receiver_metrics_config(config: &serde_json::Value) -> bool {
    let Some(metrics) = config
        .as_object()
        .and_then(|config| config.get("metrics"))
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };

    metrics
        .get("interval")
        .is_some_and(|interval| !interval.is_null())
        || metrics
            .get("views")
            .is_some_and(|views| views.as_array().is_none_or(|views| !views.is_empty()))
}

impl OtelDataflowSpec {
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

        for (topic_name, topic) in &self.topics {
            let path = format!("topics.{topic_name}");
            errors.extend(
                topic
                    .validation_errors(&path)
                    .into_iter()
                    .map(|error| Error::InvalidUserConfig { error }),
            );
        }

        if let Err(e) = self.engine.telemetry.validate() {
            errors.push(e);
        }

        if self.engine.telemetry.metrics.uses_its_provider() {
            match self.engine.observability.pipeline.as_ref() {
                Some(pipeline) => {
                    let receivers = pipeline
                        .nodes
                        .iter()
                        .filter(|(_, node)| node.r#type.as_str() == INTERNAL_TELEMETRY_RECEIVER_URN)
                        .map(|(node_id, _)| node_id)
                        .collect::<Vec<_>>();
                    let receiver_count = receivers.len();
                    if receiver_count != 1 {
                        errors.push(Error::InvalidUserConfig {
                            error: format!(
                                "engine.telemetry.metrics.provider 'its' requires exactly one internal telemetry receiver in engine.observability.pipeline; found {receiver_count}"
                            ),
                        });
                    } else {
                        let mut pruned_pipeline = pipeline.clone().into_pipeline_config();
                        let removed_nodes = pruned_pipeline.remove_unconnected_nodes();
                        if removed_nodes
                            .iter()
                            .any(|(node_id, _)| node_id == receivers[0])
                        {
                            errors.push(Error::InvalidUserConfig {
                                error: format!(
                                    "engine.telemetry.metrics.provider 'its' requires internal telemetry receiver '{}' to remain connected to a valid downstream path in engine.observability.pipeline",
                                    receivers[0]
                                ),
                            });
                        }
                    }
                }
                None => errors.push(Error::InvalidUserConfig {
                    error: "engine.telemetry.metrics.provider 'its' requires engine.observability.pipeline"
                        .to_owned(),
                }),
            }
        }

        if let Some(pipeline) = self.engine.observability.pipeline.as_ref() {
            for (node_id, node) in pipeline
                .nodes
                .iter()
                .filter(|(_, node)| node.r#type.as_str() == INTERNAL_TELEMETRY_RECEIVER_URN)
            {
                if !self.engine.telemetry.metrics.uses_its_provider()
                    && has_internal_receiver_metrics_config(&node.config)
                {
                    errors.push(Error::InvalidUserConfig {
                        error: format!(
                            "internal telemetry receiver '{node_id}' metrics configuration requires engine internal metrics to use the ITS provider"
                        ),
                    });
                }
            }
        }

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
                if let Err(e) = pipeline_cfg.validate(
                    &SYSTEM_PIPELINE_GROUP_ID.into(),
                    &SYSTEM_OBSERVABILITY_PIPELINE_ID.into(),
                ) {
                    errors.push(e);
                }
            }
        }

        for (pipeline_group_id, pipeline_group) in &self.groups {
            if pipeline_group_id.as_ref() == SYSTEM_PIPELINE_GROUP_ID {
                errors.push(Error::InvalidUserConfig {
                    error: format!(
                        "groups.{} is reserved for engine-managed pipelines and cannot be configured by users",
                        SYSTEM_PIPELINE_GROUP_ID
                    ),
                });
                continue;
            }
            if let Err(e) = pipeline_group.validate(pipeline_group_id) {
                errors.push(e);
            }
            if pipeline_group
                .policies
                .as_ref()
                .and_then(|policies| policies.resources.as_ref())
                .and_then(|resources| resources.memory_limiter.as_ref())
                .is_some()
            {
                errors.push(Error::InvalidUserConfig {
                    error: format!(
                        "groups.{pipeline_group_id}.policies.resources.memory_limiter is not supported; configure the process-wide limiter only at top-level policies.resources.memory_limiter"
                    ),
                });
            }
            for (pipeline_id, pipeline) in &pipeline_group.pipelines {
                if pipeline
                    .policies()
                    .and_then(|policies| policies.resources.as_ref())
                    .and_then(|resources| resources.memory_limiter.as_ref())
                    .is_some()
                {
                    errors.push(Error::InvalidUserConfig {
                        error: format!(
                            "groups.{pipeline_group_id}.pipelines.{pipeline_id}.policies.resources.memory_limiter is not supported; configure the process-wide limiter only at top-level policies.resources.memory_limiter"
                        ),
                    });
                }
            }
        }

        if !errors.is_empty() {
            Err(Error::InvalidConfiguration { errors })
        } else {
            Ok(())
        }
    }
}
