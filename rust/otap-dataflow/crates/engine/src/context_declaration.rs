// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Collects and compiles context declarations before runtime construction.
//!
//! # Type map
//!
//! - `ContextEntrySelector`: one context entry name and the representation a consumer requests.
//! - `ContextEntrySelectorForm`: the value, stored-name, or original-name representation.
//! - `ContextConsumerSelector`: a named-entry or all-entry consumer selection.
//! - `ContextDeclaration`: one component or engine declaration of context behavior.
//! - `ContextDeclarationProvider`: a component factory's declaration callback registration.
//! - `ContextDeclarationFn`: the signature implemented by declaration callbacks.
//! - `ConfigNodeContextDeclaration`: typed component configs that derive and validate declarations.
//! - `NodeContextDeclarations`: a sorted, deduplicated declaration set for one node.
//! - `CompiledContextBindings`: compiled node bindings for every pipeline in a configuration.
//! - `CompiledNodeBindings`: component declarations, transport-header behavior, and authorized
//!   identity capture for one node.
//! - `ContextDeclarationsByPipeline`: declarations indexed by pipeline and node.
//! - `ContextRuntimeRequirements`: immutable engine-lifetime requirements for binding preparation.
//! - `OriginalNameRetention`: the default and per-name original-header retention disposition.
//! - `PreparedContext`: requirements and bindings prepared from one resolved configuration.
//! - `TestDeclarationConfig`: test-only typed configuration used to verify declaration matching.

use crate::PipelineFactory;
use crate::error::Error as EngineError;
use otel_arrow_dfe_config::authorized_identity_policy::AuthorizedIdentityPolicy;
use otel_arrow_dfe_config::context_bindings::{
    CompiledHeaderPropagationPolicy, ContextFieldId, ContextLayout, ContextPrimitive,
};
use otel_arrow_dfe_config::context_policy::ContextEntryDeclaration;
use otel_arrow_dfe_config::engine::ResolvedOtelDataflowSpec;
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::node::{NodeKind, NodeUserConfig};
use otel_arrow_dfe_config::transport_headers_policy::{
    CompiledHeaderCapturePolicy, HeaderCapturePolicy, HeaderPropagationPolicy,
    TransportHeadersPolicy,
};
use otel_arrow_dfe_config::{ContextEntryName, NodeId as ConfigNodeId, PipelineKey};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

/// A context entry and its requested representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContextEntrySelector {
    /// Configured entry name.
    pub name: ContextEntryName,
    /// Requested representation.
    pub form: ContextEntrySelectorForm,
}

/// Context entry representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextEntrySelectorForm {
    /// Value only.
    Value,
    /// Stored name and value, preserving configured spelling.
    StoredKeyValue,
    /// Original name and value.
    OriginalKeyValue,
}

/// Context entries read by a consumer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextConsumerSelector {
    /// Selects named context entries in order.
    Entries {
        /// Entries to read.
        entries: Box<[ContextEntrySelector]>,
    },
    /// Selects every context entry using its stored name.
    AllStored,
}

/// A node's declared context behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextDeclaration {
    /// Declares a context entry produced by the node.
    Produces {
        /// Produced entry name.
        entry: ContextEntryName,
    },
    /// Declares context reads.
    Consumes {
        /// Entries to read.
        selector: ContextConsumerSelector,
    },
    /// Declares the receiver's header capture policy.
    HeaderCapture {
        /// Resolved capture policy.
        policy: HeaderCapturePolicy,
    },
    /// Declares the exporter's header propagation policy.
    HeaderPropagation {
        /// Resolved propagation policy.
        policy: HeaderPropagationPolicy,
    },
    /// Declares the receiver's authorized identity claim projection policy.
    AuthorizedIdentityCapture {
        /// Resolved authorized identity policy.
        policy: AuthorizedIdentityPolicy,
    },
}

impl ContextDeclaration {
    fn is_component_declaration(&self) -> bool {
        matches!(self, Self::Produces { .. } | Self::Consumes { .. })
    }
}

/// Derives context declarations from component configuration.
#[derive(Clone, Copy)]
pub struct ContextDeclarationProvider {
    /// Declaration callback.
    pub declarations: ContextDeclarationFn,
}

/// Derives deterministic context declarations from node configuration.
pub type ContextDeclarationFn = fn(&serde_json::Value) -> Result<NodeContextDeclarations, Error>;

