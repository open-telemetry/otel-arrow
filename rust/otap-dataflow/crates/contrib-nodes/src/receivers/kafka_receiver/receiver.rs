// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// ToDo: update tests to start broker in memory
// ToDo: Possible optimization to improve how we determine signal type from a message
//.      check every message against list of topics + excluded topics to get signal type

use super::config::{HeaderExtraction, KafkaReceiverConfig};
use super::errors::DecodeError;
use super::headers::HeaderExtractions;
use super::metrics::KafkaReceiverMetrics;
use super::offset_tracker::OffsetTracker;
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
use std::time::Duration;

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

/// Dynamically assigns compact `u8` IDs to actual Kafka topic names.
///
/// Used to encode topic identity into [`CallData`] for Ack/Nack routing
/// while supporting regex-matched topic names that aren't known at config
/// time.
struct TopicRegistry {
    name_to_id: HashMap<String, u8>,
    id_to_name: Vec<String>,
}

impl TopicRegistry {
    fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
        }
    }

    /// Get or assign a `u8` ID for the given topic name.
    fn get_or_assign(&mut self, topic: &str) -> u8 {
        // if topic hasn't been seen yet then we assign topic a id
        if let Some(&id) = self.name_to_id.get(topic) {
            return id;
        }
        // get id for a new topic
        let id = self.id_to_name.len() as u8;
        let owned = topic.to_string();
        self.id_to_name.push(owned.clone());
        let _ = self.name_to_id.insert(owned, id);
        id
    }

    /// Look up a topic name by its assigned ID.
    fn name_for(&self, id: u8) -> Option<&str> {
        self.id_to_name.get(id as usize).map(|s| s.as_str())
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
    /// Dynamically assigns `u8` IDs to actual topic names for CallData encoding.
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
        config: KafkaReceiverConfig,
    ) -> Result<Self, ConfigError> {
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

        Ok(Self {
            config,
            metrics,
            offset_tracker: OffsetTracker::new(),
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
    /// Updates the offset tracker's internal [`TopicPartitionList`] in-place
    /// and commits synchronously. Only commits when auto-commit is disabled.
    fn commit_offsets<C: ConsumerContext>(
        &mut self,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
    ) -> Result<(), EngineError> {
        if self.config.is_auto_commit() {
            return Ok(());
        }
        let tpl = self.offset_tracker.committable_tpl();
        if tpl.count() == 0 {
            return Ok(());
        }
        match consumer.commit(tpl, CommitMode::Sync) {
            Ok(()) => {
                self.metrics.offset_commits.add(1);
                Ok(())
            }
            Err(e) => {
                self.metrics.offset_commit_errors.add(1);
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

        loop {
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
                            let snapshot = self.metrics.snapshot();
                            _ = telemetry_cancel_handle.cancel().await;
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        },
                        Ok(NodeControlMsg::Ack(ack_msg)) => {
                            self.metrics.acks_received.add(1);
                            if manual_commit && !ack_msg.unwind.route.calldata.is_empty() {
                                let (topic_id, partition, offset) =
                                    decode_calldata(&ack_msg.unwind.route.calldata);
                                // Resolve the dynamic topic ID back to
                                // the actual topic name via the registry.
                                if let Some(name) = self.topic_registry.name_for(topic_id) {
                                    let advanced = self
                                        .offset_tracker
                                        .acknowledge(name, partition, offset);
                                    if advanced {
                                        self.commit_offsets(&consumer, &receiver_id)?;
                                    }
                                }
                            }
                        },
                        Ok(NodeControlMsg::Nack(nack_msg)) => {
                            self.metrics.nacks_received.add(1);
                            // Treat nack as ack (advance past failed message).
                            // TODO: future work — retry logic, DLQ
                            if manual_commit && !nack_msg.unwind.route.calldata.is_empty() {
                                let (topic_id, partition, offset) =
                                    decode_calldata(&nack_msg.unwind.route.calldata);
                                if let Some(name) = self.topic_registry.name_for(topic_id) {
                                    let advanced = self
                                        .offset_tracker
                                        .acknowledge(name, partition, offset);
                                    if advanced {
                                        self.commit_offsets(&consumer, &receiver_id)?;
                                    }
                                }
                            }
                        },
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            // Report current receiver metrics.
                            _ = metrics_reporter.report(&mut self.metrics);
                        },
                        Ok(NodeControlMsg::TimerTick { .. }) => {
                            // Periodic safety-net commit: flush any committable
                            // offsets that haven't been committed via ack/nack yet.
                            self.commit_offsets(&consumer, &receiver_id)?;
                        },
                        Err(e) => {
                            return Err(EngineError::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message — do nothing
                        }
                    }
                }

                // 2. Consume Kafka messages
                result = consumer.recv() => {
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

                            // Assign a compact u8 ID for this actual topic name.
                            // The registry remembers the mapping for Ack/Nack lookup.
                            let topic_id =
                                self.topic_registry.get_or_assign(&topic);

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
                                        // Track offset as in-flight
                                        self.offset_tracker
                                            .track(&topic, partition, offset);
                                        // Subscribe so Ack/Nack carries
                                        // offset identity back to us
                                        let calldata =
                                            encode_calldata(topic_id, partition, offset);
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
                                        self.offset_tracker
                                            .track(&topic, partition, offset);
                                        let advanced = self
                                            .offset_tracker
                                            .acknowledge(&topic, partition, offset);
                                        if advanced {
                                            self.commit_offsets(
                                                &consumer,
                                                &receiver_id,
                                            )?;
                                        }
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
fn encode_calldata(topic_id: u8, partition: i32, offset: i64) -> CallData {
    let topic_partition = ((topic_id as u64) << 32) | (partition as u32 as u64);
    smallvec![
        Context8u8::from(topic_partition),
        Context8u8::from(offset as u64),
    ]
}

/// Decode Kafka message identity from [`CallData`] returned in Ack/Nack.
fn decode_calldata(calldata: &CallData) -> (u8, i32, i64) {
    let topic_partition: u64 = calldata[0].into();
    let topic_id = (topic_partition >> 32) as u8;
    let partition = (topic_partition & 0xFFFF_FFFF) as i32;
    let offset: u64 = calldata[1].into();
    (topic_id, partition, offset as i64)
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

        // Build the Kafka consumer with the appropriate client context.
        // When the `aws` feature is enabled, check whether AWS MSK IAM
        // authentication is configured and, if so, create a consumer with
        // the custom OAUTHBEARER token-refresh context.
        #[cfg(feature = "aws")]
        if let Some(client_ctx) = build_aws_msk_context(self.config.auth()) {
            let consumer = client_config
                .create_with_context(client_ctx)
                .map_err(map_kafka_client_err)?;
            return self
                .as_mut()
                .run_receive_loop(ctrl_msg_recv, effect_handler, consumer)
                .await;
        }

        let consumer = client_config.create().map_err(map_kafka_client_err)?;
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
        HeaderExtraction, IsolationLevel, KafkaReceiverConfigBuilder, SignalConfig,
    };

    use crate::common::kafka::MessageFormat;
    use otap_df_channel::mpsc;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::runtime_ctrl_msg_channel;
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::local::receiver::Receiver as _;
    use otap_df_engine::message::{Receiver, Sender};
    use otap_df_engine::testing::test_node;
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
    use otap_df_telemetry::reporter::MetricsReporter;
    use prost::Message;
    use rdkafka::ClientConfig;
    use rdkafka::message::{Header, OwnedHeaders};
    use rdkafka::producer::{FutureProducer, FutureRecord};
    use rdkafka::util::Timeout;
    use std::collections::HashMap;
    use std::time::Duration;
    use testcontainers::core::{IntoContainerPort, WaitFor};
    use testcontainers::runners::AsyncRunner;
    use testcontainers::{ContainerAsync, GenericImage, ImageExt};
    use tokio::task::LocalSet;
    use tokio::time::timeout;

    fn create_test_producer(brokers: &str) -> FutureProducer {
        ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "20000")
            .create()
            .expect("Failed to create producer")
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

    /// Opaque bundle of channel handles whose lifetimes keep the test
    /// receiver running. Drop this to tear down all channels.
    #[allow(dead_code)]
    struct KeepAlive(Vec<Box<dyn std::any::Any>>);

    // ---- Testcontainers Kafka broker helper ----

    /// Starts a Kafka broker in Docker via testcontainers (KRaft mode, no ZooKeeper).
    ///
    /// Returns the container handle (must stay alive for the broker to remain
    /// running) and the broker address string (`127.0.0.1:<port>`).
    #[allow(dead_code)]
    async fn start_kafka_container() -> (ContainerAsync<GenericImage>, String) {
        let host_port = portpicker::pick_unused_port().expect("no free port available");

        let container = GenericImage::new("apache/kafka", "4.1.0")
            .with_wait_for(WaitFor::message_on_stdout("Kafka Server started"))
            .with_mapped_port(host_port, 9092.tcp())
            .with_env_var("KAFKA_NODE_ID", "1")
            .with_env_var("KAFKA_PROCESS_ROLES", "broker,controller")
            .with_env_var(
                "KAFKA_LISTENERS",
                "PLAINTEXT://0.0.0.0:9092,CONTROLLER://0.0.0.0:9093",
            )
            .with_env_var(
                "KAFKA_ADVERTISED_LISTENERS",
                format!("PLAINTEXT://127.0.0.1:{host_port}"),
            )
            .with_env_var("KAFKA_CONTROLLER_LISTENER_NAMES", "CONTROLLER")
            .with_env_var(
                "KAFKA_LISTENER_SECURITY_PROTOCOL_MAP",
                "CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT",
            )
            .with_env_var("KAFKA_CONTROLLER_QUORUM_VOTERS", "1@localhost:9093")
            .with_env_var("KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR", "1")
            .with_env_var("KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR", "1")
            .with_env_var("KAFKA_TRANSACTION_STATE_LOG_MIN_ISR", "1")
            .with_env_var("KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS", "0")
            .with_env_var("KAFKA_NUM_PARTITIONS", "1")
            .start()
            .await
            .expect("Failed to start Kafka container");

        let broker_addr = format!("127.0.0.1:{host_port}");

        (container, broker_addr)
    }

    /// Creates a [`KafkaReceiver`] with all the engine wiring (control channel,
    /// pdata channel, effect handler) needed to run it in a test.
    ///
    /// Returns the boxed receiver, the control channel, the effect handler,
    /// and the pdata receiver channel from which consumed messages can be read.
    #[allow(dead_code)]
    fn setup_receiver_harness(
        brokers: &str,
        traces_topics: &[&str],
        metrics_topics: &[&str],
        logs_topics: &[&str],
        msg_format: MessageFormat,
    ) -> (
        Box<KafkaReceiver>,
        local::ControlChannel<OtapPdata>,
        local::EffectHandler<OtapPdata>,
        Receiver<OtapPdata>,
        KeepAlive,
    ) {
        setup_receiver_harness_with_headers(
            brokers,
            traces_topics,
            metrics_topics,
            logs_topics,
            msg_format,
            HashMap::new(),
        )
    }

    /// Like [`setup_receiver_harness`] but also accepts a header extraction
    /// configuration so that Kafka message headers are mapped to span
    /// trace-ids and/or attributes.
    #[allow(dead_code)]
    fn setup_receiver_harness_with_headers(
        brokers: &str,
        traces_topics: &[&str],
        metrics_topics: &[&str],
        logs_topics: &[&str],
        msg_format: MessageFormat,
        resource_attrs_from_headers: HashMap<String, HeaderExtraction>,
    ) -> (
        Box<KafkaReceiver>,
        local::ControlChannel<OtapPdata>,
        local::EffectHandler<OtapPdata>,
        Receiver<OtapPdata>,
        KeepAlive,
    ) {
        let kafka_config = KafkaReceiverConfig::try_from(
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
        .expect("test config should be valid");

        let pipeline_ctx = make_pipeline_ctx();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(KAFKA_RECEIVER_URN));
        let receiver = Box::new(
            KafkaReceiver::new(pipeline_ctx, kafka_config).expect("kafka receiver config is valid"),
        );

        let (control_sender, control_receiver) = mpsc::Channel::new(32);
        let control_receiver = LocalReceiver::mpsc(control_receiver);
        let ctrl_msg_chan = local::ControlChannel::new(Receiver::Local(control_receiver));

        let mut pdata_senders = HashMap::new();
        let (sender, recv) = mpsc::Channel::new(32);
        let pdata_sender = Sender::Local(LocalSender::mpsc(sender));
        let pdata_receiver = Receiver::Local(LocalReceiver::mpsc(recv));
        let _ = pdata_senders.insert(std::borrow::Cow::Borrowed("test_receiver"), pdata_sender);

        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = runtime_ctrl_msg_channel(10);
        let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let effect_handler = local::EffectHandler::new(
            test_node("test_receiver"),
            pdata_senders,
            node_config.default_output.clone(),
            pipeline_ctrl_msg_tx,
            metrics_reporter,
        );

        let keep_alive = KeepAlive(vec![
            Box::new(control_sender),
            Box::new(pipeline_ctrl_msg_rx),
            Box::new(metrics_rx),
        ]);
        (
            receiver,
            ctrl_msg_chan,
            effect_handler,
            pdata_receiver,
            keep_alive,
        )
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
        let registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry);
        controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0)
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

    // ---- Integration tests (Kafka broker via testcontainers) ----
    // These tests require Docker and are skipped by default in CI.
    // Run locally with: cargo test -- --ignored

    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_traces() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let req = create_traces_with_spans();
        let mut bytes = vec![];
        req.encode(&mut bytes).expect("encode");

        for i in 0..3 {
            let _ = producer
                .send(
                    FutureRecord::to("test-traces-proto")
                        .payload(&bytes)
                        .key(&format!("test-key-{i}")),
                    Timeout::After(Duration::from_secs(10)),
                )
                .await
                .expect("Failed to send message");
        }

        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) = setup_receiver_harness(
            &brokers,
            &["test-traces-proto"],
            &[],
            &[],
            MessageFormat::OtlpProto,
        );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                for i in 0..3 {
                    let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                        .await
                        .unwrap_or_else(|_| panic!("Timed out waiting for trace message {i}"))
                        .unwrap_or_else(|_| panic!("No trace message received for {i}"));

                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    assert!(matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)));
                    assert_eq!(proto.as_bytes(), &bytes);
                }
            })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_logs() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let req = create_logs_service_request();
        let mut bytes = vec![];
        req.encode(&mut bytes).expect("encode");

        for i in 0..3 {
            let _ = producer
                .send(
                    FutureRecord::to("test-logs-proto")
                        .payload(&bytes)
                        .key(&format!("test-key-{i}")),
                    Timeout::After(Duration::from_secs(10)),
                )
                .await
                .expect("Failed to send message");
        }

        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) = setup_receiver_harness(
            &brokers,
            &[],
            &[],
            &["test-logs-proto"],
            MessageFormat::OtlpProto,
        );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                for i in 0..3 {
                    let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                        .await
                        .unwrap_or_else(|_| panic!("Timed out waiting for log message {i}"))
                        .unwrap_or_else(|_| panic!("No log message received for {i}"));

                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    assert!(matches!(proto, OtlpProtoBytes::ExportLogsRequest(_)));
                    assert_eq!(proto.as_bytes(), &bytes);
                }
            })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_metrics() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let req = create_metrics_service_request();
        let mut bytes = vec![];
        req.encode(&mut bytes).expect("encode");

        for i in 0..3 {
            let _ = producer
                .send(
                    FutureRecord::to("test-metrics-proto")
                        .payload(&bytes)
                        .key(&format!("test-key-{i}")),
                    Timeout::After(Duration::from_secs(10)),
                )
                .await
                .expect("Failed to send message");
        }

        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) = setup_receiver_harness(
            &brokers,
            &[],
            &["test-metrics-proto"],
            &[],
            MessageFormat::OtlpProto,
        );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                for i in 0..3 {
                    let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                        .await
                        .unwrap_or_else(|_| panic!("Timed out waiting for metric message {i}"))
                        .unwrap_or_else(|_| panic!("No metric message received for {i}"));

                    let proto: OtlpProtoBytes = pdata
                        .take_payload()
                        .try_into_with_default()
                        .expect("to OtlpProtoBytes");
                    assert!(matches!(proto, OtlpProtoBytes::ExportMetricsRequest(_)));
                    assert_eq!(proto.as_bytes(), &bytes);
                }
            })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_traces_otap() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let bytes = create_traces_with_spans_otap_bytes();

        for i in 0..3 {
            let _ = producer
                .send(
                    FutureRecord::to("test-traces-otap")
                        .payload(&bytes)
                        .key(&format!("test-key-{i}")),
                    Timeout::After(Duration::from_secs(10)),
                )
                .await
                .expect("Failed to send message");
        }

        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) = setup_receiver_harness(
            &brokers,
            &["test-traces-otap"],
            &[],
            &[],
            MessageFormat::OtapProto,
        );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                for i in 0..3 {
                    let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                        .await
                        .unwrap_or_else(|_| panic!("Timed out waiting for trace message {i}"))
                        .unwrap_or_else(|_| panic!("No trace message received for {i}"));

                    let payload: OtapPayload = pdata.take_payload();
                    assert!(
                        matches!(
                            payload,
                            OtapPayload::OtapArrowRecords(OtapArrowRecords::Traces(_))
                        ),
                        "Expected OtapArrowRecords::Traces for message {i}"
                    );
                }
            })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_metrics_otap() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let bytes = create_metrics_otap_arrow_records_bytes();

        for i in 0..3 {
            let _ = producer
                .send(
                    FutureRecord::to("test-metrics-otap")
                        .payload(&bytes)
                        .key(&format!("test-key-{i}")),
                    Timeout::After(Duration::from_secs(10)),
                )
                .await
                .expect("Failed to send message");
        }

        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) = setup_receiver_harness(
            &brokers,
            &[],
            &["test-metrics-otap"],
            &[],
            MessageFormat::OtapProto,
        );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                for i in 0..3 {
                    let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                        .await
                        .unwrap_or_else(|_| panic!("Timed out waiting for metric message {i}"))
                        .unwrap_or_else(|_| panic!("No metric message received for {i}"));

                    let payload: OtapPayload = pdata.take_payload();
                    if let OtapPayload::OtapArrowRecords(arrow_records) = payload {
                        let expected = OtapArrowRecords::Metrics(Metrics::default());
                        assert_eq!(expected, arrow_records);
                    } else {
                        panic!("Expected OtapArrowRecords::Metrics for message {i}");
                    }
                }
            })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_logs_otap() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let bytes = create_logs_otap_arrow_records_bytes();

        for i in 0..3 {
            let _ = producer
                .send(
                    FutureRecord::to("test-logs-otap")
                        .payload(&bytes)
                        .key(&format!("test-key-{i}")),
                    Timeout::After(Duration::from_secs(10)),
                )
                .await
                .expect("Failed to send message");
        }

        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) = setup_receiver_harness(
            &brokers,
            &[],
            &[],
            &["test-logs-otap"],
            MessageFormat::OtapProto,
        );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                for i in 0..3 {
                    let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                        .await
                        .unwrap_or_else(|_| panic!("Timed out waiting for log message {i}"))
                        .unwrap_or_else(|_| panic!("No log message received for {i}"));

                    let payload: OtapPayload = pdata.take_payload();
                    if let OtapPayload::OtapArrowRecords(arrow_records) = payload {
                        let expected = OtapArrowRecords::Logs(Logs::default());
                        assert_eq!(expected, arrow_records);
                    } else {
                        panic!("Expected OtapArrowRecords::Logs for message {i}");
                    }
                }
            })
            .await;
    }

    // ---- Header extraction integration tests (testcontainers) ----

    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_traces_header_extraction() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

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
            let headers = OwnedHeaders::new().insert(Header {
                key: "x-tenant-id",
                value: Some(tenant_value.as_bytes()),
            });

            let _ = producer
                .send(
                    FutureRecord::to("test-traces-headers")
                        .payload(&payload_bytes)
                        .key(&format!("test-key-{i}"))
                        .headers(headers),
                    Timeout::After(Duration::from_secs(10)),
                )
                .await
                .expect("Failed to send message");
        }

        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) =
            setup_receiver_harness_with_headers(
                &brokers,
                &["test-traces-headers"],
                &[],
                &[],
                MessageFormat::OtlpProto,
                resource_attrs_from_headers,
            );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                for i in 0..3 {
                    let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                        .await
                        .unwrap_or_else(|_| panic!("Timed out waiting for trace message {i}"))
                        .unwrap_or_else(|_| panic!("No trace message received for {i}"));

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
            })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_traces_header_extraction_otap() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

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
            let headers = OwnedHeaders::new()
                .insert(Header {
                    key: "x-tenant-id",
                    value: Some(tenant_value.as_bytes()),
                })
                .insert(Header {
                    key: "MessageFormat",
                    value: Some(MSG_FORMAT_OTAP),
                });

            let _ = producer
                .send(
                    FutureRecord::to("test-traces-headers-otap")
                        .payload(&otap_bytes)
                        .key(&format!("test-key-{i}"))
                        .headers(headers),
                    Timeout::After(Duration::from_secs(10)),
                )
                .await
                .expect("Failed to send message");
        }

        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) =
            setup_receiver_harness_with_headers(
                &brokers,
                &["test-traces-headers-otap"],
                &[],
                &[],
                MessageFormat::OtapProto,
                resource_attrs_from_headers,
            );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                for i in 0..3 {
                    let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                        .await
                        .unwrap_or_else(|_| panic!("Timed out waiting for trace message {i}"))
                        .unwrap_or_else(|_| panic!("No trace message received for {i}"));

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
            })
            .await;
    }

    // ---- CallData encode/decode roundtrip tests ----

    #[test]
    fn encode_decode_calldata_roundtrip() {
        let cases: Vec<(u8, i32, i64)> = vec![
            (0, 0, 0),
            (0, 0, 100),
            (1, 3, 999_999),
            (2, 11, i64::MAX),
            (5, 0, 42),
            (10, 1, 1_000_000),
            (255, 2, 0),
        ];

        for (topic_id, partition, offset) in cases {
            let calldata = encode_calldata(topic_id, partition, offset);
            let (dec_tid, dec_part, dec_off) = decode_calldata(&calldata);
            assert_eq!(dec_tid, topic_id, "topic_id mismatch");
            assert_eq!(dec_part, partition, "partition mismatch");
            assert_eq!(dec_off, offset, "offset mismatch");
        }
    }

    #[test]
    fn encode_calldata_produces_two_slots() {
        let calldata = encode_calldata(1, 5, 42);
        assert_eq!(calldata.len(), 2);
    }

    // ---- TopicRegistry tests ----

    #[test]
    fn topic_registry_assigns_sequential_ids() {
        let mut reg = TopicRegistry::new();

        assert_eq!(reg.get_or_assign("traces-prod"), 0);
        assert_eq!(reg.get_or_assign("metrics-prod"), 1);
        assert_eq!(reg.get_or_assign("logs-prod"), 2);

        // Same topic returns the same ID.
        assert_eq!(reg.get_or_assign("traces-prod"), 0);
    }

    #[test]
    fn topic_registry_name_for_roundtrip() {
        let mut reg = TopicRegistry::new();

        let id = reg.get_or_assign("my-topic");
        assert_eq!(reg.name_for(id), Some("my-topic"));
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
        // Unbalanced parenthesis is an invalid regex — rejected at config validation time
        let result = KafkaReceiverConfig::try_from(
            KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
                .with_traces(SignalConfig::new(vec!["^traces-(".to_string()])),
        );
        assert!(
            result.is_err(),
            "invalid regex should fail at config construction"
        );
    }

    // ---- Transport header capture policy integration tests (testcontainers) ----

    /// Verifies that when a capture policy is configured, matching Kafka message
    /// headers are captured into the OtapPdata context as TransportHeaders.
    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_capture_policy_captures_headers() {
        use otap_df_config::transport_headers_policy::{CaptureDefaults, CaptureRule};

        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let req = create_traces_with_spans();
        let mut payload_bytes = vec![];
        req.encode(&mut payload_bytes).expect("encode");

        // Send a message with Kafka headers.
        let headers = OwnedHeaders::new()
            .insert(Header {
                key: "X-Tenant-Id",
                value: Some(b"acme-corp"),
            })
            .insert(Header {
                key: "X-Request-Id",
                value: Some(b"req-12345"),
            })
            .insert(Header {
                key: "X-Unrelated",
                value: Some(b"ignored"),
            });

        let _ = producer
            .send(
                FutureRecord::to("test-capture-policy")
                    .payload(&payload_bytes)
                    .key("key-1")
                    .headers(headers),
                Timeout::After(Duration::from_secs(10)),
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

        let (receiver, ctrl_chan, mut effect_handler, mut pdata_rx, _handles) =
            setup_receiver_harness(
                &brokers,
                &["test-capture-policy"],
                &[],
                &[],
                MessageFormat::OtlpProto,
            );

        // Install the capture policy on the effect handler.
        effect_handler.set_capture_policy(Some(capture_policy));

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                let pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

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
            })
            .await;
    }

    /// Verifies that when no capture policy is configured, transport headers
    /// are not set on the OtapPdata context (existing behavior is preserved).
    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_no_capture_policy_no_transport_headers() {
        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let req = create_traces_with_spans();
        let mut payload_bytes = vec![];
        req.encode(&mut payload_bytes).expect("encode");

        // Send a message with headers, but without a capture policy.
        let headers = OwnedHeaders::new().insert(Header {
            key: "X-Tenant-Id",
            value: Some(b"acme-corp"),
        });

        let _ = producer
            .send(
                FutureRecord::to("test-no-capture-policy")
                    .payload(&payload_bytes)
                    .key("key-1")
                    .headers(headers),
                Timeout::After(Duration::from_secs(10)),
            )
            .await
            .expect("Failed to send message");

        // No capture policy set on the effect handler.
        let (receiver, ctrl_chan, effect_handler, mut pdata_rx, _handles) = setup_receiver_harness(
            &brokers,
            &["test-no-capture-policy"],
            &[],
            &[],
            MessageFormat::OtlpProto,
        );

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                let pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Transport headers should NOT be set when no capture policy is configured.
                assert!(
                    pdata.transport_headers().is_none(),
                    "transport_headers should be None when no capture policy is configured"
                );
            })
            .await;
    }

    /// Verifies that the capture policy (transport headers) and resource_attrs_from_headers
    /// (resource attribute injection) work independently and simultaneously.
    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_capture_policy_coexists_with_resource_attrs_from_headers() {
        use otap_df_config::transport_headers_policy::{CaptureDefaults, CaptureRule};

        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let req = create_traces_with_spans();
        let mut payload_bytes = vec![];
        req.encode(&mut payload_bytes).expect("encode");

        // Send a message with headers for both mechanisms.
        let headers = OwnedHeaders::new()
            .insert(Header {
                key: "X-Tenant-Id",
                value: Some(b"acme-corp"),
            })
            .insert(Header {
                key: "x-env",
                value: Some(b"production"),
            });

        let _ = producer
            .send(
                FutureRecord::to("test-capture-and-extract")
                    .payload(&payload_bytes)
                    .key("key-1")
                    .headers(headers),
                Timeout::After(Duration::from_secs(10)),
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

        let (receiver, ctrl_chan, mut effect_handler, mut pdata_rx, _handles) =
            setup_receiver_harness_with_headers(
                &brokers,
                &["test-capture-and-extract"],
                &[],
                &[],
                MessageFormat::OtlpProto,
                resource_attrs_from_headers,
            );

        effect_handler.set_capture_policy(Some(capture_policy));

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                let mut pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

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
            })
            .await;
    }

    /// Verifies that the capture policy works with OTAP Arrow format messages.
    #[tokio::test]
    #[ignore]
    async fn test_kafka_receiver_capture_policy_otap_format() {
        use otap_df_config::transport_headers_policy::{CaptureDefaults, CaptureRule};

        let (_container, brokers) = start_kafka_container().await;
        let producer = create_test_producer(&brokers);

        let otap_bytes = create_traces_with_spans_otap_bytes();

        let headers = OwnedHeaders::new()
            .insert(Header {
                key: "X-Tenant-Id",
                value: Some(b"acme-corp"),
            })
            .insert(Header {
                key: "MessageFormat",
                value: Some(MSG_FORMAT_OTAP),
            });

        let _ = producer
            .send(
                FutureRecord::to("test-capture-policy-otap")
                    .payload(&otap_bytes)
                    .key("key-1")
                    .headers(headers),
                Timeout::After(Duration::from_secs(10)),
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

        let (receiver, ctrl_chan, mut effect_handler, mut pdata_rx, _handles) =
            setup_receiver_harness(
                &brokers,
                &["test-capture-policy-otap"],
                &[],
                &[],
                MessageFormat::OtapProto,
            );

        effect_handler.set_capture_policy(Some(capture_policy));

        let local = LocalSet::new();
        local
            .run_until(async {
                let _handle = tokio::task::spawn_local(async move {
                    let _ = receiver.start(ctrl_chan, effect_handler).await;
                });

                let pdata = timeout(Duration::from_secs(30), pdata_rx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

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
            })
            .await;
    }
}
