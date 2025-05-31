// SPDX-License-Identifier: Apache-2.0

//! The configuration for a tenant.

use crate::error::Error;
use crate::pipeline::PipelineSpec;
use crate::{Description, PipelineId, TenantId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A tenant specification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TenantSpec {
    /// An optional description of the tenant.
    pub description: Option<Description>,

    /// The pipelines in this tenant, keyed by their unique pipeline id.
    pub pipelines: HashMap<PipelineId, PipelineSpec>,
}

impl TenantSpec {
    /// Validates the tenant specification.
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
