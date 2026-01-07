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

use crate::socket_options;
use base64::Engine;
use base64::prelude::*;
use http::Uri;
use ipnet::IpNet;
use serde::Deserialize;
use std::borrow::Cow;
use std::env;
use std::io;
use std::net::IpAddr;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

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
    ProxyConnectionFailed(#[from] io::Error),

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
}

/// Proxy configuration that can be set explicitly or read from environment.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[doc(hidden)]
pub struct ProxyConfig {
    /// HTTP proxy URL (e.g., "http://proxy.example.com:3128")
    /// If not set, reads from HTTP_PROXY/http_proxy environment variable.
    ///
    /// Note: This is the URL of the proxy server itself. The proxy connection is currently
    /// plain HTTP (no TLS to the proxy). For HTTPS targets, we still use HTTP CONNECT to
    /// establish a tunnel and then perform TLS inside the tunnel.
    #[serde(default)]
    pub http_proxy: Option<SensitiveUrl>,

    /// Proxy URL to use for *HTTPS targets* (e.g., "http://proxy.example.com:3128").
    /// If not set, reads from HTTPS_PROXY/https_proxy environment variable.
    ///
    /// Note: Despite the name, this does **not** mean we connect to the proxy over HTTPS.
    /// It means this proxy is selected when the target scheme is `https`.
    /// The TLS handshake for the target happens inside the tunnel established via HTTP CONNECT.
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
        }
    }
}

impl std::fmt::Display for ProxyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyConfig")
            .field("http_proxy", &self.http_proxy)
            .field("https_proxy", &self.https_proxy)
            .field("all_proxy", &self.all_proxy)
            .field("no_proxy", &self.no_proxy)
            .finish()
    }
}

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
            otap_df_telemetry::otel_debug!("Proxy.Bypass", host = host, port = port);
            return None;
        }

        // Select proxy based on scheme
        let proxy = match scheme {
            "https" => self.https_proxy.as_ref().or(self.all_proxy.as_ref()),
            _ => self.http_proxy.as_ref().or(self.all_proxy.as_ref()),
        };

        if let Some(url) = proxy {
            if log::log_enabled!(log::Level::Debug) {
                otap_df_telemetry::otel_debug!(
                    "Proxy.Using",
                    proxy = url.to_string(),
                    target = uri.to_string()
                );
            }
        } else {
            otap_df_telemetry::otel_debug!("Proxy.None", target = uri.to_string());
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
                    otap_df_telemetry::otel_warn!("Proxy.InvalidCidr", pattern = pattern_host);
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
    pub fn has_proxy(&self) -> bool {
        self.http_proxy.is_some() || self.https_proxy.is_some() || self.all_proxy.is_some()
    }
}

/// Parses a proxy URL and returns (host, port, auth_header_value).
fn parse_proxy_url(proxy_url: &str) -> Result<(String, u16, Option<String>), ProxyError> {
    // Avoid leaking credentials in error messages.
    let proxy_url_redacted = SensitiveUrl::new(proxy_url).to_string();

    let uri: Uri = proxy_url.parse().map_err(|_| {
        ProxyError::InvalidProxyUrl(format!("failed to parse proxy URL ({proxy_url_redacted})"))
    })?;

    // Reject https:// proxy URLs - we don't support TLS to the proxy server itself
    if uri.scheme_str() == Some("https") {
        return Err(ProxyError::InvalidProxyUrl(format!(
            "https:// proxy URLs are not supported (proxy URL: {}). \
Use http:// instead - the CONNECT tunnel will still encrypt \
traffic to the final destination for https:// targets.",
            proxy_url_redacted
        )));
    }

    let host = uri
        .host()
        .ok_or_else(|| {
            ProxyError::InvalidProxyUrl(format!("missing host in proxy URL ({proxy_url_redacted})"))
        })?
        .to_string();

    let port = uri.port_u16().unwrap_or(3128); // Default proxy port

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

    Ok((host, port, auth_header))
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
    let stream = TcpStream::connect((proxy_host, proxy_port)).await?;

    http_connect_tunnel_on_stream(stream, target_host, target_port, None).await
}

