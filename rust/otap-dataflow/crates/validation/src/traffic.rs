// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Traffic configuration helpers for validation scenarios. `Generator` describes
//! the synthetic signals to emit, and `Capture` describes how they are received
//! and validated.

use crate::ValidationInstructions;
use crate::error::ValidationError;
use crate::template::render_jinja;
use minijinja::context;
use otap_df_core_nodes::receivers::traffic_generator::config::DataSource;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::path::PathBuf;
const DEFAULT_MAX_SIGNAL_COUNT: usize = 2000;
const DEFAULT_MAX_BATCH_SIZE: usize = 100;
const DEFAULT_SIGNALS_PER_SECOND: usize = 100;
const DEFAULT_WEIGHT_ZERO: u32 = 0;
const DEFAULT_LOG_WEIGHT: u32 = 100;
const DEFAULT_IDLE_TIMEOUT_SECS: u8 = 3;

/// Describes a connection between a Generator/Capture and a test container.
///
/// The `node_template` is a Jinja2 template string for the custom receiver or
/// exporter node config. The framework renders it with `{{ port }}` set to the
/// allocated host port that is mapped to the container's `internal_port`.
///
/// Whether this describes an exporter or receiver is determined by which struct
/// holds the connection: [`Generator::to_container`] (exporter) or
/// [`Capture::from_container`] (receiver).
///
/// # Example
///
///
/// ContainerConnection::new("kafka")
///     .internal_port(9092)
///     .node_template(r#"
/// type: "urn:otel:exporter:kafka"
/// config:
///   broker: "127.0.0.1:{{ port }}"
///   topic: "otlp-logs"
/// "#)
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerConnection {
    /// Label matching a container added via
    /// [`Scenario::add_container`](crate::scenario::Scenario::add_container).
    pub(crate) container_label: String,
    /// The container's internal port (before host mapping).
    /// `None` until set via [`internal_port`](Self::internal_port).
    pub(crate) internal_port: Option<u16>,
    /// Jinja2 template for the custom node config. Rendered with
    /// `{{ port }}` = the allocated host port mapped to `internal_port`.
    pub(crate) node_template: String,
    /// Allocated host port (set by the framework during config wiring).
    /// `None` until the connection is wired.
    pub(crate) allocated_port: Option<u16>,
}

impl ContainerConnection {
    /// Create a new container connection referencing a container by its label.
    #[must_use]
    pub fn new(container_label: impl Into<String>) -> Self {
        Self {
            container_label: container_label.into(),
            internal_port: None,
            node_template: String::new(),
            allocated_port: None,
        }
    }

    /// Set the internal port on the container to connect to. This is the port
    /// the container listens on internally (before testcontainers host mapping).
    #[must_use]
    pub fn internal_port(mut self, port: u16) -> Self {
        self.internal_port = Some(port);
        self
    }

    /// Provide a Jinja2 template for the custom node configuration.
    /// The template is rendered with `{{ port }}` available in the context,
    /// set to the host port mapped to the container's internal port.
    #[must_use]
    pub fn node_template(mut self, template: impl Into<String>) -> Self {
        self.node_template = template.into();
        self
    }

