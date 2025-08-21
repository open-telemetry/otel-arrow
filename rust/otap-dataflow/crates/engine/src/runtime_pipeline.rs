// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use crate::control::{Controllable, NodeControlMsg, pipeline_ctrl_msg_channel};
use crate::error::Error;
use crate::node::{Node, NodeDefs, NodeId, NodeType, NodeWithPDataReceiver, NodeWithPDataSender};
use crate::pipeline_ctrl::PipelineCtrlMsgManager;
use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};

use otap_df_config::pipeline::PipelineConfig;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::runtime::Builder;
use tokio::task::LocalSet;

/// Represents a runtime pipeline configuration that includes nodes with their respective configurations and instances.
///
/// Note: Having a Debug bound on `PData` allows us to use it in logging and debugging contexts,
/// which is useful for tracing the pipeline's execution and state.
pub struct RuntimePipeline<PData: Debug> {
    /// The pipeline configuration that defines the structure and behavior of the pipeline.
    config: PipelineConfig,
    /// A map node id to receiver runtime node.
    receivers: Vec<ReceiverWrapper<PData>>,
    /// A map node id to processor runtime node.
    processors: Vec<ProcessorWrapper<PData>>,
    /// A map node id to exporter runtime node.
    exporters: Vec<ExporterWrapper<PData>>,

    /// A precomputed map of all node IDs to their Node trait objects (? @@@) for efficient access
    /// Indexed by NodeIndex
    nodes: NodeDefs<PData, PipeNode>,
}

/// PipeNode contains runtime-specific info.
pub(crate) struct PipeNode {
    index: usize, // NodeIndex into the appropriate vector w/ offset precomputed
}

impl PipeNode {
    /// Construct a pipe node with index referring to one an entry in
    /// the appropriate RuntimePipeline Vec.
    pub(crate) fn new(index: usize) -> Self {
        Self { index }
    }
}

impl<PData: 'static + Debug + Clone> RuntimePipeline<PData> {
    /// Creates a new `RuntimePipeline` from the given pipeline configuration and nodes.
    #[must_use]
    pub(crate) fn new(
        config: PipelineConfig,
        receivers: Vec<ReceiverWrapper<PData>>,
        processors: Vec<ProcessorWrapper<PData>>,
        exporters: Vec<ExporterWrapper<PData>>,
        nodes: NodeDefs<PData, PipeNode>,
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
        for exporter in self.exporters {
            _ = control_senders.insert(exporter.node_id().index, exporter.control_sender());
            let pipeline_ctrl_msg_tx = pipeline_ctrl_msg_tx.clone();
            futures.push(
                local_tasks.spawn_local(async move { exporter.start(pipeline_ctrl_msg_tx).await }),
            );
        }
        for processor in self.processors {
            _ = control_senders.insert(processor.node_id().index, processor.control_sender());
            let pipeline_ctrl_msg_tx = pipeline_ctrl_msg_tx.clone();
            futures.push(
                local_tasks.spawn_local(async move { processor.start(pipeline_ctrl_msg_tx).await }),
            );
        }
        for receiver in self.receivers {
            _ = control_senders.insert(receiver.node_id().index, receiver.control_sender());
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
    pub fn get_node(&self, node_id: usize) -> Option<&dyn Node> {
        let ndef = self.nodes.get(node_id)?;

        match ndef.ntype {
            NodeType::Receiver => self.receivers.get(ndef.inner.index).map(|r| r as &dyn Node),
            NodeType::Processor => self
                .processors
                .get(ndef.inner.index)
                .map(|p| p as &dyn Node),
            NodeType::Exporter => self.exporters.get(ndef.inner.index).map(|e| e as &dyn Node),
        }
    }

    /// Gets a mutable NodeWithPDataSender reference (processors and receivers).
    #[must_use]
    pub fn get_mut_node_with_pdata_sender(
        &mut self,
        node_id: usize,
    ) -> Option<&mut dyn NodeWithPDataSender<PData>> {
        let ndef = self.nodes.get(node_id)?;

        match ndef.ntype {
            NodeType::Receiver => self
                .receivers
                .get_mut(ndef.inner.index)
                .map(|r| r as &mut dyn NodeWithPDataSender<PData>),
            NodeType::Processor => self
                .processors
                .get_mut(ndef.inner.index)
                .map(|p| p as &mut dyn NodeWithPDataSender<PData>),
            NodeType::Exporter => None,
        }
    }

    /// Gets a mutable NodeWithPDataReceiver reference (processors and exporters).
    #[must_use]
    pub fn get_mut_node_with_pdata_receiver(
        &mut self,
        node_id: usize,
    ) -> Option<&mut dyn NodeWithPDataReceiver<PData>> {
        let ndef = self.nodes.get(node_id)?;

        match ndef.ntype {
            NodeType::Receiver => None,
            NodeType::Processor => self
                .processors
                .get_mut(ndef.inner.index)
                .map(|p| p as &mut dyn NodeWithPDataReceiver<PData>),
            NodeType::Exporter => self
                .exporters
                .get_mut(ndef.inner.index)
                .map(|e| e as &mut dyn NodeWithPDataReceiver<PData>),
        }
    }

    /// Sends a node control message to the specified node.
    pub async fn send_node_control_message(
        &self,
        node_id: &NodeId,
        ctrl_msg: NodeControlMsg,
    ) -> Result<(), Error<PData>> {
        match self.nodes.get(node_id.index) {
            Some(ndef) => match ndef.ntype {
                NodeType::Receiver => {
                    self.receivers
                        .get(ndef.inner.index)
                        .expect("precomputed")
                        .send_control_msg(ctrl_msg)
                        .await
                }
                NodeType::Processor => {
                    self.processors
                        .get(ndef.inner.index)
                        .expect("precomputed")
                        .send_control_msg(ctrl_msg)
                        .await
                }
                NodeType::Exporter => {
                    self.exporters
                        .get(ndef.inner.index)
                        .expect("precomputed")
                        .send_control_msg(ctrl_msg)
                        .await
                }
            }
            .map_err(|e| Error::NodeControlMsgSendError {
                node: node_id.clone(),
                error: e,
            }),
            None => Err(Error::InternalError {
                message: format!("node {node_id:?}"),
            }),
        }
    }
}
