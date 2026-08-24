// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Attributes describing the resource, engine, pipeline, and node context.
//!
//! Note: At the moment, these attributes are used for metrics aggregation and reporting.

use otap_df_telemetry::attributes::{
    AttributeKeySchema, AttributeSetHandler, AttributeSetKeySchema, AttributeValue,
};
use otap_df_telemetry::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};
use otap_df_telemetry_macros::{AttributeEnum, attribute_set};
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

/// Engine attributes (core id, numa node id, ...).
#[attribute_set(scope, name = "controller.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct EngineAttributeSet {
    /// Core identifier.
    pub core_id: usize,

    /// NUMA node identifier.
    pub numa_node_id: usize,
}

static ENGINE_ENTITY_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
    name: "engine",
    fields: &[],
};

/// Empty attribute set for the engine-global entity. Process/host identity
/// now lives on the OTel Resource layer, so engine-wide metrics carry no
/// scope attributes.
#[derive(Debug, Clone, Default, Hash)]
pub struct EngineEntityAttributeSet;

impl AttributeSetHandler for EngineEntityAttributeSet {
    fn descriptor(&self) -> &'static AttributesDescriptor {
        &ENGINE_ENTITY_DESCRIPTOR
    }

    fn attribute_values(&self) -> &[AttributeValue] {
        &[]
    }
}

/// Pipeline attributes.
#[attribute_set(scope, name = "pipeline.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct PipelineAttributeSet {
    /// Pipeline identifier as defined in the configuration.
    pub pipeline_id: Cow<'static, str>,

    /// Engine attributes.
    #[compose]
    pub engine_attrs: EngineAttributeSet,

    /// Pipeline group identifier.
    pub pipeline_group_id: Cow<'static, str>,

    /// Deployment generation for this runtime instance.
    pub deployment_generation: u64,
}

/// Host scope of an extension. Composed into [`ExtensionAttributeSet`] to
/// disambiguate extensions across hosting scopes.
///
/// Fields are private; the type can only be constructed through a scope-kind
/// constructor (e.g. [`ExtensionScopeAttributeSet::pipeline`]). This enforces
/// the invariant that every scope value has a populated payload matching its
/// `scope.kind` discriminator -- there is no way to build a "kind-less" or
/// inconsistent scope set in the public API.
///
/// When new scope kinds are introduced (e.g. `"engine"`, `"group"`),
/// add a corresponding `#[compose]` payload field below and a matching
/// constructor; existing constructors keep new payloads at `Default` so the
/// descriptor stays stable across scope kinds.
#[attribute_set(scope, name = "extension.scope.attrs")]
#[derive(Debug, Clone, Hash)]
pub struct ExtensionScopeAttributeSet {
    /// Scope kind discriminator. Always paired with the populated payload
    /// field that matches it.
    #[attribute_key = "scope.kind"]
    pub(crate) kind: Cow<'static, str>,

    /// Pipeline-scope payload. Populated when `kind == "pipeline"`; left at
    /// `Default::default()` for other scope kinds.
    #[compose]
    pub(crate) pipeline: PipelineAttributeSet,
}

impl Default for ExtensionScopeAttributeSet {
    /// Sentinel default used by the `#[compose]` macro to compute the cached
    /// composed descriptor once at startup. The produced value carries an
    /// empty `scope.kind` and is **not** a valid scope identity -- production
    /// telemetry must construct values through a scope-kind constructor
    /// (e.g. [`ExtensionScopeAttributeSet::pipeline`]).
    fn default() -> Self {
        Self {
            kind: Cow::Borrowed(""),
            pipeline: PipelineAttributeSet::default(),
        }
    }
}

impl ExtensionScopeAttributeSet {
    /// Pipeline-host scope. The full pipeline attribute set (group id,
    /// pipeline id, engine id, generation, resource attrs, ...) is composed
    /// into the resulting scope so two distinct `(group, pipeline)` pairs
    /// can never collide on identity, regardless of the characters they
    /// contain.
    #[must_use]
    pub fn pipeline(pipeline: PipelineAttributeSet) -> Self {
        Self {
            kind: Cow::Borrowed("pipeline"),
            pipeline,
        }
    }
}

/// Extension attributes, including the host scope.
#[attribute_set(scope, name = "extension.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct ExtensionAttributeSet {
    /// Extension unique identifier within its host scope.
    pub extension_id: Cow<'static, str>,

    /// Host scope of the extension.
    #[compose]
    pub extension_scope: ExtensionScopeAttributeSet,

    /// Physical variant of the extension (`"local"` or `"shared"`).
    #[attribute_key = "extension.variant"]
    pub extension_variant: Cow<'static, str>,
}

