// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OAuth 2.0 Client Auth extension.
//!
//! Acquires and refreshes OAuth 2.0 access tokens using the client-credentials
//! grant and exposes them to data-path nodes through the `BearerTokenProvider`
//! capability. See `design.md` for the design.

otap_df_telemetry::otel_component_scope!(
    urn = OAUTH2_CLIENT_AUTH_URN,
    target = "otel.extension.oauth2_client_auth",
);

mod auth;
pub mod config;
pub mod error;
mod jwt_crypto;
mod metrics;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_engine::ExtensionFactory;
use otap_df_engine::capability::auth::bearer_token_provider::BearerTokenProvider;
use otap_df_engine::config::ExtensionConfig;
use otap_df_engine::context::ExtensionContext;
use otap_df_engine::extension::wrapper::ExtensionVariant;
use otap_df_engine::extension::{ExtensionBundle, ExtensionWrapper};
use otap_df_engine::extension_capabilities;
use otap_df_otap::OTAP_EXTENSION_FACTORIES;
use tokio::sync::watch;

use self::auth::Auth;
use self::config::Config;
use self::metrics::OAuth2ClientAuthMetrics;
use crate::common::token_refresh::{TokenProviderExtension, TokenProviderMetricsTracker};

/// The OAuth 2.0 Client Auth extension: the shared bearer-token refresher
/// driven by an OAuth 2.0 token endpoint.
pub type OAuth2ClientAuthExtension = TokenProviderExtension<Auth, OAuth2ClientAuthMetrics>;

/// URN under which this extension is registered.
pub const OAUTH2_CLIENT_AUTH_URN: &str = "urn:otel:extension:oauth2_client_auth";

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

/// Builds an `OAuth2ClientAuthExtension` bundle.
fn create(
    ext_ctx: &ExtensionContext,
    name: otap_df_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    // Validate config now so a bad config fails fast at wiring time.
    let config = parse_config(&ext_config.config)?;

    let auth = Auth::new(&config).map_err(|e| ConfigError::InvalidUserConfig {
        error: format!("failed to initialize OAuth2 client: {e}"),
    })?;

    // Register a dedicated entity + metric set for this extension instance.
    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set = ext_ctx.register_metric_set_for_entity::<OAuth2ClientAuthMetrics>(entity_key);
    let tracker = TokenProviderMetricsTracker::new(metric_set);

    // Empty token cache; the background refresh loop publishes the first token.
    let (tx, _rx) = watch::channel(None);

    let extension = OAuth2ClientAuthExtension::new(&name, auth, config.expiry_buffer, tx, tracker);

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe_timeout_override(config.startup_timeout)
        .shared::<OAuth2ClientAuthExtension>(extension)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the OAuth 2.0 Client Auth extension.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static OAUTH2_CLIENT_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: OAUTH2_CLIENT_AUTH_URN,
    description: "Active+Shared extension exposing BearerTokenProvider via the OAuth 2.0 client-credentials and JWT-bearer grants",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: OAuth2ClientAuthExtension => [BearerTokenProvider]
    )),
    create,
    validate_config,
};
