// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic receiver.

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::TopicName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::topic::{
    SubscriptionGroupName, TopicAckPropagationMode, TopicBroadcastOnLagPolicy,
};
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{CallData, Context8u8, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::topic::{RecvItem, SubscriberOptions, Subscription, SubscriptionMode};
use otap_df_engine::{
    Interests, MessageSourceLocalEffectHandlerExtension, ProducerEffectHandlerExtension,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smallvec::smallvec;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// URN for the topic receiver.
pub const TOPIC_RECEIVER_URN: &str = "urn:otel:receiver:topic";

/// Telemetry metrics for the topic receiver.
#[metric_set(name = "topic.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct TopicReceiverMetrics {
    /// Number of messages forwarded to downstream.
    #[metric(unit = "{item}")]
    pub forwarded_messages: Counter<u64>,
    /// Number of forward failures to downstream channel.
    #[metric(unit = "{item}")]
    pub forward_failures: Counter<u64>,
    /// Number of lag notifications emitted by broadcast subscriptions.
    #[metric(unit = "{event}")]
    pub lagged_notifications: Counter<u64>,
    /// Total messages missed across lag notifications.
    #[metric(unit = "{item}")]
    pub lagged_messages: Counter<u64>,
    /// Number of broadcast subscriptions disconnected because of lag.
    #[metric(unit = "{event}")]
    pub lag_disconnects: Counter<u64>,
    /// Number of downstream backpressure events (>= 500ms blocked).
    #[metric(unit = "{event}")]
    pub downstream_backpressure_events: Counter<u64>,
    /// Total milliseconds blocked while forwarding to downstream.
    #[metric(unit = "ms")]
    pub downstream_blocked_ms: Counter<u64>,
    /// Number of downstream ACK controls successfully bridged to topic ack.
    #[metric(unit = "{item}")]
    pub bridged_downstream_acks: Counter<u64>,
    /// Number of downstream NACK controls successfully bridged to topic nack.
    #[metric(unit = "{item}")]
    pub bridged_downstream_nacks: Counter<u64>,
    /// Number of downstream ACK/NACK controls ignored because topic Ack/Nack
    /// propagation is disabled for this receiver.
    #[metric(unit = "{event}")]
    pub bridge_controls_ignored_propagation_disabled: Counter<u64>,
    /// Number of downstream ACK/NACK controls missing the bridged topic
    /// message id in calldata.
    #[metric(unit = "{event}")]
    pub bridge_missing_calldata: Counter<u64>,
    /// Number of downstream ACK/NACK controls carrying an id that is not
    /// currently tracked by the topic runtime.
    ///
    /// With the current raw `message_id` bridge this also includes invalid or
    /// forged ids; those causes are not distinguishable yet.
    #[metric(unit = "{event}")]
    pub bridge_invalid_or_untracked_id: Counter<u64>,
    /// Number of downstream ACK/NACK controls that failed to bridge for some
    /// runtime reason other than an unknown message id.
    #[metric(unit = "{event}")]
    pub bridge_runtime_failures: Counter<u64>,
}

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
    config: TopicReceiverConfig,
    subscription: Subscription<OtapPdata>,
    ack_propagation_mode: TopicAckPropagationMode,
    broadcast_on_lag: Option<TopicBroadcastOnLagPolicy>,
    metrics: MetricSet<TopicReceiverMetrics>,
}

