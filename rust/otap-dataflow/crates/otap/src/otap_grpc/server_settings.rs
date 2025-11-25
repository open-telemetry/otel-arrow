// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared configuration for gRPC-based receivers.

use crate::compression::{self, CompressionMethod};
use crate::otap_grpc::otlp::server::Settings;
use otap_df_config::byte_units;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tonic::codec::EnabledCompressionEncodings;
use tonic::transport::server::TcpIncoming;

/// Common configuration shared across gRPC receivers.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GrpcServerSettings {
    /// The endpoint details: protocol, name, port.
    pub listening_addr: SocketAddr,

    /// Compression methods accepted for requests. Omitted field defaults to accepting zstd, gzip,
    /// and deflate (in that preference order).
    #[serde(
        default,
        deserialize_with = "compression::deserialize_compression_methods"
    )]
    pub request_compression: Option<Vec<CompressionMethod>>,

    /// Compression methods used for responses. Defaults to no compression, falling back to the
    /// request list when explicitly configured via the legacy `compression_method` option.
    #[serde(
        default,
        deserialize_with = "compression::deserialize_compression_methods"
    )]
    pub response_compression: Option<Vec<CompressionMethod>>,

    // --- All the following settings have defaults that should be reasonable for most users ---
    // -----------------------------------------------------------------------------------------
    /// Maximum number of concurrent in-flight requests.
    /// Defaults to `0`, which means the receiver adopts the downstream pdata channel capacity so
    /// backpressure flows upstream automatically. Any non-zero value is still clamped to that
    /// capacity at runtime.
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: usize,

    /// Whether newly accepted sockets should have `TCP_NODELAY` enabled.
    /// Keeping this `true` (the default) avoids Nagle's algorithm and minimizes per-export latency.
    /// Disabling it trades slightly higher latency for fewer small TCP packets when workloads
    /// involve very bursty, tiny messages.
    #[serde(default = "default_tcp_nodelay")]
    pub tcp_nodelay: bool,

    /// TCP keepalive timeout for accepted sockets.
    /// The 45s default evicts dead clients in under a minute without incurring much background
    /// traffic. Raise it to reduce keepalive chatter, or set to `null` to disable kernel keepalives
    /// entirely (at the cost of slower leak detection on broken links).
    #[serde(default = "default_tcp_keepalive", with = "humantime_serde")]
    pub tcp_keepalive: Option<Duration>,

    /// Interval between TCP keepalive probes once keepalive is active.
    /// Defaults to 15s so the kernel confirms progress quickly after the keepalive timeout. Longer
    /// intervals reduce packets, shorter intervals detect stalled peers faster. Ignored if
    /// `tcp_keepalive` is `null`.
    #[serde(default = "default_tcp_keepalive_interval", with = "humantime_serde")]
    pub tcp_keepalive_interval: Option<Duration>,

    /// Number of TCP keepalive probes sent before a connection is declared dead.
    /// The default (5) balances resilience to transient loss with timely reclamation of resources.
    /// Smaller values clean up faster during outages, larger values favor noisy or lossy networks.
    #[serde(default = "default_tcp_keepalive_retries")]
    pub tcp_keepalive_retries: Option<u32>,

    /// Per-connection concurrency limit enforced by the transport layer.
    /// By default it mirrors the effective `max_concurrent_requests`, so transport- and
    /// application-level backpressure remain aligned. Lower values gate connection bursts earlier,
    /// while higher values only help if you also raise `max_concurrent_requests`. Set to `0` to
    /// revert to the derived default.
    #[serde(default)]
    pub transport_concurrency_limit: Option<usize>,

    /// Whether the gRPC server should shed load immediately once concurrency limits are hit.
    /// Leaving this `true` (default) results in fast `resource_exhausted` responses and protects
    /// the single-threaded runtime from unbounded queues. Turning it off allows requests to queue
    /// but increases memory usage and tail latency under sustained overload.
    #[serde(default = "default_load_shed")]
    pub load_shed: bool,

    /// Initial HTTP/2 stream window size, in bytes.
    /// Accepts plain integers or suffixed strings such as `8MiB`. The default 8MiB window reduces
    /// flow-control stalls for large OTLP batches; trimming it lowers per-stream memory but may
    /// throttle throughput, while increasing it benefits high-bandwidth deployments at the cost of
    /// larger buffers.
    #[serde(
        default = "default_initial_stream_window_size",
        deserialize_with = "byte_units::deserialize"
    )]
    pub initial_stream_window_size: Option<u32>,

    /// Initial HTTP/2 connection window size, in bytes.
    /// Accepts plain integers or suffixed strings such as `32MiB`. Defaults to 24MiB, giving room
    /// for several simultaneous large streams; adjust using the same trade-offs as the stream
    /// window but applied per connection.
    #[serde(
        default = "default_initial_connection_window_size",
        deserialize_with = "byte_units::deserialize"
    )]
    pub initial_connection_window_size: Option<u32>,

    /// Whether to rely on HTTP/2 adaptive window sizing instead of the manual values above.
    /// Disabled by default so the receiver uses predictable static windows. Enabling this lets tonic
    /// adjust flow-control windows dynamically, which can improve throughput on high-bandwidth links
    /// but makes memory usage and latency more workload dependent (and largely ignores the window
    /// sizes configured above).
    #[serde(default = "default_http2_adaptive_window")]
    pub http2_adaptive_window: bool,

    /// Maximum HTTP/2 frame size, in bytes.
    /// Accepts plain integers or suffixed strings such as `16KiB`. The 16KiB default matches the
    /// current tuning: large enough to keep framing overhead low for sizeable batches yet still
    /// bounded; larger values further decrease framing costs at the expense of bigger per-frame
    /// buffers, while smaller values force additional fragmentation and CPU work on jumbo exports.
    #[serde(
        default = "default_max_frame_size",
        deserialize_with = "byte_units::deserialize"
    )]
    pub max_frame_size: Option<u32>,

    /// Maximum size for inbound gRPC messages, in bytes.
    /// Accepts plain integers or suffixed strings such as `4MiB`. Defaults to tonic's 4MiB limit.
    #[serde(
        default = "default_max_decoding_message_size",
        deserialize_with = "byte_units::deserialize"
    )]
    pub max_decoding_message_size: Option<u32>,

    /// Interval between HTTP/2 keepalive pings.
    /// The default 30s ping keeps intermediaries aware of idle-but-healthy connections. Shorten it
    /// to detect broken links faster, lengthen it to reduce ping traffic, or set to `null` to
    /// disable HTTP/2 keepalives.
    #[serde(default = "default_http2_keepalive_interval", with = "humantime_serde")]
    pub http2_keepalive_interval: Option<Duration>,

    /// Timeout waiting for an HTTP/2 keepalive acknowledgement.
    /// Defaults to 10s, balancing rapid detection of stalled peers with tolerance for transient
    /// network jitter. Decrease it for quicker failover or increase it for chatty-but-latent paths.
    #[serde(default = "default_http2_keepalive_timeout", with = "humantime_serde")]
    pub http2_keepalive_timeout: Option<Duration>,

    /// Upper bound on concurrently active HTTP/2 streams per connection.
    /// By default this tracks the effective `max_concurrent_requests`, keeping logical and transport
    /// concurrency aligned. Lower values improve fairness between chatty clients. Higher values
    /// matter only if you also raise `max_concurrent_requests`. Set to `0` to inherit the derived
    /// default.
    #[serde(default)]
    pub max_concurrent_streams: Option<u32>,

    /// Whether to wait for the result (default: false)
    ///
    /// When enabled, the receiver will not send a response until the
    /// immediate downstream component has acknowledged receipt of the
    /// data.  This does not guarantee that data has been fully
    /// processed or successfully exported to the final destination,
    /// since components are able acknowledge early.
    ///
    /// Note when wait_for_result=false, it is impossible to
    /// see a failure, errors are effectively suppressed.
    #[serde(default = "default_wait_for_result")]
    pub wait_for_result: bool,

    /// Timeout for RPC requests. If not specified, no timeout is applied.
    /// Format: humantime format (e.g., "30s", "5m", "1h", "500ms")
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,
}

