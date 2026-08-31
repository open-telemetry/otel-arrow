// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component context declarations collected before runtime construction.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, OnceLock};

use linkme::distributed_slice;
use otel_arrow_dfe_config::engine::ResolvedOtelDataflowSpec;
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::node::NodeKind;
use otel_arrow_dfe_config::{NodeId as ConfigNodeId, PipelineKey};

use crate::PipelineFactory;
use crate::error::Error as EngineError;

/// A configuration-dependent declaration provider registered by a component.
#[derive(Clone, Copy)]
pub struct ContextDeclarationProvider {
    /// The registered component's URN.
    pub urn: &'static str,
    /// Produces declarations using a component configuration.
    pub declarations: ContextDeclarationFn,
}

impl ContextDeclarationProvider {
    /// Creates a provider that derives declarations from component configuration.
    #[must_use]
    pub const fn from_config(urn: &'static str, declarations: ContextDeclarationFn) -> Self {
        Self { urn, declarations }
    }
}

/// Context declaration providers registered by components.
#[distributed_slice]
pub static CONTEXT_DECLARATION_PROVIDERS: [ContextDeclarationProvider];

/// Context access identifier scoped to one node factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextAccessId(usize);

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

/// Generic context registers selected by one consumer binding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextReadSelector {
    /// Selects named registers in order; providers canonicalize unordered inputs.
    Registers {
        /// Logical context register names.
        names: Box<[String]>,
    },
    /// Selects every context register reachable at this node.
    All,
}

/// One component context declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextDeclaration {
    /// Adds one named value to outgoing context.
    Produces {
        /// Provider-local identity used to retrieve the compiled access.
        access: ContextAccessId,
        /// The logical header name that will be produced.
        name: String,
    },
    /// Reads context through one compiled access.
    Consumes {
        /// Provider-local identity used to retrieve the compiled access.
        access: ContextAccessId,
        /// Generic register selection.
        selector: ContextReadSelector,
    },
}

/// Deterministically describes a node factory's context access.
pub type ContextDeclarationFn = fn(&serde_json::Value) -> Result<Vec<ContextDeclaration>, Error>;

/// Generation assigned to a compiled context policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextPolicyGeneration(u64);

impl ContextPolicyGeneration {
    /// Creates a context policy generation.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Builds deterministic producer declarations from configuration-sized inputs.
#[derive(Default)]
pub struct ContextDeclarationsBuilder {
    produced_names: BTreeSet<String>,
}

impl ContextDeclarationsBuilder {
    /// Creates an empty declaration builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a produced register, normalizing its logical name.
    ///
    /// # Errors
    ///
    /// Returns an error when another configured name has the same normalized
    /// logical name.
    pub fn produce(&mut self, name: impl AsRef<str>) -> Result<(), Error> {
        let name = name.as_ref().to_ascii_lowercase();
        if !self.produced_names.insert(name.clone()) {
            return Err(Error::InvalidUserConfig {
                error: format!("duplicate context register name after normalization: `{name}`"),
            });
        }
        Ok(())
    }

    /// Returns produced declarations in normalized-name order with assigned IDs.
    #[must_use]
    pub fn finish(self) -> Vec<ContextDeclaration> {
        self.produced_names
            .into_iter()
            .enumerate()
            .map(|(index, name)| ContextDeclaration::Produces {
                access: ContextAccessId::new(index),
                name,
            })
            .collect()
    }
}

/// Opaque context policy compiled from the resolved configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledContextPolicy {
    generation: ContextPolicyGeneration,
    // Replaced by executable plans in the next compiler pass.
    declarations: HashMap<PipelineKey, HashMap<ConfigNodeId, Box<[ContextDeclaration]>>>,
}

impl CompiledContextPolicy {
    /// Returns the generation associated with this compiled policy.
    #[must_use]
    pub const fn generation(&self) -> ContextPolicyGeneration {
        self.generation
    }

    /// Returns a copy carrying the supplied generation.
    #[must_use]
    pub fn with_generation(&self, generation: ContextPolicyGeneration) -> Arc<Self> {
        Arc::new(Self {
            generation,
            declarations: self.declarations.clone(),
        })
    }

    /// Returns whether two policies compile the same declarations.
    #[must_use]
    pub fn equivalent_declarations(&self, other: &Self) -> bool {
        self.declarations == other.declarations
    }
}

