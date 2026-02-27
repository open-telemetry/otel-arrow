// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use azure_core::credentials::AccessToken;
use otap_df_channel::error::RecvError;
use otap_df_config::SignalType;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
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
use super::error::Error;
use super::gzip_batcher::FinalizeResult;
use super::gzip_batcher::{self, GzipBatcher};
use super::heartbeat::Heartbeat;
use super::in_flight_exports::{CompletedExport, InFlightExports};
use super::metrics::{AzureMonitorExporterMetrics, AzureMonitorExporterMetricsRc};
use super::state::AzureMonitorExporterState;
use super::transformer::Transformer;
use otap_df_otap::pdata::{Context, OtapPdata};
use reqwest::header::HeaderValue;

use otap_df_telemetry::{otel_debug, otel_error, otel_info, otel_warn};

use std::cell::RefCell;
use std::rc::Rc;

const MAX_IN_FLIGHT_EXPORTS: usize = 16;
const PERIODIC_EXPORT_INTERVAL: u64 = 3;
const HEARTBEAT_INTERVAL_SECONDS: u64 = 60;
/// Minimum interval between token refresh attempts (10 seconds).
const MIN_TOKEN_REFRESH_INTERVAL_SECS: u64 = 10;
/// Buffer time before token expiry to trigger a refresh.
/// Azure Identity SDK caches tokens internally and won't issue a new token
/// until ~5 minutes before expiry, so we schedule refresh at 295 seconds before expiry.
const TOKEN_EXPIRY_BUFFER_SECS: u64 = 295;

/// Azure Monitor exporter.
pub struct AzureMonitorExporter {
    config: Config,
    transformer: Transformer,
    gzip_batcher: GzipBatcher,
    state: AzureMonitorExporterState,
    metrics: AzureMonitorExporterMetricsRc,
    client_pool: LogsIngestionClientPool,
    in_flight_exports: InFlightExports,
    last_batch_queued_at: tokio::time::Instant,
    heartbeat: Heartbeat,
}

