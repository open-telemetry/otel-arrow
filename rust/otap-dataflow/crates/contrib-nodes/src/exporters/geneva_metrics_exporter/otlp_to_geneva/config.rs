// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for OTLP metrics to Geneva metrics mapping.

use serde::Deserialize;

const BANNED_MONITORING_ACCOUNTS: &[&str] = &[
    "",
    "%MDM_MONITORING_ACCOUNT%",
    "%MONITORING_MDM_ACCOUNT_NAME%",
    "!AZUREDB_METRICS_ACCOUNT!",
    "!AZUREDB_SHOEBOX_METRICS_ACCOUNT!",
    "<unknown>",
    "Default",
    "MDM ACCOUNT",
    "<monitoringAccountPlaceholder>",
    "*",
    "{{AccountName}}",
    "<<Metric Account Name>>",
];

/// Scope attribute selection for one instrumentation scope name.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScopeAttributes {
    /// Instrumentation scope name, or `*` for all scopes.
    pub name: String,
    /// Attribute keys to include, or `*` for all attributes.
    pub keys: Vec<String>,
}

/// Configuration for mapping OTLP metrics to Geneva metrics.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Monitoring account receiving the metrics.
    pub monitoring_account: String,

    /// Default metric namespace when OTLP attributes do not override it.
    pub metric_namespace: String,

    /// Resource attribute keys to add as dimensions, or `*` for all attributes.
    #[serde(default)]
    pub resource_attributes: Vec<String>,

    /// Whether resource dimensions override point and scope dimensions.
    #[serde(default)]
    pub honor_resource_attributes: bool,

    /// Scope-specific attribute keys to add as dimensions.
    #[serde(default)]
    pub scope_attributes: Vec<ScopeAttributes>,

    /// Whether scope dimensions override point and resource dimensions.
    #[serde(default)]
    pub honor_scope_attributes: bool,

    /// Whether OTLP exemplars should be omitted.
    #[serde(default)]
    pub disable_exemplars: bool,
}

impl Config {
    /// Validates the mapping configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.monitoring_account.trim().is_empty() {
            return Err("monitoring_account must not be empty".to_string());
        }
        if BANNED_MONITORING_ACCOUNTS.contains(&self.monitoring_account.as_str()) {
            return Err(format!(
                "monitoring_account {:?} is a reserved placeholder and cannot be used",
                self.monitoring_account
            ));
        }
        if self.metric_namespace.trim().is_empty() {
            return Err("metric_namespace must not be empty".to_string());
        }
        if self
            .scope_attributes
            .iter()
            .any(|selection| selection.name.trim().is_empty() || selection.keys.is_empty())
        {
            return Err("scope_attributes entries require a name and at least one key".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A configuration supplies a monitoring account and fallback metric namespace.
    /// Guarantees: The minimal OTLP mapping configuration is accepted.
    #[test]
    fn accepts_minimal_config() {
        let config = Config {
            monitoring_account: "example-account".to_string(),
            metric_namespace: "example-namespace".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        };

        assert_eq!(config.validate(), Ok(()));
    }

    /// Scenario: A configuration omits the fallback monitoring account.
    /// Guarantees: OTLP metrics cannot be mapped without a default destination account.
    #[test]
    fn rejects_empty_monitoring_account() {
        let config = Config {
            monitoring_account: " ".to_string(),
            metric_namespace: "example-namespace".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        };

        assert_eq!(
            config.validate(),
            Err("monitoring_account must not be empty".to_string())
        );
    }

    /// Scenario: A configuration's monitoring account is a reserved placeholder value.
    /// Guarantees: OTLP metrics cannot be mapped to a monitoring account known to be invalid.
    #[test]
    fn rejects_banned_monitoring_account() {
        for account in BANNED_MONITORING_ACCOUNTS {
            if account.trim().is_empty() {
                continue;
            }
            let config = Config {
                monitoring_account: (*account).to_string(),
                metric_namespace: "example-namespace".to_string(),
                resource_attributes: Vec::new(),
                honor_resource_attributes: false,
                scope_attributes: Vec::new(),
                honor_scope_attributes: false,
                disable_exemplars: false,
            };

            assert!(config.validate().is_err());
        }
    }

    /// Scenario: A configuration omits the fallback metric namespace.
    /// Guarantees: OTLP metrics cannot start without a namespace used when no override attribute exists.
    #[test]
    fn rejects_empty_metric_namespace() {
        let config = Config {
            monitoring_account: "example-account".to_string(),
            metric_namespace: " ".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        };

        assert_eq!(
            config.validate(),
            Err("metric_namespace must not be empty".to_string())
        );
    }

    /// Scenario: A scope attribute selection has no usable scope name.
    /// Guarantees: Invalid scope attribute selection is rejected before mapping starts.
    #[test]
    fn rejects_invalid_scope_attribute_selection() {
        let config = Config {
            monitoring_account: "example-account".to_string(),
            metric_namespace: "example-namespace".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: vec![ScopeAttributes {
                name: " ".to_string(),
                keys: vec!["service.name".to_string()],
            }],
            honor_scope_attributes: false,
            disable_exemplars: false,
        };

        assert_eq!(
            config.validate(),
            Err("scope_attributes entries require a name and at least one key".to_string())
        );
    }
}
