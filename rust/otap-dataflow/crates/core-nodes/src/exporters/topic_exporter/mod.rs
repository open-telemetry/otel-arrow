// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic exporter.

mod metrics;

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = TOPIC_EXPORTER_URN,
    target = "otel.exporter.topic",
);

use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};
use linkme::distributed_slice;
use metrics::{TopicExporterMetrics, TopicRejectionReason, TopicTrackedResult};
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::topic::{TopicAckPropagationMode, TopicQueueOnFullPolicy};
use otel_arrow_dfe_config::{SignalType, TopicName};
use otel_arrow_dfe_engine::config::ExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NackCause, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::runtime_services::CodecEffectHandler;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::topic::{
    PublishOutcome, TopicHandle, TrackedPublishOutcome, TrackedPublishReceipt,
    TrackedTopicPublisher, TrackedTryPublishOutcome,
};
use otel_arrow_dfe_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::metrics::ExporterAttempt;
use otel_arrow_dfe_otap::pdata::{OtapPdata, PdataDelivery, PdataEffectHandlerExtension};
use otel_arrow_dfe_pdata_codec::{CodecError, OtapPayload, PdataPayloadDecodeError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// URN for the topic exporter.
pub const TOPIC_EXPORTER_URN: &str = "urn:otel:exporter:topic";

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
    metrics: TopicExporterMetrics,
}

/// Delivery stays outside the future so shutdown can cancel work and still NACK upstream.
struct PendingPublish<'a> {
    delivery: PdataDelivery,
    attempt: ExporterAttempt,
    future: Pin<Box<dyn Future<Output = PublishProgress> + 'a>>,
}

enum PublishProgress {
    Prepared(Result<OtapPayload, PdataPayloadDecodeError>),
    Admitted(Result<Admission, Error>),
}

#[derive(Debug)]
enum TopicAttemptError {
    Runtime(Error),
    Rejected(TopicRejectionReason),
}

enum AdmissionStep {
    Wait(Arc<OtapPdata>),
    Complete(Result<Admission, TopicAttemptError>),
}

/// Successful topic admission, optionally with a downstream outcome to track.
enum Admission {
    Untracked,
    Tracked(TrackedPublishReceipt),
}

/// Declares the topic exporter as a local exporter factory.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static TOPIC_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: TOPIC_EXPORTER_URN,
    create:
        |pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         exporter_config: &ExporterConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
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
            if pipeline.input_requires_ack_propagation()
                && ack_propagation_mode == TopicAckPropagationMode::Disabled
            {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!(
                        "topic exporter `{}` has a direct source that requires downstream acknowledgements; set policies.ack_propagation.mode to auto for topic `{}`",
                        node.name, config.topic
                    ),
                });
            }
            if pipeline.input_requires_ack_propagation() && !topic_binding.is_balanced_only() {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!(
                        "topic exporter `{}` has a direct source requiring a single downstream acknowledgement; topic `{}` must use a balanced-only implementation",
                        node.name, config.topic
                    ),
                });
            }
            let metrics =
                TopicExporterMetrics::register(&pipeline, topic_binding.name().to_string());
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
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |config| TopicExporter::parse_config(config).map(|_| ()),
};

