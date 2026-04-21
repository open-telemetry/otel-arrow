// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics views level configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// OpenTelemetry Metrics View configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ViewConfig {
    /// Selector to match instruments for this view transformation.
    pub selector: MetricSelector,

    /// Stream configuration for the transformation expected.
    pub stream: MetricStream,
}

/// OpenTelemetry Metric Selector configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct MetricSelector {
    /// The name of the instrument to match.
    pub instrument_name: Option<String>,
    /// The instrumentation scope (meter) name to match.
    /// When set, the view only applies to instruments created under this scope.
    pub scope_name: Option<String>,
}

/// OpenTelemetry Metric Stream configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct MetricStream {
    /// The new name of the instrument matching the selector.
    pub name: Option<String>,
    /// The new description of the instrument matching the selector.
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
        assert_eq!(config.selector.scope_name.as_deref(), None);
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
        assert_eq!(config.selector.scope_name.as_deref(), None);
        assert_eq!(config.stream.name.as_deref(), None);
        assert_eq!(config.stream.description.as_deref(), None);
    }

    #[test]
    fn test_view_config_with_scope_name() {
        let yaml_str = r#"
            selector:
              instrument_name: "successful_rows"
              scope_name: "azure_monitor_exporter.metrics"
            stream:
              name: "exporter_sent_log_records"
              description: "Number of log records successfully sent by the exporter."
            "#;
        let config: ViewConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(
            config.selector.instrument_name.as_deref(),
            Some("successful_rows")
        );
        assert_eq!(
            config.selector.scope_name.as_deref(),
            Some("azure_monitor_exporter.metrics")
        );
        assert_eq!(
            config.stream.name.as_deref(),
            Some("exporter_sent_log_records")
        );
    }
}
