// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::common::kafka::auth::Auth;
use crate::common::kafka::security::{apply_sasl_config, resolve_security_protocol};
use crate::common::kafka::{
    DebugContext, LogLevel, MessageFormat, TlsConfig, debug_list_to_string,
    default_message_format_header, validate_kafka_topic,
};
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
    ///   purely through ack/nack signals from downstream processing.
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
pub struct KafkaReceiverConfig(KafkaReceiverConfigBuilder);

impl TryFrom<KafkaReceiverConfigBuilder> for KafkaReceiverConfig {
    type Error = String;

    fn try_from(builder: KafkaReceiverConfigBuilder) -> Result<Self, String> {
        // Reject empty required string fields
        if builder.brokers.is_empty() {
            return Err("brokers can't be empty".to_string());
        }
        if builder.client_id.is_empty() {
            return Err("client_id can't be empty".to_string());
        }
        if builder.group_id.is_empty() {
            return Err("group_id can't be empty".to_string());
        }

        // Reject empty optional string fields when explicitly set
        if let Some(ref id) = builder.group_instance_id {
            if id.is_empty() {
                return Err("valid group_instance_id can't be empty".to_string());
            }
        }
        if builder.message_format_header.is_empty() {
            return Err("message_format_header can't be empty".to_string());
        }

        // Reject empty keys in resource_attrs_from_headers
        for (header_key, extraction) in &builder.resource_attrs_from_headers {
            if header_key.is_empty() {
                return Err("resource_attrs_from_headers contains an empty header key".to_string());
            }
            if extraction.key.is_empty() {
                return Err(format!(
                    "resource_attrs_from_headers['{header_key}'].key can't be empty"
                ));
            }
        }

        // At least one signal must have non-empty topics
        if builder.traces.topics().is_empty()
            && builder.metrics.topics().is_empty()
            && builder.logs.topics().is_empty()
        {
            return Err(
                "at least one signal (traces, metrics, or logs) must have non-empty topics"
                    .to_string(),
            );
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
                return Err("kafka topics overlap across signals".to_string());
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
            auth.validate().map_err(|e| format!("auth: {e}"))?;
        }

        // Validate TLS configuration when present
        if let Some(ref tls) = builder.tls {
            tls.validate().map_err(|e| format!("tls: {e}"))?;
        }

        // Fetch byte constraints
        if builder.max_fetch_bytes < builder.min_fetch_bytes {
            return Err(format!(
                "max_fetch_bytes ({}) must be >= min_fetch_bytes ({})",
                builder.max_fetch_bytes, builder.min_fetch_bytes
            ));
        }

        if builder.max_partition_fetch_bytes <= 0 {
            return Err("max_partition_fetch_bytes must be > 0".to_string());
        }

        if builder.commit.interval_ms == Some(0) {
            return Err("commit.interval_ms, when set, must be > 0".to_string());
        }

        Ok(Self(builder))
    }
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
    fn validate_topic_regexes(topics: &[String], signal: &str) -> Result<(), String> {
        for topic in topics {
            if topic.starts_with('^') {
                let _ = Regex::new(topic).map_err(|e| {
                    format!("invalid regex topic pattern in {signal}: '{topic}': {e}")
                })?;
            }
        }
        Ok(())
    }

    /// Validate exclude_topics constraints for a signal.
    ///
    /// - `exclude_topics` only allowed when at least one topic is a regex pattern.
    /// - Each exclude_topics entry must be non-empty and a valid regex.
    fn validate_exclude_topics(signal: &SignalConfig, signal_name: &str) -> Result<(), String> {
        if signal.exclude_topics.is_empty() {
            return Ok(());
        }

        // exclude_topics only allowed when at least one topic is a regex pattern
        let has_regex = signal.topics.iter().any(|t| t.starts_with('^'));
        if !has_regex {
            return Err(format!(
                "{signal_name}.exclude_topics is only allowed when at least one topic is a regex pattern"
            ));
        }

        // Each entry must be non-empty and valid regex
        for pattern in &signal.exclude_topics {
            if pattern.is_empty() {
                return Err(format!(
                    "{signal_name}.exclude_topics entries must be non-empty"
                ));
            }
            let _ = Regex::new(pattern).map_err(|e| {
                format!("invalid regex in {signal_name}.exclude_topics: '{pattern}': {e}")
            })?;
        }

        Ok(())
    }

