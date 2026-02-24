// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic exporter.
//!
//! Note: This implementation is incomplete and only focus on the configuration.

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::TopicName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::topic::TopicQueueOnFullPolicy;
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
use serde_json::Value;
use std::sync::Arc;

/// URN for the topic exporter.
pub const TOPIC_EXPORTER_URN: &str = "urn:otel:topic:exporter";

/// Topic exporter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicExporterConfig {
    /// Topic name to publish to.
    pub topic: TopicName,
    /// Optional local override for publish behavior when topic queue is full.
    /// If omitted, runtime falls back to the topic declaration policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_on_full: Option<TopicQueueOnFullPolicy>,
}

/// Exporter for topic publishing.
pub struct TopicExporter {
    #[allow(dead_code)]
    config: TopicExporterConfig,
}

/// Declares the topic exporter as a local exporter factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static TOPIC_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: TOPIC_EXPORTER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        let config = TopicExporter::parse_config(&node_config.config)?;
        Ok(ExporterWrapper::local(
            TopicExporter { config },
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |config| TopicExporter::parse_config(config).map(|_| ()),
};

impl TopicExporter {
    /// Parses and validates topic exporter configuration.
    pub fn parse_config(config: &Value) -> Result<TopicExporterConfig, ConfigError> {
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse topic exporter config: {e}"),
        })
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for TopicExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    // Temp behavior: acknowledge input until topic runtime is wired in.
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::TopicExporter;
    use otap_df_config::topic::TopicQueueOnFullPolicy;
    use serde_json::json;

    #[test]
    fn parse_config_accepts_minimal_topic() {
        let cfg = TopicExporter::parse_config(&json!({"topic": "raw"})).expect("valid config");
        assert_eq!(cfg.topic.as_ref(), "raw");
        assert!(cfg.queue_on_full.is_none());
    }

    #[test]
    fn parse_config_accepts_local_queue_on_full_override() {
        let cfg = TopicExporter::parse_config(&json!({
            "topic": "raw",
            "queue_on_full": "drop_newest"
        }))
        .expect("valid config");
        assert_eq!(cfg.topic.as_ref(), "raw");
        assert_eq!(cfg.queue_on_full, Some(TopicQueueOnFullPolicy::DropNewest));
    }

    #[test]
    fn parse_config_rejects_unknown_queue_on_full_variant() {
        let err = TopicExporter::parse_config(&json!({
            "topic": "raw",
            "queue_on_full": "unknown_variant"
        }))
        .expect_err("unknown queue_on_full should fail");
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn parse_config_rejects_empty_topic() {
        let err = TopicExporter::parse_config(&json!({"topic": "   "}))
            .expect_err("empty topic should fail");
        assert!(err.to_string().contains("topic name must be non-empty"));
    }
}
