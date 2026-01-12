// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::Error;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

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
}

impl Config {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Error> {
        // Validate auth configuration
        if self.auth.scope.is_empty() {
            return Err(Error::ConfigurationError(
                "Invalid configuration: auth scope must be non-empty".to_string(),
            ));
        }

        // Validate API configuration
        if self.api.dcr_endpoint.is_empty() {
            return Err(Error::ConfigurationError(
                "Invalid configuration: dcr_endpoint must be non-empty".to_string(),
            ));
        }
        if self.api.stream_name.is_empty() {
            return Err(Error::ConfigurationError(
                "Invalid configuration: stream_name must be non-empty".to_string(),
            ));
        }
        if self.api.dcr.is_empty() {
            return Err(Error::ConfigurationError(
                "Invalid configuration: dcr must be non-empty".to_string(),
            ));
        }

        self.validate_schema_unique_columns()?;

        Ok(())
    }

    fn validate_schema_unique_columns(&self) -> Result<(), Error> {
        let mut seen = HashSet::new();
        let mut duplicates = HashSet::new();

        for value in self.api.schema.resource_mapping.values() {
            if !seen.insert(value.clone()) {
                _ = duplicates.insert(value.clone());
            }
        }

        for value in self.api.schema.scope_mapping.values() {
            if !seen.insert(value.clone()) {
                _ = duplicates.insert(value.clone());
            }
        }

        for (key, value) in &self.api.schema.log_record_mapping {
            match value {
                Value::String(s) => {
                    if !seen.insert(s.clone()) {
                        _ = duplicates.insert(s.clone());
                    }
                }
                Value::Object(map) => {
                    if key != "attributes" {
                        return Err(Error::ConfigurationError(
                            "Invalid configuration: log_record_mapping key has invalid nested structure, only 'attributes' is allowed".to_string(),
                        ));
                    }

                    for (_, v) in map {
                        if let Value::String(s) = v {
                            if !seen.insert(s.clone()) {
                                _ = duplicates.insert(s.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !duplicates.is_empty() {
            let mut columns: Vec<String> = duplicates.into_iter().collect();
            columns.sort();
            return Err(Error::ConfigurationDuplicateColumnsError { columns });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
            Error::ConfigurationError(
                "Invalid configuration: dcr_endpoint must be non-empty".to_string()
            )
        );
    }

    #[test]
    fn test_schema_no_duplicate_columns() {
        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "https://example.com".to_string(),
                stream_name: "mystream".to_string(),
                dcr: "mydcr".to_string(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::from([
                        ("service.name".into(), "ServiceName".into()),
                        ("service.version".into(), "ServiceVersion".into()),
                    ]),
                    scope_mapping: HashMap::from([("scope.name".into(), "ScopeName".into())]),
                    log_record_mapping: HashMap::from([
                        ("body".into(), json!("Body")),
                        ("severity_text".into(), json!("Severity")),
                        (
                            "attributes".into(),
                            json!({"user.id": "UserId", "request.path": "RequestPath"}),
                        ),
                    ]),
                },
            },
            auth: AuthConfig::default(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_schema_duplicate_columns() {
        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "https://example.com".to_string(),
                stream_name: "mystream".to_string(),
                dcr: "mydcr".to_string(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::from([
                        ("service.name".into(), "Name".into()), // "Name" - first occurrence
                    ]),
                    scope_mapping: HashMap::from([
                        ("scope.name".into(), "Name".into()), // "Name" - duplicate!
                    ]),
                    log_record_mapping: HashMap::from([
                        ("body".into(), json!("Body")),
                        ("severity_text".into(), json!("Body")), // "Body" - duplicate!
                        ("attributes".into(), json!({"user.name": "Name"})), // "Name" - another duplicate!
                    ]),
                },
            },
            auth: AuthConfig::default(),
        };

        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ConfigurationDuplicateColumnsError { columns } => {
                assert!(
                    columns.contains(&"Name".to_string()) && columns.contains(&"Body".to_string())
                );
            }
            other => panic!(
                "Expected ConfigurationDuplicateColumnsError, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_schema_duplicate_columns_in_nested_log_record_mapping() {
        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "https://example.com".to_string(),
                stream_name: "mystream".to_string(),
                dcr: "mydcr".to_string(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::from([(
                        "service.name".into(),
                        "ServiceName".into(),
                    )]),
                    scope_mapping: HashMap::from([("scope.name".into(), "ScopeName".into())]),
                    log_record_mapping: HashMap::from([
                        ("body".into(), json!("Body")),
                        (
                            "attributes".into(),
                            json!({
                                "user.id": "UserId",
                                "user.name": "UserName",
                                "request.user": "UserName"  // "UserName" - duplicate within nested!
                            }),
                        ),
                    ]),
                },
            },
            auth: AuthConfig::default(),
        };

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, Error::ConfigurationDuplicateColumnsError { columns } if columns.contains(&"UserName".to_string()))
        );
    }

    #[test]
    fn test_schema_nested_object_only_allowed_for_attributes() {
        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "https://example.com".to_string(),
                stream_name: "mystream".to_string(),
                dcr: "mydcr".to_string(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::new(),
                    scope_mapping: HashMap::new(),
                    log_record_mapping: HashMap::from([
                        ("body".into(), json!({"nested": "NotAllowed"})), // object for non-attributes key
                    ]),
                },
            },
            auth: AuthConfig::default(),
        };

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, Error::ConfigurationError(msg) if msg == "Invalid configuration: log_record_mapping key has invalid nested structure, only 'attributes' is allowed")
        );
    }
}
