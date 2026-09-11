// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::error::KafkaReceiverError;
use super::receiver::topics::{
    compile_exclude_regexes, compile_topic_regexes, matches_any_exclude, matches_any_topic,
};
use crate::common::kafka::auth::Auth;
use crate::common::kafka::security::{apply_sasl_config, resolve_security_protocol};
use crate::common::kafka::{
    DebugContext, LogLevel, MessageFormat, TlsConfig, debug_list_to_string,
    default_message_format_header, validate_kafka_topic,
};
use otel_arrow_dfe_config::SignalType;
use rdkafka::ClientConfig;
use regex::Regex;
use serde::Deserialize;

use std::collections::{HashMap, HashSet};

/// rdkafka configuration keys that correspond to first-class
/// [`KafkaReceiverConfig`] fields. Entries in `consumer_config` using
/// these keys may be overwritten when the receiver builds its rdkafka
/// client configuration.
pub(crate) const MANAGED_CONSUMER_CONFIG_KEYS: &[&str] = &[
    "bootstrap.servers",
    "group.id",
    "client.id",
    "group.instance.id",
    "auto.commit.interval.ms",
    "enable.auto.commit",
    "enable.auto.offset.store",
    "auto.offset.reset",
    "session.timeout.ms",
    "heartbeat.interval.ms",
    "fetch.min.bytes",
    "fetch.max.bytes",
    "fetch.wait.max.ms",
    "max.partition.fetch.bytes",
    "isolation.level",
    "security.protocol",
    "ssl.ca.location",
    "ssl.certificate.location",
    "ssl.key.location",
    "ssl.key.password",
    "enable.ssl.certificate.verification",
    "sasl.mechanism",
    "sasl.username",
    "sasl.password",
    "partition.assignment.strategy",
    "debug",
];

/// Auto offset reset behavior
#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoOffsetReset {
    /// Start from the beginning of the topic
    Earliest,
    /// Start from the end of the topic
    Latest,
    /// Throw error if no offset is found
    Error,
}

/// Consumer isolation level
#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    /// Read all messages (including uncommitted)
    ReadUncommitted,
    /// Read only committed messages
    ReadCommitted,
}

/// Commit mode for offset management.
#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitMode {
    /// Kafka auto-commits offsets periodically (at-most-once semantics).
    /// Simpler but may lose data if processing fails after commit.
    Auto,
    /// Offsets committed only after successful downstream processing
    /// (at-least-once semantics). Default and recommended for production.
    Manual,
}

/// Policy applied when a non-permanent NACK reaches the Kafka receiver.
#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransientNackMode {
    /// Treat the NACK as terminal, commit it, and skip Kafka redelivery.
    CommitAndSkip,
    /// Pause the partition and replay from Kafka after exponential backoff.
    Replay,
}

/// Configuration for non-permanent NACK handling.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransientNackConfig {
    /// Whether a non-permanent NACK is committed and skipped or replayed from Kafka.
    #[serde(default = "default_transient_nack_mode")]
    pub mode: TransientNackMode,
    /// Delay before the first Kafka replay attempt, in milliseconds.
    #[serde(default = "default_transient_nack_initial_backoff_ms")]
    pub initial_backoff_ms: u64,
    /// Maximum delay between Kafka replay attempts, in milliseconds.
    #[serde(default = "default_transient_nack_max_backoff_ms")]
    pub max_backoff_ms: u64,
}

impl Default for TransientNackConfig {
    fn default() -> Self {
        Self {
            mode: default_transient_nack_mode(),
            initial_backoff_ms: default_transient_nack_initial_backoff_ms(),
            max_backoff_ms: default_transient_nack_max_backoff_ms(),
        }
    }
}

/// Validated replay timing used by the runtime retry state machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplayBackoffConfig {
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
}

impl ReplayBackoffConfig {
    /// Delay before the first replay attempt, in milliseconds.
    #[must_use]
    pub(crate) const fn initial_backoff_ms(&self) -> u64 {
        self.initial_backoff_ms
    }

    /// Maximum delay between replay attempts, in milliseconds.
    #[must_use]
    pub(crate) const fn max_backoff_ms(&self) -> u64 {
        self.max_backoff_ms
    }
}

impl From<&TransientNackConfig> for ReplayBackoffConfig {
    fn from(config: &TransientNackConfig) -> Self {
        Self {
            initial_backoff_ms: config.initial_backoff_ms,
            max_backoff_ms: config.max_backoff_ms,
        }
    }
}

/// Effective transient-NACK behavior after commit-mode defaults are resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EffectiveTransientNackPolicy {
    /// Downstream feedback is inactive because Kafka owns offset commits.
    Inactive,
    /// A transient NACK is terminal and allows the Kafka offset to advance.
    CommitAndSkip,
    /// A transient NACK starts partition-local Kafka replay.
    Replay(ReplayBackoffConfig),
}

// DLQ-PHASE-2 (Remove): producer-only bounds; the port channel and downstream
// exporter provide backpressure, and the DLQ config is redesigned for the port.
/// Hard cap on outstanding (in-flight) DLQ deliveries. The DLQ is an
/// error-only, low-volume path, so this is intentionally small and NOT
/// user-configurable.
pub(crate) const DLQ_MAX_IN_FLIGHT: usize = 5;

// DLQ-PHASE-2 (Remove): application-level guardrail for the in-receiver
// producer/re-read; gone once the DLQ becomes an output port.
/// Fixed application-level bound (milliseconds) on a single DLQ operation: the
/// producer send-await and the re-read fetch. This is intentionally independent
/// of librdkafka's own `message.timeout.ms` (which the producer leaves at its
/// default) so a stalled broker can never hold a source offset uncommitted for
/// longer than this before the message is counted as `dlq.loss` and advanced.
pub(crate) const DLQ_OP_TIMEOUT_MS: u64 = 10_000;

/// A failure category that can be dead-lettered.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DlqCapture {
    /// The payload could not be decoded using the configured signal encoding.
    Decode,
    /// The topic did not map to a configured signal.
    UnknownTopic,
    /// The message was permanently rejected (permanent NACK) downstream.
    PermanentNack,
}

/// Returns the default DLQ capture set: every supported category.
fn default_dlq_capture() -> Vec<DlqCapture> {
    vec![
        DlqCapture::Decode,
        DlqCapture::UnknownTopic,
        DlqCapture::PermanentNack,
    ]
}

/// Per-signal DLQ topic overrides. Any omitted signal falls back to the global
/// `topic`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DlqPerSignalTopics {
    /// DLQ topic for traces.
    #[serde(default)]
    pub traces: Option<String>,
    /// DLQ topic for metrics.
    #[serde(default)]
    pub metrics: Option<String>,
    /// DLQ topic for logs.
    #[serde(default)]
    pub logs: Option<String>,
}

// DLQ-PHASE-2 (Remove): producer-only; the port's downstream exporter owns the
// DLQ connection, so this block is dropped when the DLQ config is redesigned.
/// Connection settings for the DLQ producer / re-read consumer. Any field left
/// unset defaults to the source consumer's connection.
#[derive(Clone, Debug, PartialEq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DlqConnection {
    /// DLQ broker list; defaults to the source brokers.
    #[serde(default)]
    pub brokers: Option<String>,
    /// DLQ auth; defaults to the source auth.
    #[serde(default)]
    pub auth: Option<Auth>,
    /// DLQ TLS; defaults to the source TLS.
    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

