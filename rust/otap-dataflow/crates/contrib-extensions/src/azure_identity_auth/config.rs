// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the Azure Identity Auth extension.

use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

/// Default OAuth scope requested when none is configured.
fn default_scope() -> String {
    "https://monitor.azure.com/.default".to_string()
}

/// Default startup readiness timeout.
///
/// Larger than the engine's 5 s readiness-probe default: Azure cold-start
/// acquisition (IMDS Managed Identity, or Workload Identity Federation with
/// SDK-internal retries/backoff) can exceed 5 s, and a failed first attempt is
/// retried on a ~10 s cadence, so the gate must allow room for a retry.
fn default_startup_timeout() -> Duration {
    Duration::from_secs(30)
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
    /// How long the engine holds data-path node startup waiting for this
    /// extension to publish its first token, before aborting pipeline startup.
    /// Accepts human-readable durations (e.g. `30s`, `1m`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_startup_timeout")]
    pub startup_timeout: Duration,
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    ///
    /// Rejects an empty/whitespace-only `scope` and any per-method field that
    /// does not apply to the selected [`AuthMethod`].
    pub fn validate(&self) -> Result<(), String> {
        if self.scope.trim().is_empty() {
            return Err("`scope` must not be empty".to_string());
        }

        if self.startup_timeout.is_zero() {
            return Err("`startup_timeout` must be greater than zero".to_string());
        }

        // `tenant_id` and `token_file_path` are Workload Identity Federation
        // inputs; they are meaningless for the other methods.
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

        // `client_id` selects a user-assigned Managed Identity or the WIF
        // application id; it has no meaning for developer tooling.
        if self.method == AuthMethod::Development && self.client_id.is_some() {
            return Err(
                "`client_id` is not valid for the `development` method".to_string(),
            );
        }

        Ok(())
    }
}
