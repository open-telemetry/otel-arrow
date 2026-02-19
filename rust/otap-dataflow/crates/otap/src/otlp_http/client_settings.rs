// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared configuration for gRPC-based clients.

use reqwest::ClientBuilder;
use serde::Deserialize;
use std::time::Duration;
use tower::limit::ConcurrencyLimitLayer;

use crate::otap_grpc::client_settings::{
    default_concurrency_limit, default_connect_timeout, default_tcp_keepalive, default_tcp_nodelay,
};

/// Common configuration shared across HTTP clients.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpClientSettings {
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
}

impl HttpClientSettings {
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
}

impl Default for HttpClientSettings {
    fn default() -> Self {
        Self {
            concurrency_limit: default_concurrency_limit(),
            connect_timeout: default_connect_timeout(),
            tcp_nodelay: default_tcp_nodelay(),
            tcp_keepalive: default_tcp_keepalive(),
            tcp_keepalive_interval: None,
            timeout: None,
        }
    }
}
