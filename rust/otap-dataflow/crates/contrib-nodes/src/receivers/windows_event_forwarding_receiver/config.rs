// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::xml::Document;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use uuid::Uuid;

/// Configuration of the single-replica WEF receiver.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// HTTPS listen address.
    pub endpoint: SocketAddr,
    /// Public HTTPS origin used in advertised subscriptions; absent keeps handshake-only mode.
    #[serde(default)]
    pub public_endpoint: Option<String>,
    /// Retain raw event XML in event.original; disabled by default.
    #[serde(default)]
    pub include_event_original: bool,
    /// Required mutual TLS credentials.
    pub tls: TlsConfig,
    /// Authorized DNS SAN source identities.
    pub auth: AuthConfig,
    /// Exactly one subscription is supported initially.
    pub subscriptions: Vec<Subscription>,
    /// Resource and timeout bounds.
    #[serde(default)]
    pub limits: Limits,
}

/// Server credentials and client trust anchors.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsConfig {
    /// PEM server certificate chain.
    pub cert_file: PathBuf,
    /// PEM server private key.
    pub key_file: PathBuf,
    /// PEM client CA certificate bundles.
    pub client_ca_files: Vec<PathBuf>,
}

/// Mutual TLS authorization policy.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    /// Exact lowercase DNS SAN identities; no wildcard matching.
    pub allowed_sources: Vec<String>,
}

/// Event representation requested from Windows.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentFormat {
    /// Event XML including rendered message and display text.
    #[default]
    RenderedText,
}

/// Starting point when no saved bookmark is available.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialRead {
    /// Collect only new matching events.
    #[default]
    NewEvents,
    /// Replay all retained matching events before collecting new events.
    AllExistingEvents,
}

/// Advertised subscription settings; field order defines the version encoding.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Subscription {
    /// Stable configured subscription name.
    pub name: String,
    /// Windows Event Log QueryList XML.
    pub query: String,
    /// Only rendered_text is supported initially.
    #[serde(default)]
    pub content_format: ContentFormat,
    /// Starting point when no bookmark is available.
    #[serde(default)]
    pub initial_read: InitialRead,
    /// Source heartbeat interval.
    #[serde(default = "sixty_seconds", with = "humantime_serde")]
    pub heartbeat: Duration,
    /// Maximum delivery latency.
    #[serde(default = "thirty_seconds", with = "humantime_serde")]
    pub max_time: Duration,
    /// Advertised WEF envelope size limit.
    #[serde(default = "envelope_size")]
    pub max_envelope_size_bytes: usize,
    /// Source connection retry count.
    #[serde(default = "retry_count")]
    pub connection_retry_count: u32,
    /// Source connection retry interval.
    #[serde(default = "sixty_seconds", with = "humantime_serde")]
    pub connection_retry: Duration,
}

/// Configurable limits, independent of the source's delivery preferences.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    /// Maximum bytes received in an HTTP body.
    pub max_request_bytes: usize,
    /// Maximum bytes after decompression.
    pub max_decompressed_bytes: usize,
    /// Maximum bytes of an individual event XML fragment.
    pub max_event_bytes: usize,
    /// Maximum bytes of opaque bookmark XML.
    pub max_bookmark_bytes: usize,
    /// Maximum source identities with retained state.
    pub max_sources: usize,
    /// Maximum unresolved WEF batches across all sources.
    pub max_in_flight_batches: usize,
    /// Maximum time awaiting downstream feedback.
    #[serde(with = "humantime_serde")]
    pub feedback_timeout: Duration,
    /// Maximum time reading an HTTP request.
    #[serde(with = "humantime_serde")]
    pub request_timeout: Duration,
    /// Maximum time establishing mutual TLS.
    #[serde(with = "humantime_serde")]
    pub handshake_timeout: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_request_bytes: 1024 * 1024, // 1 MiB per HTTP request body.
            max_decompressed_bytes: 8 * 1024 * 1024, // 8 MiB after decompression.
            max_event_bytes: 1024 * 1024,   // 1 MiB per event XML fragment.
            max_bookmark_bytes: 64 * 1024,  // 64 KiB per bookmark.
            max_sources: 1024,              // 1,024 source identities, not bytes.
            max_in_flight_batches: 64,      // 64 unresolved batches across all sources.
            feedback_timeout: thirty_seconds(),
            request_timeout: thirty_seconds(),
            handshake_timeout: Duration::from_secs(10),
        }
    }
}

