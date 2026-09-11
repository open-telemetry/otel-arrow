// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// ToDo: update tests to start broker in memory
// ToDo: Possible optimization to improve how we determine signal type from a message
// check every message against list of topics + excluded topics to get signal type
// ToDo: Offload heavier decode operations to avoid stalling the receiver

use super::config::KafkaReceiverConfig;
use super::error::KafkaReceiverError;
use super::identity::OwnershipGeneration;
use super::metrics::{KafkaReceiverMetrics, KafkaReceiverRejectionReason};
use super::offset_tracker::OffsetTracker;
use super::rebalance::{RebalanceState, RebalancingConsumerContext};
use super::retry::RetryManager;
#[cfg(feature = "aws")]
use crate::common::kafka::security::build_aws_msk_context;
use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::transport_headers_policy::HeaderCapturePolicy;
use otel_arrow_dfe_config::validation::validate_typed_config;
use otel_arrow_dfe_engine::config::ReceiverConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::error::{Error as EngineError, ReceiverErrorKind, format_error_sources};
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{Interests, ProducerEffectHandlerExtension, ReceiverFactory};
use otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, ReceiverRejectionErrorType};
use rdkafka::Message as _;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{Consumer, ConsumerContext};
use rdkafka::error::KafkaError;
use rdkafka::message::BorrowedMessage;
use regex::Regex;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

mod consumer;
// pub(crate) so the metrics module can implement `From<DlqReason>` for its
// bounded attribute enum (single reason mapping, no duplication).
pub(crate) mod dlq;
pub(crate) mod decode;
mod offset_feedback;
mod replay;
// pub(crate) so config-time DLQ loop-prevention can reuse the same topic
// include/exclude matching as the runtime router (single source of truth).
pub(crate) mod topics;
mod transport_headers;

use consumer::{LAG_REFRESH_TOTAL_DEADLINE, LagRefreshTask, close_consumer_bounded};

use dlq::{DlqCompletion, DlqManager, DlqReason, DlqSource};
use decode::{SignalDecoder, encode_calldata};
use topics::{
    TopicRegistry, compile_exclude_regexes, compile_topic_regexes, detect_message_format,
    matches_any_exclude, matches_any_topic,
};
use transport_headers::capture_transport_headers;

/// URN for the Kafka Receiver
pub const KAFKA_RECEIVER_URN: &str = "urn:otel:receiver:kafka";

/// Kafka receiver for OpenTelemetry data.
///
/// Receives telemetry data (traces, metrics, logs) from Apache Kafka topics using the rdkafka client.
///
/// Offset management uses per-offset tracking: each consumed message is tracked individually,
/// and only the lowest un-acknowledged offset per partition is committed to Kafka. This prevents
/// offset skipping when acknowledgements arrive out-of-order from the downstream pipeline.
pub struct KafkaReceiver {
    config: KafkaReceiverConfig,
    metrics: KafkaReceiverMetrics,
    /// Per-offset tracker. Only active when auto-commit is disabled.
    offset_tracker: OffsetTracker,
    /// Partition-local delivery generations and transient-NACK replay state.
    retry_manager: RetryManager,
    /// Shared consumer-group rebalance state. Updated by the consumer
    /// context's rebalance callbacks (served inline by `consumer.recv()`, so on
    /// the pipeline thread) and reconciled by the receive loop. Only active when
    /// auto-commit is disabled.
    rebalance_state: Arc<RebalanceState>,
    /// Dynamically assigns `u32` IDs to actual topic names for CallData encoding.
    topic_registry: TopicRegistry,
    /// Pre-compiled regexes parallel to each signal's topic list. Each entry
    /// is `Some(Regex)` when the corresponding config topic starts with `^`,
    /// or `None` for literal topic names matched via exact equality.
    traces_topic_regexes: Vec<Option<Regex>>,
    metrics_topic_regexes: Vec<Option<Regex>>,
    logs_topic_regexes: Vec<Option<Regex>>,
    /// Pre-compiled exclude topic regexes for each signal.
    traces_exclude_regexes: Vec<Regex>,
    metrics_exclude_regexes: Vec<Regex>,
    logs_exclude_regexes: Vec<Regex>,
    /// Dead-letter-queue egress. `None` when the DLQ is disabled. Constructed
    /// in `start` so an unreachable DLQ connection fails receiver startup.
    dlq: Option<DlqManager>,
    // TODO: add this back once we can reset it without re-creation: https://github.com/open-telemetry/otel-arrow/issues/1669
    // used to decode otap bytes
    // pdata_consumer: PdataConsumer,
}

/// Declares the kafka receiver as a local receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
// DLQ-PHASE-2 (Add): declare outputs ["default", "dlq"] and default_output
// "default" so dead-lettered messages can leave via the "dlq" port
// (WiringContract::UNRESTRICTED already permits the extra port).
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static KAFKA_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: KAFKA_RECEIVER_URN,
    create:
        |pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         receiver_config: &ReceiverConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            Ok(ReceiverWrapper::local(
                KafkaReceiver::from_config(pipeline, &node_config.config)?,
                node,
                node_config,
                receiver_config,
            ))
        },
    validate_config: validate_typed_config::<KafkaReceiverConfig>,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

