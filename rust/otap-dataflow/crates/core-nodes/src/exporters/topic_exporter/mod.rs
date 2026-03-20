// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic exporter.

use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};
use linkme::distributed_slice;
use otap_df_config::TopicName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::topic::{TopicAckPropagationMode, TopicQueueOnFullPolicy};
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::topic::{
    PublishOutcome, TopicHandle, TrackedPublishOutcome, TrackedTryPublishOutcome,
};
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

/// URN for the topic exporter.
pub const TOPIC_EXPORTER_URN: &str = "urn:otel:exporter:topic";

/// Telemetry metrics for the topic exporter.
#[metric_set(name = "topic.exporter.metrics")]
#[derive(Debug, Default, Clone)]
pub struct TopicExporterMetrics {
    /// Number of messages published to the topic.
    #[metric(unit = "{item}")]
    pub published_messages: Counter<u64>,
    /// Number of messages dropped due to queue full policy.
    #[metric(unit = "{item}")]
    pub dropped_messages_on_full: Counter<u64>,
    /// Number of end-to-end acks bridged back to upstream.
    #[metric(unit = "{item}")]
    pub end_to_end_acks: Counter<u64>,
    /// Number of end-to-end nacks bridged back to upstream.
    #[metric(unit = "{item}")]
    pub end_to_end_nacks: Counter<u64>,
    /// Number of messages rejected because tracked outcome capacity was exhausted.
    #[metric(unit = "{item}")]
    pub dropped_messages_on_outcome_capacity: Counter<u64>,
    /// Current number of tracked publishes waiting for a terminal outcome.
    ///
    /// Future: add a pending-bytes gauge once retained payload size accounting
    /// is available for tracked publishes.
    #[metric(unit = "{item}")]
    pub tracked_in_flight: Gauge<u64>,
    /// Number of tracked publishes that resolved by timeout.
    ///
    /// Future: add an outcome-latency histogram once histogram instruments are
    /// available in the telemetry layer.
    #[metric(unit = "{item}")]
    pub outcome_timeouts: Counter<u64>,
    /// Number of pending end-to-end messages nacked during shutdown.
    #[metric(unit = "{item}")]
    pub shutdown_nacks: Counter<u64>,
}

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
    topic: TopicHandle<OtapPdata>,
    queue_on_full: TopicQueueOnFullPolicy,
    ack_propagation_mode: TopicAckPropagationMode,
    metrics: MetricSet<TopicExporterMetrics>,
}

