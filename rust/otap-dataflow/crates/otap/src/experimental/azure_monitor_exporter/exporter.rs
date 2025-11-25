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
                let log_entries = self.transformer.convert_to_log_analytics(&request);

                if log_entries.is_empty() {
                    // TODO: Use debug level when logging is integrated
                    effect_handler
                        .info("[AzureMonitorExporter] No logs to send")
                        .await;
                    return Ok(());
                }

                // TODO: Use debug level when logging is integrated
                effect_handler
                    .info(&format!(
                        "[AzureMonitorExporter] Sending {} log entries to stream '{}'",
                        log_entries.len(),
                        self.config.api.stream_name,
                    ))
                    .await;

                // Debug: Print first entry as sample
                if let Some(first) = log_entries.first() {
                    // TODO: Use debug level when logging is integrated
                    effect_handler
                        .info(&format!(
                            "[AzureMonitorExporter] Sample entry: {}",
                            serde_json::to_string_pretty(first).unwrap_or_default()
                        ))
                        .await;
                }

                // Send to Azure Log Analytics
                self.client
                    .send(&log_entries)
                    .await
                    .map_err(|e| Error::InternalError {
                        message: format!("Azure Monitor HTTP send failed: {e}"),
                    })?;

                // TODO: Use debug level when logging is integrated
                effect_handler
                    .info(&format!(
                        "[AzureMonitorExporter] Successfully sent {} logs",
                        log_entries.len()
                    ))
                    .await;
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

        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    effect_handler
                        .info("[AzureMonitorExporter] Shutting down")
                        .await;
                    return Ok(TerminalState::new(
                        deadline,
                        std::iter::empty::<otap_df_telemetry::metrics::MetricSetSnapshot>(),
                    ));
                }
                Message::PData(pdata) => {
                    if let Err(e) = self.handle_pdata(pdata, &effect_handler).await {
                        effect_handler
                            .info(&format!(
                                "[AzureMonitorExporter] Error processing data: {e}"
                            ))
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
