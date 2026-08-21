// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OAuth 2.0 client construction and token acquisition.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use oauth2::basic::{
    BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenType,
};
use oauth2::{
    AccessToken, AsyncHttpClient, Client, ClientId, ClientSecret, EndpointNotSet, HttpRequest,
    HttpResponse, RefreshToken, Scope, StandardRevocableToken, TokenResponse, TokenUrl,
};
use otap_df_engine::capability::auth::BearerToken;
use otap_df_otap::tls_utils::{read_file_with_limit_async, read_file_with_limit_sync};
use rand::RngExt;
use reqwest::{Certificate, Identity};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use super::config::{Config, GrantType, SignatureAlgorithm};
use super::error::Error;
use super::jwt_crypto;
use crate::common::token_refresh::TokenSource;

/// URN grant type sent to the token endpoint for the JWT-bearer grant.
const JWT_BEARER_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";
/// Validity window of a signed assertion. Kept short: the assertion is minted
/// per acquisition and only needs to survive the token request.
const ASSERTION_LIFETIME_SECS: u64 = 300;
/// Maximum number of bytes of a token endpoint's response body echoed back in
/// an error. The body is attacker-influenced and is logged on every refresh
/// failure, so it is truncated to keep it diagnostic rather than a log sink.
const MAX_ERROR_BODY_BYTES: usize = 256;

/// An OAuth 2.0 token source: a grant, token endpoint, scopes, and a
/// TLS-configured HTTP client used to reach the token endpoint.
#[derive(Clone)]
pub struct Auth {
    /// Grant used to acquire tokens.
    grant_type: GrantType,
    /// Parsed token endpoint URL.
    token_url: TokenUrl,
    /// Requested scopes.
    scopes: Vec<String>,
    /// Extra parameters appended to the token request body.
    endpoint_params: Vec<(String, String)>,
    /// Inline client identifier.
    client_id: Option<String>,
    /// Path to a file holding the client identifier (re-read each acquisition).
    client_id_file: Option<PathBuf>,
    /// Inline client secret (client-credentials grant).
    client_secret: Option<SecretString>,
    /// Path to a file holding the client secret (re-read each acquisition).
    client_secret_file: Option<PathBuf>,
    /// RSA algorithm used to sign the JWT-bearer assertion.
    signature_algorithm: SignatureAlgorithm,
    /// Inline signing key (PEM) for the JWT-bearer assertion.
    client_certificate_key: Option<SecretString>,
    /// Path to a file holding the signing key (re-read each acquisition).
    client_certificate_key_file: Option<PathBuf>,
    /// Optional `kid` header placed on the signed assertion.
    client_certificate_key_id: Option<String>,
    /// Assertion issuer (`iss`); defaults to the client id.
    iss: Option<String>,
    /// Assertion audience (`aud`); defaults to the token URL.
    audience: Option<String>,
    /// Extra claims added to the signed assertion.
    claims: Vec<(String, String)>,
    /// Assumed lifetime for a token whose response omits `expires_in`.
    default_token_lifetime: Duration,
    /// TLS-configured HTTP client used to reach the token endpoint.
    client: reqwest::Client,
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        // The reqwest/rustls HTTP client needs a process-wide crypto provider
        // installed before any TLS request is made.
        otap_df_otap::crypto::ensure_crypto_provider();

        // The JWT-bearer grant signs an assertion on every acquisition. Fail
        // here rather than letting `jsonwebtoken` panic at the first signature
        // when this build has no signing backend.
        if config.grant_type == GrantType::JwtBearer {
            if !jwt_crypto::SIGNING_AVAILABLE {
                return Err(Error::JwtSigning {
                    message: jwt_crypto::NO_BACKEND_MESSAGE.to_owned(),
                });
            }
            jwt_crypto::ensure_provider();
        }

        // OAuth 2.0 requires the token endpoint to be TLS-protected; warn (but
        // do not block) when a plaintext endpoint is configured.
        if config.token_url.starts_with("http://") {
            otel_warn!(
                "oauth2_client_auth.insecure_token_url",
                token_url = config.token_url.as_str()
            );
        }

        let token_url =
            TokenUrl::new(config.token_url.clone()).map_err(|e| Error::BuildHttpClient {
                reason: format!("invalid token_url: {e}"),
            })?;

