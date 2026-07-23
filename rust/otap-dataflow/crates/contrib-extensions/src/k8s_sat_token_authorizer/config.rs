// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the Kubernetes SAT (service-account-token) authorizer
//! extension.

use std::collections::HashSet;
use std::time::Duration;

use serde::Deserialize;

/// Default time an authorization decision is cached, keyed by the opaque token.
fn default_cache_ttl() -> Duration {
    Duration::from_secs(300)
}

/// Default maximum number of cached decisions.
fn default_cache_max_entries() -> usize {
    1024
}

/// Configuration for the Kubernetes SAT authorizer extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Audiences the presented token must be valid for. Sent to the Kubernetes
    /// `TokenReview` API as the requested audiences; the API server accepts the
    /// token only if it is valid for at least one. Must be non-empty so a token
    /// minted for an unrelated audience is never admitted.
    pub audiences: Vec<String>,

    /// Allow-list of service accounts admitted after authentication. Each entry
    /// may be given as a full username
    /// (`system:serviceaccount:<namespace>:<name>`), or the shorthand
    /// `<namespace>:<name>` or `<namespace>/<name>`. An empty list admits any
    /// service account the API server authenticates (audience-only admission).
    ///
    /// Mutually exclusive with [`resource_attributes`](Self::resource_attributes):
    /// admission is either a static allow-list or a Kubernetes RBAC check, not
    /// both.
    #[serde(default)]
    pub allowed_service_accounts: Vec<String>,

    /// Kubernetes RBAC admission via `SubjectAccessReview`. When set, after the
    /// token is authenticated the extension asks the API server whether the
    /// authenticated identity may perform `verb` on the described resource; the
    /// request is admitted only if RBAC allows it.
    ///
    /// Mutually exclusive with
    /// [`allowed_service_accounts`](Self::allowed_service_accounts). When
    /// neither is set, any authenticated service account is admitted
    /// (audience-only admission).
    #[serde(default)]
    pub resource_attributes: Option<ResourceAttributesConfig>,

    /// How long a reached decision is cached, keyed by the opaque token, to
    /// bound `TokenReview` calls to the API server. Accepts human-readable
    /// durations (e.g. `5m`, `30s`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_cache_ttl")]
    pub cache_ttl: Duration,

    /// Upper bound on the number of cached decisions, bounding memory. Must be
    /// greater than zero.
    #[serde(default = "default_cache_max_entries")]
    pub cache_max_entries: usize,
}

/// Kubernetes RBAC resource the authenticated identity must be permitted to act
/// on, checked via `SubjectAccessReview`.
///
/// Mirrors the fields of the Kubernetes `ResourceAttributes` type. `resource`
/// and `verb` are required; the rest narrow the check.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceAttributesConfig {
    /// API group of the resource (e.g. `telemetry.opentelemetry.io`). Empty for
    /// the core group.
    #[serde(default)]
    pub group: Option<String>,
    /// API version of the resource (e.g. `v1`).
    #[serde(default)]
    pub version: Option<String>,
    /// Resource type to authorize (e.g. `telemetry`). Required.
    pub resource: String,
    /// Action to authorize (e.g. `export`, `create`). Required.
    pub verb: String,
    /// Namespace for namespaced resources. Empty means cluster-scoped.
    #[serde(default)]
    pub namespace: Option<String>,
    /// Specific resource name. Empty means any resource of this type.
    #[serde(default)]
    pub name: Option<String>,
    /// Subresource to authorize (e.g. `status`).
    #[serde(default)]
    pub subresource: Option<String>,
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    pub fn validate(&self) -> Result<(), String> {
        if self.audiences.is_empty() {
            return Err("`audiences` must not be empty".to_string());
        }
        if self.audiences.iter().any(|a| a.trim().is_empty()) {
            return Err("`audiences` must not contain empty entries".to_string());
        }
        if self.cache_ttl.is_zero() {
            return Err("`cache_ttl` must be greater than zero".to_string());
        }
        if self.cache_max_entries == 0 {
            return Err("`cache_max_entries` must be greater than zero".to_string());
        }
        // Surface a malformed allow-list entry at wiring time rather than
        // silently never matching it at request time.
        for entry in &self.allowed_service_accounts {
            let _ = normalize_service_account(entry)
                .map_err(|e| format!("invalid `allowed_service_accounts` entry `{entry}`: {e}"))?;
        }

        if let Some(ra) = &self.resource_attributes {
            // Admission is one strategy: a static allow-list or an RBAC check.
            if !self.allowed_service_accounts.is_empty() {
                return Err(
                    "`allowed_service_accounts` and `resource_attributes` are mutually exclusive; set only one"
                        .to_string(),
                );
            }
            if ra.resource.trim().is_empty() {
                return Err("`resource_attributes.resource` must not be empty".to_string());
            }
            if ra.verb.trim().is_empty() {
                return Err("`resource_attributes.verb` must not be empty".to_string());
            }
        }

        Ok(())
    }

    /// Builds the canonical allow-list set of service-account usernames, or
    /// `None` when the list is empty (any authenticated account is admitted).
    ///
    /// Every entry is normalized to its full `system:serviceaccount:<ns>:<name>`
    /// username so admission is an O(1) set lookup against the username the API
    /// server returns.
    #[must_use]
    pub fn allowed_service_account_set(&self) -> Option<HashSet<String>> {
        if self.allowed_service_accounts.is_empty() {
            return None;
        }
        Some(
            self.allowed_service_accounts
                .iter()
                // Entries are validated in `validate`, so normalization cannot
                // fail here; fall back to the raw entry defensively.
                .map(|e| normalize_service_account(e).unwrap_or_else(|_| e.clone()))
                .collect(),
        )
    }
}

/// Canonicalizes a service-account allow-list entry into the full
/// `system:serviceaccount:<namespace>:<name>` username the API server returns.
///
/// Accepts three input shapes: the full username verbatim, `<namespace>:<name>`,
/// and `<namespace>/<name>`. Rejects entries with empty namespace or name.
pub(crate) fn normalize_service_account(entry: &str) -> Result<String, String> {
    const PREFIX: &str = "system:serviceaccount:";
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err("entry is empty".to_string());
    }

    if let Some(rest) = trimmed.strip_prefix(PREFIX) {
        // Full username form: validate it has a non-empty namespace and name.
        let (namespace, name) = rest
            .split_once(':')
            .ok_or_else(|| "expected `system:serviceaccount:<namespace>:<name>`".to_string())?;
        if namespace.is_empty() || name.is_empty() {
            return Err("namespace and name must both be non-empty".to_string());
        }
        return Ok(trimmed.to_string());
    }

    // Shorthand form: `<namespace>/<name>` or `<namespace>:<name>`.
    let (namespace, name) = trimmed
        .split_once(['/', ':'])
        .ok_or_else(|| "expected `<namespace>/<name>` or `<namespace>:<name>`".to_string())?;
    if namespace.is_empty() || name.is_empty() {
        return Err("namespace and name must both be non-empty".to_string());
    }
    Ok(format!("{PREFIX}{namespace}:{name}"))
}
