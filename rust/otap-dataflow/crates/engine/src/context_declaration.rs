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
use otel_arrow_dfe_config::context_policy::ContextEntryDeclaration as ConfigContextEntryDeclaration;
use otel_arrow_dfe_config::engine::ResolvedOtelDataflowSpec;
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::node::{NodeKind, NodeUserConfig};
use otel_arrow_dfe_config::transport_headers_policy::{
    CompiledHeaderCapturePolicy, HeaderCapturePolicy, HeaderPropagationPolicy,
    TransportHeadersPolicy,
};
use otel_arrow_dfe_config::{ContextEntryName, NodeId as ConfigNodeId, PipelineKey};
use std::collections::{BTreeMap, HashMap};
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

    fn context_runtime_requirements(&self) -> ContextRuntimeRequirements {
        let mut requirements = ContextRuntimeRequirements::none();
        match self {
            Self::Consumes {
                selector: ContextConsumerSelector::Entries { entries },
            } => {
                for entry in entries {
                    if entry.form == ContextEntrySelectorForm::OriginalKeyValue {
                        _ = requirements
                            .original_name_retention
                            .overrides
                            .insert(original_name_key(&entry.name), true);
                    }
                }
            }
            Self::Consumes {
                selector: ContextConsumerSelector::AllStored,
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
        requirements
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
}

/// Declarations and transport-header policies compiled for one node.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledNodeBindings {
    /// Declarations supplied by the component factory.
    component_declarations: NodeContextDeclarations,
    /// Receiver header capture policy compiled for the engine requirements.
    header_capture: Option<CompiledHeaderCapturePolicy>,
    /// Exporter header propagation policy resolved from node or pipeline config.
    header_propagation: Option<HeaderPropagationPolicy>,
    /// Receiver authorized identity claim projection policy.
    authorized_identity_capture: Option<AuthorizedIdentityPolicy>,
}

/// Declarations indexed by pipeline and node identifiers.
type ContextDeclarationsByPipeline =
    HashMap<PipelineKey, HashMap<ConfigNodeId, NodeContextDeclarations>>;

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
    fn compile(declarations: &ContextDeclarationsByPipeline) -> Self {
        declarations
            .values()
            .flat_map(HashMap::values)
            .flat_map(NodeContextDeclarations::iter)
            .fold(Self::none(), |requirements, declaration| {
                requirements.union(declaration.context_runtime_requirements())
            })
    }

    fn none() -> Self {
        Self {
            original_name_retention: OriginalNameRetention {
                default_preserve_original: false,
                overrides: BTreeMap::new(),
            },
        }
    }

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

    fn preserves_original_name(&self, name: &ContextEntryName) -> bool {
        self.preserves_original_key(&original_name_key(name))
    }

    fn preserves_original_key(&self, name: &str) -> bool {
        self.overrides
            .get(name)
            .copied()
            .unwrap_or(self.default_preserve_original)
    }
}

fn original_name_key(name: &ContextEntryName) -> Box<str> {
    name.as_str().to_ascii_lowercase().into()
}

impl CompiledNodeBindings {
    fn compile(
        declarations: NodeContextDeclarations,
        requirements: &ContextRuntimeRequirements,
    ) -> Self {
        let mut component_declarations = Vec::new();
        let mut header_capture = None;
        let mut header_propagation = None;
        let mut authorized_identity_capture = None;
        for declaration in declarations {
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
            header_capture,
            header_propagation,
            authorized_identity_capture,
        }
    }

    fn is_empty(&self) -> bool {
        self.component_declarations.is_empty()
            && self.header_capture.is_none()
            && self.header_propagation.is_none()
            && self.authorized_identity_capture.is_none()
    }
}