    /// Render the node template with the allocated port in the Jinja2 context.
    /// Returns the rendered YAML string for the custom node configuration.
    pub(crate) fn render(&self) -> Result<String, ValidationError> {
        let port = self.allocated_port.ok_or_else(|| {
            ValidationError::Config(format!(
                "container connection to '{}' has no allocated port",
                self.container_label
            ))
        })?;
        render_jinja(&self.node_template, context! { port => port })
    }
}

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
/// Construct via [`TlsConfig::tls_only`] or [`TlsConfig::mtls`].
///
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

    /// Optional transport headers to attach to each generated pdata message.
    ///
    /// Keys are header names. Values are optional fixed strings; when left
    /// empty (`None`), a random value is generated once at startup by the
    /// fake data generator.
    pub(crate) transport_headers: HashMap<String, Option<String>>,

    /// Optional connection to a test container. When set, the generator's
    /// exporter node is rendered from the connection's Jinja2 template instead
    /// of the built-in OTLP/OTAP exporter, and SUV pipeline wiring is skipped.
    pub(crate) container_connection: Option<ContainerConnection>,
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
    /// Seconds to wait with no incoming messages before declaring the data
    /// stream settled and performing the final validation check.
    pub(crate) idle_timeout: u8,

    /// Transport header keys the capture pipeline's receiver should extract
    /// from inbound gRPC metadata. Each key generates a `match_names` rule
    /// in the pipeline's `header_capture` policy.
    pub(crate) capture_header_keys: Vec<String>,

    /// Optional connection to a test container. When set, the capture's
    /// receiver node is rendered from the connection's Jinja2 template instead
    /// of the built-in OTLP/OTAP receiver, and SUV pipeline wiring is skipped.
    pub(crate) container_connection: Option<ContainerConnection>,
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

    /// Configure transport headers to inject into generated traffic.
    ///
    /// Each key is a header name; the value is an optional fixed string.
    /// When the value is `None`, the fake data generator assigns a random
    /// value at startup.
    #[must_use]
    pub fn with_transport_headers(
        mut self,
        headers: impl IntoIterator<Item = (impl Into<String>, Option<impl Into<String>>)>,
    ) -> Self {
        self.transport_headers = headers
            .into_iter()
            .map(|(k, v)| (k.into(), v.map(Into::into)))
            .collect();
        self
    }

    /// Connect this generator to a test container using a custom exporter.
    ///
    /// The generator's `suv_exporter` node will be rendered from the
    /// connection's Jinja2 `node_template` instead of the built-in OTLP/OTAP
    /// exporter. The framework allocates a host port, maps it to the
    /// container's internal port, and makes it available as `{{ port }}` in
    /// the template.
    ///
    /// When a container connection is set, SUV pipeline receiver wiring is
    /// skipped for this generator (the container acts as the intermediary).
    #[must_use]
    pub fn to_container(mut self, connection: ContainerConnection) -> Self {
        self.container_connection = Some(connection);
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
            suv_port: 0,
            control_ports: vec![],
            max_signal_count: DEFAULT_MAX_SIGNAL_COUNT,
            max_batch_size: DEFAULT_MAX_BATCH_SIZE,
            signals_per_second: DEFAULT_SIGNALS_PER_SECOND,
            metric_weight: DEFAULT_WEIGHT_ZERO,
            trace_weight: DEFAULT_WEIGHT_ZERO,
            log_weight: DEFAULT_LOG_WEIGHT,
            data_source: DataSource::Static,
            tls: None,
            transport_headers: HashMap::new(),
            container_connection: None,
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

    /// Set how long (in seconds) the validation exporter should wait with no
    /// incoming messages before declaring the data stream settled.
    #[must_use]
    pub fn idle_timeout(mut self, timeout_secs: u8) -> Self {
        self.idle_timeout = timeout_secs;
        self
    }
    /// Configure transport header keys to capture from inbound signals.
    ///
    /// Each key becomes a `match_names` rule in the capture pipeline's
    /// `header_capture` policy, enabling the receiver to extract those
    /// headers from gRPC metadata so they can be validated by transport
    /// header validation instructions.
    #[must_use]
    pub fn with_capture_header_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.capture_header_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// Connect this capture to a test container using a custom receiver.
    ///
    /// The capture's `suv_receiver` node will be rendered from the
    /// connection's Jinja2 `node_template` instead of the built-in OTLP/OTAP
    /// receiver. The framework allocates a host port, maps it to the
    /// container's internal port, and makes it available as `{{ port }}` in
    /// the template.
    ///
    /// When a container connection is set, SUV pipeline exporter wiring is
    /// skipped for this capture (the container acts as the intermediary).
    #[must_use]
    pub fn from_container(mut self, connection: ContainerConnection) -> Self {
        self.container_connection = Some(connection);
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
            suv_port: 0,
            control_ports: vec![],
            control_streams: vec![],
            validate: vec![],
            idle_timeout: DEFAULT_IDLE_TIMEOUT_SECS,
            capture_header_keys: Vec::new(),
            container_connection: None,
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
        assert_eq!(g.suv_port, 0);
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
        assert!(g.transport_headers.is_empty());
    }

    #[test]
    fn capture_defaults_match_expected() {
        let c = Capture::default();
        assert_eq!(c.suv_receiver_type, MessageType::Otlp);
        assert_eq!(c.suv_port, 0);
        assert_eq!(c.control_ports, Vec::<u16>::new());
        assert_eq!(c.control_streams, Vec::<String>::new());
        assert_eq!(c.validate, vec![]);
        assert_eq!(c.core_start, 1);
        assert_eq!(c.core_end, 1);
        assert!(c.capture_header_keys.is_empty());
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

    #[test]
    fn capture_with_header_keys() {
        let c = Capture::default().with_capture_header_keys(["x-tenant-id", "x-request-id"]);
        assert_eq!(
            c.capture_header_keys,
            vec!["x-tenant-id".to_string(), "x-request-id".to_string()]
        );
    }

    #[test]
    fn generator_with_transport_headers() {
        let g = Generator::logs().with_transport_headers([
            ("x-tenant-id", Some("acme")),
            ("x-request-id", None::<&str>),
        ]);
        assert_eq!(g.transport_headers.len(), 2);
        assert_eq!(
            g.transport_headers.get("x-tenant-id"),
            Some(&Some("acme".to_string()))
        );
        assert_eq!(
            g.transport_headers.get("x-request-id"),
            Some(&None::<String>)
        );
    }
}
