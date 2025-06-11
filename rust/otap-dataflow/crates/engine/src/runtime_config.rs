// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use otap_df_channel::error::SendError;
use otap_df_config::{NodeId, node::NodeConfig, pipeline::PipelineConfig};
use std::collections::HashMap;
use std::rc::Rc;

use crate::control::{ControlMsg, Controllable};
use crate::error::Error;
use crate::message::Sender;
use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};

/// Represents a runtime pipeline configuration that includes nodes with their respective configurations and instances.
pub struct RuntimePipeline<PData> {
    /// The pipeline configuration that defines the structure and behavior of the pipeline.
    config: PipelineConfig,
    /// A map of node id -> runtime node, where each node can be a receiver, processor, or exporter.
    /// This map allows for quick access to nodes by their unique identifiers.
    nodes: HashMap<NodeId, RuntimeNode<PData>>,
}

/// Represents a node in the runtime pipeline, which can be a receiver, processor, or exporter.
pub enum RuntimeNode<PData> {
    /// A node that acts as a receiver, receiving data from an external source.
    Receiver {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeConfig>,
        /// The instance of the receiver that processes incoming data.
        instance: ReceiverWrapper<PData>,
        /// Sender for control messages.
        control_sender: Option<ReceiverWrapper<PData>>,
        /// Receiver for control messages.
        control_receiver: Option<ReceiverWrapper<PData>>,
    },
    /// A node that processes data, transforming or analyzing it.
    Processor {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeConfig>,
        /// The instance of the processor that performs operations on the data.
        instance: ProcessorWrapper<PData>,
        /// Sender for control messages.
        control_sender: Option<ReceiverWrapper<PData>>,
        /// Receiver for control messages.
        control_receiver: Option<ReceiverWrapper<PData>>,
        /// Sender for PData messages.
        pdata_sender: Option<ReceiverWrapper<PData>>,
        /// Receiver for PData messages.
        pdata_receiver: Option<ReceiverWrapper<PData>>,
    },
    /// A node that exports data to an external destination.
    Exporter {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeConfig>,
        /// The instance of the exporter that sends data to an external system.
        instance: ExporterWrapper<PData>,
        /// Sender for control messages.
        control_sender: Option<ReceiverWrapper<PData>>,
        /// Receiver for control messages.
        control_receiver: Option<ReceiverWrapper<PData>>,
        /// Sender for PData messages.
        pdata_sender: Option<ReceiverWrapper<PData>>,
        /// Receiver for PData messages.
        pdata_receiver: Option<ReceiverWrapper<PData>>,
    },
}

#[async_trait::async_trait(?Send)]
impl<PData> Controllable for RuntimeNode<PData> {
    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: ControlMsg) -> Result<(), SendError<ControlMsg>> {
        match self {
            RuntimeNode::Receiver { instance, .. } => instance.send_control_msg(msg).await,
            RuntimeNode::Processor { .. } => {
                unimplemented!("Processor control message handling is not implemented yet");
            }
            RuntimeNode::Exporter { .. } => {
                unimplemented!("Exporter control message handling is not implemented yet");
            }
        }
    }

    /// Returns the control message sender for the node.
    fn control_sender(&self) -> Sender<ControlMsg> {
        match self {
            RuntimeNode::Receiver { instance, .. } => instance.control_sender(),
            RuntimeNode::Processor { .. } => {
                unimplemented!("Processor control message handling is not implemented yet");
            }
            RuntimeNode::Exporter { .. } => {
                unimplemented!("Exporter control message handling is not implemented yet");
            }
        }
    }
}

impl<PData> RuntimeNode<PData> {
    /// Flag indicating whether the node is shared (true) or local (false).
    #[must_use]
    pub fn is_shared(&self) -> bool {
        matches!(
            self,
            RuntimeNode::Receiver {
                instance: ReceiverWrapper::Shared { .. },
                ..
            } | RuntimeNode::Processor {
                instance: ProcessorWrapper::Shared { .. },
                ..
            } | RuntimeNode::Exporter {
                instance: ExporterWrapper::Shared { .. },
                ..
            }
        )
    }
}

// ToDo create 2 versions of this function into otlp and otap crates.
impl<PData> RuntimePipeline<PData> {
    /// Creates a new `RuntimePipeline` from the given pipeline configuration and nodes.
    #[must_use]
    pub fn new(config: PipelineConfig, nodes: HashMap<NodeId, RuntimeNode<PData>>) -> Self {
        Self { config, nodes }
    }

    /// Returns the number of nodes in the pipeline.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns a reference to the pipeline configuration.
    #[must_use]
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Sends a control message to the specified node.
    pub async fn send_control_message(
        &self,
        node_id: NodeId,
        ctrl_msg: ControlMsg,
    ) -> Result<(), Error<PData>> {
        if let Some(node) = self.nodes.get(&node_id) {
            node.send_control_msg(ctrl_msg)
                .await
                .map_err(|e| Error::ControlMsgSendError {
                    node: node_id,
                    error: e,
                })
        } else {
            Err(Error::UnknownNode { node_id })
        }
    }
}
