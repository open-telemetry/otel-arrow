// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OAuth 2.0 client construction and token acquisition.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::SystemTime;

use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{
    AsyncHttpClient, ClientId, ClientSecret, HttpRequest, HttpResponse, Scope, TokenResponse,
    TokenUrl,
};
use otap_df_engine::capability::auth::BearerToken;
use otap_df_telemetry::otel_warn;
use reqwest::{Certificate, Identity};

use super::config::{Config, GrantType};
use super::error::Error;

/// An OAuth 2.0 token source: a grant, token endpoint, scopes, and a
/// TLS-configured HTTP client used to reach the token endpoint.
#[derive(Clone)]
pub struct Auth {
    /// Grant used to acquire tokens.
    grant_type: GrantType,
    /// Parsed token endpoint URL.
    token_url: TokenUrl,
    /// Requested scopes.
    scopes: Vec<Scope>,
    /// Extra parameters appended to the token request body.
    endpoint_params: Vec<(String, String)>,
    /// Inline client identifier.
    client_id: Option<String>,
    /// Path to a file holding the client identifier (re-read each acquisition).
    client_id_file: Option<PathBuf>,
    /// Inline client secret.
    client_secret: Option<String>,
    /// Path to a file holding the client secret (re-read each acquisition).
    client_secret_file: Option<PathBuf>,
    /// reqwest-backed HTTP executor for the oauth2 client.
    http: HttpExecutor,
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        // The reqwest/rustls HTTP client needs a process-wide crypto provider
        // installed before any TLS request is made.
        otap_df_otap::crypto::ensure_crypto_provider();

        // OAuth 2.0 requires the token endpoint to be TLS-protected; warn (but
        // do not block) when a plaintext endpoint is configured.
        if config.token_url.starts_with("http://") {
            otel_warn!(
                "oauth2_client_auth.insecure_token_url",
                token_url = config.token_url.as_str()
            );
        }

        let token_url = TokenUrl::new(config.token_url.clone()).map_err(|e| {
            Error::BuildHttpClient {
                reason: format!("invalid token_url: {e}"),
            }
        })?;

        let http = HttpExecutor {
            client: build_reqwest_client(config)?,
        };

        Ok(Self {
            grant_type: config.grant_type,
            token_url,
            scopes: config.scopes.iter().cloned().map(Scope::new).collect(),
            endpoint_params: config
                .endpoint_params
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            client_id: config.client_id.clone(),
            client_id_file: config.client_id_file.clone(),
            client_secret: config.client_secret.clone(),
            client_secret_file: config.client_secret_file.clone(),
            http,
        })
    }

    /// Acquires a single token (no retries) and converts it into a
    /// [`BearerToken`].
    pub async fn get_token(&self) -> Result<BearerToken, Error> {
        match self.grant_type {
            GrantType::ClientCredentials => self.get_token_client_credentials().await,
        }
    }

    /// Acquires a token using the client-credentials grant.
    async fn get_token_client_credentials(&self) -> Result<BearerToken, Error> {
        // Credentials are read fresh on each acquisition so the file forms can
        // rotate without a restart.
        let client_id =
            read_credential(self.client_id_file.as_ref(), self.client_id.as_ref(), "client_id")
                .await?;
        let client_secret = read_credential(
            self.client_secret_file.as_ref(),
            self.client_secret.as_ref(),
            "client_secret",
        )
        .await?;

        let client = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_token_uri(self.token_url.clone());

        let mut request = client.exchange_client_credentials();
        for scope in &self.scopes {
            request = request.add_scope(scope.clone());
        }
        for (name, value) in &self.endpoint_params {
            request = request.add_extra_param(name.clone(), value.clone());
        }

        let response = request
            .request_async(&self.http)
            .await
            .map_err(|e| Error::TokenAcquisition {
                message: e.to_string(),
            })?;

        Ok(to_bearer_token(&response))
    }
}

/// Converts an OAuth 2.0 token response into a [`BearerToken`], carrying the
/// relative `expires_in` through as an absolute expiry.
fn to_bearer_token(response: &BasicTokenResponse) -> BearerToken {
    let secret = response.access_token().secret().to_owned();
    match response.expires_in() {
        // Let the capability crate centralize the absolute-to-monotonic `Instant`
        // conversion so every provider handles expiry the same way.
        Some(expires_in) => BearerToken::from_absolute_expiry(secret, SystemTime::now() + expires_in),
        None => BearerToken::without_expiry(secret),
    }
}

/// Reads a credential value, preferring the file form (re-read on each call so
/// the credential can rotate without a restart) over the inline value.
async fn read_credential(
    file: Option<&PathBuf>,
    inline: Option<&String>,
    field: &str,
) -> Result<String, Error> {
    if let Some(path) = file {
        let contents = tokio::fs::read_to_string(path)
            .await
            .map_err(|source| Error::ReadCredentialFile {
                path: path.clone(),
                source,
            })?;
        return Ok(contents.trim().to_owned());
    }
    if let Some(value) = inline {
        return Ok(value.clone());
    }
    Err(Error::TokenAcquisition {
        message: format!("no `{field}` or `{field}_file` configured"),
    })
}

