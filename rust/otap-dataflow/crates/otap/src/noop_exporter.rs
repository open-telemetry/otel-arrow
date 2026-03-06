// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// The URN for the noop exporter
pub const NOOP_EXPORTER_URN: &str = "urn:otel:exporter:noop";

/// Configuration for the noop exporter.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NoopExporterConfig {
    /// Optional delay before acknowledging each message.
    /// Useful for simulating a slow exporter to test backpressure behavior.
    /// Format: humantime (e.g., "500ms", "1s", "2s").
    /// Default: no delay (immediate ack).
    #[serde(default, with = "humantime_serde")]
    pub delay: Option<Duration>,
}

/// Exporter that does nothing, with an optional configurable delay before acking.
pub struct NoopExporter {
    delay: Option<Duration>,
}

/// Declare the Noop Exporter as a local exporter factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: NOOP_EXPORTER_URN,
    create: NoopExporter::create_exporter,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<NoopExporterConfig>,
};

impl NoopExporter {
    fn create_exporter(
        _pipeline: PipelineContext,
        node: NodeId,
        node_config: Arc<NodeUserConfig>,
        exporter_config: &ExporterConfig,
    ) -> Result<ExporterWrapper<OtapPdata>, otap_df_config::error::Error> {
        let config: NoopExporterConfig = serde_json::from_value(node_config.config.clone())
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                error: format!("Failed to parse noop-exporter configuration: {e}"),
            })?;

        Ok(ExporterWrapper::local(
            NoopExporter {
                delay: config.delay,
            },
            node,
            node_config,
            exporter_config,
        ))
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for NoopExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    if let Some(delay) = self.delay {
                        tokio::time::sleep(delay).await;
                    }
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
    use crate::testing::*;
    use otap_df_engine::Interests;
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
