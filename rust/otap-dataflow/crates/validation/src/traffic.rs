// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Traffic configuration helpers for validation scenarios. `Generator` describes
//! the synthetic signals to emit, and `Capture` describes how they are received
//! and validated.

use crate::ValidationInstructions;
use crate::error::ValidationError;
use otap_df_core_nodes::receivers::fake_data_generator::config::DataSource;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::PathBuf;

const DEFAULT_SUV_PORT: u16 = 4318;
const DEFAULT_SUV_ENDPOINT_PORT: u16 = 4317;
const DEFAULT_MAX_SIGNAL_COUNT: usize = 2000;
const DEFAULT_MAX_BATCH_SIZE: usize = 100;
const DEFAULT_SIGNALS_PER_SECOND: usize = 100;
const DEFAULT_WEIGHT_ZERO: u32 = 0;
const DEFAULT_LOG_WEIGHT: u32 = 100;

/// Protocols supported by generators and receivers.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// OTLP type
    Otlp,
    /// OTAP type
    Otap,
}

/// TLS configuration for validation scenarios.
///
/// When provided, the traffic-generator exporter connects to the SUV receiver
/// over TLS (or mTLS) instead of plain-text gRPC.
///
/// Construct via [`TlsScenarioConfig::tls_only`] or [`TlsScenarioConfig::mtls`].
///
/// **Note:** Requires the `experimental-tls` feature to be enabled on `otap-df-otap`,
/// otherwise the rendered pipeline config will fail to deserialize.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
    pub(crate) suv_exporter_type: MessageType,
    /// Node name in the SUV pipeline to rewrite for exporter endpoint.
    pub(crate) suv_exporter_node: String,
    /// Core range start for this generator pipeline.
    pub(crate) core_start: u16,
    /// Core range end for this generator pipeline.
    pub(crate) core_end: u16,
    /// Port the system-under-validation exporter should target.
    pub(crate) suv_port: u16,
    /// Control ports the exporter should target (one per connected capture).
    pub(crate) control_ports: Vec<u16>,
    /// Maximum number of signals the load generator should emit.
    pub(crate) max_signal_count: usize,
    /// Maximum batch size emitted by the load generator.
    pub(crate) max_batch_size: usize,
    /// Signal emission rate (per second) for the load generator.
    pub(crate) signals_per_second: usize,
    /// Weight for metrics generation (0-100).
    pub(crate) metric_weight: u32,
    /// Weight for trace generation (0-100).
    pub(crate) trace_weight: u32,
    /// Weight for log generation (0-100).
    pub(crate) log_weight: u32,
    /// static vs semantic messages
    pub(crate) data_source: DataSource,

    pub(crate) tls: Option<TlsConfig>,
}

/// Configuration describing how validation receivers capture generated traffic.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Capture {
    /// Type to use for the system-under-validation receiver.
    pub(crate) suv_receiver_type: MessageType,
    /// Node name in the SUV pipeline to rewrite for receiver listen addr.
    pub(crate) suv_receiver_node: String,
    /// Core range start for this capture pipeline.
    pub(crate) core_start: u16,
    /// Core range end for this capture pipeline.
    pub(crate) core_end: u16,
    /// Listening port for the system-under-validation receiver.
    pub(crate) suv_port: u16,
    /// Listening ports for control receivers (one per connected generator).
    pub(crate) control_ports: Vec<u16>,
    /// Generator labels whose control streams this capture should receive.
    pub(crate) control_streams: Vec<String>,
    /// List of validations to make with the captured data
    pub(crate) validate: Vec<ValidationInstructions>,
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

    /// Set the maximum batch size for emitted signals.
    #[must_use]
    pub fn max_batch_size(mut self, batch_size: usize) -> Self {
        self.max_batch_size = batch_size;
        self
    }

    /// Emit over OTLP gRPC.
    #[must_use]
    pub fn otlp_grpc(mut self, exporter_node: impl Into<String>) -> Self {
        self.suv_exporter_type = MessageType::Otlp;
        self.suv_exporter_node = exporter_node.into();
        self
    }

    /// Emit over OTAP gRPC.
    #[must_use]
    pub fn otap_grpc(mut self, exporter_node: impl Into<String>) -> Self {
        self.suv_exporter_type = MessageType::Otap;
        self.suv_exporter_node = exporter_node.into();
        self
    }

    /// Set the core range for this generator pipeline.
    #[must_use]
    pub fn core_range(mut self, start: u16, end: u16) -> Self {
        self.core_start = start;
        self.core_end = end;
        self
    }

    /// set the traffic generator to use static as source
    #[must_use]
    pub fn static_signals(mut self) -> Self {
        self.data_source = DataSource::Static;
        self
    }

    /// set the traffic generator to use semantic convention as source
    #[must_use]
    pub fn semantic_signals(mut self) -> Self {
        self.data_source = DataSource::SemanticConventions;
        self
    }

    /// Enable TLS (or mTLS) between the traffic generator and the SUV receiver.
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
            suv_exporter_node: String::new(),
            core_start: 2,
            core_end: 2,
            suv_port: DEFAULT_SUV_ENDPOINT_PORT,
            control_ports: vec![],
            max_signal_count: DEFAULT_MAX_SIGNAL_COUNT,
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
            signals_per_second: DEFAULT_SIGNALS_PER_SECOND,
            metric_weight: DEFAULT_WEIGHT_ZERO,
            trace_weight: DEFAULT_WEIGHT_ZERO,
            log_weight: DEFAULT_LOG_WEIGHT,
            data_source: DataSource::Static,
            tls: None,
        }
    }
}

