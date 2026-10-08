// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! File-backed username/password credentials exposed through SASL.

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

type FlatFileSaslAuthExtension = BackgroundProviderExtension<
    FlatFileSaslAuth,
    FlatFileSaslAuthMetrics,
    SaslCredential,
    SaslCredentialProvider,
>;

/// URN for the flat-file SASL credential provider.
pub const FLAT_FILE_SASL_AUTH_URN: &str = "urn:otel:extension:flat_file_sasl_auth";

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
    let refresh_policy = BackgroundProviderRefreshPolicy::periodic(config.credentials_file_refresh)
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: format!("invalid `credentials_file_refresh`: {error}"),
        })?;
    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set = ext_ctx.register_metric_set_for_entity::<FlatFileSaslAuthMetrics>(entity_key);
    let (tx, _rx) = watch::channel(None);
    let extension = FlatFileSaslAuthExtension::new(
        &name,
        FlatFileSaslAuth::new(config),
        refresh_policy,
        tx,
        BackgroundProviderMetricsTracker::new(metric_set),
    );

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe()
        .shared::<FlatFileSaslAuthExtension>(extension)
        .build()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })
}

/// Factory registration for the flat-file SASL credential provider.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static FLAT_FILE_SASL_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: FLAT_FILE_SASL_AUTH_URN,
    description: "Active+Shared extension exposing SaslCredentialProvider via username and password files",
    documentation_url: "https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/contrib-extensions/src/flat_file_sasl_auth/README.md",
    capabilities: Some(extension_capabilities!(
        shared: FlatFileSaslAuthExtension => [SaslCredentialProvider]
    )),
    create,
    validate_config,
};
