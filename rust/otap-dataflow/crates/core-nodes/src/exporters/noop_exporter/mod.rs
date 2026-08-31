// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::SignalType;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::views::otap::OtapLogsView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;

use std::sync::Arc;

/// The URN for the noop exporter.
pub const NOOP_EXPORTER_URN: &str = "urn:otel:exporter:noop";

/// Exporter that does nothing.
pub struct NoopExporterOld;

/// Declare the Noop Exporter as a local exporter factory.

// // TODO: is this called?
// #[allow(unsafe_code)]
// #[otap_df_engine::component_inventory(category = Exporter)]
// #[distributed_slice(OTAP_EXPORTER_FACTORIES)]
// pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
//     name: NOOP_EXPORTER_URN,
//     create: |_pipeline: PipelineContext,
//              node: NodeId,
//              node_config: Arc<NodeUserConfig>,
//              exporter_config: &ExporterConfig,
//              _capabilities: &otap_df_engine::capability::registry::Capabilities| {
//         Ok(ExporterWrapper::local(
//             NoopExporter {},
//             node,
//             node_config,
//             exporter_config,
//         ))
//     },
//     wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
//     validate_config: otap_df_config::validation::no_config,
// };

/*

#[async_trait(?Send)]
impl Exporter<OtapPdata> for NoopExporterOld {
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

 */

/// Console exporter that prints OTLP data to stdout.
pub struct NoopExporter {
}

/// Declare the Console Exporter as a local exporter factory
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: NOOP_EXPORTER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ExporterWrapper::local(
            NoopExporter{},
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::no_config,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for NoopExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {

        Ok(TerminalState::default())
    }
}

use tracing::Level;

impl NoopExporter {
    /// There is some sort of interaction which silently fails to add nodes
    /// to pipelines unless the node has 1+ method (arbitrary name) with a 
    /// non-obvious body. I have traced it down to tracing::event but need
    /// to go further to understand what's going on.
    async fn sentinel(&self, _payload: &OtapPayload) {
        tracing::event!(Level::DEBUG, "sentinel");
    }
}
