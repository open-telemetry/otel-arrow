// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP/HTTPS proxy support for gRPC clients.
//!
//! This module provides proxy configuration and utilities for routing gRPC
//! connections through corporate HTTP/HTTPS proxies. It reads proxy configuration
//! from standard environment variables:
//!
//! - `HTTP_PROXY` / `http_proxy`: Proxy for HTTP connections
//! - `HTTPS_PROXY` / `https_proxy`: Proxy for HTTPS connections
//! - `ALL_PROXY` / `all_proxy`: Fallback proxy for all connections
//! - `NO_PROXY` / `no_proxy`: Comma-separated list of hosts to bypass proxy
//!
//! The implementation uses HTTP CONNECT method to establish tunnels through proxies.
//! It supports both:
//! - `http://proxy:port` (plaintext to proxy)
//! - `https://proxy:port` (TLS to proxy, requires `experimental-tls`)

use crate::socket_options;
use base64::Engine;
use base64::prelude::*;
use http::Uri;
use ipnet::IpNet;
#[cfg(feature = "experimental-tls")]
use otap_df_config::tls::TlsClientConfig;
use otap_df_telemetry::{otel_debug, otel_warn};
#[cfg(feature = "experimental-tls")]
use rustls::RootCertStore;
#[cfg(feature = "experimental-tls")]
use rustls_native_certs::load_native_certs;
#[cfg(feature = "experimental-tls")]
use rustls_pki_types::pem::PemObject;
#[cfg(feature = "experimental-tls")]
use rustls_pki_types::{CertificateDer, ServerName};
use serde::Deserialize;
use std::borrow::Cow;
use std::env;
use std::io;
use std::net::IpAddr;
#[cfg(feature = "experimental-tls")]
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
#[cfg(feature = "experimental-tls")]
use tokio::sync::OnceCell;
#[cfg(feature = "experimental-tls")]
use tokio_rustls::TlsConnector;

#[cfg(feature = "experimental-tls")]
use crate::tls_utils::read_file_with_limit_async;

/// A URL-like string that should be treated as sensitive.
///
/// `Debug` and `Display` redact credentials by default.
/// Use [`SensitiveUrl::expose`] when the original value is required.
#[derive(Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct SensitiveUrl(String);

impl SensitiveUrl {
    /// Creates a new sensitive URL wrapper.
    ///
    /// Note: this does not validate the URL; validation (if needed) happens at use sites.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the underlying URL string.
    ///
    /// Prefer using `Display`/`Debug` when logging.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    fn redacted(&self) -> Cow<'_, str> {
        let url = self.expose();

        // Redact credentials in the authority if present.
        // This intentionally mirrors the existing behavior of `ProxyConfig`'s logging.
        if let Ok(uri) = url.parse::<Uri>() {
            if let Some(authority) = uri.authority() {
                let auth_str = authority.as_str();
                if let Some((_, host)) = auth_str.rsplit_once('@') {
                    return Cow::Owned(url.replace(auth_str, &format!("[REDACTED]@{host}")));
                }
            }
        }

        if let Some((_, host)) = url.rsplit_once('@') {
            return Cow::Owned(format!("[REDACTED]@{host}"));
        }

        Cow::Borrowed(url)
    }
}

impl std::fmt::Display for SensitiveUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.redacted().as_ref())
    }
}

impl std::fmt::Debug for SensitiveUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SensitiveUrl")
            .field(&self.redacted())
            .finish()
    }
}

impl From<String> for SensitiveUrl {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for SensitiveUrl {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Errors that can occur during proxy connection.
#[derive(Debug, Error)]
pub enum ProxyError {
    /// Invalid proxy URL format
    #[error("invalid proxy URL: {0}")]
    InvalidProxyUrl(String),

    /// TCP connection to proxy failed
    #[error("failed to connect to proxy: {0}")]
    ProxyConnectionFailed(io::Error),

    /// TCP connection to target failed (when no proxy is configured)
    #[error("failed to connect to target: {0}")]
    TargetConnectionFailed(io::Error),

    /// HTTP CONNECT request failed
    #[error("HTTP CONNECT failed with status {status}: {message}")]
    ConnectFailed {
        /// HTTP status code returned by proxy
        status: u16,
        /// Status message from proxy
        message: String,
    },

    /// Invalid HTTP response from proxy
    #[error("invalid HTTP response from proxy: {0}")]
    InvalidResponse(String),

    /// Invalid target URI
    #[error("invalid target URI: {0}")]
    InvalidUri(String),

    /// TLS handshake or setup failure while connecting to an HTTPS proxy.
    #[cfg(feature = "experimental-tls")]
    #[error("failed to establish TLS connection to proxy: {0}")]
    ProxyTlsHandshake(String),
}

/// Proxy configuration that can be set explicitly or read from environment.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[doc(hidden)]
pub struct ProxyConfig {
    /// HTTP proxy URL (e.g., "http://proxy.example.com:3128")
    /// If not set, reads from HTTP_PROXY/http_proxy environment variable.
    #[serde(default)]
    pub http_proxy: Option<SensitiveUrl>,

    /// Proxy URL to use for *HTTPS targets* (e.g., "http://proxy.example.com:3128"
    /// or "https://proxy.example.com:3128").
    /// If not set, reads from HTTPS_PROXY/https_proxy environment variable.
    #[serde(default)]
    pub https_proxy: Option<SensitiveUrl>,

    /// Fallback proxy URL for all protocols.
    /// If not set, reads from ALL_PROXY/all_proxy environment variable.
    ///
    /// Note: This is the proxy server URL (not a signal to use TLS to the proxy).
    #[serde(default)]
    pub all_proxy: Option<SensitiveUrl>,

    /// Comma-separated list of hosts that should bypass the proxy.
    /// Supports wildcards (e.g., "*.local,localhost,127.0.0.1").
    /// If not set, reads from NO_PROXY/no_proxy environment variable.
    #[serde(default)]
    pub no_proxy: Option<String>,

    /// TLS/mTLS settings for HTTPS proxy connections (`https://proxy-host:port`).
    ///
    /// This setting is ignored for `http://` proxy URLs.
    /// Requires the `experimental-tls` feature.
    #[cfg(feature = "experimental-tls")]
    #[serde(default)]
    pub tls: Option<TlsClientConfig>,
}

impl ProxyConfig {
    /// Creates a new ProxyConfig that reads from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            http_proxy: env::var("HTTP_PROXY")
                .or_else(|_| env::var("http_proxy"))
                .ok()
                .filter(|s| !s.is_empty())
                .map(SensitiveUrl::from),
            https_proxy: env::var("HTTPS_PROXY")
                .or_else(|_| env::var("https_proxy"))
                .ok()
                .filter(|s| !s.is_empty())
                .map(SensitiveUrl::from),
            all_proxy: env::var("ALL_PROXY")
                .or_else(|_| env::var("all_proxy"))
                .ok()
                .filter(|s| !s.is_empty())
                .map(SensitiveUrl::from),
            no_proxy: env::var("NO_PROXY")
                .or_else(|_| env::var("no_proxy"))
                .ok()
                .filter(|s| !s.is_empty()),
            #[cfg(feature = "experimental-tls")]
            tls: None,
        }
    }

