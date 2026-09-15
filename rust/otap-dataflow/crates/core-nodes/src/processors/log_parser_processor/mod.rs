//! Declarative parsing and normalization of already-framed log records.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = LOG_PARSER_PROCESSOR_URN,
    target = "otel.processor.log_parser",
);

use std::sync::Arc;

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::{SignalType, error::Error as ConfigError, node::NodeUserConfig};
use otel_arrow_dfe_engine::{
    ConsumerEffectHandlerExtension, MessageSourceLocalEffectHandlerExtension, ProcessorFactory,
    config::ProcessorConfig,
    context::PipelineContext,
    control::{AckMsg, NodeControlMsg},
    error::Error as EngineError,
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::{FlowMetricHook, ProcessorWrapper},
};
use otel_arrow_dfe_otap::{OTAP_PROCESSOR_FACTORIES, pdata::OtapPdata};
use otel_arrow_dfe_pdata::{
    OtapArrowRecords, OtapPayloadHelpers, TryIntoWithOptions,
    otap::transform::sanitize::sanitize_otap_batch,
};
use serde_json::Value;

use metrics::{ParserErrorType, ParserMetrics};

#[cfg(feature = "bench")]
mod bench;
mod metrics;
mod parse_logs;
#[cfg(test)]
mod tests;
#[cfg(feature = "bench")]
pub use bench::BenchLogParser;

/// Component identifier for declarative log parsing.
pub const LOG_PARSER_PROCESSOR_URN: &str = "urn:otel:processor:log_parser";

/// Parses framed logs while preserving their original delivery context.
pub struct LogParserProcessor {
    log_parser: parse_logs::Parser,
    metrics: ParserMetrics,
    sanitize_results: bool,
}

fn parse_config(config: &Value) -> Result<(parse_logs::Parser, bool), ConfigError> {
    let mut config = config.clone();
    let fields = config
        .as_object_mut()
        .ok_or_else(|| ConfigError::InvalidUserConfig {
            error: "log parser configuration must be an object".into(),
        })?;
    let skip_sanitize_result = fields
        .remove("skip_sanitize_result")
        .map(serde_json::from_value::<bool>)
        .transpose()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })?
        .unwrap_or(false);
    let config =
        serde_json::from_value(config).map_err(|error| ConfigError::InvalidUserConfig {
            error: error.to_string(),
        })?;
    let parser = parse_logs::Parser::new(config)
        .map_err(|error| ConfigError::InvalidUserConfig { error })?;
    Ok((parser, !skip_sanitize_result))
}

impl LogParserProcessor {
    fn from_config(pipeline_ctx: &PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let (log_parser, sanitize_results) = parse_config(config)?;
        Ok(Self {
            log_parser,
            metrics: ParserMetrics::register(pipeline_ctx),
            sanitize_results,
        })
    }

    async fn process_logs(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), (ParserErrorType, EngineError)> {
        let (context, payload) = pdata.into_parts();
        let converted: Result<OtapArrowRecords, _> = payload.try_into_with_default();
        let mut batch =
            converted.map_err(|error| (ParserErrorType::PayloadConversion, error.into()))?;
        batch
            .decode_transport_optimized_ids()
            .map_err(|error| (ParserErrorType::IdDecode, error.into()))?;
        let (mut batch, counts) = self
            .log_parser
            .apply(batch)
            .map_err(|error| (ParserErrorType::Internal, error.into()))?;
        self.metrics
            .record_parsing(self.log_parser.format(), counts);
        if self.sanitize_results {
            sanitize_otap_batch(&mut batch);
        }
        let has_data = !batch.is_empty();
        let mut pdata = OtapPdata::new(context, batch.into());
        if has_data {
            effect_handler
                .send_message_with_source_node(pdata)
                .await
                .map_err(|error| (ParserErrorType::OutputSend, error.into()))?;
        } else {
            pdata.complete_processor_without_output(effect_handler);
            effect_handler
                .notify_ack(AckMsg::new(pdata))
                .await
                .map_err(|error| (ParserErrorType::OutputSend, error))?;
        }
        Ok(())
    }
}

fn create_log_parser_processor(
    pipeline_ctx: PipelineContext,
    node_id: NodeId,
    user_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
    _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let processor = LogParserProcessor::from_config(&pipeline_ctx, &user_config.config)?;
    Ok(ProcessorWrapper::local(
        processor,
        node_id,
        user_config,
        processor_config,
    ))
}

fn validate_log_parser_config(config: &Value) -> Result<(), ConfigError> {
    parse_config(config).map(|_| ())
}

/// Factory for the declarative log parser processor.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Processor)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static LOG_PARSER_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: LOG_PARSER_PROCESSOR_URN,
    create: create_log_parser_processor,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: validate_log_parser_config,
};

#[async_trait(?Send)]
impl Processor<OtapPdata> for LogParserProcessor {
    async fn process(
        &mut self,
        message: Message<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match message {
            Message::Control(NodeControlMsg::CollectTelemetry {
                mut metrics_reporter,
            }) => {
                self.metrics
                    .report(&mut metrics_reporter)
                    .map_err(|error| EngineError::InternalError {
                        message: error.to_string(),
                    })?;
            }
            Message::PData(pdata) => {
                if pdata.signal_type() != SignalType::Logs {
                    effect_handler.send_message_with_source_node(pdata).await?;
                    return Ok(());
                }
                match self.process_logs(pdata, effect_handler).await {
                    Ok(()) => self.metrics.record_success(),
                    Err((kind, error)) => {
                        self.metrics.record_failure(kind);
                        return Err(error);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
