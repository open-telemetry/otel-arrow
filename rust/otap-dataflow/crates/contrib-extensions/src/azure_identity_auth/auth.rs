// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure credential construction and token acquisition.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId, WorkloadIdentityCredential,
    WorkloadIdentityCredentialOptions,
};
use otel_arrow_dfe_engine::capability::auth::BearerToken;
use tokio::sync::Semaphore;

use super::config::{AuthMethod, Config};
use super::error::Error;
use crate::common::background_refresh::BackgroundProviderSource;

/// Wraps an Azure credential plus the scope it acquires tokens for.
#[derive(Clone)]
pub struct Auth {
    credential: CredentialExecution,
    scope: String,
}

#[derive(Clone)]
enum CredentialExecution {
    Async(Arc<dyn TokenCredential>),
    ManagedIdentity(Arc<ManagedIdentityExecution>),
}

type CredentialFactory =
    dyn Fn() -> Result<Arc<dyn TokenCredential>, Error> + Send + Sync + 'static;

struct ManagedIdentityExecution {
    credential: Arc<Mutex<Option<Arc<dyn TokenCredential>>>>,
    credential_factory: Arc<CredentialFactory>,
    admission: Arc<Semaphore>,
}

impl ManagedIdentityExecution {
    fn new(credential_factory: Arc<CredentialFactory>) -> Self {
        Self {
            credential: Arc::new(Mutex::new(None)),
            credential_factory,
            admission: Arc::new(Semaphore::new(1)),
        }
    }

    async fn get_token(&self, scope: String) -> Result<AccessToken, Error> {
        let permit = Arc::clone(&self.admission)
            .acquire_owned()
            .await
            .map_err(|error| Error::ManagedIdentityWorker {
                message: format!("acquisition admission closed: {error}"),
            })?;
        let runtime = tokio::runtime::Handle::current();
        let credential_factory = Arc::clone(&self.credential_factory);
        let credential = Arc::clone(&self.credential);

        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let credential = {
                let mut guard = credential
                    .lock()
                    .map_err(|_| Error::ManagedIdentityWorker {
                        message: "credential state lock was poisoned".to_string(),
                    })?;
                if guard.is_none() {
                    *guard = Some(credential_factory()?);
                }
                guard.as_ref().cloned().expect("credential is initialized")
            };

            runtime
                .block_on(
                    credential.get_token(&[scope.as_str()], Some(TokenRequestOptions::default())),
                )
                .map_err(|source| Error::TokenAcquisition { source })
        })
        .await
        .map_err(|error| Error::ManagedIdentityWorker {
            message: format!("acquisition worker failed: {error}"),
        })?
    }
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        // Azure credentials use a `reqwest`/`rustls` HTTP client, which requires
        // a process-wide crypto provider to be installed.
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
        let credential = match config.method {
            AuthMethod::ManagedIdentity => {
                let config = config.clone();
                let factory: Arc<CredentialFactory> = Arc::new(move || create_credential(&config));
                CredentialExecution::ManagedIdentity(Arc::new(ManagedIdentityExecution::new(
                    factory,
                )))
            }
            AuthMethod::Development | AuthMethod::WorkloadIdentity => {
                CredentialExecution::Async(create_credential(config)?)
            }
        };
        Ok(Self {
            credential,
            scope: config.scope.clone(),
        })
    }

    /// Builds an `Auth` from an already-constructed credential. Test-only.
    #[cfg(test)]
    pub(crate) fn from_credential(credential: Arc<dyn TokenCredential>, scope: String) -> Self {
        Self {
            credential: CredentialExecution::Async(credential),
            scope,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_managed_identity_factory<F>(credential_factory: F, scope: String) -> Self
    where
        F: Fn() -> Result<Arc<dyn TokenCredential>, Error> + Send + Sync + 'static,
    {
        Self {
            credential: CredentialExecution::ManagedIdentity(Arc::new(
                ManagedIdentityExecution::new(Arc::new(credential_factory)),
            )),
            scope,
        }
    }
}

#[async_trait]
impl BackgroundProviderSource<BearerToken> for Auth {
    type Error = Error;

    /// Acquires a single token (no retries) and converts it into a
    /// [`BearerToken`].
    async fn fetch(&self) -> Result<BearerToken, Error> {
        let access = match &self.credential {
            CredentialExecution::Async(credential) => credential
                .get_token(&[&self.scope], Some(TokenRequestOptions::default()))
                .await
                .map_err(|source| Error::TokenAcquisition { source })?,
            CredentialExecution::ManagedIdentity(execution) => {
                execution.get_token(self.scope.clone()).await?
            }
        };

        // Let the capability crate centralize the absolute-expiry -> monotonic
        // `Instant` conversion so every provider handles it the same way.
        Ok(BearerToken::from_absolute_expiry(
            access.token.secret().to_owned(),
            access.expires_on.into(),
        ))
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("azure_identity_auth.token_refresh_failed", error = %error);
    }

    fn expires_on(value: &BearerToken) -> Option<Instant> {
        value.expires_on()
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
