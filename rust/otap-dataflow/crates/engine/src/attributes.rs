// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attributes describing the resource, engine, pipeline, and node context.
//!
//! Note: At the moment, these attributes are used for metrics aggregation and reporting.

use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
use otap_df_telemetry::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};
use otap_df_telemetry_macros::attribute_set;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::hash::Hash;

/// Convert from config `AttributeValue` to telemetry `AttributeValue`.
#[must_use]
pub fn config_to_telemetry_attr(
    value: &otap_df_config::pipeline::telemetry::AttributeValue,
) -> AttributeValue {
    use otap_df_config::pipeline::telemetry::AttributeValue as ConfigValue;
    match value {
        ConfigValue::String(s) => AttributeValue::String(s.clone()),
        ConfigValue::Bool(b) => AttributeValue::Boolean(*b),
        ConfigValue::I64(i) => AttributeValue::Int(*i),
        ConfigValue::F64(f) => AttributeValue::Double(*f),
        ConfigValue::Array(arr) => {
            // Encode arrays as a string representation
            AttributeValue::String(format!("{:?}", arr))
        }
    }
}

/// Convert a map of config `TelemetryAttribute`s to a telemetry `BTreeMap`,
/// extracting just the values (briefs are config-layer metadata only).
#[must_use]
pub fn config_map_to_telemetry(
    map: &std::collections::HashMap<
        String,
        otap_df_config::pipeline::telemetry::TelemetryAttribute,
    >,
) -> BTreeMap<String, AttributeValue> {
    map.iter()
        .map(|(k, attr)| (k.clone(), config_to_telemetry_attr(attr.value())))
        .collect()
}

/// Resource attributes (host id, process instance id, container id, ...).
#[attribute_set(name = "resource.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct ResourceAttributeSet {
    /// Unique process instance identifier (base32-encoded UUID v7).
    #[attribute]
    pub process_instance_id: Cow<'static, str>,
    /// Host identifier, when available (e.g. hostname).
    #[attribute]
    pub host_id: Cow<'static, str>,
    /// Container identifier, when available (e.g. Docker or containerd container ID).
    #[attribute]
    pub container_id: Cow<'static, str>,
}

/// Engine attributes (core id, numa node id, ...).
#[attribute_set(name = "controller.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct EngineAttributeSet {
    /// Core identifier.
    #[attribute]
    pub core_id: usize,

    /// Resource attributes.
    #[compose]
    pub resource_attrs: ResourceAttributeSet,

    /// NUMA node identifier.
    #[attribute]
    pub numa_node_id: usize,
}

/// Pipeline attributes.
#[attribute_set(name = "pipeline.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct PipelineAttributeSet {
    /// Pipeline identifier as defined in the configuration.
    #[attribute]
    pub pipeline_id: Cow<'static, str>,

    /// Engine attributes.
    #[compose]
    pub engine_attrs: EngineAttributeSet,

    /// Pipeline group identifier.
    #[attribute]
    pub pipeline_group_id: Cow<'static, str>,
}

/// Node attributes.
#[attribute_set(name = "node.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeAttributeSet {
    /// Node unique identifier (in scope of the pipeline).
    #[attribute]
    pub node_id: Cow<'static, str>,

    /// Pipeline attributes.
    #[compose]
    pub pipeline_attrs: PipelineAttributeSet,

    /// Node plugin URN.
    #[attribute(key = "node.urn")]
    pub node_urn: Cow<'static, str>,
    /// Node type (e.g., "receiver", "processor", "exporter").
    #[attribute]
    pub node_type: Cow<'static, str>,
}

/// Node attributes extended with user-configured custom telemetry attributes.
///
/// This is only used when a node has non-empty `telemetry_attributes` in its config.
/// Nodes without custom attributes use [`NodeAttributeSet`] directly, avoiding
/// empty `custom={}` noise in telemetry output.
#[attribute_set(name = "node.custom.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeWithCustomAttributeSet {
    /// Base node attributes.
    #[compose]
    pub node_attrs: NodeAttributeSet,

    /// Custom user-defined telemetry attributes.
    #[compose]
    pub custom_attrs: CustomAttributeSet,
}

/// A custom attribute set that holds arbitrary key-value pairs as a single
/// "custom" attribute with a `Map` value. This allows extending telemetry
/// with user-defined attributes without requiring static descriptors.
#[derive(Debug, Clone)]
pub struct CustomAttributeSet {
    values: Vec<AttributeValue>,
}

impl Default for CustomAttributeSet {
    fn default() -> Self {
        Self {
            values: vec![AttributeValue::Map(BTreeMap::new())],
        }
    }
}

impl Hash for CustomAttributeSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.values.len().hash(state);
        for v in &self.values {
            v.to_string_value().hash(state);
        }
    }
}

static CUSTOM_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
    name: "custom.attrs",
    fields: &[AttributeField {
        key: "custom",
        brief: "Custom user-defined attributes",
        r#type: AttributeValueType::Map,
    }],
};

impl CustomAttributeSet {
    /// Create a new custom attribute set from a map of key-value pairs.
    #[must_use]
    pub fn new(custom_attrs: BTreeMap<String, AttributeValue>) -> Self {
        Self {
            values: vec![AttributeValue::Map(custom_attrs)],
        }
    }
}

impl AttributeSetHandler for CustomAttributeSet {
    fn descriptor(&self) -> &'static AttributesDescriptor {
        &CUSTOM_ATTRIBUTES_DESCRIPTOR
    }

    fn attribute_values(&self) -> &[AttributeValue] {
        &self.values
    }
}

/// Channel endpoint attributes (sender or receiver).
#[attribute_set(name = "channel.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct ChannelAttributeSet {
    /// Unique channel identifier (in scope of the pipeline).
    #[attribute(key = "channel.id")]
    pub channel_id: Cow<'static, str>,

    /// Node attributes.
    #[compose]
    pub node_attrs: NodeAttributeSet,

    /// Port name for the channel endpoint.
    ///
    /// On the sender side, this is the port to which the channel is connected.
    /// On the receiver side, this defaults to the node's input port.
    #[attribute(key = "node.port")]
    pub node_port: Cow<'static, str>,

    /// Channel payload kind ("control" or "pdata").
    #[attribute(key = "channel.kind")]
    pub channel_kind: Cow<'static, str>,
    /// Concurrency mode of the channel ("local" or "shared").
    #[attribute(key = "channel.mode")]
    pub channel_mode: Cow<'static, str>,
    /// Channel type ("mpsc", "mpmc", "spsc", "spmc").
    #[attribute(key = "channel.type")]
    pub channel_type: Cow<'static, str>,
    /// Channel implementation ("tokio", "flume", "internal").
    #[attribute(key = "channel.impl")]
    pub channel_impl: Cow<'static, str>,
}
