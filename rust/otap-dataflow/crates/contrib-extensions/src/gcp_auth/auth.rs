// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Google Cloud compute metadata server client and token acquisition.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::auth::BearerToken;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use super::config::Config;
use super::error::Error;
use crate::common::background_refresh::BackgroundProviderSource;

/// Required HTTP header name for all requests to the GCE/GKE metadata server.
const METADATA_FLAVOR_HEADER: &str = "Metadata-Flavor";

/// Required HTTP header value for all requests to the GCE/GKE metadata server.
const METADATA_FLAVOR_GOOGLE: &str = "Google";

/// Maximum number of bytes of a metadata server error body echoed in logs.
const MAX_ERROR_BODY_BYTES: usize = 256;

/// A Google Cloud metadata token source.
#[derive(Clone)]
pub struct Auth {
    /// Full URL of the metadata server token endpoint, including any `scopes`
    /// query parameter.
    token_url: reqwest::Url,
    /// Assumed lifetime for a token whose response omits `expires_in`.
    default_token_lifetime: Duration,
    /// HTTP client used to reach the metadata server.
    client: reqwest::Client,
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        otel_arrow_dfe_otap::crypto::ensure_crypto_provider();

        let token_url = build_token_url(config)?;

        // NOTE: Traffic to the link-local GCE/GKE metadata server
        // (`metadata.google.internal` / `169.254.169.254`) must bypass any
        // ambient HTTP_PROXY/HTTPS_PROXY environment variables so requests are
        // never routed to an external corporate proxy.
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .no_proxy()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .build()
            .map_err(|e| Error::BuildHttpClient {
                reason: e.to_string(),
            })?;

        Ok(Self {
            token_url,
            default_token_lifetime: config.default_token_lifetime,
            client,
        })
    }
}

#[async_trait]
impl BackgroundProviderSource<BearerToken> for Auth {
    type Error = Error;

    /// Acquires a single OAuth 2.0 access token from the GCE/GKE metadata server.
    async fn fetch(&self) -> Result<BearerToken, Error> {
        let response = self
            .client
            .get(self.token_url.clone())
            .header(METADATA_FLAVOR_HEADER, METADATA_FLAVOR_GOOGLE)
            .send()
            .await
            .map_err(|e| Error::TokenAcquisition {
                message: format!("metadata server request failed: {e}"),
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::TokenAcquisition {
                message: format!(
                    "metadata server returned {status}: {}",
                    truncate_body(&body)
                ),
            });
        }

        let token: MetadataTokenResponse =
            response.json().await.map_err(|e| Error::TokenAcquisition {
                message: format!("invalid metadata token response: {e}"),
            })?;

        to_bearer_token(&token, self.default_token_lifetime)
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("gcp_auth.token_refresh_failed", error = %error);
    }

    fn expires_on(value: &BearerToken) -> Option<Instant> {
        value.expires_on()
    }
}

/// Builds the full metadata server token URL for the configured endpoint,
/// service account, and scopes.
fn build_token_url(config: &Config) -> Result<reqwest::Url, Error> {
    let base = config.resolved_endpoint();
    let service_account = config.service_account.trim();
    let raw_url =
        format!("{base}/computeMetadata/v1/instance/service-accounts/{service_account}/token");

    let mut url = reqwest::Url::parse(&raw_url).map_err(|e| Error::BuildHttpClient {
        reason: format!("invalid metadata endpoint URL `{raw_url}`: {e}"),
    })?;

    if !config.scopes.is_empty() {
        let scopes = config
            .scopes
            .iter()
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join(",");
        let _ = url.query_pairs_mut().append_pair("scopes", &scopes);
    }

    Ok(url)
}

/// JSON payload returned by the GCE/GKE metadata server token endpoint.
#[derive(Debug, Deserialize)]
struct MetadataTokenResponse {
    /// Issued OAuth 2.0 access token.
    access_token: SecretString,
    /// Token type. Defaults to `"Bearer"` if omitted.
    #[serde(default = "default_token_type")]
    token_type: String,
    /// Relative token lifetime in seconds, accepted as a JSON number or string.
    #[serde(default, deserialize_with = "deserialize_expires_in")]
    expires_in: Option<u64>,
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

/// Deserializes `expires_in` from either a JSON number or a JSON string.
fn deserialize_expires_in<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ExpiresIn {
        Number(u64),
        Text(String),
    }

    match Option::<ExpiresIn>::deserialize(deserializer)? {
        None => Ok(None),
        Some(ExpiresIn::Number(seconds)) => Ok(Some(seconds)),
        Some(ExpiresIn::Text(text)) => text
            .trim()
            .parse()
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

/// Converts a metadata token response into a [`BearerToken`].
fn to_bearer_token(
    response: &MetadataTokenResponse,
    fallback_lifetime: Duration,
) -> Result<BearerToken, Error> {
    if !response.token_type.eq_ignore_ascii_case("bearer") {
        return Err(Error::TokenAcquisition {
            message: format!(
                "metadata server returned unsupported token_type `{}`; only Bearer is supported",
                response.token_type
            ),
        });
    }

    let secret = response.access_token.expose_secret().trim().to_owned();
    if secret.is_empty() {
        return Err(Error::TokenAcquisition {
            message: "metadata server returned an empty `access_token`".to_string(),
        });
    }

    let expires_in = response
        .expires_in
        .filter(|&secs| secs > 0)
        .map_or(fallback_lifetime, Duration::from_secs);

    Ok(BearerToken::from_relative_expiry(secret, expires_in))
}

/// Truncates an untrusted response body to [`MAX_ERROR_BODY_BYTES`] for
/// inclusion in an error message.
fn truncate_body(body: &str) -> String {
    let mut end = MAX_ERROR_BODY_BYTES.min(body.len());
    while end > 0 && !body.is_char_boundary(end) {
        end -= 1;
    }
    if end == body.len() {
        body.to_owned()
    } else {
        format!("{}... [truncated]", &body[..end])
    }
}
