// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use std::sync::Arc;

/// The URN for the noop exporter
pub const NOOP_EXPORTER_URN: &str = "urn:otel:noop:exporter";

/// Exporter that does nothing
pub struct NoopExporter;

/// Declare the Noop Exporter as a local exporter factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: NOOP_EXPORTER_URN,
    create: |node: NodeId, node_config: Arc<NodeUserConfig>, exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            NoopExporter::from_config()?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

impl NoopExporter {
    /// create a new instance of the `[NoopExporter]` from json config value
    pub fn from_config() -> Result<Self, otap_df_config::error::Error> {
        Ok(Self {})
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for NoopExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        _effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<(), Error<OtapPdata>> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                _ => {
                    // do nothing
                }
            }
        }

        Ok(())
    }
}
