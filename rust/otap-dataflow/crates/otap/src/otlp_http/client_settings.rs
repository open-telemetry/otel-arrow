// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared configuration for HTTP-based clients.

use reqwest::ClientBuilder;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io;
use std::time::Duration;
use tower::limit::ConcurrencyLimitLayer;
use {
    crate::tls_utils::read_file_with_limit_async, otap_df_config::tls::TlsClientConfig,
    otap_df_telemetry::otel_error, reqwest::Certificate, reqwest::Identity,
};

use crate::compression::{CompressionMethod, deserialize_compression_method};
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

    /// Client-side TLS/mTLS configuration.
    #[serde(default)]
    pub tls: Option<TlsClientConfig>,

    /// Compression method to apply to outbound request bodies. When set, the
    /// body is encoded with the chosen algorithm and a `Content-Encoding`
    /// header is added to the request. Accepts a single value (`"gzip"`,
    /// `"zstd"`, `"deflate"`) or the explicit string `"none"`. Defaults to
    /// no compression.
    #[serde(
        default,
        deserialize_with = "deserialize_compression_method",
        alias = "compression_method"
    )]
    pub compression: Option<CompressionMethod>,

    /// Custom User-Agent header for outbound HTTP requests. When set, this
    /// value is sent as the User-Agent header. When not set, no User-Agent
    /// header is sent (reqwest does not add one by default).
    #[serde(default)]
    pub user_agent: Option<String>,

    /// Static headers added to every outbound OTLP/HTTP request (e.g. an
    /// `Authorization` or backend routing header).
    ///
    /// Protocol headers (`Content-Type` / `Content-Encoding` /
    /// `Content-Length` / `Host`) and response-negotiation headers
    /// (`Accept` / `Accept-Encoding`) cannot be set here and are rejected by
    /// [`HttpClientSettings::validate`]; they are managed by the exporter.
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl HttpClientSettings {
    /// Validates the settings at config load time.
    ///
    /// Checks that `user_agent`, when set, is non-empty and contains only
    /// characters valid in an HTTP header value (visible ASCII, 32-127).
    ///
    /// Checks that every entry in `headers` has a valid HTTP header name and a
    /// value representable as an HTTP header value, that no protocol-reserved
    /// header (`Content-Type` / `Content-Encoding` / `Content-Length` / `Host`)
    /// or response-negotiation header (`Accept` / `Accept-Encoding`) is
    /// overridden, and that no header name is a case-insensitive duplicate of
    /// another (HTTP header names are case-insensitive).
    pub fn validate(&self) -> Result<(), HttpClientError> {
        if let Some(ua) = &self.user_agent {
            if ua.trim().is_empty() {
                return Err(HttpClientError::InvalidConfig(
                    "user_agent must be non-empty when set".to_string(),
                ));
            }
            if HeaderValue::from_str(ua).is_err() {
                return Err(HttpClientError::InvalidConfig(
                    "user_agent contains characters that cannot be represented as an HTTP header \
                     value (must be visible ASCII)"
                        .to_string(),
                ));
            }
        }

        let mut seen_names = HashSet::new();
        for (name, value) in &self.headers {
            let header_name = http::HeaderName::from_bytes(name.as_bytes()).map_err(|_| {
                HttpClientError::InvalidConfig(format!(
                    "header name \"{name}\" is not a valid HTTP header name"
                ))
            })?;
            if HeaderValue::from_str(value).is_err() {
                return Err(HttpClientError::InvalidConfig(format!(
                    "header \"{name}\" has a value that cannot be represented as an HTTP header \
                     value (must be visible ASCII)"
                )));
            }
            // Reject headers the exporter or HTTP client manages itself: the
            // protocol headers it sets per request, plus the response-negotiation
            // headers (`accept` / `accept-encoding`) whose effective value is
            // dictated by what the client can actually parse and decompress and so
            // is not something a user can truthfully declare here.
            if matches!(
                header_name.as_str(),
                "content-type"
                    | "content-encoding"
                    | "content-length"
                    | "host"
                    | "accept"
                    | "accept-encoding"
            ) {
                return Err(HttpClientError::InvalidConfig(format!(
                    "header \"{name}\" is reserved and cannot be set via `headers`; it is managed \
                     by the exporter"
                )));
            }
            // HTTP header names are case-insensitive, so two keys differing only in
            // case (e.g. `X-Foo` and `x-foo`) would collide on the wire. Reject such
            // duplicates rather than sending an ambiguous request. `header_name` is
            // already normalized to lowercase, so it is the canonical key here.
            if !seen_names.insert(header_name.as_str().to_string()) {
                return Err(HttpClientError::InvalidConfig(format!(
                    "header \"{name}\" is specified more than once; HTTP header names are \
                     case-insensitive, so keys that differ only in case are duplicates"
                )));
            }
        }

        Ok(())
    }

    /// Returns the configured request-body compression method, if any.
    #[must_use]
    pub fn compression(&self) -> Option<CompressionMethod> {
        self.compression
    }

    /// Returns a non-zero concurrency limit.
    #[must_use]
    pub fn effective_concurrency_limit(&self) -> usize {
        self.concurrency_limit.max(1)
    }

    /// Returns a configured client-builder
    pub async fn client_builder(&self) -> Result<ClientBuilder, HttpClientError> {
        let mut client_builder = ClientBuilder::new()
            .use_rustls_tls()
            .connect_timeout(self.connect_timeout)
            .tcp_nodelay(self.tcp_nodelay)
            .connector_layer(ConcurrencyLimitLayer::new(
                self.effective_concurrency_limit(),
            ));

        if let Some(ua) = &self.user_agent {
            client_builder = client_builder.user_agent(ua.as_str());
        }

        if let Some(tcp_keepalive) = self.tcp_keepalive {
            client_builder = client_builder.tcp_keepalive(tcp_keepalive);
        }

        if let Some(tcp_keepalive_interval) = self.tcp_keepalive_interval {
            client_builder = client_builder.tcp_keepalive_interval(tcp_keepalive_interval)
        }

        if let Some(timeout) = self.timeout {
            client_builder = client_builder.timeout(timeout)
        }

        if let Some(tls) = &self.tls {
            let mut certs = vec![];

            if let Some(ca_pem) = &tls.ca_pem {
                let cert = Certificate::from_pem(ca_pem.as_bytes())?;
                certs.push(cert);
            }

            if let Some(ca_file) = &tls.ca_file {
                let ca_pem = read_file_with_limit_async(ca_file).await.map_err(|e| {
                    otel_error!(
                        "tls.ca_file.read_error",
                        ca_file = ?ca_file,
                        error = ?e,
                        message = "Failed to read CA file"
                    );
                    e
                })?;
                let cert = Certificate::from_pem(&ca_pem)?;
                certs.push(cert);
            }

            if tls.include_system_ca_certs_pool.unwrap_or(true) {
                client_builder = client_builder.tls_certs_merge(certs);
            } else {
                client_builder = client_builder.tls_certs_only(certs);
            }

            if let Some(true) = &tls.insecure_skip_verify {
                client_builder = client_builder.danger_accept_invalid_certs(true);
            }

            // mTLS client certificate configuration
            let client_cert_configured = tls.config.cert_file.is_some()
                || tls
                    .config
                    .cert_pem
                    .as_ref()
                    .is_some_and(|pem| !pem.trim().is_empty());
            let client_key_configured = tls.config.key_file.is_some()
                || tls
                    .config
                    .key_pem
                    .as_ref()
                    .is_some_and(|pem| !pem.trim().is_empty());

            if client_cert_configured || client_key_configured {
                if !(client_cert_configured && client_key_configured) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "TLS configuration error: both cert and key must be provided for mTLS. \
                             Provide both cert_file/cert_pem and key_file/key_pem, or neither.",
                    )
                    .into());
                }

                // Read cert and key
                let cert_pem = if let Some(cert_file) = &tls.config.cert_file {
                    read_file_with_limit_async(cert_file).await.map_err(|e| {
                        otel_error!(
                            "tls.cert_file.read_error",
                            cert_file = ?cert_file,
                            error = ?e, message = "Failed to read client cert file"
                        );
                        e
                    })?
                } else if let Some(cert_pem) = &tls.config.cert_pem {
                    cert_pem.as_bytes().to_vec()
                } else {
                    unreachable!()
                };

                let key_pem = if let Some(key_file) = &tls.config.key_file {
                    read_file_with_limit_async(key_file).await.map_err(|e| {
                        otel_error!(
                            "tls.key_file.read_error",
                            key_file = ?key_file,
                            error = ?e,
                            message = "Failed to read client key file"
                        );
                        e
                    })?
                } else if let Some(key_pem) = &tls.config.key_pem {
                    key_pem.as_bytes().to_vec()
                } else {
                    unreachable!()
                };

                // Combine cert and key into PEM format for Identity
                let mut identity_pem = cert_pem;
                identity_pem.extend_from_slice(&key_pem);

                let identity = Identity::from_pem(&identity_pem).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Failed to create identity from cert and key: {}", e),
                    )
                })?;

                client_builder = client_builder.identity(identity);
            }
        }

        Ok(client_builder)
    }
}

