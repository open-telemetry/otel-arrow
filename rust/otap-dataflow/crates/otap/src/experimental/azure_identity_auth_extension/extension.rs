// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure Identity Auth Extension implementation.

use async_trait::async_trait;
use azure_core::credentials::{AccessToken, TokenCredential};
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId,
};
use std::sync::Arc;

use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::extension::{EffectHandler, Extension};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::terminal_state::TerminalState;

use crate::pdata::OtapPdata;

use super::config::{AuthMethod, Config};
use super::error::Error;

/// Minimum delay between token refresh retry attempts in seconds.
const MIN_RETRY_DELAY_SECS: f64 = 5.0;
/// Maximum delay between token refresh retry attempts in seconds.
const MAX_RETRY_DELAY_SECS: f64 = 30.0;
/// Maximum jitter percentage (±10%) to add to retry delays.
const MAX_RETRY_JITTER_RATIO: f64 = 0.10;

/// Azure Identity Auth Extension.
///
/// This extension provides Azure authentication services to the pipeline.
/// It manages Azure credentials and provides token acquisition capabilities.
pub struct AzureIdentityAuthExtension {
    /// The Azure credential provider.
    credential: Arc<dyn TokenCredential>,
    /// The OAuth scope for token acquisition.
    scope: String,
    /// The authentication method being used.
    method: AuthMethod,
}

// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
impl AzureIdentityAuthExtension {
    /// Creates a new Azure Identity Auth Extension with the given configuration.
    pub fn new(config: Config) -> Result<Self, Error> {
        let credential = Self::create_credential(&config)?;

        Ok(Self {
            credential,
            scope: config.auth.scope.clone(),
            method: config.auth.method.clone(),
        })
    }

    /// Creates a credential provider based on the configuration.
    fn create_credential(config: &Config) -> Result<Arc<dyn TokenCredential>, Error> {
        match config.auth.method {
            AuthMethod::ManagedIdentity => {
                let mut options = ManagedIdentityCredentialOptions::default();

                if let Some(client_id) = &config.auth.client_id {
                    println!(
                        "[AzureIdentityAuthExtension] Using user-assigned managed identity with client_id: {}",
                        client_id
                    );
                    options.user_assigned_id = Some(UserAssignedId::ClientId(client_id.clone()));
                } else {
                    println!(
                        "[AzureIdentityAuthExtension] Using system-assigned managed identity"
                    );
                }

                Ok(ManagedIdentityCredential::new(Some(options))
                    .map_err(|e| Error::create_credential(AuthMethod::ManagedIdentity, e))?)
            }
            AuthMethod::Development => {
                println!(
                    "[AzureIdentityAuthExtension] Using developer tools credential (Azure CLI / Azure Developer CLI)"
                );
                Ok(
                    DeveloperToolsCredential::new(Some(DeveloperToolsCredentialOptions::default()))
                        .map_err(|e| Error::create_credential(AuthMethod::Development, e))?,
                )
            }
        }
    }

    /// Gets a token from the credential provider.
    async fn get_token_internal(&self) -> Result<AccessToken, Error> {
        let token_response = self
            .credential
            .get_token(
                &[&self.scope],
                Some(azure_core::credentials::TokenRequestOptions::default()),
            )
            .await
            .map_err(Error::token_acquisition)?;

        Ok(token_response)
    }

