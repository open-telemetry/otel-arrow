// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use azure_core::credentials::{AccessToken, TokenCredential};
use azure_core::time::OffsetDateTime;
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId,
};
use std::sync::Arc;

use crate::experimental::azure_monitor_exporter::config::{AuthConfig, AuthMethod};

#[derive(Clone)]
pub struct Auth {
    credential: Arc<dyn TokenCredential>,
    scope: String,
    cached_token: AccessToken,
}

impl Auth {
    pub fn new(auth_config: &AuthConfig) -> Result<Self, String> {
        let credential = Self::create_credential(auth_config)?;

        Ok(Self {
            credential,
            scope: auth_config.scope.clone(),
            cached_token: AccessToken {
                token: "".into(),
                expires_on: OffsetDateTime::now_utc(),
            },
        })
    }

    pub fn from_credential(credential: Arc<dyn TokenCredential>, scope: String) -> Self {
        Self {
            credential,
            scope,
            cached_token: AccessToken {
                token: "".into(),
                expires_on: OffsetDateTime::now_utc(),
            },
        }
    }

    pub async fn get_token(&mut self) -> Result<AccessToken, String> {
        if self.cached_token.expires_on
            > OffsetDateTime::now_utc() + azure_core::time::Duration::minutes(5)
        {
            return Ok(self.cached_token.clone());
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
        self.cached_token = token_response.clone();

        Ok(token_response)
    }

    pub fn invalidate_token(&mut self) {
        self.cached_token = AccessToken {
            token: "".into(),
            expires_on: OffsetDateTime::now_utc(),
        };
    }

    #[allow(clippy::print_stdout)]
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

    #[tokio::test]
    async fn test_get_token_caches_result() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let mut auth = Auth::from_credential(credential, "scope".to_string());

        // First call should hit the credential
        let token1 = auth.get_token().await.unwrap();
        assert_eq!(token1.token.secret(), "test_token");
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Second call should use cache (token valid for 60 mins, buffer is 5 mins)
        let token2 = auth.get_token().await.unwrap();
        assert_eq!(token2.token.secret(), "test_token");
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_get_token_refreshes_when_expired() {
        let call_count = Arc::new(Mutex::new(0));
        // Token expires in 4 minutes (less than 5 min buffer)
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(4),
            call_count: call_count.clone(),
        });

        let mut auth = Auth::from_credential(credential, "scope".to_string());

        // First call
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Second call - should refresh because 4 mins < 5 mins buffer
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_invalidate_token() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let mut auth = Auth::from_credential(credential, "scope".to_string());

        // First call
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Invalidate
        auth.invalidate_token();

        // Should refresh
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[test]
    fn test_new_with_managed_identity() {
        let auth_config = AuthConfig {
            method: AuthMethod::ManagedIdentity,
            client_id: Some("test-client-id".to_string()),
            scope: "https://test.scope".to_string(),
        };

        let auth = Auth::new(&auth_config);
        assert!(auth.is_ok());
        let auth = auth.unwrap();
        assert_eq!(auth.scope, "https://test.scope");
        assert_eq!(auth.cached_token.token.secret(), "");
    }

    #[test]
    fn test_new_with_system_assigned_managed_identity() {
        let auth_config = AuthConfig {
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            scope: "https://test.scope".to_string(),
        };

        let auth = Auth::new(&auth_config);
        assert!(auth.is_ok());
        let auth = auth.unwrap();
        assert_eq!(auth.scope, "https://test.scope");
        assert_eq!(auth.cached_token.token.secret(), "");
    }

    #[test]
    fn test_new_with_development_auth() {
        let auth_config = AuthConfig {
            method: AuthMethod::Development,
            client_id: None,
            scope: "https://test.scope".to_string(),
        };

        let auth = Auth::new(&auth_config);
        // This might fail if Azure CLI is not installed, but we can still test the code path
        match auth {
            Ok(auth) => {
                assert_eq!(auth.scope, "https://test.scope");
                assert_eq!(auth.cached_token.token.secret(), "");
            }
            Err(err) => {
                // Expected if Azure CLI/Azure Developer CLI is not installed
                assert!(err.contains("Failed to create developer tools credential"));
            }
        }
    }

    #[tokio::test]
    async fn test_get_token_with_exactly_5_minute_buffer() {
        let call_count = Arc::new(Mutex::new(0));
        // Token expires in exactly 5 minutes (at the buffer boundary)
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(5),
            call_count: call_count.clone(),
        });

        let mut auth = Auth::from_credential(credential, "scope".to_string());

        // First call
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Second call - should not use cache because expires_on is not > now + 5 minutes
        let _ = auth.get_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_get_token_error_handling() {
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

        let credential = Arc::new(FailingCredential);
        let mut auth = Auth::from_credential(credential, "scope".to_string());

        let result = auth.get_token().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to get token"));
    }

    #[tokio::test]
    async fn test_cached_token_is_cloned() {
        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let mut auth = Auth::from_credential(credential, "scope".to_string());

        // Get token twice and verify they are different instances (cloned)
        let token1 = auth.get_token().await.unwrap();
        let token2 = auth.get_token().await.unwrap();

        // Both should have same value
        assert_eq!(token1.token.secret(), token2.token.secret());
        // But only one call should have been made
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_from_credential_initializes_with_expired_token() {
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: Arc::new(Mutex::new(0)),
        });

        let auth = Auth::from_credential(credential, "test_scope".to_string());

        // Check that initial cached token is expired
        assert_eq!(auth.cached_token.token.secret(), "");
        assert!(auth.cached_token.expires_on <= OffsetDateTime::now_utc());
        assert_eq!(auth.scope, "test_scope");
    }
}
