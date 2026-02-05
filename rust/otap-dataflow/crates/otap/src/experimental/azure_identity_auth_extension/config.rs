// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for the Azure Identity Auth Extension.

use serde::Deserialize;

use super::Error;

/// Authentication method for Azure.
#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// Use Managed Identity (system or user-assigned with client_id).
    #[serde(alias = "msi", alias = "managed_identity")]
    #[default]
    ManagedIdentity,

    /// Use developer tools (Azure CLI, Azure Developer CLI).
    #[serde(alias = "dev", alias = "developer", alias = "cli")]
    Development,
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::ManagedIdentity => write!(f, "managed_identity"),
            AuthMethod::Development => write!(f, "development"),
        }
    }
}

/// Configuration for the Azure Identity Auth Extension.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Authentication method to use.
    #[serde(default)]
    pub method: AuthMethod,

    /// Client ID for user-assigned managed identity (optional).
    /// Only used when method is ManagedIdentity.
    /// If not provided with ManagedIdentity, system-assigned identity will be used.
    pub client_id: Option<String>,

    /// OAuth scope for token acquisition.
    /// Defaults to "https://management.azure.com/.default" for general Azure management.
    #[serde(default = "default_scope")]
    pub scope: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            method: AuthMethod::default(),
            client_id: None,
            scope: default_scope(),
        }
    }
}

impl Config {
    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), Error> {
        // Validate scope is not empty
        if self.scope.is_empty() {
            return Err(Error::Config("OAuth scope cannot be empty".to_string()));
        }

        Ok(())
    }
}

fn default_scope() -> String {
    "https://management.azure.com/.default".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.method, AuthMethod::ManagedIdentity);
        assert!(config.client_id.is_none());
        assert_eq!(config.scope, "https://management.azure.com/.default");
    }

    #[test]
    fn test_auth_method_display() {
        assert_eq!(
            format!("{}", AuthMethod::ManagedIdentity),
            "managed_identity"
        );
        assert_eq!(format!("{}", AuthMethod::Development), "development");
    }

    #[test]
    fn test_config_validation_empty_scope() {
        let config = Config {
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            scope: "".to_string(),
        };
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_managed_identity_system_assigned() {
        let json = r#"{
            "method": "managed_identity",
            "scope": "https://monitor.azure.com/.default"
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.method, AuthMethod::ManagedIdentity);
        assert!(config.client_id.is_none());
    }

    #[test]
    fn test_deserialize_managed_identity_user_assigned() {
        let json = r#"{
            "method": "msi",
            "client_id": "12345-abcde",
            "scope": "https://monitor.azure.com/.default"
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.method, AuthMethod::ManagedIdentity);
        assert_eq!(config.client_id, Some("12345-abcde".to_string()));
    }

    #[test]
    fn test_deserialize_development() {
        let json = r#"{
            "method": "development"
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.method, AuthMethod::Development);
    }

    #[test]
    fn test_deserialize_with_defaults() {
        let json = r#"{}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.method, AuthMethod::ManagedIdentity);
        assert_eq!(config.scope, "https://management.azure.com/.default");
    }
}