impl KafkaReceiver {
    /// Create a new kafka receiver from the config.
    ///
    /// Config is already validated via [`KafkaReceiverConfig`]'s `TryFrom`
    /// implementation, so this only performs regex compilation.
    pub fn new(
        pipeline_ctx: PipelineContext,
        mut config: KafkaReceiverConfig,
    ) -> Result<Self, ConfigError> {
        // Kafka static membership requires each consumer-group member to have a
        // unique group.instance.id. On a multi-core pipeline every core would
        // otherwise share the configured ID and fence one another, so suffix it
        // with the pipeline core ID.
        if pipeline_ctx.num_cores() > 1
            && let Some(base_id) = config.group_instance_id()
        {
            let resolved = format!("{base_id}-{}", pipeline_ctx.core_id());
            config.set_group_instance_id(resolved);
        }

        // Warn about consumer_config keys that may be overwritten by first-class fields.
        for key in config.overridden_consumer_config_keys() {
            otel_warn!(
                "kafka.receiver.consumer_config.override",
                key = %key,
                "consumer_config contains key '{key}' which is also managed by a \
                 first-class config field and may be overwritten",
            );
        }

        // Pre-compile regex patterns (starting with ^) so invalid
        // patterns fail fast at config time.
        let traces_topic_regexes = compile_topic_regexes(config.traces_topics())?;
        let metrics_topic_regexes = compile_topic_regexes(config.metrics_topics())?;
        let logs_topic_regexes = compile_topic_regexes(config.logs_topics())?;

        // Pre-compile exclude topic regexes.
        let traces_exclude_regexes = compile_exclude_regexes(config.traces_exclude_topics())?;
        let metrics_exclude_regexes = compile_exclude_regexes(config.metrics_exclude_topics())?;
        let logs_exclude_regexes = compile_exclude_regexes(config.logs_exclude_topics())?;

        let metrics = KafkaReceiverMetrics::register(&pipeline_ctx);

        let rebalance_state = Arc::new(RebalanceState::new(config.is_auto_commit()));

        Ok(Self {
            config,
            metrics,
            offset_tracker: OffsetTracker::new(),
            retry_manager: RetryManager::new(),
            rebalance_state,
            topic_registry: TopicRegistry::new(),
            traces_topic_regexes,
            metrics_topic_regexes,
            logs_topic_regexes,
            traces_exclude_regexes,
            metrics_exclude_regexes,
            logs_exclude_regexes,
            dlq: None,
        })
    }

