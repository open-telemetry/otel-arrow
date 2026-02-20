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
    ConsumerEffectHandlerExtension, Interests, MessageSourceLocalEffectHandlerExtension,
    ProcessorFactory, ProducerEffectHandlerExtension,
    config::ProcessorConfig,
    context::PipelineContext,
    control::{AckMsg, NackMsg, NodeControlMsg},
    error::{Error as EngineError, ProcessorErrorKind},
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use otap_df_opl::parser::OplParser;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use otap_df_query_engine::pipeline::{Pipeline, routing::RouterExtType, state::ExecutionState};
use otap_df_telemetry::metrics::MetricSet;
use serde_json::Value;
use slotmap::Key as _;

use crate::{
    OTAP_PROCESSOR_FACTORIES,
    accessory::slots::Key,
    pdata::{Context, OtapPdata},
    transform_processor::routing::RouterImpl,
};

use self::config::{Config, Query};
use self::context::Contexts;
use self::metrics::Metrics;

mod config;
mod context;
mod metrics;
mod routing;

/// URN for the TransformProcessor
pub const TRANSFORM_PROCESSOR_URN: &str = "urn:otel:transform:processor";

/// Opentelemetry Processing Language Processor
pub struct TransformProcessor {
    pipeline: Pipeline,
    execution_state: ExecutionState,
    signal_scope: SignalScope,
    contexts: Contexts,
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
        let pipeline_expr = match &config.query {
            Query::KqlQuery(query) => KqlParser::parse(query),
            Query::OplQuery(query) => OplParser::parse(query),
        }
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Could not parse TransformProcessor query: {e:?}"),
        })?
        .pipeline;

        // TODO: it would be nice if we could validate that the pipeline expr is supported by the
        // query engine here. Currently, validation happens lazily when the first batch is seen.
        // https://github.com/open-telemetry/otel-arrow/issues/1634

        let mut execution_state = ExecutionState::new();
        execution_state.set_extension::<RouterExtType>(Box::new(RouterImpl::new()));

        Ok(Self {
            signal_scope: SignalScope::try_from(&pipeline_expr)?,
            pipeline: Pipeline::new(pipeline_expr),
            metrics: pipeline_ctx.register_metrics::<Metrics>(),
            contexts: Contexts::new(config.inbound_request_limit, config.outbound_request_limit),
            execution_state,
        })
    }

    /// determine if the transformation should be applied to this pdata, or if it should be skipped
    fn should_process(&self, pdata: &OtapPayload) -> bool {
        match self.signal_scope {
            SignalScope::All => true,
            SignalScope::Signal(signal_type) => signal_type == pdata.signal_type(),
        }
    }

    /// sends any result batches that were produced by the pipeline to the appropriate output ports
    /// while managing subscriptions and context
    async fn handle_exec_result(
        &mut self,
        inbound_context: Context,
        pipeline_result: Result<OtapArrowRecords, EngineError>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        let router_impl = self
            .execution_state
            .get_extension_mut::<RouterExtType>()
            .and_then(|router| router.as_any_mut().downcast_mut::<RouterImpl>())
            .ok_or_else(|| EngineError::ProcessorError {
                processor: effect_handler.processor_id(),
                kind: ProcessorErrorKind::Other,
                source_detail: "Router not found in pipeline exec state".into(),
                error: "Routing error:".into(),
            })?;

        // access the batch that was the output of the call to pipeline.execute. This should
        // eventually be sent on the default output port
        let default_otap_batch = match pipeline_result {
            Ok(otap_batch) => otap_batch,
            Err(e) => {
                // clear any batches that are in the buffer to be routed, as the pipeline failed
                // to execute
                router_impl.routed.clear();
                return Err(e);
            }
        };

        // TODO - there's probably some optimization we can make below where if there's only one
        // non-empty batch to be output, we don't need to change any contexts or subscriptions

        if router_impl.routed.is_empty() {
            // there were no other record batches that were maybe split off this batch to be
            // routed somewhere else, so we don't need to juggle any inbound/outbound contexts
            // and we can just handle the batch normally.
            let pdata = OtapPdata::new(inbound_context, default_otap_batch.into());
            effect_handler.send_message_with_source_node(pdata).await?;
            return Ok(());
        }

        // keep error reason if there was an error, so we can send it to upstream in Nack once
        // all routed outbound batches have been Ack/Nack'd
        let inbound_ctx_key = self
            .contexts
            .insert_inbound(inbound_context, None)
            .ok_or_else(|| EngineError::ProcessorError {
                processor: effect_handler.processor_id(),
                kind: ProcessorErrorKind::Other,
                error: "inbound slots not available".into(),
                source_detail: "".into(),
            })?;

        // send the output of the pipeline to the default output port while juggling the context for
        // the output of the pipeline. We need to do this b/c we'll be emitting this batch, plus
        // any routed batches, and we don't want to Ack the inbound context until we receive Acks
        // from all downstream batches (including this result)
        let mut pdata = OtapPdata::new(Context::default(), default_otap_batch.into());
        let outbound_key = self
            .contexts
            .insert_outbound(inbound_ctx_key)
            .ok_or_else(|| {
                // if we can't emit the default batch, we won't be able to route any of the
                // routed batches, so clear them to ensure they're not stuck in the router's
                // buffer
                router_impl.routed.clear();

                // clear the inbound slot we allocated above as we haven't emitted anything
                // that would eventually get Ack/Nack'd to clear it later
                self.contexts.clear_inbound(inbound_ctx_key);

                EngineError::ProcessorError {
                    processor: effect_handler.processor_id(),
                    kind: ProcessorErrorKind::Other,
                    error: "outbound slots not available".into(),
                    source_detail: "".into(),
                }
            })?;
        if !outbound_key.is_null() {
            effect_handler.subscribe_to(
                Interests::NACKS | Interests::ACKS,
                outbound_key.into(),
                &mut pdata,
            );
        }
        effect_handler.send_message_with_source_node(pdata).await?;

        // handle any batches that need to be forwarded to a specific output port thanks to invocation
        // of a "route_to" operator call
        for (route_name, otap_batch) in router_impl.routed.drain(..) {
            // Find the port name that matches the route name.
            let port_name = effect_handler
                .connected_ports()
                .iter()
                .find(|p| p.as_ref() == route_name.as_str())
                .ok_or_else(|| EngineError::ProcessorError {
                    processor: effect_handler.processor_id(),
                    kind: ProcessorErrorKind::Transport,
                    error: "Routing error: ".into(),
                    source_detail: format!("output port name {} not configured", route_name),
                })?
                .clone();

            // setup the pdata with the new outbound context
            let payload = OtapPayload::OtapArrowRecords(otap_batch);
            let context = Context::default();
            let mut pdata = OtapPdata::new(context, payload);
            let outbound_key = self
                .contexts
                .insert_outbound(inbound_ctx_key)
                .ok_or_else(|| {
                    // the message could not be routed b/c there wasn't room for its context in
                    // the outbound slot map. set error on the inbound key to ensure we eventually
                    // nack the inbound request
                    self.contexts.set_failed_inbound(
                        inbound_ctx_key,
                        "outbound slots were not available".into(),
                    );

                    EngineError::ProcessorError {
                        processor: effect_handler.processor_id(),
                        kind: ProcessorErrorKind::Other,
                        error: "outbound slots not available".into(),
                        source_detail: "".into(),
                    }
                })?;
            if !outbound_key.is_null() {
                effect_handler.subscribe_to(
                    Interests::NACKS | Interests::ACKS,
                    outbound_key.into(),
                    &mut pdata,
                );
            }

            effect_handler
                .send_message_with_source_node_to(port_name, pdata)
                .await?;
        }

        Ok(())
    }

    /// Clears the outbound context for the given key and send an Ack/Nack for the any
    /// associated inbound if the inbound has no outstanding outbounds.
    async fn handle_ack_nack_inbound(
        &mut self,
        outbound_key: Key,
        signal_type: SignalType,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        // clear the outbound context.
        if let Some(inbound) = self.contexts.clear_outbound(outbound_key) {
            // if here, we have cleared the final outbound context for some inbound batch,
            // which means we can now Ack or Nack the inbound context
            let (context, error_reason) = inbound;
            let pdata = OtapPdata::new(context, OtapPayload::empty(signal_type));
            if let Some(error) = error_reason {
                effect_handler.notify_nack(NackMsg::new(error, pdata)).await
            } else {
                effect_handler.notify_ack(AckMsg::new(pdata)).await
            }
        } else {
            Ok(())
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
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
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
                NodeControlMsg::Ack(ack_message) => {
                    self.handle_ack_nack_inbound(
                        ack_message.calldata.try_into()?,
                        ack_message.accepted.signal_type(),
                        effect_handler,
                    )
                    .await?;
                }
                NodeControlMsg::Nack(nack_message) => {
                    let outbound_key: Key = nack_message.calldata.try_into()?;
                    self.contexts
                        .set_failed_outbound(outbound_key, nack_message.reason);
                    self.handle_ack_nack_inbound(
                        outbound_key,
                        nack_message.refused.signal_type(),
                        effect_handler,
                    )
                    .await?;
                }
                _ => {
                    // other types of control messages are ignored for now
                }
            },
            Message::PData(pdata) => {
                let (context, payload) = pdata.into_parts();
                if !self.should_process(&payload) {
                    // skip handling this pdata
                    effect_handler
                        .send_message_with_source_node(OtapPdata::new(context, payload))
                        .await?;
                } else {
                    let mut otap_batch: OtapArrowRecords = payload.try_into()?;
                    otap_batch.decode_transport_optimized_ids()?;
                    let result = self
                        .pipeline
                        .execute_with_state(otap_batch, &mut self.execution_state)
                        .await
                        .inspect(|_| self.metrics.msgs_transformed.inc())
                        .map_err(|e| {
                            self.metrics.msgs_transform_failed.inc();
                            EngineError::ProcessorError {
                                processor: effect_handler.processor_id(),
                                kind: ProcessorErrorKind::Other,
                                error: format!("Error executing query engine pipeline {e}"),
                                source_detail: e.to_string(),
                            }
                        });

                    self.handle_exec_result(context, result, effect_handler)
                        .await?;
                };
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use otap_df_channel::mpsc::Receiver;
    use serde_json::json;

    use otap_df_config::{PortName, node::NodeUserConfig};
    use otap_df_engine::{
        context::ControllerContext,
        control::{PipelineControlMsg, pipeline_ctrl_msg_channel},
        effect_handler::SourceTagging,
        local::message::LocalSender,
        message::Sender,
        node::NodeWithPDataSender,
        testing::{
            processor::{TEST_OUT_PORT_NAME, TestContext, TestRuntime},
            test_node, test_nodes,
        },
    };
    use otap_df_pdata::{
        otap::Logs,
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
        testing::round_trip::{otap_to_otlp, otlp_to_otap, to_otap_logs},
    };

    use crate::{pdata::OtapPdata, testing::TestCallData};

    /// Helper to create test log records with specific severity levels
    fn create_log_records(severities: &[&str]) -> Vec<LogRecord> {
        severities
            .iter()
            .map(|severity| LogRecord::build().severity_text(*severity).finish())
            .collect()
    }

    /// Helper to create pdata with subscribers for testing Ack/Nack
    fn create_pdata_with_subscriber(
        otap_batch: OtapArrowRecords,
        interests: Interests,
        call_data_id: u64,
        node_id: usize,
    ) -> OtapPdata {
        OtapPdata::new_default(otap_batch.into()).test_subscribe_to(
            interests,
            TestCallData::new_with(call_data_id, 0).into(),
            node_id,
        )
    }

    /// Helper to send an Ack for a given context
    async fn send_ack(
        ctx: &mut TestContext<OtapPdata>,
        context: Context,
        signal_type: SignalType,
    ) -> Result<(), EngineError> {
        let (_, ack) = Context::next_ack(AckMsg::new(OtapPdata::new(
            context,
            OtapPayload::empty(signal_type),
        )))
        .unwrap();
        ctx.process(Message::Control(NodeControlMsg::Ack(ack)))
            .await
    }

    /// Helper to send a Nack for a given context
    async fn send_nack(
        ctx: &mut TestContext<OtapPdata>,
        context: Context,
        signal_type: SignalType,
        reason: &str,
    ) -> Result<(), EngineError> {
        let (_, nack) = Context::next_nack(NackMsg::new(
            reason,
            OtapPdata::new(context, OtapPayload::empty(signal_type)),
        ))
        .unwrap();
        ctx.process(Message::Control(NodeControlMsg::Nack(nack)))
            .await
    }

    fn try_create_with_config(
        config: Value,
        runtime: &TestRuntime<OtapPdata>,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let mut node_config = NodeUserConfig::new_processor_config(TRANSFORM_PROCESSOR_URN);
        node_config.config = config;
        node_config.default_output = Some(TEST_OUT_PORT_NAME.into());

        let telemetry_registry_handle = runtime.metrics_registry();
        let controller_context = ControllerContext::new(telemetry_registry_handle);
        let pipeline_context = controller_context.pipeline_context_with(
            "group_id".into(),
            "pipeline_id".into(),
            0,
            1,
            0,
        );
        let node_id = test_node("transform-processor");
        create_transform_processor(
            pipeline_context,
            node_id,
            Arc::new(node_config),
            runtime.config(),
        )
    }

    fn try_create_with_kql_query(
        query: &str,
        runtime: &TestRuntime<OtapPdata>,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        try_create_with_config(json!({ "kql_query": query }), runtime)
    }

    fn try_create_with_opl_query(
        query: &str,
        runtime: &TestRuntime<OtapPdata>,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        try_create_with_config(json!({ "opl_query": query }), runtime)
    }

    #[test]
    fn test_unparsable_query_is_config_time_error() {
        let runtime = TestRuntime::<OtapPdata>::new();
        match try_create_with_kql_query("logs | invalid operator", &runtime) {
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
        let telemetry_registry = runtime.metrics_registry();
        let metrics_reporter = runtime.metrics_reporter();
        let query = "logs | where severity_text == \"ERROR\"";
        let processor = try_create_with_kql_query(query, &runtime).expect("created processor");
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

                let mut msgs_transformed = 0;
                let mut msgs_transform_failed = 0;
                telemetry_registry.visit_current_metrics(|desc, _attrs, iter| {
                    if desc.name == "transform.processor" {
                        for (field, v) in iter {
                            let val = v.to_u64_lossy();
                            match field.name {
                                "msgs.transformed" => msgs_transformed += val,
                                "msgs.transform.failed" => msgs_transform_failed += val,
                                _ => {}
                            }
                        }
                    }
                });

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
        let processor = try_create_with_kql_query(query, &runtime).expect("created processor");
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
        let processor = try_create_with_kql_query(query, &runtime).expect("created processor");
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

    /// test helper function to set a pdata sender on the processor wrapper for the named output port
    /// returns the receiver for the channel
    fn set_pdata_sender(
        port_name: &'static str,
        processor: &mut ProcessorWrapper<OtapPdata>,
    ) -> Receiver<OtapPdata> {
        let test_node_id = NodeId {
            index: 1,
            name: "test_node".into(),
        };
        let (test_port_tx, test_port_rx) = otap_df_channel::mpsc::Channel::new(10);
        processor
            .set_pdata_sender(
                test_node_id,
                PortName::from(port_name),
                Sender::Local(LocalSender::mpsc(test_port_tx)),
            )
            .unwrap();

        test_port_rx
    }

    #[test]
    fn test_simple_route_to() {
        // test ensure it will only operate on all signals
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = "logs | route_to \"test_port\"";
        let mut processor = try_create_with_opl_query(query, &runtime).expect("created processor");

        let test_port_rx = set_pdata_sender("test_port", &mut processor);
        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let input = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource::default(),
                        vec![ScopeLogs::new(
                            InstrumentationScope::default(),
                            vec![LogRecord::build().severity_text("ERROR").finish()],
                        )],
                    )],
                }));
                let pdata = OtapPdata::new_default(input.clone().into());
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
                let result = out.into_iter().next().expect("one result");

                // expect we got an empty batch:
                assert_eq!(result, OtapArrowRecords::Logs(Logs::default()));
                // TODO when we support Ack/Nack here assert on the context of this message

                let mut routed = Vec::new();
                while let Ok(msg) = test_port_rx.try_recv() {
                    routed.push(msg);
                }
                assert_eq!(routed.len(), 1);
                let (_context, payload) = routed.pop().unwrap().into_parts();
                match payload {
                    OtapPayload::OtapArrowRecords(result) => {
                        assert_eq!(result, input)
                    }
                    _ => panic!("unexpected payload type"),
                }
                // TODO when we support Ack/Nack here assert on routed context
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_conditional_route_to() {
        // test ensure it will only operate on all signals
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = r#"logs
            | if (severity_text == "ERROR") {
                route_to "error_port"
            } else if (severity_text == "INFO") {
                route_to "info_port"
            }"#;
        let mut processor = try_create_with_opl_query(query, &runtime).expect("created processor");

        let error_port_rx = set_pdata_sender("error_port", &mut processor);
        let info_port_rx = set_pdata_sender("info_port", &mut processor);

        fn assert_logs_records_equal(otap_batch: OtapArrowRecords, log_record: LogRecord) {
            let result = otap_to_otlp(&otap_batch);
            match result {
                OtlpProtoMessage::Logs(logs) => {
                    assert_eq!(
                        &logs.resource_logs[0].scope_logs[0].log_records,
                        &[log_record]
                    )
                }
                _ => panic!("unexpected result"),
            }
        }

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let error_log_record = LogRecord::build().severity_text("ERROR").finish();
                let info_log_record = LogRecord::build().severity_text("INFO").finish();
                let other_log_record = LogRecord::build().severity_text("DEBUG").finish();
                let input = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource::default(),
                        vec![ScopeLogs::new(
                            InstrumentationScope::default(),
                            vec![
                                error_log_record.clone(),
                                info_log_record.clone(),
                                other_log_record.clone(),
                            ],
                        )],
                    )],
                }));
                let pdata = OtapPdata::new_default(input.clone().into());
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                // check anything not routed get outputted to the default port
                let out = ctx
                    .drain_pdata()
                    .await
                    .into_iter()
                    .map(OtapPdata::payload)
                    .map(OtapArrowRecords::try_from)
                    .map(Result::unwrap);
                let default_result = out.into_iter().next().expect("one result");
                assert_logs_records_equal(default_result, other_log_record);

                // check error log record got routed to correct out pot
                let mut routed = Vec::new();
                while let Ok(msg) = error_port_rx.try_recv() {
                    routed.push(msg);
                }
                assert_eq!(routed.len(), 1);
                let (_context, payload) = routed.pop().unwrap().into_parts();
                match payload {
                    OtapPayload::OtapArrowRecords(result) => {
                        assert_logs_records_equal(result, error_log_record);
                    }
                    _ => panic!("unexpected payload type"),
                }

                // check error log record got routed to correct out pot
                let mut routed = Vec::new();
                while let Ok(msg) = info_port_rx.try_recv() {
                    routed.push(msg);
                }
                assert_eq!(routed.len(), 1);
                let (_context, payload) = routed.pop().unwrap().into_parts();
                match payload {
                    OtapPayload::OtapArrowRecords(result) => {
                        assert_logs_records_equal(result, info_log_record);
                    }
                    _ => panic!("unexpected payload type"),
                }
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_ack_nack_with_subscribers_no_routing() {
        // Smoke test to ensure Ack/Nack handling works correctly when there are subscribers.
        // This verifies that the context tracking mechanism properly preserves subscriber
        // information through the transform processor, allowing proper Ack/Nack propagation.
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = "logs | where severity_text == \"INFO\"";
        let processor = try_create_with_kql_query(query, &runtime).expect("created processor");

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                // Enable source node tagging to simulate multi-source wiring
                ctx.set_source_tagging(SourceTagging::Enabled);

                // Create a log record
                let log_records = vec![LogRecord::build().severity_text("INFO").finish()];

                let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource::default(),
                        vec![ScopeLogs::new(
                            InstrumentationScope::default(),
                            log_records.clone(),
                        )],
                    )],
                }));

                // Create pdata with a subscriber - this simulates an upstream component
                // that wants to be notified when processing completes
                let pdata = OtapPdata::new_default(otap_batch.into()).test_subscribe_to(
                    Interests::ACKS,
                    TestCallData::new_with(1, 0).into(),
                    999,
                );

                let (mut inbound_context, payload) = pdata.into_parts();
                let pdata = OtapPdata::new(inbound_context.clone(), payload);

                // Process the message through the transform processor
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                // Drain output and verify message was transformed and emitted
                let mut output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1, "Should emit one transformed message");

                let outbound_pdata = output.pop().unwrap();

                let (outbound_context, _payload) = outbound_pdata.clone().into_parts();

                // assert that since the pipeline did no routing, the outbound context should be
                // same as the inbound (after adding the processor's source node)
                assert_eq!(inbound_context.source_node(), Some(999));
                assert_eq!(outbound_context.source_node(), Some(0));
                inbound_context.set_source_node(0);
                assert_eq!(inbound_context, outbound_context);
                assert!(outbound_context.has_subscribers());
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_multi_source_tags_nonzero_node_id() {
        // When needs_source_tag is enabled (simulating multi-source wiring), the
        // processor should push a source-node frame with its own (non-zero) node ID
        // and empty interests onto the context stack.
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = "logs | where severity_text == \"INFO\"";

        // Build the processor with node index 5 (non-zero) by allocating 6 node IDs.
        let nodes = test_nodes(vec!["n0", "n1", "n2", "n3", "n4", "transform-processor"]);
        let node_id = nodes[5].clone();
        assert_eq!(node_id.index, 5);

        let mut node_config = NodeUserConfig::new_processor_config(TRANSFORM_PROCESSOR_URN);
        node_config.config = json!({ "kql_query": query });
        node_config.default_output = Some(TEST_OUT_PORT_NAME.into());

        let telemetry_registry_handle = runtime.metrics_registry();
        let controller_context = ControllerContext::new(telemetry_registry_handle);
        let pipeline_context = controller_context.pipeline_context_with(
            "group_id".into(),
            "pipeline_id".into(),
            0,
            1,
            0,
        );
        let processor = create_transform_processor(
            pipeline_context,
            node_id,
            Arc::new(node_config),
            runtime.config(),
        )
        .expect("created processor");

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                // Enable source node tagging to simulate multi-source wiring
                ctx.set_source_tagging(SourceTagging::Enabled);

                let log_records = vec![LogRecord::build().severity_text("INFO").finish()];
                let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource::default(),
                        vec![ScopeLogs::new(
                            InstrumentationScope::default(),
                            log_records.clone(),
                        )],
                    )],
                }));

                // Subscribe at node 999 with ACK interest
                let pdata = OtapPdata::new_default(otap_batch.into()).test_subscribe_to(
                    Interests::ACKS,
                    TestCallData::new_with(1, 0).into(),
                    999,
                );

                let (inbound_context, payload) = pdata.into_parts();
                assert_eq!(inbound_context.source_node(), Some(999));

                let pdata = OtapPdata::new(inbound_context, payload);
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                let mut output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1, "Should emit one transformed message");

                let outbound_pdata = output.pop().unwrap();
                let (outbound_context, _payload) = outbound_pdata.into_parts();

                // The processor at node 5 should have tagged its source with empty interests
                assert_eq!(outbound_context.source_node(), Some(5));
                // The subscriber at node 999 is still present (the source-node frame
                // has empty interests and does not count as a subscriber)
                assert!(outbound_context.has_subscribers());

                // Verify the stack structure: frame[0] is the subscriber,
                // frame[1] is the source-node tag.
                let frames = outbound_context.frames();
                assert_eq!(frames.len(), 2);
                // Bottom frame: the ACK subscriber at node 999
                assert_eq!(frames[0].node_id, 999);
                assert_eq!(frames[0].interests, Interests::ACKS);
                // Top frame: the source-node tag at node 5 with empty interests
                assert_eq!(frames[1].node_id, 5);
                assert_eq!(frames[1].interests, Interests::empty());
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_ack_with_subscribers_with_routing() {
        // test to ensure we don't Ack the inbound batch until all outbound batches have been Ack'd
        // in the case that the inbound batch has subscribers
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = r#"logs
            | if (severity_text == "ERROR") {
                route_to "error_port"
            } else if (severity_text == "INFO") {
                route_to "info_port"
            }"#;
        let mut processor = try_create_with_opl_query(query, &runtime).expect("created processor");

        let error_port_rx = set_pdata_sender("error_port", &mut processor);
        let info_port_rx = set_pdata_sender("info_port", &mut processor);

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let log_records = create_log_records(&["ERROR", "INFO", "DEBUG"]);
                let input = to_otap_logs(log_records);

                let upstream_node_id = 999;
                let pdata = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS,
                    1,
                    upstream_node_id,
                );

                // preserve the inbound context for later
                let (inbound_context, payload) = pdata.into_parts();
                let pdata = OtapPdata::new(inbound_context.clone(), payload);

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                // assert that since some routing (splitting of the batch) did occur, we get a
                // new context
                let (outbound_context1, _) = ctx.drain_pdata().await.pop().unwrap().into_parts();
                assert_ne!(inbound_context, outbound_context1);

                let (outbound_context2, _) = error_port_rx.recv().await.unwrap().into_parts();
                let (outbound_context3, _) = info_port_rx.recv().await.unwrap().into_parts();

                let (pipeline_ctrl_tx, mut pipeline_ctrl_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_ctrl_tx);

                // now we'll Ack the outbound messages and ensure that we eventually emit an ack
                // for the inbound message
                send_ack(&mut ctx, outbound_context1, SignalType::Logs)
                    .await
                    .unwrap();
                // no ack b/c not all outbound are ack'd
                assert!(pipeline_ctrl_rx.is_empty());

                send_ack(&mut ctx, outbound_context2, SignalType::Logs)
                    .await
                    .unwrap();
                // still no ack b/c not all outbound are ack'd
                assert!(pipeline_ctrl_rx.is_empty());

                send_ack(&mut ctx, outbound_context3, SignalType::Logs)
                    .await
                    .unwrap();
                // now we've ack'd all three outbound, so it should emit an Ack message
                let ack_msg = pipeline_ctrl_rx.recv().await.unwrap();
                match ack_msg {
                    PipelineControlMsg::DeliverAck { node_id, .. } => {
                        assert_eq!(node_id, upstream_node_id);
                    }
                    other => {
                        panic!("got unexpected pipeline ctrl message {other:?}")
                    }
                };
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_nack_with_subscribers_with_routing_downstream_routed_error() {
        // test that the inbound batch will be Nack'd with the reason from a downstream routed
        // batch that was Nack'd by something downstreams
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = r#"logs
            | if (severity_text == "ERROR") {
                route_to "error_port"
            }
            "#;
        let mut processor = try_create_with_opl_query(query, &runtime).expect("created processor");
        let error_port_rx = set_pdata_sender("error_port", &mut processor);

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let log_records = create_log_records(&["ERROR", "INFO"]);
                let input = to_otap_logs(log_records);

                let upstream_node_id = 999;
                let pdata = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    1,
                    upstream_node_id,
                );

                // preserve the inbound context for later
                let (inbound_context, payload) = pdata.into_parts();
                let pdata = OtapPdata::new(inbound_context.clone(), payload);

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                // get the outbound context from the default output port
                let (outbound_ctx_default, _) = ctx.drain_pdata().await.pop().unwrap().into_parts();
                assert_ne!(inbound_context, outbound_ctx_default);

                // get the outbound context from the routed output port
                let (outbound_ctx_routed, _) = error_port_rx.recv().await.unwrap().into_parts();

                let (pipeline_ctrl_tx, mut pipeline_ctrl_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_ctrl_tx);

                // simulate an Ack coming from the message that got sent on the default output port
                send_ack(&mut ctx, outbound_ctx_default, SignalType::Logs)
                    .await
                    .unwrap();
                // ensure we haven't Ack'd yet b/c there are still outstanding outbound messages
                assert!(pipeline_ctrl_rx.is_empty());

                // simulate a Nack coming from the message that got routed
                send_nack(
                    &mut ctx,
                    outbound_ctx_routed,
                    SignalType::Logs,
                    "downstream routed error",
                )
                .await
                .unwrap();

                // now ensure that we receive a Nack for the inbound b/c one of the downstream
                // routed messages was Nack'd
                let nack_msg = pipeline_ctrl_rx.recv().await.unwrap();
                match nack_msg {
                    PipelineControlMsg::DeliverNack { node_id, nack } => {
                        assert_eq!(node_id, upstream_node_id);
                        assert_eq!(nack.reason, "downstream routed error");
                    }
                    other => {
                        panic!("got unexpected pipeline ctrl message {other:?}")
                    }
                };
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_nack_with_subscribers_with_routing_downstream_default_error() {
        // Test that we correctly Nack the inbound message when the batch that gets sent
        // via the default output port is Nack'd by something downstream of this processor
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = r#"logs
            | if (severity_text == "ERROR") {
                route_to "error_port"
            }
            "#;
        let mut processor = try_create_with_opl_query(query, &runtime).expect("created processor");
        let error_port_rx = set_pdata_sender("error_port", &mut processor);

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let log_records = create_log_records(&["ERROR", "INFO"]);
                let input = to_otap_logs(log_records);

                let upstream_node_id = 999;
                let pdata = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    1,
                    upstream_node_id,
                );

                // preserve the inbound context for later
                let (inbound_context, payload) = pdata.into_parts();
                let pdata = OtapPdata::new(inbound_context.clone(), payload);

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                // get the outbound context from the default output port
                let (outbound_ctx_default, _) = ctx.drain_pdata().await.pop().unwrap().into_parts();
                assert_ne!(inbound_context, outbound_ctx_default);

                // get the outbound context from the routed output port
                let (outbound_ctx_routed, _) = error_port_rx.recv().await.unwrap().into_parts();

                let (pipeline_ctrl_tx, mut pipeline_ctrl_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_ctrl_tx);

                // simulate an Nack coming from the message that got sent on the default output port
                send_nack(
                    &mut ctx,
                    outbound_ctx_default,
                    SignalType::Logs,
                    "downstream default error",
                )
                .await
                .unwrap();
                // ensure we haven't Ack'd yet b/c there are still outstanding outbound messages
                assert!(pipeline_ctrl_rx.is_empty());

                // simulate a Ack coming from the message that got routed
                send_ack(&mut ctx, outbound_ctx_routed, SignalType::Logs)
                    .await
                    .unwrap();

                // now ensure that we receive a Nack for the inbound b/c one of the downstream
                // messages sent on the default output port were Nack'd
                let nack_msg = pipeline_ctrl_rx.recv().await.unwrap();
                match nack_msg {
                    PipelineControlMsg::DeliverNack { node_id, nack } => {
                        assert_eq!(node_id, upstream_node_id);
                        assert_eq!(nack.reason, "downstream default error");
                    }
                    other => {
                        panic!("got unexpected pipeline ctrl message {other:?}")
                    }
                };
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_ack_nack_full_contexts_outbound_slots_for_routed_batch() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = r#"logs
            | if (severity_text == "ERROR") {
                route_to "error_port"
            }
            "#;

        let mut processor = try_create_with_config(
            json!({
                "opl_query": query,
                "outbound_request_limit": 1,
            }),
            &runtime,
        )
        .unwrap();
        let error_port_rx = set_pdata_sender("error_port", &mut processor);

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let log_records = create_log_records(&["ERROR", "INFO"]);
                let input = to_otap_logs(log_records);

                let upstream_node_id = 999;
                let pdata = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    1,
                    upstream_node_id,
                );

                // preserve the inbound context for later
                let (inbound_context, payload) = pdata.into_parts();
                let pdata = OtapPdata::new(inbound_context.clone(), payload);

                // try to process the message. This should return an error b/c only 1
                // outbound slot is available, but two outbound batches will be produced
                let err = ctx
                    .process(Message::PData(pdata))
                    .await
                    .expect_err("process error");
                assert!(err.to_string().contains("outbound slots not available"));

                // we should still have sent the first batch produced
                let (outbound_ctx_default, _) = ctx.drain_pdata().await.pop().unwrap().into_parts();
                assert_ne!(inbound_context, outbound_ctx_default);

                // the second batch, which will have been routed, will not have been sent due to
                // insufficient outbound slots
                assert!(error_port_rx.is_empty());

                let (pipeline_ctrl_tx, mut pipeline_ctrl_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_ctrl_tx);

                // because there's only one outbound message, if this message gets Ack'd, we should
                // then Nack the inbound batch b/c some part of it was not processed
                send_ack(&mut ctx, outbound_ctx_default, SignalType::Logs)
                    .await
                    .unwrap();

                let nack_msg = pipeline_ctrl_rx.try_recv().unwrap();
                match nack_msg {
                    PipelineControlMsg::DeliverNack { node_id, nack } => {
                        assert_eq!(node_id, upstream_node_id);
                        assert_eq!(nack.reason, "outbound slots were not available");
                    }
                    other => {
                        panic!("got unexpected pipeline ctrl message {other:?}")
                    }
                };
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_ack_nack_full_contexts_outbound_slots_for_default_batch() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = r#"logs
            | if (severity_text == "ERROR") {
                route_to "error_port"
            }
            "#;

        let mut processor = try_create_with_config(
            json!({
                "opl_query": query,
                "inbound_request_limit": 2,
                "outbound_request_limit": 2,
            }),
            &runtime,
        )
        .unwrap();
        let error_port_rx = set_pdata_sender("error_port", &mut processor);

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let log_records = create_log_records(&["ERROR", "INFO"]);
                let input = to_otap_logs(log_records);

                let upstream_node_id = 999;
                let pdata = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    1,
                    upstream_node_id,
                );

                // send a first batch that should fill up the slot map
                ctx.process(Message::PData(pdata)).await.unwrap();

                // send another pdata - this should fail b/c the outbound slot map is full
                let pdata = OtapPdata::new_default(input.clone().into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::new_with(1, 0).into(),
                    upstream_node_id,
                );
                let err = ctx
                    .process(Message::PData(pdata))
                    .await
                    .expect_err("process error");
                assert!(err.to_string().contains("outbound slots not available"));

                // now drain and ack the the messages from the first batch to clear out the slot map
                let (outbound_ctx_default, _) = ctx.drain_pdata().await.pop().unwrap().into_parts();
                let (outbound_ctx_routed, _) = error_port_rx.recv().await.unwrap().into_parts();

                for pdata_ctx in [outbound_ctx_default, outbound_ctx_routed] {
                    send_ack(&mut ctx, pdata_ctx, SignalType::Logs)
                        .await
                        .unwrap();
                }

                // send another pdata and it should succeed b/c the slot map is cleared out
                let pdata = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    1,
                    upstream_node_id,
                );
                ctx.process(Message::PData(pdata)).await.unwrap();

                // send yet another failed batch -- we do this to ensure that when we returned
                // an error for the first failed batch, that we cleared the inbound slot. If
                // we didn't, we'd see an error saying the inbound slot cannot be allocated
                let pdata = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    1,
                    upstream_node_id,
                );
                let err = ctx
                    .process(Message::PData(pdata))
                    .await
                    .expect_err("process error");
                assert!(err.to_string().contains("outbound slots not available"));
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_ack_nack_full_contexts_inbound_slots() {
        // Test that when inbound slots are full, the processor returns an error
        // and properly cleans up so subsequent requests can succeed once slots are freed
        let runtime = TestRuntime::<OtapPdata>::new();
        let query = r#"logs
            | if (severity_text == "ERROR") {
                route_to "error_port"
            }
            "#;

        let mut processor = try_create_with_config(
            json!({
                "opl_query": query,
                "inbound_request_limit": 1,
                "outbound_request_limit": 10,
            }),
            &runtime,
        )
        .unwrap();
        let error_port_rx = set_pdata_sender("error_port", &mut processor);

        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let log_records = create_log_records(&["ERROR", "INFO"]);
                let input = to_otap_logs(log_records);

                let upstream_node_id = 999;

                // Send first batch - this should fill the single inbound slot
                let pdata1 = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    1,
                    upstream_node_id,
                );
                ctx.process(Message::PData(pdata1)).await.unwrap();

                // Try to send another batch - this should fail because inbound slot is full
                let pdata2 = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    2,
                    upstream_node_id,
                );
                let err = ctx
                    .process(Message::PData(pdata2))
                    .await
                    .expect_err("should fail when inbound slots full");
                assert!(
                    err.to_string().contains("inbound slots not available"),
                    "Expected inbound slots error, got: {}",
                    err
                );

                // Collect the outbound messages from the first batch
                let (outbound_ctx_default, _) = ctx.drain_pdata().await.pop().unwrap().into_parts();
                let (outbound_ctx_routed, _) = error_port_rx.recv().await.unwrap().into_parts();

                // Ack both outbound messages to free the inbound slot
                for pdata_ctx in [outbound_ctx_default, outbound_ctx_routed] {
                    send_ack(&mut ctx, pdata_ctx, SignalType::Logs)
                        .await
                        .unwrap();
                }

                // Now try again - should succeed because inbound slot was freed
                let pdata3 = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    3,
                    upstream_node_id,
                );
                ctx.process(Message::PData(pdata3))
                    .await
                    .expect("should succeed after inbound slot freed");

                // Verify we can process again and fill the slot
                let pdata4 = create_pdata_with_subscriber(
                    input.clone(),
                    Interests::ACKS | Interests::NACKS,
                    4,
                    upstream_node_id,
                );
                let err = ctx
                    .process(Message::PData(pdata4))
                    .await
                    .expect_err("should fail again when inbound slot full");
                assert!(
                    err.to_string().contains("inbound slots not available"),
                    "Expected inbound slots error, got: {}",
                    err
                );
            })
            .validate(|_ctx| async move {})
    }
}