/// Context declarations derived from typed node configuration.
pub trait ConfigNodeContextDeclaration: serde::de::DeserializeOwned {
    /// Declares the context reads and writes for this configuration.
    fn context_declarations(&self) -> NodeContextDeclarations;

    /// Checks these declarations against the compiled bindings.
    fn validate_context_declarations(
        &self,
        pipeline_ctx: &crate::context::PipelineContext,
    ) -> Result<(), Error> {
        pipeline_ctx
            .compiled_context_bindings()
            .validate_node_declarations(
                &pipeline_ctx.pipeline_key(),
                &pipeline_ctx.node_id(),
                &self.context_declarations(),
            )
    }
}

/// Sorted, unique context declarations.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeContextDeclarations {
    /// Sorted and deduplicated declarations.
    declarations: Box<[ContextDeclaration]>,
}

impl FromIterator<ContextDeclaration> for NodeContextDeclarations {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = ContextDeclaration>,
    {
        let mut uniq: Vec<_> = iter.into_iter().collect();
        uniq.sort();
        uniq.dedup();
        Self {
            declarations: uniq.into_boxed_slice(),
        }
    }
}

impl IntoIterator for NodeContextDeclarations {
    type Item = ContextDeclaration;
    type IntoIter = std::vec::IntoIter<ContextDeclaration>;

    fn into_iter(self) -> Self::IntoIter {
        self.declarations.into_vec().into_iter()
    }
}

impl NodeContextDeclarations {
    /// Iterates over declarations in sorted order.
    pub fn iter(&self) -> impl Iterator<Item = &ContextDeclaration> {
        self.declarations.iter()
    }

    /// Returns whether the declaration set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Returns the declaration count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.declarations.len()
    }
}

impl ContextDeclarationProvider {
    /// Creates a declaration provider for a configuration type.
    #[must_use]
    pub const fn from_typed_config<T>() -> Self
    where
        T: ConfigNodeContextDeclaration,
    {
        Self {
            declarations: typed_context_declarations::<T>,
        }
    }
}

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

/// Context bindings compiled for all pipelines in one resolved configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledContextBindings {
    /// Compiled node bindings indexed first by pipeline, then by node.
    by_pipeline: HashMap<PipelineKey, HashMap<ConfigNodeId, CompiledNodeBindings>>,
    /// Immutable field and composite-entry layout shared by all bindings.
    layout: Arc<ContextLayout>,
}

/// Declarations and context policies compiled for one node.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledNodeBindings {
    component_declarations: NodeContextDeclarations,
    header_capture: Option<CompiledHeaderCapturePolicy>,
    header_propagation: Option<CompiledHeaderPropagationPolicy>,
    authorized_identity_capture: Option<AuthorizedIdentityPolicy>,
}

#[derive(Default)]
/// Complete compiler input containing node behavior and scoped entry definitions.
pub struct DeclaredContextPolicy {
    /// Component and wrapper declarations keyed by their owning node.
    pub nodes: HashMap<PipelineKey, HashMap<ConfigNodeId, NodeContextDeclarations>>,
    /// Entry definitions with their declaring scopes.
    pub entries: BTreeSet<ContextEntryDeclaration>,
}

type CompiledPropagationByPipeline =
    HashMap<PipelineKey, HashMap<ContextDeclaration, CompiledHeaderPropagationPolicy>>;

/// Immutable engine-wide requirements used to compile context bindings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextRuntimeRequirements {
    original_names: BTreeSet<ContextPrimitive>,
}

/// Requirements and node bindings prepared from one resolved configuration.
#[derive(Debug, Clone)]
pub struct PreparedContext {
    /// Runtime requirements derived from these declarations.
    pub runtime_requirements: ContextRuntimeRequirements,
    /// Node bindings compiled using the selected engine requirements.
    pub bindings: Arc<CompiledContextBindings>,
}

struct PreparedDeclarations {
    declarations: DeclaredContextPolicy,
    layout: Arc<ContextLayout>,
    propagation: CompiledPropagationByPipeline,
    runtime_requirements: ContextRuntimeRequirements,
}

impl ContextRuntimeRequirements {
    /// Returns whether the installed requirements satisfy every candidate requirement.
    #[must_use]
    pub fn can_satisfy(&self, candidate: &Self) -> bool {
        self.original_names.is_superset(&candidate.original_names)
    }

    fn preserves_original_field(&self, layout: &ContextLayout, field: ContextFieldId) -> bool {
        self.original_names.contains(layout.primitive(field))
    }
}

