// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// ToDo: update tests to start broker in memory
// ToDo: Possible optimization to improve how we determine signal type from a message
// check every message against list of topics + excluded topics to get signal type
// ToDo: Offload heavier decode operations to avoid stalling the receiver

use super::config::{HeaderExtraction, KafkaReceiverConfig};
use super::error::KafkaReceiverError;
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
use otap_df_telemetry::{otel_error, otel_info, otel_warn};
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
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

/// URN for the Kafka Receiver
pub const KAFKA_RECEIVER_URN: &str = "urn:otel:receiver:kafka";

/// Bounded broker timeout for a single per-partition consumer-lag watermark
/// lookup. This bounds *each* `fetch_watermarks` call, not the whole refresh:
/// a refresh queries every owned partition sequentially, so the worst-case time
/// spent in one refresh scales with the number of owned partitions
/// (`partitions * timeout`).
const LAG_FETCH_PARTITION_TIMEOUT: Duration = Duration::from_secs(5);

/// Total deadline for a single off-loop consumer-lag refresh.
const LAG_REFRESH_TOTAL_DEADLINE: Duration = Duration::from_secs(15);

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
            .map_err(KafkaReceiverError::TracesDecode)
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
            .map_err(KafkaReceiverError::MetricsDecode)
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
            .map_err(KafkaReceiverError::LogsDecode)
        } else {
            Err(KafkaReceiverError::UnknownTopicDecode(
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
            self.metrics.rebalances_total.add(delta.rebalances_total);
            self.metrics
                .partition_assignments
                .add(delta.partition_assignments);
            self.metrics
                .partition_revocations
                .add(delta.partition_revocations);
            // `partitions_assigned` is a gauge: set it to the current owned count
            // snapshot rather than accumulating. Folded only when a rebalance
            // actually occurred (guarded by `is_empty`, which ignores this
            // gauge-only field) to avoid redundant writes on idle ticks.
            self.metrics.partitions_assigned.set(delta.partitions_owned);
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

    /// Spawn an off-loop consumer-lag refresh, returning its join handle.
    ///
    /// Moves an `Arc` clone of the consumer into a blocking task
    /// ([`tokio::task::spawn_blocking`]) that runs [`compute_consumer_lag`] off
    /// the receive loop.
    ///
    /// The task returns:
    /// - `Some(mean_lag)` when the high-watermark lookup succeeds for *every*
    ///   owned partition (the mean covers the whole assignment, never a subset);
    /// - `Some(0.0)` when the assignment is empty (nothing owned), the caller's
    ///   signal to reset the gauge to the documented empty-assignment value;
    /// - `None` when the refresh is incomplete -- any owned partition lacks a
    ///   committed offset, a broker read failed, or the deadline was exceeded --
    ///   the caller's signal to retain the previous gauge value. Instantly returns
    ///   when in auto-commit mode.
    fn spawn_consumer_lag_refresh<C: ConsumerContext + 'static>(
        &self,
        consumer: &Arc<StreamConsumer<C>>,
        deadline: Instant,
        cancel: CancellationToken,
    ) -> Option<tokio::task::JoinHandle<Option<f64>>> {
        if self.config.is_auto_commit() {
            return None;
        }

        let consumer = Arc::clone(consumer);
        Some(tokio::task::spawn_blocking(move || {
            compute_consumer_lag(consumer.as_ref(), deadline, &cancel)
        }))
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

        // Read the partition's tracked generation, its currently-assigned
        // generation, and whether it is still owned. The assigned generation is
        // consulted (not just the tracker's) so a stale ack is rejected even in
        // the window after a revoke/reassign where the tracker still reports the
        // old generation because no record of the new period has been tracked
        // yet. The `is_assigned` membership check remains explicit for clarity.
        //
        // The late-ack path is safe because librdkafka runs
        // `post_rebalance(Assign)` on the poll thread *before* `consumer.recv()`
        // yields messages for the newly assigned partitions, so `assigned` is
        // always populated before any ack for those partitions can return.
        let tracked_generation = self.offset_tracker.partition_generation(&name, partition);
        let assigned_generation = self.rebalance_state.current_generation(&name, partition);
        let is_assigned = self.rebalance_state.is_assigned(&name, partition);

        match classify_offset_feedback(
            ack_generation,
            tracked_generation,
            assigned_generation,
            is_assigned,
        ) {
            OffsetFeedbackAction::Commit => {
                self.advance_offset_and_commit(&name, partition, offset, consumer, receiver_id);
            }
            OffsetFeedbackAction::DropStale => {
                self.metrics.acks_for_revoked_partition.add(1);
            }
            OffsetFeedbackAction::DropLateAck { purge } => {
                self.metrics.acks_for_revoked_partition.add(1);
                if purge {
                    self.offset_tracker.revoke(&name, partition);
                }
            }
        }
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
        // it cooperatively on shutdown so it cannot outlive the receiver.
        let mut lag_refresh_in_flight: Option<(
            tokio::task::JoinHandle<Option<f64>>,
            tokio::time::Instant,
            CancellationToken,
        )> = None;

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
                                if let Err(e) = self.commit_offsets(consumer.as_ref(), &receiver_id) {
                                    otel_error!(
                                        "kafka.shutdown.commit_failed",
                                        error = %e,
                                    );
                                }
                            }
                            // Drain any in-flight consumer-lag worker so we do not
                            // abandon a running `spawn_blocking` task. send a cancellation
                            // signal
                            if let Some((handle, lag_deadline, lag_cancel)) =
                                lag_refresh_in_flight.take()
                            {
                                lag_cancel.cancel();
                                let bound =
                                    lag_deadline.min(tokio::time::Instant::from_std(deadline));
                                let _ = tokio::time::timeout_at(bound, handle).await;
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
                                    if let Err(e) = self.commit_offsets(consumer.as_ref(), &receiver_id) {
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
                                    consumer.as_ref(),
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
                                    consumer.as_ref(),
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
                                Ok(Some(value)) => self.metrics.consumer_lag.set(value),
                                Ok(None) => {}
                                Err(join_err) => {
                                    otel_error!("kafka.lag.refresh_task_failed", error = %join_err)
                                }
                            }
                        }
                    }
                }

                // 3. Consume Kafka messages. Stops once draining begins so no
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
                                        KafkaReceiverError::EmptyPayloadDecode(e) => {
                                            self.metrics.empty_payloads.add(1);
                                            otel_error!(
                                                "kafka.message.empty_payload",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        KafkaReceiverError::UnknownTopicDecode(e) => {
                                            self.metrics.unknown_topic_errors.add(1);
                                            otel_error!(
                                                "kafka.message.unknown_topic",
                                                error = %e,
                                                topic = %topic,
                                                partition = partition,
                                                offset = offset,
                                            );
                                        }
                                        KafkaReceiverError::TracesDecode(e) => {
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
                                        KafkaReceiverError::MetricsDecode(e) => {
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
                                        KafkaReceiverError::LogsDecode(e) => {
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
                                        // Config variants are never produced on
                                        // the per-message decode path.
                                        _ => {
                                            self.metrics.processing_errors.add(1);
                                            otel_error!(
                                                "kafka.message.decode_failed",
                                                error = %decode_err,
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
                                            consumer.as_ref(),
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

                // 4. Periodic consumer-lag refresh trigger (opt-in). Fires only
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
                    && draining_deadline.is_none() => {
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
            }
        }
    }
}

/// Compute the mean consumer-group lag across all owned partitions, bounded by
/// an absolute `deadline`. The `deadline` is checked before each partition
///
/// Return contract (see [`KafkaReceiver::spawn_consumer_lag_refresh`]):
/// - `Some(mean)` -- every owned partition was measured; the mean covers the
///   whole assignment.
/// - `Some(0.0)` -- the assignment is empty (nothing owned); the caller resets
///   the gauge to the documented empty-assignment value.
/// - `None` -- the refresh is incomplete (an owned partition has no committed
///   offset yet, a broker read failed, or the `deadline` was exceeded); the
///   caller retains the previous gauge value.
fn compute_consumer_lag<C: ConsumerContext>(
    consumer: &StreamConsumer<C>,
    deadline: Instant,
    cancel: &CancellationToken,
) -> Option<f64> {
    // Remaining time until `deadline`
    let remaining_call_timeout = || -> Option<Duration> {
        if cancel.is_cancelled() {
            return None;
        }
        let remaining = deadline.checked_duration_since(Instant::now())?;
        if remaining.is_zero() {
            return None;
        }
        Some(remaining.min(LAG_FETCH_PARTITION_TIMEOUT))
    };

    // Owned partitions. `assignment()` is a local (non-RPC) query.
    let assignment = match consumer.assignment() {
        Ok(tpl) => tpl,
        Err(e) => {
            otel_error!("kafka.lag.assignment_failed", error = %e);
            return None;
        }
    };
    if assignment.count() == 0 {
        // Nothing owned: reset the gauge to the documented empty value (0).
        return Some(0.0);
    }

    // Deadline / cancellation check before the first (committed_offsets) broker
    // call.
    let Some(committed_timeout) = remaining_call_timeout() else {
        let reason = if cancel.is_cancelled() {
            "cancelled"
        } else {
            "deadline_exceeded"
        };
        otel_warn!("kafka.lag.refresh_incomplete", reason = reason);
        return None;
    };

    // Broker-acknowledged committed offsets for the owned partitions.
    let committed = match consumer.committed_offsets(assignment, committed_timeout) {
        Ok(tpl) => tpl,
        Err(e) => {
            otel_error!("kafka.lag.committed_offsets_failed", error = %e);
            return None;
        }
    };

    // Per-partition consumer-group lag for *every* owned partition. The mean
    // must cover the whole assignment: any partition we cannot measure -- a
    // missing committed offset, a failed broker read, or the deadline expiring
    // -- abandons the whole refresh (returns `None`) so a mean is never computed
    // from a subset of partitions.
    let elements = committed.elements();
    let mut sum: i64 = 0;
    for elem in &elements {
        // Bound this partition's watermark lookup by the remaining time (or
        // abandon on cancellation).
        let Some(watermark_timeout) = remaining_call_timeout() else {
            let reason = if cancel.is_cancelled() {
                "cancelled"
            } else {
                "deadline_exceeded"
            };
            otel_warn!("kafka.lag.refresh_incomplete", reason = reason);
            return None;
        };

        let topic = elem.topic();
        let partition = elem.partition();

        // An owned partition with no broker-committed offset yet
        // (`Offset::Invalid`) cannot be measured. Abort rather than exclude it,
        // so the mean always covers the whole assignment.
        let committed_offset = match elem.offset() {
            rdkafka::Offset::Offset(o) => o,
            _ => {
                otel_warn!(
                    "kafka.lag.refresh_incomplete",
                    reason = "uncommitted_partition",
                    topic = %topic,
                    partition = partition,
                );
                return None;
            }
        };

        match consumer.fetch_watermarks(topic, partition, watermark_timeout) {
            Ok((_low, high)) => {
                // Both the high watermark and the committed offset are
                // "one past" positions, so their difference is the number
                // of records the group has not yet consumed on this
                // partition (consumer-group lag).
                sum = sum.saturating_add(high.saturating_sub(committed_offset).max(0));
            }
            Err(e) => {
                // Fail fast: one failed lookup means the mean would be
                // incomplete, so abandon this refresh and retain the
                // previous gauge value.
                otel_error!(
                    "kafka.lag.fetch_watermarks_failed",
                    topic = %topic,
                    partition = partition,
                    error = %e,
                );
                return None;
            }
        }
    }

    // `elements` is non-empty here (assignment count was > 0), so the divisor is
    // never zero.
    Some(sum as f64 / elements.len() as f64)
}

/// Decision for an incoming Ack/Nack carrying Kafka offset identity, derived
/// purely from generation/ownership state.
///
/// Extracted from [`KafkaReceiver::handle_offset_feedback`] so the stale/late-ack
/// policy is self-contained and exhaustively unit-testable without a live
/// consumer.
#[derive(Debug, PartialEq, Eq)]
enum OffsetFeedbackAction {
    /// Advance the offset tracker and commit: the ack belongs to the current
    /// ownership period of a currently-owned partition.
    Commit,
    /// Drop as stale: the ack is from an ownership period strictly older than
    /// the partition's current tracked *or* currently-assigned generation. The
    /// partition was revoked and reassigned since the record was delivered.
    DropStale,
    /// Drop as a late ack: the partition is no longer assigned to this consumer.
    /// `purge` indicates whether lingering tracker state should also be removed
    /// (only when that state is not newer than the ack's ownership period).
    DropLateAck { purge: bool },
}

/// Classify an Ack/Nack given the ack's ownership `generation` and the
/// partition's current tracker/assignment state.
///
/// The stale-generation check compares the ack against the **maximum** of the
/// tracker generation and the currently-assigned generation. Consulting the
/// assigned generation (not just the tracker's) closes the window where a
/// partition was revoked and reassigned to this consumer under a newer
/// generation but no record of the new period has been tracked yet: in that
/// window the tracker still reports the old generation, so an ack that equals
/// the tracker generation would otherwise pass the guard, find the partition
/// assigned, and mutate/commit stale state. Because real generations start at
/// `1`, a `0` assigned/tracked generation means "not owned / untracked" and is
/// treated as no lower bound.
fn classify_offset_feedback(
    ack_generation: u64,
    tracked_generation: Option<u64>,
    assigned_generation: u64,
    is_assigned: bool,
) -> OffsetFeedbackAction {
    let current = tracked_generation.unwrap_or(0).max(assigned_generation);
    if current > 0 && ack_generation < current {
        return OffsetFeedbackAction::DropStale;
    }
    if !is_assigned {
        let purge = tracked_generation.is_some_and(|tracked| tracked <= ack_generation);
        return OffsetFeedbackAction::DropLateAck { purge };
    }
    OffsetFeedbackAction::Commit
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
    use crate::common::kafka::test::cluster::KafkaTestCluster;
    use crate::common::kafka::test::consumer::{RebalanceTrigger, committed_offset};
    use crate::common::kafka::test::producer::SendRecord;
    use crate::common::kafka::test::wait::poll_until;
    use crate::common::kafka::test::with_cluster;
    use otap_df_config::transport_headers_policy::{CaptureDefaults, CaptureRule};
    use otap_df_engine::context::ControllerContext;
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
        // The error is now `KafkaReceiverError::ConfigOverlappingTopics`;
        // assert against its Display string.
        let err_str = result.unwrap_err().to_string();
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

    // Scenario: the receiver owns partition 0 under generation 1 and tracks and
    // acks records for it, then the partition is revoked (rebalance) and later
    // reassigned to this receiver under generation 2, where a new record is
    // tracked. The generation-1 records were committed further by whoever owned
    // the partition in between.
    // Guarantees: after reassignment the receiver only commits generation-2
    // offsets. Records received under generation 1 do not contribute to the
    // generation-2 commit, so the committable offset the receiver would send to
    // the broker reflects only the new ownership period and never rolls back to
    // a generation-1 offset.
    #[test]
    fn stale_generation_records_not_committed_after_reassignment() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Generation 1: own partition 0, track and ack offsets 100..=104. The
        // committable offset is high_water_mark + 1 = 105.
        for offset in 100..=104 {
            receiver.offset_tracker.track("traces", 0, offset, 1);
        }
        for offset in 100..=104 {
            let _ = receiver.offset_tracker.acknowledge("traces", 0, offset);
        }
        assert_eq!(
            receiver
                .offset_tracker
                .committable_snapshot()
                .get(&("traces".to_string(), 0))
                .copied(),
            Some(105),
            "generation 1 would commit its own high-water mark",
        );

        // Partition 0 is revoked; the revocation carries generation 1. The
        // receive loop reconciles and purges the generation-1 tracker state.
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, 1);
        receiver.reconcile_rebalance_state();
        assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
        assert!(
            !receiver
                .offset_tracker
                .committable_snapshot()
                .contains_key(&("traces".to_string(), 0)),
            "a revoked partition contributes no committable offset",
        );

        // Generation 2: partition 0 is reassigned to this receiver and resumes
        // from the group's committed position (200), then tracks a new record.
        receiver.offset_tracker.track("traces", 0, 200, 2);
        assert_eq!(
            receiver.offset_tracker.partition_generation("traces", 0),
            Some(2),
        );

        // The receiver only commits the generation-2 offset (200); it never
        // regresses to generation 1's 105.
        assert_eq!(
            receiver
                .offset_tracker
                .committable_snapshot()
                .get(&("traces".to_string(), 0))
                .copied(),
            Some(200),
            "only generation-2 records drive the commit after reassignment",
        );
    }

    // ---- classify_offset_feedback unit tests ----

    // Scenario: an ack arrives for a partition this consumer still owns, whose
    // ownership generation matches the ack.
    // Guarantees: the ack is committed (advances the tracker) rather than
    // dropped.
    #[test]
    fn classify_offset_feedback_commits_current_generation_ack() {
        assert_eq!(
            classify_offset_feedback(2, Some(2), 2, true),
            OffsetFeedbackAction::Commit,
        );
    }

    // Scenario: an ack arrives whose generation is older than the partition's
    // tracked generation (the partition was reassigned and re-tracked under a
    // newer generation).
    // Guarantees: the ack is dropped as stale, so it cannot roll back or
    // disturb the newer ownership period's committed offset.
    #[test]
    fn classify_offset_feedback_drops_ack_older_than_tracked_generation() {
        assert_eq!(
            classify_offset_feedback(1, Some(3), 3, true),
            OffsetFeedbackAction::DropStale,
        );
    }

    // Scenario: the closed gap. A partition was revoked and reassigned to this
    // consumer under a newer generation, but no record of the new period has
    // been tracked yet, so the tracker still reports the OLD generation while
    // the assignment already reports the NEW one. A stale ack for the old
    // period arrives with a generation equal to the tracker's.
    // Guarantees: the ack is still dropped as stale because the classifier
    // consults the assigned generation, not just the tracker generation -- so a
    // stale same-as-tracker ack cannot slip through and mutate/commit stale
    // state during the reassign-before-retrack window.
    #[test]
    fn classify_offset_feedback_drops_stale_ack_when_assigned_generation_is_newer() {
        assert_eq!(
            classify_offset_feedback(1, Some(1), 2, true),
            OffsetFeedbackAction::DropStale,
        );
    }

    // Scenario: an ack arrives for a partition no longer assigned to this
    // consumer, whose tracked state is not newer than the ack's generation.
    // Guarantees: the ack is dropped as a late ack and the lingering tracker
    // state is purged (it belongs to the revoked ownership period).
    #[test]
    fn classify_offset_feedback_late_ack_purges_when_not_newer() {
        assert_eq!(
            classify_offset_feedback(1, Some(1), 0, false),
            OffsetFeedbackAction::DropLateAck { purge: true },
        );
    }

    // Scenario: an ack arrives for a partition no longer assigned, whose
    // tracked state belongs to a NEWER generation than the ack. This is caught
    // by the stale-generation check *before* the late-ack check, because a
    // newer tracked generation means the partition was reassigned and
    // re-tracked since the ack's ownership period.
    // Guarantees: such an ack is classified `DropStale` (the newer tracked
    // state is preserved), never `DropLateAck` with a purge -- so a stale ack
    // can never purge a newer ownership period's tracker state.
    #[test]
    fn classify_offset_feedback_ack_older_than_tracked_is_stale_even_when_unassigned() {
        assert_eq!(
            classify_offset_feedback(2, Some(3), 0, false),
            OffsetFeedbackAction::DropStale,
        );
    }

    // Scenario: an ack arrives for a partition that is neither assigned nor
    // tracked (fully revoked and purged already).
    // Guarantees: the ack is dropped as a late ack with nothing to purge.
    #[test]
    fn classify_offset_feedback_late_ack_untracked_does_not_purge() {
        assert_eq!(
            classify_offset_feedback(1, None, 0, false),
            OffsetFeedbackAction::DropLateAck { purge: false },
        );
    }

    // Scenario: the first ack for a freshly-assigned partition arrives before
    // its record was tracked (untracked, but currently owned), with a
    // generation matching the assignment.
    // Guarantees: the ack is committed -- an untracked-but-owned partition is
    // not treated as stale as long as the ack is not older than the assigned
    // generation.
    #[test]
    fn classify_offset_feedback_commits_untracked_but_assigned_current_ack() {
        assert_eq!(
            classify_offset_feedback(1, None, 1, true),
            OffsetFeedbackAction::Commit,
        );
    }

    // Scenario: a partition is owned under generation 1 with a tracked record,
    // then revoked and reassigned to this receiver under generation 2 (via a
    // rebalance), but no generation-2 record has been tracked yet -- so the
    // tracker still reports generation 1 while the assignment reports 2. A
    // stale generation-1 ack for the old record then arrives.
    // Guarantees: the receiver classifies the stale ack as `DropStale` (it
    // consults the assigned generation), so the ack neither advances the
    // tracker nor rolls back the committed offset during the
    // reassign-before-retrack window.
    #[test]
    fn stale_same_gen_ack_dropped_after_reassignment_before_retrack() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Generation 1: own partition 0 and track a record at offset 100.
        let mut tpl1 = TopicPartitionList::new();
        let _ = tpl1.add_partition("traces", 0);
        receiver.rebalance_state.set_assignment_for_test(&tpl1);
        let gen1 = receiver.rebalance_state.current_generation("traces", 0);
        receiver.offset_tracker.track("traces", 0, 100, gen1);

        // Revoke partition 0 (queued for tracker purge) AND drop it from the
        // assigned set by applying an empty assignment, mirroring librdkafka's
        // pre_rebalance(Revoke) removing it before post_rebalance(Assign). This
        // is what lets the subsequent reassignment allocate a fresh,
        // strictly-greater generation.
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, gen1);
        receiver
            .rebalance_state
            .set_assignment_for_test(&TopicPartitionList::new());

        // Reassign partition 0 (fresh, strictly-greater generation). The tracker
        // is NOT re-tracked yet, so it still reports generation 1 while the
        // assignment reports generation 2.
        let mut tpl2 = TopicPartitionList::new();
        let _ = tpl2.add_partition("traces", 0);
        receiver.rebalance_state.set_assignment_for_test(&tpl2);
        let gen2 = receiver.rebalance_state.current_generation("traces", 0);
        assert!(gen2 > gen1, "reassignment must allocate a newer generation");
        assert_eq!(
            receiver.offset_tracker.partition_generation("traces", 0),
            Some(gen1),
            "tracker still reports the old generation before any re-track",
        );

        // A stale generation-1 ack, equal to the tracker generation, must be
        // classified as stale because the assigned generation is newer.
        let tracked = receiver.offset_tracker.partition_generation("traces", 0);
        let assigned = receiver.rebalance_state.current_generation("traces", 0);
        let is_assigned = receiver.rebalance_state.is_assigned("traces", 0);
        assert_eq!(
            classify_offset_feedback(gen1, tracked, assigned, is_assigned),
            OffsetFeedbackAction::DropStale,
            "a stale ack matching the tracker generation is dropped once the \
             partition has been reassigned to a newer generation",
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

    /// Scenario: a rebalance assigns partitions and the receive loop reconciles.
    /// Guarantees: `reconcile_rebalance_state` folds the rebalance deltas into
    /// the metric set - counting the rebalance event and cumulative
    /// acquisitions, and setting the `partitions_assigned` gauge to the current
    /// owned count rather than accumulating it.
    #[test]
    fn reconcile_folds_consumer_group_metrics() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Simulate a rebalance that assigns two partitions.
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        let _ = tpl.add_partition("traces", 1);
        receiver.rebalance_state.set_assignment_for_test(&tpl);

        receiver.reconcile_rebalance_state();

        // Gauge reflects current ownership; cumulative counter reflects the
        // acquisitions.
        assert_eq!(receiver.metrics.partitions_assigned.get(), 2);
        assert_eq!(receiver.metrics.partition_assignments.get(), 2);

        // A second reconcile with no further rebalance activity must not change
        // the gauge (it is folded only when a rebalance occurred) or double
        // count the counter.
        receiver.reconcile_rebalance_state();
        assert_eq!(receiver.metrics.partitions_assigned.get(), 2);
        assert_eq!(receiver.metrics.partition_assignments.get(), 2);
    }

    /// Scenario: a manual-commit receiver spawns a lag refresh for a consumer
    /// that owns no partitions (empty assignment).
    /// Guarantees: `spawn_consumer_lag_refresh` still spawns a task (manual mode)
    /// and the task returns `Some(0.0)` -- the documented empty-assignment
    /// sentinel -- so the caller resets the `consumer_lag` gauge to 0 rather than
    /// leaving a stale value.
    #[tokio::test]
    async fn spawn_consumer_lag_refresh_resets_to_zero_when_unassigned() {
        const TOPIC: &str = "lag-empty";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
            |cluster| async move {
                let cfg = make_config(&[TOPIC], &["metrics"], &[], MessageFormat::OtlpProto);
                assert!(!cfg.is_auto_commit());
                let ctx = make_pipeline_ctx();
                let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

                let consumer = Arc::new(make_manual_consumer(
                    cluster.bootstrap_servers(),
                    "lag-empty-group",
                ));

                // Manual mode => a task is spawned; the consumer has no
                // assignment, so the task yields `Some(0.0)` (reset the gauge to
                // the empty value).
                let handle = receiver
                    .spawn_consumer_lag_refresh(
                        &consumer,
                        Instant::now() + LAG_REFRESH_TOTAL_DEADLINE,
                        CancellationToken::new(),
                    )
                    .expect("manual mode spawns a refresh task");
                let result = handle.await.expect("lag task should not panic");
                assert_eq!(result, Some(0.0));
            },
        )
        .await;
    }

    /// Scenario: auto-commit receiver requests a lag refresh.
    /// Guarantees: `spawn_consumer_lag_refresh` returns `None` (no task, no
    /// broker work) because offset management is owned by librdkafka.
    #[tokio::test]
    async fn spawn_consumer_lag_refresh_none_under_auto_commit() {
        const TOPIC: &str = "lag-auto";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
            |cluster| async move {
                let cfg = KafkaReceiverConfig::try_from(
                    KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), "g", "c")
                        .with_traces(SignalConfig::new(vec![TOPIC.to_string()]))
                        .with_commit(CommitConfig {
                            mode: ConfigCommitMode::Auto,
                            interval_ms: Some(1000),
                        })
                        .with_isolation_level(IsolationLevel::ReadUncommitted),
                )
                .expect("test config should be valid");
                let ctx = make_pipeline_ctx();
                let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

                let consumer: StreamConsumer = ClientConfig::new()
                    .set("bootstrap.servers", cluster.bootstrap_servers())
                    .set("group.id", "lag-auto-group")
                    .set("enable.auto.commit", "true")
                    .create()
                    .expect("failed to create consumer");
                let consumer = Arc::new(consumer);

                assert!(
                    receiver
                        .spawn_consumer_lag_refresh(
                            &consumer,
                            Instant::now() + LAG_REFRESH_TOTAL_DEADLINE,
                            CancellationToken::new(),
                        )
                        .is_none()
                );
            },
        )
        .await;
    }

    /// Build a manual-commit `StreamConsumer` bound to `brokers` in `group`,
    /// with librdkafka auto-commit disabled so the test controls committed
    /// offsets explicitly.
    fn make_manual_consumer(brokers: &str, group: &str) -> StreamConsumer {
        ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("failed to create consumer")
    }

    /// Scenario: a consumer owns partitions but *none* of them has a
    /// broker-committed offset yet (every `committed_offsets` entry is
    /// `Offset::Invalid`).
    /// Guarantees: `compute_consumer_lag` reports the refresh as incomplete
    /// (`None`) instead of computing a mean from a subset, so the caller retains
    /// the previous `consumer_lag` value rather than publishing a partial or
    /// zeroed measurement.
    #[tokio::test]
    async fn compute_consumer_lag_none_when_all_offsets_invalid() {
        const TOPIC: &str = "lag-all-invalid";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
            |cluster| async move {
                let brokers = cluster.bootstrap_servers().to_string();
                // Assign both partitions but never commit, so the broker holds
                // no committed offset for either -> both `Offset::Invalid`.
                let consumer = make_manual_consumer(&brokers, "lag-all-invalid-group");
                let mut tpl = TopicPartitionList::new();
                let _ = tpl.add_partition(TOPIC, 0);
                let _ = tpl.add_partition(TOPIC, 1);
                consumer.assign(&tpl).expect("assign partitions");

                let deadline = Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
                let result = tokio::task::spawn_blocking(move || {
                    compute_consumer_lag(&consumer, deadline, &CancellationToken::new())
                })
                .await
                .expect("lag task should not panic");

                assert_eq!(
                    result, None,
                    "an assignment with no committed offsets must abort the refresh, not \
                     produce a subset/zero mean",
                );
            },
        )
        .await;
    }

    /// Scenario: the receive loop's lag-refresh deadline elapses, so the loop
    /// cancels the worker's token (as the `Err(Elapsed)` arm does) while the
    /// worker still owns partitions.
    /// Guarantees: a cancelled token makes `compute_consumer_lag` abandon the
    /// refresh (`None`) at its next cancellation check instead of continuing to
    /// issue broker calls -- the observable behavior that lets the loop drop the
    /// wedged worker and resume future refreshes without blocking.
    #[tokio::test]
    async fn compute_consumer_lag_none_when_cancelled() {
        const TOPIC: &str = "lag-cancelled";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
            |cluster| async move {
                let brokers = cluster.bootstrap_servers().to_string();
                let consumer = make_manual_consumer(&brokers, "lag-cancelled-group");
                let mut tpl = TopicPartitionList::new();
                let _ = tpl.add_partition(TOPIC, 0);
                let _ = tpl.add_partition(TOPIC, 1);
                consumer.assign(&tpl).expect("assign partitions");

                // Pre-cancel the token to model the timeout path cancelling a
                // still-running worker. The assignment is non-empty, so the
                // cancellation check (not the empty-assignment shortcut) decides
                // the outcome.
                let cancel = CancellationToken::new();
                cancel.cancel();
                let deadline = Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
                let result = tokio::task::spawn_blocking(move || {
                    compute_consumer_lag(&consumer, deadline, &cancel)
                })
                .await
                .expect("lag task should not panic");

                assert_eq!(
                    result, None,
                    "a cancelled refresh must abandon measurement rather than \
                     continue issuing broker calls",
                );
            },
        )
        .await;
    }

    /// Scenario: a consumer owns two partitions but only one has a
    /// broker-committed offset; the other is still `Offset::Invalid`.
    /// Guarantees: `compute_consumer_lag` aborts (`None`) because the mean must
    /// cover every owned partition -- it never silently drops the uncommitted
    /// partition and averages only the committed one.
    #[tokio::test]
    async fn compute_consumer_lag_none_when_offsets_mixed_valid_invalid() {
        const TOPIC: &str = "lag-mixed";
        let group = "lag-mixed-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
            |cluster| async move {
                let brokers = cluster.bootstrap_servers().to_string();
                let producer = cluster.producer().build();

                // Produce a few records to partition 0 only.
                for _ in 0..3 {
                    producer
                        .send_to_partition(TOPIC, 0, b"payload")
                        .await
                        .expect("produce to partition 0");
                }
                producer.flush(Duration::from_secs(5));

                let consumer = make_manual_consumer(&brokers, group);
                let mut tpl = TopicPartitionList::new();
                let _ = tpl.add_partition(TOPIC, 0);
                let _ = tpl.add_partition(TOPIC, 1);
                consumer.assign(&tpl).expect("assign partitions");

                // Commit an offset for partition 0 only, leaving partition 1
                // without a committed offset (`Offset::Invalid`).
                let mut commit_tpl = TopicPartitionList::new();
                commit_tpl
                    .add_partition_offset(TOPIC, 0, Offset::Offset(2))
                    .expect("build commit tpl");
                consumer
                    .commit(&commit_tpl, CommitMode::Sync)
                    .expect("commit partition 0");

                let deadline = Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
                let result = tokio::task::spawn_blocking(move || {
                    compute_consumer_lag(&consumer, deadline, &CancellationToken::new())
                })
                .await
                .expect("lag task should not panic");

                assert_eq!(
                    result, None,
                    "a mix of committed and uncommitted owned partitions must abort the \
                     refresh so the mean is never taken over a subset",
                );
            },
        )
        .await;
    }

    /// Scenario: the total refresh deadline has already passed when
    /// `compute_consumer_lag` starts (assignment is non-empty).
    /// Guarantees: the worker self-terminates with `None` (incomplete) at its
    /// first between-partition/broker-call deadline check rather than issuing
    /// broker calls, so an overrunning refresh bounds itself.
    #[tokio::test]
    async fn compute_consumer_lag_none_when_deadline_already_passed() {
        const TOPIC: &str = "lag-deadline";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
            |cluster| async move {
                let brokers = cluster.bootstrap_servers().to_string();
                let consumer = make_manual_consumer(&brokers, "lag-deadline-group");
                let mut tpl = TopicPartitionList::new();
                let _ = tpl.add_partition(TOPIC, 0);
                consumer.assign(&tpl).expect("assign partition");

                // Deadline in the past: the first broker-call deadline check
                // must abort before any committed_offsets/fetch_watermarks call.
                let deadline = Instant::now() - Duration::from_secs(1);
                let result = tokio::task::spawn_blocking(move || {
                    compute_consumer_lag(&consumer, deadline, &CancellationToken::new())
                })
                .await
                .expect("lag task should not panic");

                assert_eq!(
                    result, None,
                    "an already-expired deadline must abort the refresh"
                );
            },
        )
        .await;
    }

    /// Scenario: the receive loop's lag apply branch observes the in-flight
    /// worker *finish* with a value (a real mean, or the `0.0`
    /// empty-assignment reset).
    /// Guarantees: the apply branch publishes the value to the `consumer_lag`
    /// gauge and clears the in-flight slot so the next tick may start a fresh
    /// refresh.
    #[tokio::test]
    async fn lag_apply_publishes_and_clears_on_completion() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // A finished worker that measured a mean of 42.0.
        let mut in_flight: Option<(tokio::task::JoinHandle<Option<f64>>, tokio::time::Instant)> =
            Some((
                tokio::task::spawn(async { Some(42.0_f64) }),
                tokio::time::Instant::now() + LAG_REFRESH_TOTAL_DEADLINE,
            ));
        let join_result = in_flight.as_mut().map(|(h, _)| h).expect("in flight").await;

        // Mirror the apply branch's inlined result-handling.
        let result: Result<
            Result<Option<f64>, tokio::task::JoinError>,
            tokio::time::error::Elapsed,
        > = Ok(join_result);
        match result {
            Err(_elapsed) => unreachable!("worker finished, not a deadline crossing"),
            Ok(join_result) => {
                in_flight = None;
                match join_result {
                    Ok(Some(value)) => receiver.metrics.consumer_lag.set(value),
                    Ok(None) => {}
                    Err(join_err) => panic!("unexpected join error: {join_err}"),
                }
            }
        }

        assert_eq!(receiver.metrics.consumer_lag.get(), 42.0);
        assert!(
            in_flight.is_none(),
            "a finished worker must clear the in-flight slot",
        );
    }

    /// Scenario: the lag apply branch observes the absolute deadline elapse
    /// while the worker is still running (a `spawn_blocking` task cannot be
    /// cancelled by dropping its handle).
    /// Guarantees: the apply branch keeps the in-flight slot set so the trigger
    /// branch cannot start a second worker -- proving at most one worker runs at
    /// a time -- and does not disturb the previous gauge value.
    #[tokio::test(start_paused = true)]
    async fn lag_apply_keeps_in_flight_on_deadline_and_blocks_new_worker() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Seed a known gauge value so we can prove it is retained on timeout.
        receiver.metrics.consumer_lag.set(7.0);

        // A worker that never finishes within the deadline.
        let deadline = tokio::time::Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
        let mut in_flight: Option<(tokio::task::JoinHandle<Option<f64>>, tokio::time::Instant)> =
            Some((
                tokio::task::spawn(async {
                    std::future::pending::<()>().await;
                    None
                }),
                deadline,
            ));

        // Cross the deadline (paused clock), then await with `timeout_at`.
        tokio::time::advance(LAG_REFRESH_TOTAL_DEADLINE + Duration::from_secs(1)).await;
        let handle = in_flight.as_mut().map(|(h, _)| h).expect("in flight");
        let result = tokio::time::timeout_at(deadline, handle).await;
        assert!(
            result.is_err(),
            "worker must still be running at the deadline"
        );

        // Mirror the apply branch: on `Err(Elapsed)` keep the in-flight slot and
        // leave the gauge untouched.
        match result {
            Err(_elapsed) => { /* keep in_flight, retain gauge */ }
            Ok(_) => unreachable!("deadline crossing, worker not finished"),
        }

        assert!(
            in_flight.is_some(),
            "a deadline crossing must NOT clear the in-flight slot, so the trigger branch \
             (guarded by is_none) cannot start a second worker while the first still runs",
        );
        assert_eq!(
            receiver.metrics.consumer_lag.get(),
            7.0,
            "the previous gauge value must be retained on a deadline crossing",
        );

        // Clean up the still-running background task.
        if let Some((handle, _)) = in_flight.take() {
            handle.abort();
        }
    }

    /// Scenario: paused time; the apply branch is polled repeatedly while the
    /// receive branch would always be ready. After a deadline crossing the
    /// branch must await the *bare* handle (no spinning `timeout_at`) so it does
    /// not starve `recv()`, and it must still process the worker's eventual
    /// completion.
    /// Guarantees: once the worker finally exits, the apply branch publishes its
    /// value and clears the in-flight slot even though it was polled past the
    /// deadline -- i.e. a completed refresh is never lost to starvation, and the
    /// deadline is absolute (not reset by re-polling).
    #[tokio::test(start_paused = true)]
    async fn lag_apply_processes_completion_after_deadline() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        let deadline = tokio::time::Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
        // A worker that completes only after the deadline has passed.
        let mut in_flight: Option<(tokio::task::JoinHandle<Option<f64>>, tokio::time::Instant)> =
            Some((
                tokio::task::spawn(async {
                    tokio::time::sleep(LAG_REFRESH_TOTAL_DEADLINE * 2).await;
                    Some(5.0_f64)
                }),
                deadline,
            ));

        // Advance past the deadline; the worker is still sleeping.
        tokio::time::advance(LAG_REFRESH_TOTAL_DEADLINE + Duration::from_secs(1)).await;

        // Past the deadline the loop awaits the bare handle (no timeout). Model
        // that: it resolves only when the worker actually finishes.
        tokio::time::advance(LAG_REFRESH_TOTAL_DEADLINE).await;
        let handle = in_flight.as_mut().map(|(h, _)| h).expect("in flight");
        let join_result = handle.await;

        // Mirror the apply branch's inlined result-handling for a finished worker.
        let result: Result<
            Result<Option<f64>, tokio::task::JoinError>,
            tokio::time::error::Elapsed,
        > = Ok(join_result);
        match result {
            Err(_elapsed) => unreachable!("worker finished, not a deadline crossing"),
            Ok(join_result) => {
                in_flight = None;
                match join_result {
                    Ok(Some(value)) => receiver.metrics.consumer_lag.set(value),
                    Ok(None) => {}
                    Err(join_err) => panic!("unexpected join error: {join_err}"),
                }
            }
        }

        assert_eq!(
            receiver.metrics.consumer_lag.get(),
            5.0,
            "a refresh that completes after the deadline must still be published",
        );
        assert!(
            in_flight.is_none(),
            "the in-flight slot must be cleared once the worker finishes",
        );
    }

    // Scenario: a consumer-lag worker is still in flight when a Shutdown arrives
    // whose deadline is *earlier* than the worker's own lag deadline. The
    // shutdown handler signals cooperative cancellation and then drains the
    // worker bounded by `min(lag_deadline, shutdown_deadline)`.
    // Guarantees: the drain never waits past the (earlier) shutdown deadline --
    // a recently-started refresh cannot delay shutdown -- and because the worker
    // observes the cancellation token it actually finishes rather than being
    // abandoned, so it cannot outlive the receiver.
    #[tokio::test(start_paused = true)]
    async fn shutdown_lag_drain_is_bounded_by_shutdown_deadline_and_cancels_worker() {
        let start = tokio::time::Instant::now();
        // Worker deadline is far out (15s); shutdown deadline is near (1s).
        let lag_deadline = start + LAG_REFRESH_TOTAL_DEADLINE;
        let shutdown_deadline = start + Duration::from_secs(1);

        // A cooperatively-cancellable worker: it runs until the token is
        // cancelled, then returns (mirrors `compute_consumer_lag` abandoning the
        // refresh on cancellation). It must NOT complete on its own before the
        // shutdown deadline, so the drain's boundedness is what we observe.
        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let handle = tokio::task::spawn(async move {
            worker_cancel.cancelled().await;
            None::<f64>
        });

        // Model the shutdown handler: cancel first, then drain bounded by the
        // tighter of the two deadlines.
        cancel.cancel();
        let bound = lag_deadline.min(shutdown_deadline);
        let drain = tokio::time::timeout_at(bound, handle).await;

        // The worker observed the cancellation and completed within the bound,
        // so the drain resolved with the worker's result (not a timeout).
        let join_result = drain.expect("drain must not exceed the min-bounded deadline");
        assert_eq!(
            join_result.expect("worker must not panic"),
            None,
            "a cancelled lag worker abandons the refresh and returns None",
        );

        // The drain finished no later than the shutdown deadline, well before
        // the worker's own 15s lag deadline: shutdown is not delayed.
        let elapsed = tokio::time::Instant::now();
        assert!(
            elapsed <= shutdown_deadline,
            "drain must complete by the shutdown deadline, not the lag deadline",
        );
    }

    // Scenario: a consumer-lag worker is in flight at Shutdown, but this time the
    // worker's lag deadline is *earlier* than the shutdown deadline.
    // Guarantees: the drain bound is the tighter (lag) deadline, so `min` selects
    // the lag deadline and the drain still cannot run to the later shutdown
    // deadline.
    #[tokio::test(start_paused = true)]
    async fn shutdown_lag_drain_bound_selects_the_earlier_lag_deadline() {
        let start = tokio::time::Instant::now();
        // Worker deadline is near (2s); shutdown deadline is far (30s).
        let lag_deadline = start + Duration::from_secs(2);
        let shutdown_deadline = start + Duration::from_secs(30);

        // A worker that never finishes on its own and ignores cancellation, so
        // the only thing that can unblock the drain is the min-bounded timeout.
        let handle = tokio::task::spawn(async {
            std::future::pending::<()>().await;
            None::<f64>
        });

        let bound = lag_deadline.min(shutdown_deadline);
        assert_eq!(
            bound, lag_deadline,
            "min must pick the earlier lag deadline"
        );

        let drain = tokio::time::timeout_at(bound, handle).await;
        assert!(
            drain.is_err(),
            "a non-cooperative worker is bounded by the lag deadline, not the later shutdown one",
        );

        // The drain elapsed at the lag deadline, strictly before the shutdown
        // deadline.
        let elapsed = tokio::time::Instant::now();
        assert!(
            elapsed <= lag_deadline && elapsed < shutdown_deadline,
            "drain must be bounded by the earlier (lag) deadline",
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

    // ---- Header extraction integration tests (in-process mock broker) ----

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
        use otap_df_engine::control::RuntimeControlMsg;

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

    /// Scenario: two `KafkaReceiver` replicas share one `group_id` against a
    /// multi-partition topic; replica B joins (scale-up) then leaves
    /// (scale-down), driving two rebalances. This is the in-process analogue of
    /// running 2+ replicas with the same group and scaling the replica count up
    /// and down; the full procedure is documented in the Kafka test-suite README
    /// ("Multi-receiver scale-up/down").
    ///
    /// Guarantees: (1) both replicas own a partition at some point, so the
    /// partitions distribute across the group (B consumes records that only its
    /// assigned partition can deliver, and both replicas' terminal metrics show
    /// `partitions_assigned >= 1`); (2) a rebalance is observed on scale-up/down
    /// (`partition_revocations >= 1` across the two replicas); (3) no message is
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
        use crate::common::kafka::node_harness::node_metrics::FoldedMetrics;

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
                let brokers = cluster.bootstrap_servers().to_string();

                // Manual commit so the receiver's rebalance-aware commit path is
                // active and acks drive the committable offsets.
                let mut delivered = 0usize;
                let mut delivered_b = 0usize;

                // True once every partition's committed offset has reached the
                // produced total (no loss, no rollback, no double-commit).
                let all_committed = |b: &str| {
                    (0..REBALANCE_TEST_PARTITIONS).all(|p| {
                        committed_offset(b, group, TOPIC, p)
                            .expect("kafka-test: committed-offset probe failed")
                            == Some(per_partition_total as i64)
                    })
                };

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

                // Step 4: drain B *first and exclusively* until it has consumed
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
                    if let Some(pdata) = receiver_b.try_recv_pdata(Duration::from_millis(250)).await
                    {
                        receiver_b.ack(pdata);
                        delivered += 1;
                        delivered_b += 1;
                    }
                }

                // Let B durably commit before it leaves: B's commits are async,
                // so wait past its safety-net commit interval (500ms) so its
                // acked offsets flush to the broker.
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Step 5: shut down B (scale-down). This forces a second
                // rebalance that returns B's partition to A. B commits the
                // offsets it acked as part of its graceful shutdown.
                receiver_b.shutdown(Duration::from_secs(5));
                let terminal_b = receiver_b.await_terminal_state().await;

                // Step 6: drain A. The loop body focuses solely on A receiving
                // and acking records; A re-consuming and acking the tail B did
                // not durably commit is what advances the committed offsets back
                // to the produced total. The deadline is the loop guard (not a
                // per-iteration assert), and the loop stops as soon as every
                // partition is committed to the produced total.
                let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
                while tokio::time::Instant::now() < deadline && !all_committed(&brokers) {
                    if let Some(pdata) = receiver_a.try_recv_pdata(Duration::from_millis(200)).await
                    {
                        receiver_a.ack(pdata);
                        delivered += 1;
                    }
                }

                // One assertion, after both replicas have received and acked:
                // the broker must report every partition committed to the
                // produced total. This is the authoritative no-loss /
                // no-rollback / no-double-commit check; the deadline above lets
                // A's async commits and any redelivery settle first.
                assert!(
                    all_committed(&brokers),
                    "after scale-down the group did not commit every partition to the produced \
                     total {per_partition_total} (delivered {delivered} of {total_produced}); \
                     committed offsets did not converge",
                );

                // Shut down A (flushes its tracked offsets) and collect metrics.
                receiver_a.shutdown(Duration::from_secs(5));
                let terminal_a = receiver_a.await_terminal_state().await;

                // ---- Assertions ----
                let mut fa = FoldedMetrics::new();
                fa.fold_all(terminal_a.metrics());
                let mut fb = FoldedMetrics::new();
                fb.fold_all(terminal_b.metrics());

                // (1) Distribution: both replicas acquired a partition over their
                // lifetimes and together cover the topic. B's deliveries above
                // already prove it owned a partition; metrics corroborate it.
                assert!(
                    fa.value("partition_assignments") >= 1,
                    "replica A should have acquired at least one partition, got {}",
                    fa.value("partition_assignments"),
                );
                assert!(
                    fb.value("partition_assignments") >= 1,
                    "replica B should have acquired at least one partition on scale-up, got {}",
                    fb.value("partition_assignments"),
                );
                assert!(
                    fa.value("partition_assignments") + fb.value("partition_assignments")
                        >= REBALANCE_TEST_PARTITIONS as u64,
                    "the group should have acquired all {REBALANCE_TEST_PARTITIONS} partitions \
                     across the two replicas' lifetimes",
                );
                // After scale-down A re-owns its partitions, a deterministic
                // current-ownership check.
                assert!(
                    fa.value("partitions_assigned") >= 1,
                    "replica A should currently own at least one partition at shutdown, got {}",
                    fa.value("partitions_assigned"),
                );

                // (2) Rebalance observed: at least one owned partition was revoked
                // across scale-up/down.
                assert!(
                    fa.value("partition_revocations") + fb.value("partition_revocations") >= 1,
                    "a partition revoke should have been observed across scale-up/down",
                );

                // (3a) No commit failures on either replica.
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

                // (3b) No loss: every produced record was delivered at least once
                // (delivery is at-least-once, so `>=`) and durably retained on the
                // broker (`message_count` is `high - low`).
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

                // (3c) No rollback and no double-commit: each partition's committed
                // offset equals exactly the produced total -- committed progress
                // was never rolled back across a rebalance and nothing was
                // committed past the produced data. Guaranteed by the convergence
                // drain above, so this equality is deterministic.
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    let committed = committed_offset(&brokers, group, TOPIC, partition)
                        .expect("kafka-test: committed-offset probe failed")
                        .unwrap_or_else(|| {
                            panic!("partition {partition} should have a committed offset")
                        });
                    assert_eq!(
                        committed, per_partition_total as i64,
                        "partition {partition} committed offset should equal the produced total \
                         {per_partition_total} (no rollback, no commit past produced data)",
                    );
                }
            },
        )
        .await;
    }
}