/// Dead-letter-queue configuration. Presence of this block enables the DLQ;
/// absence (the default `None`) disables it and preserves today's behavior.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DlqConfig {
    /// Global DLQ topic applied to all captured signals unless a per-signal
    /// override is set.
    #[serde(default)]
    pub topic: Option<String>,
    /// Optional per-signal topic overrides.
    #[serde(default)]
    pub per_signal: Option<DlqPerSignalTopics>,
    /// Failure categories to dead-letter. Defaults to all supported categories.
    #[serde(default = "default_dlq_capture")]
    pub capture: Vec<DlqCapture>,
    /// Connection overrides for the DLQ producer. The re-read consumer always
    /// uses the source connection, not these overrides.
    #[serde(default)]
    pub connection: Option<DlqConnection>,
}

/// Resolved DLQ configuration stored on [`KafkaReceiverConfig`]. Resolves the
/// per-signal topics and capture set once so the runtime has a single source of
/// truth.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedDlqConfig {
    pub(crate) traces_topic: Option<String>,
    pub(crate) metrics_topic: Option<String>,
    pub(crate) logs_topic: Option<String>,
    pub(crate) capture_decode: bool,
    pub(crate) capture_unknown_topic: bool,
    pub(crate) capture_permanent_nack: bool,
    pub(crate) connection: DlqConnection,
}

impl ResolvedDlqConfig {
    /// Resolve the DLQ topic for a given signal, or `None` when that signal is
    /// not routed to the DLQ.
    #[must_use]
    pub(crate) fn topic_for(&self, signal: SignalType) -> Option<&str> {
        match signal {
            SignalType::Traces => self.traces_topic.as_deref(),
            SignalType::Metrics => self.metrics_topic.as_deref(),
            SignalType::Logs => self.logs_topic.as_deref(),
        }
    }

    /// The set of distinct DLQ topics configured across all signals.
    #[must_use]
    pub(crate) fn all_topics(&self) -> Vec<&str> {
        let mut topics: Vec<&str> = [
            self.traces_topic.as_deref(),
            self.metrics_topic.as_deref(),
            self.logs_topic.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        topics.sort_unstable();
        topics.dedup();
        topics
    }
}

/// Partition assignment strategy for consumer group rebalancing.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RebalanceStrategy {
    /// Range assignor - assigns partitions per topic in contiguous ranges.
    Range,
    /// Round-robin assignor - distributes partitions across consumers evenly.
    RoundRobin,
    /// Cooperative sticky assignor - minimizes partition movement using
    /// cooperative incremental rebalancing.
    CooperativeSticky,
}

impl RebalanceStrategy {
    /// Convert to the string value expected by librdkafka's
    /// `partition.assignment.strategy` property.
    #[must_use]
    pub fn to_librdkafka_value(&self) -> &'static str {
        match self {
            Self::Range => "range",
            Self::RoundRobin => "roundrobin",
            Self::CooperativeSticky => "cooperative-sticky",
        }
    }
}

/// Commit configuration replacing flat `enable_auto_commit` + `commit_interval_ms`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CommitConfig {
    /// Commit mode: `auto` or `manual`.
    #[serde(default = "default_commit_mode")]
    pub mode: CommitMode,
    /// Commit interval in milliseconds (optional).
    ///
    /// - In `auto` mode: forwarded to rdkafka as `auto.commit.interval.ms`.
    ///   When omitted, the property is not set and librdkafka retains its
    ///   positive default (5000 ms).
    /// - In `manual` mode: controls a periodic safety-net timer for offset commits.
    ///   When omitted, no periodic timer is created and offsets are committed
    ///   When omitted, the safety-net timer is disabled and terminal feedback
    ///   drives commits. Transient NACKs in replay mode remain uncommitted.
    #[serde(default)]
    pub interval_ms: Option<u64>,
}

impl Default for CommitConfig {
    fn default() -> Self {
        Self {
            mode: CommitMode::Manual,
            interval_ms: None,
        }
    }
}

/// Per-signal configuration for traces, metrics, or logs.
///
/// Replaces the flat `traces_topic`, `metrics_topic`, `logs_topic` and
/// `default_msg_format` fields with a Go-style nested structure that allows
/// per-signal encoding and exclude patterns.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct SignalConfig {
    /// Topics to subscribe to for this signal.
    /// Entries starting with `^` are treated as regex patterns.
    #[serde(default)]
    topics: Vec<String>,
    /// Topic patterns to exclude (must be valid regex).
    /// Only allowed when at least one topic in the same signal is a regex pattern.
    #[serde(default)]
    exclude_topics: Vec<String>,
    /// Encoding format for messages on this signal's topics.
    /// Individual messages can override this via the message format header
    /// (defaults to `"MessageFormat"`).
    #[serde(default)]
    encoding: MessageFormat,
}

impl SignalConfig {
    /// Create a new signal configuration with the given topics.
    #[must_use]
    pub fn new(topics: Vec<String>) -> Self {
        Self {
            topics,
            ..Default::default()
        }
    }

    /// The topics configured for this signal.
    #[must_use]
    pub fn topics(&self) -> &[String] {
        &self.topics
    }

    /// The exclude topic patterns for this signal.
    #[must_use]
    pub fn exclude_topics(&self) -> &[String] {
        &self.exclude_topics
    }

    /// The encoding format for messages on this signal's topics.
    #[must_use]
    pub fn encoding(&self) -> MessageFormat {
        self.encoding
    }

    /// Set the exclude topic patterns.
    #[must_use]
    pub fn with_exclude_topics(mut self, exclude_topics: Vec<String>) -> Self {
        self.exclude_topics = exclude_topics;
        self
    }

    /// Set the encoding format.
    #[must_use]
    pub fn with_encoding(mut self, encoding: MessageFormat) -> Self {
        self.encoding = encoding;
        self
    }
}

/// How to interpret the raw Kafka header bytes when inserting an attribute.
///
/// The receiver first decodes the header value as UTF-8, then parses it
/// according to this type. On parse failure the attribute is skipped and an
/// error is logged.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeValueType {
    /// UTF-8 string (no further parsing).
    String,
    /// Boolean (`true` / `false`).
    Bool,
    /// Signed 64-bit integer.
    Int,
    /// 64-bit floating-point number.
    Float,
}

/// Describes how to inject an extracted Kafka header value into resource
/// attributes. The raw header bytes are first decoded as UTF-8, then parsed
/// according to `value_type`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct HeaderExtraction {
    /// The resource attribute key to use when inserting the value.
    pub key: String,
    /// How to interpret the raw header bytes.
    pub value_type: AttributeValueType,
}

