// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Partition Processor for OTAP pipelines.
//!
//! This processor will partition incoming OTAP batches by the evaluated result of some expression
//! and set the partition value in the outgoing batches metadata.

use std::sync::Arc;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::{node::NodeUserConfig, validation::validate_typed_config};
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::AckMsg;
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
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_otap::transport_headers::{TransportHeader, ValueKind};
use otap_df_pdata::{OtapArrowRecords, OtapPayload, TryIntoWithOptions};
use otap_df_query_engine::parser::default_parser_options;
use otap_df_query_engine::pipeline::partition::{PartitionValue, Partitioner};
use otap_df_query_engine_languages::opl::parser::OplParser;
use serde_json::Value;
use slotmap::Key;

use crate::processors::partition_processor::config::{
    Config, PartitionByConfig, PartitionValueSerializeStrategy,
};
use crate::processors::transform_processor::context::Contexts;

mod config;

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
    // TODO - should this also try to parse the OPL expr, etc?
    validate_config: validate_typed_config::<Config>,
};

/// partition processor.
pub struct PartitionProcessor {
    contexts: Contexts,
    partitioner: Partitioner,
    header_name: String,
    serialization_strategy: PartitionValueSerializeStrategy,
}

impl PartitionProcessor {
    fn from_config(
        _pipeline_ctx: &PipelineContext,
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
        })
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
                _ => {
                    todo!()
                }
            },
            Message::PData(pdata) => {
                let (mut inbound_context, payload) = pdata.into_parts();
                let signal_type = payload.signal_type();
                let mut otap_batch: OtapArrowRecords = payload.try_into_with_default()?;
                otap_batch.decode_transport_optimized_ids()?;

                let Ok(mut partitions) = self.partitioner.partition(otap_batch) else {
                    todo!()
                };

                match partitions.len() {
                    0 => {
                        // no partitions, just Ack the inbound
                        let pdata =
                            OtapPdata::new(inbound_context, OtapPayload::empty(signal_type));

                        // TODO - were we supposed to preserve the original pdata here?
                        effect_handler.notify_ack(AckMsg::new(pdata)).await?;
                    }
                    1 => {
                        // safety: we can expect here because we've checked there is at least one partition
                        // so call to `next` will be `Some`
                        let partition = partitions.next().expect("at least one partition");

                        // there's only one partition, so we don't need to juggle inbound and outbound contexts
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

    use super::*;

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
}
