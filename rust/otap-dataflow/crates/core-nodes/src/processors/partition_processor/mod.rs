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
use otap_df_config::node::NodeUserConfig;
use otap_df_config::tenant::compiled::{KeyId, TenantTokenRegistry, TenantView, TokenScratch};
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
    ConsumerEffectHandlerExtension, FlowMetricAccumulation, Interests,
    MessageSourceLocalEffectHandlerExtension, ProcessorFactory, ProducerEffectHandlerExtension,
};
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::accessory::context::split_contexts::Contexts;
use otap_df_otap::accessory::slots::Key;
use otap_df_otap::pdata::{Context, OtapPdata};
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
pub static PARTITION_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: PARTITION_PROCESSOR_URN,
    create: create_partition_processor,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: |value| {
        let config: Config = serde_json::from_value(value.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // try to parse and plan the OPL expression - this will provide early feedback
        // about invalid expressions in user's config
        match config.partition_by {
            PartitionByConfig::OplExpression(opl_expression) => {
                let (expr, function_defs) =
                    OplParser::parse_expr_with_options(&opl_expression, default_parser_options())
                        .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                        error: format!("Could not parse OPL Expression: {e:?}"),
                    })?;

                let _ = Partitioner::try_new(expr, function_defs).map_err(|e| {
                    otap_df_config::error::Error::InvalidUserConfig {
                        error: format!("Could not plan partitioner from OPL expression: {e:?}"),
                    }
                })?;
            }
        };

        Ok(())
    },
};

/// partition processor.
pub struct PartitionProcessor {
    contexts: Contexts,
    partitioner: Partitioner,
    /// Everything needed to name a partition, kept in its own field so that
    /// naming borrows disjointly from the partitioner that yields the values.
    namer: PartitionNamer,
    metrics: MetricSet<Metrics>,
}

/// Writes a partition value onto the tenant key this processor names.
struct PartitionNamer {
    /// The engine's compiled tenant tokens, and the key this processor writes.
    ///
    /// Resolving the key once at startup keeps the per-partition path down to
    /// a slot write; the key name itself never travels.
    registry: Arc<TenantTokenRegistry>,
    key: KeyId,
    /// Reused across partitions so serializing a value costs no allocation.
    value_buf: Vec<u8>,
    scratch: TokenScratch,
    serialization_strategy: PartitionValueSerializeStrategy,
}

impl PartitionNamer {
    /// Build the outbound context for one partition by naming its value.
    ///
    /// One allocation per partition, for the derived context itself; the key
    /// name does not travel and the value buffer is reused, so nothing else
    /// on this path touches the allocator.
    fn name(&mut self, source: &Context, value: PartitionValue) -> Arc<[u64]> {
        self.value_buf.clear();
        serialize_partition_value(&self.serialization_strategy, value, &mut self.value_buf);
        let words = match source.tenant() {
            Some(words) => words.as_ref(),
            None => self.registry.empty_context().as_ref(),
        };
        let view = TenantView::new(words);
        self.registry
            .rewrite(&mut self.scratch, &view, self.key, &self.value_buf)
            .expect("partition key was verified to hold a value slot at startup")
    }
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

