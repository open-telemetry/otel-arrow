// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! An internal telemetry receiver.
//! This receiver is used to receive internal telemetry data from the OpenTelemetry SDK
//! and forward it to the pipeline engine for processing.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use opentelemetry_proto::tonic::logs::v1::LogsData;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::error::{Error, ReceiverErrorKind};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use serde_json::Value;
use std::sync::Arc;

/// The URN for the internal telemetry receiver
pub const INTERNAL_TELEMETRY_RECEIVER_URN: &str = "urn:otel:otap:internal_telemetry:receiver";

/// A Receiver that receives internal telemetry data.
pub struct InternalTelemetryReceiver {
    /// Configuration for the internal telemetry receiver
    #[allow(dead_code)]
    config: Config,

    /// The channel to receive internal telemetry data from the OpenTelemetry SDK.
    internal_telemetry_receiver: crossbeam_channel::Receiver<LogsData>,

    /// The node id of this receiver
    node_id: NodeId,
}

/// Declares the internal telemetry receiver as a local receiver factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static INTERNAL_TELEMETRY_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: INTERNAL_TELEMETRY_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            InternalTelemetryReceiver::from_config(pipeline, &node_config.config, node.clone())?,
            node,
            node_config,
            receiver_config,
        ))
    },
};

impl InternalTelemetryReceiver {
    /// creates a new InternalTelemetryReceiver
    /// TODO: Fail if more than one instance is created, as the internal telemetry channel should
    /// have only one receiver instance (it can be multiple replicas).
    /// We can do this validation during configuration time.
    #[must_use]
    pub fn new(
        pipeline_ctx: PipelineContext,
        config: Config,
        node_id: NodeId,
    ) -> Result<Self, otap_df_config::error::Error> {
        let internal_telemetry_receiver = pipeline_ctx
            .internal_telemetry_receiver()
            .ok_or_else(|| otap_df_config::error::Error::InternalError {
                details: "Internal telemetry receiver channel not configured in pipeline context"
                    .to_string(),
            })?
            .clone();
        Ok(Self {
            config,
            internal_telemetry_receiver,
            node_id,
        })
    }

    /// Creates a new internal telemetry receiver from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
        node_id: NodeId,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        InternalTelemetryReceiver::new(pipeline_ctx, config, node_id)
    }
}

/// Implement the Receiver trait for the InternalTelemetryReceiver
#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for InternalTelemetryReceiver {
    async fn start(
        mut self: Box<Self>,
        mut _ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        _effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let receiver = &self.internal_telemetry_receiver;
        loop {
            tokio::select! {
                result = tokio::task::spawn_blocking({
                    let receiver = receiver.clone();
                    move || receiver.recv()
                }) => {
                    match result {
                        Ok(Ok(logs_data)) => {
                            let count: usize = logs_data
                                .resource_logs
                                .iter()
                                .flat_map(|rl| &rl.scope_logs)
                                .map(|sl| sl.log_records.len())
                                .sum();

                            println!("The InternalTelemetryReceiver received a logs data batch with {} log records. Node name: '{}'", count, self.node_id);
                            //TODO: Send the received logs data to the next consumers through the effect handler
                            // Make sure no new internal telemetry data is produced for this entire pipeline.
                            //effect_handler.send_data(pdata).await?;
                        }
                        Ok(Err(e)) => {
                            return Err(Error::ReceiverError {
                                receiver: self.node_id.clone(),
                                kind: ReceiverErrorKind::Connect,
                                error: "There was a problem receiving logs data".to_string(),
                                source_detail: e.to_string(),
                            });
                        }
                        Err(e) => {
                            return Err(Error::ReceiverError {
                                receiver: self.node_id.clone(),
                                kind: ReceiverErrorKind::Connect,
                                error: "Spawn blocking task failed".to_string(),
                                source_detail: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Configuration for the internal telemetry receiver
#[derive(serde::Deserialize)]
pub struct Config {}

#[cfg(test)]
mod tests {
    use otap_df_config::{PipelineGroupId, PipelineId};
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;

    use super::*;

    #[test]
    fn test_internal_telemetry_receiver_new_with_no_channel() {
        let metrics_handle = MetricsRegistryHandle::new();
        let controller_context = ControllerContext::new(metrics_handle);
        let pipeline_group_id: PipelineGroupId = "test_group".into();
        let pipeline_id: PipelineId = "test_pipeline".into();

        let pipeline_ctx =
            controller_context.pipeline_context_with(pipeline_group_id, pipeline_id, 0, 0);
        let node_id = NodeId {
            name: "test_node".into(),
            index: 0,
        };
        let config = Config {};
        let result = InternalTelemetryReceiver::new(pipeline_ctx, config, node_id);
        if let Err(otap_df_config::error::Error::InternalError { details }) = &result {
            assert!(details.contains("channel"),);
        } else {
            panic!("Expected InternalError due to missing internal telemetry receiver channel");
        }
    }
}
