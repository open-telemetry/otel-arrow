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
//!    [`TopicRoutingError::InvalidHeaderTopic`] and the batch is permanently
//!    nacked — it does **not** fall back to the static topic.
//! 2. **Static fallback**: The per-signal `topic` from config, used only when
//!    the configured header is absent (or no header key is configured).
//!
//! Each signal type can use a different transport header key (or none), allowing
//! independent dynamic routing per signal.
//!

// TODO: allow prefix or acl mechanism so the operator can have some control over where these messages wind up (e.g. topic must start with tenant_)
// TODO: Consider adding an operator-controlled restriction (e.g., allowlist, prefix constraint, or regex)

use super::config::SignalConfig;
use super::metrics::KafkaExporterMetrics;
use crate::common::kafka::validate_kafka_topic;
use otap_df_config::transport_headers::TransportHeader;
use otap_df_otap::pdata::Context;
use std::borrow::Cow;

/// Error returned when topic routing cannot produce a usable Kafka topic.
#[derive(Debug, thiserror::Error)]
pub enum TopicRoutingError {
    /// A topic was supplied via a transport header but it failed Kafka topic
    /// validation. This is a non-retryable condition: the same header will
    /// always be invalid, so the exporter permanently nacks the batch rather
    /// than silently rerouting it to the static topic.
    #[error("invalid Kafka topic '{topic}' from transport header: {reason}")]
    InvalidHeaderTopic {
        /// The offending topic value extracted from the transport header.
        topic: String,
        /// Human-readable reason the topic failed validation.
        reason: String,
    },
}

