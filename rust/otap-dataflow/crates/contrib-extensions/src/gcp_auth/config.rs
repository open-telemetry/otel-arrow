// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the Google Cloud (GCP) Auth extension.

use std::time::Duration;

use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TOKEN_USABLE_MARGIN;
use serde::Deserialize;

/// Default base URL for the Google Compute Engine / GKE metadata server.
pub const DEFAULT_METADATA_ENDPOINT: &str = "http://metadata.google.internal";

/// Environment variable recognized by Google Cloud client libraries to override
/// the metadata server host (for example `169.254.169.254` or a test server).
pub const GCE_METADATA_HOST_ENV: &str = "GCE_METADATA_HOST";

/// Default service account identifier queried on the metadata server.
pub const DEFAULT_SERVICE_ACCOUNT: &str = "default";

/// Default OAuth 2.0 scope requested from the metadata server.
pub const DEFAULT_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

/// Default service account identifier when omitted from config.
fn default_service_account() -> String {
    DEFAULT_SERVICE_ACCOUNT.to_string()
}

/// Default OAuth 2.0 scopes requested from the metadata server.
fn default_scopes() -> Vec<String> {
    vec![DEFAULT_SCOPE.to_string()]
}

/// Default duration ahead of expiry at which a token is refreshed.
///
/// GCE caches access tokens until 5 minutes remain before expiry, so a 4-minute
/// lead time (accounting for the refresh loop's early jitter) avoids fetching
/// before the metadata server rotates the token.
fn default_expiry_buffer() -> Duration {
    Duration::from_secs(240)
}

/// Default assumed lifetime for a token whose response omits `expires_in`.
///
/// GCP metadata server tokens typically expire after 1 hour (`3599` seconds);
/// 1 hour bounds the stale window if an endpoint omits `expires_in`.
fn default_token_lifetime() -> Duration {
    Duration::from_secs(60 * 60)
}

/// Default per-request timeout on the metadata client.
fn default_timeout() -> Duration {
    Duration::from_secs(10)
}

/// Default connection-establishment timeout on the metadata client.
fn default_connect_timeout() -> Duration {
    Duration::from_secs(5)
}

/// Default startup readiness timeout.
fn default_startup_timeout() -> Duration {
    Duration::from_secs(30)
}

/// Configuration for the Google Cloud (GCP) Auth extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Base URL of the GCE/GKE metadata server.
    ///
    /// When omitted, the extension checks the `GCE_METADATA_HOST` environment
    /// variable (`http://$GCE_METADATA_HOST`) and otherwise defaults to
    /// `http://metadata.google.internal`.
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Service account email or alias on the metadata server (under
    /// `/computeMetadata/v1/instance/service-accounts/{service_account}/token`).
    /// Defaults to `"default"`.
    #[serde(default = "default_service_account")]
    pub service_account: String,

    /// OAuth 2.0 scopes requested from the metadata server. Defaults to
    /// `["https://www.googleapis.com/auth/cloud-platform"]`.
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,

    /// Refresh this far ahead of the token's expiry. Accepts human-readable
    /// durations (e.g. `5m`, `60s`). Must be greater than the fixed usability
    /// margin ([`TOKEN_USABLE_MARGIN`]).
    #[serde(with = "humantime_serde", default = "default_expiry_buffer")]
    pub expiry_buffer: Duration,

    /// Assumed token lifetime when the metadata server's response omits
    /// `expires_in`. Accepts human-readable durations. Must be non-zero and
    /// greater than `expiry_buffer`.
    #[serde(with = "humantime_serde", default = "default_token_lifetime")]
    pub default_token_lifetime: Duration,

    /// Per-request timeout on the metadata HTTP client. Accepts human-readable
    /// durations. Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_timeout")]
    pub timeout: Duration,

    /// Timeout for establishing a connection to the metadata server. Accepts
    /// human-readable durations. Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_connect_timeout")]
    pub connect_timeout: Duration,

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
            service_account: default_service_account(),
            scopes: default_scopes(),
            expiry_buffer: default_expiry_buffer(),
            default_token_lifetime: default_token_lifetime(),
            timeout: default_timeout(),
            connect_timeout: default_connect_timeout(),
            startup_timeout: default_startup_timeout(),
        }
    }
}

impl Config {
    /// Resolves the base URL of the metadata server, honoring explicit config,
    /// `GCE_METADATA_HOST`, and [`DEFAULT_METADATA_ENDPOINT`] in that order.
    #[must_use]
    pub fn resolved_endpoint(&self) -> String {
        self.resolved_endpoint_with_env(std::env::var(GCE_METADATA_HOST_ENV).ok().as_deref())
    }

    /// Resolves the base URL of the metadata server using the supplied
    /// `GCE_METADATA_HOST` value instead of reading process environment state.
    #[must_use]
    pub(crate) fn resolved_endpoint_with_env(&self, env_host: Option<&str>) -> String {
        if let Some(endpoint) = &self.endpoint {
            return endpoint.trim().trim_end_matches('/').to_string();
        }
        if let Some(host) = env_host {
            let trimmed = host.trim().trim_end_matches('/');
            if !trimmed.is_empty() {
                if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    return trimmed.to_string();
                }
                return format!("http://{trimmed}");
            }
        }
        DEFAULT_METADATA_ENDPOINT.to_string()
    }

    /// Validates the configuration beyond what deserialization checks.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(endpoint) = &self.endpoint {
            let _ = validate_base_endpoint(endpoint, "endpoint")?;
        }

        let sa = self.service_account.trim();
        if sa.is_empty() {
            return Err("`service_account` must not be empty".to_string());
        }
        if sa.contains('/')
            || sa.contains('\\')
            || sa.contains('?')
            || sa.contains('#')
            || sa.contains('%')
            || sa == "."
            || sa == ".."
            || sa.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(
                "`service_account` must be a single path segment without `/`, `\\`, `?`, `#`, `%`, `..`, whitespace, or control characters"
                    .to_string(),
            );
        }

        for scope in &self.scopes {
            if scope.trim().is_empty() {
                return Err("`scopes` entries must not be empty".to_string());
            }
        }

        if self.expiry_buffer <= TOKEN_USABLE_MARGIN {
            return Err(format!(
                "`expiry_buffer` must be greater than {}s, the window before expiry in which a token is no longer used",
                TOKEN_USABLE_MARGIN.as_secs()
            ));
        }

        if self.default_token_lifetime <= self.expiry_buffer {
            return Err(
                "`default_token_lifetime` must be greater than `expiry_buffer`".to_string(),
            );
        }

        if self.timeout.is_zero() {
            return Err("`timeout` must be greater than zero".to_string());
        }

        if self.connect_timeout.is_zero() {
            return Err("`connect_timeout` must be greater than zero".to_string());
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
pub(crate) fn validate_base_endpoint(raw: &str, field_name: &str) -> Result<reqwest::Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("`{field_name}` must not be empty when set"));
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(format!(
            "`{field_name}` must start with `http://` or `https://`"
        ));
    }
    let url = reqwest::Url::parse(trimmed)
        .map_err(|e| format!("invalid `{field_name}` URL `{trimmed}`: {e}"))?;
    if !matches!(url.scheme(), "http" | "https") || url.cannot_be_a_base() {
        return Err(format!(
            "`{field_name}` must be a hierarchical `http://` or `https://` URL"
        ));
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(format!("`{field_name}` must include a non-empty host"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(format!(
            "`{field_name}` must not include userinfo credentials"
        ));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(format!(
            "`{field_name}` must not include query parameters or a URL fragment"
        ));
    }
    Ok(url)
}
