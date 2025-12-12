// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Service-level telemetry configurations.

pub mod telemetry;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::service::telemetry::TelemetryConfig;

/// Service-level telemetry configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ServiceConfig {
    /// The telemetry backend to which to report metrics.
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

#[cfg(test)]
mod tests {
    use crate::pipeline::service::telemetry::AttributeValue;

    use super::*;

    #[test]
    fn test_service_config_deserialize() {
        let yaml_str = r#"
            telemetry:
                reporting_channel_size: 200
                reporting_interval: "2s"
                resource:
                    service.name: "example_service"
            "#;

        let config: ServiceConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(config.telemetry.reporting_channel_size, 200);
        assert_eq!(config.telemetry.reporting_interval.as_secs(), 2);
        if let Some(AttributeValue::String(value)) = config.telemetry.resource.get("service.name") {
            assert_eq!(value, "example_service");
        } else {
            panic!("service.name attribute not found or not a String");
        }
    }
}
