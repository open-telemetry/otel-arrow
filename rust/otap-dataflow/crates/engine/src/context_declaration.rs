// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component context declarations collected before runtime construction.

use std::collections::HashMap;
use std::sync::Arc;

use otel_arrow_dfe_config::engine::ResolvedOtelDataflowSpec;
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::node::NodeKind;
use otel_arrow_dfe_config::{NodeId as ConfigNodeId, PipelineKey};

use crate::PipelineFactory;
use crate::error::Error as EngineError;

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
    /// Selects one named register.
    Register(String),
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

/// Opaque context policy compiled from the resolved configuration.
#[derive(Debug)]
pub struct CompiledContextPolicy {
    // Replaced by executable plans in the next compiler pass.
    #[allow(dead_code)]
    declarations: HashMap<PipelineKey, HashMap<ConfigNodeId, Box<[ContextDeclaration]>>>,
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

        Ok(Arc::new(CompiledContextPolicy { declarations }))
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
        let declarations = match kind {
            NodeKind::Receiver => {
                self.get_receiver_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .context_declarations
            }
            NodeKind::Processor => {
                self.get_processor_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .context_declarations
            }
            NodeKind::Exporter => {
                self.get_exporter_factory_map()
                    .get(urn)
                    .ok_or_else(&missing_factory)?
                    .context_declarations
            }
        };
        declarations.map_or_else(
            || Ok(Vec::new()),
            |declare| declare(config).map_err(|error| EngineError::ConfigError(Box::new(error))),
        )
    }
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
        let selector = ContextReadSelector::Register("x-topic".into());
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

    /// Scenario: Generic selectors represent one, ordered, and all-register reads.
    /// Guarantees: selector equality preserves register selection and order.
    #[test]
    fn selectors_preserve_generic_read_contracts() {
        assert_ne!(
            ContextReadSelector::Register("tenant".into()),
            ContextReadSelector::Register("region".into())
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
}
