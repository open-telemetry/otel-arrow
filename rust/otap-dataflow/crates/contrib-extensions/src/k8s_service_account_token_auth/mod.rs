// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kubernetes service-account-token auth extension.
//!
//! Authenticates and admits inbound Kubernetes service-account tokens for
//! data-path nodes through the `BearerTokenAuthorizer` capability. Each token is
//! validated via the Kubernetes `TokenReview` API (authentication) and the
//! resulting identity is admitted via an allow-list or an RBAC
//! `SubjectAccessReview` check. See
//! `docs/k8s-service-account-token-auth-extension.md`
//! for the design.
//!
//! The extension provides two capability variants over common `core`,
//! [`config`], and `reviewer` logic: a `Send` `Arc`/`Mutex` variant and a
//! `!Send` `Rc`/`RefCell` variant for thread-per-core consumers. Both live in
//! `authorizer`, each with its own `cache` hot path.

pub mod config;
pub mod error;

mod authorizer;
mod cache;
mod core;
mod reviewer;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::extension::ExtensionUserConfig;
use otap_df_engine::ExtensionFactory;
use otap_df_engine::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer;
use otap_df_engine::config::ExtensionConfig;
use otap_df_engine::context::ExtensionContext;
use otap_df_engine::extension::{ExtensionBundle, ExtensionWrapper};
use otap_df_engine::extension_capabilities;
use otap_df_otap::OTAP_EXTENSION_FACTORIES;

use self::authorizer::{LocalK8sServiceAccountTokenAuth, SharedK8sServiceAccountTokenAuth};
use self::config::Config;

/// URN under which this extension is registered.
pub const K8S_SERVICE_ACCOUNT_TOKEN_AUTH_URN: &str =
    "urn:otel:extension:k8s_service_account_token_auth";

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

/// Builds the dual-variant authorizer bundle.
fn create(
    _ext_ctx: &ExtensionContext,
    name: otap_df_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    // Validate config now so a bad config fails fast at wiring time.
    let config = parse_config(&ext_config.config)?;

    // Build both capability variants from the same config. The shared variant is
    // `Arc`-backed (Send) and the local variant is `Rc`-backed (lock-free,
    // thread-per-core); each has its own cache and lazily-built client.
    let shared = SharedK8sServiceAccountTokenAuth::new_with_timeout(
        &name,
        config.audiences.clone(),
        config.cache_ttl,
        config.cache_max_entries,
        config.review_timeout,
    );
    let local = LocalK8sServiceAccountTokenAuth::new_with_timeout(
        &name,
        config.audiences,
        config.cache_ttl,
        config.cache_max_entries,
        config.review_timeout,
    );

    // The extension is passive: it exposes the authorizer capability and builds
    // its Kubernetes client lazily on first use.
    ExtensionWrapper::builder(name, ext_config, extension_config)
        .passive()
        .cloned()
        .shared::<SharedK8sServiceAccountTokenAuth>(shared)
        .local::<LocalK8sServiceAccountTokenAuth>(local)
        .build()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
}

/// Factory registration for the Kubernetes service-account-token auth extension.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static K8S_SERVICE_ACCOUNT_TOKEN_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: K8S_SERVICE_ACCOUNT_TOKEN_AUTH_URN,
    description: "Passive extension exposing BearerTokenAuthorizer (shared + local variants) via Kubernetes TokenReview",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        (shared: SharedK8sServiceAccountTokenAuth, local: LocalK8sServiceAccountTokenAuth) => [BearerTokenAuthorizer]
    )),
    create,
    validate_config,
};
