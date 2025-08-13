// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

use crate::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    error::Error,
    exporter::ExporterWrapper,
    message::Sender,
    processor::ProcessorWrapper,
    receiver::ReceiverWrapper,
    runtime_pipeline::RuntimePipeline,
};
use serde_json::Value;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::{collections::HashMap, sync::OnceLock};

pub mod context;
pub mod error;
pub mod exporter;
pub mod message;
pub mod processor;
pub mod receiver;

pub mod config;
pub mod control;
mod effect_handler;
pub mod local;
pub mod node;
pub mod pipeline_ctrl;
pub mod runtime_pipeline;
pub mod shared;
pub mod testing;

use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::Receiver;
use crate::node::{NodeWithPDataReceiver, NodeWithPDataSender};
use crate::shared::message::{SharedReceiver, SharedSender};
pub use linkme::distributed_slice;
use node::Node;
use otap_df_config::node::{DispatchStrategy, NodeUserConfig};
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
    pub create: fn(
        node_config: Arc<NodeUserConfig>,
        receiver_config: &ReceiverConfig,
    ) -> Result<ReceiverWrapper<PData>, otap_df_config::error::Error>,
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
    pub create: fn(
        config: &Value,
        processor_config: &ProcessorConfig,
    ) -> Result<ProcessorWrapper<PData>, otap_df_config::error::Error>,
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
    pub create: fn(
        node_config: Arc<NodeUserConfig>,
        exporter_config: &ExporterConfig,
    ) -> Result<ExporterWrapper<PData>, otap_df_config::error::Error>,
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

/// Builds a pipeline factory for initialization.
///
/// This function is used as a placeholder when declaring a pipeline factory with the
/// `#[factory_registry]` attribute macro. The macro will replace this placeholder with
/// proper lazy initialization using `LazyLock`.
///
/// # Example
/// ```rust,ignore
/// #[factory_registry(MyData)]
/// static FACTORY_REGISTRY: PipelineFactory<MyData> = build_factory();
/// ```
#[must_use]
pub const fn build_factory<PData: 'static + Clone>() -> PipelineFactory<PData> {
    // This function should never actually be called since the macro replaces it entirely.
    // If it is called, that indicates a bug in the macro system.
    panic!(
        "build_registry() should never be called - it's replaced by the #[factory_registry] macro"
    )
}

/// A pipeline factory.
///
/// This factory contains a registry of all the micro-factories for receivers, processors, and
/// exporters, as well as the logic for creating pipelines based on a given configuration.
pub struct PipelineFactory<PData: 'static + Clone> {
    receiver_factory_map: OnceLock<HashMap<&'static str, ReceiverFactory<PData>>>,
    processor_factory_map: OnceLock<HashMap<&'static str, ProcessorFactory<PData>>>,
    exporter_factory_map: OnceLock<HashMap<&'static str, ExporterFactory<PData>>>,
    receiver_factories: &'static [ReceiverFactory<PData>],
    processor_factories: &'static [ProcessorFactory<PData>],
    exporter_factories: &'static [ExporterFactory<PData>],
}

