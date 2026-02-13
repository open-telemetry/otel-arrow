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
///     required_attribute_key: "cloud.resource_id"
///     allowed_values:
///       - "/subscriptions/xxx/resourceGroups/yyy/..."
///     case_sensitive: false  # optional, defaults to true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The resource attribute key that must be present on all resources.
    /// This is a required field with no default.
    pub required_attribute_key: String,

    /// List of allowed values for the required attribute.
    /// Empty list rejects all values.
    #[serde(default)]
    pub allowed_values: Vec<String>,

    /// Whether to perform case-sensitive comparison of attribute values.
    /// Note: this only affects `allowed_values` matching. The `required_attribute_key`
    /// key lookup is always case-sensitive.
    /// Default: true
    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: bool,
}

const fn default_case_sensitive() -> bool {
    true
}

impl Config {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.required_attribute_key.trim().is_empty() {
            return Err(ConfigError::InvalidUserConfig {
                error: "required_attribute_key cannot be empty".to_string(),
            });
        }
        if self.allowed_values.is_empty() {
            return Err(ConfigError::InvalidUserConfig {
                error: "allowed_values cannot be empty (would reject all data)".to_string(),
            });
        }
        Ok(())
    }

    /// Returns a pre-processed set of allowed values for efficient lookup.
    /// If case_sensitive is false, all values are lowercased.
    #[must_use]
    pub fn allowed_values_set(&self) -> HashSet<String> {
        if self.case_sensitive {
            self.allowed_values.iter().cloned().collect()
        } else {
            self.allowed_values
                .iter()
                .map(|v| v.to_lowercase())
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a config for testing
    fn test_config(required_attribute_key: &str, allowed_values: Vec<&str>) -> Config {
        Config {
            required_attribute_key: required_attribute_key.to_string(),
            allowed_values: allowed_values.into_iter().map(String::from).collect(),
            case_sensitive: true,
        }
    }

    #[test]
    fn test_validate_empty_attribute() {
        let config = Config {
            required_attribute_key: "".to_string(),
            allowed_values: vec!["value".to_string()],
            case_sensitive: true,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_allowed_values() {
        let config = test_config("cloud.resource_id", vec![]);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = test_config("cloud.resource_id", vec!["/subscriptions/123"]);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_allowed_values_set_case_sensitive() {
        let config = Config {
            required_attribute_key: "cloud.resource_id".to_string(),
            allowed_values: vec!["Value1".to_string(), "Value2".to_string()],
            case_sensitive: true,
        };
        let set = config.allowed_values_set();
        assert!(set.contains("Value1"));
        assert!(set.contains("Value2"));
        assert!(!set.contains("value1"));
    }

    #[test]
    fn test_allowed_values_set_case_insensitive() {
        let config = Config {
            required_attribute_key: "cloud.resource_id".to_string(),
            allowed_values: vec!["Value1".to_string(), "VALUE2".to_string()],
            case_sensitive: false,
        };
        let set = config.allowed_values_set();
        assert!(set.contains("value1"));
        assert!(set.contains("value2"));
        assert!(!set.contains("Value1"));
    }

    #[test]
    fn test_deserialize_config() {
        let json = r#"{
            "required_attribute_key": "my.attribute",
            "allowed_values": ["val1", "val2"],
            "case_sensitive": false
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.required_attribute_key, "my.attribute");
        assert_eq!(config.allowed_values, vec!["val1", "val2"]);
        assert!(!config.case_sensitive);
    }
}