/// Builder for Kafka receiver configuration.
///
/// This is the serde deserialization target. Use [`KafkaReceiverConfig`]
/// (via `TryFrom` or `#[serde(try_from)]`) for the validated configuration.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct KafkaReceiverConfigBuilder {
    /// Kafka broker addresses (comma-separated). Required.
    brokers: String,

    /// Kafka consumer group ID. Required.
    group_id: String,

    /// Kafka client ID. Required.
    client_id: String,

    /// Static group instance ID for Kafka static membership.
    #[serde(default)]
    group_instance_id: Option<String>,

    /// Configuration for authentication.
    #[serde(default)]
    auth: Option<Auth>,

    /// Optional TLS configuration for broker connections.
    #[serde(default)]
    tls: Option<TlsConfig>,

    /// Per-signal configuration for traces.
    #[serde(default)]
    traces: SignalConfig,

    /// Per-signal configuration for metrics.
    #[serde(default)]
    metrics: SignalConfig,

    /// Per-signal configuration for logs.
    #[serde(default)]
    logs: SignalConfig,

    /// Auto offset reset behavior when no offset is found.
    #[serde(default = "default_auto_offset_reset")]
    auto_offset_reset: AutoOffsetReset,

    /// Commit configuration (replaces `enable_auto_commit` + `commit_interval_ms`).
    #[serde(default)]
    commit: CommitConfig,

    /// Policy for non-permanent NACKs returned by downstream nodes.
    #[serde(default)]
    transient_nack: Option<TransientNackConfig>,

    /// Optional dead-letter-queue configuration. `None` (the default) = no DLQ;
    /// failed messages retain today's behavior (counted, logged, offset
    /// advanced). Presence of the block enables the DLQ.
    #[serde(default)]
    dlq: Option<DlqConfig>,

    /// Interval, in milliseconds, between consumer-lag refreshes.
    ///
    /// Enables `receiver.kafka.consumer.group.lag` (consumer-group lag measured
    /// against broker-committed offsets). Each refresh runs off the receive loop
    /// in a bounded background task, so it never blocks message processing; it
    /// queries the broker-committed offsets and one high watermark per owned
    /// partition, so raise the interval under large partition fan-out or broker
    /// slowness. Recommended: `60000` (60s).
    ///
    /// Manual commit mode only; ignored under auto-commit. Defaults to `None`
    /// (disabled). Must be `> 0` when set.
    #[serde(default)]
    lag_refresh_interval_ms: Option<u64>,

    /// Session timeout in milliseconds.
    #[serde(default = "default_session_timeout_ms")]
    session_timeout_ms: u64,

    /// Heartbeat interval in milliseconds.
    #[serde(default = "default_heartbeat_interval_ms")]
    heartbeat_interval_ms: u64,

    /// Minimum number of bytes to fetch.
    #[serde(default = "default_min_fetch_bytes")]
    min_fetch_bytes: i32,

    /// Maximum number of bytes to fetch.
    #[serde(default = "default_max_fetch_bytes")]
    max_fetch_bytes: i32,

    /// Maximum time to wait for a fetch response in milliseconds.
    #[serde(default = "default_max_fetch_wait_ms")]
    max_fetch_wait_ms: u64,

    /// Maximum bytes per partition per fetch.
    #[serde(default = "default_max_partition_fetch_bytes")]
    max_partition_fetch_bytes: i32,

    /// Consumer isolation level.
    #[serde(default = "default_isolation_level")]
    isolation_level: IsolationLevel,

    /// Header extraction rules for all signal types (traces, metrics, logs).
    ///
    /// Maps Kafka header keys to target resource attributes. When a message
    /// arrives on any signal topic, the receiver looks for these header keys
    /// and injects the values into the resource attributes of every resource
    /// in the payload.
    #[serde(default)]
    resource_attrs_from_headers: HashMap<String, HeaderExtraction>,

    /// Enable idempotent message processing.
    ///
    /// When `true` and commit mode is `manual`, the receiver checks
    /// whether an incoming message offset has already been seen (either
    /// currently in-flight or previously committed) and skips duplicates.
    ///
    /// Defaults to `false` (process every message).
    #[serde(default)]
    enable_idempotency: bool,

    /// Partition assignment strategy for consumer group rebalancing.
    ///
    /// When set, overrides librdkafka's default (`range,roundrobin`).
    /// Options: `range`, `round_robin`, `cooperative_sticky`.
    #[serde(default)]
    rebalance_strategy: Option<RebalanceStrategy>,

    /// Kafka header key for the message format indicator.
    ///
    /// The receiver checks incoming Kafka messages for a header matching this
    /// key. If the header is present and its value maps to a known encoding
    /// (`otlp` or `otap`), that encoding is used to decode the message. If the
    /// header is absent or the value is unrecognized, the receiver falls back
    /// to the per-signal `encoding` config.
    ///
    /// Defaults to `"MessageFormat"`. Users can override the key name but
    /// header-based format detection is always active.
    #[serde(default = "default_message_format_header")]
    message_format_header: String,

    /// List of librdkafka debug contexts to enable.
    ///
    /// Useful for troubleshooting Kafka connectivity, authentication, or
    /// message consumption issues. Each entry maps to one of librdkafka's
    /// debug context flags.
    ///
    /// Example:
    /// ```yaml
    /// debug:
    ///   - consumer
    ///   - cgrp
    ///   - fetch
    /// ```
    #[serde(default)]
    debug: Option<Vec<DebugContext>>,

    /// Librdkafka log level (syslog severity).
    ///
    /// Controls the verbosity of librdkafka's internal logging. When `None`
    /// (default), rdkafka infers the level from the application's `log`
    /// crate configuration.
    #[serde(default)]
    log_level: Option<LogLevel>,

    /// Consumer configuration options for additional librdkafka settings.
    /// Applied first; built-in options take precedence on conflict.
    #[serde(default)]
    consumer_config: HashMap<String, String>,
}

impl AutoOffsetReset {
    /// Convert to the string value expected by librdkafka
    #[must_use]
    pub fn to_kafka_value(&self) -> &'static str {
        match self {
            AutoOffsetReset::Earliest => "earliest",
            AutoOffsetReset::Latest => "latest",
            AutoOffsetReset::Error => "error",
        }
    }
}

impl IsolationLevel {
    /// Convert to the string value expected by librdkafka
    #[must_use]
    pub fn to_kafka_value(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "read_uncommitted",
            IsolationLevel::ReadCommitted => "read_committed",
        }
    }
}

/// Validated Kafka receiver configuration.
///
/// Created from [`KafkaReceiverConfigBuilder`] via `TryFrom`, which runs
/// comprehensive validation (topics disjoint, regex compilation, exclude_topics
/// constraints, fetch byte constraints, etc.). Deserialization via serde also
/// goes through this validation automatically.
///
/// This matches the exporter's `KafkaExporterConfig` / `KafkaExporterConfigBuilder`
/// pattern, ensuring consistency across Kafka components.
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "KafkaReceiverConfigBuilder")]
pub struct KafkaReceiverConfig {
    inner: KafkaReceiverConfigBuilder,
    transient_nack_policy: EffectiveTransientNackPolicy,
    /// Resolved DLQ configuration, or `None` when the DLQ is disabled.
    dlq: Option<ResolvedDlqConfig>,
}

impl TryFrom<KafkaReceiverConfigBuilder> for KafkaReceiverConfig {
    type Error = KafkaReceiverError;

