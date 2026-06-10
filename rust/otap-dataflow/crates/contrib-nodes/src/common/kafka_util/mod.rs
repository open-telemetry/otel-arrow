use rdkafka::config::RDKafkaLogLevel;
use serde::Deserialize;

pub mod auth;
pub mod aws;
pub mod security;

/// TLS configuration for Kafka broker connections.
///
/// All file-path fields are optional so that callers can configure only the
/// settings they need:
///
/// - **Server-only TLS** (no mTLS): provide just `ca_file`.
/// - **mTLS**: provide `ca_file`, `cert_file`, and `key_file`.
/// - **System trust store**: use an empty `tls: {}` block to enable TLS
///   without specifying any certificate paths.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TlsConfig {
    /// Path to the CA certificate used to verify the broker's TLS certificate.
    /// Corresponds to `ssl.ca.location`.
    #[serde(default)]
    ca_file: Option<String>,

    /// Path to the client TLS certificate (PEM).
    /// Corresponds to `ssl.certificate.location`.
    #[serde(default)]
    cert_file: Option<String>,

    /// Path to the client TLS private key (PEM).
    /// Corresponds to `ssl.key.location`.
    #[serde(default)]
    key_file: Option<String>,

    /// Password for an encrypted client TLS private key.
    /// Corresponds to `ssl.key.password`.
    #[serde(default)]
    key_password: Option<String>,

    /// If true, disables TLS certificate verification.
    #[serde(default)]
    insecure: bool,
}

impl Default for TlsConfig {
    /// Returns a TLS configuration that uses the operating system's trusted
    /// CA certificates. Equivalent to the `tls: {}` YAML shape.
    fn default() -> Self {
        Self {
            ca_file: None,
            cert_file: None,
            key_file: None,
            key_password: None,
            insecure: false,
        }
    }
}

impl TlsConfig {
    /// Create a TLS configuration for mutual TLS (mTLS) with all certificate
    /// paths set.
    #[must_use]
    pub fn new(
        ca_file: String,
        cert_file: String,
        key_file: String,
        key_password: Option<String>,
        insecure: bool,
    ) -> Self {
        Self {
            ca_file: Some(ca_file),
            cert_file: Some(cert_file),
            key_file: Some(key_file),
            key_password,
            insecure,
        }
    }

    /// Create a TLS configuration with only a CA certificate for server
    /// verification (no client authentication).
    #[must_use]
    pub fn ca_only(ca_file: String) -> Self {
        Self {
            ca_file: Some(ca_file),
            ..Self::default()
        }
    }

    /// Set the CA certificate path.
    #[must_use]
    pub fn with_ca_file(mut self, ca_file: impl Into<String>) -> Self {
        self.ca_file = Some(ca_file.into());
        self
    }

    /// Set the client TLS certificate path.
    #[must_use]
    pub fn with_cert_file(mut self, cert_file: impl Into<String>) -> Self {
        self.cert_file = Some(cert_file.into());
        self
    }

    /// Set the client TLS private key path.
    #[must_use]
    pub fn with_key_file(mut self, key_file: impl Into<String>) -> Self {
        self.key_file = Some(key_file.into());
        self
    }

    /// Set the password for the client TLS private key.
    #[must_use]
    pub fn with_key_password(mut self, key_password: impl Into<String>) -> Self {
        self.key_password = Some(key_password.into());
        self
    }

    /// Set the `insecure` flag, disabling TLS certificate verification.
    #[must_use]
    pub fn with_insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    /// Path to the CA certificate (PEM), if configured.
    #[must_use]
    pub fn ca_file(&self) -> Option<&str> {
        self.ca_file.as_deref()
    }

    /// Path to the client TLS certificate (PEM), if configured.
    #[must_use]
    pub fn cert_file(&self) -> Option<&str> {
        self.cert_file.as_deref()
    }

    /// Path to the client TLS private key (PEM), if configured.
    #[must_use]
    pub fn key_file(&self) -> Option<&str> {
        self.key_file.as_deref()
    }

    /// Password for an encrypted client TLS private key, if configured.
    #[must_use]
    pub fn key_password(&self) -> Option<&str> {
        self.key_password.as_deref()
    }

    /// Whether TLS certificate verification is disabled.
    #[must_use]
    pub fn insecure(&self) -> bool {
        self.insecure
    }

