// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use crate::control::{
    ControlMsg, Controllable, NodeRequestReceiver, PipelineControlMsg, node_request_channel,
};
use crate::error::Error;
use crate::message::Sender;
use crate::node::Node;
use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};
use otap_df_config::{NodeId, pipeline::PipelineConfig};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Debug;
use tokio::runtime::Builder;
use tokio::task::LocalSet;
use tokio::time::Instant;

/// Represents a runtime pipeline configuration that includes nodes with their respective configurations and instances.
pub struct RuntimePipeline<PData: Debug> {
    /// The pipeline configuration that defines the structure and behavior of the pipeline.
    config: PipelineConfig,
    /// A map node id to receiver runtime node.
    receivers: HashMap<NodeId, ReceiverWrapper<PData>>,
    /// A map node id to processor runtime node.
    processors: HashMap<NodeId, ProcessorWrapper<PData>>,
    /// A map node id to exporter runtime node.
    exporters: HashMap<NodeId, ExporterWrapper<PData>>,
    /// A precomputed map of all node IDs to their Node trait objects for efficient access
    nodes: HashMap<NodeId, NodeType>,
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

impl<PData: 'static + Debug> RuntimePipeline<PData> {
    /// Creates a new `RuntimePipeline` from the given pipeline configuration and nodes.
    #[must_use]
    pub fn new(
        config: PipelineConfig,
        receivers: HashMap<NodeId, ReceiverWrapper<PData>>,
        processors: HashMap<NodeId, ProcessorWrapper<PData>>,
        exporters: HashMap<NodeId, ExporterWrapper<PData>>,
    ) -> Self {
        let mut nodes = HashMap::new();
        for id in receivers.keys() {
            _ = nodes.insert(id.clone(), NodeType::Receiver);
        }
        for id in processors.keys() {
            _ = nodes.insert(id.clone(), NodeType::Processor);
        }
        for id in exporters.keys() {
            _ = nodes.insert(id.clone(), NodeType::Exporter);
        }
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
        let (node_req_tx, node_req_rx) = node_request_channel(
            self.config
                .pipeline_settings()
                .default_pipeline_ctrl_msg_channel_size,
        );

        // Create a task for each node type and pass the node request channel to each node, so
        // they can communicate with the runtime pipeline.
        for (node_id, exporter) in self.exporters {
            _ = control_senders.insert(node_id, exporter.control_sender());
            let node_req_tx = node_req_tx.clone();
            futures.push(local_tasks.spawn_local(async move { exporter.start(node_req_tx).await }));
        }
        for (node_id, processor) in self.processors {
            _ = control_senders.insert(node_id, processor.control_sender());
            let node_req_tx = node_req_tx.clone();
            futures
                .push(local_tasks.spawn_local(async move { processor.start(node_req_tx).await }));
        }
        for (node_id, receiver) in self.receivers {
            _ = control_senders.insert(node_id, receiver.control_sender());
            let node_req_tx = node_req_tx.clone();
            futures.push(local_tasks.spawn_local(async move { receiver.start(node_req_tx).await }));
        }

        // Create a task to process pipeline control messages, i.e. messages sent from nodes to
        // the pipeline engine.
        futures.push(local_tasks.spawn_local(async move {
            let timer_manager = NodeRequestManager::new(node_req_rx, control_senders);
            timer_manager.run().await;
            Ok(())
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
    pub fn get_node(&self, node_id: &NodeId) -> Option<&dyn Node> {
        match self.nodes.get(node_id)? {
            NodeType::Receiver => self.receivers.get(node_id).map(|r| r as &dyn Node),
            NodeType::Processor => self.processors.get(node_id).map(|p| p as &dyn Node),
            NodeType::Exporter => self.exporters.get(node_id).map(|e| e as &dyn Node),
        }
    }

    /// Sends a control message to the specified node.
    pub async fn send_control_message(
        &self,
        node_id: NodeId,
        ctrl_msg: ControlMsg,
    ) -> Result<(), Error<PData>> {
        if let Some(node) = self.get_node(&node_id) {
            node.send_control_msg(ctrl_msg)
                .await
                .map_err(|e| Error::ControlMsgSendError {
                    node: node_id.clone(),
                    error: e,
                })
        } else {
            Err(Error::UnknownNode { node_id })
        }
    }
}

/// Manages node requests such as recurrent and cancelable timers.
///
/// For now, this manager is only responsible for managing timers for nodes in the pipeline.
/// It uses a priority queue to efficiently handle timer expirations and cancellations.
///
/// - On StartTimer: adds a timer for the node with the specified duration.
/// - On CancelTimer: marks the timer for the node as canceled.
/// - On timer expiration: if not canceled, processes the timer expiration for the node.
///
/// This manager is optimized for a single-threaded runtime.
///
/// Current limitations:
/// - Does not support multiple timers for the same node.
/// - Does not support other types of node requests yet (ACK/NACK).
pub struct NodeRequestManager {
    node_request_receiver: NodeRequestReceiver,
    control_senders: HashMap<NodeId, Sender<ControlMsg>>,
    timers: BinaryHeap<Reverse<(Instant, NodeId)>>,
    canceled: HashSet<NodeId>,
    timer_map: HashMap<NodeId, Instant>,
    durations: HashMap<NodeId, std::time::Duration>,
}

impl NodeRequestManager {
    /// Creates a new NodeRequestManager.
    #[must_use]
    pub fn new(
        node_request_receiver: NodeRequestReceiver,
        control_senders: HashMap<NodeId, Sender<ControlMsg>>,
    ) -> Self {
        Self {
            node_request_receiver,
            control_senders,
            timers: BinaryHeap::new(),
            canceled: HashSet::new(),
            timer_map: HashMap::new(),
            durations: HashMap::new(),
        }
    }

    /// Runs the manager event loop.
    pub async fn run(mut self) {
        loop {
            let next_expiry = self.timers.peek().map(|Reverse((instant, _))| *instant);
            tokio::select! {
                biased;
                msg = self.node_request_receiver.recv() => {
                    let Some(msg) = msg.ok() else { break; };
                    match msg {
                        PipelineControlMsg::Shutdown => break,
                        PipelineControlMsg::StartTimer { node_id, duration } => {
                            let when = Instant::now() + duration;
                            self.timers.push(Reverse((when, node_id.clone())));
                            let _ = self.timer_map.insert(node_id.clone(), when);
                            let _ = self.durations.insert(node_id.clone(), duration);
                            let _ = self.canceled.remove(&node_id);
                        }
                        PipelineControlMsg::CancelTimer { node_id } => {
                            let _ = self.canceled.insert(node_id.clone());
                            let _ = self.timer_map.remove(&node_id);
                            let _ = self.durations.remove(&node_id);
                        }
                    }
                }
                _ = async {
                    if let Some(when) = next_expiry {
                        let now = Instant::now();
                        if when > now {
                            tokio::time::sleep_until(when).await;
                        }
                    } else {
                        futures::future::pending::<()>().await;
                    }
                }, if next_expiry.is_some() => {
                    if let Some(Reverse((when, node_id))) = self.timers.pop() {
                        if !self.canceled.contains(&node_id) {
                            if let Some(&exp) = self.timer_map.get(&node_id) {
                                if exp == when {
                                    // Timer fires: handle expiration
                                    if let Some(sender) = self.control_senders.get(&node_id) {
                                        let _ = sender.send(ControlMsg::TimerTick {}).await;
                                    } else {
                                        eprintln!("No control sender for node: {node_id}");
                                    }

                                    // Schedule next recurrence if still not canceled
                                    if let Some(&duration) = self.durations.get(&node_id) {
                                        let next_when = Instant::now() + duration;
                                        self.timers.push(Reverse((next_when, node_id.clone())));
                                        let _ = self.timer_map.insert(node_id.clone(), next_when);
                                    }
                                }
                            }
                        } else {
                            let _ = self.timer_map.remove(&node_id);
                            let _ = self.durations.remove(&node_id);
                            let _ = self.canceled.remove(&node_id);
                        }
                    }
                }
            }
        }
    }
}
