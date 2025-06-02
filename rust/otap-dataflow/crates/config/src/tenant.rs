// SPDX-License-Identifier: Apache-2.0

//! The configuration for a tenant.

use crate::error::Error;
use crate::pipeline::PipelineConfig;
use crate::{Description, PipelineId, TenantId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single tenant.
/// Contains tenant-specific settings and all its pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TenantConfig {
    /// An optional description of the tenant.
    pub description: Option<Description>,

    /// Settings that apply to this tenant only.
    pub settings: TenantSettings,

    /// All pipelines belonging to this tenant, keyed by pipeline ID.
    pub pipelines: HashMap<PipelineId, PipelineConfig>,
}

/// Tenant-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TenantSettings {
    /// If false, the tenant is disabled and cannot process data.
    pub enabled: bool,
}

impl TenantConfig {
    /// Validates the tenant configuration.
    pub fn validate(&self, tenant_id: &TenantId) -> Result<(), Error> {
        let mut errors = Vec::new();

        for (pipeline_id, pipeline) in &self.pipelines {
            if let Err(e) = pipeline.validate(tenant_id, pipeline_id) {
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