    fn try_from(mut builder: KafkaReceiverConfigBuilder) -> Result<Self, KafkaReceiverError> {
        // Consume the user-facing optional value so the validated config keeps
        // exactly one source of truth: the effective policy resolved below.
        let configured_transient_nack = builder.transient_nack.take();
        let transient_nack = configured_transient_nack
            .as_ref()
            .cloned()
            .unwrap_or_default();

        // Reject empty required string fields
        if builder.brokers.is_empty() {
            return Err(KafkaReceiverError::ConfigEmptyField {
                field: "brokers".to_string(),
            });
        }
        if builder.client_id.is_empty() {
            return Err(KafkaReceiverError::ConfigEmptyField {
                field: "client_id".to_string(),
            });
        }
        if builder.group_id.is_empty() {
            return Err(KafkaReceiverError::ConfigEmptyField {
                field: "group_id".to_string(),
            });
        }

        // Reject empty optional string fields when explicitly set
        if let Some(ref id) = builder.group_instance_id
            && id.is_empty()
        {
            return Err(KafkaReceiverError::ConfigEmptyField {
                field: "group_instance_id".to_string(),
            });
        }
        if builder.message_format_header.is_empty() {
            return Err(KafkaReceiverError::ConfigEmptyField {
                field: "message_format_header".to_string(),
            });
        }

        // Reject empty keys in resource_attrs_from_headers
        for (header_key, extraction) in &builder.resource_attrs_from_headers {
            if header_key.is_empty() {
                return Err(KafkaReceiverError::ConfigEmptyHeaderKey);
            }
            if extraction.key.is_empty() {
                return Err(KafkaReceiverError::ConfigEmptyExtractionKey {
                    header_key: header_key.clone(),
                });
            }
        }

        // At least one signal must have non-empty topics
        if builder.traces.topics().is_empty()
            && builder.metrics.topics().is_empty()
            && builder.logs.topics().is_empty()
        {
            return Err(KafkaReceiverError::ConfigNoSignalTopics);
        }

        if builder.traces.encoding() == MessageFormat::Syslog {
            return Err(KafkaReceiverError::ConfigUnsupportedEncoding {
                signal: "traces".to_string(),
                encoding: "syslog".to_string(),
            });
        }
        if builder.metrics.encoding() == MessageFormat::Syslog {
            return Err(KafkaReceiverError::ConfigUnsupportedEncoding {
                signal: "metrics".to_string(),
                encoding: "syslog".to_string(),
            });
        }

        // Topics must be disjoint across signals
        {
            let traces: HashSet<&str> =
                builder.traces.topics().iter().map(|s| s.as_str()).collect();
            let metrics: HashSet<&str> = builder
                .metrics
                .topics()
                .iter()
                .map(|s| s.as_str())
                .collect();
            let logs: HashSet<&str> = builder.logs.topics().iter().map(|s| s.as_str()).collect();

            if !traces.is_disjoint(&metrics)
                || !traces.is_disjoint(&logs)
                || !metrics.is_disjoint(&logs)
            {
                return Err(KafkaReceiverError::ConfigOverlappingTopics);
            }
        }

        // Validate literal topic names (skip regex patterns starting with '^')
        KafkaReceiverConfigBuilder::validate_topic_names(&builder.traces.topics, "traces")?;
        KafkaReceiverConfigBuilder::validate_topic_names(&builder.metrics.topics, "metrics")?;
        KafkaReceiverConfigBuilder::validate_topic_names(&builder.logs.topics, "logs")?;

        // Validate regex patterns compile
        KafkaReceiverConfigBuilder::validate_topic_regexes(builder.traces.topics(), "traces")?;
        KafkaReceiverConfigBuilder::validate_topic_regexes(builder.metrics.topics(), "metrics")?;
        KafkaReceiverConfigBuilder::validate_topic_regexes(builder.logs.topics(), "logs")?;

        // Validate exclude_topics
        KafkaReceiverConfigBuilder::validate_exclude_topics(&builder.traces, "traces")?;
        KafkaReceiverConfigBuilder::validate_exclude_topics(&builder.metrics, "metrics")?;
        KafkaReceiverConfigBuilder::validate_exclude_topics(&builder.logs, "logs")?;

        // Validate auth configuration when present
        if let Some(ref auth) = builder.auth {
            auth.validate()
                .map_err(|e| KafkaReceiverError::ConfigInvalidAuth {
                    message: e.to_string(),
                })?;
        }

        // Validate TLS configuration when present
        if let Some(ref tls) = builder.tls {
            tls.validate()
                .map_err(|e| KafkaReceiverError::ConfigInvalidTls {
                    message: e.to_string(),
                })?;
        }

        // Fetch byte constraints. These fields are signed (`i32`), so a negative
        // value can arrive via deserialization; guard the absolute bounds before
        // the relative `max >= min` check so an out-of-range value produces a
        // field-specific error rather than a confusing "max < min" message.
        if builder.min_fetch_bytes < 1 {
            // librdkafka `fetch.min.bytes` has a minimum of 1.
            return Err(KafkaReceiverError::ConfigNonPositiveValue {
                field: "min_fetch_bytes".to_string(),
            });
        }
        if builder.max_fetch_bytes < 0 {
            // librdkafka `fetch.max.bytes` permits 0 (valid range 0..2147483135),
            // so reject only negatives here; the relative check below enforces
            // max >= min.
            return Err(KafkaReceiverError::ConfigNegativeValue {
                field: "max_fetch_bytes".to_string(),
            });
        }
        if builder.max_fetch_bytes < builder.min_fetch_bytes {
            return Err(KafkaReceiverError::ConfigInvalidFetchBytes {
                max: builder.max_fetch_bytes,
                min: builder.min_fetch_bytes,
            });
        }

        if builder.max_partition_fetch_bytes <= 0 {
            return Err(KafkaReceiverError::ConfigNonPositiveValue {
                field: "max_partition_fetch_bytes".to_string(),
            });
        }

        if builder.commit.interval_ms == Some(0) {
            return Err(KafkaReceiverError::ConfigNonPositiveValue {
                field: "commit.interval_ms".to_string(),
            });
        }

        if transient_nack.initial_backoff_ms == 0 {
            return Err(KafkaReceiverError::ConfigNonPositiveValue {
                field: "transient_nack.initial_backoff_ms".to_string(),
            });
        }

        if transient_nack.max_backoff_ms == 0 {
            return Err(KafkaReceiverError::ConfigNonPositiveValue {
                field: "transient_nack.max_backoff_ms".to_string(),
            });
        }

        if transient_nack.initial_backoff_ms > transient_nack.max_backoff_ms {
            return Err(KafkaReceiverError::ConfigInvalidTransientNackBackoff {
                initial: transient_nack.initial_backoff_ms,
                max: transient_nack.max_backoff_ms,
            });
        }

        if matches!(builder.commit.mode, CommitMode::Auto)
            && configured_transient_nack
                .as_ref()
                .is_some_and(|config| matches!(config.mode, TransientNackMode::Replay))
        {
            return Err(KafkaReceiverError::ConfigTransientNackReplayRequiresManual);
        }

        let transient_nack_policy = match (builder.commit.mode, transient_nack.mode) {
            (CommitMode::Auto, _) => EffectiveTransientNackPolicy::Inactive,
            (CommitMode::Manual, TransientNackMode::CommitAndSkip) => {
                EffectiveTransientNackPolicy::CommitAndSkip
            }
            (CommitMode::Manual, TransientNackMode::Replay) => {
                EffectiveTransientNackPolicy::Replay(ReplayBackoffConfig::from(&transient_nack))
            }
        };

        if builder.lag_refresh_interval_ms == Some(0) {
            return Err(KafkaReceiverError::ConfigNonPositiveValue {
                field: "lag_refresh_interval_ms".to_string(),
            });
        }

        // Consumer-group timing must be strictly positive: librdkafka rejects a
        // zero `session.timeout.ms` / `heartbeat.interval.ms` (valid range starts
        // at 1). Note `max_fetch_wait_ms` is intentionally NOT checked here: it
        // maps to `fetch.wait.max.ms`, whose valid range is 0..300000, so 0 is a
        // legitimate low-latency setting.
        if builder.session_timeout_ms == 0 {
            return Err(KafkaReceiverError::ConfigNonPositiveValue {
                field: "session_timeout_ms".to_string(),
            });
        }
        if builder.heartbeat_interval_ms == 0 {
            return Err(KafkaReceiverError::ConfigNonPositiveValue {
                field: "heartbeat_interval_ms".to_string(),
            });
        }

        // Kafka requires the heartbeat interval to be lower than the session
        // timeout; librdkafka rejects the consumer otherwise. Fail fast at
        // construction with a clear error instead of at consumer creation.
        if builder.heartbeat_interval_ms >= builder.session_timeout_ms {
            return Err(KafkaReceiverError::ConfigInvalidHeartbeat {
                heartbeat: builder.heartbeat_interval_ms,
                session: builder.session_timeout_ms,
            });
        }

        // Resolve and validate the DLQ. Consumed off the builder so the
        // validated config keeps a single source of truth (the effective DLQ).
        let dlq = builder
            .dlq
            .take()
            .map(|dlq_config| resolve_dlq(&builder, dlq_config))
            .transpose()?;

        Ok(Self {
            inner: builder,
            transient_nack_policy,
            dlq,
        })
    }
}

