// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

use std::{collections::HashMap, sync::OnceLock};

use serde_json::Value;

use crate::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    error::Error,
    exporter::ExporterWrapper,
    processor::ProcessorWrapper,
    receiver::ReceiverWrapper,
    runtime_config::{RuntimeNode, RuntimePipeline},
};

pub mod error;
pub mod exporter;
pub mod message;
pub mod processor;
pub mod receiver;

pub mod config;
mod effect_handler;
pub mod local;
pub mod pipeline;
pub mod runtime_config;
pub mod shared;

pub mod testing;

pub use linkme::distributed_slice;

/// Trait for factory types that expose a name.
///
/// This trait is used to define a common interface for different types of factories
/// that create instances of receivers, processors, or exporters.
pub trait NamedFactory {
    /// Returns the name of the node factory.
    fn name(&self) -> &'static str;
}

/// A factory for creating receivers.
pub struct ReceiverFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new receiver instance.
    pub create: fn(config: &Value, receiver_config: &ReceiverConfig) -> ReceiverWrapper<PData>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ReceiverFactory<PData> {
    fn clone(&self) -> Self {
        ReceiverFactory {
            name: self.name,
            create: self.create,
        }
    }
}

impl<PData> NamedFactory for ReceiverFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// A factory for creating processors.
pub struct ProcessorFactory<PData> {
    /// The name of the processor.
    pub name: &'static str,
    /// A function that creates a new processor instance.
    pub create: fn(config: &Value, processor_config: &ProcessorConfig) -> ProcessorWrapper<PData>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ProcessorFactory<PData> {
    fn clone(&self) -> Self {
        ProcessorFactory {
            name: self.name,
            create: self.create,
        }
    }
}

impl<PData> NamedFactory for ProcessorFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// A factory for creating exporter.
pub struct ExporterFactory<PData> {
    /// The name of the receiver.
    pub name: &'static str,
    /// A function that creates a new exporter instance.
    pub create: fn(config: &Value, exporter_config: &ExporterConfig) -> ExporterWrapper<PData>,
}

// Note: We don't use `#[derive(Clone)]` here to avoid forcing the `PData` type to implement `Clone`.
impl<PData> Clone for ExporterFactory<PData> {
    fn clone(&self) -> Self {
        ExporterFactory {
            name: self.name,
            create: self.create,
        }
    }
}

impl<PData> NamedFactory for ExporterFactory<PData> {
    fn name(&self) -> &'static str {
        self.name
    }
}

/// Returns a map of factory names to factory instances.
pub fn get_factory_map<T>(
    factory_map: &'static OnceLock<HashMap<&'static str, T>>,
    factory_slice: &'static [T],
) -> &'static HashMap<&'static str, T>
where
    T: NamedFactory + Clone,
{
    factory_map.get_or_init(|| {
        factory_slice
            .iter()
            .map(|f| (f.name(), f.clone()))
            .collect::<HashMap<&'static str, T>>()
    })
}

/// Builds a factory registry for initialization.
///
/// This function is used as a placeholder when declaring a factory registry with the
/// `#[factory_registry]` attribute macro. The macro will replace this placeholder with
/// proper lazy initialization using `LazyLock`.
///
/// # Example
/// ```rust,ignore
/// #[factory_registry(MyData)]
/// static FACTORY_REGISTRY: FactoryRegistry<MyData> = build_registry();
/// ```
#[must_use]
pub const fn build_registry<PData: 'static>() -> FactoryRegistry<PData> {
    // This function should never actually be called since the macro replaces it entirely.
    // If it is called, that indicates a bug in the macro system.
    panic!(
        "build_registry() should never be called - it's replaced by the #[factory_registry] macro"
    )
}

/// Generic factory registry that encapsulates all factory maps for a given pdata type.
pub struct FactoryRegistry<PData: 'static> {
    receiver_factory_map: OnceLock<HashMap<&'static str, ReceiverFactory<PData>>>,
    processor_factory_map: OnceLock<HashMap<&'static str, ProcessorFactory<PData>>>,
    exporter_factory_map: OnceLock<HashMap<&'static str, ExporterFactory<PData>>>,
    receiver_factories: &'static [ReceiverFactory<PData>],
    processor_factories: &'static [ProcessorFactory<PData>],
    exporter_factories: &'static [ExporterFactory<PData>],
}

