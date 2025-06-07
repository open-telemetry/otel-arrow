// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

use std::vec;

use crate::grpc::OTLPData;
use linkme::distributed_slice;
use otap_df_config::{node::NodeKind, pipeline::PipelineConfig};
use otap_df_engine::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig}, error::Error, runtime_config::{RuntimeNode, RuntimePipeline}, ExporterFactory, FactoryRegistry, ProcessorFactory, ReceiverFactory
};

/// compression formats
pub mod compression;
/// gRPC service implementation
pub mod grpc;
/// otlp exporter implementation
pub mod otlp_exporter;
/// Implementation of OTLP Receiver that implements the receiver trait
pub mod otlp_receiver;
/// Generated protobuf files
pub mod proto;

/// grpc mock server for testing
#[cfg(test)]
mod mock;

/// A slice of receiver factories for OTAP data.
#[distributed_slice]
pub static RECEIVER_FACTORIES: [ReceiverFactory<OTLPData>] = [..];

/// A slice of local processor factories for OTAP data.
#[distributed_slice]
pub static PROCESSOR_FACTORIES: [ProcessorFactory<OTLPData>] = [..];

/// A slice of local exporter factories for OTAP data.
#[distributed_slice]
pub static EXPORTER_FACTORIES: [ExporterFactory<OTLPData>] = [..];

/// Global factory registry for OTLP data.
static FACTORY_REGISTRY: FactoryRegistry<OTLPData> = FactoryRegistry::new();

/// Creates a runtime pipeline from the given pipeline configuration.
pub fn create_runtime_pipeline(
    config: PipelineConfig,
) -> Result<RuntimePipeline<OTLPData>, Error<OTLPData>> {
    let receiver_factory_map = FACTORY_REGISTRY.get_receiver_factory_map(&RECEIVER_FACTORIES);
    let processor_factory_map = FACTORY_REGISTRY.get_processor_factory_map(&PROCESSOR_FACTORIES);
    let exporter_factory_map = FACTORY_REGISTRY.get_exporter_factory_map(&EXPORTER_FACTORIES);
    let mut nodes = vec![]; // ToDo(LQ): initialize with the correct size

    // ToDo(LQ): Generate all the errors.
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
                instance: create(
                    &node_config.config,
                    &receiver_config,
                )
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
                    instance: create(
                        &node_config.config,
                        &processor_config,
                    )
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
                    instance: create(
                        &node_config.config,
                        &exporter_config,
                    )
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

#[cfg(test)]
mod tests {
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};

    use crate::create_runtime_pipeline;

    #[test]
    fn test_plugins() {
        for receiver in crate::RECEIVER_FACTORIES.iter() {
            println!("Receiver: {}", receiver.name);
        }

        for exporter in crate::EXPORTER_FACTORIES.iter() {
            println!("Exporter: {}", exporter.name);
        }
    }

    #[test]
    fn test_create_runtime_pipeline() {
        let config = PipelineConfigBuilder::new()
            .add_receiver("otlp_receiver", "urn:otel:otlp:receiver", None)
            .add_exporter("otlp_exporter1", "urn:otel:otlp:exporter", None)
            .add_exporter("otlp_exporter2", "urn:otel:otlp:exporter", None)
            // ToDo(LQ): Check the validity of the outport.
            .broadcast("otlp_receiver", "out_port", ["otlp_exporter1", "otlp_exporter2"])
            .build(PipelineType::OTLP, "namespace", "pipeline")
            .expect("Failed to build pipeline config");
        let result = create_runtime_pipeline(config);
        assert!(result.is_ok(), "Failed to create runtime pipeline: {:?}", result.err());
    }
}
