// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Protocol-neutral configuration for startup acquisition of two Key Vault secrets.

use std::time::{Duration, Instant};

use azure_core::http::Url;
use serde::Deserialize;

use crate::common::azure_identity::IdentityConfig;

fn default_startup_timeout() -> Duration {
    Duration::from_secs(30)
}

/// A named secret, optionally pinned to an immutable Key Vault version.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretReference {
    /// Key Vault secret name (1-127 ASCII letters, digits, or hyphens).
    pub name: String,
    /// A 32-character hexadecimal version. Omit to retrieve latest at startup.
    #[serde(default)]
    pub version: Option<String>,
}

impl SecretReference {
    fn validate(&self) -> Result<(), &'static str> {
        if self.name.is_empty()
            || self.name.len() > 127
            || !self
                .name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err("secret name must contain 1-127 ASCII letters, digits, or hyphens");
        }
        if let Some(version) = &self.version
            && (version.len() != 32 || !version.bytes().all(|byte| byte.is_ascii_hexdigit()))
        {
            return Err("secret version must contain exactly 32 ASCII hexadecimal digits");
        }
        Ok(())
    }
}

/// Shared Key Vault acquisition options; no inline or file credential fallback.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// HTTPS Azure Key Vault data-plane endpoint, without a path or query.
    pub vault_url: String,
    /// Reference to the username secret.
    pub username_secret: SecretReference,
    /// Reference to the password secret.
    pub password_secret: SecretReference,
    /// Azure identity selection. Defaults to system-assigned managed identity.
    #[serde(default)]
    pub identity: IdentityConfig,
    /// Maximum startup readiness wait and per-acquisition duration.
    #[serde(with = "humantime_serde", default = "default_startup_timeout")]
    pub startup_timeout: Duration,
}

impl Config {
    /// Reject unsafe endpoints, invalid references, and inapplicable identity fields.
    pub fn validate(&self) -> Result<(), String> {
        validate_vault_url(&self.vault_url).map_err(str::to_owned)?;
        self.username_secret
            .validate()
            .map_err(|reason| format!("invalid username_secret: {reason}"))?;
        self.password_secret
            .validate()
            .map_err(|reason| format!("invalid password_secret: {reason}"))?;
        self.identity.validate()?;
        if self.startup_timeout.is_zero()
            || Instant::now().checked_add(self.startup_timeout).is_none()
        {
            return Err("startup_timeout must be positive and representable".to_owned());
        }
        Ok(())
    }
}

fn validate_vault_url(endpoint: &str) -> Result<(), &'static str> {
    const INVALID: &str = "vault_url must be an HTTPS Azure Key Vault endpoint without credentials, a custom port, path, query, or fragment";
    // Do not expose URL parser errors: they can include attacker-controlled input.
    if endpoint.trim() != endpoint || endpoint.contains('\\') {
        return Err(INVALID);
    }
    let url = Url::parse(endpoint).map_err(|_| INVALID)?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some_and(|port| port != 443)
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(INVALID);
    }
    let host = url.host_str().ok_or(INVALID)?;
    let authority = endpoint.split_once("://").ok_or(INVALID)?.1;
    let authority = authority.strip_suffix('/').unwrap_or(authority);
    if !authority.eq_ignore_ascii_case(host)
        && !authority.eq_ignore_ascii_case(&format!("{host}:443"))
    {
        return Err(INVALID);
    }
    // The SDK validates challenge resources and disables redirects. Restrict the
    // initial request too, so arbitrary hosts cannot solicit Azure access tokens.
    let name = [
        ".vault.azure.net",
        ".vault.usgovcloudapi.net",
        ".vault.azure.cn",
    ]
    .iter()
    .find_map(|suffix| host.strip_suffix(suffix))
    .ok_or(INVALID)?;
    if !(3..=24).contains(&name.len())
        || !name.as_bytes()[0].is_ascii_alphabetic()
        || !name.as_bytes()[name.len() - 1].is_ascii_alphanumeric()
        || name.contains("--")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(INVALID);
    }
    Ok(())
}
