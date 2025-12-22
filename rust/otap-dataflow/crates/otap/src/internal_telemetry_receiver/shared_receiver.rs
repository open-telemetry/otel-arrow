// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared (Send + Sync) implementation of the internal telemetry receiver.
//!
//! This receiver operates in multithreaded mode, receiving OTLP-encoded bytes
//! from the OtlpTracingLayer and routing them through the dataflow pipeline.

use crate::pdata::{Context, OtapPdata};
use async_trait::async_trait;
use bytes::Bytes;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::shared::receiver as shared;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::{OtlpProtoBytes, OtapPayload};
use otap_df_telemetry::otel_info;
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

/// Configuration for the internal telemetry receiver
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Flush interval for sending batched telemetry (default: 1s)
    #[serde(default = "default_flush_interval")]
    #[serde(with = "humantime_serde")]
    pub flush_interval: Duration,
    
    /// Maximum number of OTLP byte buffers to queue before dropping (default: 1000)
    #[serde(default = "default_max_buffer_size")]
    pub max_buffer_size: usize,
    
    /// Channel buffer size for receiving OTLP bytes (default: 100)
    #[serde(default = "default_channel_buffer")]
    pub channel_buffer: usize,
}

fn default_flush_interval() -> Duration {
    Duration::from_secs(1)
}

fn default_max_buffer_size() -> usize {
    1000
}

fn default_channel_buffer() -> usize {
    100
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flush_interval: default_flush_interval(),
            max_buffer_size: default_max_buffer_size(),
            channel_buffer: default_channel_buffer(),
        }
    }
}

/// Global channel for sending OTLP bytes from OtlpTracingLayer to receiver
static OTLP_BYTES_SENDER: std::sync::OnceLock<mpsc::UnboundedSender<Vec<u8>>> = std::sync::OnceLock::new();

/// Get the global OTLP bytes sender for the tracing layer
pub fn get_otlp_bytes_sender() -> Option<&'static mpsc::UnboundedSender<Vec<u8>>> {
    OTLP_BYTES_SENDER.get()
}

/// Initialize the global OTLP bytes sender (called by receiver on startup)
fn init_otlp_bytes_sender(sender: mpsc::UnboundedSender<Vec<u8>>) {
    let _ = OTLP_BYTES_SENDER.set(sender);
}

/// Internal telemetry receiver that collects self-diagnostic telemetry
pub struct InternalTelemetryReceiver {
    config: Config,
}

impl InternalTelemetryReceiver {
    /// Create a new internal telemetry receiver
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl shared::Receiver<OtapPdata> for InternalTelemetryReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel<OtapPdata>,
        effect_handler: shared::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!("InternalTelemetryReceiver.Start", 
                   message = "Starting internal telemetry receiver");
        
        // Create channel for receiving OTLP bytes from tracing layer
        let (otlp_tx, mut otlp_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        
        // Initialize global sender for OtlpTracingLayer to use
        init_otlp_bytes_sender(otlp_tx);
        
        otel_info!("InternalTelemetryReceiver.Ready",
                   message = "Ready to receive OTLP bytes from tracing layer");
        
        // Set up periodic flush timer
        let mut flush_timer = interval(self.config.flush_interval);
        flush_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        // Buffer for collecting OTLP byte buffers between flushes
        let mut otlp_buffer = Vec::with_capacity(self.config.max_buffer_size);
        
        loop {
            tokio::select! {
                // Handle control messages from the engine
                ctrl_result = ctrl_msg_recv.recv() => {
                    match ctrl_result {
                        Ok(NodeControlMsg::Shutdown { .. }) => {
                            otel_info!("InternalTelemetryReceiver.Shutdown",
                                       message = "Shutting down internal telemetry receiver",
                                       buffered_count = otlp_buffer.len());
                            
                            // Flush any remaining OTLP bytes
                            for otlp_bytes in otlp_buffer.drain(..) {
                                // Wrap as OtapPdata and send
                                let bytes = Bytes::from(otlp_bytes);
                                let proto_bytes = OtlpProtoBytes::ExportLogsRequest(bytes);
                                let payload: OtapPayload = proto_bytes.into();
                                let pdata = OtapPdata::new(Context::default(), payload);
                                let _ = effect_handler.send_message(pdata).await;
                            }
                            
                            return Ok(TerminalState::default());
                        }
                        Ok(_) => {
                            // Handle other control messages if needed
                        }
                        Err(_) => {
                            // Control channel closed
                            return Ok(TerminalState::default());
                        }
                    }
                }
                
                // Receive OTLP bytes from the tracing layer
                Some(otlp_bytes) = otlp_rx.recv() => {
                    otlp_buffer.push(otlp_bytes);
                    
                    // Drop oldest if buffer is full
                    if otlp_buffer.len() >= self.config.max_buffer_size {
                        otel_info!("InternalTelemetryReceiver.BufferFull",
                                   message = "Dropping oldest OTLP bytes, buffer full",
                                   max_size = self.config.max_buffer_size);
                        // Keep only the most recent half
                        let _ = otlp_buffer.drain(0..otlp_buffer.len() / 2).count();
                    }
                }
                
                // Periodic flush
                _ = flush_timer.tick() => {
                    if !otlp_buffer.is_empty() {
                        let count = otlp_buffer.len();
                        
                        // Send all buffered OTLP bytes as OtapPdata
                        for otlp_bytes in otlp_buffer.drain(..) {
                            let bytes = Bytes::from(otlp_bytes);
                            let proto_bytes = OtlpProtoBytes::ExportLogsRequest(bytes);
                            let payload: OtapPayload = proto_bytes.into();
                            let pdata = OtapPdata::new(Context::default(), payload);
                            if let Err(e) = effect_handler.send_message(pdata).await {
                                otel_info!("InternalTelemetryReceiver.SendFailed",
                                           message = "Failed to send telemetry",
                                           error = format!("{:?}", e));
                            }
                        }
                        
                        otel_info!("InternalTelemetryReceiver.Flushed",
                                   message = "Flushed OTLP telemetry",
                                   count = count);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.flush_interval, Duration::from_secs(1));
        assert_eq!(config.max_buffer_size, 1000);
        assert_eq!(config.channel_buffer, 100);
    }
}
