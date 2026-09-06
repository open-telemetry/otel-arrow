// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component context declarations collected before runtime construction.

use crate::PipelineFactory;
use crate::error::Error as EngineError;
use linkme::distributed_slice;
use otel_arrow_dfe_config::engine::ResolvedOtelDataflowSpec;
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::node::{NodeKind, NodeUserConfig};
use otel_arrow_dfe_config::transport_headers_policy::{
    CompiledHeaderCapturePolicy, HeaderCapturePolicy, HeaderPropagationPolicy,
    TransportHeadersPolicy,
};
use otel_arrow_dfe_config::{
    ContextEntryName, NodeId as ConfigNodeId, PipelineGroupId, PipelineKey, TopicName,
};
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, OnceLock};

const TOPIC_EXPORTER_URN: &str = "urn:otel:exporter:topic";
const TOPIC_RECEIVER_URN: &str = "urn:otel:receiver:topic";

type TopicKey = (PipelineGroupId, TopicName);
type ContextNodeKey = (PipelineKey, ConfigNodeId);

enum TopicEndpoint {
    Producer(TopicName),
    Consumer(TopicName),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct OriginalNameConsumers {
    all_original_names: bool,
    entries: BTreeSet<ContextEntryName>,
    propagation_policies: Vec<HeaderPropagationPolicy>,
}

impl OriginalNameConsumers {
    fn add_declarations(
        &mut self,
        declarations: &NodeContextDeclarations,
        propagation_policy: Option<&HeaderPropagationPolicy>,
    ) {
        for declaration in declarations.iter() {
            let ContextDeclaration::Consumes { selector } = declaration else {
                continue;
            };
            match selector {
                ContextConsumerSelector::Entries { entries } => {
                    self.entries.extend(
                        entries
                            .iter()
                            .filter(|entry| {
                                entry.read == ContextEntrySelectorForm::OriginalKeyValue
                            })
                            .map(|entry| entry.name.clone()),
                    );
                }
                ContextConsumerSelector::AllNormalized => {}
                ContextConsumerSelector::AllOriginal => self.all_original_names = true,
                ContextConsumerSelector::HeaderPropagationPolicy => {
                    if let Some(policy) = propagation_policy {
                        self.add_propagation_policy(policy);
                    }
                }
            }
        }
    }

    fn add_propagation_policy(&mut self, policy: &HeaderPropagationPolicy) {
        if !self.propagation_policies.contains(policy) {
            self.propagation_policies.push(policy.clone());
        }
    }

    fn consumes(&self, name: &ContextEntryName) -> bool {
        self.all_original_names
            || self.entries.contains(name)
            || self
                .propagation_policies
                .iter()
                .any(|policy| policy.propagates_original_name(name))
    }

    fn extend(&mut self, other: &Self) -> bool {
        let mut changed = false;
        if other.all_original_names && !self.all_original_names {
            self.all_original_names = true;
            changed = true;
        }
        let previous_entries = self.entries.len();
        self.entries.extend(other.entries.iter().cloned());
        changed |= self.entries.len() != previous_entries;
        for policy in &other.propagation_policies {
            if !self.propagation_policies.contains(policy) {
                self.propagation_policies.push(policy.clone());
                changed = true;
            }
        }
        changed
    }

    fn is_empty(&self) -> bool {
        !self.all_original_names && self.entries.is_empty() && self.propagation_policies.is_empty()
    }
}

/// The entry selector
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContextEntrySelector {
    /// The name
    pub name: ContextEntryName,
    /// The form
    pub read: ContextEntrySelectorForm,
}

/// When an association list is captured, do we store the associated
/// field names (e.g., header names)?
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextEntrySelectorForm {
    /// Consumers use only the value selected by normalized field name.
    #[default]
    Value,
    /// Consumers use normalized field names with their values.
    NormalizedKeyValue,
    /// Consumers use original field names with their values.
    OriginalKeyValue,
}

/// Generic context registers selected by one consumer binding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextConsumerSelector {
    /// Selects named context entries in order.
    Entries {
        /// Logical context entry references.
        entries: Box<[ContextEntrySelector]>,
    },
    /// Selects every context entry using normalized names.
    AllNormalized,
    /// Selects every context entry using original names.
    AllOriginal,
    /// Selects entries according to the effective header propagation policy.
    HeaderPropagationPolicy,
}

/// One component context declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextDeclaration {
    /// Adds one named value to outgoing context.
    Produces {
        /// The logical context entry that will be produced.
        entry: ContextEntryName,
    },
    /// Reads context through one compiled access.
    Consumes {
        /// Generic register selection.
        selector: ContextConsumerSelector,
    },
}

/// A configuration-dependent declaration provider registered by a component.
#[derive(Clone, Copy)]
pub struct ContextDeclarationProvider {
    /// The registered component's URN.
    pub urn: &'static str,
    /// Produces declarations using a component configuration.
    pub declarations: ContextDeclarationFn,
}

/// Deterministically describes a node factory's context access.
pub type ContextDeclarationFn = fn(&serde_json::Value) -> Result<NodeContextDeclarations, Error>;

/// How Config structs declare node context bindings.
pub trait ConfigNodeContextDeclaration: serde::de::DeserializeOwned {
    /// Returns the context accesses required by this configuration.
    fn context_declarations(&self) -> NodeContextDeclarations;

