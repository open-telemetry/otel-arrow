// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Identity Auth Extension implementation.
//!
//! This extension provides Azure authentication services to the pipeline.
//! It manages Azure credentials and provides token acquisition capabilities
//! to consumers (e.g., exporters) via the [`BearerTokenProvider`] trait.
//!
//! # Architecture
//!
//! `AzureIdentityAuthExtension` is a single `Clone` struct that serves both
//! as the pipeline extension (implementing [`Extension`] and driving the token
//! refresh loop) and as the registry service (implementing [`BearerTokenProvider`]).
//!
//! Consumers retrieve it from the extension
//! registry via `registry.get::<dyn BearerTokenProvider>("name")`.
//!
//! State is shared through `Arc`:
//! - `Arc<dyn TokenCredential>` — the Azure credential provider
//! - `Arc<watch::Sender<Option<BearerToken>>>` — token broadcast channel

use async_trait::async_trait;
use azure_core::credentials::{AccessToken, TokenCredential};
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId,
};
use otap_df_engine::extension::bearer_token_provider::{BearerToken, BearerTokenProvider};
use otap_df_telemetry::{otel_debug, otel_error, otel_info, otel_warn};
use std::sync::Arc;
use tokio::sync::watch;

use otap_df_engine::control::ExtensionControlMsg;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::extension::{ControlChannel, EffectHandler, Extension};
use otap_df_engine::terminal_state::TerminalState;

use super::config::{AuthMethod, Config};
use super::error::Error;

/// Minimum delay between token refresh retry attempts in seconds.
const MIN_RETRY_DELAY_SECS: f64 = 5.0;
/// Maximum delay between token refresh retry attempts in seconds.
const MAX_RETRY_DELAY_SECS: f64 = 30.0;
/// Maximum jitter percentage (±10%) to add to retry delays.
const MAX_RETRY_JITTER_RATIO: f64 = 0.10;

/// Buffer time before token expiry to trigger refresh (in seconds).
/// Tokens will be refreshed ~5 minutes before they expire.
const TOKEN_EXPIRY_BUFFER_SECS: u64 = 299;
/// Minimum interval between token refresh attempts (in seconds).
const MIN_TOKEN_REFRESH_INTERVAL_SECS: u64 = 10;
/// Retry interval when token refresh fails (in seconds).
const TOKEN_REFRESH_RETRY_SECS: u64 = 10;

/// Azure Identity Auth Extension.
///
/// This is a single `Clone` struct that serves as both the pipeline extension
/// (implementing [`Extension`] to drive the token refresh loop) and the registry
/// service (implementing [`BearerTokenProvider`]).
///
/// Consumers retrieve this via `registry.get::<dyn BearerTokenProvider>("name")`.
/// Cheap to clone — all state is behind `Arc`.
#[derive(Clone)]
pub struct AzureIdentityAuthExtension {
    /// The configured name of this extension instance (from YAML config key).
    name: String,
    /// The Azure credential provider.
    credential: Arc<dyn TokenCredential>,
    /// Human-readable description of the credential type created.
    credential_type: &'static str,
    /// The OAuth scope for token acquisition.
    scope: String,
    /// The authentication method being used.
    method: AuthMethod,
    /// Optional client ID for user-assigned managed identity.
    client_id: Option<String>,
    /// Sender for broadcasting / subscribing to token refresh events.
    token_sender: Arc<watch::Sender<Option<BearerToken>>>,
}

impl AzureIdentityAuthExtension {
    /// Creates a new Azure Identity Auth Extension.
    pub fn new(name: String, config: Config) -> Result<Self, Error> {
        let (credential, credential_type) = Self::create_credential(&config)?;
        let (token_sender, _) = watch::channel(None);
        let token_sender = Arc::new(token_sender);

        Ok(Self {
            name,
            credential,
            credential_type,
            scope: config.scope,
            method: config.method,
            client_id: config.client_id,
            token_sender,
        })
    }

