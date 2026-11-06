// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Collects and compiles context declarations before runtime construction.
//!
//! Primitive reads, writes, and all-captured selections always specify a source domain.
//! Composite selections instead name the composite and optionally its member; the
//! definition supplies each member's value source (a domain-backed field, configured
//! constant, or random generator) and the entire entry's presence gate.
//! Original wire names are supported only for transport-header values.

/// Runtime context policies shared independently of compiler bookkeeping.
mod bindings;
/// Editing processors.
mod edit;
/// Runtime failures of context binding operations.
mod error;
/// Compiles logical context layouts and resolves member projections.
mod layout;
/// Materializes selected composites at runtime.
mod materialization;
/// Merging processors.
mod merge;
/// Request-scoped message metadata carried by pdata.
mod metadata;
/// Compiles and applies exporter transport-header propagation policies.
mod propagation;
/// Node-facing reads over existing header storage and composite projections.
mod reads;

use PreparedHeaderPropagationPolicy as HeaderPropagationPolicy;
pub use bindings::{
    Arrival, ArrivalBinding, ArrivalHeaders, ClaimRef, Claims, NodeBindings, PropagatedValue,
    SinkBinding,
};
pub use edit::{EditBinding, EditEntry, EditValue};
pub use error::ContextError;
pub use layout::*;
pub use materialization::*;
pub use merge::{MergeBinding, MetadataMerger};
pub use metadata::{
    AuthorizedClaimValue, AuthorizedIdentityEntries, AuthorizedIdentityEntriesIter,
    AuthorizedIdentityEntry, Metadata, PeerAddrMerger,
};
pub use propagation::CompiledHeaderPropagationPolicy;
#[doc(hidden)]
pub use propagation::PreparedHeaderPropagationPolicy;
pub use reads::{HeaderValue, HeadersRead, Read, ReadMember, ReadValue, ValueRead};

use crate::PipelineFactory;
use crate::error::Error as EngineError;
use otel_arrow_dfe_config::context::authorized_identity::AuthorizedIdentityPolicy;
use otel_arrow_dfe_config::context::policy::{
    ContextEntryDeclaration as ConfigContextEntryDeclaration, ContextEntryPart,
};
use otel_arrow_dfe_config::context::transport_policy::{
    HeaderCapturePolicy, TransportHeadersPolicy,
};
use otel_arrow_dfe_config::engine::ResolvedOtelDataflowSpec;
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::node::{NodeKind, NodeUserConfig};
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
        /// Matching semantics used by the existing consumer.
        matching: ContextNameMatching,
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

/// Matching semantics for a declared primitive name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextNameMatching {
    /// Match the configured stored spelling exactly.
    Exact,
    /// Match ASCII case-insensitively, as Kafka topic routing does.
    AsciiCaseInsensitive,
}

/// One selected value source after resolving a primitive or composite target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectedSource<'a> {
    /// Value supplied by a domain-backed primitive field.
    Field(ContextDomain, &'a ContextEntryName),
    /// Inline configured value with no source domain or wire name.
    Constant(&'a ContextEntryName),
    /// Generated random value with no source domain or wire name.
    Randomness(&'a ContextEntryName),
}

impl<'a> SelectedSource<'a> {
    /// Resolves one value-bearing composite part, excluding conditions.
    fn from_part(part: &'a ContextEntryPart) -> Option<Self> {
        match part {
            ContextEntryPart::Constant { name, .. } => Some(Self::Constant(name)),
            ContextEntryPart::Randomness { name, .. } => Some(Self::Randomness(name)),
            ContextEntryPart::TransportHeader { name, .. } => {
                Some(Self::Field(ContextDomain::TransportHeader, name))
            }
            ContextEntryPart::AuthorizedIdentity { name, .. } => {
                Some(Self::Field(ContextDomain::AuthorizedIdentity, name))
            }
            ContextEntryPart::TransportHeaderMatch { .. } => None,
        }
    }
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

    /// Visits selected values, excluding condition-only fields.
    fn visit_selected_sources<'a>(
        &'a self,
        composites: &'a [ConfigContextEntryDeclaration],
        mut visit: impl FnMut(SelectedSource<'a>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        match self {
            Self::Primitive { domain, name, .. } => visit(SelectedSource::Field(*domain, name)),
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
                visit(
                    SelectedSource::from_part(part)
                        .expect("selected composite member is value-bearing"),
                )
            }
            Self::Composite { name } => {
                let declaration = composite_declaration(name, composites)?;
                for source in declaration
                    .definition
                    .0
                    .iter()
                    .filter_map(SelectedSource::from_part)
                {
                    visit(source)?;
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
    /// Number of values the consumer takes from each selected member.
    pub take: ContextEntryTake,
}

/// Specifies which already-stored values a consumer reads from each selected member.
///
/// Presence is independent of `take`: a read is present whenever its target
/// is present, so `Exists` reads stop at the first match without yielding values.
/// This does not change capture or storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextEntryTake {
    /// Presence only; members yield no values.
    Exists,
    /// The first value of each member, in source order.
    First,
    /// Every value of each member, in source order.
    All,
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

/// Selects named entries or all captured primitives within one source domain.
///
/// This is the compiled counterpart of user-configured context selection
/// (for example `PropagationSelectorType`). Selection vocabulary added here
/// must remain expressible in configuration; the operation applied to a
/// selection (read, hash, propagate, export) is chosen by the binding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextConsumerSelector {
    /// Selects named context entries in order.
    Entries {
        /// Entries to read.
        entries: Box<[ContextEntrySelector]>,
    },
    /// Selects every primitive in one domain using its stored name.
    ///
    /// Only transport headers support this selection; all-captured authorized
    /// identity reads are rejected during binding compilation.
    AllCaptured {
        /// Source authority domain.
        domain: ContextDomain,
    },
}

/// One step of a processor's `Edits` declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextAction {
    /// Appends the values the processor supplies for this transport header
    /// when it forms each output.
    Add {
        /// Produced transport-header entry.
        entry: ContextEntryName,
    },
}

/// A node's declared context behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextDeclaration {
    /// Declares that a receiver forms each new context at arrival.
    Arrival,
    /// Declares a static context producer (e.g. traffic generator).
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
    /// Declares that a processor forms each output context from one input context.
    Edits {
        /// Actions applied to the input.
        actions: Box<[ContextAction]>,
    },
    /// Declares that a processor forms each output context from many input contexts.
    Merges,
}

impl ContextDeclaration {
    /// Returns whether this is an exclusive kind of processor binding.
    fn is_processor_materialization(&self) -> bool {
        matches!(self, Self::Edits { .. } | Self::Merges)
    }

    /// Returns why a factory of `kind` may not return this declaration,
    /// printed after ", which"
    fn kind_violation(&self, kind: NodeKind) -> Option<&'static str> {
        match (self, kind) {
            (
                Self::HeaderCapture { .. }
                | Self::HeaderPropagation { .. }
                | Self::AuthorizedIdentityCapture { .. },
                _,
            ) => Some("is managed by the engine"),
            (Self::Consumes { .. }, NodeKind::Processor | NodeKind::Exporter) => None,
            (Self::Consumes { .. }, NodeKind::Receiver) => Some("is not allowed for receivers"),
            (Self::Arrival | Self::Produces { .. }, NodeKind::Receiver) => None,
            (Self::Arrival | Self::Produces { .. }, NodeKind::Processor | NodeKind::Exporter) => {
                Some("is only supported by receivers")
            }
            (Self::Edits { .. } | Self::Merges, NodeKind::Processor) => None,
            (Self::Edits { .. } | Self::Merges, NodeKind::Receiver | NodeKind::Exporter) => {
                Some("is only supported by processors")
            }
        }
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
                    entry.target.visit_selected_sources(composites, |source| {
                        if entry.form != ContextEntrySelectorForm::OriginalKeyValue {
                            return Ok(());
                        }
                        match source {
                            SelectedSource::Field(ContextDomain::TransportHeader, name) => {
                                _ = requirements
                                    .original_name_retention
                                    .overrides
                                    .insert(original_name_key(name), true);
                            }
                            SelectedSource::Field(domain, name) => {
                                return Err(invalid_context(format!(
                                    "original wire name requested for {domain:?} context entry `{name}`; only transport headers have original wire names"
                                )));
                            }
                            SelectedSource::Constant(name) => {
                                return Err(invalid_context(format!(
                                    "original wire name requested for constant context entry `{name}`; constants have no original wire names"
                                )));
                            }
                            SelectedSource::Randomness(name) => {
                                return Err(invalid_context(format!(
                                    "original wire name requested for randomness context entry `{name}`; generated values have no original wire names"
                                )));
                            }
                        }
                        Ok(())
                    })?;
                }
            }
            Self::Consumes {
                selector: ContextConsumerSelector::AllCaptured { .. },
            }
            | Self::Arrival
            | Self::Produces { .. }
            | Self::HeaderCapture { .. }
            | Self::AuthorizedIdentityCapture { .. }
            | Self::Edits { .. }
            | Self::Merges => {}
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

    /// Iterates over component-owned declarations, excluding engine-managed policies.
    fn component_declarations(&self) -> impl Iterator<Item = &ContextDeclaration> {
        self.iter().filter(|declaration| {
            !matches!(
                declaration,
                ContextDeclaration::HeaderCapture { .. }
                    | ContextDeclaration::HeaderPropagation { .. }
                    | ContextDeclaration::AuthorizedIdentityCapture { .. }
            )
        })
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

    /// Declares a receiver that forms each new context at arrival.
    #[must_use]
    pub const fn arriving() -> Self {
        Self {
            declarations: default_arrival_context_declarations,
        }
    }

    /// Default Context merging function that maintains only network_info.
    #[must_use]
    pub const fn merging() -> Self {
        Self {
            declarations: default_merge_context_declarations,
        }
    }
}

