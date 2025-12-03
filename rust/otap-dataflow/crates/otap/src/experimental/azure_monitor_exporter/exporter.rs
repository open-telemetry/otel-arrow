// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::max;
use std::pin::Pin;

use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use futures::{Future, FutureExt, StreamExt};  // Add FutureExt here
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
    total_rows_sent: f64,
    total_batches_sent: f64,
    total_processing_duration: tokio::time::Duration,
    processing_started: bool,
    processing_start_time: tokio::time::Instant,
    
    // Concurrent send management
    in_flight_sends: FuturesUnordered<Pin<Box<dyn Future<Output = Result<f64, String>>>>>,
    max_in_flight: usize,
    pending_rows: f64,  // Rows in flight but not yet confirmed
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
            total_rows_sent: 0.0,
            total_batches_sent: 0.0,
            total_processing_duration: tokio::time::Duration::ZERO,
            processing_started: false,
            processing_start_time: tokio::time::Instant::now(),
            in_flight_sends: FuturesUnordered::new(),
            max_in_flight: 10,
            pending_rows: 0.0,
        })
    }

    async fn process_entry(&mut self, json_bytes: &[u8]) -> Result<(), String> {
        let now = tokio::time::Instant::now();

        match self.gzip_batcher.push(json_bytes) {
            gzip_batcher::PushResult::Ok => {
                // Nothing to flush
            }
            gzip_batcher::PushResult::Full(batch, row_count) => {
                self.last_send_started = tokio::time::Instant::now();

                // Wait if we've hit the concurrency limit
                while self.in_flight_sends.len() >= self.max_in_flight {
                    // Wait for at least one send to complete
                    if let Some(result) = self.in_flight_sends.next().await {
                        match result {
                            Ok(rows) => {
                                self.total_rows_sent += rows;
                                self.pending_rows -= rows;
                                self.total_batches_sent += 1.0;
                            }
                            Err(e) => {
                                // Log error but continue
                                println!("[AzureMonitorExporter] Send failed: {}", e);
                            }
                        }
                    }
                }

                // Queue the send
                self.queue_send(batch, row_count);

                let now = tokio::time::Instant::now();
                self.total_processing_duration += now.elapsed();

                println!("[AzureMonitorExporter] Total rows sent: {:.0}, Pending: {:.0}, In-flight: {}, Throughput: {:.2} rows/s",
                    self.total_rows_sent, 
                    self.pending_rows,
                    self.in_flight_sends.len(),
                    self.total_rows_sent / self.processing_start_time.elapsed().as_secs_f64());
            }
            gzip_batcher::PushResult::TooLarge => {
                // Log entry too large to send
                return Err("Log entry too large to send".to_string());
            }
        }
        
        self.total_processing_duration += now.elapsed();

        Ok(())
    }

    async fn flush_batcher(&mut self) -> Result<(), String> {
        let now = tokio::time::Instant::now();

        match self.gzip_batcher.flush() {
            gzip_batcher::FlushResult::Empty => {
                // Nothing to flush
            },
            gzip_batcher::FlushResult::Flush(batch, row_count) => {
                self.last_send_started = tokio::time::Instant::now();

                // Queue the send without waiting for concurrency
                self.queue_send(batch, row_count);
            }
        }

        self.total_processing_duration += now.elapsed();

        Ok(())
    }

    fn queue_send(&mut self, batch: Vec<u8>, row_count: f64) {
        let mut client = self.client.clone();
        self.pending_rows += row_count;
        
        let send_fut = Box::pin(async move {
            client
                .send(batch)
                .await
                .map(|_| row_count)
        });
        
        self.in_flight_sends.push(send_fut);
    }

    async fn drain_in_flight_sends(&mut self) {
        while let Some(result) = self.in_flight_sends.next().await {
            match result {
                Ok(rows) => {
                    self.total_rows_sent += rows;
                    self.pending_rows -= rows;
                    self.total_batches_sent += 1.0;
                }
                Err(e) => {
                    println!("[AzureMonitorExporter] Send failed during drain: {}", e);
                }
            }
        }
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

        if self.processing_started == false {
            self.processing_started = true;
            self.processing_start_time = tokio::time::Instant::now();
        }

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

                    let log_entries = self.transformer.convert_to_log_analytics(&request);

                    for json_bytes in log_entries {
                        self.process_entry(&json_bytes).await?;
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

                        let log_entries = self.transformer.convert_to_log_analytics(&request);

                        for json_bytes in log_entries {
                            self.process_entry(&json_bytes).await?;
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
        };

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
        let mut next_token_refresh = tokio::time::Instant::now();

        loop {
            // Use futures::select_biased to prioritize draining in-flight sends
            futures::select_biased! {
                // Priority 1: Drain completed sends
                result = self.in_flight_sends.select_next_some() => {
                    match result {
                        Ok(rows) => {
                            self.total_rows_sent += rows;
                            self.pending_rows -= rows;
                            self.total_batches_sent += 1.0;
                        }
                        Err(e) => {
                            effect_handler
                                .info(&format!("[AzureMonitorExporter] Send failed: {}", e))
                                .await;
                        }
                    }
                }

                // Priority 2: Token refresh
                _ = tokio::time::sleep_until(next_token_refresh).fuse() => {
                    // Token is expiring soon or has expired, refresh it
                    effect_handler
                        .info("[AzureMonitorExporter] Refreshing token")
                        .await;

                    self.client
                        .ensure_valid_token()
                        .await
                        .map_err(|e| Error::InternalError { message: format!("Failed to refresh token: {}", e) })?;

                    // token valid until is 5 minutes before expiry
                    // we schedule refresh even earlier to account for
                    // any possible delays
                    let refresh_target = self.client.token_valid_until - tokio::time::Duration::from_secs(300);
                    let min_refresh_time = tokio::time::Instant::now() + std::time::Duration::from_secs(30);

                    next_token_refresh = max(refresh_target, min_refresh_time);

                    // Convert Instant to SystemTime for display
                    let duration_until_refresh = next_token_refresh.saturating_duration_since(tokio::time::Instant::now());
                    let refresh_time = std::time::SystemTime::now() + duration_until_refresh;
                    let next_token_refresh_datetime: chrono::DateTime<chrono::Local> = refresh_time.into();
                    let current_date_time: chrono::DateTime<chrono::Local> = std::time::SystemTime::now().into();

                    effect_handler
                        .info(&format!(
                            "[AzureMonitorExporter] Next token refresh scheduled at {} with local time {}",
                            next_token_refresh_datetime.format("%Y-%m-%d %H:%M:%S"),
                            current_date_time.format("%Y-%m-%d %H:%M:%S")
                        ))
                        .await;
                }

                // Priority 3: Periodic flush
                _ = tokio::time::sleep_until(next_send).fuse() => {
                    if self.last_send_started + SEND_INTERVAL <= tokio::time::Instant::now() {
                        self.flush_batcher()
                            .await
                            .map_err(|e| Error::InternalError { message: format!("Failed to flush batcher: {}", e) })?;
                    }

                    next_send = max(self.last_send_started, tokio::time::Instant::now()) + SEND_INTERVAL;
                }

                // Priority 4: Handle incoming messages
                msg = msg_chan.recv().fuse() => {
                    match msg {
                        Ok(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                            effect_handler
                                .info("[AzureMonitorExporter] Shutting down")
                                .await;

                            // Flush any remaining data
                            self.flush_batcher()
                                .await
                                .map_err(|e| Error::InternalError { message: format!("Failed to flush batcher during shutdown: {}", e) })?;

                            // Wait for all in-flight sends to complete
                            effect_handler
                                .info(&format!("[AzureMonitorExporter] Waiting for {} in-flight sends to complete", self.in_flight_sends.len()))
                                .await;

                            self.drain_in_flight_sends().await;

                            effect_handler
                                .info(&format!(
                                    "[AzureMonitorExporter] Final stats - Sent: {:.0} rows in {:.0} batches, Throughput: {:.2} rows/s",
                                    self.total_rows_sent,
                                    self.total_batches_sent,
                                    self.total_rows_sent / self.processing_start_time.elapsed().as_secs_f64()
                                ))
                                .await;

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
