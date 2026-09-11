// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Collects and compiles context declarations before runtime construction.

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
use otel_arrow_dfe_config::{ContextEntryName, NodeId as ConfigNodeId, PipelineKey};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

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
    /// Stored name and value. The variant name is historical; it does not
    /// lowercase an explicitly configured stored name.
    NormalizedKeyValue,
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
    /// Selects every context entry using stored names. The variant name is
    /// historical; it does not lowercase explicitly configured stored names.
    AllNormalized,
}

/// A node's context access or transport-header policy.
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
}

impl ContextDeclaration {
    fn is_component_declaration(&self) -> bool {
        matches!(self, Self::Produces { .. } | Self::Consumes { .. })
    }

    fn contributes_original_name_requirement(&self) -> bool {
        matches!(self, Self::Consumes { .. } | Self::HeaderPropagation { .. })
    }

    fn requires_original_name(&self, name: &ContextEntryName) -> bool {
        match self {
            Self::Consumes {
                selector: ContextConsumerSelector::Entries { entries },
            } => entries.iter().any(|entry| {
                entry.form == ContextEntrySelectorForm::OriginalKeyValue && &entry.name == name
            }),
            Self::Consumes {
                selector: ContextConsumerSelector::AllNormalized,
            }
            | Self::Produces { .. }
            | Self::HeaderCapture { .. } => false,
            Self::HeaderPropagation { policy } => policy.propagates_original_name(name),
        }
    }
}

/// Derives context declarations from component configuration.
#[derive(Clone, Copy)]
pub struct ContextDeclarationProvider {
    /// Component URN.
    pub urn: &'static str,
    /// Declaration callback.
    pub declarations: ContextDeclarationFn,
}

/// Derives deterministic context declarations from node configuration.
pub type ContextDeclarationFn = fn(&serde_json::Value) -> Result<NodeContextDeclarations, Error>;

/// Context declarations derived from typed node configuration.
pub trait ConfigNodeContextDeclaration: serde::de::DeserializeOwned {
    /// Declares the context reads and writes for this configuration.
    fn context_declarations(&self) -> NodeContextDeclarations;

    /// Checks these declarations against the compiled policy.
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

// linkme's generated #[link_section] requires an unsafe-code allowance.
/// Context declaration providers registered by nodes.
#[allow(unsafe_code)]
#[distributed_slice]
pub static CONTEXT_DECLARATION_PROVIDERS: [ContextDeclarationProvider];

/// Sorted, unique context declarations.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeContextDeclarations {
    /// Sorted and deduplicated declarations.
    byid: Box<[ContextDeclaration]>,
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
            byid: uniq.into_boxed_slice(),
        }
    }
}

impl IntoIterator for NodeContextDeclarations {
    type Item = ContextDeclaration;
    type IntoIter = std::vec::IntoIter<ContextDeclaration>;

    fn into_iter(self) -> Self::IntoIter {
        self.byid.into_vec().into_iter()
    }
}

impl NodeContextDeclarations {
    /// Iterates over declarations in sorted order.
    pub fn iter(&self) -> impl Iterator<Item = &ContextDeclaration> {
        self.byid.iter()
    }

    /// Returns whether the declaration set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.byid.is_empty()
    }

    /// Returns the declaration count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.byid.len()
    }
}

