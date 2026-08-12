// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the Kubernetes service-account-token auth extension.

use std::collections::HashSet;
use std::time::Duration;

use serde::Deserialize;

/// Default maximum duration of a Kubernetes review request.
pub(crate) const DEFAULT_REVIEW_TIMEOUT: Duration = Duration::from_secs(10);

/// Default time an authorization decision is cached, keyed by the token's
/// SHA-256 digest (never the plaintext token; see `cache.rs`).
fn default_cache_ttl() -> Duration {
    Duration::from_secs(300)
}

/// Default maximum number of cached decisions.
fn default_cache_max_entries() -> usize {
    1024
}

fn default_review_timeout() -> Duration {
    DEFAULT_REVIEW_TIMEOUT
}

/// Configuration for the Kubernetes service-account-token auth extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Per-audience (per-tenant) admission entries. Each entry ties **one**
    /// audience to its own admission policy, so a service account is only trusted
    /// for the audience it is bound to. `TokenReview` is sent the union of all
    /// audiences; admission then uses the entry for the audience the API server
    /// actually confirmed, which prevents one tenant's identity from being
    /// admitted against another tenant's audience. Must be non-empty, with a
    /// unique audience per entry.
    pub audiences: Vec<AudienceConfig>,

    /// How long a reached decision is cached, keyed by the token's SHA-256
    /// digest, to bound `TokenReview` calls to the API server. Accepts
    /// human-readable durations (e.g. `5m`, `30s`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_cache_ttl")]
    pub cache_ttl: Duration,

    /// Upper bound on the number of cached decisions, bounding memory. Must be
    /// greater than zero.
    #[serde(default = "default_cache_max_entries")]
    pub cache_max_entries: usize,

    /// Maximum duration of each `TokenReview` or `SubjectAccessReview` request.
    /// Accepts human-readable durations (e.g. `10s`, `500ms`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_review_timeout")]
    pub review_timeout: Duration,
}

/// One audience-scoped admission entry.
///
/// Ties a single audience to the policy used to admit tokens presented for it.
/// The two admission strategies (`allowed_service_accounts` and
/// `resource_attributes`) are mutually exclusive within an entry; when neither
/// is set, any account authenticated for this audience is admitted.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudienceConfig {
    /// The audience this entry admits. A token is admitted through this entry
    /// only when `TokenReview` confirms it is valid for this exact audience.
    pub audience: String,

    /// Allow-list of service accounts admitted for this audience. Each entry may
    /// be a full username (`system:serviceaccount:<namespace>:<name>`) or the
    /// shorthand `<namespace>:<name>` / `<namespace>/<name>`. An empty list
    /// admits any account authenticated for this audience. Mutually exclusive
    /// with [`resource_attributes`](Self::resource_attributes).
    #[serde(default)]
    pub allowed_service_accounts: Vec<String>,

    /// Kubernetes RBAC admission for this audience via `SubjectAccessReview`.
    /// Mutually exclusive with
    /// [`allowed_service_accounts`](Self::allowed_service_accounts).
    #[serde(default)]
    pub resource_attributes: Option<ResourceAttributesConfig>,
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
        if self.cache_ttl.is_zero() {
            return Err("`cache_ttl` must be greater than zero".to_string());
        }
        if self.cache_max_entries == 0 {
            return Err("`cache_max_entries` must be greater than zero".to_string());
        }
        if self.review_timeout.is_zero() {
            return Err("`review_timeout` must be greater than zero".to_string());
        }

        let mut seen_audiences = HashSet::new();
        for entry in &self.audiences {
            entry.validate()?;
            // A duplicate audience would make admission for it ambiguous.
            if !seen_audiences.insert(entry.audience.trim().to_string()) {
                return Err(format!("duplicate audience `{}`", entry.audience.trim()));
            }
        }

        Ok(())
    }
}

impl AudienceConfig {
    /// Validates a single audience entry.
    fn validate(&self) -> Result<(), String> {
        if self.audience.trim().is_empty() {
            return Err("`audiences[].audience` must not be empty".to_string());
        }
        let audience = self.audience.trim();

        // Surface a malformed allow-list entry at wiring time rather than
        // silently never matching it at request time.
        for entry in &self.allowed_service_accounts {
            let _ = normalize_service_account(entry).map_err(|e| {
                format!("invalid `allowed_service_accounts` entry `{entry}` for audience `{audience}`: {e}")
            })?;
        }

        if let Some(ra) = &self.resource_attributes {
            // Admission is one strategy: a static allow-list or an RBAC check.
            if !self.allowed_service_accounts.is_empty() {
                return Err(format!(
                    "`allowed_service_accounts` and `resource_attributes` are mutually exclusive for audience `{audience}`; set only one"
                ));
            }
            if ra.resource.trim().is_empty() {
                return Err(format!(
                    "`resource_attributes.resource` must not be empty for audience `{audience}`"
                ));
            }
            if ra.verb.trim().is_empty() {
                return Err(format!(
                    "`resource_attributes.verb` must not be empty for audience `{audience}`"
                ));
            }
        }

        Ok(())
    }

    /// The canonical allow-list set of service-account usernames for this
    /// audience entry, or `None` when the list is empty (any authenticated
    /// account is admitted for this audience).
    ///
    /// Every entry is normalized to its full `system:serviceaccount:<ns>:<name>`
    /// username so admission is an O(1) set lookup against the username the API
    /// server returns.
    #[must_use]
    pub(crate) fn allowed_service_account_set(&self) -> Option<HashSet<String>> {
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
