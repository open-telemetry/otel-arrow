// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Identity Auth extension.
//!
//! Acquires and refreshes Azure access tokens and exposes them to data-path
//! nodes through the `BearerTokenProvider` capability. See
//! `design.md` for the design.

otap_df_telemetry::otel_component_scope!(
    urn = AZURE_IDENTITY_AUTH_URN,
    target = "microsoft.extension.azure_identity_auth",
);

mod auth;
pub mod config;
pub mod error;
mod metrics;

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::Duration;

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
use self::metrics::AzureIdentityAuthMetrics;
use crate::common::token_refresh::{TokenProviderExtension, TokenProviderMetricsTracker};

/// The Azure Identity Auth extension: the shared bearer-token refresher driven
/// by an Azure credential.
pub type AzureIdentityAuthExtension = TokenProviderExtension<Auth, AzureIdentityAuthMetrics>;

/// URN under which this extension is registered.
pub const AZURE_IDENTITY_AUTH_URN: &str = "urn:microsoft:extension:azure_identity_auth";

/// Refresh this many seconds before `expires_on`. Not user-configurable: Azure
/// token lifetimes are fixed by the platform.
const TOKEN_EXPIRY_BUFFER_SECS: u64 = 299;

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

/// Builds an `AzureIdentityAuthExtension` bundle.
fn create(
    ext_ctx: &ExtensionContext,
    name: otap_df_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    // Validate config now so a bad config fails fast at wiring time.
    let config = parse_config(&ext_config.config)?;

    let auth = Auth::new(&config).map_err(|e| ConfigError::InvalidUserConfig {
        error: format!("failed to initialize Azure credential: {e}"),
    })?;

    // Register a dedicated entity + metric set for this extension instance.
    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set = ext_ctx.register_metric_set_for_entity::<AzureIdentityAuthMetrics>(entity_key);
    let tracker = TokenProviderMetricsTracker::new(metric_set);

    // Empty token cache; the background refresh loop publishes the first token.
    let (tx, _rx) = watch::channel(None);

    let extension = AzureIdentityAuthExtension::new(
        &name,
        auth,
        Duration::from_secs(TOKEN_EXPIRY_BUFFER_SECS),
        tx,
        tracker,
    );

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe_timeout_override(config.startup_timeout)
        .shared::<AzureIdentityAuthExtension>(extension)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the Azure Identity Auth extension.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static AZURE_IDENTITY_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: AZURE_IDENTITY_AUTH_URN,
    description: "Active+Shared extension exposing BearerTokenProvider via azure_identity",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: AzureIdentityAuthExtension => [BearerTokenProvider]
    )),
    create,
    validate_config,
};
