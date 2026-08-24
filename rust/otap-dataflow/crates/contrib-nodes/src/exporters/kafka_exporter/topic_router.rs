// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dynamic topic routing for the Kafka exporter.
//!
//! Resolves the destination Kafka topic for a payload using a priority hierarchy:
//!
//! 1. **Transport header** (`topic_from_transport_header` on the per-signal config):
//!    If configured for the signal type and the header is present in the pdata
//!    context, its value becomes the topic for the batch. If the header is
//!    present but its value is an invalid Kafka topic, routing fails with
//!    [`KafkaExporterError::InvalidHeaderTopic`] and the batch is permanently
//!    nacked -- it does **not** fall back to the static topic.
//! 2. **Static fallback**: The per-signal `topic` from config, used only when
//!    the configured header is absent (or no header key is configured).
//!
//! Each signal type can use a different transport header key (or none), allowing
//! independent dynamic routing per signal.
//!
//! # Security: constraining dynamic routing
//!
//! A client-controlled transport header can select the destination topic, so a
//! header-supplied topic is constrained two ways before it is used:
//!
//! 1. It must be a syntactically valid Kafka topic ([`validate_kafka_topic`]).
//! 2. If the signal configures an operator allowlist -- an exact-match list
//!    ([`SignalConfig::allowed_topics`]) and/or a regex list
//!    ([`SignalConfig::allowed_topics_regex`]) -- the topic must match it. Regex
//!    patterns are anchored to a whole-topic match, so a prefix/suffix pattern
//!    cannot be satisfied by a substring of a client-supplied topic.
//!
//! A header-supplied topic that fails either check is non-retryable, so the
//! batch is permanently nacked rather than being rerouted to the static topic.
//! The static per-signal topic is operator-controlled and is never subject to
//! the allowlist.

use super::config::SignalConfig;
use super::error::KafkaExporterError;
use super::metrics::{KafkaExporterMetrics, KafkaTopicSource};
use crate::common::kafka::validate_kafka_topic;
use otap_df_config::SignalType;
use otap_df_config::transport_headers::TransportHeader;
use otap_df_otap::pdata::Context;
use regex::Regex;
use std::borrow::Cow;

/// Stateless topic router for the Kafka exporter.
///
/// Resolves the destination Kafka topic by inspecting the per-signal config
/// and the pdata context's transport headers. No fields, no construction,
/// no heap allocation.
///
/// The router records `exporter.kafka.routing.messages` with the bounded
/// `topic.source` value at the point where the source is determined, so callers
/// only need to know the resolved topic -- not how it was resolved.
pub struct TopicRouter;

