// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration structures for the Kafka exporter.

use super::topic_regex;
pub use crate::common::kafka::TlsConfig;
use crate::common::kafka::auth::Auth;
use crate::common::kafka::security::{apply_sasl_config, resolve_security_protocol};
use crate::common::kafka::{
    DebugContext, LogLevel, MessageFormat, debug_list_to_string, default_message_format_header,
    validate_kafka_topic,
};
use rdkafka::ClientConfig;
use serde::Deserialize;
use std::collections::HashMap;

/// rdkafka configuration keys that correspond to first-class
/// [`KafkaExporterConfig`] fields. Entries in `producer_config` using
/// these keys may be overwritten when the exporter builds its rdkafka
/// client configuration.
pub(crate) const MANAGED_PRODUCER_CONFIG_KEYS: &[&str] = &[
    "bootstrap.servers",
    "client.id",
    "message.timeout.ms",
    "compression.type",
    "request.required.acks",
    "message.max.bytes",
    "linger.ms",
    "partitioner",
    "allow.auto.create.topics",
    "security.protocol",
    "ssl.ca.location",
    "ssl.certificate.location",
    "ssl.key.location",
    "ssl.key.password",
    "enable.ssl.certificate.verification",
    "sasl.mechanism",
    "sasl.username",
    "sasl.password",
    "debug",
];

/// Per-signal exporter configuration.
///
/// Each signal type (traces, metrics, logs) can optionally have its own topic
/// and encoding format. When a signal is `None` in [`KafkaExporterConfig`],
/// that signal type will not be exported.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SignalConfig {
    /// Kafka topic to produce messages to.
    topic: String,

    /// Encoding format for messages on this signal's topic.
    #[serde(default)]
    encoding: MessageFormat,

    /// Transport header name to look up for dynamic topic routing.
    ///
    /// When set and the header is present in the pdata context, its value
    /// becomes the Kafka destination topic instead of the static `topic` field.
    ///
    /// The lookup matches on the header's normalized logical name. Captured
    /// transport headers are lowercased on ingress, so this value is lowercased
    /// during config validation ([`KafkaExporterConfig::try_from`]) to match --
    /// e.g., `"X-Target-Topic"` is stored as `"x-target-topic"`. If a capture
    /// policy stores the header under a custom `store_as` name, this value must
    /// equal that stored name.
    #[serde(default)]
    topic_from_transport_header: Option<String>,

    /// Enable partitioning by transport headers (default: false).
    ///
    /// When `true`, all transport headers from the pdata context are hashed
    /// (by normalized name and raw value) to produce a deterministic partition
    /// key. This ensures that requests carrying the same set of transport
    /// headers (e.g., same tenant ID, same auth token) are routed to the same
    /// Kafka partition, regardless of original header casing.
    ///
    /// Combine with [`KafkaExporterConfig::partitioning_strategy`] to control
    /// which hashing algorithm librdkafka uses to map the key to a partition.
    #[serde(default)]
    partition_by_transport_headers: bool,

    /// Operator allowlist of exact topic names permitted for
    /// header-supplied (dynamic) routing.
    ///
    /// When non-empty, a topic selected via `topic_from_transport_header` must
    /// exactly match one of these entries (or fully match a pattern in
    /// [`SignalConfig::allowed_topics_regex`]); otherwise the batch is
    /// permanently nacked. Empty (the default) means no exact-match constraint.
    ///
    /// This constrains where a client-controlled routing header may direct data;
    /// it does not affect the static `topic`, which the operator already
    /// controls.
    #[serde(default)]
    allowed_topics: Vec<String>,

    /// Operator allowlist of regex patterns permitted for header-supplied
    /// (dynamic) routing.
    ///
    /// When non-empty, a topic selected via `topic_from_transport_header` must
    /// match one of these regex patterns (or exactly match an entry in
    /// [`SignalConfig::allowed_topics`]); otherwise the batch is permanently
    /// nacked. Empty (the default) means no regex constraint. Entries must be
    /// valid regular expressions and are compiled once at exporter construction
    /// (and on reconfigure); an invalid pattern is a configuration error.
    ///
    /// This constrains where a client-controlled routing header may direct data;
    /// it does not affect the static `topic`, which the operator already
    /// controls.
    #[serde(default)]
    allowed_topics_regex: Vec<String>,
}

impl SignalConfig {
    /// Create a new signal configuration.
    #[must_use]
    pub fn new(topic: String, encoding: MessageFormat) -> Self {
        Self {
            topic,
            encoding,
            topic_from_transport_header: None,
            partition_by_transport_headers: false,
            allowed_topics: Vec::new(),
            allowed_topics_regex: Vec::new(),
        }
    }

    /// The Kafka topic to produce messages to.
    #[must_use]
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// The encoding format for messages on this signal's topic.
    #[must_use]
    pub fn encoding(&self) -> MessageFormat {
        self.encoding
    }

    /// The transport header name for dynamic topic routing, if set.
    #[must_use]
    pub fn topic_from_transport_header(&self) -> Option<&str> {
        self.topic_from_transport_header.as_deref()
    }

    /// Set the transport header name for dynamic topic routing.
    #[must_use]
    pub fn with_topic_from_transport_header(mut self, key: impl Into<String>) -> Self {
        self.topic_from_transport_header = Some(key.into());
        self
    }

    /// Whether partitioning by transport headers is enabled for this signal.
    #[must_use]
    pub fn partition_by_transport_headers(&self) -> bool {
        self.partition_by_transport_headers
    }

    /// Set the partition by transport headers flag.
    #[must_use]
    pub fn with_partition_by_transport_headers(mut self, enabled: bool) -> Self {
        self.partition_by_transport_headers = enabled;
        self
    }

    /// The exact-match allowlist of topics permitted for header-supplied
    /// routing. Empty means no exact-match constraint.
    #[must_use]
    pub fn allowed_topics(&self) -> &[String] {
        &self.allowed_topics
    }

    /// Set the exact-match allowlist of topics permitted for header-supplied
    /// routing.
    #[must_use]
    pub fn with_allowed_topics<I, S>(mut self, topics: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_topics = topics.into_iter().map(Into::into).collect();
        self
    }

    /// The regex allowlist for topics permitted for header-supplied routing.
    /// Empty means no regex constraint. Entries must be valid regular
    /// expressions and are matched against the header-supplied topic.
    #[must_use]
    pub fn allowed_topics_regex(&self) -> &[String] {
        &self.allowed_topics_regex
    }

    /// Set the regex allowlist for topics permitted for header-supplied
    /// routing.
    #[must_use]
    pub fn with_allowed_topics_regex<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_topics_regex = patterns.into_iter().map(Into::into).collect();
        self
    }
}

/// Required acknowledgments from Kafka brokers before a produce request is
/// considered complete.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredAcks {
    /// No acknowledgment (acks=0). Fastest, but no delivery guarantee.
    None,
    /// Leader acknowledgment (acks=1). Default -- the leader writes and acks
    /// without waiting for followers.
    One,
    /// All in-sync replicas must acknowledge (acks=-1). Strongest durability
    /// guarantee.
    All,
}

impl RequiredAcks {
    /// Returns the librdkafka-compatible string value for
    /// `request.required.acks`.
    #[must_use]
    pub fn as_kafka_value(&self) -> &'static str {
        match self {
            RequiredAcks::None => "0",
            RequiredAcks::One => "1",
            RequiredAcks::All => "-1",
        }
    }
}

