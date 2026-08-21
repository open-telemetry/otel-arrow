// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = NOOP_EXPORTER_URN,
    target = "otel.exporter.noop",
);

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::config::ExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use std::sync::Arc;

/// The URN for the noop exporter.
pub const NOOP_EXPORTER_URN: &str = "urn:otel:exporter:noop";

/// Exporter that does nothing.
pub struct NoopExporter;

/// Declare the Noop Exporter as a local exporter factory.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: NOOP_EXPORTER_URN,
    create:
        |_pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         exporter_config: &ExporterConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            Ok(ExporterWrapper::local(
                NoopExporter {},
                node,
                node_config,
                exporter_config,
            ))
        },
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otel_arrow_dfe_config::validation::no_config,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for NoopExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {
                    // do nothing
                }
            }
        }

        Ok(TerminalState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_otap::testing::{
        test_exporter_no_subscription, test_exporter_with_subscription,
    };
    use serde_json::json;

    #[test]
    fn test_noop_exporter_no_subscription() {
        test_exporter_no_subscription(&NOOP_EXPORTER, json!({}));
    }

    #[test]
    fn test_noop_exporter_with_subscription() {
        test_exporter_with_subscription(
            &NOOP_EXPORTER,
            json!({}),
            Interests::ACKS,
            Interests::ACKS,
        );
        test_exporter_with_subscription(
            &NOOP_EXPORTER,
            json!({}),
            Interests::ACKS | Interests::RETURN_DATA,
            Interests::ACKS,
        );
        test_exporter_with_subscription(
            &NOOP_EXPORTER,
            json!({}),
            Interests::NACKS,
            Interests::empty(),
        );
    }
}
