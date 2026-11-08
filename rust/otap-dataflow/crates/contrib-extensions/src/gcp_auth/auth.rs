// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Google Cloud Application Default Credentials (ADC) and compute metadata
//! server credential construction and token acquisition via
//! [`google_cloud_auth`].

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use google_cloud_auth::credentials::{Builder as AdcBuilder, CacheableResource, Credentials, mds};
use http::Extensions;
use otel_arrow_dfe_engine::capability::auth::BearerToken;

use super::config::Config;
use super::error::Error;
use crate::common::background_refresh::{BackgroundFetchOutcome, BackgroundProviderSource};

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

/// A Google Cloud token source backed by [`google_cloud_auth`].
#[derive(Clone)]
pub struct Auth {
    endpoint: Option<String>,
    scopes: Vec<String>,
    timeout: Duration,
    state: Arc<Mutex<CredentialState>>,
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
        let scopes = config.scopes.iter().map(|s| s.trim().to_string()).collect();

        // NOTE: `google-cloud-auth` credential builders spawn an internal
        // `TokenCache` task via `tokio::spawn` when `.build()` is called. We
        // defer building `Credentials` until the first async `fetch_outcome()`
        // call so `Auth::new()` remains callable outside a Tokio runtime (such
        // as during synchronous pipeline wiring) and never initiates background
        // network or file I/O before startup.
        Ok(Self {
            endpoint,
            scopes,
            timeout: config.timeout,
            state: Arc::new(Mutex::new(CredentialState::Uninitialized)),
        })
    }

    fn build_credentials(&self) -> Result<Credentials, Error> {
        // When `endpoint` is omitted, use `google_cloud_auth::credentials::Builder`
        // so full Application Default Credentials (ADC) discovery runs
        // (`GOOGLE_APPLICATION_CREDENTIALS` -> well-known `gcloud` ADC file ->
        // GCE/GKE compute metadata server fallback). When `endpoint` is
        // explicitly configured, target that metadata server endpoint directly
        // via `mds::Builder`.
        let credentials = match &self.endpoint {
            Some(endpoint) => {
                let mut builder = mds::Builder::default().with_endpoint(endpoint.clone());
                if !self.scopes.is_empty() {
                    builder = builder.with_scopes(self.scopes.iter().cloned());
                }
                builder.build()
            }
            None => {
                let mut builder = AdcBuilder::default();
                if !self.scopes.is_empty() {
                    builder = builder.with_scopes(self.scopes.iter().cloned());
                }
                builder.build()
            }
        };
        credentials.map_err(|e| Error::BuildCredentials {
            reason: format_error_chain(&e),
        })
    }

    fn reset_cached_credentials(&self) {
        if let Ok(mut guard) = self.state.lock()
            && matches!(*guard, CredentialState::Active(_))
        {
            *guard = CredentialState::Uninitialized;
        }
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
        let (credentials, extensions, cached_token) = {
            let mut guard = self.state.lock().map_err(|_| Error::TokenAcquisition {
                message: "GCP auth credential state lock poisoned".to_string(),
            })?;
            match &*guard {
                CredentialState::ShutDown => {
                    return Err(Error::TokenAcquisition {
                        message: "GCP auth extension is shut down".to_string(),
                    });
                }
                CredentialState::Uninitialized => {
                    let credentials = self.build_credentials()?;
                    *guard = CredentialState::Active(CachedCredentials {
                        credentials: credentials.clone(),
                        last_token: None,
                    });
                    (credentials, Extensions::new(), None)
                }
                CredentialState::Active(cached) => {
                    let (ext, last_tok) = match &cached.last_token {
                        Some((ext, tok)) => (ext.clone(), Some(tok.clone())),
                        None => (Extensions::new(), None),
                    };
                    (cached.credentials.clone(), ext, last_tok)
                }
            }
        };

        // NOTE: `google-cloud-auth` constructs an untimed `reqwest::Client::new()`
        // internally, so bound `credentials.headers(extensions)` with `self.timeout`
        // to ensure a stalled metadata server cannot hold `fetch_lock` indefinitely.
        //
        // `google-cloud-auth`'s `TokenCache` only surfaces an error when no valid
        // token remains in its cache, terminates its background refresh loop
        // permanently on non-transient errors, and retains any HTTP 200 payload
        // that `HeaderValue::from_str` accepted (including non-ASCII bytes or
        // non-Bearer token types). Resetting `self.state` on every error path
        // drops the inner receiver so any poisoned or dead `TokenCache` is
        // replaced on the next retry.
        let outcome: Result<BackgroundFetchOutcome<BearerToken>, Error> = async {
            let headers_res = tokio::time::timeout(self.timeout, credentials.headers(extensions))
                .await
                .map_err(|_| Error::TokenAcquisition {
                    message: format!(
                        "timed out after {:?} waiting for GCP metadata token",
                        self.timeout
                    ),
                })?;

            match headers_res {
                Ok(CacheableResource::New { entity_tag, data }) => {
                    let token = data
                        .get(http::header::AUTHORIZATION)
                        .and_then(|value| value.to_str().ok())
                        .and_then(BearerToken::from_header_value)
                        .ok_or_else(|| Error::TokenAcquisition {
                            message: "invalid Bearer Authorization header from google-cloud-auth"
                                .to_string(),
                        })?;

                    let mut ext = Extensions::new();
                    let _ = ext.insert(entity_tag);
                    if let Ok(mut guard) = self.state.lock()
                        && let CredentialState::Active(cached) = &mut *guard
                    {
                        cached.last_token = Some((ext, token.clone()));
                    }
                    Ok(BackgroundFetchOutcome::Updated(token))
                }
                Ok(CacheableResource::NotModified) => {
                    let token = cached_token.ok_or_else(|| Error::TokenAcquisition {
                        message: "received NotModified without cached token".to_string(),
                    })?;
                    Ok(BackgroundFetchOutcome::Unchanged(token))
                }
                Err(err) => Err(Error::TokenAcquisition {
                    message: format_error_chain(&err),
                }),
            }
        }
        .await;

        outcome.inspect_err(|_| self.reset_cached_credentials())
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
        if let Ok(mut guard) = self.state.lock() {
            *guard = CredentialState::ShutDown;
        }
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
