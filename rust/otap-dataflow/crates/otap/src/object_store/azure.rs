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
    use otap_df_engine::capability::CapabilityError;
    use otap_df_engine::capability::auth::BearerToken;
    use otap_df_engine::capability::auth::bearer_token_provider::TokenStream;

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
            let token = self.tokens.lock().await.pop().unwrap();
            Ok(BearerToken::without_expiry(token))
        }

        fn token_stream(&self) -> TokenStream {
            Box::pin(futures::stream::empty())
        }
    }
}