impl ContextDeclarationProvider {
    /// Creates a declaration provider for a configuration type.
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

/// Context policy compiled from all pipelines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledContextPolicy {
    nodes: HashMap<PipelineKey, HashMap<ConfigNodeId, CompiledNodeContext>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledNodeContext {
    bindings: Box<[CompiledContextBinding]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledContextBinding {
    pub(crate) declaration: ContextDeclaration,
    pub(crate) access: CompiledContextAccess,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompiledContextAccess {
    Produces,
    Consumes,
    HeaderCapture(CompiledHeaderCapturePolicy),
    HeaderPropagation,
}

type DeclaredContextPolicy = HashMap<PipelineKey, HashMap<ConfigNodeId, NodeContextDeclarations>>;

impl CompiledNodeContext {
    fn compile(
        declarations: NodeContextDeclarations,
        original_name_requirements: &[ContextDeclaration],
    ) -> Self {
        let bindings = declarations
            .into_iter()
            .map(|declaration| {
                let access = match &declaration {
                    ContextDeclaration::Produces { .. } => CompiledContextAccess::Produces,
                    ContextDeclaration::Consumes { .. } => CompiledContextAccess::Consumes,
                    ContextDeclaration::HeaderCapture { policy } => {
                        CompiledContextAccess::HeaderCapture(policy.clone().compile(|name| {
                            original_name_requirements
                                .iter()
                                .any(|declaration| declaration.requires_original_name(name))
                        }))
                    }
                    ContextDeclaration::HeaderPropagation { .. } => {
                        CompiledContextAccess::HeaderPropagation
                    }
                };
                CompiledContextBinding {
                    declaration,
                    access,
                }
            })
            .collect();

        Self { bindings }
    }
}

impl CompiledContextPolicy {
    /// Creates a policy with no registered nodes. Node validation always fails.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    fn compile(declarations: DeclaredContextPolicy) -> Self {
        let original_name_requirements: Vec<_> = declarations
            .values()
            .flat_map(HashMap::values)
            .flat_map(NodeContextDeclarations::iter)
            .filter(|declaration| declaration.contributes_original_name_requirement())
            .cloned()
            .collect();

        let nodes = declarations
            .into_iter()
            .map(|(pipeline, nodes)| {
                let nodes = nodes
                    .into_iter()
                    .map(|(node, declarations)| {
                        (
                            node,
                            CompiledNodeContext::compile(declarations, &original_name_requirements),
                        )
                    })
                    .collect();
                (pipeline, nodes)
            })
            .collect();

        Self { nodes }
    }

    /// Returns the node's compiled bindings.
    pub(crate) fn node_bindings(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
    ) -> Option<&[CompiledContextBinding]> {
        Some(&self.nodes.get(pipeline)?.get(node)?.bindings)
    }

    /// Returns whether two policies compile identical bindings for one pipeline.
    #[must_use]
    pub fn pipeline_bindings_match(&self, other: &Self, pipeline: &PipelineKey) -> bool {
        self.nodes.get(pipeline) == other.nodes.get(pipeline)
    }

    /// Checks component declarations against this node's compiled bindings.
    /// Call after parsing the node configuration.
    pub fn validate_node_declarations(
        &self,
        pipeline: &PipelineKey,
        node: &ConfigNodeId,
        declarations: &NodeContextDeclarations,
    ) -> Result<(), Error> {
        match self.nodes.get(pipeline).and_then(|nodes| nodes.get(node)) {
            Some(expected)
                if expected
                    .bindings
                    .iter()
                    .map(|binding| &binding.declaration)
                    .filter(|declaration| declaration.is_component_declaration())
                    .eq(declarations.iter()) =>
            {
                Ok(())
            }
            _ => Err(Error::UnrecognizedContextDeclaration {}),
        }
    }
}

impl<PData: 'static + Clone + std::fmt::Debug> PipelineFactory<PData> {
    /// Compiles context policies for the full resolved engine configuration.
    pub fn compile_context_policy(
        &self,
        resolved: &ResolvedOtelDataflowSpec,
    ) -> Result<Arc<CompiledContextPolicy>, EngineError> {
        let mut declarations = DeclaredContextPolicy::new();

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
                let wrapper_declaration = Self::wrapper_context_declaration(
                    node_config,
                    &pipeline.policies.transport_headers,
                );
                let declarations = component_declarations
                    .into_iter()
                    .chain(wrapper_declaration)
                    .collect();
                let _ = declarations_by_node.insert(node_id.clone(), declarations);
            }
            let _ = declarations.insert(pipeline_key, declarations_by_node);
        }

