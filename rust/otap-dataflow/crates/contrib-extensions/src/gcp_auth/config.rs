// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the Google Cloud (GCP) Auth extension.

use std::time::Duration;

use serde::Deserialize;

/// Default base URL for the Google Compute Engine / GKE metadata server.
pub const DEFAULT_METADATA_ENDPOINT: &str = "http://metadata.google.internal";

/// Default OAuth 2.0 scope requested for Google Cloud credentials.
pub const DEFAULT_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

/// Default OAuth 2.0 scopes requested for Google Cloud credentials.
fn default_scopes() -> Vec<String> {
    vec![DEFAULT_SCOPE.to_string()]
}

/// Default per-attempt token acquisition timeout.
fn default_timeout() -> Duration {
    Duration::from_secs(10)
}

/// Default startup readiness timeout.
fn default_startup_timeout() -> Duration {
    Duration::from_secs(30)
}

/// Configuration for the Google Cloud (GCP) Auth extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Optional base URL override for the GCE/GKE compute metadata server.
    ///
    /// When omitted (`None`), credentials are resolved using standard Google
    /// Cloud Application Default Credentials (ADC) via
    /// `google_cloud_auth::credentials::Builder` (`GOOGLE_APPLICATION_CREDENTIALS`,
    /// the well-known `gcloud` ADC file, and finally the GCE/GKE metadata
    /// server at `http://metadata.google.internal` or `http://$GCE_METADATA_HOST`).
    ///
    /// When set, `google_cloud_auth::credentials::mds::Builder::with_endpoint`
    /// is used directly against this metadata server base URL (note that
    /// `google-cloud-auth` still checks `GCE_METADATA_HOST` first when set in
    /// the process environment).
    #[serde(default)]
    pub endpoint: Option<String>,

    /// OAuth 2.0 scopes requested for the access token. Defaults to
    /// `["https://www.googleapis.com/auth/cloud-platform"]`.
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,

    /// Per-attempt timeout applied when acquiring or refreshing a token via
    /// `google-cloud-auth`. Accepts human-readable durations (e.g. `10s`,
    /// `500ms`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_timeout")]
    pub timeout: Duration,

    /// How long the engine holds data-path node startup waiting for this
    /// extension to publish its first token, before aborting pipeline startup.
    /// Accepts human-readable durations (e.g. `30s`, `1m`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_startup_timeout")]
    pub startup_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: None,
            scopes: default_scopes(),
            timeout: default_timeout(),
            startup_timeout: default_startup_timeout(),
        }
    }
}

impl Config {
    /// Validates and normalizes `endpoint` (stripping trailing slashes) for
    /// `google_cloud_auth::credentials::mds::Builder::with_endpoint`.
    pub(crate) fn normalized_endpoint(&self) -> Result<Option<String>, String> {
        let Some(endpoint) = &self.endpoint else {
            return Ok(None);
        };
        validate_base_endpoint(endpoint)?;
        Ok(Some(endpoint.trim().trim_end_matches('/').to_string()))
    }

    /// Validates the configuration beyond what deserialization checks.
    pub fn validate(&self) -> Result<(), String> {
        let _ = self.normalized_endpoint()?;

        for scope in &self.scopes {
            if scope.trim().is_empty() {
                return Err("`scopes` entries must not be empty".to_string());
            }
        }

        if self.timeout.is_zero() {
            return Err("`timeout` must be greater than zero".to_string());
        }

        if self.startup_timeout.is_zero() {
            return Err("`startup_timeout` must be greater than zero".to_string());
        }

        Ok(())
    }
}

/// Parses and validates a metadata server base URL, ensuring it uses `http://`
/// or `https://`, has a non-empty host, and carries no credentials, query
/// parameters, or fragment.
pub(crate) fn validate_base_endpoint(raw: &str) -> Result<(), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("`endpoint` must not be empty when set".to_string());
    }
    let scheme_len = if trimmed
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
    {
        8
    } else if trimmed
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
    {
        7
    } else {
        return Err("`endpoint` must start with `http://` or `https://`".to_string());
    };
    if trimmed.contains('#') || trimmed.contains('?') {
        return Err("`endpoint` must not include query parameters or a URL fragment".to_string());
    }
    if trimmed[scheme_len..].trim_start_matches('/').is_empty() {
        return Err("`endpoint` must include a non-empty host".to_string());
    }
    let uri = trimmed
        .parse::<http::Uri>()
        .map_err(|e| format!("invalid `endpoint` URL `{trimmed}`: {e}"))?;
    if uri
        .authority()
        .is_some_and(|authority| authority.as_str().contains('@'))
    {
        return Err("`endpoint` must not include userinfo credentials".to_string());
    }
    if uri.host().is_none_or(str::is_empty) {
        return Err("`endpoint` must include a non-empty host".to_string());
    }
    Ok(())
}
