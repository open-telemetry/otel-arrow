// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline wiring helpers for validation scenarios. Allows loading YAML
//! pipelines and rewriting receiver/exporter endpoints at runtime so tests
//! can bind to ephemeral ports.

use crate::error::ValidationError;
use crate::template::render_jinja;
use minijinja::context;
use serde_yaml::{Mapping, Value};
use std::fs;

/// Config key path for OTLP gRPC receiver listening address
/// (`nodes.<node>.config.protocols.grpc.listening_addr`).
const OTLP_RECEIVER_KEY: &str = "protocols.grpc.listening_addr";

/// Config key path for OTAP gRPC receiver listening address
/// (`nodes.<node>.config.listening_addr`).
const OTAP_RECEIVER_KEY: &str = "listening_addr";

/// Config key path for gRPC exporter endpoint, used by both OTLP and OTAP
/// (`nodes.<node>.config.grpc_endpoint`).
const GRPC_EXPORTER_KEY: &str = "grpc_endpoint";

/// Address prefix for receiver endpoints. The allocated port is appended.
const RECEIVER_ADDR_PREFIX: &str = "127.0.0.1:";

/// Address prefix for exporter endpoints. The allocated port is appended.
const EXPORTER_ADDR_PREFIX: &str = "http://127.0.0.1:";

/// Describes a connection between a node in the SUV pipeline and a test
/// container. During config wiring the framework allocates a host port
/// mapped to the container's `internal_port`, then rewrites the specified
/// config key in the pipeline YAML with the formatted address.
///
/// The `config_key` is a dot-separated path relative to
/// `nodes.<node_name>.config`. For example, `"broker"` targets
/// `nodes.<node>.config.broker`, while `"protocols.grpc.listening_addr"`
/// targets `nodes.<node>.config.protocols.grpc.listening_addr`.
///
/// The `address_template` is a Jinja2 template string with `{{ port }}`
/// available in the context. For example, `"127.0.0.1:{{ port }}"` or
/// `"http://127.0.0.1:{{ port }}"`.
///
/// # Example
///
///
/// Pipeline::from_yaml(yaml)?
///     .connect_container(
///         PipelineContainerConnection::new("kafka")
///             .internal_port(9092)
///             .node("kafka_sink")
///             .config_key("broker")
///             .address_template("127.0.0.1:{{ port }}")
///     )
///
pub struct PipelineContainerConnection {
    /// Label matching a container added via
    /// [`Scenario::add_container`](crate::scenario::Scenario::add_container).
    pub(crate) container_label: String,
    /// The container's internal port (before host mapping).
    /// `None` until set via [`internal_port`](Self::internal_port).
    pub(crate) internal_port: Option<u16>,
    /// The node name in the SUV pipeline YAML to rewrite.
    pub(crate) node_name: String,
    /// Dot-separated path to the config key relative to
    /// `nodes.<node_name>.config`.
    pub(crate) config_key_path: String,
    /// Jinja2 template for the address value. `{{ port }}` is set to the
    /// allocated host port.
    pub(crate) address_template: String,
}

impl PipelineContainerConnection {
    /// Create a new pipeline container connection referencing a container by
    /// its label.
    #[must_use]
    pub fn new(container_label: impl Into<String>) -> Self {
        Self {
            container_label: container_label.into(),
            internal_port: None,
            node_name: String::new(),
            config_key_path: String::new(),
            address_template: String::new(),
        }
    }

    /// Set the internal port on the container to connect to.
    #[must_use]
    pub fn internal_port(mut self, port: u16) -> Self {
        self.internal_port = Some(port);
        self
    }

    /// Set the node name in the SUV pipeline YAML whose config will be
    /// rewritten.
    #[must_use]
    pub fn node(mut self, name: impl Into<String>) -> Self {
        self.node_name = name.into();
        self
    }

    /// Set the dot-separated config key path relative to
    /// `nodes.<node>.config`.
    #[must_use]
    pub fn config_key(mut self, path: impl Into<String>) -> Self {
        self.config_key_path = path.into();
        self
    }

    /// Set the address template string. This is a Jinja2 template with
    /// `{{ port }}` available in the context, set to the allocated host port.
    #[must_use]
    pub fn address_template(mut self, template: impl Into<String>) -> Self {
        self.address_template = template.into();
        self
    }

    /// Render the address template with the given port.
    pub(crate) fn render_address(&self, port: u16) -> Result<String, ValidationError> {
        render_jinja(&self.address_template, context! { port => port })
    }
}

/// Pipeline configuration wrapper that supports rewiring logical endpoints.
pub struct Pipeline {
    pub(crate) suv_yaml: Value,
    pub(crate) core_start: u16,
    pub(crate) core_end: u16,
    /// Container connections declared on this pipeline, consumed during
    /// config wiring.
    pub(crate) container_connections: Vec<PipelineContainerConnection>,
}

