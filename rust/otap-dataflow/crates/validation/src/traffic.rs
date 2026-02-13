// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module defines structs to describe the traffic being created and captured for validation

use crate::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_SUV_ADDR: &str = "127.0.0.1:4318";
const DEFAULT_SUV_ENDPOINT: &str = "http://127.0.0.1:4317";
const DEFAULT_CONTROL_ADDR: &str = "127.0.0.1:4316";
const DEFAULT_CONTROL_ENDPOINT: &str = "http://127.0.0.1:4316";
const DEFAULT_MAX_SIGNAL_COUNT: usize = 2000;
const DEFAULT_MAX_BATCH_SIZE: usize = 100;
const DEFAULT_SIGNALS_PER_SECOND: usize = 100;
const DEFAULT_WEIGHT_ZERO: u32 = 0;
const DEFAULT_LOG_WEIGHT: u32 = 100;

/// Helps distinguish between the message types
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// otlp type
    Otlp,
    /// otap type
    Otap,
}

/// TLS configuration for the traffic generator's connection to the SUV receiver.
///
/// When provided, the traffic-generator exporter connects over TLS (or mTLS)
/// instead of plain-text gRPC.
///
/// Construct via [`TlsConfig::tls_only`] or [`TlsConfig::mtls`].
///
/// **Note:** Requires the `experimental-tls` feature to be enabled on `otap-df-otap`,
/// otherwise the rendered pipeline config will fail to deserialize.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TlsConfig {
    pub(crate) ca_cert_path: PathBuf,
    pub(crate) client_cert_path: Option<PathBuf>,
    pub(crate) client_key_path: Option<PathBuf>,
    pub(crate) server_name: String,
}

impl TlsConfig {
    /// Create a TLS-only config (no client cert).
    #[must_use]
    pub fn tls_only(ca_cert_path: impl Into<PathBuf>) -> Self {
        Self {
            ca_cert_path: ca_cert_path.into(),
            client_cert_path: None,
            client_key_path: None,
            server_name: "localhost".to_string(),
        }
    }

    /// Create an mTLS config with client certificate and key.
    #[must_use]
    pub fn mtls(
        ca_cert_path: impl Into<PathBuf>,
        client_cert_path: impl Into<PathBuf>,
        client_key_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            ca_cert_path: ca_cert_path.into(),
            client_cert_path: Some(client_cert_path.into()),
            client_key_path: Some(client_key_path.into()),
            server_name: "localhost".to_string(),
        }
    }

    /// Override the server name used for TLS SNI and certificate verification.
    ///
    /// Defaults to `"localhost"`. Set this when the server certificate uses a
    /// different SAN/CN than `localhost`.
    #[must_use]
    pub fn with_server_name(mut self, name: impl Into<String>) -> Self {
        self.server_name = name.into();
        self
    }

    pub(crate) fn ca_cert_str(&self) -> Result<&str, ValidationError> {
        self.ca_cert_path
            .to_str()
            .ok_or_else(|| ValidationError::Config("ca_cert_path is not valid UTF-8".into()))
    }

    pub(crate) fn client_cert_str(&self) -> Result<&str, ValidationError> {
        match self.client_cert_path.as_deref() {
            None => Ok(""),
            Some(path) => path.to_str().ok_or_else(|| {
                ValidationError::Config("client_cert_path is not valid UTF-8".into())
            }),
        }
    }

    pub(crate) fn client_key_str(&self) -> Result<&str, ValidationError> {
        match self.client_key_path.as_deref() {
            None => Ok(""),
            Some(path) => path.to_str().ok_or_else(|| {
                ValidationError::Config("client_key_path is not valid UTF-8".into())
            }),
        }
    }

    pub(crate) fn is_mtls(&self) -> bool {
        self.client_cert_path.is_some() && self.client_key_path.is_some()
    }
}

/// Configuration describing how the traffic generator should emit signals.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Generator {
    /// Type to use for the system-under-validation exporter.
    pub suv_exporter_type: MessageType,
    /// Endpoint the system-under-validation exporter should target.
    pub suv_endpoint: String,
    /// Endpoint the control exporter should target.
    pub control_endpoint: String,
    /// Maximum number of signals the load generator should emit.
    pub max_signal_count: usize,
    /// Maximum batch size emitted by the load generator.
    pub max_batch_size: usize,
    /// Signal emission rate (per second) for the load generator.
    pub signals_per_second: usize,
    /// Weight for metrics generation (0-100).
    pub metric_weight: u32,
    /// Weight for trace generation (0-100).
    pub trace_weight: u32,
    /// Weight for log generation (0-100).
    pub log_weight: u32,
    /// TLS configuration for connecting to the SUV receiver.
    #[serde(skip)]
    pub tls: Option<TlsConfig>,
}

/// Configuration describing how validation receivers capture generated traffic.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Capture {
    /// Type to use for the system-under-validation receiver.
    pub suv_receiver_type: MessageType,
    /// Listening address for the system-under-validation receiver.
    pub suv_listening_addr: String,
    /// Listening address for the control receiver.
    pub control_listening_addr: String,
}