/// Declares the topic receiver as a local receiver factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static TOPIC_RECEIVER: ReceiverFactory<OtapPdata> =
    ReceiverFactory {
        name: TOPIC_RECEIVER_URN,
        create: |pipeline: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 receiver_config: &ReceiverConfig| {
            let config = TopicReceiver::parse_config(&node_config.config)?;
            let topic_set = pipeline.topic_set::<OtapPdata>().ok_or_else(|| {
                ConfigError::InvalidUserConfig {
                    error: "Topic set is not available in pipeline context".to_owned(),
                }
            })?;
            let topic_binding = topic_set.get_required(&config.topic).map_err(|_| {
                ConfigError::InvalidUserConfig {
                    error: format!(
                        "Unknown topic `{}` for topic receiver (pipeline `{}`/`{}`)",
                        config.topic,
                        pipeline.pipeline_group_id(),
                        pipeline.pipeline_id(),
                    ),
                }
            })?;
            let mode = match &config.subscription {
                TopicSubscriptionConfig::Broadcast {} => SubscriptionMode::Broadcast,
                TopicSubscriptionConfig::Balanced { group } => SubscriptionMode::Balanced {
                    group: group.clone(),
                },
            };
            let subscription = topic_binding
                .subscribe(mode, SubscriberOptions::default())
                .map_err(|e| ConfigError::InvalidUserConfig {
                    error: format!(
                        "Failed to subscribe topic receiver to `{}`: {e}",
                        config.topic
                    ),
                })?;
            let ack_propagation_mode = topic_binding.default_ack_propagation_mode();
            let broadcast_on_lag =
                matches!(&config.subscription, TopicSubscriptionConfig::Broadcast {})
                    .then(|| topic_binding.broadcast_on_lag_policy());
            let metrics = pipeline
                .register_metrics_with_topic::<TopicReceiverMetrics>(topic_binding.name().into());
            Ok(ReceiverWrapper::local(
                TopicReceiver {
                    config,
                    subscription,
                    ack_propagation_mode,
                    broadcast_on_lag,
                    metrics,
                },
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

    fn decode_topic_message_id(calldata: &CallData) -> Option<u64> {
        calldata.first().map(|value| u64::from(*value))
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for TopicReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let TopicReceiver {
            config,
            mut subscription,
            ack_propagation_mode,
            broadcast_on_lag,
            mut metrics,
        } = *self;
        let subscription_mode = match &config.subscription {
            TopicSubscriptionConfig::Broadcast {} => "broadcast".to_owned(),
            TopicSubscriptionConfig::Balanced { group } => format!("balanced(group={})", group),
        };
        let receiver_id = effect_handler.receiver_id();
        otel_info!(
            "topic_receiver.start",
            node = receiver_id.name.as_ref(),
            topic = config.topic.as_ref(),
            subscription = subscription_mode,
            ack_propagation = format!("{ack_propagation_mode:?}"),
            message = "Topic receiver started"
        );
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;
        let mut draining_reason: Option<String> = None;
        let mut drained_notified = false;

        let run_result: Result<(), Error> = async {
            loop {
                tokio::select! {
                    biased;

                    ctrl = ctrl_msg_recv.recv() => {
                        match ctrl {
                            Ok(NodeControlMsg::CollectTelemetry {
                                mut metrics_reporter,
                            }) => {
                                _ = metrics_reporter.report(&mut metrics);
                            }
                            Ok(NodeControlMsg::Ack(ack)) => {
                                if ack_propagation_mode != TopicAckPropagationMode::Auto {
                                    metrics
                                        .bridge_controls_ignored_propagation_disabled
                                        .add(1);
                                } else if let Some(message_id) =
                                    Self::decode_topic_message_id(&ack.unwind.route.calldata)
                                {
                                    match subscription.ack(message_id) {
                                        Ok(()) => metrics.bridged_downstream_acks.add(1),
                                        Err(Error::MessageNotTracked) => {
                                            metrics.bridge_invalid_or_untracked_id.add(1);
                                            otel_warn!(
                                                "topic_receiver.bridge_ack_untracked_or_invalid_id",
                                                node = receiver_id.name.as_ref(),
                                                topic = config.topic.as_ref(),
                                                message_id = message_id,
                                                message = "Failed to ack topic message because the downstream control referenced an untracked or invalid message id"
                                            );
                                        }
                                        Err(e) => {
                                            metrics.bridge_runtime_failures.add(1);
                                            otel_warn!(
                                                "topic_receiver.bridge_ack_failed",
                                                node = receiver_id.name.as_ref(),
                                                topic = config.topic.as_ref(),
                                                error = e.to_string(),
                                                message = "Failed to ack topic message from downstream ack control"
                                            );
                                        }
                                    }
                                } else {
                                    metrics.bridge_missing_calldata.add(1);
                                    otel_warn!(
                                        "topic_receiver.bridge_ack_missing_calldata",
                                        node = receiver_id.name.as_ref(),
                                        topic = config.topic.as_ref(),
                                        message = "Downstream ack missing topic message id calldata"
                                    );
                                }
                            }
                            Ok(NodeControlMsg::DrainIngress { reason, .. }) => {
                                if !drained_notified {
                                    // TopicReceiver does not hold a separate wait_for_result
                                    // backlog like OTLP/OTAP receivers. Once ingress is marked as
                                    // draining, the runtime can consider the receiver drained
                                    // immediately; later topic deliveries are rejected/nacked
                                    // below instead of being admitted into the pipeline.
                                    draining_reason = Some(reason);
                                    effect_handler.notify_receiver_drained().await?;
                                    drained_notified = true;
                                }
                            }
                            Ok(NodeControlMsg::Nack(nack)) => {
                                if ack_propagation_mode != TopicAckPropagationMode::Auto {
                                    metrics
                                        .bridge_controls_ignored_propagation_disabled
                                        .add(1);
                                } else if let Some(message_id) =
                                    Self::decode_topic_message_id(&nack.unwind.route.calldata)
                                {
                                    match subscription.nack(message_id, nack.reason.as_str()) {
                                        Ok(()) => metrics.bridged_downstream_nacks.add(1),
                                        Err(Error::MessageNotTracked) => {
                                            metrics.bridge_invalid_or_untracked_id.add(1);
                                            otel_warn!(
                                                "topic_receiver.bridge_nack_untracked_or_invalid_id",
                                                node = receiver_id.name.as_ref(),
                                                topic = config.topic.as_ref(),
                                                message_id = message_id,
                                                message = "Failed to nack topic message because the downstream control referenced an untracked or invalid message id"
                                            );
                                        }
                                        Err(e) => {
                                            metrics.bridge_runtime_failures.add(1);
                                            otel_warn!(
                                                "topic_receiver.bridge_nack_failed",
                                                node = receiver_id.name.as_ref(),
                                                topic = config.topic.as_ref(),
                                                error = e.to_string(),
                                                message = "Failed to nack topic message from downstream nack control"
                                            );
                                        }
                                    }
                                } else {
                                    metrics.bridge_missing_calldata.add(1);
                                    otel_warn!(
                                        "topic_receiver.bridge_nack_missing_calldata",
                                        node = receiver_id.name.as_ref(),
                                        topic = config.topic.as_ref(),
                                        message = "Downstream nack missing topic message id calldata"
                                    );
                                }
                            }
                            Ok(NodeControlMsg::Shutdown { .. }) => break,
                            Ok(_) => {}
                            Err(e) => return Err(Error::ChannelRecvError(e)),
                        }
                    }

                    recv = subscription.recv() => {
                        match recv {
                            Ok(RecvItem::Message(env)) => {
                                if let Some(reason) = draining_reason.as_deref() {
                                    // DrainIngress was already acknowledged to the runtime, so any
                                    // message delivered after that point must be bounced at the
                                    // topic boundary rather than forwarded into the pipeline.
                                    if ack_propagation_mode == TopicAckPropagationMode::Auto
                                        && env.tracked
                                    {
                                        match subscription.nack(env.id, reason) {
                                            Ok(()) => metrics.bridged_downstream_nacks.add(1),
                                            Err(Error::MessageNotTracked) => {
                                                metrics.bridge_invalid_or_untracked_id.add(1);
                                            }
                                            Err(e) => {
                                                metrics.bridge_runtime_failures.add(1);
                                                otel_warn!(
                                                    "topic_receiver.drain_ingress_reject_failed",
                                                    node = receiver_id.name.as_ref(),
                                                    topic = config.topic.as_ref(),
                                                    error = e.to_string(),
                                                    message = "Failed to reject topic message while receiver was draining ingress"
                                                );
                                            }
                                        }
                                    }
                                    tokio::task::consume_budget().await;
                                    continue;
                                }

                                // Topic hop is a transport boundary: reset in-process
                                // Ack/Nack routing context before forwarding.
                                // Use source-tag-aware send so fan-in wiring can attribute source node.
                                let mut pdata = env.payload.clone_without_context();
                                if ack_propagation_mode == TopicAckPropagationMode::Auto
                                    && env.tracked
                                {
                                    let topic_message_calldata = smallvec![Context8u8::from(env.id)];
                                    effect_handler.subscribe_to(
                                        Interests::ACKS | Interests::NACKS,
                                        topic_message_calldata,
                                        &mut pdata,
                                    );
                                }
                                let send_started_at = Instant::now();
                                if let Err(e) = effect_handler.send_message_with_source_node(pdata).await {
                                    metrics.forward_failures.add(1);
                                    otel_warn!(
                                        "topic_receiver.forward_failed",
                                        node = receiver_id.name.as_ref(),
                                        topic = config.topic.as_ref(),
                                        error = e.to_string(),
                                        message = "Topic receiver failed forwarding to downstream channel"
                                    );
                                    return Err(Error::from(e));
                                }
                                metrics.forwarded_messages.add(1);
                                let blocked_for = send_started_at.elapsed();
                                if blocked_for.as_millis() >= 500 {
                                    metrics.downstream_backpressure_events.add(1);
                                    metrics
                                        .downstream_blocked_ms
                                        .add(blocked_for.as_millis() as u64);
                                    otel_warn!(
                                        "topic_receiver.downstream_backpressure",
                                        node = receiver_id.name.as_ref(),
                                        topic = config.topic.as_ref(),
                                        blocked_ms = blocked_for.as_millis() as u64,
                                        message = "Topic receiver blocked while forwarding to downstream pipeline channel"
                                    );
                                }
                                tokio::task::consume_budget().await;
                            }
                            Ok(RecvItem::Lagged { missed }) => {
                                metrics.lagged_notifications.add(1);
                                metrics.lagged_messages.add(missed);
                                if broadcast_on_lag == Some(TopicBroadcastOnLagPolicy::Disconnect) {
                                    metrics.lag_disconnects.add(1);
                                    otel_warn!(
                                        "topic_receiver.lag_disconnect",
                                        topic = config.topic.as_ref(),
                                        missed = missed,
                                        message = "Topic receiver lagged and will disconnect."
                                    );
                                } else {
                                    otel_warn!(
                                        "topic_receiver.lagged",
                                        topic = config.topic.as_ref(),
                                        missed = missed,
                                        message = "Topic receiver lagged and skipped messages."
                                    );
                                }
                                tokio::task::consume_budget().await;
                            }
                            Err(Error::SubscriptionClosed) => break,
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
            Ok(())
        }
        .await;

        _ = telemetry_cancel_handle.cancel().await;
        run_result?;
        Ok(TerminalState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{TOPIC_RECEIVER, TOPIC_RECEIVER_URN, TopicReceiver, TopicSubscriptionConfig};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_config::topic::TopicAckPropagationMode;
    use otap_df_engine::config::ReceiverConfig;
    use otap_df_engine::control::{
        AckMsg, Controllable, NodeControlMsg, pipeline_completion_msg_channel,
        runtime_ctrl_msg_channel,
    };
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::message::Sender as PDataSender;
    use otap_df_engine::node::NodeWithPDataSender;
    use otap_df_engine::testing::exporter::create_test_pipeline_context;
    use otap_df_engine::testing::{create_not_send_channel, setup_test_runtime, test_node};
    use otap_df_engine::topic::{
        PipelineTopicBinding, TopicBroadcastOnLagPolicy, TopicBroker, TopicOptions, TopicSet,
        TrackedPublishOutcome,
    };
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_otap::testing::{create_test_pdata, next_ack};
    use otap_df_telemetry::reporter::MetricsReporter;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

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

    #[test]
    fn bridges_downstream_ack_to_topic_outcome_when_enabled() {
        let (rt, local_tasks) = setup_test_runtime();
        rt.block_on(local_tasks.run_until(async move {
            let broker = TopicBroker::<OtapPdata>::new();
            let topic_name =
                otap_df_config::TopicName::parse("ingress").expect("topic name should parse");
            let base_handle = broker
                .create_in_memory_topic(
                    topic_name.clone(),
                    TopicOptions::Mixed {
                        balanced_capacity: 16,
                        broadcast_capacity: 16,
                        on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                    },
                )
                .expect("topic should be created");
            let receiver_handle = PipelineTopicBinding::from(base_handle.clone())
                .with_default_ack_propagation_mode(TopicAckPropagationMode::Auto);

            let receiver_set = TopicSet::new("receiver-set");
            _ = receiver_set.insert(topic_name.clone(), receiver_handle);

            let mut receiver_ctx = create_test_pipeline_context();
            receiver_ctx.set_topic_set(receiver_set);

            let receiver_node = test_node("topic_receiver");
            let mut receiver_user_cfg = NodeUserConfig::new_receiver_config(TOPIC_RECEIVER_URN);
            receiver_user_cfg.config = json!({
                "topic": "ingress",
                "subscription": {
                    "mode": "balanced",
                    "group": "sut-workers"
                }
            });

            let mut receiver = (TOPIC_RECEIVER.create)(
                receiver_ctx,
                receiver_node.clone(),
                Arc::new(receiver_user_cfg),
                &ReceiverConfig::new("topic_receiver"),
            )
            .expect("topic receiver should be created");

            let (receiver_output_tx, receiver_output_rx) = create_not_send_channel::<OtapPdata>(8);
            receiver
                .set_pdata_sender(
                    receiver_node.clone(),
                    "".into(),
                    PDataSender::Local(LocalSender::mpsc(receiver_output_tx)),
                )
                .expect("receiver output channel should be wired");

            let receiver_ctrl = receiver.control_sender();
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel::<OtapPdata>(32);
            let (pipeline_completion_tx, _pipeline_completion_rx) =
                pipeline_completion_msg_channel::<OtapPdata>(32);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
            let receiver_task = tokio::task::spawn_local(async move {
                receiver
                    .start(
                        runtime_ctrl_tx,
                        pipeline_completion_tx,
                        metrics_reporter,
                        otap_df_engine::Interests::empty(),
                    )
                    .await
            });

            let publisher = base_handle.tracked_publisher();
            let receipt = publisher
                .publish(Arc::new(create_test_pdata()))
                .await
                .expect("publish should succeed");

            let forwarded = tokio::time::timeout(Duration::from_secs(2), receiver_output_rx.recv())
                .await
                .expect("timed out waiting for receiver output")
                .expect("receiver output channel should stay open");

            let (_node_id, ack_for_receiver) = next_ack(AckMsg::new(forwarded))
                .expect("receiver should attach ack calldata for topic bridge");
            receiver_ctrl
                .send(NodeControlMsg::Ack(ack_for_receiver))
                .await
                .expect("failed to send ack control to topic receiver");

            let outcome = tokio::time::timeout(Duration::from_secs(2), receipt.wait_for_outcome())
                .await
                .expect("timed out waiting for tracked topic outcome");
            assert_eq!(outcome, TrackedPublishOutcome::Ack);

            receiver_ctrl
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("receiver shutdown should be sent");

            let receiver_result = receiver_task.await.expect("receiver task should join");
            assert!(receiver_result.is_ok(), "receiver should stop cleanly");
        }));
    }

    #[test]
    fn broadcast_receiver_stops_when_disconnected_on_lag() {
        let (rt, local_tasks) = setup_test_runtime();
        rt.block_on(local_tasks.run_until(async move {
            let broker = TopicBroker::<OtapPdata>::new();
            let topic_name =
                otap_df_config::TopicName::parse("ingress").expect("topic name should parse");
            let handle = broker
                .create_in_memory_topic(
                    topic_name.clone(),
                    TopicOptions::BroadcastOnly {
                        capacity: 4,
                        on_lag: TopicBroadcastOnLagPolicy::Disconnect,
                    },
                )
                .expect("topic should be created");

            let receiver_set = TopicSet::new("receiver-set");
            _ = receiver_set.insert(topic_name.clone(), handle.clone());

            let mut receiver_ctx = create_test_pipeline_context();
            receiver_ctx.set_topic_set(receiver_set);

            let receiver_node = test_node("topic_receiver");
            let mut receiver_user_cfg = NodeUserConfig::new_receiver_config(TOPIC_RECEIVER_URN);
            receiver_user_cfg.config = json!({
                "topic": "ingress",
                "subscription": { "mode": "broadcast" }
            });

            let mut receiver = (TOPIC_RECEIVER.create)(
                receiver_ctx,
                receiver_node.clone(),
                Arc::new(receiver_user_cfg),
                &ReceiverConfig::new("topic_receiver"),
            )
            .expect("topic receiver should be created");

            let (receiver_output_tx, receiver_output_rx) = create_not_send_channel::<OtapPdata>(1);
            receiver
                .set_pdata_sender(
                    receiver_node.clone(),
                    "".into(),
                    PDataSender::Local(LocalSender::mpsc(receiver_output_tx)),
                )
                .expect("receiver output channel should be wired");

            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel::<OtapPdata>(32);
            let (pipeline_completion_tx, _pipeline_completion_rx) =
                pipeline_completion_msg_channel::<OtapPdata>(32);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
            let receiver_task = tokio::task::spawn_local(async move {
                receiver
                    .start(
                        runtime_ctrl_tx,
                        pipeline_completion_tx,
                        metrics_reporter,
                        otap_df_engine::Interests::empty(),
                    )
                    .await
            });

            handle
                .publish(Arc::new(create_test_pdata()))
                .await
                .expect("initial publish should succeed");

            let _ = tokio::time::timeout(Duration::from_secs(2), receiver_output_rx.recv())
                .await
                .expect("timed out waiting for initial receiver output")
                .expect("receiver should forward at least one message before lagging");

            for _ in 0..32 {
                handle
                    .publish(Arc::new(create_test_pdata()))
                    .await
                    .expect("publish should succeed");
            }

            let receiver_result = tokio::time::timeout(Duration::from_secs(2), receiver_task)
                .await
                .expect("receiver should stop after lag disconnect")
                .expect("receiver task should join");
            assert!(
                receiver_result.is_ok(),
                "receiver should stop cleanly after lag disconnect"
            );
        }));
    }
}