    /// Merges explicit config with environment variables.
    /// Explicit values take precedence over environment variables.
    #[must_use]
    pub fn merge_with_env(self) -> Self {
        let env_config = Self::from_env();
        Self {
            http_proxy: self.http_proxy.or(env_config.http_proxy),
            https_proxy: self.https_proxy.or(env_config.https_proxy),
            all_proxy: self.all_proxy.or(env_config.all_proxy),
            no_proxy: self.no_proxy.or(env_config.no_proxy),
            #[cfg(feature = "experimental-tls")]
            tls: self.tls,
        }
    }
}

impl std::fmt::Display for ProxyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("ProxyConfig");
        let _ = dbg.field("http_proxy", &self.http_proxy);
        let _ = dbg.field("https_proxy", &self.https_proxy);
        let _ = dbg.field("all_proxy", &self.all_proxy);
        let _ = dbg.field("no_proxy", &self.no_proxy);
        #[cfg(feature = "experimental-tls")]
        let _ = dbg.field("tls", &self.tls.as_ref().map(|_| "[configured]"));
        dbg.finish()
    }
}

pub(crate) trait AsyncReadWrite: AsyncRead + AsyncWrite {}
impl<T: AsyncRead + AsyncWrite + ?Sized> AsyncReadWrite for T {}

/// Connected proxy stream (plain TCP or TLS-wrapped TCP).
///
/// We intentionally use a boxed trait object for simplicity and extensibility.
/// This adds one allocation per connection and dynamic dispatch per I/O poll.
/// In this path, gRPC runs over long-lived HTTP/2 connections, so that overhead is
/// usually amortized and negligible compared with network/TLS work.
///
/// If profiling shows this as a bottleneck, switch to an enum-based stream type
/// to remove allocation/dispatch overhead.
pub(crate) type ProxyTcpStream = Box<dyn AsyncReadWrite + Send + Unpin>;

impl ProxyConfig {
    /// Returns the proxy URL for a given target URI, or None if no proxy should be used.
    #[must_use]
    pub(crate) fn get_proxy_for_uri(&self, uri: &Uri) -> Option<&str> {
        let host = uri.host().unwrap_or("");

        let scheme = uri.scheme_str().unwrap_or("http");
        let port = uri.port_u16().unwrap_or(match scheme {
            "https" => 443,
            _ => 80,
        });

        // Check if host should bypass proxy
        if self.should_bypass(host, port) {
            otel_debug!("otap_grpc_exporter.proxy.bypass", host = host, port = port);
            return None;
        }

        // Select proxy based on scheme
        let proxy = match scheme {
            "https" => self.https_proxy.as_ref().or(self.all_proxy.as_ref()),
            _ => self.http_proxy.as_ref().or(self.all_proxy.as_ref()),
        };

        if let Some(url) = proxy {
            otel_debug!(
                "otap_grpc_exporter.proxy.using",
                proxy = url.to_string(),
                target = uri.to_string(),
            );
        } else {
            otel_debug!("otap_grpc_exporter.proxy.none", target = uri.to_string());
        }

        proxy.map(|u| u.expose())
    }

    /// Checks if a host should bypass the proxy based on NO_PROXY rules.
    ///
    /// Supports:
    /// - "*" - matches all hosts
    /// - "*.example.com" - wildcard domain matching
    /// - ".example.com" - domain suffix matching
    /// - "example.com" - exact hostname matching
    /// - "192.168.1.1" - exact IP matching
    /// - "192.168.0.0/16" - CIDR notation for IP ranges
    /// - "example.com:443" - exact hostname match with a specific port
    /// - "[::1]:4317" - IPv6 literal match with a specific port
    fn should_bypass(&self, host: &str, port: u16) -> bool {
        let no_proxy = match &self.no_proxy {
            Some(np) => np,
            None => return false,
        };

        fn split_host_and_port(pattern: &str) -> (Cow<'_, str>, Option<u16>) {
            // Bracketed IPv6: [::1]:4317
            if let Some(rest) = pattern.strip_prefix('[') {
                if let Some(end) = rest.find(']') {
                    let host = &rest[..end];
                    let after = &rest[end + 1..];
                    if let Some(port_str) = after.strip_prefix(':') {
                        if !port_str.is_empty() && port_str.chars().all(|c| c.is_ascii_digit()) {
                            if let Ok(p) = port_str.parse::<u16>() {
                                return (Cow::Borrowed(host), Some(p));
                            }
                        }
                    }
                    return (Cow::Borrowed(host), None);
                }
            }

            // CIDRs never have ports.
            if pattern.contains('/') {
                return (Cow::Borrowed(pattern), None);
            }

            // Hostname/IPv4 with port: example.com:443, 127.0.0.1:4317
            if let Some((lhs, rhs)) = pattern.rsplit_once(':') {
                if !lhs.contains(':')
                    && !rhs.is_empty()
                    && rhs.chars().all(|c| c.is_ascii_digit())
                    && rhs.len() <= 5
                {
                    if let Ok(p) = rhs.parse::<u16>() {
                        return (Cow::Borrowed(lhs), Some(p));
                    }
                }
            }

            (Cow::Borrowed(pattern), None)
        }

        // TODO(perf): Pre-parse NO_PROXY rules to avoid allocations in hot path
        // Normalize host:
        // - accept bracketed IPv6 literals from `http::Uri::host()` (e.g. "[::1]")
        // - accept absolute FQDNs with trailing dot (e.g. "example.com.")
        let host_norm = host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim_end_matches('.');
        let host_lower = host_norm.to_lowercase();

        // Try to parse host as an IP address for CIDR matching
        let host_ip = host_norm.parse::<IpAddr>().ok();

        for pattern in no_proxy.split(',') {
            let pattern = pattern.trim().to_lowercase();
            if pattern.is_empty() {
                continue;
            }

            let (pattern_host, pattern_port) = split_host_and_port(&pattern);
            if let Some(pattern_port) = pattern_port {
                if pattern_port != port {
                    continue;
                }
            }

            let pattern_host = pattern_host.as_ref().trim_end_matches('.');

            // Handle "*" wildcard for all hosts
            if pattern_host == "*" {
                return true;
            }

            // Handle CIDR notation (e.g., "192.168.0.0/16", "10.0.0.0/8")
            if pattern_host.contains('/') {
                if let Ok(net) = pattern_host.parse::<IpNet>() {
                    if let Some(ip) = host_ip {
                        if net.contains(&ip) {
                            return true;
                        }
                    }
                } else {
                    otap_df_telemetry::otel_warn!(
                        "otap_grpc_exporter.proxy.invalid_cidr",
                        pattern = pattern_host
                    );
                }
                continue;
            }

            // Handle wildcard prefix patterns like "*.example.com"
            if let Some(suffix) = pattern_host.strip_prefix("*.") {
                if host_lower.ends_with(&format!(".{suffix}")) || host_lower == suffix {
                    return true;
                }
            } else if let Some(suffix) = pattern_host.strip_prefix('.') {
                // Handle ".example.com" (matches subdomains and the domain itself)
                if host_lower.ends_with(pattern_host) || host_lower == suffix {
                    return true;
                }
            } else if host_lower == pattern_host {
                // Exact match (hostname or IP)
                return true;
            }
        }

        false
    }

