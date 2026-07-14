// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Partition Processor for OTAP pipelines.
//!
//! This processor will partition incoming OTAP batches by the evaluated result of some expression
//! and set the partition value in the outgoing batches metadata.

use std::sync::Arc;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::{node::NodeUserConfig, validation::validate_typed_config};
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::ProcessorErrorKind;
use otap_df_engine::local::processor::{EffectHandler, Processor};
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::wiring_contract::WiringContract;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, Interests, MessageSourceLocalEffectHandlerExtension,
    ProcessorFactory, ProducerEffectHandlerExtension,
};
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::accessory::context::split_contexts::Contexts;
use otap_df_otap::accessory::slots::Key;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_otap::transport_headers::{TransportHeader, ValueKind};
use otap_df_pdata::{OtapArrowRecords, OtapPayload, TryIntoWithOptions};
use otap_df_query_engine::parser::default_parser_options;
use otap_df_query_engine::pipeline::partition::{PartitionValue, Partitioner};
use otap_df_query_engine_languages::opl::parser::OplParser;
use otap_df_telemetry::metrics::MetricSet;
use serde_json::Value;
use slotmap::Key as _;

use self::config::{Config, PartitionByConfig, PartitionValueSerializeStrategy};
use self::metrics::Metrics;

mod config;
mod metrics;

/// URN for the partition processor
pub const PARTITION_PROCESSOR_URN: &str = "urn:otel:processor:partition";

fn create_partition_processor(
    pipeline_ctx: PipelineContext,
    node_id: NodeId,
    user_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
    _capabilities: &otap_df_engine::capability::registry::Capabilities,
) -> Result<ProcessorWrapper<OtapPdata>, otap_df_config::error::Error> {
    let processor = PartitionProcessor::from_config(&pipeline_ctx, &user_config.config)?;
    Ok(ProcessorWrapper::local(
        processor,
        node_id,
        user_config,
        processor_config,
    ))
}

/// Register partition processor
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static TRANSFORM_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: PARTITION_PROCESSOR_URN,
    create: create_partition_processor,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: validate_typed_config::<Config>,
};

/// partition processor.
pub struct PartitionProcessor {
    contexts: Contexts,
    partitioner: Partitioner,
    header_name: String,
    serialization_strategy: PartitionValueSerializeStrategy,
    metrics: MetricSet<Metrics>,
}