/// Node attributes.
#[attribute_set(scope, name = "node.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeAttributeSet {
    /// Node unique identifier (in scope of the pipeline).
    pub node_id: Cow<'static, str>,

    /// Pipeline attributes.
    #[compose]
    pub pipeline_attrs: PipelineAttributeSet,

    /// Node plugin URN.
    #[attribute_key = "node.urn"]
    pub node_urn: Cow<'static, str>,
    /// Node type (e.g., "receiver", "processor", "exporter").
    pub node_type: Cow<'static, str>,
}

/// Node attributes extended with user-configured custom telemetry attributes.
///
/// This is used only when a node has non-empty `entity.extend.identity_attributes` in its config.
/// Nodes without custom attributes use [`NodeAttributeSet`] directly, avoiding
/// empty `custom={}` noise in telemetry output.
#[attribute_set(scope, name = "node.custom.attrs")]
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
#[attribute_set(scope, name = "node.topic.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeWithTopicAttributeSet {
    /// Base node attributes.
    #[compose]
    pub node_attrs: NodeAttributeSet,
    /// Topic name associated with the node metrics.
    pub topic: Cow<'static, str>,
}

/// Node attributes (including custom telemetry attributes) extended with a topic name.
#[attribute_set(scope, name = "node.custom.topic.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct NodeWithCustomTopicAttributeSet {
    /// Base node + custom telemetry attributes.
    #[compose]
    pub node_custom_attrs: NodeWithCustomAttributeSet,
    /// Topic name associated with the node metrics.
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

impl AttributeSetKeySchema for CustomAttributeSet {
    const KEY_SCHEMA: &'static [AttributeKeySchema] = &[AttributeKeySchema::Key("custom")];
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_telemetry::attributes::{AttributeEnum, AttributeSetHandler};

    /// Distinct `(group, pipeline)` pairs must not collide on attribute
    /// values: flattening into a single `/`-separated string allows two
    /// real scopes to register the same telemetry entity.
    #[test]
    fn pipeline_scope_ids_are_unambiguous_across_group_pipeline_splits() {
        let a = ExtensionScopeAttributeSet::pipeline(PipelineAttributeSet {
            pipeline_group_id: "a/b".into(),
            pipeline_id: "c".into(),
            ..PipelineAttributeSet::default()
        });
        let b = ExtensionScopeAttributeSet::pipeline(PipelineAttributeSet {
            pipeline_group_id: "a".into(),
            pipeline_id: "b/c".into(),
            ..PipelineAttributeSet::default()
        });
        // `attribute_values` reuses a thread-local buffer; copy each set
        // before invoking the next.
        let a_values = a.attribute_values().to_vec();
        let b_values = b.attribute_values().to_vec();
        assert_ne!(
            a_values, b_values,
            "distinct (group, pipeline) pairs must not collide on attribute values; \
             flattening `{{group}}/{{pipeline}}` into one opaque string allows \
             two real scopes to register the same telemetry entity"
        );
    }

    /// Scenario: Channel entity dimensions are represented by closed enum value sets.
    /// Guarantees: Their cardinalities and exported lowercase values remain stable.
    #[test]
    fn channel_attribute_enums_have_stable_values() {
        assert_eq!(ChannelKind::CARDINALITY, 2);
        assert_eq!(ChannelKind::VARIANTS, &["control", "pdata"]);
        assert_eq!(ChannelMode::CARDINALITY, 2);
        assert_eq!(ChannelMode::VARIANTS, &["local", "shared"]);
        assert_eq!(ChannelType::CARDINALITY, 2);
        assert_eq!(ChannelType::VARIANTS, &["mpsc", "mpmc"]);
        assert_eq!(ChannelImplementation::CARDINALITY, 3);
        assert_eq!(
            ChannelImplementation::VARIANTS,
            &["internal", "tokio", "flume"]
        );
    }