impl PreparedDeclarations {
    fn prepare(declarations: DeclaredContextPolicy) -> Result<Self, Error> {
        let mut primitives = BTreeSet::new();
        for declaration in declarations
            .nodes
            .values()
            .flat_map(HashMap::values)
            .flat_map(NodeContextDeclarations::iter)
        {
            match declaration {
                ContextDeclaration::Produces { entry } => {
                    let _ = primitives.insert(ContextPrimitive::standalone(entry.clone()));
                }
                ContextDeclaration::HeaderCapture { policy } => {
                    primitives.extend(policy.context_primitives());
                }
                ContextDeclaration::Consumes { .. }
                | ContextDeclaration::HeaderPropagation { .. }
                | ContextDeclaration::AuthorizedIdentityCapture { .. } => {}
            }
        }
        let layout = ContextLayout::compile(primitives, declarations.entries.clone())?;
        let mut original_names = BTreeSet::new();
        let mut propagation = HashMap::new();
        for (pipeline, nodes) in &declarations.nodes {
            let mut pipeline_propagation = HashMap::new();
            for declaration in nodes.values().flat_map(NodeContextDeclarations::iter) {
                match declaration {
                    ContextDeclaration::HeaderPropagation { policy } => {
                        let compiled = policy.compile(layout.clone(), pipeline)?;
                        original_names.extend(
                            compiled
                                .original_name_fields()
                                .map(|field| layout.primitive(field).clone()),
                        );
                        let _ = pipeline_propagation.insert(declaration.clone(), compiled);
                    }
                    ContextDeclaration::Consumes {
                        selector: ContextConsumerSelector::Entries { entries },
                    } => {
                        for entry in entries {
                            if entry.form == ContextEntrySelectorForm::OriginalKeyValue {
                                let reference = entry.name.as_str().try_into()?;
                                let (_, fields) = layout.resolve(&reference, pipeline)?;
                                original_names.extend(
                                    fields
                                        .into_iter()
                                        .map(|field| layout.primitive(field).clone()),
                                );
                            }
                        }
                    }
                    ContextDeclaration::Produces { .. }
                    | ContextDeclaration::Consumes {
                        selector: ContextConsumerSelector::AllStored,
                    }
                    | ContextDeclaration::HeaderCapture { .. }
                    | ContextDeclaration::AuthorizedIdentityCapture { .. } => {}
                }
            }
            let _ = propagation.insert(pipeline.clone(), pipeline_propagation);
        }

        Ok(Self {
            declarations,
            layout,
            propagation,
            runtime_requirements: ContextRuntimeRequirements { original_names },
        })
    }

    fn compile(
        self,
        installed_requirements: &ContextRuntimeRequirements,
    ) -> Result<CompiledContextBindings, Error> {
        let by_pipeline = self
            .declarations
            .nodes
            .into_iter()
            .map(|(pipeline, nodes)| {
                let propagation = &self.propagation[&pipeline];
                let nodes = nodes
                    .into_iter()
                    .map(|(node, declarations)| {
                        Ok((
                            node,
                            CompiledNodeBindings::compile(
                                declarations,
                                installed_requirements,
                                &self.layout,
                                &pipeline,
                                propagation,
                            )?,
                        ))
                    })
                    .collect::<Result<_, Error>>()?;
                Ok((pipeline, nodes))
            })
            .collect::<Result<_, Error>>()?;
        Ok(CompiledContextBindings {
            by_pipeline,
            layout: self.layout,
        })
    }
}

impl CompiledNodeBindings {
    fn compile(
        declarations: NodeContextDeclarations,
        requirements: &ContextRuntimeRequirements,
        layout: &Arc<ContextLayout>,
        pipeline: &PipelineKey,
        propagation: &HashMap<ContextDeclaration, CompiledHeaderPropagationPolicy>,
    ) -> Result<Self, Error> {
        let mut component_declarations = Vec::new();
        let mut header_capture = None;
        let mut header_propagation = None;
        let mut authorized_identity_capture = None;
        for declaration in declarations {
            match &declaration {
                ContextDeclaration::Produces { .. } | ContextDeclaration::Consumes { .. } => {
                    component_declarations.push(declaration);
                }
                ContextDeclaration::HeaderCapture { policy } => {
                    header_capture = Some(policy.clone().compile_bound(
                        layout.clone(),
                        pipeline,
                        |field| requirements.preserves_original_field(layout, field),
                    )?);
                }
                ContextDeclaration::HeaderPropagation { .. } => {
                    header_propagation = Some(propagation[&declaration].clone());
                }
                ContextDeclaration::AuthorizedIdentityCapture { policy } => {
                    authorized_identity_capture = Some(policy.clone());
                }
            }
        }
        Ok(Self {
            component_declarations: component_declarations.into_iter().collect(),
            header_capture,
            header_propagation,
            authorized_identity_capture,
        })
    }

