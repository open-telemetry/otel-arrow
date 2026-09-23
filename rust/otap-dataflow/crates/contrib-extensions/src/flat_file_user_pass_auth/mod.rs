// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file user pass extension.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = FLAT_FILE_USER_PASS_AUTH_URN,
    target = "otel.extension.flat_file_user_pass_auth",
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
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::extension::ExtensionUserConfig;
use otel_arrow_dfe_engine::ExtensionFactory;
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::{
    BASIC_AUTH_CREDENTIAL_USABLE_MARGIN, BasicAuthProvider,
};
use otel_arrow_dfe_engine::config::ExtensionConfig;
use otel_arrow_dfe_engine::context::ExtensionContext;
use otel_arrow_dfe_engine::extension::wrapper::ExtensionVariant;
use otel_arrow_dfe_engine::extension::{ExtensionBundle, ExtensionWrapper};
use otel_arrow_dfe_engine::extension_capabilities;
use otel_arrow_dfe_otap::OTAP_EXTENSION_FACTORIES;
use tokio::sync::watch;

use crate::common::background_refresh::{
    BackgroundProviderExtension, BackgroundProviderMetricsTracker, BackgroundProviderRefreshPolicy,
};

use self::auth::FlatFileUserPassAuth;
use self::config::Config;
use self::metrics::FlatFileUserPassAuthMetrics;

/// The flat file user pass extension using the shared background refresher.
pub type FlatFileUserPassAuthExtension = BackgroundProviderExtension<
    FlatFileUserPassAuth,
    FlatFileUserPassAuthMetrics,
    BasicAuthCredential,
    BasicAuthProvider,
>;

/// URN under which this extension is registered.
pub const FLAT_FILE_USER_PASS_AUTH_URN: &str = "urn:otel:extension:flat_file_user_pass_auth";

/// Next-refresh delay used for non-expiring credentials (~1 day). The loop is still
/// woken by control messages in the meantime.
const NON_EXPIRING_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL: Duration =
    Duration::from_secs(24 * 60 * 60);

/// Default refresh interval.
const DEFAULT_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Minimum refresh interval.
const MINIMUM_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 5);

/// Refresh this many seconds before `expires_on` (~1 min).
const BASIC_AUTH_CREDENTIAL_EXPIRY_BUFFER_SECS: Duration = Duration::from_secs(60);

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

/// Builds an `FlatFileUserPassAuthExtension` bundle.
fn create(
    ext_ctx: &ExtensionContext,
    name: otel_arrow_dfe_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    // Validate config now so a bad config fails fast at wiring time.
    let config = parse_config(&ext_config.config)?;

    // Register a dedicated entity + metric set for this extension instance.
    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set =
        ext_ctx.register_metric_set_for_entity::<FlatFileUserPassAuthMetrics>(entity_key);
    let tracker = BackgroundProviderMetricsTracker::new(metric_set);

    // Empty token cache; the background refresh loop publishes the first token.
    let (tx, _rx) = watch::channel(None);

    let extension = FlatFileUserPassAuthExtension::new(
        &name,
        FlatFileUserPassAuth::new(config),
        BackgroundProviderRefreshPolicy::new(
            BASIC_AUTH_CREDENTIAL_USABLE_MARGIN,
            NON_EXPIRING_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL,
            BASIC_AUTH_CREDENTIAL_EXPIRY_BUFFER_SECS,
        )
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("failed to initialize flat file user pass extension: {e}"),
        })?,
        tx,
        tracker,
    );

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe()
        .shared::<FlatFileUserPassAuthExtension>(extension)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the flat file user pass extension.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static FLAT_FILE_USER_PASS_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: FLAT_FILE_USER_PASS_AUTH_URN,
    description: "Active+Shared extension exposing BasicAuthProvider via the supplied username and password",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: FlatFileUserPassAuthExtension => [BasicAuthProvider]
    )),
    create,
    validate_config,
};