/// Builder for the Kafka exporter.
///
/// Use [`KafkaExporterConfigBuilder::new`] with the `with_*` builder methods
/// to configure settings, then convert to a validated [`KafkaExporterConfig`]
/// via [`TryFrom`] / [`TryInto`].
///
/// This type is also the serde deserialization target -- the validated
/// [`KafkaExporterConfig`] uses `#[serde(try_from)]` to run validation
/// automatically during deserialization.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct KafkaExporterConfigBuilder {
    /// Kafka broker addresses (comma-separated, e.g., "kafka1:9092,kafka2:9092").
    brokers: String,

    /// Kafka client ID sent to the brokers for tracking. Required.
    client_id: String,

    /// Per-signal configuration for traces. `None` means traces are not
    /// exported.
    #[serde(default)]
    traces: Option<SignalConfig>,

    /// Per-signal configuration for metrics. `None` means metrics are not
    /// exported.
    #[serde(default)]
    metrics: Option<SignalConfig>,

    /// Per-signal configuration for logs. `None` means logs are not exported.
    #[serde(default)]
    logs: Option<SignalConfig>,

    /// Request timeout in milliseconds (default: 5000).
    #[serde(default = "default_timeout_ms")]
    timeout_ms: u64,

    /// Optional compression type for Kafka messages.
    #[serde(default)]
    compression: Option<CompressionType>,

    /// Required acknowledgments from brokers (default: `one`).
    ///
    /// - `none` (acks=0): fire-and-forget, no delivery guarantee
    /// - `one`  (acks=1): leader writes and acks without waiting for followers
    /// - `all`  (acks=-1): all in-sync replicas must acknowledge
    #[serde(default = "default_required_acks")]
    required_acks: RequiredAcks,

    /// Maximum size of a message in bytes (default: 1,000,000).
    ///
    /// Corresponds to librdkafka's `message.max.bytes`.
    #[serde(default = "default_max_message_bytes")]
    max_message_bytes: usize,

    /// Artificial delay in milliseconds to wait for additional messages before
    /// sending a batch to the broker (default: 5).
    ///
    /// Corresponds to librdkafka's `linger.ms`. Higher values improve
    /// throughput at the cost of latency.
    #[serde(default = "default_linger_ms")]
    linger_ms: u32,

    /// Maximum number of Kafka deliveries the exporter keeps in flight
    /// concurrently before it stops accepting new pdata (default: 10).
    ///
    /// Each accepted pdata is encoded and enqueued to librdkafka, and its
    /// delivery future is added to a bounded in-flight set. When the set is
    /// full the exporter parks the next pdata and only resumes intake as
    /// deliveries complete, so this value bounds both concurrency and
    /// in-flight memory and propagates backpressure upstream.
    ///
    /// The default of `10` pipelines deliveries for higher throughput.
    ///
    /// Must be in the range `1` to `100000`. A value of `0` is rejected because
    /// it would stall the exporter; values above `100000` are rejected because
    /// they exceed librdkafka's default producer queue depth and only inflate
    /// in-flight memory without increasing pipelining.
    #[serde(default = "default_max_in_flight")]
    max_in_flight: usize,

    /// Authentication configuration (same structure as the Kafka receiver).
    #[serde(default)]
    auth: Option<Auth>,

    /// Optional TLS settings for communicating with the broker.
    #[serde(default)]
    tls: Option<TlsConfig>,

    /// Partitioner strategy for Kafka message routing.
    ///
    /// Controls which librdkafka partitioner algorithm maps partition keys to
    /// Kafka partitions (default: `consistent_random`).
    #[serde(default = "default_partitioning_strategy")]
    partitioning_strategy: PartitionerStrategy,

    /// Whether the broker may auto-create topics this exporter produces to
    /// (maps to librdkafka `allow.auto.create.topics`).
    ///
    /// Defaults to `true`, This value is always written to the
    /// client config and takes precedence over any `producer_config` entry for
    /// the same key.
    ///
    /// Security note: because a client-controlled routing header
    /// (`topic_from_transport_header`) can select the destination topic, leaving
    /// auto-creation enabled lets a client cause the broker to spawn arbitrary
    /// topics. Operators who route by header (or otherwise want default-deny)
    /// should set this to `false`.
    #[serde(default = "default_allow_auto_create_topics")]
    allow_auto_create_topics: bool,

    /// Kafka header key for the message format indicator.
    ///
    /// The exporter writes the encoding format (`otlp` or `otap`) under this
    /// header key on every outgoing Kafka message. This allows consumers to
    /// detect the message encoding from the header.
    ///
    /// Defaults to `"MessageFormat"`. Users can override the key name but the
    /// header is always written.
    #[serde(default = "default_message_format_header")]
    message_format_header: String,

    /// List of librdkafka debug contexts to enable.
    ///
    /// Useful for troubleshooting Kafka connectivity, authentication, or
    /// message delivery issues. Each entry maps to one of librdkafka's
    /// debug context flags.
    ///
    /// Example:
    /// ```yaml
    /// debug:
    ///   - broker
    ///   - security
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

    /// Additional librdkafka producer settings as key-value string pairs.
    ///
    /// These are applied first; built-in options (brokers, compression, etc.)
    /// take precedence on conflict. Use this for advanced tuning knobs that
    /// are not directly exposed as config fields.
    ///
    /// Example:
    /// ```yaml
    /// producer_config:
    ///   "queue.buffering.max.messages": "100000"
    /// ```
    #[serde(default)]
    producer_config: HashMap<String, String>,
}

impl KafkaExporterConfigBuilder {
    // ---- Constructor and builder methods ----

    /// Create a new exporter configuration builder with required fields and
    /// defaults.
    ///
    /// Use the `with_*` builder methods to override individual settings, then
    /// call `.try_into()` to obtain a validated [`KafkaExporterConfig`].
    #[must_use]
    pub fn new(brokers: impl Into<String>, client_id: impl Into<String>) -> Self {
        Self {
            brokers: brokers.into(),
            client_id: client_id.into(),
            traces: None,
            metrics: None,
            logs: None,
            timeout_ms: default_timeout_ms(),
            compression: None,
            required_acks: default_required_acks(),
            max_message_bytes: default_max_message_bytes(),
            linger_ms: default_linger_ms(),
            max_in_flight: default_max_in_flight(),
            auth: None,
            tls: None,
            partitioning_strategy: default_partitioning_strategy(),
            allow_auto_create_topics: default_allow_auto_create_topics(),
            message_format_header: default_message_format_header(),
            debug: None,
            log_level: None,
            producer_config: HashMap::new(),
        }
    }

    /// Set the traces signal configuration.
    #[must_use]
    pub fn with_traces(mut self, traces: SignalConfig) -> Self {
        self.traces = Some(traces);
        self
    }

    /// Set the metrics signal configuration.
    #[must_use]
    pub fn with_metrics(mut self, metrics: SignalConfig) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Set the logs signal configuration.
    #[must_use]
    pub fn with_logs(mut self, logs: SignalConfig) -> Self {
        self.logs = Some(logs);
        self
    }

    /// Set the request timeout in milliseconds.
    #[must_use]
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set the compression type.
    #[must_use]
    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = Some(compression);
        self
    }

    /// Set the required acknowledgment level.
    #[must_use]
    pub fn with_required_acks(mut self, acks: RequiredAcks) -> Self {
        self.required_acks = acks;
        self
    }

    /// Set the maximum message size in bytes.
    #[must_use]
    pub fn with_max_message_bytes(mut self, bytes: usize) -> Self {
        self.max_message_bytes = bytes;
        self
    }

    /// Set the linger time in milliseconds.
    #[must_use]
    pub fn with_linger_ms(mut self, ms: u32) -> Self {
        self.linger_ms = ms;
        self
    }

    /// Set the maximum number of concurrent in-flight Kafka deliveries.
    ///
    /// Must be in the range `1` to `100000` (validated when the config is
    /// built); values outside that range are rejected.
    #[must_use]
    pub fn with_max_in_flight(mut self, max_in_flight: usize) -> Self {
        self.max_in_flight = max_in_flight;
        self
    }

    /// Set the authentication configuration.
    #[must_use]
    pub fn with_auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set the TLS configuration.
    #[must_use]
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Set additional librdkafka producer settings.
    #[must_use]
    pub fn with_producer_config(mut self, config: HashMap<String, String>) -> Self {
        self.producer_config = config;
        self
    }

    /// Set the partitioner strategy.
    #[must_use]
    pub fn with_partitioning_strategy(mut self, strategy: PartitionerStrategy) -> Self {
        self.partitioning_strategy = strategy;
        self
    }

    /// Set whether the broker may auto-create topics (default: `true`).
    #[must_use]
    pub fn with_allow_auto_create_topics(mut self, allow: bool) -> Self {
        self.allow_auto_create_topics = allow;
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
    /// `vec![DebugContext::Broker, DebugContext::Security]`).
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
    ///
    /// Produces an rdkafka [`ClientConfig`] from all exporter settings.
    /// Custom `producer_config` entries are applied first; built-in options
    /// take precedence on conflict (same pattern as the receiver's
    /// `build_client_config`).
    #[must_use]
    pub fn build_client_config(&self) -> ClientConfig {
        let mut config = ClientConfig::new();

        // Apply user-provided producer_config first. Built-in options set below
        // take precedence on conflict (same pattern as the receiver's
        // consumer_config).
        for (key, value) in &self.producer_config {
            _ = config.set(key, value);
        }

        // Core connection settings
        _ = config.set("bootstrap.servers", &self.brokers);
        _ = config.set("client.id", &self.client_id);
        _ = config.set("message.timeout.ms", self.timeout_ms.to_string());

        // Compression
        if let Some(ref compression) = self.compression {
            _ = config.set("compression.type", compression.as_str());
        }

        // Producer tuning
        _ = config.set("request.required.acks", self.required_acks.as_kafka_value());
        _ = config.set("message.max.bytes", self.max_message_bytes.to_string());
        _ = config.set("linger.ms", self.linger_ms.to_string());

        // Partitioner strategy
        _ = config.set("partitioner", self.partitioning_strategy.as_kafka_value());

        // Topic auto-creation posture. Always written (rather than relying on
        // the librdkafka default) so the emitted value is explicit and takes
        // precedence over any `producer_config` override.
        _ = config.set(
            "allow.auto.create.topics",
            if self.allow_auto_create_topics {
                "true"
            } else {
                "false"
            },
        );

        // Security protocol, TLS, and SASL settings (shared with receiver)
        let protocol = resolve_security_protocol(self.tls.as_ref(), self.auth.as_ref());
        _ = config.set("security.protocol", protocol);

        if let Some(ref tls) = self.tls {
            tls.apply_to_client_config(&mut config);
        }

        apply_sasl_config(self.auth.as_ref(), &mut config);

        // Debug contexts and log level (applied last so they override any
        // value that might have been set via producer_config).
        if let Some(ref contexts) = self.debug {
            _ = config.set("debug", debug_list_to_string(contexts));
        }
        if let Some(level) = self.log_level {
            _ = config.set_log_level(level.to_rdkafka());
        }

        config
    }
}

/// Validated configuration for the Kafka exporter.
///
/// This type guarantees that all structural invariants hold:
/// - `brokers` is non-empty
/// - `client_id` is non-empty
/// - At least one signal (`traces`, `metrics`, or `logs`) is configured
///
/// Construct via [`TryFrom<KafkaExporterConfigBuilder>`] or by deserializing
/// directly (serde runs validation automatically via `try_from`).
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "KafkaExporterConfigBuilder")]
pub struct KafkaExporterConfig(KafkaExporterConfigBuilder);

