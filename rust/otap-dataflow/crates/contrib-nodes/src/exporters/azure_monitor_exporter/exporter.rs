// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_config::SignalType;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::capability::bearer_token_provider::BearerToken;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::capability::bearer_token_provider::BearerTokenProvider;
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
use super::metrics::{AzureMonitorExporterMetrics, AzureMonitorExporterMetricsRc};
use super::state::AzureMonitorExporterState;
use super::transformer::Transformer;
use futures::StreamExt;
use otap_df_otap::pdata::{Context, OtapPdata};
use reqwest::header::HeaderValue;

use otap_df_telemetry::{otel_debug, otel_error, otel_info, otel_warn};

use bytes::Bytes;
use std::cell::RefCell;
use std::rc::Rc;

/// Max concurrent HTTP requests in flight to the Logs Ingestion API.
const MAX_IN_FLIGHT_EXPORTS: usize = 16;
const PERIODIC_EXPORT_INTERVAL: u64 = 3;

/// Safety margin before actual token expiry within which the exporter stops
/// accepting new pdata. Kept small and aligned with the extension's own
/// usability margin (`TOKEN_USABLE_MARGIN_SECS`): the bound
/// `BearerTokenProvider` extension refreshes the token well ahead of expiry, so
/// under healthy operation this margin is never reached. It only gates data
/// acceptance in the degraded case where a refresh is failing and the cached
/// token is genuinely about to expire; until then a still-valid token keeps
/// being served, matching the provider's "serve valid tokens near expiry"
/// behavior.
const TOKEN_USABLE_MARGIN_SECS: u64 = 30;

/// The exporter's view of the current bearer token's remaining lifetime.
///
/// Models the three states explicitly instead of encoding them in a single
/// `Instant` with a sentinel: no token yet, a non-expiring token, or a token
/// with a known expiry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenExpiry {
    /// No usable token yet (before the first token arrives).
    None,
    /// A token with no known expiry; valid until replaced.
    NeverExpires,
    /// A token that expires at the given monotonic instant.
    At(tokio::time::Instant),
}

impl TokenExpiry {
    /// Derives the expiry state from a provider token. A token without a known
    /// `expires_on` is treated as non-expiring ("valid until replaced").
    #[inline]
    fn from_token(token: &BearerToken) -> Self {
        match token.expires_on() {
            Some(expires_on) => Self::At(tokio::time::Instant::from_std(expires_on)),
            None => Self::NeverExpires,
        }
    }

