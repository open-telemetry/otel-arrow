// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline wiring helpers for validation scenarios. Allows loading YAML
//! pipelines and rewriting receiver/exporter endpoints at runtime so tests
//! can bind to ephemeral ports.

#![cfg_attr(not(test), allow(dead_code))]

use crate::error::ValidationError;
use serde_yaml::Value;
use std::fs;

/// Pipeline configuration wrapper that supports rewiring logical endpoints.
pub struct Pipeline {
    pub(crate) suv_yaml: Value,
    pub(crate) input_wire: Option<EndpointWire>,
    pub(crate) output_wire: Option<EndpointWire>,
}

impl Pipeline {
    /// Load a pipeline from a YAML file path.
    pub fn from_file(path: &str) -> Result<Self, ValidationError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ValidationError::Io(format!("failed to read pipeline yaml: {e}")))?;
        Ok(Self::from_yaml(&content))
    }

    /// Load a pipeline from a YAML string slice.
    #[must_use]
    pub fn from_yaml(yaml: &str) -> Self {
        let suv_yaml: Value = serde_yaml::from_str(yaml).expect("invalid pipeline yaml");
        Self {
            suv_yaml,
            input_wire: None,
            output_wire: None,
        }
    }

    /// Wire a node's OTLP gRPC receiver to a logical endpoint label.
    #[must_use]
    pub fn wire_otlp_grpc_receiver(mut self, node_name: impl Into<String>) -> Self {
        self.input_wire = Some(EndpointWire {
            node: node_name.into(),
            kind: EndpointKind::OtlpGrpcReceiver,
        });
        self
    }

    /// Wire a node's OTLP gRPC exporter to a logical endpoint label.
    #[must_use]
    pub fn wire_otlp_grpc_exporter(mut self, node_name: impl Into<String>) -> Self {
        self.output_wire = Some(EndpointWire {
            node: node_name.into(),
            kind: EndpointKind::OtlpGrpcExporter,
        });
        self
    }

    /// Wire a node's OTAP gRPC receiver to a logical endpoint label.
    #[must_use]
    pub fn wire_otap_grpc_receiver(mut self, node_name: impl Into<String>) -> Self {
        self.input_wire = Some(EndpointWire {
            node: node_name.into(),
            kind: EndpointKind::OtapGrpcReceiver,
        });
        self
    }

    /// Wire a node's OTAP gRPC exporter to a logical endpoint label.
    #[must_use]
    pub fn wire_otap_grpc_exporter(mut self, node_name: impl Into<String>) -> Self {
        self.output_wire = Some(EndpointWire {
            node: node_name.into(),
            kind: EndpointKind::OtapGrpcExporter,
        });
        self
    }

    /// Serialize the current pipeline configuration into a YAML string.
    pub(crate) fn to_yaml_string(&self) -> Result<String, ValidationError> {
        serde_yaml::to_string(&self.suv_yaml)
            .map_err(|e| ValidationError::Config(format!("failed to serialize pipeline yaml: {e}")))
    }

    pub(crate) fn update_pipeline(
        &mut self,
        input_addr: &str,
        output_endpoint: &str,
    ) -> Result<(), ValidationError> {
        let wire = self
            .input_wire
            .clone()
            .ok_or_else(|| ValidationError::Config("no input wire configured".into()))?;
        match wire.kind {
            EndpointKind::OtlpGrpcReceiver => self.set_otlp_receiver_addr(input_addr)?,
            EndpointKind::OtapGrpcReceiver => self.set_otap_receiver_addr(input_addr)?,
            _ => {
                return Err(ValidationError::Config(
                    "input wire is not a receiver".into(),
                ));
            }
        }

        let out_wire = self
            .output_wire
            .clone()
            .ok_or_else(|| ValidationError::Config("no output wire configured".into()))?;
        match out_wire.kind {
            EndpointKind::OtlpGrpcExporter | EndpointKind::OtapGrpcExporter => {
                self.set_exporter_endpoint(output_endpoint)
            }
            _ => Err(ValidationError::Config(
                "output wire is not an exporter".into(),
            )),
        }
    }

    fn set_otlp_receiver_addr(&mut self, addr: &str) -> Result<(), ValidationError> {
        let node = self
            .input_wire
            .as_ref()
            .filter(|w| matches!(w.kind, EndpointKind::OtlpGrpcReceiver))
            .map(|w| w.node.clone())
            .ok_or_else(|| ValidationError::Config("input wire is not an OTLP receiver".into()))?;
        let nodes = self
            .suv_yaml
            .get_mut("nodes")
            .and_then(Value::as_mapping_mut)
            .ok_or_else(|| ValidationError::Config("pipeline missing nodes map".into()))?;
        let node_cfg = nodes
            .get_mut(&node)
            .and_then(Value::as_mapping_mut)
            .ok_or_else(|| ValidationError::Config(format!("missing node {node}")))?;
        let config = node_cfg
            .entry(Value::from("config"))
            .or_insert_with(|| Value::Mapping(Default::default()));
        let config_map = config.as_mapping_mut().ok_or_else(|| {
            ValidationError::Config(format!("config section for node {node} is not a mapping"))
        })?;
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
        let _ = grpc_map.insert(Value::from("listening_addr"), Value::from(addr.to_string()));
        Ok(())
    }

    fn set_otap_receiver_addr(&mut self, addr: &str) -> Result<(), ValidationError> {
        let node = self
            .input_wire
            .as_ref()
            .filter(|w| matches!(w.kind, EndpointKind::OtapGrpcReceiver))
            .map(|w| w.node.clone())
            .ok_or_else(|| ValidationError::Config("input wire is not an OTAP receiver".into()))?;
        let nodes = self
            .suv_yaml
            .get_mut("nodes")
            .and_then(Value::as_mapping_mut)
            .ok_or_else(|| ValidationError::Config("pipeline missing nodes map".into()))?;
        let node_cfg = nodes
            .get_mut(&node)
            .and_then(Value::as_mapping_mut)
            .ok_or_else(|| ValidationError::Config(format!("missing node {node}")))?;
        let config = node_cfg
            .entry(Value::from("config"))
            .or_insert_with(|| Value::Mapping(Default::default()));
        let config_map = config.as_mapping_mut().ok_or_else(|| {
            ValidationError::Config(format!("config section for node {node} is not a mapping"))
        })?;
        let _ = config_map.insert(Value::from("listening_addr"), Value::from(addr.to_string()));
        Ok(())
    }

    fn set_exporter_endpoint(&mut self, endpoint: &str) -> Result<(), ValidationError> {
        let node = self
            .output_wire
            .as_ref()
            .map(|w| w.node.clone())
            .ok_or_else(|| ValidationError::Config("output wire missing".into()))?;
        let nodes = self
            .suv_yaml
            .get_mut("nodes")
            .and_then(Value::as_mapping_mut)
            .ok_or_else(|| ValidationError::Config("pipeline missing nodes map".into()))?;
        let node_cfg = nodes
            .get_mut(&node)
            .and_then(Value::as_mapping_mut)
            .ok_or_else(|| ValidationError::Config(format!("missing node {node}")))?;
        let config = node_cfg
            .entry(Value::from("config"))
            .or_insert_with(|| Value::Mapping(Default::default()));
        let config_map = config.as_mapping_mut().ok_or_else(|| {
            ValidationError::Config(format!("config section for node {node} is not a mapping"))
        })?;
        let _ = config_map.insert(
            Value::from("grpc_endpoint"),
            Value::from(endpoint.to_string()),
        );
        Ok(())
    }
}

