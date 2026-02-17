// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation phase for [`OtelDataflowSpec`].

use crate::engine::{ENGINE_CONFIG_VERSION_V1, OtelDataflowSpec};
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
