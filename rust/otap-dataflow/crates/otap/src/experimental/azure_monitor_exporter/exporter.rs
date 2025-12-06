// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::max;
use std::collections::HashMap;

use async_trait::async_trait;
use bytes::Bytes;
use futures::future::BoxFuture; // Add this import
use futures::stream::FuturesUnordered;
use futures::{Future, FutureExt, StreamExt};

// Add FutureExt here
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use prost::Message as _;

use crate::experimental::azure_monitor_exporter::client::LogsIngestionClient;
use crate::experimental::azure_monitor_exporter::config::Config;
use crate::experimental::azure_monitor_exporter::gzip_batcher::{GzipBatcher, PushResult, FlushResult};
use crate::experimental::azure_monitor_exporter::transformer::Transformer;
use crate::pdata::{Context, OtapPdata};

const MAX_IN_FLIGHT: usize = 10;
const DEFAULT_TOKEN_REFRESH_BACKOFF_SECONDS: u64 = 10;
const EXPORT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3);

/// Azure Monitor Exporter sending telemetry to Azure Monitor.
///
/// This exporter processes OTLP logs and sends them to Azure Monitor
/// using the Data Collection Rules (DCR) API.
pub struct AzureMonitorExporter {
    config: Config,
    transformer: Transformer,
    gzip_batcher: GzipBatcher,
    last_export_started: tokio::time::Instant,
    total_rows_exported: f64,
    total_batches_exported: f64,
    processing_start_time: tokio::time::Instant,
    processing_start_time_set: bool,
    max_in_flight: usize,
    msg_id: u64,
}

struct ClientPool {
    clients: Vec<LogsIngestionClient>,
}

impl ClientPool {
    fn from_client(client: LogsIngestionClient) -> Self {
        let mut clients = Vec::with_capacity(MAX_IN_FLIGHT);
        for _ in 0..MAX_IN_FLIGHT {
            clients.push(client.clone());
        }
        Self { clients }
    }

    #[inline(always)]
    fn take(&mut self) -> LogsIngestionClient {
        self.clients
            .pop()
            .expect("client pool is empty")
    }

    #[inline(always)]
    fn release(&mut self, client: LogsIngestionClient) {
        self.clients.push(client);
    }
}

struct CompletedExport {
    batch_id: u64,
    result: Result<(), String>,
    client: LogsIngestionClient,
    row_count: f64,  // Add row count to track throughput
}

struct InFlightExports {
    futures: FuturesUnordered<BoxFuture<'static, CompletedExport>>,
}

impl InFlightExports {
    fn new() -> Self {
        Self {
            futures: FuturesUnordered::new(),
        }
    }

    fn len(&self) -> usize {
        self.futures.len()
    }

    fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    fn push<F>(&mut self, future: F) 
    where F: Future<Output = CompletedExport> + Send + 'static 
    {
        self.futures.push(future.boxed());
    }

    /// Returns a future that resolves once the next export finishes.
    /// If there are no in-flight exports, this future will never resolve (Pending).
    fn next_completion(&mut self) -> impl Future<Output = CompletedExport> + '_ {
        if self.futures.is_empty() {
            futures::future::pending().left_future()
        } else {
            self.futures.next().map(|res| res.unwrap()).right_future()
        }
    }
}

impl AzureMonitorExporter {
    /// Build a new exporter from configuration.
    pub fn new(config: Config) -> Result<Self, otap_df_config::error::Error> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig { error: e })?;

        // Create log transformer
        let transformer = Transformer::new(&config);

        // Create Gzip batcher
        let gzip_batcher = GzipBatcher::new();

        Ok(Self {
            config,
            transformer,
            gzip_batcher,
            last_export_started: tokio::time::Instant::now(),
            total_rows_exported: 0.0,
            total_batches_exported: 0.0,
            processing_start_time: tokio::time::Instant::now(),
            processing_start_time_set: false,
            max_in_flight: 10,
            msg_id: 0,
        })
    }
}

fn make_export_future (
    mut client: LogsIngestionClient,
    batch: Bytes,
    batch_id: u64,
    row_count: f64,  // Add row count parameter
) -> impl Future<Output = CompletedExport> {
    async move {
        let result = client.send(batch).await;
        CompletedExport {
            batch_id,
            result,
            client,
            row_count,
        }
    }
}