/// Validates a signal's static topic and its dynamic-routing allowlist entries.
///
/// Errors are prefixed with the field name (e.g. `topic:`,
/// `allowed_topics[0]:`, `allowed_topics_regex[1]:`) so the caller can prepend
/// the signal name.
///
/// Each `allowed_topics_regex` pattern is compiled via
/// `topic_regex::compile_anchor_and_validate` -- validated
/// as a standalone regex and then anchored to a whole-topic match as
/// `\A(?:<pattern>)\z` -- so an invalid pattern (including one crafted to break
/// out of the anchoring wrapper) fails fast at config time, including through
/// the factory `validate_config` path, which runs this validation without
/// constructing an exporter.
fn validate_signal_topics(signal: &SignalConfig) -> Result<(), String> {
    validate_kafka_topic(&signal.topic).map_err(|e| format!("topic: {e}"))?;
    for (i, t) in signal.allowed_topics.iter().enumerate() {
        validate_kafka_topic(t).map_err(|e| format!("allowed_topics[{i}]: {e}"))?;
    }
    for (i, p) in signal.allowed_topics_regex.iter().enumerate() {
        // Compile via the same helper the exporter uses at runtime so validation
        // and runtime compilation stay in lock-step -- including the standalone
        // check that rejects a pattern which would otherwise break out of the
        // whole-topic anchoring wrapper.
        topic_regex::compile_anchor_and_validate(p)
            .map(drop)
            .map_err(|message| {
                format!("allowed_topics_regex[{i}]: invalid regex '{p}': {message}")
            })?;
    }
    Ok(())
}

impl TryFrom<KafkaExporterConfigBuilder> for KafkaExporterConfig {
    type Error = String;

    fn try_from(mut builder: KafkaExporterConfigBuilder) -> Result<Self, Self::Error> {
        if builder.client_id.is_empty() {
            return Err("client_id can't be empty".to_string());
        }

        if builder.brokers.is_empty() {
            return Err("brokers can't be empty".to_string());
        }

        if builder.message_format_header.is_empty() {
            return Err("message_format_header can't be empty".to_string());
        }

        if builder.traces.is_none() && builder.metrics.is_none() && builder.logs.is_none() {
            return Err(
                "at least one signal (traces, metrics, or logs) must be configured".to_string(),
            );
        }

        // Validate the request timeout. `timeout_ms` is assigned to librdkafka's
        // `message.timeout.ms`, where a value of `0` means an *infinite* delivery
        // timeout. Because the exporter awaits each delivery inline, an infinite
        // (or unreasonably large) timeout would let a broker outage block the
        // main loop from ever servicing the Shutdown control message. Reject `0`
        // and anything above `MAX_TIMEOUT_MS` so shutdown always stays bounded.
        if builder.timeout_ms == 0 {
            return Err(
                "timeout_ms must be > 0; a value of 0 sets message.timeout.ms to infinite in \
                 librdkafka and can prevent the exporter from shutting down"
                    .to_string(),
            );
        }
        if builder.timeout_ms > MAX_TIMEOUT_MS {
            return Err(format!(
                "timeout_ms must be <= {MAX_TIMEOUT_MS} (30s); larger values can delay shutdown"
            ));
        }

        // Reject a non-positive concurrency bound. `max_in_flight` caps the
        // number of outstanding deliveries; a value of `0` would stall the
        // exporter because no delivery could ever be admitted.
        if builder.max_in_flight == 0 {
            return Err(
                "max_in_flight must be > 0; a value of 0 would stall the exporter \
                 (no deliveries could ever be outstanding)"
                    .to_string(),
            );
        }
        if builder.max_in_flight > MAX_IN_FLIGHT_LIMIT {
            return Err(format!(
                "max_in_flight must be <= {MAX_IN_FLIGHT_LIMIT}; larger values cannot \
                 increase pipelining beyond librdkafka's default producer queue depth \
                 and only inflate in-flight memory"
            ));
        }

        // Validate topic names and dynamic-routing allowlists for each signal.
        if let Some(ref signal) = builder.traces {
            validate_signal_topics(signal).map_err(|e| format!("traces.{e}"))?;
        }
        if let Some(ref signal) = builder.metrics {
            validate_signal_topics(signal).map_err(|e| format!("metrics.{e}"))?;
        }
        if let Some(ref signal) = builder.logs {
            validate_signal_topics(signal).map_err(|e| format!("logs.{e}"))?;
        }

        // Validate auth configuration when present
        if let Some(ref auth) = builder.auth {
            auth.validate().map_err(|e| format!("auth: {e}"))?;
        }

        // Validate TLS configuration when present
        if let Some(ref tls) = builder.tls {
            tls.validate().map_err(|e| format!("tls: {e}"))?;
        }

        // Normalize each signal's dynamic-routing header key to match how
        // transport headers store their logical names. Captured headers are
        // lowercased on ingress (`wire_name.to_ascii_lowercase()`), so a natural
        // config like `X-Target-Topic` would otherwise never match and silently
        // fall back to the static topic. Normalizing once here means the router
        // can do a plain equality check without re-normalizing per message.
        for signal in [
            builder.traces.as_mut(),
            builder.metrics.as_mut(),
            builder.logs.as_mut(),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(header) = signal.topic_from_transport_header.as_mut() {
                *header = header.to_ascii_lowercase();
            }
        }

        Ok(Self(builder))
    }
}

impl KafkaExporterConfig {
    /// Get the broker addresses.
    #[must_use]
    pub fn brokers(&self) -> &str {
        &self.0.brokers
    }

    /// Get the client ID.
    #[must_use]
    pub fn client_id(&self) -> &str {
        &self.0.client_id
    }

    /// Get the traces signal configuration, if set.
    #[must_use]
    pub fn traces(&self) -> Option<&SignalConfig> {
        self.0.traces.as_ref()
    }

    /// Get the metrics signal configuration, if set.
    #[must_use]
    pub fn metrics(&self) -> Option<&SignalConfig> {
        self.0.metrics.as_ref()
    }

    /// Get the logs signal configuration, if set.
    #[must_use]
    pub fn logs(&self) -> Option<&SignalConfig> {
        self.0.logs.as_ref()
    }

    /// Request timeout in milliseconds.
    #[must_use]
    pub fn timeout_ms(&self) -> u64 {
        self.0.timeout_ms
    }

    /// Get the compression type, if set.
    #[must_use]
    pub fn compression(&self) -> Option<&CompressionType> {
        self.0.compression.as_ref()
    }

    /// Get the required acknowledgment level.
    #[must_use]
    pub fn required_acks(&self) -> RequiredAcks {
        self.0.required_acks
    }

    /// Maximum message size in bytes.
    #[must_use]
    pub fn max_message_bytes(&self) -> usize {
        self.0.max_message_bytes
    }

    /// Linger time in milliseconds.
    #[must_use]
    pub fn linger_ms(&self) -> u32 {
        self.0.linger_ms
    }

    /// Maximum number of concurrent in-flight Kafka deliveries.
    #[must_use]
    pub fn max_in_flight(&self) -> usize {
        self.0.max_in_flight
    }

    /// Get the authentication configuration, if set.
    #[must_use]
    pub fn auth(&self) -> Option<&Auth> {
        self.0.auth.as_ref()
    }

    /// Get the TLS configuration, if set.
    #[must_use]
    pub fn tls(&self) -> Option<&TlsConfig> {
        self.0.tls.as_ref()
    }

    /// Get the additional producer configuration key-value pairs.
    #[must_use]
    pub fn producer_config(&self) -> &HashMap<String, String> {
        &self.0.producer_config
    }

    /// Get the partitioning strategy.
    #[must_use]
    pub fn partitioning_strategy(&self) -> PartitionerStrategy {
        self.0.partitioning_strategy
    }

    /// Whether the broker may auto-create topics this exporter produces to.
    #[must_use]
    pub fn allow_auto_create_topics(&self) -> bool {
        self.0.allow_auto_create_topics
    }

    /// The Kafka header key used for the message format indicator.
    ///
    /// Defaults to `"MessageFormat"`. The format header is always written on
    /// every outgoing Kafka message.
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

    /// Returns any `producer_config` keys that overlap with rdkafka keys
    /// managed by first-class config fields and may be overwritten.
    #[must_use]
    pub fn overridden_producer_config_keys(&self) -> Vec<&str> {
        self.0
            .producer_config
            .keys()
            .filter(|k| MANAGED_PRODUCER_CONFIG_KEYS.contains(&k.as_str()))
            .map(String::as_str)
            .collect()
    }

    /// Build Kafka client configuration.
    #[must_use]
    pub fn build_client_config(&self) -> ClientConfig {
        self.0.build_client_config()
    }
}

// ---- Default functions for serde ----

/// Maximum accepted `timeout_ms` (30 seconds).
///
/// `timeout_ms` maps to librdkafka's `message.timeout.ms`. Values above this
/// ceiling (and `0`, which librdkafka interprets as infinite) are rejected at
/// config validation time so the exporter's shutdown path always stays bounded.
pub(crate) const MAX_TIMEOUT_MS: u64 = 30_000;