    /// Verifies this parsed configuration against the engine-compiled declarations.
    fn validate_context_declarations(
        &self,
        pipeline_ctx: &crate::context::PipelineContext,
    ) -> Result<(), Error> {
        pipeline_ctx
            .compiled_context_policy()
            .validate_node_declarations(
                &pipeline_ctx.pipeline_key(),
                &pipeline_ctx.node_id(),
                &self.context_declarations(),
            )
    }
}

// `#[allow(unsafe_code)]` is required because `linkme::distributed_slice`
// emits a static with `#[link_section = "..."]`, which the engine crate's
// `-D unsafe-code` lint would otherwise reject.
/// Context declaration providers registered by components.
#[allow(unsafe_code)]
#[distributed_slice]
pub static CONTEXT_DECLARATION_PROVIDERS: [ContextDeclarationProvider];

/// A fixed set of context bindings, equals a bi-directional mapping
/// from ContextAccessId to/from ContextDeclaration. Created by
/// collecting FromIterator<ContextDeclaration>.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeContextDeclarations {
    /// Indexed by ContextAccessId; sorted in the builder
    byid: Box<[ContextDeclaration]>,
}

/// Context access identifier scoped to one node factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextAccessId(usize);

impl FromIterator<ContextDeclaration> for NodeContextDeclarations {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = ContextDeclaration>,
    {
        let mut uniq: Vec<_> = iter.into_iter().collect();
        uniq.sort();
        uniq.dedup();
        Self {
            byid: uniq.into_boxed_slice(),
        }
    }
}

impl NodeContextDeclarations {
    /// Iterates over declarations in access ID order.
    pub fn iter(&self) -> impl Iterator<Item = &ContextDeclaration> {
        self.byid.iter()
    }

    /// Is this empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.byid.is_empty()
    }

    /// Return the number of declarations
    #[must_use]
    pub fn len(&self) -> usize {
        self.byid.len()
    }
}

impl ContextAccessId {
    /// Creates a provider-local access identifier.
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the provider-local identifier.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl ContextDeclarationProvider {
    /// Creates a provider that derives declarations from component configuration.
    #[must_use]
    pub const fn from_config(urn: &'static str, declarations: ContextDeclarationFn) -> Self {
        Self { urn, declarations }
    }

    /// Creates a provider that derives declarations from a typed component configuration.
    #[must_use]
    pub const fn from_typed_config<T>(urn: &'static str) -> Self
    where
        T: ConfigNodeContextDeclaration,
    {
        Self {
            urn,
            declarations: typed_context_declarations::<T>,
        }
    }
}

/// Generic function used in from_typed_config.
fn typed_context_declarations<T>(
    config: &serde_json::Value,
) -> Result<NodeContextDeclarations, Error>
where
    T: ConfigNodeContextDeclaration,
{
    Ok(
        otel_arrow_dfe_config::validation::deserialize_typed_config::<T>(config)?
            .context_declarations(),
    )
}

/// Context policy compiled from resolved configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledContextPolicy {
    declarations: HashMap<PipelineKey, HashMap<ConfigNodeId, NodeContextDeclarations>>,
    original_name_consumers: HashMap<ContextNodeKey, OriginalNameConsumers>,
}

impl CompiledContextPolicy {
    /// The empty state has no declarations, bind always fails.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            declarations: HashMap::new(),
            original_name_consumers: HashMap::new(),
        }
    }

    /// Compiles exact per-match original-name requirements into a capture policy.
    #[must_use]
    pub(crate) fn compile_header_capture_policy(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
        policy: HeaderCapturePolicy,
    ) -> CompiledHeaderCapturePolicy {
        let consumers = self
            .original_name_consumers
            .get(&(pipeline.clone(), node.clone()));
        policy.compile(|name| consumers.is_some_and(|consumers| consumers.consumes(name)))
    }

    #[cfg(test)]
    fn consumes_original_name(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
        name: &ContextEntryName,
    ) -> bool {
        self.original_name_consumers
            .get(&(pipeline.clone(), node.clone()))
            .is_some_and(|consumers| consumers.consumes(name))
    }

    /// Validates that a node's declarations appear verbatim in the compiled policy.
    pub fn validate_node_declarations(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
        declarations: &NodeContextDeclarations,
    ) -> Result<(), Error> {
        match self
            .declarations
            .get(pipeline)
            .and_then(|nodes| nodes.get(node))
        {
            Some(expected) if expected == declarations => Ok(()),
            _ => Err(Error::UnrecognizedContextDeclaration {}),
        }
    }
}

