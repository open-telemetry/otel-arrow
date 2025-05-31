// SPDX-License-Identifier: Apache-2.0

//! The configuration for the data plane.

use crate::error::Error;
use crate::tenant::TenantSpec;
use crate::{Description, TenantId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A data plane specification containing multiple tenants.
/// This is the top-level configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataPlaneSpec {
    /// An optional description of this data plane.
    description: Option<Description>,

    /// The tenants in this data plane, keyed by their unique tenant id.
    tenants: HashMap<TenantId, TenantSpec>,
}

impl DataPlaneSpec {
    /// Creates a new `DataPlaneSpec` with the given JSON string.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let spec: DataPlaneSpec =
            serde_json::from_str(json).map_err(|e| Error::DeserializationError {
                tenant_id: None,
                pipeline_id: None,
                format: "JSON".to_string(),
                details: e.to_string(),
            })?;
        spec.validate()?;
        Ok(spec)
    }

    /// Validates the data plane specification and returns a [`Error::InvalidConfiguration`] error
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
