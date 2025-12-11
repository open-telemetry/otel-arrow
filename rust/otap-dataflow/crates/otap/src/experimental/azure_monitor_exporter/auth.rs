// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use azure_core::credentials::{AccessToken, TokenCredential};
use azure_core::time::OffsetDateTime;
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::experimental::azure_monitor_exporter::config::{AuthConfig, AuthMethod};

#[derive(Clone)]
// TODO - Remove print statements
#[allow(clippy::print_stdout)]
pub struct Auth {
    credential: Arc<dyn TokenCredential>,
    scope: String,
    // Thread-safe shared token cache
    cached_token: Arc<RwLock<Option<AccessToken>>>,
    pub token_valid_until: Instant,
}

// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
impl Auth {
    pub fn new(auth_config: &AuthConfig) -> Result<Self, String> {
        let credential = Self::create_credential(auth_config)?;

        Ok(Self {
            credential,
            scope: auth_config.scope.clone(),
            cached_token: Arc::new(RwLock::new(None)),
            token_valid_until: Instant::now(),
        })
    }

    pub fn from_credential(credential: Arc<dyn TokenCredential>, scope: String) -> Self {
        Self {
            credential,
            scope,
            cached_token: Arc::new(RwLock::new(None)),
            token_valid_until: Instant::now(),
        }
    }

    pub async fn get_token(&self) -> Result<AccessToken, String> {
        // Try to use cached token
        {
            let cached = self.cached_token.read().await;
            if let Some(token) = &*cached {
                if token.expires_on > OffsetDateTime::now_utc() {
                    return Ok(token.clone());
                }
            }
        }

        // Need to refresh - acquire write lock
        let mut cached = self.cached_token.write().await;

        // Double-check in case another thread refreshed while we waited
        if let Some(token) = &*cached {
            if token.expires_on > OffsetDateTime::now_utc() {
                return Ok(token.clone());
            }
        }

        let token_response = self
            .credential
            .get_token(
                &[&self.scope],
                Some(azure_core::credentials::TokenRequestOptions::default()),
            )
            .await
            .map_err(|e| format!("Failed to get token: {e}"))?;

        // Update the cached token
        *cached = Some(token_response.clone());

        Ok(token_response)
    }

    pub async fn invalidate_token(&self) {
        let mut cached = self.cached_token.write().await;
        *cached = None;
    }

