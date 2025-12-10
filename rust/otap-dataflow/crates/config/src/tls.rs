// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Common TLS configuration for both client and server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct TlsConfig {
    /// Path to the TLS cert to use for TLS required connections.
    pub cert_file: Option<PathBuf>,

    /// In memory PEM encoded TLS cert to use for TLS required connections.
    pub cert_pem: Option<String>,

    /// Path to the TLS key to use for TLS required connections.
    pub key_file: Option<PathBuf>,

    /// In memory PEM encoded TLS key to use for TLS required connections.
    pub key_pem: Option<String>,
    // TODO: Implement these fields
    // /// MinVersion sets the minimum TLS version that is acceptable.
    // /// If not set, TLS 1.2 will be used.
    // pub min_version: Option<String>,

    // /// MaxVersion sets the maximum TLS version that is acceptable.
    // pub max_version: Option<String>,

    // /// CipherSuites is a list of TLS cipher suites that the TLS transport can use.
    // pub cipher_suites: Option<Vec<String>>,

    // /// Trusted platform module configuration
    // pub tpm_config: Option<TpmConfig>,
    /// Minimum interval between certificate reload checks.
    /// Certificates are only reloaded if file modification time has changed.
    /// Defaults to 5 minutes ("5m").
    /// Format: Standard duration string (e.g., "30s", "5m", "1h").
    #[serde(default = "default_reload_interval", with = "humantime_serde")]
    #[schemars(with = "Option<String>")]
    pub reload_interval: Option<Duration>,
}

fn default_reload_interval() -> Option<Duration> {
    Some(Duration::from_secs(300))
}

/// TLS configuration specific to client connections.
///
/// This configuration is used by components that initiate TLS connections, such as:
/// - **Exporters**: When sending telemetry to a backend.
/// - **Scrapers/clients**: When pulling data from a target (e.g., Prometheus scrape, OTLP/HTTP client).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct TlsClientConfig {
    // TODO: Implement client TLS configuration fields when needed.
}

/// TLS configuration specific to server connections (terminating TLS/mTLS).
///
/// This configuration is used by components that accept inbound TLS connections, such as:
/// - **Servers/receivers**: When accepting telemetry from agents or other collectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct TlsServerConfig {
    /// Common TLS configuration.
    #[serde(flatten)]
    pub config: TlsConfig,

    /// Path to the TLS cert to use by the server to verify a client certificate.
    pub client_ca_file: Option<PathBuf>,

    /// In memory PEM encoded cert to use by the server to verify a client certificate.
    pub client_ca_pem: Option<String>,

    /// Controls whether system CA certificates are loaded for client certificate verification.
    pub include_system_ca_certs_pool: Option<bool>,
    // /// Path to the Certificate Revocation List (CRL) to use by the server to verify a client certificate.
    // pub client_crl_file: Option<PathBuf>,
    /// Timeout for TLS handshake. Defaults to 10 seconds.
    /// Prevents slow/malicious clients from holding connection slots.
    #[serde(default = "default_handshake_timeout", with = "humantime_serde")]
    #[schemars(with = "Option<String>")]
    pub handshake_timeout: Option<Duration>,
}

fn default_handshake_timeout() -> Option<Duration> {
    Some(Duration::from_secs(10))
}
