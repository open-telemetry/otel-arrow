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
        if self.cached_token.expires_on > OffsetDateTime::now_utc() + azure_core::time::Duration::minutes(5)
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