    /// Returns true if any proxy is configured.
    ///
    /// Note: This returns true if *any* proxy URL is set, regardless of whether
    /// the current target matches a NO_PROXY rule. The bypass logic is handled
    /// per-request in `get_proxy_for_uri`.
    #[must_use]
    pub const fn has_proxy(&self) -> bool {
        self.http_proxy.is_some() || self.https_proxy.is_some() || self.all_proxy.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProxyScheme {
    Http,
    #[cfg(feature = "experimental-tls")]
    Https,
}

/// Parsed proxy URL parts.
#[derive(Debug)]
struct ParsedProxyUrl {
    host: String,
    port: u16,
    auth_header: Option<String>,
    scheme: ProxyScheme,
}

/// Parses a proxy URL and returns host/port/auth/scheme.
fn parse_proxy_url(proxy_url: &str) -> Result<ParsedProxyUrl, ProxyError> {
    // Avoid leaking credentials in error messages.
    let proxy_url_redacted = SensitiveUrl::new(proxy_url).to_string();

    let uri: Uri = proxy_url.parse().map_err(|_| {
        ProxyError::InvalidProxyUrl(format!("failed to parse proxy URL ({proxy_url_redacted})"))
    })?;

    let scheme = match uri.scheme_str().unwrap_or("http") {
        "http" => ProxyScheme::Http,
        "https" => {
            #[cfg(feature = "experimental-tls")]
            {
                ProxyScheme::Https
            }
            #[cfg(not(feature = "experimental-tls"))]
            {
                return Err(ProxyError::InvalidProxyUrl(format!(
                    "https:// proxy URLs require the `experimental-tls` feature (proxy URL: {proxy_url_redacted})"
                )));
            }
        }
        other => {
            return Err(ProxyError::InvalidProxyUrl(format!(
                "unsupported proxy URL scheme `{other}` ({proxy_url_redacted})"
            )));
        }
    };

    let host = uri
        .host()
        .ok_or_else(|| {
            ProxyError::InvalidProxyUrl(format!("missing host in proxy URL ({proxy_url_redacted})"))
        })?
        .to_string();

    let port = uri.port_u16().unwrap_or(match scheme {
        ProxyScheme::Http => 3128,
        #[cfg(feature = "experimental-tls")]
        ProxyScheme::Https => 443,
    });

    // Extract credentials if present
    let auth_header = if let Some(authority) = uri.authority() {
        let auth_str = authority.as_str();
        if let Some((user_pass, _)) = auth_str.rsplit_once('@') {
            let encoded = BASE64_STANDARD.encode(user_pass);
            Some(format!("Basic {}", encoded))
        } else {
            None
        }
    } else {
        None
    };

    Ok(ParsedProxyUrl {
        host,
        port,
        auth_header,
        scheme,
    })
}

#[cfg(feature = "experimental-tls")]
fn tls_server_name_for_proxy(
    proxy_host: &str,
    tls: Option<&TlsClientConfig>,
) -> Result<ServerName<'static>, ProxyError> {
    let name = tls
        .and_then(|cfg| cfg.server_name.as_deref())
        .unwrap_or(proxy_host);

    if let Ok(ip) = name.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip.into()));
    }

    ServerName::try_from(name.to_string()).map_err(|e| {
        ProxyError::InvalidProxyUrl(format!(
            "invalid proxy server name `{name}` for TLS verification: {e}"
        ))
    })
}

#[cfg(feature = "experimental-tls")]
async fn read_proxy_tls_file(path: &Path, field: &str) -> Result<Vec<u8>, ProxyError> {
    read_file_with_limit_async(path).await.map_err(|e| {
        ProxyError::InvalidProxyUrl(format!(
            "failed to read proxy.tls.{field} from {}: {e}",
            path.display()
        ))
    })
}

#[cfg(feature = "experimental-tls")]
async fn load_proxy_system_root_certs() -> Result<Vec<CertificateDer<'static>>, ProxyError> {
    // Cache system roots for the process lifetime to avoid repeated blocking OS/keychain reads.
    static SYSTEM_ROOTS: OnceCell<Vec<CertificateDer<'static>>> = OnceCell::const_new();

    let roots = SYSTEM_ROOTS
        .get_or_try_init(|| async {
            let native = tokio::task::spawn_blocking(load_native_certs)
                .await
                .map_err(|e| {
                    ProxyError::InvalidProxyUrl(format!(
                        "failed to load system CA certificates (join error): {e}"
                    ))
                })?;

            for error in native.errors {
                otel_warn!("otap_grpc_exporter.proxy.tls.native_cert.load_error", error = ?error);
            }

            Ok::<Vec<CertificateDer<'static>>, ProxyError>(native.certs)
        })
        .await?;

    Ok(roots.clone())
}

