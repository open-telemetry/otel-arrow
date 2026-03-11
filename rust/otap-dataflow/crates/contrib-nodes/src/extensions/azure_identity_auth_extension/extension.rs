// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Identity Auth Extension implementation.
//!
//! This extension provides Azure authentication services to the pipeline.
//! It manages Azure credentials and provides token acquisition capabilities
//! to consumers (e.g., exporters) via the [`BearerTokenProviderHandle`].
//!
//! # Architecture
//!
//! The extension creates a [`BearerTokenProviderHandle`] at construction time,
//! which wraps an `AzureTokenProvider` implementing the [`BearerTokenProvider`]
//! trait. The extension background task refreshes tokens periodically and
//! broadcasts updates through a `watch` channel that subscribers receive via
//! the handle.
//!
//! Consumers retrieve the handle from the extension registry:
//!
//! ```rust,ignore
//! let handle = extension_registry
//!     .get::<BearerTokenProviderHandle>("azure_auth")?;
//! let token = handle.get_token().await?;
//! ```

use async_trait::async_trait;
use azure_core::credentials::{AccessToken, TokenCredential};
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId,
};
use otap_df_engine::extensions::{
    BearerToken, BearerTokenError, BearerTokenProvider,
};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::extension as local;

use super::config::{AuthMethod, Config};
use super::error::Error;

/// Buffer time before token expiry to trigger refresh (in seconds).
const TOKEN_EXPIRY_BUFFER_SECS: u64 = 299;
/// Minimum interval between token refresh attempts (in seconds).
const MIN_TOKEN_REFRESH_INTERVAL_SECS: u64 = 10;
/// Minimum delay between token refresh retry attempts in seconds.
const MIN_RETRY_DELAY_SECS: f64 = 5.0;
/// Maximum delay between token refresh retry attempts in seconds.
const MAX_RETRY_DELAY_SECS: f64 = 30.0;
/// Maximum jitter percentage (±10%) to add to retry delays.
const MAX_RETRY_JITTER_RATIO: f64 = 0.10;
/// Retry interval when token refresh fails (in seconds).
const TOKEN_REFRESH_RETRY_SECS: u64 = 10;

/// The token provider implementation that backs the [`BearerTokenProviderHandle`].
///
/// Reads the latest token from a shared cache (updated by the extension's
/// background refresh loop) and exposes a watch channel for subscribers.
/// If the cache is empty or expired, falls back to fetching from Azure directly.
pub(crate) struct AzureTokenProvider {
    /// The Azure credential provider.
    credential: Arc<dyn TokenCredential>,
    /// The OAuth scope for token acquisition.
    scope: String,
    /// Sender for broadcasting token refresh events (used by `subscribe_token_refresh`).
    token_sender: Arc<watch::Sender<Option<BearerToken>>>,
    /// Cached token updated by the extension's refresh loop (used by `get_token`).
    token_cache_for_demonstration: Arc<Mutex<Option<BearerToken>>>,
}

#[async_trait]
impl BearerTokenProvider for AzureTokenProvider {
    async fn get_token(&self) -> Result<BearerToken, BearerTokenError> {
        // Fast path: return cached token if it hasn't expired.
        {
            let cache = self.token_cache_for_demonstration.lock().expect("token_cache lock poisoned");
            if let Some(cached) = cache.as_ref() {
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                if cached.expires_on > now_secs {
                    return Ok(cached.clone());
                }
            }
        }

        // Slow path: cache is empty or expired — fetch from Azure.
        let access_token = self
            .credential
            .get_token(
                &[&self.scope],
                Some(azure_core::credentials::TokenRequestOptions::default()),
            )
            .await
            .map_err(|e| BearerTokenError {
                message: e.to_string(),
            })?;

        let token = BearerToken::new(
            access_token.token.secret().to_string(),
            access_token.expires_on.unix_timestamp(),
        );

        // Update cache so next call hits the fast path.
        *self.token_cache_for_demonstration.lock().expect("token_cache lock poisoned") = Some(token.clone());

        Ok(token)
    }

    fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>> {
        self.token_sender.subscribe()
    }
}

/// Azure Identity Auth Extension.
///
/// Runs as a background task that periodically refreshes the Azure bearer
/// token and broadcasts updates through the token provider handle.
pub struct AzureIdentityAuthExtension {
    /// The Azure credential provider (shared with the token provider).
    credential: Arc<dyn TokenCredential>,
    /// Human-readable description of the credential type.
    credential_type: &'static str,
    /// The OAuth scope for token acquisition.
    scope: String,
    /// Sender for broadcasting token refresh events.
    token_sender: Arc<watch::Sender<Option<BearerToken>>>,
    /// Shared cache updated by the refresh loop, read by the provider handle.
    token_cache_for_demonstration: Arc<Mutex<Option<BearerToken>>>,
}

impl AzureIdentityAuthExtension {
    /// Creates a new Azure Identity Auth Extension and its associated token provider.
    ///
    /// Returns both the extension (for the background task) and the provider
    /// (to be wrapped in a `BearerTokenProviderHandle` and registered).
    pub(crate) fn new(config: &Config) -> Result<(Self, AzureTokenProvider), Error> {
        let (credential, credential_type) = Self::create_credential(config)?;
        let (token_sender, _) = watch::channel(None);
        let token_sender = Arc::new(token_sender);
        let token_cache_for_demonstration = Arc::new(Mutex::new(None));

        let provider = AzureTokenProvider {
            credential: Arc::clone(&credential),
            scope: config.scope.clone(),
            token_sender: Arc::clone(&token_sender),
            token_cache_for_demonstration: Arc::clone(&token_cache_for_demonstration),
        };

        let extension = Self {
            credential,
            credential_type,
            scope: config.scope.clone(),
            token_sender,
            token_cache_for_demonstration,
        };

        Ok((extension, provider))
    }

