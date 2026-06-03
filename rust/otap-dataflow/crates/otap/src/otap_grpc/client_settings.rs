// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared configuration for gRPC-based clients.

use crate::compression::CompressionMethod;
use crate::otap_grpc::proxy::ProxyConfig;
use crate::tls_utils;
use hyper_util::rt::TokioIo;
use otap_df_config::byte_units;
use otap_df_config::tls::TlsClientConfig;
use serde::Deserialize;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tonic::codec::CompressionEncoding;
use tonic::transport::Channel;
use tonic::transport::Endpoint;
use tower::service_fn;

/// Controls optional startup-time endpoint validation.
///
/// When a client is being created, it can optionally perform a check to detect configuration
/// problems early rather than waiting for the first export RPC to fail.
///
/// The default is [`StartupCheck::None`], which preserves the existing lazy-connection
/// behaviour.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StartupCheck {
    /// No startup check; connections are fully lazy (existing behavior).
    #[default]
    None,

    /// Verify that the endpoint hostname resolves via DNS at startup.
    ///
    /// Note: When a proxy is configured and would handle the target endpoint, this check is
    /// skipped because the proxy is expected to perform name resolution.
    Dns,

    /// Perform one eager gRPC connection attempt at startup.
    ///
    /// This validates the entire connection path including proxy tunneling and TLS handshake.
    Connect,
}

/// Common configuration shared across gRPC clients.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GrpcClientSettings {
    /// The gRPC endpoint to connect to (e.g. `"http://localhost:4317"`).
    ///
    /// If no scheme is provided, `http://` is assumed.
    #[serde(deserialize_with = "deserialize_grpc_endpoint")]
    pub grpc_endpoint: String,

    /// Compression method to use for outbound requests. Defaults to no compression.
    #[serde(default, alias = "compression_method")]
    pub compression: Option<CompressionMethod>,

    /// Maximum number of concurrent in-flight requests allowed by the transport stack.
    #[serde(default = "default_concurrency_limit")]
    pub concurrency_limit: usize,

    /// Timeout for establishing TCP connections.
    ///
    /// When a proxy is configured, this timeout covers the entire connection process,
    /// including the TCP connection to the proxy and the HTTP CONNECT handshake.
    #[serde(default = "default_connect_timeout", with = "humantime_serde")]
    pub connect_timeout: Duration,

    /// Whether to enable `TCP_NODELAY`.
    #[serde(default = "default_tcp_nodelay")]
    pub tcp_nodelay: bool,

    /// TCP keepalive timeout for outbound connections.
    #[serde(default = "default_tcp_keepalive", with = "humantime_serde")]
    pub tcp_keepalive: Option<Duration>,

    /// Interval between TCP keepalive probes once keepalive is active.
    #[serde(default, with = "humantime_serde")]
    pub tcp_keepalive_interval: Option<Duration>,

    /// Number of TCP keepalive probes sent before a connection is declared dead.
    #[serde(default)]
    pub tcp_keepalive_retries: Option<u32>,

    /// Initial HTTP/2 stream window size, in bytes.
    #[serde(
        default = "default_initial_stream_window_size",
        deserialize_with = "byte_units::deserialize"
    )]
    pub initial_stream_window_size: Option<u32>,

    /// Initial HTTP/2 connection window size, in bytes.
    #[serde(
        default = "default_initial_connection_window_size",
        deserialize_with = "byte_units::deserialize"
    )]
    pub initial_connection_window_size: Option<u32>,

    /// Whether to rely on HTTP/2 adaptive window sizing instead of the manual values above.
    #[serde(default = "default_http2_adaptive_window")]
    pub http2_adaptive_window: bool,

    /// Interval between HTTP/2 keepalive pings.
    #[serde(default = "default_http2_keepalive_interval", with = "humantime_serde")]
    pub http2_keepalive_interval: Option<Duration>,

    /// Timeout waiting for an HTTP/2 keepalive acknowledgement.
    #[serde(default = "default_http2_keepalive_timeout", with = "humantime_serde")]
    pub http2_keepalive_timeout: Option<Duration>,

    /// Whether to send HTTP/2 keepalives while idle.
    #[serde(default = "default_keep_alive_while_idle")]
    pub keep_alive_while_idle: bool,

    /// Timeout for RPC requests. If not specified, no timeout is applied.
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,

    /// Client-side TLS/mTLS configuration.
    #[serde(default)]
    pub tls: Option<TlsClientConfig>,

    /// Internal Tower buffer size for the gRPC client.
    #[serde(default)]
    pub buffer_size: Option<usize>,

    /// Optional startup-time endpoint check.
    ///
    /// - `none` (default): no check is performed.
    /// - `dns`: verify the endpoint host resolves at startup.
    /// - `connect`: perform one eager connection attempt at startup.
    #[serde(default)]
    pub startup_check: StartupCheck,

    /// HTTP/HTTPS proxy configuration.
    /// If not specified, proxy settings are read from environment variables:
    /// - `HTTP_PROXY` / `http_proxy`: Proxy for HTTP connections
    /// - `HTTPS_PROXY` / `https_proxy`: Proxy for HTTPS connections
    /// - `ALL_PROXY` / `all_proxy`: Fallback proxy for all connections
    /// - `NO_PROXY` / `no_proxy`: Comma-separated list of hosts to bypass proxy
    #[serde(default)]
    #[doc(hidden)]
    pub proxy: Option<ProxyConfig>,

    /// Custom User-Agent header for outbound gRPC requests. When set, tonic
    /// **prepends** this value to its default `tonic/x.x.x` User-Agent (e.g.
    /// `my-app/1.0 tonic/0.12.x`). When not set, only the default tonic
    /// User-Agent is sent.
    #[serde(default)]
    pub user_agent: Option<String>,
}