impl<PData: 'static> FactoryRegistry<PData> {
    /// Creates a new factory registry with the given factory slices.
    #[must_use]
    pub const fn new(
        receiver_factories: &'static [ReceiverFactory<PData>],
        processor_factories: &'static [ProcessorFactory<PData>],
        exporter_factories: &'static [ExporterFactory<PData>],
    ) -> Self {
        Self {
            receiver_factory_map: OnceLock::new(),
            processor_factory_map: OnceLock::new(),
            exporter_factory_map: OnceLock::new(),
            receiver_factories,
            processor_factories,
            exporter_factories,
        }
    }

    /// Gets the receiver factory map, initializing it if necessary.
    pub fn get_receiver_factory_map(&self) -> &HashMap<&'static str, ReceiverFactory<PData>> {
        self.receiver_factory_map.get_or_init(|| {
            self.receiver_factories
                .iter()
                .map(|f| (f.name(), f.clone()))
                .collect::<HashMap<&'static str, ReceiverFactory<PData>>>()
        })
    }

    /// Gets the processor factory map, initializing it if necessary.
    pub fn get_processor_factory_map(&self) -> &HashMap<&'static str, ProcessorFactory<PData>> {
        self.processor_factory_map.get_or_init(|| {
            self.processor_factories
                .iter()
                .map(|f| (f.name(), f.clone()))
                .collect::<HashMap<&'static str, ProcessorFactory<PData>>>()
        })
    }

    /// Gets the exporter factory map, initializing it if necessary.
    pub fn get_exporter_factory_map(&self) -> &HashMap<&'static str, ExporterFactory<PData>> {
        self.exporter_factory_map.get_or_init(|| {
            self.exporter_factories
                .iter()
                .map(|f| (f.name(), f.clone()))
                .collect::<HashMap<&'static str, ExporterFactory<PData>>>()
        })
    }

    /// Creates a runtime pipeline from the given pipeline configuration.
    pub fn create_runtime_pipeline(
        &self,
        config: otap_df_config::pipeline::PipelineConfig,
    ) -> Result<RuntimePipeline<PData>, Error<PData>> {
        let receiver_factory_map = self.get_receiver_factory_map();
        let processor_factory_map = self.get_processor_factory_map();
        let exporter_factory_map = self.get_exporter_factory_map();
        let mut nodes = Vec::new();

        // Generate all the errors.
        for (node_id, node_config) in config.node_iter() {
            match node_config.kind {
                otap_df_config::node::NodeKind::Receiver => {
                    let factory = receiver_factory_map
                        .get(node_config.plugin_urn.as_ref())
                        .ok_or_else(|| Error::UnknownReceiver {
                            plugin_urn: node_config.plugin_urn.clone(),
                        })?;
                    let receiver_config = ReceiverConfig::new(node_id.clone());
                    let create = factory.create;
                    nodes.push(RuntimeNode::Receiver {
                        config: node_config.clone(),
                        instance: create(&node_config.config, &receiver_config),
                    });
                }
                otap_df_config::node::NodeKind::Processor => {
                    let factory = processor_factory_map
                        .get(node_config.plugin_urn.as_ref())
                        .ok_or_else(|| Error::UnknownProcessor {
                            plugin_urn: node_config.plugin_urn.clone(),
                        })?;
                    let processor_config = ProcessorConfig::new(node_id.clone());
                    let create = factory.create;
                    nodes.push(RuntimeNode::Processor {
                        config: node_config.clone(),
                        instance: create(&node_config.config, &processor_config),
                    });
                }
                otap_df_config::node::NodeKind::Exporter => {
                    let factory = exporter_factory_map
                        .get(node_config.plugin_urn.as_ref())
                        .ok_or_else(|| Error::UnknownExporter {
                            plugin_urn: node_config.plugin_urn.clone(),
                        })?;
                    let exporter_config = ExporterConfig::new(node_id.clone());
                    let create = factory.create;
                    nodes.push(RuntimeNode::Exporter {
                        config: node_config.clone(),
                        instance: create(&node_config.config, &exporter_config),
                    });
                }
                otap_df_config::node::NodeKind::ProcessorChain => {
                    return Err(Error::UnsupportedNodeKind {
                        kind: "ProcessorChain".into(),
                    });
                }
            }
        }

        Ok(RuntimePipeline::new(config, nodes))
    }
}