impl<PData: 'static + Clone + std::fmt::Debug> PipelineFactory<PData> {
    /// Compiles context policy from the complete resolved configuration.
    pub fn compile_context_policy(
        &self,
        resolved: &ResolvedOtelDataflowSpec,
    ) -> Result<Arc<CompiledContextPolicy>, EngineError> {
        let mut declarations = HashMap::new();
        let mut original_name_consumers = HashMap::new();
        let mut upstream: HashMap<ContextNodeKey, Vec<ContextNodeKey>> = HashMap::new();
        let mut topic_producers: HashMap<TopicKey, Vec<ContextNodeKey>> = HashMap::new();
        let mut topic_consumers: HashMap<TopicKey, Vec<ContextNodeKey>> = HashMap::new();

        for pipeline in &resolved.pipelines {
            let pipeline_key = PipelineKey::new(
                pipeline.pipeline_group_id.clone(),
                pipeline.pipeline_id.clone(),
            );
            let mut declarations_by_node = HashMap::new();
            for (node_id, node_config) in pipeline.pipeline.node_iter() {
                let node_key = (pipeline_key.clone(), node_id.clone());
                let declarations = self.node_context_declarations(
                    node_config.kind(),
                    node_config.r#type.as_ref(),
                    &node_config.config,
                )?;
                let mut node_consumers = OriginalNameConsumers::default();
                node_consumers.add_declarations(
                    &declarations,
                    Self::effective_propagation_policy(
                        node_config,
                        &pipeline.policies.transport_headers,
                    ),
                );
                if let Some(endpoint) = Self::topic_endpoint(node_config)? {
                    let (endpoints, topic) = match endpoint {
                        TopicEndpoint::Producer(topic) => (&mut topic_producers, topic),
                        TopicEndpoint::Consumer(topic) => (&mut topic_consumers, topic),
                    };
                    endpoints
                        .entry((pipeline.pipeline_group_id.clone(), topic))
                        .or_default()
                        .push(node_key.clone());
                }
                let _ = original_name_consumers.insert(node_key, node_consumers);
                let _ = declarations_by_node.insert(node_id.clone(), declarations);
            }

            for connection in pipeline.pipeline.connection_iter() {
                for consumer in connection.to_nodes() {
                    let consumer = (pipeline_key.clone(), consumer);
                    upstream.entry(consumer).or_default().extend(
                        connection
                            .from_nodes()
                            .into_iter()
                            .map(|producer| (pipeline_key.clone(), producer)),
                    );
                }
            }
            let _ = declarations.insert(pipeline_key, declarations_by_node);
        }

        connect_topic_edges(&mut upstream, &topic_producers, &topic_consumers);
        propagate_original_name_consumers(&mut original_name_consumers, &upstream);

        Ok(Arc::new(CompiledContextPolicy {
            declarations,
            original_name_consumers,
        }))
    }

    fn effective_propagation_policy<'a>(
        node: &'a NodeUserConfig,
        pipeline_policy: &'a Option<TransportHeadersPolicy>,
    ) -> Option<&'a HeaderPropagationPolicy> {
        node.header_propagation.as_ref().or_else(|| {
            pipeline_policy
                .as_ref()
                .map(|policy| &policy.header_propagation)
        })
    }

    fn topic_endpoint(node: &NodeUserConfig) -> Result<Option<TopicEndpoint>, EngineError> {
        let endpoint = match node.r#type.as_ref() {
            TOPIC_EXPORTER_URN => TopicEndpoint::Producer,
            TOPIC_RECEIVER_URN => TopicEndpoint::Consumer,
            _ => return Ok(None),
        };
        let topic = node.config.get("topic").cloned().ok_or_else(|| {
            EngineError::ConfigError(Box::new(Error::InvalidUserConfig {
                error: format!("topic node `{}` is missing `config.topic`", node.r#type),
            }))
        })?;
        serde_json::from_value(topic)
            .map(|topic| Some(endpoint(topic)))
            .map_err(|error| {
                EngineError::ConfigError(Box::new(Error::InvalidUserConfig {
                    error: format!(
                        "topic node `{}` has invalid `config.topic`: {error}",
                        node.r#type
                    ),
                }))
            })
    }

    /// Returns context declarations for a single node.
    fn node_context_declarations(
        &self,
        kind: NodeKind,
        urn: &str,
        config: &serde_json::Value,
    ) -> Result<NodeContextDeclarations, EngineError> {
        let missing_factory = || {
            EngineError::ConfigError(Box::new(Error::InvalidUserConfig {
                error: format!("node factory `{urn}` is not registered"),
            }))
        };
        let validate_config = match kind {
            NodeKind::Receiver => {
                self.get_receiver_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .validate_config
            }
            NodeKind::Processor => {
                self.get_processor_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .validate_config
            }
            NodeKind::Exporter => {
                self.get_exporter_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .validate_config
            }
        };
        validate_config(config).map_err(|error| EngineError::ConfigError(Box::new(error)))?;

        Ok(context_declaration_provider(urn)
            .map(|provider| {
                (provider.declarations)(config)
                    .map_err(|error| EngineError::ConfigError(Box::new(error)))
            })
            .transpose()?
            .unwrap_or_default())
    }
}

fn connect_topic_edges(
    upstream: &mut HashMap<ContextNodeKey, Vec<ContextNodeKey>>,
    producers: &HashMap<TopicKey, Vec<ContextNodeKey>>,
    consumers: &HashMap<TopicKey, Vec<ContextNodeKey>>,
) {
    for (topic, topic_producers) in producers {
        if let Some(topic_consumers) = consumers.get(topic) {
            for consumer in topic_consumers {
                upstream
                    .entry(consumer.clone())
                    .or_default()
                    .extend(topic_producers.iter().cloned());
            }
        }
    }
}

