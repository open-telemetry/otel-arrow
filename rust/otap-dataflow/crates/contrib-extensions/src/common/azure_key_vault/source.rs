// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! SDK-backed, protocol-neutral startup secret acquisition.

use azure_core::time::OffsetDateTime;
use azure_security_keyvault_secrets::{SecretClient, models::SecretClientGetSecretOptions};
use secrecy::{ExposeSecret, SecretString};

use super::config::{Config, SecretReference};
use super::error::{Error, FailureKind, Stage};
use crate::common::azure_identity::create_credential;

/// A fully validated pair, without protocol-specific credential restrictions.
#[derive(Debug)]
pub struct SecretPair {
    /// Exact username value; never normalized.
    pub username: SecretString,
    /// Exact password value; never normalized.
    pub password: SecretString,
}

/// Shared Azure Key Vault client and acquisition configuration.
///
/// The SDK owns its token cache and request retry policy. No secret cache,
/// custom retry loop, lock, or task is added here.
pub struct Source {
    client: SecretClient,
    config: Config,
}

impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyVaultSource").finish_non_exhaustive()
    }
}

impl Source {
    /// Constructs SDK identity and client without retrieving any secret.
    pub fn new(config: Config) -> Result<Self, Error> {
        config.validate().map_err(|_| Error {
            stage: Stage::Client,
            kind: FailureKind::Configuration,
        })?;
        let credential = create_credential((&config.identity).into())
            .map_err(|error| Error::from_sdk(Stage::Identity, error))?;
        // Default SDK transport disables redirects. Challenge-resource
        // verification stays enabled; no arbitrary scope override is allowed.
        let client = SecretClient::new(&config.vault_url, credential, None).map_err(|_| Error {
            stage: Stage::Client,
            kind: FailureKind::Configuration,
        })?;
        Ok(Self { client, config })
    }

    /// Injects an SDK client with an offline transport for tests.
    #[cfg(test)]
    pub(crate) fn with_client(config: Config, client: SecretClient) -> Self {
        Self { client, config }
    }

    /// Reads and validates both secrets within one bounded acquisition.
    pub async fn fetch_pair(&self) -> Result<SecretPair, Error> {
        tokio::time::timeout(self.config.startup_timeout, async {
            let username = self
                .read(&self.config.username_secret, Stage::Username)
                .await?;
            let password = self
                .read(&self.config.password_secret, Stage::Password)
                .await?;
            Ok(SecretPair { username, password })
        })
        .await
        .map_err(|_| Error {
            stage: Stage::Acquisition,
            kind: FailureKind::Timeout,
        })?
    }

    async fn read(&self, reference: &SecretReference, stage: Stage) -> Result<SecretString, Error> {
        let mut options = SecretClientGetSecretOptions::default();
        options.secret_version.clone_from(&reference.version);
        let secret = self
            .client
            .get_secret(&reference.name, Some(options))
            .await
            .map_err(|error| Error::from_sdk(stage, error))?
            .into_model()
            .map_err(|error| Error::from_sdk(stage, error))?;
        // Do not format the SDK model: its Debug output contains plaintext.
        // Move the value into a zeroizing allocation before checking metadata.
        let value = secret.value.map(SecretString::from).ok_or(Error {
            stage,
            kind: FailureKind::InvalidValue,
        })?;
        let now = OffsetDateTime::now_utc();
        if value.expose_secret().is_empty()
            || secret.attributes.as_ref().is_some_and(|attributes| {
                attributes.enabled == Some(false)
                    || attributes.expires.is_some_and(|expiry| expiry <= now)
                    || attributes
                        .not_before
                        .is_some_and(|not_before| not_before > now)
            })
        {
            return Err(Error {
                stage,
                kind: FailureKind::InvalidValue,
            });
        }
        Ok(value)
    }
}