/// A reqwest-backed HTTP executor implementing the oauth2 crate's
/// [`AsyncHttpClient`].
///
/// The client is built from the extension's shared [`TlsClientConfig`], so the
/// token endpoint is reached with the same TLS behavior as the rest of the
/// collector. A custom executor (rather than the oauth2 crate's built-in
/// reqwest integration) keeps a single reqwest version in the token path.
///
/// [`TlsClientConfig`]: otap_df_config::tls::TlsClientConfig
#[derive(Clone)]
struct HttpExecutor {
    client: reqwest::Client,
}

impl<'c> AsyncHttpClient<'c> for HttpExecutor {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send + 'c>>;

    fn call(&'c self, request: HttpRequest) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move {
            let request =
                reqwest::Request::try_from(request).map_err(|e| Error::TokenAcquisition {
                    message: format!("invalid token request: {e}"),
                })?;
            let response =
                client
                    .execute(request)
                    .await
                    .map_err(|e| Error::TokenAcquisition {
                        message: format!("token endpoint request failed: {e}"),
                    })?;

            let mut builder = http::Response::builder()
                .status(response.status())
                .version(response.version());
            for (name, value) in response.headers() {
                builder = builder.header(name, value);
            }
            let body = response
                .bytes()
                .await
                .map_err(|e| Error::TokenAcquisition {
                    message: format!("reading token response failed: {e}"),
                })?
                .to_vec();
            builder
                .body(body)
                .map_err(|e| Error::TokenAcquisition {
                    message: format!("invalid token response: {e}"),
                })
        })
    }
}

/// Builds a reqwest client for the token endpoint from the extension's shared
/// TLS config, mirroring how the OTLP/HTTP exporter builds its client.
fn build_reqwest_client(config: &Config) -> Result<reqwest::Client, Error> {
    let mut builder = reqwest::Client::builder().use_rustls_tls();

    if let Some(timeout) = config.timeout {
        builder = builder.timeout(timeout);
    }

    if let Some(tls) = &config.tls {
        // `insecure_skip_verify` disables server certificate verification; it
        // maps to reqwest's `danger_accept_invalid_certs`, matching the
        // OTLP/HTTP exporter. Intended only for local development or testing
        // against a self-signed endpoint.
        if tls.insecure_skip_verify == Some(true) {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let mut certs = Vec::new();
        if let Some(ca_pem) = &tls.ca_pem {
            certs.push(Certificate::from_pem(ca_pem.as_bytes()).map_err(build_err)?);
        }
        if let Some(ca_file) = &tls.ca_file {
            let ca_pem = std::fs::read(ca_file).map_err(|source| Error::ReadCredentialFile {
                path: ca_file.clone(),
                source,
            })?;
            certs.push(Certificate::from_pem(&ca_pem).map_err(build_err)?);
        }

        if tls.include_system_ca_certs_pool.unwrap_or(true) {
            builder = builder.tls_certs_merge(certs);
        } else {
            builder = builder.tls_certs_only(certs);
        }

        let cert_configured = tls.config.cert_file.is_some()
            || tls
                .config
                .cert_pem
                .as_ref()
                .is_some_and(|p| !p.trim().is_empty());
        let key_configured = tls.config.key_file.is_some()
            || tls
                .config
                .key_pem
                .as_ref()
                .is_some_and(|p| !p.trim().is_empty());

        if cert_configured || key_configured {
            if !(cert_configured && key_configured) {
                return Err(Error::BuildHttpClient {
                    reason: "both a client certificate and key are required for mTLS".to_string(),
                });
            }
            let mut identity_pem = read_pem(tls.config.cert_file.as_ref(), tls.config.cert_pem.as_ref())?;
            identity_pem.extend_from_slice(&read_pem(
                tls.config.key_file.as_ref(),
                tls.config.key_pem.as_ref(),
            )?);
            builder = builder.identity(Identity::from_pem(&identity_pem).map_err(build_err)?);
        }
    }

    builder.build().map_err(build_err)
}

/// Reads PEM material from a file (preferred) or an inline string.
fn read_pem(file: Option<&PathBuf>, inline: Option<&String>) -> Result<Vec<u8>, Error> {
    if let Some(path) = file {
        return std::fs::read(path).map_err(|source| Error::ReadCredentialFile {
            path: path.clone(),
            source,
        });
    }
    if let Some(pem) = inline {
        return Ok(pem.as_bytes().to_vec());
    }
    Err(Error::BuildHttpClient {
        reason: "missing client certificate or key PEM".to_string(),
    })
}

/// Maps any displayable error into an [`Error::BuildHttpClient`].
fn build_err(error: impl std::fmt::Display) -> Error {
    Error::BuildHttpClient {
        reason: error.to_string(),
    }
}
