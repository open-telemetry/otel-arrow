// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry receiver.
//!
//! This receiver drains the engine thread's internal log buffer and emits
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
use otap_df_telemetry::drain_thread_log_buffer;
use otap_df_telemetry::metrics::MetricSetSnapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::time::Duration;

/// The URN for the internal telemetry receiver.
pub const INTERNAL_TELEMETRY_RECEIVER_URN: &str = "urn:otel:otap:internal_telemetry:receiver";

/// Configuration for the internal telemetry receiver.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Interval in milliseconds between buffer drains.
    #[serde(default = "default_drain_interval_ms")]
    pub drain_interval_ms: u64,
}

fn default_drain_interval_ms() -> u64 {
    1000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            drain_interval_ms: default_drain_interval_ms(),
        }
    }
}

/// A receiver that drains the engine's internal log buffer and emits OTLP logs.
pub struct InternalTelemetryReceiver {
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
        let drain_interval = Duration::from_millis(self.config.drain_interval_ms);

        // Start periodic telemetry collection
        let _ = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        loop {
            tokio::select! {
                biased;

                // Handle control messages with priority
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            // Drain any remaining logs before shutdown
                            self.drain_and_send(&effect_handler).await?;
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

                // Periodic drain
                _ = tokio::time::sleep(drain_interval) => {
                    self.drain_and_send(&effect_handler).await?;
                }
            }
        }
    }
}

impl InternalTelemetryReceiver {
    /// Drain the thread-local log buffer and send as OTLP logs.
    async fn drain_and_send(&self, effect_handler: &local::EffectHandler<OtapPdata>) -> Result<(), Error> {
        if let Some(batch) = drain_thread_log_buffer() {
            if !batch.records.is_empty() {
                let bytes = batch.encode_export_logs_request();
                let pdata = OtapPdata::new_todo_context(
                    OtlpProtoBytes::ExportLogsRequest(bytes).into(),
                );
                effect_handler.send_message(pdata).await?;
            }
        }
        Ok(())
    }
}