    /// Validate the TLS configuration.
    ///
    /// # Errors
    ///
    /// Returns a human-readable description if:
    /// - `cert_file` is set without `key_file`, or vice versa.
    /// - `key_password` is set without `key_file`.
    /// - Any provided path string is empty.
    pub fn validate(&self) -> Result<(), String> {
        // Reject empty strings — likely a config typo.
        if self.ca_file.as_deref() == Some("") {
            return Err("'ca_file' must not be empty".to_string());
        }
        if self.cert_file.as_deref() == Some("") {
            return Err("'cert_file' must not be empty".to_string());
        }
        if self.key_file.as_deref() == Some("") {
            return Err("'key_file' must not be empty".to_string());
        }
        if self.key_password.as_deref() == Some("") {
            return Err("'key_password' must not be empty".to_string());
        }

        // cert_file and key_file must be provided together for mTLS.
        match (&self.cert_file, &self.key_file) {
            (Some(_), None) => {
                return Err("'cert_file' requires 'key_file' to be set".to_string());
            }
            (None, Some(_)) => {
                return Err("'key_file' requires 'cert_file' to be set".to_string());
            }
            _ => {}
        }

        // key_password without key_file is nonsensical.
        if self.key_password.is_some() && self.key_file.is_none() {
            return Err("'key_password' requires 'key_file' to be set".to_string());
        }

        Ok(())
    }
}

/// Message encoding format stored in Kafka message headers.
#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageFormat {
    /// OTLP protobuf encoding.
    OtlpProto,
    /// OTLP JSON encoding.
    // Todo: implement
    //OtlpJson,
    /// OTAP (Arrow) protobuf encoding.
    OtapProto,
    // others eventually
}

impl Default for MessageFormat {
    /// The default message format is OTLP protobuf.
    fn default() -> Self {
        Self::OtlpProto
    }
}

/// Librdkafka log level (syslog severity levels).
///
/// Controls the verbosity of librdkafka's internal logging. When not
/// configured, rdkafka infers the level from the `log` crate's global
/// configuration.
///
/// Levels are listed from highest to lowest severity.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// System is unusable (syslog level 0).
    Emerg,
    /// Action must be taken immediately (syslog level 1).
    Alert,
    /// Critical conditions (syslog level 2).
    Critical,
    /// Error conditions (syslog level 3).
    Error,
    /// Warning conditions (syslog level 4).
    Warning,
    /// Normal but significant conditions (syslog level 5).
    Notice,
    /// Informational messages (syslog level 6).
    Info,
    /// Debug-level messages (syslog level 7).
    Debug,
}

impl LogLevel {
    /// Converts to the corresponding [`RDKafkaLogLevel`] for use with
    /// rdkafka's `ClientConfig::set_log_level`.
    #[must_use]
    pub fn to_rdkafka(self) -> RDKafkaLogLevel {
        match self {
            Self::Emerg => RDKafkaLogLevel::Emerg,
            Self::Alert => RDKafkaLogLevel::Alert,
            Self::Critical => RDKafkaLogLevel::Critical,
            Self::Error => RDKafkaLogLevel::Error,
            Self::Warning => RDKafkaLogLevel::Warning,
            Self::Notice => RDKafkaLogLevel::Notice,
            Self::Info => RDKafkaLogLevel::Info,
            Self::Debug => RDKafkaLogLevel::Debug,
        }
    }
}

/// Librdkafka debug context.
///
/// Each variant enables debug logging for a specific subsystem within
/// librdkafka. Multiple contexts can be combined in a list.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugContext {
    /// General client operations.
    Generic,
    /// Broker communication and state.
    Broker,
    /// Topic-level operations.
    Topic,
    /// Metadata requests and responses.
    Metadata,
    /// Feature negotiation with brokers.
    Feature,
    /// Internal queue operations.
    Queue,
    /// Message production and consumption.
    Msg,
    /// Kafka protocol wire traffic.
    Protocol,
    /// Consumer group operations.
    Cgrp,
    /// Security and authentication.
    Security,
    /// Fetch request processing.
    Fetch,
    /// Interceptor plugin callbacks.
    Interceptor,
    /// Plugin loading.
    Plugin,
    /// High-level consumer operations.
    Consumer,
    /// Admin client operations.
    Admin,
    /// Exactly-once semantics / idempotent producer.
    Eos,
    /// Mock cluster (testing).
    Mock,
    /// Partition assignor operations.
    Assignor,
    /// Configuration diagnostics.
    Conf,
    /// Client telemetry.
    Telemetry,
    /// Enable all debug contexts.
    All,
}