impl CompiledContextBindings {
    /// Creates an empty binding set. Node validation always fails.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            by_pipeline: HashMap::new(),
        }
    }

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

    /// Returns whether two binding sets contain identical non-empty bindings for one pipeline.
    ///
    /// Nodes without context declarations do not affect compiled bindings and
    /// may be added, removed, or renamed during an otherwise safe live update.
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
    /// Call after parsing the node configuration.
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

    /// Compiles candidate bindings using the immutable installed requirements.
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

    fn compile_bindings(
        declarations: ContextDeclarationsByPipeline,
        requirements: &ContextRuntimeRequirements,
    ) -> Arc<CompiledContextBindings> {
        Arc::new(CompiledContextBindings::compile(declarations, requirements))
    }

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
                let _ = declarations_by_node.insert(node_id.clone(), declarations);
            }
            let _ = declarations.insert(pipeline_key, declarations_by_node);
        }

        Ok(declarations)
    }

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
                        policy
                            .compile_context(context)
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
    use otel_arrow_dfe_config::transport_headers::{TransportHeader, TransportHeaders};
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

    fn unused_test_receiver(
        _: crate::context::PipelineContext,
        _: crate::node::NodeId,
        _: Arc<NodeUserConfig>,
        _: &crate::config::ReceiverConfig,
        _: &crate::capability::registry::Capabilities,
    ) -> Result<crate::receiver::ReceiverWrapper<()>, Error> {
        unreachable!("context compilation does not construct test nodes")
    }

    fn unused_test_exporter(
        _: crate::context::PipelineContext,
        _: crate::node::NodeId,
        _: Arc<NodeUserConfig>,
        _: &crate::config::ExporterConfig,
        _: &crate::capability::registry::Capabilities,
    ) -> Result<crate::exporter::ExporterWrapper<()>, Error> {
        unreachable!("context compilation does not construct test nodes")
    }

    fn unused_test_processor(
        _: crate::context::PipelineContext,
        _: crate::node::NodeId,
        _: Arc<NodeUserConfig>,
        _: &crate::config::ProcessorConfig,
        _: &crate::capability::registry::Capabilities,
    ) -> Result<crate::processor::ProcessorWrapper<()>, Error> {
        unreachable!("context compilation does not construct test nodes")
    }

    fn accept_test_config(_: &serde_json::Value) -> Result<(), Error> {
        Ok(())
    }

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

    static TEST_EXPORTERS: [crate::ExporterFactory<()>; 3] = [
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
    ];

    static TEST_PROCESSORS: [crate::ProcessorFactory<()>; 1] = [crate::ProcessorFactory {
        name: "urn:otel:processor:type_router",
        create: unused_test_processor,
        context_declarations: None,
        wiring_contract: crate::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: accept_test_config,
    }];

    fn test_pipeline_factory() -> PipelineFactory<()> {
        PipelineFactory::new(&TEST_RECEIVERS, &TEST_PROCESSORS, &TEST_EXPORTERS, &[])
    }

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

    fn resolve_conditional_pipeline(composite: &str, selector: &str) -> ResolvedOtelDataflowSpec {
        otel_arrow_dfe_config::engine::OtelDataflowSpec::from_yaml(&conditional_pipeline_yaml(
            composite, selector,
        ))
        .expect("conditional pipeline YAML is valid")
        .resolve()
    }

    fn declarations_by_pipeline(
        effective: NodeContextDeclarations,
    ) -> ContextDeclarationsByPipeline {
        HashMap::from([(
            pipeline("group", "pipeline"),
            HashMap::from([(ConfigNodeId::from("node"), effective)]),
        )])
    }

    fn compiled_bindings(effective: NodeContextDeclarations) -> CompiledContextBindings {
        let declarations = declarations_by_pipeline(effective);
        let requirements = ContextRuntimeRequirements::compile(&declarations);
        CompiledContextBindings::compile(declarations, &requirements)
    }

    fn context_runtime_requirements(
        effective: NodeContextDeclarations,
    ) -> ContextRuntimeRequirements {
        ContextRuntimeRequirements::compile(&declarations_by_pipeline(effective))
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
        let requirements = context_runtime_requirements(declarations);
        assert!(requirements.preserves_original_name(&context_name("original")));
        assert!(!requirements.preserves_original_name(&context_name("value")));
    }

    /// Scenario: propagation preserves arbitrary names but overrides one stored name.
    /// Guarantees: the profile uses a true default with one case-insensitive exception.
    #[test]
    fn requirements_canonicalize_default_and_overrides() {
        let propagation: HeaderPropagationPolicy = serde_json::from_value(serde_json::json!({
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
                            name: context_name("x-tenant"),
                            form: ContextEntrySelectorForm::OriginalKeyValue,
                        }]
                        .into_boxed_slice(),
                    },
                },
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::Entries {
                        entries: vec![ContextEntrySelector {
                            name: context_name("authorization"),
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
                        name: context_name("X-Tenant"),
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
                        name: context_name("x-request-id"),
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
                policy: serde_json::from_value(serde_json::json!({
                    "default": {
                        "selector": {"type": "all_captured"},
                        "name": "preserve"
                    }
                }))
                .expect("valid propagation policy"),
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
        let node_propagation = HeaderPropagationPolicy::default();
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
                &[],
            )
            .expect("wrapper declarations"),
            [ContextDeclaration::HeaderPropagation {
                policy: pipeline_policy.header_propagation,
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
                "context entry reference `tenant:missing` does not select a transport-header member",
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
        let declarations = HashMap::from([(
            pipeline.clone(),
            HashMap::from([(node.clone(), declarations)]),
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
}