    /// Validate that all literal (non-regex) topic names are valid Kafka topics.
    ///
    /// Entries starting with `^` are treated as regex patterns and are skipped
    /// here (they are validated separately by [`Self::validate_topic_regexes`]).
    fn validate_topic_names(topics: &[String], signal: &str) -> Result<(), String> {
        for topic in topics {
            if !topic.starts_with('^') {
                validate_kafka_topic(topic).map_err(|e| format!("{signal}.topics: {e}"))?;
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
        if auto_commit {
            if let Some(interval) = self.commit.interval_ms {
                _ = config.set("auto.commit.interval.ms", interval.to_string());
            }
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
        &self.0.brokers
    }

    /// Get the group id.
    #[must_use]
    pub fn group_id(&self) -> &str {
        &self.0.group_id
    }

    /// Get the client_id.
    #[must_use]
    pub fn client_id(&self) -> &str {
        &self.0.client_id
    }

    /// Get the static group instance ID, if configured.
    #[must_use]
    pub fn group_instance_id(&self) -> Option<&str> {
        self.0.group_instance_id.as_deref()
    }

    /// Set the static group instance ID.
    pub fn set_group_instance_id(&mut self, id: String) {
        self.0.group_instance_id = Some(id);
    }

    /// Get the traces signal configuration.
    #[must_use]
    pub fn traces(&self) -> &SignalConfig {
        &self.0.traces
    }

    /// Get the metrics signal configuration.
    #[must_use]
    pub fn metrics(&self) -> &SignalConfig {
        &self.0.metrics
    }

    /// Get the logs signal configuration.
    #[must_use]
    pub fn logs(&self) -> &SignalConfig {
        &self.0.logs
    }

    /// Get the traces topics.
    #[must_use]
    pub fn traces_topics(&self) -> &[String] {
        &self.0.traces.topics
    }

    /// Get the metrics topics.
    #[must_use]
    pub fn metrics_topics(&self) -> &[String] {
        &self.0.metrics.topics
    }

    /// Get the logs topics.
    #[must_use]
    pub fn logs_topics(&self) -> &[String] {
        &self.0.logs.topics
    }

    /// Get the traces exclude topics.
    #[must_use]
    pub fn traces_exclude_topics(&self) -> &[String] {
        &self.0.traces.exclude_topics
    }

    /// Get the metrics exclude topics.
    #[must_use]
    pub fn metrics_exclude_topics(&self) -> &[String] {
        &self.0.metrics.exclude_topics
    }

    /// Get the logs exclude topics.
    #[must_use]
    pub fn logs_exclude_topics(&self) -> &[String] {
        &self.0.logs.exclude_topics
    }

    /// Get the traces encoding.
    #[must_use]
    pub fn traces_encoding(&self) -> MessageFormat {
        self.0.traces.encoding
    }

    /// Get the metrics encoding.
    #[must_use]
    pub fn metrics_encoding(&self) -> MessageFormat {
        self.0.metrics.encoding
    }

    /// Get the logs encoding.
    #[must_use]
    pub fn logs_encoding(&self) -> MessageFormat {
        self.0.logs.encoding
    }

    /// Returns `true` if auto-commit mode is enabled.
    #[must_use]
    pub fn is_auto_commit(&self) -> bool {
        matches!(self.0.commit.mode, CommitMode::Auto)
    }

    /// Get the commit configuration.
    #[must_use]
    pub fn commit(&self) -> &CommitConfig {
        &self.0.commit
    }

    /// Get the configured commit interval in milliseconds.
    #[must_use]
    pub fn commit_interval_ms(&self) -> Option<u64> {
        self.0.commit.interval_ms
    }

    /// Returns `true` if idempotent message processing is enabled.
    #[must_use]
    pub fn is_idempotent(&self) -> bool {
        self.0.enable_idempotency
    }

    /// Get the TLS configuration, if set.
    #[must_use]
    pub fn tls(&self) -> Option<&TlsConfig> {
        self.0.tls.as_ref()
    }

    /// Get the configured rebalance strategy.
    #[must_use]
    pub fn rebalance_strategy(&self) -> Option<RebalanceStrategy> {
        self.0.rebalance_strategy
    }

    /// The Kafka header key used for the message format indicator.
    ///
    /// Defaults to `"MessageFormat"`. Header-based format detection is always
    /// active.
    #[must_use]
    pub fn message_format_header(&self) -> &str {
        &self.0.message_format_header
    }

    /// The librdkafka debug contexts, if configured.
    #[must_use]
    pub fn debug(&self) -> Option<&[DebugContext]> {
        self.0.debug.as_deref()
    }

    /// The librdkafka log level, if configured.
    #[must_use]
    pub fn log_level(&self) -> Option<LogLevel> {
        self.0.log_level
    }

    /// Get the header extraction rules.
    #[must_use]
    pub fn resource_attrs_from_headers(&self) -> &HashMap<String, HeaderExtraction> {
        &self.0.resource_attrs_from_headers
    }

    /// Get the list of all topics to subscribe to.
    #[must_use]
    pub fn all_topics(&self) -> Vec<&str> {
        let mut topics = Vec::with_capacity(
            self.0.traces.topics.len() + self.0.metrics.topics.len() + self.0.logs.topics.len(),
        );
        topics.extend(self.0.traces.topics.iter().map(String::as_str));
        topics.extend(self.0.metrics.topics.iter().map(String::as_str));
        topics.extend(self.0.logs.topics.iter().map(String::as_str));
        topics
    }

    /// Get the authentication configuration, if set.
    #[must_use]
    pub fn auth(&self) -> Option<&Auth> {
        self.0.auth.as_ref()
    }

    /// Build Kafka client configuration.
    #[must_use]
    pub fn build_client_config(&self) -> ClientConfig {
        self.0.build_client_config()
    }

    /// Returns any `consumer_config` keys that overlap with rdkafka keys
    /// managed by first-class config fields and may be overwritten.
    #[must_use]
    pub fn overridden_consumer_config_keys(&self) -> Vec<&str> {
        self.0
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
mod tests {
    use super::*;
    use serde_json::json;

    // ---- AutoOffsetReset ----

    #[test]
    fn auto_offset_reset_to_kafka_value_earliest() {
        assert_eq!(AutoOffsetReset::Earliest.to_kafka_value(), "earliest");
    }

    #[test]
    fn auto_offset_reset_to_kafka_value_latest() {
        assert_eq!(AutoOffsetReset::Latest.to_kafka_value(), "latest");
    }

    #[test]
    fn auto_offset_reset_to_kafka_value_error() {
        assert_eq!(AutoOffsetReset::Error.to_kafka_value(), "error");
    }

    // ---- IsolationLevel ----

    #[test]
    fn isolation_level_to_kafka_value_read_uncommitted() {
        assert_eq!(
            IsolationLevel::ReadUncommitted.to_kafka_value(),
            "read_uncommitted"
        );
    }

    #[test]
    fn isolation_level_to_kafka_value_read_committed() {
        assert_eq!(
            IsolationLevel::ReadCommitted.to_kafka_value(),
            "read_committed"
        );
    }

    // ---- CommitConfig ----

    #[test]
    fn commit_config_defaults() {
        let cfg = CommitConfig::default();
        assert_eq!(cfg.mode, CommitMode::Manual);
        assert_eq!(cfg.interval_ms, None);
    }

    #[test]
    fn commit_config_deserialize_auto() {
        let json = json!({"mode": "auto", "interval_ms": 5000});
        let cfg: CommitConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.mode, CommitMode::Auto);
        assert_eq!(cfg.interval_ms, Some(5000));
    }

    #[test]
    fn commit_config_deserialize_manual() {
        let json = json!({"mode": "manual", "interval_ms": 500});
        let cfg: CommitConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.mode, CommitMode::Manual);
        assert_eq!(cfg.interval_ms, Some(500));
    }

    #[test]
    fn commit_config_deserialize_defaults_when_empty() {
        let json = json!({});
        let cfg: CommitConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.mode, CommitMode::Manual);
        assert_eq!(cfg.interval_ms, None);
    }

    // ---- SignalConfig ----

    #[test]
    fn signal_config_defaults() {
        let cfg = SignalConfig::default();
        assert!(cfg.topics().is_empty());
        assert!(cfg.exclude_topics().is_empty());
        assert_eq!(cfg.encoding(), MessageFormat::OtlpProto);
    }

    #[test]
    fn signal_config_deserialize_with_topics() {
        let json = json!({"topics": ["traces-prod", "^traces-team-.*"]});
        let cfg: SignalConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.topics(), &["traces-prod", "^traces-team-.*"]);
        assert!(cfg.exclude_topics().is_empty());
        assert_eq!(cfg.encoding(), MessageFormat::OtlpProto);
    }

    #[test]
    fn signal_config_deserialize_with_exclude_and_encoding() {
        let json = json!({
            "topics": ["^traces-.*"],
            "exclude_topics": ["^traces-test$"],
            "encoding": "otap_proto"
        });
        let cfg: SignalConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.topics(), &["^traces-.*"]);
        assert_eq!(cfg.exclude_topics(), &["^traces-test$"]);
        assert_eq!(cfg.encoding(), MessageFormat::OtapProto);
    }

