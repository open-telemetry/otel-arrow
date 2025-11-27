// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otlp::OtlpProtoBytes;
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

    /// Spawns a background task to send a batch of data.
    fn spawn_send_batch(
        client: LogsIngestionClient,
        batch: Vec<u8>,
        effect_handler: EffectHandler<OtapPdata>,
    ) {
        _ = tokio::spawn(async move {
            if let Err(e) = client.send(batch).await {
                effect_handler
                    .info(&format!("[AzureMonitorExporter] Failed to send batch: {e}"))
                    .await;
            }
        });
    }

    /// Handle a single pdata message.
    async fn handle_pdata(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // TODO: Ack/Nack handling
        // Split pdata into context and payload
        let (_context, payload) = pdata.into_parts();

        // Convert OTAP payload to OTLP bytes
        // TODO: This conversion step should be eliminated
        let otlp_bytes: OtlpProtoBytes =
            payload
                .try_into()
                .map_err(|e| Error::PdataConversionError {
                    error: format!("Failed to convert OTAP to OTLP: {e:?}"),
                })?;

        match otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                let request = ExportLogsServiceRequest::decode(bytes.as_ref()).map_err(|e| {
                    Error::PDataError {
                        reason: format!("Failed to decode OTLP logs request: {e}"),
                    }
                })?;

                // Use the transformer with config
                let log_entries_iter = self.transformer.convert_to_log_analytics(&request);

                for json_bytes in log_entries_iter {
                    match self.gzip_batcher.push(&json_bytes) {
                        gzip_batcher::PushResult::Ok => {
                            // Successfully added to batch
                        }
                        gzip_batcher::PushResult::Full(batch) => {
                            self.last_send_started = tokio::time::Instant::now();
                            Self::spawn_send_batch(
                                self.client.clone(),
                                batch,
                                effect_handler.clone(),
                            );
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

        Ok(())
    }
}

const SEND_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

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

        // Initialize the flush deadline
        let mut next_send = self.last_send_started + SEND_INTERVAL;

        loop {
            tokio::select! {
                // 1. Handle flush timeout
                _ = tokio::time::sleep_until(next_send) => {
                    if self.last_send_started + SEND_INTERVAL <= tokio::time::Instant::now() {
                            match self.gzip_batcher.flush() {
                                gzip_batcher::FlushResult::Empty => {}
                                gzip_batcher::FlushResult::Flush(batch) => {
                                    Self::spawn_send_batch(
                                        self.client.clone(),
                                        batch,
                                        effect_handler.clone(),
                                    );
                                }
                            }
                    }

                    // Reset the timer
                    next_send = tokio::time::Instant::now() + SEND_INTERVAL;
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
                                    Self::spawn_send_batch(
                                        self.client.clone(),
                                        batch,
                                        effect_handler.clone(),
                                    );
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