#[cfg(feature = "experimental-tls")]
async fn build_proxy_tls_connector(
    proxy_host: &str,
    tls: Option<&TlsClientConfig>,
) -> Result<(TlsConnector, ServerName<'static>), ProxyError> {
    if let Some(cfg) = tls {
        if cfg.insecure.unwrap_or(false) {
            return Err(ProxyError::InvalidProxyUrl(
                "proxy.tls.insecure=true is not supported for https:// proxies; use http:// proxy URL for plaintext proxy connections".to_string(),
            ));
        }
        if cfg.insecure_skip_verify.unwrap_or(false) {
            return Err(ProxyError::InvalidProxyUrl(
                "proxy.tls.insecure_skip_verify=true is not supported".to_string(),
            ));
        }
    }

    if let Err(err) = crate::crypto::install_crypto_provider() {
        // It is expected to fail once a provider is already installed.
        otel_debug!(
            "otap_grpc_exporter.proxy.tls.provider.install",
            error = ?err,
            message = "rustls provider install returned non-fatal error"
        );
    }

    let mut roots = RootCertStore::empty();
    let include_system = tls
        .and_then(|cfg| cfg.include_system_ca_certs_pool)
        .unwrap_or(true);

    if include_system {
        for cert in load_proxy_system_root_certs().await? {
            if let Err(err) = roots.add(cert) {
                otel_warn!(
                    "otap_grpc_exporter.proxy.tls.native_cert.add_error",
                    error = ?err
                );
            }
        }
    }

    if let Some(cfg) = tls {
        if let Some(ca_file) = &cfg.ca_file {
            let ca_pem = read_proxy_tls_file(ca_file.as_path(), "ca_file").await?;
            for cert in CertificateDer::pem_slice_iter(&ca_pem) {
                let cert = cert.map_err(|e| {
                    ProxyError::InvalidProxyUrl(format!(
                        "failed to parse proxy.tls.ca_file PEM certificate: {e}"
                    ))
                })?;
                roots.add(cert).map_err(|e| {
                    ProxyError::InvalidProxyUrl(format!(
                        "failed to add proxy.tls.ca_file certificate to root store: {e}"
                    ))
                })?;
            }
        }

        if let Some(ca_pem) = &cfg.ca_pem {
            for cert in CertificateDer::pem_slice_iter(ca_pem.as_bytes()) {
                let cert = cert.map_err(|e| {
                    ProxyError::InvalidProxyUrl(format!(
                        "failed to parse proxy.tls.ca_pem certificate: {e}"
                    ))
                })?;
                roots.add(cert).map_err(|e| {
                    ProxyError::InvalidProxyUrl(format!(
                        "failed to add proxy.tls.ca_pem certificate to root store: {e}"
                    ))
                })?;
            }
        }
    }

    if roots.is_empty() {
        return Err(ProxyError::InvalidProxyUrl(
            "no trust anchors available for HTTPS proxy TLS verification; configure proxy.tls.ca_file/ca_pem or enable system CAs".to_string(),
        ));
    }

    let builder = rustls::ClientConfig::builder().with_root_certificates(roots);
    let client_config = if let Some(cfg) = tls {
        let cert_configured = cfg.config.cert_file.is_some()
            || cfg
                .config
                .cert_pem
                .as_ref()
                .is_some_and(|pem| !pem.trim().is_empty());
        let key_configured = cfg.config.key_file.is_some()
            || cfg
                .config
                .key_pem
                .as_ref()
                .is_some_and(|pem| !pem.trim().is_empty());

        if cert_configured || key_configured {
            if !(cert_configured && key_configured) {
                return Err(ProxyError::InvalidProxyUrl(
                    "proxy.tls mTLS is partially configured: both certificate and key must be provided"
                        .to_string(),
                ));
            }

            let cert_pem = match (&cfg.config.cert_file, &cfg.config.cert_pem) {
                (Some(path), _) => read_proxy_tls_file(path.as_path(), "cert_file").await?,
                (None, Some(pem)) => pem.as_bytes().to_vec(),
                (None, None) => {
                    return Err(ProxyError::InvalidProxyUrl(
                        "proxy.tls certificate is required for mTLS but missing".to_string(),
                    ));
                }
            };

            let key_pem = match (&cfg.config.key_file, &cfg.config.key_pem) {
                (Some(path), _) => read_proxy_tls_file(path.as_path(), "key_file").await?,
                (None, Some(pem)) => pem.as_bytes().to_vec(),
                (None, None) => {
                    return Err(ProxyError::InvalidProxyUrl(
                        "proxy.tls key is required for mTLS but missing".to_string(),
                    ));
                }
            };

            let cert_chain: Vec<_> = CertificateDer::pem_slice_iter(&cert_pem)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    ProxyError::InvalidProxyUrl(format!(
                        "failed to parse proxy.tls client certificate PEM: {e}"
                    ))
                })?;
            let private_key =
                rustls_pki_types::PrivateKeyDer::from_pem_slice(&key_pem).map_err(|e| {
                    ProxyError::InvalidProxyUrl(format!(
                        "failed to parse proxy.tls client key PEM: {e}"
                    ))
                })?;

            builder
                .with_client_auth_cert(cert_chain, private_key)
                .map_err(|e| {
                    ProxyError::InvalidProxyUrl(format!(
                        "failed to configure proxy.tls client certificate authentication: {e}"
                    ))
                })?
        } else {
            builder.with_no_client_auth()
        }
    } else {
        builder.with_no_client_auth()
    };

    let server_name = tls_server_name_for_proxy(proxy_host, tls)?;
    Ok((
        TlsConnector::from(std::sync::Arc::new(client_config)),
        server_name,
    ))
}

/// Establishes an HTTP CONNECT tunnel through a proxy.
///
/// This function connects to the proxy, sends an HTTP CONNECT request,
/// and returns the TCP stream once the tunnel is established.
#[cfg(test)]
async fn http_connect_tunnel(
    proxy_host: &str,
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, ProxyError> {
    // Connect to the proxy
    let stream = TcpStream::connect((proxy_host, proxy_port))
        .await
        .map_err(ProxyError::ProxyConnectionFailed)?;

    http_connect_tunnel_on_stream(stream, target_host, target_port, None).await
}

async fn http_connect_tunnel_on_stream<S>(
    mut stream: S,
    target_host: &str,
    target_port: u16,
    proxy_auth: Option<&str>,
) -> Result<S, ProxyError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // Send HTTP CONNECT request
    // Note: We use "Connection: Keep-Alive" instead of "Proxy-Connection" as the latter
    // is non-standard, although widely supported. Modern proxies should respect "Connection".

    // Handle IPv6 addresses in Host header (must be bracketed)
    let formatted_target = if target_host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{}]", target_host)
    } else {
        target_host.to_string()
    };

    // TODO(proxy): make User-Agent configurable via ProxyConfig / exporter settings.
    let mut connect_request = format!(
        "CONNECT {formatted_target}:{target_port} HTTP/1.1\r\n\
         Host: {formatted_target}:{target_port}\r\n\
         User-Agent: otap-dataflow\r\n\
         Connection: Keep-Alive\r\n"
    );

    if let Some(auth) = proxy_auth {
        connect_request.push_str(&format!("Proxy-Authorization: {}\r\n", auth));
    }

    connect_request.push_str("\r\n");

    // Avoid logging the raw request to reduce the risk of leaking headers or internal targets.
    otel_debug!(
        "otap_grpc_exporter.proxy.connect_request",
        target = format!("{formatted_target}:{target_port}"),
        has_auth = proxy_auth.is_some()
    );

    stream
        .write_all(connect_request.as_bytes())
        .await
        .map_err(ProxyError::ProxyConnectionFailed)?;

    // Read the response status line
    let mut buf_reader = BufReader::new(stream);
    let mut status_line = String::new();
    if buf_reader
        .read_line(&mut status_line)
        .await
        .map_err(ProxyError::ProxyConnectionFailed)?
        == 0
    {
        return Err(ProxyError::InvalidResponse(
            "unexpected EOF while reading status line".to_string(),
        ));
    }

    otel_debug!(
        "otap_grpc_exporter.proxy.connect_response",
        status_line = status_line.trim()
    );

    // Parse "HTTP/1.1 200 Connection established".
    // Be robust to multiple ASCII spaces/tabs between tokens.
    let mut parts = status_line.trim().split_ascii_whitespace();
    let _http_version = parts.next();
    let status_str = parts.next().ok_or_else(|| {
        ProxyError::InvalidResponse(format!("invalid status line: {status_line}"))
    })?;

    // The (optional) reason phrase is not required by all proxies.
    // We keep it only for error reporting.
    let message = parts.collect::<Vec<_>>().join(" ");

    let status: u16 = status_str
        .parse()
        .map_err(|_| ProxyError::InvalidResponse(format!("invalid status code: {status_str}")))?;

    // Read remaining headers (skip until empty line)
    // Limit the number of headers and their size to prevent potential DoS/memory exhaustion
    let mut header_count = 0;
    const MAX_HEADERS: usize = 100;
    const MAX_HEADER_SIZE: u64 = 8192;

    loop {
        let mut header_line = String::new();
        let mut limited_reader = (&mut buf_reader).take(MAX_HEADER_SIZE);
        let bytes_read = limited_reader
            .read_line(&mut header_line)
            .await
            .map_err(ProxyError::ProxyConnectionFailed)?;

        if bytes_read == 0 {
            return Err(ProxyError::InvalidResponse(
                "unexpected EOF while reading headers".to_string(),
            ));
        }

        // If we read MAX_HEADER_SIZE bytes and didn't find a newline, the line is too long
        if bytes_read as u64 == MAX_HEADER_SIZE && !header_line.ends_with('\n') {
            return Err(ProxyError::InvalidResponse(
                "header line too long".to_string(),
            ));
        }

        if header_line.trim().is_empty() {
            break;
        }
        header_count += 1;
        if header_count > MAX_HEADERS {
            return Err(ProxyError::InvalidResponse(
                "too many headers in proxy response".to_string(),
            ));
        }
    }

    if !(200..300).contains(&status) {
        return Err(ProxyError::ConnectFailed { status, message });
    }

    // Check for buffered data before returning the stream
    if !buf_reader.buffer().is_empty() {
        return Err(ProxyError::InvalidResponse(
            "unexpected data after CONNECT response headers".to_string(),
        ));
    }

    Ok(buf_reader.into_inner())
}