impl DebugContext {
    /// Returns the librdkafka string representation of this debug context.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::Broker => "broker",
            Self::Topic => "topic",
            Self::Metadata => "metadata",
            Self::Feature => "feature",
            Self::Queue => "queue",
            Self::Msg => "msg",
            Self::Protocol => "protocol",
            Self::Cgrp => "cgrp",
            Self::Security => "security",
            Self::Fetch => "fetch",
            Self::Interceptor => "interceptor",
            Self::Plugin => "plugin",
            Self::Consumer => "consumer",
            Self::Admin => "admin",
            Self::Eos => "eos",
            Self::Mock => "mock",
            Self::Assignor => "assignor",
            Self::Conf => "conf",
            Self::Telemetry => "telemetry",
            Self::All => "all",
        }
    }
}

/// Joins a slice of [`DebugContext`] values into the comma-separated format
/// expected by librdkafka's `debug` configuration property.
#[must_use]
pub fn debug_list_to_string(contexts: &[DebugContext]) -> String {
    contexts
        .iter()
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

/// Default header key used to indicate the message encoding format.
pub const MSG_FORMAT_HEADER: &str = "MessageFormat";

/// Returns the default message format header key as an owned `String`.
///
/// Intended for use as a serde default function.
#[must_use]
pub fn default_message_format_header() -> String {
    MSG_FORMAT_HEADER.to_string()
}

/// header value for OTLP format in bytes
pub const MSG_FORMAT_OTLP: &[u8] = b"otlp";

/// header value for OTAP format in bytes
pub const MSG_FORMAT_OTAP: &[u8] = b"otap";

/// Maximum length of a Kafka topic name.
///
/// Kafka enforces a hard limit of 249 characters for topic names.
pub const MAX_KAFKA_TOPIC_LEN: usize = 249;

/// Validates that a string is a legal Kafka topic name.
///
/// Equivalent to the regex `^[a-zA-Z0-9._-]{1,249}$` with an additional
/// rejection of `"."` and `".."` as ambiguous path components.
///
/// These rules mirror the Kafka broker's own validation in
/// `org.apache.kafka.common.internals.Topic`.
///
/// Kafka topic names must:
/// - be non-empty
/// - not be `"."` or `".."` (ambiguous path components rejected by the broker)
/// - contain only ASCII alphanumeric characters, `'.'`, `'_'`, or `'-'`
/// - be at most 249 characters long
///
/// # Errors
///
/// Returns a human-readable description of the first rule violation found.
pub fn validate_kafka_topic(topic: &str) -> Result<(), String> {
    if topic.is_empty() {
        return Err("topic name must not be empty".to_string());
    }

    if topic == "." || topic == ".." {
        return Err("topic name must not be an ambiguous path component".to_string());
    }

    if topic.len() > MAX_KAFKA_TOPIC_LEN {
        return Err(format!(
            "topic name exceeds maximum length of {MAX_KAFKA_TOPIC_LEN} characters"
        ));
    }

    if let Some(pos) = topic
        .bytes()
        .position(|b| !(b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-'))
    {
        let bad_char = topic.as_bytes()[pos] as char;
        return Err(format!(
            "topic name contains invalid character '{bad_char}' at position {pos}; \
             only [a-zA-Z0-9._-] are allowed"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Valid topics ----

    #[test]
    fn validate_accepts_simple_topic() {
        assert!(validate_kafka_topic("my-topic").is_ok());
    }

    #[test]
    fn validate_accepts_single_char() {
        assert!(validate_kafka_topic("a").is_ok());
    }

    #[test]
    fn validate_accepts_dots_underscores_dashes() {
        assert!(validate_kafka_topic("a.b_c-d").is_ok());
    }

    #[test]
    fn validate_accepts_max_length_topic() {
        let topic = "a".repeat(MAX_KAFKA_TOPIC_LEN);
        assert!(validate_kafka_topic(&topic).is_ok());
    }

    #[test]
    fn validate_accepts_numeric_topic() {
        assert!(validate_kafka_topic("12345").is_ok());
    }

    #[test]
    fn validate_accepts_leading_underscore() {
        assert!(validate_kafka_topic("__consumer_offsets").is_ok());
    }

    #[test]
    fn validate_accepts_leading_dot() {
        assert!(validate_kafka_topic(".hidden-topic").is_ok());
    }

    #[test]
    fn validate_accepts_leading_dash() {
        assert!(validate_kafka_topic("-my-topic").is_ok());
    }

    // ---- Invalid topics ----

    #[test]
    fn validate_rejects_empty_string() {
        let err = validate_kafka_topic("").unwrap_err();
        assert!(err.contains("empty"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_single_dot() {
        let err = validate_kafka_topic(".").unwrap_err();
        assert!(err.contains("ambiguous"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_double_dot() {
        let err = validate_kafka_topic("..").unwrap_err();
        assert!(err.contains("ambiguous"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_exceeding_max_length() {
        let topic = "a".repeat(MAX_KAFKA_TOPIC_LEN + 1);
        let err = validate_kafka_topic(&topic).unwrap_err();
        assert!(err.contains("maximum length"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_space() {
        let err = validate_kafka_topic("topic with space").unwrap_err();
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_slash() {
        let err = validate_kafka_topic("topic/slash").unwrap_err();
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_colon() {
        let err = validate_kafka_topic("topic:colon").unwrap_err();
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_null_byte() {
        let err = validate_kafka_topic("topic\0null").unwrap_err();
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    #[test]
    fn validate_rejects_non_ascii() {
        let err = validate_kafka_topic("topic-café").unwrap_err();
        assert!(err.contains("invalid character"), "unexpected error: {err}");
    }

    // ---- LogLevel ----

    #[test]
    fn log_level_deserialize_all_variants() {
        let cases = [
            ("\"emerg\"", LogLevel::Emerg),
            ("\"alert\"", LogLevel::Alert),
            ("\"critical\"", LogLevel::Critical),
            ("\"error\"", LogLevel::Error),
            ("\"warning\"", LogLevel::Warning),
            ("\"notice\"", LogLevel::Notice),
            ("\"info\"", LogLevel::Info),
            ("\"debug\"", LogLevel::Debug),
        ];
        for (json, expected) in cases {
            let got: LogLevel = serde_json::from_str(json).unwrap();
            assert_eq!(got, expected, "failed for {json}");
        }
    }

    #[test]
    fn log_level_to_rdkafka_roundtrip() {
        // Verify each variant converts to the expected discriminant value.
        assert_eq!(LogLevel::Emerg.to_rdkafka() as i32, 0);
        assert_eq!(LogLevel::Alert.to_rdkafka() as i32, 1);
        assert_eq!(LogLevel::Critical.to_rdkafka() as i32, 2);
        assert_eq!(LogLevel::Error.to_rdkafka() as i32, 3);
        assert_eq!(LogLevel::Warning.to_rdkafka() as i32, 4);
        assert_eq!(LogLevel::Notice.to_rdkafka() as i32, 5);
        assert_eq!(LogLevel::Info.to_rdkafka() as i32, 6);
        assert_eq!(LogLevel::Debug.to_rdkafka() as i32, 7);
    }

    // ---- DebugContext ----

    #[test]
    fn debug_context_deserialize_all_variants() {
        let cases = [
            ("\"generic\"", DebugContext::Generic),
            ("\"broker\"", DebugContext::Broker),
            ("\"topic\"", DebugContext::Topic),
            ("\"metadata\"", DebugContext::Metadata),
            ("\"feature\"", DebugContext::Feature),
            ("\"queue\"", DebugContext::Queue),
            ("\"msg\"", DebugContext::Msg),
            ("\"protocol\"", DebugContext::Protocol),
            ("\"cgrp\"", DebugContext::Cgrp),
            ("\"security\"", DebugContext::Security),
            ("\"fetch\"", DebugContext::Fetch),
            ("\"interceptor\"", DebugContext::Interceptor),
            ("\"plugin\"", DebugContext::Plugin),
            ("\"consumer\"", DebugContext::Consumer),
            ("\"admin\"", DebugContext::Admin),
            ("\"eos\"", DebugContext::Eos),
            ("\"mock\"", DebugContext::Mock),
            ("\"assignor\"", DebugContext::Assignor),
            ("\"conf\"", DebugContext::Conf),
            ("\"telemetry\"", DebugContext::Telemetry),
            ("\"all\"", DebugContext::All),
        ];
        for (json, expected) in cases {
            let got: DebugContext = serde_json::from_str(json).unwrap();
            assert_eq!(got, expected, "failed for {json}");
        }
    }

    #[test]
    fn debug_context_as_str_roundtrip() {
        let all = [
            DebugContext::Generic,
            DebugContext::Broker,
            DebugContext::Topic,
            DebugContext::Metadata,
            DebugContext::Feature,
            DebugContext::Queue,
            DebugContext::Msg,
            DebugContext::Protocol,
            DebugContext::Cgrp,
            DebugContext::Security,
            DebugContext::Fetch,
            DebugContext::Interceptor,
            DebugContext::Plugin,
            DebugContext::Consumer,
            DebugContext::Admin,
            DebugContext::Eos,
            DebugContext::Mock,
            DebugContext::Assignor,
            DebugContext::Conf,
            DebugContext::Telemetry,
            DebugContext::All,
        ];
        for ctx in all {
            let s = ctx.as_str();
            let json = format!("\"{s}\"");
            let roundtripped: DebugContext = serde_json::from_str(&json).unwrap();
            assert_eq!(roundtripped, ctx, "roundtrip failed for {s}");
        }
    }

    #[test]
    fn debug_list_to_string_multiple() {
        let csv =
            debug_list_to_string(&[DebugContext::Broker, DebugContext::Topic, DebugContext::Msg]);
        assert_eq!(csv, "broker,topic,msg");
    }

    #[test]
    fn debug_list_to_string_single() {
        assert_eq!(debug_list_to_string(&[DebugContext::All]), "all");
    }

    #[test]
    fn debug_list_to_string_empty() {
        assert_eq!(debug_list_to_string(&[]), "");
    }

    // ---- TLS constructors ----

    #[test]
    fn tls_default_is_empty() {
        let tls = TlsConfig::default();
        assert!(tls.ca_file().is_none());
        assert!(tls.cert_file().is_none());
        assert!(tls.key_file().is_none());
        assert!(tls.key_password().is_none());
        assert!(!tls.insecure());
    }

    #[test]
    fn tls_default_validates_successfully() {
        assert!(TlsConfig::default().validate().is_ok());
    }

    #[test]
    fn tls_ca_only_constructor() {
        let tls = TlsConfig::ca_only("ca.pem".into());
        assert_eq!(tls.ca_file(), Some("ca.pem"));
        assert!(tls.cert_file().is_none());
        assert!(tls.key_file().is_none());
        assert!(tls.key_password().is_none());
        assert!(!tls.insecure());
    }

    #[test]
    fn tls_ca_only_validates_successfully() {
        assert!(TlsConfig::ca_only("ca.pem".into()).validate().is_ok());
    }

    #[test]
    fn tls_with_insecure_sets_flag() {
        let tls = TlsConfig::default().with_insecure(true);
        assert!(tls.insecure());
    }

    #[test]
    fn tls_ca_only_with_insecure() {
        let tls = TlsConfig::ca_only("ca.pem".into()).with_insecure(true);
        assert_eq!(tls.ca_file(), Some("ca.pem"));
        assert!(tls.insecure());
        assert!(tls.validate().is_ok());
    }

    #[test]
    fn tls_with_ca_file() {
        let tls = TlsConfig::default().with_ca_file("ca.pem");
        assert_eq!(tls.ca_file(), Some("ca.pem"));
        assert!(tls.cert_file().is_none());
        assert!(tls.key_file().is_none());
        assert!(tls.key_password().is_none());
        assert!(!tls.insecure());
    }

    #[test]
    fn tls_with_cert_and_key_files() {
        let tls = TlsConfig::default()
            .with_cert_file("cert.pem")
            .with_key_file("key.pem");
        assert_eq!(tls.cert_file(), Some("cert.pem"));
        assert_eq!(tls.key_file(), Some("key.pem"));
        assert!(tls.ca_file().is_none());
    }

    #[test]
    fn tls_with_key_password() {
        let tls = TlsConfig::default()
            .with_cert_file("cert.pem")
            .with_key_file("key.pem")
            .with_key_password("secret");
        assert_eq!(tls.key_password(), Some("secret"));
    }

    #[test]
    fn tls_builder_full_mtls() {
        let tls = TlsConfig::default()
            .with_ca_file("ca.pem")
            .with_cert_file("cert.pem")
            .with_key_file("key.pem")
            .with_key_password("secret")
            .with_insecure(false);
        assert_eq!(tls.ca_file(), Some("ca.pem"));
        assert_eq!(tls.cert_file(), Some("cert.pem"));
        assert_eq!(tls.key_file(), Some("key.pem"));
        assert_eq!(tls.key_password(), Some("secret"));
        assert!(!tls.insecure());
        assert!(tls.validate().is_ok());
    }

    // ---- TLS validation ----

    #[test]
    fn tls_validate_full_mtls_succeeds() {
        let tls = TlsConfig::new(
            "ca.pem".into(),
            "cert.pem".into(),
            "key.pem".into(),
            None,
            false,
        );
        assert!(tls.validate().is_ok());
    }

    #[test]
    fn tls_validate_ca_only_succeeds() {
        let tls = TlsConfig {
            ca_file: Some("ca.pem".into()),
            cert_file: None,
            key_file: None,
            key_password: None,
            insecure: false,
        };
        assert!(tls.validate().is_ok());
    }

    #[test]
    fn tls_validate_empty_block_succeeds() {
        let tls = TlsConfig {
            ca_file: None,
            cert_file: None,
            key_file: None,
            key_password: None,
            insecure: false,
        };
        assert!(tls.validate().is_ok());
    }

    #[test]
    fn tls_validate_with_key_password_succeeds() {
        let tls = TlsConfig::new(
            "ca.pem".into(),
            "cert.pem".into(),
            "key.pem".into(),
            Some("secret".into()),
            false,
        );
        assert!(tls.validate().is_ok());
    }

    #[test]
    fn tls_validate_cert_without_key_fails() {
        let tls = TlsConfig {
            ca_file: Some("ca.pem".into()),
            cert_file: Some("cert.pem".into()),
            key_file: None,
            key_password: None,
            insecure: false,
        };
        let err = tls.validate().unwrap_err();
        assert!(err.contains("cert_file"), "unexpected error: {err}");
        assert!(err.contains("key_file"), "unexpected error: {err}");
    }

    #[test]
    fn tls_validate_key_without_cert_fails() {
        let tls = TlsConfig {
            ca_file: Some("ca.pem".into()),
            cert_file: None,
            key_file: Some("key.pem".into()),
            key_password: None,
            insecure: false,
        };
        let err = tls.validate().unwrap_err();
        assert!(err.contains("key_file"), "unexpected error: {err}");
        assert!(err.contains("cert_file"), "unexpected error: {err}");
    }

    #[test]
    fn tls_validate_key_password_without_key_fails() {
        let tls = TlsConfig {
            ca_file: Some("ca.pem".into()),
            cert_file: None,
            key_file: None,
            key_password: Some("secret".into()),
            insecure: false,
        };
        let err = tls.validate().unwrap_err();
        assert!(err.contains("key_password"), "unexpected error: {err}");
        assert!(err.contains("key_file"), "unexpected error: {err}");
    }

    #[test]
    fn tls_validate_empty_ca_file_fails() {
        let tls = TlsConfig {
            ca_file: Some("".into()),
            cert_file: None,
            key_file: None,
            key_password: None,
            insecure: false,
        };
        let err = tls.validate().unwrap_err();
        assert!(err.contains("ca_file"), "unexpected error: {err}");
        assert!(err.contains("empty"), "unexpected error: {err}");
    }

    #[test]
    fn tls_validate_empty_cert_file_fails() {
        let tls = TlsConfig {
            ca_file: None,
            cert_file: Some("".into()),
            key_file: Some("key.pem".into()),
            key_password: None,
            insecure: false,
        };
        let err = tls.validate().unwrap_err();
        assert!(err.contains("cert_file"), "unexpected error: {err}");
        assert!(err.contains("empty"), "unexpected error: {err}");
    }

    #[test]
    fn tls_validate_empty_key_file_fails() {
        let tls = TlsConfig {
            ca_file: None,
            cert_file: Some("cert.pem".into()),
            key_file: Some("".into()),
            key_password: None,
            insecure: false,
        };
        let err = tls.validate().unwrap_err();
        assert!(err.contains("key_file"), "unexpected error: {err}");
        assert!(err.contains("empty"), "unexpected error: {err}");
    }

    #[test]
    fn tls_validate_empty_key_password_fails() {
        let tls = TlsConfig {
            ca_file: None,
            cert_file: Some("cert.pem".into()),
            key_file: Some("key.pem".into()),
            key_password: Some("".into()),
            insecure: false,
        };
        let err = tls.validate().unwrap_err();
        assert!(err.contains("key_password"), "unexpected error: {err}");
        assert!(err.contains("empty"), "unexpected error: {err}");
    }
}
