// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Fanout processor that clones incoming pdata to every connected out port.

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::MessageSourceLocalEffectHandlerExtension;
use otel_arrow_dfe_engine::ProcessorFactory;
use otel_arrow_dfe_engine::config::ProcessorConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::local::processor::{EffectHandler, Processor};
use otel_arrow_dfe_engine::message::Message;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::processor::ProcessorWrapper;
use otel_arrow_dfe_otap::OTAP_PROCESSOR_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::metrics::MetricSet;
use otel_arrow_dfe_telemetry_macros::metric_set;
use std::sync::Arc;

/// URN that identifies the temporary fanout processor used in validation pipelines.
pub const FANOUT_PROCESSOR_URN: &str = "urn:otel:processor:fanout_temp";

#[metric_set(name = "processor.fanout")]
#[derive(Debug, Default, Clone)]
struct FanoutMetrics {
    /// Number of messages forwarded.
    forwarded: otel_arrow_dfe_telemetry::instrument::Counter<u64>,
}

struct FanoutProcessor {
    metrics: MetricSet<FanoutMetrics>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
/// Distributed-slice factory that registers the fanout processor with the engine.
pub static FANOUT_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: FANOUT_PROCESSOR_URN,
    create:
        |pipeline_ctx: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         processor_config: &ProcessorConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            let metrics = pipeline_ctx.register_metrics::<FanoutMetrics>();
            Ok(ProcessorWrapper::local(
                FanoutProcessor { metrics },
                node,
                node_config,
                processor_config,
            ))
        },
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract {
        output_fanout: otel_arrow_dfe_engine::wiring_contract::OutputFanoutRule::AtMostPerOutput(1),
    },
    validate_config: otel_arrow_dfe_config::validation::no_config,
};

#[async_trait(?Send)]
impl Processor<OtapPdata> for FanoutProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            Message::Control(NodeControlMsg::CollectTelemetry {
                mut metrics_reporter,
            }) => {
                let _ = metrics_reporter.report(&mut self.metrics);
            }
            Message::Control(_) => {}
            Message::PData(pdata) => {
                let ports = effect_handler.connected_ports();
                if ports.is_empty() {
                    effect_handler.send_message_with_source_node(pdata).await?;
                } else {
                    for port in ports {
                        effect_handler
                            .send_message_with_source_node_to(port, pdata.clone())
                            .await?;
                    }
                }
                self.metrics.forwarded.add(1);
            }
        }
        Ok(())
    }
}
