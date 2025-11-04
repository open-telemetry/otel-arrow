// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Service configuration specification.

pub mod telemetry;

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::service::telemetry::TelemetryConfig;

/// Configuration for the service.
#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    /// Internal telemetry configuration.
    #[serde(default)]
    pub telemetry_config: TelemetryConfig,

}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            telemetry_config: TelemetryConfig::default(),
        }
    }
}

impl ServiceConfig {
    /// Returns the telemetry configuration.
    #[must_use]
    pub fn telemetry(&self) -> &TelemetryConfig {
        &self.telemetry_config
    }
}