impl Capture {
    /// Capture OTLP gRPC traffic.
    #[must_use]
    pub fn otlp_grpc(mut self, receiver_node: impl Into<String>) -> Self {
        self.suv_receiver_type = MessageType::Otlp;
        self.suv_receiver_node = receiver_node.into();
        self
    }

    /// Capture OTAP gRPC traffic.
    #[must_use]
    pub fn otap_grpc(mut self, receiver_node: impl Into<String>) -> Self {
        self.suv_receiver_type = MessageType::Otap;
        self.suv_receiver_node = receiver_node.into();
        self
    }

    /// Set the validations to perform on captured data.
    #[must_use]
    pub fn validate(mut self, validations: Vec<ValidationInstructions>) -> Self {
        self.validate = validations;
        self
    }

    /// Set the generator labels whose control streams this capture should
    /// receive. Each label must correspond to a generator added to the
    /// [`Scenario`](crate::scenario::Scenario) so that the control path can
    /// be wired during configuration.
    #[must_use]
    pub fn control_streams(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.control_streams = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Set the core range for this capture pipeline.
    #[must_use]
    pub fn core_range(mut self, start: u16, end: u16) -> Self {
        self.core_start = start;
        self.core_end = end;
        self
    }

    /// Serialize the configured validations as JSON (for template contexts).
    #[must_use]
    pub fn validations_config(&self) -> String {
        if self.validate.is_empty() {
            return "".to_string();
        }
        serde_yaml::to_string(&self.validate).unwrap_or_else(|_| "".to_string())
    }
}

impl Default for Capture {
    fn default() -> Self {
        Self {
            suv_receiver_type: MessageType::Otlp,
            suv_receiver_node: String::new(),
            core_start: 1,
            core_end: 1,
            suv_port: DEFAULT_SUV_PORT,
            control_ports: vec![],
            control_streams: vec![],
            validate: vec![],
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
        assert_eq!(g.suv_port, DEFAULT_SUV_ENDPOINT_PORT);
        assert_eq!(g.control_ports, Vec::<u16>::new());
        assert_eq!(g.max_signal_count, 2000);
        assert_eq!(g.max_batch_size, 100);
        assert_eq!(g.signals_per_second, 100);
        assert_eq!(g.metric_weight, 0);
        assert_eq!(g.trace_weight, 0);
        assert_eq!(g.log_weight, 100);
        assert_eq!(g.data_source, DataSource::Static);
        assert_eq!(g.core_start, 2);
        assert_eq!(g.core_end, 2);
    }

    #[test]
    fn capture_defaults_match_expected() {
        let c = Capture::default();
        assert_eq!(c.suv_receiver_type, MessageType::Otlp);
        assert_eq!(c.suv_port, DEFAULT_SUV_PORT);
        assert_eq!(c.control_ports, Vec::<u16>::new());
        assert_eq!(c.control_streams, Vec::<String>::new());
        assert_eq!(c.validate, vec![]);
        assert_eq!(c.core_start, 1);
        assert_eq!(c.core_end, 1);
    }

    #[test]
    fn generator_fixed_count_and_protocols() {
        let g = Generator::default()
            .fixed_count(42)
            .otap_grpc("exporter")
            .otlp_grpc("exporter"); // last call wins
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
        let c = Capture::default().otap_grpc("receiver");
        assert_eq!(c.suv_receiver_type, MessageType::Otap);
    }
}