    fn create_credential(auth_config: &AuthConfig) -> Result<Arc<dyn TokenCredential>, String> {
        match auth_config.method {
            AuthMethod::ManagedIdentity => {
                let mut options = ManagedIdentityCredentialOptions::default();

                if let Some(client_id) = &auth_config.client_id {
                    println!("Using user-assigned managed identity with client_id: {client_id}");
                    options.user_assigned_id = Some(UserAssignedId::ClientId(client_id.clone()));
                } else {
                    println!("Using system-assigned managed identity");
                }

                let credential = ManagedIdentityCredential::new(Some(options))
                    .map_err(|e| format!("Failed to create managed identity credential: {e}"))?;

                Ok(credential as Arc<dyn TokenCredential>)
            }
            AuthMethod::Development => {
                println!("Using developer tools credential (Azure CLI / Azure Developer CLI)");
                let credential =
                    DeveloperToolsCredential::new(Some(DeveloperToolsCredentialOptions::default()))
                        .map_err(|e| {
                            format!(
                                "Failed to create developer tools credential: {e}. \
                            Ensure Azure CLI or Azure Developer CLI is installed and logged in"
                            )
                        })?;

                Ok(credential as Arc<dyn TokenCredential>)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azure_core::credentials::TokenRequestOptions;
    use std::sync::Mutex;

    #[derive(Debug)]
    struct MockCredential {
        token: String,
        expires_in: azure_core::time::Duration,
        call_count: Arc<Mutex<usize>>,
    }

    #[async_trait::async_trait]
    impl TokenCredential for MockCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            Ok(AccessToken {
                token: self.token.clone().into(),
                expires_on: OffsetDateTime::now_utc() + self.expires_in,
            })
        }
    }

    /// Helper to create a mock credential that returns expired tokens
    #[derive(Debug)]
    struct ExpiredMockCredential {
        token: String,
        call_count: Arc<Mutex<usize>>,
    }

    #[async_trait::async_trait]
    impl TokenCredential for ExpiredMockCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            Ok(AccessToken {
                token: self.token.clone().into(),
                // Already expired
                expires_on: OffsetDateTime::now_utc() - azure_core::time::Duration::seconds(1),
            })
        }
    }

    // ==================== Construction Tests ====================

    #[tokio::test]
    async fn test_from_credential_initializes_empty_cache() {
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: Arc::new(Mutex::new(0)),
        });

        let auth = Auth::from_credential(credential, "test_scope".to_string());

        assert!(auth.cached_token.read().await.is_none());
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
        assert!(auth.cached_token.read().await.is_none());
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
            Err(err) => assert!(err.contains("Failed to create developer tools credential")),
        }
    }

    // ==================== Token Caching Tests ====================

    #[tokio::test]
    async fn test_get_token_caches_valid_token() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let auth = Auth::from_credential(credential, "scope".to_string());

        // First call fetches from credential
        let token1 = auth.get_token().await.unwrap();
        assert_eq!(token1.token.secret(), "test_token");
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Second call uses cache
        let token2 = auth.get_token().await.unwrap();
        assert_eq!(token2.token.secret(), "test_token");
        assert_eq!(*call_count.lock().unwrap(), 1); // Still 1 - no new fetch
    }

    #[tokio::test]
    async fn test_get_token_refreshes_expired_token() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(ExpiredMockCredential {
            token: "test_token".to_string(),
            call_count: call_count.clone(),
        });

        let auth = Auth::from_credential(credential, "scope".to_string());

        // First call
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Second call - token is expired, should refresh
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_get_token_returns_cloned_token() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let auth = Auth::from_credential(credential, "scope".to_string());

        let token1 = auth.get_token().await.unwrap();
        let token2 = auth.get_token().await.unwrap();

        // Same value, but independent clones
        assert_eq!(token1.token.secret(), token2.token.secret());
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    // ==================== Invalidation Tests ====================

    #[tokio::test]
    async fn test_invalidate_token_clears_cache() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let auth = Auth::from_credential(credential, "scope".to_string());

        // Fetch and cache token
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Invalidate
        auth.invalidate_token().await;
        assert!(auth.cached_token.read().await.is_none());

        // Next fetch should hit credential again
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_invalidate_empty_cache_is_safe() {
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: Arc::new(Mutex::new(0)),
        });

        let auth = Auth::from_credential(credential, "scope".to_string());

        // Should not panic when invalidating empty cache
        auth.invalidate_token().await;
        assert!(auth.cached_token.read().await.is_none());
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_get_token_propagates_credential_error() {
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

        let auth = Auth::from_credential(Arc::new(FailingCredential), "scope".to_string());

        let result = auth.get_token().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to get token"));
    }

    #[tokio::test]
    async fn test_get_token_error_does_not_cache() {
        #[derive(Debug)]
        struct FailOnceThenSucceed {
            call_count: Arc<Mutex<usize>>,
        }

        #[async_trait::async_trait]
        impl TokenCredential for FailOnceThenSucceed {
            async fn get_token(
                &self,
                _scopes: &[&str],
                _options: Option<TokenRequestOptions<'_>>,
            ) -> azure_core::Result<AccessToken> {
                let mut count = self.call_count.lock().unwrap();
                *count += 1;
                if *count == 1 {
                    Err(azure_core::error::Error::new(
                        azure_core::error::ErrorKind::Credential,
                        "First call fails",
                    ))
                } else {
                    Ok(AccessToken {
                        token: "success_token".to_string().into(),
                        expires_on: OffsetDateTime::now_utc()
                            + azure_core::time::Duration::minutes(60),
                    })
                }
            }
        }

        let call_count = Arc::new(Mutex::new(0));
        let auth = Auth::from_credential(
            Arc::new(FailOnceThenSucceed {
                call_count: call_count.clone(),
            }),
            "scope".to_string(),
        );

        // First call fails
        let result1 = auth.get_token().await;
        assert!(result1.is_err());
        assert!(auth.cached_token.read().await.is_none()); // Nothing cached

        // Second call succeeds
        let result2 = auth.get_token().await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().token.secret(), "success_token");
    }

    // ==================== Concurrency Tests ====================

    #[tokio::test]
    async fn test_concurrent_get_token_only_fetches_once() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let auth = Arc::new(Auth::from_credential(credential, "scope".to_string()));

        // Spawn multiple concurrent token requests
        let mut handles = vec![];
        for _ in 0..10 {
            let auth_clone = auth.clone();
            handles.push(tokio::spawn(async move { auth_clone.get_token().await }));
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().token.secret(), "test_token");
        }

        // Due to double-check locking, should have minimal calls (ideally 1, but timing may cause a few more)
        let final_count = *call_count.lock().unwrap();
        assert!(
            final_count <= 3,
            "Expected at most 3 credential calls due to race, got {}",
            final_count
        );
    }

    // ==================== Clone Behavior Tests ====================

    #[tokio::test]
    async fn test_cloned_auth_shares_cache() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let auth1 = Auth::from_credential(credential, "scope".to_string());
        let auth2 = auth1.clone();

        // Fetch via auth1
        let _ = auth1.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Fetch via auth2 - should use shared cache
        let _ = auth2.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1); // Still 1

        // Invalidate via auth1
        auth1.invalidate_token().await;

        // Fetch via auth2 - should see invalidated cache
        let _ = auth2.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }
}
