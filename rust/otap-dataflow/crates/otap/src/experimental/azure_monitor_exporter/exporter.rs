// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::max;

use async_trait::async_trait;
use bytes::Bytes;
use futures::FutureExt;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_channel::error::RecvError;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::ConsumerEffectHandlerExtension;  // Add this import
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use prost::Message as _;

use super::client::{LogsIngestionClient, LogsIngestionClientPool};
use super::config::Config;
use super::gzip_batcher::{self, GzipBatcher};
use super::transformer::Transformer;
use super::state::AzureMonitorExporterState;
use crate::experimental::azure_monitor_exporter::gzip_batcher::FlushResult;
use crate::pdata::{Context, OtapPdata};
use super::in_flight_exports::{InFlightExports, CompletedExport};

const MAX_IN_FLIGHT_EXPORTS: usize = 32;
const PERIODIC_EXPORT_INTERVAL: u64 = 3;

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
}

impl AzureMonitorExporter
{
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
            client_pool: LogsIngestionClientPool::new(MAX_IN_FLIGHT_EXPORTS),
            in_flight_exports: InFlightExports::new(MAX_IN_FLIGHT_EXPORTS),
            last_batch_queued_at: tokio::time::Instant::now(),
        })
    }

    async fn finalize_export(&mut self, effect_handler: &EffectHandler<OtapPdata>, completed_export: CompletedExport) -> Result<(), Error> {
        let CompletedExport {
            batch_id,
            client,
            result,
            row_count,
        } = completed_export;

        // Return the client to the pool
        self.client_pool.release(client);

        match result {
            Ok(()) => {
                // Export succeeded - Ack only fully-completed messages
                let completed_messages = self.state.remove_batch_success(batch_id);
                for (msg_id, context, bytes) in completed_messages {
                    println!("Sending Ack for message {} in batch {} for {} rows", batch_id, msg_id, row_count);
                    effect_handler
                        .notify_ack(AckMsg::new(OtapPdata::new(
                            context,
                            OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)),
                        )))
                        .await?;
                }
            }
            Err(e) => {
                // Export failed - Nack ALL messages in this batch, remove entirely
                let failed_messages = self.state.remove_batch_failure(batch_id);
                for (msg_id, context, bytes) in failed_messages {
                    println!("Sending Nack for message {} in batch {} for {} rows", batch_id, msg_id, row_count);
                    effect_handler
                        .notify_nack(NackMsg::new(
                            &e,
                            OtapPdata::new(
                                context,
                                OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)),
                            ),
                        ))
                        .await?;
                }
            }
        }

        Ok(())
    }    

    fn get_next_token_refresh(token_valid_until: tokio::time::Instant) -> tokio::time::Instant {
        let token_lifetime = token_valid_until.saturating_duration_since(tokio::time::Instant::now());
        let token_expiry_buffer = tokio::time::Duration::from_secs(token_lifetime.as_secs() / 5);
        let next_token_refresh = token_valid_until - token_expiry_buffer;
        max(next_token_refresh, tokio::time::Instant::now() + std::time::Duration::from_secs(30))
    }

    async fn queue_pending_batch(&mut self, effect_handler: &EffectHandler<OtapPdata>) -> Result<(), Error> {
        let pending_batch = match self.gzip_batcher.take_pending_batch() {
            Some(batch) => batch,
            None => return Ok(()), // No pending batch - nothing to do
        };
        
        let client = self.client_pool.take();
        if let Some(completed_export) = self.in_flight_exports.push_export(
            client,
            pending_batch.batch_id,
            pending_batch.row_count,
            pending_batch.compressed_data,
        ).await {
            self.finalize_export(effect_handler, completed_export).await?;
        }

        self.last_batch_queued_at = tokio::time::Instant::now();

        Ok(())
    }

    async fn handle_pdata(&mut self, effect_handler: &EffectHandler<OtapPdata>, request: ExportLogsServiceRequest, context: Context, bytes: Bytes, msg_id: u64) -> Result<(), Error>
    {
        if context.may_return_payload() {
            self.state.add_msg_data(msg_id, context, bytes.clone());
        }
        else {
            self.state.add_msg_data(msg_id, context, Bytes::new());
        }

        let log_entries_iterator = self.transformer
            .convert_to_log_analytics(&request);

        for log_entry in log_entries_iterator {
            match self.gzip_batcher.push(&log_entry) {
                gzip_batcher::PushResult::Ok(batch_id) => {
                    // current batch id is being associated with the current message
                    self.state.add_batch_msg_relationship(batch_id, msg_id);
                }
                gzip_batcher::PushResult::BatchReady(new_batch_id) => {
                    // new batch id is being associated with the current message
                    self.state.add_batch_msg_relationship(new_batch_id, msg_id);
                    self.queue_pending_batch(effect_handler).await?;
                }
                gzip_batcher::PushResult::TooLarge => {
                    // TODO: Log the error or take appropriate action
                    print!("Log entry too large to be added to the batch: {:?}", log_entry);
                }
            }
        }

        if let Some((context, bytes)) = self.state.delete_msg_data_if_orphaned(msg_id) {
            effect_handler
                .notify_nack(NackMsg::new(
                    "No valid log entries produced",
                    OtapPdata::new(
                        context,
                        OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)),
                    ),
                ))
                .await?;
        }

        Ok(())
    }

    async fn drain_in_flight_exports(&mut self, effect_handler: &EffectHandler<OtapPdata>) -> Result<(), Error> {
        let completed_exports = self.in_flight_exports.drain().await;
        for completed_export in completed_exports {
            self.finalize_export(effect_handler, completed_export).await?;
        }
        Ok(())
    }

    async fn queue_current_batch(&mut self, effect_handler: &EffectHandler<OtapPdata>) -> Result<(), Error> {
        match self.gzip_batcher.flush() {
            FlushResult::Flush => {
                return self.queue_pending_batch(effect_handler).await;
            }
            FlushResult::Empty => Ok(()),
        }
    }

    async fn handle_shutdown(&mut self, effect_handler: &EffectHandler<OtapPdata>) -> Result<(), Error> {
        self.queue_current_batch(effect_handler).await?;
        self.drain_in_flight_exports(effect_handler).await?;

        for (msg_id, context, bytes) in self.state.drain_all() {
            print!("Found orphaned message {msg_id} in shutdown");

            effect_handler
                .notify_nack(NackMsg::new(
                    "Shutdown before export completed",
                    OtapPdata::new(
                        context,
                        OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)),
                    ),
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

                match payload {
                    OtapPayload::OtapArrowRecords(otap_records) => match otap_records {
                        OtapArrowRecords::Logs(otap_records) => {
                            let otlp_bytes: OtlpProtoBytes =
                                OtapPayload::OtapArrowRecords(OtapArrowRecords::Logs(otap_records))
                                    .try_into()
                                    .map_err(|e| Error::InternalError {
                                        message: format!("Failed to convert OTAP to OTLP: {:?}", e),
                                    })?;

                            let OtlpProtoBytes::ExportLogsRequest(bytes) = otlp_bytes else {
                                return Err(Error::InternalError {
                                    message: "Expected ExportLogsRequest bytes".to_string(),
                                });
                            };

                            let request = ExportLogsServiceRequest::decode(&bytes[..])
                                .map_err(|e| Error::InternalError {
                                    message: format!("Failed to decode logs request: {}", e),
                                })?;

                            self.handle_pdata(effect_handler, request, context, bytes, *msg_id).await?;
                        }
                        OtapArrowRecords::Metrics(_) | OtapArrowRecords::Traces(_) => {
                            // Unsupported signal types - silently drop
                        }
                    },

                    OtapPayload::OtlpBytes(otlp_bytes) => match otlp_bytes {
                        OtlpProtoBytes::ExportLogsRequest(bytes) => {
                            let request = ExportLogsServiceRequest::decode(bytes.as_ref())
                                .map_err(|e| Error::InternalError {
                                    message: format!("Failed to decode OTLP logs request: {e}"),
                                })?;

                            self.handle_pdata(effect_handler, request, context, bytes, *msg_id).await?;
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

        let mut original_client = LogsIngestionClient::new(&self.config)
            .map_err(|e| Error::InternalError { message: format!("Failed to create client: {e}") })?;

        original_client.refresh_token()
            .await
            .map_err(|e| Error::InternalError { message: format!("Failed to refresh token: {e}") })?;

        self.client_pool.initialize(&original_client);
        let mut next_token_refresh = Self::get_next_token_refresh(original_client.token_valid_until);
        
        loop {
            let next_export = max(self.last_batch_queued_at + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL),
                tokio::time::Instant::now() + tokio::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL));

            if self.in_flight_exports.len() >= MAX_IN_FLIGHT_EXPORTS {
                if let Some(completed) = self.in_flight_exports.next_completion().await {
                    self.finalize_export(&effect_handler, completed).await?;
                }
                continue;
            }

            while let Some(completed_export) = self.in_flight_exports.next_completion().now_or_never().flatten()
            {
                self.finalize_export(&effect_handler, completed_export).await?;
            }

            futures::select_biased! {
                _ = tokio::time::sleep_until(next_token_refresh).fuse() => {
                    original_client.refresh_token()
                        .await
                        .map_err(|e| Error::InternalError { message: format!("Failed to refresh token: {e}") })?;

                    next_token_refresh = Self::get_next_token_refresh(original_client.token_valid_until);
                }

                _ = tokio::time::sleep_until(next_export).fuse() => {
                    println!("Periodic export check");

                    if self.last_batch_queued_at.elapsed() >= std::time::Duration::from_secs(PERIODIC_EXPORT_INTERVAL) {
                        self.queue_current_batch(&effect_handler).await?;
                    }
                }

                completed = self.in_flight_exports.next_completion().fuse() => {
                    if let Some(completed_export) = completed {
                        self.finalize_export(&effect_handler, completed_export).await?;
                    }
                }

                msg = msg_chan.recv().fuse() => {
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
            }
        }
    }
}
