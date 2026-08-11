// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure credential construction and token acquisition.

use std::sync::Arc;

use azure_core::credentials::{TokenCredential, TokenRequestOptions};
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId, WorkloadIdentityCredential,
    WorkloadIdentityCredentialOptions,
};
use otap_df_engine::capability::bearer_token_provider::BearerToken;

use super::config::{AuthMethod, Config};
use super::error::Error;

/// Wraps an Azure credential plus the scope it acquires tokens for.
#[derive(Clone)]
pub struct Auth {
    credential: Arc<dyn TokenCredential>,
    scope: String,
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        // Azure credentials use a `reqwest`/`rustls` HTTP client, which requires
        // a process-wide crypto provider to be installed.
        otap_df_otap::crypto::ensure_crypto_provider();
        let credential = create_credential(config)?;
        Ok(Self {
            credential,
            scope: config.scope.clone(),
        })
    }

    /// Acquires a single token (no retries) and converts it into a
    /// [`BearerToken`].
    pub async fn get_token(&self) -> Result<BearerToken, Error> {
        let access = self
            .credential
            .get_token(&[&self.scope], Some(TokenRequestOptions::default()))
            .await
            .map_err(|source| Error::TokenAcquisition { source })?;

        // Let the capability crate centralize the absolute-expiry -> monotonic
        // `Instant` conversion so every provider handles it the same way.
        Ok(BearerToken::from_absolute_expiry(
            access.token.secret().to_owned(),
            access.expires_on.into(),
        ))
    }

    /// Builds an `Auth` from an already-constructed credential. Test-only.
    #[cfg(test)]
    pub(crate) fn from_credential(credential: Arc<dyn TokenCredential>, scope: String) -> Self {
        Self { credential, scope }
    }
}

/// Constructs an Azure credential for the configured authentication method.
fn create_credential(config: &Config) -> Result<Arc<dyn TokenCredential>, Error> {
    match config.method {
        AuthMethod::ManagedIdentity => {
            let mut options = ManagedIdentityCredentialOptions::default();
            if let Some(client_id) = &config.client_id {
                options.user_assigned_id = Some(UserAssignedId::ClientId(client_id.clone()));
            }
            let cred = ManagedIdentityCredential::new(Some(options)).map_err(|source| {
                Error::CreateCredential {
                    method: AuthMethod::ManagedIdentity,
                    source,
                }
            })?;
            Ok(cred)
        }
        AuthMethod::Development => {
            let cred =
                DeveloperToolsCredential::new(Some(DeveloperToolsCredentialOptions::default()))
                    .map_err(|source| Error::CreateCredential {
                        method: AuthMethod::Development,
                        source,
                    })?;
            Ok(cred)
        }
        AuthMethod::WorkloadIdentity => {
            let options = WorkloadIdentityCredentialOptions {
                client_id: config.client_id.clone(),
                tenant_id: config.tenant_id.clone(),
                token_file_path: config.token_file_path.clone(),
                ..Default::default()
            };
            let cred = WorkloadIdentityCredential::new(Some(options)).map_err(|source| {
                Error::CreateCredential {
                    method: AuthMethod::WorkloadIdentity,
                    source,
                }
            })?;
            Ok(cred)
        }
    }
}
