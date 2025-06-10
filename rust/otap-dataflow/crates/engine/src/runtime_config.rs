// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use std::collections::HashMap;
use std::rc::Rc;

use otap_df_config::{node::NodeConfig, pipeline::PipelineConfig, NodeId};

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

// ToDo create 2 versions of this function into otlp and otap crates.
impl<PData> RuntimePipeline<PData> {
    /// Creates a new `RuntimePipeline` from the given pipeline configuration and nodes.
    #[must_use]
    pub fn new(config: PipelineConfig, nodes: HashMap<NodeId, RuntimeNode<PData>>) -> Self {
        Self { config, nodes }
    }
}
