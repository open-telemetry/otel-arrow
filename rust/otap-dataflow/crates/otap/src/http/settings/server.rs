// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for OTLP HTTP servers.

use otel_arrow_dfe_config::byte_units;
use otel_arrow_dfe_config::tls::TlsServerConfig;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;

/// Settings for the OTLP/HTTP server.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpServerSettings {
    /// Listening address for OTLP/HTTP (typically `0.0.0.0:4318`).
    pub listening_addr: SocketAddr,

    /// Maximum number of concurrent in-flight requests.
    /// Defaults to `0`, which means the receiver adopts the downstream pdata channel capacity.
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: usize,

    /// Whether newly accepted sockets should have `TCP_NODELAY` enabled.
    #[serde(default = "default_tcp_nodelay")]
    pub tcp_nodelay: bool,

    /// TCP keepalive timeout for accepted sockets.
    #[serde(default = "default_tcp_keepalive", with = "humantime_serde")]
    pub tcp_keepalive: Option<Duration>,

    /// Interval between TCP keepalive probes.
    #[serde(default = "default_tcp_keepalive_interval", with = "humantime_serde")]
    pub tcp_keepalive_interval: Option<Duration>,

    /// Number of TCP keepalive probes before a connection is declared dead.
    #[serde(default = "default_tcp_keepalive_retries")]
    pub tcp_keepalive_retries: Option<u32>,

    /// Maximum request body size, in bytes.
    /// Accepts plain integers or suffixed strings such as `4MiB`.
    ///
    /// Enforced for both compressed wire bytes and the decompressed request body.
    #[serde(
        default = "default_max_request_body_size",
        deserialize_with = "deserialize_body_size"
    )]
    pub max_request_body_size: u32,

    /// Whether to wait for the immediate downstream ACK/NACK before responding.
    #[serde(default = "default_wait_for_result")]
    pub wait_for_result: bool,

    /// Timeout for processing a request (send + optional wait).
    ///
    /// Defaults to a conservative value to reduce the risk of slow-client denial of service.
    #[serde(default = "default_http_timeout", with = "humantime_serde")]
    pub timeout: Option<Duration>,

    /// Whether to accept gzip/deflate HTTP request bodies (via `Content-Encoding`).
    ///
    /// When disabled, non-identity encodings are rejected with `415 Unsupported Media Type`.
    #[serde(default = "default_accept_compressed_requests")]
    pub accept_compressed_requests: bool,

    /// Optional TLS configuration.
    #[serde(default)]
    pub tls: Option<TlsServerConfig>,
}

impl Default for HttpServerSettings {
    fn default() -> Self {
        Self {
            listening_addr: "127.0.0.1:4318".parse().expect("valid default addr"),
            max_concurrent_requests: default_max_concurrent_requests(),
            tcp_nodelay: default_tcp_nodelay(),
            tcp_keepalive: default_tcp_keepalive(),
            tcp_keepalive_interval: default_tcp_keepalive_interval(),
            tcp_keepalive_retries: default_tcp_keepalive_retries(),
            max_request_body_size: default_max_request_body_size(),
            wait_for_result: default_wait_for_result(),
            timeout: default_http_timeout(),
            accept_compressed_requests: default_accept_compressed_requests(),
            tls: None,
        }
    }
}

const fn default_max_concurrent_requests() -> usize {
    0
}

const fn default_tcp_nodelay() -> bool {
    true
}

const fn default_tcp_keepalive() -> Option<Duration> {
    Some(Duration::from_secs(45))
}

const fn default_tcp_keepalive_interval() -> Option<Duration> {
    Some(Duration::from_secs(15))
}

const fn default_tcp_keepalive_retries() -> Option<u32> {
    Some(5)
}

const fn default_max_request_body_size() -> u32 {
    // Keep this aligned with tonic's default max-decoding size (4MiB) for symmetry.
    4 * 1024 * 1024
}

fn deserialize_body_size<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(byte_units::deserialize(deserializer)?.unwrap_or(default_max_request_body_size()))
}

const fn default_wait_for_result() -> bool {
    false
}

const fn default_accept_compressed_requests() -> bool {
    true
}

const fn default_http_timeout() -> Option<Duration> {
    Some(Duration::from_secs(30))
}

/// Tune max concurrent requests relative to downstream capacity.
pub fn tune_max_concurrent_requests(config: &mut HttpServerSettings, downstream_capacity: usize) {
    let safe_capacity = downstream_capacity.max(1);
    if config.max_concurrent_requests == 0 || config.max_concurrent_requests > safe_capacity {
        config.max_concurrent_requests = safe_capacity;
    }

    // Preserve a bounded timeout even when configuration explicitly sets it to null.
    if config.timeout.is_none() {
        config.timeout = default_http_timeout();
    }
}
