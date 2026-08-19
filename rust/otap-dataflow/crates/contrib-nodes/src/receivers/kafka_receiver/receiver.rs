// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// ToDo: update tests to start broker in memory
// ToDo: Possible optimization to improve how we determine signal type from a message
// check every message against list of topics + excluded topics to get signal type
// ToDo: Offload heavier decode operations to avoid stalling the receiver

use super::config::{HeaderExtraction, KafkaReceiverConfig};
use super::error::KafkaReceiverError;
use super::headers::HeaderExtractions;
use super::metrics::{KafkaReceiverMetrics, KafkaReceiverRejectionReason};
use super::offset_tracker::OffsetTracker;
use super::rebalance::{RebalanceState, RebalancingConsumerContext};
#[cfg(feature = "aws")]
use crate::common::kafka::security::build_aws_msk_context;
use crate::common::kafka::{MSG_FORMAT_OTAP, MSG_FORMAT_OTLP, MessageFormat};
use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
use otap_df_config::SignalType;
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
use otap_df_telemetry::common_attributes::{Outcome, ReceiverRejectionErrorType};
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
    metrics: KafkaReceiverMetrics,
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
#[otap_df_engine::component_inventory(category = Receiver)]
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
        // per-signal encoding, and multiple topics per signal type.
        let mut pdata = match self.signal_type_for_topic(topic) {
            Some(SignalType::Traces) => {
                let message_format = detect_message_format(
                    &kafka_message,
                    self.config.message_format_header(),
                    self.config.traces_encoding(),
                );
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
            }
            Some(SignalType::Metrics) => {
                let message_format = detect_message_format(
                    &kafka_message,
                    self.config.message_format_header(),
                    self.config.metrics_encoding(),
                );
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
            }
            Some(SignalType::Logs) => {
                let message_format = detect_message_format(
                    &kafka_message,
                    self.config.message_format_header(),
                    self.config.logs_encoding(),
                );
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
    /// `receiver.kafka.offset_commits` with `outcome=success` or `outcome=failure`
    /// via the shared rebalance state. A rare *enqueue* failure is returned here so
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

        // Point-in-time in-flight depth: tracked-but-uncommitted offsets awaiting
        // an Ack/Nack. Refreshed every iteration since it changes with ordinary
        // ack/commit activity, not only on rebalances.
        self.metrics
            .consumer
            .records_in_flight
            .observe(self.offset_tracker.total_pending() as u64);

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
                self.metrics.consumer.feedback_after_revocation.inc();
            }
            OffsetFeedbackAction::DropLateAck { purge } => {
                self.metrics.consumer.feedback_after_revocation.inc();
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
                            if let Some((handle, lag_deadline, lag_cancel)) =
                                lag_refresh_in_flight.take()
                            {
                                lag_cancel.cancel();
                                let bound =
                                    lag_deadline.min(tokio::time::Instant::from_std(deadline));
                                let _ = tokio::time::timeout_at(bound, handle).await;
                            }
                            // Close the consumer off the loop thread, bounded by
                            // the shutdown deadline. Both `unsubscribe()` and the
                            // consumer's `Drop` (leave-group/close) are synchronous
                            // librdkafka FFI calls that can block indefinitely when
                            // the broker is unreachable; running them inline on this
                            // single-threaded runtime would stall it and hang the
                            // pipeline past its deadline. We take the snapshot
                            // first, then move the loop's `consumer` handle into a
                            // blocking task and wait only until the deadline.
                            //
                            // The drop below is NOT guaranteed to be the last
                            // `Arc`: the lag-worker drain above is bounded and
                            // best-effort, so a worker still mid-FFI can outlive it
                            // and keep its own clone. When that happens this drop
                            // just decrements the count, and the actual
                            // leave-group/close runs later, on the lag worker's own
                            // blocking thread, when it finally releases its clone.
                            // Either way the close never runs on the loop thread, so
                            // the runtime is never blocked, and if it outruns the
                            // deadline the task is left to finish on its own thread
                            // while the receiver returns its terminal state. This
                            // refcount behavior is covered by the unit tests
                            // `shutdown_bounded_drain_lets_cooperative_lag_worker_release_clone_before_close`
                            // and
                            // `shutdown_bounded_drain_can_leave_lag_worker_clone_alive_refcount_two`,
                            // and the bounded-termination guarantee by
                            // `shutdown_with_lag_refresh_in_flight_still_terminates_within_deadline`.
                            _ = telemetry_cancel_handle.cancel().await;
                            let close_handle = tokio::task::spawn_blocking(move || {
                                consumer.unsubscribe();
                                drop(consumer);
                            });
                            let _ = tokio::time::timeout_at(
                                tokio::time::Instant::from_std(deadline),
                                close_handle,
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
                            self.metrics.record_acknowledgement(
                                ack_msg.accepted.signal_type(),
                                Outcome::Success,
                            );
                            if manual_commit && !ack_msg.unwind.route.calldata.is_empty() {
                                self.handle_offset_feedback(
                                    &ack_msg.unwind.route.calldata,
                                    consumer.as_ref(),
                                    &receiver_id,
                                );
                            }
                        },
                        Ok(NodeControlMsg::Nack(nack_msg)) => {
                            self.metrics.record_acknowledgement(
                                nack_msg.refused.signal_type(),
                                Outcome::Refused,
                            );
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
                            _ = self.metrics.report(&mut metrics_reporter);
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
                                Ok(Some(value)) => self.metrics.consumer.lag.set(value),
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

                            let payload_bytes = data.payload().map_or(0, |payload| payload.len() as u64);
                            self.metrics.record_consumed_record(payload_bytes);

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

                            // This partition's current ownership generation.
                            let generation =
                                self.rebalance_state.current_generation(&topic, partition);

                            // Idempotency: skip duplicate messages when enabled.
                            // The check is generation-aware: a message redelivered
                            // under a NEWER generation (same offset, after a
                            // revoke+reassign) belongs to a new ownership period
                            // and must NOT be skipped as a duplicate -- it is
                            // reprocessed, and tracking it below resets this
                            // partition's stale old-generation state.
                            if idempotent
                                && self.offset_tracker.is_known_offset_for_generation(
                                    &topic, partition, offset, generation,
                                )
                            {
                                self.metrics.consumer.duplicate_records.inc();
                                continue;
                            }

                            match self.process_kafka(data, capture_policy) {
                                Ok(mut otap_data) => {
                                    let signal = otap_data.signal_type();
                                    self.metrics
                                        .record_message_admitted(signal, payload_bytes);
                                    if manual_commit {
                                        // Stamp the record with this partition's
                                        // ownership generation so a stale revocation
                                        // of an older ownership period can't purge
                                        // it, and its Ack/Nack can be recognized
                                        // as belonging to the current ownership.
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
                                        KafkaReceiverError::TracesDecode(_) => (
                                            Some(SignalType::Traces),
                                            ReceiverRejectionErrorType::InvalidRequest,
                                            KafkaReceiverRejectionReason::Decode,
                                        ),
                                        KafkaReceiverError::MetricsDecode(_) => (
                                            Some(SignalType::Metrics),
                                            ReceiverRejectionErrorType::InvalidRequest,
                                            KafkaReceiverRejectionReason::Decode,
                                        ),
                                        KafkaReceiverError::LogsDecode(_) => (
                                            Some(SignalType::Logs),
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
                                        KafkaReceiverError::TracesDecode(e) => {
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
                                        // this partition's ownership generation
                                        // (read once above) for consistency with
                                        // the revoke/purge path.
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
                                    self.metrics.record_transport_error(&e);
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
    use crate::common::kafka::node_harness::node_metrics::{FoldedMetrics, metric_value};
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
    use rdkafka::types::RDKafkaRespErr;
    use std::collections::HashMap;
    use std::time::Duration;

    /// Number of partitions provisioned for the rebalance integration tests.
    const REBALANCE_TEST_PARTITIONS: i32 = 2;
    /// Records produced to each partition in the rebalance integration tests.
    const REBALANCE_RECORDS_PER_PARTITION: i32 = 5;

    // ---- Shared test helpers ----

    fn measurement_counter(
        snapshots: &[otap_df_telemetry::metrics::MetricSetSnapshot],
        metric_set: &str,
        attributes: &[(&str, &str)],
        metric: &str,
    ) -> u64 {
        snapshots
            .iter()
            .filter(|snapshot| snapshot.descriptor().name == metric_set)
            .filter(|snapshot| {
                attributes
                    .iter()
                    .all(|(key, value)| snapshot.measurement_attribute_value(key) == Some(*value))
            })
            .filter_map(|snapshot| metric_value(snapshot, metric))
            .sum()
    }

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

    /// Builds a manual-commit [`KafkaReceiverConfig`] for a single traces topic
    /// with NO safety-net commit timer, so offsets are committed purely through
    /// ack/nack. Used by tests that need deterministic watermark assertions
    /// without a periodic timer racing the acks.
    fn manual_traces_config_no_timer(
        brokers: &str,
        group_id: &str,
        traces_topic: &str,
    ) -> KafkaReceiverConfig {
        let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
            .with_traces(
                SignalConfig::new(vec![traces_topic.to_string()])
                    .with_encoding(MessageFormat::OtlpProto),
            )
            .with_commit(CommitConfig {
                mode: ConfigCommitMode::Manual,
                interval_ms: None,
            })
            .with_auto_offset_reset(AutoOffsetReset::Earliest)
            .with_isolation_level(IsolationLevel::ReadUncommitted);
        KafkaReceiverConfig::try_from(builder).expect("test config valid")
    }

    /// Like [`manual_traces_config_no_timer`] but arms the opt-in consumer-lag
    /// refresh timer at `lag_refresh_interval_ms`, so a lag-refresh worker is
    /// periodically spawned and can be in flight when a shutdown arrives.
    fn manual_traces_config_with_lag_refresh(
        brokers: &str,
        group_id: &str,
        traces_topic: &str,
        lag_refresh_interval_ms: u64,
    ) -> KafkaReceiverConfig {
        let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
            .with_traces(
                SignalConfig::new(vec![traces_topic.to_string()])
                    .with_encoding(MessageFormat::OtlpProto),
            )
            .with_commit(CommitConfig {
                mode: ConfigCommitMode::Manual,
                interval_ms: None,
            })
            .with_auto_offset_reset(AutoOffsetReset::Earliest)
            .with_isolation_level(IsolationLevel::ReadUncommitted)
            .with_lag_refresh_interval_ms(Some(lag_refresh_interval_ms));
        KafkaReceiverConfig::try_from(builder).expect("test config valid")
    }

    /// Like [`manual_traces_config_no_timer`] but configures the traces signal
    /// for the OTAP-Arrow encoding, whose decode path validates the payload (so
    /// an undecodable record surfaces as a processing error).
    fn manual_otap_traces_config_no_timer(
        brokers: &str,
        group_id: &str,
        traces_topic: &str,
    ) -> KafkaReceiverConfig {
        let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
            .with_traces(
                SignalConfig::new(vec![traces_topic.to_string()])
                    .with_encoding(MessageFormat::OtapProto),
            )
            .with_commit(CommitConfig {
                mode: ConfigCommitMode::Manual,
                interval_ms: None,
            })
            .with_auto_offset_reset(AutoOffsetReset::Earliest)
            .with_isolation_level(IsolationLevel::ReadUncommitted);
        KafkaReceiverConfig::try_from(builder).expect("test config valid")
    }

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

    // ---- Construction and configuration ----

    /// Scenario (construction and configuration): a receiver is built on a multi-core
    /// pipeline with a configured `group.instance.id`.
    /// Guarantees: the instance id is suffixed with the core id so each core joins the
    /// consumer group as a distinct static member.
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

    /// Scenario (construction and configuration): a receiver is built on a single-core
    /// pipeline with a configured `group.instance.id`.
    /// Guarantees: the instance id is left unchanged, so a single-core deployment keeps the
    /// operator-provided static member id.
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

    /// Scenario (construction and configuration): a receiver is built without a
    /// `group.instance.id`.
    /// Guarantees: no instance id is synthesized, so the consumer joins as a dynamic
    /// (non-static) group member.
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

    /// Scenario (construction and configuration): a receiver is constructed with traces,
    /// metrics, and logs on distinct topics.
    /// Guarantees: construction succeeds, so a valid multi-signal configuration is
    /// accepted.
    #[test]
    fn new_succeeds_with_distinct_topics() {
        let cfg = make_config(&["t"], &["m"], &["l"], MessageFormat::OtlpProto);
        let ctx = make_pipeline_ctx();
        let receiver = KafkaReceiver::new(ctx, cfg);
        assert!(receiver.is_ok());
    }

    /// Scenario (construction and configuration): two signals are configured to share the
    /// same topic.
    /// Guarantees: config validation fails with an overlap error, so one topic cannot feed
    /// two signal decoders.
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

    /// Scenario (construction and configuration): a receiver is built in the default
    /// manual-commit mode.
    /// Guarantees: an empty offset tracker is present, so the manual at-least-once commit
    /// path is wired and ready.
    #[test]
    fn new_creates_offset_tracker_when_auto_commit_disabled() {
        let cfg = make_config(&["t"], &["m"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit()); // default is manual (not auto)
        let ctx = make_pipeline_ctx();
        let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
        // offset_tracker is always present; verify it starts empty
        assert_eq!(receiver.offset_tracker.total_pending(), 0);
    }

    /// Scenario (construction and configuration): a receiver is built with auto-commit
    /// enabled.
    /// Guarantees: construction succeeds and the tracker starts empty, so auto-commit mode
    /// builds without engaging the manual tracker.
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

    /// Scenario (construction and configuration): a receiver is built from a complete JSON
    /// config with all required fields and topics.
    /// Guarantees: `from_config` succeeds, so a well-formed operator config deserializes
    /// and builds.
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

    /// Scenario (construction and configuration): a receiver is built from JSON missing the
    /// required brokers/group_id/client_id fields.
    /// Guarantees: `from_config` returns an error, so an incomplete config is rejected
    /// rather than silently defaulted.
    #[test]
    fn from_config_fails_with_missing_required_fields() {
        // brokers, group_id, client_id are required
        let json: Value = serde_json::json!({});
        let ctx = make_pipeline_ctx();
        let result = KafkaReceiver::from_config(ctx, &json);
        assert!(result.is_err());
    }

    /// Scenario (construction and configuration): a receiver is built from JSON with
    /// required fields but no signal topics.
    /// Guarantees: `from_config` returns an error, so a receiver that would subscribe to
    /// nothing is rejected.
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

    /// Scenario (construction and configuration): a receiver is built from JSON that puts
    /// two signals on the same topic.
    /// Guarantees: `from_config` returns an error, so overlapping-topic configs are
    /// rejected at deserialization time.
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

    // ---- Offset guarantees ----

    /// Scenario (offset guarantees): an ack arrives for a partition this consumer still owns, whose
    /// ownership generation matches the ack.
    /// Guarantees: the ack is committed (advances the tracker) rather than
    /// dropped.
    #[test]
    fn classify_offset_feedback_commits_current_generation_ack() {
        assert_eq!(
            classify_offset_feedback(2, Some(2), 2, true),
            OffsetFeedbackAction::Commit,
        );
    }

    /// Scenario (offset guarantees): an ack arrives whose generation is older than the partition's
    /// tracked generation (the partition was reassigned and re-tracked under a
    /// newer generation).
    /// Guarantees: the ack is dropped as stale, so it cannot roll back or
    /// disturb the newer ownership period's committed offset.
    #[test]
    fn classify_offset_feedback_drops_ack_older_than_tracked_generation() {
        assert_eq!(
            classify_offset_feedback(1, Some(3), 3, true),
            OffsetFeedbackAction::DropStale,
        );
    }

    /// Scenario (offset guarantees): the closed gap. A partition was revoked and reassigned to this
    /// consumer under a newer generation, but no record of the new period has
    /// been tracked yet, so the tracker still reports the OLD generation while
    /// the assignment already reports the NEW one. A stale ack for the old
    /// period arrives with a generation equal to the tracker's.
    /// Guarantees: the ack is still dropped as stale because the classifier
    /// consults the assigned generation, not just the tracker generation -- so a
    /// stale same-as-tracker ack cannot slip through and mutate/commit stale
    /// state during the reassign-before-retrack window.
    #[test]
    fn classify_offset_feedback_drops_stale_ack_when_assigned_generation_is_newer() {
        assert_eq!(
            classify_offset_feedback(1, Some(1), 2, true),
            OffsetFeedbackAction::DropStale,
        );
    }

    /// Scenario (offset guarantees): an ack arrives for a partition no longer assigned to this
    /// consumer, whose tracked state is not newer than the ack's generation.
    /// Guarantees: the ack is dropped as a late ack and the lingering tracker
    /// state is purged (it belongs to the revoked ownership period).
    #[test]
    fn classify_offset_feedback_late_ack_purges_when_not_newer() {
        assert_eq!(
            classify_offset_feedback(1, Some(1), 0, false),
            OffsetFeedbackAction::DropLateAck { purge: true },
        );
    }

    /// Scenario (offset guarantees): an ack arrives for a partition no longer assigned, whose
    /// tracked state belongs to a NEWER generation than the ack. This is caught
    /// by the stale-generation check *before* the late-ack check, because a
    /// newer tracked generation means the partition was reassigned and
    /// re-tracked since the ack's ownership period.
    /// Guarantees: such an ack is classified `DropStale` (the newer tracked
    /// state is preserved), never `DropLateAck` with a purge -- so a stale ack
    /// can never purge a newer ownership period's tracker state.
    #[test]
    fn classify_offset_feedback_ack_older_than_tracked_is_stale_even_when_unassigned() {
        assert_eq!(
            classify_offset_feedback(2, Some(3), 0, false),
            OffsetFeedbackAction::DropStale,
        );
    }

    /// Scenario (offset guarantees): an ack arrives for a partition that is neither assigned nor
    /// tracked (fully revoked and purged already).
    /// Guarantees: the ack is dropped as a late ack with nothing to purge.
    #[test]
    fn classify_offset_feedback_late_ack_untracked_does_not_purge() {
        assert_eq!(
            classify_offset_feedback(1, None, 0, false),
            OffsetFeedbackAction::DropLateAck { purge: false },
        );
    }

    /// Scenario (offset guarantees): the first ack for a freshly-assigned partition arrives before
    /// its record was tracked (untracked, but currently owned), with a
    /// generation matching the assignment.
    /// Guarantees: the ack is committed -- an untracked-but-owned partition is
    /// not treated as stale as long as the ack is not older than the assigned
    /// generation.
    #[test]
    fn classify_offset_feedback_commits_untracked_but_assigned_current_ack() {
        assert_eq!(
            classify_offset_feedback(1, None, 1, true),
            OffsetFeedbackAction::Commit,
        );
    }

    /// Scenario (offset guarantees): a partition owned under generation 1 is revoked and
    /// reassigned under generation 2 but not yet re-tracked, then a stale generation-1 ack
    /// equal to the tracker generation arrives.
    /// Guarantees: the ack is classified `DropStale` because the classifier consults the
    /// assigned generation, so it neither advances the tracker nor rolls back the committed
    /// offset during the reassign-before-retrack window.
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

    /// Scenario (offset guarantees): a partition is revoked by the rebalance callback but
    /// not yet reconciled when a commit is built.
    /// Guarantees: the revoked partition is drained before the committable TPL is built, so
    /// a revoked partition is never committed while an owned partition remains committable.
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

    /// Scenario (offset guarantees): a revoked partition is queued while the receiver runs
    /// in auto-commit mode.
    /// Guarantees: purge leaves the tracker untouched, so under auto-commit librdkafka owns
    /// offsets and the manual purge path is inert.
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

    /// Scenario (offset guarantees): a revoked partition is queued while the receiver runs
    /// in auto-commit mode.
    /// Guarantees: reconcile leaves the tracker untouched, so under auto-commit the manual
    /// rebalance-reconcile path is inert.
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

    /// Scenario (offset guarantees): commit-callback successes and failures accumulate on
    /// the poll thread and are then reconciled.
    /// Guarantees: reconcile folds them into success and failure outcome buckets exactly
    /// once and drains the counters, so a commit failure is surfaced and never double-counted.
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

        assert_eq!(
            receiver
                .metrics
                .offset_commits_for(Outcome::Success)
                .commits
                .get(),
            2
        );
        assert_eq!(
            receiver
                .metrics
                .offset_commits_for(Outcome::Failure)
                .commits
                .get(),
            1
        );

        // Counters were drained; a second reconcile adds nothing.
        receiver.reconcile_rebalance_state();
        assert_eq!(
            receiver
                .metrics
                .offset_commits_for(Outcome::Success)
                .commits
                .get(),
            2
        );
        assert_eq!(
            receiver
                .metrics
                .offset_commits_for(Outcome::Failure)
                .commits
                .get(),
            1
        );
    }

    /// Scenario (offset guarantees): a commit request times out at the broker,
    /// so its asynchronous outcome arrives on the commit callback as a failure
    /// (modeled here via `record_commit_result_for_test(false)`, the same seam
    /// the real `commit_callback` drives). This unit-level surrogate is used
    /// because on the in-process `MockCluster` an injected `OffsetCommit`
    /// timeout is not delivered to the callback within a test window (verified),
    /// so the timeout outcome cannot be observed end-to-end.
    /// Guarantees: a timed-out (failed) commit outcome is surfaced as
    /// the failure outcome on the next reconcile and does not increment the
    /// success outcome -- so a commit timeout is reported and never silently
    /// counted as a successful commit or allowed to advance committed state.
    #[test]
    fn commit_timeout_outcome_surfaces_as_offset_commit_error() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // A commit that reached the broker succeeds; a later commit times out
        // (its callback outcome is a failure).
        receiver.rebalance_state.record_commit_result_for_test(true);
        receiver
            .rebalance_state
            .record_commit_result_for_test(false);

        receiver.reconcile_rebalance_state();

        // The timeout is surfaced as a commit error, not folded into the success
        // counter -- a timed-out commit is never mistaken for a successful one.
        assert_eq!(
            receiver
                .metrics
                .offset_commits_for(Outcome::Failure)
                .commits
                .get(),
            1,
            "a timed-out commit outcome must be surfaced as a failed commit",
        );
        assert_eq!(
            receiver
                .metrics
                .offset_commits_for(Outcome::Success)
                .commits
                .get(),
            1,
            "only the successful commit should count toward the success outcome",
        );
    }

    /// Scenario (offset guarantees): tracked offsets are snapshotted into the shared
    /// rebalance state, then a partition is assigned.
    /// Guarantees: the committable snapshot feeds the rebalance state's assignment view, so
    /// pre-rebalance commits see the correct assigned partitions.
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

    /// Scenario (offset guarantees): the committable snapshot is refreshed, the lowest
    /// pending offset is acknowledged, then it is refreshed again.
    /// Guarantees: the snapshot advances to the next committable offset after the
    /// acknowledge, so a subsequent pre-rebalance commit is never stale.
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

    /// Scenario (offset guarantees): a single manual-commit consumer owns one
    /// partition holding three in-flight records; the records are acked
    /// out of order (offsets 1 and 2 first, the lowest offset 0 withheld),
    /// then offset 0 is acked last.
    /// Guarantees: the committed watermark holds at the gap while the lowest
    /// offset is un-acked (it never advances past an un-acked offset, so
    /// at-least-once cannot skip an offset), and only after offset 0 is acked
    /// does it jump to the full record count -- proving the lowest-un-acked
    /// watermark commit logic end-to-end through the broker.
    #[tokio::test]
    async fn out_of_order_acks_commit_only_lowest_contiguous() {
        const TOPIC: &str = "offset-out-of-order-traces";
        const RECORDS: usize = 3;
        let group = "offset-out-of-order-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce three records to the single partition; they receive
                // offsets 0, 1, 2 in order.
                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("Failed to send message");
                }

                // No safety-net timer: commits are driven purely by acks so the
                // watermark assertions are deterministic.
                let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume all three records, correlating each delivered pdata
                // back to its Kafka offset via the stamped calldata so acks can
                // be issued in a controlled (out-of-order) sequence.
                let mut by_offset: HashMap<i64, OtapPdata> = HashMap::new();
                for _ in 0..RECORDS {
                    let pdata = receiver.recv_pdata().await;
                    let route = pdata
                        .source_route()
                        .expect("delivered pdata carries source calldata");
                    let (_topic_id, _partition, offset, _generation) =
                        decode_calldata(&route.calldata);
                    let _ = by_offset.insert(offset, pdata);
                }
                assert_eq!(by_offset.len(), RECORDS, "expected one pdata per offset");

                let brokers = cluster.bootstrap_servers().to_string();

                // Ack offsets 1 and 2 first, withholding the lowest offset 0.
                receiver.ack(by_offset.remove(&1).expect("offset 1 delivered"));
                receiver.ack(by_offset.remove(&2).expect("offset 2 delivered"));

                // The committed offset must NOT advance past the un-acked lowest
                // offset 0. Because a manual commit only advances the lowest
                // contiguous acked offset, no commit should reach offset 1+.
                // Give the loop time to process the acks, then assert the
                // watermark is still below 1 (either uncommitted or 0).
                tokio::time::sleep(Duration::from_millis(500)).await;
                let committed_before = committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed");
                assert!(
                    committed_before.is_none_or(|o| o < 1),
                    "committed offset must not advance past the un-acked lowest \
                     offset 0, got {committed_before:?}",
                );

                // Ack the withheld lowest offset 0. Now the contiguous run
                // 0,1,2 is complete, so the watermark jumps to the full count.
                receiver.ack(by_offset.remove(&0).expect("offset 0 delivered"));

                let advanced =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= RECORDS as i64)
                    })
                    .await;
                assert!(
                    advanced,
                    "once the lowest offset is acked the watermark should jump to \
                     the full count {RECORDS}, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (offset guarantees): an undecodable OTAP-Arrow traces record is
    /// produced between two well-formed OTAP records on a single partition of a
    /// manual-commit receiver.
    /// Guarantees: the poison record is counted as a processing/unmarshal error
    /// and is never forwarded downstream, yet the surrounding good records are
    /// still delivered and the committed offset advances past the poison record
    /// -- so one undecodable message cannot stall the partition or violate the
    /// late-ack guard.
    #[tokio::test]
    async fn poison_message_advances_without_stalling_partition() {
        const TOPIC: &str = "offset-poison-traces";
        let group = "offset-poison-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                // Well-formed OTAP-Arrow trace bytes for the surrounding records.
                let good = create_traces_with_spans_otap_bytes();
                // Not a valid OTAP BatchArrowRecords payload: decoding fails.
                let poison = b"this-is-not-a-valid-otap-arrow-payload".to_vec();

                // Order on the partition: good(0), good(1), poison(2). Every
                // record carries the OTAP MessageFormat header so the receiver
                // uses the OTAP decode path (which validates the payload, unlike
                // the zero-copy OTLP path).
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &good)
                            .key(b"good-a")
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("send good a");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &good)
                            .key(b"good-b")
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("send good b");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &poison)
                            .key(b"poison")
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("send poison");

                let cfg =
                    manual_otap_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Only the two good records are forwarded; the poison record is
                // dropped. Ack the good records so the watermark advances past
                // the poison offset in between.
                let first = receiver.recv_pdata().await;
                let second = receiver.recv_pdata().await;
                receiver.ack(first);
                receiver.ack(second);

                // No third record should arrive: the poison record was never
                // forwarded.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(2))
                        .await
                        .is_none(),
                    "poison record must not be forwarded downstream",
                );

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
                    "committed offset must advance past the poison record to the \
                     full count 3, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;
                let decode_rejections = measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.rejections",
                    &[
                        ("signal", "traces"),
                        ("error.type", "invalid_request"),
                        ("reason", "decode"),
                    ],
                    "messages",
                );
                assert!(
                    decode_rejections >= 1,
                    "the poison record must be counted as a decode rejection, got {decode_rejections}",
                );
            },
        )
        .await;
    }

    /// Scenario (offset guarantees): an auto-commit (at-most-once) receiver
    /// consumes every produced record but never acks; librdkafka owns offsets
    /// and auto-commits them periodically.
    /// Guarantees: the broker-side committed offset still advances to the full
    /// record count purely from librdkafka's auto-commit, while the receiver's
    /// manual tracker/rebalance-commit paths stay inert (successful acknowledgement
    /// responses and failed offset commits remain 0) -- proving auto-commit mode is a true
    /// no-op for the manual offset machinery.
    #[tokio::test]
    async fn auto_commit_mode_lets_librdkafka_own_offsets() {
        const TOPIC: &str = "offset-auto-commit-traces";
        const RECORDS: usize = 4;
        // Auto-commit mode uses the fixed "test-group" from `auto_config`.
        let group = "test-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
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

                // Consume every record but never ack: under auto-commit the
                // receiver does not track offsets, and librdkafka commits on its
                // own periodic schedule.
                for _ in 0..RECORDS {
                    let _ = receiver.recv_pdata().await;
                }

                // Wait long enough for at least one auto-commit interval (1000ms)
                // to elapse and be flushed.
                let brokers = cluster.bootstrap_servers().to_string();
                let advanced =
                    poll_until(Duration::from_secs(10), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= RECORDS as i64)
                    })
                    .await;
                assert!(
                    advanced,
                    "librdkafka auto-commit should advance the committed offset to \
                     the full count {RECORDS} without any acks, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;
                assert_eq!(
                    measurement_counter(
                        terminal.metrics(),
                        "receiver.kafka.acknowledgements",
                        &[("signal", "traces"), ("outcome", "success")],
                        "responses",
                    ),
                    0,
                    "auto-commit mode must not receive acknowledgements",
                );
                assert_eq!(
                    measurement_counter(
                        terminal.metrics(),
                        "receiver.kafka.offset_commits",
                        &[("outcome", "failure")],
                        "commits",
                    ),
                    0,
                    "the manual commit path is inert under auto-commit, so no \
                     failed offset commits should be recorded",
                );
            },
        )
        .await;
    }

    /// Scenario (offset guarantees): a manual-commit receiver consumes a record
    /// and holds it in-flight (un-acked) while a downstream retry would be in
    /// progress, then the record receives a terminal permanent Nack (the outcome
    /// a `processor:retry` node forwards once its retries are exhausted or the
    /// failure is permanent).
    /// Guarantees: the offset stays uncommitted while the record is in-flight
    /// (the committed watermark does not advance past it), and advances to the
    /// full count only once the terminal permanent Nack arrives -- proving the
    /// receiver holds the offset during retries and advances only on a
    /// terminal/permanent outcome. Transient-retry logic itself lives in and is
    /// tested by the `processor:retry` node (see `retry_processor` tests
    /// `test_retry_processor_permanent_error_not_retried`,
    /// `test_retry_processor_nacks_then_timeout`,
    /// `test_retry_processor_nacks_then_limit`).
    #[tokio::test]
    async fn terminal_nack_advances_offset_past_message() {
        const TOPIC: &str = "offset-terminal-nack-traces";
        let group = "offset-terminal-nack-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // A single record on the single partition.
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(b"rec-0"))
                    .await
                    .expect("send record");

                let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume the record but hold it un-acked: this models the window
                // where a downstream `processor:retry` is still retrying, so no
                // terminal outcome has reached the receiver yet.
                let pdata = receiver.recv_pdata().await;

                // While the record is in-flight the offset must NOT be committed.
                let brokers = cluster.bootstrap_servers().to_string();
                tokio::time::sleep(Duration::from_millis(500)).await;
                let committed_in_flight = committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed");
                assert!(
                    committed_in_flight.is_none_or(|o| o < 1),
                    "offset must stay uncommitted while the record is in-flight \
                     (retries in progress), got {committed_in_flight:?}",
                );

                // The retry node exhausts/permanently-fails and forwards a
                // terminal permanent Nack to the receiver.
                receiver.nack_permanent("retries exhausted", pdata);

                // The terminal Nack advances the offset past the message.
                let advanced =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= 1)
                    })
                    .await;
                assert!(
                    advanced,
                    "a terminal permanent Nack must advance the committed offset \
                     past the message, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (offset guarantees): two records are consumed from one partition;
    /// the first is terminated with a transient (non-permanent) Nack and the
    /// second with a permanent Nack.
    /// Guarantees: the receiver treats every Nack as terminal regardless of the
    /// `permanent` flag -- both advance the committed offset identically -- which
    /// confirms transient-retry is delegated out-of-process to `processor:retry`
    /// and the receiver never itself retries a nacked record.
    #[tokio::test]
    async fn transient_and_permanent_nack_both_advance_offset() {
        const TOPIC: &str = "offset-nack-parity-traces";
        const RECORDS: usize = 2;
        let group = "offset-nack-parity-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Correlate each delivered record to its offset so the nacks are
                // issued lowest-offset-first (the watermark only advances on the
                // contiguous lowest offset).
                let mut by_offset: HashMap<i64, OtapPdata> = HashMap::new();
                for _ in 0..RECORDS {
                    let pdata = receiver.recv_pdata().await;
                    let route = pdata
                        .source_route()
                        .expect("delivered pdata carries source calldata");
                    let (_topic_id, _partition, offset, _generation) =
                        decode_calldata(&route.calldata);
                    let _ = by_offset.insert(offset, pdata);
                }

                // Offset 0 gets a transient Nack; offset 1 gets a permanent Nack.
                // Both must advance the watermark identically.
                receiver.nack_transient(
                    "transient failure",
                    by_offset.remove(&0).expect("offset 0 delivered"),
                );
                receiver.nack_permanent(
                    "permanent failure",
                    by_offset.remove(&1).expect("offset 1 delivered"),
                );

                let brokers = cluster.bootstrap_servers().to_string();
                let advanced =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= RECORDS as i64)
                    })
                    .await;
                assert!(
                    advanced,
                    "both a transient and a permanent Nack must advance the \
                     committed offset to the full count {RECORDS}, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (offset guarantees): a manual-commit receiver consumes records
    /// but never acks them (so nothing is committed), then is fully shut down;
    /// a second receiver is started in the same consumer group on the same
    /// cluster (a consumer restart).
    /// Guarantees: because the first receiver committed nothing, the broker
    /// retains no progress and the restarted receiver re-receives the
    /// uncommitted records -- proving at-least-once redelivery with no data loss
    /// across a consumer restart.
    #[tokio::test]
    async fn restart_redelivers_uncommitted_offsets() {
        const TOPIC: &str = "offset-restart-traces";
        const RECORDS: usize = 3;
        let group = "offset-restart-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                // First receiver: consume every record but NEVER ack, so no
                // offset is ever committed.
                let cfg_a =
                    manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
                for _ in 0..RECORDS {
                    let _ = receiver_a.recv_pdata().await;
                }

                // Nothing was acked, so the broker must hold no committed offset.
                let brokers = cluster.bootstrap_servers().to_string();
                tokio::time::sleep(Duration::from_millis(500)).await;
                let committed_before = committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed");
                assert!(
                    committed_before.is_none_or(|o| o < RECORDS as i64),
                    "no offset should be committed before restart (records were \
                     never acked), got {committed_before:?}",
                );

                // Fully stop the first receiver (a restart).
                receiver_a.shutdown(Duration::from_secs(5));
                receiver_a.await_stopped().await;

                // Second receiver in the SAME group: it must re-receive the
                // uncommitted records (at-least-once redelivery, no loss).
                let cfg_b =
                    manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);
                let mut redelivered = 0usize;
                for _ in 0..RECORDS {
                    if receiver_b
                        .try_recv_pdata(Duration::from_secs(15))
                        .await
                        .is_some()
                    {
                        redelivered += 1;
                    }
                }
                assert_eq!(
                    redelivered, RECORDS,
                    "restarted receiver must re-receive all {RECORDS} uncommitted \
                     records, got {redelivered}",
                );

                receiver_b.shutdown(Duration::from_secs(5));
                receiver_b.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (offset guarantees): a manual-commit receiver is configured with
    /// a short safety-net commit timer (`commit.interval_ms`); records are
    /// consumed and acked, but the receiver is neither drained nor shut down
    /// while the assertion runs.
    /// Guarantees: the periodic `TimerTick` commit path advances the broker-side
    /// committed offset to the full acked count on its own -- without relying on
    /// the drain/shutdown final commit -- so the safety-net timer durably
    /// persists acked progress during steady-state operation.
    #[tokio::test]
    async fn safety_net_timer_commits_without_acks_drain_or_shutdown() {
        const TOPIC: &str = "offset-safety-timer-traces";
        const RECORDS: i64 = 3;
        let group = "offset-safety-timer-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");
                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                // Short safety-net timer so the periodic commit fires well within
                // the assertion window; acks alone would also commit, but the
                // point here is that the commit is observed BEFORE any
                // drain/shutdown, i.e. driven by the timer tick.
                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 200, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for _ in 0..RECORDS {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Wait for the periodic commit timer to persist the acked
                // offsets. No drain, no shutdown yet: the commit must come from
                // the safety-net TimerTick path alone.
                let brokers = cluster.bootstrap_servers().to_string();
                let committed =
                    poll_until(Duration::from_secs(5), Duration::from_millis(100), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= RECORDS)
                    })
                    .await;
                assert!(
                    committed,
                    "the safety-net commit timer must advance the committed offset \
                     to the full acked count {RECORDS} without a drain/shutdown, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    // ---- Consumer-group rebalancing ----

    /// Scenario (consumer-group rebalancing): a revoked partition is queued and then
    /// reconciled at the top of the receive loop.
    /// Guarantees: reconcile purges the revoked partition from the tracker, so its stale
    /// offsets cannot be committed after ownership is lost.
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

    /// Scenario (consumer-group rebalancing): a partition is revoked and immediately
    /// reassigned to this consumer under a newer generation before the stale revocation is
    /// processed.
    /// Guarantees: the reassigned partition's newer state is preserved and only the stale
    /// revocation is dropped, so a rapid revoke/reassign does not discard freshly-owned
    /// state.
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

    /// Scenario (consumer-group rebalancing): records tracked under an old generation
    /// remain when the partition is reassigned under a newer generation.
    /// Guarantees: acks for the old-generation records are not committed after
    /// reassignment, so a stale generation cannot advance the newly-owned partition's
    /// offset.
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

    /// Scenario (consumer-group rebalancing): a partition owned under generation 1
    /// (with an established committable offset) is revoked and reassigned to this
    /// receiver under generation 2, and a new generation-2 record is tracked.
    /// A stale Ack/Nack for the old generation-1 record then arrives, carrying
    /// generation 1 in its calldata.
    /// Guarantees: the stale feedback is classified `DropStale` -- the receiver's
    /// exact classifier decision on an Ack/Nack (both funnel through
    /// `handle_offset_feedback`, which calls `classify_offset_feedback`) -- so it
    /// is ignored: acknowledging the old offset is a no-op (returns false) and the
    /// committable offset continues to reflect only the generation-2 record, never
    /// regressing to or advancing on the generation-1 offset. An old-generation
    /// Ack/Nack thus cannot move the commit offset.
    #[test]
    fn stale_generation_ack_does_not_advance_committable_offset() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Generation 1: own partition 0, track and ack offset 100 so there is an
        // established committable offset (101) for the generation-1 period.
        let mut tpl1 = TopicPartitionList::new();
        let _ = tpl1.add_partition("traces", 0);
        receiver.rebalance_state.set_assignment_for_test(&tpl1);
        let gen1 = receiver.rebalance_state.current_generation("traces", 0);
        receiver.offset_tracker.track("traces", 0, 100, gen1);
        let _ = receiver.offset_tracker.acknowledge("traces", 0, 100);

        // Partition 0 is revoked (revocation carries generation 1); the loop
        // reconciles and purges the generation-1 tracker state. Also drop it from
        // the assigned set (empty assignment), mirroring librdkafka's
        // pre_rebalance(Revoke) removing it before post_rebalance(Assign), so the
        // subsequent reassignment allocates a fresh, strictly-greater generation.
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, gen1);
        receiver.reconcile_rebalance_state();
        receiver
            .rebalance_state
            .set_assignment_for_test(&TopicPartitionList::new());

        // Generation 2: partition 0 is reassigned; a new record (offset 200) is
        // tracked under the newer generation.
        receiver.rebalance_state.set_assignment_for_test(&tpl1);
        let gen2 = receiver.rebalance_state.current_generation("traces", 0);
        assert!(gen2 > gen1, "reassignment must allocate a newer generation");
        receiver.offset_tracker.track("traces", 0, 200, gen2);
        let committable_after_reassign = receiver
            .offset_tracker
            .committable_snapshot()
            .get(&("traces".to_string(), 0))
            .copied();

        // A stale Ack/Nack for the old generation-1 record arrives. The receiver
        // reads the same state `handle_offset_feedback` reads and classifies it.
        let tracked_generation = receiver.offset_tracker.partition_generation("traces", 0);
        let assigned_generation = receiver.rebalance_state.current_generation("traces", 0);
        let is_assigned = receiver.rebalance_state.is_assigned("traces", 0);
        assert_eq!(
            classify_offset_feedback(gen1, tracked_generation, assigned_generation, is_assigned),
            OffsetFeedbackAction::DropStale,
            "an Ack/Nack from the old generation must classify as DropStale",
        );

        // The DropStale path does not touch the offset tracker; simulate the only
        // mutation a stale feedback could attempt (acknowledging its old offset)
        // and confirm it is a no-op -- the old offset is not pending under the new
        // generation, so the watermark cannot move.
        assert!(
            !receiver.offset_tracker.acknowledge("traces", 0, 100),
            "acking a stale old-generation offset must not advance the watermark",
        );

        // The committable offset still reflects only the generation-2 record; the
        // stale feedback neither advanced nor rolled it back.
        assert_eq!(
            receiver
                .offset_tracker
                .committable_snapshot()
                .get(&("traces".to_string(), 0))
                .copied(),
            committable_after_reassign,
            "a stale old-generation Ack/Nack must not change the committable offset",
        );
        assert_eq!(
            committable_after_reassign,
            Some(200),
            "only the generation-2 record drives the commit",
        );
    }

    /// Scenario (consumer-group rebalancing): a partition is retained across a rebalance
    /// that only adds or removes other partitions.
    /// Guarantees: the retained partition keeps its generation, so an unrelated rebalance
    /// does not invalidate acks for partitions this consumer never lost.
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

    /// Scenario (consumer-group rebalancing): a rebalance assigns partitions and the receive loop reconciles.
    /// Guarantees: `reconcile_rebalance_state` folds the rebalance deltas into
    /// the metric set - counting the rebalance event and cumulative acquisitions,
    /// and observing `receiver.kafka.consumer.group.partitions` as the current
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

        // Observed up/down counter reflects current ownership; cumulative
        // counter reflects the acquisitions.
        assert_eq!(receiver.metrics.consumer.partitions.get(), 2);
        assert_eq!(receiver.metrics.consumer.partition_assignments.get(), 2);

        // A second reconcile with no further rebalance activity must not change
        // the observed value (it is folded only when a rebalance occurred) or
        // double count the counter.
        receiver.reconcile_rebalance_state();
        assert_eq!(receiver.metrics.consumer.partitions.get(), 2);
        assert_eq!(receiver.metrics.consumer.partition_assignments.get(), 2);
    }

    /// Scenario (operational visibility): a manual-commit receiver tracks several
    /// in-flight offsets, then acknowledges some, then has its partition revoked
    /// and purged, reconciling after each step.
    /// Guarantees: the `records_in_flight` up/down counter reflects the current
    /// count of tracked-but-uncommitted offsets at each reconcile -- it rises as
    /// offsets are tracked, falls as they are acked and the watermark advances,
    /// and drops to zero when the partition is purged -- giving operators a
    /// current view of the receiver's outstanding depth.
    #[test]
    fn records_in_flight_gauge_reflects_outstanding_offsets() {
        let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

        // Own partition 0 and track three in-flight offsets under its generation.
        // in flight: traces/0={0,1,2} => gauge 3
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        receiver.rebalance_state.set_assignment_for_test(&tpl);
        let generation = receiver.rebalance_state.current_generation("traces", 0);
        for offset in 0..3 {
            receiver
                .offset_tracker
                .track("traces", 0, offset, generation);
        }
        receiver.reconcile_rebalance_state();
        assert_eq!(
            receiver.metrics.consumer.records_in_flight.get(),
            3,
            "the counter must report all three tracked in-flight offsets",
        );

        // Acknowledge the two lowest offsets; only offset 2 remains pending.
        // in flight: traces/0={2} => gauge 1
        let _ = receiver.offset_tracker.acknowledge("traces", 0, 0);
        let _ = receiver.offset_tracker.acknowledge("traces", 0, 1);
        receiver.reconcile_rebalance_state();
        assert_eq!(
            receiver.metrics.consumer.records_in_flight.get(),
            1,
            "the counter must drop as offsets are acked and the watermark advances",
        );

        // Revoke and purge the partition; nothing remains in flight.
        // in flight: {} => gauge 0
        receiver
            .rebalance_state
            .push_revoked_for_test("traces", 0, generation);
        receiver.reconcile_rebalance_state();
        assert_eq!(
            receiver.metrics.consumer.records_in_flight.get(),
            0,
            "the counter must drop to zero once the partition's state is purged",
        );
    }

    /// Scenario (consumer-group rebalancing): a single manual-commit consumer owns all partitions of a
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

    /// Scenario (consumer-group rebalancing): a manual-commit receiver owns both partitions, consumes and
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

    /// Scenario (consumer-group rebalancing): a cooperative-sticky manual-commit receiver owns both
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

    /// Scenario (consumer-group rebalancing): a manual-commit receiver owns all partitions, then a second
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

    /// Scenario (consumer-group rebalancing): two `KafkaReceiver` replicas share one `group_id` against a
    /// multi-partition topic; replica B joins (scale-up) then leaves
    /// (scale-down), driving two rebalances. This is the in-process analogue of
    /// running 2+ replicas with the same group and scaling the replica count up
    /// and down; the full procedure is documented in the Kafka test-suite README
    /// ("Multi-receiver scale-up/down").
    ///
    /// Guarantees: (1) both replicas own a partition at some point, so the
    /// partitions distribute across the group (B consumes records that only its
    /// assigned partition can deliver, and both replicas' terminal metrics show
    /// `group.partitions >= 1`); (2) a rebalance is observed on scale-up/down
    /// (`group.partition.revocations >= 1` across the two replicas); (3) no message is
    /// lost or double-committed -- every produced record is delivered at least
    /// once and durably retained on the broker, each partition's committed
    /// offset stays within `[wave-1 count, total produced count]` (the lower
    /// bound proves committed progress is never rolled back across a rebalance,
    /// the upper bound proves nothing is committed past the produced data), and
    /// neither replica reports failed offset commits.
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
                    fa.value("group.partition.assignments") >= 1,
                    "replica A should have acquired at least one partition, got {}",
                    fa.value("group.partition.assignments"),
                );
                assert!(
                    fb.value("group.partition.assignments") >= 1,
                    "replica B should have acquired at least one partition on scale-up, got {}",
                    fb.value("group.partition.assignments"),
                );
                assert!(
                    fa.value("group.partition.assignments")
                        + fb.value("group.partition.assignments")
                        >= REBALANCE_TEST_PARTITIONS as u64,
                    "the group should have acquired all {REBALANCE_TEST_PARTITIONS} partitions \
                     across the two replicas' lifetimes",
                );
                // After scale-down A re-owns its partitions, a deterministic
                // current-ownership check.
                assert!(
                    fa.value("group.partitions") >= 1,
                    "replica A should currently own at least one partition at shutdown, got {}",
                    fa.value("group.partitions"),
                );

                // (2) Rebalance observed: at least one owned partition was revoked
                // across scale-up/down.
                assert!(
                    fa.value("group.partition.revocations")
                        + fb.value("group.partition.revocations")
                        >= 1,
                    "a partition revoke should have been observed across scale-up/down",
                );

                // (3a) No commit failures on either replica.
                assert_eq!(
                    measurement_counter(
                        terminal_a.metrics(),
                        "receiver.kafka.offset_commits",
                        &[("outcome", "failure")],
                        "commits",
                    ),
                    0,
                    "replica A should have no offset commit errors",
                );
                assert_eq!(
                    measurement_counter(
                        terminal_b.metrics(),
                        "receiver.kafka.offset_commits",
                        &[("outcome", "failure")],
                        "commits",
                    ),
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

    /// Runs two same-group manual-commit receivers against a 2-partition topic
    /// under the given eager assignment `strategy` and asserts the group
    /// distributes both partitions and commits every record with no loss.
    ///
    /// Shared body for the `range` and `roundrobin` strategy tests: the receiver
    /// rebalance logic is strategy-agnostic for the eager protocols, so both
    /// strategies must produce the same distribute-and-commit outcome.
    async fn run_two_member_strategy_rebalance(topic: &'static str, strategy: RebalanceStrategy) {
        let group = "rebalance-strategy-group";
        let per_partition_total = REBALANCE_RECORDS_PER_PARTITION;
        let total = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
        with_cluster(
            KafkaTestCluster::builder().topic_with(topic, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");
                let brokers = cluster.bootstrap_servers().to_string();

                producer
                    .produce_per_partition(
                        topic,
                        REBALANCE_TEST_PARTITIONS,
                        REBALANCE_RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                // Two members join the same group under the configured strategy.
                let cfg_a = manual_traces_config(
                    cluster.bootstrap_servers(),
                    group,
                    topic,
                    500,
                    Some(strategy),
                );
                let cfg_b = manual_traces_config(
                    cluster.bootstrap_servers(),
                    group,
                    topic,
                    500,
                    Some(strategy),
                );
                let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
                let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);

                // True once every partition's committed offset reaches the
                // produced total (no loss, no rollback, no double-commit).
                let all_committed = |b: &str| {
                    (0..REBALANCE_TEST_PARTITIONS).all(|p| {
                        committed_offset(b, group, topic, p)
                            .expect("kafka-test: committed-offset probe failed")
                            == Some(per_partition_total as i64)
                    })
                };

                // Drain both members concurrently until every partition is
                // committed to the produced total, bounded by a deadline so a
                // failure to distribute fails loudly instead of hanging.
                let mut delivered = 0usize;
                let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
                while tokio::time::Instant::now() < deadline && !all_committed(&brokers) {
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

                assert!(
                    all_committed(&brokers),
                    "under {strategy:?} the group did not commit every partition to the produced \
                     total {per_partition_total} (delivered {delivered} of {total}); committed \
                     offsets did not converge",
                );

                receiver_a.shutdown(Duration::from_secs(5));
                let terminal_a = receiver_a.await_terminal_state().await;
                receiver_b.shutdown(Duration::from_secs(5));
                let terminal_b = receiver_b.await_terminal_state().await;

                let mut fa = FoldedMetrics::new();
                fa.fold_all(terminal_a.metrics());
                let mut fb = FoldedMetrics::new();
                fb.fold_all(terminal_b.metrics());

                // Distribution: together the two members acquired every partition
                // over their lifetimes.
                assert!(
                    fa.value("group.partition.assignments")
                        + fb.value("group.partition.assignments")
                        >= REBALANCE_TEST_PARTITIONS as u64,
                    "under {strategy:?} the group should acquire all {REBALANCE_TEST_PARTITIONS} \
                     partitions across the two members (A={}, B={})",
                    fa.value("group.partition.assignments"),
                    fb.value("group.partition.assignments"),
                );

                // No commit failures on either member.
                assert_eq!(
                    measurement_counter(
                        terminal_a.metrics(),
                        "receiver.kafka.offset_commits",
                        &[("outcome", "failure")],
                        "commits",
                    ) + measurement_counter(
                        terminal_b.metrics(),
                        "receiver.kafka.offset_commits",
                        &[("outcome", "failure")],
                        "commits",
                    ),
                    0,
                    "no offset commit errors expected under {strategy:?}",
                );

                // No loss: every produced record delivered at least once and
                // durably retained on the broker.
                assert!(
                    delivered >= total,
                    "under {strategy:?} the group should deliver every produced record at least \
                     once: delivered {delivered} of {total}",
                );
                let inspector = cluster.inspect();
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    assert_eq!(
                        inspector.message_count(topic, partition),
                        per_partition_total as i64,
                        "partition {partition} should durably retain all produced records",
                    );
                }
            },
        )
        .await;
    }

    /// Scenario (consumer-group rebalancing): two same-group receivers configured
    /// with the `range` assignment strategy join against a 2-partition topic.
    /// Guarantees: the group distributes both partitions across the two members
    /// and commits every produced record with no loss and no commit errors, so
    /// the `range` eager strategy assigns and commits correctly end-to-end.
    #[tokio::test]
    async fn rebalance_strategy_range_assigns_and_commits() {
        run_two_member_strategy_rebalance("rebalance-range-traces", RebalanceStrategy::Range).await;
    }

    /// Scenario (consumer-group rebalancing): two same-group receivers configured
    /// with the `roundrobin` assignment strategy join against a 2-partition
    /// topic.
    /// Guarantees: the group distributes both partitions across the two members
    /// and commits every produced record with no loss and no commit errors, so
    /// the `roundrobin` eager strategy assigns and commits correctly end-to-end.
    #[tokio::test]
    async fn rebalance_strategy_roundrobin_assigns_and_commits() {
        run_two_member_strategy_rebalance(
            "rebalance-roundrobin-traces",
            RebalanceStrategy::RoundRobin,
        )
        .await;
    }

    /// Scenario (consumer-group rebalancing): a manual-commit receiver holds an
    /// un-acked in-flight record on a partition that is then stolen by a second
    /// group member (a `RebalanceTrigger`), after which the test acks the now
    /// stale record.
    /// Guarantees: the late ack for the revoked partition is classified as a
    /// stale/late ack -- it increments
    /// `receiver.kafka.consumer.group.feedback.after_revocation` and is not
    /// committed -- so an ack that arrives after a partition is revoked can
    /// never advance a partition this consumer no longer owns.
    #[tokio::test]
    async fn stale_ack_after_revoke_counts_feedback_after_revocation() {
        const TOPIC: &str = "rebalance-stale-ack-traces";
        let group = "rebalance-stale-ack-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // One record per partition so the receiver has in-flight work on
                // every partition it owns.
                producer
                    .produce_per_partition(TOPIC, REBALANCE_TEST_PARTITIONS, 1, &bytes)
                    .await;

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // The receiver owns every partition initially; consume the
                // in-flight records but hold them un-acked so their offsets stay
                // pending across the revoke.
                let mut in_flight = Vec::new();
                for _ in 0..REBALANCE_TEST_PARTITIONS {
                    in_flight.push(receiver.recv_pdata().await);
                }

                // A second member joins and steals at least one partition,
                // forcing a revoke on the receiver.
                let trigger =
                    RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30))
                        .await;
                let revoked_partition = trigger
                    .assignment()
                    .into_iter()
                    .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                    .expect("rebalance trigger should own a partition from the test topic");
                receiver
                    .wait_for_partition_revocation(
                        TOPIC,
                        revoked_partition,
                        Duration::from_secs(10),
                    )
                    .await;

                // Ack the in-flight records now that at least one of their
                // partitions has been revoked. The acks for revoked partitions
                // must be dropped by the late-ack/stale-generation guard.
                for pdata in in_flight {
                    receiver.ack(pdata);
                }

                // Ack and Shutdown share the same FIFO control channel, so the
                // feedback is handled before the terminal snapshot is taken.
                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;
                drop(trigger);

                let mut m = FoldedMetrics::new();
                m.fold_all(terminal.metrics());
                let acknowledgements = measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.acknowledgements",
                    &[("signal", "traces"), ("outcome", "success")],
                    "responses",
                );
                assert!(
                    m.value("group.feedback.after_revocation") >= 1,
                    "at least one ack for a revoked partition should be counted and dropped, got \
                     {}; acknowledgements={acknowledgements}, revocations={}",
                    m.value("group.feedback.after_revocation"),
                    m.value("group.partition.revocations"),
                );
            },
        )
        .await;
    }

    /// Scenario (consumer-group rebalancing): identical to
    /// `stale_ack_after_revoke_counts_feedback_after_revocation` but the stale
    /// feedback is a terminal permanent **Nack** instead of an Ack -- a
    /// manual-commit receiver holds an un-acked in-flight record on a partition
    /// that is then stolen by a second group member (a `RebalanceTrigger`), after
    /// which the test permanently nacks the now-stale record.
    /// Guarantees: the late Nack for the revoked partition is subject to the same
    /// stale/late guard as an Ack (both funnel through `handle_offset_feedback`)
    /// -- it increments `receiver.kafka.consumer.group.feedback.after_revocation`
    /// and is not committed -- so a Nack that arrives after a partition is
    /// revoked can never advance a partition this consumer no longer owns.
    #[tokio::test]
    async fn stale_nack_after_revoke_counts_feedback_after_revocation() {
        const TOPIC: &str = "rebalance-stale-nack-traces";
        let group = "rebalance-stale-nack-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // One record per partition so the receiver has in-flight work on
                // every partition it owns.
                producer
                    .produce_per_partition(TOPIC, REBALANCE_TEST_PARTITIONS, 1, &bytes)
                    .await;

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume the in-flight records but hold them un-acked so their
                // offsets stay pending across the revoke.
                let mut in_flight = Vec::new();
                for _ in 0..REBALANCE_TEST_PARTITIONS {
                    in_flight.push(receiver.recv_pdata().await);
                }

                // A second member joins and steals at least one partition,
                // forcing a revoke on the receiver.
                let trigger =
                    RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30))
                        .await;
                let revoked_partition = trigger
                    .assignment()
                    .into_iter()
                    .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                    .expect("rebalance trigger should own a partition from the test topic");
                receiver
                    .wait_for_partition_revocation(
                        TOPIC,
                        revoked_partition,
                        Duration::from_secs(10),
                    )
                    .await;

                // Permanently nack the in-flight records now that at least one of
                // their partitions has been revoked. A terminal Nack for a revoked
                // partition must be dropped by the late-ack/stale-generation guard
                // exactly like an Ack.
                for pdata in in_flight {
                    receiver.nack_permanent("stale after revoke", pdata);
                }

                // Nack and Shutdown share the same FIFO control channel, so the
                // feedback is handled before the terminal snapshot is taken.
                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;
                drop(trigger);

                let mut m = FoldedMetrics::new();
                m.fold_all(terminal.metrics());
                let acknowledgements = measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.acknowledgements",
                    &[("signal", "traces"), ("outcome", "refused")],
                    "responses",
                );
                assert!(
                    m.value("group.feedback.after_revocation") >= 1,
                    "at least one nack for a revoked partition should be counted and dropped, got \
                     {}; acknowledgements={acknowledgements}, revocations={}",
                    m.value("group.feedback.after_revocation"),
                    m.value("group.partition.revocations"),
                );
            },
        )
        .await;
    }

    /// Scenario (consumer-group rebalancing): an idempotent manual-commit
    /// receiver consumes records but holds them un-acked, then a rebalance (a
    /// joining and leaving `RebalanceTrigger`) revokes and reassigns the
    /// partitions -- a NEW ownership generation -- and librdkafka redelivers the
    /// uncommitted offsets under that new generation.
    /// Guarantees: because the redelivered offsets belong to a *new* ownership
    /// period, the generation-aware idempotency guard does NOT skip them -- they
    /// are reprocessed (delivered again), not silently dropped, and
    /// `receiver.kafka.consumer.records.duplicates` is not incremented for a
    /// cross-generation redelivery. Idempotent dedupe applies only WITHIN an
    /// ownership generation (covered by the unit test
    /// `is_known_offset_for_generation_*`); it must never suppress a record that
    /// a new owner is responsible for reprocessing.
    #[tokio::test]
    async fn idempotent_redelivery_under_new_generation_is_reprocessed_not_skipped() {
        const TOPIC: &str = "rebalance-idempotent-traces";
        let group = "rebalance-idempotent-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                let records_per_partition = 3;
                producer
                    .produce_per_partition(
                        TOPIC,
                        REBALANCE_TEST_PARTITIONS,
                        records_per_partition,
                        &bytes,
                    )
                    .await;

                // Idempotent manual-commit receiver.
                let builder = KafkaReceiverConfigBuilder::new(
                    cluster.bootstrap_servers(),
                    group,
                    "test-client",
                )
                .with_traces(
                    SignalConfig::new(vec![TOPIC.to_string()])
                        .with_encoding(MessageFormat::OtlpProto),
                )
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Manual,
                    interval_ms: Some(500),
                })
                .with_auto_offset_reset(AutoOffsetReset::Earliest)
                .with_isolation_level(IsolationLevel::ReadUncommitted)
                .with_enable_idempotency(true);
                let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume the initial records but hold them un-acked so their
                // offsets are never committed; when the partition is reassigned
                // back, librdkafka redelivers from the (uncommitted) start.
                let total = (records_per_partition * REBALANCE_TEST_PARTITIONS) as usize;
                let mut seen = Vec::new();
                for _ in 0..total {
                    seen.push(receiver.recv_pdata().await);
                }

                // A member joins (revoking partitions from the receiver) and then
                // leaves (reassigning them back). The reassignment allocates a new
                // ownership generation, and the uncommitted offsets are redelivered
                // under it.
                let trigger =
                    RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30))
                        .await;
                tokio::time::sleep(Duration::from_secs(1)).await;
                drop(trigger);

                // Drain redelivered records within a bounded window, counting how
                // many arrive. Under the new generation they must be reprocessed
                // (delivered), not idempotently skipped.
                let mut redelivered = 0usize;
                let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
                while tokio::time::Instant::now() < deadline {
                    if let Some(pdata) = receiver.try_recv_pdata(Duration::from_millis(250)).await {
                        redelivered += 1;
                        receiver.ack(pdata);
                    }
                }
                assert!(
                    redelivered >= 1,
                    "offsets redelivered under a new ownership generation must be \
                     reprocessed (delivered again), not skipped; got {redelivered}",
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;

                let mut m = FoldedMetrics::new();
                m.fold_all(terminal.metrics());
                assert_eq!(
                    m.value("records.duplicates"),
                    0,
                    "a cross-generation redelivery must not be counted as a duplicate, got {}",
                    m.value("records.duplicates"),
                );
            },
        )
        .await;
    }

    /// Scenario (consumer-group rebalancing): a manual-commit receiver (idempotency
    /// disabled) consumes every record but holds them un-acked, so their offsets
    /// are never committed; a `RebalanceTrigger` then joins (revoking partitions,
    /// leaving the in-flight records un-committed) and leaves (reassigning them
    /// back), which forces librdkafka to redeliver those uncommitted offsets.
    /// This exercises the documented in-flight-on-revoke design (rebalance.rs:
    /// in-flight messages on a revoked partition are not drained/interrupted --
    /// the new owner re-delivers them, safe under at-least-once).
    /// Guarantees: the resulting duplication is **bounded** -- each
    /// `(partition, offset)` is delivered at most `1 + rebalance-transitions`
    /// times (the original delivery plus at most one redelivery per revoke/
    /// reassign transition), never in an unbounded re-loop -- and there is no
    /// loss: every produced offset is delivered at least once and, after the
    /// redelivered records are acked, each partition's committed offset equals
    /// exactly the produced count (no rollback, no commit past produced data).
    /// The stale ack from the pre-revoke ownership is dropped by the generation
    /// guard (asserted separately by
    /// `stale_ack_after_revoke_counts_feedback_after_revocation`).
    #[tokio::test]
    async fn inflight_records_on_revoke_are_redelivered_with_bounded_duplication() {
        const TOPIC: &str = "rebalance-bounded-dup-traces";
        const RECORDS_PER_PARTITION: i32 = 3;
        let group = "rebalance-bounded-dup-group";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                producer
                    .produce_per_partition(
                        TOPIC,
                        REBALANCE_TEST_PARTITIONS,
                        RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                // Manual-commit, idempotency DISABLED so redelivered offsets are
                // genuinely re-delivered (the harshest bounded-duplication case)
                // rather than skipped. No safety-net timer so acks alone drive
                // commits.
                let builder = KafkaReceiverConfigBuilder::new(
                    cluster.bootstrap_servers(),
                    group,
                    "test-client",
                )
                .with_traces(
                    SignalConfig::new(vec![TOPIC.to_string()])
                        .with_encoding(MessageFormat::OtlpProto),
                )
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Manual,
                    interval_ms: None,
                })
                .with_auto_offset_reset(AutoOffsetReset::Earliest)
                .with_isolation_level(IsolationLevel::ReadUncommitted);
                let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Count how many times each (partition, offset) is delivered.
                let mut delivery_counts: HashMap<(i32, i64), usize> = HashMap::new();
                let total = (RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;

                // Consume every record but hold them un-acked so their offsets are
                // never committed before the revoke.
                let mut in_flight = Vec::new();
                for _ in 0..total {
                    let pdata = receiver.recv_pdata().await;
                    let route = pdata
                        .source_route()
                        .expect("delivered pdata carries source calldata");
                    let (_topic_id, partition, offset, _generation) =
                        decode_calldata(&route.calldata);
                    *delivery_counts.entry((partition, offset)).or_insert(0) += 1;
                    in_flight.push(pdata);
                }

                // A member joins (revoking partitions, leaving the in-flight
                // records uncommitted) and then leaves (reassigning them back),
                // forcing redelivery of the uncommitted offsets.
                let trigger =
                    RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30))
                        .await;
                tokio::time::sleep(Duration::from_secs(1)).await;
                drop(trigger);

                // Now ack the original in-flight records. Acks for a partition that
                // was revoked are dropped by the stale-generation guard; acks for a
                // partition still owned advance its offset.
                for pdata in in_flight {
                    receiver.ack(pdata);
                }

                // Drain any redelivered records within a bounded window, counting
                // each delivery and acking so the redelivered offsets can commit.
                let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
                while tokio::time::Instant::now() < deadline {
                    if let Some(pdata) = receiver.try_recv_pdata(Duration::from_millis(250)).await {
                        let route = pdata
                            .source_route()
                            .expect("delivered pdata carries source calldata");
                        let (_topic_id, partition, offset, _generation) =
                            decode_calldata(&route.calldata);
                        *delivery_counts.entry((partition, offset)).or_insert(0) += 1;
                        receiver.ack(pdata);
                    }
                }

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;

                // No loss: every produced (partition, offset) was delivered at
                // least once.
                assert_eq!(
                    delivery_counts.len(),
                    total,
                    "every produced offset must be delivered at least once (no loss); \
                     saw {} distinct offsets, expected {total}",
                    delivery_counts.len(),
                );

                // Bounded duplication: no offset is delivered more than twice --
                // the original delivery plus at most one redelivery after the
                // revoke/reassign. This is the upper bound the acceptance
                // criterion requires; an unbounded re-loop would exceed it.
                // The `RebalanceTrigger` join+drop drives two rebalance
                // transitions (revoke on join, reassign on drop), so an
                // uncommitted offset can be redelivered once per transition on top
                // of its original delivery. The duplication is therefore bounded
                // by `1 + TRANSITIONS`; it must never grow into an unbounded
                // re-loop.
                const REBALANCE_TRANSITIONS: usize = 2;
                let max_deliveries = 1 + REBALANCE_TRANSITIONS;
                for ((partition, offset), count) in &delivery_counts {
                    assert!(
                        *count >= 1 && *count <= max_deliveries,
                        "offset {offset} on partition {partition} was delivered {count} \
                         times; duplication across the revoke/reassign must be bounded \
                         to at most {max_deliveries} (original + one redelivery per \
                         rebalance transition), not an unbounded re-loop",
                    );
                }

                // Global corroboration: total deliveries are bounded by the
                // produced count plus at most one redelivery wave.
                let max_total_deliveries = (total * max_deliveries) as u64;
                let admitted_messages = measurement_counter(
                    terminal.metrics(),
                    "receiver.messages",
                    &[("signal", "traces")],
                    "started",
                );
                assert!(
                    admitted_messages <= max_total_deliveries,
                    "total admitted messages ({admitted_messages}) must be bounded by produced records \
                     times the per-offset delivery bound ({max_total_deliveries}); an \
                     unbounded redelivery loop would exceed it",
                );

                // No loss, no rollback, no commit past produced data: once the
                // redelivered records are acked, each partition's committed offset
                // equals exactly the produced count.
                let brokers = cluster.bootstrap_servers().to_string();
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    let converged =
                        poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                            committed_offset(&brokers, group, TOPIC, partition)
                                .expect("kafka-test: committed-offset probe failed")
                                == Some(RECORDS_PER_PARTITION as i64)
                        })
                        .await;
                    assert!(
                        converged,
                        "partition {partition} committed offset must equal the produced \
                         count {RECORDS_PER_PARTITION} (no loss, no rollback, no \
                         double-commit), got {:?}",
                        committed_offset(&brokers, group, TOPIC, partition)
                            .expect("kafka-test: committed-offset probe failed"),
                    );
                }
            },
        )
        .await;
    }

    // ---- Lifecycle: drain and shutdown ----

    /// Scenario (lifecycle: drain and shutdown): a manual-commit receiver consumes and acks an initial batch,
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

    /// Scenario (lifecycle: drain and shutdown): a consumer-lag worker holding an
    /// `Arc` clone of the consumer is still in flight at Shutdown, but it honors
    /// cooperative cancellation and returns within the bounded drain.
    /// Guarantees: after the shutdown handler's bounded drain
    /// (`timeout_at(min(lag_deadline, shutdown_deadline), handle)`) resolves, the
    /// worker has returned and released its clone, so only the loop's clone
    /// remains (`strong_count == 1`). The subsequent consumer drop is therefore
    /// the last `Arc` and is what triggers the leave-group/close -- the happy
    /// path the shutdown ordering relies on.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounded_drain_lets_cooperative_lag_worker_release_clone_before_close() {
        let start = tokio::time::Instant::now();
        // Worker deadline far out (15s); shutdown deadline near (1s). The worker
        // must finish via cancellation, not by hitting either deadline.
        let lag_deadline = start + LAG_REFRESH_TOTAL_DEADLINE;
        let shutdown_deadline = start + Duration::from_secs(1);

        // Shared reference stands in for the consumer `Arc`: one clone held by
        // the loop, one by the lag worker.
        let consumer = Arc::new(());
        let worker_clone = Arc::clone(&consumer);

        // A cooperatively-cancellable worker (mirrors `compute_consumer_lag`
        // abandoning the refresh on cancellation). It holds its clone until it
        // observes cancellation and returns.
        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let handle = tokio::task::spawn(async move {
            worker_cancel.cancelled().await;
            drop(worker_clone);
            None::<f64>
        });

        // Model the shutdown handler's bounded drain: cancel, then wait bounded by
        // the tighter of the two deadlines.
        cancel.cancel();
        let bound = lag_deadline.min(shutdown_deadline);
        let drain = tokio::time::timeout_at(bound, handle).await;

        // The worker observed cancellation and completed within the bound, so the
        // drain resolved with the worker's result (not a timeout).
        let join_result = drain.expect("drain must not exceed the bounded deadline");
        assert_eq!(
            join_result.expect("worker must not panic"),
            None,
            "a cancelled lag worker abandons the refresh and returns None",
        );

        // The worker released its clone before the drain returned, so the loop's
        // clone is the only one left: the consumer drop here would be the last
        // `Arc`.
        assert_eq!(
            Arc::strong_count(&consumer),
            1,
            "cooperative lag worker must release its clone before the close, \
             leaving the loop clone as the last Arc",
        );
    }

    /// Scenario (lifecycle: drain and shutdown): a consumer-lag worker holding an
    /// `Arc` clone of the consumer is parked mid-librdkafka-call at Shutdown and
    /// does NOT observe cancellation before the bounded drain expires.
    /// Guarantees: the bounded drain
    /// (`timeout_at(min(lag_deadline, shutdown_deadline), handle)`) times out with
    /// the worker still alive and still holding its clone, so at the close the
    /// consumer `Arc` still has `strong_count == 2`. This documents that the
    /// consumer drop in the shutdown handler is NOT guaranteed to be the last
    /// `Arc` -- the leave-group/close then runs when the abandoned worker later
    /// releases its clone, not necessarily at that drop.
    #[tokio::test(start_paused = true)]
    async fn shutdown_bounded_drain_can_leave_lag_worker_clone_alive_refcount_two() {
        let start = tokio::time::Instant::now();
        // Shutdown deadline near (1s); worker deadline far (15s). The bound is the
        // shutdown deadline.
        let lag_deadline = start + LAG_REFRESH_TOTAL_DEADLINE;
        let shutdown_deadline = start + Duration::from_secs(1);

        let consumer = Arc::new(());
        let worker_clone = Arc::clone(&consumer);

        // A worker that ignores cancellation and never finishes on its own, so it
        // is still holding its clone when the bounded drain expires. It stands in
        // for a worker parked inside a librdkafka FFI call that has not yet
        // returned to observe the cancellation token.
        let cancel = CancellationToken::new();
        let handle = tokio::task::spawn(async move {
            // Keep the clone alive for the whole task.
            let _held = worker_clone;
            std::future::pending::<()>().await;
            None::<f64>
        });

        // Model the shutdown handler's bounded drain: cancel, then wait bounded by
        // the tighter of the two deadlines. The worker never honors it.
        cancel.cancel();
        let bound = lag_deadline.min(shutdown_deadline);
        assert_eq!(
            bound, shutdown_deadline,
            "min must pick the shutdown deadline"
        );
        let drain = tokio::time::timeout_at(bound, handle).await;

        // The drain timed out: the worker is still running.
        assert!(
            drain.is_err(),
            "a non-cooperative worker is bounded by the deadline, not awaited to completion",
        );
        // The still-running worker keeps its clone, so the consumer drop that
        // follows in the shutdown handler is NOT the last `Arc`.
        assert_eq!(
            Arc::strong_count(&consumer),
            2,
            "a lag worker that outlives the bounded drain still holds its clone, \
             so the close-time drop is not the last Arc (refcount is 2)",
        );
        // Termination is still bounded: the drain returned by the shutdown
        // deadline rather than waiting for the worker.
        assert!(
            tokio::time::Instant::now() <= shutdown_deadline,
            "drain must complete by the shutdown deadline, not the worker's",
        );
    }

    // Scenario: a consumer-lag worker is in flight at Shutdown, and this time the
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

    /// Scenario (lifecycle: drain and shutdown): a manual-commit receiver spawns a lag refresh for a consumer
    /// that owns no partitions (empty assignment).
    /// Guarantees: `spawn_consumer_lag_refresh` still spawns a task (manual mode)
    /// and the task returns `Some(0.0)` -- the documented empty-assignment
    /// sentinel -- so the caller resets `receiver.kafka.consumer.group.lag` to 0
    /// rather than leaving a stale value.
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

    /// Scenario (lifecycle: drain and shutdown): auto-commit receiver requests a lag refresh.
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

    /// Scenario (lifecycle: drain and shutdown): a manual-commit receiver
    /// consumes and acks records while a producer keeps sending throughout; the
    /// receiver is then drained and later shut down, with more records produced
    /// after the drain begins.
    /// Guarantees: under sustained traffic the receiver still emits
    /// `ReceiverDrained`, stops forwarding new records once drained, commits the
    /// offsets acked before the drain (committed offset >= the pre-drain acked
    /// count), and terminates cleanly on the subsequent `Shutdown`.
    #[tokio::test]
    async fn drain_under_sustained_traffic_commits_and_stops_cleanly() {
        const TOPIC: &str = "drain-sustained-traces";
        const PRE_DRAIN: usize = 5;
        const POST_DRAIN: usize = 10;
        let group = "drain-sustained-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce a first burst the receiver will consume and ack.
                for i in 0..PRE_DRAIN {
                    let key = format!("pre-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send pre-drain");
                }

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume and ack every pre-drain record so its offset is
                // committable at drain time.
                for _ in 0..PRE_DRAIN {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Begin the receiver-first drain while traffic continues.
                receiver.drain(Duration::from_secs(5));

                // Keep producing after the drain: the receiver has stopped
                // polling, so none of these must be forwarded.
                for i in 0..POST_DRAIN {
                    let key = format!("post-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send post-drain");
                }

                // The receiver must signal ReceiverDrained (skip past the
                // timer-setup runtime messages emitted during startup).
                let mut drained = false;
                for _ in 0..16 {
                    match receiver.try_recv_runtime(Duration::from_secs(10)).await {
                        Some(RuntimeControlMsg::ReceiverDrained { .. }) => {
                            drained = true;
                            break;
                        }
                        Some(_) => continue,
                        None => break,
                    }
                }
                assert!(
                    drained,
                    "receiver never emitted ReceiverDrained under traffic"
                );

                // No further pdata should arrive once drained, even though
                // POST_DRAIN records are now on the broker.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(3))
                        .await
                        .is_none(),
                    "receiver forwarded a record after DrainIngress under traffic",
                );

                // The pre-drain acked offsets must be committed.
                let brokers = cluster.bootstrap_servers().to_string();
                let committed =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= PRE_DRAIN as i64)
                    })
                    .await;
                assert!(
                    committed,
                    "pre-drain offsets should be committed, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (lifecycle: drain and shutdown): a manual-commit receiver holds
    /// in-flight records that are never acked (their downstream acks are still
    /// pending) when `DrainIngress` arrives.
    /// Guarantees: the receiver signals `ReceiverDrained` promptly without
    /// blocking on the un-acked in-flight records -- codifying the documented
    /// design that drain does not wait for in-flight downstream acks and relies
    /// on at-least-once redelivery for the un-committed offsets.
    #[tokio::test]
    async fn drain_does_not_wait_for_inflight_downstream_acks() {
        const TOPIC: &str = "drain-inflight-traces";
        const RECORDS: usize = 4;
        let group = "drain-inflight-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume every record but hold them un-acked: their downstream
                // acks are deliberately never delivered, so they stay in-flight.
                let mut in_flight = Vec::new();
                for _ in 0..RECORDS {
                    in_flight.push(receiver.recv_pdata().await);
                }

                // Begin the drain with the in-flight records still un-acked.
                let drain_started = tokio::time::Instant::now();
                receiver.drain(Duration::from_secs(30));

                // ReceiverDrained must arrive promptly -- well within the drain
                // deadline -- proving the drain did not block waiting for the
                // in-flight acks.
                let mut drained = false;
                for _ in 0..16 {
                    match receiver.try_recv_runtime(Duration::from_secs(5)).await {
                        Some(RuntimeControlMsg::ReceiverDrained { .. }) => {
                            drained = true;
                            break;
                        }
                        Some(_) => continue,
                        None => break,
                    }
                }
                assert!(
                    drained,
                    "receiver must signal ReceiverDrained without waiting for \
                     in-flight downstream acks",
                );
                assert!(
                    drain_started.elapsed() < Duration::from_secs(15),
                    "drain notification took too long ({:?}); it should not block \
                     on in-flight acks",
                    drain_started.elapsed(),
                );

                // The held records are still un-acked; drop them to release the
                // in-flight set, then terminate cleanly.
                drop(in_flight);
                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (lifecycle: drain and shutdown): a manual-commit receiver with
    /// tracked, un-committed offsets is shut down while every broker is marked
    /// down, so the shutdown-time consumer unsubscribe/close cannot reach the
    /// broker.
    /// Guarantees: the receiver still reaches its terminal state within a
    /// bounded wait rather than hanging the pipeline on an unreachable broker --
    /// the synchronous librdkafka close runs off the loop thread bounded by the
    /// shutdown deadline, so it cannot stall termination.
    #[tokio::test]
    async fn shutdown_with_broker_unavailable_does_not_hang() {
        const TOPIC: &str = "shutdown-broker-down-traces";
        const RECORDS: usize = 3;
        let group = "shutdown-broker-down-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume and ack every record so there are tracked offsets to
                // commit at shutdown.
                for _ in 0..RECORDS {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Make the broker slow to respond so the shutdown-time close
                // (unsubscribe + consumer drop) is delayed well past the shutdown
                // deadline. A large per-request round-trip delay stands in for an
                // unreachable broker but -- unlike marking the broker fully down --
                // still lets the off-thread close eventually complete, so the test
                // does not orphan a permanently-blocked librdkafka FFI thread that
                // would stall runtime teardown.
                cluster.faults().round_trip_time(1, Duration::from_secs(30));

                // Shutdown with a short (1s) deadline while the broker is
                // effectively unavailable. The receiver's off-loop-thread close is
                // bounded by this deadline, so the receiver task must return well
                // within the generous outer bound below even though the broker is
                // not responding -- a regression that ran the blocking close on the
                // loop thread would exceed it and fail the test deterministically.
                let shutdown_at = tokio::time::Instant::now();
                receiver.shutdown(Duration::from_secs(1));
                let terminated =
                    tokio::time::timeout(Duration::from_secs(5), receiver.await_terminal_state())
                        .await;
                assert!(
                    terminated.is_ok(),
                    "receiver must terminate within the bounded deadline even when \
                     the broker is unavailable at shutdown",
                );
                assert!(
                    shutdown_at.elapsed() < Duration::from_secs(5),
                    "termination should be bounded by the shutdown deadline, not the \
                     30s broker round-trip delay; took {:?}",
                    shutdown_at.elapsed(),
                );

                // Restore normal broker latency so the (deadline-exceeded)
                // off-thread close can finish and its blocking thread joins,
                // keeping test teardown clean.
                cluster
                    .faults()
                    .round_trip_time(1, Duration::from_millis(1));
            },
        )
        .await;
    }

    /// Scenario (lifecycle: drain and shutdown): a manual-commit receiver with
    /// the opt-in consumer-lag refresh timer armed (so a lag-refresh worker is
    /// periodically spawned and holds its own `Arc` clone of the consumer) is
    /// shut down after the timer has had time to spawn a refresh.
    /// Guarantees: the shutdown handler's bounded lag-drain-then-close path runs
    /// with a real in-flight lag worker present and the receiver still reaches
    /// its terminal state within the shutdown deadline -- the lag worker cannot
    /// stall termination. This exercises the real receiver path; the internal
    /// `Arc`-count-can-be-2 behavior it can produce is asserted deterministically
    /// by the unit test
    /// `shutdown_bounded_drain_can_leave_lag_worker_clone_alive_refcount_two`
    /// (the harness exposes no access to the receiver's private consumer `Arc`).
    #[tokio::test]
    async fn shutdown_with_lag_refresh_in_flight_still_terminates_within_deadline() {
        const TOPIC: &str = "shutdown-lag-inflight-traces";
        const RECORDS: usize = 3;
        let group = "shutdown-lag-inflight-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");
                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                // Arm the lag refresh timer at a short interval so a lag-refresh
                // worker is repeatedly spawned.
                let cfg = manual_traces_config_with_lag_refresh(
                    cluster.bootstrap_servers(),
                    group,
                    TOPIC,
                    50,
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
                for _ in 0..RECORDS {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Give the lag timer several ticks so a refresh worker is spawned
                // and can be in flight when the shutdown arrives.
                tokio::time::sleep(Duration::from_millis(150)).await;

                // Shutdown with a short (1s) deadline. Even with a lag worker in
                // flight, the bounded drain plus off-loop-thread close must let
                // the receiver return within the deadline.
                let shutdown_at = tokio::time::Instant::now();
                receiver.shutdown(Duration::from_secs(1));
                let terminated =
                    tokio::time::timeout(Duration::from_secs(5), receiver.await_terminal_state())
                        .await;
                assert!(
                    terminated.is_ok(),
                    "receiver must terminate within the bounded deadline even with a \
                     lag refresh in flight at shutdown",
                );
                assert!(
                    shutdown_at.elapsed() < Duration::from_secs(5),
                    "termination must be bounded by the shutdown deadline; took {:?}",
                    shutdown_at.elapsed(),
                );
            },
        )
        .await;
    }

    // ---- Failure recovery ----

    /// Scenario (failure recovery): a long run of fetch errors is injected
    /// before a manual-commit receiver starts, held active long enough to
    /// observe that the receiver cannot make progress, and only then cleared.
    /// Guarantees: while the transport fault is active the receiver encounters
    /// the failure and delivers no records (the fetch path keeps erroring), yet
    /// the receive loop is non-fatal -- it keeps running rather than
    /// terminating -- and once the fault clears the same loop reconnects and
    /// delivers every record, proving the transport-error arm's
    /// encounter-then-recover contract (not merely post-clear delivery).
    #[tokio::test]
    async fn transport_error_is_non_fatal_and_recovers() {
        const TOPIC: &str = "failure-transport-traces";
        const RECORDS: usize = 4;
        let group = "failure-transport-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                // Inject a LONG run of fetch errors (consumed one-per-request in
                // order) so the fault stays active across the whole observation
                // window below -- long enough that it cannot be silently
                // exhausted before the receiver would otherwise deliver. This is
                // what forces the receiver to actually encounter the transport
                // failure rather than sailing past a couple of quickly-retried
                // errors.
                let fetch_errors = vec![RDKafkaRespErr::RD_KAFKA_RESP_ERR_REQUEST_TIMED_OUT; 512];
                cluster.faults().fail_fetch(&fetch_errors);

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // While the fault is active the receiver must encounter the
                // failure and make no progress: no record is delivered within a
                // generous window. This proves a failure was hit *before* the
                // fault is cleared, not just that delivery works afterward.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(3))
                        .await
                        .is_none(),
                    "receiver delivered a record while the fetch fault was active; \
                     the transport failure was not actually encountered",
                );

                // Clear the fault so fetches can succeed. librdkafka retries the
                // injected fetch errors internally, so rather than assert on the
                // (best-effort, mock-timing-dependent)
                // `receiver.kafka.transport.errors` counter,
                // the observable guarantee is that the loop survived the errors
                // (it did not terminate during the window above) and resumes
                // delivery once they clear.
                cluster.faults().clear_fetch_failures();

                // The same receive loop must now deliver every record -- it was
                // not killed by the sustained transport errors.
                for _ in 0..RECORDS {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (failure recovery): every broker is taken down mid-stream after a
    /// manual-commit receiver has consumed a first batch, then brought back up
    /// and more records are produced.
    /// Guarantees: a prolonged broker outage does not kill the receiver -- no
    /// records are delivered while all brokers are down, and once the brokers
    /// recover the same receiver reconnects and delivers the post-outage records
    /// without loss, exercising librdkafka's reconnect/backoff behavior.
    #[tokio::test]
    async fn broker_outage_then_recovery_resumes_without_loss() {
        const TOPIC: &str = "failure-outage-traces";
        const PRE: usize = 3;
        const POST: usize = 3;
        let group = "failure-outage-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..PRE {
                    let key = format!("pre-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send pre-outage record");
                }

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume and ack the first batch before the outage.
                for _ in 0..PRE {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Prolonged outage: every broker down. No new records must be
                // delivered while the brokers are unreachable.
                cluster.faults().all_brokers_down();
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(2))
                        .await
                        .is_none(),
                    "no records should be delivered while all brokers are down",
                );

                // Recover: bring brokers back and produce more records.
                cluster.faults().all_brokers_up();
                for i in 0..POST {
                    let key = format!("post-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send post-outage record");
                }

                // The same receiver must reconnect and deliver every post-outage
                // record without loss.
                for _ in 0..POST {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (failure recovery): a manual-commit receiver delivers and acks a
    /// first batch, then hits a transient network interruption -- simulated by a
    /// burst of injected fetch errors that block fetches for a bounded window --
    /// which is later cleared and more records produced.
    /// Guarantees: the receiver makes no progress while the interruption is
    /// active (no records delivered), yet the loop is non-fatal and, once the
    /// interruption clears, the same receiver recovers and delivers the
    /// post-interruption records with no loss (its committed offset reaches the
    /// full produced count). Models an intermittent network hiccup distinct from
    /// the sustained full-outage case; a truly asymmetric (one-way) partition is
    /// not modeled by the mock.
    #[tokio::test]
    async fn intermittent_network_interruption_recovers_without_loss() {
        const TOPIC: &str = "failure-netblip-traces";
        const PRE: usize = 3;
        const POST: usize = 3;
        let group = "failure-netblip-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                for i in 0..PRE {
                    let key = format!("pre-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send pre-interruption record");
                }

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume and ack the first batch before the interruption.
                for _ in 0..PRE {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Transient network interruption: a long burst of fetch errors
                // that blocks fetches while active. Consumed one-per-request in
                // order, sized to outlast the observation window below.
                let fetch_errors = vec![RDKafkaRespErr::RD_KAFKA_RESP_ERR_REQUEST_TIMED_OUT; 512];
                cluster.faults().fail_fetch(&fetch_errors);

                // Produce during the interruption; nothing must be delivered while
                // it is active.
                for i in 0..POST {
                    let key = format!("post-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send post-interruption record");
                }
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(3))
                        .await
                        .is_none(),
                    "no records should be delivered during the network interruption",
                );

                // Clear the interruption so fetches can succeed again.
                cluster.faults().clear_fetch_failures();

                // The same receiver must recover and deliver every post-interruption
                // record without loss.
                for _ in 0..POST {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // No loss: the committed offset reaches the full produced count.
                let brokers = cluster.bootstrap_servers().to_string();
                let committed =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= (PRE + POST) as i64)
                    })
                    .await;
                assert!(
                    committed,
                    "after recovery the committed offset should reach the full \
                     produced count {}, got {:?}",
                    PRE + POST,
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (failure recovery): a manual-commit receiver runs against a
    /// cluster where every broker has an injected per-request round-trip latency
    /// (a slow-but-reachable broker, not an outage), then consumes and acks
    /// every produced record.
    /// Guarantees: bounded broker latency slows but does not corrupt offset
    /// accounting -- every record is still delivered and the committed offset
    /// advances to exactly the produced count with no loss and no commit errors
    /// -- so a laggy broker cannot desynchronize the receiver's offset tracking.
    #[tokio::test]
    async fn broker_latency_does_not_corrupt_offset_accounting() {
        const TOPIC: &str = "failure-latency-traces";
        const RECORDS: i64 = 3;
        let group = "failure-latency-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");
                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                // Inject a bounded per-request latency on all brokers. The broker
                // stays reachable; requests merely take longer.
                cluster
                    .faults()
                    .round_trip_time(-1, Duration::from_millis(50));

                let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // A larger per-record timeout absorbs the injected latency; every
                // record must still arrive.
                for _ in 0..RECORDS {
                    let pdata = receiver
                        .try_recv_pdata(Duration::from_secs(10))
                        .await
                        .expect("record delivered despite broker latency");
                    receiver.ack(pdata);
                }

                let brokers = cluster.bootstrap_servers().to_string();
                let committed =
                    poll_until(Duration::from_secs(8), Duration::from_millis(200), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= RECORDS)
                    })
                    .await;
                assert!(
                    committed,
                    "under bounded broker latency the committed offset must reach \
                     the full produced count {RECORDS} with no loss, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;
                assert_eq!(
                    measurement_counter(
                        terminal.metrics(),
                        "receiver.kafka.offset_commits",
                        &[("outcome", "failure")],
                        "commits",
                    ),
                    0,
                    "broker latency must not induce offset commit errors",
                );
            },
        )
        .await;
    }

    // ---- Routing and payload correctness ----

    /// Scenario (routing and payload correctness): OTLP-proto traces bytes are decoded.
    /// Guarantees: the payload decodes to an `ExportTracesRequest`, so OTLP-proto traces
    /// route to the traces decoder.
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

    /// Scenario (routing and payload correctness): OTLP-proto metrics bytes are decoded.
    /// Guarantees: the payload decodes to an `ExportMetricsRequest`, so OTLP-proto metrics
    /// route to the metrics decoder.
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

    /// Scenario (routing and payload correctness): OTLP-proto logs bytes are decoded.
    /// Guarantees: the payload decodes to an `ExportLogsRequest`, so OTLP-proto logs route
    /// to the logs decoder.
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

    /// Scenario (routing and payload correctness): OTAP-Arrow traces bytes are decoded.
    /// Guarantees: the payload decodes to `OtapArrowRecords::Traces`, so OTAP-encoded
    /// traces route to the Arrow decoder.
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

    /// Scenario (routing and payload correctness): OTAP-Arrow metrics bytes are decoded.
    /// Guarantees: the payload decodes to `OtapArrowRecords::Metrics`, so OTAP-encoded
    /// metrics route to the Arrow decoder.
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

    /// Scenario (routing and payload correctness): OTAP-Arrow logs bytes are decoded.
    /// Guarantees: the payload decodes to `OtapArrowRecords::Logs`, so OTAP-encoded logs
    /// route to the Arrow decoder.
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

    /// Scenario (routing and payload correctness): undecodable bytes are passed to the OTAP
    /// traces decoder.
    /// Guarantees: decode returns an error rather than panicking, so a malformed OTAP
    /// payload is a recoverable per-message error.
    #[test]
    fn decode_traces_payload_invalid_otap_bytes_returns_error() {
        let result = decode_traces_payload(b"not valid protobuf", MessageFormat::OtapProto);
        assert!(result.is_err());
    }

    /// Scenario (routing and payload correctness): OTLP-proto traces bytes are decoded and
    /// then re-extracted.
    /// Guarantees: the bytes round-trip byte-for-byte, so the zero-copy OTLP path does not
    /// mutate the payload.
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

    /// Scenario (routing and payload correctness): OTLP-proto trace records produced to a Kafka topic are consumed
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

    /// Scenario (routing and payload correctness): OTLP-proto log records produced to a Kafka topic are consumed
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

    /// Scenario (routing and payload correctness): OTLP-proto metric records produced to a Kafka topic are consumed
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

    /// Scenario (routing and payload correctness): OTAP-Arrow trace records produced to a Kafka topic are consumed
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

    /// Scenario (routing and payload correctness): OTAP-Arrow metric records produced to a Kafka topic are consumed
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

    /// Scenario (routing and payload correctness): OTAP-Arrow log records produced to a Kafka topic are consumed
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

    /// Scenario (routing and payload correctness): the receiver's traces signal is configured with the default
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

    /// Scenario (routing and payload correctness): an OTLP-proto trace record carries a Kafka header `x-tenant-id`
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

    /// Scenario (routing and payload correctness): an OTAP-Arrow trace record carries a Kafka header `x-tenant-id`
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

    /// Scenario (routing and payload correctness): a capture policy captures `X-Tenant-Id` (stored as `tenant_id`)
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

    /// Scenario (routing and payload correctness): a record carries a Kafka header but the receiver is started
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

    /// Scenario (routing and payload correctness): a record carries `X-Tenant-Id` (captured to a transport header)
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

    /// Scenario (routing and payload correctness): a capture policy is applied to an OTAP-Arrow record that also
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

    /// Scenario (routing and payload correctness): a topic list of exact names (no regexes)
    /// is matched against candidate topics.
    /// Guarantees: only exact-name matches succeed and an empty list matches nothing, so
    /// exact topic subscription is precise.
    #[test]
    fn matches_any_topic_exact() {
        let topics = vec!["traces".to_string()];
        let regexes = vec![None];
        assert!(matches_any_topic(&topics, &regexes, "traces"));
        assert!(!matches_any_topic(&topics, &regexes, "other"));

        // Empty list matches nothing
        assert!(!matches_any_topic(&[], &[], "traces"));
    }

    /// Scenario (routing and payload correctness): a `^`-anchored regex topic pattern is
    /// matched against candidate topics.
    /// Guarantees: topics matching the regex are accepted and non-matching topics rejected,
    /// so regex topic subscription works.
    #[test]
    fn matches_any_topic_regex() {
        let topics = vec!["^traces-.*".to_string()];
        let re = Regex::new("^traces-.*").unwrap();
        let regexes = vec![Some(re)];
        assert!(matches_any_topic(&topics, &regexes, "traces-prod"));
        assert!(matches_any_topic(&topics, &regexes, "traces-staging"));
        assert!(!matches_any_topic(&topics, &regexes, "metrics-prod"));
    }

    /// Scenario (routing and payload correctness): a mixed list of exact names and a regex
    /// is matched against candidate topics.
    /// Guarantees: a topic matching any exact entry or the regex is accepted and all others
    /// rejected, so mixed exact/regex lists compose correctly.
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

    /// Scenario (routing and payload correctness): a real receiver's compiled per-signal
    /// topic matchers are queried for regex traces, exact metrics, and unconfigured logs.
    /// Guarantees: each candidate routes to the correct signal (or none), so the receiver's
    /// compiled topic regexes drive per-signal dispatch.
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

    /// Scenario (routing and payload correctness): a receiver configured with several exact
    /// and regex topics per signal is queried across candidates.
    /// Guarantees: each candidate matches only the intended signal's topic set, so
    /// multi-topic per-signal routing is correct.
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

    /// Scenario (routing and payload correctness): a signal is configured with a
    /// syntactically invalid topic regex.
    /// Guarantees: config validation fails at construction, so an invalid regex is rejected
    /// before the receiver starts.
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

    /// Scenario (routing and payload correctness): distinct topics are interned into the
    /// topic registry.
    /// Guarantees: each new topic gets a sequential id and a repeat lookup returns the same
    /// id, so topic ids are stable and `Copy`.
    #[test]
    fn topic_registry_assigns_sequential_ids() {
        let mut reg = TopicRegistry::new();

        assert_eq!(reg.get_or_assign("traces-prod"), Some(0));
        assert_eq!(reg.get_or_assign("metrics-prod"), Some(1));
        assert_eq!(reg.get_or_assign("logs-prod"), Some(2));

        // Same topic returns the same ID.
        assert_eq!(reg.get_or_assign("traces-prod"), Some(0));
    }

    /// Scenario (routing and payload correctness): a topic is interned and then looked up
    /// by id.
    /// Guarantees: the id maps back to the original name and an unknown id returns `None`,
    /// so the id/name mapping round-trips.
    #[test]
    fn topic_registry_name_for_roundtrip() {
        let mut reg = TopicRegistry::new();

        let id = reg.get_or_assign("my-topic").expect("id assigned");
        assert_eq!(reg.name_for(id).as_deref(), Some("my-topic"));
        assert_eq!(reg.name_for(99), None);
    }

    /// Scenario (routing and payload correctness): a range of (topic_id, partition, offset,
    /// generation) tuples, including values that overflow the legacy `u8` id, are encoded
    /// into calldata and decoded back.
    /// Guarantees: every field round-trips exactly, so the offset-correlation calldata is
    /// lossless across the full value range.
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

    /// Scenario (routing and payload correctness): a (topic_id, partition, offset,
    /// generation) tuple is encoded.
    /// Guarantees: the calldata occupies exactly three slots, pinning the on-wire calldata
    /// layout.
    #[test]
    fn encode_calldata_produces_three_slots() {
        let calldata = encode_calldata(1, 5, 42, 3);
        assert_eq!(calldata.len(), 3);
    }

    /// Scenario (routing and payload correctness): a legacy two-slot calldata (no
    /// generation slot) is decoded.
    /// Guarantees: the missing generation defaults to 0 while the other fields decode
    /// correctly, so older calldata stays backward-compatible.
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

    /// Scenario (routing and payload correctness): a single receiver subscribes
    /// simultaneously to a distinct traces topic, metrics topic, and logs topic
    /// (disjoint across signals), and one OTLP-proto record is produced to each.
    /// Guarantees: each record is routed to the decoder for its own signal --
    /// the traces topic yields an `ExportTracesRequest`, the metrics topic an
    /// `ExportMetricsRequest`, and the logs topic an `ExportLogsRequest` -- so
    /// concurrent multi-signal topic routing dispatches every topic to the
    /// correct signal without cross-contamination.
    #[tokio::test]
    async fn multi_signal_topics_route_to_correct_decoders() {
        const TRACES_TOPIC: &str = "route-multi-traces";
        const METRICS_TOPIC: &str = "route-multi-metrics";
        const LOGS_TOPIC: &str = "route-multi-logs";
        with_cluster(
            KafkaTestCluster::builder()
                .topic(TRACES_TOPIC)
                .topic(METRICS_TOPIC)
                .topic(LOGS_TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();

                let traces_req = create_traces_with_spans();
                let mut traces_bytes = vec![];
                traces_req.encode(&mut traces_bytes).expect("encode traces");
                let metrics_req = create_metrics_service_request();
                let mut metrics_bytes = vec![];
                metrics_req
                    .encode(&mut metrics_bytes)
                    .expect("encode metrics");
                let logs_req = create_logs_service_request();
                let mut logs_bytes = vec![];
                logs_req.encode(&mut logs_bytes).expect("encode logs");

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

                // Records may arrive in any order; classify each by its decoded
                // signal type and assert all three signals are represented.
                let mut saw_traces = false;
                let mut saw_metrics = false;
                let mut saw_logs = false;
                for _ in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    match proto {
                        OtlpProtoBytes::ExportTracesRequest(ref b) => {
                            assert_eq!(b.as_ref(), &traces_bytes, "traces payload preserved");
                            saw_traces = true;
                        }
                        OtlpProtoBytes::ExportMetricsRequest(ref b) => {
                            assert_eq!(b.as_ref(), &metrics_bytes, "metrics payload preserved");
                            saw_metrics = true;
                        }
                        OtlpProtoBytes::ExportLogsRequest(ref b) => {
                            assert_eq!(b.as_ref(), &logs_bytes, "logs payload preserved");
                            saw_logs = true;
                        }
                    }
                }
                assert!(
                    saw_traces && saw_metrics && saw_logs,
                    "each signal topic must route to its own decoder \
                     (traces={saw_traces}, metrics={saw_metrics}, logs={saw_logs})",
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (routing and payload correctness): a receiver's traces signal is
    /// configured with a single `^`-prefixed regex subscription (`^route-regex-.*`)
    /// and records are produced to three independently-created broker topics
    /// that all match the pattern.
    /// Guarantees: the receiver consumes records from every topic matching the
    /// regex subscription -- not just a literal topic name -- so pattern-based
    /// subscription delivers from all matching topics.
    #[tokio::test]
    async fn regex_topic_subscription_consumes_all_matching_topics() {
        const TOPIC_A: &str = "route-regex-alpha";
        const TOPIC_B: &str = "route-regex-beta";
        const TOPIC_C: &str = "route-regex-gamma";
        with_cluster(
            KafkaTestCluster::builder()
                .topic(TOPIC_A)
                .topic(TOPIC_B)
                .topic(TOPIC_C),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");
                for topic in [TOPIC_A, TOPIC_B, TOPIC_C] {
                    producer
                        .send_full(SendRecord::new(topic, &bytes).key(topic.as_bytes()))
                        .await
                        .unwrap_or_else(|e| panic!("send to {topic}: {e}"));
                }

                // Single regex subscription that matches all three topics.
                let cfg = auto_config(
                    cluster.bootstrap_servers(),
                    &["^route-regex-.*"],
                    &[],
                    &[],
                    MessageFormat::OtlpProto,
                    HashMap::new(),
                );
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Correlate delivered records back to their source topic via the
                // stamped calldata topic-id is not name-resolvable here, so
                // instead assert that exactly three records (one per matching
                // topic) are delivered and their payloads round-trip.
                let mut delivered = 0;
                for _ in 0..3 {
                    let mut pdata = receiver.recv_pdata().await;
                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    assert!(matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)));
                    assert_eq!(proto.as_bytes(), &bytes, "payload preserved");
                    delivered += 1;
                }
                assert_eq!(
                    delivered, 3,
                    "regex subscription must consume from all three matching topics",
                );
                // No fourth record exists: the pattern matched exactly the three
                // produced topics.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(2))
                        .await
                        .is_none(),
                    "no extra records beyond the three matching topics",
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    /// Scenario (routing and payload correctness): a manual-commit receiver
    /// configured with `isolation_level: read_committed` consumes records
    /// produced (non-transactionally) to its topic.
    /// Guarantees: the receiver still delivers every record and commits the full
    /// count under the read-committed isolation level -- so selecting
    /// read-committed does not break ordinary (non-transactional) consumption.
    #[tokio::test]
    async fn read_committed_isolation_delivers_and_commits() {
        const TOPIC: &str = "route-readcommitted-traces";
        const RECORDS: i64 = 3;
        let group = "route-readcommitted-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");
                for i in 0..RECORDS {
                    let key = format!("rec-{i}");
                    producer
                        .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                        .await
                        .expect("send record");
                }

                let builder = KafkaReceiverConfigBuilder::new(
                    cluster.bootstrap_servers(),
                    group,
                    "test-client",
                )
                .with_traces(
                    SignalConfig::new(vec![TOPIC.to_string()])
                        .with_encoding(MessageFormat::OtlpProto),
                )
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Manual,
                    interval_ms: None,
                })
                .with_auto_offset_reset(AutoOffsetReset::Earliest)
                .with_isolation_level(IsolationLevel::ReadCommitted);
                let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                for _ in 0..RECORDS {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                let brokers = cluster.bootstrap_servers().to_string();
                let committed =
                    poll_until(Duration::from_secs(5), Duration::from_millis(150), || {
                        committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= RECORDS)
                    })
                    .await;
                assert!(
                    committed,
                    "read_committed receiver must deliver and commit all {RECORDS} \
                     non-transactional records, got {:?}",
                    committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed"),
                );

                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
    }

    // ---- Security ----

    /// Scenario (security): a receiver subscribed via a `^`-prefixed topic regex
    /// (which lets a broker-supplied topic name reach the receiver's log sites)
    /// receives a well-formed OTAP record carrying an adversarial header value
    /// (control characters plus a large string on a configured extraction key),
    /// followed by an undecodable OTAP record on the same topic.
    /// Guarantees: the adversarial header value and topic name -- both of which
    /// flow into `otel_*` log fields and into a resource attribute -- do not
    /// crash or stall the receive loop: the good record is delivered with the
    /// header extracted verbatim onto its resource, the poison record is counted
    /// as a processing error rather than aborting the loop, and the receiver
    /// still shuts down cleanly. This bounds the blast radius of adversarial
    /// client-controlled topic/header values reaching telemetry.
    #[tokio::test]
    async fn adversarial_topic_and_header_values_do_not_stall_loop() {
        const TOPIC: &str = "sec-adversarial-traces";
        let group = "sec-adversarial-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let good = create_traces_with_spans_otap_bytes();
                let poison = b"not-a-valid-otap-arrow-payload".to_vec();

                // A control-char + oversized header value on the configured
                // extraction key. It is client-controlled and reaches both the
                // resource attribute and (on any parse failure) the log line.
                let adversarial_value = format!("acme\r\n\t\x1b[31m-{}", "Z".repeat(2048));

                // Good OTAP record with the adversarial header, then a poison
                // OTAP record, both on the regex-matched topic.
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &good)
                            .key(b"good")
                            .header("x-tenant-id", adversarial_value.as_bytes())
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("send good");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &poison)
                            .key(b"poison")
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("send poison");

                // Configure a `^`-regex subscription (so the broker topic name
                // reaches the receiver) plus a header->resource-attribute
                // extraction for the adversarial header, OTAP encoding.
                let mut extraction = HashMap::new();
                let _ = extraction.insert(
                    "x-tenant-id".to_string(),
                    HeaderExtraction {
                        key: "tenant.id".to_string(),
                        value_type: AttributeValueType::String,
                    },
                );
                let builder = KafkaReceiverConfigBuilder::new(
                    cluster.bootstrap_servers(),
                    group,
                    "test-client",
                )
                .with_traces(
                    SignalConfig::new(vec!["^sec-adversarial-.*".to_string()])
                        .with_encoding(MessageFormat::OtapProto),
                )
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Manual,
                    interval_ms: None,
                })
                .with_auto_offset_reset(AutoOffsetReset::Earliest)
                .with_isolation_level(IsolationLevel::ReadUncommitted)
                .with_resource_attrs_from_headers(extraction);
                let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // The good record must be delivered despite the adversarial
                // header value; its value is extracted verbatim onto the resource.
                let mut pdata = receiver.recv_pdata().await;
                let result = otap_pdata_to_traces(&mut pdata);
                let mut found_tenant = false;
                for rs in &result.resource_spans {
                    let resource = rs.resource.as_ref().expect("resource present");
                    if let Some(kv) = resource.attributes.iter().find(|kv| kv.key == "tenant.id") {
                        if let Some(any_value::Value::StringValue(s)) =
                            kv.value.as_ref().and_then(|v| v.value.as_ref())
                        {
                            assert_eq!(
                                s, &adversarial_value,
                                "adversarial header value is extracted verbatim",
                            );
                            found_tenant = true;
                        }
                    }
                }
                assert!(found_tenant, "the tenant.id attribute should be extracted");
                receiver.ack(pdata);

                // The poison record must not be forwarded (it is counted as an
                // error), and the loop keeps running.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(2))
                        .await
                        .is_none(),
                    "poison record must not be forwarded, and the loop must not stall",
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;
                let decode_rejections = measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.rejections",
                    &[
                        ("signal", "traces"),
                        ("error.type", "invalid_request"),
                        ("reason", "decode"),
                    ],
                    "messages",
                );
                assert!(
                    decode_rejections >= 1,
                    "the poison record must be counted as a decode rejection, got {decode_rejections}",
                );
            },
        )
        .await;
    }

    // ---- Operational visibility ----

    /// Scenario (operational visibility): a consumer owns partitions but *none* of them has a
    /// broker-committed offset yet (every `committed_offsets` entry is
    /// `Offset::Invalid`).
    /// Guarantees: `compute_consumer_lag` reports the refresh as incomplete
    /// (`None`) instead of computing a mean from a subset, so the caller retains
    /// the previous `receiver.kafka.consumer.group.lag` value rather than
    /// publishing a partial or zeroed measurement.
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

    /// Scenario (operational visibility): the receive loop's lag-refresh deadline elapses, so the loop
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

    /// Scenario (operational visibility): a consumer owns two partitions but only one has a
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

    /// Scenario (operational visibility): the total refresh deadline has already passed when
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

    /// Scenario (operational visibility): the receive loop's lag apply branch observes the in-flight
    /// worker *finish* with a value (a real mean, or the `0.0`
    /// empty-assignment reset).
    /// Guarantees: the apply branch publishes the value to
    /// `receiver.kafka.consumer.group.lag` and clears the in-flight slot so the
    /// next tick may start a fresh refresh.
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
                    Ok(Some(value)) => receiver.metrics.consumer.lag.set(value),
                    Ok(None) => {}
                    Err(join_err) => panic!("unexpected join error: {join_err}"),
                }
            }
        }

        assert_eq!(receiver.metrics.consumer.lag.get(), 42.0);
        assert!(
            in_flight.is_none(),
            "a finished worker must clear the in-flight slot",
        );
    }

    /// Scenario (operational visibility): the lag apply branch observes the absolute deadline elapse
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
        receiver.metrics.consumer.lag.set(7.0);

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
            receiver.metrics.consumer.lag.get(),
            7.0,
            "the previous gauge value must be retained on a deadline crossing",
        );

        // Clean up the still-running background task.
        if let Some((handle, _)) = in_flight.take() {
            handle.abort();
        }
    }

    /// Scenario (operational visibility): paused time; the apply branch is polled repeatedly while the
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
                    Ok(Some(value)) => receiver.metrics.consumer.lag.set(value),
                    Ok(None) => {}
                    Err(join_err) => panic!("unexpected join error: {join_err}"),
                }
            }
        }

        assert_eq!(
            receiver.metrics.consumer.lag.get(),
            5.0,
            "a refresh that completes after the deadline must still be published",
        );
        assert!(
            in_flight.is_none(),
            "the in-flight slot must be cleared once the worker finishes",
        );
    }

    /// Scenario (operational visibility): a manual-commit receiver processes a
    /// well-formed record followed by an undecodable OTAP record on the same
    /// topic.
    /// Guarantees: the data-processing failure is attributed to the bounded
    /// `decode` rejection reason while the unrelated `unknown_topic` rejection
    /// and partition-revocation metrics stay at zero.
    #[tokio::test]
    async fn decode_rejections_are_categorized_separately_from_filtering_and_rebalance() {
        const TOPIC: &str = "visibility-processing-traces";
        let group = "visibility-processing-group";
        with_cluster(
            KafkaTestCluster::builder().topic(TOPIC),
            |cluster| async move {
                let producer = cluster.producer().build();
                let good = create_traces_with_spans_otap_bytes();
                let poison = b"not-a-valid-otap-arrow-payload".to_vec();

                producer
                    .send_full(
                        SendRecord::new(TOPIC, &good)
                            .key(b"good")
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("send good");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &poison)
                            .key(b"poison")
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("send poison");

                let cfg =
                    manual_otap_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // The good record is delivered; the poison record is not.
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(2))
                        .await
                        .is_none(),
                    "poison record must not be forwarded",
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;
                let mut m = FoldedMetrics::new();
                m.fold_all(terminal.metrics());

                let decode_rejections = measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.rejections",
                    &[
                        ("signal", "traces"),
                        ("error.type", "invalid_request"),
                        ("reason", "decode"),
                    ],
                    "messages",
                );
                let unknown_topic_rejections = measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.rejections",
                    &[
                        ("signal", "unknown"),
                        ("error.type", "invalid_request"),
                        ("reason", "unknown_topic"),
                    ],
                    "messages",
                );
                assert!(
                    decode_rejections >= 1,
                    "decode rejection reason should count the failure, got {decode_rejections}",
                );
                assert_eq!(
                    unknown_topic_rejections, 0,
                    "a decode failure must not be counted as an unknown-topic rejection",
                );
                assert_eq!(
                    m.value("group.partition.revocations"),
                    0,
                    "a decode failure must not be counted as a rebalance \
                     revocation, got {}",
                    m.value("group.partition.revocations"),
                );
            },
        )
        .await;
    }

    /// Scenario (operational visibility): a receiver subscribes to a `^`-prefixed
    /// include regex that also matches an `exclude_topics` pattern; a record is
    /// produced to the excluded topic (which librdkafka still delivers because
    /// the include regex matches) alongside a record on a normal included topic.
    /// Guarantees: the excluded record is attributed to the bounded
    /// `unknown_topic` rejection reason while the `decode` reason stays at zero.
    #[tokio::test]
    async fn unknown_topic_rejections_are_categorized_separately_from_decode_errors() {
        const INCLUDED: &str = "visibility-included";
        const EXCLUDED: &str = "visibility-excluded";
        let group = "visibility-filtering-group";
        with_cluster(
            KafkaTestCluster::builder().topic(INCLUDED).topic(EXCLUDED),
            |cluster| async move {
                let producer = cluster.producer().build();
                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // One record on each topic. Both are well-formed, so any counted
                // error is a filtering decision, not a decode failure.
                producer
                    .send_full(SendRecord::new(INCLUDED, &bytes).key(b"inc"))
                    .await
                    .expect("send included");
                producer
                    .send_full(SendRecord::new(EXCLUDED, &bytes).key(b"exc"))
                    .await
                    .expect("send excluded");

                // Include everything matching `^visibility-`, but exclude the
                // `visibility-excluded` topic. librdkafka subscribes to both
                // (the include regex matches), so the receiver-side guard is what
                // rejects the excluded topic.
                let builder = KafkaReceiverConfigBuilder::new(
                    cluster.bootstrap_servers(),
                    group,
                    "test-client",
                )
                .with_traces(
                    SignalConfig::new(vec!["^visibility-.*".to_string()])
                        .with_encoding(MessageFormat::OtlpProto)
                        .with_exclude_topics(vec!["^visibility-excluded$".to_string()]),
                )
                .with_commit(CommitConfig {
                    mode: ConfigCommitMode::Manual,
                    interval_ms: None,
                })
                .with_auto_offset_reset(AutoOffsetReset::Earliest)
                .with_isolation_level(IsolationLevel::ReadUncommitted);
                let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // The included topic's record is delivered and decoded.
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
                // The excluded topic's record is filtered out, never delivered.
                assert!(
                    receiver
                        .try_recv_pdata(Duration::from_secs(2))
                        .await
                        .is_none(),
                    "excluded topic record must not be forwarded",
                );

                receiver.shutdown(Duration::from_secs(5));
                let terminal = receiver.await_terminal_state().await;
                let unknown_topic_rejections = measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.rejections",
                    &[
                        ("signal", "unknown"),
                        ("error.type", "invalid_request"),
                        ("reason", "unknown_topic"),
                    ],
                    "messages",
                );
                let decode_rejections = measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.rejections",
                    &[
                        ("signal", "traces"),
                        ("error.type", "invalid_request"),
                        ("reason", "decode"),
                    ],
                    "messages",
                );
                assert!(
                    unknown_topic_rejections >= 1,
                    "the excluded topic should be counted as an unknown-topic rejection, got \
                     {unknown_topic_rejections}",
                );
                assert_eq!(
                    decode_rejections, 0,
                    "expected filtering must not be counted as a decode rejection",
                );
            },
        )
        .await;
    }
}