    /// Returns whether the token is still usable at `now`. An expiring token is
    /// usable only while more than `margin` remains before expiry, so a request
    /// started now still completes against a valid token; otherwise the exporter
    /// stops accepting pdata and back-pressures.
    #[inline]
    fn is_usable(self, now: tokio::time::Instant, margin: std::time::Duration) -> bool {
        match self {
            Self::None => false,
            Self::NeverExpires => true,
            Self::At(expiry) => expiry > now + margin,
        }
    }
}

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
    token_provider: Box<dyn BearerTokenProvider>,
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
        let metric_set = pipeline_ctx.register_metrics::<AzureMonitorExporterMetrics>();
        let metrics: AzureMonitorExporterMetricsRc = Rc::new(RefCell::new(
            super::metrics::AzureMonitorExporterMetricsTracker::new(metric_set),
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
            token_provider,
        })
    }

    /// Update all gauges (in-flight exports + state map sizes).
    #[inline]
    fn sync_gauges(&self) {
        let mut m = self.metrics.borrow_mut();
        m.set_in_flight_exports(self.in_flight_exports.len() as u64);
        m.set_in_flight_log_records(self.in_flight_exports.queued_rows());
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
        row_count: u64,
        duration: std::time::Duration,
    ) -> Result<(), EngineError> {
        // Export succeeded - Ack only fully-completed messages
        let completed_messages = self.state.remove_batch_success(batch_id);
        {
            let mut m = self.metrics.borrow_mut();
            m.add_messages(completed_messages.len() as u64);
            m.add_rows(row_count);
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
        row_count: u64,
        error: Error,
    ) -> Result<(), EngineError> {
        // Export failed - Nack ALL messages in this batch, remove entirely
        let failed_messages = self.state.remove_batch_failure(batch_id);
        {
            let mut m = self.metrics.borrow_mut();
            m.add_failed_messages(failed_messages.len() as u64);
            m.add_failed_rows(row_count);
            m.add_failed_batch();
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

    async fn handle_logs(
        &mut self,
        effect_handler: &EffectHandler<OtapPdata>,
        context: Context,
        payload: OtapPayload,
        log_entries: Vec<Bytes>,
        msg_id: u64,
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
                    self.queue_pending_batch(effect_handler).await?;
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

                let log_entries = match &payload {
                    OtapPayload::OtapArrowRecords(otap_records) => match otap_records {
                        OtapArrowRecords::Logs(_) => {
                            let mut otap_records = otap_records.clone();
                            otap_records.decode_transport_optimized_ids().map_err(|e| {
                                let error = Error::LogsViewCreationFailed { source: e };
                                EngineError::InternalError {
                                    message: error.to_string(),
                                }
                            })?;
                            let logs_view = OtapLogsView::try_from(&otap_records).map_err(|e| {
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
                    self.handle_logs(effect_handler, context, payload, log_entries, *msg_id)
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

        // Subscribe to bearer tokens from the bound provider capability. The
        // stream yields the current token immediately and then a new value each
        // time the provider refreshes it. Credential acquisition and refresh
        // scheduling are owned by the provider extension.
        let mut token_stream = self.token_provider.token_stream();
        let mut token_stream_active = true;

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

        // pdata is not accepted until a usable token arrives. Starts as `None`
        // so `has_token` is false until the first token is published.
        let mut token_expiry = TokenExpiry::None;

        loop {
            // We have a valid token as long as it won't expire within the
            // usability safety margin.
            let has_token = token_expiry.is_usable(
                tokio::time::Instant::now(),
                tokio::time::Duration::from_secs(TOKEN_USABLE_MARGIN_SECS),
            );
            let at_capacity = self.in_flight_exports.len() >= MAX_IN_FLIGHT_EXPORTS;
            let accepting_pdata = has_token && !at_capacity;

            tokio::select! {
                biased;

                // Receive bearer tokens (initial + refreshes) from the provider.
                maybe_token = token_stream.next(), if token_stream_active => {
                    match maybe_token {
                        Some(token) => {
                            match HeaderValue::from_str(&format!("Bearer {}", token.expose_token())) {
                                Ok(header) => {
                                    self.client_pool.update_auth(header.clone());
                                    if let Some(ref mut hb) = self.heartbeat {
                                        hb.update_auth(header.clone());
                                    }

                                    token_expiry = TokenExpiry::from_token(&token);

                                    otel_info!("azure_monitor_exporter.auth.token_acquired");
                                }
                                Err(e) => {
                                    otel_error!("azure_monitor_exporter.auth.header_creation_failed", error = ?e);
                                }
                            }
                        }
                        None => {
                            // Provider closed the token stream; no further refreshes
                            // will arrive. Stop polling it.
                            token_stream_active = false;
                            otel_warn!("azure_monitor_exporter.auth.token_stream_ended");
                        }
                    }
                }

                _ = tokio::time::sleep_until(next_heartbeat_send), if has_token && self.heartbeat.is_some() => {
                    next_heartbeat_send = tokio::time::Instant::now() + self.config.heartbeat.frequency;
                    self.metrics.borrow_mut().add_heartbeat();
                    if let Some(ref mut hb) = self.heartbeat {
                        match hb.send().await {
                            Ok(_) => otel_debug!("azure_monitor_exporter.heartbeat.sent"),
                            Err(e) => otel_warn!("azure_monitor_exporter.heartbeat.send_failed", error = %e),
                        }
                    }
                }

                completed = self.in_flight_exports.next_completion() => {
                    if let Some(completed_export) = completed {
                        self.finalize_export(&effect_handler, completed_export).await?;
                    }
                }

                _ = tokio::time::sleep_until(next_periodic_export), if accepting_pdata => {
                    next_periodic_export = tokio::time::Instant::now() + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL);

                    if self.last_batch_queued_at.elapsed() >= std::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL) && self.gzip_batcher.has_pending_data() {
                        otel_debug!("azure_monitor_exporter.export.periodic_flush");
                        self.queue_current_batch(&effect_handler).await?;
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
                                let cl = m.client_success_latency();
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
    use super::super::config::{ApiConfig, HeartbeatConfig, SchemaConfig};
    use super::*;
    use bytes::Bytes;
    use futures::StreamExt;
    use http::StatusCode;
    use otap_df_channel::mpsc;
    use otap_df_engine::Interests;
    use otap_df_engine::capability::CapabilityError;
    use otap_df_engine::capability::bearer_token_provider::{BearerToken, TokenStream};
    use otap_df_engine::context::{ControllerContext, PipelineContext};
    use otap_df_engine::local::exporter::EffectHandler;
    use otap_df_engine::local::message::LocalReceiver;
    use otap_df_engine::message::Receiver;
    use otap_df_engine::node::NodeId;
    use otap_df_otap::pdata::Context;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    /// Test double for the `BearerTokenProvider` capability. Yields a single
    /// non-expiring token and then ends the stream.
    struct MockTokenProvider;

    #[async_trait(?Send)]
    impl BearerTokenProvider for MockTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            Ok(BearerToken::new("test-token".to_owned(), None))
        }

        fn token_stream(&self) -> TokenStream {
            futures::stream::once(async { BearerToken::new("test-token".to_owned(), None) }).boxed()
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
            .handle_export_success(&effect_handler, batch_id, 10, Duration::from_secs(1))
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
            .handle_export_failure(&effect_handler, batch_id, 10, error)
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

    // ==================== Token usability logic ====================

    #[test]
    fn token_usable_when_expiry_beyond_margin() {
        let now = tokio::time::Instant::now();
        let margin = Duration::from_secs(TOKEN_USABLE_MARGIN_SECS);
        let expiry = TokenExpiry::At(now + Duration::from_secs(TOKEN_USABLE_MARGIN_SECS + 60));
        assert!(expiry.is_usable(now, margin));
    }

    #[test]
    fn token_unusable_within_margin() {
        // A token that expires inside the margin must stop pdata acceptance so
        // an in-flight request can't outlive the token.
        let now = tokio::time::Instant::now();
        let margin = Duration::from_secs(TOKEN_USABLE_MARGIN_SECS);
        let expiry = TokenExpiry::At(now + Duration::from_secs(TOKEN_USABLE_MARGIN_SECS - 5));
        assert!(!expiry.is_usable(now, margin));
    }

    #[test]
    fn token_unusable_at_exact_margin_boundary() {
        // `expiry == now + margin`: the strictly-greater check treats the exact
        // boundary as not usable.
        let now = tokio::time::Instant::now();
        let margin = Duration::from_secs(TOKEN_USABLE_MARGIN_SECS);
        assert!(!TokenExpiry::At(now + margin).is_usable(now, margin));
    }

    #[test]
    fn token_unusable_when_already_expired() {
        let now = tokio::time::Instant::now();
        let margin = Duration::from_secs(TOKEN_USABLE_MARGIN_SECS);
        let expiry = TokenExpiry::At(now - Duration::from_secs(1));
        assert!(!expiry.is_usable(now, margin));
    }

    #[test]
    fn startup_state_gates_pdata() {
        // The loop initializes `token_expiry = TokenExpiry::None`, which must
        // read as "no usable token" so pdata is gated until the first token
        // arrives.
        let now = tokio::time::Instant::now();
        let margin = Duration::from_secs(TOKEN_USABLE_MARGIN_SECS);
        assert!(!TokenExpiry::None.is_usable(now, margin));
    }

    #[test]
    fn expiry_uses_token_expiry_when_present() {
        let expires_on = Instant::now() + Duration::from_secs(3600);
        let token = BearerToken::new("secret".to_owned(), Some(expires_on));
        assert_eq!(
            TokenExpiry::from_token(&token),
            TokenExpiry::At(tokio::time::Instant::from_std(expires_on))
        );
    }

    #[test]
    fn non_expiring_token_never_expires_and_stays_usable() {
        let now = tokio::time::Instant::now();
        let margin = Duration::from_secs(TOKEN_USABLE_MARGIN_SECS);
        let token = BearerToken::new("secret".to_owned(), None);
        let expiry = TokenExpiry::from_token(&token);
        assert_eq!(expiry, TokenExpiry::NeverExpires);
        assert!(expiry.is_usable(now, margin));
    }
}