fn propagate_original_name_consumers(
    requirements: &mut HashMap<ContextNodeKey, OriginalNameConsumers>,
    upstream: &HashMap<ContextNodeKey, Vec<ContextNodeKey>>,
) {
    let mut pending: Vec<_> = requirements
        .iter()
        .filter_map(|(node, consumers)| (!consumers.is_empty()).then_some(node.clone()))
        .collect();
    while let Some(consumer_node) = pending.pop() {
        let Some(consumer_requirements) = requirements.get(&consumer_node).cloned() else {
            continue;
        };
        if let Some(producers) = upstream.get(&consumer_node) {
            for producer in producers {
                if requirements
                    .get_mut(producer)
                    .is_some_and(|requirements| requirements.extend(&consumer_requirements))
                {
                    pending.push(producer.clone());
                }
            }
        }
    }
}

fn context_declaration_provider(urn: &str) -> Option<ContextDeclarationProvider> {
    static PROVIDERS: OnceLock<HashMap<&'static str, ContextDeclarationProvider>> = OnceLock::new();
    PROVIDERS
        .get_or_init(|| {
            CONTEXT_DECLARATION_PROVIDERS
                .iter()
                .map(|provider| (provider.urn, *provider))
                .collect()
        })
        .get(urn)
        .copied()
}

#[cfg(test)]
mod preserve_original_name_tests {
    use super::*;
    use otel_arrow_dfe_config::transport_headers::TransportHeaders;
    use otel_arrow_dfe_config::transport_headers_policy::{CaptureDefaults, CaptureRule};

    const TEST_DECLARATION_URN: &str = "urn:test:processor:context_declaration";

    #[derive(serde::Deserialize)]
    struct TestDeclarationConfig {
        entry: ContextEntryName,
    }