    /// Gets a token with retry logic.
    ///
    /// This method implements exponential backoff with jitter for retrying
    /// token acquisition on failure.
    pub async fn get_token(&mut self) -> Result<AccessToken, Error> {
        let mut attempt = 0_i32;
        loop {
            attempt += 1;

            match self.get_token_internal().await {
                Ok(token) => {
                    println!(
                        "[AzureIdentityAuthExtension] Obtained access token, expires on {}",
                        token.expires_on
                    );
                    return Ok(token);
                }
                Err(e) => {
                    println!(
                        "[AzureIdentityAuthExtension] Failed to obtain access token (attempt {}): {}",
                        attempt, e
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

            println!(
                "[AzureIdentityAuthExtension] Retrying in {:.1}s...",
                delay_secs
            );
            tokio::time::sleep(delay).await;
        }
    }

    /// Returns the authentication method being used.
    pub fn method(&self) -> &AuthMethod {
        &self.method
    }

    /// Returns the OAuth scope.
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// Returns a clone of the credential for sharing with other components.
    pub fn credential(&self) -> Arc<dyn TokenCredential> {
        self.credential.clone()
    }
}

#[async_trait(?Send)]
impl Extension<OtapPdata> for AzureIdentityAuthExtension {
    #[allow(clippy::print_stdout)]
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        effect_handler
            .info(&format!(
                "Azure Identity Auth Extension started with {} authentication",
                self.method
            ))
            .await;

        // Main event loop - extensions only handle control messages
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { reason, .. }) => {
                    effect_handler
                        .info(&format!(
                            "Azure Identity Auth Extension shutting down: {}",
                            reason
                        ))
                        .await;
                    break;
                }
                Message::Control(NodeControlMsg::TimerTick {}) => {
                    // Could implement periodic token refresh here if needed
                    effect_handler
                        .info("Azure Identity Auth Extension received timer tick")
                        .await;
                }
                Message::Control(NodeControlMsg::Config { config }) => {
                    // Handle dynamic configuration updates
                    effect_handler
                        .info(&format!(
                            "Azure Identity Auth Extension received config update: {:?}",
                            config
                        ))
                        .await;
                }
                Message::PData(_) => {
                    // Extensions don't process pipeline data - this shouldn't happen
                    effect_handler
                        .info("Azure Identity Auth Extension received unexpected PData message")
                        .await;
                }
                _ => {
                    // Handle other control messages as needed
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
        let cred: Arc<dyn TokenCredential> = Arc::new(MockCredential {
            token: token.to_string(),
            expires_in,
            call_count,
        });
        cred
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

    impl AzureIdentityAuthExtension {
        /// Creates an extension with a mock credential for testing.
        #[cfg(test)]
        pub fn from_mock(
            credential: Arc<dyn TokenCredential>,
            scope: String,
            method: AuthMethod,
        ) -> Self {
            Self {
                credential,
                scope,
                method,
            }
        }
    }

    // ==================== Construction Tests ====================

    #[tokio::test]
    async fn test_from_mock_creates_extension() {
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            Arc::new(AtomicUsize::new(0)),
        );

        let ext = AzureIdentityAuthExtension::from_mock(
            credential,
            "test_scope".to_string(),
            AuthMethod::Development,
        );
        assert_eq!(ext.scope(), "test_scope");
        assert_eq!(ext.method(), &AuthMethod::Development);
    }

    #[tokio::test]
    async fn test_new_with_managed_identity_system_assigned() {
        let config = Config {
            auth: super::super::config::AuthConfig {
                method: AuthMethod::ManagedIdentity,
                client_id: None,
                scope: "https://test.scope".to_string(),
            },
        };

        let ext = AzureIdentityAuthExtension::new(config);
        assert!(ext.is_ok());
        let ext = ext.unwrap();
        assert_eq!(ext.scope(), "https://test.scope");
    }

    #[tokio::test]
    async fn test_new_with_managed_identity_user_assigned() {
        let config = Config {
            auth: super::super::config::AuthConfig {
                method: AuthMethod::ManagedIdentity,
                client_id: Some("test-client-id".to_string()),
                scope: "https://test.scope".to_string(),
            },
        };

        let ext = AzureIdentityAuthExtension::new(config);
        assert!(ext.is_ok());
    }

    #[tokio::test]
    async fn test_new_with_development_auth() {
        let config = Config {
            auth: super::super::config::AuthConfig {
                method: AuthMethod::Development,
                client_id: None,
                scope: "https://test.scope".to_string(),
            },
        };

        // May fail if Azure CLI not installed - both outcomes are valid
        let result = AzureIdentityAuthExtension::new(config);
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

        let ext = AzureIdentityAuthExtension::from_mock(
            credential,
            "scope".to_string(),
            AuthMethod::Development,
        );

        let token = ext.get_token_internal().await.unwrap();
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

        let ext = AzureIdentityAuthExtension::from_mock(
            credential,
            "scope".to_string(),
            AuthMethod::Development,
        );

        // Each call to get_token_internal should call the credential
        let _ = ext.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        let _ = ext.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        let _ = ext.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    // ==================== Credential Sharing Tests ====================

    #[tokio::test]
    async fn test_credential_returns_shared_reference() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let ext = AzureIdentityAuthExtension::from_mock(
            credential,
            "scope".to_string(),
            AuthMethod::Development,
        );

        // Get shared credential
        let shared_cred = ext.credential();

        // Both should work
        let token1 = ext.get_token_internal().await.unwrap();
        let token2 = shared_cred
            .get_token(&["scope"], Some(TokenRequestOptions::default()))
            .await
            .unwrap();

        assert_eq!(token1.token.secret(), "test_token");
        assert_eq!(token2.token.secret(), "test_token");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_get_token_internal_propagates_credential_error() {
        #[derive(Debug)]
        struct FailingCredential;

        #[async_trait::async_trait]
        impl TokenCredential for FailingCredential {
            async fn get_token(
                &self,
                _scopes: &[&str],
                _options: Option<TokenRequestOptions<'_>>,
            ) -> azure_core::Result<AccessToken> {
                Err(azure_core::error::Error::new(
                    azure_core::error::ErrorKind::Credential,
                    "Mock credential failure",
                ))
            }
        }

        let credential: Arc<dyn TokenCredential> = Arc::new(FailingCredential);
        let ext = AzureIdentityAuthExtension::from_mock(
            credential,
            "scope".to_string(),
            AuthMethod::Development,
        );

        let result = ext.get_token_internal().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Auth {
                kind: super::super::error::AuthErrorKind::TokenAcquisition,
                ..
            } => {}
            err => panic!("Expected Auth token acquisition error, got: {:?}", err),
        }
    }
}