/// Maximum accepted `max_in_flight` (100,000 deliveries).
///
/// This matches librdkafka's default `queue.buffering.max.messages`, so any
/// valid `max_in_flight` stays within the producer's queue depth and cannot, on
/// its own, drive the queue to `QueueFull`. Values above this ceiling cannot
/// increase pipelining beyond what the producer queue admits and only inflate
/// the exporter's in-flight memory, so they are rejected at config validation.
pub(crate) const MAX_IN_FLIGHT_LIMIT: usize = 100_000;

/// Default timeout in milliseconds.
fn default_timeout_ms() -> u64 {
    5000
}

/// Default required acks.
fn default_required_acks() -> RequiredAcks {
    RequiredAcks::One
}

/// Default maximum message size in bytes.
fn default_max_message_bytes() -> usize {
    1_000_000
}

/// Default linger in milliseconds.
fn default_linger_ms() -> u32 {
    5
}

/// Default maximum number of concurrent in-flight Kafka deliveries.
///
/// Defaults to `10`, which pipelines up to ten deliveries at a time for
/// higher throughput while keeping in-flight memory bounded.
fn default_max_in_flight() -> usize {
    10
}

/// Default partitioner strategy.
fn default_partitioning_strategy() -> PartitionerStrategy {
    PartitionerStrategy::ConsistentRandom
}

/// Default topic auto-creation posture.
fn default_allow_auto_create_topics() -> bool {
    true
}

/// Compression type for Kafka messages.
///
/// All variants are accepted by the config parser and passed through to
/// rdkafka's `compression.type` setting. Currently only `Gzip` and `Zstd`
/// have been validated in production; `Snappy` and `Lz4` are defined for
/// completeness but have not been end-to-end tested.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    /// Gzip compression.
    Gzip,
    /// Snappy compression.
    Snappy,
    /// LZ4 compression.
    Lz4,
    /// Zstandard compression.
    Zstd,
}

impl CompressionType {
    /// Returns the string representation for rdkafka configuration.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            CompressionType::Gzip => "gzip",
            CompressionType::Snappy => "snappy",
            CompressionType::Lz4 => "lz4",
            CompressionType::Zstd => "zstd",
        }
    }
}

/// Partitioner strategy for Kafka message routing.
///
/// Controls which librdkafka partitioner algorithm is used to map partition
/// keys to Kafka partitions. This is passed directly to the `partitioner`
/// configuration property. Defaults to [`ConsistentRandom`](PartitionerStrategy::ConsistentRandom).
///
/// See <https://karafka.io/docs/Librdkafka-Configuration/> for details.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionerStrategy {
    /// Random distribution.
    Random,
    /// CRC32 hash of key. Empty and NULL keys are mapped to a single partition.
    Consistent,
    /// CRC32 hash of key. Empty and NULL keys are randomly partitioned.
    ConsistentRandom,
    /// Java Producer compatible Murmur2 hash of key.
    /// NULL keys are mapped to a single partition.
    Murmur2,
    /// Java Producer compatible Murmur2 hash of key.
    /// NULL keys are randomly partitioned. This is functionally equivalent to
    /// the default partitioner in the Java Producer.
    Murmur2Random,
    /// FNV-1a hash of key. NULL keys are mapped to a single partition.
    Fnv1a,
    /// FNV-1a hash of key. NULL keys are randomly partitioned.
    Fnv1aRandom,
}

impl PartitionerStrategy {
    /// Returns the librdkafka-compatible string value for the `partitioner`
    /// configuration property.
    #[must_use]
    pub fn as_kafka_value(&self) -> &'static str {
        match self {
            PartitionerStrategy::Random => "random",
            PartitionerStrategy::Consistent => "consistent",
            PartitionerStrategy::ConsistentRandom => "consistent_random",
            PartitionerStrategy::Murmur2 => "murmur2",
            PartitionerStrategy::Murmur2Random => "murmur2_random",
            PartitionerStrategy::Fnv1a => "fnv1a",
            PartitionerStrategy::Fnv1aRandom => "fnv1a_random",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdkafka::config::RDKafkaLogLevel;

    // ---- SignalConfig ----

    #[test]
    fn test_signal_config_deserialization() {
        let json = r#"{"topic": "my-topic", "encoding": "otap_proto"}"#;
        let cfg: SignalConfig = serde_json::from_str(json).expect("valid signal config");
        assert_eq!(cfg.topic(), "my-topic");
        assert_eq!(cfg.encoding(), MessageFormat::OtapProto);
    }

    #[test]
    fn test_signal_config_encoding_defaults_to_otlp_proto() {
        let json = r#"{"topic": "my-topic"}"#;
        let cfg: SignalConfig = serde_json::from_str(json).expect("valid signal config");
        assert_eq!(cfg.topic(), "my-topic");
        assert_eq!(cfg.encoding(), MessageFormat::OtlpProto);
    }

    // ---- RequiredAcks ----

    #[test]
    fn test_required_acks_deserialization() {
        assert_eq!(
            serde_json::from_str::<RequiredAcks>(r#""none""#).unwrap(),
            RequiredAcks::None,
        );
        assert_eq!(
            serde_json::from_str::<RequiredAcks>(r#""one""#).unwrap(),
            RequiredAcks::One,
        );
        assert_eq!(
            serde_json::from_str::<RequiredAcks>(r#""all""#).unwrap(),
            RequiredAcks::All,
        );
    }

    #[test]
    fn test_required_acks_as_kafka_value() {
        assert_eq!(RequiredAcks::None.as_kafka_value(), "0");
        assert_eq!(RequiredAcks::One.as_kafka_value(), "1");
        assert_eq!(RequiredAcks::All.as_kafka_value(), "-1");
    }

    // ---- KafkaExporterConfig deserialization ----

    #[test]
    fn test_config_deserialization() {
        let json = r#"{
            "brokers": "localhost:9092",
            "client_id": "my-client",
            "traces": {"topic": "traces", "encoding": "otap_proto"},
            "metrics": {"topic": "metrics"},
            "logs": {"topic": "logs"},
            "timeout_ms": 10000,
            "required_acks": "all",
            "max_message_bytes": 2000000,
            "linger_ms": 10
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert_eq!(config.brokers(), "localhost:9092");
        assert_eq!(config.client_id(), "my-client");

        let traces = config.traces().unwrap();
        assert_eq!(traces.topic(), "traces");
        assert_eq!(traces.encoding(), MessageFormat::OtapProto);

        let metrics = config.metrics().unwrap();
        assert_eq!(metrics.topic(), "metrics");
        assert_eq!(metrics.encoding(), MessageFormat::OtlpProto);

        let logs = config.logs().unwrap();
        assert_eq!(logs.topic(), "logs");

        assert_eq!(config.timeout_ms(), 10000);
        assert_eq!(config.required_acks(), RequiredAcks::All);
        assert_eq!(config.max_message_bytes(), 2_000_000);
        assert_eq!(config.linger_ms(), 10);
    }

    #[test]
    fn test_config_defaults() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "my-client",
            "logs": {"topic": "my-logs"}
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert_eq!(config.brokers(), "kafka:9092");
        assert_eq!(config.client_id(), "my-client");
        assert!(config.traces().is_none());
        assert!(config.metrics().is_none());
        assert!(config.logs().is_some());
        assert_eq!(config.timeout_ms(), 5000);
        assert_eq!(config.required_acks(), RequiredAcks::One);
        assert_eq!(config.max_message_bytes(), 1_000_000);
        assert_eq!(config.linger_ms(), 5);
        assert_eq!(
            config.partitioning_strategy(),
            PartitionerStrategy::ConsistentRandom
        );
        assert!(config.auth().is_none());
        assert!(config.tls().is_none());
        assert!(config.producer_config().is_empty());
    }

    #[test]
    fn test_config_missing_client_id_fails() {
        let json = r#"{
            "brokers": "kafka:9092",
            "logs": {"topic": "my-logs"}
        }"#;

        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err(), "expected missing client_id to fail");
    }

    #[test]
    fn test_config_per_signal_encoding() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "traces": {"topic": "t", "encoding": "otlp_proto"},
            "metrics": {"topic": "m", "encoding": "otap_proto"},
            "logs": {"topic": "l", "encoding": "otlp_proto"}
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert_eq!(
            config.traces().unwrap().encoding(),
            MessageFormat::OtlpProto
        );
        assert_eq!(
            config.metrics().unwrap().encoding(),
            MessageFormat::OtapProto
        );
        assert_eq!(config.logs().unwrap().encoding(), MessageFormat::OtlpProto);
    }

    // ---- Validation via TryFrom ----

