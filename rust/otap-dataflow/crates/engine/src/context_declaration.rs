// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Collects and compiles context declarations before runtime construction.
//!
//! Primitive reads, writes, and all-stored selections always specify a source domain.
//! Composite selections instead name the composite and optionally its member; the
//! definition supplies each member's domain and the entire entry's presence gate.
//! Original wire names are supported only for transport-header values.

/// Compiles logical context layouts and resolves field projections.
mod layout;
/// Compiles and applies exporter transport-header propagation policies.
mod propagation;

pub use layout::*;
pub use propagation::CompiledHeaderPropagationPolicy;
use propagation::CompiledHeaderPropagationPolicy as HeaderPropagationPolicy;

use crate::PipelineFactory;
use crate::error::Error as EngineError;
use otel_arrow_dfe_config::authorized_identity_policy::AuthorizedIdentityPolicy;
use otel_arrow_dfe_config::context_policy::ContextEntryDeclaration as ConfigContextEntryDeclaration;
use otel_arrow_dfe_config::engine::ResolvedOtelDataflowSpec;
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::node::{NodeKind, NodeUserConfig};
use otel_arrow_dfe_config::transport_headers_policy::{
    CompiledHeaderCapturePolicy, HeaderCapturePolicy, TransportHeadersPolicy,
};
use otel_arrow_dfe_config::{ContextEntryName, NodeId as ConfigNodeId, PipelineKey};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

/// Selects a domain-scoped primitive, a composite member, or a whole composite.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextEntryTarget {
    /// A primitive stored name within its source domain.
    Primitive {
        /// Source authority domain.
        domain: ContextDomain,
        /// Stored primitive name.
        name: ContextEntryName,
    },
    /// A member whose source domain is specified by its composite definition.
    CompositeMember {
        /// Composite entry name.
        composite: ContextEntryName,
        /// Member name, including any configured alias.
        member: ContextEntryName,
    },
    /// All members of a composite, retaining their individual source domains.
    Composite {
        /// Composite entry name.
        name: ContextEntryName,
    },
}

impl ContextEntryTarget {
    /// Returns the enclosing composite name, or none for a primitive.
    fn composite_name(&self) -> Option<&ContextEntryName> {
        match self {
            Self::Primitive { .. } => None,
            Self::CompositeMember { composite, .. } => Some(composite),
            Self::Composite { name } => Some(name),
        }
    }

    /// Visits selected value sources with their domains, excluding condition-only fields.
    fn visit_sources(
        &self,
        composites: &[ConfigContextEntryDeclaration],
        mut visit: impl FnMut(ContextDomain, &ContextEntryName) -> Result<(), Error>,
    ) -> Result<(), Error> {
        match self {
            Self::Primitive { domain, name } => visit(*domain, name),
            Self::CompositeMember { composite, member } => {
                let declaration = composite_declaration(composite, composites)?;
                let part = declaration
                    .definition
                    .0
                    .iter()
                    .find(|part| part.member_name() == Some(member))
                    .ok_or_else(|| {
                        invalid_context(format!("unknown context member `{composite}:{member}`"))
                    })?;
                visit(part.domain(), part.reference().name())
            }
            Self::Composite { name } => {
                let declaration = composite_declaration(name, composites)?;
                for part in &declaration.definition.0 {
                    if part.member_name().is_some() {
                        visit(part.domain(), part.reference().name())?;
                    }
                }
                Ok(())
            }
        }
    }
}

/// Wraps a context compilation failure as an invalid user configuration.
fn invalid_context(error: impl Into<String>) -> Error {
    Error::InvalidUserConfig {
        error: error.into(),
    }
}

/// Finds exactly one composite declaration or reports a missing or duplicate name.
fn composite_declaration<'a>(
    name: &ContextEntryName,
    declarations: &'a [ConfigContextEntryDeclaration],
) -> Result<&'a ConfigContextEntryDeclaration, Error> {
    let mut matching = declarations.iter().filter(|entry| &entry.name == name);
    let declaration = matching
        .next()
        .ok_or_else(|| invalid_context(format!("unknown composite context entry `{name}`")))?;
    if matching.next().is_some() {
        return Err(invalid_context(format!(
            "duplicate composite context entry `{name}`"
        )));
    }
    Ok(declaration)
}

/// Pairs a target with its value representation without weakening composite presence requirements.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContextEntrySelector {
    /// Primitive source or explicitly qualified composite selection.
    pub target: ContextEntryTarget,
    /// Representation applied to selected value members, excluding condition-only fields.
    pub form: ContextEntrySelectorForm,
}

/// Specifies whether a consumer needs values, stored names, or original wire names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextEntrySelectorForm {
    /// Value only.
    Value,
    /// Stored name and value, preserving configured spelling.
    StoredKeyValue,
    /// Original wire name and value for transport-header fields only.
    OriginalKeyValue,
}

/// Selects named entries or all stored primitives within one source domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextConsumerSelector {
    /// Selects named context entries in order.
    Entries {
        /// Entries to read.
        entries: Box<[ContextEntrySelector]>,
    },
    /// Selects every primitive in one domain using its stored name.
    AllStored {
        /// Source authority domain.
        domain: ContextDomain,
    },
}

/// A node's declared context behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextDeclaration {
    /// Declares a primitive produced by the node without granting write authority.
    Produces {
        /// Source authority domain.
        domain: ContextDomain,
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
    /// Returns whether the declaration describes component reads or writes.
    fn is_component_declaration(&self) -> bool {
        matches!(self, Self::Produces { .. } | Self::Consumes { .. })
    }

    /// Validates selected sources and computes their original-header-name retention needs.
    fn context_runtime_requirements(
        &self,
        composites: &[ConfigContextEntryDeclaration],
    ) -> Result<ContextRuntimeRequirements, Error> {
        let mut requirements = ContextRuntimeRequirements::none();
        match self {
            Self::Consumes {
                selector: ContextConsumerSelector::Entries { entries },
            } => {
                for entry in entries {
                    entry.target.visit_sources(composites, |domain, name| {
                        if entry.form == ContextEntrySelectorForm::OriginalKeyValue {
                            if domain != ContextDomain::TransportHeader {
                                return Err(invalid_context(format!(
                                    "original wire name requested for {domain:?} context entry `{name}`; only transport headers have original wire names"
                                )));
                            }
                            _ = requirements
                                .original_name_retention
                                .overrides
                                .insert(original_name_key(name), true);
                        }
                        Ok(())
                    })?;
                }
            }
            Self::Consumes {
                selector: ContextConsumerSelector::AllStored { .. },
            }
            | Self::Produces { .. }
            | Self::HeaderCapture { .. }
            | Self::AuthorizedIdentityCapture { .. } => {}
            Self::HeaderPropagation { policy } => {
                requirements
                    .original_name_retention
                    .default_preserve_original = policy.propagates_original_name_by_default();
                policy.visit_original_name_requirement_names(|name| {
                    let preserve_original = policy.propagates_original_name(name);
                    if preserve_original
                        != requirements
                            .original_name_retention
                            .default_preserve_original
                    {
                        _ = requirements
                            .original_name_retention
                            .overrides
                            .insert(original_name_key(name), preserve_original);
                    }
                });
            }
        }
        Ok(requirements)
    }
}

/// Registers a component factory's context declaration callback.
#[derive(Clone, Copy)]
pub struct ContextDeclarationProvider {
    /// Declaration callback.
    pub declarations: ContextDeclarationFn,
}

/// Derives deterministic declarations from serialized node configuration.
pub type ContextDeclarationFn = fn(&serde_json::Value) -> Result<NodeContextDeclarations, Error>;

