// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{Config, Error};
use async_trait::async_trait;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::error::{Error as EngineError, ExporterErrorKind};
use otel_arrow_dfe_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::ExporterInbox;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::pdata::OtapPdata;

/// Azure Data Explorer (ADX) exporter.
///
/// Sends OpenTelemetry log data to Azure Data Explorer via the Kusto streaming
/// ingestion REST API, using gzip-compressed JSON payloads.
pub struct AzureDataExplorerExporter {
    config: Config,
    // transformer: Transformer,
    // metrics: AzureDataExplorerExporterMetricsRc,
    // client_pool: AzureDataExplorerClientPool,
    // in_flight_exports: InFlightExports,
    // retained_exports: VecDeque<RetainedExport>,
    token_provider: Option<Box<dyn BearerTokenProvider>>,
}

impl AzureDataExplorerExporter {
    /// Build a new exporter from configuration.
    pub fn new(
        pipeline_ctx: PipelineContext,
        config: Config,
        token_provider: Box<dyn BearerTokenProvider>,
    ) -> Result<Self, Error> {
        config
            .validate()
            .map_err(|e| Error::Config(e.to_string()))?;

        let _framework_inputs = (pipeline_ctx, token_provider);
        Err(Error::NotImplemented)
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for AzureDataExplorerExporter {
    async fn start(
        self: Box<Self>,
        _inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let _framework_state = (&self.config, &self.token_provider);
        Err(EngineError::ExporterError {
            exporter: effect_handler.exporter_id(),
            kind: ExporterErrorKind::Configuration,
            error: Error::NotImplemented.to_string(),
            source_detail: String::new(),
        })
    }
}