/// Applies TCP socket options (nodelay, keepalive) to a stream.
fn apply_socket_options(
    stream: TcpStream,
    tcp_nodelay: bool,
    tcp_keepalive: Option<Duration>,
    tcp_keepalive_interval: Option<Duration>,
    tcp_keepalive_retries: Option<u32>,
) -> io::Result<TcpStream> {
    socket_options::apply_socket_options(
        stream,
        tcp_nodelay,
        tcp_keepalive,
        tcp_keepalive_interval,
        tcp_keepalive_retries,
    )
}

/// Establishes a TCP connection to a target, optionally through an HTTP CONNECT proxy.
///
/// This is intended for gRPC transports that manage TLS separately (e.g., tonic's
/// `Endpoint` with `.tls_config(...)`).
///
/// Note: TCP socket options (nodelay, keepalive) are applied to the connection
/// to the proxy server itself. Since the tunnel is just a byte stream over this
/// connection, these settings effectively apply to the tunneled traffic as well.
pub(crate) async fn connect_tcp_stream_with_proxy_config(
    target_uri: &Uri,
    proxy_config: &ProxyConfig,
    tcp_nodelay: bool,
    tcp_keepalive: Option<Duration>,
    tcp_keepalive_interval: Option<Duration>,
    tcp_keepalive_retries: Option<u32>,
) -> Result<ProxyTcpStream, ProxyError> {
    let scheme = target_uri.scheme_str().unwrap_or("http");
    let host = target_uri
        .host()
        .ok_or_else(|| ProxyError::InvalidUri("missing host".to_string()))?;
    let port = target_uri.port_u16().unwrap_or(match scheme {
        "https" => 443,
        _ => 80,
    });

    if let Some(proxy_url) = proxy_config.get_proxy_for_uri(target_uri) {
        let ParsedProxyUrl {
            host: proxy_host,
            port: proxy_port,
            auth_header: proxy_auth,
            scheme: proxy_scheme,
        } = parse_proxy_url(proxy_url)?;

        otel_debug!(
            "otap_grpc_exporter.proxy.connecting",
            host = proxy_host.as_str(),
            port = proxy_port
        );
        let stream = TcpStream::connect((proxy_host.as_str(), proxy_port))
            .await
            .map_err(|e| {
                otel_warn!(
                    "otap_grpc_exporter.proxy.connect_failed",
                    host = proxy_host.as_str(),
                    port = proxy_port,
                    error_kind = format!("{:?}", e.kind()),
                    raw_os_error = e.raw_os_error().map(|c| c.to_string()).unwrap_or_default()
                );
                ProxyError::ProxyConnectionFailed(e)
            })?;
        otel_debug!(
            "otap_grpc_exporter.proxy.connected",
            host = proxy_host.as_str(),
            port = proxy_port
        );

        // Apply socket options to the proxy connection
        let stream = apply_socket_options(
            stream,
            tcp_nodelay,
            tcp_keepalive,
            tcp_keepalive_interval,
            tcp_keepalive_retries,
        )
        .map_err(ProxyError::ProxyConnectionFailed)?;

        let stream = match proxy_scheme {
            ProxyScheme::Http => {
                http_connect_tunnel_on_stream(stream, host, port, proxy_auth.as_deref()).await?
            }
            #[cfg(feature = "experimental-tls")]
            ProxyScheme::Https => {
                let (connector, server_name) =
                    build_proxy_tls_connector(proxy_host.as_str(), proxy_config.tls.as_ref())
                        .await?;
                let tls_stream = connector
                    .connect(server_name, stream)
                    .await
                    .map_err(|e| ProxyError::ProxyTlsHandshake(e.to_string()))?;
                let tls_stream =
                    http_connect_tunnel_on_stream(tls_stream, host, port, proxy_auth.as_deref())
                        .await?;
                return Ok(Box::new(tls_stream));
            }
        };

        Ok(Box::new(stream))
    } else {
        let stream = TcpStream::connect((host, port))
            .await
            .map_err(ProxyError::TargetConnectionFailed)?;
        let stream = apply_socket_options(
            stream,
            tcp_nodelay,
            tcp_keepalive,
            tcp_keepalive_interval,
            tcp_keepalive_retries,
        )
        .map_err(ProxyError::TargetConnectionFailed)?;
        Ok(Box::new(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn tunnel_with_mock_proxy_response(response: Vec<u8>) -> Result<(), ProxyError> {
        let (client, mut server) = tokio::io::duplex(32 * 1024);
        let server_task = tokio::spawn(async move {
            // Drain CONNECT request bytes before sending a mocked proxy response.
            let mut request_buf = vec![0u8; 2048];
            let _ = server.read(&mut request_buf).await;
            if !response.is_empty() {
                let _ = server.write_all(&response).await;
            }
            let _ = server.shutdown().await;
        });

        let result = http_connect_tunnel_on_stream(client, "example.com", 4317, None)
            .await
            .map(|_| ());
        server_task.await.unwrap();
        result
    }

    #[cfg(feature = "experimental-tls")]
    async fn start_tls_proxy_for_handshake_failure()
    -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
        use otap_test_tls_certs::{ExtendedKeyUsage, generate_ca};
        use rustls_pki_types::pem::PemObject;
        use rustls_pki_types::{CertificateDer, PrivateKeyDer};
        use std::sync::Arc;
        use tokio_rustls::TlsAcceptor;

        crate::crypto::ensure_crypto_provider();

        let ca = generate_ca("Proxy Handshake Test CA");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        let cert_chain: Vec<_> = CertificateDer::pem_slice_iter(server.cert_pem.as_bytes())
            .collect::<Result<_, _>>()
            .expect("parse proxy cert chain");
        let private_key =
            PrivateKeyDer::from_pem_slice(server.key_pem.as_bytes()).expect("parse proxy key");
        let tls_server_cfg = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .expect("build proxy tls server config");
        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_server_cfg));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let _ = tls_acceptor.accept(stream).await;
            }
        });
        (addr, task)
    }

    #[test]
    fn test_proxy_config_from_env() {
        // This test just verifies the code compiles and runs
        let config = ProxyConfig::from_env();
        let _ = config.has_proxy();
    }

    #[test]
    fn test_no_proxy_bypass() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("localhost,*.local,127.0.0.1,.example.com".to_string()),
            ..ProxyConfig::default()
        };

        assert!(config.should_bypass("localhost", 80));
        assert!(config.should_bypass("test.local", 80));
        assert!(config.should_bypass("127.0.0.1", 80));
        assert!(config.should_bypass("sub.example.com", 80));
        assert!(config.should_bypass("example.com", 80));
        assert!(!config.should_bypass("example.org", 80));
        assert!(!config.should_bypass("proxy.example.org", 80));
    }

    #[test]
    fn test_proxy_selection() {
        let config = ProxyConfig {
            http_proxy: Some("http://http-proxy:3128".into()),
            https_proxy: Some("http://https-proxy:3128".into()),
            no_proxy: Some("localhost".to_string()),
            ..ProxyConfig::default()
        };

        let http_uri: Uri = "http://example.com".parse().unwrap();
        let https_uri: Uri = "https://example.com".parse().unwrap();
        let localhost_uri: Uri = "http://localhost".parse().unwrap();

        assert_eq!(
            config.get_proxy_for_uri(&http_uri),
            Some("http://http-proxy:3128")
        );
        assert_eq!(
            config.get_proxy_for_uri(&https_uri),
            Some("http://https-proxy:3128")
        );
        assert_eq!(config.get_proxy_for_uri(&localhost_uri), None);
    }

    #[test]
    fn test_no_proxy_host_port_matching() {
        let config = ProxyConfig {
            all_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("example.com:443,example.net:80,[::1]:4317".to_string()),
            ..ProxyConfig::default()
        };

        assert!(config.should_bypass("example.com", 443));
        assert!(config.should_bypass("example.net", 80));

        let https_default_port: Uri = "https://example.com".parse().unwrap();
        let https_other_port: Uri = "https://example.com:444".parse().unwrap();
        let http_default_port: Uri = "http://example.net".parse().unwrap();
        let ipv6_port: Uri = "http://[::1]:4317".parse().unwrap();

        let host = https_default_port.host().unwrap_or("");
        let scheme = https_default_port.scheme_str().unwrap_or("http");
        let derived_port = https_default_port.port_u16().unwrap_or(match scheme {
            "https" => 443,
            _ => 80,
        });
        assert_eq!(scheme, "https");
        assert_eq!(host, "example.com");
        assert_eq!(derived_port, 443);
        assert!(config.should_bypass(host, derived_port));

        // Default ports are applied based on scheme.
        assert_eq!(
            config.get_proxy_for_uri(&https_default_port),
            None,
            "https default port should bypass"
        );
        assert_eq!(
            config.get_proxy_for_uri(&https_other_port),
            Some("http://proxy:3128")
        );
        assert_eq!(
            config.get_proxy_for_uri(&http_default_port),
            None,
            "http default port should bypass"
        );
        assert_eq!(
            config.get_proxy_for_uri(&ipv6_port),
            None,
            "ipv6 host:port should bypass"
        );
    }

    #[test]
    fn test_no_proxy_trailing_dot_hostname() {
        let config = ProxyConfig {
            all_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("example.com".to_string()),
            ..ProxyConfig::default()
        };

        let fqdn_with_dot: Uri = "https://example.com.".parse().unwrap();
        assert_eq!(config.get_proxy_for_uri(&fqdn_with_dot), None);
    }

    #[test]
    fn test_all_proxy_fallback() {
        let config = ProxyConfig {
            all_proxy: Some("http://all-proxy:3128".into()),
            ..ProxyConfig::default()
        };

        let http_uri: Uri = "http://example.com".parse().unwrap();
        let https_uri: Uri = "https://example.com".parse().unwrap();

        assert_eq!(
            config.get_proxy_for_uri(&http_uri),
            Some("http://all-proxy:3128")
        );
        assert_eq!(
            config.get_proxy_for_uri(&https_uri),
            Some("http://all-proxy:3128")
        );
    }

    #[test]
    fn test_parse_proxy_url() {
        let parsed = parse_proxy_url("http://proxy.example.com:3128").unwrap();
        assert_eq!(parsed.host, "proxy.example.com");
        assert_eq!(parsed.port, 3128);
        assert!(parsed.auth_header.is_none());
        assert_eq!(parsed.scheme, ProxyScheme::Http);

        let parsed = parse_proxy_url("http://proxy.example.com").unwrap();
        assert_eq!(parsed.host, "proxy.example.com");
        assert_eq!(parsed.port, 3128); // Default proxy port
        assert!(parsed.auth_header.is_none());
        assert_eq!(parsed.scheme, ProxyScheme::Http);

        // Test with credentials
        let parsed = parse_proxy_url("http://user:pass@proxy.example.com:8080").unwrap();
        assert_eq!(parsed.host, "proxy.example.com");
        assert_eq!(parsed.port, 8080);
        assert!(parsed.auth_header.is_some());
        assert!(
            parsed
                .auth_header
                .as_deref()
                .is_some_and(|header| header.starts_with("Basic "))
        );
    }

    #[cfg(feature = "experimental-tls")]
    #[test]
    fn test_parse_proxy_url_accepts_https() {
        let parsed = parse_proxy_url("https://secure-proxy.example.com:8443").unwrap();
        assert_eq!(parsed.host, "secure-proxy.example.com");
        assert_eq!(parsed.port, 8443);
        assert!(parsed.auth_header.is_none());
        assert_eq!(parsed.scheme, ProxyScheme::Https);
    }

    #[cfg(feature = "experimental-tls")]
    #[test]
    fn test_parse_proxy_url_https_default_port() {
        let parsed = parse_proxy_url("https://secure-proxy.example.com").unwrap();
        assert_eq!(parsed.host, "secure-proxy.example.com");
        assert_eq!(parsed.port, 443);
        assert!(parsed.auth_header.is_none());
        assert_eq!(parsed.scheme, ProxyScheme::Https);
    }

    #[cfg(not(feature = "experimental-tls"))]
    #[test]
    fn test_parse_proxy_url_rejects_https_without_tls_feature() {
        let err = parse_proxy_url("https://secure-proxy.example.com").unwrap_err();
        assert!(err.to_string().contains("experimental-tls"));
    }

    #[test]
    fn test_parse_proxy_url_error_redacts_credentials() {
        let err = parse_proxy_url("socks5://user:pass@proxy.example.com:8443").unwrap_err();
        match err {
            ProxyError::InvalidProxyUrl(msg) => {
                assert!(!msg.contains("user:pass"));
                assert!(msg.contains("[REDACTED]@proxy.example.com"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn test_build_proxy_tls_connector_rejects_insecure() {
        let tls = TlsClientConfig {
            insecure: Some(true),
            ..TlsClientConfig::default()
        };
        let err = match build_proxy_tls_connector("proxy.example.com", Some(&tls)).await {
            Ok(_) => panic!("expected proxy TLS config validation error"),
            Err(err) => err,
        };
        match err {
            ProxyError::InvalidProxyUrl(msg) => assert!(msg.contains("insecure=true")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn test_build_proxy_tls_connector_rejects_insecure_skip_verify() {
        let tls = TlsClientConfig {
            insecure_skip_verify: Some(true),
            ..TlsClientConfig::default()
        };
        let err = match build_proxy_tls_connector("proxy.example.com", Some(&tls)).await {
            Ok(_) => panic!("expected proxy TLS config validation error"),
            Err(err) => err,
        };
        match err {
            ProxyError::InvalidProxyUrl(msg) => assert!(msg.contains("insecure_skip_verify=true")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn test_build_proxy_tls_connector_rejects_empty_trust_store() {
        let tls = TlsClientConfig {
            include_system_ca_certs_pool: Some(false),
            ..TlsClientConfig::default()
        };
        let err = match build_proxy_tls_connector("proxy.example.com", Some(&tls)).await {
            Ok(_) => panic!("expected trust store validation error"),
            Err(err) => err,
        };
        match err {
            ProxyError::InvalidProxyUrl(msg) => assert!(msg.contains("no trust anchors available")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn test_build_proxy_tls_connector_reports_ca_file_read_error_as_config_error() {
        use std::path::PathBuf;

        let tls = TlsClientConfig {
            ca_file: Some(PathBuf::from(
                "/tmp/otap-dataflow-missing-proxy-ca-file.pem",
            )),
            include_system_ca_certs_pool: Some(false),
            ..TlsClientConfig::default()
        };
        let err = match build_proxy_tls_connector("proxy.example.com", Some(&tls)).await {
            Ok(_) => panic!("expected proxy TLS config file read error"),
            Err(err) => err,
        };
        match err {
            ProxyError::InvalidProxyUrl(msg) => {
                assert!(msg.contains("failed to read proxy.tls.ca_file"));
                assert!(msg.contains("/tmp/otap-dataflow-missing-proxy-ca-file.pem"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn test_build_proxy_tls_connector_rejects_partial_mtls_config() {
        use otap_df_config::tls::TlsConfig;
        use otap_test_tls_certs::generate_ca;

        let ca = generate_ca("Proxy Config Test CA");
        let tls = TlsClientConfig {
            config: TlsConfig {
                cert_pem: Some(
                    "-----BEGIN CERTIFICATE-----\ninvalid\n-----END CERTIFICATE-----".to_string(),
                ),
                ..TlsConfig::default()
            },
            ca_pem: Some(ca.cert_pem),
            include_system_ca_certs_pool: Some(false),
            ..TlsClientConfig::default()
        };
        let err = match build_proxy_tls_connector("proxy.example.com", Some(&tls)).await {
            Ok(_) => panic!("expected partial mTLS validation error"),
            Err(err) => err,
        };
        match err {
            ProxyError::InvalidProxyUrl(msg) => assert!(msg.contains("partially configured")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[cfg(feature = "experimental-tls")]
    #[test]
    fn test_tls_server_name_for_proxy_rejects_invalid_name() {
        let tls = TlsClientConfig {
            server_name: Some("invalid host name".to_string()),
            ..TlsClientConfig::default()
        };
        let err = tls_server_name_for_proxy("proxy.example.com", Some(&tls)).unwrap_err();
        match err {
            ProxyError::InvalidProxyUrl(msg) => assert!(msg.contains("invalid proxy server name")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn test_connect_tcp_stream_with_proxy_config_maps_tls_handshake_error() {
        use otap_df_config::tls::TlsConfig;
        use otap_test_tls_certs::generate_ca;

        let (proxy_addr, proxy_task) = start_tls_proxy_for_handshake_failure().await;
        let wrong_ca = generate_ca("Wrong Proxy Trust CA");

        let proxy_config = ProxyConfig {
            https_proxy: Some(format!("https://127.0.0.1:{}", proxy_addr.port()).into()),
            tls: Some(TlsClientConfig {
                config: TlsConfig::default(),
                ca_pem: Some(wrong_ca.cert_pem),
                include_system_ca_certs_pool: Some(false),
                server_name: Some("localhost".to_string()),
                ..TlsClientConfig::default()
            }),
            ..ProxyConfig::default()
        };
        let target_uri: Uri = "https://example.com:4317".parse().unwrap();

        let err = match connect_tcp_stream_with_proxy_config(
            &target_uri,
            &proxy_config,
            true,
            None,
            None,
            None,
        )
        .await
        {
            Ok(_) => panic!("expected proxy TLS handshake error"),
            Err(err) => err,
        };
        match err {
            ProxyError::ProxyTlsHandshake(msg) => assert!(!msg.is_empty()),
            other => panic!("unexpected error: {other:?}"),
        }

        proxy_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_connect_tcp_stream_with_proxy_config_direct_connect_error_maps_to_target() {
        // Force immediate DNS resolution failure in direct-connect (no proxy) path.
        let target_uri: Uri = "http://nonexistent.invalid:4317".parse().unwrap();
        let proxy_config = ProxyConfig::default();

        let err = match connect_tcp_stream_with_proxy_config(
            &target_uri,
            &proxy_config,
            true,
            None,
            None,
            None,
        )
        .await
        {
            Ok(_) => panic!("expected direct target connect failure"),
            Err(err) => err,
        };

        match err {
            ProxyError::TargetConnectionFailed(_) => {}
            other => panic!("expected TargetConnectionFailed, got {other:?}"),
        }
    }

    #[test]
    fn test_wildcard_no_proxy() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("*".to_string()),
            ..ProxyConfig::default()
        };

        assert!(config.should_bypass("anything.example.com", 80));
        assert!(config.should_bypass("localhost", 80));
    }

    #[test]
    fn test_no_proxy_cidr_ipv4() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("192.168.0.0/16,10.0.0.0/8,172.16.0.0/12".to_string()),
            ..ProxyConfig::default()
        };

        // Should bypass for IPs in the CIDR ranges
        assert!(config.should_bypass("192.168.1.1", 80));
        assert!(config.should_bypass("192.168.255.254", 80));
        assert!(config.should_bypass("10.0.0.1", 80));
        assert!(config.should_bypass("10.255.255.255", 80));
        assert!(config.should_bypass("172.16.0.1", 80));
        assert!(config.should_bypass("172.31.255.255", 80));

        // Should NOT bypass for IPs outside the ranges
        assert!(!config.should_bypass("8.8.8.8", 80));
        assert!(!config.should_bypass("1.2.3.4", 80));
        assert!(!config.should_bypass("172.32.0.1", 80));
        assert!(!config.should_bypass("192.169.0.1", 80));

        // Hostnames should not match CIDR patterns
        assert!(!config.should_bypass("example.com", 80));
    }

    #[test]
    fn test_no_proxy_cidr_ipv6() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("fe80::/10,::1/128".to_string()),
            ..ProxyConfig::default()
        };

        // Should bypass for IPs in the CIDR ranges
        assert!(config.should_bypass("fe80::1", 80));
        assert!(config.should_bypass("fe80::abcd:1234", 80));
        assert!(config.should_bypass("::1", 80));

        // Should NOT bypass for IPs outside the ranges
        assert!(!config.should_bypass("2001:db8::1", 80));
        assert!(!config.should_bypass("::2", 80));
    }

    #[test]
    fn test_no_proxy_mixed_patterns() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("localhost,*.local,192.168.0.0/16,.example.com,127.0.0.1".to_string()),
            ..ProxyConfig::default()
        };

        // Hostname patterns
        assert!(config.should_bypass("localhost", 80));
        assert!(config.should_bypass("test.local", 80));
        assert!(config.should_bypass("sub.example.com", 80));

        // CIDR patterns
        assert!(config.should_bypass("192.168.1.100", 80));
        assert!(!config.should_bypass("192.169.1.100", 80));

        // Exact IP
        assert!(config.should_bypass("127.0.0.1", 80));

        // Should not match
        assert!(!config.should_bypass("example.org", 80));
        assert!(!config.should_bypass("8.8.8.8", 80));
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();

            let mut buf = vec![0u8; 2048];
            let n = socket.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]);

            assert!(req.starts_with("CONNECT example.com:4317 HTTP/1.1"));
            assert!(req.contains("Host: example.com:4317"));

            socket
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
        });

        let stream = http_connect_tunnel("127.0.0.1", addr.port(), "example.com", 4317)
            .await
            .unwrap();
        drop(stream);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_ipv6_formatting() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();

            let mut buf = vec![0u8; 2048];
            let n = socket.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]);

            // Verify IPv6 address is bracketed in CONNECT line and Host header
            assert!(req.starts_with("CONNECT [::1]:4317 HTTP/1.1"));
            assert!(req.contains("Host: [::1]:4317"));

            socket
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
        });

        let stream = http_connect_tunnel("127.0.0.1", addr.port(), "::1", 4317)
            .await
            .unwrap();
        drop(stream);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_with_auth() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();

            let mut buf = vec![0u8; 2048];
            let n = socket.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]);

            assert!(req.starts_with("CONNECT example.com:4317 HTTP/1.1"));
            assert!(req.contains("Proxy-Authorization: Basic dXNlcjpwYXNz")); // user:pass base64

            socket
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
        });

        // Manually call http_connect_tunnel_on_stream to pass auth
        let stream = TcpStream::connect(addr).await.unwrap();
        let auth = Some("Basic dXNlcjpwYXNz"); // user:pass
        let stream = http_connect_tunnel_on_stream(stream, "example.com", 4317, auth)
            .await
            .unwrap();
        drop(stream);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_rejects_non_2xx() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();

            let mut buf = vec![0u8; 2048];
            let _ = socket.read(&mut buf).await.unwrap();

            socket
                .write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n")
                .await
                .unwrap();
        });

        let err = http_connect_tunnel("127.0.0.1", addr.port(), "example.com", 4317)
            .await
            .unwrap_err();
        match err {
            ProxyError::ConnectFailed { status, .. } => assert_eq!(status, 403),
            other => panic!("unexpected error: {other:?}"),
        }

        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_rejects_eof_before_status_line() {
        let err = tunnel_with_mock_proxy_response(Vec::new())
            .await
            .unwrap_err();
        match err {
            ProxyError::InvalidResponse(msg) => {
                assert!(msg.contains("unexpected EOF while reading status line"))
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_rejects_invalid_status_line() {
        let err = tunnel_with_mock_proxy_response(b"HTTP/1.1\r\n\r\n".to_vec())
            .await
            .unwrap_err();
        match err {
            ProxyError::InvalidResponse(msg) => assert!(msg.contains("invalid status line")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_rejects_invalid_status_code() {
        let err = tunnel_with_mock_proxy_response(b"HTTP/1.1 abc Invalid\r\n\r\n".to_vec())
            .await
            .unwrap_err();
        match err {
            ProxyError::InvalidResponse(msg) => assert!(msg.contains("invalid status code")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_rejects_too_many_headers() {
        let mut response = b"HTTP/1.1 200 Connection established\r\n".to_vec();
        for idx in 0..=100 {
            response.extend_from_slice(format!("X-Test-{idx}: value\r\n").as_bytes());
        }
        response.extend_from_slice(b"\r\n");

        let err = tunnel_with_mock_proxy_response(response).await.unwrap_err();
        match err {
            ProxyError::InvalidResponse(msg) => {
                assert!(msg.contains("too many headers in proxy response"))
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_rejects_header_line_too_long() {
        let mut response = b"HTTP/1.1 200 Connection established\r\nX-Long: ".to_vec();
        response.extend(std::iter::repeat_n(b'a', 9000));

        let err = tunnel_with_mock_proxy_response(response).await.unwrap_err();
        match err {
            ProxyError::InvalidResponse(msg) => assert!(msg.contains("header line too long")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_rejects_buffered_data_after_headers() {
        let response = b"HTTP/1.1 200 Connection established\r\n\r\nEXTRA".to_vec();
        let err = tunnel_with_mock_proxy_response(response).await.unwrap_err();
        match err {
            ProxyError::InvalidResponse(msg) => {
                assert!(msg.contains("unexpected data after CONNECT response headers"))
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_http_connect_tunnel_ipv6_formatting_robustness() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(_) => break,
                };
                let mut buf = vec![0u8; 2048];
                let n = socket.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]);

                if req.contains("CONNECT [2001:db8::1]:4317")
                    || req.contains("CONNECT example.com:4317")
                    || req.contains("CONNECT my:host:4317")
                {
                    socket.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await.unwrap();
                } else {
                    // Fail if we see unexpected bracketing
                    socket
                        .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                        .await
                        .unwrap();
                }
            }
        });

        // 1. Valid IPv6
        let stream = TcpStream::connect(addr).await.unwrap();
        let _ = http_connect_tunnel_on_stream(stream, "2001:db8::1", 4317, None)
            .await
            .unwrap();

        // 2. Regular Hostname
        let stream = TcpStream::connect(addr).await.unwrap();
        let _ = http_connect_tunnel_on_stream(stream, "example.com", 4317, None)
            .await
            .unwrap();

        // 3. Hostname with colon (should not be bracketed)
        let stream = TcpStream::connect(addr).await.unwrap();
        let _ = http_connect_tunnel_on_stream(stream, "my:host", 4317, None)
            .await
            .unwrap();

        // Abort the server task
        server.abort();
    }
}