impl TopicRoutingError {
    /// Builds an [`TopicRoutingError::InvalidHeaderTopic`] and emits the routing
    /// warning once, so all "header present but unusable as a topic" cases
    /// (non-UTF-8 value or failed Kafka topic validation) share a single
    /// construction and log site.
    fn invalid_header_topic(topic: impl Into<String>, reason: impl Into<String>) -> Self {
        let topic = topic.into();
        let reason = reason.into();
        tracing::warn!(
            header_topic = %topic,
            %reason,
            "invalid Kafka topic from transport header, permanently nacking batch"
        );
        Self::InvalidHeaderTopic { topic, reason }
    }
}

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
    /// Returns `Ok(Cow::Borrowed)` on the static path (zero allocation, borrows
    /// from `signal_config`) or `Ok(Cow::Owned)` on the header path (one
    /// allocation for the extracted header value).
    ///
    /// If a topic is supplied via a transport header but is invalid, this
    /// returns [`TopicRoutingError::InvalidHeaderTopic`] instead of falling
    /// back to the static topic. The caller is expected to permanently nack the
    /// batch, since rerouting an explicitly-requested-but-invalid topic to the
    /// static topic could silently misdeliver tenant data.
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
    ) -> Result<Cow<'a, str>, TopicRoutingError> {
        // Priority 1: topic from a transport header, if configured and present.
        if let Some(header) = Self::header_topic(signal_config, context) {
            // A present routing header must be a usable Kafka topic. If it is
            // not (non-UTF-8 value, or a value that fails Kafka topic
            // validation) this is non-retryable: surface an error so the batch
            // is permanently nacked rather than silently falling back to the
            // static topic, which would misdeliver the data.
            let topic = header.value_as_str().ok_or_else(|| {
                TopicRoutingError::invalid_header_topic(
                    String::from_utf8_lossy(&header.value),
                    "value is not valid UTF-8",
                )
            })?;
            validate_kafka_topic(topic)
                .map_err(|reason| TopicRoutingError::invalid_header_topic(topic, reason))?;

            metrics.inc_topic_from_header();
            return Ok(Cow::Owned(topic.to_owned()));
        }

        // Priority 2: static per-signal topic (zero-allocation borrow).
        metrics.inc_topic_from_static_config();
        Ok(Cow::Borrowed(signal_config.topic()))
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

    // ---- Transport header resolution tests ----

    #[test]
    fn test_resolve_header_present() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = context_with_headers(vec![make_transport_header(
            "X-Target-Topic",
            "tenant-a-logs",
        )]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "tenant-a-logs");
        assert!(matches!(topic, Cow::Owned(_)));
        assert_eq!(metrics.topic_from_header.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_header_absent() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Other-Header", "value")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_header.get(), 0);
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_header_no_transport_headers_on_context() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_header_not_configured() {
        let config = make_signal_config("fallback-logs", None);
        let ctx = context_with_headers(vec![make_transport_header("X-Target-Topic", "topic-a")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_header_wins_over_static() {
        let config = make_signal_config("static-topic", Some("x-target-topic"));
        let ctx = context_with_headers(vec![make_transport_header(
            "X-Target-Topic",
            "header-topic",
        )]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "header-topic");
        assert_eq!(metrics.topic_from_header.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_static_path_returns_borrowed() {
        let config = make_signal_config("my-topic", None);
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "my-topic");
        assert!(
            matches!(topic, Cow::Borrowed(_)),
            "static path should return Cow::Borrowed (zero allocation)"
        );
    }

    #[test]
    fn test_resolve_header_path_returns_owned() {
        let config = make_signal_config("fallback", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "dynamic")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "dynamic");
        assert!(
            matches!(topic, Cow::Owned(_)),
            "header path should return Cow::Owned"
        );
    }

    #[test]
    fn test_per_signal_header_keys() {
        let traces_config = make_signal_config("otlp_spans", Some("x-traces-topic"));
        let metrics_config = make_signal_config("otlp_metrics", None);
        let logs_config = make_signal_config("otlp_logs", Some("x-logs-topic"));

        let ctx = context_with_headers(vec![
            make_transport_header("X-Traces-Topic", "custom-traces"),
            make_transport_header("X-Logs-Topic", "custom-logs"),
        ]);
        let mut metrics = KafkaExporterMetrics::default();

        // Traces: header present -> dynamic topic
        let topic = TopicRouter::resolve(&traces_config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "custom-traces");

        // Metrics: no header key configured -> static fallback
        let topic =
            TopicRouter::resolve(&metrics_config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "otlp_metrics");

        // Logs: header present -> dynamic topic
        let topic = TopicRouter::resolve(&logs_config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "custom-logs");

        assert_eq!(metrics.topic_from_header.get(), 2);
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_per_signal_header_key_absent_falls_back() {
        let config = make_signal_config("fallback-logs", Some("x-logs-topic"));
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert_eq!(metrics.topic_from_static_config.get(), 1);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    // ---- Invalid header topic returns an error (no static fallback) ----

    #[test]
    fn test_resolve_invalid_header_topic_empty_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "")]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidHeaderTopic { .. })
        ));
        // No fallback to static topic, and no topic routing metric incremented.
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_dot_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", ".")]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidHeaderTopic { .. })
        ));
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_dotdot_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "..")]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidHeaderTopic { .. })
        ));
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_bad_chars_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "bad topic/name")]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidHeaderTopic { .. })
        ));
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_too_long_errors() {
        let long_topic = "a".repeat(250);
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", &long_topic)]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidHeaderTopic { .. })
        ));
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

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
        let mut metrics = KafkaExporterMetrics::default();

        let result = TopicRouter::resolve(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidHeaderTopic { .. })
        ));
        // No fallback to static topic, and no topic routing metric incremented.
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_header.get(), 0);
    }

    #[test]
    fn test_resolve_valid_header_topic_still_works() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(vec![make_transport_header("X-Topic", "valid-topic-123")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "valid-topic-123");
        assert!(matches!(topic, Cow::Owned(_)));
        assert_eq!(metrics.topic_from_header.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

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
        let mut metrics = KafkaExporterMetrics::default();

        let topic = TopicRouter::resolve(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "tenant-a-logs");
        assert_eq!(metrics.topic_from_header.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }
}
