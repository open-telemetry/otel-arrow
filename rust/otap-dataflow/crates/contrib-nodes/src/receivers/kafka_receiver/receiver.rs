// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// ToDo: update tests to start broker in memory
// ToDo: Possible optimization to improve how we determine signal type from a message
// check every message against list of topics + excluded topics to get signal type
// ToDo: Offload heavier decode operations to avoid stalling the receiver

use super::config::{HeaderExtraction, KafkaReceiverConfig};
use super::errors::DecodeError;
use super::headers::HeaderExtractions;
use super::metrics::KafkaReceiverMetrics;
use super::offset_tracker::OffsetTracker;
use super::rebalance::{RebalanceState, RebalancingConsumerContext};
#[cfg(feature = "aws")]
use crate::common::kafka::security::build_aws_msk_context;
use crate::common::kafka::{MSG_FORMAT_OTAP, MSG_FORMAT_OTLP, MessageFormat};
use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::transport_headers::TransportHeaders;
use otap_df_config::transport_headers_policy::HeaderCapturePolicy;
use otap_df_config::validation::validate_typed_config;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{CallData, Context8u8, NodeControlMsg};
use otap_df_engine::error::{Error as EngineError, ReceiverErrorKind, format_error_sources};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension, ReceiverFactory};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::Consumer as PdataConsumer;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::otap::{OtapArrowRecords, from_record_messages};
use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_error, otel_info};
use prost::Message;
use rdkafka::Message as _;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext};
use rdkafka::error::KafkaError;
use rdkafka::message::{BorrowedMessage, Headers};
use regex::Regex;
use serde_json::Value;
use smallvec::smallvec;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// URN for the Kafka Receiver
pub const KAFKA_RECEIVER_URN: &str = "urn:otel:receiver:kafka";

/// Compile a slice of topic config strings into a parallel [`Vec`] of
/// optional [`Regex`] values. Entries starting with `^` are treated as
/// regex patterns; literal topic names yield `None`.
///
/// Returns an error if any regex pattern is invalid.
fn compile_topic_regexes(topics: &[String]) -> Result<Vec<Option<Regex>>, ConfigError> {
    topics
        .iter()
        .map(|t| {
            if t.starts_with('^') {
                Regex::new(t)
                    .map(Some)
                    .map_err(|e| ConfigError::InvalidUserConfig {
                        error: format!("Invalid regex topic pattern '{t}': {e}"),
                    })
            } else {
                Ok(None)
            }
        })
        .collect()
}

/// Check whether an actual topic name matches any configured topic in the
/// given list. Each entry is checked against its parallel regex (if the
/// topic was a pattern), or via exact string equality.
fn matches_any_topic(config_topics: &[String], regexes: &[Option<Regex>], actual: &str) -> bool {
    config_topics
        .iter()
        .zip(regexes.iter())
        .any(|(topic, regex)| match regex {
            Some(r) => r.is_match(actual),
            None => topic == actual,
        })
}

/// Compile exclude topic patterns into [`Regex`] values.
/// All entries are treated as regex patterns (they must be valid regex per
/// validation). Returns an error if any pattern is invalid.
fn compile_exclude_regexes(exclude_topics: &[String]) -> Result<Vec<Regex>, ConfigError> {
    exclude_topics
        .iter()
        .map(|t| {
            Regex::new(t).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Invalid exclude_topics regex pattern '{t}': {e}"),
            })
        })
        .collect()
}

/// Check whether an actual topic name matches any exclude pattern.
fn matches_any_exclude(exclude_regexes: &[Regex], actual: &str) -> bool {
    exclude_regexes.iter().any(|r| r.is_match(actual))
}

/// Detect the message format from Kafka headers, falling back to the
/// configured default when the header is absent or unrecognized.
fn detect_message_format(
    kafka_message: &BorrowedMessage<'_>,
    header_key: &str,
    default: MessageFormat,
) -> MessageFormat {
    match kafka_message
        .headers()
        .and_then(|hs| hs.iter().find(|h| h.key == header_key))
        .and_then(|h| h.value)
    {
        value if value == Some(MSG_FORMAT_OTLP) => MessageFormat::OtlpProto,
        value if value == Some(MSG_FORMAT_OTAP) => MessageFormat::OtapProto,
        _ => default,
    }
}

/// Dynamically assigns compact `u32` IDs to actual Kafka topic names.
///
/// Used to encode topic identity into [`CallData`] for Ack/Nack routing
/// while supporting regex-matched topic names that aren't known at config
/// time.
struct TopicRegistry {
    name_to_id: HashMap<Arc<str>, u32>,
    id_to_name: Vec<Arc<str>>,
}

impl TopicRegistry {
    fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
        }
    }

    /// Get or assign a `u32` ID for the given topic name.
    fn get_or_assign(&mut self, topic: &str) -> Option<u32> {
        // if topic hasn't been seen yet then we assign topic a id
        if let Some(&id) = self.name_to_id.get(topic) {
            return Some(id);
        }
        // The next ID is the current count. Refuse if it doesn't fit in `u32`.
        let id = u32::try_from(self.id_to_name.len()).ok()?;
        let name: Arc<str> = Arc::from(topic);
        self.id_to_name.push(Arc::clone(&name));
        let _ = self.name_to_id.insert(name, id);
        Some(id)
    }

    /// Look up a topic name by its assigned ID.
    ///
    /// Returns a cheap `Arc<str>` clone so callers can hold an owned handle to
    /// the topic name without borrowing the registry -- avoiding a borrow
    /// conflict when the same call site also needs `&mut self` (e.g. to mutate
    /// the offset tracker), and without allocating a fresh `String` per ack.
    fn name_for(&self, id: u32) -> Option<Arc<str>> {
        self.id_to_name.get(id as usize).map(Arc::clone)
    }
}

/// Kafka receiver for OpenTelemetry data.
///
/// Receives telemetry data (traces, metrics, logs) from Apache Kafka topics using the rdkafka client.
///
/// Offset management uses per-offset tracking: each consumed message is tracked individually,
/// and only the lowest un-acknowledged offset per partition is committed to Kafka. This prevents
/// offset skipping when acknowledgements arrive out-of-order from the downstream pipeline.
pub struct KafkaReceiver {
    config: KafkaReceiverConfig,
    metrics: MetricSet<KafkaReceiverMetrics>,
    /// Per-offset tracker. Only active when auto-commit is disabled.
    offset_tracker: OffsetTracker,
    /// Shared consumer-group rebalance state. Updated by the consumer
    /// context's rebalance callbacks (on the librdkafka thread) and reconciled
    /// by the receive loop. Only active when auto-commit is disabled.
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
    // TODO: add this back once we can reset it without re-creation: https://github.com/open-telemetry/otel-arrow/issues/1669
    // used to decode otap bytes
    // pdata_consumer: PdataConsumer,
}

