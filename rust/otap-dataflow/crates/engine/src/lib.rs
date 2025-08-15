// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

use crate::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    error::Error,
    exporter::ExporterWrapper,
    local::message::{LocalReceiver, LocalSender},
    message::{Receiver, Sender},
    node::{Node, NodeDefs, NodeId, NodeType},
    processor::ProcessorWrapper,
    receiver::ReceiverWrapper,
    runtime_pipeline::{PipeNode, RuntimePipeline},
    shared::message::{SharedReceiver, SharedSender},
};
use otap_df_config::{
    NodeId as NodeName, PortName,
    node::{DispatchStrategy, NodeUserConfig},
    pipeline::PipelineConfig,
};
use serde_json::Value;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::{collections::HashMap, sync::OnceLock};

pub mod error;
pub mod exporter;
pub mod message;
pub mod processor;
pub mod receiver;
pub mod retry_processor;

pub mod config;
pub mod control;
mod effect_handler;
pub mod local;
pub mod node;
pub mod pipeline_ctrl;
pub mod runtime_pipeline;
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
    pub create: fn(
        node: NodeId,
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
        node: NodeId,
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
        node: NodeId,
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
pub const fn build_factory<PData: 'static + Debug + Clone>() -> PipelineFactory<PData> {
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
pub struct PipelineFactory<PData: 'static + Debug + Clone> {
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
        factory: &PipelineFactory<PData>,
        config: PipelineConfig,
    ) -> Result<RuntimePipeline<PData>, Error<PData>> {
        let mut receivers = Vec::new();
        let mut processors = Vec::new();
        let mut exporters = Vec::new();
        let mut receiver_names = HashMap::new();
        let mut processor_names = HashMap::new();
        let mut exporter_names = HashMap::new();
        let mut nodes = NodeDefs::default();

        // Create runtime nodes based on the pipeline configuration.
        // ToDo(LQ): Collect all errors instead of failing fast to provide better feedback.
        for (name, node_config) in config.node_iter() {
            match node_config.kind {
                otap_df_config::node::NodeKind::Receiver => Self::create_receiver(
                    factory,
                    &mut receiver_names,
                    &mut nodes,
                    &mut receivers,
                    name.clone(),
                    node_config.clone(),
                )?,
                otap_df_config::node::NodeKind::Processor => Self::create_processor(
                    factory,
                    &mut processor_names,
                    &mut nodes,
                    &mut processors,
                    name.clone(),
                    node_config.clone(),
                )?,
                otap_df_config::node::NodeKind::Exporter => Self::create_exporter(
                    factory,
                    &mut exporter_names,
                    &mut nodes,
                    &mut exporters,
                    name.clone(),
                    node_config.clone(),
                )?,
                otap_df_config::node::NodeKind::ProcessorChain => {
                    // ToDo(LQ): Implement processor chain optimization to eliminate intermediary channels.
                    return Err(Error::UnsupportedNodeKind {
                        kind: "ProcessorChain".into(),
                    });
                }
            }
        }

        let edges = collect_hyper_edges_runtime(&receivers, &processors);

        let mut pipeline = RuntimePipeline::new(config, receivers, processors, exporters, nodes);

        // First pass: collect all channel assignments to avoid multiple mutable borrows
        struct ChannelAssignment<PData> {
            source: NodeId,
            port: PortName,
            sender: Sender<PData>,
            destinations: Vec<(NodeId, Receiver<PData>)>,
        }
        let mut assignments = Vec::new();
        for hyper_edge in edges {
            // Get source node
            let src_node =
                pipeline
                    .get_node(hyper_edge.source.index)
                    .ok_or_else(|| Error::UnknownNode {
                        node: hyper_edge.source.name.clone(),
                    })?;

            // Get destination nodes: note the order of dest_nodes matches hyper_edge.destinations
            // and preserved by select_channel_type(). The zip() below depends on both of these.
            let mut dest_nodes = Vec::with_capacity(hyper_edge.destinations.len());
            for name in &hyper_edge.destinations {
                let node = processor_names
                    .get(name)
                    .map(|id| pipeline.get_node(id.index).expect("ok"))
                    .or_else(|| {
                        exporter_names
                            .get(name)
                            .map(|id| pipeline.get_node(id.index).expect("ok"))
                    })
                    .ok_or_else(|| Error::UnknownNode { node: name.clone() })?;
                dest_nodes.push(node);
            }

            // Select channel type
            let (pdata_sender, pdata_receivers) = Self::select_channel_type(
                src_node,
                &dest_nodes,
                NonZeroUsize::new(1000).expect("Buffer size must be non-zero"),
            )?;

            // Prepare assignments
            let destinations = dest_nodes
                .into_iter()
                .map(|n| n.node_id())
                .zip(pdata_receivers.into_iter())
                .collect();
            assignments.push(ChannelAssignment {
                source: hyper_edge.source,
                port: hyper_edge.port,
                sender: pdata_sender,
                destinations,
            });
        }

        // Second pass: perform all assignments
        for assignment in assignments {
            let src_node = pipeline
                .get_mut_sender(assignment.source.index)
                .ok_or_else(|| Error::UnknownNode {
                    node: assignment.source.name.clone(),
                })?;
            src_node.set_pdata_sender(
                assignment.source,
                assignment.port.clone(),
                assignment.sender,
            )?;
            for (dest, receiver) in assignment.destinations {
                let dest_node =
                    pipeline
                        .get_mut_receiver(dest.index)
                        .ok_or_else(|| Error::UnknownNode {
                            node: dest.name.clone(),
                        })?;
                dest_node.set_pdata_receiver(dest, receiver)?;
            }
        }
        Ok(pipeline)
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
        dest_nodes: &Vec<&dyn Node>,
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
        factory: &PipelineFactory<PData>,
        names: &mut HashMap<NodeName, NodeId>,
        nodes: &mut NodeDefs<PData, PipeNode>,
        receivers: &mut Vec<ReceiverWrapper<PData>>,
        name: NodeName,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = factory
            .get_receiver_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownReceiver {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let runtime_config = ReceiverConfig::new(name.clone());
        let create = factory.create;

        let node_id = nodes.next(
            name.clone(),
            NodeType::Receiver,
            PipeNode::new(receivers.len()),
        )?;
        if names.insert(name.clone(), node_id.clone()).is_some() {
            return Err(Error::ReceiverAlreadyExists { receiver: node_id });
        }

        receivers.push(
            create(node_id, node_config, &runtime_config)
                .map_err(|e| Error::ConfigError(Box::new(e)))?,
        );
        Ok(())
    }

    /// Creates a processor node and adds it to the list of runtime nodes.
    fn create_processor(
        factory: &PipelineFactory<PData>,
        names: &mut HashMap<NodeName, NodeId>,
        nodes: &mut NodeDefs<PData, PipeNode>,
        processors: &mut Vec<ProcessorWrapper<PData>>,
        name: NodeName,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = factory
            .get_processor_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownProcessor {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let processor_config = ProcessorConfig::new(name.clone());
        let create = factory.create;

        let node_id = nodes.next(
            name.clone(),
            NodeType::Processor,
            PipeNode::new(processors.len()),
        )?;
        if names.insert(name.clone(), node_id.clone()).is_some() {
            return Err(Error::ProcessorAlreadyExists { processor: node_id });
        }
        processors.push(
            create(node_id, &node_config.config, &processor_config)
                .map_err(|e| Error::ConfigError(Box::new(e)))?,
        );

        Ok(())
    }

    /// Creates an exporter node and adds it to the list of runtime nodes.
    fn create_exporter(
        factory: &PipelineFactory<PData>,
        names: &mut HashMap<NodeName, NodeId>,
        nodes: &mut NodeDefs<PData, PipeNode>,
        exporters: &mut Vec<ExporterWrapper<PData>>,
        name: NodeName,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = factory
            .get_exporter_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownExporter {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let exporter_config = ExporterConfig::new(name.clone());
        let create = factory.create;

        let node_id = nodes.next(
            name.clone(),
            NodeType::Exporter,
            PipeNode::new(exporters.len()),
        )?;

        if names.insert(name.clone(), node_id.clone()).is_some() {
            return Err(Error::ExporterAlreadyExists { exporter: node_id });
        }
        exporters.push(
            create(node_id, node_config, &exporter_config)
                .map_err(|e| Error::ConfigError(Box::new(e)))?,
        );
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

    // names are from the configuration, not yet resolved
    destinations: Vec<NodeName>,
}

/// Returns a vector of all hyper-edges in the runtime graph.
///
/// Each item represents a hyper-edge with source node id, port, dispatch strategy, and destination
/// node ids.
fn collect_hyper_edges_runtime<PData>(
    receivers: &Vec<ReceiverWrapper<PData>>,
    processors: &Vec<ProcessorWrapper<PData>>,
) -> Vec<HyperEdgeRuntime> {
    let mut edges = Vec::new();
    let mut nodes: Vec<&dyn Node> = Vec::new();
    nodes.extend(receivers.iter().map(|node| node as &dyn Node));
    nodes.extend(processors.iter().map(|node| node as &dyn Node));

    for node in nodes {
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
                source: node.node_id(),
                port: port.clone(),
                dispatch_strategy: hyper_edge_cfg.dispatch_strategy.clone(),
                destinations,
            });
        }
    }
    edges
}
