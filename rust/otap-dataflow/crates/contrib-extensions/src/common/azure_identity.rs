// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Protocol-neutral Azure identity configuration and credential construction.

use std::path::Path;
#[cfg(any(feature = "azure-key-vault-sasl-auth", test))]
use std::path::PathBuf;
use std::sync::Arc;

use azure_core::credentials::TokenCredential;
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId, WorkloadIdentityCredential,
    WorkloadIdentityCredentialOptions,
};
use serde::Deserialize;

/// Azure identity authentication flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// Azure Managed Identity (system- or user-assigned).
    #[serde(alias = "msi", alias = "managed_identity")]
    #[default]
    ManagedIdentity,
    /// Local developer tooling (Azure CLI / `azd`). Local development only.
    #[serde(alias = "dev", alias = "developer", alias = "cli")]
    Development,
    /// Workload Identity Federation (projected ServiceAccount token).
    #[serde(alias = "wif", alias = "workload_identity")]
    WorkloadIdentity,
}

impl AuthMethod {
    /// Returns a stable, human-readable name for the method.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AuthMethod::ManagedIdentity => "managed_identity",
            AuthMethod::Development => "development",
            AuthMethod::WorkloadIdentity => "workload_identity",
        }
    }
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Owned identity configuration for consumers that use a nested identity block.
#[cfg(any(feature = "azure-key-vault-sasl-auth", test))]
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityConfig {
    /// Authentication flow to use; defaults to Managed Identity.
    #[serde(default)]
    pub method: AuthMethod,
    /// User-assigned Managed Identity or workload application client ID.
    /// Omit for system-assigned Managed Identity or `AZURE_CLIENT_ID` fallback.
    #[serde(default)]
    pub client_id: Option<String>,
    /// Workload tenant ID; omit for `AZURE_TENANT_ID` fallback.
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Workload federated token file; omit for `AZURE_FEDERATED_TOKEN_FILE` fallback.
    #[serde(default)]
    pub token_file_path: Option<PathBuf>,
}

#[cfg(any(feature = "azure-key-vault-sasl-auth", test))]
impl IdentityConfig {
    /// Rejects inapplicable fields, explicitly blank IDs, and an empty token path.
    ///
    /// Omitted workload fields remain valid so the SDK can resolve environment
    /// defaults. Non-empty IDs are passed through without UUID parsing.
    pub fn validate(&self) -> Result<(), String> {
        let options = IdentityOptions::from(self);
        options.validate_applicable_fields()?;
        for (name, value) in [
            ("client_id", options.client_id),
            ("tenant_id", options.tenant_id),
        ] {
            if value.is_some_and(|value| value.trim().is_empty()) {
                return Err(format!("`{name}` must not be empty"));
            }
        }
        if options
            .token_file_path
            .is_some_and(|path| path.as_os_str().is_empty())
        {
            return Err("`token_file_path` must not be empty".to_string());
        }
        Ok(())
    }
}

/// Borrowed identity inputs, allowing consumers to retain their public config shape.
#[derive(Debug, Clone, Copy)]
pub struct IdentityOptions<'a> {
    /// Selected authentication flow.
    pub method: AuthMethod,
    /// Optional user-assigned Managed Identity or workload client ID.
    pub client_id: Option<&'a str>,
    /// Optional workload tenant ID.
    pub tenant_id: Option<&'a str>,
    /// Optional workload federated token path.
    pub token_file_path: Option<&'a Path>,
}

#[cfg(any(feature = "azure-key-vault-sasl-auth", test))]
impl<'a> From<&'a IdentityConfig> for IdentityOptions<'a> {
    fn from(config: &'a IdentityConfig) -> Self {
        Self {
            method: config.method,
            client_id: config.client_id.as_deref(),
            tenant_id: config.tenant_id.as_deref(),
            token_file_path: config.token_file_path.as_deref(),
        }
    }
}

impl IdentityOptions<'_> {
    /// Checks field applicability without newly rejecting legacy blank inputs.
    ///
    /// New consumers should use `IdentityConfig::validate` for strict checks.
    pub fn validate_applicable_fields(&self) -> Result<(), String> {
        if self.method != AuthMethod::WorkloadIdentity {
            if self.tenant_id.is_some() {
                return Err(format!(
                    "`tenant_id` is only valid for the `workload_identity` method, not `{}`",
                    self.method
                ));
            }
            if self.token_file_path.is_some() {
                return Err(format!(
                    "`token_file_path` is only valid for the `workload_identity` method, not `{}`",
                    self.method
                ));
            }
        }
        if self.method == AuthMethod::Development && self.client_id.is_some() {
            return Err("`client_id` is not valid for the `development` method".to_string());
        }
        Ok(())
    }
}

