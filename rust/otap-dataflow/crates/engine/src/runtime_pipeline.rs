// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use crate::config::{ExporterConfig, ProcessorConfig, ReceiverConfig};
use crate::context::{NodeDefinition, NodeUniq, Unique};
use crate::control::{Controllable, NodeControlMsg, pipeline_ctrl_msg_channel};
use crate::error::Error;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::{Receiver, Sender};
use crate::node::{Node, NodeWithPDataReceiver, NodeWithPDataSender};
use crate::pipeline_ctrl::PipelineCtrlMsgManager;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::{ExporterFactory, ProcessorFactory, ReceiverFactory};
use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};

use otap_df_config::PortName;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::{NodeId, node::DispatchStrategy, pipeline::PipelineConfig};
use std::collections::HashMap;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::{Arc, OnceLock};
use tokio::runtime::Builder;
use tokio::task::LocalSet;

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

/// Represents a runtime pipeline configuration that includes nodes with their respective configurations and instances.
///
/// Note: Having a Debug bound on `PData` allows us to use it in logging and debugging contexts,
/// which is useful for tracing the pipeline's execution and state.
pub struct RuntimePipeline<PData: Debug> {
    /// The pipeline configuration that defines the structure and behavior of the pipeline.
    config: PipelineConfig,
    /// A map node id to receiver runtime node.
    receivers: HashMap<Unique, ReceiverWrapper<PData>>,
    /// A map node id to processor runtime node.
    processors: HashMap<Unique, ProcessorWrapper<PData>>,
    /// A map node id to exporter runtime node.
    exporters: HashMap<Unique, ExporterWrapper<PData>>,
    /// A precomputed map of all node IDs to their Node trait objects (? @@@) for efficient access
    /// Indexed by Unique
    nodes: Vec<NodeDefinition>,
}

/// Represents a hyper-edge in the runtime graph, corresponding to a source node's output port,
/// its dispatch strategy, and the set of destination node ids connected to that port.
pub(crate) struct HyperEdgeRuntime {
    pub(crate) source: NodeUniq,

    // ToDo(LQ): Use port name for telemetry and debugging purposes.
    pub(crate) port: PortName,

    #[allow(dead_code)]
    dispatch_strategy: DispatchStrategy,

    pub(crate) destinations: Vec<NodeUniq>,
}

/// Enum to identify the type of a node for registry lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Represents a node that acts as a receiver, receiving data from an external source.
    Receiver,
    /// Represents a node that processes data, transforming or analyzing it.
    Processor,
    /// Represents a node that exports data to an external destination.
    Exporter,
}

