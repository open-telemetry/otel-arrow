// SPDX-License-Identifier: Apache-2.0

//! The configuration for the dataflow engine.

use crate::NamespaceId;
use crate::error::Error;
use crate::namespace::NamespaceConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration for the pipeline engine.
/// Contains engine-level settings and all namespaces.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EngineConfig {
    /// Settings that apply to the entire engine instance.
    settings: EngineSettings,

    /// All namespaces managed by this engine, keyed by namespace ID.
    namespaces: HashMap<NamespaceId, NamespaceConfig>,
}

/// Global settings for the engine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EngineSettings {
    // TBD: Add settings fields as needed
}

impl EngineConfig {
    /// Creates a new `EngineConfig` with the given JSON string.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let config: EngineConfig =
            serde_json::from_str(json).map_err(|e| Error::DeserializationError {
                context: Default::default(),
                format: "JSON".to_string(),
                details: e.to_string(),
            })?;
        config.validate()?;
        Ok(config)
    }

    /// Validates the engine configuration and returns a [`Error::InvalidConfiguration`] error
    /// containing all validation errors found in the namespaces.
    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = Vec::new();

        for (namespace_id, namespace) in &self.namespaces {
            if let Err(e) = namespace.validate(namespace_id) {
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