fn get_next_token_refresh(token_valid_until: tokio::time::Instant) -> tokio::time::Instant {
    let token_lifetime = token_valid_until.saturating_duration_since(tokio::time::Instant::now());
    let token_expiry_buffer = tokio::time::Duration::from_secs(token_lifetime.as_secs() / 5);
    let next_token_refresh = token_valid_until - token_expiry_buffer;
    max(next_token_refresh, tokio::time::Instant::now() + std::time::Duration::from_secs(30))
}

async fn create_client_pool_with_retry(client: &mut LogsIngestionClient) -> Result<ClientPool, String> {
    let mut create_client_pool_backoff_seconds = std::time::Duration::from_secs(10);
    let create_client_pool_attempts = 3;

    for _ in 0..create_client_pool_attempts {
        match client.ensure_valid_token().await {
            Ok(()) => {
                return Ok(ClientPool::from_client(client.clone()));
            }
            Err(e) => {
                // Log the error and retry after a delay
                println!("[AzureMonitorExporter] Failed to refresh token: {}", e);
                tokio::time::sleep(create_client_pool_backoff_seconds).await;
                create_client_pool_backoff_seconds = create_client_pool_backoff_seconds * 2;
            }
        }
    }

    Err(format!("Failed to create client pool after {create_client_pool_attempts} attempts"))
}

