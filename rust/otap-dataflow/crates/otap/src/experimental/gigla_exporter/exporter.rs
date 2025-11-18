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

use crate::experimental::gigla_exporter::config::Config;
use crate::experimental::gigla_exporter::client::GigLaClient;
use crate::experimental::gigla_exporter::transformer::Transformer;
use crate::pdata::OtapPdata;

/// GigLA exporter sending telemetry to the GigLA backend.
///
/// This exporter processes OTLP logs and sends them to Azure GigLA
/// (Geneva Infrastructure General-purpose Logging Analytics).
pub struct GigLaExporter {
    config: Config,
    client: GigLaClient,
    transformer: Transformer,
}

impl GigLaExporter {
    /// Build a new exporter from configuration.
    pub fn new(config: Config) -> Result<Self, otap_df_config::error::Error> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig { error: e })?;

        // Create GigLA client with the full config
        let client = GigLaClient::new(&config)
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig { error: e })?;

        // Create log transformer
        let transformer = Transformer::new(&config);

        Ok(Self {
            config,
            client,
            transformer,
        })
    }

    /// Handle a single pdata message.
    async fn handle_pdata(
        &self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Split pdata into context and payload
        let (_context, payload) = pdata.into_parts();

        // TODO: Ack/Nack handling

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
                let request = ExportLogsServiceRequest::decode(bytes.as_slice()).map_err(|e| {
                    Error::PDataError {
                        reason: format!("Failed to decode OTLP logs request: {e}"),
                    }
                })?;

                // Use the transformer with config
                let log_entries = self.transformer.convert_to_log_analytics(&request);

                if log_entries.is_empty() {
                    // TODO: Use debug level when logging is integrated
                    effect_handler.info("[GigLaExporter] No logs to send").await;
                    return Ok(());
                }

                // TODO: Use debug level when logging is integrated
                effect_handler
                    .info(&format!(
                        "[GigLaExporter] Sending {} log entries to stream '{}'",
                        log_entries.len(),
                        self.config.api.stream_name,
                    ))
                    .await;

                // Debug: Print first entry as sample
                if let Some(first) = log_entries.first() {
                    // TODO: Use debug level when logging is integrated
                    effect_handler
                        .info(&format!(
                            "[GigLaExporter] Sample entry: {}",
                            serde_json::to_string_pretty(first).unwrap_or_default()
                        ))
                        .await;
                }

                // Send to Azure Log Analytics
                self.client
                    .send(&log_entries)
                    .await
                    .map_err(|e| Error::InternalError {
                        message: format!("GigLA HTTP send failed: {e}"),
                    })?;

                // TODO: Use debug level when logging is integrated
                effect_handler
                    .info(&format!(
                        "[GigLaExporter] Successfully sent {} logs",
                        log_entries.len()
                    ))
                    .await;
            }
            OtlpProtoBytes::ExportMetricsRequest(_) => {
                // TODO: Use debug level when logging is integrated
                effect_handler
                    .info("[GigLaExporter] Metrics not supported; dropping payload")
                    .await;
            }
            OtlpProtoBytes::ExportTracesRequest(_) => {
                // TODO: Use debug level when logging is integrated
                effect_handler
                    .info("[GigLaExporter] Traces not supported; dropping payload")
                    .await;
            }
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for GigLaExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        effect_handler
            .info(&format!(
                "[GigLaExporter] Starting: endpoint={}, stream={}, dcr={}",
                self.config.api.dcr_endpoint, self.config.api.stream_name, self.config.api.dcr
            ))
            .await;

        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    effect_handler.info("[GigLaExporter] Shutting down").await;
                    return Ok(TerminalState::new(
                        deadline,
                        std::iter::empty::<otap_df_telemetry::metrics::MetricSetSnapshot>(),
                    ));
                }
                Message::PData(pdata) => {
                    if let Err(e) = self.handle_pdata(pdata, &effect_handler).await {
                        effect_handler
                            .info(&format!("[GigLaExporter] Error processing data: {e}"))
                            .await;
                    }
                }
                _ => {
                    // Ignore other message types
                }
            }
        }
    }
}