    /// Creates a credential provider based on the configuration.
    ///
    /// Returns the credential and a human-readable description of the credential type.
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
                Ok(token) => {
                    otel_debug!(
                        "azure_identity_auth.get_token_succeeded",
                        expires_on = %token.expires_on
                    );
                    return Ok(token);
                }
                Err(e) => {
                    otel_warn!(
                        "azure_identity_auth.get_token_failed",
                        attempt = attempt,
                        error = %e
                    );
                }
            }

            // Calculate exponential backoff: 5s, 10s, 20s, 30s (capped)
            let base_delay_secs = MIN_RETRY_DELAY_SECS * 2.0_f64.powi(attempt - 1);
            let capped_delay_secs = base_delay_secs.min(MAX_RETRY_DELAY_SECS);

            // Add jitter: random value between -10% and +10% of the delay
            let jitter_range = capped_delay_secs * MAX_RETRY_JITTER_RATIO;
            let jitter = if jitter_range > 0.0 {
                let random_factor = rand::random::<f64>() * 2.0 - 1.0;
                random_factor * jitter_range
            } else {
                0.0
            };

            let delay_secs = (capped_delay_secs + jitter).max(1.0);
            let delay = tokio::time::Duration::from_secs_f64(delay_secs);

            otel_warn!(
                "azure_identity_auth.retry_scheduled",
                delay_secs = %delay_secs
            );
            tokio::time::sleep(delay).await;
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

    /// Returns the authentication method being used.
    #[must_use]
    pub fn method(&self) -> &AuthMethod {
        &self.method
    }

    /// Returns the OAuth scope.
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.scope
    }
}

#[async_trait]
impl BearerTokenProvider for AzureIdentityAuthExtension {
    async fn get_token(&self) -> Result<BearerToken, otap_df_engine::extension::registry::Error> {
        let access_token = self.get_token_with_retry().await?;

        Ok(BearerToken::new(
            access_token.token.secret().to_string(),
            access_token.expires_on.unix_timestamp(),
        ))
    }

    fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>> {
        self.token_sender.subscribe()
    }
}

#[async_trait(?Send)]
impl Extension for AzureIdentityAuthExtension {
    otap_df_engine::extension_capabilities!(BearerTokenProvider);

    async fn start(
        self: Box<Self>,
        mut ctrl_chan: ControlChannel,
        _: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        otel_info!(
            "azure_identity_auth.start",
            name = self.name.as_str(),
            credential_type = self.credential_type,
            scope = self.scope.as_str(),
            client_id = self.client_id.as_deref().unwrap_or("none"),
        );

        // Fetch initial token immediately
        let mut next_token_refresh = tokio::time::Instant::now();

        // Main event loop — extensions handle control messages and proactive token refresh
        loop {
            tokio::select! {
                biased;

                // Proactive token refresh — keeps Azure Identity's internal cache warm
                _ = tokio::time::sleep_until(next_token_refresh) => {
                    match self.get_token_with_retry().await {
                        Ok(access_token) => {
                            let bearer_token = BearerToken::new(
                                access_token.token.secret().to_string(),
                                access_token.expires_on.unix_timestamp(),
                            );

                            // Broadcast the new token to all subscribers
                            let _ = self.token_sender.send(Some(bearer_token.clone()));

                            // Schedule next refresh
                            next_token_refresh = Self::get_next_token_refresh(&bearer_token);

                            let refresh_in = next_token_refresh.saturating_duration_since(tokio::time::Instant::now());
                            let total_secs = refresh_in.as_secs();
                            let hours = total_secs / 3600;
                            let minutes = (total_secs % 3600) / 60;
                            let seconds = total_secs % 60;

                            otel_info!(
                                "azure_identity_auth.token_refreshed",
                                refresh_in = format!("{}h {}m {}s", hours, minutes, seconds)
                            );
                        }
                        Err(e) => {
                            otel_error!(
                                "azure_identity_auth.token_refresh_loop_failed",
                                error = ?e,
                                retry_secs = TOKEN_REFRESH_RETRY_SECS
                            );
                            // Retry after a short delay
                            next_token_refresh = tokio::time::Instant::now()
                                + tokio::time::Duration::from_secs(TOKEN_REFRESH_RETRY_SECS);
                        }
                    }
                }

                // Handle control messages
                msg = ctrl_chan.recv() => {
                    match msg? {
                        ExtensionControlMsg::Shutdown { reason, .. } => {
                            otel_info!(
                                "azure_identity_auth.shutdown",
                                reason = %reason
                            );
                            break;
                        }
                        ExtensionControlMsg::Config { config } => {
                            otel_info!(
                                "azure_identity_auth.config_update",
                                config = ?config
                            );
                        }
                        ExtensionControlMsg::CollectTelemetry { .. } => {
                            // Telemetry collection handled by pipeline metrics
                        }
                    }
                }
            }
        }

        Ok(TerminalState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azure_core::credentials::TokenRequestOptions;
    use azure_core::time::OffsetDateTime;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct MockCredential {
        token: String,
        expires_in: azure_core::time::Duration,
        call_count: Arc<AtomicUsize>,
    }

    fn make_mock_credential(
        token: &str,
        expires_in: azure_core::time::Duration,
        call_count: Arc<AtomicUsize>,
    ) -> Arc<dyn TokenCredential> {
        Arc::new(MockCredential {
            token: token.to_string(),
            expires_in,
            call_count,
        })
    }

    #[async_trait::async_trait]
    impl TokenCredential for MockCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            let _ = self.call_count.fetch_add(1, Ordering::SeqCst);

            Ok(AccessToken {
                token: self.token.clone().into(),
                expires_on: OffsetDateTime::now_utc() + self.expires_in,
            })
        }
    }