    fn is_empty(&self) -> bool {
        self.component_declarations.is_empty()
            && self.header_capture.is_none()
            && self.header_propagation.is_none()
            && self.authorized_identity_capture.is_none()
    }
}

impl CompiledContextBindings {
    /// Compiles complete declarations using their own runtime requirements.
    pub fn compile(declarations: DeclaredContextPolicy) -> Result<Self, Error> {
        let prepared = PreparedDeclarations::prepare(declarations)?;
        let requirements = prepared.runtime_requirements.clone();
        prepared.compile(&requirements)
    }

    /// Creates an empty binding set. Node validation always fails.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            by_pipeline: HashMap::new(),
            layout: Arc::new(ContextLayout::default()),
        }
    }

    /// Returns the immutable layout used by typed component bindings.
    #[must_use]
    pub fn layout(&self) -> &Arc<ContextLayout> {
        &self.layout
    }

    /// Returns the node's compiled header capture policy.
    #[must_use]
    pub fn header_capture_policy(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
    ) -> Option<&CompiledHeaderCapturePolicy> {
        self.by_pipeline
            .get(pipeline)?
            .get(node)?
            .header_capture
            .as_ref()
    }

    /// Returns the node's compiled header propagation policy.
    #[must_use]
    pub fn header_propagation_policy(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
    ) -> Option<&CompiledHeaderPropagationPolicy> {
        self.by_pipeline
            .get(pipeline)?
            .get(node)?
            .header_propagation
            .as_ref()
    }

    /// Returns the node's authorized identity claim projection policy.
    pub(crate) fn authorized_identity_policy(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
    ) -> Option<&AuthorizedIdentityPolicy> {
        self.by_pipeline
            .get(pipeline)?
            .get(node)?
            .authorized_identity_capture
            .as_ref()
    }

    /// Returns whether two binding sets contain identical non-empty bindings for one pipeline.
    #[must_use]
    pub fn pipeline_bindings_match(&self, other: &Self, pipeline: &PipelineKey) -> bool {
        let current = self.by_pipeline.get(pipeline);
        let candidate = other.by_pipeline.get(pipeline);
        let current_binding_count = current
            .into_iter()
            .flat_map(|nodes| nodes.values())
            .filter(|node| !node.is_empty())
            .count();
        let candidate_binding_count = candidate
            .into_iter()
            .flat_map(|nodes| nodes.values())
            .filter(|node| !node.is_empty())
            .count();

        current_binding_count == candidate_binding_count
            && current
                .into_iter()
                .flat_map(|nodes| nodes.iter())
                .all(|(node_id, node)| {
                    node.is_empty()
                        || candidate
                            .and_then(|nodes| nodes.get(node_id))
                            .is_some_and(|candidate_node| candidate_node == node)
                })
    }

    /// Checks component declarations against this node's compiled bindings.
    pub fn validate_node_declarations(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
        declarations: &NodeContextDeclarations,
    ) -> Result<(), Error> {
        match self
            .by_pipeline
            .get(pipeline)
            .and_then(|nodes| nodes.get(node))
        {
            Some(expected) if expected.component_declarations == *declarations => Ok(()),
            _ => Err(Error::UnrecognizedContextDeclaration {}),
        }
    }
}

