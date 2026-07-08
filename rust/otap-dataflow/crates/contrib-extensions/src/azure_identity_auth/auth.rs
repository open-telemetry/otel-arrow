// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure credential construction and token acquisition.

use std::sync::Arc;
use std::time::{Duration, Instant};

use azure_core::credentials::{TokenCredential, TokenRequestOptions};
use azure_core::time::OffsetDateTime;
use azure_identity::{ManagedIdentityCredential, ManagedIdentityCredentialOptions, UserAssignedId};
use otap_df_engine::capability::BearerToken;

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

        let expires_on = instant_from_unix_expiry(access.expires_on);
        Ok(BearerToken::new(access.token.secret().to_owned(), expires_on))
    }
}

/// Converts an absolute UNIX expiry timestamp into a monotonic [`Instant`]
/// anchored at "now". After this single conversion the schedule is immune to
/// wall-clock jumps. Saturates at zero for already-expired timestamps.
fn instant_from_unix_expiry(expires_on: OffsetDateTime) -> Option<Instant> {
    let now_unix = OffsetDateTime::now_utc().unix_timestamp();
    let secs_until_expiry = expires_on.unix_timestamp().saturating_sub(now_unix).max(0);
    Some(Instant::now() + Duration::from_secs(secs_until_expiry as u64))
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
        // Implemented in later changes.
        method @ (AuthMethod::Development | AuthMethod::WorkloadIdentity) => {
            Err(Error::CreateCredential {
                method,
                source: azure_core::error::Error::with_message(
                    azure_core::error::ErrorKind::Other,
                    "auth method not yet supported",
                ),
            })
        }
    }
}