/// Internal wiring descriptor connecting a pipeline node to an endpoint rewrite.
#[derive(Clone)]
pub struct EndpointWire {
    /// Node id in the pipeline YAML.
    pub node: String,
    /// Kind of endpoint to rewrite.
    pub kind: EndpointKind,
}

/// Types of endpoints that can be rewired in validation pipelines.
#[derive(Clone, Copy)]
pub enum EndpointKind {
    /// OTLP gRPC receiver listening address.
    OtlpGrpcReceiver,
    /// OTLP gRPC exporter destination.
    OtlpGrpcExporter,
    /// OTAP gRPC receiver listening address.
    OtapGrpcReceiver,
    /// OTAP gRPC exporter destination.
    OtapGrpcExporter,
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
        let mut pipeline = Pipeline::from_yaml(sample_yaml())
            .wire_otlp_grpc_receiver("receiver")
            .wire_otlp_grpc_exporter("exporter");
        pipeline
            .update_pipeline("127.0.0.1:5555", "http://127.0.0.1:7777")
            .expect("update should succeed");
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
        let mut pipeline = Pipeline::from_yaml(sample_yaml())
            .wire_otap_grpc_receiver("otap_recv")
            .wire_otap_grpc_exporter("otap_exp");
        pipeline
            .update_pipeline("127.0.0.1:6000", "http://127.0.0.1:7000")
            .expect("update should succeed");
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
        let mut pipeline = Pipeline::from_yaml(sample_yaml()).wire_otlp_grpc_receiver("receiver");
        let err = pipeline
            .update_pipeline("127.0.0.1:5555", "http://127.0.0.1:7777")
            .unwrap_err();
        assert!(matches!(err, ValidationError::Config(_)));
    }
}
