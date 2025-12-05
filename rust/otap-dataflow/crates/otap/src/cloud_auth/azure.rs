// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{path::PathBuf, sync::Arc};

use azure_core::credentials::TokenCredential;
use azure_identity::AzureCliCredential;
use serde::{Deserialize, Serialize};

/// Azure authentication methods. This can be leveraged in component
/// configuration objects for a consistent way to specify Azure auth information.
/// The next step here may be to add an equivalent to the Go collector's auth
/// extensions rather thatn borrow this across component configs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMethod {
    /// See [azure_identity::AzureCliCredential].
    AzureCli {
        /// Additional tenants that the credential should be allowed to
        /// authenticate in.
        #[serde(default)]
        additionally_allowed_tenants: Vec<String>,

        /// Set this to specify a subscription other than the currently active
        /// one in the Azure cli.
        subscription: Option<String>,

        /// Tenant ID to authenticate in. Defaults to the value of the azure
        /// cli's default tenant.
        tenant_id: Option<String>,
    },
    /// See [azure_identity::ManagedIdentityCredential].
    ManagedIdentity {
        /// User assigned identity to use when authenticating, otherwise the
        /// system assigned identity will be used if available.
        user_assigned_id: Option<UserAssignedId>,
    },
    /// See [azure_identity::WorkloadIdentityCredential].
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

/// Equivalent of [azure_identity::UserAssignedId]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

/// Create a [TokenCredential] from the given [AuthMethod].
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

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_azure_cli_minimal() {
        let json = json!({
            "type": "azure_cli"
        })
        .to_string();
        let expected = AuthMethod::AzureCli {
            additionally_allowed_tenants: vec![],
            subscription: None,
            tenant_id: None,
        };
        test_deserialize(&json, expected);
    }

    fn test_deserialize(json: &str, expected: AuthMethod) {
        let deserialized: AuthMethod =
            serde_json::from_str(json).expect("Failed to deserialize AuthMethod");
        assert_eq!(deserialized, expected);
    }
}