impl GrpcServerSettings {
    /// Returns the compression methods accepted for requests.
    #[must_use]
    pub fn request_compression_methods(&self) -> Vec<CompressionMethod> {
        match &self.request_compression {
            Some(methods) => methods.clone(),
            None => compression::DEFAULT_COMPRESSION_METHODS.to_vec(),
        }
    }

    /// Returns the compression methods configured for responses.
    #[must_use]
    pub fn response_compression_methods(&self) -> Vec<CompressionMethod> {
        match &self.response_compression {
            Some(methods) => methods.clone(),
            None => Vec::new(),
        }
    }

    /// Returns the first configured compression method for responses, if any.
    #[must_use]
    pub fn preferred_response_compression(&self) -> Option<CompressionMethod> {
        self.response_compression
            .as_ref()
            .and_then(|methods| methods.first().copied())
    }

    /// Builds the Tonic TCP Incoming.
    #[must_use]
    pub fn build_tcp_incoming(&self, tcp_listener: TcpListener) -> TcpIncoming {
        TcpIncoming::from(tcp_listener)
            .with_nodelay(Some(self.tcp_nodelay))
            .with_keepalive(self.tcp_keepalive)
            .with_keepalive_interval(self.tcp_keepalive_interval)
            .with_keepalive_retries(self.tcp_keepalive_retries)
    }