impl AzureMonitorExporter {
    /// Build a new exporter from configuration.
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Result<Self, Error> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| Error::Config(e.to_string()))?;

        // Register metrics with the telemetry system
        let metric_set = pipeline_ctx.register_metrics::<AzureMonitorExporterMetrics>();
        let metrics: AzureMonitorExporterMetricsRc = Rc::new(RefCell::new(
            super::metrics::AzureMonitorExporterMetricsTracker::new(metric_set),
        ));

        // Create log transformer
        let transformer = Transformer::new(&config, metrics.clone());

        // Create Gzip batcher
        let gzip_batcher = GzipBatcher::new();

        // Create heartbeat handler
        let heartbeat = Heartbeat::new(&config.api)?;

        Ok(Self {
            config,
            transformer,
            gzip_batcher,
            state: AzureMonitorExporterState::new(),
            metrics: metrics.clone(),
            client_pool: LogsIngestionClientPool::new(MAX_IN_FLIGHT_EXPORTS + 1, metrics),
            in_flight_exports: InFlightExports::new(MAX_IN_FLIGHT_EXPORTS),
            last_batch_queued_at: tokio::time::Instant::now(),
            heartbeat,
        })
    }

    /// Update all gauges (in-flight exports + state map sizes).
    #[inline]
    fn sync_gauges(&self) {
        let mut m = self.metrics.borrow_mut();
        m.set_in_flight_exports(self.in_flight_exports.len() as u64);
        m.set_batch_to_msg_count(self.state.batch_to_msg.len() as u64);
        m.set_msg_to_batch_count(self.state.msg_to_batch.len() as u64);
        m.set_msg_to_data_count(self.state.msg_to_data.len() as u64);
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
        {
            let mut m = self.metrics.borrow_mut();
            m.add_messages(completed_messages.len() as u64);
            m.add_rows(row_count as u64);
            m.add_batch();
        }

        otel_debug!(
            "azure_monitor_exporter.export.success",
            batch_id = batch_id,
            row_count = row_count,
            duration_ms = duration.as_millis() as u64
        );

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
        {
            let mut m = self.metrics.borrow_mut();
            m.add_failed_messages(failed_messages.len() as u64);
            m.add_failed_rows(row_count as u64);
            m.add_failed_batch();
        }

        otel_error!("azure_monitor_exporter.export.failed", batch_id = batch_id, error = %error);

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
                    self.metrics.borrow_mut().add_log_entry_too_large();
                    otel_warn!(
                        "azure_monitor_exporter.message.log_entry_too_large",
                        msg_id = msg_id,
                        size_bytes = log_entry.len()
                    );
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
                    otel_error!("azure_monitor_exporter.message.batch_push_failed", msg_id = msg_id, error = %error);
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
            otel_debug!(
                "azure_monitor_exporter.message.no_valid_entries",
                msg_id = msg_id
            );
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
            otel_warn!(
                "azure_monitor_exporter.shutdown.orphaned_message",
                msg_id = msg_id
            );
            effect_handler
                .notify_nack(NackMsg::new(
                    "Shutdown before export completed",
                    OtapPdata::new(context, payload),
                ))
                .await?;
        }

        otel_info!("azure_monitor_exporter.exporter.shutdown");

        Ok(())
    }

    #[inline]
    fn get_next_token_refresh(token: AccessToken) -> tokio::time::Instant {
        let now = azure_core::time::OffsetDateTime::now_utc();
        let duration_remaining = if token.expires_on > now {
            (token.expires_on - now).unsigned_abs()
        } else {
            std::time::Duration::ZERO
        };

        let token_valid_until = tokio::time::Instant::now() + duration_remaining;
        let next_token_refresh =
            token_valid_until - tokio::time::Duration::from_secs(TOKEN_EXPIRY_BUFFER_SECS);
        std::cmp::max(
            next_token_refresh,
            tokio::time::Instant::now()
                + tokio::time::Duration::from_secs(MIN_TOKEN_REFRESH_INTERVAL_SECS),
        )
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
                    OtapPayload::OtapArrowRecords(otap_records) => match otap_records {
                        OtapArrowRecords::Logs(otap_records) => {
                            let otap_arrow_records = OtapArrowRecords::Logs(otap_records);

                            let logs_view =
                                OtapLogsView::try_from(&otap_arrow_records).map_err(|e| {
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
                            otel_warn!(
                                "azure_monitor_exporter.message.unsupported_signal",
                                signal = "metrics_or_traces",
                                format = "otap_arrow"
                            );
                        }
                    },

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
                            otel_warn!(
                                "azure_monitor_exporter.message.unsupported_signal",
                                signal = "metrics_or_traces",
                                format = "otlp_proto"
                            );
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

        let mut auth = Auth::new(&self.config.auth, self.metrics.clone()).map_err(|e| {
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

        // Start periodic telemetry collection and retain the cancel handle for graceful shutdown
        let telemetry_timer_cancel_handle = effect_handler
            .start_periodic_telemetry(std::time::Duration::from_secs(1))
            .await
            .map_err(|e| EngineError::InternalError {
                message: format!("Failed to start telemetry timer: {e}"),
            })?;

        let mut next_token_refresh = tokio::time::Instant::now();
        let mut next_periodic_export = tokio::time::Instant::now()
            + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL);
        let mut next_heartbeat_send = tokio::time::Instant::now();

        loop {
            // Determine if we should accept new messages
            let at_capacity = self.in_flight_exports.len() >= MAX_IN_FLIGHT_EXPORTS;

            tokio::select! {
                biased;

                _ = tokio::time::sleep_until(next_token_refresh) => {
                    match auth.get_token().await {
                        Ok(access_token) => {
                            match HeaderValue::from_str(&format!("Bearer {}", access_token.token.secret())) {
                                Ok(header) => {
                                    self.client_pool.update_auth(header.clone());
                                    self.heartbeat.update_auth(header.clone());

                                    // Schedule next token refresh
                                    next_token_refresh = Self::get_next_token_refresh(access_token);

                                    let refresh_in = next_token_refresh.saturating_duration_since(tokio::time::Instant::now());
                                    let total_secs = refresh_in.as_secs();
                                    let hours = total_secs / 3600;
                                    let minutes = (total_secs % 3600) / 60;
                                    let seconds = total_secs % 60;

                                    otel_info!("azure_monitor_exporter.auth.token_refresh", refresh_in = format!("{}h {}m {}s", hours, minutes, seconds));
                                }
                                Err(e) => {
                                    otel_error!("azure_monitor_exporter.auth.header_creation_failed", error = ?e);
                                    // Retry every 10 seconds
                                    next_token_refresh = tokio::time::Instant::now() + tokio::time::Duration::from_secs(10);
                                }
                            }

                        }
                        Err(e) => {
                            otel_error!("azure_monitor_exporter.auth.token_refresh_failed", error = ?e);
                            // Retry every 10 seconds
                            next_token_refresh = tokio::time::Instant::now() + tokio::time::Duration::from_secs(10);
                        }
                    }
                }

                _ = tokio::time::sleep_until(next_heartbeat_send) => {
                    next_heartbeat_send = tokio::time::Instant::now() + tokio::time::Duration::from_secs(HEARTBEAT_INTERVAL_SECONDS);
                    self.metrics.borrow_mut().add_heartbeat();
                    match self.heartbeat.send().await {
                        Ok(_) => otel_debug!("azure_monitor_exporter.heartbeat.sent"),
                        Err(e) => otel_warn!("azure_monitor_exporter.heartbeat.send_failed", error = ?e),
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
                        otel_debug!("azure_monitor_exporter.export.periodic_flush");
                        self.queue_current_batch(&effect_handler).await?;
                    }
                }

                msg = msg_chan.recv() => {
                    match msg {
                        Ok(Message::Control(NodeControlMsg::CollectTelemetry { mut metrics_reporter })) => {
                            self.sync_gauges();
                            if tracing::enabled!(tracing::Level::DEBUG) {
                                let m = self.metrics.borrow();
                                let cl = m.client_success_latency();
                                let al = m.auth_success_latency();
                                let bs = m.batch_size();
                                otel_debug!(
                                    "azure_monitor_exporter.metrics.collect",
                                    successful_rows = m.successful_row_count(),
                                    successful_batches = m.successful_batch_count(),
                                    successful_messages = m.successful_msg_count(),
                                    failed_rows = m.failed_row_count(),
                                    failed_batches = m.failed_batch_count(),
                                    failed_messages = m.failed_msg_count(),
                                    client_success_latency_avg_ms = if cl.count > 0 { cl.sum / cl.count as f64 } else { 0.0 },
                                    client_success_latency_min_ms = if cl.count > 0 { cl.min } else { 0.0 },
                                    client_success_latency_max_ms = if cl.count > 0 { cl.max } else { 0.0 },
                                    client_success_latency_count = cl.count,
                                    auth_success_latency_avg_ms = if al.count > 0 { al.sum / al.count as f64 } else { 0.0 },
                                    auth_success_latency_count = al.count,
                                    batch_size_avg_bytes = if bs.count > 0 { bs.sum / bs.count as f64 } else { 0.0 },
                                    batch_size_min_bytes = if bs.count > 0 { bs.min } else { 0.0 },
                                    batch_size_max_bytes = if bs.count > 0 { bs.max } else { 0.0 },
                                    batch_size_count = bs.count,
                                    in_flight = self.in_flight_exports.len()
                                );
                            }
                            let _ = self.metrics.borrow_mut().report(&mut metrics_reporter);
                        }
                        Ok(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                            let _ = telemetry_timer_cancel_handle.cancel().await;
                            self.handle_shutdown(&effect_handler).await?;
                            let snapshot = self.metrics.borrow().metrics().snapshot();
                            return Ok(TerminalState::new(
                                deadline,
                                [snapshot],
                            ));
                        }
                        other => {
                            self.handle_message(&effect_handler, other, &mut msg_id).await?;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::{ApiConfig, AuthConfig, SchemaConfig};
    use super::*;
    use azure_core::time::OffsetDateTime;
    use bytes::Bytes;
    use http::StatusCode;
    use otap_df_engine::context::{ControllerContext, PipelineContext};
    use otap_df_engine::local::exporter::EffectHandler;
    use otap_df_engine::node::NodeId;
    use otap_df_otap::pdata::Context;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::collections::HashMap;

    fn create_test_pipeline_ctx() -> PipelineContext {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0)
    }

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
        let pipeline_ctx = create_test_pipeline_ctx();
        let _ = AzureMonitorExporter::new(pipeline_ctx, config).unwrap();
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

        // Should be 3600 - 295 = 3305 seconds before refresh
        // Allow some delta for execution time
        let expected = 3305.0;
        let actual = duration_until_refresh.as_secs_f64();
        assert!(
            (actual - expected).abs() < 5.0,
            "Expected ~{}, got {}",
            expected,
            actual
        );
    }

    #[tokio::test]
    async fn test_handle_export_success() {
        let config = create_test_config();
        let pipeline_ctx = create_test_pipeline_ctx();
        let mut exporter = AzureMonitorExporter::new(pipeline_ctx, config).unwrap();

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
        let m = exporter.metrics.borrow();
        assert_eq!(m.successful_batch_count(), 1);
        assert_eq!(m.successful_msg_count(), 1);
        assert_eq!(m.successful_row_count(), 10);
        drop(m);

        // Verify state cleared
        assert!(exporter.state.batch_to_msg.is_empty());
        assert!(exporter.state.msg_to_data.is_empty());
    }

    #[tokio::test]
    async fn test_handle_export_failure() {
        let config = create_test_config();
        let pipeline_ctx = create_test_pipeline_ctx();
        let mut exporter = AzureMonitorExporter::new(pipeline_ctx, config).unwrap();

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
        let m = exporter.metrics.borrow();
        assert_eq!(m.failed_batch_count(), 1);
        assert_eq!(m.failed_msg_count(), 1);
        assert_eq!(m.failed_row_count(), 10);
        drop(m);

        // Verify state cleared
        assert!(exporter.state.batch_to_msg.is_empty());
        assert!(exporter.state.msg_to_data.is_empty());
    }
}
