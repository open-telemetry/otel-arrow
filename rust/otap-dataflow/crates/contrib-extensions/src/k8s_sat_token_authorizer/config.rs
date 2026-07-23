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

/// Default startup readiness timeout.
///
/// Larger than the engine's 5 s readiness-probe default: constructing the
/// in-cluster Kubernetes client (reading the projected service-account token and
/// cluster CA, resolving the API server) plus a first-attempt retry on a ~10 s
/// cadence can exceed 5 s, so the gate must allow room for a retry.
fn default_startup_timeout() -> Duration {
    Duration::from_secs(30)
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
    #[serde(default)]
    pub allowed_service_accounts: Vec<String>,

    /// How long a reached decision is cached, keyed by the opaque token, to
    /// bound `TokenReview` calls to the API server. Accepts human-readable
    /// durations (e.g. `5m`, `30s`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_cache_ttl")]
    pub cache_ttl: Duration,

    /// Upper bound on the number of cached decisions, bounding memory. Must be
    /// greater than zero.
    #[serde(default = "default_cache_max_entries")]
    pub cache_max_entries: usize,

    /// How long the engine holds data-path node startup waiting for this
    /// extension to construct its Kubernetes client, before aborting pipeline
    /// startup. Accepts human-readable durations (e.g. `30s`, `1m`). Must be
    /// non-zero.
    #[serde(with = "humantime_serde", default = "default_startup_timeout")]
    pub startup_timeout: Duration,
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
        if self.startup_timeout.is_zero() {
            return Err("`startup_timeout` must be greater than zero".to_string());
        }
        // Surface a malformed allow-list entry at wiring time rather than
        // silently never matching it at request time.
        for entry in &self.allowed_service_accounts {
            let _ = normalize_service_account(entry)
                .map_err(|e| format!("invalid `allowed_service_accounts` entry `{entry}`: {e}"))?;
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