/// Returns `true` when two `bootstrap.servers` strings name the same broker
/// set, comparing trimmed, non-empty `host:port` entries as unordered sets so
/// reordered or differently-spaced broker lists still count as one cluster.
fn same_kafka_cluster(a: &str, b: &str) -> bool {
    let set = |s: &str| -> HashSet<String> {
        s.split(',')
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .map(str::to_string)
            .collect()
    };
    set(a) == set(b)
}

/// Compiled include/exclude matchers for one ingest signal, mirroring the
/// runtime router so config-time loop prevention respects exclude patterns.
struct IngestMatcher {
    topics: Vec<String>,
    regexes: Vec<Option<Regex>>,
    excludes: Vec<Regex>,
}

/// Validate a [`DlqConfig`] against the surrounding receiver config and resolve
/// it into a [`ResolvedDlqConfig`].
///
/// Validation rules:
/// - manual commit is required (DLQ delivery guarantees depend on the receiver
///   controlling offset commits);
/// - a DLQ topic must resolve for every captured signal (either the global
///   `topic` or a `per_signal` entry);
/// - every DLQ topic must be a legal Kafka topic name and, when the DLQ reuses
///   the source cluster, must not be an ingest topic the consumer subscribes to
///   (loop prevention, mirroring the runtime include/exclude routing);
/// - `capture` must be non-empty.
fn resolve_dlq(
    builder: &KafkaReceiverConfigBuilder,
    dlq: DlqConfig,
) -> Result<ResolvedDlqConfig, KafkaReceiverError> {
    // Manual commit mode required.
    if matches!(builder.commit.mode, CommitMode::Auto) {
        return Err(KafkaReceiverError::ConfigDlqRequiresManual);
    }

    if dlq.capture.is_empty() {
        return Err(KafkaReceiverError::ConfigDlqEmptyCapture);
    }

    let capture_decode = dlq.capture.contains(&DlqCapture::Decode);
    let capture_unknown_topic = dlq.capture.contains(&DlqCapture::UnknownTopic);
    let capture_permanent_nack = dlq.capture.contains(&DlqCapture::PermanentNack);

    // Resolve a DLQ topic per signal: a per-signal override, else the global
    // topic. Only signals that actually ingest need a DLQ topic; the
    // `unknown_topic` category is not signal-scoped, so it uses whichever
    // topic resolves (validated below to be non-empty when captured).
    let per_signal = dlq.per_signal.as_ref();
    let resolve = |override_topic: Option<&String>| -> Option<String> {
        override_topic
            .cloned()
            .or_else(|| dlq.topic.clone())
            .filter(|t| !t.is_empty())
    };

    let traces_ingests = !builder.traces.topics.is_empty();
    let metrics_ingests = !builder.metrics.topics.is_empty();
    let logs_ingests = !builder.logs.topics.is_empty();

    let traces_topic = resolve(per_signal.and_then(|p| p.traces.as_ref()));
    let metrics_topic = resolve(per_signal.and_then(|p| p.metrics.as_ref()));
    let logs_topic = resolve(per_signal.and_then(|p| p.logs.as_ref()));

    // Every ingesting signal must have a resolvable DLQ topic.
    for (ingests, topic, signal) in [
        (traces_ingests, &traces_topic, "traces"),
        (metrics_ingests, &metrics_topic, "metrics"),
        (logs_ingests, &logs_topic, "logs"),
    ] {
        if ingests && topic.is_none() {
            return Err(KafkaReceiverError::ConfigDlqMissingTopic {
                signal: signal.to_string(),
            });
        }
    }

    // The DLQ reuses the source cluster when its override brokers (if any)
    // resolve to the same broker set as the source consumer.
    let same_cluster = dlq
        .connection
        .as_ref()
        .and_then(|c| c.brokers.as_deref())
        .is_none_or(|b| same_kafka_cluster(b, &builder.brokers));

    // Loop prevention: on the same cluster a DLQ topic must not be one the
    // consumer actually subscribes to. Mirror the runtime router exactly --
    // per-signal include patterns minus exclude patterns -- so an excluded
    // topic is not a false-positive overlap. Compile each signal's patterns
    // once, then test every resolved DLQ topic against all three signals.
    let ingest_matchers = if same_cluster {
        let compile = |signal: &SignalConfig| -> Result<IngestMatcher, KafkaReceiverError> {
            let regexes = compile_topic_regexes(&signal.topics).map_err(|e| {
                KafkaReceiverError::ConfigInvalidDlqTopic {
                    topic: String::new(),
                    message: e.to_string(),
                }
            })?;
            let excludes = compile_exclude_regexes(&signal.exclude_topics).map_err(|e| {
                KafkaReceiverError::ConfigInvalidDlqTopic {
                    topic: String::new(),
                    message: e.to_string(),
                }
            })?;
            Ok(IngestMatcher {
                topics: signal.topics.clone(),
                regexes,
                excludes,
            })
        };
        Some([
            compile(&builder.traces)?,
            compile(&builder.metrics)?,
            compile(&builder.logs)?,
        ])
    } else {
        None
    };

    let is_ingest_topic = |topic: &str| -> bool {
        ingest_matchers.as_ref().is_some_and(|matchers| {
            matchers.iter().any(|m| {
                matches_any_topic(&m.topics, &m.regexes, topic)
                    && !matches_any_exclude(&m.excludes, topic)
            })
        })
    };

    // Validate every resolved DLQ topic name and enforce loop prevention.
    for topic in [&traces_topic, &metrics_topic, &logs_topic]
        .into_iter()
        .flatten()
    {
        validate_kafka_topic(topic).map_err(|message| {
            KafkaReceiverError::ConfigInvalidDlqTopic {
                topic: topic.clone(),
                message,
            }
        })?;
        if is_ingest_topic(topic) {
            return Err(KafkaReceiverError::ConfigDlqTopicOverlapsIngest {
                topic: topic.clone(),
            });
        }
    }

    // DLQ-PHASE-2 (Remove): producer-connection validation; the downstream
    // exporter validates its own connection in port mode.
    let connection = dlq.connection.unwrap_or_default();
    if let Some(auth) = &connection.auth {
        auth.validate()
            .map_err(|message| KafkaReceiverError::ConfigInvalidDlqConnection { message })?;
    }
    if let Some(tls) = &connection.tls {
        tls.validate()
            .map_err(|message| KafkaReceiverError::ConfigInvalidDlqConnection { message })?;
    }

    Ok(ResolvedDlqConfig {
        traces_topic,
        metrics_topic,
        logs_topic,
        capture_decode,
        capture_unknown_topic,
        capture_permanent_nack,
        connection,
    })
}

impl KafkaReceiverConfigBuilder {
    /// Create a new configuration builder with the required connection fields.
    ///
    /// All other fields are set to their defaults. Use the `with_*` builder
    /// methods to override individual settings.
    #[must_use]
    pub fn new(
        brokers: impl Into<String>,
        group_id: impl Into<String>,
        client_id: impl Into<String>,
    ) -> Self {
        Self {
            brokers: brokers.into(),
            group_id: group_id.into(),
            client_id: client_id.into(),
            group_instance_id: None,
            auth: None,
            tls: None,
            traces: SignalConfig::default(),
            metrics: SignalConfig::default(),
            logs: SignalConfig::default(),
            auto_offset_reset: default_auto_offset_reset(),
            commit: CommitConfig::default(),
            transient_nack: None,
            dlq: None,
            lag_refresh_interval_ms: None,
            session_timeout_ms: default_session_timeout_ms(),
            heartbeat_interval_ms: default_heartbeat_interval_ms(),
            min_fetch_bytes: default_min_fetch_bytes(),
            max_fetch_bytes: default_max_fetch_bytes(),
            max_fetch_wait_ms: default_max_fetch_wait_ms(),
            max_partition_fetch_bytes: default_max_partition_fetch_bytes(),
            isolation_level: default_isolation_level(),
            resource_attrs_from_headers: HashMap::new(),
            enable_idempotency: false,
            rebalance_strategy: None,
            message_format_header: default_message_format_header(),
            debug: None,
            log_level: None,
            consumer_config: HashMap::new(),
        }
    }

