// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Google Cloud Application Default Credentials (ADC) and compute metadata
//! server credential construction and token acquisition via
//! [`google_cloud_auth`].

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use google_cloud_auth::credentials::{Builder as AdcBuilder, CacheableResource, Credentials, mds};
use http::Extensions;
use otel_arrow_dfe_engine::capability::auth::BearerToken;
use tokio::runtime::Handle;
use tokio::sync::{oneshot, watch};

use super::config::Config;
use super::error::Error;
use crate::common::background_refresh::{BackgroundFetchOutcome, BackgroundProviderSource};

type CredentialsFactory = dyn Fn() -> Result<Credentials, Error> + Send + Sync + 'static;

/// Holds the active `google-cloud-auth` [`Credentials`] instance and the most
/// recent [`EntityTag`](google_cloud_auth::credentials::EntityTag) + token pair
/// observed from its internal `TokenCache`.
struct CachedCredentials {
    credentials: Credentials,
    last_token: Option<(Extensions, BearerToken)>,
}

/// Lifecycle state of the `google-cloud-auth` credential cache.
enum CredentialState {
    Uninitialized,
    Active(CachedCredentials),
    ShutDown,
}

/// Coordinates bounded, single-slot blocking credential construction, active
/// fetch cancellation, and `Credentials` handle ownership across shutdown.
struct CredentialExecution {
    // Async mutex bounds concurrent credential construction and token fetches
    // to one active operation and holds the sole `Credentials` handle. Waiting
    // on this lock during shutdown guarantees any in-flight blocking build or
    // `headers()` wait has exited and dropped `Credentials` before returning.
    state: Arc<tokio::sync::Mutex<CredentialState>>,
    credentials_factory: Arc<CredentialsFactory>,
    shutdown: watch::Sender<bool>,
}

impl CredentialExecution {
    fn new(credentials_factory: Arc<CredentialsFactory>) -> Self {
        Self {
            state: Arc::new(tokio::sync::Mutex::new(CredentialState::Uninitialized)),
            credentials_factory,
            shutdown: watch::channel(false).0,
        }
    }
}

/// A Google Cloud token source backed by [`google_cloud_auth`].
#[derive(Clone)]
pub struct Auth {
    timeout: Duration,
    execution: Arc<CredentialExecution>,
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        config
            .validate()
            .map_err(|reason| Error::BuildCredentials { reason })?;
        let endpoint = config
            .normalized_endpoint()
            .map_err(|reason| Error::BuildCredentials { reason })?;
        let scopes: Vec<String> = config.scopes.iter().map(|s| s.trim().to_string()).collect();

        // NOTE: `google-cloud-auth` credential builders perform synchronous ADC
        // file I/O and spawn an internal `TokenCache` task via `tokio::spawn`
        // when `.build()` is called. We defer building `Credentials` until the
        // first async `fetch_outcome()` call (offloaded onto `spawn_blocking`)
        // so `Auth::new()` remains callable outside a Tokio runtime and never
        // blocks the pipeline's local runtime thread.
        let credentials_factory: Arc<CredentialsFactory> =
            Arc::new(move || build_credentials(endpoint.as_deref(), &scopes));
        Ok(Self {
            timeout: config.timeout,
            execution: Arc::new(CredentialExecution::new(credentials_factory)),
        })
    }

    /// Builds an `Auth` with a custom credential factory. Test-only.
    #[cfg(test)]
    pub(crate) fn from_credentials_factory<F>(credentials_factory: F, timeout: Duration) -> Self
    where
        F: Fn() -> Result<Credentials, Error> + Send + Sync + 'static,
    {
        Self {
            timeout,
            execution: Arc::new(CredentialExecution::new(Arc::new(credentials_factory))),
        }
    }

    /// Reports whether credential construction / fetch admission is currently available.
    #[cfg(test)]
    pub(crate) fn admission_available(&self) -> bool {
        self.execution.state.try_lock().is_ok()
    }
}

fn build_credentials(endpoint: Option<&str>, scopes: &[String]) -> Result<Credentials, Error> {
    // When `endpoint` is omitted, use `google_cloud_auth::credentials::Builder`
    // so full Application Default Credentials (ADC) discovery runs
    // (`GOOGLE_APPLICATION_CREDENTIALS` -> well-known `gcloud` ADC file ->
    // GCE/GKE compute metadata server fallback). When `endpoint` is explicitly
    // configured, target that metadata server endpoint directly via
    // `mds::Builder`.
    let credentials = match endpoint {
        Some(endpoint) => {
            let mut builder = mds::Builder::default().with_endpoint(endpoint.to_string());
            if !scopes.is_empty() {
                builder = builder.with_scopes(scopes.iter().cloned());
            }
            builder.build()
        }
        None => {
            let mut builder = AdcBuilder::default();
            if !scopes.is_empty() {
                builder = builder.with_scopes(scopes.iter().cloned());
            }
            builder.build()
        }
    };
    credentials.map_err(|e| Error::BuildCredentials {
        reason: format_error_chain(&e),
    })
}