    // ---- Getters ----

    #[test]
    fn getters_return_expected_values() {
        let cfg: KafkaReceiverConfig =
            KafkaReceiverConfigBuilder::new("broker:9092", "test-group", "test-client")
                .with_traces(SignalConfig {
                    topics: vec!["traces-topic".to_string()],
                    ..Default::default()
                })
                .with_metrics(SignalConfig {
                    topics: vec!["metrics-topic".to_string()],
                    ..Default::default()
                })
                .with_logs(SignalConfig {
                    topics: vec!["logs-topic".to_string()],
                    ..Default::default()
                })
                .try_into()
                .unwrap();
        assert_eq!(cfg.brokers(), "broker:9092");
        assert_eq!(cfg.group_id(), "test-group");
        assert_eq!(cfg.client_id(), "test-client");
        assert_eq!(cfg.traces_topics(), &["traces-topic"]);
        assert_eq!(cfg.metrics_topics(), &["metrics-topic"]);
        assert_eq!(cfg.logs_topics(), &["logs-topic"]);
        assert!(!cfg.is_auto_commit());
        assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
    }

    #[test]
    fn getters_return_empty_for_missing_topics() {
        // Only traces has topics; metrics and logs are empty.
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_commit(CommitConfig {
                mode: CommitMode::Auto,
                interval_ms: Some(1000),
            })
            .try_into()
            .unwrap();
        assert!(!cfg.metrics_topics().is_empty() || cfg.metrics_topics().is_empty()); // always true
        assert!(cfg.metrics_topics().is_empty());
        assert!(cfg.logs_topics().is_empty());
        assert!(cfg.is_auto_commit());
    }

    // ---- validate (empty string fields) ----

