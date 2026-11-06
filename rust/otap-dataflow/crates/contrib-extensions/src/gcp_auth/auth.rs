// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Google Cloud compute metadata server credential construction and token
//! acquisition via [`google_cloud_auth`].

use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use google_cloud_auth::credentials::{CacheableResource, Credentials, mds};
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

/// A Google Cloud metadata token source backed by [`google_cloud_auth`].
#[derive(Clone)]
pub struct Auth {
    endpoint: Option<String>,
    scopes: Vec<String>,
    state: Arc<Mutex<Option<CachedCredentials>>>,
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let endpoint = config
            .normalized_endpoint()
            .map_err(|reason| Error::BuildCredentials { reason })?;

        let mut scopes = Vec::with_capacity(config.scopes.len());
        for scope in &config.scopes {
            let trimmed = scope.trim();
            if trimmed.is_empty() {
                return Err(Error::BuildCredentials {
                    reason: "`scopes` entries must not be empty".to_string(),
                });
            }
            scopes.push(trimmed.to_string());
        }

        // NOTE: `mds::Builder::build()` spawns `google-cloud-auth`'s internal
        // `TokenCache` task via `tokio::spawn`. We defer building `Credentials`
        // until the first async `fetch_outcome()` call so `Auth::new()` remains
        // callable outside a Tokio runtime (such as during synchronous pipeline
        // wiring) and never initiates background network traffic before startup.
        Ok(Self {
            endpoint,
            scopes,
            state: Arc::new(Mutex::new(None)),
        })
    }

    fn build_credentials(&self) -> Result<Credentials, Error> {
        let mut builder = mds::Builder::default();
        if let Some(endpoint) = &self.endpoint {
            builder = builder.with_endpoint(endpoint.clone());
        }
        if !self.scopes.is_empty() {
            builder = builder.with_scopes(self.scopes.iter().cloned());
        }
        builder.build().map_err(|e| Error::BuildCredentials {
            reason: e.to_string(),
        })
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
        let (credentials, extensions) = {
            let mut guard = self.state.lock().map_err(|_| Error::TokenAcquisition {
                message: "GCP auth credential state lock poisoned".to_string(),
            })?;
            if guard.is_none() {
                let credentials = self.build_credentials()?;
                *guard = Some(CachedCredentials {
                    credentials,
                    last_token: None,
                });
            }
            let cached = guard.as_ref().expect("populated above");
            let ext = cached
                .last_token
                .as_ref()
                .map(|(ext, _)| ext.clone())
                .unwrap_or_default();
            (cached.credentials.clone(), ext)
        };

        let headers_res = credentials.headers(extensions).await;
        match headers_res {
            Ok(CacheableResource::New { entity_tag, data }) => {
                let header = data
                    .get(http::header::AUTHORIZATION)
                    .ok_or_else(|| Error::TokenAcquisition {
                        message: "missing Authorization header from google-cloud-auth".to_string(),
                    })?
                    .to_str()
                    .map_err(|e| Error::TokenAcquisition {
                        message: format!(
                            "non-ASCII Authorization header from google-cloud-auth: {e}"
                        ),
                    })?;

                let token = BearerToken::from_header_value(header).ok_or_else(|| {
                    if let Ok(mut guard) = self.state.lock() {
                        *guard = None;
                    }
                    Error::TokenAcquisition {
                        message: "invalid Bearer Authorization header from google-cloud-auth"
                            .to_string(),
                    }
                })?;

                let mut ext = Extensions::new();
                let _ = ext.insert(entity_tag);
                if let Ok(mut guard) = self.state.lock()
                    && let Some(cached) = guard.as_mut()
                {
                    cached.last_token = Some((ext, token.clone()));
                }
                Ok(BackgroundFetchOutcome::Updated(token))
            }
            Ok(CacheableResource::NotModified) => {
                let cached_token = self
                    .state
                    .lock()
                    .ok()
                    .and_then(|guard| {
                        guard
                            .as_ref()
                            .and_then(|c| c.last_token.as_ref().map(|(_, token)| token.clone()))
                    })
                    .ok_or_else(|| Error::TokenAcquisition {
                        message: "received NotModified without cached token".to_string(),
                    })?;
                Ok(BackgroundFetchOutcome::Unchanged(cached_token))
            }
            Err(err) => {
                // NOTE: `google-cloud-auth`'s `TokenCache` only surfaces an
                // error when no valid token remains in its cache (initial
                // acquisition failed, or a previously cached token expired
                // while refreshes were failing), and terminates its background
                // refresh loop permanently on non-transient errors. Resetting
                // `self.state` drops the inner receiver so any redundant SDK
                // retry task exits and the next retry scheduled by
                // `BackgroundProviderExtension` starts a fresh `TokenCache`.
                if let Ok(mut guard) = self.state.lock() {
                    *guard = None;
                }
                Err(Error::TokenAcquisition {
                    message: format_error_chain(&err),
                })
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
        if let Ok(mut guard) = self.state.lock() {
            *guard = None;
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
