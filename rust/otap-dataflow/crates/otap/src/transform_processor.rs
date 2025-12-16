// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transform Processor for OTAP pipelines.
//!
//! This processor performs transformations on the OTAP batches using the
//! [`otap_df_query_engine`] crate.
//!
//! Note: this processor and the query engine that it uses are still under active development.
//! The configuration may change in the future and support for various transformation query is
//! still being developed.
//!
//! ToDo: Handle Ack and Nack
//! ToDo: Detect unsupported pipelines at config time instead of run time.

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
    error::{Error as EngineError, ProcessorErrorKind},
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use otap_df_query_engine::pipeline::Pipeline;
use otap_df_telemetry::metrics::MetricSet;
use serde_json::Value;

use crate::{OTAP_PROCESSOR_FACTORIES, pdata::OtapPdata};

use self::config::Config;
use self::metrics::Metrics;

mod config;
mod metrics;

/// URN for the TransformProcessor
pub const TRANSFORM_PROCESSOR_URN: &str = "urn:otel:transform:processor";

/// Opentelemetry Processing Language Processor
pub struct TransformProcessor {
    pipeline: Pipeline,
    signal_scope: SignalScope,
    metrics: MetricSet<Metrics>,
}

/// Identifier for which signal types the transformation pipeline should be applied.
enum SignalScope {
    // Apply transformation to all signal types
    All,

    // Apply transformation to telemetry of one particular signal type
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
        let query = pipeline_expr.get_query_slice(pipeline_expr.get_query_location());
        Ok(if query.starts_with("logs") {
            Self::Signal(SignalType::Logs)
        } else if query.starts_with("traces") {
            Self::Signal(SignalType::Traces)
        } else if query.starts_with("metrics") {
            Self::Signal(SignalType::Metrics)
        } else if query.starts_with("signal") {
            Self::All
        } else {
            return Err(ConfigError::InvalidUserConfig {
                error: "could not determine signal type from query".into(),
            });
        })
    }
}

impl TransformProcessor {
    /// Create new instance from serialized configuration
    fn from_config(pipeline_ctx: &PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse TransformProcessor config: {e}"),
            })?;

        // TODO we should pass some context to the parser so we can determine if there are valid
        // identifiers when checking the config:
        // https://github.com/open-telemetry/otel-arrow/issues/1530
        let pipeline_expr = KqlParser::parse(&config.query)
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Could not parse TransformProcessor query: {e:?}"),
            })?
            .pipeline;

        // TODO: it would be nice if we could validate that the pipeline expr is supported by the
        // query engine here. Currently, validation happens lazily when the first batch is seen.
        // https://github.com/open-telemetry/otel-arrow/issues/1634

        Ok(Self {
            signal_scope: SignalScope::try_from(&pipeline_expr)?,
            pipeline: Pipeline::new(pipeline_expr),
            metrics: pipeline_ctx.register_metrics::<Metrics>(),
        })
    }

    /// determine if the transformation should be applied to this pdata, or if it should be skipped
    fn should_process(&self, pdata: &OtapPayload) -> bool {
        match self.signal_scope {
            SignalScope::All => true,
            SignalScope::Signal(signal_type) => signal_type == pdata.signal_type(),
        }
    }
}

/// Factory for creating [`TransformProcessor`] during plugin registration
fn create_transform_processor(
    pipeline_ctx: PipelineContext,
    node_id: NodeId,
    user_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let processor = TransformProcessor::from_config(&pipeline_ctx, &user_config.config)?;
    Ok(ProcessorWrapper::local(
        processor,
        node_id,
        user_config,
        processor_config,
    ))
}

/// Register TransformProcessor
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static TRANSFORM_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: TRANSFORM_PROCESSOR_URN,
    create: create_transform_processor,
};