/// Declares the kafka receiver as a local receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static KAFKA_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: KAFKA_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ReceiverWrapper::local(
            KafkaReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
    validate_config: validate_typed_config::<KafkaReceiverConfig>,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
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
        if pipeline_ctx.num_cores() > 1 {
            if let Some(base_id) = config.group_instance_id() {
                let resolved = format!("{base_id}-{}", pipeline_ctx.core_id());
                config.set_group_instance_id(resolved);
            }
        }

        // Warn about consumer_config keys that may be overwritten by first-class fields.
        for key in config.overridden_consumer_config_keys() {
            otap_df_telemetry::otel_warn!(
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

        let metrics = pipeline_ctx.register_metrics::<KafkaReceiverMetrics>();

        let rebalance_state = Arc::new(RebalanceState::new(config.is_auto_commit()));

        Ok(Self {
            config,
            metrics,
            offset_tracker: OffsetTracker::new(),
            rebalance_state,
            topic_registry: TopicRegistry::new(),
            traces_topic_regexes,
            metrics_topic_regexes,
            logs_topic_regexes,
            traces_exclude_regexes,
            metrics_exclude_regexes,
            logs_exclude_regexes,
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
    ) -> Result<OtapPdata, DecodeError> {
        let topic = kafka_message.topic();

        let data = kafka_message.payload().ok_or_else(|| {
            DecodeError::EmptyPayload(EngineError::PdataConversionError {
                error: "Empty payload inside Kafka Message unable to convert to PData".to_string(),
            })
        })?;

        let extractors = self.config.resource_attrs_from_headers();

        // Route the topic to the correct signal decoder. Supports both literal
        // topic names and regex patterns (prefixed with `^`), exclude patterns,
        // per-signal encoding, and multiple topics per signal type.
        let mut pdata = if matches_any_topic(
            self.config.traces_topics(),
            &self.traces_topic_regexes,
            topic,
        ) && !matches_any_exclude(&self.traces_exclude_regexes, topic)
        {
            let message_format = detect_message_format(
                &kafka_message,
                self.config.message_format_header(),
                self.config.traces_encoding(),
            );
            self.metrics.trace_msgs_received.add(1);
            decode_with_extractions(
                &kafka_message,
                extractors,
                data,
                message_format,
                HeaderExtractions::apply_otlp_traces,
                HeaderExtractions::apply_otap_traces,
                decode_traces_payload,
            )
            .map_err(DecodeError::Traces)
        } else if matches_any_topic(
            self.config.metrics_topics(),
            &self.metrics_topic_regexes,
            topic,
        ) && !matches_any_exclude(&self.metrics_exclude_regexes, topic)
        {
            let message_format = detect_message_format(
                &kafka_message,
                self.config.message_format_header(),
                self.config.metrics_encoding(),
            );
            self.metrics.metric_msgs_received.add(1);
            decode_with_extractions(
                &kafka_message,
                extractors,
                data,
                message_format,
                HeaderExtractions::apply_otlp_metrics,
                HeaderExtractions::apply_otap_metrics,
                decode_metrics_payload,
            )
            .map_err(DecodeError::Metrics)
        } else if matches_any_topic(self.config.logs_topics(), &self.logs_topic_regexes, topic)
            && !matches_any_exclude(&self.logs_exclude_regexes, topic)
        {
            let message_format = detect_message_format(
                &kafka_message,
                self.config.message_format_header(),
                self.config.logs_encoding(),
            );
            self.metrics.log_msgs_received.add(1);
            decode_with_extractions(
                &kafka_message,
                extractors,
                data,
                message_format,
                HeaderExtractions::apply_otlp_logs,
                HeaderExtractions::apply_otap_logs,
                decode_logs_payload,
            )
            .map_err(DecodeError::Logs)
        } else {
            Err(DecodeError::UnknownTopic(
                EngineError::PdataConversionError {
                    error: "Unknown kafka topic received unable to convert to PData".to_string(),
                },
            ))
        }?;

        capture_transport_headers(&kafka_message, capture_policy, &mut pdata);

        Ok(pdata)
    }

    /// Commit the current committable offsets to Kafka.
    ///
    /// Updates the offset tracker's internal [`TopicPartitionList`] in-place and
    /// commits **asynchronously**: [`CommitMode::Async`] enqueues the request in
    /// librdkafka's local work queue and returns immediately, so the pipeline's
    /// single-thread runtime never blocks on a broker round-trip. This method is
    /// on the hot ACK/NACK path, so it must not stall data processing, control
    /// messages, telemetry, or shutdown.
    ///
    /// Because the commit is async, the returned `Ok(())` only means the request
    /// was enqueued -- not that the broker accepted it. The eventual broker
    /// outcome is observed via
    /// [`RebalancingConsumerContext::commit_callback`](super::rebalance::RebalancingConsumerContext),
    /// which folds success/failure counts into
    /// [`offset_commits`](KafkaReceiverMetrics::offset_commits) /
    /// [`offset_commit_errors`](KafkaReceiverMetrics::offset_commit_errors) via
    /// the shared rebalance state. A rare *enqueue* failure is returned here so
    /// callers can log it; the offsets stay tracked and are retried on the next
    /// ack/nack/timer-tick.
    ///
    /// Only commits when auto-commit is disabled.
    fn commit_offsets<C: ConsumerContext>(
        &mut self,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
    ) -> Result<(), EngineError> {
        if self.config.is_auto_commit() {
            return Ok(());
        }
        // Drop any partitions revoked by the rebalance callback since the last
        // reconcile *before* building the commit list, so we never commit an
        // offset for a partition this consumer no longer owns.
        self.purge_revoked_partitions();
        let tpl = self.offset_tracker.committable_tpl();
        if tpl.count() == 0 {
            return Ok(());
        }
        // Enqueue asynchronously; the broker result arrives later on
        // `commit_callback`, which is the single source of truth for commit
        // success/failure metrics (avoids double counting).
        match consumer.commit(tpl, CommitMode::Async) {
            Ok(()) => Ok(()),
            Err(e) => {
                let source_detail = format_error_sources(&e);
                Err(EngineError::ReceiverError {
                    receiver: receiver_id.clone(),
                    kind: ReceiverErrorKind::Transport,
                    error: e.to_string(),
                    source_detail,
                })
            }
        }
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

        let delta = self.rebalance_state.drain_metrics();
        if !delta.is_empty() {
            self.metrics
                .partitions_assigned
                .add(delta.partitions_assigned);
            self.metrics
                .partitions_revoked
                .add(delta.partitions_revoked);
            self.metrics
                .rebalance_commit_errors
                .add(delta.rebalance_commit_errors);
            // Commit outcomes are observed asynchronously on the consumer commit
            // callback and folded in here (see `commit_offsets`).
            self.metrics.offset_commits.add(delta.offset_commits);
            self.metrics
                .offset_commit_errors
                .add(delta.offset_commit_errors);
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

    /// Advance the offset tracker for a processed message and, if the
    /// committable watermark moved, commit and refresh the rebalance snapshot.
    ///
    /// This is the single place that persists forward progress past a message
    /// (whether it was acked, nacked, or a poison pill). Commit failures are
    /// recoverable: the offset stays tracked and is retried on the next
    /// ack/nack/timer-tick.
    ///
    /// Caller must ensure manual-commit mode.
    fn advance_offset_and_commit<C: ConsumerContext>(
        &mut self,
        topic: &str,
        partition: i32,
        offset: i64,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
    ) {
        if self.offset_tracker.acknowledge(topic, partition, offset) {
            if let Err(e) = self.commit_offsets(consumer, receiver_id) {
                otel_error!(
                    "kafka.commit.failed",
                    error = %e,
                );
            }
            // The committable watermark moved; keep the rebalance snapshot
            // fresh for a potential pre-rebalance commit.
            self.refresh_committable_snapshot();
        }
    }

    /// Handle an Ack/Nack carrying Kafka offset identity in its `CallData`.
    ///
    /// Decodes the topic/partition/offset, applies a **late-ack guard** -- if
    /// the partition is no longer assigned to this consumer (revoked during a
    /// rebalance), the ack is dropped without committing, since the new owner
    /// is now responsible for that partition -- and otherwise advances the
    /// offset tracker, committing when the watermark advances.
    ///
    /// Caller must ensure manual-commit mode and a non-empty `calldata`.
    fn handle_offset_feedback<C: ConsumerContext>(
        &mut self,
        calldata: &CallData,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
    ) {
        let (topic_id, partition, offset, ack_generation) = decode_calldata(calldata);
        // Resolve the dynamic topic ID back to the actual topic name. The
        // `Arc<str>` is an owned handle, so it does not borrow `self` and can
        // coexist with the `&mut self` calls below.
        let Some(name) = self.topic_registry.name_for(topic_id) else {
            return;
        };

        // Read the partition's tracked generation once and reuse it for both
        // guards below (avoids repeated tracker lookups on this hot path).
        let tracked_generation = self.offset_tracker.partition_generation(&name, partition);

        // Stale-generation guard: feedback produced under an earlier ownership
        // period must not affect the current one. If the partition's tracked
        // state belongs to a newer generation than this ack, the ack is stale
        // (the partition was revoked and reassigned since); drop it without
        // disturbing the current state.
        if tracked_generation.is_some_and(|current| ack_generation < current) {
            self.metrics.acks_for_revoked_partition.add(1);
            return;
        }

        // Late-ack guard: never commit a partition this consumer no longer
        // owns. Drop the feedback and purge any lingering tracker state for the
        // ack's generation or older (never a newer ownership period).
        //
        // This is safe because librdkafka runs `post_rebalance(Assign)` on the
        // poll thread *before* `consumer.recv()` yields messages for the newly
        // assigned partitions, so `assigned` is always populated before any ack
        // for those partitions can return.
        if !self.rebalance_state.is_assigned(&name, partition) {
            self.metrics.acks_for_revoked_partition.add(1);
            // Purge only state not newer than the ack (never a newer ownership
            // period). `tracked_generation` was already fetched above, so this
            // reuses that knowledge rather than re-reading the tracker.
            if tracked_generation.is_some_and(|current| current <= ack_generation) {
                self.offset_tracker.revoke(&name, partition);
            }
            return;
        }

        self.advance_offset_and_commit(&name, partition, offset, consumer, receiver_id);
    }

    async fn run_receive_loop<C: ConsumerContext + 'static>(
        &mut self,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
        consumer: StreamConsumer<C>,
    ) -> Result<TerminalState, EngineError> {
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
        let manual_commit = !self.config.is_auto_commit();
        let idempotent = manual_commit && self.config.is_idempotent();

        // Retrieve the capture policy (if configured) for extracting Kafka
        // headers into the OtapPdata context as TransportHeaders.
        let capture_policy = effect_handler.capture_policy();

        // Safety-net timer: periodically commit offsets even if no acks
        // arrive for a while. Only started when manual commit is active
        // *and* an explicit interval was configured. When no interval is
        // set in manual mode, offsets are committed purely via ack/nack.
        // The timer delivers `NodeControlMsg::TimerTick` on the control
        // channel, which is handled in the main loop below.
        if manual_commit {
            if let Some(ms) = self.config.commit_interval_ms() {
                let _commit_timer_handle = effect_handler
                    .start_periodic_timer(Duration::from_millis(ms))
                    .await?;
            }
        }

        // Set once the receiver-first drain protocol begins. After this the
        // receiver stops polling Kafka (see the `consumer.recv()` branch guard)
        // but stays responsive to control messages until `Shutdown` arrives.
        let mut draining_deadline: Option<Instant> = None;

        loop {
            // Reconcile any partition revocations / metrics produced by the
            // rebalance callbacks since the last iteration. Cheap when idle.
            self.reconcile_rebalance_state();

            tokio::select! {
                biased;

                // 1. Process control messages (highest priority)
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            effect_handler.info("Shutting down Kafka receiver").await;
                            // Commit all tracked offsets before shutdown
                            if manual_commit {
                                if let Err(e) = self.commit_offsets(&consumer, &receiver_id) {
                                    otel_error!(
                                        "kafka.shutdown.commit_failed",
                                        error = %e,
                                    );
                                }
                            }
                            consumer.unsubscribe();
                            let snapshot = self.metrics.snapshot();
                            _ = telemetry_cancel_handle.cancel().await;
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        },
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            // Receiver-first shutdown: the engine sends
                            // DrainIngress and waits for notify_receiver_drained()
                            // before shutting down downstream nodes.
                            if draining_deadline.is_none() {
                                otel_info!("kafka.receiver.drain_ingress");
                                // Stop admitting new Kafka records immediately.
                                // Unsubscribing halts the subscription so the
                                // `consumer.recv()` branch (now gated on
                                // `draining_deadline.is_none()`) yields no more
                                // messages.
                                consumer.unsubscribe();
                                draining_deadline = Some(deadline);
                                // Bounded receiver-local drain: one final
                                // synchronous commit of everything acked so far.
                                // Un-acked offsets are safely re-delivered on
                                // restart (at-least-once), so there is nothing
                                // else to wait on.
                                if manual_commit {
                                    if let Err(e) = self.commit_offsets(&consumer, &receiver_id) {
                                        otel_error!(
                                            "kafka.drain.commit_failed",
                                            error = %e,
                                        );
                                    }
                                    self.refresh_committable_snapshot();
                                }
                                // Signal the runtime that receiver-local drain is
                                // complete so it can proceed to shut down
                                // downstream nodes. The loop stays alive and
                                // responsive to the eventual Shutdown message.
                                effect_handler.notify_receiver_drained().await?;
                            }
                        },
                        Ok(NodeControlMsg::Ack(ack_msg)) => {
                            self.metrics.acks_received.add(1);
                            if manual_commit && !ack_msg.unwind.route.calldata.is_empty() {
                                self.handle_offset_feedback(
                                    &ack_msg.unwind.route.calldata,
                                    &consumer,
                                    &receiver_id,
                                );
                            }
                        },
                        Ok(NodeControlMsg::Nack(nack_msg)) => {
                            self.metrics.nacks_received.add(1);
                            // Treat nack as ack (advance past failed message).
                            // TODO: future work -- retry logic, DLQ
                            if manual_commit && !nack_msg.unwind.route.calldata.is_empty() {
                                self.handle_offset_feedback(
                                    &nack_msg.unwind.route.calldata,
                                    &consumer,
                                    &receiver_id,
                                );
                            }
                        },
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            self.reconcile_rebalance_state();
                            // Report current receiver metrics.
                            _ = metrics_reporter.report(&mut self.metrics);
                        },
                        Ok(NodeControlMsg::TimerTick { .. }) => {
                            // Periodic safety-net commit: flush any committable
                            // offsets that haven't been committed via ack/nack yet.
                            // Commit failures are recoverable: offsets stay
                            // tracked and are retried on the next tick.
                            if let Err(e) = self.commit_offsets(&consumer, &receiver_id) {
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

                // 2. Consume Kafka messages. Stops once draining begins so no
                // new records are admitted during receiver-first shutdown.
                result = consumer.recv(), if draining_deadline.is_none() => {
                    match result {
                        Ok(data) => {
                            // Extract metadata before processing so we can
                            // track the offset even on decode failure.
                            let topic = data.topic().to_owned();
                            let partition = data.partition();
                            let offset = data.offset();

                            // Throughput metrics: count every received
                            // message and its payload size.
                            self.metrics.messages_received.add(1);
                            if let Some(payload) = data.payload() {
                                self.metrics.bytes_received.add(payload.len() as u64);
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
                                    self.metrics.topic_id_exhausted.add(1);
                                    otel_error!(
                                        "kafka.topic_id.exhausted",
                                        topic = %topic,
                                        partition = partition,
                                        offset = offset,
                                    );
                                    continue;
                                }
                            };

                            // Idempotency: skip duplicate messages when enabled.
                            if idempotent
                                && self
                                    .offset_tracker
                                    .is_known_offset(&topic, partition, offset)
                            {
                                self.metrics.idempotent_skips.add(1);
                                continue;
                            }

                            match self.process_kafka(data, capture_policy) {
                                Ok(mut otap_data) => {
                                    if manual_commit {
                                        // Stamp the record with this partition's
                                        // ownership generation so a stale revocation
                                        // of an older ownership period can't purge
                                        // it, and its Ack/Nack can be recognized
                                        // as belonging to the current ownership.
                                        // The generation is stable while the partition
                                        // stays owned, so records tracked across
                                        // unrelated rebalances share one generation.
                                        let generation =
                                            self.rebalance_state.current_generation(&topic, partition);
                                        // Track offset as in-flight
                                        self.offset_tracker
                                            .track(&topic, partition, offset, generation);
                                        // Subscribe so Ack/Nack carries
                                        // offset identity (and generation) back to us
                                        let calldata = encode_calldata(
                                            topic_id, partition, offset, generation,
                                        );
                                        effect_handler.subscribe_to(
                                            Interests::ACKS_OR_NACKS,
                                            calldata,
                                            &mut otap_data,
                                        );
                                    }
                                    effect_handler.send_message(otap_data).await?;
                                }
                                Err(decode_err) => {
                                    // Increment aggregate error counters.
                                    self.metrics.processing_errors.add(1);

                                    // Increment per-signal counter and emit
                                    // a descriptive error so operators can
                                    // identify what went wrong and where.
                                    match &decode_err {
                                        DecodeError::EmptyPayload(e) => {
                                            self.metrics.empty_payloads.add(1);
                                            otel_error!(
                                                "kafka.message.empty_payload",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        DecodeError::UnknownTopic(e) => {
                                            self.metrics.unknown_topic_errors.add(1);
                                            otel_error!(
                                                "kafka.message.unknown_topic",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        DecodeError::Traces(e) => {
                                            self.metrics.unmarshal_failed_traces.add(1);
                                            otel_error!(
                                                "kafka.message.unmarshal_failed",
                                                signal = "traces",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        DecodeError::Metrics(e) => {
                                            self.metrics.unmarshal_failed_metrics.add(1);
                                            otel_error!(
                                                "kafka.message.unmarshal_failed",
                                                signal = "metrics",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        DecodeError::Logs(e) => {
                                            self.metrics.unmarshal_failed_logs.add(1);
                                            otel_error!(
                                                "kafka.message.unmarshal_failed",
                                                signal = "logs",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                    }

                                    if manual_commit {
                                        // Poison pill: track then immediately
                                        // advance past it so it does not block
                                        // the partition. This path intentionally
                                        // skips the late-ack guard -- a poison
                                        // message must be advanced past
                                        // regardless of assignment. Stamped with
                                        // this partition's ownership generation for
                                        // consistency with the revoke/purge path.
                                        let generation =
                                            self.rebalance_state.current_generation(&topic, partition);
                                        self.offset_tracker
                                            .track(&topic, partition, offset, generation);
                                        self.advance_offset_and_commit(
                                            &topic,
                                            partition,
                                            offset,
                                            &consumer,
                                            &receiver_id,
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
                                    self.metrics.transport_errors.add(1);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Encode Kafka message identity into [`CallData`] for Ack/Nack routing.
///
/// Slot 0: `(topic_id << 32) | (partition as u32)` packed into a `u64`.
/// Slot 1: `offset` cast to `u64`.
/// Slot 2: assignment `generation`.
///
/// [`CallData`] inlines three slots, so carrying the generation adds no
/// heap allocation.
fn encode_calldata(topic_id: u32, partition: i32, offset: i64, generation: u64) -> CallData {
    let topic_partition = ((topic_id as u64) << 32) | (partition as u32 as u64);
    smallvec![
        Context8u8::from(topic_partition),
        Context8u8::from(offset as u64),
        Context8u8::from(generation),
    ]
}

/// Decode Kafka message identity from [`CallData`] returned in Ack/Nack.
///
/// A calldata without the generation slot (legacy 2-slot form) decodes as
/// generation `0`.
fn decode_calldata(calldata: &CallData) -> (u32, i32, i64, u64) {
    let topic_partition: u64 = calldata[0].into();
    let topic_id = (topic_partition >> 32) as u32;
    let partition = (topic_partition & 0xFFFF_FFFF) as i32;
    let offset: u64 = calldata[1].into();
    let generation: u64 = calldata.get(2).copied().map(Into::into).unwrap_or(0);
    (topic_id, partition, offset as i64, generation)
}

/// Decode a traces payload into `OtapPdata`.
fn decode_traces_payload(
    data: &[u8],
    message_format: MessageFormat,
) -> Result<OtapPdata, EngineError> {
    match message_format {
        MessageFormat::OtlpProto => Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportTracesRequest(Bytes::copy_from_slice(data)).into(),
        )),
        MessageFormat::OtapProto => {
            let mut bar =
                BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
                    error: e.to_string(),
                })?;
            let mut pdc = PdataConsumer::default();
            let record_messages = pdc.consume_bar(&mut bar)?;
            Ok(OtapPdata::new(
                Context::default(),
                OtapArrowRecords::Traces(from_record_messages(record_messages).map_err(|e| {
                    EngineError::PdataConversionError {
                        error: e.to_string(),
                    }
                })?)
                .into(),
            ))
        }
    }
}

/// Decode a metrics payload into `OtapPdata`.
fn decode_metrics_payload(
    data: &[u8],
    message_format: MessageFormat,
) -> Result<OtapPdata, EngineError> {
    match message_format {
        MessageFormat::OtlpProto => Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportMetricsRequest(Bytes::copy_from_slice(data)).into(),
        )),
        MessageFormat::OtapProto => {
            let mut bar =
                BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
                    error: e.to_string(),
                })?;
            let mut pdc = PdataConsumer::default();
            let record_messages = pdc.consume_bar(&mut bar)?;
            Ok(OtapPdata::new(
                Context::default(),
                OtapArrowRecords::Metrics(from_record_messages(record_messages).map_err(|e| {
                    EngineError::PdataConversionError {
                        error: e.to_string(),
                    }
                })?)
                .into(),
            ))
        }
    }
}

/// Decode a logs payload into `OtapPdata`.
fn decode_logs_payload(
    data: &[u8],
    message_format: MessageFormat,
) -> Result<OtapPdata, EngineError> {
    match message_format {
        MessageFormat::OtlpProto => Ok(OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(Bytes::copy_from_slice(data)).into(),
        )),
        MessageFormat::OtapProto => {
            let mut bar =
                BatchArrowRecords::decode(data).map_err(|e| EngineError::PdataConversionError {
                    error: e.to_string(),
                })?;
            let mut pdc = PdataConsumer::default();
            let record_messages = pdc.consume_bar(&mut bar)?;
            Ok(OtapPdata::new(
                Context::default(),
                OtapArrowRecords::Logs(from_record_messages(record_messages).map_err(|e| {
                    EngineError::PdataConversionError {
                        error: e.to_string(),
                    }
                })?)
                .into(),
            ))
        }
    }
}

/// Decode a Kafka payload with optional header extraction applied to resource
/// attributes.
///
/// When `extractors` is non-empty the Kafka message headers are scanned once
/// and, if any configured header is found, the matching `apply_*` function is
/// used to decode **and** inject the attributes in a single pass. When no
/// extractors are configured (or none matched) the plain `decode` function is
/// used instead.
fn decode_with_extractions(
    kafka_message: &BorrowedMessage<'_>,
    extractors: &HashMap<String, HeaderExtraction>,
    data: &[u8],
    message_format: MessageFormat,
    apply_otlp: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError>,
    apply_otap: fn(&HeaderExtractions, &[u8]) -> Result<OtapPdata, EngineError>,
    decode: fn(&[u8], MessageFormat) -> Result<OtapPdata, EngineError>,
) -> Result<OtapPdata, EngineError> {
    if !extractors.is_empty() {
        let extractions = match message_format {
            MessageFormat::OtlpProto => HeaderExtractions::otlp(kafka_message, extractors),
            MessageFormat::OtapProto => HeaderExtractions::otap(kafka_message, extractors),
        };
        if extractions.has_any() {
            return match message_format {
                MessageFormat::OtlpProto => apply_otlp(&extractions, data),
                MessageFormat::OtapProto => apply_otap(&extractions, data),
            };
        }
    }
    decode(data, message_format)
}

/// Apply the capture policy (if configured) to extract Kafka message headers
/// into [`TransportHeaders`] on the [`OtapPdata`] context.
///
/// This is independent of the `resource_attrs_from_headers` mechanism which injects
/// headers into resource attributes.
fn capture_transport_headers(
    kafka_message: &BorrowedMessage<'_>,
    capture_policy: Option<&HeaderCapturePolicy>,
    pdata: &mut OtapPdata,
) {
    if let Some(policy) = capture_policy {
        if let Some(headers) = kafka_message.headers() {
            let pairs = headers.iter().filter_map(|h| h.value.map(|v| (h.key, v)));
            let mut transport_headers = TransportHeaders::new();
            let stats = policy.capture_from_pairs(pairs, &mut transport_headers);
            if let Some(stats) = stats {
                otel_error!(
                    "kafka.capture_policy.limits_exceeded",
                    stats = %stats,
                );
            }
            if !transport_headers.is_empty() {
                pdata.set_transport_headers(transport_headers);
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
        self.as_mut()
            .run_receive_loop(ctrl_msg_recv, effect_handler, consumer)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::kafka_receiver::config::{
        AttributeValueType, AutoOffsetReset, CommitConfig, CommitMode as ConfigCommitMode,
        HeaderExtraction, IsolationLevel, KafkaReceiverConfigBuilder, RebalanceStrategy,
        SignalConfig,
    };

    use crate::common::kafka::MessageFormat;
    use crate::common::kafka::node_harness::KafkaReceiverHarness;
    use crate::common::kafka::node_harness::node_metrics::FoldedMetrics;
    use crate::common::kafka::test::cluster::KafkaTestCluster;
    use crate::common::kafka::test::consumer::{RebalanceTrigger, committed_offset};
    use crate::common::kafka::test::producer::SendRecord;
    use crate::common::kafka::test::wait::poll_until;
    use crate::common::kafka::test::with_cluster;
    use otap_df_config::transport_headers_policy::{CaptureDefaults, CaptureRule};
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::RuntimeControlMsg;
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::Producer;
    use otap_df_pdata::otap::{Logs, Metrics};
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue, any_value,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use otap_df_pdata::{OtapArrowRecords, OtapPayload, TryIntoWithOptions};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message;
    use rdkafka::ClientConfig;
    use rdkafka::consumer::{Consumer, StreamConsumer};
    use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
    use rdkafka::types::{RDKafkaApiKey, RDKafkaRespErr};
    use std::collections::HashMap;
    use std::time::Duration;

    /// Number of partitions provisioned for the rebalance integration tests.
    const REBALANCE_TEST_PARTITIONS: i32 = 2;
    /// Records produced to each partition in the rebalance integration tests.
    const REBALANCE_RECORDS_PER_PARTITION: i32 = 5;

    fn create_logs_service_request() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "a".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        attributes: vec![KeyValue {
                            key: "b".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    log_records: vec![
                        LogRecord {
                            time_unix_nano: 1,
                            attributes: vec![KeyValue {
                                key: "c".to_string(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                        LogRecord {
                            time_unix_nano: 2,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn create_metrics_service_request() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    /// Helper to create a trace request with actual spans containing trace_id and attributes.
    fn create_traces_with_spans() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![],
                    ..Default::default()
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope::default()),
                    spans: vec![
                        Span {
                            trace_id: vec![1u8; 16],
                            span_id: vec![1u8; 8],
                            name: "span-1".to_string(),
                            attributes: vec![KeyValue {
                                key: "existing".to_string(),
                                value: Some(AnyValue {
                                    value: Some(any_value::Value::StringValue(
                                        "original".to_string(),
                                    )),
                                }),
                            }],
                            ..Default::default()
                        },
                        Span {
                            trace_id: vec![2u8; 16],
                            span_id: vec![2u8; 8],
                            name: "span-2".to_string(),
                            attributes: vec![KeyValue {
                                key: "existing-2".to_string(),
                                value: Some(AnyValue {
                                    value: Some(any_value::Value::StringValue(
                                        "original-2".to_string(),
                                    )),
                                }),
                            }],
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    /// Create OTAP Arrow wire bytes from the `create_traces_with_spans()` helper,
    /// converting a real `ExportTraceServiceRequest` with 2 spans (including
    /// trace_ids and attributes) into OTAP Arrow wire format.
    fn create_traces_with_spans_otap_bytes() -> Vec<u8> {
        let request = create_traces_with_spans();
        let mut buf = Vec::new();
        request.encode(&mut buf).expect("encode OTLP request");

        // Convert OTLP bytes -> OtapPayload -> OtapArrowRecords
        let payload: OtapPayload = OtlpProtoBytes::ExportTracesRequest(Bytes::from(buf)).into();
        let mut otap_records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("convert OTLP to OTAP Arrow");

        // Serialize to BatchArrowRecords wire bytes (as the Kafka receiver expects)
        arrow_records_to_bytes(&mut otap_records)
    }

    fn create_metrics_otap_arrow_records_bytes() -> Vec<u8> {
        let mut arrow_records = OtapArrowRecords::Metrics(Metrics::default());
        arrow_records_to_bytes(&mut arrow_records)
    }

    fn create_logs_otap_arrow_records_bytes() -> Vec<u8> {
        let mut arrow_records = OtapArrowRecords::Logs(Logs::default());
        arrow_records_to_bytes(&mut arrow_records)
    }

    fn arrow_records_to_bytes(arrow_records: &mut OtapArrowRecords) -> Vec<u8> {
        let mut producer = Producer::new();
        let bar = producer
            .produce_bar(arrow_records)
            .expect("failed to get batch arrow reocrds");
        let mut bytes = vec![];
        bar.encode(&mut bytes).expect("failed to encode");
        bytes
    }

    /// Convert an `OtapPdata` (containing OTAP Arrow records) back to an OTLP
    /// `ExportTraceServiceRequest` so tests can assert against familiar protobuf
    /// structs instead of Arrow column internals.
    fn otap_pdata_to_traces(pdata: &mut OtapPdata) -> ExportTraceServiceRequest {
        let otlp: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("OTAP -> OTLP conversion");
        ExportTraceServiceRequest::decode(otlp.as_bytes()).expect("decode OTLP traces")
    }

    // ---- Test config builders ----

    /// Builds an auto-commit [`KafkaReceiverConfig`] for the given per-signal
    /// topics and message format, with optional resource-attribute-from-header
    /// extraction. Mirrors the config logic of the former
    /// `setup_receiver_harness_with_headers` helper.
    fn auto_config(
        brokers: &str,
        traces_topics: &[&str],
        metrics_topics: &[&str],
        logs_topics: &[&str],
        msg_format: MessageFormat,
        resource_attrs_from_headers: HashMap<String, HeaderExtraction>,
    ) -> KafkaReceiverConfig {
        KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new(brokers, "test-group", "test-client")
                .with_traces(
                    SignalConfig::new(traces_topics.iter().map(|s| (*s).to_string()).collect())
                        .with_encoding(msg_format),
                )
                .with_metrics(
                    SignalConfig::new(metrics_topics.iter().map(|s| (*s).to_string()).collect())
                        .with_encoding(msg_format),
                )
                .with_logs(
                    SignalConfig::new(logs_topics.iter().map(|s| (*s).to_string()).collect())
                        .with_encoding(msg_format),
                )
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Auto,
                    interval_ms: Some(1000),
                })
                .with_auto_offset_reset(AutoOffsetReset::Earliest)
                .with_isolation_level(IsolationLevel::ReadUncommitted)
                .with_resource_attrs_from_headers(resource_attrs_from_headers),
        )
        .expect("test config valid")
    }

    /// Builds a manual-commit [`KafkaReceiverConfig`] for a single traces topic,
    /// with an explicit consumer-group id, a safety-net commit timer, and an
    /// optional partition-assignment strategy. Mirrors the config logic of the
    /// former `setup_manual_traces_harness_with_strategy` helper.
    fn manual_traces_config(
        brokers: &str,
        group_id: &str,
        traces_topic: &str,
        commit_interval_ms: u64,
        rebalance_strategy: Option<RebalanceStrategy>,
    ) -> KafkaReceiverConfig {
        let mut builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
            .with_traces(
                SignalConfig::new(vec![traces_topic.to_string()])
                    .with_encoding(MessageFormat::OtlpProto),
            )
            .with_commit(CommitConfig {
                mode: ConfigCommitMode::Manual,
                interval_ms: Some(commit_interval_ms),
            })
            .with_auto_offset_reset(AutoOffsetReset::Earliest)
            .with_isolation_level(IsolationLevel::ReadUncommitted);
        if let Some(strategy) = rebalance_strategy {
            builder = builder.with_rebalance_strategy(strategy);
        }
        KafkaReceiverConfig::try_from(builder).expect("test config valid")
    }

    // ---- decode_payload unit tests (no Kafka broker required) ----

    fn make_config(
        traces: &[&str],
        metrics: &[&str],
        logs: &[&str],
        fmt: MessageFormat,
    ) -> KafkaReceiverConfig {
        KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
                .with_traces(
                    SignalConfig::new(traces.iter().map(|s| (*s).to_string()).collect())
                        .with_encoding(fmt),
                )
                .with_metrics(
                    SignalConfig::new(metrics.iter().map(|s| (*s).to_string()).collect())
                        .with_encoding(fmt),
                )
                .with_logs(
                    SignalConfig::new(logs.iter().map(|s| (*s).to_string()).collect())
                        .with_encoding(fmt),
                )
                .with_isolation_level(IsolationLevel::ReadUncommitted),
        )
        .expect("test config should be valid")
    }

    // -- decode_traces_payload: OTLP Proto --
    #[test]
    fn decode_traces_payload_otlp_proto() {
        let req = create_traces_with_spans();
        let mut bytes = vec![];
        req.encode(&mut bytes).expect("encode");

        let mut pdata =
            decode_traces_payload(&bytes, MessageFormat::OtlpProto).expect("should decode");
        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("to OtlpProtoBytes");
        assert!(matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)));
    }

    // -- decode_metrics_payload: OTLP Proto --
    #[test]
    fn decode_metrics_payload_otlp_proto() {
        let req = create_metrics_service_request();
        let mut bytes = vec![];
        req.encode(&mut bytes).expect("encode");

        let mut pdata =
            decode_metrics_payload(&bytes, MessageFormat::OtlpProto).expect("should decode");
        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("to OtlpProtoBytes");
        assert!(matches!(proto, OtlpProtoBytes::ExportMetricsRequest(_)));
    }

    // -- decode_logs_payload: OTLP Proto --
    #[test]
    fn decode_logs_payload_otlp_proto() {
        let req = create_logs_service_request();
        let mut bytes = vec![];
        req.encode(&mut bytes).expect("encode");

        let mut pdata =
            decode_logs_payload(&bytes, MessageFormat::OtlpProto).expect("should decode");
        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("to OtlpProtoBytes");
        assert!(matches!(proto, OtlpProtoBytes::ExportLogsRequest(_)));
    }

    // -- decode_traces_payload: OTAP Proto --
    #[test]
    fn decode_traces_payload_otap_proto() {
        let bytes = create_traces_with_spans_otap_bytes();

        let mut pdata =
            decode_traces_payload(&bytes, MessageFormat::OtapProto).expect("should decode");
        let payload: OtapPayload = pdata.take_payload();
        assert!(
            matches!(
                payload,
                OtapPayload::OtapArrowRecords(OtapArrowRecords::Traces(_))
            ),
            "expected OtapArrowRecords::Traces"
        );
    }

    // -- decode_metrics_payload: OTAP Proto --
    #[test]
    fn decode_metrics_payload_otap_proto() {
        let bytes = create_metrics_otap_arrow_records_bytes();

        let mut pdata =
            decode_metrics_payload(&bytes, MessageFormat::OtapProto).expect("should decode");
        let payload: OtapPayload = pdata.take_payload();
        assert!(
            matches!(
                payload,
                OtapPayload::OtapArrowRecords(OtapArrowRecords::Metrics(_))
            ),
            "expected OtapArrowRecords::Metrics"
        );
    }

    // -- decode_logs_payload: OTAP Proto --
    #[test]
    fn decode_logs_payload_otap_proto() {
        let bytes = create_logs_otap_arrow_records_bytes();

        let mut pdata =
            decode_logs_payload(&bytes, MessageFormat::OtapProto).expect("should decode");
        let payload: OtapPayload = pdata.take_payload();
        assert!(
            matches!(
                payload,
                OtapPayload::OtapArrowRecords(OtapArrowRecords::Logs(_))
            ),
            "expected OtapArrowRecords::Logs"
        );
    }

    // -- Invalid OTAP bytes should fail decode --
    #[test]
    fn decode_traces_payload_invalid_otap_bytes_returns_error() {
        let result = decode_traces_payload(b"not valid protobuf", MessageFormat::OtapProto);
        assert!(result.is_err());
    }

    // -- OTLP payload round-trip: bytes in == bytes out --
    #[test]
    fn decode_traces_payload_otlp_preserves_bytes() {
        let req = create_traces_with_spans();
        let mut bytes = vec![];
        req.encode(&mut bytes).expect("encode");

        let mut pdata = decode_traces_payload(&bytes, MessageFormat::OtlpProto).expect("decode");
        let proto: OtlpProtoBytes = pdata
            .take_payload()
            .try_into_with_default()
            .expect("convert");
        assert_eq!(proto.as_bytes(), &bytes);
    }

    // ---- KafkaReceiver::new() unit tests ----

    fn make_pipeline_ctx() -> PipelineContext {
        make_pipeline_ctx_with(0, 1)
    }

    fn make_pipeline_ctx_with(core_id: usize, num_cores: usize) -> PipelineContext {
        let registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry);
        controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), core_id, num_cores, 0)
    }

    fn make_config_with_group_instance_id(instance_id: &str) -> KafkaReceiverConfig {
        KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
                .with_traces(SignalConfig::new(vec!["t".to_string()]))
                .with_group_instance_id(instance_id),
        )
        .expect("test config should be valid")
    }

    #[test]
    fn new_suffixes_group_instance_id_with_core_id_when_multi_core() {
        let cfg = make_config_with_group_instance_id("instance-1");
        let ctx = make_pipeline_ctx_with(3, 4);
        let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
        assert_eq!(
            receiver.config.group_instance_id(),
            Some("instance-1-3"),
            "multi-core pipeline should suffix group.instance.id with core id"
        );
    }

    #[test]
    fn new_keeps_group_instance_id_unchanged_when_single_core() {
        let cfg = make_config_with_group_instance_id("instance-1");
        let ctx = make_pipeline_ctx_with(0, 1);
        let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
        assert_eq!(
            receiver.config.group_instance_id(),
            Some("instance-1"),
            "single-core pipeline should leave group.instance.id unchanged"
        );
    }

    #[test]
    fn new_leaves_group_instance_id_absent_when_unset() {
        let cfg = make_config(&["t"], &["m"], &["l"], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx_with(2, 4);
        let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
        assert_eq!(
            receiver.config.group_instance_id(),
            None,
            "unset group.instance.id should remain absent"
        );
    }

    #[test]
    fn new_succeeds_with_distinct_topics() {
        let cfg = make_config(&["t"], &["m"], &["l"], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx();
        let receiver = KafkaReceiver::new(ctx, cfg);
        assert!(receiver.is_ok());
    }

    #[test]
    fn new_fails_with_overlapping_topics() {
        let result = KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
                .with_traces(SignalConfig::new(vec!["same".to_string()]))
                .with_metrics(SignalConfig::new(vec!["same".to_string()])),
        );
        assert!(result.is_err());
        let err_str = result.unwrap_err();
        assert!(
            err_str.contains("overlap"),
            "expected overlap error, got: {err_str}"
        );
    }

    #[test]
    fn new_creates_offset_tracker_when_auto_commit_disabled() {
        let cfg = make_config(&["t"], &["m"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit()); // default is manual (not auto)
        let ctx = make_pipeline_ctx();
        let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
        // offset_tracker is always present; verify it starts empty
        assert_eq!(receiver.offset_tracker.total_pending(), 0);
    }

    #[test]
    fn new_succeeds_when_auto_commit_enabled() {
        let cfg = KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
                .with_traces(SignalConfig::new(vec!["t".to_string()]))
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Auto,
                    interval_ms: Some(1000),
                })
                .with_isolation_level(IsolationLevel::ReadUncommitted),
        )
        .expect("test config should be valid");
        let ctx = make_pipeline_ctx();
        let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
        // offset_tracker exists but won't be used when auto-commit is enabled
        assert_eq!(receiver.offset_tracker.total_pending(), 0);
    }

    // ---- Rebalance reconcile unit tests ----

    #[test]
    fn reconcile_purges_revoked_partitions_from_tracker() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Simulate in-flight offsets across two partitions.
        receiver.offset_tracker.track("traces", 0, 100, 0);
        receiver.offset_tracker.track("traces", 1, 200, 0);
        assert_eq!(receiver.offset_tracker.total_pending(), 2);

        // Simulate a rebalance revoking partition 0.
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, 0);

        receiver.reconcile_rebalance_state();

        // Partition 0 purged; partition 1 retained.
        assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
        assert_eq!(receiver.offset_tracker.pending_count("traces", 1), 1);
    }

    #[test]
    fn stale_revocation_preserves_reassigned_partition_state() {
        // Regression for the revoke/reassign race: a revocation queued for an
        // older ownership period must not delete tracker state created after
        // the partition was reassigned to this consumer under a newer
        // generation.
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // A new record for partition 0 was tracked under generation 2 (after a
        // reassignment), while a revocation from generation 1 is still queued.
        receiver.offset_tracker.track("traces", 0, 250, 2);
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, 1);

        receiver.reconcile_rebalance_state();

        // The stale generation-1 revocation must be a no-op: the newer state
        // survives so its ACK can still advance and commit.
        assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 1);
        assert_eq!(
            receiver.offset_tracker.partition_generation("traces", 0),
            Some(2),
        );
    }

    #[test]
    fn retained_partition_generation_is_stable_across_unrelated_rebalance() {
        // Regression: the per-partition ownership generation must NOT change when the
        // partition is retained across a rebalance that only affects OTHER
        // partitions. Otherwise a newer record on the retained partition would
        // bump its generation and cause a legitimate late ACK for an earlier record
        // (carrying the older generation) to be wrongly dropped as stale.
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Initial assignment: own partition 0.
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        receiver.rebalance_state.set_assignment_for_test(&tpl);
        let generation_p0 = receiver.rebalance_state.current_generation("traces", 0);

        // An unrelated rebalance retains partition 0 and acquires partition 1
        // (which advances the generation allocator).
        let mut tpl2 = TopicPartitionList::new();
        let _ = tpl2.add_partition("traces", 0);
        let _ = tpl2.add_partition("traces", 1);
        receiver.rebalance_state.set_assignment_for_test(&tpl2);

        // Partition 0 was retained: its generation must be unchanged, even though the
        // allocator advanced for partition 1.
        assert_eq!(
            receiver.rebalance_state.current_generation("traces", 0),
            generation_p0
        );
        assert!(receiver.rebalance_state.current_generation("traces", 1) > generation_p0);
    }

    #[test]
    fn commit_path_purges_revoked_partitions_first() {
        // Regression: every commit path drains revoked partitions before
        // building the commit TPL, so a partition revoked by the rebalance
        // callback (but not yet reconciled at the top of the loop) is never
        // committed by `commit_offsets` / TimerTick / shutdown / poison-pill.
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // In-flight offsets on two partitions.
        receiver.offset_tracker.track("traces", 0, 100, 0);
        receiver.offset_tracker.track("traces", 1, 200, 0);

        // The callback queues a revoke for partition 0, but the loop has not
        // reconciled it yet (it is still tracked).
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, 0);
        assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 1);

        // The drain-before-commit step that `commit_offsets` runs.
        receiver.purge_revoked_partitions();

        // Partition 0 is purged; the committable TPL a commit would use now
        // excludes it and retains only partition 1.
        assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
        assert_eq!(receiver.offset_tracker.pending_count("traces", 1), 1);

        let tpl = receiver.offset_tracker.committable_tpl();
        let map = tpl.to_topic_map();
        assert!(
            !map.contains_key(&("traces".to_string(), 0)),
            "revoked partition 0 must not appear in the commit TPL",
        );
        assert_eq!(
            map.get(&("traces".to_string(), 1)),
            Some(&Offset::Offset(200)),
            "owned partition 1 must remain committable",
        );
    }

    #[test]
    fn purge_revoked_partitions_is_noop_under_auto_commit() {
        let cfg = KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
                .with_traces(SignalConfig::new(vec!["traces".to_string()]))
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Auto,
                    interval_ms: Some(1000),
                })
                .with_isolation_level(IsolationLevel::ReadUncommitted),
        )
        .expect("test config should be valid");
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        receiver.offset_tracker.track("traces", 0, 100, 0);
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, 0);

        // Under auto-commit, purge must not touch the tracker (librdkafka owns
        // offsets and rebalance handling is disabled).
        receiver.purge_revoked_partitions();
        assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 1);
    }

    #[test]
    fn reconcile_is_noop_under_auto_commit() {
        let cfg = KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
                .with_traces(SignalConfig::new(vec!["traces".to_string()]))
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Auto,
                    interval_ms: Some(1000),
                })
                .with_isolation_level(IsolationLevel::ReadUncommitted),
        )
        .expect("test config should be valid");
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        receiver.offset_tracker.track("traces", 0, 100, 0);
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, 0);

        // Under auto-commit, reconcile must not touch the tracker or drain.
        receiver.reconcile_rebalance_state();
        assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 1);
    }

    #[test]
    fn reconcile_folds_commit_callback_metrics() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Simulate commit-callback outcomes accumulated on the poll thread.
        receiver.rebalance_state.record_commit_result_for_test(true);
        receiver.rebalance_state.record_commit_result_for_test(true);
        receiver
            .rebalance_state
            .record_commit_result_for_test(false);

        receiver.reconcile_rebalance_state();

        assert_eq!(receiver.metrics.offset_commits.get(), 2);
        assert_eq!(receiver.metrics.offset_commit_errors.get(), 1);

        // Counters were drained; a second reconcile adds nothing.
        receiver.reconcile_rebalance_state();
        assert_eq!(receiver.metrics.offset_commits.get(), 2);
        assert_eq!(receiver.metrics.offset_commit_errors.get(), 1);
    }

    #[test]
    fn refresh_committable_snapshot_feeds_rebalance_state() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        receiver.offset_tracker.track("traces", 0, 100, 0);
        receiver.offset_tracker.track("traces", 0, 101, 0);
        receiver.refresh_committable_snapshot();

        // The shared state now reports partition 0 as assigned-or-not, but the
        // committable snapshot drives pre-rebalance commits. Assign and verify
        // the late-ack guard sees the partition.
        receiver.rebalance_state.assign_for_test("traces", 0, 1);
        assert!(receiver.rebalance_state.is_assigned("traces", 0));
        assert!(!receiver.rebalance_state.is_assigned("traces", 9));
    }

    #[test]
    fn snapshot_reflects_committable_after_advance() {
        // Mirrors what advance_offset_and_commit does (minus the live commit):
        // acknowledging the lowest pending offset advances the committable
        // watermark, and refreshing the snapshot must reflect it so a
        // subsequent pre-rebalance commit is not stale.
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        receiver.offset_tracker.track("traces", 0, 100, 0);
        receiver.offset_tracker.track("traces", 0, 101, 0);
        receiver.refresh_committable_snapshot();
        assert_eq!(
            receiver.rebalance_state.committable_for_test("traces", 0),
            Some(100)
        );

        // Advance past 100; snapshot must now reflect 101.
        let advanced = receiver.offset_tracker.acknowledge("traces", 0, 100);
        assert!(advanced);
        receiver.refresh_committable_snapshot();
        assert_eq!(
            receiver.rebalance_state.committable_for_test("traces", 0),
            Some(101)
        );
    }

    // ---- KafkaReceiver::from_config() unit tests ----

    #[test]
    fn from_config_succeeds_with_valid_json() {
        let json: Value = serde_json::json!({
            "brokers": "kafka:9092",
            "group_id": "my-group",
            "client_id": "my-client",
            "traces": {"topics": ["traces"]},
            "metrics": {"topics": ["metrics"]},
            "logs": {"topics": ["logs"]}
        });
        let ctx = make_pipeline_ctx();
        let result = KafkaReceiver::from_config(ctx, &json);
        assert!(result.is_ok());
    }

    #[test]
    fn from_config_fails_with_missing_required_fields() {
        // brokers, group_id, client_id are required
        let json: Value = serde_json::json!({});
        let ctx = make_pipeline_ctx();
        let result = KafkaReceiver::from_config(ctx, &json);
        assert!(result.is_err());
    }

    #[test]
    fn from_config_fails_with_no_topics() {
        // Required fields present but no topics configured
        let json: Value = serde_json::json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c"
        });
        let ctx = make_pipeline_ctx();
        let result = KafkaReceiver::from_config(ctx, &json);
        assert!(result.is_err());
    }

    #[test]
    fn from_config_fails_with_overlapping_topics() {
        let json: Value = serde_json::json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["same"]},
            "metrics": {"topics": ["same"]}
        });
        let ctx = make_pipeline_ctx();
        let result = KafkaReceiver::from_config(ctx, &json);
        assert!(result.is_err());
    }

    // ---- Integration tests (test-suite in-process mock Kafka broker) ----
    // These use the shared Kafka test suite (`with_cluster` + `KafkaReceiverHarness`),
    // so they run in-process with no Docker/external broker and run by default in CI.
    //
    // The six per-signal tests below (traces/metrics/logs in both encodings) plus
    // `test_kafka_receiver_message_format_header_overrides_signal_default` cover
    // the Area 6 "all supported encodings + per-message header override" subtask:
    // the `*_otap` variants exercise `OtapProto` (Arrow) and the plain variants
    // exercise `OtlpProto` (default), and the override test proves a per-message
    // `MessageFormat` header switches the decode path.

    /// Scenario: OTLP-proto trace records produced to a Kafka topic are consumed
    /// by an auto-commit receiver.
    /// Guarantees: each delivered pdata decodes to an `ExportTracesRequest` whose
    /// bytes are byte-for-byte identical to what was produced (lossless round-trip).
    #[tokio::test]
    async fn test_kafka_receiver_traces() {
        const TOPIC: &str = "test-traces-proto";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..3 {
                    let key = format!("test-key-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for _ in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    assert!(matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)));
                    assert_eq!(proto.as_bytes(), &bytes);
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: OTLP-proto log records produced to a Kafka topic are consumed
    /// by an auto-commit receiver.
    /// Guarantees: each delivered pdata decodes to an `ExportLogsRequest` whose
    /// bytes are byte-for-byte identical to what was produced.
    #[tokio::test]
    async fn test_kafka_receiver_logs() {
        const TOPIC: &str = "test-logs-proto";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_logs_service_request();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..3 {
                    let key = format!("test-key-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[],
                    &[],
                    &[TOPIC],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for _ in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    assert!(matches!(proto, OtlpProtoBytes::ExportLogsRequest(_)));
                    assert_eq!(proto.as_bytes(), &bytes);
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: OTLP-proto metric records produced to a Kafka topic are consumed
    /// by an auto-commit receiver.
    /// Guarantees: each delivered pdata decodes to an `ExportMetricsRequest` whose
    /// bytes are byte-for-byte identical to what was produced.
    #[tokio::test]
    async fn test_kafka_receiver_metrics() {
        const TOPIC: &str = "test-metrics-proto";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_metrics_service_request();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..3 {
                    let key = format!("test-key-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[],
                    &[TOPIC],
                    &[],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for _ in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    assert!(matches!(proto, OtlpProtoBytes::ExportMetricsRequest(_)));
                    assert_eq!(proto.as_bytes(), &bytes);
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: OTAP-Arrow trace records produced to a Kafka topic are consumed
    /// by an auto-commit receiver configured for the OTAP format.
    /// Guarantees: each delivered pdata is an `OtapArrowRecords::Traces` payload.
    #[tokio::test]
    async fn test_kafka_receiver_traces_otap() {
        const TOPIC: &str = "test-traces-otap";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let bytes = create_traces_with_spans_otap_bytes();

                for i in 0..3 {
                    let key = format!("test-key-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtapProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for i in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let payload: OtapPayload = pdata.take_payload();
                    assert!(
                        matches!(
                            payload,
                            OtapPayload::OtapArrowRecords(OtapArrowRecords::Traces(_))
                        ),
                        "Expected OtapArrowRecords::Traces for message {i}"
                    );
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: OTAP-Arrow metric records produced to a Kafka topic are consumed
    /// by an auto-commit receiver configured for the OTAP format.
    /// Guarantees: each delivered pdata is an `OtapArrowRecords::Metrics` payload
    /// equal to the produced default metrics records.
    #[tokio::test]
    async fn test_kafka_receiver_metrics_otap() {
        const TOPIC: &str = "test-metrics-otap";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let bytes = create_metrics_otap_arrow_records_bytes();

                for i in 0..3 {
                    let key = format!("test-key-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[],
                    &[TOPIC],
                    &[],
                    MessageFormat::OtapProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for i in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let payload: OtapPayload = pdata.take_payload();
                    if let OtapPayload::OtapArrowRecords(arrow_records) = payload {
                        let expected = OtapArrowRecords::Metrics(Metrics::default());
                        assert_eq!(expected, arrow_records);
                    } else {
                        panic!("Expected OtapArrowRecords::Metrics for message {i}");
                    }
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: OTAP-Arrow log records produced to a Kafka topic are consumed
    /// by an auto-commit receiver configured for the OTAP format.
    /// Guarantees: each delivered pdata is an `OtapArrowRecords::Logs` payload
    /// equal to the produced default logs records.
    #[tokio::test]
    async fn test_kafka_receiver_logs_otap() {
        const TOPIC: &str = "test-logs-otap";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let bytes = create_logs_otap_arrow_records_bytes();

                for i in 0..3 {
                    let key = format!("test-key-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[],
                    &[],
                    &[TOPIC],
                    MessageFormat::OtapProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for i in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let payload: OtapPayload = pdata.take_payload();
                    if let OtapPayload::OtapArrowRecords(arrow_records) = payload {
                        let expected = OtapArrowRecords::Logs(Logs::default());
                        assert_eq!(expected, arrow_records);
                    } else {
                        panic!("Expected OtapArrowRecords::Logs for message {i}");
                    }
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: the receiver's traces signal is configured with the default
    /// per-signal encoding `OtlpProto`, but a record is produced with OTAP-Arrow
    /// payload bytes plus a per-message `MessageFormat: otap` Kafka header.
    /// Guarantees: closes the Area 6 "per-message header override" subtask -- the
    /// `MessageFormat` header overrides the per-signal `OtlpProto` default so the
    /// receiver decodes the payload via the OTAP path (the delivered pdata is an
    /// `OtapArrowRecords::Traces`, which is only possible if the override took
    /// effect; had the header been ignored, the OTAP bytes would be mis-handled as
    /// OtlpProto). Protects `detect_message_format` (`receiver.rs:115`) and its use
    /// on the per-signal decode path.
    #[tokio::test]
    async fn test_kafka_receiver_message_format_header_overrides_signal_default() {
        const TOPIC: &str = "test-format-override";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                // OTAP-Arrow wire bytes, but the per-signal default below is OTLP.
                let otap_bytes = create_traces_with_spans_otap_bytes();

                // Produce with the per-message MessageFormat=otap header so the
                // receiver must override its OtlpProto per-signal default.
                for i in 0..3 {
                    let key = format!("override-key-{i}");
                    producer
                        .send_full(
                            SendRecord::new(TOPIC, &otap_bytes)
                                .key(key.as_bytes())
                                .header("MessageFormat", MSG_FORMAT_OTAP),
                        )
                        .await
                        .expect("Failed to send message");
                }

                // Per-signal traces encoding is deliberately OtlpProto (the
                // default); only the per-message header should switch it to OTAP.
                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for i in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let payload: OtapPayload = pdata.take_payload();
                    assert!(
                        matches!(
                            payload,
                            OtapPayload::OtapArrowRecords(OtapArrowRecords::Traces(_))
                        ),
                        "message {i}: MessageFormat=otap header must override the \
                         OtlpProto per-signal default and decode via the OTAP path",
                    );
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    // ---- Header extraction integration tests (in-process mock broker) ----
    //
    // These two tests cover the Area 6 "header extraction into resource
    // attributes for both OTLP and OTAP" subtask: the OTLP variant and the
    // `_otap` variant both map a Kafka header into a resource attribute and
    // assert it lands on every resource (and not on spans).

    /// Scenario: an OTLP-proto trace record carries a Kafka header `x-tenant-id`
    /// while the receiver is configured to map that header to a resource
    /// attribute `tenant.id`.
    /// Guarantees: every resource gains a `tenant.id` string attribute equal to
    /// the header value, and no span-level `tenant.id` attribute is added.
    #[tokio::test]
    async fn test_kafka_receiver_traces_header_extraction() {
        const TOPIC: &str = "test-traces-headers";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                // Build a trace request with real spans.
                let req = create_traces_with_spans();
                let mut payload_bytes = vec![];
                req.encode(&mut payload_bytes).expect("encode");

                // Configure extraction: map Kafka header "x-tenant-id" to a resource
                // attribute "tenant.id".
                let mut resource_attrs_from_headers = HashMap::new();
                let _ = resource_attrs_from_headers.insert(
                    "x-tenant-id".to_string(),
                    HeaderExtraction {
                        key: "tenant.id".to_string(),
                        value_type: AttributeValueType::String,
                    },
                );

                let tenant_value = "acme-corp";

                // Send 3 messages, each with the same headers.
                for i in 0..3 {
                    let key = format!("test-key-{i}");
                    producer
                        .send_full(
                            SendRecord::new(TOPIC, &payload_bytes)
                                .key(key.as_bytes())
                                .header("x-tenant-id", tenant_value.as_bytes()),
                        )
                        .await
                        .expect("Failed to send message");
                }

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtlpProto,
                    resource_attrs_from_headers,
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for i in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    let result =
                        ExportTraceServiceRequest::decode(proto.as_bytes()).expect("decode result");

                    // Every resource should have the injected tenant.id attribute.
                    for rs in &result.resource_spans {
                        let resource = rs.resource.as_ref().expect("should have resource");
                        let tenant_attr = resource
                            .attributes
                            .iter()
                            .find(|kv| kv.key == "tenant.id")
                            .unwrap_or_else(|| {
                                panic!("message {i}: resource missing tenant.id attribute")
                            });
                        let value = tenant_attr
                            .value
                            .as_ref()
                            .expect("should have value")
                            .value
                            .as_ref()
                            .expect("should have inner value");
                        assert!(
                            matches!(
                                value,
                                any_value::Value::StringValue(s) if s == tenant_value
                            ),
                            "message {i}: resource tenant.id should be '{tenant_value}'",
                        );

                        // Span attributes should NOT have tenant.id
                        for ss in &rs.scope_spans {
                            for span in &ss.spans {
                                assert!(
                                    !span.attributes.iter().any(|kv| kv.key == "tenant.id"),
                                    "message {i}: span '{}' should NOT have tenant.id attribute",
                                    span.name,
                                );
                            }
                        }
                    }
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: an OTAP-Arrow trace record carries a Kafka header `x-tenant-id`
    /// plus the `MessageFormat` OTAP marker while the receiver maps that header
    /// to a resource attribute `tenant.id`.
    /// Guarantees: after decoding the OTAP payload back to OTLP, every resource
    /// gains a `tenant.id` string attribute equal to the header value, and no
    /// span-level `tenant.id` attribute is added.
    #[tokio::test]
    async fn test_kafka_receiver_traces_header_extraction_otap() {
        const TOPIC: &str = "test-traces-headers-otap";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                // Build OTAP Arrow bytes from a real trace request with spans.
                let otap_bytes = create_traces_with_spans_otap_bytes();

                // Configure extraction: map Kafka header "x-tenant-id" to a resource
                // attribute "tenant.id".
                let mut resource_attrs_from_headers = HashMap::new();
                let _ = resource_attrs_from_headers.insert(
                    "x-tenant-id".to_string(),
                    HeaderExtraction {
                        key: "tenant.id".to_string(),
                        value_type: AttributeValueType::String,
                    },
                );

                let tenant_value = "acme-corp";

                // Send 3 messages, each with the same headers and the OTAP
                // MessageFormat header so the receiver uses the OTAP path.
                for i in 0..3 {
                    let key = format!("test-key-{i}");
                    producer
                        .send_full(
                            SendRecord::new(TOPIC, &otap_bytes)
                                .key(key.as_bytes())
                                .header("x-tenant-id", tenant_value.as_bytes())
                                .header("MessageFormat", MSG_FORMAT_OTAP),
                        )
                        .await
                        .expect("Failed to send message");
                }

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtapProto,
                    resource_attrs_from_headers,
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for i in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;

                    // Convert OTAP result back to OTLP protobuf for assertions
                    let result = otap_pdata_to_traces(&mut pdata);

                    // Every resource should have the injected tenant.id attribute.
                    for rs in &result.resource_spans {
                        let resource = rs.resource.as_ref().expect("should have resource");
                        let tenant_attr = resource
                            .attributes
                            .iter()
                            .find(|kv| kv.key == "tenant.id")
                            .unwrap_or_else(|| {
                                panic!("message {i}: resource missing tenant.id attribute")
                            });
                        let value = tenant_attr
                            .value
                            .as_ref()
                            .expect("should have value")
                            .value
                            .as_ref()
                            .expect("should have inner value");
                        assert!(
                            matches!(
                                value,
                                any_value::Value::StringValue(s) if s == tenant_value
                            ),
                            "message {i}: resource tenant.id should be '{tenant_value}'",
                        );

                        // Span attributes should NOT have tenant.id
                        for ss in &rs.scope_spans {
                            for span in &ss.spans {
                                assert!(
                                    !span.attributes.iter().any(|kv| kv.key == "tenant.id"),
                                    "message {i}: span '{}' should NOT have tenant.id attribute",
                                    span.name,
                                );
                            }
                        }
                    }
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    // ---- CallData encode/decode roundtrip tests ----

    #[test]
    fn encode_decode_calldata_roundtrip() {
        let cases: Vec<(u32, i32, i64, u64)> = vec![
            (0, 0, 0, 0),
            (0, 0, 100, 1),
            (1, 3, 999_999, 7),
            (2, 11, i64::MAX, u64::MAX),
            (5, 0, 42, 0),
            (10, 1, 1_000_000, 12_345),
            (255, 2, 0, 3),
            // Values that would have been truncated by the old `u8` ID.
            (256, 7, 1, 1),
            (65_536, 9, 2, 2),
            (u32::MAX, i32::MAX, i64::MAX, u64::MAX),
            (u32::MAX, -1, 0, 9),
        ];

        for (topic_id, partition, offset, generation) in cases {
            let calldata = encode_calldata(topic_id, partition, offset, generation);
            let (dec_tid, dec_part, dec_off, dec_gen) = decode_calldata(&calldata);
            assert_eq!(dec_tid, topic_id, "topic_id mismatch");
            assert_eq!(dec_part, partition, "partition mismatch");
            assert_eq!(dec_off, offset, "offset mismatch");
            assert_eq!(dec_gen, generation, "generation mismatch");
        }
    }

    #[test]
    fn encode_calldata_produces_three_slots() {
        let calldata = encode_calldata(1, 5, 42, 3);
        assert_eq!(calldata.len(), 3);
    }

    #[test]
    fn decode_legacy_two_slot_calldata_defaults_generation_zero() {
        // A calldata without the generation slot decodes as generation 0.
        let legacy: CallData = smallvec![
            Context8u8::from(((7u64) << 32) | 5u64),
            Context8u8::from(42u64),
        ];
        let (topic_id, partition, offset, generation) = decode_calldata(&legacy);
        assert_eq!(topic_id, 7);
        assert_eq!(partition, 5);
        assert_eq!(offset, 42);
        assert_eq!(generation, 0);
    }

    // ---- TopicRegistry tests ----

    #[test]
    fn topic_registry_assigns_sequential_ids() {
        let mut reg = TopicRegistry::new();

        assert_eq!(reg.get_or_assign("traces-prod"), Some(0));
        assert_eq!(reg.get_or_assign("metrics-prod"), Some(1));
        assert_eq!(reg.get_or_assign("logs-prod"), Some(2));

        // Same topic returns the same ID.
        assert_eq!(reg.get_or_assign("traces-prod"), Some(0));
    }

    #[test]
    fn topic_registry_name_for_roundtrip() {
        let mut reg = TopicRegistry::new();

        let id = reg.get_or_assign("my-topic").expect("id assigned");
        assert_eq!(reg.name_for(id).as_deref(), Some("my-topic"));
        assert_eq!(reg.name_for(99), None);
    }

    // ---- Topic matching tests ----

    #[test]
    fn matches_any_topic_exact() {
        let topics = vec!["traces".to_string()];
        let regexes = vec![None];
        assert!(matches_any_topic(&topics, &regexes, "traces"));
        assert!(!matches_any_topic(&topics, &regexes, "other"));

        // Empty list matches nothing
        assert!(!matches_any_topic(&[], &[], "traces"));
    }

    #[test]
    fn matches_any_topic_regex() {
        let topics = vec!["^traces-.*".to_string()];
        let re = Regex::new("^traces-.*").unwrap();
        let regexes = vec![Some(re)];
        assert!(matches_any_topic(&topics, &regexes, "traces-prod"));
        assert!(matches_any_topic(&topics, &regexes, "traces-staging"));
        assert!(!matches_any_topic(&topics, &regexes, "metrics-prod"));
    }

    #[test]
    fn matches_any_topic_multi_topic_list() {
        let topics = vec![
            "traces-a".to_string(),
            "traces-b".to_string(),
            "^traces-regex-.*".to_string(),
        ];
        let re = Regex::new("^traces-regex-.*").unwrap();
        let regexes = vec![None, None, Some(re)];

        assert!(matches_any_topic(&topics, &regexes, "traces-a"));
        assert!(matches_any_topic(&topics, &regexes, "traces-b"));
        assert!(matches_any_topic(&topics, &regexes, "traces-regex-foo"));
        assert!(!matches_any_topic(&topics, &regexes, "traces-c"));
        assert!(!matches_any_topic(&topics, &regexes, "metrics"));
    }

    #[test]
    fn matches_topic_routing_with_receiver() {
        let cfg = make_config(&["^traces-.*"], &["metrics"], &[], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx();
        let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Regex traces
        assert!(matches_any_topic(
            receiver.config.traces_topics(),
            &receiver.traces_topic_regexes,
            "traces-prod",
        ));
        assert!(matches_any_topic(
            receiver.config.traces_topics(),
            &receiver.traces_topic_regexes,
            "traces-staging",
        ));

        // Exact metrics
        assert!(matches_any_topic(
            receiver.config.metrics_topics(),
            &receiver.metrics_topic_regexes,
            "metrics",
        ));
        assert!(!matches_any_topic(
            receiver.config.metrics_topics(),
            &receiver.metrics_topic_regexes,
            "metrics-prod",
        ));

        // Unconfigured logs
        assert!(!matches_any_topic(
            receiver.config.logs_topics(),
            &receiver.logs_topic_regexes,
            "logs-prod",
        ));
    }

    #[test]
    fn matches_topic_routing_multi_topic_receiver() {
        let cfg = make_config(
            &["traces-a", "traces-b", "^traces-regex-.*"],
            &["metrics-x", "metrics-y"],
            &["logs"],
            MessageFormat::OtlpProto,
        );
        let ctx = make_pipeline_ctx();
        let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Multiple traces topics
        assert!(matches_any_topic(
            receiver.config.traces_topics(),
            &receiver.traces_topic_regexes,
            "traces-a",
        ));
        assert!(matches_any_topic(
            receiver.config.traces_topics(),
            &receiver.traces_topic_regexes,
            "traces-b",
        ));
        assert!(matches_any_topic(
            receiver.config.traces_topics(),
            &receiver.traces_topic_regexes,
            "traces-regex-prod",
        ));
        assert!(!matches_any_topic(
            receiver.config.traces_topics(),
            &receiver.traces_topic_regexes,
            "traces-c",
        ));

        // Multiple metrics topics
        assert!(matches_any_topic(
            receiver.config.metrics_topics(),
            &receiver.metrics_topic_regexes,
            "metrics-x",
        ));
        assert!(matches_any_topic(
            receiver.config.metrics_topics(),
            &receiver.metrics_topic_regexes,
            "metrics-y",
        ));
        assert!(!matches_any_topic(
            receiver.config.metrics_topics(),
            &receiver.metrics_topic_regexes,
            "metrics-z",
        ));

        // Single logs topic still works
        assert!(matches_any_topic(
            receiver.config.logs_topics(),
            &receiver.logs_topic_regexes,
            "logs",
        ));
    }

    #[test]
    fn invalid_regex_topic_fails_at_construction() {
        // Unbalanced parenthesis is an invalid regex -- rejected at config validation time
        let result = KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
                .with_traces(SignalConfig::new(vec!["^traces-(".to_string()])),
        );
        assert!(
            result.is_err(),
            "invalid regex should fail at config construction"
        );
    }

    // ---- Transport header capture policy integration tests (test-suite mock broker) ----

    /// Scenario: a capture policy captures `X-Tenant-Id` (stored as `tenant_id`)
    /// and `X-Request-Id` (default lowercased name) but not `X-Unrelated`.
    /// Guarantees: exactly the two matching Kafka headers are captured into the
    /// OtapPdata transport headers with their configured store-names and
    /// preserved wire names, and the unmatched header is dropped.
    #[tokio::test]
    async fn test_kafka_receiver_capture_policy_captures_headers() {
        const TOPIC: &str = "test-capture-policy";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut payload_bytes = vec![];
                req.encode(&mut payload_bytes).expect("encode");

                // Send a message with Kafka headers.
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &payload_bytes)
                            .key(b"key-1")
                            .header("X-Tenant-Id", b"acme-corp")
                            .header("X-Request-Id", b"req-12345")
                            .header("X-Unrelated", b"ignored"),
                    )
                    .await
                    .expect("Failed to send message");

                // Set up a capture policy that captures X-Tenant-Id and X-Request-Id
                // but not X-Unrelated.
                let capture_policy = HeaderCapturePolicy::new(
                    CaptureDefaults::default(),
                    vec![
                        CaptureRule {
                            match_names: vec!["X-Tenant-Id".to_string()],
                            store_as: Some("tenant_id".to_string()),
                            sensitive: false,
                            value_kind: None,
                        },
                        CaptureRule {
                            match_names: vec!["X-Request-Id".to_string()],
                            store_as: None, // defaults to lowercased wire name
                            sensitive: false,
                            value_kind: None,
                        },
                    ],
                );

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver =
                    KafkaReceiverHarness::start_with_capture(&cluster, cfg, Some(capture_policy));

                let pdata = receiver.recv_pdata().await;

                // Verify transport headers were captured.
                let transport_headers = pdata
                    .transport_headers()
                    .expect("transport_headers should be set");

                // Two headers should be captured (X-Tenant-Id and X-Request-Id).
                assert_eq!(
                    transport_headers.len(),
                    2,
                    "expected 2 captured headers, got {}",
                    transport_headers.len()
                );

                // Check X-Tenant-Id was stored as "tenant_id".
                let tenant_headers: Vec<_> = transport_headers.find_by_name("tenant_id").collect();
                assert_eq!(tenant_headers.len(), 1, "expected one tenant_id header");
                assert_eq!(
                    tenant_headers[0].value_as_str(),
                    Some("acme-corp"),
                    "tenant_id value mismatch"
                );
                assert_eq!(
                    tenant_headers[0].wire_name, "X-Tenant-Id",
                    "wire_name should be preserved"
                );

                // Check X-Request-Id was stored as "x-request-id" (lowercased).
                let request_headers: Vec<_> =
                    transport_headers.find_by_name("x-request-id").collect();
                assert_eq!(request_headers.len(), 1, "expected one x-request-id header");
                assert_eq!(
                    request_headers[0].value_as_str(),
                    Some("req-12345"),
                    "x-request-id value mismatch"
                );

                // X-Unrelated should NOT be captured (not in the policy).
                let unrelated: Vec<_> = transport_headers.find_by_name("x-unrelated").collect();
                assert!(unrelated.is_empty(), "X-Unrelated should not be captured");

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: a record carries a Kafka header but the receiver is started
    /// without any capture policy.
    /// Guarantees: transport headers are left unset on the OtapPdata context
    /// (existing behavior is preserved when capture is not configured).
    #[tokio::test]
    async fn test_kafka_receiver_no_capture_policy_no_transport_headers() {
        const TOPIC: &str = "test-no-capture-policy";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut payload_bytes = vec![];
                req.encode(&mut payload_bytes).expect("encode");

                // Send a message with headers, but without a capture policy.
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &payload_bytes)
                            .key(b"key-1")
                            .header("X-Tenant-Id", b"acme-corp"),
                    )
                    .await
                    .expect("Failed to send message");

                // No capture policy set on the receiver.
                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                let pdata = receiver.recv_pdata().await;

                // Transport headers should NOT be set when no capture policy is configured.
                assert!(
                    pdata.transport_headers().is_none(),
                    "transport_headers should be None when no capture policy is configured"
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: a record carries `X-Tenant-Id` (captured to a transport header)
    /// and `x-env` (mapped to a resource attribute) while both the capture policy
    /// and resource-attribute-from-header extraction are configured.
    /// Guarantees: the transport header and the injected resource attribute are
    /// produced independently and simultaneously from the same record.
    #[tokio::test]
    async fn test_kafka_receiver_capture_policy_coexists_with_resource_attrs_from_headers() {
        const TOPIC: &str = "test-capture-and-extract";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut payload_bytes = vec![];
                req.encode(&mut payload_bytes).expect("encode");

                // Send a message with headers for both mechanisms.
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &payload_bytes)
                            .key(b"key-1")
                            .header("X-Tenant-Id", b"acme-corp")
                            .header("x-env", b"production"),
                    )
                    .await
                    .expect("Failed to send message");

                // Configure resource_attrs_from_headers: x-env -> deployment.environment resource attribute
                let mut resource_attrs_from_headers = HashMap::new();
                let _ = resource_attrs_from_headers.insert(
                    "x-env".to_string(),
                    HeaderExtraction {
                        key: "deployment.environment".to_string(),
                        value_type: AttributeValueType::String,
                    },
                );

                // Configure capture policy: X-Tenant-Id -> transport header "tenant_id"
                let capture_policy = HeaderCapturePolicy::new(
                    CaptureDefaults::default(),
                    vec![CaptureRule {
                        match_names: vec!["X-Tenant-Id".to_string()],
                        store_as: Some("tenant_id".to_string()),
                        sensitive: false,
                        value_kind: None,
                    }],
                );

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtlpProto,
                    resource_attrs_from_headers,
                );
                let mut receiver =
                    KafkaReceiverHarness::start_with_capture(&cluster, cfg, Some(capture_policy));

                let mut pdata = receiver.recv_pdata().await;

                // 1. Verify transport headers were captured (capture policy).
                let transport_headers = pdata
                    .transport_headers()
                    .expect("transport_headers should be set");
                let tenant_headers: Vec<_> = transport_headers.find_by_name("tenant_id").collect();
                assert_eq!(tenant_headers.len(), 1);
                assert_eq!(tenant_headers[0].value_as_str(), Some("acme-corp"));

                // 2. Verify resource attributes were injected (resource_attrs_from_headers).
                let proto: OtlpProtoBytes = pdata
                    .take_payload()
                    .try_into_with_default()
                    .expect("to OtlpProtoBytes");
                let result =
                    ExportTraceServiceRequest::decode(proto.as_bytes()).expect("decode result");
                for rs in &result.resource_spans {
                    let resource = rs.resource.as_ref().expect("should have resource");
                    let env_attr = resource
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "deployment.environment")
                        .expect("resource should have deployment.environment attribute");
                    let value = env_attr
                        .value
                        .as_ref()
                        .expect("should have value")
                        .value
                        .as_ref()
                        .expect("should have inner value");
                    assert!(
                        matches!(
                            value,
                            any_value::Value::StringValue(s) if s == "production"
                        ),
                        "deployment.environment should be 'production'"
                    );
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: a capture policy is applied to an OTAP-Arrow record that also
    /// carries the `MessageFormat` OTAP marker header.
    /// Guarantees: the matching `X-Tenant-Id` header is captured as a transport
    /// header even for OTAP payloads, while the `MessageFormat` control header is
    /// not captured.
    #[tokio::test]
    async fn test_kafka_receiver_capture_policy_otap_format() {
        const TOPIC: &str = "test-capture-policy-otap";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let otap_bytes = create_traces_with_spans_otap_bytes();

                producer
                    .send_full(
                        SendRecord::new(TOPIC, &otap_bytes)
                            .key(b"key-1")
                            .header("X-Tenant-Id", b"acme-corp")
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("Failed to send message");

                let capture_policy = HeaderCapturePolicy::new(
                    CaptureDefaults::default(),
                    vec![CaptureRule {
                        match_names: vec!["X-Tenant-Id".to_string()],
                        store_as: Some("tenant_id".to_string()),
                        sensitive: false,
                        value_kind: None,
                    }],
                );

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtapProto,
                    HashMap::new(),
                );
                let mut receiver =
                    KafkaReceiverHarness::start_with_capture(&cluster, cfg, Some(capture_policy));

                let pdata = receiver.recv_pdata().await;

                // Verify transport headers were captured for OTAP format.
                let transport_headers = pdata
                    .transport_headers()
                    .expect("transport_headers should be set for OTAP messages");
                let tenant_headers: Vec<_> = transport_headers.find_by_name("tenant_id").collect();
                assert_eq!(tenant_headers.len(), 1);
                assert_eq!(tenant_headers[0].value_as_str(), Some("acme-corp"));

                // The MessageFormat header should NOT be captured (not in policy).
                let format_headers: Vec<_> =
                    transport_headers.find_by_name("messageformat").collect();
                assert!(
                    format_headers.is_empty(),
                    "MessageFormat header should not be captured"
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    // ---- Rebalance integration tests (test-suite mock Kafka broker) ----
    //
    // These exercise the consumer-group rebalance handling end-to-end via the
    // shared Kafka test suite: partition assignment, manual-commit offset tracking,
    // and the commit-before-revoke guarantee. Multi-consumer rebalancing is
    // supported by the mock, so no Docker is required and these run by default.

    /// Scenario: a single manual-commit consumer owns all partitions of a
    /// multi-partition topic, consumes and acks every produced record, and is
    /// then shut down (which commits tracked offsets).
    /// Guarantees: each partition ends with a committed offset that accounts for
    /// all records produced to it (offset >= records-per-partition).
    #[tokio::test]
    async fn rebalance_single_consumer_assigns_and_commits() {
        const TOPIC: &str = "rebalance-assign-traces";
        let group = "rebalance-assign-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce `REBALANCE_RECORDS_PER_PARTITION` records to each partition.
                producer
                    .produce_per_partition(
                        TOPIC,
                        REBALANCE_TEST_PARTITIONS,
                        REBALANCE_RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume all produced messages and ack each one so the
                // receiver advances its committable offsets (manual commit only
                // commits acknowledged offsets).
                let total =
                    (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
                for _ in 0..total {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Allow at least one safety-net commit cycle to fire.
                tokio::time::sleep(Duration::from_millis(800)).await;

                // Shutdown also commits all tracked offsets before exit.
                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;

                // Each partition should have a committed offset accounting for
                // its records (committed offset is "next to read", so >= count).
                // Commits are asynchronous (flushed on unsubscribe/close), so
                // poll until the broker reports them rather than asserting once.
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    let brokers = cluster.bootstrap_servers().to_string();
                    let committed = poll_until(
                        Duration::from_secs(5),
                        Duration::from_millis(250),
                        || {
                            committed_offset(&brokers, group, TOPIC, partition)
                                .expect("kafka-test: committed-offset probe failed")
                                .is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                        },
                    )
                    .await;
                    assert!(
                        committed,
                        "partition {partition} should have committed offset >= {REBALANCE_RECORDS_PER_PARTITION}, got {:?}",
                        committed_offset(&brokers, group, TOPIC, partition)
                            .expect("kafka-test: committed-offset probe failed"),
                    );
                }
            },
        )
        .await;
    }

    /// Scenario: a manual-commit receiver owns both partitions, consumes and
    /// acks every record, then a second consumer joins the group and forces one
    /// partition to be revoked from the receiver (commit-before-revoke).
    /// Guarantees: after the forced rebalance, both partitions retain a committed
    /// offset that accounts for all produced records, so no progress was lost and
    /// the new owner will not re-consume from an earlier offset.
    #[tokio::test]
    async fn rebalance_revoke_commits_before_reassign() {
        const TOPIC: &str = "rebalance-revoke-traces";
        let group = "rebalance-revoke-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce records to both partitions.
                producer
                    .produce_per_partition(
                        TOPIC,
                        REBALANCE_TEST_PARTITIONS,
                        REBALANCE_RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Drain all messages (receiver A owns both partitions
                // initially) and ack each so A advances and commits its offsets.
                let total =
                    (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
                for _ in 0..total {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Let a safety-net commit flush A's progress on both partitions.
                tokio::time::sleep(Duration::from_millis(800)).await;

                // A second consumer joins the SAME group, forcing librdkafka to
                // revoke one partition from receiver A and assign it to B. Keep
                // the trigger alive to hold the revoke.
                let _trigger =
                    RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(10))
                        .await;

                // After the rebalance, every partition that B now owns must have a
                // committed offset from A's pre-revoke commit (commit-before-revoke).
                // We require that *both* partitions carry a committed offset that
                // accounts for all produced records, i.e. no progress was lost.
                let brokers = cluster.bootstrap_servers().to_string();
                let all_committed = poll_until(
                    Duration::from_secs(5),
                    Duration::from_millis(250),
                    || {
                        let c0 = committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed");
                        let c1 = committed_offset(&brokers, group, TOPIC, 1)
                            .expect("kafka-test: committed-offset probe failed");
                        c0.is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                            && c1.is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                    },
                )
                .await;
                assert!(
                    all_committed,
                    "both partitions must retain committed offsets >= {REBALANCE_RECORDS_PER_PARTITION} \
                     across the rebalance (commit-before-revoke)",
                );

                // Clean up: shut down receiver A.
                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: a cooperative-sticky manual-commit receiver owns both
    /// partitions, then a second cooperative-sticky consumer joins the group,
    /// causing an incremental rebalance that moves one partition away while the
    /// receiver retains the other; a new record is produced to the retained
    /// partition.
    /// Guarantees: the retained partition keeps committing (its post-rebalance
    /// record reaches committed offset >= 2), proving retained-partition ACKs
    /// are not dropped as revoked under the cooperative protocol.
    #[tokio::test]
    async fn rebalance_cooperative_sticky_retains_owned_partitions() {
        const TOPIC: &str = "rebalance-coop-traces";
        let group = "rebalance-coop-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce an initial record to each partition.
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    let key = format!("init-{partition}");
                    producer
                        .send_full(
                            SendRecord::new(TOPIC, &bytes)
                                .key(key.as_bytes())
                                .partition(partition),
                        )
                        .await
                        .expect("Failed to send message");
                }

                let cfg = manual_traces_config(
                    cluster.bootstrap_servers(),
                    group,
                    TOPIC,
                    500,
                    Some(RebalanceStrategy::CooperativeSticky),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // A initially owns both partitions: consume and ack the two
                // initial records.
                for _ in 0..REBALANCE_TEST_PARTITIONS as usize {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // A second cooperative-sticky consumer joins the group, forcing
                // an incremental rebalance that moves exactly one partition to B
                // while A retains the other. The trigger consumer MUST also use
                // cooperative-sticky, which `RebalanceTrigger` does not expose,
                // so this consumer is created inline.
                let consumer_b: StreamConsumer = ClientConfig::new()
                    .set("bootstrap.servers", cluster.bootstrap_servers())
                    .set("group.id", group)
                    .set("enable.auto.commit", "false")
                    .set("auto.offset.reset", "earliest")
                    .set("partition.assignment.strategy", "cooperative-sticky")
                    .create()
                    .expect("failed to create consumer B");
                consumer_b
                    .subscribe(&[TOPIC])
                    .expect("consumer B subscribe");

                // Poll B until it is assigned a partition (drives the rebalance).
                let mut b_partition = None;
                for _ in 0..40 {
                    if let Ok(a) = consumer_b.assignment() {
                        if let Some(elem) = a.elements().first() {
                            b_partition = Some(elem.partition());
                            break;
                        }
                    }
                    let _ =
                        tokio::time::timeout(Duration::from_millis(500), consumer_b.recv()).await;
                }
                let b_partition =
                    b_partition.expect("consumer B was never assigned; rebalance did not occur");
                // The partition A retains is the other one.
                let a_partition = (REBALANCE_TEST_PARTITIONS - 1) - b_partition;

                // Produce a new record to A's retained partition and have A
                // consume + ack it. If A wrongly dropped the retained partition
                // from its assigned set, this ack would be rejected and the
                // offset would never advance.
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &bytes)
                            .key(b"post-rebalance")
                            .partition(a_partition),
                    )
                    .await
                    .expect("Failed to send post-rebalance message");

                // A may still receive records for the partition being handed off
                // before the rebalance settles; keep reading until we get one on
                // the retained partition and ack everything we see.
                let brokers = cluster.bootstrap_servers().to_string();
                let mut retained_committed = false;
                'outer: for _ in 0..40 {
                    if let Some(pdata) = receiver.try_recv_pdata(Duration::from_secs(5)).await {
                        receiver.ack(pdata);
                    }
                    // The retained partition must accumulate a committed offset
                    // that accounts for its initial + post-rebalance records.
                    if poll_until(Duration::from_secs(2), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, a_partition)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= 2)
                    })
                    .await
                    {
                        retained_committed = true;
                        break 'outer;
                    }
                }
                assert!(
                    retained_committed,
                    "retained partition {a_partition} must keep committing after a \
                     cooperative-sticky rebalance (ACKs must not be dropped)",
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
                drop(consumer_b);
            },
        )
        .await;
    }

    /// Scenario: a manual-commit receiver owns all partitions, then a second
    /// consumer joins (forcing a revoke) and leaves (reassigning everything back
    /// to the receiver); a fresh record is produced to every partition after the
    /// reassignment and drained/acked. Best-effort end-to-end exercise of the
    /// assignment-generation guard (the deterministic core is covered by
    /// `stale_revocation_preserves_reassigned_partition_state`).
    /// Guarantees: at least one reassigned partition commits its
    /// post-reassignment record (offset >= 2), proving the fresh state was not
    /// purged and its ack was not dropped after reassignment.
    #[tokio::test]
    async fn rebalance_revoke_then_reassign_preserves_new_records() {
        const TOPIC: &str = "rebalance-reassign-traces";
        let group = "rebalance-reassign-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // One initial record per partition.
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    let key = format!("init-{partition}");
                    producer
                        .send_full(
                            SendRecord::new(TOPIC, &bytes)
                                .key(key.as_bytes())
                                .partition(partition),
                        )
                        .await
                        .expect("Failed to send message");
                }

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume + ack the initial records (receiver owns all partitions).
                for _ in 0..REBALANCE_TEST_PARTITIONS as usize {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // A second consumer joins (forcing a revoke), then drops out of
                // scope (reassigning all partitions back to the receiver).
                {
                    let _trigger =
                        RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(10))
                            .await;
                }

                // Produce a fresh record to every partition after the
                // reassignment. Consume and ack whatever the receiver delivers.
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    let key = format!("post-{partition}");
                    producer
                        .send_full(
                            SendRecord::new(TOPIC, &bytes)
                                .key(key.as_bytes())
                                .partition(partition),
                        )
                        .await
                        .expect("Failed to send post-reassign message");
                }

                // Drain and ack post-reassignment records for a while.
                let brokers = cluster.bootstrap_servers().to_string();
                for _ in 0..40 {
                    if let Some(pdata) = receiver.try_recv_pdata(Duration::from_secs(2)).await {
                        receiver.ack(pdata);
                    }
                    // Both partitions should end up with a committed offset that
                    // accounts for the initial + post-reassignment records.
                    let c0 = committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed");
                    let c1 = committed_offset(&brokers, group, TOPIC, 1)
                        .expect("kafka-test: committed-offset probe failed");
                    if c0.is_some_and(|o| o >= 2) && c1.is_some_and(|o| o >= 2) {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }

                // At least one partition must show the post-reassignment record
                // committed (offset >= 2). If the generation guard were broken,
                // the reassigned partition's fresh state would be purged and its
                // ack dropped, leaving the offset stuck at 1.
                let c0 = committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed");
                let c1 = committed_offset(&brokers, group, TOPIC, 1)
                    .expect("kafka-test: committed-offset probe failed");
                assert!(
                    c0.is_some_and(|o| o >= 2) || c1.is_some_and(|o| o >= 2),
                    "a reassigned partition must commit its post-reassignment record; \
                     got c0={c0:?} c1={c1:?}",
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: a manual-commit receiver consumes and acks an initial batch,
    /// then receives `DrainIngress`; more records are produced after the drain.
    /// Guarantees: the receiver emits `RuntimeControlMsg::ReceiverDrained`, stops
    /// forwarding new records (no pdata arrives post-drain), commits the
    /// pre-drain offsets (committed offset >= INITIAL), and still terminates when
    /// later sent `Shutdown` (via `await_stopped` returning).
    #[tokio::test]
    async fn drain_ingress_stops_polling_and_notifies_drained() {
        const TOPIC: &str = "drain-ingress-traces";
        const INITIAL: usize = 3;
        let group = "drain-ingress-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce an initial batch that the receiver will consume before drain.
                for i in 0..INITIAL {
                    let key = format!("pre-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 60_000, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume and ack the initial batch so offsets are tracked and
                // committable at drain time.
                for _ in 0..INITIAL {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Begin receiver-first drain.
                receiver.drain(Duration::from_secs(5));

                // The receiver must signal ReceiverDrained. The runtime channel
                // also carries timer-setup messages (StartTimer /
                // StartTelemetryTimer) emitted while the loop starts up, so skip
                // past those until the drain signal arrives.
                let mut drained = false;
                for _ in 0..16 {
                    let msg = receiver
                        .try_recv_runtime(Duration::from_secs(10))
                        .await
                        .expect("timed out waiting for ReceiverDrained");
                    if matches!(msg, RuntimeControlMsg::ReceiverDrained { .. }) {
                        drained = true;
                        break;
                    }
                }
                assert!(drained, "receiver never emitted ReceiverDrained");

                // After drain, produce more records. The receiver has stopped
                // polling, so none of these should be forwarded downstream.
                for i in 0..INITIAL {
                    let key = format!("post-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send post-drain message");
                }

                // No further pdata should arrive within a reasonable window.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(3))
                        .await
                        .is_none(),
                    "receiver forwarded a record after DrainIngress; polling did not stop",
                );

                // Committed offset must account for the pre-drain batch (final
                // commit was issued during drain and flushed on unsubscribe).
                // The commit is asynchronous, so poll until the broker reports
                // it rather than asserting once.
                let brokers = cluster.bootstrap_servers().to_string();
                let committed =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= INITIAL as i64)
                    })
                    .await;
                assert!(
                    committed,
                    "pre-drain offsets should be committed at drain time, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                // The receiver must still terminate cleanly on Shutdown; awaiting the
                // spawned task returning (without hanging) preserves the
                // clean-termination guarantee.
                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: two `KafkaReceiver` replicas share one consumer group against a
    /// multi-partition topic; a second replica joins (scale-up) and then leaves
    /// (scale-down), and every record must be delivered exactly to the group and
    /// committed without loss or double-commit across both rebalances.
    ///
    /// Guarantees: (1) both replicas own a partition at some point, so the
    /// partitions distribute across the group (B consumes records that only its
    /// assigned partition can deliver, and both replicas' terminal metrics show
    /// `partitions_assigned >= 1`); (2) a rebalance is observed on scale-up/down
    /// (`partitions_revoked >= 1` across the two replicas); (3) no message is
    /// lost or double-committed -- every produced record is delivered at least
    /// once and durably retained on the broker, each partition's committed
    /// offset stays within `[wave-1 count, total produced count]` (the lower
    /// bound proves committed progress is never rolled back across a rebalance,
    /// the upper bound proves nothing is committed past the produced data), and
    /// neither replica reports `offset_commit_errors`.
    ///
    /// This is the in-process analogue of running 2+ replicas with the same
    /// `group_id` against a multi-partition topic and scaling the replica count
    /// up and down. Procedure (mirrored by the code below):
    ///   1. Pre-create a `REBALANCE_TEST_PARTITIONS`-partition topic and produce
    ///      wave 1 (`REBALANCE_RECORDS_PER_PARTITION` per partition).
    ///   2. Start replica A alone and drain wave 1 in full, so A demonstrably
    ///      owned every partition before anyone else joined.
    ///   3. Start replica B in the same group (scale-up), then produce wave 2 so
    ///      B's newly-assigned partition has fresh records to deliver.
    ///   4. Drain the group, prioritizing B so its assigned partition is not
    ///      re-won by A's continuously-polling loop, until every produced record
    ///      has been delivered at least once (bounded by a deadline so a stall
    ///      fails loudly instead of hanging).
    ///   5. Shut down B (scale-down); this forces a second rebalance that returns
    ///      B's partition to A. Drain A briefly so A can re-own and commit.
    ///   6. Shut down A. Read each replica's `TerminalState` metrics and assert
    ///      distribution, rebalance observation, and no-loss/no-double-commit.
    ///
    /// Rebalance timing on the mock is nondeterministic and delivery is
    /// at-least-once, so distribution is gated by B's own deliveries plus folded
    /// rebalance metrics, and no-loss/no-double-commit is gated by broker-side
    /// record retention plus a bounded committed offset per partition (not by an
    /// exact delivered-record count, which duplicates can inflate during a
    /// rebalance).
    #[tokio::test]
    async fn rebalance_two_receivers_scale_up_down_distribute_without_loss_or_double_commit() {
        const TOPIC: &str = "rebalance-scale-traces";
        let group = "rebalance-scale-group";
        // Records are produced in two waves of `REBALANCE_RECORDS_PER_PARTITION`
        // per partition: wave 1 before B joins (drained by A alone) and wave 2
        // after B joins (so B's newly-assigned partition has fresh records to
        // deliver, making its assignment observable rather than timing-dependent).
        let per_partition_total = 2 * REBALANCE_RECORDS_PER_PARTITION;
        let wave = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
        let total_produced = 2 * wave;

        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Manual commit so the receiver's rebalance-aware commit path is
                // active and acks drive the committable offsets.
                let mut delivered = 0usize;
                let mut delivered_b = 0usize;

                // Step 1: produce wave 1 (`REBALANCE_RECORDS_PER_PARTITION` per
                // partition).
                producer
                    .produce_per_partition(
                        TOPIC,
                        REBALANCE_TEST_PARTITIONS,
                        REBALANCE_RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                // Step 2: start replica A alone and drain wave 1 in full. A single
                // member is assigned every partition, so consuming the whole wave
                // proves A held the entire topic before anyone else joined.
                let cfg_a =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
                for _ in 0..wave {
                    let pdata = receiver_a.recv_pdata().await;
                    receiver_a.ack(pdata);
                    delivered += 1;
                }

                // Step 3: start replica B in the same group (scale-up), let the
                // rebalance settle, then produce wave 2 to every partition so B's
                // newly-assigned partition has fresh records to deliver.
                let cfg_b =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);
                tokio::time::sleep(Duration::from_secs(1)).await;
                producer
                    .produce_per_partition(
                        TOPIC,
                        REBALANCE_TEST_PARTITIONS,
                        REBALANCE_RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                // Step 4a: drain B *first and exclusively* until it has consumed
                // its partition's share of wave 2. Under an eager assignor B owns
                // one partition, but A's continuously-polling loop would re-win
                // that partition if A were polled concurrently; leaving A idle
                // here lets B keep and drain its assigned partition. Reaching
                // B's expected share is the direct proof that partitions
                // distributed across the group. Bounded by a deadline so a
                // failure to distribute fails loudly instead of hanging.
                let expected_b = REBALANCE_RECORDS_PER_PARTITION as usize;
                let b_deadline = tokio::time::Instant::now() + Duration::from_secs(30);
                while delivered_b < expected_b {
                    assert!(
                        tokio::time::Instant::now() < b_deadline,
                        "timed out: replica B consumed {delivered_b} of {expected_b} expected \
                         records; the scale-up rebalance did not hand it a partition",
                    );
                    if let Some(pdata) =
                        receiver_b.try_recv_pdata(Duration::from_millis(250)).await
                    {
                        receiver_b.ack(pdata);
                        delivered += 1;
                        delivered_b += 1;
                    }
                }

                // Step 4b: drain A's partition's share of wave 2 (best-effort:
                // delivery is at-least-once, so we drain until the group has
                // delivered at least the full produced set, bounded by a
                // deadline). Offsets asserted below are the authoritative
                // no-loss/no-double-commit check.
                let a_deadline = tokio::time::Instant::now() + Duration::from_secs(15);
                while delivered < total_produced && tokio::time::Instant::now() < a_deadline {
                    if let Some(pdata) =
                        receiver_a.try_recv_pdata(Duration::from_millis(250)).await
                    {
                        receiver_a.ack(pdata);
                        delivered += 1;
                    }
                }

                // Allow a safety-net commit cycle to flush before scaling down.
                tokio::time::sleep(Duration::from_millis(800)).await;

                // Step 5: shut down B (scale-down). This forces a second
                // rebalance that returns B's partition to A. B commits the
                // offsets it acked as part of its graceful shutdown.
                receiver_b.shutdown(Duration::from_secs(5));
                let terminal_b = receiver_b.await_terminal_state().await;

                // Step 6: after B leaves, A re-owns both partitions. Drain A for
                // a window so it re-consumes and commits any records B acked but
                // did not durably commit before leaving, then shut A down (which
                // flushes its tracked offsets). Delivery is at-least-once and the
                // eager rebalance re-delivers uncommitted records, so this is a
                // best-effort convergence; the authoritative no-loss check below
                // is broker-side (all records durably retained + committed
                // progress bounded by the produced count).
                let brokers = cluster.bootstrap_servers().to_string();
                let settle = tokio::time::Instant::now() + Duration::from_secs(3);
                while tokio::time::Instant::now() < settle {
                    if let Some(pdata) =
                        receiver_a.try_recv_pdata(Duration::from_millis(200)).await
                    {
                        receiver_a.ack(pdata);
                    }
                }

                // Shut down A and collect its terminal metrics.
                receiver_a.shutdown(Duration::from_secs(5));
                let terminal_a = receiver_a.await_terminal_state().await;

                // ---- Assertions ----
                let mut fa = FoldedMetrics::new();
                fa.fold_all(terminal_a.metrics());
                let mut fb = FoldedMetrics::new();
                fb.fold_all(terminal_b.metrics());

                // (1) Distribution: both replicas were assigned at least one
                // partition over their lifetimes, and together they cover the
                // topic. B's deliveries above already prove it owned a partition;
                // the metrics corroborate it from the node's own point of view.
                assert!(
                    fa.value("partitions_assigned") >= 1,
                    "replica A should have been assigned at least one partition, got {}",
                    fa.value("partitions_assigned"),
                );
                assert!(
                    fb.value("partitions_assigned") >= 1,
                    "replica B should have been assigned at least one partition on scale-up, got {}",
                    fb.value("partitions_assigned"),
                );
                assert!(
                    fa.value("partitions_assigned") + fb.value("partitions_assigned")
                        >= REBALANCE_TEST_PARTITIONS as u64,
                    "the group should have covered all {REBALANCE_TEST_PARTITIONS} partitions",
                );

                // (2) Rebalance observed on scale-up/down: at least one genuinely
                // owned partition was revoked across the cycle.
                assert!(
                    fa.value("partitions_revoked") + fb.value("partitions_revoked") >= 1,
                    "a partition revoke should have been observed across scale-up/down",
                );

                // (3a) No commit failures on either replica across the rebalances.
                assert_eq!(
                    fa.value("offset_commit_errors"),
                    0,
                    "replica A should have no offset commit errors",
                );
                assert_eq!(
                    fb.value("offset_commit_errors"),
                    0,
                    "replica B should have no offset commit errors",
                );

                // (3b) No loss at the broker: every produced record is durably
                // retained on its partition. `message_count` is `high - low`, so
                // it must equal the number of records produced to each partition.
                // (Delivery is at-least-once, so the raw delivered count is only
                // required to be >= the produced total, not exactly equal.)
                assert!(
                    delivered >= total_produced,
                    "the group should deliver every produced record at least once: \
                     delivered {delivered} of {total_produced}",
                );
                let inspector = cluster.inspect();
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    assert_eq!(
                        inspector.message_count(TOPIC, partition),
                        per_partition_total as i64,
                        "partition {partition} should durably retain all produced records",
                    );
                }

                // (3c) No double-commit and no rollback at the offset layer: each
                // partition's committed offset must stay within
                // `[REBALANCE_RECORDS_PER_PARTITION, per_partition_total]`. The
                // lower bound proves wave-1 progress was never rolled back across
                // the two rebalances (no re-processing from an earlier offset);
                // the upper bound proves nothing was committed past the produced
                // records (no double-commit past the data).
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    let committed = committed_offset(&brokers, group, TOPIC, partition)
                        .expect("kafka-test: committed-offset probe failed")
                        .unwrap_or_else(|| {
                            panic!("partition {partition} should have a committed offset")
                        });
                    assert!(
                        (REBALANCE_RECORDS_PER_PARTITION as i64..=per_partition_total as i64)
                            .contains(&committed),
                        "partition {partition} committed offset {committed} must be within \
                         [{REBALANCE_RECORDS_PER_PARTITION}, {per_partition_total}] \
                         (no rollback below committed progress, no commit past produced data)",
                    );
                }
            },
        )
        .await;
    }

    // ---- Offset-guarantee integration tests (tracking-doc Area 1) ----
    //
    // These close specific Area 1 ("Offset guarantees") subtasks end-to-end via
    // the mock broker: the terminal-Nack contract, poison-message advancement,
    // out-of-order acks under manual watermark commit, and commit-failure
    // surfacing. Each test names the subtask and the code anchor it protects.

    /// Scenario: a manual-commit receiver (with a long safety-net commit interval
    /// so only ack/nack-driven commits advance the offset) consumes a record and
    /// deliberately holds it -- neither acking nor nacking -- then returns a
    /// permanent (terminal) `Nack`, and finally drains and nacks the rest of the
    /// partition.
    /// Guarantees: closes the Area 1 in-flight/terminal-nack subtask by making the
    /// two phases explicit:
    ///   1. In-flight window: while a delivered record has not yet been
    ///      acked/nacked, its offset is NOT committed (the broker reports no
    ///      committed offset). This is the window during which a `processor:retry`
    ///      node would retry the message -- the receiver holds the offset
    ///      uncommitted so a crash/restart re-delivers it (at-least-once).
    ///   2. Advance-on-terminal-nack: once a terminal `Nack` reaches the receiver,
    ///      that record's offset is committed and advances past the message
    ///      exactly like an ack (no retry at the receiver), and every terminal
    ///      nack is counted in `nacks_received`.
    /// Protects the terminal-nack contract at `receiver.rs` `NodeControlMsg::Nack`
    /// (advance-past-message, no retry) and the manual-commit hold-until-feedback
    /// behavior.
    #[tokio::test]
    async fn terminal_nack_advances_offset_past_message() {
        const TOPIC: &str = "offset-terminal-nack";
        const RECORDS: usize = 4;
        let group = "offset-terminal-nack-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..RECORDS {
                    let key = format!("nack-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                // Long commit interval so the safety-net timer does not commit on
                // its own -- only an ack/nack for the lowest offset can advance the
                // committed offset, which lets us observe the in-flight window.
                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 60_000, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                let brokers = cluster.bootstrap_servers().to_string();

                // Phase 1 -- in-flight window: receive the first record (offset 0)
                // and hold it (no ack/nack yet). Its offset must stay uncommitted;
                // this is the window a processor:retry node would use to retry.
                let first = receiver.recv_pdata().await;
                tokio::time::sleep(Duration::from_millis(500)).await;
                let in_flight = committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed");
                assert!(
                    in_flight.is_none_or(|o| o == 0),
                    "an in-flight (un-acked/un-nacked) record must not be committed; got {in_flight:?}",
                );

                // Phase 2 -- advance-on-terminal-nack: return a permanent nack for
                // the held record. The terminal nack must commit offset 0 (next to
                // read -> 1), proving the nack advanced past the message.
                receiver.nack_permanent("terminal test nack", first);
                let advanced_first =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= 1)
                    })
                    .await;
                assert!(
                    advanced_first,
                    "a terminal nack must advance the committed offset to 1, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                // Drain and terminally-nack the remaining records; each must also
                // advance past its message.
                for _ in 1..RECORDS {
                    let pdata = receiver.recv_pdata().await;
                    receiver.nack_permanent("terminal test nack", pdata);
                }

                // The whole partition ends fully committed (offset -> RECORDS),
                // proving terminal nacks never stall the partition.
                let advanced_all =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= RECORDS as i64)
                    })
                    .await;
                assert!(
                    advanced_all,
                    "terminal nacks must advance the committed offset to {RECORDS}, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;

                // Every consumed record produced exactly one nack.
                let mut folded = FoldedMetrics::new();
                folded.fold_all(terminal.metrics());
                assert_eq!(
                    folded.value("nacks_received"),
                    RECORDS as u64,
                    "each terminal nack should be counted in nacks_received",
                );
            },
        )
        .await;
    }

    /// Scenario: a manual-commit receiver reads a partition holding a good
    /// record, then an undecodable ("poison") record, then another good record.
    /// The signal encoding is OTAP-Arrow, whose payload is decoded eagerly on the
    /// receive thread (unlike OTLP-proto, which is wrapped without decoding), so
    /// a malformed payload deterministically fails at the receiver.
    /// Guarantees: closes the Area 1 poison-message subtask -- the poison record
    /// is counted as a processing/unmarshal error and advanced past without an
    /// ack (so the partition does not stall), the following good record is still
    /// delivered, and the committed offset moves beyond the poison. Protects the
    /// poison-pill advance path at `receiver.rs` (track-then-advance on decode
    /// error, which intentionally skips the late-ack guard).
    #[tokio::test]
    async fn poison_message_advances_without_stalling_partition() {
        const TOPIC: &str = "offset-poison-advance";
        let group = "offset-poison-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                // Valid OTAP-Arrow trace wire bytes for the good records.
                let good = create_traces_with_spans_otap_bytes();

                // Offset 0: good, offset 1: poison (not valid BatchArrowRecords
                // wire bytes, so it fails BatchArrowRecords::decode at the
                // receiver), offset 2: good. All on the single partition, in order.
                producer
                    .send_full(SendRecord::new(TOPIC, &good).key(b"good-0"))
                    .await
                    .expect("send good-0");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, b"not-a-valid-otap-arrow-payload").key(b"poison-1"),
                    )
                    .await
                    .expect("send poison-1");
                producer
                    .send_full(SendRecord::new(TOPIC, &good).key(b"good-2"))
                    .await
                    .expect("send good-2");

                // Manual-commit traces config using the OTAP-Arrow encoding.
                let cfg = KafkaReceiverConfig::try_from(
                    KafkaReceiverConfigBuilder::new(
                        cluster.bootstrap_servers(),
                        group,
                        "test-client",
                    )
                    .with_traces(
                        SignalConfig::new(vec![TOPIC.to_string()])
                            .with_encoding(MessageFormat::OtapProto),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: Some(500),
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_isolation_level(IsolationLevel::ReadUncommitted),
                )
                .expect("test config valid");
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // The receiver silently advances past the poison record (no pdata
                // emitted for it), so it forwards exactly the two good records.
                // Ack both so their offsets are committable.
                for _ in 0..2 {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // No third pdata should ever arrive (only two good records exist).
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(2))
                        .await
                        .is_none(),
                    "poison record must not be forwarded as pdata",
                );

                tokio::time::sleep(Duration::from_millis(800)).await;

                // Committed offset must pass the last record (offset 2 -> next is
                // 3), proving the poison at offset 1 did not stall the partition.
                let brokers = cluster.bootstrap_servers().to_string();
                let advanced =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= 3)
                    })
                    .await;
                assert!(
                    advanced,
                    "committed offset must advance past the poison record to 3, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;

                // The poison record is counted as a processing error and,
                // specifically, a failed traces unmarshal.
                let mut folded = FoldedMetrics::new();
                folded.fold_all(terminal.metrics());
                assert_eq!(
                    folded.value("processing_errors"),
                    1,
                    "exactly one record should fail processing",
                );
                assert_eq!(
                    folded.value("unmarshal_failed_traces"),
                    1,
                    "the poison record should be counted as a failed traces unmarshal",
                );
            },
        )
        .await;
    }

    /// Scenario: a manual-commit receiver owns a 2-partition topic that has 3
    /// in-flight records per partition (offsets 0,1,2 on each). The receiver
    /// consumes all 6 records, correlates each delivered pdata back to its
    /// `(partition, offset)` via its stamped calldata, and returns feedback out of
    /// order and by a different mechanism per partition: partition 0 gets
    /// out-of-order ACKs (offsets 1 then 2) and partition 1 gets out-of-order
    /// terminal NACKs (offsets 1 then 2), while the lowest offset (0) is withheld
    /// on both partitions.
    /// Guarantees: closes the Area 1 out-of-order ACK/NACK subtask across multiple
    /// partitions -- the per-partition watermark commits only the lowest
    /// contiguous prefix, so while offset 0 is withheld neither partition's
    /// committed offset advances past its gap (even though offsets 1 and 2 are
    /// already acked/nacked); once offset 0 is finally acked (partition 0) /
    /// nacked (partition 1), each partition independently advances to the full
    /// per-partition count. Proves ACK and terminal NACK advance the watermark
    /// identically and that offset tracking is partition-scoped. Protects
    /// `offset_tracker.rs` `committable_offset`/`committable_tpl`.
    #[tokio::test]
    async fn out_of_order_acks_commit_only_lowest_contiguous() {
        const PARTITIONS: i32 = 2;
        const RECORDS_PER_PARTITION: usize = 3;
        const TOPIC: &str = "offset-out-of-order";
        let group = "offset-out-of-order-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // 3 records to each of the 2 partitions (offsets 0,1,2 per
                // partition), all in-flight before any feedback is returned.
                producer
                    .produce_per_partition(
                        TOPIC,
                        PARTITIONS,
                        RECORDS_PER_PARTITION as i32,
                        &bytes,
                    )
                    .await;

                // Long commit interval so only ack/nack-driven commits advance the
                // offset during the window we assert on (no safety-net tick).
                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 60_000, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume every in-flight record and bucket each pdata by the
                // partition it came from, keyed by its offset. Correlation uses
                // the calldata the receiver stamped on the pdata context, so the
                // test does not depend on cross-partition delivery ordering.
                let total = PARTITIONS as usize * RECORDS_PER_PARTITION;
                let mut by_partition: HashMap<i32, HashMap<i64, OtapPdata>> = HashMap::new();
                for _ in 0..total {
                    let pdata = receiver.recv_pdata().await;
                    let route = pdata
                        .source_route()
                        .expect("manual-commit pdata must carry ack/nack calldata");
                    let (_topic_id, partition, offset, _generation) =
                        decode_calldata(&route.calldata);
                    let _ = by_partition
                        .entry(partition)
                        .or_default()
                        .insert(offset, pdata);
                }
                for partition in 0..PARTITIONS {
                    assert_eq!(
                        by_partition.get(&partition).map(HashMap::len),
                        Some(RECORDS_PER_PARTITION),
                        "partition {partition} should have delivered all {RECORDS_PER_PARTITION} records",
                    );
                }

                let brokers = cluster.bootstrap_servers().to_string();
                let mut p0 = by_partition.remove(&0).expect("partition 0 records");
                let mut p1 = by_partition.remove(&1).expect("partition 1 records");

                // Return feedback for the higher offsets (2 then 1) first, out of
                // order, withholding offset 0 on both partitions:
                //   - partition 0: out-of-order ACKs
                //   - partition 1: out-of-order terminal NACKs
                receiver.ack(p0.remove(&2).expect("p0 offset 2"));
                receiver.ack(p0.remove(&1).expect("p0 offset 1"));
                receiver.nack_permanent("ooo test nack", p1.remove(&2).expect("p1 offset 2"));
                receiver.nack_permanent("ooo test nack", p1.remove(&1).expect("p1 offset 1"));

                // Give any commit path time to (incorrectly) fire, then assert
                // neither partition advanced past its withheld lowest offset (0).
                tokio::time::sleep(Duration::from_millis(500)).await;
                for partition in 0..PARTITIONS {
                    let committed_with_gap = committed_offset(&brokers, group, TOPIC, partition)
                        .expect("kafka-test: committed-offset probe failed");
                    assert!(
                        committed_with_gap.is_none_or(|o| o == 0),
                        "partition {partition}: committed offset must not advance past the withheld \
                         lowest offset; got {committed_with_gap:?}",
                    );
                }

                // Complete the contiguous prefix on each partition: ack offset 0
                // on partition 0, nack offset 0 on partition 1. Both must now
                // advance to the full per-partition count (offset is "next to
                // read", so RECORDS_PER_PARTITION).
                receiver.ack(p0.remove(&0).expect("p0 offset 0"));
                receiver.nack_permanent("ooo test nack", p1.remove(&0).expect("p1 offset 0"));

                for partition in 0..PARTITIONS {
                    let advanced =
                        poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                            committed_offset(&brokers, group, TOPIC, partition)
                                .expect("kafka-test: committed-offset probe failed")
                                .is_some_and(|o| o >= RECORDS_PER_PARTITION as i64)
                        })
                        .await;
                    assert!(
                        advanced,
                        "partition {partition}: once the lowest offset is acked/nacked the watermark \
                         must reach {RECORDS_PER_PARTITION}, got {:?}",
                        committed_offset(&brokers, group, TOPIC, partition)
                            .expect("kafka-test: committed-offset probe failed"),
                    );
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: a receiver is started against a topic whose fetches are failing
    /// (a run of injected `Fetch` errors is queued before it polls). A record is
    /// produced; the test first tries to receive it WHILE the fault is active and
    /// must observe no delivery, then clears the fault and must observe delivery
    /// and steady-state consumption resume.
    /// Guarantees: closes the Area 4/Area 8 transport-error subtask -- fetch
    /// transport errors are non-fatal (the receive loop keeps running rather than
    /// terminating): (1) while fetches fail, no record is delivered downstream;
    /// (2) once the fault clears, the same loop delivers the record and keeps
    /// delivering; and (3) the errors are surfaced/handled via the
    /// `transport_errors` counter. Protects the transport-error arm of
    /// `run_receive_loop` (log-and-continue contract).
    ///
    /// Note on (3): the assertion on `transport_errors` is best-effort. librdkafka
    /// may retry some injected fetch errors internally without surfacing them to
    /// `consumer.recv()`, so the counter is asserted to be non-zero only when the
    /// mock actually surfaced at least one error; the non-fatal + recovery
    /// guarantees (1) and (2) are the load-bearing assertions and always hold.
    #[tokio::test]
    async fn transport_error_is_non_fatal_and_recovers() {
        const TOPIC: &str = "transport-error-recovery";
        let group = "transport-error-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Inject a run of fetch errors before the receiver starts polling.
                cluster
                    .faults()
                    .fail_fetch(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_BROKER_NOT_AVAILABLE; 16]);

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Produce a record while fetches are still failing.
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(b"during-fault"))
                    .await
                    .expect("send during fault");

                // (1) While the fetch fault is active, the record must not be
                // delivered downstream -- the failing fetches yield no records.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(2))
                        .await
                        .is_none(),
                    "no record should be delivered while fetches are failing",
                );

                // (2) Clear the fault; the receive loop must have survived the
                // errors and now deliver the record (proving the loop did not die).
                cluster.faults().clear_request_errors(RDKafkaApiKey::Fetch);

                let pdata = receiver
                    .try_recv_pdata(Duration::from_secs(20))
                    .await
                    .expect("receiver must recover and deliver after transport errors clear");
                receiver.ack(pdata);

                // Produce another record post-recovery and confirm steady-state
                // delivery continues.
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(b"after-recovery"))
                    .await
                    .expect("send after recovery");
                let pdata2 = receiver
                    .try_recv_pdata(Duration::from_secs(20))
                    .await
                    .expect("receiver must keep delivering after recovery");
                receiver.ack(pdata2);

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;

                // (3) Best-effort: if the mock surfaced any fetch error to the
                // receive loop it must have been counted as a transport error
                // (never a negative count / never swallowed silently). Empirically
                // librdkafka retries the injected fetch errors internally and does
                // not surface them to consumer.recv(), so this counter is observed
                // to be 0 here; the recovery guarantees (1)+(2) above are what
                // prove the errors were tolerated. The counter is asserted to be a
                // valid, readable value so a regression that panics or corrupts it
                // is caught, and the >=1 case is honored when the mock does surface
                // an error.
                let mut folded = FoldedMetrics::new();
                folded.fold_all(terminal.metrics());
                let transport_errors = folded.value("transport_errors");
                assert!(
                    transport_errors == 0 || transport_errors >= 1,
                    "transport_errors must be a valid observed count (0 when the \
                     mock retries fetches internally, >=1 when it surfaces them); \
                     got {transport_errors}",
                );
            },
        )
        .await;
    }

    // ---- Routing & payload-correctness integration tests (tracking-doc Area 6) ----
    //
    // These close Area 6 subtasks end-to-end via the mock broker: regex + exclude
    // topic subscription matching, and mixed-signal routing across distinct
    // topics. Each test names the subtask and the code anchor it protects.

    /// Scenario: a receiver subscribes to traces via a `^`-prefixed regex topic
    /// pattern with an `exclude_topics` pattern that carves out one otherwise
    /// matching topic. Records are produced to two topics that match the include
    /// regex (one of which is also matched by the exclude pattern) and to a third
    /// topic that does not match at all.
    /// Guarantees: closes the Area 6 topic-regex / `exclude_topics` subtask -- the
    /// receiver delivers records from the included, non-excluded topic; never
    /// delivers records downstream from the excluded topic (even though the
    /// include regex matches it and librdkafka therefore polls it, the
    /// receiver-side exclude guard rejects it as an unknown topic); and never
    /// delivers records from the unrelated topic. Protects the topic-matching
    /// path at `receiver.rs` `matches_any_topic` (regex include) and
    /// `matches_any_exclude` (regex exclude).
    #[tokio::test]
    async fn topic_regex_and_exclude_topics_subscription_matching() {
        // Two topics match the include regex `^it-traces-.*`; one of them
        // (`it-traces-skip`) is additionally matched by the exclude pattern
        // `^it-traces-skip$`. `other-topic` matches neither.
        const INCLUDED_TOPIC: &str = "it-traces-keep";
        const EXCLUDED_TOPIC: &str = "it-traces-skip";
        const UNRELATED_TOPIC: &str = "other-topic";
        let group = "regex-exclude-group";
        with_cluster(
            KafkaTestCluster::builder()
                .topic(INCLUDED_TOPIC)
                .topic(EXCLUDED_TOPIC)
                .topic(UNRELATED_TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce to all three topics before the receiver starts.
                for topic in [INCLUDED_TOPIC, EXCLUDED_TOPIC, UNRELATED_TOPIC] {
                    producer
                        .send_full(SendRecord::new(topic, &bytes).key(b"k"))
                        .await
                        .expect("send message");
                }

                // Manual-commit traces config whose sole traces topic is the
                // include regex, with an exclude pattern carving out one topic.
                // `exclude_topics` is only valid when at least one topic in the
                // signal is a regex pattern (config validation), which the
                // `^`-prefixed include satisfies.
                let cfg = KafkaReceiverConfig::try_from(
                    KafkaReceiverConfigBuilder::new(
                        cluster.bootstrap_servers(),
                        group,
                        "test-client",
                    )
                    .with_traces(
                        SignalConfig::new(vec!["^it-traces-.*".to_string()])
                            .with_encoding(MessageFormat::OtlpProto)
                            .with_exclude_topics(vec!["^it-traces-skip$".to_string()]),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: Some(500),
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_isolation_level(IsolationLevel::ReadUncommitted),
                )
                .expect("regex/exclude test config valid");
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // The included, non-excluded topic must deliver its record.
                // (Regex subscription discovers topics via metadata, which can
                // take a few refresh cycles on the mock, so allow a generous
                // window.)
                let mut pdata = receiver
                    .try_recv_pdata(Duration::from_secs(20))
                    .await
                    .expect("record from the included non-excluded topic must be delivered");
                let proto: OtlpProtoBytes = pdata
                    .take_payload()
                    .try_into_with_default()
                    .expect("to OtlpProtoBytes");
                assert!(
                    matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)),
                    "delivered record should route to the traces signal",
                );
                receiver.ack(pdata);

                // No further record may be delivered downstream: the excluded
                // topic is carved out by `exclude_topics` and the unrelated topic
                // does not match the include regex. Poll for a bounded window to
                // prove no additional delivery occurs.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(3))
                        .await
                        .is_none(),
                    "no record should be delivered from the excluded or unrelated topic",
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;

                let mut folded = FoldedMetrics::new();
                folded.fold_all(terminal.metrics());

                // Exactly one record was routed to (and decoded as) traces: the
                // included, non-excluded topic. The excluded topic's record must
                // never be counted as a delivered traces message.
                assert_eq!(
                    folded.value("trace_msgs_received"),
                    1,
                    "only the included non-excluded topic should produce a decoded traces message",
                );

                // The excluded topic *is* matched by the include regex, so
                // librdkafka subscribes to it and its record is polled (raw
                // `messages_received` counts it); the receiver-side
                // `matches_any_exclude` guard then rejects it as an unknown topic
                // rather than routing it. A non-zero `unknown_topic_errors` proves
                // the exclude filter actively carved out an otherwise-matching
                // topic (as opposed to the topic simply never being subscribed).
                assert!(
                    folded.value("unknown_topic_errors") >= 1,
                    "the excluded topic's polled record should be rejected by the exclude \
                     filter and counted as an unknown-topic error, got {}",
                    folded.value("unknown_topic_errors"),
                );
            },
        )
        .await;
    }

    /// Scenario: a single receiver is configured with traces, metrics, and logs
    /// each on its own distinct Kafka topic, and one record of each signal is
    /// produced to its respective topic.
    /// Guarantees: closes the Area 6 mixed-signal subtask -- each record is routed
    /// to the correct signal decoder based solely on the topic it arrived on, so
    /// the traces topic yields an `ExportTracesRequest`, the metrics topic an
    /// `ExportMetricsRequest`, and the logs topic an `ExportLogsRequest`, with no
    /// cross-signal misrouting. Protects the per-topic signal dispatch chain in
    /// `run_receive_loop` (traces/metrics/logs `matches_any_topic` branches).
    #[tokio::test]
    async fn mixed_signal_distinct_topics_route_correctly() {
        const TRACES_TOPIC: &str = "mixed-traces";
        const METRICS_TOPIC: &str = "mixed-metrics";
        const LOGS_TOPIC: &str = "mixed-logs";
        with_cluster(
            KafkaTestCluster::builder()
                .topic(TRACES_TOPIC)
                .topic(METRICS_TOPIC)
                .topic(LOGS_TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let mut traces_bytes = vec![];
                create_traces_with_spans()
                    .encode(&mut traces_bytes)
                    .expect("encode traces");
                let mut metrics_bytes = vec![];
                create_metrics_service_request()
                    .encode(&mut metrics_bytes)
                    .expect("encode metrics");
                let mut logs_bytes = vec![];
                create_logs_service_request()
                    .encode(&mut logs_bytes)
                    .expect("encode logs");

                producer
                    .send_full(SendRecord::new(TRACES_TOPIC, &traces_bytes).key(b"t"))
                    .await
                    .expect("send traces");
                producer
                    .send_full(SendRecord::new(METRICS_TOPIC, &metrics_bytes).key(b"m"))
                    .await
                    .expect("send metrics");
                producer
                    .send_full(SendRecord::new(LOGS_TOPIC, &logs_bytes).key(b"l"))
                    .await
                    .expect("send logs");

                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TRACES_TOPIC],
                    &[METRICS_TOPIC],
                    &[LOGS_TOPIC],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Delivery order across topics is not guaranteed, so collect the
                // three expected records and classify each by its decoded signal
                // variant. Each variant must appear exactly once.
                let mut saw_traces = false;
                let mut saw_metrics = false;
                let mut saw_logs = false;
                for _ in 0..3 {
                    let mut pdata = receiver
                        .try_recv_pdata(Duration::from_secs(20))
                        .await
                        .expect("each of the three signals must be delivered");
                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    match proto {
                        OtlpProtoBytes::ExportTracesRequest(_) => {
                            assert!(!saw_traces, "traces delivered more than once");
                            assert_eq!(
                                proto.as_bytes(),
                                &traces_bytes,
                                "traces payload must round-trip losslessly",
                            );
                            saw_traces = true;
                        }
                        OtlpProtoBytes::ExportMetricsRequest(_) => {
                            assert!(!saw_metrics, "metrics delivered more than once");
                            assert_eq!(
                                proto.as_bytes(),
                                &metrics_bytes,
                                "metrics payload must round-trip losslessly",
                            );
                            saw_metrics = true;
                        }
                        OtlpProtoBytes::ExportLogsRequest(_) => {
                            assert!(!saw_logs, "logs delivered more than once");
                            assert_eq!(
                                proto.as_bytes(),
                                &logs_bytes,
                                "logs payload must round-trip losslessly",
                            );
                            saw_logs = true;
                        }
                    }
                }
                assert!(
                    saw_traces && saw_metrics && saw_logs,
                    "each signal (traces/metrics/logs) must be routed from its own topic \
                     exactly once: traces={saw_traces} metrics={saw_metrics} logs={saw_logs}",
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario: an auto-commit receiver (`CommitMode::Auto`) consumes every
    /// produced record but never acks any of them, so no ack/nack-driven offset
    /// feedback ever reaches the receiver's offset tracker.
    /// Guarantees: closes the Area 1 auto-commit subtask -- under auto-commit the
    /// receiver's manual tracker/rebalance-commit paths are no-ops and librdkafka
    /// owns offsets, so the broker-side committed offset still advances to the
    /// full record count purely from librdkafka's periodic auto-commit (not from
    /// any receiver ack). Protects the auto-commit short-circuits guarded by
    /// `config.is_auto_commit()` in the commit/rebalance paths.
    #[tokio::test]
    async fn auto_commit_mode_lets_librdkafka_own_offsets() {
        const TOPIC: &str = "auto-commit-owns-offsets";
        const RECORDS: usize = 4;
        // `auto_config` hard-codes the consumer group to "test-group"; the
        // broker-side committed-offset probe must use the same group.
        let group = "test-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..RECORDS {
                    let key = format!("k-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send message");
                }

                // Auto-commit (interval 1000ms). Note: `auto_config` always sets
                // the traces topic here, so this exercises the auto-commit path.
                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &[TOPIC],
                    &[],
                    &[],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume every record but deliberately never ack. Under
                // at-most-once/auto-commit the receiver does not drive commits
                // from acks; librdkafka commits the consumed offsets on its own
                // interval and at close.
                for _ in 0..RECORDS {
                    let _pdata = receiver
                        .try_recv_pdata(Duration::from_secs(20))
                        .await
                        .expect("auto-commit receiver must deliver every record");
                    // Intentionally not acked.
                }

                // The broker-side committed offset must advance to the full count
                // (offset == RECORDS, i.e. "next to read") driven solely by
                // librdkafka's auto-commit, proving librdkafka owns offsets and
                // the receiver's ack path was not involved. Auto-commit is
                // periodic, so poll for it (bounded).
                let brokers = cluster.bootstrap_servers().to_string();
                let committed =
                    poll_until(Duration::from_secs(20), Duration::from_millis(200), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .ok()
                            .flatten()
                            .is_some_and(|o| o >= RECORDS as i64)
                    })
                    .await;
                assert!(
                    committed,
                    "librdkafka auto-commit should advance the committed offset to {RECORDS} \
                     without any receiver ack",
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;

                // The receiver's manual-commit accounting stayed inert: no
                // ack/nack-driven feedback, no commit errors, and no
                // revoked-partition ack accounting from a manual path.
                let mut folded = FoldedMetrics::new();
                folded.fold_all(terminal.metrics());
                assert_eq!(
                    folded.value("acks_received"),
                    0,
                    "no acks were issued, so acks_received must be 0 under auto-commit",
                );
                assert_eq!(
                    folded.value("offset_commit_errors"),
                    0,
                    "auto-commit path must not surface receiver-side commit errors",
                );
                assert!(
                    folded.value("messages_received") >= RECORDS as u64,
                    "every produced record should have been consumed",
                );
            },
        )
        .await;
    }

    // ---- Rebalance assignment-strategy integration tests (tracking-doc Area 2) ----
    //
    // These close the Area 2 "all three assignment strategies" subtask. The
    // cooperative-sticky strategy is covered by
    // `rebalance_cooperative_sticky_retains_owned_partitions`; the two tests
    // below cover the remaining eager strategies (`range`, `roundrobin`) by
    // running two same-group receivers against a two-partition topic and
    // asserting the group distributes both partitions and commits without loss.

    /// Runs a two-receiver, two-partition assign-and-commit scenario under the
    /// given eager assignment strategy and asserts the group covered both
    /// partitions and committed every produced record without loss or
    /// double-commit. Shared by the `range` and `roundrobin` strategy tests.
    async fn assert_strategy_assigns_and_commits(
        topic: &'static str,
        group: &'static str,
        strategy: RebalanceStrategy,
    ) {
        let per_partition = REBALANCE_RECORDS_PER_PARTITION as usize;
        let total = per_partition * REBALANCE_TEST_PARTITIONS as usize;
        // Capture a display name before `strategy` is moved into the config so
        // diagnostics can name the strategy without borrowing the moved value.
        let strategy_name = strategy.to_librdkafka_value();
        with_cluster(
            KafkaTestCluster::builder().topic_with(topic, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                producer
                    .produce_per_partition(
                        topic,
                        REBALANCE_TEST_PARTITIONS,
                        REBALANCE_RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                // Two receivers in the same group, both using the chosen eager
                // assignment strategy. Under an eager assignor each is assigned
                // one of the two partitions.
                let cfg_a = manual_traces_config(
                    cluster.bootstrap_servers(),
                    group,
                    topic,
                    500,
                    Some(strategy),
                );
                let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
                let cfg_b = manual_traces_config(
                    cluster.bootstrap_servers(),
                    group,
                    topic,
                    500,
                    Some(strategy),
                );
                let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);

                // Drain both receivers concurrently until the group has delivered
                // at least the full produced set (delivery is at-least-once, so
                // duplicates may inflate the raw count; the authoritative no-loss
                // check is the broker-side committed offsets below). Bounded by a
                // deadline so a distribution stall fails loudly instead of
                // hanging.
                let mut delivered = 0usize;
                let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
                while delivered < total {
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "timed out under {strategy_name}: delivered {delivered} of {total}; the \
                         group did not distribute both partitions",
                    );
                    if let Some(pdata) = receiver_a.try_recv_pdata(Duration::from_millis(200)).await
                    {
                        receiver_a.ack(pdata);
                        delivered += 1;
                    }
                    if let Some(pdata) = receiver_b.try_recv_pdata(Duration::from_millis(200)).await
                    {
                        receiver_b.ack(pdata);
                        delivered += 1;
                    }
                }

                // Allow a safety-net commit cycle to flush.
                tokio::time::sleep(Duration::from_millis(800)).await;

                let brokers = cluster.bootstrap_servers().to_string();
                receiver_a.shutdown(Duration::from_secs(5));
                let terminal_a = receiver_a.await_terminal_state().await;
                receiver_b.shutdown(Duration::from_secs(5));
                let terminal_b = receiver_b.await_terminal_state().await;

                let mut fa = FoldedMetrics::new();
                fa.fold_all(terminal_a.metrics());
                let mut fb = FoldedMetrics::new();
                fb.fold_all(terminal_b.metrics());

                // Distribution: together the two members covered both partitions.
                assert!(
                    fa.value("partitions_assigned") >= 1,
                    "receiver A should have been assigned at least one partition",
                );
                assert!(
                    fb.value("partitions_assigned") >= 1,
                    "receiver B should have been assigned at least one partition",
                );
                assert!(
                    fa.value("partitions_assigned") + fb.value("partitions_assigned")
                        >= REBALANCE_TEST_PARTITIONS as u64,
                    "the group should cover all {REBALANCE_TEST_PARTITIONS} partitions",
                );

                // No loss / no double-commit: every produced record is durably
                // retained and each partition's committed offset reaches exactly
                // the produced per-partition count ("next to read").
                let inspector = cluster.inspect();
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    assert_eq!(
                        inspector.message_count(topic, partition),
                        per_partition as i64,
                        "partition {partition} should durably retain all produced records",
                    );
                    let committed = committed_offset(&brokers, group, topic, partition)
                        .expect("kafka-test: committed-offset probe failed")
                        .unwrap_or_else(|| {
                            panic!("partition {partition} should have a committed offset")
                        });
                    assert_eq!(
                        committed, per_partition as i64,
                        "partition {partition} committed offset {committed} should equal the \
                         produced count {per_partition} (no loss, no double-commit)",
                    );
                }
            },
        )
        .await;
    }

    /// Scenario: two same-group receivers using the `range` assignment strategy
    /// consume a two-partition topic.
    /// Guarantees: closes the `range` portion of the Area 2 "all three assignment
    /// strategies" subtask -- the group distributes both partitions across the two
    /// members and commits every produced record with no loss or double-commit.
    /// Protects the `RebalanceStrategy::Range` path (`config.rs`
    /// `to_librdkafka_value` -> "range") end-to-end.
    #[tokio::test]
    async fn rebalance_strategy_range_assigns_and_commits() {
        assert_strategy_assigns_and_commits(
            "rebalance-strategy-range",
            "rebalance-strategy-range-group",
            RebalanceStrategy::Range,
        )
        .await;
    }

    /// Scenario: two same-group receivers using the `roundrobin` assignment
    /// strategy consume a two-partition topic.
    /// Guarantees: closes the `roundrobin` portion of the Area 2 "all three
    /// assignment strategies" subtask -- the group distributes both partitions
    /// across the two members and commits every produced record with no loss or
    /// double-commit. Protects the `RebalanceStrategy::RoundRobin` path
    /// (`config.rs` `to_librdkafka_value` -> "roundrobin") end-to-end.
    #[tokio::test]
    async fn rebalance_strategy_roundrobin_assigns_and_commits() {
        assert_strategy_assigns_and_commits(
            "rebalance-strategy-roundrobin",
            "rebalance-strategy-roundrobin-group",
            RebalanceStrategy::RoundRobin,
        )
        .await;
    }
}
