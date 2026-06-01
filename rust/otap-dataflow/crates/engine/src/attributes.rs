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
/// extracting just the keys and values.
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

    /// Deployment generation for this runtime instance.
    #[attribute]
    pub deployment_generation: u64,
}

/// Host scope of an extension. Composed into [`ExtensionAttributeSet`] so
/// extensions hosted by distinct pipelines/cores/generations resolve to
/// distinct entities.
#[attribute_set(name = "extension.scope.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct ExtensionScopeAttributeSet {
    /// Scope kind (e.g., `"pipeline"`). Empty for the default scope.
    #[attribute(key = "scope.kind")]
    pub kind: Cow<'static, str>,

    /// Opaque scope identifier; encoding is owned by the constructor.
    #[attribute(key = "scope.id")]
    pub id: Cow<'static, str>,
}

impl ExtensionScopeAttributeSet {
    /// Constructs a scope with a caller-supplied kind and id.
    #[must_use]
    pub fn new(kind: impl Into<Cow<'static, str>>, id: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
        }
    }

    /// Constructs a pipeline-host scope. Id is
    /// `"<group>/<pipeline>/core/<core>/gen/<generation>"`.
    #[must_use]
    pub fn pipeline(
        pipeline_group_id: impl Into<Cow<'static, str>>,
        pipeline_id: impl Into<Cow<'static, str>>,
        core_id: usize,
        deployment_generation: u64,
    ) -> Self {
        let group = pipeline_group_id.into();
        let pipeline = pipeline_id.into();
        Self::new(
            "pipeline",
            format!("{group}/{pipeline}/core/{core_id}/gen/{deployment_generation}"),
        )
    }
}

/// Extension attributes. Composes [`ExtensionScopeAttributeSet`] so the
/// same `(extension.id, extension.variant)` hosted at distinct scopes
/// resolves to distinct entities.
#[attribute_set(name = "extension.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct ExtensionAttributeSet {
    /// Extension unique identifier within its host scope.
    #[attribute]
    pub extension_id: Cow<'static, str>,

    /// Host scope of the extension.
    #[compose]
    pub extension_scope: ExtensionScopeAttributeSet,

    /// Physical variant of the extension (`"local"` or `"shared"`).
    #[attribute(key = "extension.variant")]
    pub extension_variant: Cow<'static, str>,
}

impl ExtensionAttributeSet {
    /// Builder-style setter for the host scope.
    #[must_use]
    pub fn with_scope(mut self, scope: ExtensionScopeAttributeSet) -> Self {
        self.extension_scope = scope;
        self
    }
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
/// This is used only when a node has non-empty `entity.extend.identity_attributes` in its config.
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

/// Node attributes extended with a topic name.
#[attribute_set(name = "node.topic.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeWithTopicAttributeSet {
    /// Base node attributes.
    #[compose]
    pub node_attrs: NodeAttributeSet,
    /// Topic name associated with the node metrics.
    #[attribute]
    pub topic: Cow<'static, str>,
}

/// Node attributes (including custom telemetry attributes) extended with a topic name.
#[attribute_set(name = "node.custom.topic.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeWithCustomTopicAttributeSet {
    /// Base node + custom telemetry attributes.
    #[compose]
    pub node_custom_attrs: NodeWithCustomAttributeSet,
    /// Topic name associated with the node metrics.
    #[attribute]
    pub topic: Cow<'static, str>,
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

/// Channel endpoint attributes for a node-hosted channel.
#[attribute_set(name = "node.channel.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeChannelAttributeSet {
    /// Unique channel identifier within the host scope.
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

/// Channel endpoint attributes for an extension-hosted channel.
///
/// Extensions only have a single control-channel kind (MPSC), so `channel.kind`
/// and `channel.type` are intentionally omitted as invariants.
#[attribute_set(name = "extension.channel.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct ExtensionChannelAttributeSet {
    /// Unique channel identifier within the host scope.
    #[attribute(key = "channel.id")]
    pub channel_id: Cow<'static, str>,

    /// Extension attributes.
    #[compose]
    pub extension_attrs: ExtensionAttributeSet,

    /// Concurrency mode of the channel ("local" or "shared").
    #[attribute(key = "channel.mode")]
    pub channel_mode: Cow<'static, str>,
    /// Channel implementation ("tokio", "internal").
    #[attribute(key = "channel.impl")]
    pub channel_impl: Cow<'static, str>,
}