impl Config {
    /// Validate settings without constructing receiver-name-dependent identities.
    ///
    /// Checks credential paths, authorization, subscriptions, and resource limits;
    /// it does not load certificates or bind sockets. When a public origin enables
    /// advertisement, the query must be well-formed XML with a QueryList root.
    /// Windows query semantics are not validated locally.
    pub fn validate(&self) -> Result<(), String> {
        if self.tls.client_ca_files.is_empty() {
            return Err("mutual TLS and at least one client CA are required".into());
        }
        if self.tls.cert_file.as_os_str().is_empty()
            || self.tls.key_file.as_os_str().is_empty()
            || self
                .tls
                .client_ca_files
                .iter()
                .any(|path| path.as_os_str().is_empty())
        {
            return Err("TLS credential paths must not be empty".into());
        }
        if self.auth.allowed_sources.is_empty() {
            return Err("allowed_sources must not be empty".into());
        }
        for source in &self.auth.allowed_sources {
            if normalize_source(source).as_deref() != Ok(source.as_str()) {
                return Err("allowed_sources must contain exact lowercase DNS identities".into());
            }
        }
        if self.subscriptions.len() != 1 {
            return Err("exactly one WEF subscription is required".into());
        }
        if let Some(endpoint) = &self.public_endpoint {
            let endpoint = url::Url::parse(endpoint)
                .map_err(|_| "public_endpoint must be an absolute HTTPS origin")?;
            if endpoint.scheme() != "https"
                || endpoint.host_str().is_none()
                || !endpoint.username().is_empty()
                || endpoint.password().is_some()
                || endpoint.path() != "/"
                || endpoint.query().is_some()
                || endpoint.fragment().is_some()
                || endpoint.port() == Some(0)
            {
                return Err("public_endpoint must be an HTTPS origin without credentials, path, query, or fragment".into());
            }
        }
        let subscription = &self.subscriptions[0];
        if subscription.name.trim().is_empty() || subscription.query.trim().is_empty() {
            return Err("subscription name and query must not be empty".into());
        }
        let limits = &self.limits;
        if [
            limits.max_request_bytes,
            limits.max_decompressed_bytes,
            limits.max_event_bytes,
            limits.max_bookmark_bytes,
            limits.max_sources,
            limits.max_in_flight_batches,
            subscription.max_envelope_size_bytes,
        ]
        .contains(&0)
        {
            return Err("size and capacity limits must be positive".into());
        }
        for duration in [
            limits.feedback_timeout,
            limits.request_timeout,
            limits.handshake_timeout,
            subscription.heartbeat,
            subscription.max_time,
            subscription.connection_retry,
        ] {
            if duration.is_zero() || duration.as_nanos() % 1_000_000 != 0 {
                return Err("durations must be positive whole milliseconds".into());
            }
        }
        if self.public_endpoint.is_some() {
            let query = Document::parse(&subscription.query)
                .map_err(|error| format!("invalid subscription query XML: {error}"))?;
            if !query.root_element().has_tag_name("QueryList") {
                return Err("subscription query must have a QueryList root".into());
            }
        }
        Ok(())
    }
}

impl Subscription {
    /// Compute UUID v5 identifiers without mutable or durable state.
    pub fn identifiers(&self, receiver_name: &str) -> Result<(Uuid, Uuid), serde_json::Error> {
        let application = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            b"urn:otel:receiver:windows_event_forwarding",
        );
        let receiver = Uuid::new_v5(&application, receiver_name.as_bytes());
        let subscription = Uuid::new_v5(&receiver, self.name.as_bytes());
        #[derive(Serialize)]
        struct CanonicalSettings {
            query: String,
            content_format: ContentFormat,
            initial_read: InitialRead,
            heartbeat_ms: u128,
            max_time_ms: u128,
            max_envelope_size_bytes: usize,
            connection_retry_count: u32,
            connection_retry_ms: u128,
        }
        let canonical = serde_json::to_vec(&CanonicalSettings {
            query: self.query.replace("\r\n", "\n").replace('\r', "\n"),
            content_format: self.content_format,
            initial_read: self.initial_read,
            heartbeat_ms: self.heartbeat.as_millis(),
            max_time_ms: self.max_time.as_millis(),
            max_envelope_size_bytes: self.max_envelope_size_bytes,
            connection_retry_count: self.connection_retry_count,
            connection_retry_ms: self.connection_retry.as_millis(),
        })?;
        Ok((subscription, Uuid::new_v5(&subscription, &canonical)))
    }
}

