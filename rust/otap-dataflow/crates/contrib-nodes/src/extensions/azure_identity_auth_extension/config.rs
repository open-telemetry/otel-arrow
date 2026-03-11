// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for the Azure Identity Auth Extension.

use serde::Deserialize;

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
    pub fn validate(&self) -> Result<(), super::error::Error> {
        if self.scope.is_empty() {
            return Err(super::error::Error::Config(
                "OAuth scope cannot be empty".to_string(),
            ));
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
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.method, AuthMethod::ManagedIdentity);
        assert!(config.client_id.is_none());
        assert_eq!(config.scope, "https://management.azure.com/.default");
    }

    #[test]
    fn auth_method_display() {
        assert_eq!(
            format!("{}", AuthMethod::ManagedIdentity),
            "managed_identity"
        );
        assert_eq!(format!("{}", AuthMethod::Development), "development");
    }

    #[test]
    fn config_validation_empty_scope() {
        let config = Config {
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            scope: String::new(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_validation_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_deserialize_managed_identity() {
        let json = serde_json::json!({
            "method": "managed_identity",
            "scope": "https://monitor.azure.com/.default"
        });
        let cfg: Config = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.method, AuthMethod::ManagedIdentity);
        assert_eq!(cfg.scope, "https://monitor.azure.com/.default");
    }

    #[test]
    fn config_deserialize_development() {
        let json = serde_json::json!({
            "method": "development",
            "scope": "https://monitor.azure.com/.default"
        });
        let cfg: Config = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.method, AuthMethod::Development);
    }

    #[test]
    fn config_rejects_unknown_fields() {
        let json = serde_json::json!({
            "method": "managed_identity",
            "scope": "https://test.scope",
            "unknown_field": true
        });
        assert!(serde_json::from_value::<Config>(json).is_err());
    }
}