fn default_arrival_context_declarations(
    _: &serde_json::Value,
) -> Result<NodeContextDeclarations, Error> {
    Ok([ContextDeclaration::Arrival].into_iter().collect())
}

fn default_merge_context_declarations(
    _: &serde_json::Value,
) -> Result<NodeContextDeclarations, Error> {
    // TODO(3926): network_info is kept implicitly; users should be able to declare and disable it.
    Ok([ContextDeclaration::Merges].into_iter().collect())
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
    by_pipeline: HashMap<PipelineKey, HashMap<ConfigNodeId, CompiledNodeState>>,
    /// Selected composite materialization plans indexed by pipeline.
    materialization_by_pipeline: HashMap<PipelineKey, ContextMaterializationPlan>,
}

/// Stores one node's declarations, selected composites, and runtime bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledNodeState {
    /// Sorted component and engine declarations; factory validation uses the component-only view.
    declarations: NodeContextDeclarations,
    /// Selected composite definitions, including the whole presence gate for member selections.
    composites: Box<[ConfigContextEntryDeclaration]>,
    /// Runtime bindings shared with node construction without compiler bookkeeping.
    bindings: Arc<NodeBindings>,
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
            .flat_map(|declaration| {
                let mut names = Vec::new();
                match declaration {
                    ContextDeclaration::Consumes {
                        selector: ContextConsumerSelector::Entries { entries },
                    } => names.extend(
                        entries
                            .iter()
                            .filter_map(|entry| entry.target.composite_name())
                            .cloned(),
                    ),
                    ContextDeclaration::HeaderPropagation { policy } => {
                        policy.visit_composite_names(|name| names.push(name.clone()));
                    }
                    ContextDeclaration::Arrival
                    | ContextDeclaration::Produces { .. }
                    | ContextDeclaration::Consumes {
                        selector: ContextConsumerSelector::AllCaptured { .. },
                    }
                    | ContextDeclaration::HeaderCapture { .. }
                    | ContextDeclaration::AuthorizedIdentityCapture { .. }
                    | ContextDeclaration::Edits { .. }
                    | ContextDeclaration::Merges => {}
                }
                names
            })
            .collect::<BTreeSet<_>>();
        let composites = composite_names
            .into_iter()
            .map(|name| {
                let mut declaration = composite_declaration(&name, context)?.clone();
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

impl CompiledNodeState {
    /// Compiles runtime bindings while retaining their source declarations.
    fn compile(
        prepared: PreparedNodeContextDeclarations,
        requirements: &ContextRuntimeRequirements,
        materialization_plan: &ContextMaterializationPlan,
    ) -> Result<Self, Error> {
        let declarations = prepared.declarations;
        let mut header_capture = None;
        let mut header_propagation = None;
        let mut authorized_identity_capture = None;
        let mut arrival = false;
        let mut values = Vec::new();
        let mut headers = None;
        let mut edit = None;
        let mut merge = None;
        for declaration in declarations.iter() {
            match declaration {
                ContextDeclaration::Arrival => arrival = true,
                ContextDeclaration::Consumes { selector } => match selector {
                    ContextConsumerSelector::Entries { entries } => {
                        for entry in entries {
                            values.push(ValueRead::compile(entry, materialization_plan)?);
                        }
                    }
                    ContextConsumerSelector::AllCaptured { domain } => match domain {
                        ContextDomain::TransportHeader => headers = Some(HeadersRead::new()),
                        ContextDomain::AuthorizedIdentity => {
                            return Err(invalid_context(
                                "all-captured authorized identity reads are not supported; use explicit named identity reads",
                            ));
                        }
                    },
                },
                ContextDeclaration::Produces { domain, entry } => match domain {
                    ContextDomain::TransportHeader => {}
                    ContextDomain::AuthorizedIdentity => {
                        return Err(invalid_context(format!(
                            "produced context entry `{entry}` is in the authorized-identity \
                                 domain; only transport-header entries can be produced"
                        )));
                    }
                },
                ContextDeclaration::HeaderCapture { policy } => {
                    header_capture = Some(
                        policy
                            .clone()
                            .compile(|name| requirements.preserves_original_name(name)),
                    );
                }
                ContextDeclaration::HeaderPropagation { policy } => {
                    header_propagation = Some(
                        policy
                            .clone()
                            .bind(materialization_plan)
                            .map_err(invalid_context)?,
                    );
                }
                ContextDeclaration::AuthorizedIdentityCapture { policy } => {
                    authorized_identity_capture = Some(policy.clone());
                }
                ContextDeclaration::Edits { actions } => {
                    edit = Some(EditBinding::compile(actions)?);
                }
                ContextDeclaration::Merges => {
                    merge = Some(MergeBinding);
                }
            }
        }

        let arrival = arrival.then_some(ArrivalBinding {
            header_capture,
            authorized_identity_capture,
            materialization: materialization_plan.clone(),
        });

        let sink = header_propagation.map(|policy| SinkBinding { policy });

        values.sort_by(|left: &ValueRead, right| left.selector().cmp(right.selector()));
        values.dedup_by(|left, right| left.selector() == right.selector());
        Ok(Self {
            declarations,
            composites: prepared.composites,
            bindings: Arc::new(NodeBindings {
                arrival,
                sink,
                values: values.into_boxed_slice(),
                headers,
                edit,
                merge,
            }),
        })
    }

    /// Returns whether the node has no component or engine context declarations.
    fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Compares source declarations and selected definitions, independently of runtime bindings.
    fn declarations_match(&self, other: &Self) -> bool {
        self.declarations == other.declarations && self.composites == other.composites
    }
}

impl CompiledContextBindings {
    /// Creates an empty binding set that rejects all node declaration validation.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            by_pipeline: HashMap::new(),
            materialization_by_pipeline: HashMap::new(),
        }
    }

    /// Compiles every node's bindings using the same engine-wide runtime requirements.
    fn compile(
        declarations: ContextDeclarationsByPipeline,
        requirements: &ContextRuntimeRequirements,
    ) -> Result<Self, Error> {
        let mut materialization_by_pipeline = HashMap::new();
        let by_pipeline = declarations
            .into_iter()
            .map(|(pipeline, prepared_nodes)| {
                let selected_composites = prepared_nodes
                    .values()
                    .flat_map(|prepared| prepared.composites.iter())
                    .map(|declaration| (declaration.name.clone(), declaration.clone()))
                    .collect::<BTreeMap<_, _>>()
                    .into_values()
                    .collect::<Vec<_>>();
                let materialization = ContextMaterializationPlan::compile(&selected_composites)?;
                if !materialization.is_empty() {
                    _ = materialization_by_pipeline
                        .insert(pipeline.clone(), materialization.clone());
                }
                let nodes = prepared_nodes
                    .into_iter()
                    .map(|(node, declarations)| {
                        Ok((
                            node,
                            CompiledNodeState::compile(
                                declarations,
                                requirements,
                                &materialization,
                            )?,
                        ))
                    })
                    .collect::<Result<_, Error>>()?;
                Ok((pipeline, nodes))
            })
            .collect::<Result<_, Error>>()?;

        Ok(Self {
            by_pipeline,
            materialization_by_pipeline,
        })
    }

    /// Returns the pipeline's selected composite materialization plan.
    ///
    /// Legacy consumers may resolve existing [`ContextProjection`] values against
    /// this plan. New consumers obtain compiled reads from [`NodeBindings`].
    #[must_use]
    pub fn context_materialization_plan(
        &self,
        pipeline: &PipelineKey,
    ) -> Option<&ContextMaterializationPlan> {
        self.materialization_by_pipeline.get(pipeline)
    }

    /// Returns the shared runtime bindings compiled for one node.
    #[must_use]
    pub fn node_bindings(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
    ) -> Option<&Arc<NodeBindings>> {
        Some(&self.by_pipeline.get(pipeline)?.get(node)?.bindings)
    }

    /// Conservatively checks one pipeline's contribution to live-update context compatibility.
    ///
    /// Compares nodes that have component- or engine-owned context declarations.
    /// Changes to other nodes remain subject to normal pipeline rollout checks.
    ///
    /// Equality of declarations and selected composite definitions is a proxy for
    /// preserving shared context assumptions, not an invariant in itself. It can
    /// reject safe changes: the compiler cannot yet prove which changes remain
    /// compatible with surviving generations across topic boundaries (#4611).
    #[must_use]
    pub fn pipeline_context_compatible(&self, other: &Self, pipeline: &PipelineKey) -> bool {
        let current = self.by_pipeline.get(pipeline);
        let candidate = other.by_pipeline.get(pipeline);
        let current_node_count = current
            .into_iter()
            .flat_map(|nodes| nodes.values())
            .filter(|node| !node.is_empty())
            .count();
        let candidate_node_count = candidate
            .into_iter()
            .flat_map(|nodes| nodes.values())
            .filter(|node| !node.is_empty())
            .count();

        current_node_count == candidate_node_count
            && current
                .into_iter()
                .flat_map(|nodes| nodes.iter())
                .all(|(node_id, node)| {
                    node.is_empty()
                        || candidate
                            .and_then(|nodes| nodes.get(node_id))
                            .is_some_and(|candidate_node| candidate_node.declarations_match(node))
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
            Some(expected)
                if expected
                    .declarations
                    .component_declarations()
                    .eq(declarations.iter()) =>
            {
                Ok(())
            }
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
        let bindings = Self::compile_bindings(declarations, &runtime_requirements)?;
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
        let bindings = Self::compile_bindings(declarations, installed_requirements)?;
        Ok(PreparedContext {
            runtime_requirements,
            bindings,
        })
    }

    /// Wraps compiled bindings in an immutable shared handle for pipeline runtimes.
    fn compile_bindings(
        declarations: ContextDeclarationsByPipeline,
        requirements: &ContextRuntimeRequirements,
    ) -> Result<Arc<CompiledContextBindings>, EngineError> {
        CompiledContextBindings::compile(declarations, requirements)
            .map(Arc::new)
            .map_err(|error| EngineError::ConfigError(Box::new(error)))
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

    /// Validates node configuration and rejects declarations invalid for the node kind.
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
        let config_error =
            |error: String| EngineError::ConfigError(Box::new(Error::InvalidUserConfig { error }));
        for declaration in declarations.iter() {
            if let Some(reason) = declaration.kind_violation(kind) {
                return Err(config_error(format!(
                    "node factory `{urn}` returned a `{declaration:?}` context declaration, \
                     which {reason}"
                )));
            }
        }
        if declarations
            .iter()
            .filter(|declaration| declaration.is_processor_materialization())
            .count()
            > 1
        {
            return Err(config_error(format!(
                "node factory `{urn}` declared more than one context edit or merge"
            )));
        }
        if declarations
            .iter()
            .any(|declaration| matches!(declaration, ContextDeclaration::Arrival))
            && declarations
                .iter()
                .any(|declaration| matches!(declaration, ContextDeclaration::Produces { .. }))
        {
            return Err(config_error(format!(
                "node factory `{urn}` declared both arrival and synthetic context production"
            )));
        }
        Ok(declarations)
    }
}

/// Exercises declaration compilation, retention requirements, and live-update compatibility.
#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::context::transport_headers::TransportHeaders;
    use otel_arrow_dfe_config::context::transport_policy::HeaderPropagationPolicy as HeaderPropagationConfig;
    use otel_arrow_dfe_config::context::transport_policy::{CaptureDefaults, CaptureRule};

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
                                matching: ContextNameMatching::Exact,
                            },
                        },
                        form: if self.original {
                            ContextEntrySelectorForm::OriginalKeyValue
                        } else {
                            ContextEntrySelectorForm::Value
                        },
                        take: ContextEntryTake::All,
                    }]
                    .into_boxed_slice(),
                },
            }]
            .into_iter()
            .collect()
        }
    }

    /// Typed component configuration that declares a produced entry and/or arrival.
    #[derive(serde::Deserialize)]
    struct TestProducerConfig {
        /// Produced entry name.
        #[serde(default)]
        entry: Option<ContextEntryName>,
        /// Whether the entry is declared in the authorized-identity domain.
        #[serde(default)]
        identity: bool,
        /// Whether the node declares arrival.
        #[serde(default)]
        arrival: bool,
    }

    impl ConfigNodeContextDeclaration for TestProducerConfig {
        /// Declares the configured produced entry and arrival, when present.
        fn context_declarations(&self) -> NodeContextDeclarations {
            self.entry
                .iter()
                .map(|entry| ContextDeclaration::Produces {
                    domain: if self.identity {
                        ContextDomain::AuthorizedIdentity
                    } else {
                        ContextDomain::TransportHeader
                    },
                    entry: entry.clone(),
                })
                .chain(self.arrival.then_some(ContextDeclaration::Arrival))
                .collect()
        }
    }

    /// Typed processor configuration that edits in one produced entry.
    #[derive(serde::Deserialize)]
    struct TestEditConfig {
        /// Added entry name.
        entry: ContextEntryName,
        /// Whether the node also declares a merge.
        #[serde(default)]
        merge: bool,
    }

    impl ConfigNodeContextDeclaration for TestEditConfig {
        /// Declares an edit adding the configured entry and an optional merge.
        fn context_declarations(&self) -> NodeContextDeclarations {
            [ContextDeclaration::Edits {
                actions: Box::new([ContextAction::Add {
                    entry: self.entry.clone(),
                }]),
            }]
            .into_iter()
            .chain(self.merge.then_some(ContextDeclaration::Merges))
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

    /// Collects selected sources for assertions over visitor traversal.
    fn collect_selected_sources<'a>(
        target: &'a ContextEntryTarget,
        composites: &'a [ConfigContextEntryDeclaration],
    ) -> Vec<SelectedSource<'a>> {
        let mut sources = Vec::new();
        target
            .visit_selected_sources(composites, |source| {
                sources.push(source);
                Ok(())
            })
            .expect("selected sources");
        sources
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
    static TEST_RECEIVERS: [crate::ReceiverFactory<()>; 4] = [
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
        crate::ReceiverFactory {
            name: "urn:test:receiver:produce",
            create: unused_test_receiver,
            context_declarations: Some(ContextDeclarationProvider::from_typed_config::<
                TestProducerConfig,
            >()),
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
                TestProducerConfig,
            >,
        },
        crate::ReceiverFactory {
            name: "urn:test:receiver:consumes",
            create: unused_test_receiver,
            context_declarations: Some(ContextDeclarationProvider::from_typed_config::<
                TestDeclarationConfig,
            >()),
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
                TestDeclarationConfig,
            >,
        },
    ];

    /// Registers ordinary exporters and a typed context consumer for compilation tests.
    static TEST_EXPORTERS: [crate::ExporterFactory<()>; 6] = [
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
        crate::ExporterFactory {
            name: "urn:test:exporter:produces",
            create: unused_test_exporter,
            context_declarations: Some(ContextDeclarationProvider::from_typed_config::<
                TestProducerConfig,
            >()),
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
                TestProducerConfig,
            >,
        },
        crate::ExporterFactory {
            name: "urn:test:exporter:merge",
            create: unused_test_exporter,
            context_declarations: Some(ContextDeclarationProvider::merging()),
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
    ];

    /// Registers the type router and processors that produce or merge context.
    static TEST_PROCESSORS: [crate::ProcessorFactory<()>; 5] = [
        crate::ProcessorFactory {
            name: "urn:otel:processor:type_router",
            create: unused_test_processor,
            context_declarations: None,
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: accept_test_config,
        },
        crate::ProcessorFactory {
            name: "urn:test:processor:plain",
            create: unused_test_processor,
            context_declarations: None,
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
        crate::ProcessorFactory {
            name: "urn:test:processor:edit",
            create: unused_test_processor,
            context_declarations: Some(ContextDeclarationProvider::from_typed_config::<
                TestEditConfig,
            >()),
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
                TestEditConfig,
            >,
        },
        crate::ProcessorFactory {
            name: "urn:test:processor:produces",
            create: unused_test_processor,
            context_declarations: Some(ContextDeclarationProvider::from_typed_config::<
                TestProducerConfig,
            >()),
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
                TestProducerConfig,
            >,
        },
        crate::ProcessorFactory {
            name: "urn:test:processor:merge",
            create: unused_test_processor,
            context_declarations: Some(ContextDeclarationProvider::merging()),
            wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
            validate_config: otel_arrow_dfe_config::validation::no_config,
        },
    ];

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
            .expect("test bindings compile")
    }

    /// Scenario: a compiler snapshot containing node policies is cloned.
    /// Guarantees: snapshot clones share the same node bindings instead of copying their policies.
    #[test]
    fn compiler_snapshot_clones_share_node_bindings() {
        let compiled = compiled_bindings(
            [
                ContextDeclaration::HeaderCapture {
                    policy: HeaderCapturePolicy::default(),
                },
                ContextDeclaration::HeaderPropagation {
                    policy: HeaderPropagationPolicy::default(),
                },
                ContextDeclaration::AuthorizedIdentityCapture {
                    policy: serde_json::from_value(serde_json::json!([
                        {"claim": "sub", "store_as": "tenant"}
                    ]))
                    .expect("identity policy"),
                },
            ]
            .into_iter()
            .collect(),
        );
        let pipeline = pipeline("group", "pipeline");
        let node = ConfigNodeId::from("node");
        let bindings = compiled
            .node_bindings(&pipeline, &node)
            .expect("node bindings");
        let cloned = compiled.clone();
        assert!(Arc::ptr_eq(
            bindings,
            cloned
                .node_bindings(&pipeline, &node)
                .expect("cloned bindings"),
        ));
    }

    /// Scenario: runtime capture, identity, and propagation policies are removed from a compiled snapshot.
    /// Guarantees: live updates compare retained declarations, including node participation, not runtime policies.
    #[test]
    fn live_update_ignores_compiled_engine_policies() {
        let current = compiled_bindings(
            [
                ContextDeclaration::HeaderCapture {
                    policy: HeaderCapturePolicy::default(),
                },
                ContextDeclaration::AuthorizedIdentityCapture {
                    policy: serde_json::from_value(serde_json::json!([
                        {"claim": "sub", "store_as": "tenant"}
                    ]))
                    .expect("identity policy"),
                },
                ContextDeclaration::HeaderPropagation {
                    policy: HeaderPropagationPolicy::default(),
                },
            ]
            .into_iter()
            .collect(),
        );
        let key = pipeline("group", "pipeline");
        let mut without_policies = current.clone();
        without_policies
            .by_pipeline
            .get_mut(&key)
            .expect("pipeline")
            .get_mut(&ConfigNodeId::from("node"))
            .expect("node")
            .bindings = Arc::new(NodeBindings::default());

        assert!(current.pipeline_context_compatible(&without_policies, &key));
        assert!(without_policies.pipeline_context_compatible(&current, &key));
    }

    /// Scenario: capture declarations differ only in wire-name casing and compile to identical policies.
    /// Guarantees: live updates reject different declarations even when compilation erases the difference.
    #[test]
    fn live_update_rejects_different_declarations_with_equivalent_policies() {
        let compile = |wire_name| {
            compiled_bindings(
                [
                    ContextDeclaration::Arrival,
                    ContextDeclaration::HeaderCapture {
                        policy: HeaderCapturePolicy::new(
                            CaptureDefaults::default(),
                            vec![CaptureRule {
                                match_names: vec![context_name(wire_name)],
                                store_as: None,
                                sensitive: false,
                                value_kind: None,
                            }],
                        ),
                    },
                ]
                .into_iter()
                .collect(),
            )
        };
        let current = compile("X-Tenant");
        let candidate = compile("x-tenant");
        let key = pipeline("group", "pipeline");
        let node = ConfigNodeId::from("node");
        assert_eq!(
            current
                .node_bindings(&key, &node)
                .expect("current node bindings")
                .header_capture_policy(),
            candidate
                .node_bindings(&key, &node)
                .expect("candidate node bindings")
                .header_capture_policy()
        );
        assert!(!current.pipeline_context_compatible(&candidate, &key));
        assert!(!candidate.pipeline_context_compatible(&current, &key));
    }

    /// Scenario: read adapters are derived from declarations that specify primitive matching semantics.
    /// Guarantees: live updates compare declarations, not derived adapters, and detect matching changes.
    #[test]
    fn read_adapters_preserve_live_update_inputs() {
        let compile = |matching| {
            compiled_bindings(
                [ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries {
                        entries: Box::new([ContextEntrySelector {
                            target: ContextEntryTarget::Primitive {
                                domain: ContextDomain::TransportHeader,
                                name: context_name("tenant"),
                                matching,
                            },
                            form: ContextEntrySelectorForm::Value,
                            take: ContextEntryTake::All,
                        }]),
                    },
                }]
                .into_iter()
                .collect(),
            )
        };
        let current = compile(ContextNameMatching::Exact);
        let mut without_adapter = current.clone();
        let key = pipeline("group", "pipeline");
        let node = without_adapter
            .by_pipeline
            .get_mut(&key)
            .expect("pipeline")
            .get_mut(&ConfigNodeId::from("node"))
            .expect("node");
        Arc::make_mut(&mut node.bindings).values = Box::new([]);
        assert!(current.pipeline_context_compatible(&without_adapter, &key));
        assert!(without_adapter.pipeline_context_compatible(&current, &key));
        let different_matching = compile(ContextNameMatching::AsciiCaseInsensitive);
        assert!(!current.pipeline_context_compatible(&different_matching, &key));
        assert!(!different_matching.pipeline_context_compatible(&current, &key));
    }

    /// Scenario: arrival adapters add pipeline materialization without changing legacy declarations or policies.
    /// Guarantees: receivers without declarations compile no arrival binding, and live-update
    /// comparison still ignores them and adapter-derived plans.
    #[test]
    fn arrival_adapters_preserve_live_update_inputs() {
        let compiled = test_pipeline_factory()
            .compile_initial_context(&resolve_conditional_pipeline(
                "[{type: constant, name: route, value: fixed}]",
                "tenant:route",
            ))
            .expect("context compiles");
        let key = pipeline("default", "main");
        let receiver = ConfigNodeId::from("receiver");
        let mut without_receiver = compiled.bindings.as_ref().clone();
        let removed = without_receiver
            .by_pipeline
            .get_mut(&key)
            .expect("pipeline")
            .remove(&receiver)
            .expect("receiver");
        assert!(removed.bindings.arrival().is_none());
        assert!(
            compiled
                .bindings
                .pipeline_context_compatible(&without_receiver, &key)
        );
        assert!(without_receiver.pipeline_context_compatible(&compiled.bindings, &key));

        let current = compiled_bindings(
            [ContextDeclaration::HeaderCapture {
                policy: HeaderCapturePolicy::default(),
            }]
            .into_iter()
            .collect(),
        );
        let mut changed = current.clone();
        let key = pipeline("group", "pipeline");
        let node = changed
            .by_pipeline
            .get_mut(&key)
            .expect("pipeline")
            .get_mut(&ConfigNodeId::from("node"))
            .expect("node");
        Arc::make_mut(&mut node.bindings).set_materialization_plan(
            compiled
                .bindings
                .context_materialization_plan(&pipeline("default", "main"))
                .expect("materialization plan")
                .clone(),
        );
        assert!(current.pipeline_context_compatible(&changed, &key));
        assert!(changed.pipeline_context_compatible(&current, &key));
    }

    /// Resolves a pipeline with producing, editing, merging, and plain nodes in one chain.
    fn resolve_role_pipeline() -> ResolvedOtelDataflowSpec {
        otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(
            r#"
version: otel_dataflow/v1
engine: {}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:receiver:produce"
            config: {arrival: true}
          edit:
            type: "urn:test:processor:edit"
            config: {entry: partition}
          merge:
            type: "urn:test:processor:merge"
            config: {}
          plain:
            type: "urn:test:processor:plain"
            config: {}
          exporter:
            type: "urn:test:exporter:example"
            config: {}
        connections:
          - from: receiver
            to: edit
          - from: edit
            to: merge
          - from: merge
            to: plain
          - from: plain
            to: exporter
"#,
        )
        .expect("role pipeline YAML is valid")
        .resolve()
    }

    /// Scenario: a pipeline contains arriving, editing, merging, and plain nodes.
    /// Guarantees: the receiver gets an arrival binding because it declares one, the editing processor
    /// gets an edit for only its declared entries, and the merging processor
    /// gets a merge binding. Plain nodes get neither edit nor merge bindings.
    #[test]
    fn edit_and_merge_bindings_follow_declarations() {
        let compiled = test_pipeline_factory()
            .compile_initial_context(&resolve_role_pipeline())
            .expect("context compiles");
        let key = pipeline("default", "main");
        let bindings = |node: &'static str| {
            Arc::clone(
                compiled
                    .bindings
                    .node_bindings(&key, &ConfigNodeId::from(node))
                    .expect("node bindings"),
            )
        };

        let receiver = bindings("receiver");
        assert!(receiver.arrival().is_some());
        assert!(receiver.merge().is_none());
        assert!(receiver.edit().is_none());

        let edit = bindings("edit");
        assert!(edit.arrival().is_none());
        assert!(edit.merge().is_none());
        let edit = edit.edit().expect("processor edit");
        assert!(
            edit.entry(ContextDomain::TransportHeader, &context_name("partition"))
                .is_ok()
        );
        assert!(
            edit.entry(ContextDomain::TransportHeader, &context_name("generated"))
                .is_err()
        );

        let merge = bindings("merge");
        assert!(merge.merge().is_some());
        assert!(merge.edit().is_none());
        assert!(merge.arrival().is_none());

        for node in ["plain", "exporter"] {
            let bindings = bindings(node);
            assert!(bindings.edit().is_none(), "{node} edit");
            assert!(bindings.merge().is_none(), "{node} merge");
        }
    }

    /// Scenario: factories violate node-kind restrictions, declare both Edits and Merges, or
    /// declare both Arrival and Produces.
    /// Guarantees: each declaration set is rejected with the factory URN and violated restriction.
    #[test]
    fn invalid_node_context_declarations_are_rejected() {
        let factory = test_pipeline_factory();
        for (kind, urn, config, reason) in [
            (
                NodeKind::Receiver,
                "urn:test:receiver:consumes",
                serde_json::json!({"entry": "tenant"}),
                "not allowed for receivers",
            ),
            (
                NodeKind::Exporter,
                "urn:test:exporter:produces",
                serde_json::json!({"entry": "tenant"}),
                "only supported by receivers",
            ),
            (
                NodeKind::Exporter,
                "urn:test:exporter:produces",
                serde_json::json!({"arrival": true}),
                "`Arrival` context declaration, which is only supported by receivers",
            ),
            (
                NodeKind::Receiver,
                "urn:test:receiver:produce",
                serde_json::json!({"entry": "tenant", "arrival": true}),
                "declared both arrival and synthetic context production",
            ),
            (
                NodeKind::Exporter,
                "urn:test:exporter:merge",
                serde_json::json!({}),
                "only supported by processors",
            ),
            (
                NodeKind::Processor,
                "urn:test:processor:produces",
                serde_json::json!({"entry": "tenant"}),
                "only supported by receivers",
            ),
            (
                NodeKind::Processor,
                "urn:test:processor:edit",
                serde_json::json!({"entry": "tenant", "merge": true}),
                "more than one context edit or merge",
            ),
        ] {
            let Err(EngineError::ConfigError(error)) =
                factory.node_context_declarations(kind, urn, &config)
            else {
                panic!("`{urn}` declarations must be rejected");
            };
            let message = error.to_string();
            assert!(message.contains(urn), "{message}");
            assert!(message.contains(reason), "{message}");
        }
    }

    /// Scenario: a receiver factory declares a produced entry in the authorized-identity domain.
    /// Guarantees: compilation fails naming the entry and domain instead of deferring the failure to node construction.
    #[test]
    fn produced_identity_entries_are_rejected_at_compilation() {
        let config = otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(
            r#"
version: otel_dataflow/v1
engine: {}
groups:
  default:
    pipelines:
      main:
        nodes:
          receiver:
            type: "urn:test:receiver:produce"
            config: {entry: subject, identity: true}
          exporter:
            type: "urn:test:exporter:example"
            config: {}
        connections:
          - from: receiver
            to: exporter
"#,
        )
        .expect("identity pipeline YAML is valid")
        .resolve();
        let Err(EngineError::ConfigError(error)) =
            test_pipeline_factory().compile_initial_context(&config)
        else {
            panic!("produced identity entries must be rejected");
        };
        let message = error.to_string();
        assert!(message.contains("`subject`"), "{message}");
        assert!(message.contains("authorized-identity domain"), "{message}");
        assert!(
            message.contains("only transport-header entries can be produced"),
            "{message}"
        );
    }

    /// Scenario: edit and merge adapters are removed from a compiled snapshot.
    /// Guarantees: live-update comparison ignores the adapters and compares a merging node by its Merges declaration.
    #[test]
    fn edit_and_merge_adapters_preserve_live_update_inputs() {
        let compiled = test_pipeline_factory()
            .compile_initial_context(&resolve_role_pipeline())
            .expect("context compiles");
        let key = pipeline("default", "main");
        let mut without_adapters = compiled.bindings.as_ref().clone();
        let nodes = without_adapters
            .by_pipeline
            .get_mut(&key)
            .expect("pipeline");
        assert_eq!(
            nodes[&ConfigNodeId::from("merge")].declarations,
            [ContextDeclaration::Merges].into_iter().collect()
        );
        for node in nodes.values_mut() {
            let bindings = Arc::make_mut(&mut node.bindings);
            bindings.edit = None;
            bindings.merge = None;
        }
        assert!(
            compiled
                .bindings
                .pipeline_context_compatible(&without_adapters, &key)
        );
        assert!(without_adapters.pipeline_context_compatible(&compiled.bindings, &key));
    }

    /// Scenario: node contexts share names across groups and pipelines, or reference unknown nodes.
    /// Guarantees: lookup uses the full pipeline/node key, shares the selected binding, and returns None for misses.
    #[test]
    fn node_bindings_lookup_uses_current_pipeline_and_node() {
        use crate::context::ControllerContext;
        use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

        let keys = [
            ("group", "pipeline", "node"),
            ("other", "pipeline", "node"),
            ("group", "other", "node"),
            ("group", "pipeline", "other"),
        ];
        let mut declarations = ContextDeclarationsByPipeline::new();
        for (group, pipeline_id, node) in keys {
            _ = declarations
                .entry(pipeline(group, pipeline_id))
                .or_default()
                .insert(
                    node.into(),
                    PreparedNodeContextDeclarations::new(NodeContextDeclarations::default(), &[])
                        .expect("empty declarations"),
                );
        }
        let requirements = ContextRuntimeRequirements::compile(&declarations);
        let compiled = Arc::new(
            CompiledContextBindings::compile(declarations, &requirements)
                .expect("bindings compile"),
        );
        let controller = ControllerContext::new(TelemetryRegistryHandle::new());
        for (group, pipeline_id, node) in keys.into_iter().chain([
            ("missing", "pipeline", "node"),
            ("group", "missing", "node"),
            ("group", "pipeline", "missing"),
        ]) {
            let mut context =
                controller.pipeline_context_with(group.into(), pipeline_id.into(), 0, 1, 0);
            context.set_compiled_context_bindings(Arc::clone(&compiled));
            let context = context.with_node_context(
                node.into(),
                "urn:otel:processor:test".into(),
                NodeKind::Processor,
                HashMap::new(),
            );
            let pipeline = pipeline(group, pipeline_id);
            let node_id = ConfigNodeId::from(node);
            let expected = compiled
                .by_pipeline
                .get(&pipeline)
                .and_then(|nodes| nodes.get(&node_id))
                .map(|state| &state.bindings);
            match (
                context.node_bindings(),
                compiled.node_bindings(&pipeline, &node_id),
                expected,
            ) {
                (Some(actual), Some(lookup), Some(expected)) => {
                    assert!(Arc::ptr_eq(&actual, expected));
                    assert!(Arc::ptr_eq(lookup, expected));
                }
                (None, None, None) => {}
                _ => panic!("unexpected binding for {group}/{pipeline_id}/{node}"),
            }
        }
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
            matching: ContextNameMatching::Exact,
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
                entries: Box::new([ContextEntrySelector {
                    target,
                    form,
                    take: ContextEntryTake::All,
                }]),
            },
        }]
        .into_iter()
        .collect()
    }

    /// Builds a conditional test composite with aliased header and identity members.
    fn mixed_composite() -> ConfigContextEntryDeclaration {
        use otel_arrow_dfe_config::context::policy::{ContextEntryDefinition, ContextScope};

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

    /// Builds a test composite containing one constant and one header member.
    fn constant_composite() -> ConfigContextEntryDeclaration {
        use otel_arrow_dfe_config::context::policy::{ContextEntryDefinition, ContextScope};

        ConfigContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: context_name("route"),
            definition: serde_json::from_value::<ContextEntryDefinition>(serde_json::json!([
                {"type": "constant", "name": "route_name", "value": "otlp-http-json"},
                {"type": "transport_header", "name": "workspace"}
            ]))
            .expect("valid constant composite"),
        }
    }

    /// Builds a test composite containing one randomness member.
    fn randomness_composite() -> ConfigContextEntryDeclaration {
        use otel_arrow_dfe_config::context::policy::{ContextEntryDefinition, ContextScope};

        ConfigContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: context_name("idempotency"),
            definition: serde_json::from_value::<ContextEntryDefinition>(serde_json::json!([
                {"type": "randomness", "name": "id", "value": "uuid7"}
            ]))
            .expect("valid randomness composite"),
        }
    }

    /// Scenario: same-name sources are declared in different domains for named reads.
    /// Guarantees: deduplication, node validation, and live binding comparison preserve the domain.
    #[test]
    fn declaration_domains_are_part_of_binding_identity() {
        let declarations = |domain| -> NodeContextDeclarations {
            [ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: Box::new([ContextEntrySelector {
                        target: primitive_target(domain, "id"),
                        form: ContextEntrySelectorForm::Value,
                        take: ContextEntryTake::All,
                    }]),
                },
            }]
            .into_iter()
            .collect()
        };
        let header = declarations(ContextDomain::TransportHeader);
        let identity = declarations(ContextDomain::AuthorizedIdentity);
        let combined: NodeContextDeclarations =
            header.iter().chain(identity.iter()).cloned().collect();
        assert_eq!(combined.len(), 2);
        assert!(
            !context_runtime_requirements(combined).preserves_original_name(&context_name("id"))
        );

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
        assert!(!installed.pipeline_context_compatible(&candidate, &key));
        assert!(!candidate.pipeline_context_compatible(&installed, &key));
    }

    /// Scenario: a node declares all-captured identity reads, alone or alongside all-captured headers.
    /// Guarantees: binding compilation rejects the unsupported identity selection instead of ignoring it.
    #[test]
    fn all_captured_identity_reads_are_rejected() {
        for domains in [
            vec![ContextDomain::AuthorizedIdentity],
            vec![
                ContextDomain::TransportHeader,
                ContextDomain::AuthorizedIdentity,
            ],
        ] {
            let declarations = declarations_by_pipeline(
                domains
                    .into_iter()
                    .map(|domain| ContextDeclaration::Consumes {
                        selector: ContextConsumerSelector::AllCaptured { domain },
                    })
                    .collect(),
            );
            let requirements = ContextRuntimeRequirements::compile(&declarations);
            assert!(matches!(
                CompiledContextBindings::compile(declarations, &requirements),
                Err(Error::InvalidUserConfig { error })
                    if error.contains("all-captured authorized identity reads are not supported")
                        && error.contains("use explicit named identity reads")
            ));
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
        let source_name = context_name("id");
        assert_eq!(
            collect_selected_sources(&whole, &context),
            [
                SelectedSource::Field(ContextDomain::TransportHeader, &source_name),
                SelectedSource::Field(ContextDomain::AuthorizedIdentity, &source_name),
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

    /// Scenario: a consumer selects constant and field members or their entire composite.
    /// Guarantees: constants add no external source requirements and reject original wire names.
    #[test]
    fn constant_members_have_no_external_or_original_name_requirements() {
        let context = [constant_composite()];
        let whole = ContextEntryTarget::Composite {
            name: context_name("route"),
        };
        let constant_name = context_name("route_name");
        let header_name = context_name("workspace");
        assert_eq!(
            collect_selected_sources(&whole, &context),
            [
                SelectedSource::Constant(&constant_name),
                SelectedSource::Field(ContextDomain::TransportHeader, &header_name),
            ]
        );

        let prepared = PreparedNodeContextDeclarations::new(
            consumer(
                member_target("route", "route_name"),
                ContextEntrySelectorForm::Value,
            ),
            &context,
        )
        .expect("constant value selection");
        assert_eq!(prepared.requirements, ContextRuntimeRequirements::none());
        for target in [member_target("route", "route_name"), whole] {
            let error = PreparedNodeContextDeclarations::new(
                consumer(target, ContextEntrySelectorForm::OriginalKeyValue),
                &context,
            )
            .expect_err("constant has no original wire name");
            assert!(
                error.to_string().contains(
                    "original wire name requested for constant context entry `route_name`"
                ),
                "{error}"
            );
        }
    }

    /// Scenario: one pipeline consumer selects a constant member from a composite.
    /// Guarantees: the compiled pipeline bindings include a materialization plan for that composite.
    #[test]
    fn selected_constant_builds_pipeline_materialization_plan() {
        let pipeline = pipeline("group", "pipeline");
        let declarations = HashMap::from([(
            pipeline.clone(),
            HashMap::from([(
                ConfigNodeId::from("node"),
                PreparedNodeContextDeclarations::new(
                    consumer(
                        member_target("route", "route_name"),
                        ContextEntrySelectorForm::Value,
                    ),
                    &[constant_composite()],
                )
                .expect("constant selection is valid"),
            )]),
        )]);
        let requirements = ContextRuntimeRequirements::compile(&declarations);

        let bindings =
            CompiledContextBindings::compile(declarations, &requirements).expect("valid bindings");

        let plan = bindings
            .context_materialization_plan(&pipeline)
            .expect("pipeline materialization plan");
        assert_eq!(plan.layout().entries().len(), 1);
        assert_eq!(plan.layout().entries()[0].name, context_name("route"));
    }

    /// Scenario: a consumer selects a generated randomness member.
    /// Guarantees: randomness adds no external requirements and rejects original wire names.
    #[test]
    fn randomness_members_have_no_external_or_original_name_requirements() {
        let context = [randomness_composite()];
        let member = member_target("idempotency", "id");
        let whole = ContextEntryTarget::Composite {
            name: context_name("idempotency"),
        };
        let randomness_name = context_name("id");
        assert_eq!(
            collect_selected_sources(&whole, &context),
            [SelectedSource::Randomness(&randomness_name)]
        );

        let prepared = PreparedNodeContextDeclarations::new(
            consumer(member.clone(), ContextEntrySelectorForm::Value),
            &context,
        )
        .expect("randomness value selection");
        assert_eq!(prepared.requirements, ContextRuntimeRequirements::none());
        for target in [member, whole] {
            let error = PreparedNodeContextDeclarations::new(
                consumer(target, ContextEntrySelectorForm::OriginalKeyValue),
                &context,
            )
            .expect_err("randomness has no original wire name");
            assert!(
                error
                    .to_string()
                    .contains("original wire name requested for randomness context entry `id`")
            );
        }
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

    /// Scenario: one selected composite references the same transport header with different casing.
    /// Guarantees: startup and live-update preparation return a config error instead of panicking.
    #[test]
    fn full_yaml_preparation_rejects_case_variant_header_references() {
        let valid = "[{type: constant, name: route_name, value: stable}, \
            {type: transport_header, name: X-Tenant, store_as: upper}]";
        let invalid = "[{type: constant, name: route_name, value: stable}, \
            {type: transport_header, name: X-Tenant, store_as: upper}, \
            {type: transport_header, name: x-tenant, store_as: lower}]";
        let factory = test_pipeline_factory();
        let installed = factory
            .compile_initial_context(&resolve_consumer_pipeline(valid, "route_name", false))
            .expect("valid initial context");
        let invalid = resolve_consumer_pipeline(invalid, "route_name", false);

        for result in [
            factory.compile_initial_context(&invalid),
            factory.compile_candidate_context(&invalid, &installed.runtime_requirements),
        ] {
            let error = result.expect_err("case-variant references must fail preparation");
            assert!(
                error
                    .to_string()
                    .contains("context entry `tenant` repeats `x-tenant`"),
                "{error}"
            );
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
                    .pipeline_context_compatible(&candidate.bindings, &key)
            );
            assert!(
                !candidate
                    .bindings
                    .pipeline_context_compatible(&installed.bindings, &key)
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
                .pipeline_context_compatible(&candidate.bindings, &key)
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
                ContextDeclaration::Arrival,
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries {
                        entries: ["x-first", "canonical"]
                            .map(|name| ContextEntrySelector {
                                target: ContextEntryTarget::Primitive {
                                    domain: ContextDomain::TransportHeader,
                                    name: context_name(name),
                                    matching: ContextNameMatching::Exact,
                                },
                                form: ContextEntrySelectorForm::OriginalKeyValue,
                                take: ContextEntryTake::All,
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
            .node_bindings(&pipeline("group", "pipeline"), &ConfigNodeId::from("node"))
            .expect("node bindings")
            .header_capture_policy()
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
                            matching: ContextNameMatching::Exact,
                        },
                        form: ContextEntrySelectorForm::OriginalKeyValue,
                        take: ContextEntryTake::All,
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
                            matching: ContextNameMatching::Exact,
                        },
                        form: ContextEntrySelectorForm::Value,
                        take: ContextEntryTake::All,
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
                                matching: ContextNameMatching::Exact,
                            },
                            form: ContextEntrySelectorForm::OriginalKeyValue,
                            take: ContextEntryTake::All,
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
                                matching: ContextNameMatching::Exact,
                            },
                            form: ContextEntrySelectorForm::Value,
                            take: ContextEntryTake::All,
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
                            matching: ContextNameMatching::Exact,
                        },
                        form: ContextEntrySelectorForm::OriginalKeyValue,
                        take: ContextEntryTake::All,
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
                            matching: ContextNameMatching::Exact,
                        },
                        form: ContextEntrySelectorForm::OriginalKeyValue,
                        take: ContextEntryTake::All,
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
        let expected = policy
            .clone()
            .bind(&ContextMaterializationPlan::default())
            .expect("prepared policy binds");
        assert_eq!(
            compiled
                .node_bindings(&pipeline("group", "pipeline"), &ConfigNodeId::from("node"))
                .expect("node bindings")
                .sink()
                .map(SinkBinding::policy),
            Some(&expected),
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
        let context: otel_arrow_dfe_config::context::policy::ContextPolicy = serde_yaml::from_str(
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
            scope: otel_arrow_dfe_config::context::policy::ContextScope::Engine,
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
            std::slice::from_ref(&declaration),
        )
        .expect("wrapper declarations");
        let ContextDeclaration::HeaderPropagation { policy } =
            declarations.iter().next().expect("propagation declaration")
        else {
            panic!("expected header propagation declaration");
        };
        let mut selected = Vec::new();
        policy.visit_composite_names(|name| selected.push(name.clone()));
        assert_eq!(selected, [context_name("tenant")]);
        let plan = ContextMaterializationPlan::compile(&[declaration])
            .expect("materialization plan compiles");
        let bound = policy.clone().bind(&plan).expect("prepared policy binds");
        assert_eq!(bound.materialization_plan(), &plan);
    }

    /// Scenario: complete YAML changes a composite condition or selected member during a live update.
    /// Guarantees: resolution binds propagation to a materialization plan and detects both changes.
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
            .node_bindings(&pipeline, &exporter)
            .expect("exporter bindings")
            .sink()
            .expect("compiled exporter sink")
            .policy();
        assert!(!policy.materialization_plan().is_empty());
        assert_eq!(
            installed.bindings.context_materialization_plan(&pipeline),
            Some(policy.materialization_plan())
        );

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
                .pipeline_context_compatible(&condition_candidate.bindings, &pipeline)
        );

        let changed_member = resolve_conditional_pipeline(current_composite, "tenant:account_id");
        let member_candidate = factory
            .compile_candidate_context(&changed_member, &installed.runtime_requirements)
            .expect("member candidate compiles");
        assert!(
            !installed
                .bindings
                .pipeline_context_compatible(&member_candidate.bindings, &pipeline)
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
                .pipeline_context_compatible(&candidate.bindings, &pipeline("default", "main"))
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
                "context entry reference `tenant:missing` does not select a value-bearing member",
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
            assert!(node.bindings.authorized_identity_policy().is_none());
        }
    }

    /// Scenario: a receiver declares an authorized identity claim projection.
    /// Guarantees: bindings retain the policy and reject changed projections in either comparison direction.
    #[test]
    fn authorized_identity_policy_is_a_compiled_receiver_binding() {
        let policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "tenant"}]))
                .expect("valid authorized identity policy");
        let declarations: NodeContextDeclarations = [
            ContextDeclaration::Arrival,
            ContextDeclaration::AuthorizedIdentityCapture {
                policy: policy.clone(),
            },
        ]
        .into_iter()
        .collect();
        let compiled = compiled_bindings(declarations);
        let changed_policy: AuthorizedIdentityPolicy = serde_json::from_value(
            serde_json::json!([{"claim": "groups", "store_as": "access_groups"}]),
        )
        .expect("valid changed authorized identity policy");
        let changed = compiled_bindings(
            [
                ContextDeclaration::Arrival,
                ContextDeclaration::AuthorizedIdentityCapture {
                    policy: changed_policy,
                },
            ]
            .into_iter()
            .collect(),
        );
        let pipeline = pipeline("group", "pipeline");

        assert_eq!(
            compiled
                .node_bindings(&pipeline, &ConfigNodeId::from("node"))
                .expect("node bindings")
                .authorized_identity_policy(),
            Some(&policy),
        );
        assert!(!compiled.pipeline_context_compatible(&changed, &pipeline));
        assert!(!changed.pipeline_context_compatible(&compiled, &pipeline));
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
        let bindings =
            CompiledContextBindings::compile(declarations, &requirements).expect("valid bindings");

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
        let expected = propagation_policy
            .bind(&ContextMaterializationPlan::default())
            .expect("prepared policy binds");
        assert_eq!(
            bindings
                .node_bindings(&pipeline, &node)
                .expect("node bindings")
                .sink()
                .map(SinkBinding::policy),
            Some(&expected)
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