/// Derives and validates context declarations from typed node configuration.
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

/// Stores one node's declarations in sorted, deduplicated order.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeContextDeclarations {
    /// Sorted and deduplicated declarations.
    declarations: Box<[ContextDeclaration]>,
}

impl FromIterator<ContextDeclaration> for NodeContextDeclarations {
    /// Collects declarations into a sorted, deduplicated set.
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
    /// An owned context declaration.
    type Item = ContextDeclaration;
    /// An owning iterator over declarations in sorted order.
    type IntoIter = std::vec::IntoIter<ContextDeclaration>;

    /// Consumes the set and yields declarations in sorted order.
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

/// Deserializes typed node configuration and derives its context declarations.
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
}

/// Stores one node's component declarations, selected composites, and engine context policies.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledNodeBindings {
    /// Declarations supplied by the component factory.
    component_declarations: NodeContextDeclarations,
    /// Selected composite definitions, including the whole presence gate for member selections.
    composites: Box<[ConfigContextEntryDeclaration]>,
    /// Receiver header capture policy compiled for the engine requirements.
    header_capture: Option<CompiledHeaderCapturePolicy>,
    /// Exporter header propagation policy resolved from node or pipeline config.
    header_propagation: Option<HeaderPropagationPolicy>,
    /// Receiver authorized identity claim projection policy.
    authorized_identity_capture: Option<AuthorizedIdentityPolicy>,
}

/// Declarations indexed by pipeline and node identifiers.
type ContextDeclarationsByPipeline =
    HashMap<PipelineKey, HashMap<ConfigNodeId, PreparedNodeContextDeclarations>>;

/// Validated declarations with their selected definitions and capture requirements.
#[derive(Debug)]
struct PreparedNodeContextDeclarations {
    /// Sorted component and engine declarations for the node.
    declarations: NodeContextDeclarations,
    /// Selected composite definitions in canonical order with their full presence gates.
    composites: Box<[ConfigContextEntryDeclaration]>,
    /// Original-header-name retention needs derived from the declarations.
    requirements: ContextRuntimeRequirements,
}

impl PreparedNodeContextDeclarations {
    /// Validates selections, canonicalizes composite definitions, and derives runtime requirements.
    fn new(
        declarations: NodeContextDeclarations,
        context: &[ConfigContextEntryDeclaration],
    ) -> Result<Self, Error> {
        let composite_names = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries { entries },
                } => Some(entries.iter()),
                _ => None,
            })
            .flatten()
            .filter_map(|entry| entry.target.composite_name())
            .collect::<BTreeSet<_>>();
        let composites = composite_names
            .into_iter()
            .map(|name| {
                let mut declaration = composite_declaration(name, context)?.clone();
                validate_definition(&declaration)?;
                declaration.definition.0.sort_unstable();
                Ok(declaration)
            })
            .collect::<Result<Box<[_]>, Error>>()?;
        let requirements = declarations.iter().try_fold(
            ContextRuntimeRequirements::none(),
            |requirements, declaration| {
                Ok::<_, Error>(
                    requirements.union(declaration.context_runtime_requirements(&composites)?),
                )
            },
        )?;
        Ok(Self {
            declarations,
            composites,
            requirements,
        })
    }
}

/// Immutable engine-wide requirements used to compile context bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRuntimeRequirements {
    /// Requirements for retaining original transport-header names.
    original_name_retention: OriginalNameRetention,
}

/// Original-name retention as a default disposition plus name-specific overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OriginalNameRetention {
    /// Disposition for names without an explicit override.
    default_preserve_original: bool,
    /// Lowercase name-specific dispositions that differ from the default.
    overrides: BTreeMap<Box<str>, bool>,
}

/// Requirements and node bindings prepared from one resolved configuration.
#[derive(Debug, Clone)]
pub struct PreparedContext {
    /// Runtime requirements derived from these declarations.
    pub runtime_requirements: ContextRuntimeRequirements,
    /// Node bindings compiled using the selected engine requirements.
    pub bindings: Arc<CompiledContextBindings>,
}

impl ContextRuntimeRequirements {
    /// Combines runtime requirements from every node in every pipeline.
    fn compile(declarations: &ContextDeclarationsByPipeline) -> Self {
        declarations
            .values()
            .flat_map(HashMap::values)
            .fold(Self::none(), |requirements, declaration| {
                requirements.union(declaration.requirements.clone())
            })
    }

    /// Creates requirements that retain no original header names.
    fn none() -> Self {
        Self {
            original_name_retention: OriginalNameRetention {
                default_preserve_original: false,
                overrides: BTreeMap::new(),
            },
        }
    }

    /// Combines both sets of runtime requirements.
    fn union(self, other: Self) -> Self {
        Self {
            original_name_retention: self
                .original_name_retention
                .union(other.original_name_retention),
        }
    }

    /// Returns whether the installed requirements satisfy every candidate requirement.
    #[must_use]
    pub fn can_satisfy(&self, candidate: &Self) -> bool {
        self.original_name_retention
            .can_satisfy(&candidate.original_name_retention)
    }

    /// Returns whether captured entries with this stored name retain the original wire name.
    #[must_use]
    pub fn preserves_original_name(&self, name: &ContextEntryName) -> bool {
        self.original_name_retention.preserves_original_name(name)
    }
}

impl OriginalNameRetention {
    /// Combines retention policies and keeps only overrides that differ from the new default.
    fn union(self, other: Self) -> Self {
        let default_preserve_original =
            self.default_preserve_original || other.default_preserve_original;
        let mut names = self
            .overrides
            .keys()
            .chain(other.overrides.keys())
            .cloned()
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        let overrides = names
            .into_iter()
            .filter_map(|name| {
                let preserve_original =
                    self.preserves_original_key(&name) || other.preserves_original_key(&name);
                (preserve_original != default_preserve_original)
                    .then_some((name, preserve_original))
            })
            .collect();
        Self {
            default_preserve_original,
            overrides,
        }
    }

    /// Returns whether this policy retains every name required by the candidate.
    fn can_satisfy(&self, candidate: &Self) -> bool {
        if candidate.default_preserve_original && !self.default_preserve_original {
            return false;
        }
        self.overrides
            .keys()
            .chain(candidate.overrides.keys())
            .all(|name| {
                !candidate.preserves_original_key(name) || self.preserves_original_key(name)
            })
    }

    /// Returns the retention disposition for a case-insensitive stored header name.
    fn preserves_original_name(&self, name: &ContextEntryName) -> bool {
        self.preserves_original_key(&original_name_key(name))
    }

    /// Looks up a lowercase name's override or falls back to the default disposition.
    fn preserves_original_key(&self, name: &str) -> bool {
        self.overrides
            .get(name)
            .copied()
            .unwrap_or(self.default_preserve_original)
    }
}

/// Converts a stored header name to its ASCII-lowercase retention key.
fn original_name_key(name: &ContextEntryName) -> Box<str> {
    name.as_str().to_ascii_lowercase().into()
}

