// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_config::SignalType;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::extensions::BearerTokenProvider;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::views::logs::LogsDataView;
use otap_df_pdata::views::otap::OtapLogsView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};

use super::client::LogsIngestionClientPool;
use super::config::Config;
use super::error::Error;
use super::gzip_batcher::FinalizeResult;
use super::gzip_batcher::{self, GzipBatcher};
use super::heartbeat::Heartbeat;
use super::in_flight_exports::{CompletedExport, InFlightExports};
use super::state::AzureMonitorExporterState;
use super::stats::AzureMonitorExporterStats;
use super::transformer::Transformer;
use crate::pdata::{Context, OtapPdata};
use reqwest::header::HeaderValue;

const MAX_IN_FLIGHT_EXPORTS: usize = 16;
const PERIODIC_EXPORT_INTERVAL: u64 = 3;
const STATS_PRINT_INTERVAL: u64 = 3;
const HEARTBEAT_INTERVAL_SECONDS: u64 = 60;

/// Azure Monitor exporter.
pub struct AzureMonitorExporter {
    config: Config,
    transformer: Transformer,
    gzip_batcher: GzipBatcher,
    state: AzureMonitorExporterState,
    client_pool: LogsIngestionClientPool,
    in_flight_exports: InFlightExports,
    last_batch_queued_at: tokio::time::Instant,
    stats: AzureMonitorExporterStats,
    heartbeat: Heartbeat,
}

// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
impl AzureMonitorExporter {
    /// Build a new exporter from configuration.
    pub fn new(config: Config) -> Result<Self, Error> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| Error::Config(e.to_string()))?;

        // Create log transformer
        let transformer = Transformer::new(&config);

        // Create Gzip batcher
        let gzip_batcher = GzipBatcher::new();

        // Create heartbeat handler
        let heartbeat = Heartbeat::new(&config.api)?;

        Ok(Self {
            config,
            transformer,
            gzip_batcher,
            state: AzureMonitorExporterState::new(),
            client_pool: LogsIngestionClientPool::new(MAX_IN_FLIGHT_EXPORTS + 1),
            in_flight_exports: InFlightExports::new(MAX_IN_FLIGHT_EXPORTS),
            last_batch_queued_at: tokio::time::Instant::now(),
            stats: AzureMonitorExporterStats::new(),
            heartbeat,
        })
    }

    async fn finalize_export(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        completed_export: CompletedExport,
    ) -> Result<(), EngineError> {
        let CompletedExport {
            batch_id,
            client,
            result,
            row_count,
        } = completed_export;

        // Return the client to the pool
        self.client_pool.release(client);

        match result {
            Ok(duration) => {
                self.handle_export_success(effect_handler, batch_id, row_count, duration)
                    .await
            }
            Err(e) => {
                self.handle_export_failure(effect_handler, batch_id, row_count, e)
                    .await
            }
        }
    }

    async fn handle_export_success(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        batch_id: u64,
        row_count: f64,
        duration: std::time::Duration,
    ) -> Result<(), EngineError> {
        // Export succeeded - Ack only fully-completed messages
        let completed_messages = self.state.remove_batch_success(batch_id);
        self.stats.add_messages(completed_messages.len() as f64);
        self.stats.add_rows(row_count);
        self.stats.add_batch();
        self.stats.add_client_latency(duration.as_secs_f64());

        for (_, context, payload) in completed_messages {
            effect_handler
                .notify_ack(AckMsg::new(OtapPdata::new(context, payload)))
                .await?;
        }
        Ok(())
    }

    async fn handle_export_failure(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        batch_id: u64,
        row_count: f64,
        error: Error,
    ) -> Result<(), EngineError> {
        // Export failed - Nack ALL messages in this batch, remove entirely
        let failed_messages = self.state.remove_batch_failure(batch_id);
        self.stats.add_failed_messages(failed_messages.len() as f64);
        self.stats.add_failed_rows(row_count);
        self.stats.add_failed_batch();

        println!(
            "[AzureMonitorExporter] Export failed: {:?} - {:?}",
            batch_id, error
        );

        for (_, context, payload) in failed_messages {
            effect_handler
                .notify_nack(NackMsg::new(
                    error.to_string(),
                    OtapPdata::new(context, payload),
                ))
                .await?;
        }
        Ok(())
    }

    async fn queue_pending_batch(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        let pending_batch = match self.gzip_batcher.take_pending_batch() {
            Some(batch) => batch,
            None => return Ok(()), // No pending batch - nothing to do
        };

        let client = self.client_pool.take();
        if let Some(completed_export) = self
            .in_flight_exports
            .push_export(
                client,
                pending_batch.batch_id,
                pending_batch.row_count,
                pending_batch.compressed_data,
            )
            .await
        {
            self.finalize_export(effect_handler, completed_export)
                .await?;
        }

        self.last_batch_queued_at = tokio::time::Instant::now();

        Ok(())
    }

    async fn handle_logs_view<T: LogsDataView>(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        context: Context,
        payload: OtapPayload,
        logs_view: &T,
        msg_id: u64,
    ) -> Result<(), EngineError> {
        if context.may_return_payload() {
            self.state.add_msg_to_data(msg_id, context, payload);
        } else {
            self.state
                .add_msg_to_data(msg_id, context, OtapPayload::empty(SignalType::Logs));
        }

        // Use a generic transformer method that accepts LogsDataView
        let log_entries = self.transformer.convert_to_log_analytics(logs_view);

        for log_entry in log_entries {
            match self.gzip_batcher.push(&log_entry) {
                Ok(gzip_batcher::PushResult::Ok(batch_id)) => {
                    // current batch id is being associated with the current message
                    self.state.add_batch_msg_relationship(batch_id, msg_id);
                }
                Ok(gzip_batcher::PushResult::BatchReady(new_batch_id)) => {
                    // new batch id is being associated with the current message
                    self.state.add_batch_msg_relationship(new_batch_id, msg_id);
                    self.queue_pending_batch(effect_handler).await?;
                }
                Ok(gzip_batcher::PushResult::TooLarge) => {
                    let error = Error::LogEntryTooLarge;
                    if let Some((context, payload)) = self.state.remove_msg_to_data(msg_id) {
                        effect_handler
                            .notify_nack(NackMsg::new(
                                error.to_string(),
                                OtapPdata::new(context, payload),
                            ))
                            .await?;
                    }
                    return Err(EngineError::InternalError {
                        message: error.to_string(),
                    });
                }
                Err(error) => {
                    if let Some((context, payload)) = self.state.remove_msg_to_data(msg_id) {
                        effect_handler
                            .notify_nack(NackMsg::new(
                                error.to_string(),
                                OtapPdata::new(context, payload),
                            ))
                            .await?;
                    }
                    return Err(EngineError::InternalError {
                        message: error.to_string(),
                    });
                }
            }
        }

        if let Some((context, payload)) = self.state.delete_msg_data_if_orphaned(msg_id) {
            effect_handler
                .notify_nack(NackMsg::new(
                    "No valid log entries produced",
                    OtapPdata::new(context, payload),
                ))
                .await?;
        }

        Ok(())
    }

    async fn drain_in_flight_exports(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        let completed_exports = self.in_flight_exports.drain().await;
        for completed_export in completed_exports {
            self.finalize_export(effect_handler, completed_export)
                .await?;
        }
        Ok(())
    }

    async fn queue_current_batch(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match self.gzip_batcher.finalize() {
            Ok(FinalizeResult::Ok) => {
                return self.queue_pending_batch(effect_handler).await;
            }
            Ok(FinalizeResult::Empty) => Ok(()),
            Err(error) => Err(EngineError::InternalError {
                message: error.to_string(),
            }),
        }
    }

    async fn handle_shutdown(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        self.queue_current_batch(effect_handler).await?;
        self.drain_in_flight_exports(effect_handler).await?;

        for (msg_id, context, payload) in self.state.drain_all() {
            print!("Found orphaned message {msg_id} in shutdown");
            effect_handler
                .notify_nack(NackMsg::new(
                    "Shutdown before export completed",
                    OtapPdata::new(context, payload),
                ))
                .await?;
        }

        Ok(())
    }

    async fn handle_message(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        msg: Result<Message<OtapPdata>, RecvError>,
        msg_id: &mut u64,
    ) -> Result<(), EngineError> {
        match msg {
            Ok(Message::PData(pdata)) => {
                *msg_id += 1;
                let (context, payload) = pdata.into_parts();
                let payload_to_match = payload.clone();

                match payload_to_match {
                    OtapPayload::OtapArrowRecords(otap_records) => {
                        match otap_records {
                            OtapArrowRecords::Logs(otap_records) => {
                                let otap_arrow_records = OtapArrowRecords::Logs(otap_records);

                                let logs_view = OtapLogsView::try_from(&otap_arrow_records)
                                    .map_err(|e| {
                                        let error = Error::LogsViewCreationFailed { source: e };
                                        EngineError::InternalError {
                                            message: error.to_string(),
                                        }
                                    })?;

                                self.handle_logs_view(
                                    effect_handler,
                                    context,
                                    payload,
                                    &logs_view,
                                    *msg_id,
                                )
                                .await?;
                            }
                            OtapArrowRecords::Metrics(_) | OtapArrowRecords::Traces(_) => {
                                // Unsupported signal types - silently drop
                            }
                        }
                    }

                    OtapPayload::OtlpBytes(otlp_bytes) => match otlp_bytes {
                        OtlpProtoBytes::ExportLogsRequest(bytes) => {
                            let logs_view = RawLogsData::new(bytes.as_ref());

                            self.handle_logs_view(
                                effect_handler,
                                context,
                                payload,
                                &logs_view,
                                *msg_id,
                            )
                            .await?;
                        }
                        OtlpProtoBytes::ExportMetricsRequest(_)
                        | OtlpProtoBytes::ExportTracesRequest(_) => {
                            // Unsupported signal types - silently drop
                        }
                    },
                }
            }

            Ok(_) => {} // Ignore other message types

            Err(e) => {
                let error = Error::ChannelRecv(e);
                return Err(EngineError::InternalError {
                    message: error.to_string(),
                });
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
impl Exporter<OtapPdata> for AzureMonitorExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        effect_handler
            .info(&format!(
                "[AzureMonitorExporter] Starting: endpoint={}, stream={}, dcr={}",
                self.config.api.dcr_endpoint, self.config.api.stream_name, self.config.api.dcr
            ))
            .await;

        let mut msg_id = 0;

        let auth = effect_handler
            .get_extension::<dyn BearerTokenProvider>(self.config.auth.as_str())
            .map_err(|e| {
                let error = Error::AuthHandlerCreation(Box::new(e));
                EngineError::InternalError {
                    message: error.to_string(),
                }
            })?;

        self.client_pool
            .initialize(&self.config.api)
            .await
            .map_err(|e| {
                let error = Error::ClientPoolInit(Box::new(e));
                EngineError::InternalError {
                    message: error.to_string(),
                }
            })?;

        // Subscribe to token refresh events from the auth extension
        let mut token_rx = auth.subscribe_token_refresh();

        // Wait for the initial token - blocks until the auth extension provides one
        println!("[AzureMonitorExporter] Waiting for initial auth token...");
        let _ =
            token_rx
                .wait_for(|t| t.is_some())
                .await
                .map_err(|_| EngineError::InternalError {
                    message: "Auth extension closed before providing a token".to_string(),
                })?;

        // Now we're guaranteed to have a token
        if let Some(token) = token_rx.borrow().as_ref() {
            let header = HeaderValue::from_str(&format!("Bearer {}", token.token.secret()))
                .map_err(|e| EngineError::InternalError {
                    message: format!("Failed to create auth header: {:?}", e),
                })?;
            self.client_pool.update_auth(header.clone());
            self.heartbeat.update_auth(header);
            println!("[AzureMonitorExporter] Initial auth token set");
        }

        let mut next_stats_print =
            tokio::time::Instant::now() + tokio::time::Duration::from_secs(STATS_PRINT_INTERVAL);
        let mut next_periodic_export = tokio::time::Instant::now()
            + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL);
        let mut next_heartbeat_send = tokio::time::Instant::now();

        loop {
            // Determine if we should accept new messages
            let at_capacity = self.in_flight_exports.len() >= MAX_IN_FLIGHT_EXPORTS;

            tokio::select! {
                biased;

                // React to token refresh events from the auth extension
                _ = token_rx.changed() => {
                    if let Some(token) = token_rx.borrow_and_update().as_ref() {
                        match HeaderValue::from_str(&format!("Bearer {}", token.token.secret())) {
                            Ok(header) => {
                                self.client_pool.update_auth(header.clone());
                                self.heartbeat.update_auth(header);
                                println!("[AzureMonitorExporter] Auth token refreshed");
                            }
                            Err(e) => {
                                println!("[AzureMonitorExporter] Failed to create auth header: {:?}", e);
                            }
                        }
                    }
                }

                _ = tokio::time::sleep_until(next_heartbeat_send) => {
                    next_heartbeat_send = tokio::time::Instant::now() + tokio::time::Duration::from_secs(HEARTBEAT_INTERVAL_SECONDS);
                    match self.heartbeat.send().await {
                        Ok(_) => println!("[AzureMonitorExporter] Heartbeat sent"),
                        Err(e) => println!("[AzureMonitorExporter] Heartbeat send failed: {:?}", e),
                    }
                }

                completed = self.in_flight_exports.next_completion() => {
                    if let Some(completed_export) = completed {
                        self.finalize_export(&effect_handler, completed_export).await?;
                    }
                }

                _ = tokio::time::sleep_until(next_periodic_export), if !at_capacity => {
                    next_periodic_export = tokio::time::Instant::now() + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL);

                    if self.last_batch_queued_at.elapsed() >= std::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL) && self.gzip_batcher.has_pending_data() {
                        println!("[AzureMonitorExporter] Periodic export pending data");
                        self.queue_current_batch(&effect_handler).await?;
                    }
                }

                msg = msg_chan.recv(), if !at_capacity => {
                    self.stats.message_received(self.in_flight_exports.len());

                    match msg {
                        Ok(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                            self.handle_shutdown(&effect_handler).await?;
                            return Ok(TerminalState::new(
                                deadline,
                                std::iter::empty::<otap_df_telemetry::metrics::MetricSetSnapshot>(),
                            ));
                        }
                        other => {
                            self.handle_message(&effect_handler, other, &mut msg_id).await?;
                        }
                    }
                }

                _ = tokio::time::sleep_until(next_stats_print) => {
                    next_stats_print = tokio::time::Instant::now() + tokio::time::Duration::from_secs(STATS_PRINT_INTERVAL);

                    let stats_start = std::time::Instant::now();

                    // Get memory stats (this is the slow part - file I/O)
                    let status = tokio::fs::read_to_string("/proc/self/status").await.unwrap_or_default();
                    let get_kb = |name: &str| -> u64 {
                        status.lines()
                            .find(|line| line.starts_with(name))
                            .and_then(|line| line.split_whitespace().nth(1)?.parse().ok())
                            .unwrap_or(0)
                    };

                    let smaps = tokio::fs::read_to_string("/proc/self/smaps_rollup").await.unwrap_or_default();
                    let get_smaps_kb = |name: &str| -> u64 {
                        smaps.lines()
                            .find(|line| line.starts_with(name))
                            .and_then(|line| line.split_whitespace().nth(1)?.parse().ok())
                            .unwrap_or(0)
                    };

                    let rss_mb = get_kb("VmRSS:") / 1024;
                    let anon_mb = get_smaps_kb("Anonymous:") / 1024;
                    let file_mb = get_smaps_kb("Private_Clean:") / 1024;
                    let data_mb = get_kb("VmData:") / 1024;

                    let stats_duration = stats_start.elapsed();

                    let elapsed = self.stats.started_at().elapsed().as_secs_f64();
                    let active = self.stats.get_active_duration_secs(self.in_flight_exports.len());
                    let throughput = if active > 0.0 {
                        self.stats.successful_row_count() / active
                    } else {
                        0.0
                    };

                    println!(
                        "\n\
─────────────── AzureMonitorExporter ──────────────────────────
memory  │ rss={}MB  anon={}MB  file={}MB  data={}MB
perf    │ th/s={:.2}  avg_lat={:.2}ms
success │ rows={:.0}  batches={:.0}  msgs={:.0}         
fail    │ rows={:.0}  batches={:.0}  msgs={:.0}       
time    │ elapsed={:.1}s  active={:.1}s  idle={:.1}s
state   | batch_to_msg={}  msg_to_batch={}  msg_to_data={}
exports | in_flight={} stats_time={:?}
───────────────────────────────────────────────────────────────\n",
                        rss_mb,
                        anon_mb,
                        file_mb,
                        data_mb,
                        throughput,
                        self.stats.average_client_latency_secs() * 1000.0,
                        self.stats.successful_row_count(),
                        self.stats.successful_batch_count(),
                        self.stats.successful_msg_count(),
                        self.stats.failed_row_count(),
                        self.stats.failed_batch_count(),
                        self.stats.failed_msg_count(),
                        elapsed,
                        active,
                        self.stats.get_idle_duration_secs(),
                        self.state.batch_to_msg.len(),
                        self.state.msg_to_batch.len(),
                        self.state.msg_to_data.len(),
                        self.in_flight_exports.len(),
                        stats_duration,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::{ApiConfig, SchemaConfig};
    use super::*;
    use crate::pdata::Context;
    use bytes::Bytes;
    use http::StatusCode;
    use otap_df_engine::local::exporter::EffectHandler;
    use otap_df_engine::node::NodeId;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        Config {
            api: ApiConfig {
                dcr_endpoint: "https://example.com".to_string(),
                stream_name: "stream".to_string(),
                dcr: "dcr-id".to_string(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::new(),
                    scope_mapping: HashMap::new(),
                    log_record_mapping: HashMap::new(),
                },
            },
            auth: "azure_identity_auth".to_string(),
        }
    }

    #[test]
    fn test_new_validates_config() {
        let config = create_test_config();
        let _ = AzureMonitorExporter::new(config).unwrap();
    }

    #[tokio::test]
    async fn test_handle_export_success() {
        let config = create_test_config();
        let mut exporter = AzureMonitorExporter::new(config).unwrap();

        let (_, reporter) = MetricsReporter::create_new_and_receiver(10);
        let node_id = NodeId {
            index: 0,
            name: "test_exporter".to_string().into(),
        };
        let effect_handler = EffectHandler::new(node_id, reporter);

        let batch_id = 1;
        let msg_id = 100;
        let context = Context::default();
        let payload =
            OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(Bytes::from("test")));

        exporter
            .state
            .add_msg_to_data(msg_id, context.clone(), payload);
        exporter.state.add_batch_msg_relationship(batch_id, msg_id);

        // This might fail due to missing sender in effect_handler, but state should be updated
        let _ = exporter
            .handle_export_success(
                &effect_handler,
                batch_id,
                10.0,
                std::time::Duration::from_secs(1),
            )
            .await;

        // Verify stats
        assert_eq!(exporter.stats.successful_batch_count(), 1.0);
        assert_eq!(exporter.stats.successful_msg_count(), 1.0);
        assert_eq!(exporter.stats.successful_row_count(), 10.0);

        // Verify state cleared
        assert!(exporter.state.batch_to_msg.is_empty());
        assert!(exporter.state.msg_to_data.is_empty());
    }

    #[tokio::test]
    async fn test_handle_export_failure() {
        let config = create_test_config();
        let mut exporter = AzureMonitorExporter::new(config).unwrap();

        let (_, reporter) = MetricsReporter::create_new_and_receiver(10);
        let node_id = NodeId {
            index: 0,
            name: "test_exporter".to_string().into(),
        };
        let effect_handler = EffectHandler::new(node_id, reporter);

        let batch_id = 1;
        let msg_id = 100;
        let context = Context::default();
        let payload =
            OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(Bytes::from("test")));

        exporter
            .state
            .add_msg_to_data(msg_id, context.clone(), payload);
        exporter.state.add_batch_msg_relationship(batch_id, msg_id);

        let error = Error::ServerError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: "Simulated error".to_string(),
            retry_after: None,
        };

        let _ = exporter
            .handle_export_failure(&effect_handler, batch_id, 10.0, error)
            .await;

        // Verify stats
        assert_eq!(exporter.stats.failed_batch_count(), 1.0);
        assert_eq!(exporter.stats.failed_msg_count(), 1.0);
        assert_eq!(exporter.stats.failed_row_count(), 10.0);

        // Verify state cleared
        assert!(exporter.state.batch_to_msg.is_empty());
        assert!(exporter.state.msg_to_data.is_empty());
    }
}
