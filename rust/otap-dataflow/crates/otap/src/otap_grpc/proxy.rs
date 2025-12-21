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

use http::Uri;
use ipnet::IpNet;
use serde::Deserialize;
use socket2::{Socket, TcpKeepalive};
use std::env;
use std::io;
use std::net::IpAddr;
#[cfg(feature = "experimental-tls")]
use std::sync::OnceLock;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[cfg(feature = "experimental-tls")]
use std::sync::Arc;
#[cfg(feature = "experimental-tls")]
use tokio_rustls::TlsConnector;

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

    /// TLS handshake failed
    #[error("TLS handshake failed: {0}")]
    TlsError(String),

    /// Invalid target URI
    #[error("invalid target URI: {0}")]
    InvalidUri(String),
}

/// Proxy configuration that can be set explicitly or read from environment.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxyConfig {
    /// HTTP proxy URL (e.g., "http://proxy.example.com:3128")
    /// If not set, reads from HTTP_PROXY/http_proxy environment variable.
    #[serde(default)]
    pub http_proxy: Option<String>,

    /// HTTPS proxy URL (e.g., "http://proxy.example.com:3128")
    /// If not set, reads from HTTPS_PROXY/https_proxy environment variable.
    #[serde(default)]
    pub https_proxy: Option<String>,

    /// Fallback proxy URL for all protocols.
    /// If not set, reads from ALL_PROXY/all_proxy environment variable.
    #[serde(default)]
    pub all_proxy: Option<String>,

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
                .ok(),
            https_proxy: env::var("HTTPS_PROXY")
                .or_else(|_| env::var("https_proxy"))
                .ok(),
            all_proxy: env::var("ALL_PROXY")
                .or_else(|_| env::var("all_proxy"))
                .ok(),
            no_proxy: env::var("NO_PROXY").or_else(|_| env::var("no_proxy")).ok(),
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

    /// Returns the proxy URL for a given target URI, or None if no proxy should be used.
    #[must_use]
    pub fn get_proxy_for_uri(&self, uri: &Uri) -> Option<&str> {
        let host = uri.host().unwrap_or("");

        // Check if host should bypass proxy
        if self.should_bypass(host) {
            return None;
        }

        // Select proxy based on scheme
        let scheme = uri.scheme_str().unwrap_or("http");
        match scheme {
            "https" => self.https_proxy.as_deref().or(self.all_proxy.as_deref()),
            _ => self.http_proxy.as_deref().or(self.all_proxy.as_deref()),
        }
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
    fn should_bypass(&self, host: &str) -> bool {
        let no_proxy = match &self.no_proxy {
            Some(np) => np,
            None => return false,
        };

        let host_lower = host.to_lowercase();

        // Try to parse host as an IP address for CIDR matching
        let host_ip = host.parse::<IpAddr>().ok();

        for pattern in no_proxy.split(',') {
            let pattern = pattern.trim().to_lowercase();
            if pattern.is_empty() {
                continue;
            }

            // Handle "*" wildcard for all hosts
            if pattern == "*" {
                return true;
            }

            // Handle CIDR notation (e.g., "192.168.0.0/16", "10.0.0.0/8")
            if pattern.contains('/') {
                if let (Some(ip), Ok(net)) = (host_ip, pattern.parse::<IpNet>()) {
                    if net.contains(&ip) {
                        return true;
                    }
                }
                continue;
            }

            // Handle wildcard prefix patterns like "*.example.com"
            if let Some(suffix) = pattern.strip_prefix("*.") {
                if host_lower.ends_with(&format!(".{suffix}")) || host_lower == suffix {
                    return true;
                }
            } else if let Some(suffix) = pattern.strip_prefix('.') {
                // Handle ".example.com" (matches subdomains and the domain itself)
                if host_lower.ends_with(&pattern) || host_lower == suffix {
                    return true;
                }
            } else if host_lower == pattern {
                // Exact match (hostname or IP)
                return true;
            }
        }

        false
    }

    /// Returns true if any proxy is configured.
    #[must_use]
    pub fn has_proxy(&self) -> bool {
        self.http_proxy.is_some() || self.https_proxy.is_some() || self.all_proxy.is_some()
    }
}

/// Parses a proxy URL and returns (host, port).
fn parse_proxy_url(proxy_url: &str) -> Result<(String, u16), ProxyError> {
    let uri: Uri = proxy_url
        .parse()
        .map_err(|_| ProxyError::InvalidProxyUrl(proxy_url.to_string()))?;

    // Reject https:// proxy URLs - we don't support TLS to the proxy server itself
    if uri.scheme_str() == Some("https") {
        return Err(ProxyError::InvalidProxyUrl(format!(
            "https:// proxy URLs are not supported (proxy URL: {}). \
                 Use http:// instead - the CONNECT tunnel will still encrypt \
                 traffic to the final destination for https:// targets.",
            proxy_url
        )));
    }

    let host = uri
        .host()
        .ok_or_else(|| ProxyError::InvalidProxyUrl(format!("missing host in {proxy_url}")))?
        .to_string();

    let port = uri.port_u16().unwrap_or(3128); // Default proxy port

    Ok((host, port))
}

