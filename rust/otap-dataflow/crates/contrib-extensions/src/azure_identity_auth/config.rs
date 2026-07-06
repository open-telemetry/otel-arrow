// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the Azure Identity Auth extension.

use std::path::PathBuf;

use serde::Deserialize;

/// Default OAuth scope requested when none is configured.
fn default_scope() -> String {
    "https://monitor.azure.com/.default".to_string()
}

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

/// Configuration for the Azure Identity Auth extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Authentication flow to use.
    #[serde(default)]
    pub method: AuthMethod,
    /// Entra client ID. User-assigned MSI client ID for `managed_identity`
    /// (omit for system-assigned); application client ID for
    /// `workload_identity` (falls back to `AZURE_CLIENT_ID`).
    #[serde(default)]
    pub client_id: Option<String>,
    /// Entra tenant ID. Only for `workload_identity` (falls back to
    /// `AZURE_TENANT_ID`).
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Path to the projected federated token file. Only for
    /// `workload_identity` (falls back to `AZURE_FEDERATED_TOKEN_FILE`).
    #[serde(default)]
    pub token_file_path: Option<PathBuf>,
    /// OAuth scope to request tokens for. Must be non-empty.
    #[serde(default = "default_scope")]
    pub scope: String,
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    ///
    /// Rejects an empty or whitespace-only `scope`.
    pub fn validate(&self) -> Result<(), String> {
        if self.scope.trim().is_empty() {
            return Err("`scope` must not be empty".to_string());
        }
        Ok(())
    }
}
