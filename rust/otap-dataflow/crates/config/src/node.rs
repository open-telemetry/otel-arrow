// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node configuration specification.
//!
//! A node is a fundamental unit in our data processing pipeline, representing either a receiver
//! (source), processor, exporter (sink), or connector (linking pipelines).
//!
//! A node can expose multiple named output ports.

use crate::error::Error;
use crate::pipeline::telemetry::{AttributeValue, TelemetryAttribute};
use crate::transport_headers_policy::{HeaderCapturePolicy, HeaderPropagationPolicy};
use crate::{CapabilityId, Description, NodeId, NodeUrn, PortName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;

/// Deserializes a `HashMap<String, String>` while rejecting duplicate keys.
///
/// Standard serde deserialization into `HashMap` silently overwrites earlier
/// entries when keys are duplicated in the source. This function detects that
/// and returns an error so the user gets immediate feedback.
fn deserialize_no_dup_keys<'de, D>(
    deserializer: D,
) -> Result<HashMap<CapabilityId, NodeId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{MapAccess, Visitor};
    use std::fmt;

    struct NoDupVisitor;

    impl<'de> Visitor<'de> for NoDupVisitor {
        type Value = HashMap<CapabilityId, NodeId>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a map with no duplicate keys")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut result = HashMap::new();
            while let Some((key, value)) = map.next_entry::<String, String>()? {
                if result.contains_key(key.as_str()) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate capability key '{key}'"
                    )));
                }
                let _ = result.insert(CapabilityId::from(key), NodeId::from(value));
            }
            Ok(result)
        }
    }

    deserializer.deserialize_map(NoDupVisitor)
}