impl TopicExporter {
    /// Parses and validates topic exporter configuration.
    pub fn parse_config(config: &Value) -> Result<TopicExporterConfig, ConfigError> {
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse topic exporter config: {e}"),
        })
    }

    /// Convert one admitted tracked publish receipt into the exporter-owned
    /// pending outcome bookkeeping.
    fn record_tracked_publish(
        receipt: TrackedPublishReceipt,
        data: OtapPdata,
        pending_messages: &mut HashMap<u64, OtapPdata>,
        pending_outcomes: &mut FuturesUnordered<
            Pin<Box<dyn Future<Output = (u64, TrackedPublishOutcome)> + Send>>,
        >,
    ) {
        let message_id = receipt.message_id();
        _ = pending_messages.insert(message_id, data);
        pending_outcomes.push(Box::pin(async move {
            (message_id, receipt.wait_for_outcome().await)
        }));
    }

    /// Fail all publish work still owned by the exporter during shutdown.
    async fn flush_shutdown_pending(
        effect_handler: &EffectHandler<OtapPdata>,
        metrics: &mut TopicExporterMetrics,
        pending_publish: Option<PendingPublish<'_>>,
        pending_messages: &mut HashMap<u64, OtapPdata>,
    ) -> Result<(), Error> {
        if let Some(pending) = pending_publish {
            drop(pending.future);
            effect_handler
                .notify_nack(Self::shutdown_nack(
                    pending.delivery.into_pdata(),
                    pending.attempt,
                    metrics,
                ))
                .await?;
        }
        for (_, data) in pending_messages.drain() {
            metrics.record_tracked(data.signal_type(), TopicTrackedResult::Shutdown);
            effect_handler
                .notify_nack(NackMsg::new(
                    "topic exporter shutdown before downstream ack",
                    data,
                ))
                .await?;
        }
        Ok(())
    }

    /// Start one topic-owned blocked publish future after the immediate fast
    /// path has already reported backpressure.
    fn start_blocked_publish<'a>(
        delivery: PdataDelivery,
        published: Arc<OtapPdata>,
        attempt: ExporterAttempt,
        tracked: Option<&'a TrackedTopicPublisher<OtapPdata>>,
        topic: &'a TopicHandle<OtapPdata>,
    ) -> PendingPublish<'a> {
        PendingPublish {
            delivery,
            attempt,
            future: Box::pin(async move {
                let result = match tracked {
                    Some(publisher) => publisher.publish(published).await.map(Admission::Tracked),
                    None => topic
                        .publish(published)
                        .await
                        .map(|()| Admission::Untracked),
                };
                PublishProgress::Admitted(result)
            }),
        }
    }

    fn record_attempt_result<T>(
        metrics: &mut TopicExporterMetrics,
        signal: SignalType,
        attempt: ExporterAttempt,
        result: Result<T, TopicAttemptError>,
    ) -> Result<T, TopicAttemptError> {
        let classified = match result {
            Ok(value) => Ok(value),
            Err(error @ TopicAttemptError::Runtime(_)) => Err(attempt.failed(error)),
            Err(error @ TopicAttemptError::Rejected(_)) => Err(attempt.refused(error)),
        };
        let result = metrics.boundary.record(attempt.complete(classified));
        if let Err(TopicAttemptError::Rejected(reason)) = &result {
            metrics.record_rejection(signal, *reason);
        }
        result
    }

    fn preparation_nack(
        error: PdataPayloadDecodeError,
        delivery: PdataDelivery,
        mut attempt: ExporterAttempt,
        metrics: &mut TopicExporterMetrics,
    ) -> NackMsg<OtapPdata> {
        let (error, mut original) = error.into_parts();
        attempt.set_item_count_with(|| original.num_items() as u64);
        let data = delivery.with_payload(original);
        let reason = error.to_string();
        let failure =
            match error.operation_cause() {
                CodecError::InvalidInput { .. } => attempt.refused(
                    NackMsg::new_permanent_with_cause(reason, data, NackCause::Refused),
                ),
                CodecError::ServiceBusy => attempt.failed(NackMsg::new(reason, data)),
                _ => attempt.failed(NackMsg::new_permanent(reason, data)),
            };
        metrics
            .boundary
            .record(attempt.complete::<(), _>(Err(failure)))
            .expect_err("failed preparation must fail the publish attempt")
    }

    fn shutdown_nack(
        data: OtapPdata,
        attempt: ExporterAttempt,
        metrics: &mut TopicExporterMetrics,
    ) -> NackMsg<OtapPdata> {
        metrics.record_rejection(data.signal_type(), TopicRejectionReason::Shutdown);
        let nack = attempt.refused(NackMsg::new(
            "topic exporter shutdown before topic admission",
            data,
        ));
        metrics
            .boundary
            .record(attempt.complete::<(), _>(Err(nack)))
            .expect_err("shutdown publish must be refused")
    }

    async fn record_tracked_outcome(
        data: OtapPdata,
        outcome: TrackedPublishOutcome,
        effect_handler: &EffectHandler<OtapPdata>,
        metrics: &mut TopicExporterMetrics,
    ) -> Result<(), Error> {
        let signal = data.signal_type();
        match outcome {
            TrackedPublishOutcome::Ack => {
                metrics.record_tracked(signal, TopicTrackedResult::Ack);
                effect_handler.notify_ack(AckMsg::new(data)).await?;
            }
            TrackedPublishOutcome::Nack { reason } => {
                metrics.record_tracked(signal, TopicTrackedResult::Nack);
                effect_handler
                    .notify_nack(NackMsg::new(reason.as_ref(), data))
                    .await?;
            }
            TrackedPublishOutcome::TimedOut => {
                metrics.record_tracked(signal, TopicTrackedResult::Timeout);
                effect_handler
                    .notify_nack(NackMsg::new("topic publish outcome timed out", data))
                    .await?;
            }
            TrackedPublishOutcome::TopicClosed => {
                metrics.record_tracked(signal, TopicTrackedResult::TopicClosed);
                effect_handler
                    .notify_nack(NackMsg::new("topic closed", data))
                    .await?;
            }
        }
        Ok(())
    }

    /// Try admission once. The block policy retains a handle to the prepared batch for retry.
    fn try_admission(
        payload: OtapPayload,
        delivery: &PdataDelivery,
        tracked: Option<&TrackedTopicPublisher<OtapPdata>>,
        topic: &TopicHandle<OtapPdata>,
        queue_on_full: &TopicQueueOnFullPolicy,
    ) -> AdmissionStep {
        let published = Arc::new(OtapPdata::new(delivery.context().clone_detached(), payload));
        let retry =
            matches!(queue_on_full, TopicQueueOnFullPolicy::Block).then(|| published.clone());
        let result = match tracked {
            Some(publisher) => publisher.try_publish(published).map(|result| match result {
                TrackedTryPublishOutcome::Published(receipt) => Ok(Admission::Tracked(receipt)),
                TrackedTryPublishOutcome::DroppedOnFull => Err(TopicRejectionReason::QueueFull),
                TrackedTryPublishOutcome::MaxInFlightReached => {
                    Err(TopicRejectionReason::OutcomeCapacity)
                }
            }),
            None => topic.try_publish(published).map(|result| match result {
                PublishOutcome::Published => Ok(Admission::Untracked),
                PublishOutcome::DroppedOnFull => Err(TopicRejectionReason::QueueFull),
            }),
        };
        match result {
            Ok(Err(reason)) => match retry {
                Some(batch) => AdmissionStep::Wait(batch),
                None => AdmissionStep::Complete(Err(TopicAttemptError::Rejected(reason))),
            },
            Ok(Ok(admission)) => AdmissionStep::Complete(Ok(admission)),
            Err(error) => AdmissionStep::Complete(Err(TopicAttemptError::Runtime(error))),
        }
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for TopicExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
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
        // Keep at most one input outside the inbox during decoding or blocked admission.
        let mut pending_publish: Option<PendingPublish<'_>> = None;

        let exporter_id = effect_handler.exporter_id();
        otel_info!(
            "topic_exporter.start",
            node = exporter_id.name.as_ref(),
            topic = topic.name().as_ref(),
            queue_on_full = format!("{queue_on_full:?}"),
            ack_propagation = format!("{ack_propagation_mode:?}"),
            message = "Topic exporter started"
        );

        let shutdown_deadline = loop {
            let idle = pending_publish.is_none();
            let (delivery, mut attempt, progress) = tokio::select! {
                biased;
                maybe_outcome = pending_outcomes.next(), if !pending_outcomes.is_empty() => {
                    if let Some((message_id, outcome)) = maybe_outcome
                        && let Some(data) = pending_messages.remove(&message_id)
                    {
                        Self::record_tracked_outcome(
                            data,
                            outcome,
                            &effect_handler,
                            &mut metrics,
                        )
                        .await?;
                    }
                    continue;
                }

                msg = msg_chan.recv_when(idle) => match msg? {
                    Message::Control(NodeControlMsg::CollectTelemetry {
                        mut metrics_reporter,
                    }) => {
                        metrics.set_tracked_in_flight(pending_messages.len());
                        _ = metrics.report(&mut metrics_reporter);
                        continue;
                    }
                    Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                        Self::flush_shutdown_pending(
                            &effect_handler,
                            &mut metrics,
                            pending_publish.take(),
                            &mut pending_messages,
                        )
                        .await?;
                        break deadline;
                    }
                    Message::Control(_) => continue,
                    Message::PData(mut data) if !idle => {
                        // The inbox can force-drain buffered inputs during shutdown.
                        let signal = data.signal_type();
                        let mut attempt = metrics.boundary.attempt(signal);
                        attempt.set_item_count_with(|| data.num_items() as u64);
                        effect_handler
                            .notify_nack(Self::shutdown_nack(data, attempt, &mut metrics))
                            .await?;
                        continue;
                    }
                    Message::PData(data) => {
                        let attempt = metrics.boundary.attempt(data.signal_type());
                        let (delivery, payload) = data.into_work();
                        if effect_handler.codec_service().requires_pipeline_materialization(&payload) {
                            pending_publish = Some(PendingPublish {
                                delivery,
                                attempt,
                                future: Box::pin(async {
                                    let result = effect_handler.materialize_otap_payload(payload).await;
                                    PublishProgress::Prepared(result.map(OtapPayload::from))
                                }),
                            });
                            continue;
                        }
                        (delivery, attempt, PublishProgress::Prepared(Ok(payload)))
                    }
                },

                progress = async {
                    pending_publish.as_mut().expect("active publish").future.as_mut().await
                }, if !idle => {
                    let pending = pending_publish.take().expect("active publish completed");
                    (pending.delivery, pending.attempt, progress)
                }
            };
            let signal = delivery.signal_type();
            let admission = match progress {
                PublishProgress::Prepared(Err(error)) => {
                    let nack = Self::preparation_nack(error, delivery, attempt, &mut metrics);
                    effect_handler.notify_nack(nack).await?;
                    tokio::task::consume_budget().await;
                    continue;
                }
                PublishProgress::Prepared(Ok(mut payload)) => {
                    attempt.set_item_count_with(|| payload.num_items() as u64);
                    let tracked = tracked_publisher
                        .as_ref()
                        .filter(|_| delivery.context().has_ack_or_nack_subscribers());
                    match Self::try_admission(payload, &delivery, tracked, &topic, &queue_on_full) {
                        AdmissionStep::Wait(batch) => {
                            pending_publish = Some(Self::start_blocked_publish(
                                delivery, batch, attempt, tracked, &topic,
                            ));
                            continue;
                        }
                        AdmissionStep::Complete(result) => result,
                    }
                }
                PublishProgress::Admitted(result) => result.map_err(TopicAttemptError::Runtime),
            };
            let data = delivery.into_pdata();
            match Self::record_attempt_result(&mut metrics, signal, attempt, admission) {
                Ok(Admission::Untracked) => effect_handler.notify_ack(AckMsg::new(data)).await?,
                Ok(Admission::Tracked(receipt)) => Self::record_tracked_publish(
                    receipt,
                    data,
                    &mut pending_messages,
                    &mut pending_outcomes,
                ),
                Err(TopicAttemptError::Runtime(error)) => return Err(error),
                Err(TopicAttemptError::Rejected(reason)) => {
                    let message = match reason {
                        TopicRejectionReason::QueueFull => {
                            otel_warn!(
                                "topic_exporter.drop_newest",
                                node = exporter_id.name.as_ref(),
                                topic = topic.name().as_ref(),
                                message = "Dropping message because topic queue is full"
                            );
                            "topic queue full: dropped newest"
                        }
                        TopicRejectionReason::OutcomeCapacity => {
                            otel_warn!(
                                "topic_exporter.outcome_capacity_full",
                                node = exporter_id.name.as_ref(),
                                topic = topic.name().as_ref(),
                                message = "Dropping message because tracked publish outcome capacity is exhausted"
                            );
                            "topic publish outcome capacity exhausted"
                        }
                        TopicRejectionReason::Shutdown => {
                            "topic exporter shutdown before topic admission"
                        }
                    };
                    effect_handler
                        .notify_nack(NackMsg::new(message, data))
                        .await?;
                }
            }
            tokio::task::consume_budget().await;
        };

        metrics.set_tracked_in_flight(0);
        Ok(TerminalState::new(
            shutdown_deadline,
            metrics.terminal_snapshots(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{TOPIC_EXPORTER, TOPIC_EXPORTER_URN, TopicExporter};
    use futures::StreamExt;
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_config::topic::{TopicAckPropagationMode, TopicQueueOnFullPolicy};
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::config::ExporterConfig;
    use otel_arrow_dfe_engine::control::{
        Controllable, NodeControlMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
        runtime_ctrl_msg_channel,
    };
    use otel_arrow_dfe_engine::local::message::LocalReceiver;
    use otel_arrow_dfe_engine::message::Receiver as PDataReceiver;
    use otel_arrow_dfe_engine::node::NodeWithPDataReceiver;
    use otel_arrow_dfe_engine::testing::exporter::create_test_pipeline_context;
    use otel_arrow_dfe_engine::testing::{create_not_send_channel, setup_test_runtime, test_node};
    use otel_arrow_dfe_engine::topic::{
        PipelineTopicBinding, SubscriberOptions, SubscriptionMode, TopicBroadcastAckMode,
        TopicBroadcastOnLagPolicy, TopicBroker, TopicOptions, TopicSet,
    };
    use otel_arrow_dfe_otap::pdata::OtapPdata;
    use otel_arrow_dfe_otap::testing::{TestCallData, create_test_pdata, next_ack, next_nack};
    use otel_arrow_dfe_pdata::OtapArrowRecords;
    use otel_arrow_dfe_pdata_codec::{
        CodecError, CodecMetadata, CodecRegistration, DecodeBudget, DecodeTask, PdataDecodeJob,
        PdataDecoder, PdataEncoding, register_pdata_codec,
    };
    use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    fn metric_value(
        snapshots: &[MetricSetSnapshot],
        set_name: &str,
        metric_name: &str,
        attributes: &[(&str, &str)],
    ) -> u64 {
        let snapshot = snapshots
            .iter()
            .find(|snapshot| {
                snapshot.descriptor().name == set_name
                    && attributes.iter().all(|(key, value)| {
                        snapshot.measurement_attribute_value(key) == Some(*value)
                    })
            })
            .expect("matching metric snapshot");
        let metric = snapshot
            .descriptor()
            .metrics
            .iter()
            .position(|metric| metric.name == metric_name)
            .expect("metric exists");
        snapshot.get_metrics()[metric].to_u64_lossy()
    }

    /// Scenario: A topic exporter specifies only its required topic name.
    /// Guarantees: Minimal configuration preserves the topic and inherits queue policy.
    #[test]
    fn parse_config_accepts_minimal_topic() {
        let cfg = TopicExporter::parse_config(&json!({"topic": "raw"})).expect("valid config");
        assert_eq!(cfg.topic.as_ref(), "raw");
        assert!(cfg.queue_on_full.is_none());
    }

    /// Scenario: A topic exporter overrides the topic queue-full policy.
    /// Guarantees: The configured bounded policy is retained for runtime use.
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

    /// Scenario: A topic exporter uses an unsupported queue-full policy value.
    /// Guarantees: Configuration rejects unknown variants before runtime startup.
    #[test]
    fn parse_config_rejects_unknown_queue_on_full_variant() {
        let err = TopicExporter::parse_config(&json!({
            "topic": "raw",
            "queue_on_full": "unknown_variant"
        }))
        .expect_err("unknown queue_on_full should fail");
        assert!(err.to_string().contains("unknown variant"));
    }

    /// Scenario: A topic exporter is configured with an empty topic name.
    /// Guarantees: Configuration rejects the invalid topic before runtime startup.
    #[test]
    fn parse_config_rejects_empty_topic() {
        let err = TopicExporter::parse_config(&json!({"topic": "   "}))
            .expect_err("empty topic should fail");
        assert!(err.to_string().contains("topic name must be non-empty"));
    }

    // Readiness belongs to the exporter task in this test, including when tests run concurrently.
    tokio::task_local! {
        static DECODE_STARTED: Arc<tokio::sync::Notify>;
    }

    const TEST_LOGS: PdataEncoding = PdataEncoding::new("test.topic-incremental-logs");
    static TEST_METADATA: CodecMetadata = CodecMetadata::new(TEST_LOGS, &[SignalType::Logs]);
    register_pdata_codec!(
        TOPIC_TEST_CODEC,
        CodecRegistration::new(&TEST_METADATA)
            .with_decoder(|_| Box::new(TestDecoder))
            .with_item_counter(|_, _| Some(1)),
    );

    struct TestDecoder;

    impl PdataDecoder for TestDecoder {
        fn start_decode(
            &mut self,
            signal: SignalType,
            bytes: &bytes::Bytes,
        ) -> Result<DecodeTask, CodecError> {
            DECODE_STARTED.with(|started| started.notify_one());
            if bytes.as_ref() == b"pending" {
                return Ok(DecodeTask::Incremental(Box::new(PendingJob)));
            }
            self.decode(signal, bytes).map(DecodeTask::Ready)
        }

        fn decode(
            &mut self,
            _: SignalType,
            bytes: &bytes::Bytes,
        ) -> Result<OtapArrowRecords, CodecError> {
            assert_eq!(bytes.as_ref(), b"ready");
            Ok(otel_arrow_dfe_pdata::encode::encode_logs_otap_batch(
                &otel_arrow_dfe_pdata::testing::fixtures::log_with_no_scope(),
            )
            .unwrap())
        }
    }

    struct PendingJob;

    impl PdataDecodeJob for PendingJob {
        fn step(&mut self, _: DecodeBudget) -> Result<Option<OtapArrowRecords>, CodecError> {
            Ok(None)
        }
    }

    /// Scenario: A full topic drains a prepared batch, then an incremental decode overlaps its subscriber ACK.
    /// Guarantees: Prepared records survive backpressure, ACK-only bytes are released, and controls and ACKs progress before shutdown returns the pending input.
    #[test]
    fn tracked_ack_and_shutdown_progress_during_incremental_decode() {
        let rt = setup_test_runtime();
        rt.block_on(async move {
            let broker = TopicBroker::<OtapPdata>::new();
            let topic_name = otel_arrow_dfe_config::TopicName::parse("ingress")
                .expect("topic name should parse");
            let base_handle = broker
                .create_in_memory_topic(
                    topic_name.clone(),
                    TopicOptions::Mixed {
                        balanced_capacity: 1,
                        broadcast_capacity: 16,
                        on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                        ack_mode: TopicBroadcastAckMode::First,
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
                &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
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
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel::<OtapPdata>(32);
            let (pipeline_completion_tx, mut pipeline_completion_rx) =
                pipeline_completion_msg_channel::<OtapPdata>(32);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
            let started = Arc::new(tokio::sync::Notify::new());
            let services = otel_arrow_dfe_engine::testing::test_pipeline_runtime_services();
            let codec = services.codecs().registry().resolve(&TEST_LOGS).unwrap();
            let input = |bytes| {
                OtapPdata::new_default(codec.admit(SignalType::Logs, bytes).unwrap().into())
            };
            let exporter_task =
                tokio::task::spawn_local(DECODE_STARTED.scope(started.clone(), async move {
                    exporter
                        .start(
                            runtime_ctrl_tx,
                            pipeline_completion_tx,
                            metrics_reporter,
                            Interests::empty(),
                            services,
                        )
                        .await
                }));

            let mut subscriber = base_handle
                .subscribe(
                    SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    SubscriberOptions::default(),
                )
                .expect("topic subscriber should be created");

            assert!(matches!(
                base_handle
                    .try_publish(Arc::new(create_test_pdata()))
                    .unwrap(),
                otel_arrow_dfe_engine::topic::PublishOutcome::Published
            ));
            let upstream_calldata = TestCallData::default();
            let owner = Arc::<[u8]>::from(&b"ready"[..]);
            let original = Arc::downgrade(&owner);
            exporter_input_tx
                .send(input(bytes::Bytes::from_owner(owner)).test_subscribe_to(
                    Interests::ACKS,
                    upstream_calldata.clone().into(),
                    4242,
                ))
                .expect("failed to send pdata to topic exporter");

            tokio::time::timeout(Duration::from_secs(30), started.notified())
                .await
                .expect("the ready input must be decoded while the topic queue is full");
            assert!(
                original.upgrade().is_none(),
                "ACK-only delivery retained its encoded input"
            );
            let receive =
                async |subscriber: &mut otel_arrow_dfe_engine::topic::Subscription<OtapPdata>| {
                    let item = tokio::time::timeout(Duration::from_secs(30), subscriber.recv())
                        .await
                        .expect("timed out waiting for topic message")
                        .expect("topic closed");
                    let otel_arrow_dfe_engine::topic::RecvItem::Message(envelope) = item else {
                        panic!("expected topic message");
                    };
                    envelope
                };
            let filler = receive(&mut subscriber).await;
            assert!(!filler.tracked);
            let envelope = receive(&mut subscriber).await;
            assert!(envelope.tracked);
            assert_eq!(
                envelope.payload.payload_ref().otap_ref(),
                Some(
                    &otel_arrow_dfe_pdata::encode::encode_logs_otap_batch(
                        &otel_arrow_dfe_pdata::testing::fixtures::log_with_no_scope()
                    )
                    .unwrap()
                )
            );
            let pending_bytes = bytes::Bytes::from_static(b"pending");
            exporter_input_tx
                .send(input(pending_bytes.clone()).test_subscribe_to(
                    Interests::NACKS | Interests::RETURN_DATA,
                    TestCallData::default().into(),
                    4343,
                ))
                .unwrap();
            tokio::time::timeout(Duration::from_secs(30), started.notified())
                .await
                .expect("the second decode must reach a pending step");

            let (snapshots, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
            exporter_ctrl
                .send(NodeControlMsg::CollectTelemetry { metrics_reporter })
                .await
                .unwrap();
            let report = tokio::time::timeout(
                Duration::from_secs(30),
                snapshots.into_stream().collect::<Vec<_>>(),
            )
            .await
            .expect("telemetry must progress during the pending decode");
            assert_eq!(
                metric_value(&report, "exporter.topic", "tracked.in.flight", &[]),
                1
            );

            subscriber
                .ack(envelope.id)
                .expect("topic ack should succeed");

            let delivered =
                tokio::time::timeout(Duration::from_secs(30), pipeline_completion_rx.recv())
                    .await
                    .expect("the tracked ACK must progress during the pending decode")
                    .expect("pipeline completion channel closed");
            match delivered {
                PipelineCompletionMsg::DeliverAck { ack } => {
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
            let terminal = tokio::time::timeout(Duration::from_secs(30), exporter_task)
                .await
                .expect("shutdown must cancel the pending decode")
                .expect("exporter task should join")
                .expect("exporter should stop cleanly");
            let PipelineCompletionMsg::DeliverNack { nack } =
                pipeline_completion_rx.recv().await.unwrap()
            else {
                panic!("expected the pending delivery to be NACKed on shutdown");
            };
            let (node, nack) = next_nack(nack).expect("original subscriber must receive the NACK");
            assert_eq!(node, 4343);
            assert!(
                pipeline_completion_rx.recv().await.is_err(),
                "unexpected duplicate completion"
            );
            assert_eq!(
                nack.refused.payload_ref().encoded_bytes(),
                Some(&pending_bytes)
            );
            assert_eq!(
                metric_value(
                    terminal.metrics(),
                    "exporter.topic.rejections",
                    "messages",
                    &[("reason", "shutdown")]
                ),
                1
            );
            assert_eq!(
                metric_value(
                    terminal.metrics(),
                    "exporter.topic.tracked",
                    "messages",
                    &[("result", "ack")],
                ),
                1
            );
        });
    }

    /// Scenario: shutdown reaches a topic exporter while one untracked publish
    /// is still waiting for queue space under `queue_on_full: block`.
    /// Guarantees: the exporter stops retrying that publish, nacks the blocked
    /// upstream pdata, and exits promptly.
    #[test]
    fn shutdown_interrupts_blocked_untracked_publish_with_block_policy() {
        let rt = setup_test_runtime();
        rt.block_on(async move {
            let broker = TopicBroker::<OtapPdata>::new();
            let topic_name = otel_arrow_dfe_config::TopicName::parse("ingress")
                .expect("topic name should parse");
            let handle = broker
                .create_in_memory_topic(
                    topic_name.clone(),
                    TopicOptions::Mixed {
                        balanced_capacity: 1,
                        broadcast_capacity: 1,
                        on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                        ack_mode: TopicBroadcastAckMode::First,
                    },
                )
                .expect("topic should be created");

            let topic_set = TopicSet::new("exporter-set");
            _ = topic_set.insert(
                topic_name.clone(),
                PipelineTopicBinding::from(handle.clone()),
            );

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
                &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
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
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel::<OtapPdata>(32);
            let (pipeline_completion_tx, mut pipeline_completion_rx) =
                pipeline_completion_msg_channel::<OtapPdata>(32);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
            let exporter_task = tokio::task::spawn_local(async move {
                exporter
                    .start(
                        runtime_ctrl_tx,
                        pipeline_completion_tx,
                        metrics_reporter,
                        Interests::empty(),
                        otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
                    )
                    .await
            });

            let _subscriber = handle
                .subscribe(
                    SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    SubscriberOptions::default(),
                )
                .expect("topic subscriber should be created");

            let first_call_data = TestCallData::new_with(1, 1);
            exporter_input_tx
                .send(create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    first_call_data.clone().into(),
                    1001,
                ))
                .expect("failed to send first pdata to topic exporter");

            let first_completion = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    if let PipelineCompletionMsg::DeliverAck { ack } = pipeline_completion_rx
                        .recv()
                        .await
                        .expect("pipeline-completion channel closed unexpectedly")
                    {
                        break ack;
                    }
                }
            })
            .await
            .expect("timed out waiting for first ack");
            let (node_id, ack) = next_ack(first_completion).expect("ack should be routable");
            assert_eq!(node_id, 1001);
            let got: TestCallData = ack
                .unwind
                .route
                .calldata
                .try_into()
                .expect("ack calldata should parse");
            assert_eq!(got, first_call_data);

            let second_call_data = TestCallData::new_with(2, 2);
            exporter_input_tx
                .send(create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    second_call_data.clone().into(),
                    1002,
                ))
                .expect("failed to send second pdata to topic exporter");

            tokio::time::sleep(Duration::from_millis(100)).await;
            exporter_ctrl
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("exporter shutdown should be sent");

            let second_completion = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    if let PipelineCompletionMsg::DeliverNack { nack } = pipeline_completion_rx
                        .recv()
                        .await
                        .expect("pipeline-completion channel closed unexpectedly")
                    {
                        break nack;
                    }
                }
            })
            .await
            .expect("timed out waiting for second nack");
            let (node_id, nack) = next_nack(second_completion).expect("nack should be routable");
            assert_eq!(node_id, 1002);
            let got: TestCallData = nack
                .unwind
                .route
                .calldata
                .try_into()
                .expect("nack calldata should parse");
            assert_eq!(got, second_call_data);
            assert!(
                nack.reason
                    .contains("topic exporter shutdown before topic admission")
            );

            let terminal = tokio::time::timeout(Duration::from_secs(2), exporter_task)
                .await
                .expect("exporter should exit promptly after shutdown")
                .expect("exporter task should join")
                .expect("exporter should stop cleanly");
            assert_eq!(
                metric_value(
                    terminal.metrics(),
                    "exporter.topic.rejections",
                    "messages",
                    &[("reason", "shutdown")],
                ),
                1
            );
        });
    }

    /// Scenario: shutdown reaches a topic exporter while one untracked publish
    /// is blocked in the topic runtime and a second buffered pdata is force-
    /// drained from the exporter inbox during shutdown.
    /// Guarantees: both non-admitted pdata messages are nacked instead of
    /// panicking the exporter loop, and the exporter exits cleanly.
    #[test]
    fn shutdown_nacks_force_drained_buffered_pdata_while_blocked_publish_is_in_flight() {
        let rt = setup_test_runtime();
        rt.block_on(async move {
            let broker = TopicBroker::<OtapPdata>::new();
            let topic_name = otel_arrow_dfe_config::TopicName::parse("ingress")
                .expect("topic name should parse");
            let handle = broker
                .create_in_memory_topic(
                    topic_name.clone(),
                    TopicOptions::Mixed {
                        balanced_capacity: 1,
                        broadcast_capacity: 1,
                        on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                        ack_mode: TopicBroadcastAckMode::First,
                    },
                )
                .expect("topic should be created");

            let topic_set = TopicSet::new("exporter-set");
            _ = topic_set.insert(
                topic_name.clone(),
                PipelineTopicBinding::from(handle.clone()),
            );

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
                &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
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
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel::<OtapPdata>(32);
            let (pipeline_completion_tx, mut pipeline_completion_rx) =
                pipeline_completion_msg_channel::<OtapPdata>(32);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
            let exporter_task = tokio::task::spawn_local(async move {
                exporter
                    .start(
                        runtime_ctrl_tx,
                        pipeline_completion_tx,
                        metrics_reporter,
                        Interests::empty(),
                        otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
                    )
                    .await
            });

            let _subscriber = handle
                .subscribe(
                    SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    SubscriberOptions::default(),
                )
                .expect("topic subscriber should be created");

            let first_call_data = TestCallData::new_with(1, 1);
            exporter_input_tx
                .send(create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    first_call_data.clone().into(),
                    3001,
                ))
                .expect("failed to send first pdata to topic exporter");

            let first_completion = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    if let PipelineCompletionMsg::DeliverAck { ack } = pipeline_completion_rx
                        .recv()
                        .await
                        .expect("pipeline-completion channel closed unexpectedly")
                    {
                        break ack;
                    }
                }
            })
            .await
            .expect("timed out waiting for first ack");
            let (node_id, ack) = next_ack(first_completion).expect("ack should be routable");
            assert_eq!(node_id, 3001);
            let got: TestCallData = ack
                .unwind
                .route
                .calldata
                .try_into()
                .expect("ack calldata should parse");
            assert_eq!(got, first_call_data);

            let second_call_data = TestCallData::new_with(2, 2);
            exporter_input_tx
                .send(create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    second_call_data.clone().into(),
                    3002,
                ))
                .expect("failed to send second pdata to topic exporter");

            let third_call_data = TestCallData::new_with(3, 3);
            exporter_input_tx
                .send(create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    third_call_data.clone().into(),
                    3003,
                ))
                .expect("failed to send third pdata to topic exporter");

            tokio::time::sleep(Duration::from_millis(100)).await;
            exporter_ctrl
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("exporter shutdown should be sent");

            let mut seen = std::collections::HashMap::new();
            while seen.len() < 2 {
                let nack = tokio::time::timeout(Duration::from_secs(2), async {
                    loop {
                        if let PipelineCompletionMsg::DeliverNack { nack } = pipeline_completion_rx
                            .recv()
                            .await
                            .expect("pipeline-completion channel closed unexpectedly")
                        {
                            break nack;
                        }
                    }
                })
                .await
                .expect("timed out waiting for shutdown nacks");
                let (node_id, nack) = next_nack(nack).expect("nack should be routable");
                let got: TestCallData = nack
                    .unwind
                    .route
                    .calldata
                    .try_into()
                    .expect("nack calldata should parse");
                _ = seen.insert(node_id, got);
                assert!(
                    nack.reason
                        .contains("topic exporter shutdown before topic admission")
                );
            }

            assert_eq!(seen.get(&3002), Some(&second_call_data));
            assert_eq!(seen.get(&3003), Some(&third_call_data));

            let exporter_result = tokio::time::timeout(Duration::from_secs(2), exporter_task)
                .await
                .expect("exporter should exit promptly after shutdown")
                .expect("exporter task should join");
            assert!(exporter_result.is_ok(), "exporter should stop cleanly");
        });
    }

    /// Scenario: shutdown reaches a topic exporter while one tracked publish is
    /// already pending downstream outcome and another tracked publish is still
    /// waiting for queue space under `queue_on_full: block`.
    /// Guarantees: shutdown nacks both the in-hand blocked publish and the
    /// already-admitted tracked publish, then exits promptly.
    #[test]
    fn shutdown_interrupts_blocked_tracked_publish_with_block_policy() {
        let rt = setup_test_runtime();
        rt.block_on(async move {
            let broker = TopicBroker::<OtapPdata>::new();
            let topic_name = otel_arrow_dfe_config::TopicName::parse("ingress")
                .expect("topic name should parse");
            let base_handle = broker
                .create_in_memory_topic(
                    topic_name.clone(),
                    TopicOptions::Mixed {
                        balanced_capacity: 1,
                        broadcast_capacity: 1,
                        on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                        ack_mode: TopicBroadcastAckMode::First,
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
                &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
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
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel::<OtapPdata>(32);
            let (pipeline_completion_tx, mut pipeline_completion_rx) =
                pipeline_completion_msg_channel::<OtapPdata>(32);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
            let exporter_task = tokio::task::spawn_local(async move {
                exporter
                    .start(
                        runtime_ctrl_tx,
                        pipeline_completion_tx,
                        metrics_reporter,
                        Interests::empty(),
                        otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
                    )
                    .await
            });

            let _subscriber = base_handle
                .subscribe(
                    SubscriptionMode::Balanced {
                        group: "workers".into(),
                    },
                    SubscriberOptions::default(),
                )
                .expect("topic subscriber should be created");

            let first_call_data = TestCallData::new_with(11, 1);
            exporter_input_tx
                .send(create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    first_call_data.clone().into(),
                    2001,
                ))
                .expect("failed to send first pdata to topic exporter");

            tokio::time::sleep(Duration::from_millis(100)).await;

            let second_call_data = TestCallData::new_with(22, 2);
            exporter_input_tx
                .send(create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    second_call_data.clone().into(),
                    2002,
                ))
                .expect("failed to send second pdata to topic exporter");

            tokio::time::sleep(Duration::from_millis(100)).await;
            exporter_ctrl
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "test shutdown".to_owned(),
                })
                .await
                .expect("exporter shutdown should be sent");

            let mut seen_nodes = Vec::new();
            let mut reasons = Vec::new();
            for _ in 0..2 {
                let completion = tokio::time::timeout(Duration::from_secs(2), async {
                    loop {
                        if let PipelineCompletionMsg::DeliverNack { nack } = pipeline_completion_rx
                            .recv()
                            .await
                            .expect("pipeline-completion channel closed unexpectedly")
                        {
                            break nack;
                        }
                    }
                })
                .await
                .expect("timed out waiting for shutdown nacks");
                let (node_id, nack) = next_nack(completion).expect("nack should be routable");
                let got: TestCallData = nack
                    .unwind
                    .route
                    .calldata
                    .try_into()
                    .expect("nack calldata should parse");
                seen_nodes.push((node_id, got));
                reasons.push(nack.reason);
            }
            assert!(seen_nodes.contains(&(2001, first_call_data)));
            assert!(seen_nodes.contains(&(2002, second_call_data)));
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason.contains("topic exporter shutdown before downstream ack"))
            );
            assert!(reasons
                .iter()
                .any(|reason| reason.contains("topic exporter shutdown before topic admission")));

            let exporter_result = tokio::time::timeout(Duration::from_secs(2), exporter_task)
                .await
                .expect("exporter should exit promptly after shutdown")
                .expect("exporter task should join");
            assert!(exporter_result.is_ok(), "exporter should stop cleanly");
        });
    }
}
