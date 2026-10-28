// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure credential construction and token acquisition.

use std::sync::Arc;
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::time::Instant;

use async_trait::async_trait;
use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId, WorkloadIdentityCredential,
    WorkloadIdentityCredentialOptions,
};
use otel_arrow_dfe_engine::capability::auth::BearerToken;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, OwnedMutexGuard, oneshot};

use super::config::Config;
use super::error::Error;
use crate::common::azure_identity::{IdentityOptions, create_credential};
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

type WorkerSlot = Option<SyncSender<ManagedIdentityRequest>>;

/// One acquisition submitted to the managed-identity worker.
struct ManagedIdentityRequest {
    /// Held until the worker finishes this request, so a cancelled caller
    /// cannot admit another acquisition while this one is still running.
    admission: OwnedMutexGuard<WorkerSlot>,
    scope: String,
    response: oneshot::Sender<Result<AccessToken, Error>>,
}

struct ManagedIdentityExecution {
    worker: Arc<Mutex<WorkerSlot>>,
    credential_factory: Arc<CredentialFactory>,
}

impl ManagedIdentityExecution {
    fn new(credential_factory: Arc<CredentialFactory>) -> Self {
        Self {
            worker: Arc::new(Mutex::new(None)),
            credential_factory,
        }
    }

    async fn get_token(&self, scope: String) -> Result<AccessToken, Error> {
        // The owned guard stores the worker handle and provides admission
        // control. It moves into the request and is released by the worker.
        let mut admission = Arc::clone(&self.worker).lock_owned().await;
        let worker = match admission.as_ref() {
            Some(worker) => worker.clone(),
            None => self.start_worker(&mut admission)?,
        };
        let (response, result) = oneshot::channel();
        let request = ManagedIdentityRequest {
            admission,
            scope,
            response,
        };

        match worker.try_send(request) {
            Ok(()) => {}
            // A worker that panicked has dropped its receiver; replace it once.
            Err(TrySendError::Disconnected(mut request)) => {
                let worker = self.start_worker(&mut request.admission)?;
                worker
                    .try_send(request)
                    .map_err(|_| Error::ManagedIdentityWorker {
                        message: "acquisition worker is unavailable".to_owned(),
                    })?;
            }
            Err(TrySendError::Full(_)) => {
                return Err(Error::ManagedIdentityWorker {
                    message: "acquisition worker already has a pending request".to_owned(),
                });
            }
        }

        result.await.map_err(|_| Error::ManagedIdentityWorker {
            message: "acquisition worker stopped before responding".to_owned(),
        })?
    }

    fn start_worker(
        &self,
        slot: &mut WorkerSlot,
    ) -> Result<SyncSender<ManagedIdentityRequest>, Error> {
        let worker = spawn_managed_identity_worker(Arc::clone(&self.credential_factory))?;
        *slot = Some(worker.clone());
        Ok(worker)
    }
}

/// Starts the dedicated managed-identity acquisition thread.
///
/// Azure Arc credential construction and acquisition perform synchronous agent
/// detection and challenge-file reads that cannot be cancelled. A dedicated OS
/// thread with its own runtime keeps that work off the pipeline's
/// thread-per-core executor and outside the pipeline runtime's blocking pool,
/// whose shutdown would otherwise wait for a stuck acquisition. The thread is
/// detached and exits after its current request once the extension drops the
/// sender.
fn spawn_managed_identity_worker(
    credential_factory: Arc<CredentialFactory>,
) -> Result<SyncSender<ManagedIdentityRequest>, Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| Error::ManagedIdentityWorker {
            message: format!("failed to build acquisition runtime: {error}"),
        })?;
    let (sender, requests) = mpsc::sync_channel::<ManagedIdentityRequest>(1);
    let _worker = std::thread::Builder::new()
        .name("azure-mi-auth".to_owned())
        .spawn(move || {
            let mut credential = None;
            while let Ok(request) = requests.recv() {
                let result = acquire_managed_identity_token(
                    &runtime,
                    credential_factory.as_ref(),
                    &mut credential,
                    &request.scope,
                );
                let _ = request.response.send(result);
            }
        })
        .map_err(|error| Error::ManagedIdentityWorker {
            message: format!("failed to start acquisition worker: {error}"),
        })?;
    Ok(sender)
}

fn acquire_managed_identity_token(
    runtime: &Runtime,
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