impl Pipeline {
    /// Load a pipeline from a YAML file path.
    pub fn from_file(path: &str) -> Result<Self, ValidationError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ValidationError::Io(format!("failed to read pipeline yaml: {e}")))?;
        Self::from_yaml(&content)
    }

    /// Load a pipeline from a YAML file with `${VAR}` placeholder substitution.
    pub fn from_file_with_vars(path: &str, vars: &[(&str, &str)]) -> Result<Self, ValidationError> {
        let mut content = fs::read_to_string(path)
            .map_err(|e| ValidationError::Io(format!("failed to read pipeline yaml: {e}")))?;
        for (key, value) in vars {
            content = content.replace(&format!("${{{key}}}"), value);
        }
        if let Some(start) = content.find("${") {
            let end = content[start..]
                .find('}')
                .map_or(content.len(), |i| start + i + 1);
            let unresolved = &content[start..end];
            return Err(ValidationError::Config(format!(
                "unresolved placeholder {unresolved} in {path}"
            )));
        }
        Self::from_yaml(&content)
    }

    /// Load a pipeline from a YAML string slice.
    pub fn from_yaml(yaml: &str) -> Result<Self, ValidationError> {
        let suv_yaml: Value = serde_yaml::from_str(yaml)
            .map_err(|e| ValidationError::Config(format!("invalid pipeline yaml: {e}")))?;
        Ok(Self {
            suv_yaml,
            core_start: 0,
            core_end: 0,
            container_connections: Vec::new(),
        })
    }

    /// Set the core range for the SUV pipeline.
    #[must_use]
    pub fn core_range(mut self, start: u16, end: u16) -> Self {
        self.core_start = start;
        self.core_end = end;
        self
    }

    /// Serialize the current pipeline configuration into a YAML string.
    pub(crate) fn to_yaml_string(&self) -> Result<String, ValidationError> {
        serde_yaml::to_string(&self.suv_yaml)
            .map_err(|e| ValidationError::Config(format!("failed to serialize pipeline yaml: {e}")))
    }

    /// Declare a connection between a pipeline node and a test container.
    /// The connection is consumed during config wiring in
    /// [`Scenario::update_configs`](crate::scenario::Scenario).
    #[must_use]
    pub fn connect_container(mut self, conn: PipelineContainerConnection) -> Self {
        self.container_connections.push(conn);
        self
    }

    /// Rewrite a config value in the pipeline YAML at the given
    /// dot-separated key path under `nodes.<node>.config`.
    pub(crate) fn set_node_config_value(
        &mut self,
        node: &str,
        key_path: &str,
        value: &str,
    ) -> Result<(), ValidationError> {
        set_config_by_path(&mut self.suv_yaml, node, key_path, value)
    }

    /// Apply a single endpoint rewrite directly.
    pub(crate) fn apply_endpoint(
        &mut self,
        wire: EndpointKind,
        port: u16,
    ) -> Result<(), ValidationError> {
        wire.apply_to_value(&mut self.suv_yaml, port)
    }
}

/// Types of endpoints that can be rewired in validation pipelines.
#[derive(Clone)]
pub enum EndpointKind {
    /// OTLP gRPC receiver listening address.
    OtlpGrpcReceiver(String),
    /// OTLP gRPC exporter destination.
    OtlpGrpcExporter(String),
    /// OTAP gRPC receiver listening address.
    OtapGrpcReceiver(String),
    /// OTAP gRPC exporter destination.
    OtapGrpcExporter(String),
}

impl EndpointKind {
    /// Apply rewrite to the given YAML value in-place using the provided port.
    pub fn apply_to_value(&self, doc: &mut Value, port: u16) -> Result<(), ValidationError> {
        let (node, key_path, value) = match self {
            EndpointKind::OtlpGrpcReceiver(node) => (
                node.as_str(),
                OTLP_RECEIVER_KEY,
                format!("{RECEIVER_ADDR_PREFIX}{port}"),
            ),
            EndpointKind::OtapGrpcReceiver(node) => (
                node.as_str(),
                OTAP_RECEIVER_KEY,
                format!("{RECEIVER_ADDR_PREFIX}{port}"),
            ),
            EndpointKind::OtlpGrpcExporter(node) | EndpointKind::OtapGrpcExporter(node) => (
                node.as_str(),
                GRPC_EXPORTER_KEY,
                format!("{EXPORTER_ADDR_PREFIX}{port}"),
            ),
        };
        set_config_by_path(doc, node, key_path, &value)
    }

    /// Return the node name this endpoint targets.
    #[must_use]
    pub fn node_name(&self) -> &str {
        match self {
            EndpointKind::OtlpGrpcReceiver(node)
            | EndpointKind::OtlpGrpcExporter(node)
            | EndpointKind::OtapGrpcReceiver(node)
            | EndpointKind::OtapGrpcExporter(node) => node,
        }
    }
}