    /// Validate that all regex topic patterns (starting with `^`) compile.
    fn validate_topic_regexes(topics: &[String], signal: &str) -> Result<(), KafkaReceiverError> {
        for topic in topics {
            if topic.starts_with('^') {
                let _ =
                    Regex::new(topic).map_err(|e| KafkaReceiverError::ConfigInvalidTopicRegex {
                        signal: signal.to_string(),
                        topic: topic.clone(),
                        message: e.to_string(),
                    })?;
            }
        }
        Ok(())
    }

    /// Validate exclude_topics constraints for a signal.
    ///
    /// - `exclude_topics` only allowed when at least one topic is a regex pattern.
    /// - Each exclude_topics entry must be non-empty and a valid regex.
    fn validate_exclude_topics(
        signal: &SignalConfig,
        signal_name: &str,
    ) -> Result<(), KafkaReceiverError> {
        if signal.exclude_topics.is_empty() {
            return Ok(());
        }

        // exclude_topics only allowed when at least one topic is a regex pattern
        let has_regex = signal.topics.iter().any(|t| t.starts_with('^'));
        if !has_regex {
            return Err(KafkaReceiverError::ConfigExcludeTopicsRequiresRegex {
                signal: signal_name.to_string(),
            });
        }

        // Each entry must be non-empty and valid regex
        for pattern in &signal.exclude_topics {
            if pattern.is_empty() {
                return Err(KafkaReceiverError::ConfigEmptyExcludeTopic {
                    signal: signal_name.to_string(),
                });
            }
            let _ =
                Regex::new(pattern).map_err(|e| KafkaReceiverError::ConfigInvalidExcludeRegex {
                    signal: signal_name.to_string(),
                    pattern: pattern.clone(),
                    message: e.to_string(),
                })?;
        }

        Ok(())
    }

    /// Validate that all literal (non-regex) topic names are valid Kafka topics.
    ///
    /// Entries starting with `^` are treated as regex patterns and are skipped
    /// here (they are validated separately by [`Self::validate_topic_regexes`]).
    fn validate_topic_names(topics: &[String], signal: &str) -> Result<(), KafkaReceiverError> {
        for topic in topics {
            if !topic.starts_with('^') {
                validate_kafka_topic(topic).map_err(|e| {
                    KafkaReceiverError::ConfigInvalidTopicName {
                        signal: signal.to_string(),
                        message: e.to_string(),
                    }
                })?;
            }
        }
        Ok(())
    }

    // ---- Builder methods ----

    /// Set the broker addresses.
    #[must_use]
    pub fn with_brokers(mut self, brokers: impl Into<String>) -> Self {
        self.brokers = brokers.into();
        self
    }

    /// Set the group ID.
    #[must_use]
    pub fn with_group_id(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = group_id.into();
        self
    }

    /// Set the client ID.
    #[must_use]
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = client_id.into();
        self
    }

    /// Set the group instance ID for static membership.
    #[must_use]
    pub fn with_group_instance_id(mut self, id: impl Into<String>) -> Self {
        self.group_instance_id = Some(id.into());
        self
    }

    /// Set the traces signal configuration.
    #[must_use]
    pub fn with_traces(mut self, traces: SignalConfig) -> Self {
        self.traces = traces;
        self
    }

    /// Set the metrics signal configuration.
    #[must_use]
    pub fn with_metrics(mut self, metrics: SignalConfig) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set the logs signal configuration.
    #[must_use]
    pub fn with_logs(mut self, logs: SignalConfig) -> Self {
        self.logs = logs;
        self
    }

    /// Set the auto offset reset behavior.
    #[must_use]
    pub fn with_auto_offset_reset(mut self, reset: AutoOffsetReset) -> Self {
        self.auto_offset_reset = reset;
        self
    }

    /// Set the commit configuration.
    #[must_use]
    pub fn with_commit(mut self, commit: CommitConfig) -> Self {
        self.commit = commit;
        self
    }

    /// Set the non-permanent NACK policy.
    #[must_use]
    pub fn with_transient_nack(mut self, transient_nack: TransientNackConfig) -> Self {
        self.transient_nack = Some(transient_nack);
        self
    }

    /// Set the dead-letter-queue configuration.
    #[must_use]
    pub fn with_dlq(mut self, dlq: DlqConfig) -> Self {
        self.dlq = Some(dlq);
        self
    }

    /// Set the consumer-lag refresh interval in milliseconds.
    ///
    /// `None` (the default) disables consumer-lag refresh.
    #[must_use]
    pub fn with_lag_refresh_interval_ms(mut self, interval_ms: Option<u64>) -> Self {
        self.lag_refresh_interval_ms = interval_ms;
        self
    }

    /// Set the isolation level.
    #[must_use]
    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = level;
        self
    }

    /// Set the minimum fetch bytes.
    #[must_use]
    pub fn with_min_fetch_bytes(mut self, bytes: i32) -> Self {
        self.min_fetch_bytes = bytes;
        self
    }

    /// Set the maximum fetch bytes.
    #[must_use]
    pub fn with_max_fetch_bytes(mut self, bytes: i32) -> Self {
        self.max_fetch_bytes = bytes;
        self
    }

    /// Set the maximum partition fetch bytes.
    #[must_use]
    pub fn with_max_partition_fetch_bytes(mut self, bytes: i32) -> Self {
        self.max_partition_fetch_bytes = bytes;
        self
    }

    /// Set the resource attributes from headers extraction rules.
    #[must_use]
    pub fn with_resource_attrs_from_headers(
        mut self,
        rules: HashMap<String, HeaderExtraction>,
    ) -> Self {
        self.resource_attrs_from_headers = rules;
        self
    }

    /// Set the consumer config overrides.
    #[must_use]
    pub fn with_consumer_config(mut self, config: HashMap<String, String>) -> Self {
        self.consumer_config = config;
        self
    }

    /// Enable or disable idempotency.
    #[must_use]
    pub fn with_enable_idempotency(mut self, enabled: bool) -> Self {
        self.enable_idempotency = enabled;
        self
    }

    /// Set the TLS configuration.
    #[must_use]
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }
    /// Set the partition assignment strategy for rebalancing.
    #[must_use]
    pub fn with_rebalance_strategy(mut self, strategy: RebalanceStrategy) -> Self {
        self.rebalance_strategy = Some(strategy);
        self
    }

    /// Set the message format header key.
    ///
    /// Defaults to `"MessageFormat"` when not explicitly set.
    #[must_use]
    pub fn with_message_format_header(mut self, header: impl Into<String>) -> Self {
        self.message_format_header = header.into();
        self
    }

    /// Enable librdkafka debug logging for the given contexts.
    ///
    /// Accepts a list of [`DebugContext`] values (e.g.,
    /// `vec![DebugContext::Consumer, DebugContext::Cgrp]`).
    #[must_use]
    pub fn with_debug(mut self, contexts: Vec<DebugContext>) -> Self {
        self.debug = Some(contexts);
        self
    }

    /// Set the librdkafka log level.
    ///
    /// When set, overrides rdkafka's default behavior of inferring the log
    /// level from the application's `log` crate configuration.
    #[must_use]
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = Some(level);
        self
    }

    /// Build Kafka client configuration.
    #[must_use]
    pub fn build_client_config(&self) -> ClientConfig {
        let mut config = ClientConfig::new();

        // Set custom consumer configuration first (built-in options override on conflict)
        for (key, value) in &self.consumer_config {
            _ = config.set(key, value);
        }

        _ = config.set("bootstrap.servers", &self.brokers);
        _ = config.set("group.id", &self.group_id);
        _ = config.set("client.id", &self.client_id);

        // Static group membership
        if let Some(ref instance_id) = self.group_instance_id {
            _ = config.set("group.instance.id", instance_id);
        }

        // Commit settings derived from CommitConfig
        let auto_commit = matches!(self.commit.mode, CommitMode::Auto);
        if auto_commit && let Some(interval) = self.commit.interval_ms {
            _ = config.set("auto.commit.interval.ms", interval.to_string());
        }
        _ = config.set(
            "enable.auto.commit",
            if auto_commit { "true" } else { "false" },
        );

        _ = config.set(
            "enable.auto.offset.store",
            if auto_commit { "true" } else { "false" },
        );

        // Offset management
        _ = config.set("auto.offset.reset", self.auto_offset_reset.to_kafka_value());

        // Session management
        _ = config.set("session.timeout.ms", self.session_timeout_ms.to_string());
        _ = config.set(
            "heartbeat.interval.ms",
            self.heartbeat_interval_ms.to_string(),
        );

        // Fetch tuning
        _ = config.set("fetch.min.bytes", self.min_fetch_bytes.to_string());
        _ = config.set("fetch.max.bytes", self.max_fetch_bytes.to_string());
        _ = config.set("fetch.wait.max.ms", self.max_fetch_wait_ms.to_string());
        _ = config.set(
            "max.partition.fetch.bytes",
            self.max_partition_fetch_bytes.to_string(),
        );

        // Isolation level
        _ = config.set("isolation.level", self.isolation_level.to_kafka_value());

        // Security protocol, TLS, and SASL settings (shared with exporter)
        let protocol = resolve_security_protocol(self.tls.as_ref(), self.auth.as_ref());
        _ = config.set("security.protocol", protocol);

        if let Some(tls) = &self.tls {
            tls.apply_to_client_config(&mut config);
        }

        apply_sasl_config(self.auth.as_ref(), &mut config);

        // Partition assignment strategy (when omitted, librdkafka defaults to range,roundrobin)
        if let Some(strategy) = self.rebalance_strategy {
            _ = config.set(
                "partition.assignment.strategy",
                strategy.to_librdkafka_value(),
            );
        }

        // Debug contexts and log level (applied last so they override any
        // value that might have been set via consumer_config).
        if let Some(ref contexts) = self.debug {
            _ = config.set("debug", debug_list_to_string(contexts));
        }
        if let Some(level) = self.log_level {
            _ = config.set_log_level(level.to_rdkafka());
        }

        config
    }
}

