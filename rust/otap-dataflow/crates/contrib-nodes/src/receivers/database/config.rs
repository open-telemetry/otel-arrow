// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Small database-neutral polling configuration.

use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

/// Bounds and timing shared by every database receiver.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollingConfig {
    /// Delay between completed query executions.
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    /// Native database call timeout.
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// Maximum rows requested from the driver at once.
    pub fetch_size: usize,
    /// Hard row limit for one poll.
    pub max_rows_per_poll: usize,
}

/// Database-row to OTLP log mapping selected before ingestion.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    /// Source columns included in the body. An empty list includes every column.
    #[serde(default)]
    pub include_columns: Vec<String>,
    /// Source-column to body-field rename mappings.
    #[serde(default)]
    pub rename_columns: BTreeMap<String, String>,
    /// Source-column to typed OTLP attribute-name mappings.
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
    /// Optional result column used as the OTLP event timestamp.
    #[serde(default)]
    pub timestamp_column: Option<String>,
    /// Result columns that must exist even when not emitted specially.
    #[serde(skip)]
    pub validation_columns: Vec<String>,
    /// Operator-approved scalar database identity attributes.
    #[serde(default)]
    pub resource_attributes: BTreeMap<String, Value>,
}

/// Scope applied to a conversion failure.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorPolicy {
    /// Discard this result and retry at the next interval.
    #[default]
    FailBatch,
    /// Stop polling this query while keeping the receiver responsive.
    StopQuery,
    /// Stop the receiver and surface the failure.
    StopReceiver,
}

impl OutputConfig {
    /// Validates mappings that do not require live result metadata.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_unique_names(
            "query.output.include_columns",
            self.include_columns.iter().map(String::as_str),
        )?;
        validate_mapping_names(
            "query.output.rename_columns",
            &self.rename_columns,
        )?;
        validate_mapping_names("query.output.attributes", &self.attributes)?;

        if let Some(column) = &self.timestamp_column {
            validate_name("query.output.timestamp_column", column)?;
        }
        for column in &self.validation_columns {
            validate_name("query.output.validation_columns", column)?;
        }
        validate_resource_attributes(&self.resource_attributes)
    }
}

impl PollingConfig {
    /// Validates timing and row bounds.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.interval.is_zero() {
            return Err(ConfigError::ZeroInterval);
        }

        fn validate_mapping_names(
            field: &'static str,
            mappings: &BTreeMap<String, String>,
        ) -> Result<(), ConfigError> {
            for source in mappings.keys() {
                validate_name(field, source)?;
            }
            validate_unique_names(field, mappings.values().map(String::as_str))
        }

        fn validate_unique_names<'a>(
            field: &'static str,
            names: impl IntoIterator<Item = &'a str>,
        ) -> Result<(), ConfigError> {
            let mut normalized = BTreeSet::new();
            for name in names {
                validate_name(field, name)?;
                if !normalized.insert(name.to_ascii_lowercase()) {
                    return Err(ConfigError::DuplicateName { field });
                }
            }
            Ok(())
        }

        fn validate_name(field: &'static str, name: &str) -> Result<(), ConfigError> {
            if name.trim().is_empty() {
                Err(ConfigError::EmptyName { field })
            } else {
                Ok(())
            }
        }

        fn validate_resource_attributes(
            attributes: &BTreeMap<String, Value>,
        ) -> Result<(), ConfigError> {
            // Connection endpoints and credentials are deliberately excluded from
            // operator metadata because OTLP resources can leave the trust boundary.
            const FORBIDDEN: [&str; 5] = [
                "db.connection_string",
                "db.system.name",
                "db.endpoint",
                "server.address",
                "server.port",
            ];
            for (key, value) in attributes {
                validate_name("query.output.resource_attributes", key)?;
                if FORBIDDEN
                    .iter()
                    .any(|forbidden| forbidden.eq_ignore_ascii_case(key))
                {
                    return Err(ConfigError::ForbiddenResourceAttribute { key: key.clone() });
                }
                if !(value.is_string() || value.is_boolean() || value.is_number()) {
                    return Err(ConfigError::NonScalarResourceAttribute { key: key.clone() });
                }
            }
            Ok(())
        }
        if self.timeout.is_zero() {
            return Err(ConfigError::ZeroTimeout);
        }
        if self.fetch_size == 0 {
            return Err(ConfigError::ZeroFetchSize);
        }
        if self.max_rows_per_poll == 0 {
            return Err(ConfigError::ZeroRowLimit);
        }
        if self.fetch_size > self.max_rows_per_poll {
            return Err(ConfigError::FetchSizeExceedsRowLimit);
        }
        Ok(())
    }
}