    impl ConfigNodeContextDeclaration for TestDeclarationConfig {
        fn context_declarations(&self) -> NodeContextDeclarations {
            [
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries {
                        entries: vec![ContextEntrySelector {
                            name: self.entry.clone(),
                            read: ContextEntrySelectorForm::Value,
                        }]
                        .into_boxed_slice(),
                    },
                },
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::HeaderPropagationPolicy,
                },
            ]
            .into_iter()
            .collect()
        }
    }

    #[allow(unsafe_code)]
    #[distributed_slice(CONTEXT_DECLARATION_PROVIDERS)]
    static TEST_CONTEXT_DECLARATIONS: ContextDeclarationProvider =
        ContextDeclarationProvider::from_typed_config::<TestDeclarationConfig>(
            TEST_DECLARATION_URN,
        );

    fn pipeline(group: &str, name: &str) -> PipelineKey {
        PipelineKey::new(group.to_owned().into(), name.to_owned().into())
    }

    fn context_node(group: &str, pipeline_name: &str, node: &str) -> ContextNodeKey {
        (pipeline(group, pipeline_name), node.to_owned().into())
    }

    fn topic(group: &str, name: &str) -> TopicKey {
        (
            group.to_owned().into(),
            TopicName::parse(name).expect("valid test topic name"),
        )
    }

    fn original_consumers(names: &[&str]) -> OriginalNameConsumers {
        OriginalNameConsumers {
            all_original_names: false,
            entries: names.iter().map(|name| context_name(name)).collect(),
            propagation_policies: Vec::new(),
        }
    }

    fn context_name(name: &str) -> ContextEntryName {
        name.try_into().expect("valid test context entry name")
    }

    /// Scenario: a pipeline is absent from the compiled context policy.
    /// Guarantees: absent consumers do not cause conservative original-name retention.
    #[test]
    fn unknown_pipeline_has_no_original_name_consumers() {
        let pipeline = PipelineKey::new("group".into(), "pipeline".into());
        let node = ConfigNodeId::from("receiver");
        assert!(!CompiledContextPolicy::empty().consumes_original_name(
            &pipeline,
            &node,
            &context_name("tenant")
        ));
    }

    /// Scenario: capture rules contain distinct names and aliases sharing one stored name.
    /// Guarantees: compiled retention follows each effective normalized context entry.
    #[test]
    fn compiled_capture_policy_tracks_each_match_name() {
        let pipeline = PipelineKey::new("group".into(), "pipeline".into());
        let node = ConfigNodeId::from("receiver");
        let policy = CompiledContextPolicy {
            declarations: HashMap::new(),
            original_name_consumers: HashMap::from([(
                (pipeline.clone(), node.clone()),
                original_consumers(&["x-first", "canonical"]),
            )]),
        };
        let capture = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![
                CaptureRule {
                    match_names: vec![context_name("x-first"), context_name("x-second")],
                    store_as: None,
                    sensitive: false,
                    value_kind: None,
                },
                CaptureRule {
                    match_names: vec![context_name("x-alias-a"), context_name("x-alias-b")],
                    store_as: Some(context_name("canonical")),
                    sensitive: false,
                    value_kind: None,
                },
            ],
        );
        let capture = policy.compile_header_capture_policy(&pipeline, &node, capture);
        let mut headers = TransportHeaders::new();

        let _ = capture.capture_from_pairs(
            [
                ("X-First", b"first".as_slice()),
                ("X-Second", b"second".as_slice()),
                ("X-Alias-A", b"alias-a".as_slice()),
                ("X-Alias-B", b"alias-b".as_slice()),
            ]
            .into_iter(),
            &mut headers,
        );

        assert_eq!(headers.as_slice()[0].wire_name(), "X-First");
        assert_eq!(headers.as_slice()[1].wire_name(), "x-second");
        assert_eq!(headers.as_slice()[2].wire_name(), "X-Alias-A");
        assert_eq!(headers.as_slice()[3].wire_name(), "X-Alias-B");
    }

    /// Scenario: declarations consume values with different key-name forms.
    /// Guarantees: only OriginalKeyValue declarations request original-name retention.
    #[test]
    fn declarations_require_only_the_requested_name_form() {
        let declarations: NodeContextDeclarations = [
            ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        name: context_name("original"),
                        read: ContextEntrySelectorForm::OriginalKeyValue,
                    }]
                    .into_boxed_slice(),
                },
            },
            ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        name: context_name("value"),
                        read: ContextEntrySelectorForm::Value,
                    }]
                    .into_boxed_slice(),
                },
            },
        ]
        .into_iter()
        .collect();
        let mut consumers = OriginalNameConsumers::default();

        consumers.add_declarations(&declarations, None);

        assert!(consumers.consumes(&context_name("original")));
        assert!(!consumers.consumes(&context_name("value")));
    }

    /// Scenario: consumers select all context entries using normalized or original names.
    /// Guarantees: only the original-name selector retains every original wire name.
    #[test]
    fn all_entry_declarations_distinguish_name_forms() {
        let normalized: NodeContextDeclarations = [ContextDeclaration::Consumes {
            selector: ContextConsumerSelector::AllNormalized,
        }]
        .into_iter()
        .collect();
        let original: NodeContextDeclarations = [ContextDeclaration::Consumes {
            selector: ContextConsumerSelector::AllOriginal,
        }]
        .into_iter()
        .collect();
        let mut normalized_consumers = OriginalNameConsumers::default();
        normalized_consumers.add_declarations(&normalized, None);
        let mut original_consumers = OriginalNameConsumers::default();
        original_consumers.add_declarations(&original, None);

        assert!(!normalized_consumers.consumes(&context_name("tenant")));
        assert!(original_consumers.consumes(&context_name("tenant")));
    }

    /// Scenario: an all-original consumer is downstream of a receiver.
    /// Guarantees: the all-entry retention requirement propagates to the receiver.
    #[test]
    fn all_original_requirement_propagates_upstream() {
        let receiver = context_node("group", "pipeline", "receiver");
        let exporter = context_node("group", "pipeline", "exporter");
        let mut requirements = HashMap::from([
            (receiver.clone(), OriginalNameConsumers::default()),
            (
                exporter.clone(),
                OriginalNameConsumers {
                    all_original_names: true,
                    ..Default::default()
                },
            ),
        ]);
        let upstream = HashMap::from([(exporter, vec![receiver.clone()])]);

        propagate_original_name_consumers(&mut requirements, &upstream);

        assert!(requirements[&receiver].consumes(&context_name("tenant")));
    }

    /// Scenario: a preserve-name exporter is downstream of two topic hops.
    /// Guarantees: the requirement propagates to every upstream producing pipeline.
    #[test]
    fn preserve_requirement_propagates_across_topic_pipelines() {
        let source_receiver = context_node("group", "source", "receiver");
        let source_topic = context_node("group", "source", "topic");
        let relay_receiver = context_node("group", "relay", "receiver");
        let relay_topic = context_node("group", "relay", "topic");
        let sink_receiver = context_node("group", "sink", "receiver");
        let sink_exporter = context_node("group", "sink", "exporter");
        let unrelated = context_node("group", "unrelated", "receiver");
        let mut requirements = HashMap::from([
            (source_receiver.clone(), OriginalNameConsumers::default()),
            (source_topic.clone(), OriginalNameConsumers::default()),
            (relay_receiver.clone(), OriginalNameConsumers::default()),
            (relay_topic.clone(), OriginalNameConsumers::default()),
            (sink_receiver.clone(), OriginalNameConsumers::default()),
            (sink_exporter.clone(), original_consumers(&["tenant"])),
            (unrelated.clone(), OriginalNameConsumers::default()),
        ]);
        let mut upstream = HashMap::from([
            (source_topic.clone(), vec![source_receiver.clone()]),
            (relay_topic.clone(), vec![relay_receiver.clone()]),
            (sink_exporter, vec![sink_receiver.clone()]),
        ]);
        let producers = HashMap::from([
            (topic("group", "first"), vec![source_topic]),
            (topic("group", "second"), vec![relay_topic]),
        ]);
        let consumers = HashMap::from([
            (topic("group", "first"), vec![relay_receiver.clone()]),
            (topic("group", "second"), vec![sink_receiver.clone()]),
        ]);

        connect_topic_edges(&mut upstream, &producers, &consumers);
        propagate_original_name_consumers(&mut requirements, &upstream);

        let tenant = context_name("tenant");
        assert!(requirements[&source_receiver].consumes(&tenant));
        assert!(requirements[&relay_receiver].consumes(&tenant));
        assert!(requirements[&sink_receiver].consumes(&tenant));
        assert!(!requirements[&unrelated].consumes(&tenant));
    }

    /// Scenario: two receivers feed disconnected exporter branches in one pipeline.
    /// Guarantees: original-name requirements propagate only through reachable connections.
    #[test]
    fn preserve_requirement_respects_pipeline_branches() {
        let first_receiver = context_node("group", "pipeline", "first_receiver");
        let first_exporter = context_node("group", "pipeline", "first_exporter");
        let second_receiver = context_node("group", "pipeline", "second_receiver");
        let mut requirements = HashMap::from([
            (first_receiver.clone(), OriginalNameConsumers::default()),
            (
                first_exporter.clone(),
                original_consumers(&["x-request-id"]),
            ),
            (second_receiver.clone(), OriginalNameConsumers::default()),
        ]);
        let upstream = HashMap::from([(first_exporter, vec![first_receiver.clone()])]);

        propagate_original_name_consumers(&mut requirements, &upstream);

        let name = context_name("x-request-id");
        assert!(requirements[&first_receiver].consumes(&name));
        assert!(!requirements[&second_receiver].consumes(&name));
    }

    /// Scenario: two pipeline groups declare topics with the same name.
    /// Guarantees: preserve requirements do not cross group-local topic boundaries.
    #[test]
    fn preserve_requirement_respects_topic_group_scope() {
        let producer = context_node("first", "pipeline", "producer");
        let consumer = context_node("second", "pipeline", "consumer");
        let mut requirements = HashMap::from([
            (producer.clone(), OriginalNameConsumers::default()),
            (consumer.clone(), original_consumers(&["tenant"])),
        ]);
        let mut upstream = HashMap::new();
        let producers = HashMap::from([(topic("first", "shared"), vec![producer.clone()])]);
        let consumers = HashMap::from([(topic("second", "shared"), vec![consumer])]);

        connect_topic_edges(&mut upstream, &producers, &consumers);
        propagate_original_name_consumers(&mut requirements, &upstream);

        assert!(!requirements[&producer].consumes(&context_name("tenant")));
    }

    /// Scenario: resolved topic nodes carry their group-local topic in generic config.
    /// Guarantees: the compiler recognizes both topic exporter and receiver endpoints.
    #[test]
    fn topic_endpoints_are_read_from_resolved_node_config() {
        for urn in [TOPIC_EXPORTER_URN, TOPIC_RECEIVER_URN] {
            let node: NodeUserConfig = serde_json::from_value(serde_json::json!({
                "type": urn,
                "config": {"topic": "shared"}
            }))
            .expect("valid test node config");

            let endpoint = PipelineFactory::<()>::topic_endpoint(&node)
                .expect("valid topic endpoint")
                .expect("topic node");

            let topic = match endpoint {
                TopicEndpoint::Producer(topic) | TopicEndpoint::Consumer(topic) => topic,
            };
            assert_eq!(topic.as_str(), "shared");
        }
    }

    /// Scenario: a node declares that it reads according to header propagation policy.
    /// Guarantees: the effective policy is interpreted through the context declaration.
    #[test]
    fn header_propagation_policy_is_a_context_declaration() {
        let config = TestDeclarationConfig {
            entry: context_name("value"),
        };
        let policy: HeaderPropagationPolicy = serde_json::from_value(serde_json::json!({
            "default": {
                "selector": {
                    "type": "named",
                    "named": ["preserved"]
                },
                "name": "preserve"
            }
        }))
        .expect("valid propagation policy");
        let mut consumers = OriginalNameConsumers::default();

        consumers.add_declarations(&config.context_declarations(), Some(&policy));

        assert!(consumers.consumes(&context_name("preserved")));
        assert!(!consumers.consumes(&context_name("value")));
    }

    /// Scenario: a node checks its parsed config against its compiled declaration snapshot.
    /// Guarantees: matching declarations pass and changed declarations are rejected verbatim.
    #[test]
    fn parsed_config_declarations_are_validated_against_compiled_policy() {
        let pipeline = pipeline("group", "pipeline");
        let node: ConfigNodeId = "node".into();
        let matching: TestDeclarationConfig =
            serde_json::from_value(serde_json::json!({"entry": "expected"}))
                .expect("valid matching config");
        let changed: TestDeclarationConfig =
            serde_json::from_value(serde_json::json!({"entry": "changed"}))
                .expect("valid changed config");
        let policy = CompiledContextPolicy {
            declarations: HashMap::from([(
                pipeline.clone(),
                HashMap::from([(node.clone(), matching.context_declarations())]),
            )]),
            original_name_consumers: HashMap::new(),
        };

        assert!(
            policy
                .validate_node_declarations(&pipeline, &node, &matching.context_declarations(),)
                .is_ok()
        );
        assert!(
            policy
                .validate_node_declarations(&pipeline, &node, &changed.context_declarations())
                .is_err()
        );
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use otel_arrow_dfe_config::engine::OtelDataflowSpec;
//     use serde::Deserialize;

//     use crate::config::{ExporterConfig, ReceiverConfig};
//     use crate::context::PipelineContext;
//     use crate::exporter::ExporterWrapper;
//     use crate::node::NodeId;
//     use crate::receiver::ReceiverWrapper;
//     use crate::wiring_contract::WiringContract;
//     use crate::{ExporterFactory, ReceiverFactory};

//     fn test_receiver_create(
//         _pipeline: PipelineContext,
//         _node: NodeId,
//         _node_config: Arc<NodeUserConfig>,
//         _config: &ReceiverConfig,
//         _capabilities: &crate::capability::registry::Capabilities,
//     ) -> Result<ReceiverWrapper<()>, Error> {
//         panic!("test receiver must not be constructed")
//     }

//     fn test_exporter_create(
//         _pipeline: PipelineContext,
//         _node: NodeId,
//         _node_config: Arc<NodeUserConfig>,
//         _config: &ExporterConfig,
//         _capabilities: &crate::capability::registry::Capabilities,
//     ) -> Result<ExporterWrapper<()>, Error> {
//         panic!("test exporter must not be constructed")
//     }

//     fn test_factory() -> PipelineFactory<()> {
//         let receivers = Box::leak(Box::new([ReceiverFactory {
//             name: "urn:test:receiver:context",
//             create: test_receiver_create,
//             wiring_contract: WiringContract::UNRESTRICTED,
//             validate_config: otel_arrow_dfe_config::validation::no_config,
//         }]));
//         let exporters = Box::leak(Box::new([ExporterFactory {
//             name: "urn:test:exporter:context",
//             create: test_exporter_create,
//             wiring_contract: WiringContract::UNRESTRICTED,
//             validate_config: otel_arrow_dfe_config::validation::no_config,
//         }]));
//         PipelineFactory::new(receivers, &[], exporters, &[])
//     }

//     #[derive(Deserialize)]
//     struct TestDeclarationConfig {
//         #[serde(default = "default_test_entry")]
//         entry: ContextEntryName,
//     }

//     fn default_test_entry() -> ContextEntryName {
//         "default".into()
//     }

//     impl ContextDeclarationConfig for TestDeclarationConfig {
//         fn context_declarations(&self) -> Result<Vec<ContextDeclaration>, Error> {
//             Ok(vec![ContextDeclaration::Produces {
//                 access: ContextAccessId::new(0),
//                 entry: self.entry.clone(),
//             }])
//         }
//     }

//     /// Scenario: A typed provider receives valid component configuration.
//     /// Guarantees: parsing and declaration extraction use the registered config type.
//     #[test]
//     fn typed_provider_parses_registered_config() {
//         let provider =
//             ContextDeclarationProvider::from_typed_config::<TestDeclarationConfig>("urn:test");
//         let config = serde_json::json!({"entry": "X-Test"});
//         let decls = (provider.declarations)(&config).unwrap();
//         assert_eq!(decls.len(), 1);
//         match &decls[0] {
//             ContextDeclaration::Produces { entry, .. } => {
//                 assert_eq!(entry.as_str(), "x-test");
//             }
//             other => panic!("unexpected declaration: {other:?}"),
//         }
//     }

//     /// Scenario: capture and component producers span one resolved configuration.
//     /// Guarantees: one sorted register layout backs every compiled receiver schema.
//     #[test]
//     fn compiles_one_global_register_layout() {
//         #[allow(unsafe_code)]
//         #[distributed_slice(CONTEXT_DECLARATION_PROVIDERS)]
//         static TEST_RECEIVER_CONTEXT_DECLARATIONS: ContextDeclarationProvider =
//             ContextDeclarationProvider::from_config(
//                 "urn:test:receiver:context",
//                 test_receiver_declarations,
//             );

//         fn test_receiver_declarations(
//             _config: &serde_json::Value,
//         ) -> Result<Vec<ContextDeclaration>, Error> {
//             Ok(vec![
//                 ContextDeclaration::Produces {
//                     access: ContextAccessId::new(0),
//                     entry: "zeta".into(),
//                 },
//                 ContextDeclaration::Produces {
//                     access: ContextAccessId::new(1),
//                     entry: "beta".into(),
//                 },
//             ])
//         }

//         let mut spec = OtelDataflowSpec::from_yaml(
//             r#"
// version: otel_dataflow/v1
// groups:
//   group:
//     pipelines:
//       pipeline:
//         policies:
//           transport_headers:
//             header_capture:
//               headers:
//                 - match_names: [x-alpha]
//                   store_as: alpha
//             header_propagation:
//               default:
//                 selector:
//                   type: all_captured
//         nodes:
//           source:
//             type: urn:test:receiver:context
//             config: {}
//           sink:
//             type: urn:test:exporter:context
//             config: {}
//         connections:
//           - from: source
//             to: sink
// "#,
//         )
//         .expect("valid config");
//         spec.engine.observability.pipeline.nodes = Default::default();
//         spec.engine.observability.pipeline.connections.clear();
//         let policy = test_factory()
//             .compile_context_policy(&spec.resolve())
//             .expect("compiled policy");

//         assert_eq!(
//             policy
//                 .compiled_context
//                 .resolve("alpha")
//                 .expect("alpha")
//                 .index(),
//             0
//         );
//         assert_eq!(
//             policy
//                 .compiled_context
//                 .resolve("beta")
//                 .expect("beta")
//                 .index(),
//             1
//         );
//         assert_eq!(
//             policy
//                 .compiled_context
//                 .resolve("zeta")
//                 .expect("zeta")
//                 .index(),
//             2
//         );
//         let pipeline = PipelineKey::new("group".into(), "pipeline".into());
//         let capture = &policy.receiver_capture[&pipeline]["source"];
//         assert!(Arc::ptr_eq(
//             capture.schema().register_layout(),
//             policy.compiled_context.register_layout()
//         ));
//         assert_eq!(
//             capture
//                 .match_header("x-alpha")
//                 .expect("x-alpha capture")
//                 .schema_item
//                 .retention,
//             ContextNameRetention::Observed
//         );
//     }

//     /// Scenario: A typed provider receives configuration missing an optional field.
//     /// Guarantees: the registered config type's Serde defaults are applied.
//     #[test]
//     fn typed_provider_applies_config_defaults() {
//         let provider =
//             ContextDeclarationProvider::from_typed_config::<TestDeclarationConfig>("urn:test");
//         let decls = (provider.declarations)(&serde_json::json!({})).unwrap();

//         assert_eq!(
//             decls,
//             vec![ContextDeclaration::Produces {
//                 access: ContextAccessId::new(0),
//                 entry: "default".into(),
//             }]
//         );
//     }

//     /// Scenario: policy compilation indexes declarations by pipeline and node.
//     /// Guarantees: each node retains only its declarations.
//     #[test]
//     fn compiled_policy_indexes_existing_configuration_ids() {
//         let pipeline = PipelineKey::new("group".into(), "pipeline".into());
//         let node: ConfigNodeId = "source".into();
//         let declaration = ContextDeclaration::Produces {
//             access: ContextAccessId::new(0),
//             entry: "tenant".into(),
//         };
//         let mut compiler = ContextCompiler::new();
//         let _ = compiler
//             .declare(ContextRegisterRequirement::new("tenant"))
//             .expect("tenant");
//         let policy = CompiledContextPolicy {
//             generation: ContextPolicyGeneration::default(),
//             compiled_context: compiler.finish(),
//             receiver_capture: HashMap::new(),
//             declarations: HashMap::from([(
//                 pipeline.clone(),
//                 HashMap::from([(node.clone(), vec![declaration.clone()].into_boxed_slice())]),
//             )]),
//         };

//         assert_eq!(
//             policy.declarations[&pipeline][&node].as_ref(),
//             [declaration].as_slice()
//         );
//         assert!(!policy.declarations[&pipeline].contains_key("other"));
//     }

//     /// Scenario: Two access IDs select the same register.
//     /// Guarantees: the declarations remain distinct.
//     #[test]
//     fn access_id_distinguishes_consumers() {
//         let selector = ContextConsumerSelector::Entries {
//             entries: vec!["x-topic".into()].into_boxed_slice(),
//         };
//         let first = ContextDeclaration::Consumes {
//             access: ContextAccessId::new(0),
//             selector: selector.clone(),
//         };
//         let second = ContextDeclaration::Consumes {
//             access: ContextAccessId::new(1),
//             selector,
//         };
//         assert_ne!(first, second);
//     }

//     /// Scenario: Generic selectors represent ordered and all-register reads.
//     /// Guarantees: selector equality preserves register selection and order.
//     #[test]
//     fn selectors_preserve_generic_read_contracts() {
//         assert_ne!(
//             ContextReadSelector::Entries {
//                 entries: vec!["tenant".into()].into_boxed_slice(),
//             },
//             ContextReadSelector::Entries {
//                 entries: vec!["region".into()].into_boxed_slice(),
//             }
//         );
//         assert_ne!(
//             ContextReadSelector::Entries {
//                 entries: vec!["tenant".into(), "region".into()].into_boxed_slice(),
//             },
//             ContextReadSelector::Entries {
//                 entries: vec!["region".into(), "tenant".into()].into_boxed_slice(),
//             },
//         );
//     }

//     /// Scenario: Producer inputs are unordered and use mixed-case names.
//     /// Guarantees: finished declarations normalize names and assign sorted IDs.
//     #[test]
//     fn builder_normalizes_and_orders_producers() {
//         let mut builder = ContextDeclarationsBuilder::new();
//         builder
//             .produce(ContextEntryName::parse("X-Tenant-Id").unwrap())
//             .unwrap();
//         builder.produce("a-first".into()).unwrap();

//         assert_eq!(
//             builder.finish(),
//             vec![
//                 ContextDeclaration::Produces {
//                     access: ContextAccessId::new(0),
//                     entry: "a-first".into(),
//                 },
//                 ContextDeclaration::Produces {
//                     access: ContextAccessId::new(1),
//                     entry: "x-tenant-id".into(),
//                 },
//             ]
//         );
//     }

//     /// Scenario: Producer inputs differ only by logical-name casing.
//     /// Guarantees: configuration compilation rejects ambiguous register names.
//     #[test]
//     fn builder_rejects_duplicate_normalized_producers() {
//         let mut builder = ContextDeclarationsBuilder::new();
//         builder
//             .produce(ContextEntryName::parse("X-Tenant-Id").unwrap())
//             .unwrap();

//         assert!(matches!(
//             builder.produce("x-tenant-id".into()),
//             Err(Error::InvalidUserConfig { .. })
//         ));
//     }
// }
