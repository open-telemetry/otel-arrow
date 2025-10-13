// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use crate::control::{
    ControlSenders, Controllable, NodeControlMsg, PipelineCtrlMsgReceiver, PipelineCtrlMsgSender,
};
use crate::error::{Error, TypedError};
use crate::node::{Node, NodeDefs, NodeId, NodeType, NodeWithPDataReceiver, NodeWithPDataSender};
use crate::pipeline_ctrl::PipelineCtrlMsgManager;
use crate::terminal_state::TerminalState;
use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};
use otap_df_config::pipeline::PipelineConfig;
use otap_df_telemetry::reporter::MetricsReporter;

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

fn report_terminal_metrics(metrics_reporter: &MetricsReporter, terminal_state: TerminalState) {
    for snapshot in terminal_state.into_metrics() {
        let _ = metrics_reporter.try_report_snapshot(snapshot);
    }
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
    pub fn run_forever(
        self,
        metrics_reporter: MetricsReporter,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<PData>,
        pipeline_ctrl_msg_rx: PipelineCtrlMsgReceiver<PData>,
    ) -> Result<Vec<()>, Error> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        let local_tasks = LocalSet::new();
        // ToDo create an optimized version of FuturesUnordered that can be used for !Send, !Sync tasks
        let mut futures = FuturesUnordered::new();
        let mut control_senders = ControlSenders::default();

        // Create a task for each node type and pass the pipeline ctrl msg channel to each node, so
        // they can communicate with the runtime pipeline.
        for exporter in self.exporters {
            control_senders.register(
                exporter.node_id(),
                NodeType::Exporter,
                exporter.control_sender(),
            );
            let pipeline_ctrl_msg_tx = pipeline_ctrl_msg_tx.clone();
            let effect_metrics_reporter = metrics_reporter.clone();
            let final_metrics_reporter = metrics_reporter.clone();
            futures.push(local_tasks.spawn_local(async move {
                exporter
                    .start(pipeline_ctrl_msg_tx, effect_metrics_reporter)
                    .await
                    .map(|terminal_state| {
                        report_terminal_metrics(&final_metrics_reporter, terminal_state);
                    })
            }));
        }
        for processor in self.processors {
            control_senders.register(
                processor.node_id(),
                NodeType::Processor,
                processor.control_sender(),
            );
            let pipeline_ctrl_msg_tx = pipeline_ctrl_msg_tx.clone();
            let metrics_reporter = metrics_reporter.clone();
            futures.push(local_tasks.spawn_local(async move {
                processor
                    .start(pipeline_ctrl_msg_tx, metrics_reporter)
                    .await
            }));
        }
        for receiver in self.receivers {
            control_senders.register(
                receiver.node_id(),
                NodeType::Receiver,
                receiver.control_sender(),
            );
            let pipeline_ctrl_msg_tx = pipeline_ctrl_msg_tx.clone();
            let effect_metrics_reporter = metrics_reporter.clone();
            let final_metrics_reporter = metrics_reporter.clone();
            futures.push(local_tasks.spawn_local(async move {
                receiver
                    .start(pipeline_ctrl_msg_tx, effect_metrics_reporter)
                    .await
                    .map(|terminal_state| {
                        report_terminal_metrics(&final_metrics_reporter, terminal_state);
                    })
            }));
        }

        // Create a task to process pipeline control messages, i.e. messages sent from nodes to
        // the pipeline engine.
        futures.push(local_tasks.spawn_local(async move {
            let manager = PipelineCtrlMsgManager::new(
                pipeline_ctrl_msg_rx,
                control_senders,
                metrics_reporter,
            );
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
    pub fn get_node(&self, node_id: usize) -> Option<&dyn Node<PData>> {
        let ndef = self.nodes.get(node_id)?;

        match ndef.ntype {
            NodeType::Receiver => self
                .receivers
                .get(ndef.inner.index)
                .map(|r| r as &dyn Node<PData>),
            NodeType::Processor => self
                .processors
                .get(ndef.inner.index)
                .map(|p| p as &dyn Node<PData>),
            NodeType::Exporter => self
                .exporters
                .get(ndef.inner.index)
                .map(|e| e as &dyn Node<PData>),
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
        ctrl_msg: NodeControlMsg<PData>,
    ) -> Result<(), TypedError<NodeControlMsg<PData>>> {
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
            .map_err(|e| TypedError::NodeControlMsgSendError {
                node_id: node_id.index,
                error: e,
            }),
            None => Err(TypedError::Error(Error::InternalError {
                message: format!("node {node_id:?}"),
            })),
        }
    }
}
