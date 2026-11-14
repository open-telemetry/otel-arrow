// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::otlp_to_geneva::ScopeAttributes;
use serde::Deserialize;
use std::time::Duration;

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

/// Authentication applied to Geneva metrics publication requests.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthConfig {
    /// Use a bound `bearer_token_provider` capability.
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

    /// Maximum time allowed to publish all packets of one input.
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
        if endpoint.scheme() != "https" {
            return Err("bearer authentication requires an HTTPS endpoint".to_string());
        }
        if self.timeout.is_zero() {
            return Err("timeout must be greater than zero".to_string());
        }
        super::otlp_to_geneva::Config::from(self).validate()
    }
}

impl From<&Config> for super::otlp_to_geneva::Config {
    fn from(config: &Config) -> Self {
        Self {
            monitoring_account: config.monitoring_account.clone(),
            metric_namespace: config.metric_namespace.clone(),
            resource_attributes: config.resource_attributes.clone(),
            honor_resource_attributes: config.honor_resource_attributes,
            scope_attributes: config.scope_attributes.clone(),
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

    /// Scenario: Each field is set to an empty, malformed, non-HTTPS, zero, reserved, or incomplete value.
    /// Guarantees: Validation rejects every invalid configuration with a field-specific error before exporter startup.
    #[test]
    fn rejects_invalid_config() {
        let cases: [(fn(&mut Config), &str); 9] = [
            (|c| c.endpoint = " ".into(), "endpoint must not be empty"),
            (
                |c| c.endpoint = "not a URL".into(),
                "endpoint must be a valid URL:",
            ),
            (
                |c| c.endpoint = "http://example.test/metrics".into(),
                "bearer authentication requires an HTTPS endpoint",
            ),
            (
                |c| c.endpoint = "ftp://example.test/metrics".into(),
                "bearer authentication requires an HTTPS endpoint",
            ),
            (
                |c| c.timeout = Duration::ZERO,
                "timeout must be greater than zero",
            ),
            (
                |c| c.metric_namespace = " ".into(),
                "metric_namespace must not be empty",
            ),
            (|c| c.monitoring_account = "Default".into(), "reserved"),
            (|c| c.monitoring_account = "*".into(), "reserved"),
            (
                |c| {
                    c.scope_attributes = vec![ScopeAttributes {
                        name: " ".to_string(),
                        keys: vec!["dimension".to_string()],
                    }]
                },
                "scope_attributes entries require a name and at least one key",
            ),
        ];
        for (invalidate, expected) in cases {
            let mut config = valid_config();
            invalidate(&mut config);
            let error = config.validate().expect_err(expected);
            assert!(
                error.contains(expected),
                "expected {expected:?}, got {error:?}"
            );
        }
    }
}
