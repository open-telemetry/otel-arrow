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
    /// Validates and normalizes `endpoint` (stripping trailing slashes without
    /// stripping the `http://` or `https://` scheme prefix) for
    /// `google_cloud_auth::credentials::mds::Builder::with_endpoint`.
    pub(crate) fn normalized_endpoint(&self) -> Result<Option<String>, String> {
        let Some(endpoint) = &self.endpoint else {
            return Ok(None);
        };
        validate_base_endpoint(endpoint, "endpoint")?;
        Ok(Some(
            trim_endpoint_trailing_slashes(endpoint.trim()).to_string(),
        ))
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

/// Returns the byte length of the `http://` or `https://` scheme prefix when
/// present (case-insensitively).
fn http_scheme_len(raw: &str) -> Option<usize> {
    if raw
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
    {
        Some(8)
    } else if raw
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
    {
        Some(7)
    } else {
        None
    }
}

/// Strips trailing `/` characters from an endpoint string without stripping the
/// `//` separator of an `http://` or `https://` scheme prefix.
pub(crate) fn trim_endpoint_trailing_slashes(raw: &str) -> &str {
    let min_len = http_scheme_len(raw).unwrap_or(0);
    let mut end = raw.len();
    while end > min_len && raw.as_bytes()[end - 1] == b'/' {
        end -= 1;
    }
    &raw[..end]
}

/// Parses and validates a metadata server base URL, ensuring it uses `http://`
/// or `https://`, has a non-empty host, and carries no credentials, query
/// parameters, or fragment.
pub(crate) fn validate_base_endpoint(raw: &str, field_name: &str) -> Result<(), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("`{field_name}` must not be empty when set"));
    }
    let Some(scheme_len) = http_scheme_len(trimmed) else {
        return Err(format!(
            "`{field_name}` must start with `http://` or `https://`"
        ));
    };
    if trimmed.contains('#') || trimmed.contains('?') {
        return Err(format!(
            "`{field_name}` must not include query parameters or a URL fragment"
        ));
    }
    if trimmed[scheme_len..].trim_start_matches('/').is_empty() {
        return Err(format!("`{field_name}` must include a non-empty host"));
    }
    let uri = trimmed
        .parse::<http::Uri>()
        .map_err(|e| format!("invalid `{field_name}` URL `{trimmed}`: {e}"))?;
    let Some(authority) = uri.authority() else {
        return Err(format!("`{field_name}` must include a non-empty host"));
    };
    if authority.as_str().contains('@') {
        return Err(format!(
            "`{field_name}` must not include userinfo credentials"
        ));
    }
    if uri.host().is_none_or(str::is_empty) {
        return Err(format!("`{field_name}` must include a non-empty host"));
    }
    Ok(())
}
