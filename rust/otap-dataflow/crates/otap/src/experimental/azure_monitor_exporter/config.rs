// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// Configuration for the Azure Monitor Exporter matching the Collector's schema.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// API configuration for Azure Monitor
    pub api: ApiConfig,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,
}

/// Authentication method for Azure
#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// Use Managed Identity (system or user-assigned with client_id)
    #[serde(alias = "msi", alias = "managed_identity")]
    #[default]
    ManagedIdentity,

    /// Use developer tools (Azure CLI, Azure Developer CLI)
    #[serde(alias = "dev", alias = "developer", alias = "cli")]
    Development,
}

/// Authentication configuration for Azure
#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    /// Authentication method to use
    #[serde(default)]
    pub method: AuthMethod,

    /// Client ID for user-assigned managed identity (optional)
    /// Only used when method is ManagedIdentity
    /// If not provided with ManagedIdentity, system-assigned identity will be used
    pub client_id: Option<String>,

    /// OAuth scope for token acquisition (defaults to "https://monitor.azure.com/.default")
    #[serde(default = "default_scope")]
    pub scope: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            method: AuthMethod::default(),
            client_id: None,
            scope: default_scope(),
        }
    }
}

fn default_scope() -> String {
    "https://monitor.azure.com/.default".to_string()
}

/// API configuration for connecting to Azure Monitor
#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    /// Data Collection Rule endpoint
    pub dcr_endpoint: String,

    /// Stream name for the logs
    pub stream_name: String,

    /// Data Collection Rule identifier
    pub dcr: String,

    /// Schema mapping configuration
    #[serde(default)]
    pub schema: SchemaConfig,
}

/// Schema mapping configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct SchemaConfig {
    /// Resource attribute mappings
    #[serde(default)]
    pub resource_mapping: HashMap<String, String>,

    /// Scope attribute mappings
    #[serde(default)]
    pub scope_mapping: HashMap<String, String>,

    /// Log record field mappings
    #[serde(default)]
    pub log_record_mapping: HashMap<String, Value>,

    /// Disable automatic schema mapping
    #[serde(default)]
    pub disable_schema_mapping: bool,
}

impl Config {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate auth configuration
        if self.auth.scope.is_empty() {
            return Err("Invalid configuration: auth scope must be non-empty".to_string());
        }

        // Validate API configuration
        if self.api.dcr_endpoint.is_empty() {
            return Err("Invalid configuration: dcr_endpoint must be non-empty".to_string());
        }
        if self.api.stream_name.is_empty() {
            return Err("Invalid configuration: stream_name must be non-empty".to_string());
        }
        if self.api.dcr.is_empty() {
            return Err("Invalid configuration: dcr must be non-empty".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "https://example.com".to_string(),
                stream_name: "mystream".to_string(),
                dcr: "mydcr".to_string(),
                schema: SchemaConfig::default(),
            },
            auth: AuthConfig {
                scope: "https://monitor.azure.com/.default".to_string(),
                client_id: Some("myclientid".to_string()),
                method: AuthMethod::ManagedIdentity,
            },
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config_missing_api_fields() {
        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "".to_string(),
                stream_name: "".to_string(),
                dcr: "".to_string(),
                schema: SchemaConfig::default(),
            },
            auth: AuthConfig::default(),
        };

        assert!(config.validate().is_err());
        assert_eq!(
            config.validate().unwrap_err(),
            "Invalid configuration: dcr_endpoint must be non-empty"
        );
    }
}