        let registry = pipeline_ctx.tenant_registry().cloned().ok_or_else(|| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "partition processor writes the tenant key `{}`, but this engine \
                     declares no `tenant_tokens`",
                    config.partition_key
                ),
            }
        })?;
        // Fail at startup rather than dropping partition values at runtime: a
        // key that is undeclared, or declared without a retained value, has
        // nowhere to carry the partition value the processor computes.
        let partition_key = registry
            .key_id(&config.partition_key)
            .filter(|key| registry.value_slot(*key).is_some())
            .ok_or_else(|| otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "partition processor writes the tenant key `{}`, which is not \
                     declared by any extractor with `retain: true`",
                    config.partition_key
                ),
            })?;

        Ok(Self {
            partitioner,
            contexts: Contexts::new(config.inbound_request_limit, config.outbound_request_limit),
            namer: PartitionNamer {
                registry,
                key: partition_key,
                value_buf: Vec::new(),
                scratch: TokenScratch::new(),
                serialization_strategy: config.header_serialization_strategy,
            },
            metrics: pipeline_ctx.register_metrics(),
        })
    }

    /// Clears the outbound context for the given key, and sends an Ack/Nack for the associated
    /// inbound once there are no outstanding outbounds.
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
            Message::PData(mut pdata) => {
                // get/preserve the original flow_metric ns counter
                let flow_metrics_counter = pdata.take_flow_compute();
                if let Some(flow) = flow_metrics_counter {
                    pdata.start_flow_metric();
                    pdata.add_flow_compute(flow);
                }

                let (mut inbound_context, payload) = pdata.into_parts();
                let signal_type = payload.signal_type();
                let mut otap_batch: OtapArrowRecords = payload.try_into_with_default()?;
                otap_batch.decode_transport_optimized_ids()?;
                let inbound_batch_num_items = otap_batch.num_items();

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

                        let derived = self.namer.name(&inbound_context, partition.value);
                        inbound_context.set_tenant(derived);

                        let pdata = OtapPdata::new(
                            inbound_context,
                            OtapPayload::OtapArrowRecords(partition.batch),
                        );
                        effect_handler.send_message_with_source_node(pdata).await?;
                    }
                    _ => {
                        // there are multiple partitions - need to emit while shuffling contexts..

                        // create context key for inbound batch
                        let inbound_ctx_key = self
                            .contexts
                            .insert_inbound(inbound_context.clone(), None)
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

                            // Fork the request-scoped metadata, then name this
                            // partition. Building the context by hand here was
                            // how the tenant context came to be dropped on this
                            // path but kept on the single-partition path.
                            let mut pdata_context = inbound_context.fork_request_scoped();
                            let derived = self.namer.name(&pdata_context, partition.value);
                            pdata_context.set_tenant(derived);

                            let outbound_batch_num_items = partition.batch.num_items();
                            let mut pdata = OtapPdata::new(pdata_context, partition.batch.into());
                            if let Some(flow_metrics_counter) = flow_metrics_counter {
                                pdata.start_flow_metric();

                                // preserve the inbound flow metrics counter, but partition
                                // its value across the various outbound batches relative to the
                                // proportion of inbound rows present in the outbound batch.
                                //
                                // The assumption here is that, since the flow count is the time
                                // accrued processing the batch upstream, that each row took
                                // roughly the same amount of time to process. This might not be
                                // exactly true, but doing the split like this anyway at least
                                // gives a good approximation of how much time was spent upstream
                                // processing the data in the outbound batch.
                                //
                                // Assuming each batch eventually reaches the end of the flow
                                // metrics sequence, the total compute duration sum will be
                                // accurate. If some, however, some outbound batches are dropped
                                // or routed elsewhere, we still get a good approximation of the
                                // the total compute duration for the outbound batches whose flow
                                // counters are accumulated eventually for the total compute
                                // duration sum metric
                                let partition_flow_count_ns = (flow_metrics_counter
                                    * outbound_batch_num_items as u64)
                                    / inbound_batch_num_items as u64;
                                pdata.add_flow_compute(partition_flow_count_ns);
                            }

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

fn serialize_partition_value(
    strategy: &PartitionValueSerializeStrategy,
    partition_value: PartitionValue,
    out: &mut Vec<u8>,
) {
    match strategy {
        PartitionValueSerializeStrategy::ToBytesLossy => match partition_value {
            PartitionValue::String(s) => out.extend_from_slice(s.as_bytes()),
            PartitionValue::Binary(b) => out.extend_from_slice(&b),
            PartitionValue::Float(f) => out.extend_from_slice(&f.to_le_bytes()),
            PartitionValue::Int(i) => out.extend_from_slice(&i.to_le_bytes()),
            PartitionValue::UInt(i) => out.extend_from_slice(&i.to_le_bytes()),
            PartitionValue::Boolean(b) => out.push(u8::from(b)),
            PartitionValue::Null => {}
        },
        PartitionValueSerializeStrategy::Json => {
            // `Vec<u8>` is an `io::Write`, so serializing appends in place and
            // the reused buffer keeps this path allocation-free.
            let ok = match partition_value {
                PartitionValue::String(v) => serde_json::to_writer(&mut *out, &v),
                PartitionValue::Binary(v) => serde_json::to_writer(&mut *out, &v),
                PartitionValue::Boolean(v) => serde_json::to_writer(&mut *out, &v),
                PartitionValue::Int(v) => serde_json::to_writer(&mut *out, &v),
                PartitionValue::UInt(v) => serde_json::to_writer(&mut *out, &v),
                PartitionValue::Null => serde_json::to_writer(&mut *out, &Value::Null),
                // JSON has no encoding for these, so they keep the textual
                // forms the previous transport-header path produced.
                PartitionValue::Float(f) if f.is_nan() => {
                    out.extend_from_slice(b"NaN");
                    Ok(())
                }
                PartitionValue::Float(f) if f.is_infinite() => {
                    out.extend_from_slice(if f.is_sign_negative() {
                        b"-Inf".as_slice()
                    } else {
                        b"Inf".as_slice()
                    });
                    Ok(())
                }
                PartitionValue::Float(f) => serde_json::to_writer(&mut *out, &f),
            };
            ok.expect("writing json to a Vec cannot fail");
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use super::*;

    use otap_df_config::tenant::compiled::{TenantTokenRegistryBuilder, TokenInputs};
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
    use otap_df_otap::{
        pdata::Context,
        testing::{TestCallData, next_ack, next_nack},
    };
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

    /// Name the partition processor writes, and a second key standing in for
    /// request metadata that arrived at the receiver and must survive.
    const PARTITION_KEY: &str = "partition-header";
    const CARRIED_KEY: &str = "h1";

    /// A registry declaring both keys with retained values, as an engine's
    /// `tenant_tokens` block would. Each key gets its own token, since a token
    /// resolves only when every one of its extractors is satisfied.
    fn test_registry() -> Arc<TenantTokenRegistry> {
        use otap_df_config::tenant::{Extractor, TenantTokenSpec, TenantTokens};

        let mut tokens = TenantTokens::default();
        for key in [PARTITION_KEY, CARRIED_KEY] {
            let _ = tokens.insert(
                key.to_owned(),
                TenantTokenSpec {
                    extractors: vec![Extractor::TransportHeader {
                        key: key.to_owned(),
                        transport_header: format!("x-{key}"),
                        retain: true,
                        bag: false,
                    }],
                },
            );
        }
        let mut builder = TenantTokenRegistryBuilder::new();
        builder.add_tokens(&tokens).expect("tokens compile");
        Arc::new(builder.build(1).expect("layout fits"))
    }

    /// Read a retained tenant value back out of an outbound context.
    fn tenant_value(context: &Context, key: &str) -> Option<Vec<u8>> {
        let registry = test_registry();
        let view = TenantView::new(context.tenant()?.as_ref());
        let key = registry.key_id(key)?;
        registry.retained_value(&view, key).map(<[u8]>::to_vec)
    }

    /// An inbound context as a receiver would have produced it.
    fn context_with_tenant(pairs: &[(&str, &[u8])]) -> Context {
        let registry = test_registry();
        let mut scratch = TokenScratch::new();
        let mut context = Context::default();
        if let Some(words) = registry.resolve(
            &mut scratch,
            TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
        ) {
            context.set_tenant(words);
        }
        context
    }

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
        let mut pipeline_context = pipeline_context;
        pipeline_context.set_tenant_registry(test_registry());
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
                "partition_key": header_name,
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
                    assert_eq!(
                        tenant_value(&context, header_name).as_deref(),
                        Some(partition_value.as_bytes())
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
                "partition_key": header_name,
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
                assert_eq!(
                    tenant_value(&context, header_name).as_deref(),
                    Some("0".as_bytes())
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
                "partition_key": header_name,
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
    fn test_preserves_existing_context_including_headers() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let expression = "attributes[\"x\"]";
        let header_name = "partition-header";
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_key": header_name,
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

                let mut context = context_with_tenant(&[("x-h1", b"hello world")]);
                context.set_peer_addr("10.0.0.1:5005".parse().unwrap());
                let mut pdata = OtapPdata::new(context, OtapPayload::OtapArrowRecords(otap_batch));
                pdata.start_flow_metric();
                pdata.add_flow_compute(8);
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");

                for mut out in ctx.drain_pdata().await {
                    let flow_counter = out.take_flow_compute();
                    let (context, _) = out.into_parts();
                    // The value the receiver resolved must survive the fan-out;
                    // building each outbound context by hand is what used to
                    // drop it on this path but not the single-partition one.
                    assert_eq!(
                        tenant_value(&context, CARRIED_KEY).as_deref(),
                        Some(b"hello world".as_slice())
                    );
                    assert_eq!(context.peer_addr(), Some("10.0.0.1:5005".parse().unwrap()));

                    // assert the flow counter is distributed outbound batches in proportion
                    // to their size relative to the input
                    let partition = tenant_value(&context, header_name).unwrap();
                    if partition == b"0" {
                        assert_eq!(flow_counter, Some(4));
                    }
                    if partition == b"1" {
                        assert_eq!(flow_counter, Some(2));
                    }
                    if partition == b"2" {
                        assert_eq!(flow_counter, Some(2));
                    }
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
                let context = context_with_tenant(&[("x-h1", b"hello world")]);
                let pdata = OtapPdata::new(context, OtapPayload::OtapArrowRecords(otap_batch));
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("no process error");
                let out = ctx.drain_pdata().await.into_iter().collect::<Vec<_>>();
                assert_eq!(out.len(), 1);
                let out = out.into_iter().next().unwrap();
                let (context, _) = out.into_parts();
                assert_eq!(
                    tenant_value(&context, CARRIED_KEY).as_deref(),
                    Some(b"hello world".as_slice())
                );
                assert_eq!(
                    tenant_value(&context, header_name).as_deref(),
                    Some(b"1".as_slice())
                );
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
                "partition_key": header_name,
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

    fn serialized(strategy: &PartitionValueSerializeStrategy, value: PartitionValue) -> Vec<u8> {
        let mut out = Vec::new();
        serialize_partition_value(strategy, value, &mut out);
        out
    }

    /// Scenario: every `PartitionValue` variant is serialized with the lossy
    /// byte strategy.
    /// Guarantees: strings and binary keep their bytes verbatim, numbers use
    /// little-endian, booleans are a single 0 or 1 byte, and null is empty --
    /// so a partition value round-trips to the same bytes a downstream
    /// exporter would have written as a header.
    #[test]
    fn to_bytes_lossy_serialization() {
        let s = PartitionValueSerializeStrategy::ToBytesLossy;

        assert_eq!(
            serialized(&s, PartitionValue::String("test".into())),
            b"test"
        );
        assert_eq!(
            serialized(&s, PartitionValue::Binary(vec![1, 2, 3])),
            [1, 2, 3]
        );
        assert_eq!(
            serialized(&s, PartitionValue::Int(-2)),
            (-2i64).to_le_bytes()
        );
        assert_eq!(serialized(&s, PartitionValue::UInt(7)), 7u64.to_le_bytes());
        assert_eq!(
            serialized(&s, PartitionValue::Float(1.5)),
            1.5f64.to_le_bytes()
        );
        assert_eq!(serialized(&s, PartitionValue::Boolean(true)), [1]);
        assert_eq!(serialized(&s, PartitionValue::Boolean(false)), [0]);
        assert!(serialized(&s, PartitionValue::Null).is_empty());
    }

    /// Scenario: every `PartitionValue` variant is serialized with the JSON
    /// strategy, including the floats JSON cannot represent.
    /// Guarantees: values are written as JSON scalars, and NaN and the
    /// infinities keep the textual forms the transport-header path produced
    /// rather than silently becoming `null`.
    #[test]
    fn json_serialization() {
        let s = PartitionValueSerializeStrategy::Json;

        assert_eq!(
            serialized(&s, PartitionValue::String("test".into())),
            br#""test""#
        );
        assert_eq!(serialized(&s, PartitionValue::Binary(vec![1, 2])), b"[1,2]");
        assert_eq!(serialized(&s, PartitionValue::Int(-2)), b"-2");
        assert_eq!(serialized(&s, PartitionValue::UInt(7)), b"7");
        assert_eq!(serialized(&s, PartitionValue::Float(1.5)), b"1.5");
        assert_eq!(serialized(&s, PartitionValue::Boolean(true)), b"true");
        assert_eq!(serialized(&s, PartitionValue::Null), b"null");

        assert_eq!(serialized(&s, PartitionValue::Float(f64::NAN)), b"NaN");
        assert_eq!(serialized(&s, PartitionValue::Float(f64::INFINITY)), b"Inf");
        assert_eq!(
            serialized(&s, PartitionValue::Float(f64::NEG_INFINITY)),
            b"-Inf"
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

        // only 1 outbound slot -- but a batch producing 3 partitions needs 3
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_key": header_name,
                "outbound_request_limit": 1,
            }),
            &runtime,
        )
        .unwrap();

        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let upstream_node_id = 999;

                // 3 distinct partition values -> 3 partitions
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

                // Ack the single emitted outbound -- should trigger a Nack for the inbound
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

        // 2 outbound slots -- the first batch (2 partitions) will fill them
        let processor = create_processor_with_config(
            serde_json::json!({
                "partition_by": { "opl_expression": expression },
                "partition_key": header_name,
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

                // 2 distinct partition values -> 2 partitions -> fills 2 outbound slots
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

                // second batch -- outbound slots are full, first insert_outbound fails immediately
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

                // a new batch should now succeed -- verifying the inbound slot from the failed
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
                "partition_key": header_name,
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

                // 2 distinct partition values -> 2 partitions (triggers the multi-partition path
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