    /// Creates a test extension from a mock credential.
    fn make_test_extension(
        credential: Arc<dyn TokenCredential>,
        scope: &str,
    ) -> AzureIdentityAuthExtension {
        let (token_sender, _) = watch::channel(None);
        AzureIdentityAuthExtension {
            name: "test".to_string(),
            credential,
            credential_type: "mock",
            scope: scope.to_string(),
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            token_sender: Arc::new(token_sender),
        }
    }

    // ==================== Construction Tests ====================

    #[tokio::test]
    async fn test_new_with_managed_identity_system_assigned() {
        let config = Config {
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            scope: "https://test.scope".to_string(),
        };

        let result = AzureIdentityAuthExtension::new("test".to_string(), config);
        assert!(result.is_ok());
        let ext = result.unwrap();
        assert_eq!(ext.scope(), "https://test.scope");
    }

    #[tokio::test]
    async fn test_new_with_managed_identity_user_assigned() {
        let config = Config {
            method: AuthMethod::ManagedIdentity,
            client_id: Some("test-client-id".to_string()),
            scope: "https://test.scope".to_string(),
        };

        let result = AzureIdentityAuthExtension::new("test".to_string(), config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_new_with_development_auth() {
        let config = Config {
            method: AuthMethod::Development,
            client_id: None,
            scope: "https://test.scope".to_string(),
        };

        // May fail if Azure CLI not installed — both outcomes are valid
        let result = AzureIdentityAuthExtension::new("test".to_string(), config);
        match result {
            Ok(ext) => assert_eq!(ext.scope(), "https://test.scope"),
            Err(Error::Auth {
                kind: super::super::error::AuthErrorKind::CreateCredential { method },
                ..
            }) => {
                assert_eq!(method, AuthMethod::Development);
            }
            Err(err) => panic!("Unexpected error type: {:?}", err),
        }
    }

    // ==================== Token Fetching Tests ====================

    #[tokio::test]
    async fn test_get_token_internal_returns_valid_token() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let service = make_test_extension(credential, "scope");

        let token = service.get_token_internal().await.unwrap();
        assert_eq!(token.token.secret(), "test_token");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_get_token_internal_calls_credential_each_time() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let service = make_test_extension(credential, "scope");

        let _ = service.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        let _ = service.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        let _ = service.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    // ==================== BearerTokenProvider Trait Tests ====================

    #[tokio::test]
    async fn test_bearer_token_provider_get_token() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "bearer_test_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let service = make_test_extension(credential, "scope");

        // Use the BearerTokenProvider trait method
        let token: BearerToken = BearerTokenProvider::get_token(&service).await.unwrap();
        assert_eq!(token.token.secret(), "bearer_test_token");
        assert!(token.expires_on > 0);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_bearer_token_provider_subscribe_token_refresh() {
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            Arc::new(AtomicUsize::new(0)),
        );

        let (token_sender, _) = watch::channel(None);
        let token_sender = Arc::new(token_sender);
        let service = AzureIdentityAuthExtension {
            name: "test".to_string(),
            credential,
            credential_type: "mock",
            scope: "scope".to_string(),
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            token_sender: token_sender.clone(),
        };

        // Get a subscriber
        let mut rx = BearerTokenProvider::subscribe_token_refresh(&service);

        // Initially should be None
        assert!(rx.borrow().is_none());

        // Simulate token broadcast (as the extension would do)
        let new_token = BearerToken::new("refreshed_token".to_string(), 12345);
        let _ = token_sender.send(Some(new_token));

        // Subscriber should receive the update
        rx.changed().await.unwrap();
        let received = rx.borrow();
        assert!(received.is_some());
        let received_token = received.as_ref().unwrap();
        assert_eq!(received_token.token.secret(), "refreshed_token");
        assert_eq!(received_token.expires_on, 12345);
    }

    #[tokio::test]
    async fn test_multiple_subscribers_receive_token_updates() {
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            Arc::new(AtomicUsize::new(0)),
        );

        let (token_sender, _) = watch::channel(None);
        let token_sender = Arc::new(token_sender);
        let service = AzureIdentityAuthExtension {
            name: "test".to_string(),
            credential,
            credential_type: "mock",
            scope: "scope".to_string(),
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            token_sender: token_sender.clone(),
        };

        // Create multiple subscribers
        let mut rx1 = BearerTokenProvider::subscribe_token_refresh(&service);
        let mut rx2 = BearerTokenProvider::subscribe_token_refresh(&service);

        // Broadcast a token
        let token = BearerToken::new("broadcast_token".to_string(), 99999);
        let _ = token_sender.send(Some(token));

        // Both subscribers should receive the update
        rx1.changed().await.unwrap();
        rx2.changed().await.unwrap();

        assert_eq!(
            rx1.borrow().as_ref().unwrap().token.secret(),
            "broadcast_token"
        );
        assert_eq!(
            rx2.borrow().as_ref().unwrap().token.secret(),
            "broadcast_token"
        );
    }