/// Error returned when building a gRPC [`Endpoint`] (including TLS/mTLS setup).
#[derive(Debug, Error)]
pub enum GrpcEndpointError {
    /// Error returned by tonic while parsing/configuring the transport endpoint.
    #[error("grpc endpoint build error: {0}")]
    Tonic(#[from] tonic::transport::Error),

    /// IO error while reading certificates/keys for TLS.
    #[error("tls configuration error: {0}")]
    Io(#[from] io::Error),

    /// Proxy configuration or connection error.
    #[error("proxy error: {0}")]
    Proxy(#[from] crate::otap_grpc::proxy::ProxyError),

    /// Invalid gRPC endpoint.
    #[error("invalid grpc_endpoint: {0}")]
    InvalidEndpoint(String),

    /// DNS resolution failed during a `startup_check: dns` check.
    #[error("startup dns check failed for \"{host}\": {source}")]
    DnsCheckFailed {
        /// The hostname that could not be resolved.
        host: String,
        /// The underlying resolution error.
        source: io::Error,
    },
}

/// Validates that a gRPC endpoint string is a well-formed URI.
///
/// When no scheme is present the endpoint is validated as if `http://` were prepended.
/// Unsupported schemes (anything other than `http` / `https`) are rejected.
fn validate_grpc_endpoint(endpoint: &str) -> Result<(), String> {
    let trimmed = endpoint.trim();
    if trimmed.is_empty() {
        return Err("grpc_endpoint is empty; expected a URI like \"http://host:port\"".to_string());
    }

    let uri: http::Uri = trimmed
        .parse()
        .map_err(|e: http::uri::InvalidUri| format!("invalid grpc_endpoint \"{trimmed}\": {e}"))?;

    // If no scheme is present, prepend http:// for validation.
    let effective = if uri.scheme().is_none() {
        format!("http://{trimmed}")
    } else {
        trimmed.to_string()
    };

    let _ = Endpoint::from_shared(effective.clone())
        .map_err(|e| format!("invalid grpc_endpoint \"{trimmed}\": {e}"))?;

    // Reject unsupported schemes.
    if let Some(scheme) = uri.scheme_str() {
        if scheme != "http" && scheme != "https" {
            return Err(format!(
                "unsupported scheme \"{scheme}\" in grpc_endpoint \"{trimmed}\"; \
                 expected \"http\" or \"https\""
            ));
        }
    }

    Ok(())
}

impl GrpcClientSettings {
    /// Performs the configured startup check, if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the startup check fails (DNS resolution failure for `dns` mode, or
    /// connection failure for `connect` mode).
    pub async fn run_startup_check(&self) -> Result<(), GrpcEndpointError> {
        match self.startup_check {
            StartupCheck::None => Ok(()),
            StartupCheck::Dns => self.run_dns_check().await,
            StartupCheck::Connect => self.run_connect_check().await,
        }
    }

    /// Resolves the endpoint hostname via DNS.
    ///
    /// Skipped when a proxy is configured and would handle the target, since the proxy is expected
    /// to perform name resolution.
    async fn run_dns_check(&self) -> Result<(), GrpcEndpointError> {
        let endpoint = self.grpc_endpoint.trim();

        // Extract host and port from the endpoint URI.
        let uri: http::Uri = endpoint.parse().map_err(|e: http::uri::InvalidUri| {
            GrpcEndpointError::InvalidEndpoint(format!("invalid URI \"{endpoint}\": {e}"))
        })?;

        let host = uri
            .host()
            .ok_or_else(|| {
                GrpcEndpointError::InvalidEndpoint(format!("no host in \"{endpoint}\""))
            })?
            .trim_matches('[')
            .trim_matches(']')
            .to_string();

        let port = uri.port_u16().unwrap_or_else(|| {
            if uri.scheme_str() == Some("https") {
                443
            } else {
                80
            }
        });

        // If the actual connection path will use a proxy for this endpoint, the proxy performs
        // DNS resolution -- skip the local check.
        let proxy = self.effective_proxy_config();
        if proxy.get_proxy_for_uri(&uri).is_some() {
            return Ok(());
        }

        // Attempt DNS resolution.
        let lookup_addr = format!("{host}:{port}");
        let mut addrs = tokio::net::lookup_host(&lookup_addr)
            .await
            .map_err(|source| GrpcEndpointError::DnsCheckFailed {
                host: host.clone(),
                source,
            })?;

        if addrs.next().is_none() {
            return Err(GrpcEndpointError::DnsCheckFailed {
                host,
                source: io::Error::new(io::ErrorKind::NotFound, "dns lookup returned no addresses"),
            });
        }

        Ok(())
    }

    /// Performs one eager connection attempt to validate the full path.
    async fn run_connect_check(&self) -> Result<(), GrpcEndpointError> {
        let channel = self.connect_channel(None).await?;
        // Drop the channel -- runtime will use a separate lazy channel.
        drop(channel);
        Ok(())
    }

    /// Returns the compression encoding to apply to requests, if any.
    #[must_use]
    pub fn compression_encoding(&self) -> Option<CompressionEncoding> {
        self.compression
            .map(|method| method.map_to_compression_encoding())
    }

    /// Returns a non-zero concurrency limit.
    #[must_use]
    pub fn effective_concurrency_limit(&self) -> usize {
        self.concurrency_limit.max(1)
    }

    fn build_endpoint_from_uri(
        &self,
        grpc_endpoint: &str,
    ) -> Result<Endpoint, tonic::transport::Error> {
        // Note: TCP settings (nodelay, keepalive) set here on the Endpoint are used
        // by tonic's default connector (non-proxy case).
        // When using a proxy, we provide a custom connector which ignores these Endpoint
        // settings but applies the same configuration manually to the socket.
        let mut endpoint = Endpoint::from_shared(grpc_endpoint.to_string())?
            .concurrency_limit(self.effective_concurrency_limit())
            .connect_timeout(self.connect_timeout)
            .tcp_nodelay(self.tcp_nodelay)
            .tcp_keepalive(self.tcp_keepalive)
            .initial_stream_window_size(self.initial_stream_window_size)
            .initial_connection_window_size(self.initial_connection_window_size)
            .keep_alive_while_idle(self.keep_alive_while_idle);

        if let Some(interval) = self.http2_keepalive_interval {
            endpoint = endpoint.http2_keep_alive_interval(interval);
        }
        if let Some(timeout) = self.http2_keepalive_timeout {
            endpoint = endpoint.keep_alive_timeout(timeout);
        }
        if let Some(interval) = self.tcp_keepalive_interval {
            endpoint = endpoint.tcp_keepalive_interval(Some(interval));
        }
        if let Some(retries) = self.tcp_keepalive_retries {
            endpoint = endpoint.tcp_keepalive_retries(Some(retries));
        }
        if self.http2_adaptive_window {
            endpoint = endpoint.http2_adaptive_window(true);
        }
        if let Some(buffer_size) = self.buffer_size {
            endpoint = endpoint.buffer_size(buffer_size);
        }
        if let Some(timeout) = self.timeout {
            endpoint = endpoint.timeout(timeout);
        }
        if let Some(ua) = &self.user_agent {
            endpoint = endpoint.user_agent(ua.as_str())?;
        }

        Ok(endpoint)
    }

    /// Builds the configured [`Endpoint`].
    pub fn build_endpoint(&self) -> Result<Endpoint, tonic::transport::Error> {
        self.build_endpoint_from_uri(&self.grpc_endpoint)
    }

    /// Builds the configured [`Endpoint`], applying TLS/mTLS settings when needed.
    pub async fn build_endpoint_with_tls(&self) -> Result<Endpoint, GrpcEndpointError> {
        let endpoint = self.build_endpoint()?;

        // Decision to enable TLS is handled by load_client_tls_config (secure by default).
        let tls = tls_utils::load_client_tls_config(self.tls.as_ref(), &self.grpc_endpoint).await?;

        if let Some(tls_config) = tls {
            Ok(endpoint.tls_config(tls_config)?)
        } else {
            Ok(endpoint)
        }
    }

    /// Returns the effective proxy configuration.
    ///
    /// If explicit proxy config is provided, it's merged with environment variables
    /// (explicit values take precedence). If no explicit config is provided, reads
    /// from environment variables only.
    ///
    /// # Performance
    ///
    /// This method reads environment variables (`HTTP_PROXY`, etc.) every time it is called.
    /// It is intended to be called during client initialization (startup), not per-request.
    #[must_use]
    fn effective_proxy_config(&self) -> ProxyConfig {
        match &self.proxy {
            Some(config) => config.clone().merge_with_env(),
            None => ProxyConfig::from_env(),
        }
    }

    /// Logs an informational message if a proxy is configured for a plain text (http://) endpoint.
    ///
    /// This warns users that the configured proxy must support HTTP CONNECT for non-TLS targets,
    /// which is a common source of connection failures with some proxy servers.
    pub fn log_proxy_info(&self) {
        let proxy = self.effective_proxy_config();
        if proxy.has_proxy() && !self.grpc_endpoint.trim_start().starts_with("https://") {
            let proxy_str = proxy.to_string();
            otap_df_telemetry::otel_info!(
                "otap_grpc_exporter.proxy.configured",
                endpoint = self.grpc_endpoint.as_str(),
                proxy = proxy_str.as_str(),
                message = "Proxy configured for http:// endpoint; using HTTP CONNECT tunneling. If your proxy does not support CONNECT for HTTP targets, consider using a transparent proxy or SOCKS proxy instead."
            );
        }
    }

    fn make_proxy_connector(
        &self,
        proxy: Arc<ProxyConfig>,
    ) -> impl tower::Service<
        http::Uri,
        Response = TokioIo<crate::otap_grpc::proxy::ProxyTcpStream>,
        Error = io::Error,
        Future = impl Send + 'static,
    > + Send
    + Clone
    + 'static {
        // Capture settings at creation time.
        // Note: If GrpcClientSettings are modified after this connector is created,
        // those changes will NOT be reflected in the connector.
        let tcp_nodelay = self.tcp_nodelay;
        let tcp_keepalive = self.tcp_keepalive;
        let tcp_keepalive_interval = self.tcp_keepalive_interval;
        let tcp_keepalive_retries = self.tcp_keepalive_retries;

        service_fn(move |uri: http::Uri| {
            let proxy = Arc::clone(&proxy);
            async move {
                // The connection timeout is handled by the tonic::Endpoint configuration
                // (via .connect_timeout()), which wraps this connector service.
                // We don't need an additional timeout here.
                crate::otap_grpc::proxy::connect_tcp_stream_with_proxy_config(
                    &uri,
                    proxy.as_ref(),
                    tcp_nodelay,
                    tcp_keepalive,
                    tcp_keepalive_interval,
                    tcp_keepalive_retries,
                )
                .await
                .map(TokioIo::new)
                .map_err(io::Error::other)
            }
        })
    }

    async fn prepare_connection(
        &self,
        timeout_override: Option<Duration>,
    ) -> Result<(Endpoint, Arc<ProxyConfig>), GrpcEndpointError> {
        let mut endpoint = self.build_endpoint_with_tls().await?;
        if let Some(timeout) = timeout_override {
            endpoint = endpoint.timeout(timeout);
        }
        let proxy = Arc::new(self.effective_proxy_config());
        Ok((endpoint, proxy))
    }

    /// Builds a gRPC channel, using proxy tunneling when configured.
    #[doc(hidden)]
    pub async fn connect_channel(
        &self,
        timeout_override: Option<Duration>,
    ) -> Result<Channel, GrpcEndpointError> {
        let (endpoint, proxy) = self.prepare_connection(timeout_override).await?;
        if proxy.has_proxy() {
            let connector = self.make_proxy_connector(proxy);
            Ok(endpoint.connect_with_connector(connector).await?)
        } else {
            Ok(endpoint.connect().await?)
        }
    }

    /// Builds a lazy gRPC channel, using proxy tunneling when configured.
    #[doc(hidden)]
    pub async fn connect_channel_lazy(
        &self,
        timeout_override: Option<Duration>,
    ) -> Result<Channel, GrpcEndpointError> {
        let (endpoint, proxy) = self.prepare_connection(timeout_override).await?;
        if proxy.has_proxy() {
            let connector = self.make_proxy_connector(proxy);
            Ok(endpoint.connect_with_connector_lazy(connector))
        } else {
            Ok(endpoint.connect_lazy())
        }
    }
}

impl Default for GrpcClientSettings {
    fn default() -> Self {
        Self {
            grpc_endpoint: String::new(),
            compression: None,
            concurrency_limit: default_concurrency_limit(),
            connect_timeout: default_connect_timeout(),
            tcp_nodelay: default_tcp_nodelay(),
            tcp_keepalive: default_tcp_keepalive(),
            tcp_keepalive_interval: None,
            tcp_keepalive_retries: None,
            initial_stream_window_size: default_initial_stream_window_size(),
            initial_connection_window_size: default_initial_connection_window_size(),
            http2_adaptive_window: default_http2_adaptive_window(),
            http2_keepalive_interval: default_http2_keepalive_interval(),
            http2_keepalive_timeout: default_http2_keepalive_timeout(),
            keep_alive_while_idle: default_keep_alive_while_idle(),
            timeout: None,
            tls: None,
            buffer_size: None,
            startup_check: StartupCheck::default(),
            proxy: None,
            user_agent: None,
        }
    }
}

pub(crate) const fn default_concurrency_limit() -> usize {
    256
}

pub(crate) const fn default_connect_timeout() -> Duration {
    Duration::from_secs(3)
}

pub(crate) const fn default_tcp_nodelay() -> bool {
    true
}

pub(crate) const fn default_tcp_keepalive() -> Option<Duration> {
    Some(Duration::from_secs(45))
}

const fn default_initial_stream_window_size() -> Option<u32> {
    Some(8 * 1024 * 1024)
}

const fn default_initial_connection_window_size() -> Option<u32> {
    Some(32 * 1024 * 1024)
}

const fn default_http2_adaptive_window() -> bool {
    false
}

const fn default_http2_keepalive_interval() -> Option<Duration> {
    Some(Duration::from_secs(30))
}

const fn default_http2_keepalive_timeout() -> Option<Duration> {
    Some(Duration::from_secs(10))
}

const fn default_keep_alive_while_idle() -> bool {
    true
}

/// Deserializes `grpc_endpoint` while validating that the value is a well-formed URI with an
/// `http` or `https` scheme.
fn deserialize_grpc_endpoint<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let endpoint = String::deserialize(deserializer)?;
    validate_grpc_endpoint(&endpoint).map_err(serde::de::Error::custom)?;
    Ok(endpoint.trim().to_string())
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use otap_df_config::tls::{TlsClientConfig, TlsConfig};
    use tempfile::NamedTempFile;

    #[test]
    fn defaults_match_previous_client_tuning() {
        let settings: GrpcClientSettings =
            serde_json::from_str(r#"{ "grpc_endpoint": "http://localhost:4317" }"#).unwrap();

        assert_eq!(settings.concurrency_limit, 256);
        assert_eq!(settings.connect_timeout, Duration::from_secs(3));
        assert_eq!(settings.tcp_keepalive, Some(Duration::from_secs(45)));
        assert_eq!(
            settings.http2_keepalive_interval,
            Some(Duration::from_secs(30))
        );
        assert_eq!(
            settings.http2_keepalive_timeout,
            Some(Duration::from_secs(10))
        );
        assert!(settings.keep_alive_while_idle);
    }

    #[test]
    fn compression_encoding_is_reported() {
        let settings: GrpcClientSettings = serde_json::from_str(
            r#"{ "grpc_endpoint": "http://localhost:4317", "compression_method": "gzip" }"#,
        )
        .unwrap();

        assert_eq!(
            settings.compression_encoding(),
            Some(CompressionEncoding::Gzip)
        );
    }

    #[test]
    fn effective_concurrency_limit_clamps_to_one() {
        let settings: GrpcClientSettings = serde_json::from_str(
            r#"{ "grpc_endpoint": "http://localhost:4317", "concurrency_limit": 0 }"#,
        )
        .unwrap();

        assert_eq!(settings.effective_concurrency_limit(), 1);
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_allows_plain_http_when_tls_unset() {
        crate::crypto::ensure_crypto_provider();
        let settings: GrpcClientSettings =
            serde_json::from_str(r#"{ "grpc_endpoint": "http://localhost:4317" }"#).unwrap();
        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        let _ = endpoint;
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_accepts_https_without_explicit_tls_block() {
        crate::crypto::ensure_crypto_provider();
        let settings: GrpcClientSettings = serde_json::from_str(
            r#"{ "grpc_endpoint": "https://localhost:4317", "tcp_nodelay": true }"#,
        )
        .unwrap();

        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        let _ = endpoint;
    }

    #[tokio::test]
    async fn client_tls_defaults_are_scheme_driven_when_tls_block_absent() {
        crate::crypto::ensure_crypto_provider();
        // No tls: block => scheme decides
        let http = tls_utils::load_client_tls_config(None, "http://localhost:4317")
            .await
            .unwrap();
        assert!(http.is_none());

        let https = tls_utils::load_client_tls_config(None, "https://localhost:4317")
            .await
            .unwrap();
        assert!(https.is_some());
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_insecure_does_not_rewrite_scheme() {
        crate::crypto::ensure_crypto_provider();
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                insecure: Some(true),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        assert_eq!(endpoint.uri().scheme_str(), Some("https"));
    }

    #[tokio::test]
    async fn client_tls_insecure_true_without_custom_ca_returns_no_explicit_tls_config() {
        crate::crypto::ensure_crypto_provider();
        let cfg = TlsClientConfig {
            insecure: Some(true),
            ..TlsClientConfig::default()
        };

        let tls = tls_utils::load_client_tls_config(Some(&cfg), "https://localhost:4317")
            .await
            .unwrap();
        assert!(tls.is_none());
    }

    #[tokio::test]
    async fn client_tls_insecure_true_with_custom_ca_still_builds_tls_config() {
        crate::crypto::ensure_crypto_provider();
        let cfg = TlsClientConfig {
            insecure: Some(true),
            ca_pem: Some(
                "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n".to_string(),
            ),
            include_system_ca_certs_pool: Some(false),
            ..TlsClientConfig::default()
        };

        let tls = tls_utils::load_client_tls_config(Some(&cfg), "http://localhost:4317")
            .await
            .unwrap();
        assert!(tls.is_some());
    }

    #[tokio::test]
    async fn client_tls_insecure_skip_verify_true_fails_fast() {
        crate::crypto::ensure_crypto_provider();
        let cfg = TlsClientConfig {
            insecure_skip_verify: Some(true),
            ..TlsClientConfig::default()
        };

        let err = tls_utils::load_client_tls_config(Some(&cfg), "https://localhost:4317")
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("insecure_skip_verify=true is not supported")
        );
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_allows_http_when_tls_is_configured() {
        crate::crypto::ensure_crypto_provider();
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                config: TlsConfig::default(),
                ca_file: None,
                ca_pem: Some(
                    "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n".to_string(),
                ),
                include_system_ca_certs_pool: Some(false),
                server_name: Some("localhost".to_string()),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        // Should succeed now (TLS enabled regardless of scheme)
        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        let _ = endpoint;
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_rejects_partial_mtls_cert_without_key() {
        crate::crypto::ensure_crypto_provider();
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                config: TlsConfig {
                    cert_pem: Some(
                        "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n"
                            .to_string(),
                    ),
                    ..TlsConfig::default()
                },
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        assert!(err.to_string().contains("certificate") || err.to_string().contains("mTLS"));
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_rejects_partial_mtls_key_without_cert() {
        crate::crypto::ensure_crypto_provider();
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                config: TlsConfig {
                    key_pem: Some(
                        "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----\n"
                            .to_string(),
                    ),
                    ..TlsConfig::default()
                },
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        assert!(err.to_string().contains("certificate") || err.to_string().contains("mTLS"));
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_errors_when_ca_file_missing() {
        crate::crypto::ensure_crypto_provider();
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                ca_file: Some("/this/path/should/not/exist/ca.pem".into()),
                include_system_ca_certs_pool: Some(false),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        let err_msg = err.to_string().to_lowercase();
        assert!(
            err_msg.contains("no such")
                || err_msg.contains("not found")
                || err_msg.contains("cannot find"),
            "unexpected error message: {err_msg}"
        );
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_enforces_tls_file_size_limit() {
        crate::crypto::ensure_crypto_provider();
        // Create a CA file > 4MB (tls_utils MAX_TLS_FILE_SIZE).
        let mut tmp = NamedTempFile::new().unwrap();
        let oversized = vec![b'a'; 4 * 1024 * 1024 + 1];
        use std::io::Write;
        tmp.write_all(&oversized).unwrap();

        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                ca_file: Some(tmp.path().to_path_buf()),
                include_system_ca_certs_pool: Some(false),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("too")
                || err.to_string().to_lowercase().contains("limit")
        );
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_errors_when_no_trust_anchors() {
        crate::crypto::ensure_crypto_provider();
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                include_system_ca_certs_pool: Some(false),
                // No ca_file or ca_pem provided
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        assert!(
            err.to_string().contains("trust anchor") || err.to_string().contains("no trust"),
            "Expected trust anchor error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_enforces_cert_file_size_limit() {
        crate::crypto::ensure_crypto_provider();
        // Create a client cert file > 4MB (tls_utils MAX_TLS_FILE_SIZE).
        let mut cert_tmp = NamedTempFile::new().unwrap();
        let mut key_tmp = NamedTempFile::new().unwrap();
        let oversized = vec![b'a'; 4 * 1024 * 1024 + 1];
        use std::io::Write;
        cert_tmp.write_all(&oversized).unwrap();
        key_tmp.write_all(b"dummy key").unwrap();

        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                config: TlsConfig {
                    cert_file: Some(cert_tmp.path().to_path_buf()),
                    key_file: Some(key_tmp.path().to_path_buf()),
                    ..TlsConfig::default()
                },
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("too")
                || err.to_string().to_lowercase().contains("limit"),
            "Expected file size limit error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_fails_with_empty_ca_pem() {
        crate::crypto::ensure_crypto_provider();
        // Test 1: Empty ca_pem with system CAs disabled → "no trust anchors" error
        let settings1 = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                ca_pem: Some("".to_string()),
                include_system_ca_certs_pool: Some(false),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err1 = settings1.build_endpoint_with_tls().await.unwrap_err();
        assert!(
            err1.to_string().contains("no trust anchors"),
            "Expected 'no trust anchors' error, got: {}",
            err1
        );

        // Test 2: Whitespace-only ca_pem with system CAs enabled → "ca_pem is empty" error
        let settings2 = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                ca_pem: Some("   ".to_string()),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err2 = settings2.build_endpoint_with_tls().await.unwrap_err();
        assert!(
            err2.to_string().contains("ca_pem is set but empty"),
            "Expected 'ca_pem is empty' error, got: {}",
            err2
        );
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_accepts_server_name_override() {
        crate::crypto::ensure_crypto_provider();
        // server_name_override should be accepted and not cause an error
        // (actual SNI behavior would require integration test with a real server)
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://127.0.0.1:4317".to_string(),
            tls: Some(TlsClientConfig {
                server_name: Some("custom.hostname.example.com".to_string()),
                // Use system CAs (default) so we have trust anchors
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        // Should successfully build the endpoint (SNI override is just configuration)
        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        let _ = endpoint;
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_enables_tls_by_default_on_http() {
        crate::crypto::ensure_crypto_provider();
        // Secure by default: http:// endpoint with empty TLS block should enable TLS (system roots)
        // because keys/certs are optional and insecure is false by default.
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://localhost:4317".to_string(),
            tls: Some(TlsClientConfig::default()), // Empty TLS config, insecure=false implicitly
            ..GrpcClientSettings::default()
        };

        // Should NOT error now
        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        let _ = endpoint;
    }

    #[tokio::test]
    async fn build_endpoint_with_tls_disables_tls_when_insecure_true() {
        crate::crypto::ensure_crypto_provider();
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                insecure: Some(true),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        // Verification: endpoint shouldn't have TLS config?
        // We can't verify that easily, but we know it returned successfully.
        let _ = endpoint;
    }

    #[test]
    fn test_proxy_config_deserialization() {
        let json = r#"{
            "grpc_endpoint": "http://localhost:4317",
            "proxy": {
                "http_proxy": "http://proxy:3128",
                "no_proxy": "localhost"
            }
        }"#;
        let settings: GrpcClientSettings = serde_json::from_str(json).unwrap();
        assert!(settings.proxy.is_some());
        let proxy = settings.proxy.as_ref().unwrap();
        assert_eq!(
            proxy.http_proxy.as_ref().map(|u| u.expose()),
            Some("http://proxy:3128")
        );
        assert_eq!(proxy.no_proxy.as_deref(), Some("localhost"));
    }

    #[test]
    fn test_effective_proxy_config_preserves_explicit() {
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://localhost:4317".to_string(),
            proxy: Some(ProxyConfig {
                http_proxy: Some("http://explicit-proxy:3128".into()),
                ..Default::default()
            }),
            ..GrpcClientSettings::default()
        };

        let effective = settings.effective_proxy_config();
        // Even if env vars are set, explicit should take precedence
        assert_eq!(
            effective.http_proxy.as_ref().map(|u| u.expose()),
            Some("http://explicit-proxy:3128")
        );
    }

    #[test]
    fn test_has_proxy_logic() {
        let config = ProxyConfig {
            http_proxy: Some("http://proxy".into()),
            ..Default::default()
        };
        assert!(config.has_proxy());

        let config = ProxyConfig {
            https_proxy: Some("http://proxy".into()),
            ..Default::default()
        };
        assert!(config.has_proxy());

        let config = ProxyConfig {
            all_proxy: Some("http://proxy".into()),
            ..Default::default()
        };
        assert!(config.has_proxy());
    }

    // --- Endpoint validation tests ---

    #[test]
    fn validate_accepts_valid_http_endpoint() {
        validate_grpc_endpoint("http://localhost:4317").unwrap();
    }

    #[test]
    fn validate_accepts_valid_https_endpoint() {
        validate_grpc_endpoint("https://collector.example.com:4317").unwrap();
    }

    #[test]
    fn validate_accepts_endpoint_with_path() {
        validate_grpc_endpoint("http://localhost:4317/v1/traces").unwrap();
    }

    #[test]
    fn validate_accepts_ipv4_endpoint() {
        validate_grpc_endpoint("http://192.168.1.1:4317").unwrap();
    }

    #[test]
    fn validate_accepts_ipv6_endpoint() {
        validate_grpc_endpoint("http://[::1]:4317").unwrap();
    }

    #[test]
    fn validate_rejects_empty_endpoint() {
        let err = validate_grpc_endpoint("").unwrap_err();
        assert!(
            err.contains("empty"),
            "expected 'empty' in error, got: {err}"
        );
    }

    #[test]
    fn validate_rejects_whitespace_only_endpoint() {
        let err = validate_grpc_endpoint("   ").unwrap_err();
        assert!(
            err.contains("empty"),
            "expected 'empty' in error, got: {err}"
        );
    }

    #[test]
    fn validate_accepts_endpoint_without_scheme() {
        validate_grpc_endpoint("localhost:4317").unwrap();
    }

    #[test]
    fn validate_rejects_unsupported_scheme() {
        let err = validate_grpc_endpoint("ftp://localhost:4317").unwrap_err();
        assert!(
            err.contains("unsupported scheme"),
            "expected 'unsupported scheme' in error, got: {err}"
        );
    }

    #[test]
    fn validate_rejects_invalid_uri() {
        assert!(validate_grpc_endpoint("not a valid url!!!").is_err());
    }

    // --- StartupCheck deserialization tests ---

    #[test]
    fn startup_check_defaults_to_none() {
        let settings: GrpcClientSettings =
            serde_json::from_str(r#"{ "grpc_endpoint": "http://localhost:4317" }"#).unwrap();
        assert_eq!(settings.startup_check, StartupCheck::None);
    }

    #[test]
    fn startup_check_deserializes_none() {
        let settings: GrpcClientSettings = serde_json::from_str(
            r#"{ "grpc_endpoint": "http://localhost:4317", "startup_check": "none" }"#,
        )
        .unwrap();
        assert_eq!(settings.startup_check, StartupCheck::None);
    }

    #[test]
    fn startup_check_deserializes_dns() {
        let settings: GrpcClientSettings = serde_json::from_str(
            r#"{ "grpc_endpoint": "http://localhost:4317", "startup_check": "dns" }"#,
        )
        .unwrap();
        assert_eq!(settings.startup_check, StartupCheck::Dns);
    }

    #[test]
    fn startup_check_deserializes_connect() {
        let settings: GrpcClientSettings = serde_json::from_str(
            r#"{ "grpc_endpoint": "http://localhost:4317", "startup_check": "connect" }"#,
        )
        .unwrap();
        assert_eq!(settings.startup_check, StartupCheck::Connect);
    }

    #[test]
    fn startup_check_rejects_unknown_value() {
        let result = serde_json::from_str::<GrpcClientSettings>(
            r#"{ "grpc_endpoint": "http://localhost:4317", "startup_check": "invalid" }"#,
        );
        assert!(result.is_err());
    }

    // --- Startup check runtime tests ---

    #[tokio::test]
    async fn startup_check_none_always_succeeds() {
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://this.host.does.not.exist.invalid:4317".to_string(),
            startup_check: StartupCheck::None,
            ..GrpcClientSettings::default()
        };
        // None mode does no I/O, so it succeeds even for unresolvable hosts.
        settings.run_startup_check().await.unwrap();
    }

    #[tokio::test]
    async fn startup_check_dns_resolves_localhost() {
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://localhost:4317".to_string(),
            startup_check: StartupCheck::Dns,
            ..GrpcClientSettings::default()
        };
        settings.run_startup_check().await.unwrap();
    }

    #[tokio::test]
    async fn startup_check_dns_fails_unresolvable() {
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://this.host.definitely.does.not.exist.invalid:4317".to_string(),
            startup_check: StartupCheck::Dns,
            ..GrpcClientSettings::default()
        };
        let err = settings.run_startup_check().await.unwrap_err();
        assert!(
            matches!(err, GrpcEndpointError::DnsCheckFailed { .. }),
            "expected DnsCheckFailed, got: {err}"
        );
    }

    #[tokio::test]
    async fn startup_check_dns_skipped_when_proxy_configured() {
        // When a proxy is configured and the endpoint is NOT in no_proxy,
        // the DNS check should be skipped (proxy resolves the target).
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://this.host.definitely.does.not.exist.invalid:4317".to_string(),
            startup_check: StartupCheck::Dns,
            proxy: Some(ProxyConfig {
                http_proxy: Some("http://my-proxy:3128".into()),
                ..Default::default()
            }),
            ..GrpcClientSettings::default()
        };
        // Should succeed because the proxy handles resolution.
        settings.run_startup_check().await.unwrap();
    }

    #[tokio::test]
    async fn startup_check_dns_not_skipped_when_proxy_does_not_apply_to_endpoint_scheme() {
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://this.host.definitely.does.not.exist.invalid:4317".to_string(),
            startup_check: StartupCheck::Dns,
            proxy: Some(ProxyConfig {
                https_proxy: Some("http://my-proxy:3128".into()),
                ..Default::default()
            }),
            ..GrpcClientSettings::default()
        };

        let err = settings.run_startup_check().await.unwrap_err();
        assert!(
            matches!(err, GrpcEndpointError::DnsCheckFailed { .. }),
            "expected DnsCheckFailed, got: {err}"
        );
    }

    #[tokio::test]
    async fn startup_check_dns_not_skipped_when_endpoint_bypasses_proxy() {
        // When the endpoint IS in no_proxy, DNS check should still run.
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://this.host.definitely.does.not.exist.invalid:4317".to_string(),
            startup_check: StartupCheck::Dns,
            proxy: Some(ProxyConfig {
                http_proxy: Some("http://my-proxy:3128".into()),
                no_proxy: Some("*.invalid".to_string()),
                ..Default::default()
            }),
            ..GrpcClientSettings::default()
        };
        // Should fail because the endpoint bypasses the proxy and can't resolve.
        let err = settings.run_startup_check().await.unwrap_err();
        assert!(
            matches!(err, GrpcEndpointError::DnsCheckFailed { .. }),
            "expected DnsCheckFailed, got: {err}"
        );
    }

    #[tokio::test]
    async fn startup_check_connect_succeeds() {
        crate::crypto::ensure_crypto_provider();
        let port = portpicker::pick_unused_port().expect("no free port");
        // Bind a TCP listener so the eager connect attempt has something to connect to.
        let _listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        let settings = GrpcClientSettings {
            grpc_endpoint: format!("http://127.0.0.1:{port}"),
            startup_check: StartupCheck::Connect,
            connect_timeout: Duration::from_secs(2),
            ..GrpcClientSettings::default()
        };
        settings.run_startup_check().await.unwrap();
    }

    #[tokio::test]
    async fn startup_check_connect_fails_connection_refused() {
        crate::crypto::ensure_crypto_provider();
        let port = portpicker::pick_unused_port().expect("no free port");
        let settings = GrpcClientSettings {
            grpc_endpoint: format!("http://127.0.0.1:{port}"),
            startup_check: StartupCheck::Connect,
            connect_timeout: Duration::from_secs(2),
            ..GrpcClientSettings::default()
        };
        let err = settings.run_startup_check().await.unwrap_err();
        assert!(
            matches!(err, GrpcEndpointError::Tonic(_)),
            "expected Tonic transport error, got: {err}"
        );
    }

    #[tokio::test]
    async fn startup_check_dns_uses_default_port_for_https() {
        // Endpoint without explicit port and https scheme should default to 443.
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost".to_string(),
            startup_check: StartupCheck::Dns,
            ..GrpcClientSettings::default()
        };
        // localhost should resolve regardless of port.
        settings.run_startup_check().await.unwrap();
    }

    #[tokio::test]
    async fn test_user_agent_sent_on_grpc_wire() {
        use bytes::Bytes;
        use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
        use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
        use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_server::{
            LogsService, LogsServiceServer,
        };
        use prost::Message;
        use tokio::sync::mpsc;
        use tonic::transport::Server;
        use tonic::{Request, Response, Status};

        use crate::otap_grpc::otlp::client::LogsServiceClient;

        // Verify default has no user_agent
        let defaults = GrpcClientSettings::default();
        assert_eq!(defaults.user_agent, None);

        // Verify deserialization
        let deserialized: GrpcClientSettings = serde_json::from_str(
            r#"{ "grpc_endpoint": "http://localhost:4317", "user_agent": "my-app/1.0" }"#,
        )
        .unwrap();
        assert_eq!(deserialized.user_agent.as_deref(), Some("my-app/1.0"));

        // Verify the header actually arrives on the wire
        struct UserAgentCapture {
            sender: mpsc::Sender<String>,
        }

        #[tonic::async_trait]
        impl LogsService for UserAgentCapture {
            async fn export(
                &self,
                request: Request<ExportLogsServiceRequest>,
            ) -> Result<Response<ExportLogsServiceResponse>, Status> {
                let ua = request
                    .metadata()
                    .get("user-agent")
                    .map(|v| v.to_str().unwrap().to_string())
                    .unwrap_or_default();
                let _ = self.sender.send(ua).await;
                Ok(Response::new(ExportLogsServiceResponse {
                    partial_success: None,
                }))
            }
        }

        let (tx, mut rx) = mpsc::channel::<String>(1);
        let service = LogsServiceServer::new(UserAgentCapture { sender: tx });

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

        let server_handle = tokio::spawn(async move {
            Server::builder()
                .add_service(service)
                .serve_with_incoming(incoming)
                .await
                .unwrap();
        });

        let settings = GrpcClientSettings {
            grpc_endpoint: format!("http://127.0.0.1:{}", addr.port()),
            user_agent: Some("my-app/1.0".to_string()),
            ..GrpcClientSettings::default()
        };

        let endpoint = settings.build_endpoint().unwrap();
        let channel = endpoint.connect().await.unwrap();

        let mut client = LogsServiceClient::new(channel);
        let req = ExportLogsServiceRequest {
            resource_logs: Vec::new(),
        };
        let mut buf = Vec::new();
        req.encode(&mut buf).unwrap();
        let _ = client.export(Bytes::from(buf)).await.unwrap();

        let observed_ua = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap();

        // Tonic prepends custom user-agent before its default
        assert!(
            observed_ua.contains("my-app/1.0"),
            "Expected user-agent to contain 'my-app/1.0', got: {observed_ua}"
        );

        server_handle.abort();
    }
}