impl TopicRouter {
    /// Resolves the destination topic for a signal and increments the
    /// appropriate topic routing metric.
    ///
    /// Returns `Ok(Cow::Borrowed)` on the static path (zero allocation, borrows
    /// from `signal_config`) or `Ok(Cow::Owned)` on the header path (one
    /// allocation for the extracted header value).
    ///
    /// If a topic is supplied via a transport header but is invalid, this
    /// returns [`KafkaExporterError::InvalidHeaderTopic`] instead of falling
    /// back to the static topic. The caller is expected to permanently nack the
    /// batch, since rerouting an explicitly-requested-but-invalid topic to the
    /// static topic could silently misdeliver tenant data.
    ///
    /// If a syntactically valid header topic is not permitted by the signal's
    /// operator-configured allowlist ([`SignalConfig::allowed_topics`] exact
    /// entries or the pre-compiled `allowed_regex` patterns), this returns
    /// [`KafkaExporterError::DisallowedHeaderTopic`] (also a permanent-nack
    /// condition). When no allowlist is configured (empty exact list and
    /// `allowed_regex` is `None`), every syntactically valid header topic is
    /// permitted (backwards compatible).
    ///
    /// # Arguments
    ///
    /// * `signal_config` - The per-signal config (static topic, header key, exact allowlist)
    /// * `allowed_regex` - Pre-compiled regex allowlist for the signal, or
    ///   `None` when the signal configures no regex patterns
    /// * `context` - The pdata context (carries transport headers)
    /// * `signal` - Signal used to attribute the bounded routing metric
    /// * `metrics` - Exporter metrics to increment topic routing counters
    pub fn resolve<'a>(
        signal_config: &'a SignalConfig,
        allowed_regex: Option<&[Regex]>,
        context: &Context,
        signal: SignalType,
        metrics: &mut KafkaExporterMetrics,
    ) -> Result<Cow<'a, str>, KafkaExporterError> {
        // Priority 1: topic from a transport header, if configured and present.
        if let Some(header) = Self::header_topic(signal_config, context) {
            // A present routing header must be a usable Kafka topic. If it is
            // not (non-UTF-8 value, or a value that fails Kafka topic
            // validation) this is non-retryable: surface an error so the batch
            // is permanently nacked rather than silently falling back to the
            // static topic, which would misdeliver the data.
            let topic = header.value_as_str().ok_or_else(|| {
                KafkaExporterError::invalid_header_topic(
                    String::from_utf8_lossy(&header.value),
                    "value is not valid UTF-8",
                )
            })?;
            validate_kafka_topic(topic)
                .map_err(|reason| KafkaExporterError::invalid_header_topic(topic, reason))?;

            // Operator-controlled routing constraint: a syntactically valid
            // header topic must still be permitted by the signal's allowlist
            // (exact match or a matching regex), if one is configured. This
            // bounds where a client-controlled routing header may direct data.
            if !Self::is_header_topic_allowed(signal_config, allowed_regex, topic) {
                return Err(KafkaExporterError::disallowed_header_topic(topic));
            }

            metrics.record_routing(signal, KafkaTopicSource::Header);
            return Ok(Cow::Owned(topic.to_owned()));
        }

        // Priority 2: static per-signal topic (zero-allocation borrow).
        metrics.record_routing(signal, KafkaTopicSource::StaticConfig);
        Ok(Cow::Borrowed(signal_config.topic()))
    }

    /// Returns `true` if a syntactically valid header-supplied `topic` is
    /// permitted for this signal.
    ///
    /// When the signal configures no constraint (empty exact allowlist and
    /// `allowed_regex` is `None`) every topic is permitted. Otherwise the topic
    /// must exactly match an `allowed_topics` entry or match one of the
    /// pre-compiled `allowed_regex` patterns.
    fn is_header_topic_allowed(
        signal_config: &SignalConfig,
        allowed_regex: Option<&[Regex]>,
        topic: &str,
    ) -> bool {
        let exact = signal_config.allowed_topics();
        if exact.is_empty() && allowed_regex.is_none() {
            return true;
        }
        if exact.iter().any(|t| t == topic) {
            return true;
        }
        // The patterns are pre-compiled anchored (`\A(?:...)\z`, see
        // `compile_allowed_topic_regexes`), so `is_match` here requires a
        // whole-topic match rather than a substring match -- a client-controlled
        // header cannot slip an unintended topic past a prefix/suffix pattern.
        allowed_regex.is_some_and(|patterns| patterns.iter().any(|re| re.is_match(topic)))
    }

    /// Returns the transport header whose name matches this signal's configured
    /// topic-routing key, or `None` if routing-by-header is not configured for
    /// the signal or no matching header is present. The first matching header
    /// wins.
    fn header_topic<'a>(
        signal_config: &SignalConfig,
        context: &'a Context,
    ) -> Option<&'a TransportHeader> {
        // `topic_from_transport_header` is pre-normalized (lowercased) in
        // `KafkaExporterConfig::try_from`, matching how transport headers store
        // their logical names, so a plain equality check is sufficient here.
        let header_key = signal_config.topic_from_transport_header()?;
        context
            .transport_headers()?
            .iter()
            .find(|h| h.name == *header_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::kafka::MessageFormat;
    use otap_df_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
    use otap_df_otap::pdata::Context;

    // ---- Test helpers ----

    fn make_transport_header(wire_name: &str, value: &str) -> TransportHeader {
        TransportHeader {
            // Mirror capture-time normalization: lowercase, dashes preserved.
            name: wire_name.to_ascii_lowercase(),
            wire_name: wire_name.to_string(),
            value_kind: ValueKind::Text,
            value: value.as_bytes().to_vec(),
        }
    }

    fn context_with_headers(headers: Vec<TransportHeader>) -> Context {
        let mut th = TransportHeaders::new();
        for h in headers {
            th.push(h);
        }
        let mut ctx = Context::default();
        ctx.set_transport_headers(th);
        ctx
    }

    fn make_signal_config(topic: &str, header_key: Option<&str>) -> SignalConfig {
        let config = SignalConfig::new(topic.to_string(), MessageFormat::OtlpProto);
        match header_key {
            Some(key) => config.with_topic_from_transport_header(key),
            None => config,
        }
    }

    fn routing_count(
        metrics: &KafkaExporterMetrics,
        signal: SignalType,
        source: KafkaTopicSource,
    ) -> u64 {
        metrics.routing_for(signal, source).messages.get()
    }

    // ---- Transport header resolution tests ----

    /// Scenario: A valid target topic header is present.
    /// Guarantees: Resolves to the header value and increments the header routing metric.
    #[test]
    fn test_resolve_header_present() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = context_with_headers(vec![make_transport_header(
            "X-Target-Topic",
            "tenant-a-logs",
        )]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("valid topic");
        assert_eq!(&*topic, "tenant-a-logs");
        assert!(matches!(topic, Cow::Owned(_)));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            1
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
    }

    /// Scenario: Target topic header is absent but another header exists.
    /// Guarantees: Falls back to the static topic and increments the static routing metric.
    #[test]
    fn test_resolve_header_absent() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Other-Header", "value")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            1
        );
    }

    /// Scenario: Context contains no transport headers at all.
    /// Guarantees: Falls back to the static topic and increments the static routing metric.
    #[test]
    fn test_resolve_header_no_transport_headers_on_context() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            1
        );
    }

    /// Scenario: The header routing is not configured for the signal.
    /// Guarantees: Falls back to the static topic ignoring present headers.
    #[test]
    fn test_resolve_header_not_configured() {
        let config = make_signal_config("fallback-logs", None);
        let ctx = context_with_headers(vec![make_transport_header("X-Target-Topic", "topic-a")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            1
        );
    }

    /// Scenario: Both static topic and transport header are available.
    /// Guarantees: The transport header value takes precedence.
    #[test]
    fn test_resolve_header_wins_over_static() {
        let config = make_signal_config("static-topic", Some("x-target-topic"));
        let ctx = context_with_headers(vec![make_transport_header(
            "X-Target-Topic",
            "header-topic",
        )]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("valid topic");
        assert_eq!(&*topic, "header-topic");
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            1
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
    }

    /// Scenario: Static fallback is chosen.
    /// Guarantees: Returns a Cow::Borrowed, avoiding allocation.
    #[test]
    fn test_resolve_static_path_returns_borrowed() {
        let config = make_signal_config("my-topic", None);
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("static topic");
        assert_eq!(&*topic, "my-topic");
        assert!(
            matches!(topic, Cow::Borrowed(_)),
            "static path should return Cow::Borrowed (zero allocation)"
        );
    }

    /// Scenario: Dynamic header topic is chosen.
    /// Guarantees: Returns a Cow::Owned of the header topic.
    #[test]
    fn test_resolve_header_path_returns_owned() {
        let config = make_signal_config("fallback", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "dynamic")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("valid topic");
        assert_eq!(&*topic, "dynamic");
        assert!(
            matches!(topic, Cow::Owned(_)),
            "header path should return Cow::Owned"
        );
    }

    /// Scenario: Different signals use different header keys for routing.
    /// Guarantees: Topics are correctly resolved based on per-signal configurations.
    #[test]
    fn test_per_signal_header_keys() {
        let traces_config = make_signal_config("otlp_spans", Some("x-traces-topic"));
        let metrics_config = make_signal_config("otlp_metrics", None);
        let logs_config = make_signal_config("otlp_logs", Some("x-logs-topic"));

        let ctx = context_with_headers(vec![
            make_transport_header("X-Traces-Topic", "custom-traces"),
            make_transport_header("X-Logs-Topic", "custom-logs"),
        ]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        // Traces: header present -> dynamic topic
        let topic =
            TopicRouter::resolve(&traces_config, None, &ctx, SignalType::Traces, &mut metrics)
                .expect("valid topic");
        assert_eq!(&*topic, "custom-traces");

        // Metrics: no header key configured -> static fallback
        let topic = TopicRouter::resolve(
            &metrics_config,
            None,
            &ctx,
            SignalType::Metrics,
            &mut metrics,
        )
        .expect("static topic");
        assert_eq!(&*topic, "otlp_metrics");

        // Logs: header present -> dynamic topic
        let topic = TopicRouter::resolve(&logs_config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("valid topic");
        assert_eq!(&*topic, "custom-logs");

        assert_eq!(
            routing_count(&metrics, SignalType::Traces, KafkaTopicSource::Header)
                + routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            2
        );
        assert_eq!(
            routing_count(
                &metrics,
                SignalType::Metrics,
                KafkaTopicSource::StaticConfig
            ),
            1
        );
    }

    /// Scenario: A signal's specifically configured header key is absent.
    /// Guarantees: Falls back to the static topic for that signal.
    #[test]
    fn test_per_signal_header_key_absent_falls_back() {
        let config = make_signal_config("fallback-logs", Some("x-logs-topic"));
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            1
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }

    // ---- Invalid header topic returns an error (no static fallback) ----

    /// Scenario: The specified transport header provides an empty topic.
    /// Guarantees: Returns an InvalidHeaderTopic error without falling back.
    #[test]
    fn test_resolve_invalid_header_topic_empty_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let result = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics);
        assert!(matches!(
            result,
            Err(KafkaExporterError::InvalidHeaderTopic { .. })
        ));
        // No fallback to static topic, and no topic routing metric incremented.
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }

    /// Scenario: The transport header topic is "." which is invalid for Kafka.
    /// Guarantees: Returns an InvalidHeaderTopic error without falling back.
    #[test]
    fn test_resolve_invalid_header_topic_dot_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", ".")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let result = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics);
        assert!(matches!(
            result,
            Err(KafkaExporterError::InvalidHeaderTopic { .. })
        ));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }

    /// Scenario: The transport header topic is ".." which is invalid for Kafka.
    /// Guarantees: Returns an InvalidHeaderTopic error without falling back.
    #[test]
    fn test_resolve_invalid_header_topic_dotdot_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "..")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let result = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics);
        assert!(matches!(
            result,
            Err(KafkaExporterError::InvalidHeaderTopic { .. })
        ));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
    }

    /// Scenario: The transport header topic contains invalid characters.
    /// Guarantees: Returns an InvalidHeaderTopic error without falling back.
    #[test]
    fn test_resolve_invalid_header_topic_bad_chars_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "bad topic/name")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let result = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics);
        assert!(matches!(
            result,
            Err(KafkaExporterError::InvalidHeaderTopic { .. })
        ));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }

    /// Scenario: The transport header topic exceeds maximum length.
    /// Guarantees: Returns an InvalidHeaderTopic error without falling back.
    #[test]
    fn test_resolve_invalid_header_topic_too_long_errors() {
        let long_topic = "a".repeat(250);
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", &long_topic)]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let result = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics);
        assert!(matches!(
            result,
            Err(KafkaExporterError::InvalidHeaderTopic { .. })
        ));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }

    /// Scenario: The transport header topic is not valid UTF-8.
    /// Guarantees: Returns an InvalidHeaderTopic error without falling back.
    #[test]
    fn test_resolve_non_utf8_header_topic_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        // A matching routing header whose value is not valid UTF-8. This must be
        // treated as a routing error (permanent nack), not as a missing header
        // that falls back to the static topic.
        let header = TransportHeader {
            name: "x-topic".to_string(),
            wire_name: "X-Topic".to_string(),
            value_kind: ValueKind::Binary,
            value: vec![0xff, 0xfe, 0xfd],
        };
        let ctx = context_with_headers(vec![header]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let result = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics);
        assert!(matches!(
            result,
            Err(KafkaExporterError::InvalidHeaderTopic { .. })
        ));
        // No fallback to static topic, and no topic routing metric incremented.
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }

    /// Scenario: A valid transport header topic is provided.
    /// Guarantees: Resolves to the header value without errors.
    #[test]
    fn test_resolve_valid_header_topic_still_works() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "valid-topic-123")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("valid topic");
        assert_eq!(&*topic, "valid-topic-123");
        assert!(matches!(topic, Cow::Owned(_)));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            1
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
    }

    /// Scenario: The transport header is mixed case but matches normalized lowercase config.
    /// Guarantees: Successfully matches and resolves the header topic.
    #[test]
    fn test_resolve_matches_normalized_config_key_for_mixed_case_header() {
        // A header arriving as `X-Target-Topic` is captured (and normalized) as
        // `x-target-topic`. The config key must be the normalized form for the
        // router to match it -- `KafkaExporterConfig::try_from` produces this
        // form from a natural config like `X-Target-Topic`.
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = context_with_headers(vec![make_transport_header(
            "X-Target-Topic",
            "tenant-a-logs",
        )]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("valid topic");
        assert_eq!(&*topic, "tenant-a-logs");
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            1
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
    }

    // ---- Security: operator allowlist / prefix constraint on dynamic routing ----

    /// Compiles allowlist regex patterns the same way the exporter does at
    /// runtime -- via `topic_regex::compile_anchor_and_validate`, which
    /// validates each pattern as a standalone regex and anchors it to a
    /// whole-topic match
    /// (`\A(?:<pattern>)\z`) -- so the router tests exercise the real
    /// whole-topic matching semantics rather than a substring search.
    fn compile(patterns: &[&str]) -> Vec<Regex> {
        patterns
            .iter()
            .map(|p| {
                crate::exporters::kafka_exporter::topic_regex::compile_anchor_and_validate(p)
                    .expect("valid regex")
            })
            .collect()
    }

    /// Scenario: a header-supplied topic that matches a configured regex
    /// allowlist pattern.
    /// Guarantees: a header topic matching an allowed regex is routed normally,
    /// so legitimate tenant-scoped routing keeps working under a constraint.
    #[test]
    fn test_allowed_regex_permits_matching_header_topic() {
        let config = make_signal_config("fallback", Some("x-topic"));
        let allowed = compile(&["tenant_.*"]);
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "tenant_a_logs")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        )
        .expect("allowed topic");
        assert_eq!(&*topic, "tenant_a_logs");
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            1
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
    }

    /// Scenario: allowlist regex patterns are compiled anchored to a whole-topic
    /// match (as the exporter does at runtime), and a client-supplied header
    /// topic that merely CONTAINS an allowed pattern is presented.
    /// Guarantees: `tenant_.*` permits the whole-topic `tenant_a_logs` but
    /// rejects `x-tenant_evil` (where the pattern only matches a substring) as a
    /// `DisallowedHeaderTopic` -- so an unanchored operator pattern cannot be
    /// bypassed by embedding the allowed fragment inside an arbitrary topic.
    #[test]
    fn test_regex_allowlist_requires_whole_topic_match() {
        let config = make_signal_config("fallback", Some("x-topic"));
        let allowed = compile(&["tenant_.*"]);

        // A topic that only contains the pattern as a substring is rejected.
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "x-tenant_evil")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );
        let result = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        );
        assert!(
            matches!(
                result,
                Err(KafkaExporterError::DisallowedHeaderTopic { .. })
            ),
            "a substring-only match must be rejected under whole-topic anchoring"
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );

        // A topic that matches the pattern as the whole string is permitted.
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "tenant_a_logs")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );
        let topic = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        )
        .expect("whole-topic match is permitted");
        assert_eq!(&*topic, "tenant_a_logs");
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            1
        );
    }

    /// Scenario (security: dynamic topic routing): the anchor-breakout operator
    /// pattern `tenant_.)\z|(?:evil.` is compiled through the real
    /// `topic_regex::compile_anchor_and_validate` helper the router uses.
    /// Guarantees: the helper rejects the pattern (so it can never become an
    /// allowlist entry), and a legitimately compiled `tenant_.*` allowlist does
    /// not route the unintended `evil`-suffixed topic that the broken-out
    /// pattern would have permitted -- confirming the bypass is closed on this
    /// authorization boundary.
    #[test]
    fn test_anchor_breakout_pattern_cannot_authorize_unintended_topic() {
        // The breakout pattern is rejected at compile time.
        let compiled = crate::exporters::kafka_exporter::topic_regex::compile_anchor_and_validate(
            r"tenant_.)\z|(?:evil.",
        );
        assert!(
            compiled.is_err(),
            "the anchor-breakout pattern must not compile into an allowlist entry"
        );

        // With only a legitimate `tenant_.*` allowlist, the topic the broken-out
        // pattern would have authorized is rejected.
        let config = make_signal_config("fallback", Some("x-topic"));
        let allowed = compile(&["tenant_.*"]);
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "x-evil_")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );
        let result = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        );
        assert!(
            matches!(
                result,
                Err(KafkaExporterError::DisallowedHeaderTopic { .. })
            ),
            "an `evil`-suffixed topic must not be authorized by a legitimate allowlist"
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }

    /// Scenario: a syntactically valid header-supplied topic that matches
    /// neither the regex allowlist nor the exact allowlist.
    /// Guarantees: a disallowed header topic returns `DisallowedHeaderTopic`
    /// (permanent-nack condition) without falling back to the static topic and
    /// without incrementing a routing metric, so a client cannot direct data to
    /// an arbitrary topic.
    #[test]
    fn test_disallowed_header_topic_is_rejected_without_fallback() {
        let config = make_signal_config("fallback", Some("x-topic"));
        let allowed = compile(&["tenant_.*"]);
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "evil-topic")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let result = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        );
        assert!(matches!(
            result,
            Err(KafkaExporterError::DisallowedHeaderTopic { .. })
        ));
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            0
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }

    /// Scenario: a header-supplied topic that exactly matches the exact-match
    /// allowlist.
    /// Guarantees: an exact-allowlisted header topic is routed, while the
    /// constraint still applies (see the disallowed case).
    #[test]
    fn test_exact_allowlist_permits_listed_header_topic() {
        let config = make_signal_config("fallback", Some("x-topic"))
            .with_allowed_topics(["approved-a", "approved-b"]);
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "approved-b")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("allowed topic");
        assert_eq!(&*topic, "approved-b");
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            1
        );
    }

    /// Scenario: a header topic matches an entry in the exact allowlist while a
    /// regex allowlist is also configured (either may satisfy the constraint).
    /// Guarantees: exact-list and regex-list are combined with OR semantics.
    #[test]
    fn test_exact_or_regex_allowlist_combined() {
        let config =
            make_signal_config("fallback", Some("x-topic")).with_allowed_topics(["special-topic"]);
        let allowed = compile(&["tenant_.*"]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        // Matches exact allowlist (not the regex).
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "special-topic")]);
        let topic = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        )
        .expect("exact match");
        assert_eq!(&*topic, "special-topic");

        // Matches the regex (not the exact list).
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "tenant_x")]);
        let topic = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        )
        .expect("regex match");
        assert_eq!(&*topic, "tenant_x");

        // Matches neither.
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "other")]);
        let result = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        );
        assert!(matches!(
            result,
            Err(KafkaExporterError::DisallowedHeaderTopic { .. })
        ));
    }

    /// Scenario: no allowlist constraint is configured (the default).
    /// Guarantees: dynamic routing is unrestricted (backwards compatible) -- any
    /// syntactically valid header topic is accepted.
    #[test]
    fn test_no_constraint_allows_any_valid_header_topic() {
        let config = make_signal_config("fallback", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "anything-valid")]);
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(&config, None, &ctx, SignalType::Logs, &mut metrics)
            .expect("valid topic");
        assert_eq!(&*topic, "anything-valid");
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            1
        );
    }

    /// Scenario: an allowlist is configured, but the request uses the static
    /// (config) topic path because no routing header is present.
    /// Guarantees: the allowlist constrains only the header-supplied path; the
    /// operator-controlled static topic is never subject to it.
    #[test]
    fn test_allowlist_does_not_constrain_static_topic() {
        // Static topic "fallback" is not in the allowlist, but it must still be
        // used when no routing header is present.
        let config = make_signal_config("fallback", Some("x-topic"));
        let allowed = compile(&["tenant_.*"]);
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::register(
            &crate::exporters::kafka_exporter::exporter::test_support::pipeline_context(),
        );

        let topic = TopicRouter::resolve(
            &config,
            Some(&allowed),
            &ctx,
            SignalType::Logs,
            &mut metrics,
        )
        .expect("static topic");
        assert_eq!(&*topic, "fallback");
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::StaticConfig),
            1
        );
        assert_eq!(
            routing_count(&metrics, SignalType::Logs, KafkaTopicSource::Header),
            0
        );
    }
}
