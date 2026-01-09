// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::max;

use async_trait::async_trait;
use azure_core::credentials::AccessToken;
use otap_df_channel::error::RecvError;
use otap_df_config::SignalType;
use otap_df_engine::ConsumerEffectHandlerExtension; // Add this import
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::views::logs::LogsDataView;
use otap_df_pdata::views::otap::OtapLogsView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};

use super::auth::Auth;
use super::client::LogsIngestionClientPool;
use super::config::Config;
use super::gzip_batcher::FinalizeResult;
use super::gzip_batcher::{self, GzipBatcher};
use super::in_flight_exports::{CompletedExport, InFlightExports};
use super::state::AzureMonitorExporterState;
use super::stats::AzureMonitorExporterStats;
use super::transformer::Transformer;
use crate::pdata::{Context, OtapPdata};

const MAX_IN_FLIGHT_EXPORTS: usize = 16;
const PERIODIC_EXPORT_INTERVAL: u64 = 3;
const STATS_PRINT_INTERVAL: u64 = 3;

/// Azure Monitor exporter.
pub struct AzureMonitorExporter {
    // Define the fields of the AzureMonitorExporter struct
    config: Config,
    transformer: Transformer,
    gzip_batcher: GzipBatcher,
    state: AzureMonitorExporterState,
    client_pool: LogsIngestionClientPool,
    in_flight_exports: InFlightExports,
    last_batch_queued_at: tokio::time::Instant,
    stats: AzureMonitorExporterStats,
}

// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
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
            state: AzureMonitorExporterState::new(),
            client_pool: LogsIngestionClientPool::new(MAX_IN_FLIGHT_EXPORTS + 1),
            in_flight_exports: InFlightExports::new(MAX_IN_FLIGHT_EXPORTS),
            last_batch_queued_at: tokio::time::Instant::now(),
            stats: AzureMonitorExporterStats::new(),
        })
    }

    async fn finalize_export(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        completed_export: CompletedExport,
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
        error: String,
    ) -> Result<(), Error> {
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
                .notify_nack(NackMsg::new(&error, OtapPdata::new(context, payload)))
                .await?;
        }
        Ok(())
    }

    fn get_next_token_refresh(token: AccessToken) -> tokio::time::Instant {
        let now = azure_core::time::OffsetDateTime::now_utc();
        let duration_remaining = if token.expires_on > now {
            (token.expires_on - now).unsigned_abs()
        } else {
            std::time::Duration::ZERO
        };

        let token_valid_until = tokio::time::Instant::now() + duration_remaining;
        let token_lifetime =
            token_valid_until.saturating_duration_since(tokio::time::Instant::now());
        let token_expiry_buffer = tokio::time::Duration::from_secs(token_lifetime.as_secs() / 5);
        let next_token_refresh = token_valid_until - token_expiry_buffer;
        max(
            next_token_refresh,
            tokio::time::Instant::now() + std::time::Duration::from_secs(30),
        )
    }

    async fn queue_pending_batch(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
                    if let Some((context, payload)) = self.state.remove_msg_to_data(msg_id) {
                        effect_handler
                            .notify_nack(NackMsg::new(
                                "Log entry too large to export",
                                OtapPdata::new(context, payload),
                            ))
                            .await?;
                    }
                    return Err(Error::InternalError {
                        message: "Log entry too large to export".to_string(),
                    });
                }
                Err(e) => {
                    if let Some((context, payload)) = self.state.remove_msg_to_data(msg_id) {
                        effect_handler
                            .notify_nack(NackMsg::new(
                                "Failed to add log entry to batch",
                                OtapPdata::new(context, payload),
                            ))
                            .await?;
                    }
                    return Err(Error::InternalError {
                        message: format!("Failed to add log entry to batch: {:?}", e),
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
        match self.gzip_batcher.finalize() {
            Ok(FinalizeResult::Ok) => {
                return self.queue_pending_batch(effect_handler).await;
            }
            Ok(FinalizeResult::Empty) => Ok(()),
            Err(e) => Err(Error::InternalError {
                message: format!("Failed to finalize batch: {:?}", e),
            }),
        }
    }

    async fn handle_shutdown(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
                                    .map_err(|e| Error::InternalError {
                                        message: format!("Failed to create OtapLogsView: {:?}", e),
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
                return Err(Error::InternalError {
                    message: format!("Channel error: {e}"),
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
    ) -> Result<TerminalState, Error> {
        effect_handler
            .info(&format!(
                "[AzureMonitorExporter] Starting: endpoint={}, stream={}, dcr={}",
                self.config.api.dcr_endpoint, self.config.api.stream_name, self.config.api.dcr
            ))
            .await;

        let mut msg_id = 0;

        let auth = Auth::new(&self.config.auth).map_err(|e| Error::InternalError {
            message: format!("Failed to create auth handler: {e}"),
        })?;

        let token = auth.get_token().await.map_err(|e| Error::InternalError {
            message: format!("Failed to refresh token: {e}"),
        })?;

        self.client_pool
            .initialize(&self.config.api, &auth)
            .await
            .expect("Failed to initialize client pool");
        let mut next_token_refresh = Self::get_next_token_refresh(token);
        let mut next_stats_print =
            tokio::time::Instant::now() + tokio::time::Duration::from_secs(STATS_PRINT_INTERVAL);
        let mut next_periodic_export = tokio::time::Instant::now()
            + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL);

        loop {
            // Determine if we should accept new messages
            let at_capacity = self.in_flight_exports.len() >= MAX_IN_FLIGHT_EXPORTS;

            tokio::select! {
                biased;

                _ = tokio::time::sleep_until(next_token_refresh) => {
                    let token = auth.get_token()
                        .await
                        .map_err(|e| Error::InternalError { message: format!("Failed to refresh token: {e}") })?;

                    next_token_refresh = Self::get_next_token_refresh(token);
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
                    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
                    let get_kb = |name: &str| -> u64 {
                        status.lines()
                            .find(|line| line.starts_with(name))
                            .and_then(|line| line.split_whitespace().nth(1)?.parse().ok())
                            .unwrap_or(0)
                    };

                    let smaps = std::fs::read_to_string("/proc/self/smaps_rollup").unwrap_or_default();
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
    use super::super::config::{ApiConfig, AuthConfig, SchemaConfig};
    use super::*;
    use crate::pdata::Context;
    use azure_core::time::OffsetDateTime;
    use bytes::Bytes;
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
            auth: AuthConfig::default(),
        }
    }

    #[test]
    fn test_new_validates_config() {
        let config = create_test_config();
        let exporter = AzureMonitorExporter::new(config);
        assert!(exporter.is_ok());
    }

    #[test]
    fn test_get_next_token_refresh_logic() {
        let now = OffsetDateTime::now_utc();
        let expires_on = now + azure_core::time::Duration::seconds(3600);

        let token = AccessToken {
            token: "secret".into(),
            expires_on,
        };

        let refresh_at = AzureMonitorExporter::get_next_token_refresh(token);
        let duration_until_refresh = refresh_at.duration_since(tokio::time::Instant::now());

        // Should be around 80% of 3600 = 2880 seconds
        // Allow some delta for execution time
        let expected = 2880.0;
        let actual = duration_until_refresh.as_secs_f64();
        assert!(
            (actual - expected).abs() < 5.0,
            "Expected ~{}, got {}",
            expected,
            actual
        );
    }

    #[test]
    fn test_get_next_token_refresh_minimum_interval() {
        let now = OffsetDateTime::now_utc();
        let expires_on = now + azure_core::time::Duration::seconds(10);

        let token = AccessToken {
            token: "secret".into(),
            expires_on,
        };

        let before = tokio::time::Instant::now();
        let refresh_at = AzureMonitorExporter::get_next_token_refresh(token);
        let duration_until_refresh = refresh_at.saturating_duration_since(before);

        // Should enforce minimum 30s refresh interval (allow 1s tolerance for execution time)
        assert!(
            duration_until_refresh.as_secs() >= 29,
            "Expected at least 29s, got {}s",
            duration_until_refresh.as_secs()
        );
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

        let _ = exporter
            .handle_export_failure(
                &effect_handler,
                batch_id,
                10.0,
                "Simulated error".to_string(),
            )
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
