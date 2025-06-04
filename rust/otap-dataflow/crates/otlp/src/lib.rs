// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

use std::{collections::HashMap, sync::OnceLock, vec};

use crate::grpc::OTLPData;
use linkme::distributed_slice;
use otap_df_config::{node::NodeKind, pipeline::PipelineConfig};
use otap_df_engine::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig}, error::Error, runtime_config::{RuntimeNode, RuntimePipeline}, ExporterFactory, ProcessorFactory, ReceiverFactory
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

static RECEIVER_FACTORY_MAP: OnceLock<HashMap<&'static str, ReceiverFactory<OTLPData>>> =
    OnceLock::new();
static PROCESSOR_FACTORY_MAP: OnceLock<HashMap<&'static str, ProcessorFactory<OTLPData>>> =
    OnceLock::new();
static EXPORTER_FACTORY_MAP: OnceLock<HashMap<&'static str, ExporterFactory<OTLPData>>> =
    OnceLock::new();

fn get_receiver_factory_map() -> &'static HashMap<&'static str, ReceiverFactory<OTLPData>> {
    RECEIVER_FACTORY_MAP.get_or_init(|| {
        RECEIVER_FACTORIES
            .iter()
            .map(|f| (f.name, f.clone()))
            .collect()
    })
}

fn get_processor_factory_map() -> &'static HashMap<&'static str, ProcessorFactory<OTLPData>> {
    PROCESSOR_FACTORY_MAP.get_or_init(|| {
        PROCESSOR_FACTORIES
            .iter()
            .map(|f| (f.name, f.clone()))
            .collect()
    })
}

fn get_exporter_factory_map() -> &'static HashMap<&'static str, ExporterFactory<OTLPData>> {
    EXPORTER_FACTORY_MAP.get_or_init(|| {
        EXPORTER_FACTORIES
            .iter()
            .map(|f| (f.name, f.clone()))
            .collect()
    })
}

/// Creates a runtime pipeline from the given pipeline configuration.
pub fn create_runtime_pipeline(
    config: PipelineConfig,
) -> Result<RuntimePipeline<OTLPData>, Error<OTLPData>> {
    let receiver_factory_map = get_receiver_factory_map();
    let processor_factory_map = get_processor_factory_map();
    let exporter_factory_map = get_exporter_factory_map();
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
            .broadcast("otlp_receiver", "out_port", targets)
            .build(PipelineType::OTLP, "namespace", "pipeline")
            .expect("Failed to build pipeline config");
        let result = create_runtime_pipeline(config);
        assert!(result.is_ok(), "Failed to create runtime pipeline: {:?}", result.err());
    }
}
