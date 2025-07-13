// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use crate::control::ControlMsg;
use crate::error::Error;
use crate::error::Error::EngineErrors;
use crate::message::{Receiver, Sender};
use crate::node::Node;
use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};
use otap_df_config::{NodeId, node::NodeUserConfig, pipeline::PipelineConfig};
use std::collections::HashMap;
use std::rc::Rc;
use tokio::runtime::Builder;
use tokio::task::LocalSet;

/// Represents a runtime pipeline configuration that includes nodes with their respective configurations and instances.
pub struct RuntimePipeline<PData> {
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

/// Represents a node in the runtime pipeline, which can be a receiver, processor, or exporter.
pub enum RuntimeNode<PData> {
    /// A node that acts as a receiver, receiving data from an external source.
    Receiver {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeUserConfig>,
        /// The instance of the receiver that processes incoming data.
        instance: ReceiverWrapper<PData>,
        /// Sender for PData messages.
        /// ToDo(LQ): Support multiple ports
        pdata_sender: Option<Sender<PData>>,
    },
    /// A node that processes data, transforming or analyzing it.
    Processor {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeUserConfig>,
        /// The instance of the processor that performs operations on the data.
        instance: ProcessorWrapper<PData>,
        /// Sender for PData messages.
        /// ToDo(LQ): Support multiple ports
        pdata_sender: Option<Sender<PData>>,
        /// Receiver for PData messages.
        pdata_receiver: Option<Receiver<PData>>,
    },
    /// A node that exports data to an external destination.
    Exporter {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeUserConfig>,
        /// The instance of the exporter that sends data to an external system.
        instance: ExporterWrapper<PData>,
        /// Receiver for PData messages.
        pdata_receiver: Option<Receiver<PData>>,
    },
}

// ToDo create 2 versions of this function into otlp and otap crates.
impl<PData: 'static> RuntimePipeline<PData> {
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

    /// Starts the pipeline by
    pub fn start(self) -> Result<(), Error<PData>> {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        let local_tasks = LocalSet::new();
        let mut handlers = Vec::with_capacity(self.node_count());

        for (_node_id, exporter) in self.exporters {
            handlers.push(local_tasks.spawn_local(async move { exporter.start().await }));
        }
        for (_node_id, processor) in self.processors {
            handlers.push(local_tasks.spawn_local(async move { processor.start().await }));
        }
        for (_node_id, receiver) in self.receivers {
            handlers.push(local_tasks.spawn_local(async move { receiver.start().await }));
        }

        // Wait for all tasks to complete, gathering any errors
        let results = rt.block_on(async {
            local_tasks
                .run_until(async { futures::future::join_all(handlers).await })
                .await
        });

        let mut errors = Vec::new();
        for result in results {
            match result {
                Ok(Ok(())) => continue,       // Task completed successfully
                Ok(Err(e)) => errors.push(e), // Task completed with an error
                Err(e) => errors.push(Error::TaskError {
                    // Task panicked
                    is_cancelled: e.is_cancelled(),
                    is_panic: e.is_panic(),
                    error: e.to_string(),
                }),
            }
        }
        if !errors.is_empty() {
            return Err(EngineErrors { errors });
        }
        Ok(())
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
