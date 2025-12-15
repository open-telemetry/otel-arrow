// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Query Engine Processor for OTAP pipelines.
//!
//! This processor performs transformations on the OTAP batches using the
//! [`otap_df_query_engine`] crate.
//!
//! TODO (docs):
//! - example
//! - notes about how this is experimental and APIs are subject to change
//! - todo about supporting additional languages?

use std::sync::Arc;

use async_trait::async_trait;
use data_engine_expressions::{Expression, PipelineExpression};
use data_engine_kql_parser::{KqlParser, Parser};
use linkme::distributed_slice;
use otap_df_config::{SignalType, error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::{
    ProcessorFactory,
    config::ProcessorConfig,
    context::PipelineContext,
    control::NodeControlMsg,
    effect_handler,
    error::{Error as EngineError, ProcessorErrorKind},
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use otap_df_pdata::OtapArrowRecords;
use otap_df_query_engine::pipeline::Pipeline;
use otap_df_telemetry::metrics::MetricSet;
use serde_json::Value;

use crate::{OTAP_PROCESSOR_FACTORIES, otap_grpc::OtapArrowBytes, pdata::OtapPdata};

use self::config::Config;
use self::metrics::Metrics;

mod config;
mod metrics;

/// URN for the QueryEngineProcessor
pub const QUERY_ENGINE_PROCESSOR_URN: &str = "urn:otel:processor:query_engine";

/// Opentelemetry Processing Language Processor
pub struct QueryEngineProcessor {
    pipeline: Pipeline,

    signal_scope: SignalScope,

    metrics: MetricSet<Metrics>,
}

/// Identifier for which signal types the transformation pipeline should be applied.
enum SignalScope {
    // apply transformation to all signal types
    All,

    // apply transformation to telemetry of one particular signal type
    Signal(SignalType),
}

impl TryFrom<&PipelineExpression> for SignalScope {
    type Error = ConfigError;

    fn try_from(pipeline_expr: &PipelineExpression) -> Result<Self, Self::Error> {
        // Current logic looks at the start of the pipeline and expects it to be in a form like
        // "logs | ..." or "traces | ...", etc.
        //
        // TODO the logic here wouldn't be safe for languages other than Kql. We might want to have
        //  the pipeline expression be able to return name of the source
        let program = pipeline_expr.get_query_slice(pipeline_expr.get_query_location());
        Ok(if program.starts_with("logs") {
            Self::Signal(SignalType::Logs)
        } else if program.starts_with("traces") {
            Self::Signal(SignalType::Traces)
        } else if program.starts_with("metrics") {
            Self::Signal(SignalType::Metrics)
        } else if program.starts_with("signal") {
            Self::All
        } else {
            return Err(ConfigError::InvalidUserConfig {
                error: "could not determine signal type from program".into(),
            });
        })
    }
}

impl QueryEngineProcessor {
    /// Create new instance from serialized configuration
    fn from_config(pipeline_ctx: &PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse QueryEngineProcessor config: {e}"),
            })?;

        // TODO we should pass some context to the parser so we can determine if there are valid
        // identifiers when checking the config:
        // https://github.com/open-telemetry/otel-arrow/issues/1530
        let pipeline_expr = KqlParser::parse(&config.program)
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Could not parse QueryEngineProcessor program: {e:?}"),
            })?
            .pipeline;

        let signal_scope = SignalScope::try_from(&pipeline_expr)?;

        let metrics = pipeline_ctx.register_metrics::<Metrics>();

        Ok(Self {
            pipeline: Pipeline::new(pipeline_expr),
            signal_scope,
            metrics,
        })
    }

    /// determine if the transformation should be applied to this pdata, or if it should be skipped
    fn should_process(&self, pdata: &OtapPdata) -> bool {
        match self.signal_scope {
            SignalScope::All => true,
            SignalScope::Signal(signal_type) => signal_type == pdata.signal_type(),
        }
    }
}

/// Factory for creating QueryEngineProcessor during plugin registration
fn create_query_engine_processor(
    pipeline_ctx: PipelineContext,
    node_id: NodeId,
    user_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let processor = QueryEngineProcessor::from_config(&pipeline_ctx, &user_config.config)?;
    Ok(ProcessorWrapper::local(
        processor,
        node_id,
        user_config,
        processor_config,
    ))
}

/// Register QueryEngineProcessor
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static QUERY_ENGINE_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: QUERY_ENGINE_PROCESSOR_URN,
    create: create_query_engine_processor,
};

#[async_trait(?Send)]
impl Processor<OtapPdata> for QueryEngineProcessor {
    async fn process(
        &mut self,
        message: Message<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match message {
            Message::Control(control_message) => match control_message {
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    if let Err(e) = metrics_reporter.report(&mut self.metrics) {
                        // TODO handle the error?
                    }
                }
                _ => {
                    // other types of control messages are ignored for now
                }
            },
            Message::PData(pdata) => {
                self.metrics.msgs_consumed.inc();
                let pdata_to_forward = if !self.should_process(&pdata) {
                    // skip handling this pdata
                    pdata
                } else {
                    let (context, payload) = pdata.into_parts();
                    let otap_batch: OtapArrowRecords = match payload.try_into() {
                        Ok(o) => o,
                        Err(e) => {
                            // TODO - update metrics here?
                            return Err(e.into());
                        }
                    };

                    match self.pipeline.execute(otap_batch).await {
                        Ok(otap_batch) => OtapPdata::new(context, otap_batch.into()),
                        Err(e) => {
                            // TODO - update metrics here?
                            return Err(EngineError::ProcessorError {
                                processor: effect_handler.processor_id(),
                                kind: ProcessorErrorKind::Other,
                                error: "Error executing query engine pipeline {e}".into(),
                                source_detail: e.to_string(),
                            });
                        }
                    }
                };

                match effect_handler.send_message(pdata_to_forward).await {
                    Ok(_) => self.metrics.msgs_forwarded.inc(),
                    Err(e) => {
                        // TODO update metrics?
                        // TODO something observable with error?
                    }
                }
            }
        };

        Ok(())
    }
}
