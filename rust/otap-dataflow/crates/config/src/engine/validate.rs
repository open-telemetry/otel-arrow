// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation phase for [`OtelDataflowSpec`].

use crate::engine::{
    ENGINE_CONFIG_VERSION_V1, INTERNAL_TELEMETRY_RECEIVER_URN, OtelDataflowSpec,
    SYSTEM_OBSERVABILITY_PIPELINE_ID, SYSTEM_PIPELINE_GROUP_ID,
};
use crate::error::Error;

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

        // The observability pipeline is always present (deserialization installs
        // the built-in default when it is omitted). Exactly one internal receiver
        // must own the process-wide log stream and destructive metric-registry
        // drain; zero receivers leave telemetry unconsumed, while multiple
        // receivers would compete for the same data.
        let observability_pipeline = &self.engine.observability.pipeline;
        let internal_receivers = observability_pipeline
            .nodes
            .iter()
            .filter(|(_, node)| node.r#type.as_str() == INTERNAL_TELEMETRY_RECEIVER_URN)
            .collect::<Vec<_>>();
        if internal_receivers.len() != 1 {
            errors.push(Error::InvalidUserConfig {
                error: format!(
                    "engine.observability.pipeline requires exactly one internal telemetry receiver; found {}",
                    internal_receivers.len()
                ),
            });
        } else if let Some((receiver_id, receiver)) = internal_receivers.first() {
            // A direct outgoing edge is not enough: the receiver could feed a
            // processor chain that never reaches an exporter. Reuse the pipeline's
            // iterative pruning to require a complete surviving downstream path.
            let mut pruned_pipeline = observability_pipeline.clone().into_pipeline_config();
            let removed_nodes = pruned_pipeline.remove_unconnected_nodes();
            if removed_nodes
                .iter()
                .any(|(node_id, _)| node_id == *receiver_id)
            {
                errors.push(Error::InvalidUserConfig {
                    error: format!(
                        "internal telemetry receiver '{receiver_id}' must remain connected to a valid downstream path in engine.observability.pipeline"
                    ),
                });
            }
            // Default log providers bypass ITS, so a metrics-only receiver is
            // normally valid. Once any producer routes logs through ITS, however,
            // the sole internal receiver must consume and forward that signal.
            if self.engine.telemetry.routes_logs_through_its()
                && !internal_receiver_logs_enabled(&receiver.config)
            {
                errors.push(Error::InvalidUserConfig {
                    error: format!(
                        "internal telemetry receiver '{receiver_id}' must enable logs while the global, engine, or admin telemetry log provider uses ITS"
                    ),
                });
            }
        }

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
            let pipeline_cfg = observability_pipeline.clone().into_pipeline_config();
            if let Err(e) = pipeline_cfg.validate(
                &SYSTEM_PIPELINE_GROUP_ID.into(),
                &SYSTEM_OBSERVABILITY_PIPELINE_ID.into(),
            ) {
                errors.push(e);
            }
        }

        let any_regular_rate_limit = self.policies.rate_limit.is_some()
            || self.groups.values().any(|pipeline_group| {
                pipeline_group
                    .policies
                    .as_ref()
                    .is_some_and(|policies| policies.rate_limit.is_some())
                    || pipeline_group.pipelines.values().any(|pipeline| {
                        pipeline
                            .policies()
                            .is_some_and(|policies| policies.rate_limit.is_some())
                    })
            });
        if any_regular_rate_limit
            && self
                .policies
                .resources
                .as_ref()
                .and_then(|resources| resources.memory_limiter.as_ref())
                .is_none()
        {
            errors.push(Error::InvalidUserConfig {
                error: "rate_limit policy requires policies.resources.memory_limiter so receivers have a process pressure source".to_owned(),
            });
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

/// Returns whether an opaque internal telemetry receiver config selects logs.
///
/// The config crate intentionally does not depend on the core-nodes crate's
/// typed receiver configuration. This mirrors its public contract locally:
/// omitting `signals` selects the default `[logs, metrics]`, while an explicit
/// list must contain `logs`. Other shapes are treated as disabled here; normal
/// component validation remains responsible for reporting their schema error.
fn internal_receiver_logs_enabled(config: &serde_json::Value) -> bool {
    match config.as_object().and_then(|config| config.get("signals")) {
        None => true,
        Some(serde_json::Value::Array(signals)) => {
            signals.iter().any(|signal| signal.as_str() == Some("logs"))
        }
        Some(_) => false,
    }
}