    /// Scenario: A node channel entity is built from typed channel dimensions.
    /// Guarantees: Scope attributes retain their established keys and string values.
    #[test]
    fn node_channel_attribute_enums_serialize_as_scope_strings() {
        let attrs = NodeChannelAttributeSet {
            channel_id: "channel-a".into(),
            node_attrs: NodeAttributeSet::default(),
            node_port: "output".into(),
            channel_kind: ChannelKind::Pdata,
            channel_mode: ChannelMode::Shared,
            channel_type: ChannelType::Mpmc,
            channel_impl: ChannelImplementation::Flume,
        };
        let attr_map: BTreeMap<&'static str, String> = attrs
            .iter_attributes()
            .map(|(key, value)| (key, value.to_string_value()))
            .collect();

        assert_eq!(
            attr_map.get("channel.kind").map(String::as_str),
            Some("pdata")
        );
        assert_eq!(
            attr_map.get("channel.mode").map(String::as_str),
            Some("shared")
        );
        assert_eq!(
            attr_map.get("channel.type").map(String::as_str),
            Some("mpmc")
        );
        assert_eq!(
            attr_map.get("channel.impl").map(String::as_str),
            Some("flume")
        );
    }
}

/// Payload carried by an internal channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AttributeEnum)]
pub enum ChannelKind {
    /// Engine control messages.
    Control,
    /// Pipeline telemetry data.
    Pdata,
}

/// Concurrency boundary crossed by an internal channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AttributeEnum)]
pub enum ChannelMode {
    /// Both endpoints run on the same local executor.
    Local,
    /// The channel can cross thread or executor boundaries.
    Shared,
}

/// Producer/consumer topology supported by an internal channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AttributeEnum)]
pub enum ChannelType {
    /// Multiple producers and a single consumer.
    Mpsc,
    /// Multiple producers and multiple consumers.
    Mpmc,
}

/// Runtime implementation backing an internal channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AttributeEnum)]
pub enum ChannelImplementation {
    /// OTAP Dataflow's internal local channel implementation.
    Internal,
    /// Tokio's channel implementation.
    Tokio,
    /// Flume's channel implementation.
    Flume,
}

/// Channel endpoint attributes for a node-hosted channel.
#[attribute_set(scope, name = "node.channel.attrs")]
#[derive(Debug, Clone, Hash)]
pub struct NodeChannelAttributeSet {
    /// Unique channel identifier within the host scope.
    #[attribute_key = "channel.id"]
    pub channel_id: Cow<'static, str>,

    /// Node attributes.
    #[compose]
    pub node_attrs: NodeAttributeSet,

    /// Port name for the channel endpoint.
    ///
    /// On the sender side, this is the port to which the channel is connected.
    /// On the receiver side, this defaults to the node's input port.
    #[attribute_key = "node.port"]
    pub node_port: Cow<'static, str>,

    /// Channel payload kind ("control" or "pdata").
    #[attribute_key = "channel.kind"]
    pub channel_kind: ChannelKind,
    /// Concurrency mode of the channel ("local" or "shared").
    #[attribute_key = "channel.mode"]
    pub channel_mode: ChannelMode,
    /// Channel type ("mpsc" or "mpmc").
    #[attribute_key = "channel.type"]
    pub channel_type: ChannelType,
    /// Channel implementation ("tokio", "flume", "internal").
    #[attribute_key = "channel.impl"]
    pub channel_impl: ChannelImplementation,
}

/// Channel endpoint attributes for a node-hosted channel, extended with user-configured custom telemetry attributes.
#[attribute_set(scope, name = "node.channel.custom.attrs")]
#[derive(Debug, Clone, Hash)]
pub struct NodeWithCustomChannelAttributeSet {
    /// Base node channel attributes.
    #[compose]
    pub channel_attrs: NodeChannelAttributeSet,

    /// Custom user-defined telemetry attributes.
    #[compose]
    pub custom_attrs: CustomAttributeSet,
}

/// Channel endpoint attributes for an extension-hosted channel.
///
/// Extensions only have a single control-channel kind (MPSC), so `channel.kind`
/// and `channel.type` are intentionally omitted as invariants.
#[attribute_set(scope, name = "extension.channel.attrs")]
#[derive(Debug, Clone, Hash)]
pub struct ExtensionChannelAttributeSet {
    /// Unique channel identifier within the host scope.
    #[attribute_key = "channel.id"]
    pub channel_id: Cow<'static, str>,

    /// Extension attributes.
    #[compose]
    pub extension_attrs: ExtensionAttributeSet,

    /// Concurrency mode of the channel ("local" or "shared").
    #[attribute_key = "channel.mode"]
    pub channel_mode: ChannelMode,
    /// Channel implementation ("tokio", "internal").
    #[attribute_key = "channel.impl"]
    pub channel_impl: ChannelImplementation,
}