async fn http_connect_tunnel_on_stream(
    stream: TcpStream,
    target_host: &str,
    target_port: u16,
    proxy_auth: Option<&str>,
) -> Result<TcpStream, ProxyError> {
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
    otap_df_telemetry::otel_debug!(
        "Proxy.ConnectRequest",
        target = format!("{formatted_target}:{target_port}"),
        has_auth = proxy_auth.is_some()
    );

    let (reader, mut writer) = stream.into_split();
    writer.write_all(connect_request.as_bytes()).await?;

    // Read the response status line
    let mut buf_reader = BufReader::new(reader);
    let mut status_line = String::new();
    if buf_reader.read_line(&mut status_line).await? == 0 {
        return Err(ProxyError::InvalidResponse(
            "unexpected EOF while reading status line".to_string(),
        ));
    }

    otap_df_telemetry::otel_debug!("Proxy.ConnectResponse", status_line = status_line.trim());

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
        let bytes_read = limited_reader.read_line(&mut header_line).await?;

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

    // Check for buffered data before reuniting
    if !buf_reader.buffer().is_empty() {
        return Err(ProxyError::InvalidResponse(
            "unexpected data after CONNECT response headers".to_string(),
        ));
    }

    // Reunite the stream
    let reader = buf_reader.into_inner();
    let stream = reader
        .reunite(writer)
        .map_err(|_| ProxyError::InvalidResponse("failed to reunite stream".to_string()))?;
    Ok(stream)
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
) -> Result<TcpStream, ProxyError> {
    let scheme = target_uri.scheme_str().unwrap_or("http");
    let host = target_uri
        .host()
        .ok_or_else(|| ProxyError::InvalidUri("missing host".to_string()))?;
    let port = target_uri.port_u16().unwrap_or(match scheme {
        "https" => 443,
        _ => 80,
    });

    if let Some(proxy_url) = proxy_config.get_proxy_for_uri(target_uri) {
        let (proxy_host, proxy_port, proxy_auth) = parse_proxy_url(proxy_url)?;

        otap_df_telemetry::otel_debug!("Proxy.Connecting", host = proxy_host, port = proxy_port);
        let stream = TcpStream::connect((proxy_host.as_str(), proxy_port))
            .await
            .map_err(|e| {
                otap_df_telemetry::otel_warn!(
                    "Proxy.ConnectFailed",
                    host = proxy_host,
                    port = proxy_port,
                    error_kind = format!("{:?}", e.kind()),
                    raw_os_error = e.raw_os_error().map(|c| c.to_string()).unwrap_or_default()
                );
                ProxyError::ProxyConnectionFailed(e)
            })?;
        otap_df_telemetry::otel_debug!("Proxy.Connected");

        // Apply socket options to the proxy connection
        let stream = apply_socket_options(
            stream,
            tcp_nodelay,
            tcp_keepalive,
            tcp_keepalive_interval,
            tcp_keepalive_retries,
        )?;

        let stream =
            http_connect_tunnel_on_stream(stream, host, port, proxy_auth.as_deref()).await?;

        Ok(stream)
    } else {
        let stream = TcpStream::connect((host, port)).await?;
        let stream = apply_socket_options(
            stream,
            tcp_nodelay,
            tcp_keepalive,
            tcp_keepalive_interval,
            tcp_keepalive_retries,
        )?;
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

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
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("localhost,*.local,127.0.0.1,.example.com".to_string()),
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
            all_proxy: None,
            no_proxy: Some("localhost".to_string()),
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
            http_proxy: None,
            https_proxy: None,
            all_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("example.com:443,example.net:80,[::1]:4317".to_string()),
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
            http_proxy: None,
            https_proxy: None,
            all_proxy: Some("http://proxy:3128".into()),
            no_proxy: Some("example.com".to_string()),
        };

        let fqdn_with_dot: Uri = "https://example.com.".parse().unwrap();
        assert_eq!(config.get_proxy_for_uri(&fqdn_with_dot), None);
    }

    #[test]
    fn test_all_proxy_fallback() {
        let config = ProxyConfig {
            http_proxy: None,
            https_proxy: None,
            all_proxy: Some("http://all-proxy:3128".into()),
            no_proxy: None,
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
        let (host, port, auth) = parse_proxy_url("http://proxy.example.com:3128").unwrap();
        assert_eq!(host, "proxy.example.com");
        assert_eq!(port, 3128);
        assert!(auth.is_none());

        let (host, port, auth) = parse_proxy_url("http://proxy.example.com").unwrap();
        assert_eq!(host, "proxy.example.com");
        assert_eq!(port, 3128); // Default proxy port
        assert!(auth.is_none());

        // Test with credentials
        let (host, port, auth) =
            parse_proxy_url("http://user:pass@proxy.example.com:8080").unwrap();
        assert_eq!(host, "proxy.example.com");
        assert_eq!(port, 8080);
        assert!(auth.is_some());
        assert!(auth.unwrap().starts_with("Basic "));
    }

    #[test]
    fn test_parse_proxy_url_rejects_https() {
        // https:// proxy URLs should be rejected with a helpful error message
        let err = parse_proxy_url("https://secure-proxy.example.com").unwrap_err();
        match err {
            ProxyError::InvalidProxyUrl(msg) => {
                assert!(msg.contains("https://"));
                assert!(msg.contains("not supported"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_parse_proxy_url_error_redacts_credentials() {
        let err = parse_proxy_url("https://user:pass@proxy.example.com:8443").unwrap_err();
        match err {
            ProxyError::InvalidProxyUrl(msg) => {
                assert!(!msg.contains("user:pass"));
                assert!(msg.contains("[REDACTED]@proxy.example.com"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_wildcard_no_proxy() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".into()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("*".to_string()),
        };

        assert!(config.should_bypass("anything.example.com", 80));
        assert!(config.should_bypass("localhost", 80));
    }

    #[test]
    fn test_no_proxy_cidr_ipv4() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".into()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("192.168.0.0/16,10.0.0.0/8,172.16.0.0/12".to_string()),
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
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("fe80::/10,::1/128".to_string()),
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
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("localhost,*.local,192.168.0.0/16,.example.com,127.0.0.1".to_string()),
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
