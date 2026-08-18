// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_config::SignalType;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otlp::OtlpProtoBytes;
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
use super::metrics::AzureMonitorExporterMetricsRc;
use super::state::AzureMonitorExporterState;
use super::transformer::Transformer;
use otap_df_otap::bearer_auth::{BearerAuth, BearerAuthEvents};
use otap_df_otap::pdata::{Context, OtapPdata};

use otap_df_telemetry::common_attributes::{HttpResponse, Outcome};

use bytes::Bytes;
use std::cell::RefCell;
use std::rc::Rc;

/// Max concurrent HTTP requests in flight to the Logs Ingestion API.
const MAX_IN_FLIGHT_EXPORTS: usize = 16;
const PERIODIC_EXPORT_INTERVAL: u64 = 3;

/// Raises shared bearer-auth warnings under the Azure Monitor event namespace.
const AZURE_MONITOR_BEARER_AUTH_EVENTS: BearerAuthEvents = BearerAuthEvents {
    invalid_token: |error| {
        otel_warn!("azure_monitor_exporter.auth.invalid_bearer_token", error = %error);
    },
    token_stream_closed: || {
        otel_warn!(
            "azure_monitor_exporter.auth.token_stream_closed",
            message =
                "bearer token provider closed its stream; no further token refreshes will arrive"
        );
    },
};

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
    heartbeat: Option<Heartbeat>,
    token_provider: Option<Box<dyn BearerTokenProvider>>,
}