impl CompiledNodeBindings {
    /// Compiles header capture and separates component declarations from engine policies.
    fn compile(
        prepared: PreparedNodeContextDeclarations,
        requirements: &ContextRuntimeRequirements,
    ) -> Self {
        let mut component_declarations = Vec::new();
        let mut header_capture = None;
        let mut header_propagation = None;
        let mut authorized_identity_capture = None;
        for declaration in prepared.declarations {
            match declaration {
                declaration @ (ContextDeclaration::Produces { .. }
                | ContextDeclaration::Consumes { .. }) => {
                    component_declarations.push(declaration);
                }
                ContextDeclaration::HeaderCapture { policy } => {
                    header_capture =
                        Some(policy.compile(|name| requirements.preserves_original_name(name)));
                }
                ContextDeclaration::HeaderPropagation { policy } => {
                    header_propagation = Some(policy);
                }
                ContextDeclaration::AuthorizedIdentityCapture { policy } => {
                    authorized_identity_capture = Some(policy);
                }
            }
        }

        Self {
            component_declarations: component_declarations.into_iter().collect(),
            composites: prepared.composites,
            header_capture,
            header_propagation,
            authorized_identity_capture,
        }
    }

    /// Returns whether the node has no component declarations or engine context policies.
    fn is_empty(&self) -> bool {
        self.component_declarations.is_empty()
            && self.header_capture.is_none()
            && self.header_propagation.is_none()
            && self.authorized_identity_capture.is_none()
    }
}

impl CompiledContextBindings {
    /// Creates an empty binding set that rejects all node declaration validation.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            by_pipeline: HashMap::new(),
        }
    }

    /// Compiles every node's bindings using the same engine-wide runtime requirements.
    fn compile(
        declarations: ContextDeclarationsByPipeline,
        requirements: &ContextRuntimeRequirements,
    ) -> Self {
        let by_pipeline = declarations
            .into_iter()
            .map(|(pipeline, nodes)| {
                let nodes = nodes
                    .into_iter()
                    .map(|(node, declarations)| {
                        (
                            node,
                            CompiledNodeBindings::compile(declarations, requirements),
                        )
                    })
                    .collect();
                (pipeline, nodes)
            })
            .collect();

        Self { by_pipeline }
    }

    /// Returns the node's compiled header capture policy.
    pub(crate) fn header_capture_policy(
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

    /// Returns the node's header propagation policy.
    pub(crate) fn header_propagation_policy(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
    ) -> Option<&HeaderPropagationPolicy> {
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

    /// Compares one pipeline's bindings while ignoring nodes without context declarations.
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

    /// Checks declarations from parsed component configuration against the node's compiled bindings.
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
        let declarations = self.context_declarations(resolved)?;
        let runtime_requirements = ContextRuntimeRequirements::compile(&declarations);
        let bindings = Self::compile_bindings(declarations, &runtime_requirements);
        Ok(PreparedContext {
            runtime_requirements,
            bindings,
        })
    }

    /// Derives candidate requirements and compiles bindings using the installed runtime requirements.
    pub fn compile_candidate_context(
        &self,
        resolved: &ResolvedOtelDataflowSpec,
        installed_requirements: &ContextRuntimeRequirements,
    ) -> Result<PreparedContext, EngineError> {
        let declarations = self.context_declarations(resolved)?;
        let runtime_requirements = ContextRuntimeRequirements::compile(&declarations);
        let bindings = Self::compile_bindings(declarations, installed_requirements);
        Ok(PreparedContext {
            runtime_requirements,
            bindings,
        })
    }

    /// Wraps compiled bindings in an immutable shared handle for pipeline runtimes.
    fn compile_bindings(
        declarations: ContextDeclarationsByPipeline,
        requirements: &ContextRuntimeRequirements,
    ) -> Arc<CompiledContextBindings> {
        Arc::new(CompiledContextBindings::compile(declarations, requirements))
    }

    /// Collects and validates component and engine declarations for each resolved pipeline node.
    fn context_declarations(
        &self,
        resolved: &ResolvedOtelDataflowSpec,
    ) -> Result<ContextDeclarationsByPipeline, EngineError> {
        let mut declarations = ContextDeclarationsByPipeline::new();

        for pipeline in &resolved.pipelines {
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
                    &pipeline.policies.context,
                )?;
                let declarations = component_declarations
                    .into_iter()
                    .chain(wrapper_declarations)
                    .collect();
                let declarations =
                    PreparedNodeContextDeclarations::new(declarations, &pipeline.policies.context)
                        .map_err(|error| EngineError::ConfigError(Box::new(error)))?;
                let _ = declarations_by_node.insert(node_id.clone(), declarations);
            }
            let _ = declarations.insert(pipeline_key, declarations_by_node);
        }

        Ok(declarations)
    }

    /// Derives engine declarations from node header overrides and resolved pipeline policies.
    fn wrapper_context_declarations(
        node: &NodeUserConfig,
        pipeline_policy: &Option<TransportHeadersPolicy>,
        authorized_identity: &Option<AuthorizedIdentityPolicy>,
        context: &[ConfigContextEntryDeclaration],
    ) -> Result<NodeContextDeclarations, EngineError> {
        let declarations = match node.kind() {
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
            NodeKind::Exporter => {
                let policy = node.header_propagation.as_ref().or_else(|| {
                    pipeline_policy
                        .as_ref()
                        .map(|policy| &policy.header_propagation)
                });
                policy
                    .cloned()
                    .map(|policy| {
                        HeaderPropagationPolicy::compile(policy, context)
                            .map(|policy| ContextDeclaration::HeaderPropagation { policy })
                            .map_err(|error| {
                                EngineError::ConfigError(Box::new(Error::InvalidUserConfig {
                                    error,
                                }))
                            })
                    })
                    .transpose()?
                    .into_iter()
                    .collect()
            }
            NodeKind::Processor => NodeContextDeclarations::default(),
        };
        Ok(declarations)
    }

    /// Validates node configuration and rejects engine-owned factory declarations.
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

