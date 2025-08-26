// // Copyright The OpenTelemetry Authors
// // SPDX-License-Identifier: Apache-2.0

// use crate::OTAP_EXPORTER_FACTORIES;
// use async_trait::async_trait;
// use linkme::distributed_slice;
// use otap_df_config::node::NodeUserConfig;
// use otap_df_engine::ExporterFactory;
// use otap_df_engine::config::ExporterConfig;
// use otap_df_engine::control::NodeControlMsg;
// use otap_df_engine::error::Error;
// use otap_df_engine::exporter::ExporterWrapper;
// use otap_df_engine::local::exporter::{EffectHandler, Exporter};
// use otap_df_engine::message::{Message, MessageChannel};
// use otap_df_engine::node::NodeId;
// use otap_df_otlp::compression::CompressionMethod;
// use serde::Deserialize;
// use std::sync::Arc;

// /// The URN for the OTLP exporter
// pub const OTLP_EXPORTER_URN: &str = "urn:otel:otlp:exporter";

// /// Configuration for the OTLP Exporter
// #[derive(Debug, Deserialize)]
// pub struct Config {
//     /// The gRPC endpoint to connect to
//     pub grpc_endpoint: String,
//     /// The compression method to use for the gRPC connection
//     pub compression_method: Option<CompressionMethod>,
// }

// /// Exporter that sends OTLP data via gRPC
// pub struct OTLPExporter {
//     config: Config,
// }

// /// Declare the OTLP Exporter as a local exporter factory
// #[allow(unsafe_code)]
// #[distributed_slice(OTAP_EXPORTER_FACTORIES)]
// pub static OTLP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
//     name: OTLP_EXPORTER_URN,
//     create: |node: NodeId, node_config: Arc<NodeUserConfig>, exporter_config: &ExporterConfig| {
//         Ok(ExporterWrapper::local(
//             OTLPExporter::from_config(&node_config.config)?,
//             node,
//             node_config,
//             exporter_config,
//         ))
//     },
// };

// impl OTLPExporter {
//     /// create a new instance of the `[OTLPExporter]` from json config value
//     pub fn from_config(config: &serde_json::Value) -> Result<Self, otap_df_config::error::Error> {
//         let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
//             otap_df_config::error::Error::InvalidUserConfig {
//                 error: e.to_string(),
//             }
//         })?;

//         Ok(Self { config })
//     }
// }

// #[async_trait(?Send)]
// impl Exporter<OtapPdata> for OTLPExporter {
//     async fn start(
//         self: Box<Self>,
//         mut msg_chan: MessageChannel<OtapPdata>,
//         effect_handler: EffectHandler<OtapPdata>,
//     ) -> Result<(), Error<OtapPdata>> {

//         loop {
//             match msg_chan.recv().await? {
//                 Message::Control(NodeControlMsg::Shutdown { .. }) => break,
//                 Message::PData(data) => {
//                 }
//                 _ => {
//                     // ignore unhandled messages
//                 }
//             }
//         }

//         Ok(())
//     }
// }
