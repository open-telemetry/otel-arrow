use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use azure_core::credentials::{AccessToken, TokenCredential};
use azure_identity::AzureCliCredential;
use object_store::{CredentialProvider, azure::AzureCredential};
use serde::{Deserialize, Serialize};

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

        let mut guard = self.state.lock().expect("Lock is not poisoned.");
        let state = guard.as_mut().expect("State is initialized.");
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

pub const DEFAULT_STORAGE_SCOPE: &str = "https://storage.azure.com/.default";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AuthMethod {
    AzureCli {
        /// Additional tenants that the credential should be allowed to
        /// authenticate in
        #[serde(default)]
        additionally_allowed_tenants: Vec<String>,

        /// Set this to specify a subscription other than the currently active
        /// one in the Azure cli.
        subscription: Option<String>,

        /// Tenant ID to authenticate in. Defaults to the value of the azure
        /// cli's default tenant.
        tenant_id: Option<String>,
    },
    ManagedIdentity {
        /// User assigned identity to use when authenticating, otherwise the
        /// system assigned identity will be used if available.
        user_assigned_id: Option<UserAssignedId>,
    },
    WorkloadIdentity {
        /// Client ID of the Entra identity. Defaults to the value of the
        /// `AZURE_CLIENT_ID` environment variable.
        client_id: Option<String>,

        /// Tenant ID of the Entra identity. Defaults to the value of the
        /// `AZURE_TENANT_ID` environment variable.
        tenant_id: Option<String>,

        /// Path to the token file to read the assertion from. Defaults to the
        /// value of the AZURE_FEDERATED_TOKEN_FILE environment variable.
        token_file_path: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
// TODO: Test how this deserializes
pub enum UserAssignedId {
    /// The client ID of a user-assigned identity
    ClientId(String),
    /// The object or principal ID of a user-assigned identity
    ObjectId(String),
    /// The Azure resource ID of a user-assigned identity
    ResourceId(String),
}

impl From<UserAssignedId> for azure_identity::UserAssignedId {
    fn from(value: UserAssignedId) -> Self {
        match value {
            UserAssignedId::ClientId(id) => azure_identity::UserAssignedId::ClientId(id),
            UserAssignedId::ObjectId(id) => azure_identity::UserAssignedId::ObjectId(id),
            UserAssignedId::ResourceId(id) => azure_identity::UserAssignedId::ResourceId(id),
        }
    }
}

pub fn from_auth_method(value: AuthMethod) -> Result<Arc<dyn TokenCredential>, azure_core::Error> {
    match value {
        AuthMethod::AzureCli {
            additionally_allowed_tenants,
            subscription,
            tenant_id,
        } => {
            let options = Some(azure_identity::AzureCliCredentialOptions {
                additionally_allowed_tenants: additionally_allowed_tenants,
                subscription: subscription,
                tenant_id: tenant_id,
                ..Default::default()
            });
            Ok(AzureCliCredential::new(options)?)
        }
        AuthMethod::ManagedIdentity { user_assigned_id } => {
            let options = azure_identity::ManagedIdentityCredentialOptions {
                user_assigned_id: user_assigned_id.map(|u| u.into()),
                ..Default::default()
            };
            Ok(azure_identity::ManagedIdentityCredential::new(Some(
                options,
            ))?)
        }
        AuthMethod::WorkloadIdentity {
            client_id,
            tenant_id,
            token_file_path,
        } => {
            let options = azure_identity::WorkloadIdentityCredentialOptions {
                client_id,
                tenant_id,
                token_file_path,
                ..Default::default()
            };
            Ok(azure_identity::WorkloadIdentityCredential::new(Some(
                options,
            ))?)
        }
    }
}