impl KafkaReceiverConfig {
    /// Get the broker addresses.
    #[must_use]
    pub fn brokers(&self) -> &str {
        &self.inner.brokers
    }

    /// Get the group id.
    #[must_use]
    pub fn group_id(&self) -> &str {
        &self.inner.group_id
    }

    /// Get the client_id.
    #[must_use]
    pub fn client_id(&self) -> &str {
        &self.inner.client_id
    }

    /// Get the static group instance ID, if configured.
    #[must_use]
    pub fn group_instance_id(&self) -> Option<&str> {
        self.inner.group_instance_id.as_deref()
    }

    /// Set the static group instance ID.
    pub fn set_group_instance_id(&mut self, id: String) {
        self.inner.group_instance_id = Some(id);
    }

    /// Get the traces signal configuration.
    #[must_use]
    pub fn traces(&self) -> &SignalConfig {
        &self.inner.traces
    }

    /// Get the metrics signal configuration.
    #[must_use]
    pub fn metrics(&self) -> &SignalConfig {
        &self.inner.metrics
    }

    /// Get the logs signal configuration.
    #[must_use]
    pub fn logs(&self) -> &SignalConfig {
        &self.inner.logs
    }

    /// Get the traces topics.
    #[must_use]
    pub fn traces_topics(&self) -> &[String] {
        &self.inner.traces.topics
    }

    /// Get the metrics topics.
    #[must_use]
    pub fn metrics_topics(&self) -> &[String] {
        &self.inner.metrics.topics
    }

    /// Get the logs topics.
    #[must_use]
    pub fn logs_topics(&self) -> &[String] {
        &self.inner.logs.topics
    }

    /// Get the traces exclude topics.
    #[must_use]
    pub fn traces_exclude_topics(&self) -> &[String] {
        &self.inner.traces.exclude_topics
    }

    /// Get the metrics exclude topics.
    #[must_use]
    pub fn metrics_exclude_topics(&self) -> &[String] {
        &self.inner.metrics.exclude_topics
    }

    /// Get the logs exclude topics.
    #[must_use]
    pub fn logs_exclude_topics(&self) -> &[String] {
        &self.inner.logs.exclude_topics
    }

    /// Get the traces encoding.
    #[must_use]
    pub fn traces_encoding(&self) -> MessageFormat {
        self.inner.traces.encoding
    }

    /// Get the metrics encoding.
    #[must_use]
    pub fn metrics_encoding(&self) -> MessageFormat {
        self.inner.metrics.encoding
    }

    /// Get the logs encoding.
    #[must_use]
    pub fn logs_encoding(&self) -> MessageFormat {
        self.inner.logs.encoding
    }

    /// Get the configured wire encoding for `signal`.
    ///
    /// Single signal-keyed accessor so callers (e.g. `detect_message_format`)
    /// need not select among the per-signal `*_encoding()` accessors.
    #[must_use]
    pub fn encoding_for(&self, signal: SignalType) -> MessageFormat {
        match signal {
            SignalType::Traces => self.inner.traces.encoding,
            SignalType::Metrics => self.inner.metrics.encoding,
            SignalType::Logs => self.inner.logs.encoding,
        }
    }

    /// Returns `true` if auto-commit mode is enabled.
    #[must_use]
    pub fn is_auto_commit(&self) -> bool {
        matches!(self.inner.commit.mode, CommitMode::Auto)
    }

    /// Get the commit configuration.
    #[must_use]
    pub fn commit(&self) -> &CommitConfig {
        &self.inner.commit
    }

    /// Get the configured commit interval in milliseconds.
    #[must_use]
    pub fn commit_interval_ms(&self) -> Option<u64> {
        self.inner.commit.interval_ms
    }

