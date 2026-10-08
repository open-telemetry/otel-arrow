// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Startup-only flat file SASL authentication extension.

mod auth;
pub mod config;

use std::sync::Arc;

use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::extension::ExtensionUserConfig;
use otel_arrow_dfe_engine::ExtensionFactory;
use otel_arrow_dfe_engine::capability::auth::SaslCredential;
use otel_arrow_dfe_engine::capability::auth::sasl_credential_provider::SaslCredentialProvider;
use otel_arrow_dfe_engine::config::ExtensionConfig;
use otel_arrow_dfe_engine::context::ExtensionContext;
use otel_arrow_dfe_engine::extension::{ExtensionBundle, ExtensionWrapper};
use otel_arrow_dfe_engine::extension_capabilities;
use otel_arrow_dfe_otap::OTAP_EXTENSION_FACTORIES;

use crate::common::secret_file::read_secret_file_sync;

use self::auth::FlatFileSaslAuth;
use self::config::Config;

/// URN under which this extension is registered.
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
    _ext_ctx: &ExtensionContext,
    name: otel_arrow_dfe_config::ExtensionId,
    ext_config: Arc<ExtensionUserConfig>,
    extension_config: &ExtensionConfig,
) -> Result<ExtensionBundle, ConfigError> {
    let config = parse_config(&ext_config.config)?;

    // The factory is synchronous: acquire once before publishing capabilities.
    // Runtime calls only clone this immutable credential and never touch disk.
    let password = read_secret_file_sync(&config.password_secret_file).map_err(|error| {
        ConfigError::InvalidUserConfig {
            error: format!(
                "failed to read `password_secret_file` {:?}: {error}",
                config.password_secret_file
            ),
        }
    })?;
    let credential = SaslCredential::new(config.username, password).map_err(|error| {
        ConfigError::InvalidUserConfig {
            error: format!(
                "invalid credential from `password_secret_file` {:?}: {error}",
                config.password_secret_file
            ),
        }
    })?;

    ExtensionWrapper::builder(name, ext_config, extension_config)
        .passive()
        .cloned()
        .shared(FlatFileSaslAuth { credential })
        .build()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })
}

/// Factory registration for startup-only flat file SASL authentication.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Extension)]
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static FLAT_FILE_SASL_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: FLAT_FILE_SASL_AUTH_URN,
    description: "Passive+Shared SASL credentials with an inline username and startup-only password file",
    documentation_url: "",
    capabilities: Some(extension_capabilities!(
        shared: FlatFileSaslAuth => [SaslCredentialProvider]
    )),
    create,
    validate_config,
};
