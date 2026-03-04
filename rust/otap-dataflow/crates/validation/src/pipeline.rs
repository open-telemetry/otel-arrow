// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline wiring helpers for validation scenarios. Allows loading YAML
//! pipelines and rewriting receiver/exporter endpoints at runtime so tests
//! can bind to ephemeral ports.

#![cfg_attr(not(test), allow(dead_code))]

use crate::error::ValidationError;
use serde_yaml::{Mapping, Value};
use std::fs;

/// Pipeline configuration wrapper that supports rewiring logical endpoints.
pub struct Pipeline {
    pub(crate) suv_yaml: Value,
    pub(crate) core_start: u16,
    pub(crate) core_end: u16,
}

impl Pipeline {
    /// Load a pipeline from a YAML file path.
    pub fn from_file(path: &str) -> Result<Self, ValidationError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ValidationError::Io(format!("failed to read pipeline yaml: {e}")))?;
        Ok(Self::from_yaml(&content))
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
        Ok(Self::from_yaml(&content))
    }

    /// Load a pipeline from a YAML string slice.
    #[must_use]
    pub fn from_yaml(yaml: &str) -> Self {
        let suv_yaml: Value = serde_yaml::from_str(yaml).expect("invalid pipeline yaml");
        Self {
            suv_yaml,
            core_start: 0,
            core_end: 0,
        }
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
        match self {
            EndpointKind::OtlpGrpcReceiver(node) => set_otlp_receiver_addr(doc, node, port)?,
            EndpointKind::OtapGrpcReceiver(node) => set_otap_receiver_addr(doc, node, port)?,
            EndpointKind::OtlpGrpcExporter(node) | EndpointKind::OtapGrpcExporter(node) => {
                set_exporter_endpoint(doc, node, port)?
            }
        }
        Ok(())
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
        .get_mut(Value::from(node.to_string()))
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| ValidationError::Config(format!("missing node {node}")))?;
    let config = node_cfg
        .entry(Value::from("config"))
        .or_insert_with(|| Value::Mapping(Default::default()));
    config.as_mapping_mut().ok_or_else(|| {
        ValidationError::Config(format!("config section for node {node} is not a mapping"))
    })
}

fn set_otlp_receiver_addr(doc: &mut Value, node: &str, port: u16) -> Result<(), ValidationError> {
    let config_map = node_config_map(doc, node)?;
    let protocols = config_map
        .entry(Value::from("protocols"))
        .or_insert_with(|| Value::Mapping(Default::default()));
    let protocols_map = protocols.as_mapping_mut().ok_or_else(|| {
        ValidationError::Config(format!(
            "protocols section for node {node} is not a mapping"
        ))
    })?;
    let grpc = protocols_map
        .entry(Value::from("grpc"))
        .or_insert_with(|| Value::Mapping(Default::default()));
    let grpc_map = grpc.as_mapping_mut().ok_or_else(|| {
        ValidationError::Config(format!("grpc section for node {node} is not a mapping"))
    })?;
    let _ = grpc_map.insert(
        Value::from("listening_addr"),
        Value::from(format!("127.0.0.1:{port}")),
    );
    Ok(())
}

fn set_otap_receiver_addr(doc: &mut Value, node: &str, port: u16) -> Result<(), ValidationError> {
    let config_map = node_config_map(doc, node)?;
    let _ = config_map.insert(
        Value::from("listening_addr"),
        Value::from(format!("127.0.0.1:{port}")),
    );
    Ok(())
}

fn set_exporter_endpoint(doc: &mut Value, node: &str, port: u16) -> Result<(), ValidationError> {
    let config_map = node_config_map(doc, node)?;
    let _ = config_map.insert(
        Value::from("grpc_endpoint"),
        Value::from(format!("http://127.0.0.1:{port}")),
    );
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
          listening_addr: "0.0.0.0:4317"
  exporter:
    config:
      grpc_endpoint: "http://default-export"
  otap_recv:
    config:
      listening_addr: "0.0.0.0:4420"
  otap_exp:
    config:
      grpc_endpoint: "http://default-otap-export"
"#
    }

    #[test]
    fn otlp_wiring_rewrites_addresses() {
        let mut pipeline = Pipeline::from_yaml(sample_yaml());
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
        let mut pipeline = Pipeline::from_yaml(sample_yaml());
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
        let mut pipeline = Pipeline::from_yaml(sample_yaml());
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
        let pipeline = Pipeline::from_yaml(sample_yaml()).core_range(3, 7);
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
}