        let client = build_reqwest_client(config)?;

        Ok(Self {
            grant_type: config.grant_type,
            token_url,
            scopes: config.scopes.clone(),
            endpoint_params: config
                .endpoint_params
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            client_id: config.client_id.clone(),
            client_id_file: config.client_id_file.clone(),
            client_secret: config.client_secret.clone(),
            client_secret_file: config.client_secret_file.clone(),
            signature_algorithm: config.signature_algorithm.unwrap_or_default(),
            client_certificate_key: config.client_certificate_key.clone(),
            client_certificate_key_file: config.client_certificate_key_file.clone(),
            client_certificate_key_id: config.client_certificate_key_id.clone(),
            iss: config.iss.clone(),
            audience: config.audience.clone(),
            claims: config
                .claims
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            default_token_lifetime: config.default_token_lifetime,
            client,
        })
    }

    /// Acquires a token using the client-credentials grant.
    async fn get_token_client_credentials(&self) -> Result<BearerToken, Error> {
        // Credentials are read fresh on each acquisition so the file forms can
        // rotate without a restart.
        let client_id = read_credential(
            self.client_id_file.as_ref(),
            self.client_id.as_deref(),
            "client_id",
        )
        .await?;
        let client_secret = read_credential(
            self.client_secret_file.as_ref(),
            self.client_secret.as_ref().map(ExposeSecret::expose_secret),
            "client_secret",
        )
        .await?;

        let client = TokenClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_token_uri(self.token_url.clone());

        let mut request = client.exchange_client_credentials();
        for scope in &self.scopes {
            request = request.add_scope(Scope::new(scope.clone()));
        }
        for (name, value) in &self.endpoint_params {
            request = request.add_extra_param(name.clone(), value.clone());
        }

        let executor = HttpExecutor {
            client: self.client.clone(),
        };
        let response =
            request
                .request_async(&executor)
                .await
                .map_err(|e| Error::TokenAcquisition {
                    message: e.to_string(),
                })?;

        to_bearer_token(&response, self.default_token_lifetime)
    }

    /// Acquires a token using the JWT-bearer grant (RFC 7523 section 2.1): it
    /// signs a JWT assertion and posts it to the token endpoint as the
    /// `assertion` parameter instead of authenticating with a secret.
    async fn get_token_jwt_bearer(&self) -> Result<BearerToken, Error> {
        // Credentials (client id + signing key) are read fresh on each
        // acquisition so the file forms can rotate without a restart.
        let client_id = read_credential(
            self.client_id_file.as_ref(),
            self.client_id.as_deref(),
            "client_id",
        )
        .await?;
        let key_pem = read_pem_credential(
            self.client_certificate_key_file.as_ref(),
            self.client_certificate_key
                .as_ref()
                .map(ExposeSecret::expose_secret),
            "client_certificate_key",
        )
        .await?;

        let assertion = self.sign_assertion(&client_id, &key_pem)?;

        // RFC 7523: the grant_type is the jwt-bearer URN and the signed JWT is
        // the `assertion`. Scopes and any endpoint params ride alongside.
        let mut form: Vec<(&str, String)> = vec![
            ("grant_type", JWT_BEARER_GRANT_TYPE.to_string()),
            ("assertion", assertion),
        ];
        if !self.scopes.is_empty() {
            form.push(("scope", self.scopes.join(" ")));
        }
        for (name, value) in &self.endpoint_params {
            form.push((name.as_str(), value.clone()));
        }

        let response = self
            .client
            .post(self.token_url.url().as_str())
            .form(&form)
            .send()
            .await
            .map_err(|e| Error::TokenAcquisition {
                message: format!("token endpoint request failed: {e}"),
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::TokenAcquisition {
                message: format!("token endpoint returned {status}: {}", truncate_body(&body)),
            });
        }

        let token: TokenEndpointResponse =
            response.json().await.map_err(|e| Error::TokenAcquisition {
                message: format!("invalid token response: {e}"),
            })?;
        to_bearer_token(&token, self.default_token_lifetime)
    }

    /// Builds and signs the JWT-bearer assertion for `client_id`.
    fn sign_assertion(&self, client_id: &str, key_pem: &[u8]) -> Result<String, Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::JwtSigning {
                message: format!("system clock is before the Unix epoch: {e}"),
            })?
            .as_secs();
        let jti: u128 = rand::rng().random();

        // Start from any operator-supplied extra claims, then set the standard
        // claims so they always take precedence.
        let mut claims = serde_json::Map::new();
        for (name, value) in &self.claims {
            let _ = claims.insert(name.clone(), serde_json::Value::String(value.clone()));
        }
        let iss = self.iss.clone().unwrap_or_else(|| client_id.to_owned());
        let aud = self
            .audience
            .clone()
            .unwrap_or_else(|| self.token_url.url().as_str().to_owned());
        let _ = claims.insert("iss".to_string(), serde_json::Value::String(iss));
        let _ = claims.insert(
            "sub".to_string(),
            serde_json::Value::String(client_id.to_owned()),
        );
        let _ = claims.insert("aud".to_string(), serde_json::Value::String(aud));
        let _ = claims.insert("iat".to_string(), serde_json::Value::from(now));
        let _ = claims.insert(
            "exp".to_string(),
            serde_json::Value::from(now + ASSERTION_LIFETIME_SECS),
        );
        let _ = claims.insert(
            "jti".to_string(),
            serde_json::Value::String(format!("{jti:032x}")),
        );

        let key = EncodingKey::from_rsa_pem(key_pem).map_err(|e| Error::JwtSigning {
            message: format!("invalid RSA signing key: {e}"),
        })?;
        let mut header = Header::new(jwt_algorithm(self.signature_algorithm));
        header.kid = self.client_certificate_key_id.clone();
        encode(&header, &serde_json::Value::Object(claims), &key).map_err(|e| Error::JwtSigning {
            message: e.to_string(),
        })
    }
}

