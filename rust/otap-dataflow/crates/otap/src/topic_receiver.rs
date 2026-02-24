// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic receiver.
//!
//! Note: This implementation is incomplete and only focus on the configuration.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::TopicName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::topic::SubscriptionGroupName;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// URN for the topic receiver.
pub const TOPIC_RECEIVER_URN: &str = "urn:otel:topic:receiver";

/// Topic receiver configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicReceiverConfig {
    /// Topic name to subscribe to.
    pub topic: TopicName,
    /// Subscription options for this receiver.
    #[serde(default)]
    pub subscription: TopicSubscriptionConfig,
}

/// Subscription mode for a topic receiver.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum TopicSubscriptionConfig {
    /// Each subscriber receives all messages.
    Broadcast {},
    /// Subscribers in the same consumer group share the stream.
    Balanced {
        /// Balanced consumer-group identifier.
        group: SubscriptionGroupName,
    },
}

impl Default for TopicSubscriptionConfig {
    fn default() -> Self {
        Self::Broadcast {}
    }
}

/// Receiver for topic subscriptions.
pub struct TopicReceiver {
    #[allow(dead_code)]
    config: TopicReceiverConfig,
}

/// Declares the topic receiver as a local receiver factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static TOPIC_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: TOPIC_RECEIVER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        let config = TopicReceiver::parse_config(&node_config.config)?;
        Ok(ReceiverWrapper::local(
            TopicReceiver { config },
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |config| TopicReceiver::parse_config(config).map(|_| ()),
};

impl TopicReceiver {
    /// Parses and validates topic receiver configuration.
    pub fn parse_config(config: &Value) -> Result<TopicReceiverConfig, ConfigError> {
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse topic receiver config: {e}"),
        })
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for TopicReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        _effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match ctrl_msg_recv.recv().await {
                Ok(NodeControlMsg::Shutdown { .. }) => break,
                Ok(_) => {}
                Err(e) => return Err(Error::ChannelRecvError(e)),
            }
        }
        Ok(TerminalState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{TopicReceiver, TopicSubscriptionConfig};
    use serde_json::json;

    #[test]
    fn parse_config_defaults_to_broadcast() {
        let cfg = TopicReceiver::parse_config(&json!({"topic": "raw"})).expect("valid config");
        assert_eq!(cfg.topic.as_ref(), "raw");
        assert_eq!(cfg.subscription, TopicSubscriptionConfig::Broadcast {});
    }

    #[test]
    fn parse_config_balanced_requires_group() {
        let err = TopicReceiver::parse_config(&json!({
            "topic": "raw",
            "subscription": { "mode": "balanced" }
        }))
        .expect_err("balanced mode without group should fail");
        assert!(err.to_string().contains("missing field `group`"));
    }

    #[test]
    fn parse_config_broadcast_forbids_group() {
        let err = TopicReceiver::parse_config(&json!({
            "topic": "raw",
            "subscription": { "mode": "broadcast", "group": "workers" }
        }))
        .expect_err("broadcast mode with group should fail");
        assert!(err.to_string().contains("unknown field `group`"));
    }

    #[test]
    fn parse_config_rejects_empty_topic_name() {
        let err = TopicReceiver::parse_config(&json!({"topic": "   "}))
            .expect_err("empty topic name should fail");
        assert!(err.to_string().contains("topic name must be non-empty"));
    }

    #[test]
    fn parse_config_rejects_empty_balanced_group_name() {
        let err = TopicReceiver::parse_config(&json!({
            "topic": "raw",
            "subscription": { "mode": "balanced", "group": "   " }
        }))
        .expect_err("empty group name should fail");
        assert!(
            err.to_string()
                .contains("subscription group name must be non-empty")
        );
    }
}
