// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the Resource Validator Processor

use otap_df_config::error::Error as ConfigError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration for the Resource Validator Processor
///
/// This processor validates that a required resource attribute exists and its value
/// is in an allowed list. Requests that fail validation are NACKed with a permanent
/// error, enabling clients to detect misconfiguration immediately.
///
/// # Example Configuration
/// ```yaml
/// processors:
///   resource_validator:
///     required_attribute: "microsoft.resourceId"
///     allowed_values:
///       - "/subscriptions/xxx/resourceGroups/yyy/..."
///     case_insensitive: true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The resource attribute key that must be present on all resources.
    /// Default: "microsoft.resourceId"
    #[serde(default = "default_required_attribute")]
    pub required_attribute: String,

    /// List of allowed values for the required attribute.
    /// If empty, only presence validation is performed.
    #[serde(default)]
    pub allowed_values: Vec<String>,

    /// Whether to perform case-insensitive comparison of attribute values.
    /// Default: false
    #[serde(default)]
    pub case_insensitive: bool,
}

fn default_required_attribute() -> String {
    "microsoft.resourceId".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            required_attribute: default_required_attribute(),
            allowed_values: Vec::new(),
            case_insensitive: false,
        }
    }
}

impl Config {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.required_attribute.is_empty() {
            return Err(ConfigError::InvalidUserConfig {
                error: "required_attribute cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    /// Returns a pre-processed set of allowed values for efficient lookup.
    /// If case_insensitive is true, all values are lowercased.
    #[must_use]
    pub fn allowed_values_set(&self) -> HashSet<String> {
        if self.case_insensitive {
            self.allowed_values
                .iter()
                .map(|v| v.to_lowercase())
                .collect()
        } else {
            self.allowed_values.iter().cloned().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.required_attribute, "microsoft.resourceId");
        assert!(config.allowed_values.is_empty());
        assert!(!config.case_insensitive);
    }

    #[test]
    fn test_validate_empty_attribute() {
        let config = Config {
            required_attribute: "".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_allowed_values_set_case_sensitive() {
        let config = Config {
            allowed_values: vec!["Value1".to_string(), "Value2".to_string()],
            case_insensitive: false,
            ..Default::default()
        };
        let set = config.allowed_values_set();
        assert!(set.contains("Value1"));
        assert!(set.contains("Value2"));
        assert!(!set.contains("value1"));
    }

    #[test]
    fn test_allowed_values_set_case_insensitive() {
        let config = Config {
            allowed_values: vec!["Value1".to_string(), "VALUE2".to_string()],
            case_insensitive: true,
            ..Default::default()
        };
        let set = config.allowed_values_set();
        assert!(set.contains("value1"));
        assert!(set.contains("value2"));
        assert!(!set.contains("Value1"));
    }

    #[test]
    fn test_deserialize_config() {
        let json = r#"{
            "required_attribute": "my.attribute",
            "allowed_values": ["val1", "val2"],
            "case_insensitive": true
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.required_attribute, "my.attribute");
        assert_eq!(config.allowed_values, vec!["val1", "val2"]);
        assert!(config.case_insensitive);
    }
}
