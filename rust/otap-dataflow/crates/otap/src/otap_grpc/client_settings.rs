// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared configuration for gRPC-based clients.

use crate::compression::CompressionMethod;

#[cfg(feature = "experimental-tls")]
use crate::tls_utils;
use otap_df_config::byte_units;
#[cfg(feature = "experimental-tls")]
use otap_df_config::tls::TlsClientConfig;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;
use tonic::codec::CompressionEncoding;
use tonic::transport::Endpoint;

/// Checks if a URI string starts with "https://" (case-insensitive, per RFC 3986).
fn is_https_endpoint(uri: &str) -> bool {
    uri.trim_start()
        .get(..8)
        .is_some_and(|s| s.eq_ignore_ascii_case("https://"))
}

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

    /// Client-side TLS/mTLS configuration.
    /// Requires the `experimental-tls` feature to be enabled.
    #[cfg(feature = "experimental-tls")]
    #[serde(default)]
    pub tls: Option<TlsClientConfig>,

    /// Internal Tower buffer size for the gRPC client.
    #[serde(default)]
    pub buffer_size: Option<usize>,
}

/// Error returned when building a gRPC [`Endpoint`] (including TLS/mTLS setup).
#[derive(Debug, Error)]
pub enum GrpcEndpointError {
    /// Error returned by tonic while parsing/configuring the transport endpoint.
    #[error("grpc endpoint build error: {0}")]
    Tonic(#[from] tonic::transport::Error),

    /// IO error while reading certificates/keys for TLS.
    #[error("tls configuration error: {0}")]
    Io(#[from] std::io::Error),

    /// TLS support is not compiled in.
    #[error("TLS support is disabled; enable the `experimental-tls` feature")]
    TlsFeatureDisabled,
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

    /// Builds the configured [`Endpoint`], applying TLS/mTLS settings when needed.
    pub async fn build_endpoint_with_tls(&self) -> Result<Endpoint, GrpcEndpointError> {
        let endpoint = self.build_endpoint()?;

        #[cfg(feature = "experimental-tls")]
        let wants_tls = is_https_endpoint(&self.grpc_endpoint) || self.tls.is_some();

        #[cfg(not(feature = "experimental-tls"))]
        let wants_tls = is_https_endpoint(&self.grpc_endpoint);

        if !wants_tls {
            return Ok(endpoint);
        }

        #[cfg(feature = "experimental-tls")]
        {
            let tls =
                tls_utils::load_client_tls_config(self.tls.as_ref(), &self.grpc_endpoint).await?;

            let Some(tls_config) = tls else {
                return Err(GrpcEndpointError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "A TLS configuration block was provided for an http:// endpoint, which is not supported. \
                     Please remove the tls configuration block or use an https:// endpoint if TLS is required.",
                )));
            };

            Ok(endpoint.tls_config(tls_config)?)
        }

        #[cfg(not(feature = "experimental-tls"))]
        {
            Err(GrpcEndpointError::TlsFeatureDisabled)
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
            #[cfg(feature = "experimental-tls")]
            tls: None,
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
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[cfg(feature = "experimental-tls")]
    use otap_df_config::tls::{TlsClientConfig, TlsConfig};

    #[cfg(feature = "experimental-tls")]
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

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_allows_plain_http_when_tls_unset() {
        let settings: GrpcClientSettings =
            serde_json::from_str(r#"{ "grpc_endpoint": "http://localhost:4317" }"#).unwrap();
        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        let _ = endpoint;
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_accepts_https_without_explicit_tls_block() {
        let settings: GrpcClientSettings = serde_json::from_str(
            r#"{ "grpc_endpoint": "https://localhost:4317", "tcp_nodelay": true }"#,
        )
        .unwrap();

        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        let _ = endpoint;
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_rejects_http_when_tls_is_configured() {
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                config: TlsConfig::default(),
                ca_file: None,
                ca_pem: Some(
                    "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n".to_string(),
                ),
                include_system_ca_certs_pool: Some(false),
                server_name_override: Some("localhost".to_string()),
            }),
            ..GrpcClientSettings::default()
        };

        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("non-https") || msg.contains("https://"));
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_rejects_partial_mtls_cert_without_key() {
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

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_rejects_partial_mtls_key_without_cert() {
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

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_errors_when_ca_file_missing() {
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
        assert!(
            err.to_string().to_lowercase().contains("no such")
                || err.to_string().to_lowercase().contains("not found")
        );
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_enforces_tls_file_size_limit() {
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

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_errors_when_no_trust_anchors() {
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

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_enforces_cert_file_size_limit() {
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

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_fails_with_empty_ca_pem() {
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://localhost:4317".to_string(),
            tls: Some(TlsClientConfig {
                ca_pem: Some("".to_string()), // Empty PEM
                include_system_ca_certs_pool: Some(false),
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        // Empty ca_pem with system certs disabled means no trust anchors
        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        assert!(
            err.to_string().contains("trust anchor") || err.to_string().contains("no trust"),
            "Expected trust anchor error for empty ca_pem, got: {}",
            err
        );
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_accepts_server_name_override() {
        // server_name_override should be accepted and not cause an error
        // (actual SNI behavior would require integration test with a real server)
        let settings = GrpcClientSettings {
            grpc_endpoint: "https://127.0.0.1:4317".to_string(),
            tls: Some(TlsClientConfig {
                server_name_override: Some("custom.hostname.example.com".to_string()),
                // Use system CAs (default) so we have trust anchors
                ..TlsClientConfig::default()
            }),
            ..GrpcClientSettings::default()
        };

        // Should successfully build the endpoint (SNI override is just configuration)
        let endpoint = settings.build_endpoint_with_tls().await.unwrap();
        let _ = endpoint;
    }

    #[cfg(feature = "experimental-tls")]
    #[tokio::test]
    async fn build_endpoint_with_tls_errors_on_empty_tls_block_with_http() {
        // Regression test: empty tls: {} block with http:// should return error, not panic
        let settings = GrpcClientSettings {
            grpc_endpoint: "http://localhost:4317".to_string(),
            tls: Some(TlsClientConfig::default()), // Empty TLS config
            ..GrpcClientSettings::default()
        };

        let err = settings.build_endpoint_with_tls().await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("A TLS configuration block was provided for an http:// endpoint"),
            "Expected error about invalid http+tls usage, got: {}",
            msg
        );
    }

    #[cfg(not(feature = "experimental-tls"))]
    #[test]
    fn build_endpoint_with_tls_rejects_https_when_feature_disabled() {
        let settings: GrpcClientSettings =
            serde_json::from_str(r#"{ "grpc_endpoint": "https://localhost:4317" }"#).unwrap();

        let err = futures::executor::block_on(settings.build_endpoint_with_tls()).unwrap_err();
        assert!(matches!(err, GrpcEndpointError::TlsFeatureDisabled));
    }

    #[cfg(not(feature = "experimental-tls"))]
    #[test]
    fn build_endpoint_with_tls_allows_http_when_feature_disabled() {
        let settings: GrpcClientSettings =
            serde_json::from_str(r#"{ "grpc_endpoint": "http://localhost:4317" }"#).unwrap();

        let endpoint = futures::executor::block_on(settings.build_endpoint_with_tls()).unwrap();
        let _ = endpoint;
    }
}
