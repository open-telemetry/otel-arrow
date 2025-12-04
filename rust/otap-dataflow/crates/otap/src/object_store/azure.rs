// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use azure_core::credentials::{AccessToken, TokenCredential};
use object_store::{CredentialProvider, azure::AzureCredential};
use std::sync::Mutex;

/// TODO(jakedern): Docs
pub const DEFAULT_STORAGE_SCOPE: &str = "https://storage.azure.com/.default";

/// TODO(jakedern): Docs
#[derive(Debug)]
pub struct AzureTokenCredentialProvider {
    token_cred: Arc<dyn TokenCredential>,
    storage_scope: String,

    state: Mutex<Option<TokenProviderState>>,
}

#[derive(Debug)]
struct TokenProviderState {
    /// The last obtained object store credential
    current_access_token: AccessToken,
    /// The object store representation of [Self::current_access_token]
    current_object_store_cred: Arc<AzureCredential>,
}

impl AzureTokenCredentialProvider {
    /// TODO(jakedern): Docs
    pub fn new(cred: Arc<dyn TokenCredential>, scope: Option<String>) -> Self {
        Self {
            token_cred: cred,
            storage_scope: scope.unwrap_or(DEFAULT_STORAGE_SCOPE.to_string()),
            state: Mutex::new(None),
        }
    }
}

/// TODO: Decide whether this is worth doing.
#[async_trait::async_trait]
impl CredentialProvider for AzureTokenCredentialProvider {
    type Credential = AzureCredential;

    async fn get_credential(&self) -> object_store::Result<Arc<Self::Credential>> {
        let maybe_new_token = self
            .token_cred
            .get_token(&[&self.storage_scope], None)
            .await
            .map_err(|e| object_store::Error::Generic {
                store: "Azure",
                source: Box::new(e),
            })?;

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
