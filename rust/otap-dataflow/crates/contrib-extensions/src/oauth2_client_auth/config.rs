// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the OAuth 2.0 Client Auth extension.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use otap_df_config::tls::TlsClientConfig;
use secrecy::SecretString;
use serde::Deserialize;

/// Default duration ahead of expiry at which a token is refreshed.
fn default_expiry_buffer() -> Duration {
    Duration::from_secs(300)
}

/// Default startup readiness timeout.
///
/// Larger than the engine's 5 s readiness-probe default: a cold-start token
/// acquisition against a slow token endpoint can exceed 5 s, and a failed first
/// attempt is retried on a ~10 s cadence, so the gate must allow room for a
/// retry.
fn default_startup_timeout() -> Duration {
    Duration::from_secs(30)
}

/// OAuth 2.0 grant used to acquire tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum GrantType {
    /// Client-credentials grant (RFC 6749 section 4.4): the client
    /// authenticates with a client id + secret and receives an access token.
    #[serde(rename = "client_credentials")]
    #[default]
    ClientCredentials,
    /// JWT-bearer authorization grant (RFC 7523 section 2.1): the client signs
    /// a JWT and sends it as the `assertion` parameter instead of a secret.
    #[serde(rename = "jwt-bearer")]
    JwtBearer,
}

impl GrantType {
    /// Returns a stable, human-readable name for the grant.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            GrantType::ClientCredentials => "client_credentials",
            GrantType::JwtBearer => "jwt-bearer",
        }
    }
}

impl std::fmt::Display for GrantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// RSA algorithm used to sign the JWT-bearer assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum SignatureAlgorithm {
    /// RSASSA-PKCS1-v1_5 using SHA-256.
    #[serde(rename = "RS256")]
    #[default]
    Rs256,
    /// RSASSA-PKCS1-v1_5 using SHA-384.
    #[serde(rename = "RS384")]
    Rs384,
    /// RSASSA-PKCS1-v1_5 using SHA-512.
    #[serde(rename = "RS512")]
    Rs512,
}

