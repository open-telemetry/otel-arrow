// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use object_store::{CredentialProvider, azure::AzureCredential};
use otap_df_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider;
use tokio::sync::Mutex;

/// Bridges the engine's bearer token capability to object_store Azure credentials.
pub struct AzureTokenCredentialProvider {
    token_provider: Mutex<Box<dyn BearerTokenProvider>>,
    state: Mutex<Option<TokenProviderState>>,
}

#[derive(Debug)]
struct TokenProviderState {
    current_token: String,
    current_object_store_cred: Arc<AzureCredential>,
}

impl std::fmt::Debug for AzureTokenCredentialProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AzureTokenCredentialProvider")
            .finish_non_exhaustive()
    }
}

impl AzureTokenCredentialProvider {
    /// Create a provider backed by a resolved bearer token capability.
    #[must_use]
    pub fn new(token_provider: Box<dyn BearerTokenProvider>) -> Self {
        Self {
            token_provider: Mutex::new(token_provider),
            state: Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl CredentialProvider for AzureTokenCredentialProvider {
    type Credential = AzureCredential;

    /// Get an [AzureCredential] from the extension-managed token cache.
    async fn get_credential(&self) -> object_store::Result<Arc<AzureCredential>> {
        let token = self
            .token_provider
            .lock()
            .await
            .get_token()
            .await
            .map_err(|e| object_store::Error::Generic {
                store: "Azure",
                source: Box::new(e),
            })?;
        let token = token.expose_token();

        let mut state = self.state.lock().await;
        if let Some(state) = state.as_ref()
            && state.current_token == token
        {
            return Ok(state.current_object_store_cred.clone());
        }

        let credential = Arc::new(AzureCredential::BearerToken(token.to_owned()));
        *state = Some(TokenProviderState {
            current_token: token.to_owned(),
            current_object_store_cred: credential.clone(),
        });
        Ok(credential)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use otap_df_engine::capability::auth::BearerToken;
    use otap_df_engine::capability::auth::bearer_token_provider::TokenStream;
    use otap_df_engine::capability::{CapabilityError, CapabilityErrorSource};

    /// Scenario: The bound capability cannot produce a token.
    /// Guarantees: The failure is reported to object_store instead of surfacing an unauthenticated request.
    #[tokio::test]
    async fn get_credential_reports_a_provider_failure() {
        let provider = setup_provider(vec![]);

        let error = provider
            .get_credential()
            .await
            .expect_err("a provider failure must fail credential acquisition");

        assert!(matches!(
            error,
            object_store::Error::Generic { store: "Azure", .. }
        ));
    }

    /// Scenario: The capability fails after a credential was already cached.
    /// Guarantees: The stale cached credential is not replayed once the provider stops issuing tokens.
    #[tokio::test]
    async fn cached_credential_is_not_replayed_after_a_provider_failure() {
        let provider = setup_provider(vec!["token1".to_string()]);
        let _ = provider
            .get_credential()
            .await
            .expect("first acquisition succeeds");

        assert!(provider.get_credential().await.is_err());
    }

    /// Scenario: The credential provider is rendered in diagnostics output.
    /// Guarantees: Rendering never discloses the bearer token it holds.
    #[test]
    fn debug_rendering_withholds_token_material() {
        let provider = setup_provider(vec!["super-secret-token".to_string()]);

        let rendered = format!("{provider:?}");

        assert!(rendered.contains("AzureTokenCredentialProvider"));
        assert!(!rendered.contains("super-secret-token"));
    }

    /// Scenario: Azure storage is built with a bound bearer token capability, with and without retry.
    /// Guarantees: The capability-backed credential path constructs a usable store in both cases.
    #[test]
    fn azure_storage_builds_when_a_token_provider_is_supplied() {
        crate::crypto::ensure_crypto_provider();
        let storage = crate::object_store::StorageType::Azure {
            base_uri: "https://mystorageaccount.blob.core.windows.net/container/telemetry"
                .to_string(),
        };

        let store = crate::object_store::from_storage_type_with_retry_and_token_provider(
            &storage,
            None,
            Some(Box::new(TestTokenProvider::new(vec!["token1".to_string()]))),
        );
        assert!(store.is_ok(), "expected a store, got {store:?}");

        let retry = crate::object_store::RetryOptions {
            max_retries: 3,
            init_backoff: std::time::Duration::from_millis(100),
            max_backoff: std::time::Duration::from_secs(5),
            backoff_base: 2.0,
            retry_timeout: std::time::Duration::from_secs(30),
        };
        let store_with_retry = crate::object_store::from_storage_type_with_retry_and_token_provider(
            &storage,
            Some(&retry),
            Some(Box::new(TestTokenProvider::new(vec!["token1".to_string()]))),
        );
        assert!(
            store_with_retry.is_ok(),
            "expected a store, got {store_with_retry:?}"
        );
    }

    /// Scenario: The capability returns repeated and refreshed token values.
    /// Guarantees: The bridge reuses credentials for equal tokens and replaces them on refresh.
    #[tokio::test]
    async fn test_same_different_same() {
        let provider = setup_provider(vec![
            "token1".to_string(),
            "token1".to_string(),
            "token2".to_string(),
            "token2".to_string(),
        ]);
        let cred1 = provider.get_credential().await.unwrap();
        let cred2 = provider.get_credential().await.unwrap();
        assert!(Arc::ptr_eq(&cred1, &cred2));

        let cred3 = provider.get_credential().await.unwrap();
        let cred4 = provider.get_credential().await.unwrap();

        assert!(!Arc::ptr_eq(&cred1, &cred3));
        assert!(Arc::ptr_eq(&cred3, &cred4));
    }

    /// Scenario: A token value returns after a different token was observed.
    /// Guarantees: Credential identity follows refresh events rather than token text history.
    #[tokio::test]
    async fn test_same_text_diff_token() {
        let provider = setup_provider(vec![
            "token1".to_string(),
            "token2".to_string(),
            "token1".to_string(),
        ]);
        let cred1 = provider.get_credential().await.unwrap();
        let _ = provider.get_credential().await.unwrap();
        let cred3 = provider.get_credential().await.unwrap();
        assert!(!Arc::ptr_eq(&cred1, &cred3));
    }

    /// Scenario: Consecutive capability reads return the same token.
    /// Guarantees: The bridge avoids reallocating the object-store credential.
    #[tokio::test]
    async fn test_same_token() {
        let provider = setup_provider(vec!["token1".to_string(), "token1".to_string()]);
        let cred1 = provider.get_credential().await.unwrap();
        let cred2 = provider.get_credential().await.unwrap();
        assert!(Arc::ptr_eq(&cred1, &cred2));
    }

    fn setup_provider(tokens: Vec<String>) -> AzureTokenCredentialProvider {
        AzureTokenCredentialProvider::new(Box::new(TestTokenProvider::new(tokens)))
    }

    #[derive(Debug)]
    struct TestTokenProvider {
        tokens: Mutex<Vec<String>>,
    }

    impl TestTokenProvider {
        fn new(mut tokens: Vec<String>) -> Self {
            // Reverse so popping from the end gives the correct order.
            tokens.reverse();
            Self {
                tokens: Mutex::new(tokens),
            }
        }
    }

    #[async_trait::async_trait]
    impl BearerTokenProvider for TestTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            match self.tokens.lock().await.pop() {
                Some(token) => Ok(BearerToken::without_expiry(token)),
                None => Err(CapabilityErrorSource::<
                    otap_df_engine::capability::auth::bearer_token_provider::BearerTokenProvider,
                >::new("test_extension".into())
                .error("no token available")),
            }
        }

        fn token_stream(&self) -> TokenStream {
            Box::pin(futures::stream::empty())
        }
    }
}
