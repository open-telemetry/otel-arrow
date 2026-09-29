// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file API key authentication extension.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = FLAT_FILE_API_KEY_AUTH_URN,
    target = "otel.extension.flat_file_api_key_auth",
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
use otel_arrow_dfe_engine::capability::auth::ApiKey;
use otel_arrow_dfe_engine::capability::auth::api_key_provider::ApiKeyProvider;
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

use self::auth::FlatFileApiKeyAuth;
use self::config::Config;
use self::metrics::FlatFileApiKeyAuthMetrics;

/// The flat file API key extension using the shared background refresher.
pub type FlatFileApiKeyAuthExtension = BackgroundProviderExtension<
    FlatFileApiKeyAuth,
    FlatFileApiKeyAuthMetrics,
    ApiKey,
    ApiKeyProvider,
>;

/// URN under which this extension is registered.
pub const FLAT_FILE_API_KEY_AUTH_URN: &str = "urn:otel:extension:flat_file_api_key_auth";

/// Default refresh interval.
const DEFAULT_API_KEY_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60);

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

/// Builds a `FlatFileApiKeyAuthExtension` bundle.
fn create(
    ext_ctx: &ExtensionContext,
    name: otel_arrow_dfe_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    let config = parse_config(&ext_config.config)?;

    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set =
        ext_ctx.register_metric_set_for_entity::<FlatFileApiKeyAuthMetrics>(entity_key);
    let tracker = BackgroundProviderMetricsTracker::new(metric_set);

    let (tx, _rx) = watch::channel(None);
    let refresh_policy = if config.key_secret_file.is_some() {
        BackgroundProviderRefreshPolicy::periodic(config.key_secret_file_refresh).map_err(|e| {
            ConfigError::InvalidUserConfig {
                error: format!("failed to initialize flat file API key auth extension: {e}"),
            }
        })?
    } else {
        BackgroundProviderRefreshPolicy::once()
    };

    let extension = FlatFileApiKeyAuthExtension::new(
        &name,
        FlatFileApiKeyAuth::new(config).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("failed to initialize flat file api key auth: {e}"),
        })?,
        refresh_policy,
        tx,
        tracker,
    );

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe()
        .shared::<FlatFileApiKeyAuthExtension>(extension)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the flat file API key authentication extension.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static FLAT_FILE_API_KEY_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: FLAT_FILE_API_KEY_AUTH_URN,
    description: "Active+Shared extension exposing ApiKeyProvider via a supplied API key",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: FlatFileApiKeyAuthExtension => [ApiKeyProvider]
    )),
    create,
    validate_config,
};