fn node_config_map<'a>(doc: &'a mut Value, node: &str) -> Result<&'a mut Mapping, ValidationError> {
    let nodes = doc
        .get_mut("nodes")
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| ValidationError::Config("pipeline missing nodes map".into()))?;
    let node_cfg = nodes
        .get_mut(node)
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| ValidationError::Config(format!("missing node {node}")))?;
    let config = node_cfg
        .entry(Value::from("config"))
        .or_insert_with(|| Value::Mapping(Default::default()));
    config.as_mapping_mut().ok_or_else(|| {
        ValidationError::Config(format!("config section for node {node} is not a mapping"))
    })
}

/// Walk a dot-separated config key path under `nodes.<node>.config` and set
/// the leaf key to the given value. Intermediate mappings are created when
/// missing.
fn set_config_by_path(
    doc: &mut Value,
    node: &str,
    key_path: &str,
    value: &str,
) -> Result<(), ValidationError> {
    if key_path.is_empty() {
        return Err(ValidationError::Config(format!(
            "empty config key path for node {node}"
        )));
    }
    let mut current: &mut Mapping = node_config_map(doc, node)?;
    let segments: Vec<&str> = key_path.split('.').collect();
    let (leaf, intermediate) = segments.split_last().expect("non-empty key_path");
    for &segment in intermediate {
        let entry = current
            .entry(Value::from(segment))
            .or_insert_with(|| Value::Mapping(Default::default()));
        current = entry.as_mapping_mut().ok_or_else(|| {
            ValidationError::Config(format!(
                "{segment} in config path for node {node} is not a mapping"
            ))
        })?;
    }
    let _ = current.insert(Value::from(*leaf), Value::from(value));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_yaml() -> &'static str {
        r#"
nodes:
  receiver:
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    config:
      grpc_endpoint: "http://default-export"
  otap_recv:
    config:
      listening_addr: "127.0.0.1:4420"
  otap_exp:
    config:
      grpc_endpoint: "http://default-otap-export"
"#
    }

    #[test]
    fn otlp_wiring_rewrites_addresses() {
        let mut pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        pipeline
            .apply_endpoint(EndpointKind::OtlpGrpcReceiver("receiver".into()), 5555)
            .expect("receiver rewrite succeeds");
        pipeline
            .apply_endpoint(EndpointKind::OtlpGrpcExporter("exporter".into()), 7777)
            .expect("exporter rewrite succeeds");
        let rendered = pipeline
            .to_yaml_string()
            .expect("serialization should succeed");

        let doc: Value = serde_yaml::from_str(&rendered).unwrap();
        let nodes = doc.get("nodes").and_then(Value::as_mapping).unwrap();
        let recv = nodes
            .get(Value::from("receiver"))
            .and_then(Value::as_mapping)
            .unwrap();
        let recv_cfg = recv
            .get(Value::from("config"))
            .and_then(Value::as_mapping)
            .unwrap();
        let protocols = recv_cfg
            .get(Value::from("protocols"))
            .and_then(Value::as_mapping)
            .unwrap();
        let grpc = protocols
            .get(Value::from("grpc"))
            .and_then(Value::as_mapping)
            .unwrap();
        assert_eq!(
            grpc.get(Value::from("listening_addr")),
            Some(&Value::from("127.0.0.1:5555"))
        );

        let exp = nodes
            .get(Value::from("exporter"))
            .and_then(Value::as_mapping)
            .unwrap();
        let exp_cfg = exp
            .get(Value::from("config"))
            .and_then(Value::as_mapping)
            .unwrap();
        assert_eq!(
            exp_cfg.get(Value::from("grpc_endpoint")),
            Some(&Value::from("http://127.0.0.1:7777"))
        );
    }

    #[test]
    fn otap_wiring_rewrites_addresses() {
        let mut pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        pipeline
            .apply_endpoint(EndpointKind::OtapGrpcReceiver("otap_recv".into()), 6000)
            .expect("receiver rewrite succeeds");
        pipeline
            .apply_endpoint(EndpointKind::OtapGrpcExporter("otap_exp".into()), 7000)
            .expect("exporter rewrite succeeds");
        let rendered = pipeline
            .to_yaml_string()
            .expect("serialization should succeed");

        let doc: Value = serde_yaml::from_str(&rendered).unwrap();
        let nodes = doc.get("nodes").and_then(Value::as_mapping).unwrap();
        let recv = nodes
            .get(Value::from("otap_recv"))
            .and_then(Value::as_mapping)
            .unwrap();
        let recv_cfg = recv
            .get(Value::from("config"))
            .and_then(Value::as_mapping)
            .unwrap();
        assert_eq!(
            recv_cfg.get(Value::from("listening_addr")),
            Some(&Value::from("127.0.0.1:6000"))
        );

        let exp = nodes
            .get(Value::from("otap_exp"))
            .and_then(Value::as_mapping)
            .unwrap();
        let exp_cfg = exp
            .get(Value::from("config"))
            .and_then(Value::as_mapping)
            .unwrap();
        assert_eq!(
            exp_cfg.get(Value::from("grpc_endpoint")),
            Some(&Value::from("http://127.0.0.1:7000"))
        );
    }

    #[test]
    fn missing_output_wire_errors() {
        let mut pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        let err = pipeline
            .apply_endpoint(EndpointKind::OtlpGrpcExporter("missing".into()), 1234)
            .unwrap_err();
        assert!(matches!(err, ValidationError::Config(_)));
    }

    #[test]
    fn from_file_invalid_path_errors() {
        let result = Pipeline::from_file("nonexistent/path.yaml");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, ValidationError::Io(_)));
        assert!(err.to_string().contains("failed to read pipeline yaml"));
    }

    #[test]
    fn core_range_sets_values() {
        let pipeline = Pipeline::from_yaml(sample_yaml()).unwrap().core_range(3, 7);
        assert_eq!(pipeline.core_start, 3);
        assert_eq!(pipeline.core_end, 7);
    }

    #[test]
    fn endpoint_kind_node_name() {
        assert_eq!(
            EndpointKind::OtlpGrpcReceiver("recv".into()).node_name(),
            "recv"
        );
        assert_eq!(
            EndpointKind::OtlpGrpcExporter("exp".into()).node_name(),
            "exp"
        );
        assert_eq!(
            EndpointKind::OtapGrpcReceiver("otap_recv".into()).node_name(),
            "otap_recv"
        );
        assert_eq!(
            EndpointKind::OtapGrpcExporter("otap_exp".into()).node_name(),
            "otap_exp"
        );
    }

    #[test]
    fn set_config_by_path_single_key() {
        let mut pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        pipeline
            .set_node_config_value("exporter", "grpc_endpoint", "http://127.0.0.1:9999")
            .expect("single key rewrite succeeds");

        let doc: Value = serde_yaml::from_str(&pipeline.to_yaml_string().unwrap()).unwrap();
        let config = doc["nodes"]["exporter"]["config"].as_mapping().unwrap();
        assert_eq!(
            config.get(Value::from("grpc_endpoint")),
            Some(&Value::from("http://127.0.0.1:9999"))
        );
    }

    #[test]
    fn set_config_by_path_nested_key() {
        let mut pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        pipeline
            .set_node_config_value(
                "receiver",
                "protocols.grpc.listening_addr",
                "127.0.0.1:5555",
            )
            .expect("nested key rewrite succeeds");

        let doc: Value = serde_yaml::from_str(&pipeline.to_yaml_string().unwrap()).unwrap();
        let addr = &doc["nodes"]["receiver"]["config"]["protocols"]["grpc"]["listening_addr"];
        assert_eq!(addr, &Value::from("127.0.0.1:5555"));
    }

    #[test]
    fn set_config_by_path_creates_intermediate_maps() {
        let mut pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        pipeline
            .set_node_config_value("exporter", "deep.nested.key", "some_value")
            .expect("creating intermediate maps succeeds");

        let doc: Value = serde_yaml::from_str(&pipeline.to_yaml_string().unwrap()).unwrap();
        let val = &doc["nodes"]["exporter"]["config"]["deep"]["nested"]["key"];
        assert_eq!(val, &Value::from("some_value"));
    }

    #[test]
    fn set_config_by_path_missing_node_errors() {
        let mut pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        let err = pipeline
            .set_node_config_value("nonexistent", "key", "value")
            .unwrap_err();
        assert!(matches!(err, ValidationError::Config(_)));
    }

    #[test]
    fn connect_container_stores_connection() {
        let pipeline = Pipeline::from_yaml(sample_yaml())
            .unwrap()
            .connect_container(
                PipelineContainerConnection::new("kafka")
                    .internal_port(9092)
                    .node("exporter")
                    .config_key("broker")
                    .address_template("127.0.0.1:{{ port }}"),
            );
        assert_eq!(pipeline.container_connections.len(), 1);
        assert_eq!(pipeline.container_connections[0].container_label, "kafka");
        assert_eq!(pipeline.container_connections[0].internal_port, Some(9092));
        assert_eq!(pipeline.container_connections[0].node_name, "exporter");
        assert_eq!(pipeline.container_connections[0].config_key_path, "broker");
        assert_eq!(
            pipeline.container_connections[0].address_template,
            "127.0.0.1:{{ port }}"
        );
    }
}