/// Invalid database polling configuration.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ConfigError {
    /// Poll interval is zero.
    #[error("query.interval must be greater than zero")]
    ZeroInterval,
    /// Query timeout is zero.
    #[error("query.timeout must be greater than zero")]
    ZeroTimeout,
    /// Driver fetch size is zero.
    #[error("query.fetch_size must be greater than zero")]
    ZeroFetchSize,
    /// Per-poll row limit is zero.
    #[error("query.max_rows_per_poll must be greater than zero")]
    ZeroRowLimit,
    /// Fetch size exceeds the poll ceiling.
    #[error("query.fetch_size must not exceed query.max_rows_per_poll")]
    FetchSizeExceedsRowLimit,
    /// A configured mapping name is empty.
    #[error("{field} names must not be empty")]
    EmptyName {
        /// Invalid configuration field.
        field: &'static str,
    },
    /// Configured names collide under OTLP's case-insensitive matching policy.
    #[error("{field} contains duplicate names")]
    DuplicateName {
        /// Invalid configuration field.
        field: &'static str,
    },
    /// A resource attribute could expose connection identity or override receiver identity.
    #[error("resource attribute '{key}' is reserved or may expose connection identity")]
    ForbiddenResourceAttribute {
        /// Rejected resource attribute key.
        key: String,
    },
    /// OTLP resource identity supports only scalar values in this receiver.
    #[error("resource attribute '{key}' must be a string, boolean, or number")]
    NonScalarResourceAttribute {
        /// Rejected resource attribute key.
        key: String,
    },
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, OutputConfig, PollingConfig};
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::time::Duration;

    /// Scenario: A driver fetch could exceed the complete poll row ceiling.
    /// Guarantees: Shared validation rejects contradictory row bounds.
    #[test]
    fn rejects_fetch_size_above_poll_limit() {
        let config = PollingConfig {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            fetch_size: 101,
            max_rows_per_poll: 100,
        };

        assert_eq!(
            config.validate(),
            Err(ConfigError::FetchSizeExceedsRowLimit)
        );
    }

    /// Scenario: Two source columns are configured with target names that differ only by case.
    /// Guarantees: Static validation rejects ambiguous OTLP field and attribute names.
    #[test]
    fn rejects_duplicate_mapping_targets() {
        let output = OutputConfig {
            attributes: BTreeMap::from([
                ("AUDIT_ID".to_owned(), "audit.id".to_owned()),
                ("ACTION_ID".to_owned(), "AUDIT.ID".to_owned()),
            ]),
            ..OutputConfig::default()
        };

        assert_eq!(
            output.validate(),
            Err(ConfigError::DuplicateName {
                field: "query.output.attributes"
            })
        );
    }

    /// Scenario: Operator resource metadata contains a nested value or connection endpoint.
    /// Guarantees: Only safe scalar database identity is admitted to exported resources.
    #[test]
    fn rejects_unsafe_resource_identity() {
        let nested = OutputConfig {
            resource_attributes: BTreeMap::from([(
                "service.instance.metadata".to_owned(),
                json!({"region": "west"}),
            )]),
            ..OutputConfig::default()
        };
        assert!(matches!(
            nested.validate(),
            Err(ConfigError::NonScalarResourceAttribute { .. })
        ));

        let endpoint = OutputConfig {
            resource_attributes: BTreeMap::from([(
                "server.address".to_owned(),
                json!("database.contoso.com"),
            )]),
            ..OutputConfig::default()
        };
        assert!(matches!(
            endpoint.validate(),
            Err(ConfigError::ForbiddenResourceAttribute { .. })
        ));
    }
}