    /// Returns the compression encodings to use for both requests and responses.
    #[must_use]
    pub fn compression_encodings(
        &self,
    ) -> (EnabledCompressionEncodings, EnabledCompressionEncodings) {
        let mut request_compression = EnabledCompressionEncodings::default();
        for method in self.request_compression_methods() {
            request_compression.enable(method.map_to_compression_encoding());
        }

        let mut response_compression = EnabledCompressionEncodings::default();
        for method in self.response_compression_methods() {
            response_compression.enable(method.map_to_compression_encoding());
        }
        (request_compression, response_compression)
    }

    /// Builds the gRPC server settings from this configuration.
    #[must_use]
    pub fn build_settings(&self) -> Settings {
        let (request_compression_encodings, response_compression_encodings) =
            self.compression_encodings();

        Settings {
            max_concurrent_requests: self.max_concurrent_requests,
            wait_for_result: self.wait_for_result,
            max_decoding_message_size: self.max_decoding_message_size.map(|value| value as usize),
            request_compression_encodings,
            response_compression_encodings,
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

const fn default_load_shed() -> bool {
    true
}

const fn default_initial_stream_window_size() -> Option<u32> {
    Some(8 * 1024 * 1024)
}

const fn default_initial_connection_window_size() -> Option<u32> {
    Some(24 * 1024 * 1024)
}

const fn default_max_frame_size() -> Option<u32> {
    Some(16 * 1024)
}

const fn default_max_decoding_message_size() -> Option<u32> {
    Some(4 * 1024 * 1024)
}

const fn default_http2_keepalive_interval() -> Option<Duration> {
    Some(Duration::from_secs(30))
}

const fn default_http2_keepalive_timeout() -> Option<Duration> {
    Some(Duration::from_secs(10))
}

const fn default_http2_adaptive_window() -> bool {
    false
}

const fn default_wait_for_result() -> bool {
    // See https://github.com/open-telemetry/otel-arrow/issues/1311
    // This matches the OTel Collector default for wait_for_result, presently.
    false
}

impl Default for GrpcServerSettings {
    fn default() -> Self {
        Self {
            listening_addr: ([0, 0, 0, 0], 0).into(),
            request_compression: None,
            response_compression: None,
            max_concurrent_requests: default_max_concurrent_requests(),
            tcp_nodelay: default_tcp_nodelay(),
            tcp_keepalive: default_tcp_keepalive(),
            tcp_keepalive_interval: default_tcp_keepalive_interval(),
            tcp_keepalive_retries: default_tcp_keepalive_retries(),
            transport_concurrency_limit: None,
            load_shed: default_load_shed(),
            initial_stream_window_size: default_initial_stream_window_size(),
            initial_connection_window_size: default_initial_connection_window_size(),
            http2_adaptive_window: default_http2_adaptive_window(),
            max_frame_size: default_max_frame_size(),
            max_decoding_message_size: default_max_decoding_message_size(),
            http2_keepalive_interval: default_http2_keepalive_interval(),
            http2_keepalive_timeout: default_http2_keepalive_timeout(),
            max_concurrent_streams: None,
            wait_for_result: default_wait_for_result(),
            timeout: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::{CompressionMethod, DEFAULT_COMPRESSION_METHODS};
    use tonic::codec::CompressionEncoding;

    #[test]
    fn defaults_match_expected_compression() {
        let settings = GrpcServerSettings::default();

        assert_eq!(
            settings.request_compression_methods(),
            DEFAULT_COMPRESSION_METHODS.to_vec()
        );
        assert!(settings.response_compression_methods().is_empty());
        assert_eq!(settings.preferred_response_compression(), None);
    }

    #[test]
    fn response_compression_prefers_first_entry() {
        let settings = GrpcServerSettings {
            response_compression: Some(vec![CompressionMethod::Gzip, CompressionMethod::Zstd]),
            ..Default::default()
        };

        assert_eq!(
            settings.preferred_response_compression(),
            Some(CompressionMethod::Gzip)
        );
        assert_eq!(
            settings.response_compression_methods(),
            vec![CompressionMethod::Gzip, CompressionMethod::Zstd]
        );
    }

    #[test]
    fn build_settings_carries_core_limits_and_compression() {
        let settings = GrpcServerSettings {
            max_concurrent_requests: 42,
            wait_for_result: true,
            max_decoding_message_size: Some(8 * 1024 * 1024),
            request_compression: Some(vec![CompressionMethod::Deflate]),
            response_compression: Some(vec![CompressionMethod::Deflate]),
            ..Default::default()
        };

        let built = settings.build_settings();
        assert_eq!(built.max_concurrent_requests, 42);
        assert!(built.wait_for_result);
        assert_eq!(built.max_decoding_message_size, Some(8 * 1024 * 1024));

        let (req, resp) = settings.compression_encodings();
        assert!(req.is_enabled(CompressionEncoding::Deflate));
        assert!(resp.is_enabled(CompressionEncoding::Deflate));
    }
}