impl Generator {
    /// Emit only logs.
    #[must_use]
    pub fn logs() -> Self {
        Generator {
            log_weight: 100,
            metric_weight: 0,
            trace_weight: 0,
            ..Generator::default()
        }
    }

    /// Emit only metrics.
    #[must_use]
    pub fn metrics() -> Self {
        Generator {
            log_weight: 0,
            metric_weight: 100,
            trace_weight: 0,
            ..Generator::default()
        }
    }

    /// Emit only traces.
    #[must_use]
    pub fn traces() -> Self {
        Generator {
            log_weight: 0,
            metric_weight: 0,
            trace_weight: 100,
            ..Generator::default()
        }
    }

    /// Emit a fixed number of signals before completing.
    #[must_use]
    pub fn fixed_count(mut self, count: usize) -> Self {
        self.max_signal_count = count;
        self
    }

    /// Emit over OTLP gRPC.
    #[must_use]
    pub fn otlp_grpc(mut self) -> Self {
        self.suv_exporter_type = MessageType::Otlp;
        self
    }

    /// Emit over OTAP gRPC.
    #[must_use]
    pub fn otap_grpc(mut self) -> Self {
        self.suv_exporter_type = MessageType::Otap;
        self
    }

    /// Enable TLS (or mTLS) for the connection to the SUV receiver.
    #[must_use]
    pub fn with_tls(mut self, config: TlsConfig) -> Self {
        self.tls = Some(config);
        self
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            suv_exporter_type: MessageType::Otlp,
            suv_endpoint: DEFAULT_SUV_ENDPOINT.to_string(),
            control_endpoint: DEFAULT_CONTROL_ENDPOINT.to_string(),
            max_signal_count: DEFAULT_MAX_SIGNAL_COUNT,
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
            signals_per_second: DEFAULT_SIGNALS_PER_SECOND,
            metric_weight: DEFAULT_WEIGHT_ZERO,
            trace_weight: DEFAULT_WEIGHT_ZERO,
            log_weight: DEFAULT_LOG_WEIGHT,
            tls: None,
        }
    }
}

impl Capture {
    /// Capture OTLP gRPC traffic.
    #[must_use]
    pub fn otlp_grpc(mut self) -> Self {
        self.suv_receiver_type = MessageType::Otlp;
        self
    }

    /// Capture OTAP gRPC traffic.
    #[must_use]
    pub fn otap_grpc(mut self) -> Self {
        self.suv_receiver_type = MessageType::Otap;
        self
    }
}

impl Default for Capture {
    fn default() -> Self {
        Self {
            suv_receiver_type: MessageType::Otlp,
            suv_listening_addr: DEFAULT_SUV_ADDR.to_string(),
            control_listening_addr: DEFAULT_CONTROL_ADDR.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_defaults_match_expected() {
        let g = Generator::default();
        assert_eq!(g.suv_exporter_type, MessageType::Otlp);
        assert_eq!(g.suv_endpoint, "http://127.0.0.1:4317");
        assert_eq!(g.control_endpoint, "http://127.0.0.1:4316");
        assert_eq!(g.max_signal_count, 2000);
        assert_eq!(g.max_batch_size, 100);
        assert_eq!(g.signals_per_second, 100);
        assert_eq!(g.metric_weight, 0);
        assert_eq!(g.trace_weight, 0);
        assert_eq!(g.log_weight, 100);
    }

    #[test]
    fn capture_defaults_match_expected() {
        let c = Capture::default();
        assert_eq!(c.suv_receiver_type, MessageType::Otlp);
        assert_eq!(c.suv_listening_addr, "127.0.0.1:4318");
        assert_eq!(c.control_listening_addr, "127.0.0.1:4316");
    }

    #[test]
    fn generator_fixed_count_and_protocols() {
        let g = Generator::default().fixed_count(42).otap_grpc().otlp_grpc(); // last call wins
        assert_eq!(g.max_signal_count, 42);
        assert_eq!(g.suv_exporter_type, MessageType::Otlp);
    }

    #[test]
    fn generator_signal_weights_helpers() {
        let logs = Generator::logs();
        assert_eq!(
            (logs.log_weight, logs.metric_weight, logs.trace_weight),
            (100, 0, 0)
        );

        let metrics = Generator::metrics();
        assert_eq!(
            (
                metrics.log_weight,
                metrics.metric_weight,
                metrics.trace_weight
            ),
            (0, 100, 0)
        );

        let traces = Generator::traces();
        assert_eq!(
            (traces.log_weight, traces.metric_weight, traces.trace_weight),
            (0, 0, 100)
        );
    }

    #[test]
    fn capture_otap_sets_type() {
        let c = Capture::default().otap_grpc();
        assert_eq!(c.suv_receiver_type, MessageType::Otap);
    }
}