    // ==================== Token Refresh Scheduling Tests ====================

    #[test]
    fn test_get_next_token_refresh_schedules_before_expiry() {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let token = BearerToken::new("test".to_string(), now_secs + 600);
        let next_refresh = AzureIdentityAuthExtension::get_next_token_refresh(&token);

        let now = tokio::time::Instant::now();
        let min_expected = now + tokio::time::Duration::from_secs(MIN_TOKEN_REFRESH_INTERVAL_SECS);

        assert!(next_refresh >= min_expected);
    }

    #[test]
    fn test_get_next_token_refresh_respects_minimum_interval() {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let token = BearerToken::new("test".to_string(), now_secs + 5);
        let next_refresh = AzureIdentityAuthExtension::get_next_token_refresh(&token);

        let now = tokio::time::Instant::now();
        let min_expected =
            now + tokio::time::Duration::from_secs(MIN_TOKEN_REFRESH_INTERVAL_SECS - 1);

        assert!(next_refresh >= min_expected);
    }

    #[test]
    fn test_get_next_token_refresh_handles_expired_token() {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let token = BearerToken::new("test".to_string(), now_secs - 100);
        let next_refresh = AzureIdentityAuthExtension::get_next_token_refresh(&token);

        let now = tokio::time::Instant::now();
        let min_expected =
            now + tokio::time::Duration::from_secs(MIN_TOKEN_REFRESH_INTERVAL_SECS - 1);

        assert!(next_refresh >= min_expected);
    }

    #[test]
    fn test_get_next_token_refresh_long_lived_token() {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let token = BearerToken::new("test".to_string(), now_secs + 3600);
        let next_refresh = AzureIdentityAuthExtension::get_next_token_refresh(&token);

        let now = tokio::time::Instant::now();
        let expected_approx =
            now + tokio::time::Duration::from_secs(3600 - TOKEN_EXPIRY_BUFFER_SECS);

        let tolerance = tokio::time::Duration::from_secs(2);
        assert!(next_refresh >= expected_approx - tolerance);
        assert!(next_refresh <= expected_approx + tolerance);
    }

    #[test]
    fn test_provider_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AzureIdentityAuthExtension>();
    }
}
