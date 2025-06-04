// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use linkme::distributed_slice;
use otap_df_engine::{ExporterFactory, ProcessorFactory, ReceiverFactory};
use std::{collections::HashMap, sync::OnceLock};
use otap_df_config::{node::NodeKind, pipeline::PipelineConfig};
use otap_df_engine::{
    config::{ExporterConfig, ProcessorConfig, ReceiverConfig},
    error::Error,
    runtime_config::{RuntimeNode, RuntimePipeline},
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

/// A slice of receiver factories for OTAP data.
#[distributed_slice]
pub static RECEIVER_FACTORIES: [ReceiverFactory<OTAPData>] = [..];

/// A slice of local processor factories for OTAP data.
#[distributed_slice]
pub static PROCESSOR_FACTORIES: [ProcessorFactory<OTAPData>] = [..];

/// A slice of local exporter factories for OTAP data.
#[distributed_slice]
pub static EXPORTER_FACTORIES: [ExporterFactory<OTAPData>] = [..];

static RECEIVER_FACTORY_MAP: OnceLock<HashMap<&'static str, ReceiverFactory<OTAPData>>> = OnceLock::new();
static PROCESSOR_FACTORY_MAP: OnceLock<HashMap<&'static str, ProcessorFactory<OTAPData>>> = OnceLock::new();
static EXPORTER_FACTORY_MAP: OnceLock<HashMap<&'static str, ExporterFactory<OTAPData>>> = OnceLock::new();

fn get_receiver_factory_map() -> &'static HashMap<&'static str, ReceiverFactory<OTAPData>> {
    RECEIVER_FACTORY_MAP.get_or_init(|| {
        RECEIVER_FACTORIES
            .iter()
            .map(|f| (f.name, f.clone()))
            .collect()
    })
}

fn get_processor_factory_map() -> &'static HashMap<&'static str, ProcessorFactory<OTAPData>> {
    PROCESSOR_FACTORY_MAP.get_or_init(|| {
        PROCESSOR_FACTORIES
            .iter()
            .map(|f| (f.name, f.clone()))
            .collect()
    })
}

fn get_exporter_factory_map() -> &'static HashMap<&'static str, ExporterFactory<OTAPData>> {
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
) -> Result<RuntimePipeline<OTAPData>, Error<OTAPData>> {
    let receiver_factory_map = get_receiver_factory_map();
    let processor_factory_map = get_processor_factory_map();
    let exporter_factory_map = get_exporter_factory_map();
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
