// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node configuration specification.
//!
//! A node is a fundamental unit in our data processing pipeline, representing either a receiver
//! (source), processor, exporter (sink), or connector (linking pipelines).
//!
//! A node can expose multiple named output ports.

use crate::pipeline::telemetry::{AttributeValue, TelemetryAttribute};
use crate::{Description, NodeUrn, PortName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;

/// User configuration for a node in the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NodeUserConfig {
    /// The node type URN identifying the plugin (factory) to use for this node.
    ///
    /// Expected format:
    /// - `urn:<namespace>:<id>:<kind>`
    /// - `<id>:<kind>` (shortcut form for the `otel` namespace)
    ///
    /// The node kind is inferred from the `<kind>` segment.
    pub r#type: NodeUrn,

    /// An optional description of this node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,

    /// Declared output ports exposed by this node.
    ///
    /// This is primarily used with top-level `connections` wiring.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<PortName>,

    /// Optional default output port name to use when a node emits pdata without specifying a port.
    /// If omitted and multiple output ports are configured, the engine will treat the default as
    /// ambiguous and require explicit port selection at runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_output: Option<PortName>,

    /// Node specific attributes to be added to internal telemetry.
    ///
    /// Supports both bare values and extended form with optional brief descriptions:
    /// ```yaml
    /// telemetry_attributes:
    ///   region: "us-west"                          # bare value
    ///   team:
    ///     value: "platform"                        # extended form
    ///     brief: "Owning team name"                # optional description
    /// ```
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        deserialize_with = "deserialize_telemetry_attributes"
    )]
    pub telemetry_attributes: HashMap<String, TelemetryAttribute>,

    /// Node-specific configuration.
    ///
    /// This configuration is interpreted by the node itself and is not interpreted and validated by
    /// the pipeline engine.
    ///
    /// Note: A pre-validation step using a JSON schema or protobuf could be added to the
    /// management plane to ensure that the configuration is valid.
    #[serde(default)]
    // The serde_json::Value serializes to an invalid schema as far as the kubernetes api is concerned.
    // The preserve-unknown-fields extension allows this to be correctly interpreted as "Any JSON type"
    #[schemars(extend("x-kubernetes-preserve-unknown-fields" = true))]
    pub config: Value,
}

/// Node kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// A source of signals
    #[default]
    Receiver,
    /// A processor of signals
    Processor,
    /// A sink of signals
    Exporter,

    // ToDo(LQ) : Add more node kinds as needed.
    // A connector between two pipelines
    // Connector,
    /// A merged chain of consecutive processors (experimental).
    ProcessorChain,
}

impl From<NodeKind> for Cow<'static, str> {
    fn from(kind: NodeKind) -> Self {
        match kind {
            NodeKind::Receiver => "receiver".into(),
            NodeKind::Processor => "processor".into(),
            NodeKind::Exporter => "exporter".into(),
            NodeKind::ProcessorChain => "processor_chain".into(),
        }
    }
}

impl NodeUserConfig {
    /// Creates a new Receiver `NodeUserConfig` with the node type URN.
    pub fn new_receiver_config<U: AsRef<str>>(node_type: U) -> Self {
        Self {
            r#type: crate::node_urn::normalize_plugin_urn_for_kind(
                node_type.as_ref(),
                NodeKind::Receiver,
            )
            .expect("invalid receiver node type"),
            description: None,
            outputs: Vec::new(),
            default_output: None,
            telemetry_attributes: HashMap::new(),
            config: Value::Null,
        }
    }

    /// Creates a new Exporter `NodeUserConfig` with the node type URN.
    pub fn new_exporter_config<U: AsRef<str>>(node_type: U) -> Self {
        Self {
            r#type: crate::node_urn::normalize_plugin_urn_for_kind(
                node_type.as_ref(),
                NodeKind::Exporter,
            )
            .expect("invalid exporter node type"),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: Vec::new(),
            default_output: None,
            config: Value::Null,
        }
    }

    /// Creates a new Processor `NodeUserConfig` with the node type URN.
    pub fn new_processor_config<U: AsRef<str>>(node_type: U) -> Self {
        Self {
            r#type: crate::node_urn::normalize_plugin_urn_for_kind(
                node_type.as_ref(),
                NodeKind::Processor,
            )
            .expect("invalid processor node type"),
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: Vec::new(),
            default_output: None,
            config: Value::Null,
        }
    }

    /// Creates a new `NodeUserConfig` with the specified node type URN and user configuration.
    #[must_use]
    pub fn with_user_config(node_type: NodeUrn, user_config: Value) -> Self {
        Self {
            r#type: node_type,
            description: None,
            telemetry_attributes: HashMap::new(),
            outputs: Vec::new(),
            default_output: None,
            config: user_config,
        }
    }

    /// Adds an output port to this node declaration.
    pub fn add_output<P: Into<PortName>>(&mut self, port_name: P) {
        let port_name: PortName = port_name.into();
        if !self.outputs.iter().any(|output| output == &port_name) {
            self.outputs.push(port_name);
        }
    }

    /// Sets the default output port name used by this node when no explicit port is specified.
    pub fn set_default_output<P: Into<PortName>>(&mut self, port: P) {
        self.default_output = Some(port.into());
    }

    /// Returns this node kind from its URN.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.r#type.kind()
    }
}