#[async_trait]
impl TokenSource for Auth {
    type Error = Error;

    /// Acquires a single token (no retries) and converts it into a
    /// [`BearerToken`].
    async fn fetch_token(&self) -> Result<BearerToken, Error> {
        match self.grant_type {
            GrantType::ClientCredentials => self.get_token_client_credentials().await,
            GrantType::JwtBearer => self.get_token_jwt_bearer().await,
        }
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("oauth2_client_auth.token_refresh_failed", error = %error);
    }
}

/// Minimal OAuth 2.0 token endpoint response (RFC 6749 section 5.1).
///
/// Both grants deserialize into this one type: the client-credentials path via
/// the `oauth2` crate (which is generic over its response type) and the
/// JWT-bearer path by parsing the response body directly. Sharing it keeps the
/// two grants from accepting different payloads.
#[derive(Debug, Deserialize, Serialize)]
struct TokenEndpointResponse {
    /// The issued access token.
    access_token: AccessToken,
    /// Token type. Absent is treated as `Bearer`, matching what the Go
    /// implementation does; anything else is rejected rather than handed to
    /// consumers as a bearer token.
    #[serde(default = "bearer_token_type")]
    token_type: BasicTokenType,
    /// Relative lifetime in seconds, when the endpoint reports one. Accepted
    /// as a JSON number or a string: both appear in the wild, and the Go
    /// implementation accepts either.
    #[serde(default, deserialize_with = "deserialize_expires_in")]
    expires_in: Option<u64>,
    /// Refresh token, if the endpoint issues one. Unused: this extension only
    /// runs grants that can re-acquire from configured credentials.
    #[serde(default)]
    refresh_token: Option<RefreshToken>,
}

/// `token_type` default for responses that omit it.
fn bearer_token_type() -> BasicTokenType {
    BasicTokenType::Bearer
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

impl TokenResponse for TokenEndpointResponse {
    type TokenType = BasicTokenType;

    fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    fn token_type(&self) -> &Self::TokenType {
        &self.token_type
    }

    fn expires_in(&self) -> Option<Duration> {
        self.expires_in.map(Duration::from_secs)
    }

    fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }

    fn scopes(&self) -> Option<&Vec<Scope>> {
        None
    }
}

/// The `oauth2` client specialized to [`TokenEndpointResponse`], so the
/// client-credentials grant parses responses exactly as the JWT-bearer grant
/// does.
type TokenClient<
    HasAuthUrl = EndpointNotSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointNotSet,