/// Errors that occur configuring Http ClientBuilder
#[derive(thiserror::Error, Debug)]
pub enum HttpClientError {
    /// Error occurred configuring reqwest client
    #[error("http client build error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// IO Error occurred reading tls cert from file
    #[error("http client build io error: {0}")]
    Io(#[from] io::Error),

    /// Invalid configuration value detected at validation time.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
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
            tls: None,
            compression: None,
            user_agent: None,
            headers: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use wiremock::matchers;
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    /// Respond implementation that asserts User-Agent is absent.
    struct AssertNoUserAgent {
        saw_user_agent: Arc<AtomicBool>,
    }

    impl Respond for AssertNoUserAgent {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            if request.headers.get("user-agent").is_some() {
                self.saw_user_agent.store(true, Ordering::SeqCst);
            }
            ResponseTemplate::new(200)
        }
    }

    #[test]
    fn validate_rejects_empty_user_agent() {
        let settings = HttpClientSettings {
            user_agent: Some(String::new()),
            ..HttpClientSettings::default()
        };

        assert!(matches!(
            settings.validate(),
            Err(HttpClientError::InvalidConfig(_))
        ));
    }

    #[test]
    fn validate_rejects_whitespace_only_user_agent() {
        let settings = HttpClientSettings {
            user_agent: Some("   ".to_string()),
            ..HttpClientSettings::default()
        };

        assert!(matches!(
            settings.validate(),
            Err(HttpClientError::InvalidConfig(_))
        ));
    }

