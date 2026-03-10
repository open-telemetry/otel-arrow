// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared configuration for HTTP-based clients.

use reqwest::ClientBuilder;
use serde::Deserialize;
use std::io;
use std::time::Duration;
use tower::limit::ConcurrencyLimitLayer;

#[cfg(feature = "experimental-tls")]
use {
    crate::tls_utils::read_file_with_limit_async, otap_df_config::tls::TlsClientConfig,
    otap_df_telemetry::otel_error, reqwest::Certificate, reqwest::Identity,
};

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
    /// Requires the `experimental-tls` feature to be enabled.
    #[cfg(feature = "experimental-tls")]
    #[serde(default)]
    pub tls: Option<TlsClientConfig>,
}

impl HttpClientSettings {
    /// Returns a non-zero concurrency limit.
    #[must_use]
    pub fn effective_concurrency_limit(&self) -> usize {
        self.concurrency_limit.max(1)
    }

    /// Returns a configured client-builder
    pub async fn client_builder(&self) -> Result<ClientBuilder, HttpClientError> {
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

        #[cfg(feature = "experimental-tls")]
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
            #[cfg(feature = "experimental-tls")]
            tls: None,
        }
    }
}