/// User configuration for a node in the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NodeUserConfig {
    /// The node type URN identifying the plugin (factory) to use for this node.
    ///
    /// Expected format:
    /// - `urn:<namespace>:<kind>:<id>`
    /// - `<kind>:<id>` (shortcut form for the `otel` namespace)
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

    /// Capability bindings mapping capability names to extension instance names.
    ///
    /// Each entry maps a capability (e.g., `bearer_token_provider`) to the name
    /// of an extension instance declared in the pipeline's `extensions` section.
    ///
    /// Example:
    /// ```yaml
    /// capabilities:
    ///   bearer_token_provider: azure_auth
    /// ```
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        deserialize_with = "deserialize_no_dup_keys"
    )]
    pub capabilities: HashMap<CapabilityId, NodeId>,

    /// Entity configuration for the node.
    ///
    /// Currently, we support entity::extend::identity_attributes, for example:
    ///
    /// ```yaml
    /// config:
    ///   ...
    /// entity:
    ///   extend:
    ///     identity_attributes:
    ///       region: "us-west"
    ///       team:
    ///         value: "platform"
    ///         brief: "team name"
    /// ```
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<NodeEntity>,

    /// Node-level header capture policy override (receivers only).
    ///
    /// When set on a receiver node, this policy **fully replaces** the
    /// pipeline-level `transport_headers.header_capture` policy for this
    /// node. When absent, the pipeline-level policy applies.
    ///
    /// Setting this field on a processor or exporter node is a
    /// configuration error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_capture: Option<HeaderCapturePolicy>,

    /// Node-level header propagation policy override (exporters only).
    ///
    /// When set on an exporter node, this policy **fully replaces** the
    /// pipeline-level `transport_headers.header_propagation` policy for this
    /// node. When absent, the pipeline-level policy applies.
    ///
    /// Setting this field on a processor or receiver node is a
    /// configuration error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_propagation: Option<HeaderPropagationPolicy>,
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
    /// A provider of shared capabilities (e.g., auth, service discovery).
    Extension,

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
            NodeKind::Extension => "extension".into(),
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
            entity: None,
            config: Value::Null,
            capabilities: HashMap::new(),
            header_capture: None,
            header_propagation: None,
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
            entity: None,
            outputs: Vec::new(),
            default_output: None,
            config: Value::Null,
            capabilities: HashMap::new(),
            header_capture: None,
            header_propagation: None,
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
            entity: None,
            outputs: Vec::new(),
            default_output: None,
            config: Value::Null,
            capabilities: HashMap::new(),
            header_capture: None,
            header_propagation: None,
        }
    }

    /// Creates a new `NodeUserConfig` with the specified node type URN and user configuration.
    #[must_use]
    pub fn with_user_config(node_type: NodeUrn, user_config: Value) -> Self {
        Self {
            r#type: node_type,
            description: None,
            entity: None,
            outputs: Vec::new(),
            default_output: None,
            config: user_config,
            capabilities: HashMap::new(),
            header_capture: None,
            header_propagation: None,
        }
    }

    /// Returns the identity attributes from the entity configuration, or an empty map if none.
    #[must_use]
    pub fn identity_attributes(&self) -> HashMap<String, TelemetryAttribute> {
        self.entity
            .as_ref()
            .and_then(|e| e.extend.as_ref())
            .map(|ext| &ext.identity_attributes)
            .cloned()
            .unwrap_or_default()
    }

    /// Validates transport header policy fields on this node and pushes any
    /// errors into the provided vector. Receivers may only declare
    /// `header_capture`; exporters may only declare `header_propagation`;
    /// processors may declare neither.
    pub fn validate_transport_header_fields(&self, node_name: &str, errors: &mut Vec<Error>) {
        let kind = self.kind();

        if self.header_capture.is_some() && kind != NodeKind::Receiver {
            errors.push(Error::InvalidUserConfig {
                error: format!(
                    "node `{node_name}`: `header_capture` is only allowed on receiver nodes \
                     (this node is a {kind})",
                    kind = match kind {
                        NodeKind::Processor => "processor",
                        NodeKind::Exporter => "exporter",
                        NodeKind::ProcessorChain => "processor_chain",
                        NodeKind::Extension => "extension",
                        NodeKind::Receiver => unreachable!(),
                    }
                ),
            });
        }

        if self.header_propagation.is_some() && kind != NodeKind::Exporter {
            errors.push(Error::InvalidUserConfig {
                error: format!(
                    "node `{node_name}`: `header_propagation` is only allowed on exporter nodes \
                     (this node is a {kind})",
                    kind = match kind {
                        NodeKind::Receiver => "receiver",
                        NodeKind::Processor => "processor",
                        NodeKind::ProcessorChain => "processor_chain",
                        NodeKind::Extension => "extension",
                        NodeKind::Exporter => unreachable!(),
                    }
                ),
            });
        }

        // Validate the selector shape inside node-level header_propagation so
        // that invalid selectors are rejected uniformly.
        if let Some(propagation) = &self.header_propagation {
            if let Err(e) = propagation.validate() {
                errors.push(Error::InvalidUserConfig {
                    error: format!("node `{node_name}`: header_propagation.default.selector: {e}"),
                });
            }
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

/// Entity configuration for a node, aligned with the semantic conventions model.
/// See https://opentelemetry.io/docs/specs/otel/entities/data-model/.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NodeEntity {
    /// Extensions to the entity's attribute sets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extend: Option<ExtendedNodeEntity>,
}

/// Node entity extensions, including user-provided identifying attributes.
/// See https://opentelemetry.io/docs/specs/otel/entities/data-model/.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExtendedNodeEntity {
    /// Attributes that identify this node in telemetry emitted
    /// from the dataflow engine.
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        deserialize_with = "deserialize_identity_attributes"
    )]
    pub identity_attributes: HashMap<String, TelemetryAttribute>,
}

