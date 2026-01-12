// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry receiver.
//!
//! This receiver consumes internal logs from the logging channel and emits
//! the logs as OTLP ExportLogsRequest messages into the pipeline.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::ReceiverFactory;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_telemetry::logs::{LogBatch, LogPayload};
use otap_df_telemetry::metrics::MetricSetSnapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// The URN for the internal telemetry receiver.
pub use otap_df_config::pipeline::service::telemetry::logs::INTERNAL_TELEMETRY_RECEIVER_URN;

/// Configuration for the internal telemetry receiver.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

/// A receiver that consumes internal logs from the logging channel and emits OTLP logs.
pub struct InternalTelemetryReceiver {
    #[allow(dead_code)]
    config: Config,
}

/// Declares the internal telemetry receiver as a local receiver factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static INTERNAL_TELEMETRY_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: INTERNAL_TELEMETRY_RECEIVER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            InternalTelemetryReceiver::from_config(&node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
};

impl InternalTelemetryReceiver {
    /// Create a new receiver with the given configuration.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Create a receiver from a JSON configuration.
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?;
        Ok(Self::new(config))
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for InternalTelemetryReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // Get the logs receiver channel from the effect handler
        let logs_receiver = effect_handler
            .logs_receiver()
            .expect("InternalTelemetryReceiver requires a logs_receiver to be configured");

        // Start periodic telemetry collection
        let _ = effect_handler
            .start_periodic_telemetry(std::time::Duration::from_secs(1))
            .await?;

        loop {
            tokio::select! {
                biased;

                // Handle control messages with priority
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            // Drain any remaining logs from channel before shutdown
                            while let Ok(payload) = logs_receiver.try_recv() {
                                self.send_payload(&effect_handler, payload).await?;
                            }
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::CollectTelemetry { .. }) => {
                            // No metrics to report for now
                        }
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // Ignore other control messages
                        }
                    }
                }

                // Receive logs from the channel
                result = logs_receiver.recv_async() => {
                    match result {
                        Ok(payload) => {
                            self.send_payload(&effect_handler, payload).await?;
                        }
                        Err(_) => {
                            // Channel closed, exit gracefully
                            return Ok(TerminalState::default());
                        }
                    }
                }
            }
        }
    }
}

impl InternalTelemetryReceiver {
    /// Send a log payload as OTLP logs.
    async fn send_payload(
        &self,
        effect_handler: &local::EffectHandler<OtapPdata>,
        payload: LogPayload,
    ) -> Result<(), Error> {
        let batch = match payload {
            LogPayload::Singleton(record) => LogBatch {
                records: vec![record],
                dropped_count: 0,
            },
            LogPayload::Batch(batch) => batch,
        };

        if !batch.records.is_empty() {
            let bytes = batch.encode_export_logs_request();
            let pdata = OtapPdata::new_todo_context(OtlpProtoBytes::ExportLogsRequest(bytes).into());
            effect_handler.send_message(pdata).await?;
        }
        Ok(())
    }
}
