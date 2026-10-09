// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file SASL authentication extension.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = FLAT_FILE_SASL_AUTH_URN,
    target = "otel.extension.flat_file_sasl_auth",
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
use otel_arrow_dfe_engine::capability::auth::SaslCredential;
use otel_arrow_dfe_engine::capability::auth::sasl_credential_provider::SaslCredentialProvider;
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

use self::auth::FlatFileSaslAuth;
use self::config::Config;
use self::metrics::FlatFileSaslAuthMetrics;

/// SASL credentials using the existing shared provider cache, watch subscriptions,
/// coalescing, retry policy, and readiness lifecycle; no protocol-specific synchronization.
pub type FlatFileSaslAuthExtension = BackgroundProviderExtension<
    FlatFileSaslAuth,
    FlatFileSaslAuthMetrics,
    SaslCredential,
    SaslCredentialProvider,
>;

/// URN under which this extension is registered.
pub const FLAT_FILE_SASL_AUTH_URN: &str = "urn:otel:extension:flat_file_sasl_auth";

const DEFAULT_SASL_CREDENTIAL_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60);

fn parse_config(config: &serde_json::Value) -> Result<Config, ConfigError> {
    let parsed: Config =
        serde_json::from_value(config.clone()).map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })?;
    parsed
        .validate()
        .map_err(|error| ConfigError::InvalidUserConfig { error })?;
    Ok(parsed)
}

fn validate_config(config: &serde_json::Value) -> Result<(), ConfigError> {
    parse_config(config).map(|_| ())
}

fn create(
    ext_ctx: &ExtensionContext,
    name: otel_arrow_dfe_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    let config = parse_config(&ext_config.config)?;
    let startup_timeout = config.startup_timeout;

    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set = ext_ctx.register_metric_set_for_entity::<FlatFileSaslAuthMetrics>(entity_key);
    let tracker = BackgroundProviderMetricsTracker::new(metric_set);
    let refresh_policy = BackgroundProviderRefreshPolicy::periodic(
        config.password_secret_file_refresh,
    )
    .map_err(|error| ConfigError::InvalidUserConfig {
        error: format!("invalid `password_secret_file_refresh`: {error}"),
    })?;
    let (tx, _rx) = watch::channel(None);
    let extension = FlatFileSaslAuthExtension::new(
        &name,
        FlatFileSaslAuth::new(config),
        refresh_policy,
        tx,
        tracker,
    );

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe_timeout_override(startup_timeout)
        .shared::<FlatFileSaslAuthExtension>(extension)
        .build()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })
}

/// Factory registration for flat file SASL authentication.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static FLAT_FILE_SASL_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: FLAT_FILE_SASL_AUTH_URN,
    description: "Active+Shared SaslCredentialProvider with an inline username and refreshed password file",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: FlatFileSaslAuthExtension => [SaslCredentialProvider]
    )),
    create,
    validate_config,
};
