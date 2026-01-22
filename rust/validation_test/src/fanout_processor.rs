// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Processor that duplicates incoming pdata to all connected output ports.

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::PortName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ProcessorFactory;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use std::sync::Arc;

pub const FANOUT_PROCESSOR_URN: &str = "urn:otel:fanout:processor";

/// Forwards each incoming pdata to every connected out port.
pub struct FanoutProcessor;

pub fn create_fanout_processor(
    _pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        FanoutProcessor,
        node,
        node_config,
        processor_config,
    ))
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static FANOUT_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: FANOUT_PROCESSOR_URN,
    create: |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig| {
        create_fanout_processor(pipeline_ctx, node, node_config, proc_cfg)
    },
};

impl FanoutProcessor {
    fn fanout_ports(effect_handler: &local::EffectHandler<OtapPdata>) -> Vec<PortName> {
        effect_handler.connected_ports()
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for FanoutProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            Message::Control(NodeControlMsg::CollectTelemetry { .. }) => Ok(()),
            Message::Control(_) => Ok(()),
            Message::PData(pdata) => {
                let (context, payload) = pdata.into_parts();
                let ports = Self::fanout_ports(effect_handler);

                for port in ports {
                    let duplicated = OtapPdata::new(context.clone(), payload.clone());
                    effect_handler
                        .send_message_to(port, duplicated)
                        .await
                        .map_err(|e| e.into())?;
                }

                Ok(())
            }
        }
    }
}