    #[test]
    fn validate_empty_brokers_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("", "g", "c").with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("brokers"), "unexpected error: {err}");
    }

    #[test]
    fn validate_empty_client_id_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "").with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("client_id"), "unexpected error: {err}");
    }

    #[test]
    fn validate_empty_group_id_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "", "c").with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("group_id"), "unexpected error: {err}");
    }

    #[test]
    fn validate_empty_group_instance_id_fails() {
        let mut cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        });
        cfg.group_instance_id = Some(String::new());
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("group_instance_id"), "unexpected error: {err}");
    }

    #[test]
    fn validate_empty_message_format_header_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_message_format_header("");
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(
            err.contains("message_format_header"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_empty_resource_attrs_header_key_fails() {
        let mut cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        });
        _ = cfg.resource_attrs_from_headers.insert(
            String::new(),
            HeaderExtraction {
                key: "attr.name".to_string(),
                value_type: AttributeValueType::String,
            },
        );
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("empty header key"), "unexpected error: {err}");
    }

    #[test]
    fn validate_empty_resource_attrs_extraction_key_fails() {
        let mut cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        });
        _ = cfg.resource_attrs_from_headers.insert(
            "X-Header".to_string(),
            HeaderExtraction {
                key: String::new(),
                value_type: AttributeValueType::String,
            },
        );
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(
            err.contains("key can't be empty"),
            "unexpected error: {err}"
        );
    }

    // ---- validate (topics disjoint) ----

    #[test]
    fn validate_all_distinct_is_valid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["traces-topic".to_string()],
                ..Default::default()
            })
            .with_metrics(SignalConfig {
                topics: vec!["metrics-topic".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["logs-topic".to_string()],
                ..Default::default()
            });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    #[test]
    fn validate_all_empty_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c");
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("at least one signal"));
    }

    #[test]
    fn validate_traces_equals_metrics_is_invalid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["same-topic".to_string()],
                ..Default::default()
            })
            .with_metrics(SignalConfig {
                topics: vec!["same-topic".to_string()],
                ..Default::default()
            });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("overlap"));
    }

    #[test]
    fn validate_traces_equals_logs_is_invalid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["same-topic".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["same-topic".to_string()],
                ..Default::default()
            });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("overlap"));
    }

    #[test]
    fn validate_metrics_equals_logs_is_invalid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_metrics(SignalConfig {
                topics: vec!["same-topic".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["same-topic".to_string()],
                ..Default::default()
            });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("overlap"));
    }

    #[test]
    fn validate_one_set_others_empty_is_valid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["only-traces".to_string()],
            ..Default::default()
        });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    #[test]
    fn validate_multi_topic_overlap_across_signals_is_invalid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["topic-a".to_string(), "topic-b".to_string()],
                ..Default::default()
            })
            .with_metrics(SignalConfig {
                topics: vec!["topic-c".to_string(), "topic-b".to_string()],
                ..Default::default()
            });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("overlap"));
    }

    #[test]
    fn validate_multi_topic_disjoint_is_valid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["traces-a".to_string(), "traces-b".to_string()],
                ..Default::default()
            })
            .with_metrics(SignalConfig {
                topics: vec!["metrics-a".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["logs-a".to_string(), "logs-b".to_string()],
                ..Default::default()
            });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    // ---- validate (regex) ----

    #[test]
    fn validate_invalid_regex_topic_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["^traces-(invalid".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("invalid regex topic pattern in traces"));
    }

    #[test]
    fn validate_valid_regex_topic_succeeds() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["^traces-.*".to_string()],
            ..Default::default()
        });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    // ---- validate (exclude_topics) ----

    #[test]
    fn validate_exclude_topics_without_regex_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["traces-prod".to_string()],
            exclude_topics: vec!["^traces-test$".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("exclude_topics is only allowed when"));
    }

    #[test]
    fn validate_exclude_topics_with_regex_succeeds() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["^traces-.*".to_string()],
            exclude_topics: vec!["^traces-test$".to_string()],
            ..Default::default()
        });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    #[test]
    fn validate_exclude_topics_empty_string_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["^traces-.*".to_string()],
            exclude_topics: vec!["".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("non-empty"));
    }

    #[test]
    fn validate_exclude_topics_invalid_regex_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["^traces-.*".to_string()],
            exclude_topics: vec!["^(invalid".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("invalid regex in traces.exclude_topics"));
    }

    // ---- validate (topic names) ----

    #[test]
    fn validate_empty_topic_name_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("traces.topics"), "unexpected error: {err}");
        assert!(err.contains("empty"), "unexpected error: {err}");
    }

    #[test]
    fn validate_dot_topic_name_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_metrics(SignalConfig {
            topics: vec![".".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("metrics.topics"), "unexpected error: {err}");
        assert!(err.contains("ambiguous"), "unexpected error: {err}");
    }

    #[test]
    fn validate_dotdot_topic_name_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_logs(SignalConfig {
            topics: vec!["..".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("logs.topics"), "unexpected error: {err}");
        assert!(err.contains("ambiguous"), "unexpected error: {err}");
    }

    #[test]
    fn validate_topic_name_invalid_chars_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["bad/topic".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("traces.topics"), "unexpected error: {err}");
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    #[test]
    fn validate_topic_name_too_long_fails() {
        let long_topic = "a".repeat(250);
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec![long_topic],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("traces.topics"), "unexpected error: {err}");
        assert!(err.contains("maximum length"), "unexpected error: {err}");
    }

    #[test]
    fn validate_regex_topic_skips_name_validation() {
        // Regex patterns start with '^' and should NOT be validated as literal topic names
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["^traces-.*".to_string()],
            ..Default::default()
        });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    #[test]
    fn validate_mixed_literal_and_regex_topics() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["valid-literal".to_string(), "^traces-.*".to_string()],
            ..Default::default()
        });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    #[test]
    fn validate_invalid_literal_among_regex_topics_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
            topics: vec!["^traces-.*".to_string(), "bad topic".to_string()],
            ..Default::default()
        });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("traces.topics"), "unexpected error: {err}");
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    #[test]
    fn validate_valid_literal_topics_across_signals() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["my-traces".to_string()],
                ..Default::default()
            })
            .with_metrics(SignalConfig {
                topics: vec!["my.metrics".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["my_logs".to_string()],
                ..Default::default()
            });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    // ---- validate (fetch bytes) ----

    #[test]
    fn validate_max_fetch_bytes_less_than_min_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_min_fetch_bytes(100)
            .with_max_fetch_bytes(50);
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("max_fetch_bytes"));
    }

    #[test]
    fn validate_max_partition_fetch_bytes_zero_fails() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_max_partition_fetch_bytes(0);
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("max_partition_fetch_bytes"));
    }

    // ---- validate (commit interval) ----

    #[test]
    fn validate_commit_interval_none_is_valid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_commit(CommitConfig {
                interval_ms: None,
                ..Default::default()
            });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    // REVIEW-FIX(#3 reject-zero-interval): zero commit interval is now invalid.
    #[test]
    fn validate_commit_interval_zero_is_invalid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_commit(CommitConfig {
                interval_ms: Some(0),
                ..Default::default()
            });
        let err = KafkaReceiverConfig::try_from(cfg).unwrap_err();
        assert!(err.contains("must be > 0"), "unexpected error: {err}");
    }

    #[test]
    fn validate_commit_interval_some_value_is_valid() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_commit(CommitConfig {
                interval_ms: Some(5000),
                ..Default::default()
            });
        assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
    }

    // ---- all_topics ----

    #[test]
    fn all_topics_returns_all_configured() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["traces-topic".to_string()],
                ..Default::default()
            })
            .with_metrics(SignalConfig {
                topics: vec!["metrics-topic".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["logs-topic".to_string()],
                ..Default::default()
            })
            .try_into()
            .unwrap();
        let topics = cfg.all_topics();
        assert_eq!(topics.len(), 3);
        assert!(topics.contains(&"traces-topic"));
        assert!(topics.contains(&"metrics-topic"));
        assert!(topics.contains(&"logs-topic"));
    }

    #[test]
    fn all_topics_returns_empty_when_none_configured() {
        // Validation requires at least one signal with topics, so we verify
        // that a builder with no topics fails validation.
        let result = KafkaReceiverConfig::try_from(KafkaReceiverConfigBuilder::new("b", "g", "c"));
        assert!(result.is_err());
    }

    #[test]
    fn all_topics_returns_only_set_topics() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["l".to_string()],
                ..Default::default()
            })
            .try_into()
            .unwrap();
        let topics = cfg.all_topics();
        assert_eq!(topics.len(), 2);
        assert!(topics.contains(&"t"));
        assert!(topics.contains(&"l"));
    }

    #[test]
    fn all_topics_returns_flattened_multi_topic_list() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t1".to_string(), "t2".to_string()],
                ..Default::default()
            })
            .with_metrics(SignalConfig {
                topics: vec!["m1".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["l1".to_string(), "l2".to_string(), "l3".to_string()],
                ..Default::default()
            })
            .try_into()
            .unwrap();
        let topics = cfg.all_topics();
        assert_eq!(topics.len(), 6);
        assert!(topics.contains(&"t1"));
        assert!(topics.contains(&"t2"));
        assert!(topics.contains(&"m1"));
        assert!(topics.contains(&"l1"));
        assert!(topics.contains(&"l2"));
        assert!(topics.contains(&"l3"));
    }

    // ---- build_client_config ----

    #[test]
    fn build_client_config_contains_expected_defaults() {
        let cfg =
            KafkaReceiverConfigBuilder::new("localhost:9092", "otel-collector", "otel-collector");
        let client_config = cfg.build_client_config();

        assert_eq!(
            client_config.get("bootstrap.servers"),
            Some("localhost:9092")
        );
        assert_eq!(client_config.get("group.id"), Some("otel-collector"));
        assert_eq!(client_config.get("client.id"), Some("otel-collector"));
        assert_eq!(client_config.get("enable.auto.commit"), Some("false"));
        assert_eq!(client_config.get("auto.offset.reset"), Some("latest"));
        assert_eq!(
            client_config.get("isolation.level"),
            Some("read_uncommitted")
        );
        // Session management
        assert_eq!(client_config.get("session.timeout.ms"), Some("10000"));
        assert_eq!(client_config.get("heartbeat.interval.ms"), Some("3000"));
        // Fetch tuning
        assert_eq!(client_config.get("fetch.min.bytes"), Some("1"));
        assert_eq!(client_config.get("fetch.max.bytes"), Some("1048576"));
        assert_eq!(client_config.get("fetch.wait.max.ms"), Some("250"));
        assert_eq!(
            client_config.get("max.partition.fetch.bytes"),
            Some("1048576")
        );
        // Default mode is manual — auto offset store should be disabled
        assert_eq!(client_config.get("enable.auto.offset.store"), Some("false"));
    }

    #[test]
    fn build_client_config_auto_commit() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_commit(CommitConfig {
                mode: CommitMode::Auto,
                interval_ms: Some(5000),
            })
            .with_auto_offset_reset(AutoOffsetReset::Earliest)
            .with_isolation_level(IsolationLevel::ReadUncommitted);
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("enable.auto.commit"), Some("true"));
        assert_eq!(client_config.get("auto.commit.interval.ms"), Some("5000"));
        assert_eq!(client_config.get("auto.offset.reset"), Some("earliest"));
        assert_eq!(
            client_config.get("isolation.level"),
            Some("read_uncommitted")
        );
        assert_eq!(client_config.get("enable.auto.offset.store"), Some("true"));
    }

    #[test]
    fn build_client_config_auto_commit_no_interval_omits_property() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_commit(CommitConfig {
            mode: CommitMode::Auto,
            interval_ms: None,
        });
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("enable.auto.commit"), Some("true"));
        assert_eq!(client_config.get("auto.commit.interval.ms"), None);
        assert_eq!(client_config.get("enable.auto.offset.store"), Some("true"));
    }

    #[test]
    fn build_client_config_manual_commit_no_auto_interval() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_commit(CommitConfig {
            mode: CommitMode::Manual,
            interval_ms: Some(3000),
        });
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("enable.auto.commit"), Some("false"));
        // auto.commit.interval.ms should NOT be set for manual commit
        assert_eq!(client_config.get("auto.commit.interval.ms"), None);
        // Manual mode should disable auto offset store
        assert_eq!(client_config.get("enable.auto.offset.store"), Some("false"));
    }

    #[test]
    fn build_client_config_custom_consumer_config() {
        let mut custom = HashMap::new();
        _ = custom.insert("custom.setting".to_string(), "custom-value".to_string());

        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_consumer_config(custom);
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("custom.setting"), Some("custom-value"));
    }

    #[test]
    fn build_client_config_builtin_overrides_custom() {
        let mut custom = HashMap::new();
        _ = custom.insert("bootstrap.servers".to_string(), "custom:1234".to_string());

        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_brokers("real-broker:9092")
            .with_consumer_config(custom);
        let client_config = cfg.build_client_config();
        assert_eq!(
            client_config.get("bootstrap.servers"),
            Some("real-broker:9092")
        );
    }

    #[test]
    fn build_client_config_group_instance_id() {
        let cfg =
            KafkaReceiverConfigBuilder::new("b", "g", "c").with_group_instance_id("instance-1");
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("group.instance.id"), Some("instance-1"));
    }

    #[test]
    fn build_client_config_no_group_instance_id_when_none() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c");
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("group.instance.id"), None);
    }

    // ---- Serde deserialization ----

    #[test]
    fn deserialize_minimal_config() {
        let json = json!({
            "brokers": "kafka:9092",
            "group_id": "my-group",
            "client_id": "my-client",
            "traces": {
                "topics": ["traces"]
            }
        });
        let cfg: KafkaReceiverConfig =
            serde_json::from_value(json).expect("should deserialize minimal config");
        assert_eq!(cfg.brokers(), "kafka:9092");
        assert_eq!(cfg.group_id(), "my-group");
        assert_eq!(cfg.client_id(), "my-client");
        // Defaults
        assert_eq!(cfg.traces_topics(), &["traces"]);
        assert!(cfg.metrics_topics().is_empty());
        assert!(cfg.logs_topics().is_empty());
        assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
        assert!(!cfg.is_auto_commit());
        assert_eq!(cfg.commit_interval_ms(), None);
    }

    #[test]
    fn deserialize_without_required_fields_fails() {
        // brokers, group_id, client_id are required -- empty object should fail
        let json = json!({});
        let result = serde_json::from_value::<KafkaReceiverConfig>(json);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_full_config() {
        let json = json!({
            "brokers": "kafka:9092",
            "group_id": "my-group",
            "client_id": "my-client",
            "group_instance_id": "instance-1",
            "traces": {
                "topics": ["traces"],
                "encoding": "otlp_proto"
            },
            "metrics": {
                "topics": ["metrics"],
                "encoding": "otap_proto"
            },
            "logs": {
                "topics": ["logs"],
                "encoding": "otlp_proto"
            },
            "auto_offset_reset": "earliest",
            "commit": {
                "mode": "auto",
                "interval_ms": 5000
            },
            "isolation_level": "read_uncommitted",
            "session_timeout_ms": 30000,
            "heartbeat_interval_ms": 5000,
            "min_fetch_bytes": 1024,
            "max_fetch_bytes": 2097152,
            "max_fetch_wait_ms": 500,
            "max_partition_fetch_bytes": 2097152,
            "enable_idempotency": true,
            "consumer_config": {"custom.setting": "value"}
        });
        let cfg: KafkaReceiverConfig =
            serde_json::from_value(json).expect("should deserialize full config");
        assert_eq!(cfg.brokers(), "kafka:9092");
        assert_eq!(cfg.group_id(), "my-group");
        assert_eq!(cfg.client_id(), "my-client");
        assert_eq!(cfg.traces_topics(), &["traces"]);
        assert_eq!(cfg.metrics_topics(), &["metrics"]);
        assert_eq!(cfg.logs_topics(), &["logs"]);
        assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
        assert_eq!(cfg.metrics_encoding(), MessageFormat::OtapProto);
        assert_eq!(cfg.logs_encoding(), MessageFormat::OtlpProto);
        assert!(cfg.is_auto_commit());
        assert_eq!(cfg.commit_interval_ms(), Some(5000));
        assert!(cfg.is_idempotent());
    }

    #[test]
    fn deserialize_topics_as_list() {
        let json = json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c",
            "traces": {
                "topics": ["traces-a", "traces-b"]
            },
            "metrics": {
                "topics": ["metrics-one"]
            },
            "logs": {
                "topics": ["logs-x", "logs-y", "logs-z"]
            }
        });
        let cfg: KafkaReceiverConfig =
            serde_json::from_value(json).expect("list topics should work");
        assert_eq!(cfg.traces_topics(), &["traces-a", "traces-b"]);
        assert_eq!(cfg.metrics_topics(), &["metrics-one"]);
        assert_eq!(cfg.logs_topics(), &["logs-x", "logs-y", "logs-z"]);
    }

    #[test]
    fn auto_offset_reset_deserialize_variants() {
        assert_eq!(
            serde_json::from_value::<AutoOffsetReset>(json!("earliest")).unwrap(),
            AutoOffsetReset::Earliest
        );
        assert_eq!(
            serde_json::from_value::<AutoOffsetReset>(json!("latest")).unwrap(),
            AutoOffsetReset::Latest
        );
        assert_eq!(
            serde_json::from_value::<AutoOffsetReset>(json!("error")).unwrap(),
            AutoOffsetReset::Error
        );
    }

    #[test]
    fn isolation_level_deserialize_variants() {
        assert_eq!(
            serde_json::from_value::<IsolationLevel>(json!("read_uncommitted")).unwrap(),
            IsolationLevel::ReadUncommitted
        );
        assert_eq!(
            serde_json::from_value::<IsolationLevel>(json!("read_committed")).unwrap(),
            IsolationLevel::ReadCommitted
        );
    }

    // ---- Default functions ----

    #[test]
    fn default_functions_return_expected_values() {
        assert_eq!(default_auto_offset_reset(), AutoOffsetReset::Latest);
        assert_eq!(default_isolation_level(), IsolationLevel::ReadUncommitted);
        assert_eq!(MessageFormat::default(), MessageFormat::OtlpProto);
        assert_eq!(default_commit_mode(), CommitMode::Manual);
        assert_eq!(default_session_timeout_ms(), 10000);
        assert_eq!(default_heartbeat_interval_ms(), 3000);
        assert_eq!(default_min_fetch_bytes(), 1);
        assert_eq!(default_max_fetch_bytes(), 1_048_576);
        assert_eq!(default_max_fetch_wait_ms(), 250);
        assert_eq!(default_max_partition_fetch_bytes(), 1_048_576);
    }

    // ---- HeaderExtraction + resource_attrs_from_headers ----

    #[test]
    fn header_extraction_deserialize() {
        let json = json!({"key": "tenant.id", "value_type": "string"});
        let extraction: HeaderExtraction = serde_json::from_value(json).unwrap();
        assert_eq!(
            extraction,
            HeaderExtraction {
                key: "tenant.id".to_string(),
                value_type: AttributeValueType::String,
            }
        );
    }

    #[test]
    fn deserialize_config_with_resource_attrs_from_headers() {
        let json = json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["traces"]},
            "resource_attrs_from_headers": {
                "x-tenant-id": {"key": "tenant.id", "value_type": "string"},
                "x-env": {"key": "deployment.env", "value_type": "string"}
            }
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json)
            .expect("should deserialize config with resource_attrs_from_headers");
        let extractors = cfg.resource_attrs_from_headers();
        assert_eq!(extractors.len(), 2);
        assert_eq!(
            extractors.get("x-tenant-id"),
            Some(&HeaderExtraction {
                key: "tenant.id".to_string(),
                value_type: AttributeValueType::String,
            })
        );
        assert_eq!(
            extractors.get("x-env"),
            Some(&HeaderExtraction {
                key: "deployment.env".to_string(),
                value_type: AttributeValueType::String,
            })
        );
    }

    #[test]
    fn getter_returns_resource_attrs_from_headers() {
        let mut extractors = HashMap::new();
        _ = extractors.insert(
            "x-tenant".to_string(),
            HeaderExtraction {
                key: "tenant".to_string(),
                value_type: AttributeValueType::String,
            },
        );
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_resource_attrs_from_headers(extractors.clone())
            .try_into()
            .unwrap();
        assert_eq!(cfg.resource_attrs_from_headers(), &extractors);
    }

    #[test]
    fn attribute_value_type_deserialize_all_variants() {
        assert_eq!(
            serde_json::from_value::<AttributeValueType>(json!("string")).unwrap(),
            AttributeValueType::String
        );
        assert_eq!(
            serde_json::from_value::<AttributeValueType>(json!("bool")).unwrap(),
            AttributeValueType::Bool
        );
        assert_eq!(
            serde_json::from_value::<AttributeValueType>(json!("int")).unwrap(),
            AttributeValueType::Int
        );
        assert_eq!(
            serde_json::from_value::<AttributeValueType>(json!("float")).unwrap(),
            AttributeValueType::Float
        );
    }

    #[test]
    fn header_extraction_deserialize_all_value_types() {
        let cases = vec![
            ("string", AttributeValueType::String),
            ("bool", AttributeValueType::Bool),
            ("int", AttributeValueType::Int),
            ("float", AttributeValueType::Float),
        ];
        for (type_str, expected_type) in cases {
            let json = json!({"key": "k", "value_type": type_str});
            let extraction: HeaderExtraction = serde_json::from_value(json).unwrap();
            assert_eq!(
                extraction,
                HeaderExtraction {
                    key: "k".to_string(),
                    value_type: expected_type,
                }
            );
        }
    }

    // ---- commit config ----

    #[test]
    fn commit_interval_ms_default_value() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .try_into()
            .unwrap();
        assert_eq!(cfg.commit_interval_ms(), None);
    }

    #[test]
    fn commit_interval_ms_set_via_builder() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_commit(CommitConfig {
                interval_ms: Some(3000),
                ..Default::default()
            })
            .try_into()
            .unwrap();
        assert_eq!(cfg.commit_interval_ms(), Some(3000));
    }

    #[test]
    fn build_client_config_sets_auto_commit_interval_when_auto_mode() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_commit(CommitConfig {
            mode: CommitMode::Auto,
            interval_ms: Some(2000),
        });
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("enable.auto.commit"), Some("true"));
        assert_eq!(client_config.get("auto.commit.interval.ms"), Some("2000"));
    }

    // ---- enable_idempotency ----

    #[test]
    fn enable_idempotency_defaults_to_false() {
        let json = json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["t"]}
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json).expect("should deserialize");
        assert!(!cfg.is_idempotent());
    }

    #[test]
    fn enable_idempotency_deserialized_when_true() {
        let json = json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["t"]},
            "enable_idempotency": true
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json).expect("should deserialize");
        assert!(cfg.is_idempotent());
    }

    // ---- per-signal encoding ----

    #[test]
    fn per_signal_encoding_defaults_to_otlp_proto() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .try_into()
            .unwrap();
        assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
        assert_eq!(cfg.metrics_encoding(), MessageFormat::OtlpProto);
        assert_eq!(cfg.logs_encoding(), MessageFormat::OtlpProto);
    }

    #[test]
    fn per_signal_encoding_can_differ() {
        let json = json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["t"], "encoding": "otlp_proto"},
            "metrics": {"topics": ["m"], "encoding": "otap_proto"},
            "logs": {"topics": ["l"], "encoding": "otap_proto"}
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
        assert_eq!(cfg.metrics_encoding(), MessageFormat::OtapProto);
        assert_eq!(cfg.logs_encoding(), MessageFormat::OtapProto);
    }

    // ---- session / fetch tuning deserialization ----

    #[test]
    fn session_and_fetch_tuning_deserialized() {
        let json = json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c",
            "session_timeout_ms": 20000,
            "heartbeat_interval_ms": 5000,
            "min_fetch_bytes": 512,
            "max_fetch_bytes": 2097152,
            "max_fetch_wait_ms": 500,
            "max_partition_fetch_bytes": 2097152,
            "traces": {"topics": ["t"]}
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("session.timeout.ms"), Some("20000"));
        assert_eq!(client_config.get("heartbeat.interval.ms"), Some("5000"));
        assert_eq!(client_config.get("fetch.min.bytes"), Some("512"));
        assert_eq!(client_config.get("fetch.max.bytes"), Some("2097152"));
        assert_eq!(client_config.get("fetch.wait.max.ms"), Some("500"));
        assert_eq!(
            client_config.get("max.partition.fetch.bytes"),
            Some("2097152")
        );
    }

    // ---- Builder methods ----

    #[test]
    fn builder_methods_set_values() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
            .with_group_instance_id("inst-1")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_auto_offset_reset(AutoOffsetReset::Earliest)
            .with_isolation_level(IsolationLevel::ReadUncommitted)
            .with_enable_idempotency(true)
            .try_into()
            .unwrap();
        assert_eq!(cfg.brokers(), "b:9092");
        assert_eq!(cfg.group_id(), "g");
        assert_eq!(cfg.client_id(), "c");
        assert_eq!(cfg.traces_topics(), &["t"]);
        assert!(cfg.is_idempotent());
    }

    // ---- TLS configuration ----

    #[test]
    fn tls_getter_returns_none_by_default() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .try_into()
            .unwrap();
        assert!(cfg.tls().is_none());
    }

    #[test]
    fn with_tls_builder_method() {
        let tls = TlsConfig::new(
            "/certs/ca.pem".to_string(),
            "/certs/client.pem".to_string(),
            "/certs/client-key.pem".to_string(),
            None,
            false,
        );
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_tls(tls)
            .try_into()
            .unwrap();
        let tls = cfg.tls().expect("tls should be set");
        assert_eq!(tls.ca_file(), Some("/certs/ca.pem"));
        assert_eq!(tls.cert_file(), Some("/certs/client.pem"));
        assert_eq!(tls.key_file(), Some("/certs/client-key.pem"));
        assert!(!tls.insecure());
    }

    #[test]
    fn deserialize_config_with_tls() {
        let json = json!({
            "brokers": "kafka:9093",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["traces"]},
            "tls": {
                "ca_file": "/certs/ca.pem",
                "cert_file": "/certs/client.pem",
                "key_file": "/certs/client-key.pem",
                "insecure": true
            }
        });
        let cfg: KafkaReceiverConfig =
            serde_json::from_value(json).expect("should deserialize config with tls");
        let tls = cfg.tls().expect("tls should be present");
        assert_eq!(tls.ca_file(), Some("/certs/ca.pem"));
        assert_eq!(tls.cert_file(), Some("/certs/client.pem"));
        assert_eq!(tls.key_file(), Some("/certs/client-key.pem"));
        assert!(tls.insecure());
    }

    #[test]
    fn deserialize_config_with_tls_insecure_defaults_false() {
        let json = json!({
            "brokers": "kafka:9093",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["traces"]},
            "tls": {
                "ca_file": "/certs/ca.pem",
                "cert_file": "/certs/client.pem",
                "key_file": "/certs/client-key.pem"
            }
        });
        let cfg: KafkaReceiverConfig =
            serde_json::from_value(json).expect("should deserialize config with tls");
        let tls = cfg.tls().expect("tls should be present");
        assert!(!tls.insecure());
    }

    #[test]
    fn build_client_config_no_tls_no_auth_sets_plaintext() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c");
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("security.protocol"), Some("PLAINTEXT"));
        assert_eq!(client_config.get("ssl.ca.location"), None);
    }

    #[test]
    fn build_client_config_tls_only_sets_ssl_protocol() {
        let tls = TlsConfig::new(
            "/certs/ca.pem".to_string(),
            "/certs/client.pem".to_string(),
            "/certs/client-key.pem".to_string(),
            None,
            false,
        );
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_tls(tls);
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("security.protocol"), Some("SSL"));
        assert_eq!(client_config.get("ssl.ca.location"), Some("/certs/ca.pem"));
        assert_eq!(
            client_config.get("ssl.certificate.location"),
            Some("/certs/client.pem")
        );
        assert_eq!(
            client_config.get("ssl.key.location"),
            Some("/certs/client-key.pem")
        );
        assert_eq!(
            client_config.get("enable.ssl.certificate.verification"),
            Some("true")
        );
    }

    #[test]
    fn build_client_config_tls_insecure_disables_verification() {
        let tls = TlsConfig::new(
            "/certs/ca.pem".to_string(),
            "/certs/client.pem".to_string(),
            "/certs/client-key.pem".to_string(),
            None,
            true,
        );
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_tls(tls);
        let client_config = cfg.build_client_config();
        assert_eq!(
            client_config.get("enable.ssl.certificate.verification"),
            Some("false")
        );
    }

    #[test]
    #[cfg(feature = "aws")]
    fn build_client_config_tls_with_sasl_sets_sasl_ssl_protocol() {
        let json = json!({
            "brokers": "kafka:9093",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["t"]},
            "auth": {
                "sasl": {
                    "mechanism": "AWS_MSK_IAM_OAUTHBEARER",
                    "aws_msk": {"region": "us-east-1"}
                }
            },
            "tls": {
                "ca_file": "/certs/ca.pem",
                "cert_file": "/certs/client.pem",
                "key_file": "/certs/client-key.pem"
            }
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("security.protocol"), Some("SASL_SSL"));
        assert_eq!(client_config.get("ssl.ca.location"), Some("/certs/ca.pem"));
        assert_eq!(client_config.get("sasl.mechanism"), Some("OAUTHBEARER"));
    }

    #[test]
    #[cfg(feature = "aws")]
    fn build_client_config_aws_msk_without_tls_sets_sasl_ssl() {
        let json = json!({
            "brokers": "kafka:9093",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["t"]},
            "auth": {
                "sasl": {
                    "mechanism": "AWS_MSK_IAM_OAUTHBEARER",
                    "aws_msk": {"region": "us-east-1"}
                }
            }
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("security.protocol"), Some("SASL_SSL"));
        assert_eq!(client_config.get("sasl.mechanism"), Some("OAUTHBEARER"));
        // No TLS config set, so SSL cert fields should not be present
        assert_eq!(client_config.get("ssl.ca.location"), None);
    }

    #[test]
    fn build_client_config_sasl_without_msk_and_no_tls_sets_sasl_plaintext() {
        let json = json!({
            "brokers": "kafka:9092",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["t"]},
            "auth": {
                "sasl": {
                    "mechanism": "PLAIN",
                    "username": "user",
                    "password": "pass"
                }
            }
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
        let client_config = cfg.build_client_config();
        assert_eq!(
            client_config.get("security.protocol"),
            Some("SASL_PLAINTEXT")
        );
        assert_eq!(client_config.get("sasl.mechanism"), Some("PLAIN"));
        assert_eq!(client_config.get("sasl.username"), Some("user"));
        assert_eq!(client_config.get("sasl.password"), Some("pass"));
    }
    // ---- RebalanceStrategy ----

    #[test]
    fn rebalance_strategy_to_librdkafka_value_range() {
        assert_eq!(RebalanceStrategy::Range.to_librdkafka_value(), "range");
    }

    #[test]
    fn rebalance_strategy_to_librdkafka_value_round_robin() {
        assert_eq!(
            RebalanceStrategy::RoundRobin.to_librdkafka_value(),
            "roundrobin"
        );
    }

    #[test]
    fn rebalance_strategy_to_librdkafka_value_cooperative_sticky() {
        assert_eq!(
            RebalanceStrategy::CooperativeSticky.to_librdkafka_value(),
            "cooperative-sticky"
        );
    }

    #[test]
    fn build_client_config_sets_rebalance_strategy() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_rebalance_strategy(RebalanceStrategy::CooperativeSticky);
        let client_config = cfg.build_client_config();
        assert_eq!(
            client_config.get("partition.assignment.strategy"),
            Some("cooperative-sticky")
        );
    }

    #[test]
    fn build_client_config_no_rebalance_strategy_when_none() {
        let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c");
        let client_config = cfg.build_client_config();
        assert_eq!(client_config.get("partition.assignment.strategy"), None);
    }

    #[test]
    fn deserialize_config_with_rebalance_strategy() {
        let json = json!({
            "brokers": "b:9092",
            "group_id": "g",
            "client_id": "c",
            "traces": {"topics": ["t"]},
            "rebalance_strategy": "cooperative_sticky"
        });
        let cfg: KafkaReceiverConfig = serde_json::from_value(json)
            .expect("should deserialize config with rebalance_strategy");
        assert_eq!(
            cfg.rebalance_strategy(),
            Some(RebalanceStrategy::CooperativeSticky)
        );
    }

    #[test]
    fn rebalance_strategy_getter_returns_none_by_default() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .try_into()
            .unwrap();
        assert_eq!(cfg.rebalance_strategy(), None);
    }

    #[test]
    fn builder_with_rebalance_strategy() {
        let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig {
                topics: vec!["t".to_string()],
                ..Default::default()
            })
            .with_rebalance_strategy(RebalanceStrategy::RoundRobin)
            .try_into()
            .unwrap();
        assert_eq!(
            cfg.rebalance_strategy(),
            Some(RebalanceStrategy::RoundRobin)
        );
    }
}