    /// Get the effective non-permanent NACK policy after commit-mode defaults.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn transient_nack_policy(&self) -> &EffectiveTransientNackPolicy {
        &self.transient_nack_policy
    }

    /// Get replay timing when the effective policy replays transient NACKs.
    #[must_use]
    pub(crate) fn replay_backoff(&self) -> Option<&ReplayBackoffConfig> {
        match &self.transient_nack_policy {
            EffectiveTransientNackPolicy::Replay(config) => Some(config),
            EffectiveTransientNackPolicy::Inactive
            | EffectiveTransientNackPolicy::CommitAndSkip => None,
        }
    }

    /// Returns `true` when non-permanent NACKs should be replayed from Kafka.
    #[must_use]
    pub fn replays_transient_nacks(&self) -> bool {
        matches!(
            self.transient_nack_policy,
            EffectiveTransientNackPolicy::Replay(_)
        )
    }

    /// Returns the resolved DLQ configuration, or `None` when disabled.
    #[must_use]
    pub(crate) fn dlq(&self) -> Option<&ResolvedDlqConfig> {
        self.dlq.as_ref()
    }

    /// Build the librdkafka `ClientConfig` for the DLQ producer.
    ///
    /// Connection (brokers/auth/tls) defaults to the source consumer's, with
    /// any `dlq.connection` overrides applied. The `client.id` is auto-derived
    /// as `{client_id}-dlq`. No producer tuning is set here, so librdkafka's
    /// own defaults apply (compression `none`, `acks=all`, `message.timeout.ms`
    /// 300000). The DLQ send is separately bounded by
    /// [`DLQ_OP_TIMEOUT_MS`](crate::receivers::kafka_receiver::config::DLQ_OP_TIMEOUT_MS).
    // DLQ-PHASE-2 (Remove): the receiver no longer builds a producer client
    // config; the downstream exporter owns the DLQ connection.
    #[must_use]
    pub(crate) fn build_dlq_producer_config(&self) -> Option<ClientConfig> {
        let dlq = self.dlq.as_ref()?;
        let mut config = ClientConfig::new();

        let brokers = dlq
            .connection
            .brokers
            .as_deref()
            .unwrap_or(&self.inner.brokers);
        _ = config.set("bootstrap.servers", brokers);
        _ = config.set("client.id", format!("{}-dlq", self.inner.client_id));

        self.apply_dlq_security(&mut config, dlq);
        Some(config)
    }

    /// Build the librdkafka `ClientConfig` for the dedicated DLQ re-read
    /// consumer used to recover the original bytes of a permanently-nacked
    /// message. It has no group subscription and manually assigns partitions.
    // DLQ-PHASE-2 (Change): the re-read consumer is retained in port mode; only
    // the producer wiring is removed.
    #[must_use]
    pub(crate) fn build_dlq_reread_consumer_config(&self) -> Option<ClientConfig> {
        // Presence of the DLQ enables the re-read consumer.
        let _dlq = self.dlq.as_ref()?;
        let mut config = ClientConfig::new();

        // Always re-read from the SOURCE cluster (that is where the original
        // bytes live). The `dlq.connection` overrides are producer-only and
        // never apply here.
        _ = config.set("bootstrap.servers", &self.inner.brokers);
        _ = config.set("client.id", format!("{}-dlq-reread", self.inner.client_id));
        // Manual assignment, no auto-commit, no group offset management.
        _ = config.set("enable.auto.commit", "false");
        _ = config.set("enable.auto.offset.store", "false");
        // A group.id is still required by librdkafka even for manual assign.
        _ = config.set("group.id", format!("{}-dlq-reread", self.inner.group_id));

        // Security uses the SOURCE connection (the re-read consumer reads
        // source topics), regardless of DLQ producer connection overrides.
        let protocol = resolve_security_protocol(self.inner.tls.as_ref(), self.inner.auth.as_ref());
        _ = config.set("security.protocol", protocol);
        if let Some(tls) = &self.inner.tls {
            tls.apply_to_client_config(&mut config);
        }
        apply_sasl_config(self.inner.auth.as_ref(), &mut config);
        Some(config)
    }

    /// Apply DLQ producer security (brokers-scoped auth/tls) to a client config,
    /// defaulting to the source connection when the DLQ does not override it.
    fn apply_dlq_security(&self, config: &mut ClientConfig, dlq: &ResolvedDlqConfig) {
        let tls = dlq.connection.tls.as_ref().or(self.inner.tls.as_ref());
        let auth = dlq.connection.auth.as_ref().or(self.inner.auth.as_ref());
        let protocol = resolve_security_protocol(tls, auth);
        _ = config.set("security.protocol", protocol);
        if let Some(tls) = tls {
            tls.apply_to_client_config(config);
        }
        apply_sasl_config(auth, config);
    }

    /// Get the configured consumer-lag refresh interval in milliseconds.
    ///
    /// Returns `None` when consumer-lag refresh is disabled (the default).
    #[must_use]
    pub fn lag_refresh_interval_ms(&self) -> Option<u64> {
        self.inner.lag_refresh_interval_ms
    }

    /// Returns `true` if idempotent message processing is enabled.
    #[must_use]
    pub fn is_idempotent(&self) -> bool {
        self.inner.enable_idempotency
    }

    /// Get the TLS configuration, if set.
    #[must_use]
    pub fn tls(&self) -> Option<&TlsConfig> {
        self.inner.tls.as_ref()
    }

    /// Get the configured rebalance strategy.
    #[must_use]
    pub fn rebalance_strategy(&self) -> Option<RebalanceStrategy> {
        self.inner.rebalance_strategy
    }

    /// The Kafka header key used for the message format indicator.
    ///
    /// Defaults to `"MessageFormat"`. Header-based format detection is always
    /// active.
    #[must_use]
    pub fn message_format_header(&self) -> &str {
        &self.inner.message_format_header
    }

    /// The librdkafka debug contexts, if configured.
    #[must_use]
    pub fn debug(&self) -> Option<&[DebugContext]> {
        self.inner.debug.as_deref()
    }

    /// The librdkafka log level, if configured.
    #[must_use]
    pub fn log_level(&self) -> Option<LogLevel> {
        self.inner.log_level
    }

    /// Get the header extraction rules.
    #[must_use]
    pub fn resource_attrs_from_headers(&self) -> &HashMap<String, HeaderExtraction> {
        &self.inner.resource_attrs_from_headers
    }

    /// Get the list of all topics to subscribe to.
    #[must_use]
    pub fn all_topics(&self) -> Vec<&str> {
        let mut topics = Vec::with_capacity(
            self.inner.traces.topics.len()
                + self.inner.metrics.topics.len()
                + self.inner.logs.topics.len(),
        );
        topics.extend(self.inner.traces.topics.iter().map(String::as_str));
        topics.extend(self.inner.metrics.topics.iter().map(String::as_str));
        topics.extend(self.inner.logs.topics.iter().map(String::as_str));
        topics
    }

    /// Get the authentication configuration, if set.
    #[must_use]
    pub fn auth(&self) -> Option<&Auth> {
        self.inner.auth.as_ref()
    }

    /// Build Kafka client configuration.
    #[must_use]
    pub fn build_client_config(&self) -> ClientConfig {
        self.inner.build_client_config()
    }

    /// Returns any `consumer_config` keys that overlap with rdkafka keys
    /// managed by first-class config fields and may be overwritten.
    #[must_use]
    pub fn overridden_consumer_config_keys(&self) -> Vec<&str> {
        self.inner
            .consumer_config
            .keys()
            .filter(|k| MANAGED_CONSUMER_CONFIG_KEYS.contains(&k.as_str()))
            .map(String::as_str)
            .collect()
    }
}

// ---- Default functions for serde ----

fn default_auto_offset_reset() -> AutoOffsetReset {
    AutoOffsetReset::Latest
}

fn default_isolation_level() -> IsolationLevel {
    IsolationLevel::ReadUncommitted
}

fn default_commit_mode() -> CommitMode {
    CommitMode::Manual
}

fn default_transient_nack_mode() -> TransientNackMode {
    TransientNackMode::Replay
}

const fn default_transient_nack_initial_backoff_ms() -> u64 {
    1_000
}

const fn default_transient_nack_max_backoff_ms() -> u64 {
    30_000
}

fn default_session_timeout_ms() -> u64 {
    10000
}

fn default_heartbeat_interval_ms() -> u64 {
    3000
}

fn default_min_fetch_bytes() -> i32 {
    1
}

fn default_max_fetch_bytes() -> i32 {
    1_048_576
}

fn default_max_fetch_wait_ms() -> u64 {
    250
}

fn default_max_partition_fetch_bytes() -> i32 {
    1_048_576
}

#[cfg(test)]
mod tests;
