// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared configuration for gRPC-based clients.

use crate::compression::CompressionMethod;
use otap_df_config::byte_units;
use serde::Deserialize;
use std::time::Duration;
use tonic::codec::CompressionEncoding;
use tonic::transport::Endpoint;

/// Common configuration shared across gRPC clients.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GrpcClientSettings {
    /// The gRPC endpoint to connect to.
    pub grpc_endpoint: String,

    /// Compression method to use for outbound requests. Defaults to no compression.
    #[serde(default, alias = "compression_method")]
    pub compression: Option<CompressionMethod>,

    /// Maximum number of concurrent in-flight requests allowed by the transport stack.
    #[serde(default = "default_concurrency_limit")]
    pub concurrency_limit: usize,

    /// Timeout for establishing TCP connections.
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

    /// Internal Tower buffer size for the gRPC client.
    #[serde(default)]
    pub buffer_size: Option<usize>,
}

impl GrpcClientSettings {
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

    /// Builds the configured [`Endpoint`].
    pub fn build_endpoint(&self) -> Result<Endpoint, tonic::transport::Error> {
        let mut endpoint = Endpoint::from_shared(self.grpc_endpoint.clone())?
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

        Ok(endpoint)
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
            buffer_size: None,
        }
    }
}

const fn default_concurrency_limit() -> usize {
    256
}

const fn default_connect_timeout() -> Duration {
    Duration::from_secs(3)
}

const fn default_tcp_nodelay() -> bool {
    true
}

const fn default_tcp_keepalive() -> Option<Duration> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