/// Deserializes `telemetry_attributes` and rejects any attribute with an `Array` value,
/// which is not supported for log record attributes.
fn deserialize_telemetry_attributes<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, TelemetryAttribute>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let attrs: HashMap<String, TelemetryAttribute> = HashMap::deserialize(deserializer)?;
    for (key, attr) in &attrs {
        if matches!(attr.value(), AttributeValue::Array(_)) {
            return Err(serde::de::Error::custom(format!(
                "unsupported telemetry attribute type for `{key}`: array attributes are not supported"
            )));
        }
    }
    Ok(attrs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn node_user_config_minimal_valid() {
        let json = r#"{
            "type": "urn:example:demo:receiver"
        }"#;
        let cfg: NodeUserConfig = serde_json::from_str(json).unwrap();
        assert!(matches!(cfg.kind(), NodeKind::Receiver));
        assert!(cfg.outputs.is_empty());
    }

    #[test]
    fn test_yaml_node_config() {
        let yaml = r#"
type: "urn:otel:type_router:processor"
outputs: ["logs", "metrics", "traces"]
config: {}
"#;
        let cfg: NodeUserConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(cfg.kind(), NodeKind::Processor));
        assert_eq!(cfg.outputs.len(), 3);
    }

    #[test]
    fn test_yaml_node_outputs() {
        let yaml = r#"
type: "debug:processor"
outputs: ["logs", "metrics", "traces"]
config: {}
"#;
        let cfg: NodeUserConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(cfg.kind(), NodeKind::Processor));
        let expected: Vec<PortName> = vec!["logs", "metrics", "traces"]
            .into_iter()
            .map(Into::into)
            .collect();
        assert_eq!(cfg.outputs, expected);
    }

    #[test]
    fn node_user_config_with_telemetry_attributes_valid() {
        let json = r#"{
            "type": "urn:example:demo:receiver",
            "telemetry_attributes": {
                "attr1": "value1",
                "attr2": 123,
                "attr3": true
            }
        }"#;
        let cfg: NodeUserConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            cfg.telemetry_attributes
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "attr1".to_string(),
                "attr2".to_string(),
                "attr3".to_string(),
            ])
        );
        // Bare values have no brief
        assert!(
            cfg.telemetry_attributes
                .get("attr1")
                .unwrap()
                .brief()
                .is_none()
        );
    }

    #[test]
    fn node_user_config_with_telemetry_attributes_extended_form() {
        let json = r#"{
            "type": "urn:example:demo:receiver",
            "telemetry_attributes": {
                "region": {"value": "us-west", "brief": "Deployment region"},
                "count": 42,
                "team": {"value": "platform"}
            }
        }"#;
        let cfg: NodeUserConfig = serde_json::from_str(json).unwrap();
        let region = cfg.telemetry_attributes.get("region").unwrap();
        assert_eq!(
            *region.value(),
            AttributeValue::String("us-west".to_string())
        );
        assert_eq!(region.brief(), Some("Deployment region"));

        let count = cfg.telemetry_attributes.get("count").unwrap();
        assert_eq!(*count.value(), AttributeValue::I64(42));
        assert!(count.brief().is_none());

        let team = cfg.telemetry_attributes.get("team").unwrap();
        assert_eq!(
            *team.value(),
            AttributeValue::String("platform".to_string())
        );
        assert!(team.brief().is_none());
    }

    #[test]
    fn node_user_config_with_telemetry_attribute_array_expects_error() {
        let json = r#"{
            "type": "urn:example:demo:receiver",
            "telemetry_attributes": {
                "attr1": "value1",
                "attr2": [1, 2, 3]
            }
        }"#;
        let cfg: Result<NodeUserConfig, _> = serde_json::from_str(json);
        assert!(cfg.is_err());
    }
}
