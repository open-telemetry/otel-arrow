// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use std::rc::Rc;

use otap_df_config::{node::NodeConfig, pipeline::PipelineConfig};

use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};

/// Represents a runtime pipeline configuration that includes nodes with their respective configurations and instances.
pub struct RuntimePipeline<PData> {
    /// The pipeline configuration that defines the structure and behavior of the pipeline.
    config: PipelineConfig,
    /// A vector of runtime nodes, each containing a configuration and an instance of a receiver, processor, or exporter.
    nodes: Vec<RuntimeNode<PData>>,
}

/// Represents a node in the runtime pipeline, which can be a receiver, processor, or exporter.
pub enum RuntimeNode<PData> {
    /// A node that acts as a receiver, receiving data from an external source.
    Receiver {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeConfig>,
        /// The instance of the receiver that processes incoming data.
        instance: ReceiverWrapper<PData>,
    },
    /// A node that processes data, transforming or analyzing it.
    Processor {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeConfig>,
        /// The instance of the processor that performs operations on the data.
        instance: ProcessorWrapper<PData>,
    },
    /// A node that exports data to an external destination.
    Exporter {
        /// The configuration for the node, including its name and channel settings.
        config: Rc<NodeConfig>,
        /// The instance of the exporter that sends data to an external system.
        instance: ExporterWrapper<PData>,
    },
}

// ToDo create 2 versions of this function into otlp and otap crates.
impl<PData> RuntimePipeline<PData> {
    /// Creates a new `RuntimePipeline` from the given pipeline configuration and nodes.
    pub fn new(config: PipelineConfig, nodes: Vec<RuntimeNode<PData>>) -> Self {
        Self { config, nodes }
    }
}
