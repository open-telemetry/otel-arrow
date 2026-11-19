// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Startup-only Azure Key Vault-backed SASL credential provider.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = AZURE_KEY_VAULT_SASL_AUTH_URN,
    target = "otel.extension.azure_key_vault_sasl_auth",
);

mod auth;
mod metrics;

/// Protocol-neutral Key Vault configuration used by the SASL wrapper.
pub mod config {
    pub use crate::common::azure_identity::{AuthMethod, IdentityConfig};
    pub use crate::common::azure_key_vault::config::{Config, SecretReference};
}

/// Safe acquisition error categories.
pub mod error {
    pub use crate::common::azure_key_vault::error::{Error, FailureKind, Stage};
}

#[cfg(test)]
mod tests;

use std::sync::Arc;

use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::extension::ExtensionUserConfig;
use otel_arrow_dfe_engine::ExtensionFactory;
use otel_arrow_dfe_engine::capability::auth::{
    SaslCredential, sasl_credential_provider::SaslCredentialProvider,
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
use self::metrics::AzureKeyVaultSaslAuthMetrics;
use crate::common::background_refresh::{
    BackgroundProviderExtension, BackgroundProviderMetricsTracker, BackgroundProviderRefreshPolicy,
};

/// Shared cached SASL snapshot, acquired once by the active startup task.
pub type AzureKeyVaultSaslAuthExtension = BackgroundProviderExtension<
    Auth,
    AzureKeyVaultSaslAuthMetrics,
    SaslCredential,
    SaslCredentialProvider,
>;

/// Registered extension identifier.
pub const AZURE_KEY_VAULT_SASL_AUTH_URN: &str = "urn:otel:extension:azure_key_vault_sasl_auth";

fn parse_config(value: &serde_json::Value) -> Result<Config, ConfigError> {
    let config: Config =
        serde_json::from_value(value.clone()).map_err(|_| ConfigError::InvalidUserConfig {
            error: "invalid Azure Key Vault config: missing, unknown, or incorrectly typed option"
                .to_owned(),
        })?;
    config
        .validate()
        .map_err(|error| ConfigError::InvalidUserConfig { error })?;
    Ok(config)
}

fn validate_config(value: &serde_json::Value) -> Result<(), ConfigError> {
    parse_config(value).map(|_| ())
}

fn create(
    ext_ctx: &ExtensionContext,
    name: otel_arrow_dfe_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    let config = parse_config(&ext_config.config)?;
    let startup_timeout = config.startup_timeout;
    let auth = Auth::new(config).map_err(|error| ConfigError::InvalidUserConfig {
        error: error.to_string(),
    })?;
    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metrics =
        ext_ctx.register_metric_set_for_entity::<AzureKeyVaultSaslAuthMetrics>(entity_key);
    let (tx, _rx) = watch::channel(None);
    let extension = AzureKeyVaultSaslAuthExtension::new(
        &name,
        auth,
        BackgroundProviderRefreshPolicy::once(),
        tx,
        BackgroundProviderMetricsTracker::new(metrics),
    );
    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe_timeout_override(startup_timeout)
        .shared::<AzureKeyVaultSaslAuthExtension>(extension)
        .build()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })
}

/// Factory registration for the startup-only SASL provider.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static AZURE_KEY_VAULT_SASL_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: AZURE_KEY_VAULT_SASL_AUTH_URN,
    description: "Active+Shared startup SASL credential provider backed by Azure Key Vault",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: AzureKeyVaultSaslAuthExtension => [SaslCredentialProvider]
    )),
    create,
    validate_config,
};
