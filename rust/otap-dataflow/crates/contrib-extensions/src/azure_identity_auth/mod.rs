// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Identity Auth extension.
//!
//! Acquires and refreshes Azure access tokens and exposes them to data-path
//! nodes through the `BearerTokenProvider` capability. See
//! `docs/azure-identity-auth-extension.md` for the design.

pub mod config;
pub mod error;
mod auth;
mod extension;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_engine::ExtensionFactory;
use otap_df_engine::capability::BearerTokenProvider;
use otap_df_engine::config::ExtensionConfig;
use otap_df_engine::context::ExtensionContext;
use otap_df_engine::extension::{ExtensionBundle, ExtensionWrapper};
use otap_df_engine::extension_capabilities;
use otap_df_otap::OTAP_EXTENSION_FACTORIES;
use tokio::sync::watch;

use self::auth::Auth;
use self::config::Config;
use self::extension::AzureIdentityAuthExtension;

/// URN under which this extension is registered.
pub const AZURE_IDENTITY_AUTH_URN: &str = "urn:microsoft:extension:azure_identity_auth";

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
    _ext_ctx: &ExtensionContext,
    name: otap_df_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    // Validate config now so a bad config fails fast at wiring time.
    let config = parse_config(&ext_config.config)?;

    // Placeholder credential for now; the Azure-backed implementation lands in
    // a later change.
    let auth = Auth::new(&config).map_err(|e| ConfigError::InvalidUserConfig {
        error: format!("failed to initialize Azure credential: {e}"),
    })?;

    // Empty token cache; the background refresh loop publishes the first token.
    let (tx, _rx) = watch::channel(None);

    let extension = AzureIdentityAuthExtension::new(&name, auth, tx);

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe()
        .shared::<AzureIdentityAuthExtension>(extension)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the Azure Identity Auth extension.
#[allow(unsafe_code)]
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