impl<PData: 'static + Clone + std::fmt::Debug> PipelineFactory<PData> {
    /// Compiles context policy from the complete resolved configuration.
    pub fn compile_context_policy(
        &self,
        resolved: &ResolvedOtelDataflowSpec,
    ) -> Result<Arc<CompiledContextPolicy>, EngineError> {
        let mut declarations = HashMap::new();

        for pipeline in &resolved.pipelines {
            let pipeline_key = PipelineKey::new(
                pipeline.pipeline_group_id.clone(),
                pipeline.pipeline_id.clone(),
            );
            let mut declarations_by_node = HashMap::new();

            for (node_id, node_config) in pipeline.pipeline.node_iter() {
                let node_declarations = self.node_context_declarations(
                    node_config.kind(),
                    node_config.r#type.as_ref(),
                    &node_config.config,
                )?;
                let _ = declarations_by_node
                    .insert(node_id.clone(), node_declarations.into_boxed_slice());
            }

            let _ = declarations.insert(pipeline_key, declarations_by_node);
        }

        Ok(Arc::new(CompiledContextPolicy {
            generation: ContextPolicyGeneration::default(),
            declarations,
        }))
    }

    fn node_context_declarations(
        &self,
        kind: NodeKind,
        urn: &str,
        config: &serde_json::Value,
    ) -> Result<Vec<ContextDeclaration>, EngineError> {
        let missing_factory = || {
            EngineError::ConfigError(Box::new(Error::InvalidUserConfig {
                error: format!("node factory `{urn}` is not registered"),
            }))
        };
        let _ = match kind {
            NodeKind::Receiver => {
                self.get_receiver_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .name
            }
            NodeKind::Processor => {
                self.get_processor_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .name
            }
            NodeKind::Exporter => {
                self.get_exporter_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .name
            }
        };
        context_declaration_provider(urn).map_or_else(
            || Ok(Vec::new()),
            |provider| {
                (provider.declarations)(config)
                    .map_err(|error| EngineError::ConfigError(Box::new(error)))
            },
        )
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
mod tests {
    use super::*;

    /// Scenario: A factory callback receives node config.
    /// Guarantees: it returns the configured declaration.
    #[test]
    fn provider_callback_returns_declarations() {
        fn test_provider(config: &serde_json::Value) -> Result<Vec<ContextDeclaration>, Error> {
            let name = config
                .get("header_name")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Ok(vec![ContextDeclaration::Produces {
                access: ContextAccessId::new(0),
                name: name.to_string(),
            }])
        }

        let config = serde_json::json!({"header_name": "x-test"});
        let decls = test_provider(&config).unwrap();
        assert_eq!(decls.len(), 1);
        match &decls[0] {
            ContextDeclaration::Produces { name, .. } => {
                assert_eq!(name, "x-test");
            }
            other => panic!("unexpected declaration: {other:?}"),
        }
    }

    /// Scenario: policy compilation indexes declarations by pipeline and node.
    /// Guarantees: each node retains only its declarations.
    #[test]
    fn compiled_policy_indexes_existing_configuration_ids() {
        let pipeline = PipelineKey::new("group".into(), "pipeline".into());
        let node: ConfigNodeId = "source".into();
        let declaration = ContextDeclaration::Produces {
            access: ContextAccessId::new(0),
            name: "tenant".into(),
        };
        let policy = CompiledContextPolicy {
            generation: ContextPolicyGeneration::default(),
            declarations: HashMap::from([(
                pipeline.clone(),
                HashMap::from([(node.clone(), vec![declaration.clone()].into_boxed_slice())]),
            )]),
        };

        assert_eq!(
            policy.declarations[&pipeline][&node].as_ref(),
            [declaration].as_slice()
        );
        assert!(!policy.declarations[&pipeline].contains_key("other"));
    }

    /// Scenario: Two access IDs select the same register.
    /// Guarantees: the declarations remain distinct.
    #[test]
    fn access_id_distinguishes_consumers() {
        let selector = ContextReadSelector::Registers {
            names: vec!["x-topic".into()].into_boxed_slice(),
        };
        let first = ContextDeclaration::Consumes {
            access: ContextAccessId::new(0),
            selector: selector.clone(),
        };
        let second = ContextDeclaration::Consumes {
            access: ContextAccessId::new(1),
            selector,
        };
        assert_ne!(first, second);
    }

    /// Scenario: Generic selectors represent ordered and all-register reads.
    /// Guarantees: selector equality preserves register selection and order.
    #[test]
    fn selectors_preserve_generic_read_contracts() {
        assert_ne!(
            ContextReadSelector::Registers {
                names: vec!["tenant".into()].into_boxed_slice(),
            },
            ContextReadSelector::Registers {
                names: vec!["region".into()].into_boxed_slice(),
            }
        );
        assert_ne!(
            ContextReadSelector::Registers {
                names: vec!["tenant".into(), "region".into()].into_boxed_slice(),
            },
            ContextReadSelector::Registers {
                names: vec!["region".into(), "tenant".into()].into_boxed_slice(),
            },
        );
    }

    /// Scenario: Producer inputs are unordered and use mixed-case names.
    /// Guarantees: finished declarations normalize names and assign sorted IDs.
    #[test]
    fn builder_normalizes_and_orders_producers() {
        let mut builder = ContextDeclarationsBuilder::new();
        builder.produce("X-Tenant-Id").unwrap();
        builder.produce("a-first").unwrap();

        assert_eq!(
            builder.finish(),
            vec![
                ContextDeclaration::Produces {
                    access: ContextAccessId::new(0),
                    name: "a-first".into(),
                },
                ContextDeclaration::Produces {
                    access: ContextAccessId::new(1),
                    name: "x-tenant-id".into(),
                },
            ]
        );
    }

    /// Scenario: Producer inputs differ only by logical-name casing.
    /// Guarantees: configuration compilation rejects ambiguous register names.
    #[test]
    fn builder_rejects_duplicate_normalized_producers() {
        let mut builder = ContextDeclarationsBuilder::new();
        builder.produce("X-Tenant-Id").unwrap();

        assert!(matches!(
            builder.produce("x-tenant-id"),
            Err(Error::InvalidUserConfig { .. })
        ));
    }
}
