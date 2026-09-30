// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use std::time::Duration;

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

/// Scope attribute selection for one instrumentation scope name.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScopeAttributes {
    /// Instrumentation scope name, or `*` for all scopes.
    pub name: String,
    /// Attribute keys to include, or `*` for all attributes.
    pub keys: Vec<String>,
}

/// Authentication applied to Geneva metrics publication requests.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthConfig {
    /// Use managed identity through a bound `bearer_token_provider` capability.
    Bearer,
}

/// Configuration for the Geneva metrics exporter.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Full Geneva metrics publication endpoint.
    pub endpoint: String,

    /// Monitoring account receiving the metrics.
    pub monitoring_account: String,

    /// Default metric namespace when OTLP attributes do not override it.
    pub metric_namespace: String,

    /// Maximum time allowed for one HTTP publication request.
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,

    /// Publication authentication mode.
    pub auth: AuthConfig,

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
    /// Validates the exporter configuration.
    pub fn validate(&self) -> Result<(), String> {
        let endpoint = self.endpoint.trim();
        if endpoint.is_empty() {
            return Err("endpoint must not be empty".to_string());
        }
        let endpoint = reqwest::Url::parse(endpoint)
            .map_err(|error| format!("endpoint must be a valid URL: {error}"))?;
        if !matches!(endpoint.scheme(), "http" | "https") {
            return Err("endpoint must use HTTP or HTTPS".to_string());
        }
        if self.monitoring_account.trim().is_empty() {
            return Err("monitoring_account must not be empty".to_string());
        }
        if self.metric_namespace.trim().is_empty() {
            return Err("metric_namespace must not be empty".to_string());
        }
        if self.timeout.is_zero() {
            return Err("timeout must be greater than zero".to_string());
        }
        if endpoint.scheme() != "https" {
            return Err("managed identity authentication requires an HTTPS endpoint".to_string());
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

impl From<&Config> for super::otlp_to_geneva::Config {
    fn from(config: &Config) -> Self {
        Self {
            monitoring_account: config.monitoring_account.clone(),
            metric_namespace: config.metric_namespace.clone(),
            resource_attributes: config.resource_attributes.clone(),
            honor_resource_attributes: config.honor_resource_attributes,
            scope_attributes: config
                .scope_attributes
                .iter()
                .map(|selection| super::otlp_to_geneva::ScopeAttributes {
                    name: selection.name.clone(),
                    keys: selection.keys.clone(),
                })
                .collect(),
            honor_scope_attributes: config.honor_scope_attributes,
            disable_exemplars: config.disable_exemplars,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> Config {
        Config {
            endpoint: "https://example.test/metrics".to_string(),
            monitoring_account: "example-account".to_string(),
            metric_namespace: "example-namespace".to_string(),
            timeout: default_timeout(),
            auth: AuthConfig::Bearer,
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        }
    }

    /// Scenario: A configuration supplies an endpoint and monitoring account.
    /// Guarantees: The minimal exporter configuration is accepted.
    #[test]
    fn accepts_minimal_config() {
        let config = valid_config();

        assert_eq!(config.validate(), Ok(()));
    }

    /// Scenario: A configuration omits a usable endpoint.
    /// Guarantees: Invalid endpoint configuration is rejected before startup.
    #[test]
    fn rejects_empty_endpoint() {
        let mut config = valid_config();
        config.endpoint = " ".to_string();

        assert_eq!(
            config.validate(),
            Err("endpoint must not be empty".to_string())
        );
    }

    /// Scenario: A configuration supplies a malformed or unsupported endpoint URL.
    /// Guarantees: Endpoint errors are reported during component validation instead of first publication.
    #[test]
    fn rejects_invalid_endpoint_urls() {
        let mut malformed = valid_config();
        malformed.endpoint = "not a URL".to_string();
        let malformed_error = malformed
            .validate()
            .expect_err("malformed endpoint should fail");
        assert!(malformed_error.starts_with("endpoint must be a valid URL:"));

        let mut unsupported_scheme = valid_config();
        unsupported_scheme.endpoint = "ftp://example.test/metrics".to_string();
        assert_eq!(
            unsupported_scheme.validate(),
            Err("endpoint must use HTTP or HTTPS".to_string())
        );
    }

    /// Scenario: A configuration omits the fallback metric namespace.
    /// Guarantees: OTLP metrics cannot start without a namespace used when no override attribute exists.
    #[test]
    fn rejects_empty_metric_namespace() {
        let mut config = valid_config();
        config.metric_namespace = " ".to_string();

        assert_eq!(
            config.validate(),
            Err("metric_namespace must not be empty".to_string())
        );
    }

    /// Scenario: Bearer authentication targets an unencrypted HTTP endpoint.
    /// Guarantees: Credentials cannot be configured for transmission without TLS.
    #[test]
    fn rejects_bearer_auth_for_http_endpoint() {
        let mut config = valid_config();
        config.endpoint = "http://example.test/metrics".to_string();
        config.auth = AuthConfig::Bearer;

        assert_eq!(
            config.validate(),
            Err("managed identity authentication requires an HTTPS endpoint".to_string())
        );
    }

    /// Scenario: Runtime-only constraints contain a zero duration or empty scope selection.
    /// Guarantees: Invalid runtime state is rejected before exporter startup.
    #[test]
    fn rejects_invalid_runtime_constraints() {
        let mut zero_timeout = valid_config();
        zero_timeout.timeout = Duration::ZERO;

        let mut invalid_scope = valid_config();
        invalid_scope.scope_attributes = vec![ScopeAttributes {
            name: " ".to_string(),
            keys: vec!["dimension".to_string()],
        }];

        for (config, expected) in [
            (zero_timeout, "timeout must be greater than zero"),
            (
                invalid_scope,
                "scope_attributes entries require a name and at least one key",
            ),
        ] {
            assert_eq!(config.validate(), Err(expected.to_string()));
        }
    }
}
