// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure credential construction and token acquisition.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use otel_arrow_dfe_engine::capability::auth::BearerToken;
use tokio::runtime::Handle;
use tokio::sync::Mutex;

use super::config::Config;
use super::error::Error;
use crate::common::azure_identity::{AuthMethod, IdentityOptions, create_credential};
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
    // Shared with the blocking task so cancellation cannot release admission
    // while synchronous Azure Arc SDK work is still running.
    credential: Arc<Mutex<Option<Arc<dyn TokenCredential>>>>,
    credential_factory: Arc<CredentialFactory>,
}

impl ManagedIdentityExecution {
    fn new(credential_factory: Arc<CredentialFactory>) -> Self {
        Self {
            credential: Arc::new(Mutex::new(None)),
            credential_factory,
        }
    }

    async fn get_token(&self, scope: String) -> Result<AccessToken, Error> {
        let mut credential = Arc::clone(&self.credential).lock_owned().await;
        let credential_factory = Arc::clone(&self.credential_factory);
        let runtime = Handle::current();
        // Acquire admission before submitting work, then retain it inside the
        // blocking task even if the caller stops awaiting the result.
        tokio::task::spawn_blocking(move || {
            acquire_managed_identity_token(
                &runtime,
                credential_factory.as_ref(),
                &mut credential,
                &scope,
            )
        })
        .await
        .map_err(|source| Error::ManagedIdentityTask { source })?
    }
}

fn acquire_managed_identity_token(
    runtime: &Handle,
    credential_factory: &CredentialFactory,
    cached: &mut Option<Arc<dyn TokenCredential>>,
    scope: &str,
) -> Result<AccessToken, Error> {
    let credential = match cached {
        Some(credential) => Arc::clone(credential),
        None => {
            let credential = {
                let _context = runtime.enter();
                credential_factory()?
            };
            *cached = Some(Arc::clone(&credential));
            credential
        }
    };

    runtime
        .block_on(credential.get_token(&[scope], Some(TokenRequestOptions::default())))
        .map_err(|source| Error::TokenAcquisition { source })
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        let credential = match config.method {
            AuthMethod::ManagedIdentity => {
                let config = config.clone();
                let factory: Arc<CredentialFactory> = Arc::new(move || {
                    create_credential(IdentityOptions::from(&config)).map_err(|source| {
                        Error::CreateCredential {
                            method: AuthMethod::ManagedIdentity,
                            source,
                        }
                    })
                });
                CredentialExecution::ManagedIdentity(Arc::new(ManagedIdentityExecution::new(
                    factory,
                )))
            }
            AuthMethod::Development | AuthMethod::WorkloadIdentity => {
                let credential =
                    create_credential(IdentityOptions::from(config)).map_err(|source| {
                        Error::CreateCredential {
                            method: config.method,
                            source,
                        }
                    })?;
                CredentialExecution::Async(credential)
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

    /// Reports whether a managed-identity acquisition could be admitted now.
    #[cfg(test)]
    pub(crate) fn managed_identity_admission_available(&self) -> bool {
        match &self.credential {
            CredentialExecution::ManagedIdentity(execution) => {
                execution.credential.try_lock().is_ok()
            }
            CredentialExecution::Async(_) => true,
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