/// Declares the topic exporter as a local exporter factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static TOPIC_EXPORTER: ExporterFactory<OtapPdata> =
    ExporterFactory {
        name: TOPIC_EXPORTER_URN,
        create: |pipeline: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 exporter_config: &ExporterConfig| {
            let config = TopicExporter::parse_config(&node_config.config)?;
            let topic_set = pipeline.topic_set::<OtapPdata>().ok_or_else(|| {
                ConfigError::InvalidUserConfig {
                    error: "Topic set is not available in pipeline context".to_owned(),
                }
            })?;
            let topic_binding = topic_set.get_required(&config.topic).map_err(|_| {
                ConfigError::InvalidUserConfig {
                    error: format!(
                        "Unknown topic `{}` for topic exporter (pipeline `{}`/`{}`)",
                        config.topic,
                        pipeline.pipeline_group_id(),
                        pipeline.pipeline_id(),
                    ),
                }
            })?;
            let queue_on_full = config
                .queue_on_full
                .clone()
                .unwrap_or_else(|| topic_binding.default_queue_on_full());
            let ack_propagation_mode = topic_binding.default_ack_propagation_mode();
            let metrics = pipeline
                .register_metrics_with_topic::<TopicExporterMetrics>(topic_binding.name().into());
            let topic = topic_binding.into_handle();
            Ok(ExporterWrapper::local(
                TopicExporter {
                    topic,
                    queue_on_full,
                    ack_propagation_mode,
                    metrics,
                },
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
        let TopicExporter {
            topic,
            queue_on_full,
            ack_propagation_mode,
            mut metrics,
        } = *self;

        let mut pending_messages: HashMap<u64, OtapPdata> = HashMap::new();
        let mut pending_outcomes: FuturesUnordered<
            Pin<Box<dyn Future<Output = (u64, TrackedPublishOutcome)> + Send>>,
        > = FuturesUnordered::new();
        let tracked_publisher = (ack_propagation_mode == TopicAckPropagationMode::Auto)
            .then(|| topic.tracked_publisher());

        let exporter_id = effect_handler.exporter_id();
        otel_info!(
            "topic_exporter.start",
            node = exporter_id.name.as_ref(),
            topic = topic.name().as_ref(),
            queue_on_full = format!("{queue_on_full:?}"),
            ack_propagation = format!("{ack_propagation_mode:?}"),
            message = "Topic exporter started"
        );
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let run_result: Result<(), Error> = async {
            loop {
                tokio::select! {
                    biased;

                    maybe_outcome = pending_outcomes.next(), if !pending_outcomes.is_empty() => {
                        if let Some((message_id, outcome)) = maybe_outcome {
                            if let Some(data) = pending_messages.remove(&message_id) {
                                // Future: record tracked publish outcome latency here once
                                // histogram instruments are available.
                                match outcome {
                                    TrackedPublishOutcome::Ack => {
                                        metrics.end_to_end_acks.add(1);
                                        effect_handler.notify_ack(AckMsg::new(data)).await?;
                                    }
                                    TrackedPublishOutcome::Nack { reason } => {
                                        metrics.end_to_end_nacks.add(1);
                                        effect_handler
                                            .notify_nack(NackMsg::new(reason.as_ref(), data))
                                            .await?;
                                    }
                                    TrackedPublishOutcome::TimedOut => {
                                        metrics.outcome_timeouts.add(1);
                                        metrics.end_to_end_nacks.add(1);
                                        effect_handler
                                            .notify_nack(NackMsg::new(
                                                "topic publish outcome timed out",
                                                data,
                                            ))
                                            .await?;
                                    }
                                    TrackedPublishOutcome::TopicClosed => {
                                        metrics.end_to_end_nacks.add(1);
                                        effect_handler
                                            .notify_nack(NackMsg::new("topic closed", data))
                                            .await?;
                                    }
                                }
                            }
                        }
                    }

                    msg = msg_chan.recv() => match msg? {
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            mut metrics_reporter,
                        }) => {
                            metrics.tracked_in_flight.set(pending_messages.len() as u64);
                            _ = metrics_reporter.report(&mut metrics);
                        }
                        Message::Control(NodeControlMsg::Shutdown { .. }) => {
                            if !pending_messages.is_empty() {
                                for (_, data) in pending_messages.drain() {
                                    metrics.shutdown_nacks.add(1);
                                    effect_handler
                                        .notify_nack(NackMsg::new(
                                            "topic exporter shutdown before downstream ack",
                                            data,
                                        ))
                                        .await?;
                                }
                            }
                            break;
                        }
                        Message::PData(data) => {
                            // Topic hop is a transport boundary: do not propagate in-process
                            // Ack/Nack routing state (node ids/call data) across pipelines.
                            let published = Arc::new(data.clone_without_context());
                            let should_track_end_to_end = ack_propagation_mode == TopicAckPropagationMode::Auto
                                && data.has_ack_or_nack_interests();

                            if should_track_end_to_end {
                                let tracked_publisher = tracked_publisher
                                    .as_ref()
                                    .expect("tracked publisher should exist when ack propagation is auto");
                                match queue_on_full {
                                    TopicQueueOnFullPolicy::Block => {
                                        let receipt = tracked_publisher.publish(published).await?;
                                        let message_id = receipt.message_id();
                                        metrics.published_messages.add(1);
                                        _ = pending_messages.insert(message_id, data);
                                        pending_outcomes.push(Box::pin(async move {
                                            (message_id, receipt.wait_for_outcome().await)
                                        }));
                                    }
                                    TopicQueueOnFullPolicy::DropNewest => match tracked_publisher.try_publish(published)? {
                                        TrackedTryPublishOutcome::Published(receipt) => {
                                            let message_id = receipt.message_id();
                                            metrics.published_messages.add(1);
                                            _ = pending_messages.insert(message_id, data);
                                            pending_outcomes.push(Box::pin(async move {
                                                (message_id, receipt.wait_for_outcome().await)
                                            }));
                                        }
                                        TrackedTryPublishOutcome::DroppedOnFull => {
                                            metrics.dropped_messages_on_full.add(1);
                                            otel_warn!(
                                                "topic_exporter.drop_newest",
                                                node = exporter_id.name.as_ref(),
                                                topic = topic.name().as_ref(),
                                                message = "Dropping message because topic queue is full"
                                            );
                                            effect_handler
                                                .notify_nack(NackMsg::new(
                                                    "topic queue full: dropped newest",
                                                    data,
                                                ))
                                                .await?;
                                        }
                                        TrackedTryPublishOutcome::MaxInFlightReached => {
                                            metrics.dropped_messages_on_outcome_capacity.add(1);
                                            otel_warn!(
                                                "topic_exporter.outcome_capacity_full",
                                                node = exporter_id.name.as_ref(),
                                                topic = topic.name().as_ref(),
                                                message = "Dropping message because tracked publish outcome capacity is exhausted"
                                            );
                                            effect_handler
                                                .notify_nack(NackMsg::new(
                                                    "topic publish outcome capacity exhausted",
                                                    data,
                                                ))
                                                .await?;
                                        }
                                    },
                                }
                            } else {
                                let publish_result = match queue_on_full {
                                    TopicQueueOnFullPolicy::Block => {
                                        topic.publish(published).await?;
                                        Ok(PublishOutcome::Published)
                                    }
                                    TopicQueueOnFullPolicy::DropNewest => topic.try_publish(published),
                                };

                                match publish_result? {
                                    PublishOutcome::Published => {
                                        metrics.published_messages.add(1);
                                        effect_handler.notify_ack(AckMsg::new(data)).await?;
                                    }
                                    PublishOutcome::DroppedOnFull => {
                                        metrics.dropped_messages_on_full.add(1);
                                        otel_warn!(
                                            "topic_exporter.drop_newest",
                                            node = exporter_id.name.as_ref(),
                                            topic = topic.name().as_ref(),
                                            message = "Dropping message because topic queue is full"
                                        );
                                        effect_handler
                                            .notify_nack(NackMsg::new(
                                                "topic queue full: dropped newest",
                                                data,
                                            ))
                                            .await?;
                                    }
                                }
                            }
                            tokio::task::consume_budget().await;
                        }
                        _ => {}
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
    use super::{TOPIC_EXPORTER, TOPIC_EXPORTER_URN, TopicExporter};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_config::topic::{TopicAckPropagationMode, TopicQueueOnFullPolicy};
    use otap_df_engine::Interests;
    use otap_df_engine::config::ExporterConfig;
    use otap_df_engine::control::{
        Controllable, NodeControlMsg, PipelineReturnMsg, pipeline_ctrl_msg_channel,
        pipeline_return_msg_channel,
    };
    use otap_df_engine::local::message::LocalReceiver;
    use otap_df_engine::message::Receiver as PDataReceiver;
    use otap_df_engine::node::NodeWithPDataReceiver;
    use otap_df_engine::testing::exporter::create_test_pipeline_context;
    use otap_df_engine::testing::{create_not_send_channel, setup_test_runtime, test_node};
    use otap_df_engine::topic::{
        PipelineTopicBinding, SubscriberOptions, SubscriptionMode, TopicBroadcastOnLagPolicy,
        TopicBroker, TopicOptions, TopicSet,
    };
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_otap::testing::{TestCallData, create_test_pdata, next_ack};
    use otap_df_telemetry::reporter::MetricsReporter;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

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

    #[test]
    fn bridges_topic_ack_event_back_to_upstream_when_enabled() {
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
            let exporter_handle = PipelineTopicBinding::from(base_handle.clone())
                .with_default_ack_propagation_mode(TopicAckPropagationMode::Auto);

            let topic_set = TopicSet::new("exporter-set");
            _ = topic_set.insert(topic_name.clone(), exporter_handle);

            let mut exporter_ctx = create_test_pipeline_context();
            exporter_ctx.set_topic_set(topic_set);

            let exporter_node = test_node("topic_exporter");
            let mut exporter_user_cfg = NodeUserConfig::new_exporter_config(TOPIC_EXPORTER_URN);
            exporter_user_cfg.config = json!({
                "topic": "ingress",
                "queue_on_full": "block"
            });

            let mut exporter = (TOPIC_EXPORTER.create)(
                exporter_ctx,
                exporter_node.clone(),
                Arc::new(exporter_user_cfg),
                &ExporterConfig::new("topic_exporter"),
            )
            .expect("topic exporter should be created");

            let (exporter_input_tx, exporter_input_rx) = create_not_send_channel::<OtapPdata>(8);
            exporter
                .set_pdata_receiver(
                    exporter_node.clone(),
                    PDataReceiver::Local(LocalReceiver::mpsc(exporter_input_rx)),
                )
                .expect("exporter input channel should be wired");

            let exporter_ctrl = exporter.control_sender();
            let (pipeline_ctrl_tx, _pipeline_ctrl_rx) = pipeline_ctrl_msg_channel::<OtapPdata>(32);
            let (pipeline_return_tx, mut pipeline_return_rx) =
                pipeline_return_msg_channel::<OtapPdata>(32);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
            let exporter_task = tokio::task::spawn_local(async move {
                exporter
                    .start(
                        pipeline_ctrl_tx,
                        pipeline_return_tx,
                        metrics_reporter,
                        Interests::empty(),
                    )
                    .await
            });

            let mut subscriber = base_handle
                .subscribe(
                    SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    SubscriberOptions::default(),
                )
                .expect("topic subscriber should be created");

            let upstream_calldata = TestCallData::default();
            exporter_input_tx
                .send(create_test_pdata().test_subscribe_to(
                    Interests::ACKS,
                    upstream_calldata.clone().into(),
                    4242,
                ))
                .expect("failed to send pdata to topic exporter");

            let envelope = match tokio::time::timeout(Duration::from_secs(2), subscriber.recv())
                .await
                .expect("timed out waiting for topic message")
                .expect("topic subscription closed unexpectedly")
            {
                otap_df_engine::topic::RecvItem::Message(env) => env,
                other => panic!("unexpected topic receive item: {other:?}"),
            };
            subscriber
                .ack(envelope.id)
                .expect("topic ack should succeed");

            let delivered = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    let msg = pipeline_return_rx
                        .recv()
                        .await
                        .expect("pipeline return channel closed unexpectedly");
                    if matches!(msg, PipelineReturnMsg::DeliverAck { .. }) {
                        break msg;
                    }
                }
            })
            .await
            .expect("timed out waiting for upstream ack control");
            match delivered {
                PipelineReturnMsg::DeliverAck { ack } => {
                    let (node_id, ack) =
                        next_ack(ack).expect("ack should route to exporter subscriber");
                    assert_eq!(node_id, 4242);
                    let got: TestCallData = ack
                        .unwind
                        .route
                        .calldata
                        .try_into()
                        .expect("ack calldata should parse");
                    assert_eq!(got, upstream_calldata);
                }
                other => panic!("expected DeliverAck, got: {other:?}"),
            }

            exporter_ctrl
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("exporter shutdown should be sent");
            let exporter_result = exporter_task.await.expect("exporter task should join");
            assert!(exporter_result.is_ok(), "exporter should stop cleanly");
        }));
    }
}