impl AzureMonitorExporter {
    /// Build a new exporter from configuration.
    ///
    /// The `token_provider` supplies OAuth bearer tokens used to authenticate
    /// to the Logs Ingestion API. It is resolved from the `bearer_token_provider`
    /// capability bound to this node (for example, by the `azure_identity_auth`
    /// extension).
    pub fn new(
        pipeline_ctx: PipelineContext,
        config: Config,
        token_provider: Box<dyn BearerTokenProvider>,
    ) -> Result<Self, Error> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| Error::Config(e.to_string()))?;

        // Register metrics with the telemetry system
        let metrics: AzureMonitorExporterMetricsRc = Rc::new(RefCell::new(
            super::metrics::AzureMonitorExporterMetricsTracker::register(&pipeline_ctx),
        ));

        // Create log transformer
        let transformer = Transformer::new(&config);

        // Create Gzip batcher
        let gzip_batcher = GzipBatcher::new(config.api.gzip_compression_level);

        // Create heartbeat handler
        let heartbeat = if config.heartbeat.enabled {
            Some(Heartbeat::new(&config.api, &config.heartbeat.overrides)?)
        } else {
            None
        };

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
            token_provider: Some(token_provider),
        })
    }

    /// Update all gauges (in-flight exports and state mappings).
    #[inline]
    fn sync_gauges(&self) {
        let mut m = self.metrics.borrow_mut();
        m.set_in_flight_exports(self.in_flight_exports.len() as u64);
        m.set_in_flight_log_records(self.in_flight_exports.queued_rows());
        m.set_state_mapping(
            super::metrics::StateMapping::BatchToMessage,
            self.state.batch_to_msg.len() as u64,
        );
        m.set_state_mapping(
            super::metrics::StateMapping::MessageToBatch,
            self.state.msg_to_batch.len() as u64,
        );
        m.set_state_mapping(
            super::metrics::StateMapping::MessageToData,
            self.state.msg_to_data.len() as u64,
        );
    }

    async fn finalize_export(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        auth: &mut BearerAuth,
        completed_export: CompletedExport,
    ) -> Result<(), EngineError> {
        let CompletedExport {
            batch_id,
            client,
            result,
            row_count,
            body_size_bytes,
            token_generation,
        } = completed_export;

        // Return the client to the pool
        self.client_pool.release(client);

        if result.as_ref().is_err_and(Error::is_unauthorized) {
            auth.invalidate(token_generation);
        }

        match result {
            Ok(duration) => {
                self.handle_export_success(
                    effect_handler,
                    batch_id,
                    row_count,
                    body_size_bytes,
                    duration,
                )
                .await
            }
            Err(e) => {
                self.handle_export_failure(effect_handler, batch_id, row_count, body_size_bytes, e)
                    .await
            }
        }
    }

    async fn handle_export_success(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        batch_id: u64,
        row_count: u64,
        body_size_bytes: u64,
        duration: std::time::Duration,
    ) -> Result<(), EngineError> {
        // Export succeeded - Ack only fully-completed messages
        let completed_messages = self.state.remove_batch_success(batch_id);
        {
            let mut m = self.metrics.borrow_mut();
            m.record_export(
                Outcome::Success,
                row_count,
                completed_messages.len() as u64,
                body_size_bytes,
            );
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
        row_count: u64,
        body_size_bytes: u64,
        error: Error,
    ) -> Result<(), EngineError> {
        // Export failed - Nack ALL messages in this batch, remove entirely
        let failed_messages = self.state.remove_batch_failure(batch_id);
        {
            let mut m = self.metrics.borrow_mut();
            m.record_export(
                Outcome::Failure,
                row_count,
                failed_messages.len() as u64,
                body_size_bytes,
            );
        }

        otel_warn!("azure_monitor_exporter.export.failed", batch_id = batch_id, error = %error);

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
        auth: &mut BearerAuth,
    ) -> Result<(), EngineError> {
        let pending_batch = match self.gzip_batcher.take_pending_batch() {
            Some(batch) => batch,
            None => return Ok(()), // No pending batch - nothing to do
        };

        self.metrics
            .borrow_mut()
            .add_batch_uncompressed_size(pending_batch.uncompressed_size as f64);
        self.metrics
            .borrow_mut()
            .add_batch_size(pending_batch.compressed_data.len() as f64);

        // Settle the completion that frees the slot before reading the token: a
        // 401 completion invalidates the cached header, and stamping this batch
        // first would send it with a credential already known to be rejected.
        if let Some(completed_export) = self.in_flight_exports.reap_if_at_capacity().await {
            self.finalize_export(effect_handler, auth, completed_export)
                .await?;
        }

        let Some((auth_header, token_generation)) = auth.header() else {
            let error = Error::NoBearerToken {
                reason: auth.not_ready_reason(),
            };
            return self
                .handle_export_failure(
                    effect_handler,
                    pending_batch.batch_id,
                    pending_batch.row_count,
                    pending_batch.compressed_data.len() as u64,
                    error,
                )
                .await;
        };

        let client = self.client_pool.take();
        self.in_flight_exports.push_export(
            client,
            pending_batch.batch_id,
            pending_batch.row_count,
            pending_batch.compressed_data,
            auth_header,
            token_generation,
        );

        self.last_batch_queued_at = tokio::time::Instant::now();

        Ok(())
    }

    async fn handle_logs(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        context: Context,
        payload: OtapPayload,
        log_entries: Vec<Bytes>,
        msg_id: u64,
        auth: &mut BearerAuth,
    ) -> Result<(), EngineError> {
        if context.may_return_payload() {
            self.state.add_msg_to_data(msg_id, context, payload);
        } else {
            self.state
                .add_msg_to_data(msg_id, context, OtapPayload::empty(SignalType::Logs));
        }

        for log_entry in log_entries {
            let entry_len = log_entry.len();
            match self.gzip_batcher.push(log_entry) {
                Ok(gzip_batcher::PushResult::Ok(batch_id)) => {
                    // current batch id is being associated with the current message
                    self.state.add_batch_msg_relationship(batch_id, msg_id);
                }
                Ok(gzip_batcher::PushResult::BatchReady(new_batch_id)) => {
                    // new batch id is being associated with the current message
                    self.state.add_batch_msg_relationship(new_batch_id, msg_id);
                    self.queue_pending_batch(effect_handler, auth).await?;
                }
                Ok(gzip_batcher::PushResult::TooLarge) => {
                    let error = Error::LogEntryTooLarge;
                    self.metrics.borrow_mut().add_log_entry_too_large();
                    otel_warn!(
                        "azure_monitor_exporter.message.log_entry_too_large",
                        msg_id = msg_id,
                        size_bytes = entry_len
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
        auth: &mut BearerAuth,
    ) -> Result<(), EngineError> {
        let completed_exports = self.in_flight_exports.drain().await;
        for completed_export in completed_exports {
            self.finalize_export(effect_handler, auth, completed_export)
                .await?;
        }
        Ok(())
    }

    async fn queue_current_batch(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        auth: &mut BearerAuth,
    ) -> Result<(), EngineError> {
        match self.gzip_batcher.finalize() {
            Ok(FinalizeResult::Ok) => {
                return self.queue_pending_batch(effect_handler, auth).await;
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
        auth: &mut BearerAuth,
    ) -> Result<(), EngineError> {
        if auth.is_ready() {
            self.queue_current_batch(effect_handler, auth).await?;
        }
        self.drain_in_flight_exports(effect_handler, auth).await?;

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

    async fn handle_message(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        msg: Result<Message<OtapPdata>, RecvError>,
        msg_id: &mut u64,
        auth: &mut BearerAuth,
    ) -> Result<(), EngineError> {
        match msg {
            Ok(Message::PData(pdata)) => {
                if !auth.is_ready() {
                    effect_handler
                        .notify_nack(NackMsg::new(auth.not_ready_reason(), pdata))
                        .await?;
                    return Ok(());
                }
                *msg_id += 1;
                let (context, payload) = pdata.into_parts();

                let log_entries = match &payload {
                    OtapPayload::OtapArrowRecords(otap_records) => match otap_records {
                        OtapArrowRecords::Logs(_) => {
                            let logs_view = OtapLogsView::try_from(otap_records).map_err(|e| {
                                let error = Error::LogsViewCreationFailed { source: e };
                                EngineError::InternalError {
                                    message: error.to_string(),
                                }
                            })?;
                            Some(self.transformer.convert_to_log_analytics(&logs_view))
                        }
                        OtapArrowRecords::Metrics(_) | OtapArrowRecords::Traces(_) => {
                            otel_warn!(
                                "azure_monitor_exporter.message.unsupported_signal",
                                signal = "metrics_or_traces",
                                format = "otap_arrow"
                            );
                            None
                        }
                    },
                    OtapPayload::OtlpBytes(otlp_bytes) => match otlp_bytes {
                        OtlpProtoBytes::ExportLogsRequest(bytes) => {
                            let logs_view = RawLogsData::new(bytes.as_ref());
                            Some(self.transformer.convert_to_log_analytics(&logs_view))
                        }
                        OtlpProtoBytes::ExportMetricsRequest(_)
                        | OtlpProtoBytes::ExportTracesRequest(_) => {
                            otel_warn!(
                                "azure_monitor_exporter.message.unsupported_signal",
                                signal = "metrics_or_traces",
                                format = "otlp_proto"
                            );
                            None
                        }
                    },
                };

                if let Some(log_entries) = log_entries {
                    self.handle_logs(effect_handler, context, payload, log_entries, *msg_id, auth)
                        .await?;
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
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        otel_info!(
            "azure_monitor_exporter.start",
            endpoint = self.config.api.dcr_endpoint.as_str(),
            stream = self.config.api.stream_name.as_str(),
            dcr = self.config.api.dcr.as_str(),
            gzip_compression_level = self.config.api.gzip_compression_level
        );

        let mut msg_id = 0;

        let mut auth = BearerAuth::new(
            self.token_provider
                .take()
                .expect("bearer token provider is present before startup"),
            AZURE_MONITOR_BEARER_AUTH_EVENTS,
        );

        self.client_pool
            .initialize(&self.config.api)
            .await
            .map_err(|e| {
                let error = Error::ClientPoolInit(Box::new(e));
                EngineError::InternalError {
                    message: error.to_string(),
                }
            })?;

        let mut next_periodic_export = tokio::time::Instant::now()
            + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL);
        let mut next_heartbeat_send = tokio::time::Instant::now();

        let margin_sleep = tokio::time::sleep_until(tokio::time::Instant::now());
        tokio::pin!(margin_sleep);
        let mut armed_margin_deadline: Option<std::time::Instant> = None;

        loop {
            let has_token = auth.is_ready();
            let at_capacity = self.in_flight_exports.len() >= MAX_IN_FLIGHT_EXPORTS;
            let accepting_pdata = has_token && !at_capacity;

            let token_margin_deadline = auth.refresh_deadline();
            if token_margin_deadline != armed_margin_deadline {
                if let Some(deadline) = token_margin_deadline {
                    margin_sleep
                        .as_mut()
                        .reset(tokio::time::Instant::from_std(deadline));
                }
                armed_margin_deadline = token_margin_deadline;
            }

            tokio::select! {
                biased;

                () = &mut margin_sleep, if token_margin_deadline.is_some() => {
                    continue;
                }

                () = auth.poll_refresh(), if auth.is_active() => {
                    continue;
                }

                _ = tokio::time::sleep_until(next_heartbeat_send), if has_token && self.heartbeat.is_some() => {
                    next_heartbeat_send = tokio::time::Instant::now() + self.config.heartbeat.frequency;
                    if let Some(ref mut hb) = self.heartbeat {
                        let (header, generation) = auth
                            .header()
                            .expect("heartbeat is gated on a usable bearer token");
                        hb.update_auth(header);
                        match hb.send().await {
                            Ok(_) => {
                                self.metrics.borrow_mut().record_heartbeat(Outcome::Success);
                                otel_debug!("azure_monitor_exporter.heartbeat.sent");
                            }
                            Err(e) => {
                                if e.is_unauthorized() {
                                    auth.invalidate(generation);
                                }
                                self.metrics.borrow_mut().record_heartbeat(Outcome::Failure);
                                otel_warn!("azure_monitor_exporter.heartbeat.send_failed", error = %e);
                            }
                        }
                    }
                }

                completed = self.in_flight_exports.next_completion() => {
                    if let Some(completed_export) = completed {
                        self.finalize_export(&effect_handler, &mut auth, completed_export).await?;
                    }
                }

                _ = tokio::time::sleep_until(next_periodic_export), if accepting_pdata => {
                    next_periodic_export = tokio::time::Instant::now() + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL);

                    if self.last_batch_queued_at.elapsed() >= std::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL) && self.gzip_batcher.has_pending_data() {
                        otel_debug!("azure_monitor_exporter.export.periodic_flush");
                        self.queue_current_batch(&effect_handler, &mut auth).await?;
                    }
                }

                // TODO: Ensure that when rejecting pdata, data loss doesn't occur. (pending on lquerel's msg channel rework)
                // Control always flows; pdata guarded by has_token && !at_capacity
                msg = msg_chan.recv_when(accepting_pdata) => {
                    match msg {
                        Ok(Message::Control(NodeControlMsg::CollectTelemetry { mut metrics_reporter })) => {
                            self.sync_gauges();
                            if tracing::enabled!(tracing::Level::DEBUG) {
                                let m = self.metrics.borrow();
                                let cl = m.http_for(HttpResponse::Http2xx).latency.get();
                                let bs = m.batch_size();
                                otel_debug!(
                                    "azure_monitor_exporter.metrics.collect",
                                    successful_items = m.export_for(Outcome::Success).items.get(),
                                    successful_batches = m.export_for(Outcome::Success).batches.get(),
                                    successful_messages = m.export_for(Outcome::Success).messages.get(),
                                    failed_items = m.export_for(Outcome::Failure).items.get(),
                                    failed_batches = m.export_for(Outcome::Failure).batches.get(),
                                    failed_messages = m.export_for(Outcome::Failure).messages.get(),
                                    client_success_latency_avg_ms = if cl.count > 0 { cl.sum / cl.count as f64 } else { 0.0 },
                                    client_success_latency_min_ms = if cl.count > 0 { cl.min } else { 0.0 },
                                    client_success_latency_max_ms = if cl.count > 0 { cl.max } else { 0.0 },
                                    client_success_latency_count = cl.count,
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
                            self.handle_shutdown(&effect_handler, &mut auth).await?;
                            let snapshots = self.metrics.borrow_mut().terminal_snapshots();
                            return Ok(TerminalState::new(
                                deadline,
                                snapshots,
                            ));
                        }
                        other => {
                            self.handle_message(&effect_handler, other, &mut msg_id, &mut auth).await?;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::{ApiConfig, HeartbeatConfig, SchemaConfig};
    use super::*;
    use bytes::Bytes;
    use futures::StreamExt;
    use http::StatusCode;
    use http::header::HeaderValue;
    use otap_df_channel::mpsc;
    use otap_df_engine::Interests;
    use otap_df_engine::capability::CapabilityError;
    use otap_df_engine::capability::auth::BearerToken;
    use otap_df_engine::capability::auth::bearer_token_provider::TokenStream;
    use otap_df_engine::context::{ControllerContext, PipelineContext};
    use otap_df_engine::local::exporter::EffectHandler;
    use otap_df_engine::local::message::LocalReceiver;
    use otap_df_engine::message::Receiver;
    use otap_df_engine::node::NodeId;
    use otap_df_otap::pdata::Context;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use rand::{RngExt, SeedableRng, rngs::SmallRng};
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Test double for the `BearerTokenProvider` capability. Yields a single
    /// non-expiring token and then ends the stream.
    struct MockTokenProvider;

    #[async_trait(?Send)]
    impl BearerTokenProvider for MockTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            Ok(BearerToken::without_expiry("test-token".to_owned()))
        }

        fn token_stream(&self) -> TokenStream {
            futures::stream::once(async { BearerToken::without_expiry("test-token".to_owned()) })
                .boxed()
        }
    }

    fn create_test_pipeline_ctx() -> PipelineContext {
        otap_df_otap::crypto::ensure_crypto_provider();
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
                azure_monitor_source_resourceid: None,
                gzip_compression_level: 6,
                user_agent: None,
            },
            heartbeat: HeartbeatConfig::default(),
        }
    }

    fn make_msg_channel(
        capacity: usize,
    ) -> (
        mpsc::Sender<NodeControlMsg<OtapPdata>>,
        mpsc::Sender<OtapPdata>,
        ExporterInbox<OtapPdata>,
    ) {
        let (control_tx, control_rx) = mpsc::Channel::<NodeControlMsg<OtapPdata>>::new(capacity);
        let (pdata_tx, pdata_rx) = mpsc::Channel::<OtapPdata>::new(capacity);
        (
            control_tx,
            pdata_tx,
            ExporterInbox::new(
                Receiver::Local(LocalReceiver::mpsc(control_rx)),
                Receiver::Local(LocalReceiver::mpsc(pdata_rx)),
                0,
                Interests::empty(),
            ),
        )
    }

    #[test]
    fn test_new_validates_config() {
        let config = create_test_config();
        let pipeline_ctx = create_test_pipeline_ctx();
        let _ =
            AzureMonitorExporter::new(pipeline_ctx, config, Box::new(MockTokenProvider)).unwrap();
    }

    fn test_effect_handler() -> EffectHandler<OtapPdata> {
        let (_, reporter) = MetricsReporter::create_new_and_receiver(10);
        EffectHandler::new(
            NodeId {
                index: 0,
                name: "test_exporter".to_string().into(),
            },
            reporter,
        )
    }

    /// Build an exporter whose client pool targets `endpoint`, as `start` would.
    async fn exporter_targeting(endpoint: String) -> AzureMonitorExporter {
        let mut config = create_test_config();
        config.api.dcr_endpoint = endpoint;
        let mut exporter = AzureMonitorExporter::new(
            create_test_pipeline_ctx(),
            config,
            Box::new(MockTokenProvider),
        )
        .unwrap();
        exporter
            .client_pool
            .initialize(&exporter.config.api)
            .await
            .unwrap();
        exporter
    }

    async fn auth_with_cached_token() -> BearerAuth {
        let mut auth = BearerAuth::new(
            Box::new(MockTokenProvider),
            AZURE_MONITOR_BEARER_AUTH_EVENTS,
        );
        auth.poll_refresh().await;
        assert!(auth.is_ready());
        auth
    }

    /// Scenario: A completed export succeeds with a known compressed request-body size.
    /// Guarantees: The successful outcome records the resolved request-body bytes.
    #[tokio::test]
    async fn test_handle_export_success() {
        let config = create_test_config();
        let pipeline_ctx = create_test_pipeline_ctx();
        let mut exporter =
            AzureMonitorExporter::new(pipeline_ctx, config, Box::new(MockTokenProvider)).unwrap();

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
            .handle_export_success(&effect_handler, batch_id, 10, 1_024, Duration::from_secs(1))
            .await;

        // Verify stats
        let m = exporter.metrics.borrow();
        let success = m.export_for(Outcome::Success);
        assert_eq!(success.batches.get(), 1);
        assert_eq!(success.messages.get(), 1);
        assert_eq!(success.items.get(), 10);
        assert_eq!(success.bytes.get(), 1_024);
        drop(m);

        // Verify state cleared
        assert!(exporter.state.batch_to_msg.is_empty());
        assert!(exporter.state.msg_to_data.is_empty());
    }

    /// Scenario: A completed export fails with a known compressed request-body size.
    /// Guarantees: The failed outcome records the resolved request-body bytes.
    #[tokio::test]
    async fn test_handle_export_failure() {
        let config = create_test_config();
        let pipeline_ctx = create_test_pipeline_ctx();
        let mut exporter =
            AzureMonitorExporter::new(pipeline_ctx, config, Box::new(MockTokenProvider)).unwrap();

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
            .handle_export_failure(&effect_handler, batch_id, 10, 512, error)
            .await;

        // Verify stats
        let m = exporter.metrics.borrow();
        let failure = m.export_for(Outcome::Failure);
        assert_eq!(failure.batches.get(), 1);
        assert_eq!(failure.messages.get(), 1);
        assert_eq!(failure.items.get(), 10);
        assert_eq!(failure.bytes.get(), 512);
        drop(m);

        // Verify state cleared
        assert!(exporter.state.batch_to_msg.is_empty());
        assert!(exporter.state.msg_to_data.is_empty());
    }

    /// Scenario: Azure Monitor returns HTTP 401 for an export stamped with the
    /// currently cached bearer-token generation.
    /// Guarantees: completion handling invalidates that generation so the exporter
    /// stops accepting pdata until the provider publishes a replacement token.
    #[tokio::test]
    async fn unauthorized_completion_invalidates_the_used_token() {
        let config = create_test_config();
        let pipeline_ctx = create_test_pipeline_ctx();
        let mut exporter =
            AzureMonitorExporter::new(pipeline_ctx, config, Box::new(MockTokenProvider)).unwrap();
        let mut auth = BearerAuth::new(
            Box::new(MockTokenProvider),
            AZURE_MONITOR_BEARER_AUTH_EVENTS,
        );
        auth.poll_refresh().await;
        let (_, token_generation) = auth.header().expect("mock provider publishes a token");

        let (_, reporter) = MetricsReporter::create_new_and_receiver(10);
        let effect_handler = EffectHandler::new(
            NodeId {
                index: 0,
                name: "test_exporter".to_string().into(),
            },
            reporter,
        );
        let client = super::super::client::LogsIngestionClient::from_parts(
            reqwest::Client::new(),
            "http://localhost".to_string(),
            exporter.metrics.clone(),
        );
        let completed = CompletedExport {
            batch_id: 1,
            client,
            result: Err(Error::unauthorized("rejected".to_string())),
            row_count: 1,
            body_size_bytes: 1,
            token_generation,
        };

        exporter
            .finalize_export(&effect_handler, &mut auth, completed)
            .await
            .unwrap();

        assert!(!auth.is_ready());
    }

    /// Queue a one-entry batch so the next `queue_pending_batch` has work to do.
    fn prime_pending_batch(exporter: &mut AzureMonitorExporter) {
        let _ = exporter
            .gzip_batcher
            .push(Bytes::from_static(br#"{"Message":"x"}"#))
            .unwrap();
        let _ = exporter.gzip_batcher.finalize().unwrap();
    }

    /// Scenario: the in-flight set is full and the export that frees the slot
    /// comes back 401, while the message being processed still has batches to
    /// queue.
    /// Guarantees: the rejected token is settled before the next batch is
    /// stamped, and that batch is failed rather than dispatched with the dead
    /// credential or aborting the node.
    #[tokio::test]
    async fn a_401_freeing_a_slot_fails_the_next_batch_instead_of_stamping_it() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_string("token expired"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut exporter = exporter_targeting(mock_server.uri()).await;
        exporter.in_flight_exports = InFlightExports::new(1);
        let mut auth = auth_with_cached_token().await;
        let effect_handler = test_effect_handler();

        // Fill the single slot. The token is still good, so this batch is stamped
        // and dispatched.
        prime_pending_batch(&mut exporter);
        exporter
            .queue_pending_batch(&effect_handler, &mut auth)
            .await
            .unwrap();
        assert!(auth.is_ready());
        assert_eq!(exporter.in_flight_exports.len(), 1);

        // At capacity: the reaped 401 invalidates the cached token, so the batch
        // that was waiting on the slot must not be dispatched.
        prime_pending_batch(&mut exporter);
        exporter.state.add_batch_msg_relationship(2, 100);
        exporter
            .queue_pending_batch(&effect_handler, &mut auth)
            .await
            .unwrap();

        assert!(!auth.is_ready(), "the 401 must invalidate the used token");
        assert_eq!(
            exporter.in_flight_exports.len(),
            0,
            "no export may be stamped with the rejected token"
        );
        assert!(
            exporter.state.batch_to_msg.is_empty(),
            "the undispatchable batch must be failed, not stranded"
        );
        assert_eq!(
            exporter
                .metrics
                .borrow()
                .export_for(Outcome::Failure)
                .batches
                .get(),
            2
        );
    }

    /// Scenario: pdata arrives before the bearer token provider has published a
    /// usable token.
    /// Guarantees: the message is refused instead of buffered, so the exporter
    /// never batches records it cannot authenticate.
    #[tokio::test]
    async fn pdata_is_refused_while_no_bearer_token_is_cached() {
        let mut exporter = exporter_targeting("http://localhost".to_string()).await;
        let mut auth = BearerAuth::new(
            Box::new(MockTokenProvider),
            AZURE_MONITOR_BEARER_AUTH_EVENTS,
        );
        assert!(!auth.is_ready(), "no token has been polled yet");
        assert!(!auth.not_ready_reason().is_empty());

        let mut msg_id = 0;
        exporter
            .handle_message(
                &test_effect_handler(),
                Ok(Message::PData(OtapPdata::new(
                    Context::default(),
                    OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(Bytes::new())),
                ))),
                &mut msg_id,
                &mut auth,
            )
            .await
            .unwrap();

        assert_eq!(msg_id, 0, "refused pdata must not consume a message id");
        assert!(exporter.state.msg_to_data.is_empty());
    }

    /// Scenario: a logs message arrives once a bearer token is cached, carrying
    /// no convertible log records.
    /// Guarantees: it is admitted for processing rather than refused for auth
    /// reasons, and is then released instead of being retained by the exporter.
    #[tokio::test]
    async fn logs_are_admitted_once_a_bearer_token_is_cached() {
        let mut exporter = exporter_targeting("http://localhost".to_string()).await;
        let mut auth = auth_with_cached_token().await;

        let mut msg_id = 0;
        exporter
            .handle_message(
                &test_effect_handler(),
                Ok(Message::PData(OtapPdata::new(
                    Context::default(),
                    OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(Bytes::new())),
                ))),
                &mut msg_id,
                &mut auth,
            )
            .await
            .unwrap();

        assert_eq!(msg_id, 1, "admitted pdata consumes a message id");
        assert!(exporter.state.msg_to_data.is_empty());
    }

    /// Scenario: buffered log entries are flushed by shutdown while a bearer
    /// token is cached.
    /// Guarantees: the batch reaches the ingestion endpoint carrying that token,
    /// and the drained export is accounted as a success.
    #[tokio::test]
    async fn shutdown_exports_buffered_logs_with_the_cached_token() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut exporter = exporter_targeting(mock_server.uri()).await;
        let mut auth = auth_with_cached_token().await;
        let effect_handler = test_effect_handler();

        exporter
            .handle_logs(
                &effect_handler,
                Context::default(),
                OtapPayload::empty(SignalType::Logs),
                vec![Bytes::from_static(br#"{"Message":"hello"}"#)],
                1,
                &mut auth,
            )
            .await
            .unwrap();
        assert_eq!(exporter.state.msg_to_data.len(), 1);

        exporter
            .handle_shutdown(&effect_handler, &mut auth)
            .await
            .unwrap();

        assert_eq!(exporter.in_flight_exports.len(), 0);
        assert!(exporter.state.msg_to_data.is_empty());
        let m = exporter.metrics.borrow();
        assert_eq!(m.export_for(Outcome::Success).batches.get(), 1);
        assert_eq!(m.export_for(Outcome::Failure).batches.get(), 0);
    }

    /// Scenario: a single logs message carries enough records to fill a batch.
    /// Guarantees: the full batch is dispatched as soon as it is ready, rather
    /// than waiting for shutdown or the periodic flush.
    #[tokio::test]
    async fn a_full_batch_is_dispatched_as_soon_as_it_is_ready() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut exporter = exporter_targeting(mock_server.uri()).await;
        let mut auth = auth_with_cached_token().await;
        let effect_handler = test_effect_handler();

        // Entries must be distinct: identical ones compress too well to reach
        // the compressed batch limit.
        let mut rng = SmallRng::seed_from_u64(7);
        let entries: Vec<Bytes> = (0..2_500)
            .map(|_| {
                let msg: String = (0..1_024)
                    .map(|_| rng.random_range(b'a'..=b'z') as char)
                    .collect();
                Bytes::from(format!(r#"{{"Message":"{msg}"}}"#))
            })
            .collect();

        exporter
            .handle_logs(
                &effect_handler,
                Context::default(),
                OtapPayload::empty(SignalType::Logs),
                entries,
                1,
                &mut auth,
            )
            .await
            .unwrap();

        assert_eq!(
            exporter.in_flight_exports.len(),
            1,
            "a ready batch is dispatched without waiting for shutdown"
        );

        exporter
            .drain_in_flight_exports(&effect_handler, &mut auth)
            .await
            .unwrap();
        assert_eq!(
            exporter
                .metrics
                .borrow()
                .export_for(Outcome::Success)
                .batches
                .get(),
            1
        );
    }

    /// Scenario: shutdown arrives while no bearer token is cached.
    /// Guarantees: the exporter skips the flush it cannot authenticate and still
    /// releases every buffered message rather than dropping it silently.
    #[tokio::test]
    async fn shutdown_without_a_token_releases_buffered_messages() {
        let mut exporter = exporter_targeting("http://localhost".to_string()).await;
        let mut auth = BearerAuth::new(
            Box::new(MockTokenProvider),
            AZURE_MONITOR_BEARER_AUTH_EVENTS,
        );
        assert!(!auth.is_ready(), "no token has been polled yet");

        exporter
            .state
            .add_msg_to_data(7, Context::default(), OtapPayload::empty(SignalType::Logs));

        exporter
            .handle_shutdown(&test_effect_handler(), &mut auth)
            .await
            .unwrap();

        assert!(exporter.state.msg_to_data.is_empty());
    }

    /// Scenario: the bearer-auth adapter reports an unusable token and a closed
    /// token stream.
    /// Guarantees: both hooks are wired to Azure Monitor's event namespace and
    /// can be raised without panicking.
    #[test]
    fn bearer_auth_events_are_reportable() {
        let invalid = HeaderValue::from_str("\n").expect_err("control chars are invalid");
        (AZURE_MONITOR_BEARER_AUTH_EVENTS.invalid_token)(&invalid);
        (AZURE_MONITOR_BEARER_AUTH_EVENTS.token_stream_closed)();
    }

    // Azure Monitor can temporarily stop accepting new pdata while it is at
    // capacity. Once shutdown is latched, the exporter channel must still drain
    // already buffered pdata before delivering the final Shutdown message.
    #[tokio::test]
    async fn test_shutdown_drains_buffered_pdata_while_at_capacity() {
        let (control_tx, pdata_tx, mut msg_chan) = make_msg_channel(8);
        let at_capacity = true;

        pdata_tx
            .send_async(OtapPdata::new(
                Context::default(),
                OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(Bytes::new())),
            ))
            .await
            .unwrap();

        control_tx
            .send_async(NodeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_millis(200),
                reason: "test".to_owned(),
            })
            .await
            .unwrap();

        control_tx
            .send_async(NodeControlMsg::TimerTick {})
            .await
            .unwrap();

        let msg = msg_chan.recv_when(!at_capacity).await.unwrap();
        assert!(matches!(
            msg,
            Message::Control(NodeControlMsg::TimerTick {})
        ));

        let msg = msg_chan.recv_when(!at_capacity).await.unwrap();
        assert!(matches!(msg, Message::PData(_)));

        drop(pdata_tx);

        let msg = msg_chan.recv_when(!at_capacity).await.unwrap();
        assert!(matches!(
            msg,
            Message::Control(NodeControlMsg::Shutdown { .. })
        ));
    }
}