/// Establishes an HTTP CONNECT tunnel through a proxy.
///
/// This function connects to the proxy, sends an HTTP CONNECT request,
/// and returns the TCP stream once the tunnel is established.
pub async fn http_connect_tunnel(
    proxy_host: &str,
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, ProxyError> {
    // Connect to the proxy
    let stream = TcpStream::connect((proxy_host, proxy_port)).await?;

    http_connect_tunnel_on_stream(stream, target_host, target_port).await
}

async fn http_connect_tunnel_on_stream(
    stream: TcpStream,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, ProxyError> {
    // Send HTTP CONNECT request
    let connect_request = format!(
        "CONNECT {target_host}:{target_port} HTTP/1.1\r\n\
         Host: {target_host}:{target_port}\r\n\
         Proxy-Connection: Keep-Alive\r\n\
         \r\n"
    );

    let (reader, mut writer) = stream.into_split();
    writer.write_all(connect_request.as_bytes()).await?;

    // Read the response status line
    let mut buf_reader = BufReader::new(reader);
    let mut status_line = String::new();
    let _ = buf_reader.read_line(&mut status_line).await?;

    // Parse "HTTP/1.1 200 Connection established"
    let parts: Vec<&str> = status_line.trim().splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(ProxyError::InvalidResponse(format!(
            "invalid status line: {status_line}"
        )));
    }

    let status: u16 = parts[1]
        .parse()
        .map_err(|_| ProxyError::InvalidResponse(format!("invalid status code: {}", parts[1])))?;

    // Read remaining headers (skip until empty line)
    loop {
        let mut header_line = String::new();
        let _ = buf_reader.read_line(&mut header_line).await?;
        if header_line.trim().is_empty() {
            break;
        }
    }

    if !(200..300).contains(&status) {
        let message = parts.get(2).unwrap_or(&"").to_string();
        return Err(ProxyError::ConnectFailed { status, message });
    }

    // Reunite the stream
    let reader = buf_reader.into_inner();
    Ok(reader.reunite(writer).expect("same stream split"))
}

/// Applies TCP socket options (nodelay, keepalive) to a stream.
fn apply_socket_options(
    stream: TcpStream,
    tcp_nodelay: bool,
    tcp_keepalive: Option<Duration>,
    tcp_keepalive_interval: Option<Duration>,
    tcp_keepalive_retries: Option<u32>,
) -> io::Result<TcpStream> {
    // Convert tokio TcpStream to std TcpStream, then to Socket
    stream.set_nodelay(tcp_nodelay)?;

    let std_stream = stream.into_std()?;
    let socket: Socket = std_stream.into();

    // Apply TCP keepalive settings
    if let Some(keepalive_time) = tcp_keepalive {
        let mut keepalive = TcpKeepalive::new().with_time(keepalive_time);

        if let Some(interval) = tcp_keepalive_interval {
            keepalive = keepalive.with_interval(interval);
        }

        #[cfg(not(target_os = "windows"))]
        if let Some(retries) = tcp_keepalive_retries {
            keepalive = keepalive.with_retries(retries);
        }

        socket.set_tcp_keepalive(&keepalive)?;
    }

    // Convert back to std TcpStream, then to tokio TcpStream
    let std_stream: std::net::TcpStream = socket.into();
    std_stream.set_nonblocking(true)?;
    TcpStream::from_std(std_stream)
}

/// Establishes a TCP connection to a target, optionally through an HTTP CONNECT proxy.
///
/// This is intended for gRPC transports that manage TLS separately (e.g., tonic's
/// `Endpoint` with `.tls_config(...)`).
pub async fn connect_tcp_stream_with_proxy_config(
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
        let (proxy_host, proxy_port) = parse_proxy_url(proxy_url)?;
        let stream = TcpStream::connect((proxy_host.as_str(), proxy_port)).await?;

        // Apply socket options to the proxy connection
        let stream = apply_socket_options(
            stream,
            tcp_nodelay,
            tcp_keepalive,
            tcp_keepalive_interval,
            tcp_keepalive_retries,
        )?;

        let stream = http_connect_tunnel_on_stream(stream, host, port).await?;

        // Apply socket options again to the tunneled connection
        let stream = apply_socket_options(
            stream,
            tcp_nodelay,
            tcp_keepalive,
            tcp_keepalive_interval,
            tcp_keepalive_retries,
        )?;

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

/// Creates a TLS connector configured for HTTP/2 (gRPC).
///
/// This function creates a TLS connector with proper ALPN configuration
/// (`h2`) which is required for HTTP/2 over TLS, essential for gRPC
/// through HTTPS proxies.
#[cfg(feature = "experimental-tls")]
pub fn create_h2_tls_connector() -> Result<TlsConnector, ProxyError> {
    use rustls::RootCertStore;
    use tokio_rustls::rustls::ClientConfig;

    static H2_TLS_CONNECTOR: OnceLock<TlsConnector> = OnceLock::new();

    if let Some(connector) = H2_TLS_CONNECTOR.get() {
        return Ok(connector.clone());
    }

    let mut roots = RootCertStore::empty();

    // Load native certificates - rustls_native_certs returns CertificateResult struct
    let cert_result = rustls_native_certs::load_native_certs();

    // Log any errors that occurred during loading
    for error in &cert_result.errors {
        log::warn!("Error loading native cert: {error}");
    }

    // Add all successfully loaded certificates
    for cert in cert_result.certs {
        // Ignore individual cert errors, just skip them
        let _ = roots.add(cert);
    }

    if roots.is_empty() {
        return Err(ProxyError::TlsError(
            "no valid root certificates found".to_string(),
        ));
    }

    let mut config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    // Critical: Set ALPN to h2 for HTTP/2 over TLS (required for gRPC)
    config.alpn_protocols = vec![b"h2".to_vec()];

    let connector = TlsConnector::from(Arc::new(config));
    let _ = H2_TLS_CONNECTOR.set(connector.clone());
    Ok(connector)
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
            http_proxy: Some("http://proxy:3128".to_string()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("localhost,*.local,127.0.0.1,.example.com".to_string()),
        };

        assert!(config.should_bypass("localhost"));
        assert!(config.should_bypass("test.local"));
        assert!(config.should_bypass("127.0.0.1"));
        assert!(config.should_bypass("sub.example.com"));
        assert!(config.should_bypass("example.com"));
        assert!(!config.should_bypass("example.org"));
        assert!(!config.should_bypass("proxy.example.org"));
    }

    #[test]
    fn test_proxy_selection() {
        let config = ProxyConfig {
            http_proxy: Some("http://http-proxy:3128".to_string()),
            https_proxy: Some("http://https-proxy:3128".to_string()),
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
    fn test_all_proxy_fallback() {
        let config = ProxyConfig {
            http_proxy: None,
            https_proxy: None,
            all_proxy: Some("http://all-proxy:3128".to_string()),
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
        let (host, port) = parse_proxy_url("http://proxy.example.com:3128").unwrap();
        assert_eq!(host, "proxy.example.com");
        assert_eq!(port, 3128);

        let (host, port) = parse_proxy_url("http://proxy.example.com").unwrap();
        assert_eq!(host, "proxy.example.com");
        assert_eq!(port, 3128); // Default proxy port
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
    fn test_wildcard_no_proxy() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".to_string()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("*".to_string()),
        };

        assert!(config.should_bypass("anything.example.com"));
        assert!(config.should_bypass("localhost"));
    }

    #[test]
    fn test_no_proxy_cidr_ipv4() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".to_string()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("192.168.0.0/16,10.0.0.0/8,172.16.0.0/12".to_string()),
        };

        // Should bypass for IPs in the CIDR ranges
        assert!(config.should_bypass("192.168.1.1"));
        assert!(config.should_bypass("192.168.255.254"));
        assert!(config.should_bypass("10.0.0.1"));
        assert!(config.should_bypass("10.255.255.255"));
        assert!(config.should_bypass("172.16.0.1"));
        assert!(config.should_bypass("172.31.255.255"));

        // Should NOT bypass for IPs outside the ranges
        assert!(!config.should_bypass("8.8.8.8"));
        assert!(!config.should_bypass("1.2.3.4"));
        assert!(!config.should_bypass("172.32.0.1"));
        assert!(!config.should_bypass("192.169.0.1"));

        // Hostnames should not match CIDR patterns
        assert!(!config.should_bypass("example.com"));
    }

    #[test]
    fn test_no_proxy_cidr_ipv6() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".to_string()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("fe80::/10,::1/128".to_string()),
        };

        // Should bypass for IPs in the CIDR ranges
        assert!(config.should_bypass("fe80::1"));
        assert!(config.should_bypass("fe80::abcd:1234"));
        assert!(config.should_bypass("::1"));

        // Should NOT bypass for IPs outside the ranges
        assert!(!config.should_bypass("2001:db8::1"));
        assert!(!config.should_bypass("::2"));
    }

    #[test]
    fn test_no_proxy_mixed_patterns() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy:3128".to_string()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: Some("localhost,*.local,192.168.0.0/16,.example.com,127.0.0.1".to_string()),
        };

        // Hostname patterns
        assert!(config.should_bypass("localhost"));
        assert!(config.should_bypass("test.local"));
        assert!(config.should_bypass("sub.example.com"));

        // CIDR patterns
        assert!(config.should_bypass("192.168.1.100"));
        assert!(!config.should_bypass("192.169.1.100"));

        // Exact IP
        assert!(config.should_bypass("127.0.0.1"));

        // Should not match
        assert!(!config.should_bypass("example.org"));
        assert!(!config.should_bypass("8.8.8.8"));
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
}
