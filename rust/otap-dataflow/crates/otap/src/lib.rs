// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use otap_df_config::{node::NodeKind, pipeline::PipelineConfig};
use otap_df_engine::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    create_factory_registry,
    error::Error,
    runtime_config::{RuntimeNode, RuntimePipeline}
};

/// gRPC service implementation
pub mod grpc;
/// Implementation of OTAP Exporter that implements the exporter trait
pub mod otap_exporter;
/// Implementation of OTAP Receiver that implements the receiver trait
pub mod otap_receiver;
/// Generated protobuf files
pub mod proto;

pub mod parquet_exporter;

/// testing utilities
#[cfg(test)]
mod mock;

// Create the factory registry with distributed slices for OTAP data
create_factory_registry!(OTAPData, OtapFactoryRegistry);

/// Creates a runtime pipeline from the given pipeline configuration.
pub fn create_runtime_pipeline(
    config: PipelineConfig,
) -> Result<RuntimePipeline<OTAPData>, Error<OTAPData>> {
    let receiver_factory_map = OtapFactoryRegistry::get_receiver_factory_map();
    let processor_factory_map = OtapFactoryRegistry::get_processor_factory_map();
    let exporter_factory_map = OtapFactoryRegistry::get_exporter_factory_map();
    let mut nodes = vec![];

    for (node_id, node_config) in config.node_iter() {
        match node_config.kind {
            NodeKind::Receiver => {
                let factory = receiver_factory_map
                    .get(node_config.plugin_urn.as_ref())
                    .ok_or_else(|| Error::UnknownReceiver{plugin_urn: node_config.plugin_urn.clone()})?;
                let receiver_config = ReceiverConfig::new(node_id.clone());
                let create = factory.create;
                nodes.push(RuntimeNode::Receiver {
                    config: node_config.clone(),
                    instance: create(&node_config.config, &receiver_config),
                });
            }
            NodeKind::Processor => {
                let factory = processor_factory_map
                    .get(node_config.plugin_urn.as_ref())
                    .ok_or_else(|| Error::UnknownProcessor{plugin_urn: node_config.plugin_urn.clone()})?;
                let processor_config = ProcessorConfig::new(node_id.clone());
                let create = factory.create;
                nodes.push(RuntimeNode::Processor {
                    config: node_config.clone(),
                    instance: create(&node_config.config, &processor_config),
                });
            }
            NodeKind::Exporter => {
                let factory = exporter_factory_map
                    .get(node_config.plugin_urn.as_ref())
                    .ok_or_else(|| Error::UnknownExporter{plugin_urn: node_config.plugin_urn.clone()})?;
                let exporter_config = ExporterConfig::new(node_id.clone());
                let create = factory.create;
                nodes.push(RuntimeNode::Exporter {
                    config: node_config.clone(),
                    instance: create(&node_config.config, &exporter_config),
                });
            }
            NodeKind::ProcessorChain => {
                return Err(Error::UnsupportedNodeKind {
                    kind: "ProcessorChain".into(),
                });
            }
        }
    }

    Ok(RuntimePipeline::new(config, nodes))
}
