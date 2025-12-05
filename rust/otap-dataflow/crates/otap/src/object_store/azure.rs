// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use azure_core::credentials::{AccessToken, TokenCredential};
use object_store::{CredentialProvider, azure::AzureCredential};
use std::sync::Mutex;

/// The default resource storage scope to request tokens for. This is
/// used in public cloud, but may need to be overridden for other clouds
/// such as Azure China or Azure Government.
///
/// See: <https://learn.microsoft.com/en-us/azure/storage/blobs/authorize-access-azure-active-directory#microsoft-authentication-library-msal>
pub const DEFAULT_STORAGE_SCOPE: &str = "https://storage.azure.com/.default";

/// [CredentialProvider] implementation that works with a [TokenCredential]
/// from the azure SDK.
///
/// object_store provides their own implementations of many credentials like
/// ManagedIdentityCredential or WorkloadIdentityCredential, however those
/// implementations are limited in functionality and do not support all options
/// like overriding storage scope. See [DEFAULT_STORAGE_SCOPE].
#[derive(Debug)]
pub struct AzureTokenCredentialProvider {
    token_cred: Arc<dyn TokenCredential>,
    storage_scope: String,
    state: Mutex<Option<TokenProviderState>>,
}

#[derive(Debug)]
struct TokenProviderState {
    /// The last obtained token from [Self::token_cred]
    current_access_token: AccessToken,
    /// The object store representation of [Self::current_access_token]
    current_object_store_cred: Arc<AzureCredential>,
}

impl AzureTokenCredentialProvider {
    /// Create a new provider based on the given [TokenCredential].
    /// If you are operating in an azure cloud other than public
    /// (the normal one), you may want to provide a different scope.
    ///
    /// See [DEFAULT_STORAGE_SCOPE].
    pub fn new(cred: Arc<dyn TokenCredential>, scope: Option<String>) -> Self {
        Self {
            token_cred: cred,
            storage_scope: scope.unwrap_or(DEFAULT_STORAGE_SCOPE.to_string()),
            state: Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl CredentialProvider for AzureTokenCredentialProvider {
    type Credential = AzureCredential;

    /// Get an [AzureCredential] based on the  underlying [TokenCredential].
    ///
    /// This has a bunch of machinery because [AzureCredential] and
    /// [TokenCredential] have different underlying representations.
    ///
    /// [TokenCredentials] typically have the responsibility of caching and
    /// refreshing tokens themselves and are based on `Cow<str>`, so they're
    /// cheapyly clonable.
    ///
    /// [AzureCredential::BearerToken] on the other hand is just a String and
    /// needs to be wrapped in Arc (hence the trait signature).
    ///
    /// The goal here is to delegate caching to the underlying [TokenCredential]
    /// while avoiding allocating a String on every call to this function by
    /// caching the [AzureCredential] representation of the current token and
    /// updating it as needed.
    async fn get_credential(&self) -> object_store::Result<Arc<AzureCredential>> {
        // First, get a token from the underlying credential, this is "free" if
        // there's a valid cached token.
        let maybe_new_token = self
            .token_cred
            .get_token(&[&self.storage_scope], None)
            .await
            .map_err(|e| object_store::Error::Generic {
                store: "Azure",
                source: Box::new(e),
            })?;

        // If state is not initialized, then this is the first time we're called.
        // Initialize the state and return early.
        let mut state = self.state.lock().expect("Lock is not poisoned.");
        if state.is_none() {
            let object_store_cred = Arc::new(AzureCredential::BearerToken(
                maybe_new_token.token.secret().to_owned(),
            ));

            *state = Some(TokenProviderState {
                current_access_token: maybe_new_token,
                current_object_store_cred: object_store_cred.clone(),
            });

            return Ok(object_store_cred);
        }

        // If the secret from the currently cached token matches the new one,
        // then we can reuse the cached [AzureCredential]. Otherwise we update.
        let state = state.as_mut().expect("State is initialized.");
        let current_token = &state.current_access_token;
        let cred = if current_token.token.secret() == maybe_new_token.token.secret() {
            state.current_object_store_cred.clone()
        } else {
            let object_store_cred = Arc::new(AzureCredential::BearerToken(
                maybe_new_token.token.secret().to_owned(),
            ));
            state.current_access_token = maybe_new_token;
            state.current_object_store_cred = object_store_cred.clone();
            object_store_cred
        };

        Ok(cred)
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use time::OffsetDateTime;

    use super::*;

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

    #[tokio::test]
    async fn test_same_token() {
        let provider = setup_provider(vec!["token1".to_string(), "token1".to_string()]);
        let cred1 = provider.get_credential().await.unwrap();
        let cred2 = provider.get_credential().await.unwrap();
        assert!(Arc::ptr_eq(&cred1, &cred2));
    }

    fn setup_provider(tokens: Vec<String>) -> AzureTokenCredentialProvider {
        let cred = Arc::new(TestTokenCredential::new(tokens));
        AzureTokenCredentialProvider::new(cred, None)
    }

    #[derive(Debug)]
    struct TestTokenCredential {
        tokens: Mutex<Vec<String>>,
    }

    impl TestTokenCredential {
        fn new(mut tokens: Vec<String>) -> Self {
            // Reverse so popping from the end gives the correct order.
            tokens.reverse();
            Self {
                tokens: Mutex::new(tokens),
            }
        }
    }

    #[async_trait::async_trait]
    impl TokenCredential for TestTokenCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<azure_core::credentials::TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            let token = self.tokens.lock().unwrap().pop().unwrap();
            Ok(AccessToken::new(
                token,
                OffsetDateTime::now_utc() + Duration::from_secs(3600),
            ))
        }
    }
}
