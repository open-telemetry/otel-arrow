// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Small database-neutral polling configuration.

use serde::Deserialize;
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
///
/// This models the generic database design's output contract. The initial
/// Oracle public schema does not expose these controls, so Oracle uses the
/// default all-column body and watermark-derived event time.
#[derive(Clone, Debug, Default)]
pub struct OutputConfig {
    /// Source columns included in the body. An empty list includes every column.
    pub include_columns: Vec<String>,
    /// Source-column to typed OTLP attribute-name mappings.
    pub attributes: BTreeMap<String, String>,
    /// Optional result column used as the OTLP event timestamp.
    pub timestamp_column: Option<String>,
    /// Result columns that must exist even when not emitted specially.
    pub validation_columns: Vec<String>,
}

impl OutputConfig {
    /// Validates mappings that do not require live result metadata.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_unique_names(
            "query.output.include_columns",
            self.include_columns.iter().map(String::as_str),
        )?;
        validate_mapping_names("query.output.attributes", &self.attributes)?;

        if let Some(column) = &self.timestamp_column {
            validate_name("query.output.timestamp_column", column)?;
        }
        for column in &self.validation_columns {
            validate_name("query.output.validation_columns", column)?;
        }
        Ok(())
    }
}

impl PollingConfig {
    /// Validates timing and row bounds.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.interval.is_zero() {
            return Err(ConfigError::ZeroInterval);
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

fn validate_mapping_names(
    field: &'static str,
    mappings: &BTreeMap<String, String>,
) -> Result<(), ConfigError> {
    validate_unique_names(field, mappings.keys().map(String::as_str))?;
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
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, OutputConfig, PollingConfig};
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

    /// Scenario: Two source columns target attribute names that differ only by case.
    /// Guarantees: Static validation rejects ambiguous OTLP attribute names.
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
}