impl<PData: 'static + Clone + std::fmt::Debug> PipelineFactory<PData> {
    /// Compiles startup requirements and node bindings from the same declarations.
    pub fn compile_initial_context(
        &self,
        resolved: &ResolvedOtelDataflowSpec,
    ) -> Result<PreparedContext, EngineError> {
        let prepared = PreparedDeclarations::prepare(self.context_declarations(resolved)?)
            .map_err(|error| EngineError::ConfigError(Box::new(error)))?;
        let runtime_requirements = prepared.runtime_requirements.clone();
        let bindings = Arc::new(
            prepared
                .compile(&runtime_requirements)
                .map_err(|error| EngineError::ConfigError(Box::new(error)))?,
        );
        Ok(PreparedContext {
            runtime_requirements,
            bindings,
        })
    }

    /// Compiles candidate bindings using the immutable installed requirements.
    pub fn compile_candidate_context(
        &self,
        resolved: &ResolvedOtelDataflowSpec,
        installed_requirements: &ContextRuntimeRequirements,
    ) -> Result<PreparedContext, EngineError> {
        let prepared = PreparedDeclarations::prepare(self.context_declarations(resolved)?)
            .map_err(|error| EngineError::ConfigError(Box::new(error)))?;
        let runtime_requirements = prepared.runtime_requirements.clone();
        let bindings = Arc::new(
            prepared
                .compile(installed_requirements)
                .map_err(|error| EngineError::ConfigError(Box::new(error)))?,
        );
        Ok(PreparedContext {
            runtime_requirements,
            bindings,
        })
    }

    fn context_declarations(
        &self,
        resolved: &ResolvedOtelDataflowSpec,
    ) -> Result<DeclaredContextPolicy, EngineError> {
        let mut declarations = DeclaredContextPolicy::default();

        for pipeline in &resolved.pipelines {
            declarations
                .entries
                .extend(pipeline.policies.context.iter().cloned());
            let pipeline_key = PipelineKey::new(
                pipeline.pipeline_group_id.clone(),
                pipeline.pipeline_id.clone(),
            );
            let mut declarations_by_node = HashMap::new();
            for (node_id, node_config) in pipeline.pipeline.node_iter() {
                let component_declarations = self.node_context_declarations(
                    node_config.kind(),
                    node_config.r#type.as_ref(),
                    &node_config.config,
                )?;
                let wrapper_declarations = Self::wrapper_context_declarations(
                    node_config,
                    &pipeline.policies.transport_headers,
                    &pipeline.policies.authorized_identity,
                );
                let declarations = component_declarations
                    .into_iter()
                    .chain(wrapper_declarations)
                    .collect();
                let _ = declarations_by_node.insert(node_id.clone(), declarations);
            }
            let _ = declarations
                .nodes
                .insert(pipeline_key, declarations_by_node);
        }

        Ok(declarations)
    }

    fn wrapper_context_declarations(
        node: &NodeUserConfig,
        pipeline_policy: &Option<TransportHeadersPolicy>,
        authorized_identity: &Option<AuthorizedIdentityPolicy>,
    ) -> NodeContextDeclarations {
        match node.kind() {
            NodeKind::Receiver => node
                .header_capture
                .as_ref()
                .or_else(|| {
                    pipeline_policy
                        .as_ref()
                        .map(|policy| &policy.header_capture)
                })
                .cloned()
                .map(|policy| ContextDeclaration::HeaderCapture { policy })
                .into_iter()
                .chain(
                    authorized_identity
                        .as_ref()
                        .filter(|policy| !policy.is_empty())
                        .cloned()
                        .map(|policy| ContextDeclaration::AuthorizedIdentityCapture { policy }),
                )
                .collect(),
            NodeKind::Exporter => node
                .header_propagation
                .as_ref()
                .or_else(|| {
                    pipeline_policy
                        .as_ref()
                        .map(|policy| &policy.header_propagation)
                })
                .cloned()
                .map(|policy| ContextDeclaration::HeaderPropagation { policy })
                .into_iter()
                .collect(),
            NodeKind::Processor => NodeContextDeclarations::default(),
        }
    }

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
        let (validate_config, context_declarations) = match kind {
            NodeKind::Receiver => {
                let factory = self
                    .get_receiver_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?;
                (factory.validate_config, factory.context_declarations)
            }
            NodeKind::Processor => {
                let factory = self
                    .get_processor_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?;
                (factory.validate_config, factory.context_declarations)
            }
            NodeKind::Exporter => {
                let factory = self
                    .get_exporter_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?;
                (factory.validate_config, factory.context_declarations)
            }
        };
        // Validate before collecting declarations. Nodes are not constructed yet.
        validate_config(config).map_err(|error| EngineError::ConfigError(Box::new(error)))?;

        let declarations = context_declarations
            .map(|provider| {
                (provider.declarations)(config)
                    .map_err(|error| EngineError::ConfigError(Box::new(error)))
            })
            .transpose()?
            .unwrap_or_default();
        // Capture and propagation declarations belong to the engine.
        if let Some(declaration) = declarations
            .iter()
            .find(|declaration| !declaration.is_component_declaration())
        {
            return Err(EngineError::ConfigError(Box::new(
                Error::InvalidUserConfig {
                    error: format!(
                        "node factory `{urn}` returned engine-owned context declaration \
                         `{declaration:?}`"
                    ),
                },
            )));
        }
        Ok(declarations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::transport_headers::TransportHeaders;
    use otel_arrow_dfe_config::transport_headers_policy::{CaptureDefaults, CaptureRule};

    /// Typed component configuration used to exercise declaration validation.
    #[derive(serde::Deserialize)]
    struct TestDeclarationConfig {
        entry: ContextEntryName,
    }

    impl ConfigNodeContextDeclaration for TestDeclarationConfig {
        fn context_declarations(&self) -> NodeContextDeclarations {
            [ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        name: self.entry.clone(),
                        form: ContextEntrySelectorForm::Value,
                    }]
                    .into_boxed_slice(),
                },
            }]
            .into_iter()
            .collect()
        }
    }

    fn pipeline(group: &str, name: &str) -> PipelineKey {
        PipelineKey::new(group.to_owned().into(), name.to_owned().into())
    }

    fn context_name(name: &str) -> ContextEntryName {
        name.try_into().expect("valid test context entry name")
    }

    fn declared_policy(effective: NodeContextDeclarations) -> DeclaredContextPolicy {
        DeclaredContextPolicy {
            nodes: HashMap::from([(
                pipeline("group", "pipeline"),
                HashMap::from([(ConfigNodeId::from("node"), effective)]),
            )]),
            entries: BTreeSet::new(),
        }
    }

    fn compiled_bindings(effective: NodeContextDeclarations) -> CompiledContextBindings {
        let prepared =
            PreparedDeclarations::prepare(declared_policy(effective)).expect("valid declarations");
        let requirements = prepared.runtime_requirements.clone();
        prepared.compile(&requirements).expect("valid bindings")
    }

    fn context_runtime_requirements(
        effective: NodeContextDeclarations,
    ) -> ContextRuntimeRequirements {
        PreparedDeclarations::prepare(declared_policy(effective))
            .expect("valid declarations")
            .runtime_requirements
    }

    /// Scenario: capture aliases share a stored name.
    /// Guarantees: each stored name determines original-name retention.
    #[test]
    fn compiled_capture_policy_tracks_each_match_name() {
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
        let bindings = compiled_bindings(
            [
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries {
                        entries: ["x-first", "canonical"]
                            .map(|name| ContextEntrySelector {
                                name: context_name(name),
                                form: ContextEntrySelectorForm::OriginalKeyValue,
                            })
                            .into(),
                    },
                },
                ContextDeclaration::HeaderCapture { policy: capture },
            ]
            .into_iter()
            .collect(),
        );
        let capture = bindings
            .header_capture_policy(&pipeline("group", "pipeline"), &ConfigNodeId::from("node"))
            .expect("compiled capture policy");
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

    /// Scenario: consumers request different name representations.
    /// Guarantees: only `OriginalKeyValue` requires original names.
    #[test]
    fn declarations_require_only_the_requested_name_form() {
        let declarations: NodeContextDeclarations = [
            ContextDeclaration::HeaderCapture {
                policy: HeaderCapturePolicy::new(
                    CaptureDefaults::default(),
                    vec![CaptureRule {
                        match_names: vec![context_name("original"), context_name("value")],
                        store_as: None,
                        sensitive: false,
                        value_kind: None,
                    }],
                ),
            },
            ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        name: context_name("original"),
                        form: ContextEntrySelectorForm::OriginalKeyValue,
                    }]
                    .into_boxed_slice(),
                },
            },
            ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        name: context_name("value"),
                        form: ContextEntrySelectorForm::Value,
                    }]
                    .into_boxed_slice(),
                },
            },
        ]
        .into_iter()
        .collect();
        let compiled = compiled_bindings(declarations);
        let source = compiled
            .header_capture_policy(&pipeline("group", "pipeline"), &"node".into())
            .unwrap();
        let mut headers = TransportHeaders::new();
        let _ = source.capture_from_pairs(
            [
                ("Original", b"one".as_slice()),
                ("Value", b"two".as_slice()),
            ]
            .into_iter(),
            &mut headers,
        );
        assert_eq!(
            headers.as_slice()[0].value.original_name.as_deref(),
            Some("Original")
        );
        assert!(headers.as_slice()[1].value.original_name.is_none());
    }

    /// Scenario: live declarations add and remove original-name consumers.
    /// Guarantees: installed requirements allow subsets but reject unsupported names and defaults.
    #[test]
    fn installed_requirements_support_only_available_original_names() {
        let requirements = |names: &[&str]| ContextRuntimeRequirements {
            original_names: names
                .iter()
                .map(|name| ContextPrimitive::standalone(context_name(name)))
                .collect(),
        };
        let installed = requirements(&["x-tenant"]);
        let removed = requirements(&[]);
        let supported = requirements(&["x-tenant"]);
        let unsupported_name = requirements(&["x-request-id"]);
        let unsupported_default = requirements(&["x-tenant", "x-request-id"]);

        assert!(installed.can_satisfy(&removed));
        assert!(installed.can_satisfy(&supported));
        assert!(!installed.can_satisfy(&unsupported_name));
        assert!(!installed.can_satisfy(&unsupported_default));
    }

    /// Scenario: a propagation declaration selects one original header name.
    /// Guarantees: prepared bindings keep the policy and require only that original name.
    #[test]
    fn header_propagation_policy_is_a_context_declaration() {
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
        let declarations: NodeContextDeclarations = [
            ContextDeclaration::Produces {
                entry: context_name("preserved"),
            },
            ContextDeclaration::HeaderPropagation {
                policy: policy.clone(),
            },
        ]
        .into_iter()
        .collect();
        let compiled = compiled_bindings(declarations.clone());

        let requirements = context_runtime_requirements(declarations.clone());
        assert!(
            requirements
                .original_names
                .contains(&ContextPrimitive::standalone(context_name("preserved")))
        );
        assert!(
            !requirements
                .original_names
                .contains(&ContextPrimitive::standalone(context_name("other")))
        );
        assert!(
            compiled
                .header_propagation_policy(
                    &pipeline("group", "pipeline"),
                    &ConfigNodeId::from("node")
                )
                .is_some()
        );
    }

    /// Scenario: node and pipeline header policies and an identity policy are configured.
    /// Guarantees: node header policies take precedence, pipeline headers provide the fallback,
    /// and authorized identity capture is declared only for receivers.
    #[test]
    fn wrapper_declarations_resolve_policy_precedence() {
        let identity_policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "tenant"}]))
                .expect("valid authorized identity policy");
        let node_capture = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![CaptureRule {
                match_names: vec![context_name("node")],
                store_as: None,
                sensitive: false,
                value_kind: None,
            }],
        );
        let pipeline_policy = TransportHeadersPolicy {
            header_capture: HeaderCapturePolicy::new(
                CaptureDefaults::default(),
                vec![CaptureRule {
                    match_names: vec![context_name("pipeline")],
                    store_as: None,
                    sensitive: false,
                    value_kind: None,
                }],
            ),
            ..Default::default()
        };
        let mut receiver = NodeUserConfig::new_receiver_config("urn:test:receiver:example");
        receiver.header_capture = Some(node_capture.clone());

        assert_eq!(
            PipelineFactory::<()>::wrapper_context_declarations(
                &receiver,
                &Some(pipeline_policy.clone()),
                &Some(identity_policy.clone()),
            ),
            [
                ContextDeclaration::HeaderCapture {
                    policy: node_capture,
                },
                ContextDeclaration::AuthorizedIdentityCapture {
                    policy: identity_policy.clone(),
                },
            ]
            .into_iter()
            .collect(),
        );

        let receiver = NodeUserConfig::new_receiver_config("urn:test:receiver:example");
        assert_eq!(
            PipelineFactory::<()>::wrapper_context_declarations(
                &receiver,
                &Some(pipeline_policy.clone()),
                &Some(identity_policy.clone()),
            ),
            [
                ContextDeclaration::HeaderCapture {
                    policy: pipeline_policy.header_capture.clone(),
                },
                ContextDeclaration::AuthorizedIdentityCapture {
                    policy: identity_policy.clone(),
                },
            ]
            .into_iter()
            .collect(),
        );

        let mut exporter = NodeUserConfig::new_exporter_config("urn:test:exporter:example");
        let node_propagation = HeaderPropagationPolicy::default();
        exporter.header_propagation = Some(node_propagation.clone());
        assert_eq!(
            PipelineFactory::<()>::wrapper_context_declarations(
                &exporter,
                &Some(pipeline_policy.clone()),
                &Some(identity_policy.clone()),
            ),
            [ContextDeclaration::HeaderPropagation {
                policy: node_propagation,
            }]
            .into_iter()
            .collect(),
        );

        let exporter = NodeUserConfig::new_exporter_config("urn:test:exporter:example");
        assert_eq!(
            PipelineFactory::<()>::wrapper_context_declarations(
                &exporter,
                &Some(pipeline_policy.clone()),
                &Some(identity_policy),
            ),
            [ContextDeclaration::HeaderPropagation {
                policy: pipeline_policy.header_propagation,
            }]
            .into_iter()
            .collect(),
        );
    }

    /// Scenario: a receiver has an absent or explicitly empty authorized identity policy.
    /// Guarantees: neither form creates an authorized identity declaration or non-empty binding.
    #[test]
    fn empty_authorized_identity_policy_produces_no_binding() {
        let receiver = NodeUserConfig::new_receiver_config("urn:test:receiver:example");

        for policy in [None, Some(AuthorizedIdentityPolicy::default())] {
            let declarations =
                PipelineFactory::<()>::wrapper_context_declarations(&receiver, &None, &policy);
            assert!(declarations.is_empty());

            let compiled = compiled_bindings(declarations);
            let node = compiled
                .by_pipeline
                .get(&pipeline("group", "pipeline"))
                .and_then(|nodes| nodes.get(&ConfigNodeId::from("node")))
                .expect("compiled node binding");
            assert!(node.is_empty());
            assert!(
                compiled
                    .authorized_identity_policy(
                        &pipeline("group", "pipeline"),
                        &ConfigNodeId::from("node"),
                    )
                    .is_none()
            );
        }
    }

    /// Scenario: a receiver declares an authorized identity claim projection.
    /// Guarantees: compiled node bindings retain the exact policy and
    /// live-update compatibility rejects changed projections in either
    /// comparison direction.
    #[test]
    fn authorized_identity_policy_is_a_compiled_receiver_binding() {
        let policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "tenant"}]))
                .expect("valid authorized identity policy");
        let declarations: NodeContextDeclarations =
            [ContextDeclaration::AuthorizedIdentityCapture {
                policy: policy.clone(),
            }]
            .into_iter()
            .collect();
        let compiled = compiled_bindings(declarations);
        let changed_policy: AuthorizedIdentityPolicy = serde_json::from_value(
            serde_json::json!([{"claim": "groups", "store_as": "access_groups"}]),
        )
        .expect("valid changed authorized identity policy");
        let changed = compiled_bindings(
            [ContextDeclaration::AuthorizedIdentityCapture {
                policy: changed_policy,
            }]
            .into_iter()
            .collect(),
        );
        let pipeline = pipeline("group", "pipeline");

        assert_eq!(
            compiled.authorized_identity_policy(&pipeline, &ConfigNodeId::from("node")),
            Some(&policy),
        );
        assert!(!compiled.pipeline_bindings_match(&changed, &pipeline));
        assert!(!changed.pipeline_bindings_match(&compiled, &pipeline));
    }

    /// Scenario: a node declares a context read and a propagation policy.
    /// Guarantees: undeclared reads and nodes fail. The propagation declaration is retained.
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
        let propagation_declaration = ContextDeclaration::HeaderPropagation {
            policy: HeaderPropagationPolicy::default(),
        };
        let declarations = matching
            .context_declarations()
            .into_iter()
            .chain(std::iter::once(propagation_declaration.clone()))
            .collect();
        let bindings = compiled_bindings(declarations);

        assert!(
            bindings
                .validate_node_declarations(&pipeline, &node, &matching.context_declarations(),)
                .is_ok()
        );
        assert!(
            bindings
                .validate_node_declarations(&pipeline, &node, &changed.context_declarations())
                .is_err()
        );
        assert!(
            bindings
                .validate_node_declarations(
                    &pipeline,
                    &ConfigNodeId::from("other"),
                    &matching.context_declarations(),
                )
                .is_err()
        );
        assert!(
            CompiledContextBindings::empty()
                .validate_node_declarations(&pipeline, &node, &matching.context_declarations())
                .is_err()
        );
        assert!(
            bindings
                .header_propagation_policy(&pipeline, &node)
                .is_some()
        );
    }

    /// Scenario: a node with no declarations is missing from the compiled bindings.
    /// Guarantees: empty declarations still require a registered node in the correct pipeline.
    #[test]
    fn empty_declarations_require_a_compiled_node() {
        let key = pipeline("group", "pipeline");
        let node = ConfigNodeId::from("node");
        let declarations = NodeContextDeclarations::default();
        let bindings = compiled_bindings(declarations.clone());

        assert!(
            bindings
                .validate_node_declarations(&key, &node, &declarations)
                .is_ok()
        );
        assert!(
            bindings
                .validate_node_declarations(&pipeline("group", "other"), &node, &declarations)
                .is_err()
        );
        assert!(
            CompiledContextBindings::empty()
                .validate_node_declarations(&key, &node, &declarations)
                .is_err()
        );
    }
}
