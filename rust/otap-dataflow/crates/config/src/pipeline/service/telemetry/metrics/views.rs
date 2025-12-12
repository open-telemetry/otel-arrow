// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics views level configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// OpenTelemetry Metrics View configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ViewConfig {
    /// Selector to match instruments for this view.
    pub selector: MetricSelector,

    /// Stream configuration for this view.
    pub stream: MetricStream,
}

/// OpenTelemetry Metric Selector configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricSelector {
    /// The name of the instrument to match.
    pub instrument_name: Option<String>,
    // TODO: Add more selector fields.
}

/// OpenTelemetry Metric Stream configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricStream {
    /// The name of the instrument.
    pub name: Option<String>,
    /// The description of the instrument.
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_config_deserialize() {
        let yaml_str = r#"
            selector:
              instrument_name: "requests.total"
            stream:
              name: "http.requests.total"
              description: "Total number of HTTP requests"
            "#;
        let config: ViewConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(
            config.selector.instrument_name.as_deref(),
            Some("requests.total")
        );
        assert_eq!(config.stream.name.as_deref(), Some("http.requests.total"));
        assert_eq!(
            config.stream.description.as_deref(),
            Some("Total number of HTTP requests")
        );
    }

    #[test]
    fn test_view_config_partial_deserialize() {
        let yaml_str = r#"
            selector:
              instrument_name: "cpu.usage"
            stream:
              name: "system.cpu.usage"
            "#;
        let config: ViewConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(
            config.selector.instrument_name.as_deref(),
            Some("cpu.usage")
        );
        assert_eq!(config.stream.name.as_deref(), Some("system.cpu.usage"));
        assert_eq!(config.stream.description.as_deref(), None);
    }

    #[test]
    fn test_view_config_empty_deserialize() {
        let yaml_str = r#"
            selector: {}
            stream: {}
            "#;
        let config: ViewConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.selector.instrument_name.as_deref(), None);
        assert_eq!(config.stream.name.as_deref(), None);
        assert_eq!(config.stream.description.as_deref(), None);
    }
}
