// SPDX-License-Identifier: Apache-2.0

//! The configuration for a namespace.

use crate::error::Error;
use crate::pipeline::PipelineConfig;
use crate::{Description, NamespaceId, PipelineId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single namespace.
/// Contains namespace-specific settings and all its pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NamespaceConfig {
    /// An optional description of the namespace.
    pub description: Option<Description>,

    /// Settings that apply to this namespace only.
    pub settings: NamespaceSettings,

    /// All pipelines belonging to this namespace, keyed by pipeline ID.
    pub pipelines: HashMap<PipelineId, PipelineConfig>,
}

/// Namespace-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NamespaceSettings {
    /// If false, the namespace is disabled and cannot process data.
    pub enabled: bool,
}

impl NamespaceConfig {
    /// Validates the namespace configuration.
    pub fn validate(&self, namespace_id: &NamespaceId) -> Result<(), Error> {
        let mut errors = Vec::new();

        for (pipeline_id, pipeline) in &self.pipelines {
            if let Err(e) = pipeline.validate(namespace_id, pipeline_id) {
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
