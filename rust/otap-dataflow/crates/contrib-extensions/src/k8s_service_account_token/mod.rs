// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kubernetes Service Account Token extension.
//!
//! Reads the pod's projected service account token from its mounted file and
//! exposes it to data-path nodes through the `BearerTokenProvider` capability,
//! re-reading and re-publishing it as kubelet rotates the token.

pub mod config;
pub mod error;
mod extension;
mod metrics;
mod token_source;

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
use otap_df_engine::extension::wrapper::ExtensionVariant;
use otap_df_engine::extension::{ExtensionBundle, ExtensionWrapper};
use otap_df_engine::extension_capabilities;
use otap_df_otap::OTAP_EXTENSION_FACTORIES;
use tokio::sync::watch;

use self::config::Config;
use self::extension::K8sServiceAccountTokenExtension;
use self::metrics::{K8sServiceAccountTokenMetrics, K8sServiceAccountTokenMetricsTracker};
use self::token_source::{FileTokenSource, TokenSource};

/// URN under which this extension is registered.
pub const K8S_SERVICE_ACCOUNT_TOKEN_URN: &str = "urn:otel:extension:k8s_service_account_token";

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

/// Builds a `K8sServiceAccountTokenExtension` bundle.
fn create(
    ext_ctx: &ExtensionContext,
    name: otap_df_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    // Validate config now so a bad config fails fast at wiring time.
    let config = parse_config(&ext_config.config)?;

    let source: Arc<dyn TokenSource> = Arc::new(FileTokenSource::new(&config));

    // Register a dedicated entity + metric set for this extension instance.
    let entity_key = ext_ctx.register_extension_entity(name.clone(), ExtensionVariant::Shared);
    let metric_set =
        ext_ctx.register_metric_set_for_entity::<K8sServiceAccountTokenMetrics>(entity_key);
    let tracker = K8sServiceAccountTokenMetricsTracker::new(metric_set);

    // Empty token cache; the background refresh loop publishes the first token.
    let (tx, _rx) = watch::channel(None);

    let extension = K8sServiceAccountTokenExtension::new(&name, source, tx, tracker);

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .active()
        .with_readiness_probe_timeout_override(config.startup_timeout)
        .shared::<K8sServiceAccountTokenExtension>(extension)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the Kubernetes Service Account Token extension.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static K8S_SERVICE_ACCOUNT_TOKEN_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: K8S_SERVICE_ACCOUNT_TOKEN_URN,
    description: "Active+Shared extension exposing BearerTokenProvider from the Kubernetes projected service account token",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: K8sServiceAccountTokenExtension => [BearerTokenProvider]
    )),
    create,
    validate_config,
};
