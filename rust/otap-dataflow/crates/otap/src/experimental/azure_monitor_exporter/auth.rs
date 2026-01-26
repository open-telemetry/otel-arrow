// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use azure_core::credentials::{AccessToken, TokenCredential};
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId,
};
use std::sync::Arc;

use super::Error;
use super::config::{AuthConfig, AuthMethod};

/// Minimum delay between token refresh retry attempts in seconds.
const MIN_RETRY_DELAY_SECS: f64 = 5.0;
/// Maximum delay between token refresh retry attempts in seconds.
const MAX_RETRY_DELAY_SECS: f64 = 30.0;
/// Maximum jitter percentage (±10%) to add to retry delays.
const MAX_RETRY_JITTER_PERC: f64 = 0.10;

#[derive(Clone, Debug)]
// TODO - Consolidate with crates/otap/src/{cloud_auth,object_store)/azure.rs
#[allow(clippy::print_stdout)]
pub struct Auth {
    credential: Arc<dyn TokenCredential>,
    scope: String,
}

// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
impl Auth {
    pub fn new(auth_config: &AuthConfig) -> Result<Self, Error> {
        let credential = Self::create_credential(auth_config)?;

        Ok(Self {
            credential,
            scope: auth_config.scope.clone(),
        })
    }

    #[cfg(test)]
    pub fn from_credential(credential: Arc<dyn TokenCredential>, scope: String) -> Self {
        Self {
            credential,
            scope,
        }
    }

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

    pub async fn get_token(&mut self) -> Result<AccessToken, Error> {
        let mut attempt = 0_i32;
        loop {
            attempt += 1;

            match self.get_token_internal().await {
                Ok(token) => {
                    println!(
                        "[AzureMonitorExporter] Obtained access token, expires on {}",
                        token.expires_on
                    );
                    return Ok(token);
                }
                Err(e) => {
                    println!(
                        "[AzureMonitorExporter] Failed to obtain access token (attempt {}): {e}",
                        attempt
                    );
                }
            }

            // Calculate exponential backoff: 5s, 10s, 20s, 30s (capped)
            let base_delay_secs = MIN_RETRY_DELAY_SECS * 2.0_f64.powi(attempt - 1);
            let capped_delay_secs = base_delay_secs.min(MAX_RETRY_DELAY_SECS);

            // Add jitter: random value between -10% and +10% of the delay
            let jitter_range = capped_delay_secs * MAX_RETRY_JITTER_PERC;
            let jitter = if jitter_range > 0.0 {
                let random_factor = rand::random::<f64>() * 2.0 - 1.0;
                random_factor * jitter_range
            } else {
                0.0
            };

            let delay_secs = (capped_delay_secs + jitter).max(1.0);
            let delay = tokio::time::Duration::from_secs_f64(delay_secs);

            println!("[AzureMonitorExporter] Retrying in {:.1}s...", delay_secs);
            tokio::time::sleep(delay).await;
        }
    }


    fn create_credential(auth_config: &AuthConfig) -> Result<Arc<dyn TokenCredential>, Error> {
        match auth_config.method {
            AuthMethod::ManagedIdentity => {
                let mut options = ManagedIdentityCredentialOptions::default();

                if let Some(client_id) = &auth_config.client_id {
                    println!("Using user-assigned managed identity with client_id: {client_id}");
                    options.user_assigned_id = Some(UserAssignedId::ClientId(client_id.clone()));
                } else {
                    println!("Using system-assigned managed identity");
                }

                Ok(ManagedIdentityCredential::new(Some(options))
                    .map_err(|e| Error::create_credential(AuthMethod::ManagedIdentity, e))?)
            }
            AuthMethod::Development => {
                println!("Using developer tools credential (Azure CLI / Azure Developer CLI)");
                Ok(DeveloperToolsCredential::new(Some(
                    DeveloperToolsCredentialOptions::default(),
                ))
                .map_err(|e| Error::create_credential(AuthMethod::Development, e))?)
            }
        }
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

    // ==================== Construction Tests ====================

    #[tokio::test]
    async fn test_from_credential_creates_auth() {
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            Arc::new(AtomicUsize::new(0)),
        );

        let auth = Auth::from_credential(credential, "test_scope".to_string());
        assert_eq!(auth.scope, "test_scope");
    }

    #[tokio::test]
    async fn test_new_with_managed_identity_user_assigned() {
        let auth_config = AuthConfig {
            method: AuthMethod::ManagedIdentity,
            client_id: Some("test-client-id".to_string()),
            scope: "https://test.scope".to_string(),
        };

        let auth = Auth::new(&auth_config);
        assert!(auth.is_ok());
        let auth = auth.unwrap();
        assert_eq!(auth.scope, "https://test.scope");
    }

    #[tokio::test]
    async fn test_new_with_managed_identity_system_assigned() {
        let auth_config = AuthConfig {
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            scope: "https://test.scope".to_string(),
        };

        let auth = Auth::new(&auth_config);
        assert!(auth.is_ok());
    }

    #[tokio::test]
    async fn test_new_with_development_auth() {
        let auth_config = AuthConfig {
            method: AuthMethod::Development,
            client_id: None,
            scope: "https://test.scope".to_string(),
        };

        // May fail if Azure CLI not installed - both outcomes are valid
        let result = Auth::new(&auth_config);
        match result {
            Ok(auth) => assert_eq!(auth.scope, "https://test.scope"),
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

        let auth = Auth::from_credential(credential, "scope".to_string());

        let token = auth.get_token_internal().await.unwrap();
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

        let auth = Auth::from_credential(credential, "scope".to_string());

        // Each call to get_token_internal should call the credential
        let _ = auth.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        let _ = auth.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        let _ = auth.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_get_token_internal_returns_cloned_tokens() {
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            Arc::new(AtomicUsize::new(0)),
        );

        let auth = Auth::from_credential(credential, "scope".to_string());

        let token1 = auth.get_token_internal().await.unwrap();
        let token2 = auth.get_token_internal().await.unwrap();

        // Same value from both calls
        assert_eq!(token1.token.secret(), token2.token.secret());
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

        let cred = FailingCredential;
        let credential: Arc<dyn TokenCredential> = Arc::new(cred);
        let auth = Auth::from_credential(credential, "scope".to_string());

        let result = auth.get_token_internal().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Auth {
                kind: super::super::error::AuthErrorKind::TokenAcquisition,
                ..
            } => {}
            err => panic!("Expected Auth token acquisition error, got: {:?}", err),
        }
    }

    // ==================== Clone Behavior Tests ====================

    #[tokio::test]
    async fn test_cloned_auth_shares_credential() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let auth1 = Auth::from_credential(credential, "scope".to_string());
        let auth2 = auth1.clone();

        // Both auth instances share the same credential
        let _ = auth1.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        let _ = auth2.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