impl<PData: 'static + Clone + Debug> PipelineFactory<PData> {
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

    /// Builds a runtime pipeline from the given pipeline configuration.
    ///
    /// Steps:
    /// - Create all runtime nodes based on the pipeline configuration.
    /// - Analyze both the local vs shared nature of each pair of connected nodes and the nature of
    ///   the hyper-edges between them to determine the best channel type.
    /// - Assign channels to the source nodes and their destination nodes based on the previous
    ///   analysis.
    pub fn build(
        &self,
        config: otap_df_config::pipeline::PipelineConfig,
    ) -> Result<RuntimePipeline<PData>, Error<PData>> {
        let mut receivers = HashMap::new();
        let mut processors = HashMap::new();
        let mut exporters = HashMap::new();

        // Create runtime nodes based on the pipeline configuration.
        // ToDo(LQ): Collect all errors instead of failing fast to provide better feedback.
        for (node_id, node_config) in config.node_iter() {
            match node_config.kind {
                otap_df_config::node::NodeKind::Receiver => {
                    self.create_receiver(&mut receivers, node_id.clone(), node_config.clone())?
                }
                otap_df_config::node::NodeKind::Processor => {
                    self.create_processor(&mut processors, node_id.clone(), node_config.clone())?
                }
                otap_df_config::node::NodeKind::Exporter => {
                    self.create_exporter(&mut exporters, node_id.clone(), node_config.clone())?
                }
                otap_df_config::node::NodeKind::ProcessorChain => {
                    // ToDo(LQ): Implement processor chain optimization to eliminate intermediary channels.
                    return Err(Error::UnsupportedNodeKind {
                        kind: "ProcessorChain".into(),
                    });
                }
            }
        }

        // First pass: collect all channel assignments to avoid multiple mutable borrows
        struct ChannelAssignment<PData> {
            source_id: NodeId,
            port: PortName,
            sender: Sender<PData>,
            destinations: Vec<(NodeId, Receiver<PData>)>,
        }
        let mut assignments = Vec::new();
        for hyper_edge in collect_hyper_edges_runtime(&receivers, &processors) {
            // Get source node
            let src_node = receivers
                .get(&hyper_edge.source)
                .map(|r| r as &dyn Node)
                .or_else(|| processors.get(&hyper_edge.source).map(|p| p as &dyn Node))
                .ok_or_else(|| Error::UnknownNode {
                    node_id: hyper_edge.source.clone(),
                })?;

            // Get destination nodes
            let mut dest_nodes = Vec::with_capacity(hyper_edge.destinations.len());
            for id in &hyper_edge.destinations {
                let node = processors
                    .get(id)
                    .map(|p| p as &dyn Node)
                    .or_else(|| exporters.get(id).map(|e| e as &dyn Node))
                    .ok_or_else(|| Error::UnknownNode {
                        node_id: id.clone(),
                    })?;
                dest_nodes.push(node);
            }

            // Select channel type
            let (sender, receivers) = Self::select_channel_type(
                src_node,
                dest_nodes,
                NonZeroUsize::new(1000).expect("Buffer size must be non-zero"),
            )?;

            // Prepare assignments
            let destinations = hyper_edge
                .destinations
                .iter()
                .cloned()
                .zip(receivers.into_iter())
                .collect();
            assignments.push(ChannelAssignment {
                source_id: hyper_edge.source,
                port: hyper_edge.port,
                sender,
                destinations,
            });
        }

        // Second pass: perform all assignments
        for assignment in assignments {
            let src_node = receivers
                .get_mut(&assignment.source_id)
                .map(|n| n as &mut dyn NodeWithPDataSender<PData>)
                .or_else(|| {
                    processors
                        .get_mut(&assignment.source_id)
                        .map(|n| n as &mut dyn NodeWithPDataSender<PData>)
                })
                .ok_or_else(|| Error::UnknownNode {
                    node_id: assignment.source_id.clone(),
                })?;
            src_node.set_pdata_sender(
                assignment.source_id.clone(),
                assignment.port.clone(),
                assignment.sender,
            )?;
            for (dest_id, receiver) in assignment.destinations {
                let dest_node = processors
                    .get_mut(&dest_id)
                    .map(|n| n as &mut dyn NodeWithPDataReceiver<PData>)
                    .or_else(|| {
                        exporters
                            .get_mut(&dest_id)
                            .map(|n| n as &mut dyn NodeWithPDataReceiver<PData>)
                    })
                    .ok_or_else(|| Error::UnknownNode {
                        node_id: dest_id.clone(),
                    })?;
                dest_node.set_pdata_receiver(dest_id, receiver)?;
            }
        }
        Ok(RuntimePipeline::new(
            config, receivers, processors, exporters,
        ))
    }

    /// Determines the best channel type from the following parameters:
    /// - Flag specifying if the channel is shared (true) or local (false).
    /// - The number of destinations connected to the channel.
    /// - The dispatch strategy for the channel (not yet supported).
    ///
    /// This function returns a tuple containing the selected sender and one receiver per
    /// destination.
    ///
    /// ToDo (LQ): Support dispatch strategies.
    fn select_channel_type(
        src_node: &dyn Node,
        dest_nodes: Vec<&dyn Node>,
        buffer_size: NonZeroUsize,
    ) -> Result<(Sender<PData>, Vec<Receiver<PData>>), Error<PData>> {
        let source_is_shared = src_node.is_shared();
        let any_dest_is_shared = dest_nodes.iter().any(|dest| dest.is_shared());
        let use_shared_channels = source_is_shared || any_dest_is_shared;
        let num_destinations = dest_nodes.len();

        if use_shared_channels {
            // Shared channels
            if num_destinations > 1 {
                let (pdata_sender, pdata_receiver) = flume::bounded(buffer_size.get());
                let pdata_receivers = (0..num_destinations)
                    .map(|_| Receiver::Shared(SharedReceiver::MpmcReceiver(pdata_receiver.clone())))
                    .collect::<Vec<_>>();
                Ok((
                    Sender::Shared(SharedSender::MpmcSender(pdata_sender)),
                    pdata_receivers,
                ))
            } else {
                let (pdata_sender, pdata_receiver) =
                    tokio::sync::mpsc::channel::<PData>(buffer_size.get());
                Ok((
                    Sender::Shared(SharedSender::MpscSender(pdata_sender)),
                    vec![Receiver::Shared(SharedReceiver::MpscReceiver(
                        pdata_receiver,
                    ))],
                ))
            }
        } else {
            // Local channels
            if num_destinations > 1 {
                // ToDo(LQ): Use a local SPMC channel when available.
                let (pdata_sender, pdata_receiver) =
                    otap_df_channel::mpmc::Channel::new(buffer_size);
                let pdata_receivers = (0..num_destinations)
                    .map(|_| Receiver::Local(LocalReceiver::MpmcReceiver(pdata_receiver.clone())))
                    .collect::<Vec<_>>();
                Ok((
                    Sender::Local(LocalSender::MpmcSender(pdata_sender)),
                    pdata_receivers,
                ))
            } else {
                // ToDo(LQ): Use a local SPSC channel when available.
                let (pdata_sender, pdata_receiver) =
                    otap_df_channel::mpsc::Channel::new(buffer_size.get());
                Ok((
                    Sender::Local(LocalSender::MpscSender(pdata_sender)),
                    vec![Receiver::Local(LocalReceiver::MpscReceiver(pdata_receiver))],
                ))
            }
        }
    }

    /// Creates a receiver node and adds it to the list of runtime nodes.
    fn create_receiver(
        &self,
        receivers: &mut HashMap<NodeId, ReceiverWrapper<PData>>,
        receiver_id: NodeId,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = self
            .get_receiver_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownReceiver {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let runtime_config = ReceiverConfig::new(receiver_id.clone());
        let create = factory.create;

        let prev_node = receivers.insert(
            receiver_id.clone(),
            create(node_config, &runtime_config).map_err(|e| Error::ConfigError(Box::new(e)))?,
        );
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
        nodes: &mut HashMap<NodeId, ProcessorWrapper<PData>>,
        processor_id: NodeId,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = self
            .get_processor_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownProcessor {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let processor_config = ProcessorConfig::new(processor_id.clone());
        let create = factory.create;
        let prev_node = nodes.insert(
            processor_id.clone(),
            create(&node_config.config, &processor_config)
                .map_err(|e| Error::ConfigError(Box::new(e)))?,
        );
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
        nodes: &mut HashMap<NodeId, ExporterWrapper<PData>>,
        exporter_id: NodeId,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = self
            .get_exporter_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownExporter {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let exporter_config = ExporterConfig::new(exporter_id.clone());
        let create = factory.create;
        let prev_node = nodes.insert(
            exporter_id.clone(),
            create(node_config, &exporter_config).map_err(|e| Error::ConfigError(Box::new(e)))?,
        );
        if prev_node.is_some() {
            return Err(Error::ExporterAlreadyExists {
                exporter: exporter_id,
            });
        }
        Ok(())
    }
}

/// Represents a hyper-edge in the runtime graph, corresponding to a source node's output port,
/// its dispatch strategy, and the set of destination node ids connected to that port.
struct HyperEdgeRuntime {
    source: NodeId,
    // ToDo(LQ): Use port name for telemetry and debugging purposes.
    port: PortName,
    #[allow(dead_code)]
    dispatch_strategy: DispatchStrategy,
    destinations: Vec<NodeId>,
}

/// Returns a vector of all hyper-edges in the runtime graph.
///
/// Each item represents a hyper-edge with source node id, port, dispatch strategy, and destination
/// node ids.
fn collect_hyper_edges_runtime<PData>(
    receivers: &HashMap<NodeId, ReceiverWrapper<PData>>,
    processors: &HashMap<NodeId, ProcessorWrapper<PData>>,
) -> Vec<HyperEdgeRuntime> {
    let mut edges = Vec::new();
    let mut nodes: Vec<(&NodeId, &dyn Node)> = Vec::new();
    nodes.extend(receivers.iter().map(|(id, node)| (id, node as &dyn Node)));
    nodes.extend(processors.iter().map(|(id, node)| (id, node as &dyn Node)));

    for (node_id, node) in nodes {
        let config = node.user_config();
        for (port, hyper_edge_cfg) in &config.out_ports {
            let destinations = hyper_edge_cfg
                .destinations
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            if destinations.is_empty() {
                continue;
            }
            edges.push(HyperEdgeRuntime {
                source: node_id.clone(),
                port: port.clone(),
                dispatch_strategy: hyper_edge_cfg.dispatch_strategy.clone(),
                destinations,
            });
        }
    }
    edges
}
