// SPDX-License-Identifier: Apache-2.0

//! The configuration for engine.

use crate::TenantId;
use crate::error::Error;
use crate::tenant::TenantConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration for the pipeline engine.
/// Contains engine-level settings and all tenants.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EngineConfig {
    /// Settings that apply to the entire engine instance.
    settings: EngineSettings,

    /// All tenants managed by this engine, keyed by tenant ID.
    tenants: HashMap<TenantId, TenantConfig>,
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
    /// containing all validation errors found in the tenants.
    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = Vec::new();

        for (tenant_id, tenant) in &self.tenants {
            if let Err(e) = tenant.validate(tenant_id) {
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