impl PartitionProcessor {
    fn from_config(
        pipeline_ctx: &PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!("Failed to parse PartitionProcessor config: {e}"),
            }
        })?;

        let partitioner = match config.partition_by {
            PartitionByConfig::OplExpression(opl_expression) => {
                let (expr, function_defs) =
                    OplParser::parse_expr_with_options(&opl_expression, default_parser_options())
                        .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                        error: format!("Could not parse OPL Expression: {e:?}"),
                    })?;

                Partitioner::try_new(expr, function_defs).map_err(|e| {
                    otap_df_config::error::Error::InvalidUserConfig {
                        error: format!("Could not plan partitioner from OPL expression: {e:?}"),
                    }
                })?
            }
        };

        Ok(Self {
            partitioner,
            contexts: Contexts::new(config.inbound_request_limit, config.outbound_request_limit),
            header_name: config.partition_header_name,
            serialization_strategy: config.header_serialization_strategy,
            metrics: pipeline_ctx.register_metrics(),
        })
    }

    /// Clears the outbound context for the given key, and send an Ack/Nack for any
    /// associated inbound inf the inbound and no outstanding outbounds
    async fn handle_ack_nack(
        &mut self,
        outbound_key: Key,
        signal_type: SignalType,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), otap_df_engine::error::Error> {
        // clear the outbound context
        if let Some(inbound) = self.contexts.clear_outbound(outbound_key) {
            // if we're in this location, we've cleared the final outbound context for some inbound
            // batch, which means we can now Ack or Nack the inbound context
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

#[async_trait(?Send)]
impl Processor<OtapPdata> for PartitionProcessor {
    async fn process(
        &mut self,
        message: Message<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), otap_df_engine::error::Error> {
        match message {
            Message::Control(control_message) => match control_message {
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    if let Err(e) = metrics_reporter.report(&mut self.metrics) {
                        return Err(otap_df_engine::error::Error::InternalError {
                            message: e.to_string(),
                        });
                    }
                }

                NodeControlMsg::Ack(ack_msg) => {
                    self.handle_ack_nack(
                        ack_msg.unwind.route.calldata.try_into()?,
                        ack_msg.accepted.signal_type(),
                        effect_handler,
                    )
                    .await?
                }

                NodeControlMsg::Nack(nack_msg) => {
                    let outbound_key: Key = nack_msg.unwind.route.calldata.try_into()?;
                    self.contexts
                        .set_failed_outbound(outbound_key, nack_msg.reason);
                    self.handle_ack_nack(
                        outbound_key,
                        nack_msg.refused.signal_type(),
                        effect_handler,
                    )
                    .await?;
                }

                NodeControlMsg::Config { .. }
                | NodeControlMsg::TimerTick { .. }
                | NodeControlMsg::Wakeup { .. }
                | NodeControlMsg::DelayedData { .. }
                | NodeControlMsg::MemoryPressureChanged { .. }
                | NodeControlMsg::DrainIngress { .. }
                | NodeControlMsg::Shutdown { .. } => {
                    // Not handled - nothing to do
                }
            },
            Message::PData(pdata) => {
                let (mut inbound_context, payload) = pdata.into_parts();
                let signal_type = payload.signal_type();
                let mut otap_batch: OtapArrowRecords = payload.try_into_with_default()?;
                otap_batch.decode_transport_optimized_ids()?;

                let mut partitions = match self.partitioner.partition(otap_batch) {
                    Ok(partitions) => {
                        self.metrics.partition_operations_succeeded.inc();
                        partitions
                    }
                    Err(e) => {
                        self.metrics.partition_operations_failed.inc();
                        return Err(otap_df_engine::error::Error::ProcessorError {
                            processor: effect_handler.processor_id(),
                            kind: ProcessorErrorKind::Other,
                            error: format!("Error partitioning batch: {e}"),
                            source_detail: e.to_string(),
                        });
                    }
                };

                match partitions.len() {
                    0 => {
                        // no partitions, just Ack the inbound
                        let pdata =
                            OtapPdata::new(inbound_context, OtapPayload::empty(signal_type));

                        effect_handler.notify_ack(AckMsg::new(pdata)).await?;
                    }
                    1 => {
                        // single partition is a special case because we don't need to create
                        // new outbound contexts. We can reuse the original context/headers, etc.

                        // safety: we can expect here because we've checked there is at least one
                        // partition so call to `next` will be `Some`
                        let partition = partitions.next().expect("at least one partition");

                        // update the header values
                        let mut headers =
                            inbound_context.take_transport_headers().unwrap_or_default();
                        headers.push(partition_value_to_transport_header(
                            self.header_name.clone(),
                            &self.serialization_strategy,
                            partition.value,
                        ));
                        inbound_context.set_transport_headers(headers);

                        let pdata = OtapPdata::new(
                            inbound_context,
                            OtapPayload::OtapArrowRecords(partition.batch),
                        );
                        effect_handler.send_message_with_source_node(pdata).await?;
                    }
                    _ => {
                        // there are multiple partitions - need to emit while shuffling contexts..

                        // save the original headers - they will be cloned for each batch
                        let original_headers = inbound_context
                            .transport_headers()
                            .cloned()
                            .unwrap_or_default();

                        // create context key for inbound batch
                        let inbound_ctx_key =
                            self.contexts
                                .insert_inbound(inbound_context, None)
                                .ok_or_else(|| otap_df_engine::error::Error::ProcessorError {
                                    processor: effect_handler.processor_id(),
                                    kind: ProcessorErrorKind::Other,
                                    error: "inbound slots not available".into(),
                                    source_detail: "".into(),
                                })?;

                        let mut outbound_emitted_subscribed = 0;

                        // send each partition with an outbound context and the partition value
                        // populated on the transport headers
                        for partition in partitions {
                            let outbound_ctx_key = self
                                .contexts
                                .insert_outbound(inbound_ctx_key)
                                .ok_or_else(|| {
                                if outbound_emitted_subscribed == 0 {
                                    // clear the inbound slot we allocated above as we haven't
                                    // emitted anything that would eventually get Ack/Nack'd to
                                    // clear it later
                                    self.contexts.clear_inbound(inbound_ctx_key);
                                } else {
                                    // set inbound failed - when we receive the Ack/Nack for
                                    // the outbound already routed, then we'll emit Nack
                                    // indicating that some partition was not emitted.
                                    self.contexts.set_failed_inbound(
                                        inbound_ctx_key,
                                        "insufficient outbound slots for partitions".into(),
                                    );
                                }

                                otap_df_engine::error::Error::ProcessorError {
                                    processor: effect_handler.processor_id(),
                                    kind: ProcessorErrorKind::Other,
                                    error: "outbound slots not available".into(),
                                    source_detail: "".into(),
                                }
                            })?;

                            // set the transport header
                            let mut pdata_context = Context::default();
                            let mut headers = original_headers.clone();
                            headers.push(partition_value_to_transport_header(
                                self.header_name.clone(),
                                &self.serialization_strategy,
                                partition.value,
                            ));
                            pdata_context.set_transport_headers(headers);

                            let mut pdata = OtapPdata::new(pdata_context, partition.batch.into());

                            if !outbound_ctx_key.is_null() {
                                effect_handler.subscribe_to(
                                    Interests::ACKS_OR_NACKS,
                                    outbound_ctx_key.into(),
                                    &mut pdata,
                                );
                            }
                            effect_handler.send_message_with_source_node(pdata).await?;

                            if !outbound_ctx_key.is_null() {
                                outbound_emitted_subscribed += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn partition_value_to_transport_header(
    name: String,
    strategy: &PartitionValueSerializeStrategy,
    partition_value: PartitionValue,
) -> TransportHeader {
    match strategy {
        PartitionValueSerializeStrategy::ToBytesLossy {
            text_as_binary_header,
        } => {
            let value_kind = if matches!(partition_value, PartitionValue::String(_))
                && !*text_as_binary_header
            {
                ValueKind::Text
            } else {
                ValueKind::Binary
            };

            let header_bytes = match partition_value {
                PartitionValue::String(s) => s.into_bytes(),
                PartitionValue::Binary(b) => b,
                PartitionValue::Float(f) => f.to_le_bytes().to_vec(),
                PartitionValue::Int(i) => i.to_le_bytes().to_vec(),
                PartitionValue::UInt(i) => i.to_le_bytes().to_vec(),
                PartitionValue::Boolean(b) => vec![if b { 1 } else { 0 }],
                PartitionValue::Null => Vec::new(),
            };

            TransportHeader {
                wire_name: name.clone(),
                name,
                value_kind,
                value: header_bytes,
            }
        }
        PartitionValueSerializeStrategy::Json => {
            let header_bytes = match partition_value {
                PartitionValue::String(str) => {
                    serde_json::to_vec(&str).expect("can json serialize string")
                }
                PartitionValue::Binary(bin) => {
                    serde_json::to_vec(&bin).expect("can json serialize byte arr")
                }
                PartitionValue::Boolean(bool) => {
                    serde_json::to_vec(&bool).expect("can json serialize bool")
                }
                PartitionValue::Float(float) => {
                    serde_json::to_vec(&float).expect("can json serialize float")
                }
                PartitionValue::Int(i) => serde_json::to_vec(&i).expect("can json serialize int"),
                PartitionValue::UInt(i) => serde_json::to_vec(&i).expect("can json serialize int"),
                PartitionValue::Null => {
                    serde_json::to_vec(&Value::Null).expect("can serialize null")
                }
            };

            TransportHeader {
                wire_name: name.clone(),
                name,
                value_kind: ValueKind::Text,
                value: header_bytes,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use super::*;

    use otap_df_engine::{
        capability::registry::Capabilities,
        context::ControllerContext,
        control::{
            PipelineCompletionMsg, pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
        },
        testing::{
            processor::{TestContext, TestRuntime},
            test_node,
        },
    };
    use otap_df_otap::testing::{TestCallData, next_ack, next_nack};
    use otap_df_pdata::{
        OtlpProtoBytes, TryFromWithOptions,
        otap::Logs,
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                resource::v1::Resource,
            },
        },
        testing::round_trip::otlp_to_otap,
    };
    use prost::Message as _;

    fn create_processor_with_config(
        config: Value,
        runtime: &TestRuntime<OtapPdata>,
    ) -> Result<ProcessorWrapper<OtapPdata>, otap_df_config::error::Error> {
        let mut node_config = NodeUserConfig::new_processor_config(PARTITION_PROCESSOR_URN);
        node_config.config = config;

        let telemetry_registry_handle = runtime.metrics_registry();
        let controller_context = ControllerContext::new(telemetry_registry_handle);
        let pipeline_context = controller_context.pipeline_context_with(
            "group_id".into(),
            "pipeline_id".into(),
            0,
            1,
            0,
        );
        let node_id = test_node("partition_processor");
        create_partition_processor(
            pipeline_context,
            node_id,
            Arc::new(node_config),
            runtime.config(),
            &Capabilities::empty(),
        )
    }

    /// Helper to send an Ack for a given context
    async fn send_ack(
        ctx: &mut TestContext<OtapPdata>,
        context: Context,
        signal_type: SignalType,
    ) -> Result<(), otap_df_engine::error::Error> {
        let ack = next_ack(AckMsg::new(OtapPdata::new(
            context,
            OtapPayload::empty(signal_type),
        )));
        let (_, ack) = ack.unwrap();
        ctx.process(Message::Control(NodeControlMsg::Ack(ack)))
            .await
    }

    /// Helper to send a Nack for a given context
    async fn send_nack(
        ctx: &mut TestContext<OtapPdata>,
        context: Context,
        signal_type: SignalType,
        reason: &str,
    ) -> Result<(), otap_df_engine::error::Error> {
        let nack = next_nack(NackMsg::new(
            reason,
            OtapPdata::new(context, OtapPayload::empty(signal_type)),
        ));
        let (_, nack) = nack.unwrap();
        ctx.process(Message::Control(NodeControlMsg::Nack(nack)))
            .await
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

    #[test]
    fn test_simple_partitioning() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_header_name": header_name,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let upstream_node_id = 999;

                let log_records = vec![
                    LogRecord::build()
                        .event_name("event0")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event2")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event3")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                        .finish(),
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

                let pdata = create_pdata_with_subscriber(
                    otap_batch,
                    Interests::ACKS_OR_NACKS,
                    1,
                    upstream_node_id,
                );
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                let mut out = ctx.drain_pdata().await.into_iter().collect::<VecDeque<_>>();
                assert_eq!(out.len(), 3);

                let expected = vec![
                    ("0", vec![log_records[0].clone(), log_records[2].clone()]),
                    ("1", vec![log_records[1].clone()]),
                    ("2", vec![log_records[3].clone()]),
                ];

                let mut outbound_contexts = Vec::with_capacity(3);

                for (partition_value, expected_log_records) in expected {
                    let emitted_batch = out.pop_front().unwrap();
                    let (context, payload) = emitted_batch.into_parts();
                    let headers = context.transport_headers().unwrap();
                    let header = headers.find_by_name(header_name).next().unwrap();
                    assert_eq!(
                        header,
                        &TransportHeader {
                            name: header_name.to_string(),
                            wire_name: header_name.to_string(),
                            value_kind: ValueKind::Text,
                            value: partition_value.as_bytes().to_vec()
                        }
                    );
                    outbound_contexts.push(context);

                    let proto_bytes = OtlpProtoBytes::try_from_with_default(payload).unwrap();
                    assert_eq!(
                        LogsData::decode(proto_bytes.as_bytes()).unwrap(),
                        LogsData {
                            resource_logs: vec![ResourceLogs::new(
                                Resource::default(),
                                vec![ScopeLogs::new(
                                    InstrumentationScope::default(),
                                    expected_log_records
                                )]
                            )]
                        }
                    )
                }

                // send the Acks and ensure we eventually get an Ack for the inbound context
                let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(10);
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                // first outbound partition Ack'd
                send_ack(&mut ctx, outbound_contexts.pop().unwrap(), SignalType::Logs)
                    .await
                    .unwrap();
                // no ack b/c not all outbound are ack'd
                assert!(pipeline_completion_rx.is_empty());

                // second outbound partition Ack'd
                send_ack(&mut ctx, outbound_contexts.pop().unwrap(), SignalType::Logs)
                    .await
                    .unwrap();
                // no ack b/c not all outbound are ack'd
                assert!(pipeline_completion_rx.is_empty());

                // final outbound partition ack'd
                send_ack(&mut ctx, outbound_contexts.pop().unwrap(), SignalType::Logs)
                    .await
                    .unwrap();

                // assert we finally receive an Ack for the inbound pdata
                let ack_msg = pipeline_completion_rx.recv().await.unwrap();
                match ack_msg {
                    PipelineCompletionMsg::DeliverAck { ack } => {
                        let (node_id, _ack) = next_ack(ack).expect("expected ack subscriber");
                        assert_eq!(node_id, upstream_node_id);
                    }
                    other => {
                        panic!("got unexpected pipeline ctrl message {other:?}")
                    }
                };
            })
            .validate(|_ctx| async move {});
    }

    #[test]
    fn test_single_partition() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_header_name": header_name,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let log_records = vec![
                    LogRecord::build()
                        .event_name("event0")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event2")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event3")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
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

                let mut out = ctx.drain_pdata().await.into_iter().collect::<VecDeque<_>>();
                assert_eq!(out.len(), 1);

                let emitted_batch = out.pop_front().unwrap();
                let (context, payload) = emitted_batch.into_parts();
                let headers = context.transport_headers().unwrap();
                let header = headers.find_by_name(header_name).next().unwrap();
                assert_eq!(
                    header,
                    &TransportHeader {
                        name: header_name.to_string(),
                        wire_name: header_name.to_string(),
                        value_kind: ValueKind::Text,
                        value: "0".as_bytes().to_vec()
                    }
                );

                let proto_bytes = OtlpProtoBytes::try_from_with_default(payload).unwrap();
                assert_eq!(
                    LogsData::decode(proto_bytes.as_bytes()).unwrap(),
                    LogsData {
                        resource_logs: vec![ResourceLogs::new(
                            Resource::default(),
                            vec![ScopeLogs::new(
                                InstrumentationScope::default(),
                                log_records.clone()
                            )]
                        )]
                    }
                )
            })
            .validate(|_ctx| async move {});
    }

    #[test]
    fn test_empty_batch() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_header_name": header_name,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let upstream_node_id = 999;
                let pdata = create_pdata_with_subscriber(
                    OtapArrowRecords::Logs(Logs::default()),
                    Interests::ACKS_OR_NACKS,
                    1,
                    upstream_node_id,
                );

                // send the Acks and ensure we eventually get an Ack for the inbound context
                let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(10);
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                ctx.process(Message::PData(pdata)).await.unwrap();

                // nothing came out b/c there was no rows going in so there's no partitions
                let out = ctx.drain_pdata().await.into_iter().collect::<Vec<_>>();
                assert_eq!(out.len(), 0);

                // check we just Ack'd it
                // assert we finally receive an Ack for the inbound pdata
                let ack_msg = pipeline_completion_rx.recv().await.unwrap();
                match ack_msg {
                    PipelineCompletionMsg::DeliverAck { ack } => {
                        let (node_id, _ack) = next_ack(ack).expect("expected ack subscriber");
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
    fn test_preserves_existing_headers() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_header_name": header_name,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let log_records = vec![
                    LogRecord::build()
                        .event_name("event0")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event2")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event3")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                        .finish(),
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

                let mut context = Context::default();
                let mut headers = context.take_transport_headers().unwrap_or_default();
                headers.push(TransportHeader::text("h1", "header1", "hello world"));
                context.set_transport_headers(headers);
                let pdata = OtapPdata::new(context, OtapPayload::OtapArrowRecords(otap_batch));
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                for out in ctx.drain_pdata().await {
                    let (mut context, _) = out.into_parts();
                    let headers = context.take_transport_headers().unwrap();
                    assert!(headers.find_by_name("h1").next().is_some());
                }

                // assert headers also preserved for a single partition batch
                let log_records = vec![
                    LogRecord::build()
                        .event_name("event0")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                        .finish(),
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
                let mut context = Context::default();
                let mut headers = context.take_transport_headers().unwrap_or_default();
                headers.push(TransportHeader::text("h1", "header1", "hello world"));
                context.set_transport_headers(headers);
                let pdata = OtapPdata::new(context, OtapPayload::OtapArrowRecords(otap_batch));
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");
                let out = ctx.drain_pdata().await.into_iter().collect::<Vec<_>>();
                assert_eq!(out.len(), 1);
                let out = out.into_iter().next().unwrap();
                let (mut context, _) = out.into_parts();
                let headers = context.take_transport_headers().unwrap();
                assert!(headers.find_by_name("h1").next().is_some());
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_partitioned_outbound_nack_causes_inbound_to_be_nackd() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_header_name": header_name,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let upstream_node_id = 999;

                let log_records = vec![
                    LogRecord::build()
                        .event_name("event0")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .event_name("event1")
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                        .finish(),
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

                let pdata = create_pdata_with_subscriber(
                    otap_batch,
                    Interests::ACKS_OR_NACKS,
                    1,
                    upstream_node_id,
                );
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                let out = ctx.drain_pdata().await.into_iter().collect::<Vec<_>>();
                assert_eq!(out.len(), 2);

                let mut outbound_contexts = out
                    .into_iter()
                    .map(|pdata| {
                        let (context, _) = pdata.into_parts();
                        context
                    })
                    .collect::<Vec<_>>();

                // send the Acks and ensure we eventually get an Ack for the inbound context
                let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(10);
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                // first outbound partition Ack'd
                send_ack(&mut ctx, outbound_contexts.pop().unwrap(), SignalType::Logs)
                    .await
                    .unwrap();
                // no ack b/c not all outbound are ack'd
                assert!(pipeline_completion_rx.is_empty());

                // second outbound partition Nack'd
                send_nack(
                    &mut ctx,
                    outbound_contexts.pop().unwrap(),
                    SignalType::Logs,
                    "error happened",
                )
                .await
                .unwrap();

                // assert we finally receive an Ack for the inbound pdata
                let ack_msg = pipeline_completion_rx.recv().await.unwrap();
                match ack_msg {
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        let (node_id, nack) = next_nack(nack).expect("expected ack subscriber");
                        assert_eq!(node_id, upstream_node_id);
                        assert_eq!(nack.reason, "error happened")
                    }
                    other => {
                        panic!("got unexpected pipeline ctrl message {other:?}")
                    }
                };
            })
            .validate(|_ctx| async move {})
    }

    #[test]
    fn test_partition_value_to_transport_header_to_bytes_lossy() {
        let header_name = "partition";
        let strategy = PartitionValueSerializeStrategy::ToBytesLossy {
            text_as_binary_header: false,
        };

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::String("test".to_string()),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Text,
                value: "test".as_bytes().to_vec()
            }
        );

        // ensure we also encode as Binary if configured ...
        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &PartitionValueSerializeStrategy::ToBytesLossy {
                text_as_binary_header: true,
            },
            PartitionValue::String("test".to_string()),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Binary,
                value: "test".as_bytes().to_vec()
            }
        );

        // check other header types ...

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Int(514),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Binary,
                value: 514i64.to_le_bytes().to_vec()
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Float(14.7),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Binary,
                value: 14.7f64.to_le_bytes().to_vec()
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Boolean(true),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Binary,
                value: vec![1]
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Boolean(false),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Binary,
                value: vec![0]
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Binary(vec![4, 1, 8]),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Binary,
                value: vec![4, 1, 8],
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Null,
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Binary,
                value: vec![]
            }
        );
    }

    #[test]
    fn test_partition_value_to_transport_header_json() {
        let header_name = "partition";
        let strategy = PartitionValueSerializeStrategy::Json;

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::String("test".to_string()),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Text,
                value: "\"test\"".as_bytes().to_vec()
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Int(514),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Text,
                value: "514".as_bytes().to_vec()
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Float(14.7),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Text,
                value: "14.7".as_bytes().to_vec()
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Boolean(true),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Text,
                value: "true".as_bytes().to_vec()
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Boolean(false),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Text,
                value: "false".as_bytes().to_vec()
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Binary(vec![4, 1, 8]),
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Text,
                value: "[4,1,8]".as_bytes().to_vec()
            }
        );

        let header = partition_value_to_transport_header(
            header_name.to_string(),
            &strategy,
            PartitionValue::Null,
        );
        assert_eq!(
            header,
            TransportHeader {
                name: header_name.to_string(),
                wire_name: header_name.to_string(),
                value_kind: ValueKind::Text,
                value: "null".as_bytes().to_vec()
            }
        );
    }

    /// When the outbound slot limit is exhausted mid-way through emitting partitions,
    /// the processor should:
    /// 1. Return an error containing "outbound slots not available"
    /// 2. Still have sent the partitions that were emitted before the failure
    /// 3. When those already-emitted outbound batches are Ack'd, Nack the inbound with
    ///    reason "insufficient outbound slots for partitions"
    #[test]
    fn test_full_outbound_slots_some_partitions_already_emitted() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";

        // only 1 outbound slot — but a batch producing 3 partitions needs 3
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_header_name": header_name,
                "outbound_request_limit": 1,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let upstream_node_id = 999;

                // 3 distinct partition values → 3 partitions
                let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource::default(),
                        vec![ScopeLogs::new(
                            InstrumentationScope::default(),
                            vec![
                                LogRecord::build()
                                    .event_name("e0")
                                    .attributes(vec![KeyValue::new("x", AnyValue::new_string("0"))])
                                    .finish(),
                                LogRecord::build()
                                    .event_name("e1")
                                    .attributes(vec![KeyValue::new("x", AnyValue::new_string("1"))])
                                    .finish(),
                                LogRecord::build()
                                    .event_name("e2")
                                    .attributes(vec![KeyValue::new("x", AnyValue::new_string("2"))])
                                    .finish(),
                            ],
                        )],
                    )],
                }));

                let pdata = create_pdata_with_subscriber(
                    otap_batch,
                    Interests::ACKS_OR_NACKS,
                    1,
                    upstream_node_id,
                );

                // process should fail because the 2nd partition can't allocate an outbound slot
                let err = ctx
                    .process(Message::PData(pdata))
                    .await
                    .expect_err("should fail when outbound slots exhausted");
                assert!(
                    err.to_string().contains("outbound slots not available"),
                    "unexpected error: {err}",
                );

                // the first partition should still have been sent
                let out = ctx.drain_pdata().await;
                assert_eq!(out.len(), 1, "first partition should have been emitted");
                let (outbound_context, _) = out.into_iter().next().unwrap().into_parts();

                // set up pipeline completion channel to observe Ack/Nack delivery
                let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(10);
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                // Ack the single emitted outbound — should trigger a Nack for the inbound
                // because some partitions were not emitted
                send_ack(&mut ctx, outbound_context, SignalType::Logs)
                    .await
                    .unwrap();

                let completion_msg = pipeline_completion_rx.recv().await.unwrap();
                match completion_msg {
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        let (node_id, nack) = next_nack(nack).expect("expected nack subscriber");
                        assert_eq!(node_id, upstream_node_id);
                        assert_eq!(nack.reason, "insufficient outbound slots for partitions",);
                    }
                    other => panic!("expected DeliverNack, got {other:?}"),
                };
            })
            .validate(|_ctx| async move {});
    }

    /// When outbound slots are already fully consumed from a prior batch, the next batch's
    /// very first outbound allocation fails (outbound_emitted_subscribed == 0). In this case
    /// the processor should clear the inbound slot it just allocated, return an error, and
    /// not leak the inbound slot so that subsequent batches can succeed once slots are freed.
    #[test]
    fn test_full_outbound_slots_no_partitions_emitted() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";

        // 2 outbound slots — the first batch (2 partitions) will fill them
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_header_name": header_name,
                "inbound_request_limit": 2,
                "outbound_request_limit": 2,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let upstream_node_id = 999;

                // 2 distinct partition values → 2 partitions → fills 2 outbound slots
                let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource::default(),
                        vec![ScopeLogs::new(
                            InstrumentationScope::default(),
                            vec![
                                LogRecord::build()
                                    .event_name("e0")
                                    .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                                    .finish(),
                                LogRecord::build()
                                    .event_name("e1")
                                    .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                                    .finish(),
                            ],
                        )],
                    )],
                }));

                let pdata = create_pdata_with_subscriber(
                    otap_batch.clone(),
                    Interests::ACKS_OR_NACKS,
                    1,
                    upstream_node_id,
                );

                // first batch succeeds and fills both outbound slots
                ctx.process(Message::PData(pdata)).await.unwrap();
                let first_batch_out = ctx.drain_pdata().await;
                assert_eq!(first_batch_out.len(), 2);

                // second batch — outbound slots are full, first insert_outbound fails immediately
                let pdata2 = create_pdata_with_subscriber(
                    otap_batch.clone(),
                    Interests::ACKS_OR_NACKS,
                    2,
                    upstream_node_id,
                );
                let err = ctx
                    .process(Message::PData(pdata2))
                    .await
                    .expect_err("should fail when outbound slots full");
                assert!(
                    err.to_string().contains("outbound slots not available"),
                    "unexpected error: {err}",
                );

                // nothing new was emitted
                assert!(ctx.drain_pdata().await.is_empty());

                // now Ack the first batch's outbounds to free the slots
                let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(10);
                let (pipeline_completion_tx, _pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                for out in first_batch_out {
                    let (outbound_ctx, _) = out.into_parts();
                    send_ack(&mut ctx, outbound_ctx, SignalType::Logs)
                        .await
                        .unwrap();
                }

                // a new batch should now succeed — verifying the inbound slot from the failed
                // second batch was properly cleaned up (not leaked)
                let pdata3 = create_pdata_with_subscriber(
                    otap_batch,
                    Interests::ACKS_OR_NACKS,
                    3,
                    upstream_node_id,
                );
                ctx.process(Message::PData(pdata3))
                    .await
                    .expect("should succeed after slots freed");

                let out = ctx.drain_pdata().await;
                assert_eq!(out.len(), 2, "third batch should produce 2 partitions");
            })
            .validate(|_ctx| async move {});
    }

    /// When the inbound slot limit is exhausted, the processor should return an error.
    /// After the outstanding inbound is cleared (via Ack'ing its outbounds), new batches
    /// should succeed.
    #[test]
    fn test_full_inbound_slots() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";

        // 1 inbound slot, plenty of outbound
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_header_name": header_name,
                "inbound_request_limit": 1,
                "outbound_request_limit": 10,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let upstream_node_id = 999;

                // 2 distinct partition values → 2 partitions (triggers the multi-partition path
                // which is the only path that allocates inbound slots)
                let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource::default(),
                        vec![ScopeLogs::new(
                            InstrumentationScope::default(),
                            vec![
                                LogRecord::build()
                                    .event_name("e0")
                                    .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                                    .finish(),
                                LogRecord::build()
                                    .event_name("e1")
                                    .attributes(vec![KeyValue::new("x", AnyValue::new_string("b"))])
                                    .finish(),
                            ],
                        )],
                    )],
                }));

                let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(10);
                let (pipeline_completion_tx, _pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                // first batch fills the single inbound slot
                let pdata1 = create_pdata_with_subscriber(
                    otap_batch.clone(),
                    Interests::ACKS_OR_NACKS,
                    1,
                    upstream_node_id,
                );
                ctx.process(Message::PData(pdata1)).await.unwrap();

                // second batch should fail because the inbound slot is occupied
                let pdata2 = create_pdata_with_subscriber(
                    otap_batch.clone(),
                    Interests::ACKS_OR_NACKS,
                    2,
                    upstream_node_id,
                );
                let err = ctx
                    .process(Message::PData(pdata2))
                    .await
                    .expect_err("should fail when inbound slots full");
                assert!(
                    err.to_string().contains("inbound slots not available"),
                    "unexpected error: {err}",
                );

                // Ack the first batch's outbounds to free the inbound slot
                let first_batch_out = ctx.drain_pdata().await;
                assert_eq!(first_batch_out.len(), 2);

                for out in first_batch_out {
                    let (outbound_ctx, _) = out.into_parts();
                    send_ack(&mut ctx, outbound_ctx, SignalType::Logs)
                        .await
                        .unwrap();
                }

                // now a new batch should succeed
                let pdata3 = create_pdata_with_subscriber(
                    otap_batch.clone(),
                    Interests::ACKS_OR_NACKS,
                    3,
                    upstream_node_id,
                );
                ctx.process(Message::PData(pdata3))
                    .await
                    .expect("should succeed after inbound slot freed");

                // verify the batch was processed
                let out = ctx.drain_pdata().await;
                assert_eq!(out.len(), 2, "third batch should produce 2 partitions");

                // and the inbound slot is full again
                let pdata4 = create_pdata_with_subscriber(
                    otap_batch,
                    Interests::ACKS_OR_NACKS,
                    4,
                    upstream_node_id,
                );
                let err = ctx
                    .process(Message::PData(pdata4))
                    .await
                    .expect_err("should fail again when inbound slot re-filled");
                assert!(
                    err.to_string().contains("inbound slots not available"),
                    "unexpected error: {err}",
                );
            })
            .validate(|_ctx| async move {});
    }
}
