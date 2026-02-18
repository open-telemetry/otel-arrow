// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared configuration for gRPC-based clients.

use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use std::time::Duration;
use tonic::codec::CompressionEncoding;
use tower::limit::ConcurrencyLimitLayer;

use crate::compression::CompressionMethod;
use crate::otap_grpc::{
    client_settings::{
        default_concurrency_limit, default_connect_timeout, default_tcp_keepalive,
        default_tcp_nodelay,
    },
    proxy::ProxyConfig,
};

/// Common configuration shared across HTTP clients.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpClientSettings {
    // TODO it's kind of weird to have this here? the client could connect to any endpoint ...
    /// The HTTP endpoint to connect to
    pub endpoint: String,

    // TODO this will happen external to the client, so maybe does not make sense to be here
    // TODO should we also specify that this same compression will be accepted for responses?
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

    /// Timeout for HTTP requests. If not specified, no timeout is applied.
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,

    // TODO buffer size?
    /// HTTP/HTTPS proxy configuration.
    /// If not specified, proxy settings are read from environment variables:
    /// - `HTTP_PROXY` / `http_proxy`: Proxy for HTTP connections
    /// - `HTTPS_PROXY` / `https_proxy`: Proxy for HTTPS connections
    /// - `ALL_PROXY` / `all_proxy`: Fallback proxy for all connections
    /// - `NO_PROXY` / `no_proxy`: Comma-separated list of hosts to bypass proxy
    #[serde(default)]
    #[doc(hidden)]
    pub proxy: Option<ProxyConfig>,
    // TODO TLS
}

impl HttpClientSettings {
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

    /// Returns a configured client-bulder
    pub fn client_builder(&self) -> ClientBuilder {
        let mut client_builder = ClientBuilder::new()
            .connect_timeout(self.connect_timeout)
            .tcp_nodelay(self.tcp_nodelay)
            .connector_layer(ConcurrencyLimitLayer::new(
                self.effective_concurrency_limit(),
            ));
        // TODO compressison

        if let Some(tcp_keepalive) = self.tcp_keepalive {
            client_builder = client_builder.tcp_keepalive(tcp_keepalive);
        }

        if let Some(tcp_keepalive_interval) = self.tcp_keepalive_interval {
            client_builder = client_builder.tcp_keepalive_interval(tcp_keepalive_interval)
        }

        if let Some(timeout) = self.timeout {
            client_builder = client_builder.timeout(timeout)
        }

        client_builder
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
    pub(crate) fn log_proxy_info(&self) {
        let proxy = self.effective_proxy_config();
        if proxy.has_proxy() && !self.endpoint.trim_start().starts_with("https://") {
            let proxy_str = proxy.to_string();
            otap_df_telemetry::otel_info!(
                "proxy.configured",
                endpoint = self.endpoint.as_str(),
                proxy = proxy_str.as_str(),
                message = "Proxy configured for http:// endpoint; using HTTP CONNECT tunneling. If your proxy does not support CONNECT for HTTP targets, consider using a transparent proxy or SOCKS proxy instead."
            );
        }
    }
}
