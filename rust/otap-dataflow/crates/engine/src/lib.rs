// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

use serde_json::Value;
use std::rc::Rc;
use std::{collections::HashMap, sync::OnceLock};

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
use otap_df_config::node::{DispatchStrategy, NodeConfig};
use otap_df_config::{NodeId, PortName};

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
        let mut nodes = HashMap::new();

        // Create runtime nodes based on the pipeline configuration.
        // ToDo(LQ): Collect all errors instead of failing fast to provide better feedback.
        for (node_id, node_config) in config.node_iter() {
            match node_config.kind {
                otap_df_config::node::NodeKind::Receiver => {
                    self.create_receiver(
                        &mut nodes,
                        node_id.clone(),
                        node_config.clone(),
                    )?
                },
                otap_df_config::node::NodeKind::Processor => self.create_processor(
                    &mut nodes,
                    node_id.clone(),
                    node_config.clone(),
                )?,
                otap_df_config::node::NodeKind::Exporter => self.create_exporter(
                    &mut nodes,
                    node_id.clone(),
                    node_config.clone(),
                )?,
                otap_df_config::node::NodeKind::ProcessorChain => {
                    // ToDo(LQ): Implement processor chain support.
                    return Err(Error::UnsupportedNodeKind {
                        kind: "ProcessorChain".into(),
                    });
                }
            }
        }

        for hyper_edge in iter_hyper_edges_runtime(&nodes) {
            // Both source and destination nodes are locals: we can create a local channel based on
            // the dispatch strategy.
            // - When we have a single source and a single destination, we can use a local SPSC channel.
            // - When we have a single source and multiple destinations, we can use a local SPMC channel.
            // Otherwise, we need to create a shared channel.
            // - When we have a singe source and multiple destinations, we can use a tokio SPSC channel.
            // - When we have a single source and multiple destinations, we can use a tokio SPMC channel.
            
            // We can start simple with MPMC channels for all cases.
        }
        Ok(RuntimePipeline::new(config, nodes))
    }

    /// Creates a receiver node and adds it to the list of runtime nodes.
    fn create_receiver(
        &self,
        nodes: &mut HashMap<NodeId, RuntimeNode<PData>>,
        receiver_id: NodeId,
        node_config: Rc<NodeConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = self.get_receiver_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownReceiver {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let receiver_config = ReceiverConfig::new(receiver_id.clone());
        let create = factory.create;

        let prev_node = nodes.insert(receiver_id.clone(), RuntimeNode::Receiver {
            config: node_config.clone(),
            instance: create(&node_config.config, &receiver_config),
            control_sender: None,
            control_receiver: None,
        });
        if prev_node.is_some() {
            return Err(Error::ReceiverAlreadyExists {
                receiver: receiver_id,
            });
        }
        Ok(())
    }

    /// Creates a processor node and adds it to the list of runtime nodes.
    fn create_processor(
        &self,
        nodes: &mut HashMap<NodeId, RuntimeNode<PData>>,
        processor_id: NodeId,
        node_config: Rc<NodeConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = self.get_processor_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownProcessor {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let processor_config = ProcessorConfig::new(processor_id.clone());
        let create = factory.create;
        let prev_node = nodes.insert(processor_id.clone(), RuntimeNode::Processor {
            config: node_config.clone(),
            instance: create(&node_config.config, &processor_config),
            control_sender: None,
            control_receiver: None,
            pdata_sender: None,
            pdata_receiver: None,
        });
        if prev_node.is_some() {
            return Err(Error::ProcessorAlreadyExists {
                processor: processor_id,
            });
        }
        Ok(())
    }

    /// Creates an exporter node and adds it to the list of runtime nodes.
    fn create_exporter(
        &self,
        nodes: &mut HashMap<NodeId, RuntimeNode<PData>>,
        exporter_id: NodeId,
        node_config: Rc<NodeConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = self.get_exporter_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownExporter {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let exporter_config = ExporterConfig::new(exporter_id.clone());
        let create = factory.create;
        let prev_node = nodes.insert(exporter_id.clone(), RuntimeNode::Exporter {
            config: node_config.clone(),
            instance: create(&node_config.config, &exporter_config),
            control_sender: None,
            control_receiver: None,
            pdata_sender: None,
            pdata_receiver: None,
        });
        if prev_node.is_some() {
            return Err(Error::ExporterAlreadyExists {
                exporter: exporter_id,
            });
        }
        Ok(())
    }
}

/// Represents a hyper-edge in the runtime graph, corresponding to a source node's output port,
/// its dispatch strategy, and the set of destination runtime nodes connected to that port.
struct HyperEdgeRuntime<'a, PData> {
    source: &'a RuntimeNode<PData>,
    port: &'a PortName,
    dispatch_strategy: &'a DispatchStrategy,
    destinations: Vec<&'a RuntimeNode<PData>>,
}


/// Returns an iterator over all hyper-edges in the runtime graph.
///
/// Each item represents a source node, one of its output ports, the dispatch strategy for that port,
/// and the destination runtime nodes connected to it.
fn iter_hyper_edges_runtime<'a, PData>(
    nodes: &'a HashMap<NodeId, RuntimeNode<PData>>,
) -> impl Iterator<Item = HyperEdgeRuntime<'a, PData>> + 'a {
    nodes.iter().flat_map(move |(_, node)| {
        let config = match node {
            RuntimeNode::Receiver { config, .. }
            | RuntimeNode::Processor { config, .. }
            | RuntimeNode::Exporter { config, .. } => config,
        };

        config.out_ports.iter().filter_map(move |(port, port_cfg)| {
            let destinations = port_cfg
                .destinations
                .iter()
                .filter_map(|dest_id| nodes.get(dest_id))
                .collect::<Vec<_>>();

            if destinations.is_empty() {
                return None;
            }

            Some(HyperEdgeRuntime {
                source: node,
                port,
                dispatch_strategy: &port_cfg.dispatch_strategy,
                destinations,
            })
        })
    })
}