async fn finalize_completed_export(
    msg_to_batch: &mut HashMap<u64, HashMap<u64, ()>>,
    batch_to_msg: &mut HashMap<u64, HashMap<u64, ()>>,
    msg_to_pdata: &mut HashMap<u64, Vec<(Context, Bytes)>>,
    completed: CompletedExport,
    effect_handler: &EffectHandler<OtapPdata>,
    total_rows_exported: f64, // Added parameter
    processing_start_time: tokio::time::Instant,
) -> Result<LogsIngestionClient, String> {
    match completed.result {
        Ok(()) => {
            let elapsed = processing_start_time.elapsed().as_secs_f64();
            let throughput = if elapsed > 0.0 {
                total_rows_exported / elapsed
            } else {
                0.0
            };

            effect_handler
                .info(&format!(
                    "[AzureMonitorExporter] Export succeeded - Batch {}: {} rows. Total: {:.0} rows in {:.2}s ({:.0} rows/s)",
                    completed.batch_id,
                    completed.row_count,
                    total_rows_exported,
                    elapsed,
                    throughput
                ))
                .await;

            if let Some(msg_ids) = batch_to_msg.remove(&completed.batch_id) {
                for (msg_id, _) in msg_ids {
                    if let Some(batch_ids) = msg_to_batch.get_mut(&msg_id) {
                        _ = batch_ids.remove(&completed.batch_id);

                        if batch_ids.is_empty() {
                            _ = msg_to_batch.remove(&msg_id);
                            if let Some(pdata_list) = msg_to_pdata.remove(&msg_id) {
                                for (context, bytes) in pdata_list {
                                    effect_handler.notify_ack(
                                        AckMsg::new(
                                            OtapPdata::new(context, OtlpProtoBytes::ExportLogsRequest(bytes).into())))
                                            .await
                                            .map_err(|e| format!("Failed to notify ack: {e}"))?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(ref e) => {
            effect_handler
                .info(&format!(
                    "[AzureMonitorExporter] Export failed - Batch {}: {} rows failed after {:.2}s - Error: {}",
                    completed.batch_id,
                    completed.row_count,
                    processing_start_time.elapsed().as_secs_f64(),
                    e
                ))
                .await;

            if let Some(msg_ids) = batch_to_msg.remove(&completed.batch_id) {
                for (msg_id, _) in msg_ids {
                    // Indiscriminate NACK:
                    // We remove the message from tracking entirely.
                    // This effectively "stops tracking the batch" for this message.
                    _ = msg_to_batch.remove(&msg_id);

                    if let Some(pdata_list) = msg_to_pdata.remove(&msg_id) {
                        for (context, bytes) in pdata_list {
                            effect_handler.notify_nack(
                                NackMsg::new(e, 
                                    OtapPdata::new(context, OtlpProtoBytes::ExportLogsRequest(bytes).into())))
                                    .await
                                    .map_err(|e| format!("Failed to notify nack: {e}"))?;
                        }
                    }
                }
            }
        }
    }

    Ok(completed.client)
}

async fn process_message(
    exporter: &mut AzureMonitorExporter,
    log_entries: Vec<Vec<u8>>,
    last_seen_batch_id: &mut Option<u64>,
    batch_to_msg: &mut HashMap<u64, HashMap<u64, ()>>,
    msg_to_batch: &mut HashMap<u64, HashMap<u64, ()>>,
    msg_to_pdata: &mut HashMap<u64, Vec<(Context, Bytes)>>,
    in_flight_exports: &mut InFlightExports,
    client_pool: &mut ClientPool,
    effect_handler: &EffectHandler<OtapPdata>,
) -> Result<(), Error> {
    for json_bytes in log_entries {
        match exporter.gzip_batcher.push(&json_bytes) {
            PushResult::Ok(batch_id) => {
                if *last_seen_batch_id != Some(batch_id) {
                    _ = batch_to_msg.entry(batch_id)
                    .or_default()
                    .insert(exporter.msg_id, ());

                    _ = msg_to_batch.entry(exporter.msg_id)
                        .or_default()
                        .insert(batch_id, ());

                    *last_seen_batch_id = Some(batch_id);
                }
            }
            PushResult::BatchReady => {                
                let Some(batch) = exporter.gzip_batcher.take_pending_batch() else {
                    return Err(Error::InternalError{ message: "Expected non-empty batch after ready".to_string() });
                };
                
                if *last_seen_batch_id != Some(batch.batch_id) {
                    _ = batch_to_msg.entry(batch.batch_id)
                        .or_default()
                        .insert(exporter.msg_id, ());

                    _ = msg_to_batch.entry(exporter.msg_id)
                        .or_default()
                        .insert(batch.batch_id, ());

                    *last_seen_batch_id = Some(batch.batch_id);
                }

                // Update stats when batch is ready to export
                exporter.last_export_started = tokio::time::Instant::now();
                exporter.total_rows_exported += batch.row_count;
                exporter.total_batches_exported += 1.0;

                // Wait if we've hit the concurrency limit
                while in_flight_exports.len() >= exporter.max_in_flight {
                    // Wait for at least one export to complete
                    let completed = in_flight_exports.next_completion().await;
                    // Handle the result of the completed export
                    match finalize_completed_export(msg_to_batch, batch_to_msg, msg_to_pdata, completed, effect_handler, exporter.total_rows_exported, exporter.processing_start_time).await {
                        Ok(client) => {
                            client_pool.release(client);
                        }
                        Err(e) => {
                            effect_handler
                                .info(&format!("[AzureMonitorExporter] Failed to finalize export: {}", e))
                                .await;
                        }
                    }
                }

                // Queue the export
                let client = client_pool.take();
                let export_fut = make_export_future(client, batch.compressed_data, batch.batch_id, batch.row_count);
                in_flight_exports.push(export_fut);
            }
            PushResult::TooLarge => {
                // Log entry too large to export
                return Err(Error::InternalError{ message: "Log entry too large to compress".to_string() });
            }
        }
    }
    Ok(())
}

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

        let mut next_export = tokio::time::Instant::now() + EXPORT_INTERVAL;

        let mut original_client = LogsIngestionClient::new(&self.config)
            .map_err(|e| Error::InternalError { message: format!("failed to create client: {e}") })?;

        let mut client_pool = create_client_pool_with_retry(&mut original_client)
            .await
            .map_err(|e| Error::InternalError { message: format!("failed to create client pool: {e}") })?;

        let mut next_token_refresh = get_next_token_refresh(original_client.token_valid_until);
        let mut token_refresh_backoff_seconds = DEFAULT_TOKEN_REFRESH_BACKOFF_SECONDS;

        // Create client pool by cloning the client
        let mut in_flight_exports = InFlightExports::new();

        let mut msg_to_batch: HashMap<u64, HashMap<u64, ()>> = HashMap::new();
        let mut batch_to_msg: HashMap<u64, HashMap<u64, ()>> = HashMap::new();
        let mut msg_to_pdata: HashMap<u64, Vec<(Context, Bytes)>> = HashMap::new();
        let mut last_seen_batch_id: Option<u64> = None;

        loop {
            // Priority 1: Token refresh
            futures::select_biased! {
                _ = tokio::time::sleep_until(next_token_refresh).fuse() => {
                    match original_client.ensure_valid_token().await {
                        Ok(()) => {
                            // Reset backoff after successful refresh
                            token_refresh_backoff_seconds = DEFAULT_TOKEN_REFRESH_BACKOFF_SECONDS;

                            // Update next token refresh time
                            let now = tokio::time::Instant::now();
                            let token_lifetime = original_client.token_valid_until.saturating_duration_since(now);
                            next_token_refresh = get_next_token_refresh(original_client.token_valid_until);
                            
                            effect_handler
                                .info(&format!(
                                    "[AzureMonitorExporter] Token refreshed. Valid for {} seconds, next refresh in {} seconds",
                                    token_lifetime.as_secs(),
                                    next_token_refresh.saturating_duration_since(now).as_secs()
                                ))
                                .await;
                        }
                        Err(e) => {
                            effect_handler
                                .info(&format!(
                                    "[AzureMonitorExporter] Token refresh failed: {}. Will retry in {} seconds",
                                    e,
                                    token_refresh_backoff_seconds
                                ))
                                .await;
                            
                            // Schedule retry with backoff
                            next_token_refresh = tokio::time::Instant::now() + 
                                std::time::Duration::from_secs(token_refresh_backoff_seconds);
                            
                            // Exponential backoff up to 5 minutes
                            token_refresh_backoff_seconds = (token_refresh_backoff_seconds * 2).min(300);
                        }
                    }
                }

                // Priority 2: Drain completed exports
                completed = in_flight_exports.next_completion().fuse() => {
                    match finalize_completed_export(&mut msg_to_batch, &mut batch_to_msg, &mut msg_to_pdata, completed, &effect_handler, self.total_rows_exported, self.processing_start_time).await {
                        Ok(client) => {
                            client_pool.release(client);
                        }
                        Err(e) => {
                            effect_handler
                                .info(&format!("[AzureMonitorExporter] Failed to finalize export: {}", e))
                                .await;
                        }
                    }
                }

                // Priority 3: Handle incoming messages
                msg = msg_chan.recv().fuse() => {
                    self.msg_id += 1;

                    match msg {
                        Ok(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                            effect_handler
                                .info("[AzureMonitorExporter] Shutting down")
                                .await;

                            while !in_flight_exports.is_empty() {
                                let completed = in_flight_exports.next_completion().await;
                                match finalize_completed_export(&mut msg_to_batch, &mut batch_to_msg, &mut msg_to_pdata, completed, &effect_handler, self.total_rows_exported, self.processing_start_time).await {
                                    Ok(client) => {
                                        client_pool.release(client);
                                    }
                                    Err(e) => {
                                        effect_handler
                                            .info(&format!("[AzureMonitorExporter] Failed to finalize export: {}", e))
                                            .await;
                                    }
                                }
                            }

                            // Flush any remaining data
                            match self.gzip_batcher.flush() {
                                FlushResult::Empty => {},
                                FlushResult::Flush => {
                                    let Some(batch) = self.gzip_batcher.take_pending_batch() else {
                                        return Err(Error::InternalError{ message: "Expected non-empty batch after flush".to_string() });
                                    };
                                    
                                    // Update stats for final flush
                                    self.total_rows_exported += batch.row_count;
                                    self.total_batches_exported += 1.0;
                                    
                                    original_client.send(batch.compressed_data)
                                        .await
                                        .map_err(|e| Error::InternalError{ message: format!("Failed to flush remaining data: {}", e) })?;
                                }
                            }

                            // Wait for all in-flight exports to complete
                            effect_handler
                                .info(&format!("[AzureMonitorExporter] Waiting for {} in-flight exports to complete", in_flight_exports.len()))
                                .await;

                            effect_handler
                                .info(&format!(
                                    "[AzureMonitorExporter] Final stats - Exported: {:.0} rows in {:.0} batches, Throughput: {:.2} rows/s",
                                    self.total_rows_exported,
                                    self.total_batches_exported,
                                    self.total_rows_exported / self.processing_start_time.elapsed().as_secs_f64()
                                ))
                                .await;

                            return Ok(TerminalState::new(
                                deadline,
                                std::iter::empty::<otap_df_telemetry::metrics::MetricSetSnapshot>(),
                            ));
                        }

                        Ok(Message::PData(pdata)) => {
                            let (context, payload) = pdata.into_parts();
                            if !self.processing_start_time_set {
                                self.processing_start_time = tokio::time::Instant::now();
                                self.processing_start_time_set = true;
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
                                                .map_err(|e| Error::InternalError{ message: format!("Failed to convert OTAP to OTLP: {:?}", e) })?;

                                        let OtlpProtoBytes::ExportLogsRequest(bytes) = otlp_bytes else {
                                            return Err(Error::InternalError{ message: "Expected ExportLogsRequest bytes".to_string() });
                                        };

                                        msg_to_pdata.entry(self.msg_id)
                                            .or_default()
                                            .push((context, bytes.clone()));

                                        let request = ExportLogsServiceRequest::decode(&bytes[..])
                                            .map_err(|e| Error::InternalError{ message: format!("Failed to decode logs request: {}", e) })?;

                                        let log_entries = self.transformer.convert_to_log_analytics(&request);

                                        process_message(
                                            &mut self,
                                            log_entries,
                                            &mut last_seen_batch_id,
                                            &mut batch_to_msg,
                                            &mut msg_to_batch,
                                            &mut msg_to_pdata,
                                            &mut in_flight_exports,
                                            &mut client_pool,
                                            &effect_handler,
                                        ).await?;                                        
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
                                                .map_err(|e| Error::InternalError { message: format!("Failed to decode OTLP logs request: {e}") })?;

                                            let log_entries = self.transformer.convert_to_log_analytics(&request);

                                            msg_to_pdata.entry(self.msg_id)
                                                .or_default()
                                                .push((context, bytes.clone()));

                                            process_message(
                                                &mut self,
                                                log_entries,
                                                &mut last_seen_batch_id,
                                                &mut batch_to_msg,
                                                &mut msg_to_batch,
                                                &mut msg_to_pdata,
                                                &mut in_flight_exports,
                                                &mut client_pool,
                                                &effect_handler,
                                            ).await?;
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
                        }
                
                        Ok(_) => {
                            // Ignore other message types
                        }
                        Err(e) => {
                            // Channel error, likely closed
                            return Err(Error::InternalError { message: format!("Channel error: {e}") });
                        }
                    }
                } // <--- This closing brace was missing

                // Priority 4: Periodic flush
                _ = tokio::time::sleep_until(next_export).fuse() => {
                    if self.last_export_started + EXPORT_INTERVAL <= tokio::time::Instant::now() {
                        match self.gzip_batcher.flush() {
                            FlushResult::Empty => (),
                            FlushResult::Flush => {
                                let Some(batch) = self.gzip_batcher.take_pending_batch() else {
                                    return Err(Error::InternalError{ message: "Expected non-empty batch after flush".to_string() });
                                };

                                self.last_export_started = tokio::time::Instant::now();
                                
                                // Update stats when batch is flushed
                                self.total_rows_exported += batch.row_count;
                                self.total_batches_exported += 1.0;
                                
                                // Wait if we've hit the concurrency limit
                                while in_flight_exports.len() >= self.max_in_flight {
                                    // Wait for at least one export to complete
                                    let result = in_flight_exports.next_completion().await;
                                    // Handle the result of the completed export
                                    match finalize_completed_export(&mut msg_to_batch, &mut batch_to_msg, &mut msg_to_pdata, result, &effect_handler, self.total_rows_exported, self.processing_start_time).await {
                                        Ok(client) => {
                                            client_pool.release(client);
                                        }
                                        Err(e) => {
                                            effect_handler
                                                .info(&format!("[AzureMonitorExporter] Failed to finalize export: {}", e))
                                                .await;
                                        }
                                    }
                                }

                                let client = client_pool.take();
                                let export_fut = make_export_future(client, batch.compressed_data, batch.batch_id, batch.row_count);
                                in_flight_exports.push(export_fut);
                            }
                        }
                    }

                    next_export = max(self.last_export_started, tokio::time::Instant::now()) + EXPORT_INTERVAL;
                }
            }
        }
    }
}