fn timeout_error(timeout: Duration) -> Error {
    Error::TokenAcquisition {
        message: format!("timed out after {timeout:?} waiting for GCP metadata token"),
    }
}

fn shutdown_error() -> Error {
    Error::TokenAcquisition {
        message: "GCP auth extension is shut down".to_string(),
    }
}

#[async_trait]
impl BackgroundProviderSource<BearerToken> for Auth {
    type Error = Error;

    async fn fetch(&self) -> Result<BearerToken, Error> {
        self.fetch_outcome()
            .await
            .map(BackgroundFetchOutcome::into_value)
    }

    async fn fetch_outcome(&self) -> Result<BackgroundFetchOutcome<BearerToken>, Error> {
        let mut shutdown = self.execution.shutdown.subscribe();
        if *shutdown.borrow() {
            return Err(shutdown_error());
        }

        // Bound both synchronous ADC discovery / credential construction and
        // the subsequent `headers()` wait with the configured per-attempt timeout.
        let deadline = tokio::time::Instant::now() + self.timeout;
        let mut state = tokio::select! {
            biased;
            _ = shutdown.wait_for(|is_shut_down| *is_shut_down) => return Err(shutdown_error()),
            res = tokio::time::timeout_at(deadline, Arc::clone(&self.execution.state).lock_owned()) => {
                res.map_err(|_| timeout_error(self.timeout))?
            }
        };

        if *shutdown.borrow() || matches!(*state, CredentialState::ShutDown) {
            *state = CredentialState::ShutDown;
            return Err(shutdown_error());
        }

        if matches!(*state, CredentialState::Uninitialized) {
            // Move the admission guard into `spawn_blocking` so at most one
            // blocking ADC discovery / file read runs at a time and shutdown
            // waits for it to exit. Dropping `_cancel_on_drop` (or setting
            // `shutdown`) tells the worker to discard any late-built
            // `Credentials` instead of caching it.
            let runtime = Handle::current();
            let factory = Arc::clone(&self.execution.credentials_factory);
            let worker_shutdown = self.execution.shutdown.subscribe();
            let (_cancel_on_drop, mut cancelled) = oneshot::channel::<()>();
            let mut build_handle = tokio::task::spawn_blocking(move || {
                let _enter = runtime.enter();
                let mut state = state;
                let is_aborted =
                    |state: &CredentialState, cancelled: &mut oneshot::Receiver<()>| {
                        *worker_shutdown.borrow()
                            || matches!(*state, CredentialState::ShutDown)
                            || !matches!(
                                cancelled.try_recv(),
                                Err(oneshot::error::TryRecvError::Empty)
                            )
                    };
                if is_aborted(&state, &mut cancelled) {
                    if *worker_shutdown.borrow() {
                        *state = CredentialState::ShutDown;
                    }
                    return Err(Error::TokenAcquisition {
                        message: "GCP credential construction cancelled".to_string(),
                    });
                }
                let credentials = (factory)()?;
                if is_aborted(&state, &mut cancelled) {
                    // Dropping `credentials` immediately aborts its internal
                    // `TokenCache` background task.
                    drop(credentials);
                    if *worker_shutdown.borrow() {
                        *state = CredentialState::ShutDown;
                    }
                    return Err(Error::TokenAcquisition {
                        message: "GCP credential construction cancelled".to_string(),
                    });
                }
                *state = CredentialState::Active(CachedCredentials {
                    credentials,
                    last_token: None,
                });
                Ok(state)
            });

            state = tokio::select! {
                biased;
                _ = shutdown.wait_for(|is_shut_down| *is_shut_down) => return Err(shutdown_error()),
                res = tokio::time::timeout_at(deadline, &mut build_handle) => {
                    res.map_err(|_| timeout_error(self.timeout))?
                        .map_err(|err| Error::BuildCredentials {
                            reason: format!("GCP credential construction task failed: {err}"),
                        })??
                }
            };
        }

        // NOTE: `google-cloud-auth` does not configure a request timeout on its
        // internal HTTP clients, so bound `credentials.headers(extensions)` with
        // `deadline` and race it against `shutdown`. Keeping `state` locked
        // without cloning `Credentials` ensures `shutdown_in_flight` cancels
        // active fetches and observes the sole `Credentials` handle dropped
        // (aborting `TokenCache`'s background refresh task) before returning.
        //
        // `google-cloud-auth`'s `TokenCache` only surfaces an error when no valid
        // token remains in its cache, terminates its background refresh loop
        // permanently on non-transient errors, and retains any HTTP 200 payload
        // that `HeaderValue::from_str` accepted (including non-ASCII bytes or
        // non-Bearer token types). Resetting `state` on every error path drops
        // the cached `Credentials` so any poisoned or dead `TokenCache` is
        // replaced on the next retry.
        enum HeadersStep {
            ShutDown,
            TimedOut,
            Completed(
                Result<
                    CacheableResource<http::HeaderMap>,
                    google_cloud_auth::errors::CredentialsError,
                >,
                Option<BearerToken>,
            ),
        }

        let step = {
            let CredentialState::Active(cached) = &*state else {
                return Err(shutdown_error());
            };
            let (extensions, cached_token) = match &cached.last_token {
                Some((ext, tok)) => (ext.clone(), Some(tok.clone())),
                None => (Extensions::new(), None),
            };
            tokio::select! {
                biased;
                _ = shutdown.wait_for(|is_shut_down| *is_shut_down) => HeadersStep::ShutDown,
                res = tokio::time::timeout_at(deadline, cached.credentials.headers(extensions)) => {
                    match res {
                        Ok(headers_res) => HeadersStep::Completed(headers_res, cached_token),
                        Err(_) => HeadersStep::TimedOut,
                    }
                }
            }
        };

        match step {
            HeadersStep::ShutDown => {
                *state = CredentialState::ShutDown;
                Err(shutdown_error())
            }
            HeadersStep::TimedOut => {
                *state = CredentialState::Uninitialized;
                Err(timeout_error(self.timeout))
            }
            HeadersStep::Completed(headers_res, cached_token) => {
                if *shutdown.borrow() {
                    *state = CredentialState::ShutDown;
                    return Err(shutdown_error());
                }
                let outcome = match headers_res {
                    Ok(CacheableResource::New { entity_tag, data }) => {
                        match data
                            .get(http::header::AUTHORIZATION)
                            .and_then(|value| value.to_str().ok())
                            .and_then(BearerToken::from_header_value)
                        {
                            Some(token) => {
                                let mut ext = Extensions::new();
                                let _ = ext.insert(entity_tag);
                                if let CredentialState::Active(cached) = &mut *state {
                                    cached.last_token = Some((ext, token.clone()));
                                }
                                Ok(BackgroundFetchOutcome::Updated(token))
                            }
                            None => Err(Error::TokenAcquisition {
                                message:
                                    "invalid Bearer Authorization header from google-cloud-auth"
                                        .to_string(),
                            }),
                        }
                    }
                    Ok(CacheableResource::NotModified) => match cached_token {
                        Some(token) => Ok(BackgroundFetchOutcome::Unchanged(token)),
                        None => Err(Error::TokenAcquisition {
                            message: "received NotModified without cached token".to_string(),
                        }),
                    },
                    Err(err) => Err(Error::TokenAcquisition {
                        message: format_error_chain(&err),
                    }),
                };
                if outcome.is_err() && matches!(*state, CredentialState::Active(_)) {
                    *state = CredentialState::Uninitialized;
                }
                outcome
            }
        }
    }

    fn clear_cache_on_error(&self) -> bool {
        true
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("gcp_auth.token_refresh_failed", error = %error);
    }

    fn expires_on(value: &BearerToken) -> Option<Instant> {
        value.expires_on()
    }

    async fn shutdown_in_flight(&self) {
        let _ = self.execution.shutdown.send_replace(true);
        let mut guard = self.execution.state.lock().await;
        *guard = CredentialState::ShutDown;
    }
}

/// Formats an error along with its causal `source()` chain so underlying HTTP
/// status codes and transport errors from `google-cloud-auth` are preserved.
fn format_error_chain(err: &dyn std::error::Error) -> String {
    let mut message = err.to_string();
    let mut current = err.source();
    while let Some(source) = current {
        let source_str = source.to_string();
        if !message.contains(&source_str) {
            message.push_str(": ");
            message.push_str(&source_str);
        }
        current = source.source();
    }
    message
}
