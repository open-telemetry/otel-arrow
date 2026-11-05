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

/// Maximum number of bytes read from a successful metadata server token
/// response before rejecting it as oversized.
const MAX_TOKEN_RESPONSE_BYTES: usize = 64 * 1024;

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
            .redirect(reqwest::redirect::Policy::none())
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
            let body = read_error_body_preview(response).await;
            return Err(Error::TokenAcquisition {
                message: format!("metadata server returned {status}: {body}"),
            });
        }

        let body = read_bounded_body(response, MAX_TOKEN_RESPONSE_BYTES).await?;
        let token: MetadataTokenResponse =
            serde_json::from_slice(&body).map_err(|e| Error::TokenAcquisition {
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
    let mut url = super::config::validate_base_endpoint(&base, "endpoint")
        .map_err(|reason| Error::BuildHttpClient { reason })?;

    let service_account = config.service_account.trim();
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|()| Error::BuildHttpClient {
                reason: format!("metadata endpoint URL `{base}` cannot be a base URL"),
            })?;
        let _ = segments.pop_if_empty().extend([
            "computeMetadata",
            "v1",
            "instance",
            "service-accounts",
            service_account,
            "token",
        ]);
    }

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
    if secret.bytes().any(|b| b <= b' ' || b >= 0x7f) {
        return Err(Error::TokenAcquisition {
            message: "metadata server returned an `access_token` containing whitespace or control characters".to_string(),
        });
    }

    let expires_in = match response.expires_in {
        Some(0) => {
            return Err(Error::TokenAcquisition {
                message: "metadata server returned an expired token (`expires_in` is 0)"
                    .to_string(),
            });
        }
        Some(secs) => Duration::from_secs(secs),
        None => fallback_lifetime,
    };

    Ok(BearerToken::from_relative_expiry(secret, expires_in))
}

/// Reads up to `max_bytes` from a successful HTTP response body, failing fast
/// if `Content-Length` or the streamed payload exceeds the limit.
async fn read_bounded_body(
    mut response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, Error> {
    if let Some(content_length) = response.content_length()
        && content_length > max_bytes as u64
    {
        return Err(Error::TokenAcquisition {
            message: format!(
                "metadata token response content-length ({content_length} bytes) exceeds limit ({max_bytes} bytes)"
            ),
        });
    }

    let mut buf = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| Error::TokenAcquisition {
            message: format!("failed to read metadata token response body: {e}"),
        })?
    {
        if buf.len().saturating_add(chunk.len()) > max_bytes {
            return Err(Error::TokenAcquisition {
                message: format!("metadata token response body exceeds limit ({max_bytes} bytes)"),
            });
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

/// Reads at most [`MAX_ERROR_BODY_BYTES`] of an error response body so a large
/// error payload cannot allocate unbounded memory.
async fn read_error_body_preview(mut response: reqwest::Response) -> String {
    let mut buf = Vec::with_capacity(MAX_ERROR_BODY_BYTES + 1);
    while buf.len() <= MAX_ERROR_BODY_BYTES {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                let remaining = (MAX_ERROR_BODY_BYTES + 1).saturating_sub(buf.len());
                buf.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
            }
            Ok(None) | Err(_) => break,
        }
    }
    let text = String::from_utf8_lossy(&buf);
    truncate_body(&text)
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