/// Configuration for the OAuth 2.0 Client Auth extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Grant type used to acquire tokens.
    #[serde(default)]
    pub grant_type: GrantType,

    /// Token endpoint URL. Must be non-empty.
    pub token_url: String,

    /// Client identifier. Required unless `client_id_file` is set.
    #[serde(default)]
    pub client_id: Option<String>,

    /// Path to a file holding the client identifier. Re-read on each
    /// acquisition and takes precedence over `client_id`, so the credential can
    /// rotate without a restart.
    #[serde(default)]
    pub client_id_file: Option<PathBuf>,

    /// Client secret. Required for `client_credentials` unless
    /// `client_secret_file` is set.
    ///
    /// Held as a [`SecretString`] so it is redacted from `Debug` output.
    /// Note that the raw pipeline config retains the cleartext;
    /// prefer `client_secret_file`.
    #[serde(default)]
    pub client_secret: Option<SecretString>,

    /// Path to a file holding the client secret. Re-read on each acquisition
    /// and takes precedence over `client_secret`.
    #[serde(default)]
    pub client_secret_file: Option<PathBuf>,

    /// Scopes requested from the token endpoint.
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Extra parameters sent to the token endpoint (e.g. `audience`).
    #[serde(default)]
    pub endpoint_params: HashMap<String, String>,

    /// Refresh this far ahead of the token's expiry. Accepts human-readable
    /// durations (e.g. `5m`, `30s`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_expiry_buffer")]
    pub expiry_buffer: Duration,

    /// Optional per-request timeout on the token client. Accepts human-readable
    /// durations. When omitted, no timeout is applied.
    #[serde(with = "humantime_serde", default)]
    pub timeout: Option<Duration>,

    /// Client TLS for the token endpoint. Uses the engine's shared
    /// [`TlsClientConfig`] so behavior matches the rest of the collector.
    #[serde(default)]
    pub tls: Option<TlsClientConfig>,

    /// How long the engine holds data-path node startup waiting for this
    /// extension to publish its first token, before aborting pipeline startup.
    /// Accepts human-readable durations (e.g. `30s`, `1m`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_startup_timeout")]
    pub startup_timeout: Duration,

    // -- JWT-bearer grant fields (rejected for `client_credentials`) --
    /// Private key (PEM) used to sign the JWT-bearer assertion. Required for
    /// the `jwt-bearer` grant unless `client_certificate_key_file` is set.
    ///
    /// Held as a [`SecretString`] so it is redacted from `Debug` output.
    /// Note that the raw pipeline config retains the cleartext;
    /// prefer `client_certificate_key_file`.
    #[serde(default)]
    pub client_certificate_key: Option<SecretString>,

    /// Path to a file holding the signing key. Re-read on each acquisition and
    /// takes precedence over `client_certificate_key`.
    #[serde(default)]
    pub client_certificate_key_file: Option<PathBuf>,

    /// RSA algorithm used to sign the assertion. Defaults to `RS256`.
    #[serde(default)]
    pub signature_algorithm: Option<SignatureAlgorithm>,

    /// Optional `kid` header placed on the signed assertion.
    #[serde(default)]
    pub client_certificate_key_id: Option<String>,

    /// Assertion issuer (`iss` claim). Defaults to `client_id`.
    #[serde(default)]
    pub iss: Option<String>,

    /// Assertion audience (`aud` claim). Defaults to `token_url`.
    #[serde(default)]
    pub audience: Option<String>,

    /// Extra claims added to the signed assertion.
    #[serde(default)]
    pub claims: HashMap<String, String>,
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    ///
    /// Rejects an empty `token_url`, a zero `expiry_buffer`/`startup_timeout`,
    /// and a missing client identifier or secret for the selected grant.
    pub fn validate(&self) -> Result<(), String> {
        if self.token_url.trim().is_empty() {
            return Err("`token_url` must not be empty".to_string());
        }

        if self.expiry_buffer.is_zero() {
            return Err("`expiry_buffer` must be greater than zero".to_string());
        }

        if self.startup_timeout.is_zero() {
            return Err("`startup_timeout` must be greater than zero".to_string());
        }

        // The reqwest/rustls token client (like the OTLP/HTTP exporter) cannot
        // override the TLS SNI, so reject `server_name_override` at config time
        // rather than silently connecting with the wrong server name.
        if let Some(tls) = &self.tls {
            if tls.server_name.is_some() {
                return Err(
                    "`tls.server_name_override` is not supported by the OAuth2 token client"
                        .to_string(),
                );
            }
        }

        // A client identifier is required for every grant; it may be supplied
        // inline or via a file that is re-read on each acquisition.
        if self.client_id.is_none() && self.client_id_file.is_none() {
            return Err("either `client_id` or `client_id_file` must be set".to_string());
        }

        // Grant-specific rules: each grant requires its own credential material
        // and rejects the fields that belong only to the other grant.
        let jwt_fields_set = self.client_certificate_key.is_some()
            || self.client_certificate_key_file.is_some()
            || self.signature_algorithm.is_some()
            || self.client_certificate_key_id.is_some()
            || self.iss.is_some()
            || self.audience.is_some()
            || !self.claims.is_empty();
        let secret_fields_set = self.client_secret.is_some() || self.client_secret_file.is_some();

        match self.grant_type {
            GrantType::ClientCredentials => {
                if !secret_fields_set {
                    return Err(
                        "either `client_secret` or `client_secret_file` must be set for the \
                         `client_credentials` grant"
                            .to_string(),
                    );
                }
                if jwt_fields_set {
                    return Err(
                        "`jwt-bearer` fields (`client_certificate_key`, `signature_algorithm`, \
                         `client_certificate_key_id`, `iss`, `audience`, `claims`) are not valid \
                         for the `client_credentials` grant"
                            .to_string(),
                    );
                }
            }
            GrantType::JwtBearer => {
                if self.client_certificate_key.is_none()
                    && self.client_certificate_key_file.is_none()
                {
                    return Err(
                        "either `client_certificate_key` or `client_certificate_key_file` must be \
                         set for the `jwt-bearer` grant"
                            .to_string(),
                    );
                }
                if secret_fields_set {
                    return Err(
                        "`client_secret`/`client_secret_file` are not valid for the `jwt-bearer` \
                         grant"
                            .to_string(),
                    );
                }
            }
        }

        Ok(())
    }
}