> = Client<
    BasicErrorResponse,
    TokenEndpointResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
>;

/// Converts a token endpoint response into a [`BearerToken`].
///
/// Rejects a non-bearer `token_type` rather than presenting it as a bearer
/// token, and substitutes `fallback_lifetime` when the endpoint reports no
/// `expires_in`, so a token is never cached indefinitely.
fn to_bearer_token(
    response: &TokenEndpointResponse,
    fallback_lifetime: Duration,
) -> Result<BearerToken, Error> {
    // RFC 6749 section 7.1 makes `token_type` case-insensitive, but the
    // `oauth2` crate only recognizes the lowercase spelling, so normalize here.
    if !response.token_type.as_ref().eq_ignore_ascii_case("bearer") {
        return Err(Error::TokenAcquisition {
            message: format!(
                "token endpoint returned unsupported token_type `{}`; only Bearer is supported",
                response.token_type.as_ref()
            ),
        });
    }
    let secret = response.access_token().secret().to_owned();
    let expires_in = response.expires_in().unwrap_or(fallback_lifetime);
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

/// Maps the configured [`SignatureAlgorithm`] to the jsonwebtoken algorithm.
fn jwt_algorithm(alg: SignatureAlgorithm) -> Algorithm {
    match alg {
        SignatureAlgorithm::Rs256 => Algorithm::RS256,
        SignatureAlgorithm::Rs384 => Algorithm::RS384,
        SignatureAlgorithm::Rs512 => Algorithm::RS512,
    }
}

/// Reads a credential value, preferring the file form (re-read on each call so
/// the credential can rotate without a restart) over the inline value.
///
/// File reads go through the collector's shared size-limited reader: this runs
/// on the per-acquisition path, so an oversized or hostile path would otherwise
/// be re-read into memory on every refresh.
async fn read_credential(
    file: Option<&PathBuf>,
    inline: Option<&str>,
    field: &str,
) -> Result<String, Error> {
    if let Some(path) = file {
        let contents =
            read_file_with_limit_async(path)
                .await
                .map_err(|source| Error::ReadCredentialFile {
                    path: path.clone(),
                    source,
                })?;
        let contents = String::from_utf8(contents).map_err(|_| Error::TokenAcquisition {
            message: format!("`{field}_file` does not contain valid UTF-8"),
        })?;
        return Ok(contents.trim().to_owned());
    }
    if let Some(value) = inline {
        return Ok(value.to_owned());
    }
    Err(Error::TokenAcquisition {
        message: format!("no `{field}` or `{field}_file` configured"),
    })
}

/// Reads PEM key material, preferring the file form (re-read on each call for
/// rotation) over the inline value. Unlike [`read_credential`], the bytes are
/// returned verbatim so the PEM structure is preserved.
async fn read_pem_credential(
    file: Option<&PathBuf>,
    inline: Option<&str>,
    field: &str,
) -> Result<Vec<u8>, Error> {
    if let Some(path) = file {
        return read_file_with_limit_async(path).await.map_err(|source| {
            Error::ReadCredentialFile {
                path: path.clone(),
                source,
            }
        });
    }
    if let Some(value) = inline {
        return Ok(value.as_bytes().to_vec());
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
            let response = client
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
            builder.body(body).map_err(|e| Error::TokenAcquisition {
                message: format!("invalid token response: {e}"),
            })
        })
    }
}

/// Builds a reqwest client for the token endpoint from the extension's shared
/// TLS config, mirroring how the OTLP/HTTP exporter builds its client.
fn build_reqwest_client(config: &Config) -> Result<reqwest::Client, Error> {
    let mut builder = reqwest::Client::builder()
        .use_rustls_tls()
        .timeout(config.timeout)
        .connect_timeout(config.connect_timeout);

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
            let ca_pem =
                read_file_with_limit_sync(ca_file).map_err(|source| Error::ReadCredentialFile {
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
            let mut identity_pem =
                read_pem(tls.config.cert_file.as_ref(), tls.config.cert_pem.as_ref())?;
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
        return read_file_with_limit_sync(path).map_err(|source| Error::ReadCredentialFile {
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
