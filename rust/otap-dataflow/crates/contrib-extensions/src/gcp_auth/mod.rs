// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Google Cloud (GCP) Auth extension.
//!
//! Acquires and refreshes Google Cloud OAuth 2.0 access tokens from the GCE/GKE
//! compute metadata server and exposes them to data-path nodes through the
//! `BearerTokenProvider` capability.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = GCP_AUTH_URN,
    target = "otel.extension.gcp_auth",
);

mod auth;
pub mod config;
pub mod error;
mod metrics;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::extension::ExtensionUserConfig;
use otel_arrow_dfe_engine::ExtensionFactory;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::{
    BearerTokenProvider, TOKEN_USABLE_MARGIN,
};
use otel_arrow_dfe_engine::config::ExtensionConfig;
use otel_arrow_dfe_engine::context::ExtensionContext;
use otel_arrow_dfe_engine::extension::wrapper::ExtensionVariant;
use otel_arrow_dfe_engine::extension::{ExtensionBundle, ExtensionWrapper};
use otel_arrow_dfe_engine::extension_capabilities;
use otel_arrow_dfe_otap::OTAP_EXTENSION_FACTORIES;
use tokio::sync::watch;

use self::auth::Auth;
use self::config::Config;
use self::metrics::GcpAuthMetrics;
use crate::common::background_refresh::{
    BackgroundProviderMetricsTracker, BackgroundProviderRefreshPolicy,
};
use crate::common::token_refresh::{NON_EXPIRING_TOKEN_REFRESH_INTERVAL, TokenProviderExtension};

/// The GCP Auth extension: the shared bearer-token refresher driven by the
/// Google Cloud compute metadata server.
pub type GcpAuthExtension = TokenProviderExtension<Auth, GcpAuthMetrics>;

/// URN under which this extension is registered.
pub const GCP_AUTH_URN: &str = "urn:otel:extension:gcp_auth";

/// Deserializes and validates the extension's user configuration.
fn parse_config(config: &serde_json::Value) -> Result<Config, ConfigError> {
    let parsed: Config = if config.is_null() {
        Config::default()
    } else {
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })?
    };
    parsed
        .validate()
        .map_err(|error| ConfigError::InvalidUserConfig { error })?;
    Ok(parsed)
}

/// Static config validation hook for the factory.
fn validate_config(config: &serde_json::Value) -> Result<(), ConfigError> {
    parse_config(config).map(|_| ())
}

/// Builds a `GcpAuthExtension` bundle.
fn create(
    ext_ctx: &ExtensionContext,
    name: otel_arrow_dfe_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    // Validate config now so a bad config fails fast at wiring time.
    let config = parse_config(&ext_config.config)?;

    let auth = Auth::new(&config).map_err(|e| ConfigError::InvalidUserConfig {
        error: format!("failed to initialize GCP auth client: {e}"),
    })?;

    // Register a dedicated entity + metric set for this extension instance.
    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set = ext_ctx.register_metric_set_for_entity::<GcpAuthMetrics>(entity_key);
    let tracker = BackgroundProviderMetricsTracker::new(metric_set);

    // Empty token cache; the background refresh loop publishes the first token.
    let (tx, _rx) = watch::channel(None);

    let extension = GcpAuthExtension::new(
        &name,
        auth,
        BackgroundProviderRefreshPolicy::expiry_driven(
            TOKEN_USABLE_MARGIN,
            NON_EXPIRING_TOKEN_REFRESH_INTERVAL,
            config.expiry_buffer,
        )
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("failed to initialize GCP auth client: {e}"),
        })?,
        tx,
        tracker,
    );

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe_timeout_override(config.startup_timeout)
        .shared::<GcpAuthExtension>(extension)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the GCP Auth extension.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static GCP_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: GCP_AUTH_URN,
    description: "Active+Shared extension exposing BearerTokenProvider via the Google Cloud compute metadata server",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: GcpAuthExtension => [BearerTokenProvider]
    )),
    create,
    validate_config,
};