    #[test]
    fn test_try_from_no_signals_fails() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test");
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("at least one signal"));
    }

    #[test]
    fn test_try_from_empty_client_id_fails() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto));
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("client_id"));
    }

    #[test]
    fn test_try_from_empty_brokers_fails() {
        let builder = KafkaExporterConfigBuilder::new("", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto));
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("brokers"));
    }

    #[test]
    fn test_try_from_one_signal_succeeds() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto));
        assert!(KafkaExporterConfig::try_from(builder).is_ok());
    }

    #[test]
    fn test_try_from_all_signals_succeeds() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_traces(SignalConfig::new("t".into(), MessageFormat::OtlpProto))
            .with_metrics(SignalConfig::new("m".into(), MessageFormat::OtlpProto))
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto));
        assert!(KafkaExporterConfig::try_from(builder).is_ok());
    }

    /// Scenario (configuration and packaging: timeout limits): a config sets `timeout_ms` to `0`.
    /// Guarantees: validation rejects `0` (which maps to librdkafka's infinite
    /// `message.timeout.ms`), so an infinite delivery timeout can never block
    /// the exporter's bounded shutdown; the error names `timeout_ms`.
    #[test]
    fn test_try_from_zero_timeout_fails() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_timeout_ms(0);
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("timeout_ms"));
    }

    /// Scenario (configuration and packaging: timeout limits): a config sets `timeout_ms` just
    /// above `MAX_TIMEOUT_MS`.
    /// Guarantees: validation rejects any value greater than the 30s ceiling,
    /// so an unreasonably large delivery timeout cannot delay shutdown; the
    /// error names `timeout_ms`.
    #[test]
    fn test_try_from_excessive_timeout_fails() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_timeout_ms(MAX_TIMEOUT_MS + 1);
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("timeout_ms"));
    }

    /// Scenario (configuration and packaging: timeout limits): a config sets `timeout_ms` exactly
    /// to `MAX_TIMEOUT_MS`.
    /// Guarantees: the inclusive upper bound is accepted, so the documented
    /// maximum is usable (the bound is `<=`, not `<`).
    #[test]
    fn test_try_from_max_timeout_succeeds() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_timeout_ms(MAX_TIMEOUT_MS);
        assert!(KafkaExporterConfig::try_from(builder).is_ok());
    }

    /// Scenario (configuration and packaging: timeout limits): a config omits `timeout_ms` and
    /// takes the serde default.
    /// Guarantees: the default (5000 ms) is within the valid range and is
    /// applied, so an unconfigured exporter has a bounded, valid delivery
    /// timeout.
    #[test]
    fn test_try_from_default_timeout_succeeds() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto));
        let config = KafkaExporterConfig::try_from(builder).unwrap();
        assert_eq!(config.timeout_ms(), 5000);
    }

    /// Scenario (backpressure and resource bounds): a config sets `max_in_flight` to `0`.
    /// Guarantees: validation rejects `0` (which would stall the exporter since
    /// no delivery could ever be admitted), so a misconfigured concurrency
    /// bound is caught at construction; the error names `max_in_flight`.
    #[test]
    fn max_in_flight_zero_is_rejected() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_max_in_flight(0);
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("max_in_flight"));
    }

    /// Scenario (backpressure and resource bounds): a config sets `max_in_flight` to a small
    /// positive value.
    /// Guarantees: any positive concurrency bound validates and is surfaced
    /// verbatim by the accessor, so operators can opt into pipelined delivery.
    #[test]
    fn positive_max_in_flight_is_accepted() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_max_in_flight(8);
        let config = KafkaExporterConfig::try_from(builder).expect("positive value is valid");
        assert_eq!(config.max_in_flight(), 8);
    }

    /// Scenario (backpressure and resource bounds): a config omits `max_in_flight` and takes
    /// the serde default.
    /// Guarantees: the default is `10`, pipelining up to ten deliveries for
    /// throughput
    #[test]
    fn default_max_in_flight_is_ten() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto));
        let config = KafkaExporterConfig::try_from(builder).expect("default is valid");
        assert_eq!(config.max_in_flight(), 10);
    }

    /// Scenario (backpressure and resource bounds): a config sets `max_in_flight`
    /// to exactly `MAX_IN_FLIGHT_LIMIT`.
    /// Guarantees: the ceiling value (librdkafka's default producer queue depth)
    /// validates and is surfaced verbatim, so the boundary is inclusive.
    #[test]
    fn max_in_flight_at_limit_is_accepted() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_max_in_flight(MAX_IN_FLIGHT_LIMIT);
        let config = KafkaExporterConfig::try_from(builder).expect("ceiling value is valid");
        assert_eq!(config.max_in_flight(), MAX_IN_FLIGHT_LIMIT);
    }

    /// Scenario (backpressure and resource bounds): a config sets `max_in_flight`
    /// above `MAX_IN_FLIGHT_LIMIT`.
    /// Guarantees: validation rejects the value (it exceeds librdkafka's default
    /// producer queue depth and cannot increase pipelining), so a memory-inflating
    /// bound is caught at construction; the error names `max_in_flight`.
    #[test]
    fn max_in_flight_above_limit_is_rejected() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_max_in_flight(MAX_IN_FLIGHT_LIMIT + 1);
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("max_in_flight"));
    }

    #[test]
    fn test_deserialization_no_signals_fails() {
        let json = r#"{"brokers": "kafka:9092", "client_id": "test"}"#;
        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("at least one signal"));
    }

    #[test]
    fn test_deserialization_empty_client_id_fails() {
        let json = r#"{"brokers": "kafka:9092", "client_id": "", "logs": {"topic": "l"}}"#;
        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("client_id"));
    }

    #[test]
    fn test_deserialization_empty_brokers_fails() {
        let json = r#"{"brokers": "", "client_id": "test", "logs": {"topic": "l"}}"#;
        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("brokers"));
    }

    // ---- Topic name validation via TryFrom ----

    #[test]
    fn test_try_from_empty_topic_fails() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("".into(), MessageFormat::OtlpProto));
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("logs.topic"), "unexpected error: {err}");
        assert!(err.contains("empty"), "unexpected error: {err}");
    }

    #[test]
    fn test_try_from_dot_topic_fails() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_traces(SignalConfig::new(".".into(), MessageFormat::OtlpProto));
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("traces.topic"), "unexpected error: {err}");
        assert!(err.contains("ambiguous"), "unexpected error: {err}");
    }

    #[test]
    fn test_try_from_dotdot_topic_fails() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_metrics(SignalConfig::new("..".into(), MessageFormat::OtlpProto));
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("metrics.topic"), "unexpected error: {err}");
        assert!(err.contains("ambiguous"), "unexpected error: {err}");
    }

    #[test]
    fn test_try_from_topic_invalid_chars_fails() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test").with_logs(
            SignalConfig::new("topic with space".into(), MessageFormat::OtlpProto),
        );
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("logs.topic"), "unexpected error: {err}");
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    #[test]
    fn test_try_from_topic_too_long_fails() {
        let long_topic = "a".repeat(250);
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_traces(SignalConfig::new(long_topic, MessageFormat::OtlpProto));
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(err.contains("traces.topic"), "unexpected error: {err}");
        assert!(err.contains("maximum length"), "unexpected error: {err}");
    }

    #[test]
    fn test_try_from_valid_topics_succeeds() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_traces(SignalConfig::new(
                "my-traces".into(),
                MessageFormat::OtlpProto,
            ))
            .with_metrics(SignalConfig::new(
                "my.metrics".into(),
                MessageFormat::OtlpProto,
            ))
            .with_logs(SignalConfig::new(
                "my_logs".into(),
                MessageFormat::OtlpProto,
            ));
        assert!(KafkaExporterConfig::try_from(builder).is_ok());
    }

    #[test]
    fn test_deserialization_empty_topic_fails() {
        let json = r#"{"brokers": "kafka:9092", "client_id": "test", "logs": {"topic": ""}}"#;
        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("logs.topic"), "unexpected error: {err}");
        assert!(err.contains("empty"), "unexpected error: {err}");
    }

    #[test]
    fn test_deserialization_dot_topic_fails() {
        let json = r#"{"brokers": "kafka:9092", "client_id": "test", "traces": {"topic": "."}}"#;
        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("ambiguous"), "unexpected error: {err}");
    }

    #[test]
    fn test_deserialization_invalid_char_topic_fails() {
        let json =
            r#"{"brokers": "kafka:9092", "client_id": "test", "logs": {"topic": "bad/topic"}}"#;
        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    // ---- Compression ----

    #[test]
    fn test_compression_types() {
        assert_eq!(CompressionType::Gzip.as_str(), "gzip");
        assert_eq!(CompressionType::Snappy.as_str(), "snappy");
        assert_eq!(CompressionType::Lz4.as_str(), "lz4");
        assert_eq!(CompressionType::Zstd.as_str(), "zstd");
    }

    #[test]
    fn test_config_with_gzip_compression() {
        let json = r#"{
            "brokers": "localhost:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "compression": "gzip"
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert!(config.compression().is_some());
        assert_eq!(config.compression().unwrap().as_str(), "gzip");
    }

    #[test]
    fn test_config_with_zstd_compression() {
        let json = r#"{
            "brokers": "localhost:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "compression": "zstd"
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert!(config.compression().is_some());
        assert_eq!(config.compression().unwrap().as_str(), "zstd");
    }

    #[test]
    fn test_all_compression_types_deserialize() {
        for (name, expected) in [
            ("gzip", "gzip"),
            ("snappy", "snappy"),
            ("lz4", "lz4"),
            ("zstd", "zstd"),
        ] {
            let json = format!(
                r#"{{"brokers":"b","client_id":"t","logs":{{"topic":"l"}},"compression":"{}"}}"#,
                name
            );
            let config: KafkaExporterConfig =
                serde_json::from_str(&json).unwrap_or_else(|_| panic!("should parse {name}"));
            assert_eq!(config.compression().unwrap().as_str(), expected);
        }
    }

    /// Scenario (Kafka integration: compression): a config built with each supported
    /// `CompressionType` is turned into an rdkafka client config.
    /// Guarantees: every codec is written to librdkafka's `compression.type`
    /// with the exact wire string (`gzip`/`snappy`/`lz4`/`zstd`), so the
    /// operator-selected codec actually reaches the producer for all four
    /// variants (not just the two validated end-to-end).
    #[test]
    fn build_client_config_maps_each_compression_codec() {
        for (codec, expected) in [
            (CompressionType::Gzip, "gzip"),
            (CompressionType::Snappy, "snappy"),
            (CompressionType::Lz4, "lz4"),
            (CompressionType::Zstd, "zstd"),
        ] {
            let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
                .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
                .with_compression(codec)
                .try_into()
                .unwrap();
            let client = config.build_client_config();
            assert_eq!(
                client.get("compression.type"),
                Some(expected),
                "unexpected compression.type for {expected}"
            );
        }
    }

    // ---- TLS ----

    #[test]
    fn test_config_with_tls() {
        let json = r#"{
            "brokers": "kafka:9093",
            "client_id": "test",
            "logs": {"topic": "l"},
            "tls": {
                "ca_file": "/certs/ca.pem",
                "cert_file": "/certs/client.pem",
                "key_file": "/certs/client-key.pem",
                "insecure": true
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert!(config.tls().is_some());
        let tls = config.tls().unwrap();
        assert_eq!(tls.ca_file(), Some("/certs/ca.pem"));
        assert_eq!(tls.cert_file(), Some("/certs/client.pem"));
        assert_eq!(tls.key_file(), Some("/certs/client-key.pem"));
        assert!(tls.insecure());
    }

    #[test]
    fn test_config_with_tls_insecure_defaults_false() {
        let json = r#"{
            "brokers": "kafka:9093",
            "client_id": "test",
            "logs": {"topic": "l"},
            "tls": {
                "ca_file": "/certs/ca.pem",
                "cert_file": "/certs/client.pem",
                "key_file": "/certs/client-key.pem"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let tls = config.tls().unwrap();
        assert!(!tls.insecure());
    }

    // ---- Unknown fields ----

    #[test]
    fn test_config_rejects_unknown_fields() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "unknown_field": "should_fail"
        }"#;

        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_signal_config_rejects_unknown_fields() {
        // A typo inside a signal block (e.g. `topic_from_transpot_header`)
        // must be rejected rather than silently ignored, which would
        // otherwise disable dynamic topic routing without warning.
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {
                "topic": "l",
                "topic_from_transpot_header": "x_target_topic"
            }
        }"#;

        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(
            result.is_err(),
            "expected unknown field in signal config to be rejected"
        );
    }

    // ---- producer_config ----

    #[test]
    fn test_producer_config_deserialization() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "producer_config": {
                "queue.buffering.max.messages": "100000",
                "batch.num.messages": "10000"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert_eq!(config.producer_config().len(), 2);
        assert_eq!(
            config.producer_config().get("queue.buffering.max.messages"),
            Some(&"100000".to_string())
        );
        assert_eq!(
            config.producer_config().get("batch.num.messages"),
            Some(&"10000".to_string())
        );
    }

    // ---- Auth ----

    #[cfg(feature = "aws")]
    #[test]
    fn test_auth_sasl_deserialization() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "auth": {
                "sasl": {
                    "mechanism": "AWS_MSK_IAM_OAUTHBEARER",
                    "aws_msk": {
                        "region": "us-east-1"
                    }
                }
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert!(config.auth().is_some());
    }

    #[test]
    fn test_auth_sasl_plain_deserialization() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "auth": {
                "sasl": {
                    "mechanism": "PLAIN",
                    "username": "myuser",
                    "password": "mypass"
                }
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert!(config.auth().is_some());
    }

    #[test]
    fn test_auth_sasl_scram_sha512_deserialization() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "auth": {
                "sasl": {
                    "mechanism": "SCRAM-SHA-512",
                    "username": "myuser",
                    "password": "mypass"
                }
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert!(config.auth().is_some());
    }

    #[test]
    fn test_auth_sasl_plain_missing_password_fails_validation() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "auth": {
                "sasl": {
                    "mechanism": "PLAIN",
                    "username": "myuser"
                }
            }
        }"#;

        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("auth"), "unexpected error: {err}");
    }

    #[test]
    fn test_auth_sasl_unknown_mechanism_fails_validation() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "auth": {
                "sasl": {
                    "mechanism": "KERBEROS"
                }
            }
        }"#;

        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown variant"), "unexpected error: {err}");
        assert!(err.contains("KERBEROS"), "unexpected error: {err}");
    }

    #[test]
    fn test_tls_ca_only_config() {
        let json = r#"{
            "brokers": "kafka:9093",
            "client_id": "test",
            "logs": {"topic": "l"},
            "tls": {
                "ca_file": "/certs/ca.pem"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let tls = config.tls().unwrap();
        assert_eq!(tls.ca_file(), Some("/certs/ca.pem"));
        assert!(tls.cert_file().is_none());
        assert!(tls.key_file().is_none());
        assert!(!tls.insecure());
    }

    #[test]
    fn test_tls_empty_block_config() {
        let json = r#"{
            "brokers": "kafka:9093",
            "client_id": "test",
            "logs": {"topic": "l"},
            "tls": {}
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let tls = config.tls().unwrap();
        assert!(tls.ca_file().is_none());
        assert!(tls.cert_file().is_none());
        assert!(tls.key_file().is_none());
        assert!(!tls.insecure());
    }

    #[test]
    fn test_tls_key_password_config() {
        let json = r#"{
            "brokers": "kafka:9093",
            "client_id": "test",
            "logs": {"topic": "l"},
            "tls": {
                "ca_file": "/certs/ca.pem",
                "cert_file": "/certs/client.pem",
                "key_file": "/certs/client-key.pem",
                "key_password": "secret"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let tls = config.tls().unwrap();
        assert_eq!(tls.key_password(), Some("secret"));
    }

    // ---- All new fields together ----

    #[cfg(feature = "aws")]
    #[test]
    fn test_config_with_all_new_fields() {
        let json = r#"{
            "brokers": "kafka1:9092,kafka2:9092",
            "client_id": "my-gateway",
            "traces": {"topic": "otlp_spans", "encoding": "otlp_proto"},
            "metrics": {"topic": "otlp_metrics", "encoding": "otlp_proto"},
            "logs": {"topic": "otlp_logs", "encoding": "otap_proto"},
            "timeout_ms": 5000,
            "compression": "zstd",
            "required_acks": "all",
            "max_message_bytes": 2000000,
            "linger_ms": 10,
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
            },
            "producer_config": {
                "queue.buffering.max.messages": "100000"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert_eq!(config.brokers(), "kafka1:9092,kafka2:9092");
        assert_eq!(config.client_id(), "my-gateway");

        let traces = config.traces().unwrap();
        assert_eq!(traces.topic(), "otlp_spans");
        assert_eq!(traces.encoding(), MessageFormat::OtlpProto);

        let logs = config.logs().unwrap();
        assert_eq!(logs.topic(), "otlp_logs");
        assert_eq!(logs.encoding(), MessageFormat::OtapProto);

        assert_eq!(config.required_acks(), RequiredAcks::All);
        assert_eq!(config.max_message_bytes(), 2_000_000);
        assert_eq!(config.linger_ms(), 10);
        assert!(config.auth().is_some());
        assert!(config.tls().is_some());
        assert_eq!(config.producer_config().len(), 1);
    }

    // ---- overridden_producer_config_keys ----

    /// Scenario (configuration and packaging: escape-hatch precedence): `producer_config` mixes
    /// keys managed by first-class fields with an unmanaged custom key.
    /// Guarantees: `overridden_producer_config_keys` reports exactly the managed
    /// keys (the operator-warning surface) and ignores unmanaged keys, so the
    /// exporter warns only about settings a first-class field will overwrite.
    #[test]
    fn overridden_keys_detects_conflicts() {
        let overrides = HashMap::from([
            ("bootstrap.servers".into(), "custom:1234".into()),
            ("linger.ms".into(), "99".into()),
            ("custom.setting".into(), "value".into()),
        ]);

        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_producer_config(overrides)
            .try_into()
            .unwrap();

        let mut conflicts = config.overridden_producer_config_keys();
        conflicts.sort();
        assert_eq!(conflicts, vec!["bootstrap.servers", "linger.ms"]);
    }

    /// Scenario (configuration and packaging: escape-hatch precedence): `producer_config` contains
    /// only keys with no first-class-field counterpart.
    /// Guarantees: `overridden_producer_config_keys` is empty, so purely
    /// advanced tuning knobs pass through the escape hatch without triggering a
    /// spurious override warning.
    #[test]
    fn overridden_keys_empty_when_no_conflicts() {
        let custom = HashMap::from([("custom.setting".into(), "value".into())]);

        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_producer_config(custom)
            .try_into()
            .unwrap();

        assert!(config.overridden_producer_config_keys().is_empty());
    }

    // ---- allow.auto.create.topics ----

    /// Scenario (configuration and packaging: escape-hatch precedence): build the librdkafka
    /// client config from a default exporter config.
    /// Guarantees: `allow.auto.create.topics` is explicitly set to `true` by default
    #[test]
    fn build_client_config_defaults_auto_create_to_true() {
        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .try_into()
            .unwrap();
        let client = config.build_client_config();
        assert_eq!(client.get("allow.auto.create.topics"), Some("true"));
    }

    /// Scenario (configuration and packaging: escape-hatch precedence): an operator explicitly
    /// opts out of broker auto-creation (default-deny).
    /// Guarantees: `with_allow_auto_create_topics(false)` sets
    /// `allow.auto.create.topics` to `false` in the built client config, so a
    /// client-controlled routing header cannot rely on the broker auto-creating
    /// arbitrary topics.
    #[test]
    fn build_client_config_disables_auto_create_when_opted_out() {
        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_allow_auto_create_topics(false)
            .try_into()
            .unwrap();
        let client = config.build_client_config();
        assert_eq!(client.get("allow.auto.create.topics"), Some("false"));
    }

    /// Scenario (configuration and packaging: escape-hatch precedence): a `producer_config`
    /// passthrough tries to set `allow.auto.create.topics` while the first-class
    /// field is set to the opposite value.
    /// Guarantees: the key is a managed key -- the first-class field wins in the
    /// built config, and the conflict is reported by
    /// `overridden_producer_config_keys` so the operator is warned.
    #[test]
    fn auto_create_key_is_managed_and_first_class_field_wins() {
        // Field explicitly false; passthrough tries to force true.
        let overrides =
            HashMap::from([("allow.auto.create.topics".to_string(), "true".to_string())]);
        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_allow_auto_create_topics(false)
            .with_producer_config(overrides)
            .try_into()
            .unwrap();

        // The managed first-class value wins over the passthrough.
        let client = config.build_client_config();
        assert_eq!(client.get("allow.auto.create.topics"), Some("false"));

        // And the conflict is surfaced for an operator warning.
        assert!(
            config
                .overridden_producer_config_keys()
                .contains(&"allow.auto.create.topics"),
            "auto-create should be a managed key so the override is reported"
        );
    }

    // ---- Configuration & packaging: timeout mapping and escape-hatch precedence ----

    /// Scenario (configuration and packaging: timeout limits): a validated `timeout_ms` is turned
    /// into an rdkafka client config.
    /// Guarantees: `timeout_ms` maps to librdkafka's `message.timeout.ms` with
    /// the exact configured value, so the per-delivery deadline the exporter
    /// relies on for bounded shutdown is actually applied to the producer.
    #[test]
    fn build_client_config_maps_timeout_to_message_timeout_ms() {
        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_timeout_ms(1234)
            .try_into()
            .unwrap();
        let client = config.build_client_config();
        assert_eq!(client.get("message.timeout.ms"), Some("1234"));
    }

    /// Scenario (configuration and packaging: escape-hatch precedence): `producer_config` supplies
    /// values for keys that also have first-class fields, alongside the
    /// first-class fields themselves.
    /// Guarantees: `producer_config` is applied first and every managed
    /// first-class field overrides the conflicting passthrough in the built
    /// client config, so an operator cannot accidentally (or maliciously)
    /// subvert a managed setting through the escape hatch.
    #[test]
    fn first_class_fields_override_producer_config_passthrough() {
        let overrides = HashMap::from([
            ("bootstrap.servers".to_string(), "evil:9999".to_string()),
            ("client.id".to_string(), "evil-client".to_string()),
            ("message.timeout.ms".to_string(), "0".to_string()),
            ("compression.type".to_string(), "gzip".to_string()),
            ("request.required.acks".to_string(), "0".to_string()),
            ("linger.ms".to_string(), "99999".to_string()),
            ("partitioner".to_string(), "murmur2".to_string()),
        ]);
        let config: KafkaExporterConfig =
            KafkaExporterConfigBuilder::new("real-broker:9092", "real-client")
                .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
                .with_timeout_ms(2000)
                .with_compression(CompressionType::Zstd)
                .with_required_acks(RequiredAcks::All)
                .with_linger_ms(5)
                .with_partitioning_strategy(PartitionerStrategy::ConsistentRandom)
                .with_producer_config(overrides)
                .try_into()
                .unwrap();

        let client = config.build_client_config();
        // Each first-class field wins over the conflicting passthrough value.
        assert_eq!(client.get("bootstrap.servers"), Some("real-broker:9092"));
        assert_eq!(client.get("client.id"), Some("real-client"));
        assert_eq!(client.get("message.timeout.ms"), Some("2000"));
        assert_eq!(client.get("compression.type"), Some("zstd"));
        assert_eq!(client.get("request.required.acks"), Some("-1"));
        assert_eq!(client.get("linger.ms"), Some("5"));
        assert_eq!(client.get("partitioner"), Some("consistent_random"));

        // The conflicts are also reported so the exporter can warn the operator.
        let mut conflicts = config.overridden_producer_config_keys();
        conflicts.sort_unstable();
        assert_eq!(
            conflicts,
            vec![
                "bootstrap.servers",
                "client.id",
                "compression.type",
                "linger.ms",
                "message.timeout.ms",
                "partitioner",
                "request.required.acks",
            ]
        );
    }

    /// Scenario (configuration and packaging: escape-hatch precedence): `producer_config` sets a
    /// tuning knob that is not managed by any first-class field.
    /// Guarantees: an unmanaged passthrough key survives unchanged into the
    /// built client config (the escape hatch remains usable for advanced
    /// librdkafka knobs) and is not flagged as an overridden key.
    #[test]
    fn unmanaged_producer_config_passes_through() {
        let overrides = HashMap::from([(
            "queue.buffering.max.messages".to_string(),
            "100000".to_string(),
        )]);
        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_producer_config(overrides)
            .try_into()
            .unwrap();

        let client = config.build_client_config();
        assert_eq!(client.get("queue.buffering.max.messages"), Some("100000"));
        assert!(config.overridden_producer_config_keys().is_empty());
    }

    /// Scenario (configuration and packaging: escape-hatch precedence): both `producer_config` and
    /// the first-class `debug` field try to set librdkafka's `debug` contexts.
    /// Guarantees: the first-class `debug` field is applied last and overrides
    /// the `producer_config` passthrough, so debug logging is driven by the
    /// managed field rather than the escape hatch.
    #[test]
    fn debug_field_overrides_producer_config_debug() {
        let overrides = HashMap::from([("debug".to_string(), "broker".to_string())]);
        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_producer_config(overrides)
            .with_debug(vec![DebugContext::Security])
            .try_into()
            .unwrap();

        let client = config.build_client_config();
        assert_eq!(
            client.get("debug"),
            Some("security"),
            "the first-class debug field is applied last and wins"
        );
    }

    /// Scenario (configuration and packaging: escape-hatch precedence): the first-class
    /// `log_level` field is configured.
    /// Guarantees: `build_client_config` applies the configured log level to the
    /// rdkafka client (a first-class-only setting with no `producer_config`
    /// string equivalent), so operator-selected verbosity takes effect.
    #[test]
    fn log_level_first_class_is_applied() {
        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_log_level(LogLevel::Debug)
            .try_into()
            .unwrap();
        let client = config.build_client_config();
        // RDKafkaLogLevel does not implement PartialEq, so compare via Debug.
        assert_eq!(
            format!("{:?}", client.log_level),
            format!("{:?}", RDKafkaLogLevel::Debug)
        );
    }

    // ---- Dynamic topic routing fields (per-signal) ----

    #[test]
    fn test_signal_config_with_topic_from_transport_header() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {
                "topic": "otlp_logs",
                "topic_from_transport_header": "x_target_topic"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let logs = config.logs().expect("logs should be configured");
        assert_eq!(logs.topic_from_transport_header(), Some("x_target_topic"));
    }

    #[test]
    fn test_signal_config_topic_from_transport_header_defaults_to_none() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "otlp_logs"}
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let logs = config.logs().expect("logs should be configured");
        assert!(logs.topic_from_transport_header().is_none());
    }

    #[test]
    fn test_signal_config_builder_with_topic_from_transport_header() {
        let signal = SignalConfig::new("otlp_logs".into(), MessageFormat::OtlpProto)
            .with_topic_from_transport_header("x_target_topic");

        assert_eq!(signal.topic_from_transport_header(), Some("x_target_topic"));
        assert_eq!(signal.topic(), "otlp_logs");
    }

    // ---- Security: dynamic-routing allowlist config ----

    /// Scenario: build a config with valid exact allowlist and regex allowlist
    /// entries.
    /// Guarantees: valid exact/regex entries are accepted and readable, and the
    /// config as a whole validates (regex patterns compile).
    #[test]
    fn allowlist_config_is_accepted() {
        let signal = SignalConfig::new("static".into(), MessageFormat::OtlpProto)
            .with_topic_from_transport_header("x-target-topic")
            .with_allowed_topics(["approved"])
            .with_allowed_topics_regex(["tenant_.*"]);

        assert_eq!(signal.allowed_topics(), &["approved".to_string()]);
        assert_eq!(signal.allowed_topics_regex(), &["tenant_.*".to_string()]);

        // The config as a whole validates (regex compiles).
        let _config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(signal)
            .try_into()
            .expect("config with allowlist should be valid");
    }

    /// Scenario: an exact allowlist entry is not a syntactically valid Kafka
    /// topic.
    /// Guarantees: config validation rejects it (with a field-scoped message),
    /// so an operator cannot configure an unusable exact allowlist entry.
    #[test]
    fn invalid_exact_allowlist_entry_is_rejected() {
        let result: Result<KafkaExporterConfig, _> = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(
                SignalConfig::new("static".into(), MessageFormat::OtlpProto)
                    .with_allowed_topics(["bad topic/name"]),
            )
            .try_into();
        let err = result.expect_err("invalid allowlist entry should be rejected");
        assert!(
            err.contains("allowed_topics[0]"),
            "error should point at the offending entry, got: {err}"
        );
    }

    /// Scenario: a regex allowlist entry is not a valid regular expression.
    /// Guarantees: config validation rejects it at config time (fail fast,
    /// including through the factory validate path), naming the field so an
    /// operator can fix it.
    #[test]
    fn invalid_regex_allowlist_entry_is_rejected() {
        let result: Result<KafkaExporterConfig, _> = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(
                SignalConfig::new("static".into(), MessageFormat::OtlpProto)
                    .with_allowed_topics_regex(["tenant_(", "["]),
            )
            .try_into();
        let err = result.expect_err("invalid regex entry should be rejected");
        assert!(
            err.contains("allowed_topics_regex[0]"),
            "error should point at the offending pattern, got: {err}"
        );
    }

    /// Scenario: a regex allowlist entry is crafted to break out of the
    /// whole-topic anchoring wrapper (`tenant_.)\z|(?:evil.`), which compiles
    /// only because its parentheses balance against the wrapper.
    /// Guarantees: config validation rejects it at config time (fail fast,
    /// including through the factory validate path), naming the offending entry,
    /// so a pattern that would under-anchor an alternation and permit unintended
    /// header-routed topics can never be accepted into a validated config.
    #[test]
    fn anchor_breakout_regex_allowlist_entry_is_rejected() {
        let result: Result<KafkaExporterConfig, _> = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(
                SignalConfig::new("static".into(), MessageFormat::OtlpProto)
                    .with_allowed_topics_regex([r"tenant_.)\z|(?:evil."]),
            )
            .try_into();
        let err = result.expect_err("anchor-breakout regex entry should be rejected");
        assert!(
            err.contains("allowed_topics_regex[0]"),
            "error should point at the offending pattern, got: {err}"
        );
    }

    #[test]
    fn test_per_signal_topic_from_transport_header() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "traces": {
                "topic": "otlp_spans",
                "topic_from_transport_header": "x_traces_topic"
            },
            "metrics": {
                "topic": "otlp_metrics"
            },
            "logs": {
                "topic": "otlp_logs",
                "topic_from_transport_header": "x_logs_topic"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let traces = config.traces().expect("traces configured");
        let metrics = config.metrics().expect("metrics configured");
        let logs = config.logs().expect("logs configured");

        assert_eq!(traces.topic_from_transport_header(), Some("x_traces_topic"));
        assert!(metrics.topic_from_transport_header().is_none());
        assert_eq!(logs.topic_from_transport_header(), Some("x_logs_topic"));
    }

    #[test]
    fn test_topic_from_transport_header_is_lowercased_on_validation() {
        // A natural mixed-case header name must be normalized (lowercased) so it
        // matches captured transport header names, which are lowercased on
        // ingress. Dashes are preserved (capture uses `to_ascii_lowercase`).
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "traces": {
                "topic": "otlp_spans",
                "topic_from_transport_header": "X-Traces-Topic"
            },
            "logs": {
                "topic": "otlp_logs",
                "topic_from_transport_header": "X-Target-Topic"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert_eq!(
            config.traces().unwrap().topic_from_transport_header(),
            Some("x-traces-topic")
        );
        assert_eq!(
            config.logs().unwrap().topic_from_transport_header(),
            Some("x-target-topic")
        );
    }

    #[test]
    fn test_config_empty_client_id_fails_validation() {
        let builder = KafkaExporterConfigBuilder::new("kafka:9092", "")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto));
        let result = KafkaExporterConfig::try_from(builder);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("client_id"),
            "error should mention client_id"
        );
    }

    #[test]
    fn test_config_empty_brokers_fails_validation() {
        let builder = KafkaExporterConfigBuilder::new("", "client_id")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto));
        let result = KafkaExporterConfig::try_from(builder);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("brokers"),
            "error should mention brokers"
        );
    }

    #[test]
    fn test_config_empty_message_format_header_fails_validation() {
        let builder = KafkaExporterConfigBuilder::new("b", "c")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_message_format_header("");
        let err = KafkaExporterConfig::try_from(builder).unwrap_err();
        assert!(
            err.contains("message_format_header"),
            "unexpected error: {err}"
        );
    }

    // ---- PartitionerStrategy ----

    #[test]
    fn test_partitioner_strategy_deserialization() {
        for (name, expected) in [
            ("random", PartitionerStrategy::Random),
            ("consistent", PartitionerStrategy::Consistent),
            ("consistent_random", PartitionerStrategy::ConsistentRandom),
            ("murmur2", PartitionerStrategy::Murmur2),
            ("murmur2_random", PartitionerStrategy::Murmur2Random),
            ("fnv1a", PartitionerStrategy::Fnv1a),
            ("fnv1a_random", PartitionerStrategy::Fnv1aRandom),
        ] {
            let json = format!(
                r#"{{"brokers":"b","client_id":"t","logs":{{"topic":"l"}},"partitioning_strategy":"{}"}}"#,
                name
            );
            let config: KafkaExporterConfig =
                serde_json::from_str(&json).unwrap_or_else(|_| panic!("should parse {name}"));
            assert_eq!(config.partitioning_strategy(), expected);
        }
    }

    #[test]
    fn test_partitioner_strategy_as_kafka_value() {
        assert_eq!(PartitionerStrategy::Random.as_kafka_value(), "random");
        assert_eq!(
            PartitionerStrategy::Consistent.as_kafka_value(),
            "consistent"
        );
        assert_eq!(
            PartitionerStrategy::ConsistentRandom.as_kafka_value(),
            "consistent_random"
        );
        assert_eq!(PartitionerStrategy::Murmur2.as_kafka_value(), "murmur2");
        assert_eq!(
            PartitionerStrategy::Murmur2Random.as_kafka_value(),
            "murmur2_random"
        );
        assert_eq!(PartitionerStrategy::Fnv1a.as_kafka_value(), "fnv1a");
        assert_eq!(
            PartitionerStrategy::Fnv1aRandom.as_kafka_value(),
            "fnv1a_random"
        );
    }

    #[test]
    fn test_partitioning_strategy_defaults_to_consistent_random() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"}
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert_eq!(
            config.partitioning_strategy(),
            PartitionerStrategy::ConsistentRandom
        );
    }

    #[test]
    fn test_partitioning_strategy_builder() {
        let config: KafkaExporterConfig = KafkaExporterConfigBuilder::new("kafka:9092", "test")
            .with_logs(SignalConfig::new("l".into(), MessageFormat::OtlpProto))
            .with_partitioning_strategy(PartitionerStrategy::Murmur2Random)
            .try_into()
            .expect("valid config");

        assert_eq!(
            config.partitioning_strategy(),
            PartitionerStrategy::Murmur2Random
        );
    }

    #[test]
    fn test_invalid_partitioning_strategy_rejected() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "l"},
            "partitioning_strategy": "invalid_strategy"
        }"#;

        let result = serde_json::from_str::<KafkaExporterConfig>(json);
        assert!(result.is_err());
    }

    // ---- partition_by_transport_headers (per-signal) ----

    #[test]
    fn test_signal_config_partition_by_transport_headers_deserialization() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "traces": {
                "topic": "otlp_spans",
                "partition_by_transport_headers": true
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let traces = config.traces().expect("traces configured");
        assert!(traces.partition_by_transport_headers());
    }

    #[test]
    fn test_signal_config_partition_by_transport_headers_defaults_to_false() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "logs": {"topic": "otlp_logs"}
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let logs = config.logs().expect("logs configured");
        assert!(!logs.partition_by_transport_headers());
    }

    #[test]
    fn test_signal_config_builder_with_partition_by_transport_headers() {
        let signal = SignalConfig::new("otlp_spans".into(), MessageFormat::OtlpProto)
            .with_partition_by_transport_headers(true);

        assert!(signal.partition_by_transport_headers());
        assert_eq!(signal.topic(), "otlp_spans");
    }

    #[test]
    fn test_per_signal_partition_by_transport_headers() {
        let json = r#"{
            "brokers": "kafka:9092",
            "client_id": "test",
            "traces": {
                "topic": "otlp_spans",
                "partition_by_transport_headers": true
            },
            "metrics": {
                "topic": "otlp_metrics",
                "partition_by_transport_headers": true
            },
            "logs": {
                "topic": "otlp_logs"
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        let traces = config.traces().expect("traces configured");
        let metrics = config.metrics().expect("metrics configured");
        let logs = config.logs().expect("logs configured");

        assert!(traces.partition_by_transport_headers());
        assert!(metrics.partition_by_transport_headers());
        assert!(!logs.partition_by_transport_headers());
    }

    #[test]
    fn test_full_config_with_partitioning() {
        let json = r#"{
            "brokers": "kafka1:9092,kafka2:9092",
            "client_id": "my-gateway",
            "partitioning_strategy": "murmur2_random",
            "traces": {
                "topic": "otlp_spans",
                "partition_by_transport_headers": true
            },
            "logs": {
                "topic": "otlp_logs",
                "partition_by_transport_headers": true
            },
            "metrics": {
                "topic": "otlp_metrics",
                "partition_by_transport_headers": true
            }
        }"#;

        let config: KafkaExporterConfig = serde_json::from_str(json).expect("valid config");
        assert_eq!(
            config.partitioning_strategy(),
            PartitionerStrategy::Murmur2Random
        );
        assert!(config.traces().unwrap().partition_by_transport_headers());
        assert!(config.logs().unwrap().partition_by_transport_headers());
        assert!(config.metrics().unwrap().partition_by_transport_headers());
    }
}
