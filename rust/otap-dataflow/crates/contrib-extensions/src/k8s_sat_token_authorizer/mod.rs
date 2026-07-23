// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kubernetes SAT (service-account-token) authorizer extension.
//!
//! Authenticates and admits inbound Kubernetes service-account tokens for
//! data-path nodes through the `BearerTokenAuthorizer` capability. Each token is
//! validated via the Kubernetes `TokenReview` API (authentication) and the
//! resulting service account is checked against a configured allow-list
//! (admission). See `docs/k8s-sat-token-authorizer-extension.md` for the design.

pub mod config;
pub mod error;
mod extension;
mod metrics;
mod reviewer;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_engine::ExtensionFactory;
use otap_df_engine::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer;
use otap_df_engine::config::ExtensionConfig;
use otap_df_engine::context::ExtensionContext;
use otap_df_engine::extension::wrapper::ExtensionVariant;
use otap_df_engine::extension::{ExtensionBundle, ExtensionWrapper};
use otap_df_engine::extension_capabilities;
use otap_df_otap::OTAP_EXTENSION_FACTORIES;

use self::config::Config;
use self::extension::K8sSatTokenAuthorizerExtension;
use self::metrics::{K8sSatTokenAuthorizerMetrics, K8sSatTokenAuthorizerMetricsTracker};

/// URN under which this extension is registered.
pub const K8S_SAT_TOKEN_AUTHORIZER_URN: &str = "urn:otel:extension:k8s_sat_token_authorizer";

/// Deserializes and validates the extension's user configuration.
fn parse_config(config: &serde_json::Value) -> Result<Config, ConfigError> {
    let parsed: Config =
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })?;
    parsed
        .validate()
        .map_err(|error| ConfigError::InvalidUserConfig { error })?;
    Ok(parsed)
}

/// Static config validation hook for the factory.
fn validate_config(config: &serde_json::Value) -> Result<(), ConfigError> {
    parse_config(config).map(|_| ())
}

/// Builds a `K8sSatTokenAuthorizerExtension` bundle.
fn create(
    ext_ctx: &ExtensionContext,
    name: otap_df_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    // Validate config now so a bad config fails fast at wiring time.
    let config = parse_config(&ext_config.config)?;
    let allowed = config.allowed_service_account_set();

    // Register a dedicated entity + metric set for this extension instance.
    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set =
        ext_ctx.register_metric_set_for_entity::<K8sSatTokenAuthorizerMetrics>(entity_key);
    let tracker = K8sSatTokenAuthorizerMetricsTracker::new(metric_set);

    let extension = K8sSatTokenAuthorizerExtension::new(
        &name,
        config.audiences.clone(),
        allowed,
        config.cache_ttl,
        config.cache_max_entries,
        tracker,
    );

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe_timeout_override(config.startup_timeout)
        .shared::<K8sSatTokenAuthorizerExtension>(extension)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the Kubernetes SAT authorizer extension.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static K8S_SAT_TOKEN_AUTHORIZER_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: K8S_SAT_TOKEN_AUTHORIZER_URN,
    description: "Active+Shared extension exposing BearerTokenAuthorizer via Kubernetes TokenReview",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: K8sSatTokenAuthorizerExtension => [BearerTokenAuthorizer]
    )),
    create,
    validate_config,
};