/// Exercises declaration compilation, retention requirements, and live-update compatibility.
#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::transport_headers::{TransportHeader, TransportHeaders};
    use otel_arrow_dfe_config::transport_headers_policy::HeaderPropagationPolicy as HeaderPropagationConfig;
    use otel_arrow_dfe_config::transport_headers_policy::{CaptureDefaults, CaptureRule};

    /// Typed component configuration used to exercise declaration validation.
    #[derive(serde::Deserialize)]
    struct TestDeclarationConfig {
        /// Selected primitive or composite member name.
        entry: ContextEntryName,
        /// Optional composite containing the selected member.
        #[serde(default)]
        composite: Option<ContextEntryName>,
        /// Whether the consumer requests the original transport-header name.
        #[serde(default)]
        original: bool,
    }

    impl ConfigNodeContextDeclaration for TestDeclarationConfig {
        /// Declares one test consumer with the configured target and name representation.
        fn context_declarations(&self) -> NodeContextDeclarations {
            [ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        target: match &self.composite {
                            Some(composite) => ContextEntryTarget::CompositeMember {
                                composite: composite.clone(),
                                member: self.entry.clone(),
                            },
                            None => ContextEntryTarget::Primitive {
                                domain: ContextDomain::TransportHeader,
                                name: self.entry.clone(),
                            },
                        },
                        form: if self.original {
                            ContextEntrySelectorForm::OriginalKeyValue
                        } else {
                            ContextEntrySelectorForm::Value
                        },
                    }]
                    .into_boxed_slice(),
                },
            }]
            .into_iter()
            .collect()
        }
    }

    /// Builds a pipeline key from test group and pipeline names.
    fn pipeline(group: &str, name: &str) -> PipelineKey {
        PipelineKey::new(group.to_owned().into(), name.to_owned().into())
    }

    /// Parses a test context name and fails if it is invalid.
    fn context_name(name: &str) -> ContextEntryName {
        name.try_into().expect("valid test context entry name")
    }

    /// Fails if context compilation unexpectedly constructs a receiver.
    fn unused_test_receiver(
        _: crate::context::PipelineContext,
        _: crate::node::NodeId,
        _: Arc<NodeUserConfig>,
        _: &crate::config::ReceiverConfig,
        _: &crate::capability::registry::Capabilities,
    ) -> Result<crate::receiver::ReceiverWrapper<()>, Error> {
        unreachable!("context compilation does not construct test nodes")
    }

    /// Fails if context compilation unexpectedly constructs an exporter.
    fn unused_test_exporter(
        _: crate::context::PipelineContext,
        _: crate::node::NodeId,
        _: Arc<NodeUserConfig>,
        _: &crate::config::ExporterConfig,
        _: &crate::capability::registry::Capabilities,
    ) -> Result<crate::exporter::ExporterWrapper<()>, Error> {
        unreachable!("context compilation does not construct test nodes")
    }

    /// Fails if context compilation unexpectedly constructs a processor.
    fn unused_test_processor(
        _: crate::context::PipelineContext,
        _: crate::node::NodeId,
        _: Arc<NodeUserConfig>,
        _: &crate::config::ProcessorConfig,
        _: &crate::capability::registry::Capabilities,
    ) -> Result<crate::processor::ProcessorWrapper<()>, Error> {
        unreachable!("context compilation does not construct test nodes")
    }

    /// Accepts configuration for test factories that need no typed validation.
    fn accept_test_config(_: &serde_json::Value) -> Result<(), Error> {
        Ok(())
    }

    /// Registers test receivers without constructing runtime nodes.
    static TEST_RECEIVERS: [crate::ReceiverFactory<()>; 2] = [
        crate::ReceiverFactory {
            name: "urn:test:receiver:example",
            create: unused_test_receiver,
            context_declarations: None,
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
        crate::ReceiverFactory {
            name: "urn:otel:receiver:internal_telemetry",
            create: unused_test_receiver,
            context_declarations: None,
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: accept_test_config,
        },
    ];

    /// Registers ordinary exporters and a typed context consumer for compilation tests.
    static TEST_EXPORTERS: [crate::ExporterFactory<()>; 4] = [
        crate::ExporterFactory {
            name: "urn:test:exporter:example",
            create: unused_test_exporter,
            context_declarations: None,
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
        crate::ExporterFactory {
            name: "urn:otel:exporter:noop",
            create: unused_test_exporter,
            context_declarations: None,
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
        crate::ExporterFactory {
            name: "urn:otel:exporter:console",
            create: unused_test_exporter,
            context_declarations: None,
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
        crate::ExporterFactory {
            name: "urn:test:exporter:context",
            create: unused_test_exporter,
            context_declarations: Some(ContextDeclarationProvider::from_typed_config::<
                TestDeclarationConfig,
            >()),
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
                TestDeclarationConfig,
            >,
        },
    ];

    /// Registers the type router used by resolved test pipelines.
    static TEST_PROCESSORS: [crate::ProcessorFactory<()>; 1] = [crate::ProcessorFactory {
        name: "urn:otel:processor:type_router",
        create: unused_test_processor,
        context_declarations: None,
        wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: accept_test_config,
    }];

    /// Builds a pipeline factory from the test node registrations.
    fn test_pipeline_factory() -> PipelineFactory<()> {
        PipelineFactory::new(&TEST_RECEIVERS, &TEST_PROCESSORS, &TEST_EXPORTERS, &[])
    }

    /// Builds pipeline YAML with a composite definition and an exporter propagation selector.
    fn conditional_pipeline_yaml(composite: &str, selector: &str) -> String {
        format!(
            r#"
version: otel_dataflow/v1
policies:
  context:
    entries:
      tenant: {composite}
engine: {{}}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: {{}}
          exporter:
            type: "urn:test:exporter:example"
            header_propagation:
              default:
                selector:
                  type: named
                  named: [{selector}]
                name: stored_name
            config: {{}}
        connections:
          - from: receiver
            to: exporter
"#
        )
    }

    /// Resolves test pipeline YAML containing a composite propagation selector.
    fn resolve_conditional_pipeline(composite: &str, selector: &str) -> ResolvedOtelDataflowSpec {
        otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(&conditional_pipeline_yaml(
            composite, selector,
        ))
        .expect("conditional pipeline YAML is valid")
        .resolve()
    }

    /// Prepares declarations for one test node without composite definitions.
    fn declarations_by_pipeline(
        effective: NodeContextDeclarations,
    ) -> ContextDeclarationsByPipeline {
        HashMap::from([(
            pipeline("group", "pipeline"),
            HashMap::from([(
                ConfigNodeId::from("node"),
                PreparedNodeContextDeclarations::new(effective, &[]).expect("valid declarations"),
            )]),
        )])
    }

    /// Compiles one test node's bindings using its own runtime requirements.
    fn compiled_bindings(effective: NodeContextDeclarations) -> CompiledContextBindings {
        let declarations = declarations_by_pipeline(effective);
        let requirements = ContextRuntimeRequirements::compile(&declarations);
        CompiledContextBindings::compile(declarations, &requirements)
    }

    /// Derives runtime requirements from one test node's declarations.
    fn context_runtime_requirements(
        effective: NodeContextDeclarations,
    ) -> ContextRuntimeRequirements {
        ContextRuntimeRequirements::compile(&declarations_by_pipeline(effective))
    }

    /// Builds a primitive test target in the specified source domain.
    fn primitive_target(domain: ContextDomain, name: &str) -> ContextEntryTarget {
        ContextEntryTarget::Primitive {
            domain,
            name: context_name(name),
        }
    }

    /// Builds a test target for one qualified composite member.
    fn member_target(composite: &str, member: &str) -> ContextEntryTarget {
        ContextEntryTarget::CompositeMember {
            composite: context_name(composite),
            member: context_name(member),
        }
    }

    /// Declares one test consumer for the given target and representation.
    fn consumer(
        target: ContextEntryTarget,
        form: ContextEntrySelectorForm,
    ) -> NodeContextDeclarations {
        [ContextDeclaration::Consumes {
            selector: ContextConsumerSelector::Entries {
                entries: Box::new([ContextEntrySelector { target, form }]),
            },
        }]
        .into_iter()
        .collect()
    }

    /// Builds a conditional test composite with aliased header and identity members.
    fn mixed_composite() -> ConfigContextEntryDeclaration {
        use otel_arrow_dfe_config::context_policy::{ContextEntryDefinition, ContextScope};

        ConfigContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: context_name("tenant"),
            definition: serde_json::from_value::<ContextEntryDefinition>(serde_json::json!([
                {"type": "transport_header", "name": "id", "store_as": "header_id"},
                {"type": "authorized_identity", "name": "id", "store_as": "identity_id"},
                {"type": "transport_header_match", "name": "environment", "value": "production"}
            ]))
            .expect("valid composite"),
        }
    }

    /// Scenario: same-name sources are declared in different domains for reads, writes, and all-stored.
    /// Guarantees: deduplication, node validation, and live binding comparison preserve the domain.
    #[test]
    fn declaration_domains_are_part_of_binding_identity() {
        let declarations = |domain| {
            [
                ContextDeclaration::Produces {
                    domain,
                    entry: context_name("id"),
                },
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries {
                        entries: Box::new([ContextEntrySelector {
                            target: primitive_target(domain, "id"),
                            form: ContextEntrySelectorForm::Value,
                        }]),
                    },
                },
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::AllStored { domain },
                },
            ]
        };
        let header = declarations(ContextDomain::TransportHeader);
        let identity = declarations(ContextDomain::AuthorizedIdentity);
        let combined: NodeContextDeclarations =
            header.clone().into_iter().chain(identity.clone()).collect();
        assert_eq!(combined.len(), 6);
        assert!(
            !context_runtime_requirements(combined).preserves_original_name(&context_name("id"))
        );

        for (header, identity) in header.into_iter().zip(identity) {
            let header: NodeContextDeclarations = [header].into_iter().collect();
            let identity: NodeContextDeclarations = [identity].into_iter().collect();
            let installed = compiled_bindings(header.clone());
            let candidate = compiled_bindings(identity.clone());
            let key = pipeline("group", "pipeline");
            let node = ConfigNodeId::from("node");
            assert!(
                installed
                    .validate_node_declarations(&key, &node, &header)
                    .is_ok()
            );
            assert!(
                installed
                    .validate_node_declarations(&key, &node, &identity)
                    .is_err()
            );
            assert!(!installed.pipeline_bindings_match(&candidate, &key));
            assert!(!candidate.pipeline_bindings_match(&installed, &key));
        }
    }

    /// Scenario: a consumer selects an aliased header member of a mixed, conditional composite.
    /// Guarantees: retention uses the header source, not its alias or condition, and keeps the full gate.
    #[test]
    fn composite_member_requirements_preserve_source_domain_and_gate() {
        let context = [mixed_composite()];
        let prepared = PreparedNodeContextDeclarations::new(
            consumer(
                member_target("tenant", "header_id"),
                ContextEntrySelectorForm::OriginalKeyValue,
            ),
            &context,
        )
        .expect("header member supports original names");
        assert!(
            prepared
                .requirements
                .preserves_original_name(&context_name("id"))
        );
        assert!(
            !prepared
                .requirements
                .preserves_original_name(&context_name("header_id"))
        );
        assert!(
            !prepared
                .requirements
                .preserves_original_name(&context_name("environment"))
        );
        assert_eq!(prepared.composites[0].definition.0.len(), 3);

        for form in [
            ContextEntrySelectorForm::Value,
            ContextEntrySelectorForm::StoredKeyValue,
        ] {
            let prepared = PreparedNodeContextDeclarations::new(
                consumer(member_target("tenant", "identity_id"), form),
                &context,
            )
            .expect("identity member");
            assert!(
                !prepared
                    .requirements
                    .preserves_original_name(&context_name("id"))
            );
        }
        let whole = ContextEntryTarget::Composite {
            name: context_name("tenant"),
        };
        let mut sources = Vec::new();
        whole
            .visit_sources(&context, |domain, name| {
                sources.push((domain, name.clone()));
                Ok(())
            })
            .expect("whole composite sources");
        assert_eq!(
            sources,
            [
                (ContextDomain::TransportHeader, context_name("id")),
                (ContextDomain::AuthorizedIdentity, context_name("id")),
            ]
        );
        let prepared = PreparedNodeContextDeclarations::new(
            consumer(whole.clone(), ContextEntrySelectorForm::StoredKeyValue),
            &context,
        )
        .expect("mixed composite supports stored values");
        assert!(
            !prepared
                .requirements
                .preserves_original_name(&context_name("id"))
        );

        let mut headers_only = mixed_composite();
        _ = headers_only.definition.0.remove(1);
        let prepared = PreparedNodeContextDeclarations::new(
            consumer(whole, ContextEntrySelectorForm::OriginalKeyValue),
            &[headers_only],
        )
        .expect("header-only composite supports original names");
        assert!(
            prepared
                .requirements
                .preserves_original_name(&context_name("id"))
        );
        assert!(
            !prepared
                .requirements
                .preserves_original_name(&context_name("environment"))
        );
    }

    /// Scenario: consumer targets are missing, select conditions as values, or request identity wire names.
    /// Guarantees: preparation fails explicitly instead of falling back to a same-named header.
    #[test]
    fn invalid_consumer_targets_and_representations_are_rejected() {
        let context = [mixed_composite()];
        for (target, form, expected) in [
            (
                member_target("missing", "header_id"),
                ContextEntrySelectorForm::Value,
                "unknown composite context entry `missing`",
            ),
            (
                member_target("tenant", "missing"),
                ContextEntrySelectorForm::Value,
                "unknown context member `tenant:missing`",
            ),
            (
                member_target("tenant", "environment"),
                ContextEntrySelectorForm::Value,
                "unknown context member `tenant:environment`",
            ),
            (
                primitive_target(ContextDomain::AuthorizedIdentity, "id"),
                ContextEntrySelectorForm::OriginalKeyValue,
                "original wire name requested for AuthorizedIdentity context entry `id`",
            ),
            (
                member_target("tenant", "identity_id"),
                ContextEntrySelectorForm::OriginalKeyValue,
                "original wire name requested for AuthorizedIdentity context entry `id`",
            ),
            (
                ContextEntryTarget::Composite {
                    name: context_name("tenant"),
                },
                ContextEntrySelectorForm::OriginalKeyValue,
                "original wire name requested for AuthorizedIdentity context entry `id`",
            ),
        ] {
            let error = PreparedNodeContextDeclarations::new(consumer(target, form), &context)
                .expect_err("invalid selection must fail");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    /// Resolves a test pipeline whose exporter consumes the specified composite member.
    fn resolve_consumer_pipeline(
        composite: &str,
        member: &str,
        original: bool,
    ) -> ResolvedOtelDataflowSpec {
        let yaml = format!(
            r#"
version: otel_dataflow/v1
policies:
  context:
    entries:
      tenant: {composite}
engine: {{}}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: {{}}
          exporter:
            type: "urn:test:exporter:context"
            config:
              entry: {member}
              composite: tenant
              original: {original}
        connections:
          - from: receiver
            to: exporter
"#
        );
        otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(&yaml)
            .expect("consumer pipeline YAML")
            .resolve()
    }

    /// Scenario: a component declares a missing member or asks for an identity member's wire name.
    /// Guarantees: both startup and live-update preparation reject invalid consumer declarations.
    #[test]
    fn full_yaml_preparation_rejects_invalid_consumer_declarations() {
        let composite = "[{type: authorized_identity, name: id}]";
        let factory = test_pipeline_factory();
        let current = factory
            .compile_initial_context(&resolve_consumer_pipeline(composite, "id", false))
            .expect("valid identity consumer");
        for (member, original, expected) in [
            ("missing", false, "unknown context member `tenant:missing`"),
            (
                "id",
                true,
                "original wire name requested for AuthorizedIdentity",
            ),
        ] {
            let resolved = resolve_consumer_pipeline(composite, member, original);
            for result in [
                factory.compile_initial_context(&resolved),
                factory.compile_candidate_context(&resolved, &current.runtime_requirements),
            ] {
                let error = result.expect_err("invalid consumer must fail preparation");
                assert!(error.to_string().contains(expected), "{error}");
            }
        }
    }

    /// Scenario: a qualified consumer stays unchanged while its composite definition changes.
    /// Guarantees: bindings detect source, domain, and presence-gate changes, but ignore definition order.
    #[test]
    fn full_yaml_bindings_track_selected_composite_definitions() {
        let composite = "[{type: transport_header, name: id, store_as: key}, \
            {type: authorized_identity, name: account}, \
            {type: transport_header_match, name: environment, value: production}]";
        let factory = test_pipeline_factory();
        let installed = factory
            .compile_initial_context(&resolve_consumer_pipeline(composite, "key", false))
            .expect("initial consumer");
        let key = pipeline("default", "main");
        for changed in [
            composite.replace("name: id", "name: other"),
            composite.replace("type: transport_header,", "type: authorized_identity,"),
            composite.replace("name: account", "name: other_account"),
            composite.replace("value: production", "value: staging"),
        ] {
            let candidate = factory
                .compile_candidate_context(
                    &resolve_consumer_pipeline(&changed, "key", false),
                    &installed.runtime_requirements,
                )
                .expect("changed consumer");
            assert!(
                !installed
                    .bindings
                    .pipeline_bindings_match(&candidate.bindings, &key)
            );
            assert!(
                !candidate
                    .bindings
                    .pipeline_bindings_match(&installed.bindings, &key)
            );
        }
        let reordered = "[{type: transport_header_match, name: environment, value: production}, \
            {type: authorized_identity, name: account}, \
            {type: transport_header, name: id, store_as: key}]";
        let candidate = factory
            .compile_candidate_context(
                &resolve_consumer_pipeline(reordered, "key", false),
                &installed.runtime_requirements,
            )
            .expect("reordered consumer");
        assert!(
            installed
                .bindings
                .pipeline_bindings_match(&candidate.bindings, &key)
        );
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
                                target: ContextEntryTarget::Primitive {
                                    domain: ContextDomain::TransportHeader,
                                    name: context_name(name),
                                },
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

        assert_eq!(headers.get(0).expect("first header").wire_name(), "X-First");
        assert_eq!(
            headers.get(1).expect("second header").wire_name(),
            "x-second"
        );
        assert_eq!(
            headers.get(2).expect("first alias").wire_name(),
            "X-Alias-A"
        );
        assert_eq!(
            headers.get(3).expect("second alias").wire_name(),
            "X-Alias-B"
        );
    }

    /// Scenario: consumers request different name representations.
    /// Guarantees: only `OriginalKeyValue` requires original names.
    #[test]
    fn declarations_require_only_the_requested_name_form() {
        let declarations: NodeContextDeclarations = [
            ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        target: ContextEntryTarget::Primitive {
                            domain: ContextDomain::TransportHeader,
                            name: context_name("original"),
                        },
                        form: ContextEntrySelectorForm::OriginalKeyValue,
                    }]
                    .into_boxed_slice(),
                },
            },
            ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        target: ContextEntryTarget::Primitive {
                            domain: ContextDomain::TransportHeader,
                            name: context_name("value"),
                        },
                        form: ContextEntrySelectorForm::Value,
                    }]
                    .into_boxed_slice(),
                },
            },
        ]
        .into_iter()
        .collect();
        let requirements = context_runtime_requirements(declarations);
        assert!(requirements.preserves_original_name(&context_name("original")));
        assert!(!requirements.preserves_original_name(&context_name("value")));
    }

    /// Scenario: propagation preserves arbitrary names but overrides one stored name.
    /// Guarantees: the profile uses a true default with one case-insensitive exception.
    #[test]
    fn requirements_canonicalize_default_and_overrides() {
        let propagation: HeaderPropagationConfig = serde_json::from_value(serde_json::json!({
            "default": {
                "selector": {"type": "all_captured"},
                "name": "preserve"
            },
            "overrides": [{
                "match": {"stored_names": ["Authorization"]},
                "name": "stored_name"
            }]
        }))
        .expect("valid propagation policy");
        let propagation = HeaderPropagationPolicy::compile(propagation, &[])
            .expect("propagation policy compiles");
        let requirements = context_runtime_requirements(
            [ContextDeclaration::HeaderPropagation {
                policy: propagation,
            }]
            .into_iter()
            .collect(),
        );

        assert!(
            requirements
                .original_name_retention
                .default_preserve_original
        );
        assert!(!requirements.preserves_original_name(&context_name("AUTHORIZATION")));
        assert!(requirements.preserves_original_name(&context_name("X-Tenant")));
        assert_eq!(
            requirements.original_name_retention.overrides,
            BTreeMap::from([(Box::<str>::from("authorization"), false)])
        );
    }

    /// Scenario: live declarations add and remove original-name consumers.
    /// Guarantees: installed requirements allow subsets but reject unsupported names and defaults.
    #[test]
    fn installed_requirements_support_only_available_original_names() {
        let installed = context_runtime_requirements(
            [
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries {
                        entries: vec![ContextEntrySelector {
                            target: ContextEntryTarget::Primitive {
                                domain: ContextDomain::TransportHeader,
                                name: context_name("x-tenant"),
                            },
                            form: ContextEntrySelectorForm::OriginalKeyValue,
                        }]
                        .into_boxed_slice(),
                    },
                },
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries {
                        entries: vec![ContextEntrySelector {
                            target: ContextEntryTarget::Primitive {
                                domain: ContextDomain::TransportHeader,
                                name: context_name("authorization"),
                            },
                            form: ContextEntrySelectorForm::Value,
                        }]
                        .into_boxed_slice(),
                    },
                },
            ]
            .into_iter()
            .collect(),
        );
        let removed = context_runtime_requirements(NodeContextDeclarations::default());
        let supported = context_runtime_requirements(
            [ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        target: ContextEntryTarget::Primitive {
                            domain: ContextDomain::TransportHeader,
                            name: context_name("X-Tenant"),
                        },
                        form: ContextEntrySelectorForm::OriginalKeyValue,
                    }]
                    .into_boxed_slice(),
                },
            }]
            .into_iter()
            .collect(),
        );
        let unsupported_name = context_runtime_requirements(
            [ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: vec![ContextEntrySelector {
                        target: ContextEntryTarget::Primitive {
                            domain: ContextDomain::TransportHeader,
                            name: context_name("x-request-id"),
                        },
                        form: ContextEntrySelectorForm::OriginalKeyValue,
                    }]
                    .into_boxed_slice(),
                },
            }]
            .into_iter()
            .collect(),
        );
        let unsupported_default = context_runtime_requirements(
            [ContextDeclaration::HeaderPropagation {
                policy: HeaderPropagationPolicy::compile(
                    serde_json::from_value(serde_json::json!({
                        "default": {
                            "selector": {"type": "all_captured"},
                            "name": "preserve"
                        }
                    }))
                    .expect("valid propagation policy"),
                    &[],
                )
                .expect("propagation policy compiles"),
            }]
            .into_iter()
            .collect(),
        );

        assert!(installed.can_satisfy(&removed));
        assert!(installed.can_satisfy(&supported));
        assert!(!installed.can_satisfy(&unsupported_name));
        assert!(!installed.can_satisfy(&unsupported_default));
    }

    /// Scenario: a propagation declaration selects one original header name.
    /// Guarantees: prepared bindings keep the policy and require only that original name.
    #[test]
    fn header_propagation_policy_is_a_context_declaration() {
        let policy: HeaderPropagationConfig = serde_json::from_value(serde_json::json!({
            "default": {
                "selector": {
                    "type": "named",
                    "named": ["preserved"]
                },
                "name": "preserve"
            }
        }))
        .expect("valid propagation policy");
        let policy =
            HeaderPropagationPolicy::compile(policy, &[]).expect("propagation policy compiles");
        let declarations: NodeContextDeclarations = [ContextDeclaration::HeaderPropagation {
            policy: policy.clone(),
        }]
        .into_iter()
        .collect();
        let compiled = compiled_bindings(declarations.clone());

        let requirements = context_runtime_requirements(declarations.clone());
        assert!(requirements.preserves_original_name(&context_name("preserved")));
        assert!(!requirements.preserves_original_name(&context_name("other")));
        assert_eq!(
            compiled.header_propagation_policy(
                &pipeline("group", "pipeline"),
                &ConfigNodeId::from("node")
            ),
            Some(&policy),
        );
    }

    /// Scenario: node and pipeline header policies and an identity policy are configured.
    /// Guarantees: node headers override pipeline defaults and only receivers declare identity capture.
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
                &[],
            )
            .expect("wrapper declarations"),
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
                &[],
            )
            .expect("wrapper declarations"),
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
        let node_propagation = HeaderPropagationConfig::default();
        exporter.header_propagation = Some(node_propagation.clone());
        assert_eq!(
            PipelineFactory::<()>::wrapper_context_declarations(
                &exporter,
                &Some(pipeline_policy.clone()),
                &Some(identity_policy.clone()),
                &[],
            )
            .expect("wrapper declarations"),
            [ContextDeclaration::HeaderPropagation {
                policy: HeaderPropagationPolicy::compile(node_propagation, &[])
                    .expect("node propagation policy compiles"),
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
                &[],
            )
            .expect("wrapper declarations"),
            [ContextDeclaration::HeaderPropagation {
                policy: HeaderPropagationPolicy::compile(pipeline_policy.header_propagation, &[])
                    .expect("pipeline propagation policy compiles"),
            }]
            .into_iter()
            .collect(),
        );
    }

    /// Scenario: an exporter selects a conditional composite transport-header member.
    /// Guarantees: wrapper compilation resolves the visible declaration before installing policy.
    #[test]
    fn wrapper_compiles_conditional_composite_header_propagation() {
        let context: otel_arrow_dfe_config::context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  tenant:
    - type: transport_header
      name: workspace
      store_as: workspace_id
    - type: transport_header_match
      name: environment
      value: production
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ConfigContextEntryDeclaration {
            scope: otel_arrow_dfe_config::context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let mut exporter = NodeUserConfig::new_exporter_config("urn:test:exporter:example");
        exporter.header_propagation = Some(
            serde_yaml::from_str(
                r#"
default:
  selector:
    type: named
    named: [tenant:workspace_id]
  name: stored_name
"#,
            )
            .expect("valid propagation policy"),
        );

        let declarations = PipelineFactory::<()>::wrapper_context_declarations(
            &exporter,
            &None,
            &None,
            &[declaration],
        )
        .expect("wrapper declarations");
        let ContextDeclaration::HeaderPropagation { policy } =
            declarations.iter().next().expect("propagation declaration")
        else {
            panic!("expected header propagation declaration");
        };
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(context_name("workspace"), b"acme"));
        assert_eq!(policy.propagate(&headers).count(), 0);
        headers.push(TransportHeader::text(
            context_name("environment"),
            b"production",
        ));

        let propagated = policy.propagate(&headers).collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace_id");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: complete YAML changes a composite condition or selected member during a live update.
    /// Guarantees: resolution compiles an effective exporter binding and reconciliation detects both changes.
    #[test]
    fn full_yaml_compilation_tracks_conditional_composite_changes() {
        let current_composite = "[{type: transport_header, name: workspace, store_as: workspace_id}, \
            {type: transport_header, name: account, store_as: account_id}, \
            {type: transport_header_match, name: environment, value: production}]";
        let factory = test_pipeline_factory();
        let current = resolve_conditional_pipeline(current_composite, "tenant:workspace_id");
        let installed = factory
            .compile_initial_context(&current)
            .expect("initial context compiles");
        let pipeline = pipeline("default", "main");
        let exporter = ConfigNodeId::from("exporter");
        let policy = installed
            .bindings
            .header_propagation_policy(&pipeline, &exporter)
            .expect("compiled exporter propagation policy");
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(context_name("workspace"), b"acme"));
        headers.push(TransportHeader::text(
            context_name("environment"),
            b"production",
        ));
        assert_eq!(policy.propagate(&headers).count(), 0);
        headers.push(TransportHeader::text(context_name("account"), b"customer"));
        let propagated = policy.propagate(&headers).collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace_id");

        let changed_condition = resolve_conditional_pipeline(
            "[{type: transport_header, name: workspace, store_as: workspace_id}, \
             {type: transport_header, name: account, store_as: account_id}, \
             {type: transport_header_match, name: environment, value: staging}]",
            "tenant:workspace_id",
        );
        let condition_candidate = factory
            .compile_candidate_context(&changed_condition, &installed.runtime_requirements)
            .expect("condition candidate compiles");
        assert!(
            !installed
                .bindings
                .pipeline_bindings_match(&condition_candidate.bindings, &pipeline)
        );

        let changed_member = resolve_conditional_pipeline(current_composite, "tenant:account_id");
        let member_candidate = factory
            .compile_candidate_context(&changed_member, &installed.runtime_requirements)
            .expect("member candidate compiles");
        assert!(
            !installed
                .bindings
                .pipeline_bindings_match(&member_candidate.bindings, &pipeline)
        );

        let changed_unselected = resolve_conditional_pipeline(
            &current_composite.replace("name: account,", "name: other_account,"),
            "tenant:workspace_id",
        );
        let unselected_candidate = factory
            .compile_candidate_context(&changed_unselected, &installed.runtime_requirements)
            .expect("changed presence gate compiles");
        assert!(
            !installed
                .bindings
                .pipeline_bindings_match(&unselected_candidate.bindings, &pipeline)
        );
    }

    /// Scenario: a live update reorders the conditions of a composite context entry.
    /// Guarantees: compilation canonicalizes condition order and preserves the installed binding.
    #[test]
    fn full_yaml_compilation_ignores_composite_condition_order() {
        let current = resolve_conditional_pipeline(
            "[{type: transport_header, name: workspace, store_as: workspace_id}, \
             {type: transport_header_match, name: environment, value: production}, \
             {type: transport_header_match, name: region, value: us-east}]",
            "tenant:workspace_id",
        );
        let reordered = resolve_conditional_pipeline(
            "[{type: transport_header, name: workspace, store_as: workspace_id}, \
             {type: transport_header_match, name: region, value: us-east}, \
             {type: transport_header_match, name: environment, value: production}]",
            "tenant:workspace_id",
        );
        let factory = test_pipeline_factory();
        let installed = factory
            .compile_initial_context(&current)
            .expect("initial context compiles");
        let candidate = factory
            .compile_candidate_context(&reordered, &installed.runtime_requirements)
            .expect("reordered context compiles");

        assert!(
            installed
                .bindings
                .pipeline_bindings_match(&candidate.bindings, &pipeline("default", "main"))
        );
    }

    /// Scenario: complete YAML contains an invalid qualified propagation selector.
    /// Guarantees: startup reports the unknown composite, unknown member, or unsupported type.
    #[test]
    fn full_yaml_compilation_reports_actionable_composite_selector_errors() {
        let cases = [
            (
                "[{type: transport_header, name: workspace, store_as: workspace_id}]",
                "missing:workspace_id",
                "unknown composite context entry `missing`",
            ),
            (
                "[{type: transport_header, name: workspace, store_as: workspace_id}]",
                "tenant:missing",
                "unknown context member `tenant:missing`",
            ),
            (
                "[{type: authorized_identity, name: customer_id}]",
                "tenant:customer_id",
                "context entry reference `tenant:customer_id` selects authorized-identity member `customer_id`, which cannot be propagated as a transport header",
            ),
        ];
        let factory = test_pipeline_factory();

        for (composite, selector, expected) in cases {
            let resolved = resolve_conditional_pipeline(composite, selector);
            let error = factory
                .compile_initial_context(&resolved)
                .expect_err("invalid selector must fail startup");
            let message = error.to_string();
            assert!(message.contains(expected), "{message}");
        }
    }

    /// Scenario: a receiver has an absent or explicitly empty authorized identity policy.
    /// Guarantees: neither form creates an authorized identity declaration or non-empty binding.
    #[test]
    fn empty_authorized_identity_policy_produces_no_binding() {
        let receiver = NodeUserConfig::new_receiver_config("urn:test:receiver:example");

        for policy in [None, Some(AuthorizedIdentityPolicy::default())] {
            let declarations =
                PipelineFactory::<()>::wrapper_context_declarations(&receiver, &None, &policy, &[])
                    .expect("wrapper declarations");
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
    /// Guarantees: bindings retain the policy and reject changed projections in either comparison direction.
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
    /// Guarantees: undeclared reads and nodes fail while the propagation declaration is retained.
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
        let declarations = HashMap::from([(
            pipeline.clone(),
            HashMap::from([(
                node.clone(),
                PreparedNodeContextDeclarations::new(declarations, &[])
                    .expect("valid declarations"),
            )]),
        )]);
        let requirements = ContextRuntimeRequirements::compile(&declarations);
        let bindings = CompiledContextBindings::compile(declarations, &requirements);

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
        let ContextDeclaration::HeaderPropagation {
            policy: propagation_policy,
        } = propagation_declaration
        else {
            unreachable!("test declaration is header propagation");
        };
        assert_eq!(
            bindings.header_propagation_policy(&pipeline, &node),
            Some(&propagation_policy)
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

    /// Scenario: unused composite definitions change while an exporter keeps the same binding.
    /// Guarantees: startup leaves unused definitions inert and candidate bindings remain compatible.
    #[test]
    fn full_yaml_compilation_ignores_unused_composites() {
        let composite = "[{type: transport_header, name: workspace}]";
        let original = conditional_pipeline_yaml(composite, "tenant:workspace");
        let changed = original.replace(
            "tenant: ",
            "unused: [{type: transport_header, name: unsupported:nested}]\n      tenant: ",
        );
        let resolve = |yaml: &str| {
            otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(yaml)
                .expect("valid config")
                .resolve()
        };
        let factory = test_pipeline_factory();
        let installed = factory
            .compile_initial_context(&resolve(&original))
            .expect("initial");
        let candidate = factory
            .compile_candidate_context(&resolve(&changed), &installed.runtime_requirements)
            .expect("unused nested definition stays inert");
        assert!(
            installed
                .bindings
                .pipeline_bindings_match(&candidate.bindings, &pipeline("default", "main"))
        );
    }

    /// Scenario: two exporters select independent composites using one name in different domains.
    /// Guarantees: each binding resolves only its own dependencies without cross-exporter collisions.
    #[test]
    fn full_yaml_compilation_keeps_exporter_layouts_independent() {
        let yaml = conditional_pipeline_yaml(
            "[{type: transport_header, name: workspace}]",
            "tenant:workspace",
        )
        .replace(
            "      tenant: ",
            "      other: [{type: authorized_identity, name: workspace}, {type: transport_header, name: account}]\n      tenant: ",
        )
        .replace(
            "        connections:",
            "          other_exporter:\n            type: urn:test:exporter:example\n            config: {}\n            header_propagation:\n              default:\n                selector: {type: named, named: ['other:account']}\n        connections:",
        )
        .replace(
            "            to: exporter",
            "            to: exporter\n          - from: receiver\n            to: other_exporter",
        );
        let resolved = otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(&yaml)
            .expect("valid pipeline")
            .resolve();
        let installed = test_pipeline_factory()
            .compile_initial_context(&resolved)
            .expect("independent dependencies compile");
        let key = pipeline("default", "main");
        let first = installed
            .bindings
            .header_propagation_policy(&key, &"exporter".into())
            .expect("first binding");
        let second = installed
            .bindings
            .header_propagation_policy(&key, &"other_exporter".into())
            .expect("second binding");
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(
            context_name("workspace"),
            b"untrusted",
        ));
        headers.push(TransportHeader::text(context_name("account"), b"selected"));
        assert_eq!(first.propagate(&headers).count(), 1);
        assert_eq!(second.propagate(&headers).count(), 0);
    }

    /// Scenario: node capture overrides mask a conflicting pipeline capture alias.
    /// Guarantees: binding compilation preserves capture precedence and original wire names.
    #[test]
    fn full_yaml_compilation_uses_effective_capture_and_retention() {
        let yaml = conditional_pipeline_yaml(
            "[{type: transport_header, name: WORKSPACE, store_as: workspace_id}]",
            "tenant:workspace_id",
        )
        .replace("name: stored_name", "name: preserve")
        .replace(
            "          receiver:\n",
            "          receiver:\n            header_capture:\n              headers:\n                - match_names: [X-Workspace]\n                  store_as: Workspace\n",
        )
        .replace(
            "policies:\n",
            "policies:\n  transport_headers:\n    header_capture:\n      headers:\n        - match_names: [X-Workspace]\n          store_as: customer\n  authorized_identity:\n    - claim: sub\n      store_as: customer\n",
        );
        let resolved = otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(&yaml)
            .expect("valid config")
            .resolve();
        let installed = test_pipeline_factory()
            .compile_initial_context(&resolved)
            .expect("effective aliases do not collide");
        let key = pipeline("default", "main");
        let capture = installed
            .bindings
            .header_capture_policy(&key, &"receiver".into())
            .expect("capture");
        let propagation = installed
            .bindings
            .header_propagation_policy(&key, &"exporter".into())
            .expect("propagation");
        let mut headers = TransportHeaders::new();
        assert!(
            capture
                .capture_from_pairs(
                    [("X-Workspace", b"acme".as_slice())].into_iter(),
                    &mut headers
                )
                .is_none()
        );
        let output = propagation.propagate(&headers).collect::<Vec<_>>();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].header_name, "X-Workspace");
        assert_eq!(headers.get(0).expect("captured").name.as_str(), "Workspace");
    }

    /// Scenario: capture aliases differ only by case and an unselected identity uses the same name.
    /// Guarantees: composite compilation does not impose a new namespace on unrelated source policies.
    #[test]
    fn full_yaml_compilation_preserves_independent_source_names() {
        let yaml = conditional_pipeline_yaml(
            "[{type: transport_header, name: workspace, store_as: workspace_id}]",
            "tenant:workspace_id",
        ).replace(
            "policies:\n",
            "policies:\n  transport_headers:\n    header_capture:\n      headers:\n        - {match_names: [x-first], store_as: Workspace}\n        - {match_names: [x-second], store_as: workspace}\n  authorized_identity:\n    - {claim: sub, store_as: workspace}\n",
        );
        let resolved = otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(&yaml)
            .expect("valid config")
            .resolve();
        let installed = test_pipeline_factory()
            .compile_initial_context(&resolved)
            .expect("independent source names remain valid");
        let key = pipeline("default", "main");
        let capture = installed
            .bindings
            .header_capture_policy(&key, &"receiver".into())
            .expect("capture");
        let policy = installed
            .bindings
            .header_propagation_policy(&key, &"exporter".into())
            .expect("propagation");
        let mut headers = TransportHeaders::new();
        assert!(
            capture
                .capture_from_pairs(
                    [
                        ("x-first", b"first".as_slice()),
                        ("x-second", b"second".as_slice())
                    ]
                    .into_iter(),
                    &mut headers,
                )
                .is_none()
        );
        let output = policy.propagate(&headers).collect::<Vec<_>>();
        assert_eq!(output.len(), 2);
        assert!(
            output
                .iter()
                .all(|header| header.header_name == "workspace_id")
        );
        assert_eq!(output[0].value, b"first");
        assert_eq!(output[1].value, b"second");
    }
}
