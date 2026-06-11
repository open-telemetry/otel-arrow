// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dynamic topic routing for the Kafka exporter.
//!
//! Resolves the destination Kafka topic for a payload using a priority hierarchy:
//!
//! 1. **Transport header** (`topic_from_transport_header` on the per-signal config):
//!    If configured for the signal type and the header is present in the pdata
//!    context, its value becomes the topic for the batch.
//! 2. **Static fallback**: The per-signal `topic` from config.
//!
//! Each signal type can use a different transport header key (or none), allowing
//! independent dynamic routing per signal.
//!

// TODO: allow prefix or acl mechanism so the operator can have some control over where these messages wind up (e.g. topic must start with tenant_)

use super::config::SignalConfig;
use super::metrics::KafkaExporterMetrics;
use crate::common::kafka::validate_kafka_topic;
use otap_df_otap::pdata::Context;
use std::borrow::Cow;

/// Stateless topic router for the Kafka exporter.
///
/// Resolves the destination Kafka topic by inspecting the per-signal config
/// and the pdata context's transport headers. No fields, no construction,
/// no heap allocation.
///
/// The router increments topic routing metrics (`topic_from_header`,
/// `topic_from_static_config`) at the point where the topic source is
/// determined, so callers only need to know the resolved topic -- not how
/// it was resolved.
pub struct TopicRouter;

impl TopicRouter {
    /// Resolves the destination topic for a signal and increments the
    /// appropriate topic routing metric.
    ///
    /// Returns `Cow::Borrowed` on the static path (zero allocation, borrows
    /// from `signal_config`) or `Cow::Owned` on the header path (one
    /// allocation for the extracted header value).
    ///
    /// # Arguments
    ///
    /// * `signal_config` - The per-signal config (carries the static topic and optional header key)
    /// * `context` - The pdata context (carries transport headers)
    /// * `metrics` - Exporter metrics to increment topic routing counters
    pub fn resolve<'a>(
        signal_config: &'a SignalConfig,
        context: &Context,
        metrics: &mut KafkaExporterMetrics,
    ) -> Cow<'a, str> {
        // Priority 1: Transport header for this signal type
        if let Some(topic) = Self::resolve_from_header(signal_config, context) {
            if validate_kafka_topic(&topic).is_ok() {
                metrics.inc_topic_from_header();
                return Cow::Owned(topic);
            }
            // Invalid topic from header -- fall back to static config.
            tracing::warn!(
                header_topic = %topic,
                static_topic = signal_config.topic(),
                "invalid Kafka topic from transport header, falling back to static topic"
            );
        }

        // Priority 2: Static per-signal topic (zero-allocation borrow)
        metrics.inc_topic_from_static_config();
        Cow::Borrowed(signal_config.topic())
    }

    /// Looks up the signal-specific header key in transport headers by
    /// normalized name.
    ///
    /// Returns the first matching header's string value, or `None` if the
    /// header key is not configured for this signal or the header is absent.
    fn resolve_from_header(signal_config: &SignalConfig, context: &Context) -> Option<String> {
        let header_key = signal_config.topic_from_transport_header()?;
        let transport_headers = context.transport_headers()?;

        for header in transport_headers.iter() {
            if header.name == *header_key {
                return header.value_as_str().map(String::from);
            }
        }

        None
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
            name: wire_name.to_lowercase().replace('-', "_"),
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

    // ---- Transport header resolution tests ----

    #[test]
    fn test_resolve_header_present() {
        let config = make_signal_config("fallback-logs", Some("x_target_topic"));
        let ctx = context_with_headers(vec![make_transport_header(
            "X-Target-Topic",
            "tenant-a-logs",
        )]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "tenant-a-logs");
        assert!(matches!(topic, Cow::Owned(_)));
        assert_eq!(metrics.topic_from_header.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_header_absent() {
        let config = make_signal_config("fallback-logs", Some("x_target_topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Other-Header", "value")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_header.get(), 0);
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_header_no_transport_headers_on_context() {
        let config = make_signal_config("fallback-logs", Some("x_target_topic"));
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_header_not_configured() {
        let config = make_signal_config("fallback-logs", None);
        let ctx = context_with_headers(vec![make_transport_header("X-Target-Topic", "topic-a")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_header_wins_over_static() {
        let config = make_signal_config("static-topic", Some("x_target_topic"));
        let ctx = context_with_headers(vec![make_transport_header(
            "X-Target-Topic",
            "header-topic",
        )]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "header-topic");
        assert_eq!(metrics.topic_from_header.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_static_path_returns_borrowed() {
        let config = make_signal_config("my-topic", None);
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "my-topic");
        assert!(
            matches!(topic, Cow::Borrowed(_)),
            "static path should return Cow::Borrowed (zero allocation)"
        );
    }

    #[test]
    fn test_resolve_header_path_returns_owned() {
        let config = make_signal_config("fallback", Some("x_topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "dynamic")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "dynamic");
        assert!(
            matches!(topic, Cow::Owned(_)),
            "header path should return Cow::Owned"
        );
    }

    #[test]
    fn test_per_signal_header_keys() {
        let traces_config = make_signal_config("otlp_spans", Some("x_traces_topic"));
        let metrics_config = make_signal_config("otlp_metrics", None);
        let logs_config = make_signal_config("otlp_logs", Some("x_logs_topic"));

        let ctx = context_with_headers(vec![
            make_transport_header("X-Traces-Topic", "custom-traces"),
            make_transport_header("X-Logs-Topic", "custom-logs"),
        ]);
        let mut metrics = KafkaExporterMetrics::default();

        // Traces: header present -> dynamic topic
        let topic = TopicRouter::resolve(&traces_config, &ctx, &mut metrics);
        assert_eq!(&*topic, "custom-traces");

        // Metrics: no header key configured -> static fallback
        let topic = TopicRouter::resolve(&metrics_config, &ctx, &mut metrics);
        assert_eq!(&*topic, "otlp_metrics");

        // Logs: header present -> dynamic topic
        let topic = TopicRouter::resolve(&logs_config, &ctx, &mut metrics);
        assert_eq!(&*topic, "custom-logs");

        assert_eq!(metrics.topic_from_header.get(), 2);
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_per_signal_header_key_absent_falls_back() {
        let config = make_signal_config("fallback-logs", Some("x_logs_topic"));
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-logs");
        assert_eq!(metrics.topic_from_static_config.get(), 1);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    // ---- Invalid header topic falls back to static ----

    #[test]
    fn test_resolve_invalid_header_topic_empty_falls_back() {
        let config = make_signal_config("fallback-topic", Some("x_topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-topic");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_dot_falls_back() {
        let config = make_signal_config("fallback-topic", Some("x_topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", ".")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-topic");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_dotdot_falls_back() {
        let config = make_signal_config("fallback-topic", Some("x_topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "..")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-topic");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_invalid_header_topic_bad_chars_falls_back() {
        let config = make_signal_config("fallback-topic", Some("x_topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "bad topic/name")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-topic");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_too_long_falls_back() {
        let long_topic = "a".repeat(250);
        let config = make_signal_config("fallback-topic", Some("x_topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", &long_topic)]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "fallback-topic");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    #[test]
    fn test_resolve_valid_header_topic_still_works() {
        let config = make_signal_config("fallback-topic", Some("x_topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "valid-topic-123")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert_eq!(&*topic, "valid-topic-123");
        assert!(matches!(topic, Cow::Owned(_)));
        assert_eq!(metrics.topic_from_header.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }
}