        Ok(Arc::new(CompiledContextPolicy::compile(declarations)))
    }

    fn wrapper_context_declaration(
        node: &NodeUserConfig,
        pipeline_policy: &Option<TransportHeadersPolicy>,
    ) -> Option<ContextDeclaration> {
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
                .map(|policy| ContextDeclaration::HeaderCapture { policy }),
            NodeKind::Exporter => node
                .header_propagation
                .as_ref()
                .or_else(|| {
                    pipeline_policy
                        .as_ref()
                        .map(|policy| &policy.header_propagation)
                })
                .cloned()
                .map(|policy| ContextDeclaration::HeaderPropagation { policy }),
            NodeKind::Processor => None,
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
        // Validate before collecting declarations. Nodes are not constructed yet.
        validate_config(config).map_err(|error| EngineError::ConfigError(Box::new(error)))?;

        let declarations = context_declaration_provider(urn)
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

    #[allow(unsafe_code)]
    #[distributed_slice(CONTEXT_DECLARATION_PROVIDERS)]
    static TEST_CONTEXT_DECLARATIONS: ContextDeclarationProvider =
        ContextDeclarationProvider::from_typed_config::<TestDeclarationConfig>(
            TEST_DECLARATION_URN,
        );

    fn pipeline(group: &str, name: &str) -> PipelineKey {
        PipelineKey::new(group.to_owned().into(), name.to_owned().into())
    }

    fn context_name(name: &str) -> ContextEntryName {
        name.try_into().expect("valid test context entry name")
    }

    fn compiled_policy(effective: NodeContextDeclarations) -> CompiledContextPolicy {
        CompiledContextPolicy::compile(HashMap::from([(
            pipeline("group", "pipeline"),
            HashMap::from([(ConfigNodeId::from("node"), effective)]),
        )]))
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
        let policy = compiled_policy(
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
        let capture = policy
            .node_bindings(&pipeline("group", "pipeline"), &ConfigNodeId::from("node"))
            .expect("compiled node bindings")
            .iter()
            .find_map(|binding| match &binding.access {
                CompiledContextAccess::HeaderCapture(policy) => Some(policy),
                _ => None,
            })
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
        assert!(
            declarations
                .iter()
                .any(|declaration| declaration.requires_original_name(&context_name("original")))
        );
        assert!(
            !declarations
                .iter()
                .any(|declaration| declaration.requires_original_name(&context_name("value")))
        );
    }

    /// Scenario: a propagation declaration selects one original header name.
    /// Guarantees: compilation keeps the policy and requires only that original name.
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
        let compiled = compiled_policy(declarations.clone());

        assert!(
            declarations
                .iter()
                .any(|declaration| declaration.requires_original_name(&context_name("preserved")))
        );
        assert!(
            !declarations
                .iter()
                .any(|declaration| declaration.requires_original_name(&context_name("other")))
        );
        assert_eq!(
            compiled
                .node_bindings(&pipeline("group", "pipeline"), &ConfigNodeId::from("node"))
                .expect("compiled node bindings")
                .iter()
                .find_map(|binding| match (&binding.declaration, &binding.access) {
                    (
                        ContextDeclaration::HeaderPropagation { policy },
                        CompiledContextAccess::HeaderPropagation,
                    ) => Some(policy),
                    _ => None,
                }),
            Some(&policy),
        );
    }

    /// Scenario: node and pipeline header policies are configured.
    /// Guarantees: node policies take precedence. Pipeline policies provide the fallback.
    #[test]
    fn wrapper_declarations_resolve_policy_precedence() {
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
            PipelineFactory::<()>::wrapper_context_declaration(
                &receiver,
                &Some(pipeline_policy.clone()),
            ),
            Some(ContextDeclaration::HeaderCapture {
                policy: node_capture,
            }),
        );

        let receiver = NodeUserConfig::new_receiver_config("urn:test:receiver:example");
        assert_eq!(
            PipelineFactory::<()>::wrapper_context_declaration(
                &receiver,
                &Some(pipeline_policy.clone()),
            ),
            Some(ContextDeclaration::HeaderCapture {
                policy: pipeline_policy.header_capture.clone(),
            }),
        );

        let mut exporter = NodeUserConfig::new_exporter_config("urn:test:exporter:example");
        let node_propagation = HeaderPropagationPolicy::default();
        exporter.header_propagation = Some(node_propagation.clone());
        assert_eq!(
            PipelineFactory::<()>::wrapper_context_declaration(
                &exporter,
                &Some(pipeline_policy.clone()),
            ),
            Some(ContextDeclaration::HeaderPropagation {
                policy: node_propagation,
            }),
        );

        let exporter = NodeUserConfig::new_exporter_config("urn:test:exporter:example");
        assert_eq!(
            PipelineFactory::<()>::wrapper_context_declaration(
                &exporter,
                &Some(pipeline_policy.clone()),
            ),
            Some(ContextDeclaration::HeaderPropagation {
                policy: pipeline_policy.header_propagation,
            }),
        );
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
        let policy = CompiledContextPolicy::compile(HashMap::from([(
            pipeline.clone(),
            HashMap::from([(node.clone(), declarations)]),
        )]));

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
        assert!(
            policy
                .validate_node_declarations(
                    &pipeline,
                    &ConfigNodeId::from("other"),
                    &matching.context_declarations(),
                )
                .is_err()
        );
        assert!(
            CompiledContextPolicy::empty()
                .validate_node_declarations(&pipeline, &node, &matching.context_declarations())
                .is_err()
        );
        assert!(
            policy
                .nodes
                .get(&pipeline)
                .and_then(|nodes| nodes.get(&node))
                .expect("compiled node declarations")
                .bindings
                .iter()
                .any(|binding| binding.declaration == propagation_declaration)
        );
    }

    /// Scenario: a node with no declarations is missing from the compiled policy.
    /// Guarantees: empty declarations still require a registered node in the correct pipeline.
    #[test]
    fn empty_declarations_require_a_compiled_node() {
        let key = pipeline("group", "pipeline");
        let node = ConfigNodeId::from("node");
        let declarations = NodeContextDeclarations::default();
        let policy = compiled_policy(declarations.clone());

        assert!(
            policy
                .validate_node_declarations(&key, &node, &declarations)
                .is_ok()
        );
        assert!(
            policy
                .validate_node_declarations(&pipeline("group", "other"), &node, &declarations)
                .is_err()
        );
        assert!(
            CompiledContextPolicy::empty()
                .validate_node_declarations(&key, &node, &declarations)
                .is_err()
        );
    }
}