/// Deserializes `identity_attributes` and rejects any attribute with an `Array` value,
/// which is not supported for log record attributes.
fn deserialize_identity_attributes<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, TelemetryAttribute>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let attrs: HashMap<String, TelemetryAttribute> = HashMap::deserialize(deserializer)?;
    for (key, attr) in &attrs {
        if matches!(attr.value(), AttributeValue::Array(_)) {
            return Err(serde::de::Error::custom(format!(
                "unsupported identity attribute type for `{key}`: array attributes are not supported"
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
            "type": "urn:example:receiver:demo"
        }"#;
        let cfg: NodeUserConfig = serde_json::from_str(json).unwrap();
        assert!(matches!(cfg.kind(), NodeKind::Receiver));
        assert!(cfg.outputs.is_empty());
    }

    #[test]
    fn test_yaml_node_config() {
        let yaml = r#"
type: "urn:otel:processor:type_router"
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
type: "processor:debug"
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
    fn node_user_config_with_entity_identity_attributes_valid() {
        let json = r#"{
            "type": "urn:example:receiver:demo",
            "entity": {
                "extend": {
                    "identity_attributes": {
                        "attr1": "value1",
                        "attr2": 123,
                        "attr3": true
                    }
                }
            }
        }"#;
        let cfg: NodeUserConfig = serde_json::from_str(json).unwrap();
        let identity_attrs = cfg.identity_attributes();
        assert_eq!(
            identity_attrs.keys().cloned().collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "attr1".to_string(),
                "attr2".to_string(),
                "attr3".to_string(),
            ])
        );
        // Bare values have no brief
        assert!(identity_attrs.get("attr1").unwrap().brief().is_none());
    }

    #[test]
    fn node_user_config_with_entity_identity_attributes_extended_form() {
        let json = r#"{
            "type": "urn:example:receiver:demo",
            "entity": {
                "extend": {
                    "identity_attributes": {
                        "region": {"value": "us-west", "brief": "Deployment region"},
                        "count": 42,
                        "team": {"value": "platform"}
                    }
                }
            }
        }"#;
        let cfg: NodeUserConfig = serde_json::from_str(json).unwrap();
        let identity_attrs = cfg.identity_attributes();
        let region = identity_attrs.get("region").unwrap();
        assert_eq!(
            *region.value(),
            AttributeValue::String("us-west".to_string())
        );
        assert_eq!(region.brief(), Some("Deployment region"));

        let count = identity_attrs.get("count").unwrap();
        assert_eq!(*count.value(), AttributeValue::I64(42));
        assert!(count.brief().is_none());

        let team = identity_attrs.get("team").unwrap();
        assert_eq!(
            *team.value(),
            AttributeValue::String("platform".to_string())
        );
        assert!(team.brief().is_none());
    }

    #[test]
    fn node_user_config_with_entity_identity_attribute_array_expects_error() {
        let json = r#"{
            "type": "urn:example:receiver:demo",
            "entity": {
                "extend": {
                    "identity_attributes": {
                        "attr1": "value1",
                        "attr2": [1, 2, 3]
                    }
                }
            }
        }"#;
        let cfg: Result<NodeUserConfig, _> = serde_json::from_str(json);
        assert!(cfg.is_err());
    }

    #[test]
    fn node_user_config_no_entity_returns_empty_identity_attributes() {
        let json = r#"{
            "type": "urn:example:receiver:demo"
        }"#;
        let cfg: NodeUserConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.identity_attributes().is_empty());
    }

    #[test]
    fn node_user_config_entity_without_extend_returns_empty_identity_attributes() {
        let json = r#"{
            "type": "urn:example:receiver:demo",
            "entity": {}
        }"#;
        let cfg: NodeUserConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.entity.is_some());
        assert!(cfg.identity_attributes().is_empty());
    }

    // -- Transport header node-level override tests --

    #[test]
    fn receiver_with_header_capture_override() {
        let yaml = r#"
type: "receiver:otap"
header_capture:
  headers:
    - match_names: ["x-request-id"]
      store_as: request_id
config:
  listening_addr: "127.0.0.1:50051"
"#;
        let cfg: NodeUserConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(cfg.kind(), NodeKind::Receiver));
        assert!(cfg.header_capture.is_some());
        assert!(cfg.header_propagation.is_none());

        // No validation errors for receiver + header_capture
        let mut errors = Vec::new();
        cfg.validate_transport_header_fields("test_node", &mut errors);
        assert!(errors.is_empty());

        let capture = cfg.header_capture.as_ref().unwrap();
        assert_eq!(capture.headers.len(), 1);
        assert_eq!(capture.headers[0].match_names, vec!["x-request-id"]);
        assert_eq!(capture.headers[0].store_as.as_deref(), Some("request_id"));
    }

    #[test]
    fn exporter_with_header_propagation_override() {
        let yaml = r#"
type: "exporter:otap"
header_propagation:
  default:
    selector:
        type: all_captured
  overrides:
    - match:
        stored_names: ["authorization"]
      action: drop
config:
  grpc_endpoint: "http://127.0.0.1:50051"
"#;
        let cfg: NodeUserConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(cfg.kind(), NodeKind::Exporter));
        assert!(cfg.header_capture.is_none());
        assert!(cfg.header_propagation.is_some());

        // No validation errors for exporter + header_propagation
        let mut errors = Vec::new();
        cfg.validate_transport_header_fields("test_node", &mut errors);
        assert!(errors.is_empty());

        let propagation = cfg.header_propagation.as_ref().unwrap();
        assert_eq!(propagation.overrides.len(), 1);
        assert_eq!(
            propagation.overrides[0].match_rule.stored_names,
            vec!["authorization"]
        );
    }

    #[test]
    fn capabilities_rejects_duplicate_keys_yaml() {
        let yaml = r#"
type: "urn:otel:exporter:test"
capabilities:
  bearer_token_provider: ext_a
  bearer_token_provider: ext_b
"#;
        let result: Result<NodeUserConfig, _> = serde_yaml::from_str(yaml);
        let err = result.expect_err("should reject duplicate capability keys");
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate"),
            "error should mention duplicate: {msg}"
        );
    }

    #[test]
    fn header_capture_on_processor_is_rejected() {
        let mut cfg = NodeUserConfig::new_processor_config("processor:batch");
        cfg.header_capture = Some(HeaderCapturePolicy::default());
        let mut errors = Vec::new();
        cfg.validate_transport_header_fields("batch", &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("header_capture"));
        assert!(errors[0].to_string().contains("processor"));
    }

    #[test]
    fn header_capture_on_exporter_is_rejected() {
        let mut cfg = NodeUserConfig::new_exporter_config("exporter:otap");
        cfg.header_capture = Some(HeaderCapturePolicy::default());
        let mut errors = Vec::new();
        cfg.validate_transport_header_fields("otap_export", &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("header_capture"));
        assert!(errors[0].to_string().contains("exporter"));
    }

    #[test]
    fn header_propagation_on_receiver_is_rejected() {
        let mut cfg = NodeUserConfig::new_receiver_config("receiver:otap");
        cfg.header_propagation = Some(HeaderPropagationPolicy::default());
        let mut errors = Vec::new();
        cfg.validate_transport_header_fields("otap_ingest", &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("header_propagation"));
        assert!(errors[0].to_string().contains("receiver"));
    }

    #[test]
    fn receiver_without_override_has_no_validation_errors() {
        let cfg = NodeUserConfig::new_receiver_config("receiver:otap");
        assert!(cfg.header_capture.is_none());
        assert!(cfg.header_propagation.is_none());
        let mut errors = Vec::new();
        cfg.validate_transport_header_fields("test", &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn exporter_with_invalid_propagation_selector_is_rejected() {
        use crate::transport_headers_policy::{
            PropagationDefault, PropagationSelector, PropagationSelectorType,
        };

        let mut cfg = NodeUserConfig::new_exporter_config("exporter:otap");
        cfg.header_propagation = Some(HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::Named,
                    named: None, // Invalid: named type requires named list
                },
                ..Default::default()
            },
            vec![],
        ));
        let mut errors = Vec::new();
        cfg.validate_transport_header_fields("otap_export", &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("header_propagation"));
        assert!(errors[0].to_string().contains("'named' list is required"));
    }

    #[test]
    fn exporter_with_valid_propagation_selector_passes() {
        use crate::transport_headers_policy::{
            PropagationDefault, PropagationSelector, PropagationSelectorType,
        };

        let mut cfg = NodeUserConfig::new_exporter_config("exporter:otap");
        cfg.header_propagation = Some(HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::Named,
                    named: Some(vec!["tenant_id".to_string()]),
                },
                ..Default::default()
            },
            vec![],
        ));
        let mut errors = Vec::new();
        cfg.validate_transport_header_fields("otap_export", &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn capabilities_rejects_duplicate_keys_json() {
        let json = r#"{
            "type": "urn:otel:exporter:test",
            "capabilities": {
                "bearer_token_provider": "ext_a",
                "bearer_token_provider": "ext_b"
            }
        }"#;
        let result: Result<NodeUserConfig, _> = serde_json::from_str(json);
        let err = result.expect_err("should reject duplicate capability keys");
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate"),
            "error should mention duplicate: {msg}"
        );
    }
}