/// Normalize a single non-wildcard DNS SAN into a source identity.
pub fn normalize_source(source: &str) -> Result<String, String> {
    if source.is_empty() || source.len() > 253 || !source.is_ascii() {
        return Err("invalid DNS source identity".into());
    }
    for label in source.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err("invalid DNS source identity".into());
        }
    }
    Ok(source.to_ascii_lowercase())
}

fn sixty_seconds() -> Duration {
    Duration::from_secs(60)
}
fn thirty_seconds() -> Duration {
    Duration::from_secs(30)
}
fn envelope_size() -> usize {
    512_000
}
fn retry_count() -> u32 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    const YAML_CONFIG: &str = r#"
endpoint: 127.0.0.1:5986
tls:
    cert_file: server.pem
    key_file: key.pem
    client_ca_files:
        - ca.pem
auth:
    allowed_sources:
        - host.example.com
subscriptions:
    - name: security-audit
      query: |
            <QueryList>
                <Query Id="0" Path="Security">
                    <Select Path="Security">*</Select>
                </Query>
            </QueryList>
"#;

    /// Scenario: an administrator sets a public origin independently of the listen socket.
    /// Guarantees: only HTTPS origins can be advertised, and absent settings retain handshake mode.
    #[test]
    fn public_endpoint_validation() {
        let mut config: Config = serde_yaml::from_str(YAML_CONFIG).unwrap();
        assert!(config.public_endpoint.is_none());
        config.validate().unwrap();
        for endpoint in [
            "https://collector.example.com:5986",
            "https://collector.example.com/",
            "https://[::1]:5986",
        ] {
            config.public_endpoint = Some(endpoint.into());
            config.validate().unwrap();
        }
        for endpoint in [
            "http://collector",
            "collector:5986",
            "https://user:secret@collector",
            "https://collector/wsman",
            "https://collector?query",
            "https://collector/#fragment",
            "https://collector:0",
        ] {
            config.public_endpoint = Some(endpoint.into());
            assert!(config.validate().is_err(), "{endpoint}");
        }
    }

    /// Scenario: query XML is malformed or has the wrong root with advertisement enabled or disabled.
    /// Guarantees: validation needs no receiver name and preserves handshake-only query handling.
    #[test]
    fn query_validation_without_receiver_identity() {
        let mut config: Config = serde_yaml::from_str(YAML_CONFIG).unwrap();
        for query in ["<QueryList>", "<Query/>", "<QueryList xmlns='urn:other'/>"] {
            config.subscriptions[0].query = query.into();
            config.public_endpoint = None;
            config.validate().unwrap();
            config.public_endpoint = Some("https://collector.example.com".into());
            assert!(config.validate().is_err(), "accepted {query}");
        }
        config.subscriptions[0].query = "<QueryList/>".into();
        config.validate().unwrap();
    }

    /// Scenario: identical subscriptions are reconstructed after restart and edited.
    /// Guarantees: IDs are stable, version changes track settings, and rollback restores it.
    #[test]
    fn deterministic_versions() {
        let mut subscription: Subscription = serde_json::from_value(serde_json::json!({
            "name": "security-audit", "query": "<QueryList/>"
        }))
        .unwrap();
        let original = subscription.identifiers("wef").unwrap();
        assert_eq!(original, subscription.clone().identifiers("wef").unwrap());
        subscription.initial_read = InitialRead::AllExistingEvents;
        let changed = subscription.identifiers("wef").unwrap();
        assert_eq!(original.0, changed.0);
        assert_ne!(original.1, changed.1);
        subscription.initial_read = InitialRead::NewEvents;
        assert_eq!(original, subscription.identifiers("wef").unwrap());
        assert_ne!(original.0, subscription.identifiers("other").unwrap().0);
    }

    /// Scenario: source certificates contain uppercase, wildcard, or malformed DNS names.
    /// Guarantees: only unambiguous DNS identities normalize successfully.
    #[test]
    fn source_names() {
        assert_eq!(
            normalize_source("HOST.Example.com").unwrap(),
            "host.example.com"
        );
        for invalid in ["", "*.example.com", "host..com", "-host.com", "host."] {
            assert!(normalize_source(invalid).is_err());
        }
    }

    /// Scenario: equivalent settings use explicit defaults, different ordering, and CRLF XML.
    /// Guarantees: default expansion and line-ending normalization preserve ID and version.
    #[test]
    fn equivalent_configuration_has_same_version() {
        let implicit: Subscription = serde_json::from_value(serde_json::json!({
            "name": "test", "query": "<QueryList>\n</QueryList>"
        }))
        .unwrap();
        let explicit: Subscription = serde_json::from_value(serde_json::json!({
            "connection_retry": "60000ms", "connection_retry_count": 5,
            "max_envelope_size_bytes": 512000, "max_time": "30000ms",
            "heartbeat": "60000ms", "initial_read": "new_events",
            "content_format": "rendered_text", "name": "test",
            "query": "<QueryList>\r\n</QueryList>"
        }))
        .unwrap();
        assert_eq!(
            implicit.identifiers("wef").unwrap(),
            explicit.identifiers("wef").unwrap()
        );
    }

    /// Scenario: each advertised delivery setting changes independently.
    /// Guarantees: every setting changes the version without changing the subscription ID.
    #[test]
    fn all_advertised_settings_affect_version() {
        let subscription: Subscription = serde_json::from_value(serde_json::json!({
            "name": "test", "query": "<QueryList/>"
        }))
        .unwrap();
        let original = subscription.identifiers("wef").unwrap();
        let mut variants = vec![subscription.clone(); 7];
        variants[0].query.push(' ');
        variants[1].initial_read = InitialRead::AllExistingEvents;
        variants[2].heartbeat += Duration::from_millis(1);
        variants[3].max_time += Duration::from_millis(1);
        variants[4].max_envelope_size_bytes += 1;
        variants[5].connection_retry_count += 1;
        variants[6].connection_retry += Duration::from_millis(1);
        for variant in variants {
            let changed = variant.identifiers("wef").unwrap();
            assert_eq!(original.0, changed.0);
            assert_ne!(original.1, changed.1);
        }
        let mut renamed = subscription;
        renamed.name = "renamed".into();
        assert_ne!(original.0, renamed.identifiers("wef").unwrap().0);
    }

    /// Scenario: subscription enums are omitted, specified, or given unsupported values.
    /// Guarantees: defaults and snake_case values round-trip; unknown values and booleans fail.
    #[test]
    fn subscription_enums() {
        let mut value = serde_json::json!({"name": "test", "query": "<QueryList/>"});
        let defaults: Subscription = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(defaults.content_format, ContentFormat::RenderedText);
        assert_eq!(defaults.initial_read, InitialRead::NewEvents);
        value["content_format"] = serde_json::json!("rendered_text");
        for policy in ["new_events", "all_existing_events"] {
            value["initial_read"] = serde_json::json!(policy);
            let subscription: Subscription = serde_json::from_value(value.clone()).unwrap();
            let serialized = serde_json::to_value(subscription).unwrap();
            assert_eq!(serialized["content_format"], "rendered_text");
            assert_eq!(serialized["initial_read"], policy);
        }
        value["content_format"] = serde_json::json!("raw");
        assert!(serde_json::from_value::<Subscription>(value.clone()).is_err());
        value["content_format"] = serde_json::json!("rendered_text");
        for invalid in [serde_json::json!("last_ten"), serde_json::json!(true)] {
            value["initial_read"] = invalid;
            assert!(serde_json::from_value::<Subscription>(value.clone()).is_err());
        }
    }

    /// Scenario: full YAML configurations specify quoted or unquoted enums and durations.
    /// Guarantees: supported enum spellings select the correct variants and settings validate.
    #[test]
    fn yaml_configuration_explicit_settings() {
        for (policy, expected) in [
            ("new_events", InitialRead::NewEvents),
            ("all_existing_events", InitialRead::AllExistingEvents),
        ] {
            for quote in ["", "\"", "'"] {
                let yaml = format!(
                    r#"{YAML_CONFIG}      content_format: {quote}rendered_text{quote}
      initial_read: {quote}{policy}{quote}
      heartbeat: 45s
      max_time: 1500ms
      connection_retry: 2m
limits:
    feedback_timeout: 15s
    max_request_bytes: 2097152
"#
                );
                let config: Config = serde_yaml::from_str(&yaml).unwrap();
                config.validate().unwrap();
                let subscription = &config.subscriptions[0];
                assert_eq!(subscription.content_format, ContentFormat::RenderedText);
                assert_eq!(subscription.initial_read, expected);
                assert_eq!(subscription.heartbeat, Duration::from_secs(45));
                assert_eq!(subscription.max_time, Duration::from_millis(1500));
                assert_eq!(subscription.connection_retry, Duration::from_secs(120));
                assert_eq!(config.limits.feedback_timeout, Duration::from_secs(15));
                assert_eq!(config.limits.max_request_bytes, 2 * 1024 * 1024);
                assert_eq!(config.limits.max_sources, 1024);
            }
        }
    }

    /// Scenario: original XML retention is omitted, enabled, disabled, or given an invalid YAML value.
    /// Guarantees: retention defaults off, accepts booleans only, and never changes the advertised subscription identity/version.
    #[test]
    fn yaml_original_xml_retention() {
        let default: Config = serde_yaml::from_str(YAML_CONFIG).unwrap();
        assert!(!default.include_event_original);
        let identifiers = default.subscriptions[0].identifiers("wef").unwrap();
        for include in [false, true] {
            let config: Config =
                serde_yaml::from_str(&format!("{YAML_CONFIG}include_event_original: {include}\n"))
                    .unwrap();
            config.validate().unwrap();
            assert_eq!(config.include_event_original, include);
            assert_eq!(
                config.subscriptions[0].identifiers("wef").unwrap(),
                identifiers
            );
        }
        for invalid in ["'true'", "1", "null"] {
            assert!(
                serde_yaml::from_str::<Config>(&format!(
                    "{YAML_CONFIG}include_event_original: {invalid}\n"
                ))
                .is_err()
            );
        }
    }

    /// Scenario: YAML enum fields contain unsupported names, wrong casing, or non-string values.
    /// Guarantees: invalid enum inputs fail deserialization instead of silently using defaults.
    #[test]
    fn yaml_configuration_rejects_invalid_enums() {
        for (field, invalid) in [
            ("content_format", "raw"),
            ("content_format", "RenderedText"),
            ("content_format", "true"),
            ("content_format", "10"),
            ("content_format", "null"),
            ("initial_read", "last_ten"),
            ("initial_read", "NewEvents"),
            ("initial_read", "true"),
            ("initial_read", "false"),
            ("initial_read", "10"),
            ("initial_read", "null"),
        ] {
            let yaml = format!("{YAML_CONFIG}      {field}: {invalid}\n");
            let error = serde_yaml::from_str::<Config>(&yaml)
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("unknown variant") || error.contains("invalid type"),
                "expected an enum error for {field}: {invalid}, got: {error}"
            );
        }
    }

    /// Scenario: YAML omits optional settings and transport, capacity, or authorization changes.
    /// Guarantees: safe defaults validate while unsupported and unbounded settings fail.
    #[test]
    fn config_validation() {
        let mut config: Config = serde_yaml::from_str(YAML_CONFIG).unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.endpoint, "127.0.0.1:5986".parse().unwrap());
        assert_eq!(config.tls.cert_file, PathBuf::from("server.pem"));
        assert_eq!(config.tls.key_file, PathBuf::from("key.pem"));
        assert_eq!(config.tls.client_ca_files, vec![PathBuf::from("ca.pem")]);
        assert_eq!(config.auth.allowed_sources, vec!["host.example.com"]);
        let subscription = &config.subscriptions[0];
        assert_eq!(subscription.name, "security-audit");
        assert!(subscription.query.starts_with("<QueryList>\n"));
        assert!(subscription.query.ends_with("</QueryList>\n"));
        assert_eq!(subscription.content_format, ContentFormat::RenderedText);
        assert_eq!(subscription.initial_read, InitialRead::NewEvents);
        assert_eq!(subscription.heartbeat, Duration::from_secs(60));
        assert_eq!(subscription.max_time, Duration::from_secs(30));
        assert_eq!(subscription.connection_retry, Duration::from_secs(60));
        let client_ca_files = std::mem::take(&mut config.tls.client_ca_files);
        assert!(config.validate().is_err());
        config.tls.client_ca_files = client_ca_files;
        config.limits.max_sources = 0;
        assert!(config.validate().is_err());
        config.limits = Limits::default();
        assert_eq!(config.limits.max_request_bytes, 1024 * 1024);
        assert_eq!(config.limits.max_decompressed_bytes, 8 * 1024 * 1024);
        assert_eq!(config.limits.max_event_bytes, 1024 * 1024);
        assert_eq!(config.limits.max_bookmark_bytes, 64 * 1024);
        assert_eq!(config.limits.max_sources, 1024);
        assert_eq!(config.limits.max_in_flight_batches, 64);
        assert_eq!(config.limits.feedback_timeout, Duration::from_secs(30));
        config.limits.feedback_timeout = Duration::from_nanos(1);
        assert!(config.validate().is_err());
        config.limits = Limits::default();
        let allowed_sources = std::mem::take(&mut config.auth.allowed_sources);
        assert!(config.validate().is_err());
        config.auth.allowed_sources = allowed_sources;
        config.subscriptions.push(config.subscriptions[0].clone());
        assert!(config.validate().is_err());
        config.subscriptions.truncate(1);
        config.tls.cert_file = PathBuf::new();
        assert!(config.validate().is_err());
    }
}