fn managed_identity_options(options: IdentityOptions<'_>) -> ManagedIdentityCredentialOptions {
    ManagedIdentityCredentialOptions {
        user_assigned_id: options
            .client_id
            .map(|id| UserAssignedId::ClientId(id.to_owned())),
        ..Default::default()
    }
}

fn workload_identity_options(options: IdentityOptions<'_>) -> WorkloadIdentityCredentialOptions {
    WorkloadIdentityCredentialOptions {
        client_id: options.client_id.map(str::to_owned),
        tenant_id: options.tenant_id.map(str::to_owned),
        token_file_path: options.token_file_path.map(Path::to_path_buf),
        ..Default::default()
    }
}

/// Constructs a credential without fetching tokens or wrapping SDK errors.
///
/// Validation is the caller's responsibility. The SDK owns the credential in an
/// `Arc`, which consumers can share with their background refresh tasks.
pub fn create_credential(
    options: IdentityOptions<'_>,
) -> azure_core::Result<Arc<dyn TokenCredential>> {
    // Azure credentials use a reqwest/rustls HTTP client, which requires a
    // process-wide crypto provider to be installed.
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    match options.method {
        AuthMethod::ManagedIdentity => Ok(ManagedIdentityCredential::new(Some(
            managed_identity_options(options),
        ))?),
        AuthMethod::Development => Ok(DeveloperToolsCredential::new(Some(
            DeveloperToolsCredentialOptions::default(),
        ))?),
        AuthMethod::WorkloadIdentity => Ok(WorkloadIdentityCredential::new(Some(
            workload_identity_options(options),
        ))?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "azure-identity-auth")]
    use crate::azure_identity_auth::config::Config;

    /// Scenario: An identity block is empty or selects any existing method alias.
    /// Guarantees: Managed Identity remains the default and all legacy aliases are accepted.
    #[test]
    fn defaults_and_aliases_are_preserved() {
        let config: IdentityConfig = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(config.method, AuthMethod::ManagedIdentity);
        assert!(config.client_id.is_none());
        assert!(config.tenant_id.is_none());
        assert!(config.token_file_path.is_none());
        assert!(config.validate().is_ok());
        for (aliases, method) in [
            (
                &["msi", "managed_identity", "managedidentity"][..],
                AuthMethod::ManagedIdentity,
            ),
            (
                &["dev", "developer", "cli", "development"][..],
                AuthMethod::Development,
            ),
            (
                &["wif", "workload_identity", "workloadidentity"][..],
                AuthMethod::WorkloadIdentity,
            ),
        ] {
            for alias in aliases {
                let config: IdentityConfig =
                    serde_json::from_value(serde_json::json!({ "method": alias })).unwrap();
                assert_eq!(config.method, method);
                assert!(config.validate().is_ok());
            }
        }
    }

    /// Scenario: An identity block contains a misspelled field or a protocol-specific scope.
    /// Guarantees: Unknown fields are rejected instead of being silently ignored.
    #[test]
    fn unknown_fields_are_rejected() {
        for value in [
            serde_json::json!({ "client": "id" }),
            serde_json::json!({ "scope": "https://vault.azure.net/.default" }),
        ] {
            assert!(serde_json::from_value::<IdentityConfig>(value).is_err());
        }
    }

    /// Scenario: Explicit identity IDs are blank or the workload token path is empty.
    /// Guarantees: Strict validation rejects blank values without rejecting omitted SDK defaults.
    #[test]
    fn strict_validation_rejects_empty_inputs() {
        for method in ["managed_identity", "workload_identity"] {
            for value in ["", " \t\n"] {
                let config: IdentityConfig = serde_json::from_value(serde_json::json!({
                    "method": method, "client_id": value,
                }))
                .unwrap();
                assert_eq!(
                    config.validate().unwrap_err(),
                    "`client_id` must not be empty"
                );
            }
        }
        for value in ["", " \t\n"] {
            let config: IdentityConfig = serde_json::from_value(serde_json::json!({
                "method": "workload_identity", "tenant_id": value,
            }))
            .unwrap();
            assert_eq!(
                config.validate().unwrap_err(),
                "`tenant_id` must not be empty"
            );
        }
        let config: IdentityConfig = serde_json::from_value(serde_json::json!({
            "method": "workload_identity", "token_file_path": "",
        }))
        .unwrap();
        assert_eq!(
            config.validate().unwrap_err(),
            "`token_file_path` must not be empty"
        );
        let config = IdentityConfig {
            method: AuthMethod::WorkloadIdentity,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    /// Scenario: A method receives an identity field that does not apply to it.
    /// Guarantees: Strict and legacy applicability checks preserve the same rejection messages.
    #[test]
    fn inapplicable_fields_are_rejected() {
        for (method, field, message) in [
            (
                "managed_identity",
                "tenant_id",
                "`tenant_id` is only valid for the `workload_identity` method, not `managed_identity`",
            ),
            (
                "managed_identity",
                "token_file_path",
                "`token_file_path` is only valid for the `workload_identity` method, not `managed_identity`",
            ),
            (
                "development",
                "tenant_id",
                "`tenant_id` is only valid for the `workload_identity` method, not `development`",
            ),
            (
                "development",
                "token_file_path",
                "`token_file_path` is only valid for the `workload_identity` method, not `development`",
            ),
            (
                "development",
                "client_id",
                "`client_id` is not valid for the `development` method",
            ),
        ] {
            let value = serde_json::json!({ "method": method, field: "value" });
            let identity: IdentityConfig = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(identity.validate().unwrap_err(), message);
            #[cfg(feature = "azure-identity-auth")]
            {
                let legacy: Config = serde_json::from_value(value).unwrap();
                assert_eq!(legacy.validate().unwrap_err(), message);
            }
        }
    }

    /// Scenario: Existing flat Azure Identity configs explicitly supply blank identity values.
    /// Guarantees: Legacy validation still accepts them and preserves scope and startup defaults.
    #[cfg(feature = "azure-identity-auth")]
    #[test]
    fn legacy_config_keeps_flat_shape_and_validation() {
        for value in [
            serde_json::json!({ "client_id": "" }),
            serde_json::json!({
                "method": "wif", "client_id": " \t", "tenant_id": "", "token_file_path": "",
            }),
        ] {
            let config: Config = serde_json::from_value(value).unwrap();
            assert!(config.validate().is_ok());
            assert_eq!(config.scope, "https://monitor.azure.com/.default");
            assert_eq!(config.startup_timeout, std::time::Duration::from_secs(30));
            let options = IdentityOptions::from(&config);
            assert_eq!(options.method, config.method);
            assert_eq!(options.client_id, config.client_id.as_deref());
            assert_eq!(options.tenant_id, config.tenant_id.as_deref());
            assert_eq!(options.token_file_path, config.token_file_path.as_deref());
        }
        assert!(serde_json::from_value::<Config>(serde_json::json!({ "identity": {} })).is_err());
    }

    /// Scenario: Managed Identity selects a client ID or leaves the identity system-assigned.
    /// Guarantees: SDK options use ClientId without additional ID parsing or another selector.
    #[test]
    fn managed_identity_client_id_is_forwarded() {
        let mut config = IdentityConfig::default();
        assert!(
            managed_identity_options(IdentityOptions::from(&config))
                .user_assigned_id
                .is_none()
        );
        config.client_id = Some("sdk-client-id".to_owned());
        assert!(config.validate().is_ok());
        assert!(matches!(
            managed_identity_options(IdentityOptions::from(&config)).user_assigned_id,
            Some(UserAssignedId::ClientId(id)) if id == "sdk-client-id"
        ));
    }

    /// Scenario: Workload identity inputs are omitted, partially supplied, or fully supplied.
    /// Guarantees: Explicit inputs are unchanged and each omitted input stays None for SDK env fallback.
    #[test]
    fn workload_options_preserve_explicit_inputs_and_environment_fallback() {
        let mut config = IdentityConfig {
            method: AuthMethod::WorkloadIdentity,
            ..Default::default()
        };
        let options = workload_identity_options(IdentityOptions::from(&config));
        assert!(options.client_id.is_none());
        assert!(options.tenant_id.is_none());
        assert!(options.token_file_path.is_none());
        config.client_id = Some("sdk-client".to_owned());
        let options = workload_identity_options(IdentityOptions::from(&config));
        assert_eq!(options.client_id.as_deref(), Some("sdk-client"));
        assert!(options.tenant_id.is_none());
        assert!(options.token_file_path.is_none());
        config.tenant_id = Some("sdk-tenant".to_owned());
        config.token_file_path = Some(PathBuf::from("projected-token"));
        assert!(config.validate().is_ok());
        let options = workload_identity_options(IdentityOptions::from(&config));
        assert_eq!(options.client_id.as_deref(), Some("sdk-client"));
        assert_eq!(options.tenant_id.as_deref(), Some("sdk-tenant"));
        assert_eq!(
            options.token_file_path.as_deref(),
            Some(Path::new("projected-token"))
        );
    }
}