    /// Creates a credential provider based on the configuration.
    fn create_credential(
        config: &Config,
    ) -> Result<(Arc<dyn TokenCredential>, &'static str), Error> {
        match config.method {
            AuthMethod::ManagedIdentity => {
                let mut options = ManagedIdentityCredentialOptions::default();

                let credential_type = if let Some(client_id) = &config.client_id {
                    options.user_assigned_id = Some(UserAssignedId::ClientId(client_id.clone()));
                    "user_assigned_managed_identity"
                } else {
                    "system_assigned_managed_identity"
                };

                Ok((
                    ManagedIdentityCredential::new(Some(options))
                        .map_err(|e| Error::create_credential(AuthMethod::ManagedIdentity, e))?,
                    credential_type,
                ))
            }
            AuthMethod::Development => Ok((
                DeveloperToolsCredential::new(Some(DeveloperToolsCredentialOptions::default()))
                    .map_err(|e| Error::create_credential(AuthMethod::Development, e))?,
                "developer_tools",
            )),
        }
    }

    /// Gets a token directly from the credential provider.
    async fn get_token_internal(&self) -> Result<AccessToken, Error> {
        self.credential
            .get_token(
                &[&self.scope],
                Some(azure_core::credentials::TokenRequestOptions::default()),
            )
            .await
            .map_err(Error::token_acquisition)
    }

    /// Gets a token with retry logic and exponential backoff.
    async fn get_token_with_retry(&self) -> Result<AccessToken, Error> {
        let mut attempt = 0_i32;
        loop {
            attempt += 1;

            match self.get_token_internal().await {
                Ok(token) => return Ok(token),
                Err(_e) if attempt < 10 => {
                    let base_delay_secs = MIN_RETRY_DELAY_SECS * 2.0_f64.powi(attempt - 1);
                    let capped_delay_secs = base_delay_secs.min(MAX_RETRY_DELAY_SECS);

                    let jitter_range = capped_delay_secs * MAX_RETRY_JITTER_RATIO;
                    let jitter = if jitter_range > 0.0 {
                        let random_factor = rand::random::<f64>() * 2.0 - 1.0;
                        random_factor * jitter_range
                    } else {
                        0.0
                    };

                    let delay_secs = (capped_delay_secs + jitter).max(1.0);
                    tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay_secs)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Calculates when the next token refresh should occur.
    fn get_next_token_refresh(token: &BearerToken) -> tokio::time::Instant {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let duration_remaining = if token.expires_on > now_secs {
            std::time::Duration::from_secs((token.expires_on - now_secs) as u64)
        } else {
            std::time::Duration::ZERO
        };

        let token_valid_until = tokio::time::Instant::now() + duration_remaining;
        let next_token_refresh =
            token_valid_until - tokio::time::Duration::from_secs(TOKEN_EXPIRY_BUFFER_SECS);
        std::cmp::max(
            next_token_refresh,
            tokio::time::Instant::now()
                + tokio::time::Duration::from_secs(MIN_TOKEN_REFRESH_INTERVAL_SECS),
        )
    }
}

#[async_trait(?Send)]
impl local::Extension for AzureIdentityAuthExtension {
    async fn start(
        self: Box<Self>,
        mut ctrl_chan: local::ControlChannel,
        effect_handler: local::EffectHandler,
    ) -> Result<(), EngineError> {
        effect_handler
            .info(&format!(
                "[azure-identity-auth] starting (credential_type={}, scope={})",
                self.credential_type, self.scope
            ))
            .await;

        // Fetch initial token immediately.
        let mut next_token_refresh = tokio::time::Instant::now();

        loop {
            tokio::select! {
                biased;

                // Proactive token refresh.
                _ = tokio::time::sleep_until(next_token_refresh) => {
                    match self.get_token_with_retry().await {
                        Ok(access_token) => {
                            let bearer_token = BearerToken::new(
                                access_token.token.secret().to_string(),
                                access_token.expires_on.unix_timestamp(),
                            );

                            // Update shared cache so get_token() can return it directly.
                            *self.token_cache_for_demonstration.lock().expect("token_cache lock poisoned") =
                                Some(bearer_token.clone());

                            // Broadcast the new token to all subscribers.
                            let _ = self.token_sender.send(Some(bearer_token.clone()));

                            next_token_refresh =
                                Self::get_next_token_refresh(&bearer_token);

                            effect_handler
                                .info("[azure-identity-auth] token refreshed")
                                .await;
                        }
                        Err(_e) => {
                            next_token_refresh = tokio::time::Instant::now()
                                + tokio::time::Duration::from_secs(TOKEN_REFRESH_RETRY_SECS);

                            effect_handler
                                .info("[azure-identity-auth] token refresh failed, retrying")
                                .await;
                        }
                    }
                }

                // Handle control messages.
                msg = ctrl_chan.recv() => {
                    match msg {
                        Ok(msg) if msg.is_shutdown() => {
                            effect_handler
                                .info("[azure-identity-auth] shutting down")
                                .await;
                            break;
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            }
        }

        Ok(())
    }
}