impl<PData: 'static + Debug + Clone> RuntimePipeline<PData> {
    /// Creates a new `RuntimePipeline` from the given pipeline configuration and nodes.
    #[must_use]
    fn new(
        config: PipelineConfig,
        receivers: HashMap<Unique, ReceiverWrapper<PData>>,
        processors: HashMap<Unique, ProcessorWrapper<PData>>,
        exporters: HashMap<Unique, ExporterWrapper<PData>>,
        nodes: Vec<NodeDefinition>,
    ) -> Self {
        Self {
            config,
            receivers,
            processors,
            exporters,
            nodes,
        }
    }

    /// Returns the number of nodes in the pipeline.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.receivers.len() + self.processors.len() + self.exporters.len()
    }

    /// Returns a reference to the pipeline configuration.
    #[must_use]
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Runs the pipeline forever, starting all nodes and handling their tasks.
    /// Returns an error if any node fails to start or if any task encounters an error.
    pub fn run_forever(self) -> Result<Vec<()>, Error<PData>> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        let local_tasks = LocalSet::new();
        // ToDo create an optimized version of FuturesUnordered that can be used for !Send, !Sync tasks
        let mut futures = FuturesUnordered::new();
        let mut control_senders = HashMap::new();
        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(
            self.config
                .pipeline_settings()
                .default_pipeline_ctrl_msg_channel_size,
        );

        // Create a task for each node type and pass the pipeline ctrl msg channel to each node, so
        // they can communicate with the runtime pipeline.
        for (node_id, exporter) in self.exporters {
            _ = control_senders.insert(node_id, exporter.control_sender());
            let pipeline_ctrl_msg_tx = pipeline_ctrl_msg_tx.clone();
            futures.push(
                local_tasks.spawn_local(async move { exporter.start(pipeline_ctrl_msg_tx).await }),
            );
        }
        for (node_id, processor) in self.processors {
            _ = control_senders.insert(node_id, processor.control_sender());
            let pipeline_ctrl_msg_tx = pipeline_ctrl_msg_tx.clone();
            futures.push(
                local_tasks.spawn_local(async move { processor.start(pipeline_ctrl_msg_tx).await }),
            );
        }
        for (node_id, receiver) in self.receivers {
            _ = control_senders.insert(node_id, receiver.control_sender());
            let pipeline_ctrl_msg_tx = pipeline_ctrl_msg_tx.clone();
            futures.push(
                local_tasks.spawn_local(async move { receiver.start(pipeline_ctrl_msg_tx).await }),
            );
        }

        // Create a task to process pipeline control messages, i.e. messages sent from nodes to
        // the pipeline engine.
        futures.push(local_tasks.spawn_local(async move {
            let manager = PipelineCtrlMsgManager::new(pipeline_ctrl_msg_rx, control_senders);
            manager.run().await
        }));

        rt.block_on(async {
            local_tasks
                .run_until(async {
                    let mut task_results = Vec::new();

                    // Process each future as they complete and handle errors
                    while let Some(result) = futures.next().await {
                        match result {
                            Ok(Ok(res)) => {
                                // Task completed successfully, collect its result
                                task_results.push(res);
                            }
                            Ok(Err(e)) => {
                                // A task returned an error
                                return Err(e);
                            }
                            Err(e) => {
                                // JoinError (panic or cancellation)
                                return Err(Error::JoinTaskError {
                                    is_canceled: e.is_cancelled(),
                                    is_panic: e.is_panic(),
                                    error: e.to_string(),
                                });
                            }
                        }
                    }
                    Ok(task_results)
                })
                .await
        })
    }

    /// Gets a reference to any node by its ID as a Node trait object
    #[must_use]
    pub fn get_node(&self, node: Unique) -> Option<&dyn Node> {
        let ndef = self.nodes.get(node.index())?;
        match ndef.ntype {
            NodeType::Receiver => self.receivers.get(&node).map(|r| r as &dyn Node),
            NodeType::Processor => self.processors.get(&node).map(|p| p as &dyn Node),
            NodeType::Exporter => self.exporters.get(&node).map(|e| e as &dyn Node),
        }
    }

    fn node_id(&self, node: Unique) -> NodeId {
        self.nodes
            .get(node.index())
            .map(|nd| nd.name.clone())
            .unwrap_or("unknown".into())
    }

    /// Sends a node control message to the specified node.
    pub async fn send_node_control_message(
        &self,
        unique: Unique,
        ctrl_msg: NodeControlMsg,
    ) -> Result<(), Error<PData>> {
        if let Some(node) = self.get_node(unique) {
            node.send_control_msg(ctrl_msg)
                .await
                .map_err(|e| Error::NodeControlMsgSendError {
                    node: self.node_id(unique),
                    error: e,
                })
        } else {
            Err(Error::UnknownNode {
                node: self.node_id(unique),
            })
        }
    }

    /// Build a new runtime from config and the factory.
    pub fn build(
        factory: &PipelineFactory<PData>,
        config: PipelineConfig,
    ) -> Result<RuntimePipeline<PData>, Error<PData>> {
        let mut receivers = HashMap::new();
        let mut processors = HashMap::new();
        let mut exporters = HashMap::new();
        let mut nodes = Vec::new();

        // Create runtime nodes based on the pipeline configuration.
        // ToDo(LQ): Collect all errors instead of failing fast to provide better feedback.
        for (node_id, node_config) in config.node_iter() {
            match node_config.kind {
                otap_df_config::node::NodeKind::Receiver => Self::create_receiver(
                    factory,
                    &mut receivers,
                    NodeUniq::next(node_id.clone(), NodeType::Receiver, &mut nodes)
                        .map_err(|_| Error::TooManyNodes {})?,
                    node_config.clone(),
                )?,
                otap_df_config::node::NodeKind::Processor => Self::create_processor(
                    factory,
                    &mut processors,
                    NodeUniq::next(node_id.clone(), NodeType::Processor, &mut nodes)
                        .map_err(|_| Error::TooManyNodes {})?,
                    node_config.clone(),
                )?,
                otap_df_config::node::NodeKind::Exporter => Self::create_exporter(
                    factory,
                    &mut exporters,
                    NodeUniq::next(node_id.clone(), NodeType::Exporter, &mut nodes)
                        .map_err(|_| Error::TooManyNodes {})?,
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

        let mut pipeline = Self::new(config, receivers, processors, exporters, nodes);

        // First pass: collect all channel assignments to avoid multiple mutable borrows
        struct ChannelAssignment<PData> {
            source: NodeUniq,
            port: PortName,
            sender: Sender<PData>,
            destinations: Vec<(NodeUniq, Receiver<PData>)>,
        }
        let mut assignments = Vec::new();
        for hyper_edge in pipeline.collect_hyper_edges_runtime()? {
            // Get source node
            let src_node = pipeline
                .receivers
                .get(&hyper_edge.source.id)
                .map(|r| r as &dyn Node)
                .or_else(|| {
                    pipeline
                        .processors
                        .get(&hyper_edge.source.id)
                        .map(|p| p as &dyn Node)
                })
                .ok_or_else(|| Error::UnknownNode {
                    node: hyper_edge.source.name.clone(),
                })?;

            // Get destination nodes: note the order of dest_nodes matches hyper_edge.destinations
            // and preserved by select_channel_type(). The zip() below depends on both of these.
            let mut dest_nodes = Vec::with_capacity(hyper_edge.destinations.len());
            for node in &hyper_edge.destinations {
                let node = pipeline
                    .processors
                    .get(&node.id)
                    .map(|p| p as &dyn Node)
                    .or_else(|| pipeline.exporters.get(&node.id).map(|e| e as &dyn Node))
                    .ok_or_else(|| Error::UnknownNode {
                        node: node.name.clone(),
                    })?;
                dest_nodes.push(node);
            }

            // Select channel type
            let (pdata_sender, pdata_receivers) = Self::select_channel_type(
                src_node,
                dest_nodes,
                NonZeroUsize::new(1000).expect("Buffer size must be non-zero"),
            )?;

            // Prepare assignments
            let destinations = hyper_edge
                .destinations
                .iter()
                .cloned()
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
                .receivers
                .get_mut(&assignment.source.id)
                .map(|n| n as &mut dyn NodeWithPDataSender<PData>)
                .or_else(|| {
                    pipeline
                        .processors
                        .get_mut(&assignment.source.id)
                        .map(|n| n as &mut dyn NodeWithPDataSender<PData>)
                })
                .ok_or_else(|| Error::UnknownNode {
                    node: assignment.source.name.clone(),
                })?;
            src_node.set_pdata_sender(
                assignment.source.name.clone(),
                assignment.port.clone(),
                assignment.sender,
            )?;
            for (dest, receiver) in assignment.destinations {
                let dest_node = pipeline
                    .processors
                    .get_mut(&dest.id)
                    .map(|n| n as &mut dyn NodeWithPDataReceiver<PData>)
                    .or_else(|| {
                        pipeline
                            .exporters
                            .get_mut(&dest.id)
                            .map(|n| n as &mut dyn NodeWithPDataReceiver<PData>)
                    })
                    .ok_or_else(|| Error::UnknownNode {
                        node: dest.name.clone(),
                    })?;
                dest_node.set_pdata_receiver(dest.name, receiver)?;
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
        factory: &PipelineFactory<PData>,
        receivers: &mut HashMap<Unique, ReceiverWrapper<PData>>,
        node: NodeUniq,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = factory
            .get_receiver_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownReceiver {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let runtime_config = ReceiverConfig::new(node.name.clone());
        let create = factory.create;

        let prev_node = receivers.insert(
            node.id,
            create(node_config, &runtime_config).map_err(|e| Error::ConfigError(Box::new(e)))?,
        );
        if prev_node.is_some() {
            return Err(Error::ReceiverAlreadyExists {
                receiver: node.name,
            });
        }
        Ok(())
    }

    /// Creates a processor node and adds it to the list of runtime nodes.
    fn create_processor(
        factory: &PipelineFactory<PData>,
        nodes: &mut HashMap<Unique, ProcessorWrapper<PData>>,
        node: NodeUniq,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = factory
            .get_processor_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownProcessor {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let processor_config = ProcessorConfig::new(node.name.clone());
        let create = factory.create;
        let prev_node = nodes.insert(
            node.id,
            create(&node_config.config, &processor_config)
                .map_err(|e| Error::ConfigError(Box::new(e)))?,
        );
        if prev_node.is_some() {
            return Err(Error::ProcessorAlreadyExists {
                processor: node.name.clone(),
            });
        }
        Ok(())
    }

    /// Creates an exporter node and adds it to the list of runtime nodes.
    fn create_exporter(
        factory: &PipelineFactory<PData>,
        nodes: &mut HashMap<Unique, ExporterWrapper<PData>>,
        node: NodeUniq,
        node_config: Arc<NodeUserConfig>,
    ) -> Result<(), Error<PData>> {
        let factory = factory
            .get_exporter_factory_map()
            .get(node_config.plugin_urn.as_ref())
            .ok_or_else(|| Error::UnknownExporter {
                plugin_urn: node_config.plugin_urn.clone(),
            })?;
        let exporter_config = ExporterConfig::new(node.name.clone());
        let create = factory.create;
        let prev_node = nodes.insert(
            node.id,
            create(node_config, &exporter_config).map_err(|e| Error::ConfigError(Box::new(e)))?,
        );
        if prev_node.is_some() {
            return Err(Error::ExporterAlreadyExists {
                exporter: node.name.clone(),
            });
        }
        Ok(())
    }

    /// Returns a vector of all hyper-edges in the runtime graph.
    ///
    /// Each item represents a hyper-edge with source node id, port, dispatch strategy, and destination
    /// node ids.
    pub(crate) fn collect_hyper_edges_runtime(
        &self,
    ) -> Result<Vec<HyperEdgeRuntime>, Error<PData>> {
        let mut edges = Vec::new();
        let mut byname: HashMap<_, &dyn Node> = HashMap::new();
        let ierr = || Error::InternalError {
            message: "invalid node index".into(),
        };

        for (idx, def) in self.nodes.iter().enumerate() {
            let u = Unique::try_from(idx).map_err(|_| ierr())?;
            _ = byname.insert(def.name.clone(), self.get_node(u).ok_or(ierr())?);
        }

        for id in self.receivers.keys().chain(self.processors.keys()) {
            let node = self.get_node(*id).ok_or(ierr())?;
            let config = node.user_config();
            for (port, hyper_edge_cfg) in &config.out_ports {
                let destinations: Vec<_> = hyper_edge_cfg
                    .destinations
                    .iter()
                    .map(|x| {
                        let node = byname.get(x).ok_or(ierr());
                        node.map(|n| n.node_uniq())
                    })
                    .collect();
                if destinations.is_empty() {
                    continue;
                }
                let destinations = {
                    let result: Result<_, _> = destinations.into_iter().collect();
                    result?
                };
                edges.push(HyperEdgeRuntime {
                    source: node.node_uniq(),
                    port: port.clone(),
                    dispatch_strategy: hyper_edge_cfg.dispatch_strategy.clone(),
                    destinations,
                });
            }
        }
        Ok(edges)
    }
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
                .map(|f| (f.name, f.clone()))
                .collect::<HashMap<&'static str, ReceiverFactory<PData>>>()
        })
    }

    /// Gets the processor factory map, initializing it if necessary.
    pub fn get_processor_factory_map(&self) -> &HashMap<&'static str, ProcessorFactory<PData>> {
        self.processor_factory_map.get_or_init(|| {
            self.processor_factories
                .iter()
                .map(|f| (f.name, f.clone()))
                .collect::<HashMap<&'static str, ProcessorFactory<PData>>>()
        })
    }

    /// Gets the exporter factory map, initializing it if necessary.
    pub fn get_exporter_factory_map(&self) -> &HashMap<&'static str, ExporterFactory<PData>> {
        self.exporter_factory_map.get_or_init(|| {
            self.exporter_factories
                .iter()
                .map(|f| (f.name, f.clone()))
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
    pub fn build(&self, config: PipelineConfig) -> Result<RuntimePipeline<PData>, Error<PData>> {
        RuntimePipeline::build(&self, config)
    }
}