    #[test]
    fn validate_rejects_non_ascii_user_agent() {
        let settings = HttpClientSettings {
            user_agent: Some("bad\nvalue".to_string()),
            ..HttpClientSettings::default()
        };

        assert!(matches!(
            settings.validate(),
            Err(HttpClientError::InvalidConfig(_))
        ));
    }

    #[test]
    fn validate_accepts_valid_user_agent() {
        let settings = HttpClientSettings {
            user_agent: Some("my-app/1.0".to_string()),
            ..HttpClientSettings::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_rejects_invalid_header_name() {
        let mut headers = HashMap::new();
        let _ = headers.insert("bad header".to_string(), "value".to_string());
        let settings = HttpClientSettings {
            headers,
            ..HttpClientSettings::default()
        };

        assert!(matches!(
            settings.validate(),
            Err(HttpClientError::InvalidConfig(_))
        ));
    }

    #[test]
    fn validate_rejects_invalid_header_value() {
        let mut headers = HashMap::new();
        let _ = headers.insert("x-test".to_string(), "bad\nvalue".to_string());
        let settings = HttpClientSettings {
            headers,
            ..HttpClientSettings::default()
        };

        assert!(matches!(
            settings.validate(),
            Err(HttpClientError::InvalidConfig(_))
        ));
    }

    #[test]
    fn validate_rejects_reserved_header() {
        for reserved in [
            "content-type",
            "Content-Type",
            "content-length",
            "host",
            "accept",
            "accept-encoding",
            "Accept-Encoding",
        ] {
            let mut headers = HashMap::new();
            let _ = headers.insert(reserved.to_string(), "x".to_string());
            let settings = HttpClientSettings {
                headers,
                ..HttpClientSettings::default()
            };

            assert!(
                matches!(settings.validate(), Err(HttpClientError::InvalidConfig(_))),
                "reserved header {reserved} should be rejected"
            );
        }
    }

    #[test]
    fn validate_accepts_valid_headers() {
        let mut headers = HashMap::new();
        let _ = headers.insert("authorization".to_string(), "Basic abc123".to_string());
        let _ = headers.insert("x-scope-orgid".to_string(), "tenant-1".to_string());
        let settings = HttpClientSettings {
            headers,
            ..HttpClientSettings::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_rejects_case_insensitive_duplicate_headers() {
        let mut headers = HashMap::new();
        let _ = headers.insert("X-Tenant".to_string(), "a".to_string());
        let _ = headers.insert("x-tenant".to_string(), "b".to_string());
        let settings = HttpClientSettings {
            headers,
            ..HttpClientSettings::default()
        };

        assert!(matches!(
            settings.validate(),
            Err(HttpClientError::InvalidConfig(_))
        ));
    }

    #[test]
    fn deserialize_accepts_headers_and_keeps_deny_unknown_fields() {
        let settings: HttpClientSettings = serde_json::from_str(
            r#"{ "headers": { "authorization": "Basic abc123", "stream-name": "default" } }"#,
        )
        .unwrap();
        assert_eq!(settings.headers.len(), 2);
        assert_eq!(
            settings.headers.get("authorization").map(String::as_str),
            Some("Basic abc123")
        );
        assert_eq!(
            settings.headers.get("stream-name").map(String::as_str),
            Some("default")
        );

        // deny_unknown_fields is preserved now that `headers` is a known field.
        assert!(serde_json::from_str::<HttpClientSettings>(r#"{ "nope": 1 }"#).is_err());
    }

    #[tokio::test]
    async fn test_no_user_agent_on_wire_when_unset() {
        crate::crypto::ensure_crypto_provider();

        let settings = HttpClientSettings::default();
        let mock_server = MockServer::start().await;

        let saw_ua = Arc::new(AtomicBool::new(false));
        Mock::given(matchers::method("POST"))
            .respond_with(AssertNoUserAgent {
                saw_user_agent: Arc::clone(&saw_ua),
            })
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = settings.client_builder().await.unwrap().build().unwrap();
        let _ = client
            .post(format!("{}/v1/logs", mock_server.uri()))
            .body("")
            .send()
            .await
            .unwrap();

        assert!(
            !saw_ua.load(Ordering::SeqCst),
            "Expected no User-Agent header when user_agent is unset, but one was sent"
        );
    }

    #[tokio::test]
    async fn test_user_agent_sent_on_wire() {
        crate::crypto::ensure_crypto_provider();

        // Verify default has no user_agent
        let defaults = HttpClientSettings::default();
        assert_eq!(defaults.user_agent, None);

        // Verify deserialization
        let settings: HttpClientSettings =
            serde_json::from_str(r#"{ "user_agent": "otap-custom-agent/1.0" }"#).unwrap();
        assert_eq!(
            settings.user_agent.as_deref(),
            Some("otap-custom-agent/1.0")
        );

        // Verify the header actually arrives on the wire
        let mock_server = MockServer::start().await;

        Mock::given(matchers::method("POST"))
            .and(matchers::header("User-Agent", "otap-custom-agent/1.0"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = settings.client_builder().await.unwrap().build().unwrap();
        let _ = client
            .post(format!("{}/v1/logs", mock_server.uri()))
            .body("")
            .send()
            .await
            .unwrap();

        // wiremock asserts the expectation on drop
    }
}
