// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::max;

use async_trait::async_trait;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use prost::Message as _;

use crate::experimental::azure_monitor_exporter::client::LogsIngestionClient;
use crate::experimental::azure_monitor_exporter::config::Config;
use crate::experimental::azure_monitor_exporter::gzip_batcher::{self, GzipBatcher};
use crate::experimental::azure_monitor_exporter::transformer::Transformer;
use crate::pdata::OtapPdata;

/// Azure Monitor Exporter sending telemetry to Azure Monitor.
///
/// This exporter processes OTLP logs and sends them to Azure Monitor
/// using the Data Collection Rules (DCR) API.
pub struct AzureMonitorExporter {
    config: Config,
    client: LogsIngestionClient,
    transformer: Transformer,
    gzip_batcher: GzipBatcher,
    last_send_started: tokio::time::Instant,
}

impl AzureMonitorExporter {
    /// Build a new exporter from configuration.
    pub fn new(config: Config) -> Result<Self, otap_df_config::error::Error> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig { error: e })?;

        // Create Azure Monitor logs ingestion client with the full config
        let client = LogsIngestionClient::new(&config)
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig { error: e })?;

        // Create log transformer
        let transformer = Transformer::new(&config);

        // Create Gzip batcher
        let gzip_batcher = GzipBatcher::new();

        Ok(Self {
            config,
            client,
            transformer,
            gzip_batcher,
            last_send_started: tokio::time::Instant::now(),
        })
    }

    /// Handle a single pdata message.
    async fn handle_pdata(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), String> {
        // TODO: Ack/Nack handling
        // Split pdata into context and payload
        let (_context, payload) = pdata.into_parts();

        match payload {
            OtapPayload::OtapArrowRecords(otap_records) => match otap_records {
                OtapArrowRecords::Logs(otap_records) => {
                    effect_handler
                        .info("Converting OTAP logs to OTLP bytes (fallback path)")
                        .await;

                    let otlp_bytes: OtlpProtoBytes =
                        OtapPayload::OtapArrowRecords(OtapArrowRecords::Logs(otap_records))
                            .try_into()
                            .map_err(|e| format!("Failed to convert OTAP to OTLP: {:?}", e))?;

                    let OtlpProtoBytes::ExportLogsRequest(bytes) = otlp_bytes else {
                        return Err("Expected ExportLogsRequest bytes".to_string());
                    };

                    let request = ExportLogsServiceRequest::decode(&bytes[..])
                        .map_err(|e| format!("Failed to decode logs request: {}", e))?;

                    let log_entries_iter = self.transformer.convert_to_log_analytics(&request);

                    for json_bytes in log_entries_iter {
                        match self.gzip_batcher.push(&json_bytes) {
                            gzip_batcher::PushResult::Ok => {
                                // Nothing to flush
                            }
                            gzip_batcher::PushResult::Full(batch) => {
                                self.last_send_started = tokio::time::Instant::now();
                                self.client
                                    .send(batch)
                                    .await
                                    .map_err(|e| format!("Failed to send batch: {}", e))?;

                                // Yield to allow the spawned task to start processing
                                tokio::task::yield_now().await;
                            }
                            gzip_batcher::PushResult::TooLarge => {
                                // Log entry too large to send
                                effect_handler
                                .info(
                                    "[AzureMonitorExporter] Log entry too large to send; dropping",
                                )
                                .await;
                            }
                        }
                    }
                }

                OtapArrowRecords::Metrics(_) => {
                    // TODO: Use debug level when logging is integrated
                    effect_handler
                        .info("[AzureMonitorExporter] Metrics not supported; dropping payload")
                        .await;
                }

                OtapArrowRecords::Traces(_) => {
                    // TODO: Use debug level when logging is integrated
                    effect_handler
                        .info("[AzureMonitorExporter] Traces not supported; dropping payload")
                        .await;
                }
            },

            OtapPayload::OtlpBytes(otlp_bytes) => {
                match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(bytes) => {
                        let request = ExportLogsServiceRequest::decode(bytes.as_ref())
                            .map_err(|e| format!("Failed to decode OTLP logs request: {e}"))?;

                        // Use the transformer with config
                        let log_entries_iter = self.transformer.convert_to_log_analytics(&request);
                        tokio::task::yield_now().await;

                        for json_bytes in log_entries_iter {
                            match self.gzip_batcher.push(&json_bytes) {
                                gzip_batcher::PushResult::Ok => {
                                    // Nothing to flush
                                }
                                gzip_batcher::PushResult::Full(batch) => {
                                    self.last_send_started = tokio::time::Instant::now();
                                    self.client
                                        .send(batch)
                                        .await
                                        .map_err(|e| format!("Failed to send batch: {}", e))?;

                                    // Yield to allow the spawned task to start processing
                                    tokio::task::yield_now().await;
                                }
                                gzip_batcher::PushResult::TooLarge => {
                                    // Log entry too large to send
                                    effect_handler
                                        .info(
                                            "[AzureMonitorExporter] Log entry too large to send; dropping",
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                    OtlpProtoBytes::ExportMetricsRequest(_) => {
                        // TODO: Use debug level when logging is integrated
                        effect_handler
                            .info("[AzureMonitorExporter] Metrics not supported; dropping payload")
                            .await;
                    }
                    OtlpProtoBytes::ExportTracesRequest(_) => {
                        // TODO: Use debug level when logging is integrated
                        effect_handler
                            .info("[AzureMonitorExporter] Traces not supported; dropping payload")
                            .await;
                    }
                }
            }
        }

        Ok(())
    }
}

const SEND_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3);

#[async_trait(?Send)]
impl Exporter<OtapPdata> for AzureMonitorExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        effect_handler
            .info(&format!(
                "[AzureMonitorExporter] Starting: endpoint={}, stream={}, dcr={}",
                self.config.api.dcr_endpoint, self.config.api.stream_name, self.config.api.dcr
            ))
            .await;

        let mut next_send = tokio::time::Instant::now() + SEND_INTERVAL;

        loop {
            // 1. Always calculate the deadline based on the LAST time we tried to flush
            tokio::select! {
                _ = tokio::time::sleep_until(next_send) => {
                    if self.last_send_started + SEND_INTERVAL <= tokio::time::Instant::now() {
                        // 2. Attempt flush
                        match self.gzip_batcher.flush() {
                            gzip_batcher::FlushResult::Empty => {
                                // Nothing to flush
                            }
                            gzip_batcher::FlushResult::Flush(batch) => {
                                self.last_send_started = tokio::time::Instant::now();
                                self.client.send(batch)
                                    .await
                                    .map_err(|e| Error::InternalError { message: format!("Failed to send batch: {}", e) })?;

                                // Yield to allow spawned send tasks to run, especially in single-threaded runtimes
                                tokio::task::yield_now().await;
                            }
                        }
                    }
                    else {
                        // This can happen if the flush took longer than SEND_INTERVAL
                        println!("[AzureMonitorExporter] Last flush still recent");
                    }

                    next_send = max(self.last_send_started, tokio::time::Instant::now()) + SEND_INTERVAL;
                }

                // 2. Handle incoming messages
                msg = msg_chan.recv() => {
                    match msg {
                        Ok(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                            effect_handler
                                .info("[AzureMonitorExporter] Shutting down")
                                .await;

                            match self.gzip_batcher.flush() {
                                gzip_batcher::FlushResult::Empty => {}
                                gzip_batcher::FlushResult::Flush(batch) => {
                                    self.last_send_started = tokio::time::Instant::now();
                                    self.client.send(batch)
                                        .await
                                        .map_err(|e| Error::InternalError { message: format!("Failed to send batch: {}", e) })?;
                                }
                            }

                            return Ok(TerminalState::new(
                                deadline,
                                std::iter::empty::<otap_df_telemetry::metrics::MetricSetSnapshot>(),
                            ));
                        }
                        Ok(Message::PData(pdata)) => {
                            // Process data
                            if let Err(e) = self.handle_pdata(pdata, &effect_handler).await {
                                effect_handler
                                    .info(&format!(
                                        "[AzureMonitorExporter] Error processing data: {e}"
                                    ))
                                    .await;
                            }
                            // Yield to allow spawned send tasks to run, especially in single-threaded runtimes
                            tokio::task::yield_now().await;
                        }
                        Ok(_) => {
                            // Ignore other message types
                        }
                        Err(e) => {
                            // Channel error, likely closed
                            return Err(Error::InternalError { message: format!("Channel error: {e}") });
                        }
                    }
                }
            }
        }
    }
}