    /// creates a new kafka receiver from yaml config
    pub fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        KafkaReceiver::new(
            pipeline_ctx,
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?,
        )
    }

    /// Returns the shared rebalance state for synchronization in component tests.
    #[cfg(test)]
    pub(crate) fn rebalance_state_for_test(&self) -> Arc<RebalanceState> {
        Arc::clone(&self.rebalance_state)
    }

    /// Returns the signal selected by the configured include and exclude rules.
    fn signal_type_for_topic(&self, topic: &str) -> Option<SignalType> {
        if matches_any_topic(
            self.config.traces_topics(),
            &self.traces_topic_regexes,
            topic,
        ) && !matches_any_exclude(&self.traces_exclude_regexes, topic)
        {
            Some(SignalType::Traces)
        } else if matches_any_topic(
            self.config.metrics_topics(),
            &self.metrics_topic_regexes,
            topic,
        ) && !matches_any_exclude(&self.metrics_exclude_regexes, topic)
        {
            Some(SignalType::Metrics)
        } else if matches_any_topic(self.config.logs_topics(), &self.logs_topic_regexes, topic)
            && !matches_any_exclude(&self.logs_exclude_regexes, topic)
        {
            Some(SignalType::Logs)
        } else {
            None
        }
    }

    /// Process a Kafka message into [`OtapPdata`].
    ///
    /// Offset tracking is handled by the caller, not inside this method. This
    /// allows the caller to track the offset even when decoding fails (poison
    /// pill handling).
    ///
    /// When a [`HeaderCapturePolicy`] is provided, matching Kafka message
    /// headers are captured into [`TransportHeaders`] and attached to the
    /// returned [`OtapPdata`] context. This is independent of the
    /// `resource_attrs_from_headers` config which injects headers into resource attributes.
    fn process_kafka(
        &mut self,
        kafka_message: BorrowedMessage<'_>,
        capture_policy: Option<&HeaderCapturePolicy>,
    ) -> Result<OtapPdata, KafkaReceiverError> {
        let topic = kafka_message.topic();

        let data = kafka_message.payload().ok_or_else(|| {
            KafkaReceiverError::EmptyPayloadDecode(EngineError::PdataConversionError {
                error: "Empty payload inside Kafka Message unable to convert to PData".to_string(),
            })
        })?;

        let extractors = self.config.resource_attrs_from_headers();

        // Route the topic to the correct signal decoder. Supports both literal
        // topic names and regex patterns (prefixed with `^`), exclude patterns,
        // per-signal encoding, and multiple topics per signal type. All
        // per-signal decode behavior (format selection, Syslog restriction,
        // extraction dispatch) is centralized in `SignalDecoder`.
        let mut pdata = match self.signal_type_for_topic(topic) {
            Some(signal) => {
                let message_format = detect_message_format(
                    &kafka_message,
                    self.config.message_format_header(),
                    self.config.encoding_for(signal),
                );
                SignalDecoder::decode_signal_with_extractions(
                    signal,
                    &kafka_message,
                    extractors,
                    data,
                    message_format,
                )
                .map_err(|source| KafkaReceiverError::SignalDecode { signal, source })
            }
            None => Err(KafkaReceiverError::UnknownTopicDecode(
                EngineError::PdataConversionError {
                    error: "Received a message from an unknown Kafka topic; unable to convert it to PData"
                        .to_string(),
                },
            )),
        }?;

        capture_transport_headers(&kafka_message, capture_policy, &mut pdata);

        Ok(pdata)
    }

    /// Drain partitions revoked by the rebalance callbacks and purge them from
    /// the offset tracker.
    ///
    /// Called both once per receive-loop iteration (via
    /// [`reconcile_rebalance_state`](Self::reconcile_rebalance_state)) **and** at
    /// the start of every commit (via [`commit_offsets`](Self::commit_offsets)),
    /// so no commit path can ever persist an offset for a partition this
    /// consumer no longer owns -- even if the revocation was queued by the
    /// callback after the last loop-top reconcile (e.g. just before a
    /// `TimerTick`, shutdown commit, or poison-pill advance).
    fn purge_revoked_partitions(&mut self) {
        if self.config.is_auto_commit() {
            return;
        }
        let revoked = self.rebalance_state.drain_revoked();
        if !revoked.is_empty() {
            for r in revoked {
                // Generation-aware purge: only remove tracker state that is not
                // newer than the revocation. If the partition was reassigned to
                // this consumer and re-tracked under a newer generation, this
                // stale revocation is a no-op and the fresh state is preserved.
                let _ = self
                    .offset_tracker
                    .revoke_if_older(&r.topic, r.partition, r.generation);
                self.retry_manager.revoke_if_older(
                    &r.topic,
                    r.partition,
                    OwnershipGeneration::from_raw(r.generation),
                );
            }
            // Owned set changed; refresh the snapshot used by pre_rebalance.
            self.refresh_committable_snapshot();
        }
    }

    /// Drain revoked partitions (see [`purge_revoked_partitions`](Self::purge_revoked_partitions))
    /// and fold rebalance counters accumulated on the callback thread into the
    /// receiver's metric set.
    ///
    /// Called once per receive-loop iteration. Drains early-return when nothing
    /// has happened, so the steady-state (no rebalance) cost is a couple of
    /// uncontended mutex lock/unlock cycles. No-op when auto-commit is enabled.
    fn reconcile_rebalance_state(&mut self) {
        if self.config.is_auto_commit() {
            return;
        }

        self.purge_revoked_partitions();

        // Point-in-time in-flight depth: tracked-but-uncommitted offsets awaiting
        // an Ack/Nack. Refreshed every iteration since it changes with ordinary
        // ack/commit activity, not only on rebalances.
        self.metrics
            .consumer
            .records_in_flight
            .observe(self.offset_tracker.total_pending() as u64);
        self.metrics
            .consumer
            .retry_partitions_paused
            .observe(self.retry_manager.paused_count() as u64);

        let delta = self.rebalance_state.drain_metrics();
        if !delta.is_empty() {
            self.metrics.consumer.rebalances.add(delta.rebalances_total);
            self.metrics
                .consumer
                .partition_assignments
                .add(delta.partition_assignments);
            self.metrics
                .consumer
                .partition_revocations
                .add(delta.partition_revocations);
            self.metrics
                .consumer
                .rebalance_commit_failures
                .add(delta.rebalance_commit_errors);
            self.metrics
                .consumer
                .rebalance_resume_failures
                .add(delta.rebalance_resume_errors);
            // `receiver.kafka.consumer.group.partitions` is an observed up/down
            // counter: observe the current owned count snapshot rather than
            // accumulating. Folded only when a rebalance actually occurred
            // (guarded by `is_empty`, which ignores this observe-only field) to
            // avoid redundant writes on idle ticks.
            self.metrics
                .consumer
                .partitions
                .observe(delta.partitions_owned);
            // Commit outcomes are observed asynchronously on the consumer commit
            // callback and folded in here (see `commit_offsets`).
            self.metrics
                .record_offset_commits(Outcome::Success, delta.offset_commits);
            self.metrics
                .record_offset_commits(Outcome::Failure, delta.offset_commit_errors);
        }
    }

    /// Refresh the shared committable snapshot from the offset tracker so the
    /// pre-rebalance callback can commit owned partitions before they are
    /// revoked. No-op when auto-commit is enabled.
    fn refresh_committable_snapshot(&self) {
        if self.config.is_auto_commit() {
            return;
        }
        self.rebalance_state
            .set_committable_snapshot(self.offset_tracker.committable_snapshot());
    }

    async fn run_receive_loop<C: ConsumerContext + 'static>(
        &mut self,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
        consumer: StreamConsumer<C>,
    ) -> Result<TerminalState, EngineError> {
        let consumer = Arc::new(consumer);

        // Start periodic telemetry collection
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;
        let topics = self.config.all_topics();

        // Subscribe to the configured topics
        consumer.subscribe(&topics).map_err(|e| {
            let source_detail = format_error_sources(&e);
            EngineError::ReceiverError {
                receiver: effect_handler.receiver_id(),
                kind: ReceiverErrorKind::Configuration,
                error: e.to_string(),
                source_detail,
            }
        })?;

        let receiver_id = effect_handler.receiver_id();
        // Move the DLQ manager into a loop-local so the completion-drain branch
        // can borrow it disjointly from `self` inside `select!`. It is placed
        // back on `self` when the loop returns (shutdown/drain).
        let mut dlq = self.dlq.take();
        let manual_commit = !self.config.is_auto_commit();
        let idempotent = manual_commit && self.config.is_idempotent();

        // Retrieve the capture policy (if configured) for extracting Kafka
        // headers into the OtapPdata context as TransportHeaders.
        let capture_policy = effect_handler.capture_policy();

        // Safety-net timer: periodically commit offsets even if no acks
        // arrive for a while. Only started when manual commit is active
        // *and* an explicit interval was configured. When no interval is
        // set in manual mode, offsets are committed via terminal feedback.
        // A transient NACK configured for replay never advances the offset.
        // The timer delivers `NodeControlMsg::TimerTick` on the control
        // channel, which is handled in the main loop below.
        if manual_commit && let Some(ms) = self.config.commit_interval_ms() {
            let _commit_timer_handle = effect_handler
                .start_periodic_timer(Duration::from_millis(ms))
                .await?;
        }

        // Opt-in consumer-lag refresh timer, derived from the configured
        // interval. Stays `None` (disabled) in auto-commit mode (no committed
        // offset to compare against) or when no interval is set, so the dedicated
        // `select!` branch below is never polled and no timer is armed. `reset()`
        // defers the first tick by one full interval so the first refresh is
        // periodic, not immediate.
        let mut lag_ticker: Option<tokio::time::Interval> = manual_commit
            .then(|| self.config.lag_refresh_interval_ms())
            .flatten()
            .map(Duration::from_millis)
            .map(|dur| {
                let mut ticker = tokio::time::interval(dur);
                ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
                ticker.reset();
                ticker
            });

        // Keeps track of the current in flight consumer_lag worker: its join
        // handle, its absolute deadline, and a cancellation token used to stop
        // it cooperatively on shutdown or ingress drain so it cannot outlive
        // the receiver.
        let mut lag_refresh_in_flight: Option<LagRefreshTask> = None;

        loop {
            // Reconcile any partition revocations / metrics produced by the
            // rebalance callbacks since the last iteration. Cheap when idle.
            self.reconcile_rebalance_state();
            // Service the idle re-read consumer keep-warm so its broker
            // connection stays live between terminal-nack recoveries.
            // DLQ-PHASE-2 (Keep): the re-read keep-warm is retained in port mode
            // (the re-read consumer stays); only the removed in-flight/pending
            // depth observation lived here before.
            if let Some(manager) = dlq.as_ref() {
                manager.keep_warm();
            }
            let retry_deadline = match (
                self.retry_manager.next_deadline(),
                self.rebalance_state.next_assignment_resume_deadline(),
            ) {
                (Some(replay), Some(resume)) => Some(replay.min(resume)),
                (Some(deadline), None) | (None, Some(deadline)) => Some(deadline),
                (None, None) => None,
            };

            tokio::select! {
                biased;

                // 1. Process control messages (highest priority)
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            effect_handler.info("Shutting down Kafka receiver").await;
                            // Best-effort drain of outstanding DLQ deliveries so
                            // their offsets can be committed. Bounded by the
                            // shutdown deadline; anything unfinished stays
                            // uncommitted and is re-delivered (and re-dead-
                            // lettered) on restart.
                            self.drain_dlq_bounded(
                                &mut dlq,
                                consumer.as_ref(),
                                &receiver_id,
                                deadline,
                            )
                            .await;
                            // Commit all tracked offsets before shutdown
                            if manual_commit {
                                if let Err(e) = self.commit_offsets(consumer.as_ref(), &receiver_id) {
                                    otel_error!(
                                        "kafka.shutdown.commit_failed",
                                        error = %e,
                                    );
                                }
                                // Fold any commit-callback outcomes that have
                                // already been recorded on the shared rebalance
                                // state (from steady-state async commits serviced
                                // earlier in the loop) into the metric set so the
                                // terminal snapshot reflects them.
                                self.reconcile_rebalance_state();
                            }
                            // Drain any in-flight consumer-lag worker so we do not
                            // abandon a running `spawn_blocking` task. Signal
                            // cooperative cancellation, then wait for it bounded by
                            // the tighter of the worker's own deadline and the
                            // shutdown deadline. This is best-effort: the worker
                            // only observes cancellation between its librdkafka
                            // calls, so if it is parked mid-FFI when the bound
                            // expires the wait returns while the worker (and its
                            // `Arc` clone of the consumer) is still alive. See the
                            // `Arc`-count note on the close below.
                            close_consumer_bounded(
                                consumer,
                                &mut lag_refresh_in_flight,
                                telemetry_cancel_handle,
                                deadline,
                            )
                            .await;
                            return Ok(TerminalState::new(
                                deadline,
                                self.metrics.terminal_snapshots(),
                            ));
                        },
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            // Receiver-first shutdown: the engine sends
                            // DrainIngress and waits for notify_receiver_drained()
                            // before shutting down downstream nodes.
                            otel_info!("kafka.receiver.drain_ingress");
                            // Stop admitting new Kafka records by returning from
                            // this receive loop after the receiver-local drain.
                            // Un-acked offsets are safely re-delivered on restart.
                            self.drain_dlq_bounded(
                                &mut dlq,
                                consumer.as_ref(),
                                &receiver_id,
                                deadline,
                            )
                            .await;
                            if manual_commit {
                                if let Err(e) = self.commit_offsets(consumer.as_ref(), &receiver_id) {
                                    otel_error!(
                                        "kafka.drain.commit_failed",
                                        error = %e,
                                    );
                                }
                                self.refresh_committable_snapshot();
                            }
                            // Signal before the bounded broker close so the
                            // downstream pipeline can begin draining promptly.
                            effect_handler.notify_receiver_drained().await?;
                            close_consumer_bounded(
                                consumer,
                                &mut lag_refresh_in_flight,
                                telemetry_cancel_handle,
                                deadline,
                            )
                            .await;
                            return Ok(TerminalState::new(
                                deadline,
                                self.metrics.terminal_snapshots(),
                            ));
                        },
                        Ok(NodeControlMsg::Ack(ack_msg)) => {
                            self.metrics.record_acknowledgement(
                                ack_msg.accepted.signal_type(),
                                Outcome::Success,
                            );
                            // DLQ-PHASE-2 (Add): if the calldata carries the DLQ
                            // discriminant (4th slot), treat this ack as a
                            // successful dead-letter (advance the source offset)
                            // instead of normal offset feedback.
                            if manual_commit && !ack_msg.unwind.route.calldata.is_empty() {
                                self.handle_terminal_offset_feedback(
                                    &ack_msg.unwind.route.calldata,
                                    consumer.as_ref(),
                                    &receiver_id,
                                );
                            }
                        },
                        Ok(NodeControlMsg::Nack(nack_msg)) => {
                            let outcome = if nack_msg.permanent {
                                Outcome::Refused
                            } else {
                                Outcome::Failure
                            };
                            self.metrics.record_acknowledgement(
                                nack_msg.refused.signal_type(),
                                outcome,
                            );
                            if !nack_msg.permanent {
                                self.metrics.consumer.transient_nacks.inc();
                            }
                            // DLQ-PHASE-2 (Add): if the calldata carries the DLQ
                            // discriminant (4th slot), treat this nack as a DLQ
                            // egress failure (record dlq.loss and advance the
                            // source offset) instead of normal offset feedback.
                            if manual_commit && !nack_msg.unwind.route.calldata.is_empty() {
                                if !nack_msg.permanent && self.config.replays_transient_nacks() {
                                    self.handle_transient_nack(
                                        &nack_msg.unwind.route.calldata,
                                        consumer.as_ref(),
                                    );
                                } else if nack_msg.permanent
                                    && self.try_dlq_permanent_nack(
                                        &mut dlq,
                                        &nack_msg.unwind.route.calldata,
                                    )
                                {
                                    // Deferred: the DLQ workflow will advance the
                                    // source offset once the delivery completes.
                                } else {
                                    self.handle_terminal_offset_feedback(
                                        &nack_msg.unwind.route.calldata,
                                        consumer.as_ref(),
                                        &receiver_id,
                                    );
                                }
                            }
                        },
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            self.reconcile_rebalance_state();
                            // Report current receiver metrics.
                            _ = self.metrics.report(&mut metrics_reporter);
                        },
                        Ok(NodeControlMsg::TimerTick { .. }) => {
                            // Periodic safety-net commit: flush any committable
                            // offsets that terminal feedback has not committed yet.
                            // Commit failures are recoverable: offsets stay
                            // tracked and are retried on the next tick.
                            if let Err(e) = self.commit_offsets(consumer.as_ref(), &receiver_id) {
                                otel_error!(
                                    "kafka.commit.failed",
                                    error = %e,
                                );
                            }
                            // Bound staleness of the rebalance commit snapshot
                            // to the commit interval.
                            self.refresh_committable_snapshot();
                        },
                        Err(e) => {
                            return Err(EngineError::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message -- do nothing
                        }
                    }
                }

                // 2. Get the result from consumer_lag worker
                result = async {
                    match lag_refresh_in_flight.as_mut() {
                        Some((handle, deadline, _cancel)) => {
                            tokio::time::timeout_at(*deadline, handle).await
                        }
                        // do nothing here
                        None => std::future::pending().await,
                    }
                }, if lag_refresh_in_flight.is_some() => {
                    match result {
                        Err(_elapsed) => {
                            // The refresh outran its deadline. Cancel the worker so
                            // it stops cooperatively at its next deadline check, and
                            // release the slot so periodic refreshes can resume (the
                            // trigger below is gated on `lag_refresh_in_flight`).
                            if let Some((_handle, _deadline, cancel)) =
                                lag_refresh_in_flight.take()
                            {
                                cancel.cancel();
                            }
                            otel_warn!("kafka.lag.refresh_incomplete", reason = "deadline_exceeded");
                        }
                        Ok(join_result) => {
                            lag_refresh_in_flight = None;
                            match join_result {
                                Ok(Some(value)) => self.metrics.consumer.lag.set(value),
                                Ok(None) => {}
                                Err(join_err) => {
                                    otel_error!("kafka.lag.refresh_task_failed", error = %join_err)
                                }
                            }
                        }
                    }
                }

                // 3. Replay a transiently NACKed partition after its backoff.
                _ = async {
                    match retry_deadline {
                        Some(deadline) => {
                            tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await
                        }
                        None => std::future::pending().await,
                    }
                }, if retry_deadline.is_some() => {
                    self.rebalance_state
                        .process_due_assignment_resumes(consumer.as_ref());
                    self.process_due_replays(consumer.as_ref());
                }

                // 4. Consume Kafka messages.
                result = consumer.recv() => {
                    match result {
                        Ok(data) => {
                            // Extract metadata before processing so we can
                            // track the offset even on decode failure.
                            let topic = data.topic().to_owned();
                            let partition = data.partition();
                            let offset = data.offset();

                            let payload_bytes = data.payload().map_or(0, |payload| payload.len() as u64);
                            self.metrics.record_consumed_record(payload_bytes);

                            // A pause can race with records already queued inside
                            // librdkafka. Discard those buffered deliveries while
                            // backoff is active; the pending seek will replay them.
                            if manual_commit
                                && self.retry_manager.blocks_delivery(&topic, partition)
                            {
                                continue;
                            }

                            // Assign a compact u32 ID for this actual topic name.
                            // The registry remembers the mapping for Ack/Nack lookup.
                            // If the ID space is exhausted, assigning another ID
                            // would wrap around and collide with an existing
                            // topic, corrupting Ack/Nack offset routing. Drop the
                            // message instead (the offset is not tracked, so it
                            // will be re-delivered on restart).
                            let topic_id = match self.topic_registry.get_or_assign(&topic) {
                                Some(id) => id,
                                None => {
                                    let rejection_signal = self.signal_type_for_topic(&topic);
                                    self.metrics.record_rejection(
                                        rejection_signal,
                                        ReceiverRejectionErrorType::Internal,
                                        KafkaReceiverRejectionReason::TopicIdExhausted,
                                    );
                                    otel_error!(
                                        "kafka.topic_id.exhausted",
                                        topic = %topic,
                                        partition = partition,
                                        offset = offset,
                                    );
                                    continue;
                                }
                            };

                            // Keep Kafka ownership and replay delivery generations
                            // independent. The ownership generation scopes tracker
                            // state across rebalances; the delivery generation
                            // invalidates pre-replay feedback within one ownership.
                            let ownership_generation_raw =
                                self.rebalance_state.current_generation(&topic, partition);
                            let ownership_generation =
                                OwnershipGeneration::from_raw(ownership_generation_raw);
                            let delivery_generation = manual_commit.then(|| {
                                self.retry_manager.delivery_generation(
                                    &topic,
                                    partition,
                                    ownership_generation,
                                )
                            });
                            let deliberate_replay = delivery_generation.is_some_and(|generation| {
                                self.retry_manager.is_replay_delivery(
                                    &topic,
                                    partition,
                                    generation,
                                )
                            });

                            // Idempotency: skip duplicate messages when enabled.
                            // The check is generation-aware: a message redelivered
                            // under a NEWER generation (same offset, after a
                            // revoke+reassign) belongs to a new ownership period
                            // and must NOT be skipped as a duplicate -- it is
                            // reprocessed, and tracking it below resets this
                            // partition's stale old-generation state.
                            if idempotent
                                && !deliberate_replay
                                && self.offset_tracker.is_known_offset_for_generation(
                                    &topic,
                                    partition,
                                    offset,
                                    ownership_generation_raw,
                                )
                            {
                                self.metrics.consumer.duplicate_records.inc();
                                continue;
                            }

                            // When the DLQ may dead-letter this message on a
                            // decode / unknown-topic failure, capture the raw
                            // bytes and source headers before `process_kafka`
                            // consumes the borrowed message. Cheap no-op when the
                            // DLQ is disabled or does not capture these
                            // categories.
                            let dlq_capture =
                                Self::dlq_inline_capture_snapshot(dlq.as_ref(), &data);

                            match self.process_kafka(data, capture_policy) {
                                Ok(mut otap_data) => {
                                    let signal = otap_data.signal_type();
                                    self.metrics
                                        .record_message_admitted(signal, payload_bytes);
                                    if let Some(delivery_generation) = delivery_generation {
                                        // Track under this partition's Kafka
                                        // ownership generation so a stale revoke
                                        // cannot purge newer state. Feedback uses
                                        // the independent delivery generation below.
                                        self.offset_tracker
                                            .track(
                                                &topic,
                                                partition,
                                                offset,
                                                ownership_generation_raw,
                                            );
                                        // Subscribe so Ack/Nack carries
                                        // offset identity (and generation) back to us
                                        let calldata = encode_calldata(
                                            topic_id,
                                            partition,
                                            offset,
                                            delivery_generation,
                                        );
                                        effect_handler.subscribe_to(
                                            Interests::ACKS_OR_NACKS,
                                            calldata,
                                            &mut otap_data,
                                        );
                                    }
                                    let send_result = effect_handler.send_message(otap_data).await;
                                    self.metrics.record_message_completed(signal);
                                    send_result?;
                                }
                                Err(decode_err) => {
                                    let (rejection_signal, rejection_error_type, rejection_reason) =
                                        match &decode_err {
                                        KafkaReceiverError::EmptyPayloadDecode(_) => (
                                            self.signal_type_for_topic(&topic),
                                            ReceiverRejectionErrorType::InvalidRequest,
                                            KafkaReceiverRejectionReason::EmptyPayload,
                                        ),
                                        KafkaReceiverError::UnknownTopicDecode(_) => (
                                            None,
                                            ReceiverRejectionErrorType::InvalidRequest,
                                            KafkaReceiverRejectionReason::UnknownTopic,
                                        ),
                                        KafkaReceiverError::SignalDecode { signal, .. } => (
                                            Some(*signal),
                                            ReceiverRejectionErrorType::InvalidRequest,
                                            KafkaReceiverRejectionReason::Decode,
                                        ),
                                        _ => (
                                            None,
                                            ReceiverRejectionErrorType::Internal,
                                            KafkaReceiverRejectionReason::Internal,
                                        ),
                                    };
                                    self.metrics.record_rejection(
                                        rejection_signal,
                                        rejection_error_type,
                                        rejection_reason,
                                    );

                                    // Emit a descriptive event so operators can
                                    // identify the specific invalid input and signal.
                                    match &decode_err {
                                        KafkaReceiverError::EmptyPayloadDecode(e) => {
                                            otel_error!(
                                                "kafka.message.empty_payload",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        KafkaReceiverError::UnknownTopicDecode(e) => {
                                            otel_error!(
                                                "kafka.message.unknown_topic",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        KafkaReceiverError::SignalDecode { signal, source } => {
                                            otel_error!(
                                                "kafka.message.unmarshal_failed",
                                                signal = SignalDecoder::label(*signal),
                                                error = %source,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        // Config variants are never produced on
                                        // the per-message decode path.
                                        _ => {
                                            otel_error!(
                                                "kafka.message.decode_failed",
                                                error = %decode_err,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                    }

                                    if let Some(delivery_generation) = delivery_generation {
                                        // Poison pill: track it so the committable
                                        // watermark cannot advance past it while
                                        // the DLQ (if any) processes it. This path
                                        // intentionally skips the late-ack guard --
                                        // a poison message must be advanced past
                                        // regardless of assignment. Stamped with
                                        // this partition's ownership generation
                                        // (read once above) for consistency with
                                        // the revoke/purge path.
                                        self.offset_tracker
                                            .track(
                                                &topic,
                                                partition,
                                                offset,
                                                ownership_generation_raw,
                                            );
                                        // Try to dead-letter the raw bytes. When
                                        // the DLQ accepts the message the offset
                                        // advance is deferred until the delivery
                                        // completes (handled in the completion
                                        // branch); otherwise advance now.
                                        let deferred = self.try_dlq_inline(
                                            &mut dlq,
                                            &decode_err,
                                            rejection_signal,
                                            &topic,
                                            partition,
                                            offset,
                                            dlq_capture,
                                        );
                                        if !deferred {
                                            self.advance_offset_and_commit(
                                                &topic,
                                                partition,
                                                offset,
                                                consumer.as_ref(),
                                                &receiver_id,
                                            );
                                        }
                                        self.retry_manager.complete_if_rewind(
                                            &topic,
                                            partition,
                                            delivery_generation,
                                            offset,
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // Kafka transport error: log and continue.
                            // Transient broker errors should not kill the receiver.
                            match &e {
                                KafkaError::PartitionEOF(_) => {
                                    otel_info!(
                                        "kafka.partition_eof",
                                        error = %e,
                                    );
                                }
                                _ => {
                                    otel_error!(
                                        "kafka.transport_error",
                                        error = %e,
                                    );
                                    self.metrics.record_transport_error(&e);
                                }
                            }
                        }
                    }
                }

                // 5. Periodic consumer-lag refresh trigger (opt-in). Fires only
                // when the timer is armed, no refresh is already in flight, and
                // the receiver is not draining, so no broker calls are issued
                // during shutdown.
                _ = async {
                    match lag_ticker.as_mut() {
                        Some(ticker) => ticker.tick().await,
                        // Unreachable: the branch guard keeps this future from
                        // being polled when the ticker is disabled.
                        None => std::future::pending().await,
                    }
                }, if lag_ticker.is_some()
                    && lag_refresh_in_flight.is_none()
                    => {
                    // pass the instant deadline to the worker so it can
                    // monitor itself during the consumer_lag calculation
                    // if deadline exceeds, it returns None
                    let cancel = CancellationToken::new();
                    if let Some(handle) = self.spawn_consumer_lag_refresh(
                        &consumer,
                        Instant::now() + LAG_REFRESH_TOTAL_DEADLINE,
                        cancel.clone(),
                    ) {
                        lag_refresh_in_flight = Some((
                            handle,
                            tokio::time::Instant::now() + LAG_REFRESH_TOTAL_DEADLINE,
                            cancel,
                        ));
                    }
                }

                // 6. Drain completed DLQ deliveries. Each completion advances
                // the source offset past the dead-lettered message (whether it
                // was produced or lost) and refills an in-flight slot from the
                // pending queue. The future borrows only the DLQ manager, so it
                // never blocks the other branches; work is bounded and runs off
                // the hot path (background producer thread / spawn_blocking
                // re-read).
                // DLQ-PHASE-2 (Remove): whole branch; DLQ completions arrive on
                // the Ack/Nack control handlers, not from a completion future.
                completion = async {
                    match dlq.as_mut() {
                        Some(manager) => manager.next_completion().await,
                        None => std::future::pending().await,
                    }
                }, if dlq.as_ref().is_some_and(DlqManager::has_work) => {
                    if let Some(completion) = completion {
                        self.handle_dlq_completion(
                            completion,
                            consumer.as_ref(),
                            &receiver_id,
                        );
                    }
                }
            }
        }
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for KafkaReceiver {
    async fn start(
        mut self: Box<Self>,
        ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let client_config = self.config.build_client_config();

        let map_kafka_client_err = |e: KafkaError| {
            let source_detail = format_error_sources(&e);
            EngineError::ReceiverError {
                receiver: effect_handler.receiver_id(),
                kind: ReceiverErrorKind::Configuration,
                error: e.to_string(),
                source_detail,
            }
        };

        // Build the Kafka consumer with a rebalance-aware client context. The
        // context records partition assignments and commits offsets before
        // partitions are revoked, sharing state with the receive loop via
        // `rebalance_state`. When the `aws` feature is enabled and AWS MSK IAM
        // auth is configured, the context also refreshes the OAUTHBEARER token.
        let rebalance_state = Arc::clone(&self.rebalance_state);

        #[cfg(feature = "aws")]
        let context = match build_aws_msk_context(self.config.auth()) {
            Some(inner) => RebalancingConsumerContext::AwsMsk {
                inner,
                state: rebalance_state,
            },
            None => RebalancingConsumerContext::Default(rebalance_state),
        };
        #[cfg(not(feature = "aws"))]
        let context = RebalancingConsumerContext::Default(rebalance_state);

        let consumer = client_config
            .create_with_context(context)
            .map_err(map_kafka_client_err)?;

        // Build the DLQ egress eagerly so a misconfigured or unreachable DLQ
        // connection fails receiver startup rather than losing data at runtime.
        // DLQ-PHASE-2 (Change): pass the effect handler so the DLQ can send out
        // the "dlq" port; drop this eager producer build and instead require the
        // "dlq" output port to be connected at wiring time.
        self.dlq = DlqManager::new(&self.config).map_err(map_kafka_client_err)?;

        self.as_mut()
            .run_receive_loop(ctrl_msg_recv, effect_handler, consumer)
            .await
    }
}

impl KafkaReceiver {
    /// Capture the raw payload bytes and source headers of a message that may
    /// need to be dead-lettered on a decode / unknown-topic failure.
    ///
    /// Returns `None` (skipping the copy) when the DLQ is disabled or does not
    /// capture inline (decode / unknown-topic) failures. This keeps the steady
    /// state -- successful decodes -- free of any extra allocation.
    fn dlq_inline_capture_snapshot(
        dlq: Option<&DlqManager>,
        message: &BorrowedMessage<'_>,
    ) -> Option<(Vec<u8>, Option<rdkafka::message::OwnedHeaders>)> {
        let dlq = dlq?;
        if !dlq.captures(DlqReason::Decode) && !dlq.captures(DlqReason::UnknownTopic) {
            return None;
        }
        let payload = message.payload().map(<[u8]>::to_vec).unwrap_or_default();
        let headers = message
            .headers()
            .map(rdkafka::message::BorrowedHeaders::detach);
        Some((payload, headers))
    }

    /// Attempt to dead-letter an inline (decode / unknown-topic) failure.
    ///
    /// Returns `true` when the DLQ accepted the message and the caller must
    /// defer the offset advance until the delivery completes; returns `false`
    /// when there is no DLQ handling for this failure (the caller advances now).
    ///
    /// An immediate loss (no topic resolved or the in-flight bound is reached)
    /// is recorded here and returns `false` so the caller advances immediately.
    fn try_dlq_inline(
        &mut self,
        dlq: &mut Option<DlqManager>,
        decode_err: &KafkaReceiverError,
        signal: Option<SignalType>,
        topic: &str,
        partition: i32,
        offset: i64,
        capture: Option<(Vec<u8>, Option<rdkafka::message::OwnedHeaders>)>,
    ) -> bool {
        let reason = match decode_err {
            KafkaReceiverError::UnknownTopicDecode(_) => DlqReason::UnknownTopic,
            KafkaReceiverError::TracesDecode(_)
            | KafkaReceiverError::MetricsDecode(_)
            | KafkaReceiverError::LogsDecode(_)
            | KafkaReceiverError::EmptyPayloadDecode(_) => DlqReason::Decode,
            // Config variants never occur on the per-message decode path.
            _ => return false,
        };

        let Some(manager) = dlq.as_mut() else {
            return false;
        };
        if !manager.captures(reason) {
            return false;
        }

        let (payload, original_headers) = capture.unwrap_or_default();
        let source = DlqSource {
            topic: Arc::from(topic),
            partition,
            offset,
        };
        let immediate = manager.submit_inline(
            reason,
            source,
            signal,
            decode_err.to_string(),
            payload,
            original_headers,
        );
        self.handle_submit_outcome(immediate)
    }

    /// Attempt to dead-letter a permanently-nacked message via the re-read
    /// consumer. Returns `true` when the offset advance is deferred to the DLQ
    /// completion, `false` when the caller must apply terminal feedback now.
    fn try_dlq_permanent_nack(
        &mut self,
        dlq: &mut Option<DlqManager>,
        calldata: &otel_arrow_dfe_engine::control::CallData,
    ) -> bool {
        if !dlq
            .as_ref()
            .is_some_and(|manager| manager.captures(DlqReason::PermanentNack))
        {
            return false;
        }
        // Resolve identity through the same generation/ownership guard used by
        // terminal feedback, so a nack for a revoked partition is dropped and
        // never dead-lettered.
        let Some(feedback) = self.resolve_offset_feedback(calldata) else {
            return false;
        };
        let source = DlqSource {
            topic: Arc::clone(&feedback.topic),
            partition: feedback.partition,
            offset: feedback.offset,
        };
        // The replay (if this offset was a rewind record) is done regardless of
        // the DLQ outcome, so clear replay state now.
        self.retry_manager.complete_if_rewind(
            &feedback.topic,
            feedback.partition,
            feedback.delivery_generation,
            feedback.offset,
        );

        let manager = dlq.as_mut().expect("dlq presence checked above");
        let immediate = manager.submit_reread(source, None, "permanent nack".to_string());
        self.handle_submit_outcome(immediate)
    }

    /// Record DLQ telemetry for a completed workflow (attempt outcome, plus a
    /// loss when the message could not be dead-lettered).
    fn record_dlq_completion(&mut self, completion: &DlqCompletion) {
        use super::metrics::{KafkaReceiverDlqOutcome, KafkaReceiverDlqReason};
        let reason = KafkaReceiverDlqReason::from(completion.reason);
        let outcome = if completion.produced {
            KafkaReceiverDlqOutcome::Produced
        } else {
            KafkaReceiverDlqOutcome::Failed
        };
        self.metrics
            .record_dlq_attempt(completion.signal, reason, outcome);
        if !completion.produced {
            // A message that could not be dead-lettered is permanent data loss.
            self.metrics.record_dlq_loss(completion.signal, reason);
        }
    }

    /// Interpret a submit outcome: `None` means the DLQ accepted the job and the
    /// offset advance is deferred to the completion; `Some(loss)` means the job
    /// was rejected immediately, so record the loss here and let the caller
    /// advance the offset now. Returns `true` when the advance is deferred.
    fn handle_submit_outcome(&mut self, immediate: Option<DlqCompletion>) -> bool {
        match immediate {
            None => true,
            Some(completion) => {
                self.record_dlq_completion(&completion);
                false
            }
        }
    }

    /// Drain outstanding DLQ deliveries until the manager has no more work or
    /// the deadline elapses, advancing each completed message's source offset.
    ///
    /// Best-effort: on the deadline, remaining deliveries are abandoned and
    /// their offsets stay uncommitted, so they are re-delivered (and re-dead-
    /// lettered) on restart. Never blocks past the deadline.
    // DLQ-PHASE-2 (Remove): no in-receiver deliveries to drain; outstanding DLQ
    // messages are handled by the downstream exporter's own shutdown.
    async fn drain_dlq_bounded<C: ConsumerContext>(
        &mut self,
        dlq: &mut Option<DlqManager>,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
        deadline: Instant,
    ) {
        let Some(manager) = dlq.as_mut() else {
            return;
        };
        while manager.has_work() {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            let remaining = deadline.saturating_duration_since(now);
            match tokio::time::timeout(remaining, manager.next_completion()).await {
                Ok(Some(completion)) => {
                    self.handle_dlq_completion(completion, consumer, receiver_id);
                }
                // No more completions or the bound elapsed.
                Ok(None) | Err(_) => break,
            }
        }
    }

    /// Handle a completed DLQ delivery from the completion branch: record
    /// telemetry and advance the source offset regardless of outcome (produced
    /// or loss), so the pipeline is never blocked.
    fn handle_dlq_completion<C: ConsumerContext>(
        &mut self,
        completion: DlqCompletion,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
    ) {
        self.record_dlq_completion(&completion);
        if completion.produced {
            otel_debug!(
                "kafka.dlq.produced",
                topic = %completion.source.topic,
                partition = completion.source.partition,
                offset = completion.source.offset,
                reason = completion.reason_str(),
            );
        } else {
            otel_error!(
                "kafka.dlq.loss",
                topic = %completion.source.topic,
                partition = completion.source.partition,
                offset = completion.source.offset,
                reason = completion.reason_str(),
                permanent = completion.permanent_failure,
            );
        }
        // Advance the source offset past the dead-lettered message.
        self.advance_offset_and_commit(
            &completion.source.topic,
            completion.source.partition,
            completion.source.offset,
            consumer,
            receiver_id,
        );
    }
}

#[cfg(test)]
mod tests;