#[async_trait(?Send)]
impl Processor<OtapPdata> for TransformProcessor {
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
                        return Err(EngineError::InternalError {
                            message: e.to_string(),
                        });
                    }
                }
                _ => {
                    // other types of control messages are ignored for now
                }
            },
            Message::PData(pdata) => {
                self.metrics.msgs_consumed.inc();
                let (context, payload) = pdata.into_parts();
                let payload = if !self.should_process(&payload) {
                    // skip handling this pdata
                    payload
                } else {
                    let mut otap_batch: OtapArrowRecords = payload.try_into()?;
                    otap_batch.decode_transport_optimized_ids()?;
                    match self.pipeline.execute(otap_batch).await {
                        Ok(otap_batch) => {
                            self.metrics.msgs_transformed.inc();
                            otap_batch.into()
                        }
                        Err(e) => {
                            self.metrics.msgs_transform_failed.inc();
                            return Err(EngineError::ProcessorError {
                                processor: effect_handler.processor_id(),
                                kind: ProcessorErrorKind::Other,
                                error: "Error executing query engine pipeline {e}".into(),
                                source_detail: e.to_string(),
                            });
                        }
                    }
                };

                effect_handler
                    .send_message(OtapPdata::new(context, payload))
                    .await
                    .inspect(|_| self.metrics.msgs_forwarded.inc())?;
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::{
        context::ControllerContext,
        testing::{
            processor::{TestContext, TestRuntime},
            test_node,
        },
    };
    use otap_df_pdata::{
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                arrow::v1::ArrowPayloadType,
                common::v1::InstrumentationScope,
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                metrics::v1::{Metric, MetricsData, ResourceMetrics, ScopeMetrics},
                resource::v1::Resource,
                trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData},
            },
        },
        testing::round_trip::{otap_to_otlp, otlp_to_otap},
    };

    use crate::pdata::OtapPdata;

    fn try_create_with_query(
        query: &str,
        runtime: &TestRuntime<OtapPdata>,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let mut node_config = NodeUserConfig::new_processor_config(TRANSFORM_PROCESSOR_URN);
        node_config.config = json!({
            "query": query
        });

        let metrics_registry_handle = runtime.metrics_registry();
        let controller_context = ControllerContext::new(metrics_registry_handle);
        let pipeline_context =
            controller_context.pipeline_context_with("group_id".into(), "pipeline_id".into(), 0, 0);
        let node_id = test_node("transform-processor");
        create_transform_processor(
            pipeline_context,
            node_id,
            Arc::new(node_config),
            runtime.config(),
        )
    }

    #[test]
    fn test_unparsable_query_is_config_time_error() {
        let runtime = TestRuntime::<OtapPdata>::new();
        match try_create_with_query("logs | invalid operator", &runtime) {
            Err(e) => {
                assert!(
                    e.to_string()
                        .contains("Could not parse TransformProcessor query")
                )
            }
            Ok(_) => {
                panic!("expected pipeline create error")
            }
        }
    }

    #[test]
    fn test_simple_transform_pipeline() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let registry = runtime.metrics_registry();
        let metrics_reporter = runtime.metrics_reporter();
        let query = "logs | where severity_text == \"ERROR\"";
        let processor = try_create_with_query(query, &runtime).expect("created processor");
        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let log_records = vec![
                    LogRecord::build().severity_text("INFO").finish(),
                    LogRecord::build().severity_text("ERROR").finish(),
                ];

                let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource::default(),
                        vec![ScopeLogs::new(
                            InstrumentationScope::default(),
                            log_records.clone(),
                        )],
                    )],
                }));

                let pdata = OtapPdata::new_default(otap_batch.into());
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                let out = ctx
                    .drain_pdata()
                    .await
                    .into_iter()
                    .map(OtapPdata::payload)
                    .map(OtapArrowRecords::try_from)
                    .map(Result::unwrap);
                let result = out
                    .into_iter()
                    .next()
                    .map(|otap_batch| otap_to_otlp(&otap_batch))
                    .expect("one result");

                match result {
                    OtlpProtoMessage::Logs(logs_data) => {
                        assert_eq!(logs_data.resource_logs.len(), 1);
                        assert_eq!(logs_data.resource_logs[0].scope_logs.len(), 1);
                        assert_eq!(
                            &logs_data.resource_logs[0].scope_logs[0].log_records,
                            &log_records[1..2]
                        )
                    }
                    invalid => {
                        panic!(
                            "invalid signal type from output. Expected logs, received {invalid:?}"
                        )
                    }
                }

                // Trigger telemetry snapshot
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect");
            })
            .validate(|_ctx| async move {
                // Allow the collector to pull from the channel
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                let mut msgs_consumed = 0;
                let mut msgs_forwarded = 0;
                let mut msgs_transformed = 0;
                let mut msgs_transform_failed = 0;
                registry.visit_current_metrics(|desc, _attrs, iter| {
                    if desc.name == "transform.processor.metrics" {
                        for (field, v) in iter {
                            let val = v.to_u64_lossy();
                            match field.name {
                                "msgs.consumed" => msgs_consumed += val,
                                "msgs.forwarded" => msgs_forwarded += val,
                                "msgs.transformed" => msgs_transformed += val,
                                "msgs.transform.failed" => msgs_transform_failed += val,
                                _ => {}
                            }
                        }
                    }
                });

                assert_eq!(msgs_consumed, 1);
                assert_eq!(msgs_forwarded, 1);
                assert_eq!(msgs_transformed, 1);
                assert_eq!(msgs_transform_failed, 0)
            });
    }

    /// Send one traces batch and one metrics batch with signals that have the same "name" values
    /// Used to test that the query selects the right signal type
    async fn send_one_traces_one_metrics_same_names(ctx: &mut TestContext<OtapPdata>) {
        let spans = vec![
            Span::build().name("foo").finish(),
            Span::build().name("bar").finish(),
        ];

        let trace_otap_batch = otlp_to_otap(&OtlpProtoMessage::Traces(TracesData::new(vec![
            ResourceSpans::new(
                Resource::default(),
                vec![ScopeSpans::new(
                    InstrumentationScope::default(),
                    spans.clone(),
                )],
            ),
        ])));

        ctx.process(Message::PData(OtapPdata::new_default(
            trace_otap_batch.into(),
        )))
        .await
        .expect("no process error");

        let metrics = vec![
            Metric::build().name("foo").finish(),
            Metric::build().name("bar").finish(),
        ];

        let metrics_otap_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(MetricsData::new(vec![
            ResourceMetrics::new(
                Resource::default(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::default(),
                    metrics.clone(),
                )],
            ),
        ])));

        ctx.process(Message::PData(OtapPdata::new_default(
            metrics_otap_batch.into(),
        )))
        .await
        .expect("no process error")
    }

    #[test]
    fn test_signal_scope() {
        // test ensure it will only operate on traces, but ignores other signals
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = "traces | where name == \"foo\"";
        let processor = try_create_with_query(query, &runtime).expect("created processor");
        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                send_one_traces_one_metrics_same_names(&mut ctx).await;
                let mut processed_pdata = ctx
                    .drain_pdata()
                    .await
                    .into_iter()
                    .map(OtapPdata::payload)
                    .map(OtapArrowRecords::try_from)
                    .map(Result::unwrap);
                let traces_batch = processed_pdata.next().expect("sent traces batch");
                let metrics_batch = processed_pdata.next().expect("sent metrics batch");

                // assert one of the spans got filtered out
                let spans = traces_batch
                    .get(ArrowPayloadType::Spans)
                    .expect("spans present");
                assert_eq!(spans.num_rows(), 1);

                // assert the metric did not get filtered out
                let metrics = metrics_batch
                    .get(ArrowPayloadType::UnivariateMetrics)
                    .expect("metrics present");
                assert_eq!(metrics.num_rows(), 2);
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_signal_scope_all() {
        // test ensure it will only operate on all signals
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = "signals | where name == \"foo\"";
        let processor = try_create_with_query(query, &runtime).expect("created processor");
        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                send_one_traces_one_metrics_same_names(&mut ctx).await;
                let mut processed_pdata = ctx
                    .drain_pdata()
                    .await
                    .into_iter()
                    .map(OtapPdata::payload)
                    .map(OtapArrowRecords::try_from)
                    .map(Result::unwrap);
                let traces_batch = processed_pdata.next().expect("sent traces batch");
                let metrics_batch = processed_pdata.next().expect("sent metrics batch");

                // assert one of the spans got filtered out
                let spans = traces_batch
                    .get(ArrowPayloadType::Spans)
                    .expect("spans present");
                assert_eq!(spans.num_rows(), 1);

                // assert it also filtered out one of the metrics
                let metrics = metrics_batch
                    .get(ArrowPayloadType::UnivariateMetrics)
                    .expect("metrics present");
                assert_eq!(metrics.num_rows(), 1);
            })
            .validate(|_ctx| async move {})
    }
}
