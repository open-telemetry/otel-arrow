// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dynamic topic routing for the Kafka exporter.
//!
//! Resolves the destination Kafka topic for a payload using a priority hierarchy:
//!
//! 1. **Tenant context** (`topic_from_tenant_key` on the per-signal config):
//!    If configured for the signal type and the key holds a value on the
//!    request, that value becomes the topic for the batch. If the value is not
//!    a valid Kafka topic, routing fails with
//!    [`TopicRoutingError::InvalidTenantTopic`] and the batch is permanently
//!    nacked -- it does **not** fall back to the static topic.
//! 2. **Static fallback**: The per-signal `topic` from config, used only when
//!    the key holds no value (or no key is configured).
//!
//! Each signal type can route on a different tenant key (or none), allowing
//! independent dynamic routing per signal.
//!
//! The key name is resolved to a value slot once, when the exporter starts, so
//! routing a batch is an indexed read rather than a name lookup.
//!

// TODO: allow prefix or acl mechanism so the operator can have some control over where these messages wind up (e.g. topic must start with tenant_)
// TODO: Consider adding an operator-controlled restriction (e.g., allowlist, prefix constraint, or regex)

use super::config::SignalConfig;
use super::metrics::KafkaExporterMetrics;
use crate::common::kafka::validate_kafka_topic;
use otap_df_otap::pdata::Context;
use std::borrow::Cow;

/// Error returned when topic routing cannot produce a usable Kafka topic.
#[derive(Debug, thiserror::Error)]
pub enum TopicRoutingError {
    /// A topic was supplied by the tenant context but it failed Kafka topic
    /// validation. This is a non-retryable condition: the same value will
    /// always be invalid, so the exporter permanently nacks the batch rather
    /// than silently rerouting it to the static topic.
    #[error("invalid Kafka topic '{topic}' from tenant context: {reason}")]
    InvalidTenantTopic {
        /// The offending topic value read from the tenant context.
        topic: String,
        /// Human-readable reason the topic failed validation.
        reason: String,
    },
}

impl TopicRoutingError {
    /// Builds a [`TopicRoutingError::InvalidTenantTopic`] and emits the routing
    /// warning once, so all "value present but unusable as a topic" cases
    /// (non-UTF-8 value or failed Kafka topic validation) share a single
    /// construction and log site.
    fn invalid_tenant_topic(topic: impl Into<String>, reason: impl Into<String>) -> Self {
        let topic = topic.into();
        let reason = reason.into();
        otap_df_telemetry::otel_warn!(
            "kafka.exporter.topic.invalid_tenant",
            tenant_topic = %topic,
            %reason,
            "invalid Kafka topic from tenant context, permanently nacking batch"
        );
        Self::InvalidTenantTopic { topic, reason }
    }
}

/// Stateless topic router for the Kafka exporter.
///
/// Resolves the destination Kafka topic by inspecting the per-signal config
/// and the request's tenant context. No fields, no construction, no heap
/// allocation.
///
/// The router increments topic routing metrics (`topic_from_tenant`,
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
    /// If a topic is supplied via the tenant context but is invalid, this
    /// returns [`TopicRoutingError::InvalidTenantTopic`] instead of falling
    /// back to the static topic. The caller is expected to permanently nack the
    /// batch, since rerouting an explicitly-requested-but-invalid topic to the
    /// static topic could silently misdeliver tenant data.
    ///
    /// # Arguments
    ///
    /// * `signal_config` - The per-signal config (carries the static topic)
    /// * `topic_slot` - The value slot the topic is read from, resolved once
    ///   at exporter start; `None` when routing by tenant key is not configured
    /// * `context` - The pdata context (carries the tenant context)
    /// * `metrics` - Exporter metrics to increment topic routing counters
    pub fn resolve<'a>(
        signal_config: &'a SignalConfig,
        topic_slot: Option<u16>,
        context: &Context,
        metrics: &mut KafkaExporterMetrics,
    ) -> Result<Cow<'a, str>, TopicRoutingError> {
        // Priority 1: topic from the tenant context, if configured and present.
        if let Some(value) = Self::tenant_topic(topic_slot, context) {
            // A present routing header must be a usable Kafka topic. If it is
            // not (non-UTF-8 value, or a value that fails Kafka topic
            // validation) this is non-retryable: surface an error so the batch
            // is permanently nacked rather than silently falling back to the
            // static topic, which would misdeliver the data.
            let topic = std::str::from_utf8(value).map_err(|_| {
                TopicRoutingError::invalid_tenant_topic(
                    String::from_utf8_lossy(value),
                    "value is not valid UTF-8",
                )
            })?;
            validate_kafka_topic(topic)
                .map_err(|reason| TopicRoutingError::invalid_tenant_topic(topic, reason))?;

            metrics.inc_topic_from_tenant();
            return Ok(Cow::Owned(topic.to_owned()));
        }

        // Priority 2: static per-signal topic (zero-allocation borrow).
        metrics.inc_topic_from_static_config();
        Ok(Cow::Borrowed(signal_config.topic()))
    }

    /// Returns the value the request carries in this signal's topic slot, or
    /// `None` if routing by tenant key is not configured for the signal or the
    /// request carries no value there.
    ///
    /// The configured key name was resolved to a slot index once at exporter
    /// start, so this is an indexed read rather than the linear scan over
    /// header names it replaces.
    fn tenant_topic(topic_slot: Option<u16>, context: &Context) -> Option<&[u8]> {
        context.tenant_view()?.slot_value(topic_slot?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::kafka::MessageFormat;
    use otap_df_config::tenant::compiled::{
        TenantTokenRegistry, TenantTokenRegistryBuilder, TokenInputs, TokenScratch,
    };
    use otap_df_config::tenant::{Extractor, TenantTokenSpec, TenantTokens};
    use otap_df_otap::pdata::Context;
    use std::sync::Arc;

    // ---- Test helpers ----

    /// Every key these tests route on. Each is read from the wire header of
    /// the same name and retained, so it has a value slot to read.
    const KEYS: [&str; 4] = [
        "x-target-topic",
        "x-topic",
        "x-traces-topic",
        "x-logs-topic",
    ];

    fn registry() -> Arc<TenantTokenRegistry> {
        let mut tokens = TenantTokens::default();
        for key in KEYS {
            let _ = tokens.insert(
                key.to_owned(),
                TenantTokenSpec {
                    extractors: vec![Extractor::TransportHeader {
                        key: key.to_owned(),
                        transport_header: key.to_owned(),
                        retain: true,
                        bag: false,
                    }],
                },
            );
        }
        let mut builder = TenantTokenRegistryBuilder::new();
        builder.add_tokens(&tokens).expect("tokens compile");
        Arc::new(builder.build(1).expect("layout fits"))
    }

    /// Route as the exporter does, with the key name already resolved to a
    /// value slot at start.
    fn route<'a>(
        config: &'a SignalConfig,
        ctx: &Context,
        metrics: &mut KafkaExporterMetrics,
    ) -> Result<Cow<'a, str>, TopicRoutingError> {
        let reg = registry();
        let slot = config
            .topic_from_tenant_key()
            .and_then(|k| reg.key_id(k))
            .and_then(|k| reg.value_slot(k));
        TopicRouter::resolve(config, slot, ctx, metrics)
    }

    fn context_with_headers(pairs: &[(&str, &[u8])]) -> Context {
        let reg = registry();
        let mut scratch = TokenScratch::new();
        let mut ctx = Context::default();
        if let Some(words) = reg.resolve(
            &mut scratch,
            TokenInputs::new(pairs.iter().map(|(k, v)| (*k, *v))),
        ) {
            ctx.set_tenant(words);
        }
        ctx
    }

    fn make_signal_config(topic: &str, key: Option<&str>) -> SignalConfig {
        let config = SignalConfig::new(topic.to_string(), MessageFormat::OtlpProto);
        match key {
            Some(key) => config.with_topic_from_tenant_key(key),
            None => config,
        }
    }

    // ---- Tenant context resolution tests ----

    #[test]
    fn test_resolve_header_present() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = context_with_headers(&[("x-target-topic", b"tenant-a-logs")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "tenant-a-logs");
        assert!(matches!(topic, Cow::Owned(_)));
        assert_eq!(metrics.topic_from_tenant.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_header_absent() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = context_with_headers(&[("x-other-header", b"value")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_tenant.get(), 0);
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_falls_back_when_context_has_no_tenant() {
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_header_not_configured() {
        let config = make_signal_config("fallback-logs", None);
        let ctx = context_with_headers(&[("x-target-topic", b"topic-a")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert!(matches!(topic, Cow::Borrowed(_)));
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_resolve_header_wins_over_static() {
        let config = make_signal_config("static-topic", Some("x-target-topic"));
        let ctx = context_with_headers(&[("x-target-topic", b"header-topic")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "header-topic");
        assert_eq!(metrics.topic_from_tenant.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_static_path_returns_borrowed() {
        let config = make_signal_config("my-topic", None);
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "my-topic");
        assert!(
            matches!(topic, Cow::Borrowed(_)),
            "static path should return Cow::Borrowed (zero allocation)"
        );
    }

    #[test]
    fn test_resolve_header_path_returns_owned() {
        let config = make_signal_config("fallback", Some("x-topic"));
        let ctx = context_with_headers(&[("x-topic", b"dynamic")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("valid topic");
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

        let ctx = context_with_headers(&[
            ("x-traces-topic", b"custom-traces"),
            ("x-logs-topic", b"custom-logs"),
        ]);
        let mut metrics = KafkaExporterMetrics::default();

        // Traces: header present -> dynamic topic
        let topic = route(&traces_config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "custom-traces");

        // Metrics: no header key configured -> static fallback
        let topic = route(&metrics_config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "otlp_metrics");

        // Logs: header present -> dynamic topic
        let topic = route(&logs_config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "custom-logs");

        assert_eq!(metrics.topic_from_tenant.get(), 2);
        assert_eq!(metrics.topic_from_static_config.get(), 1);
    }

    #[test]
    fn test_per_signal_header_key_absent_falls_back() {
        let config = make_signal_config("fallback-logs", Some("x-logs-topic"));
        let ctx = Context::default();
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("static topic");
        assert_eq!(&*topic, "fallback-logs");
        assert_eq!(metrics.topic_from_static_config.get(), 1);
        assert_eq!(metrics.topic_from_tenant.get(), 0);
    }

    // ---- Invalid header topic returns an error (no static fallback) ----

    #[test]
    fn test_resolve_invalid_header_topic_empty_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(&[("x-topic", b"")]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = route(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidTenantTopic { .. })
        ));
        // No fallback to static topic, and no topic routing metric incremented.
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_tenant.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_dot_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(&[("x-topic", b".")]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = route(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidTenantTopic { .. })
        ));
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_tenant.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_dotdot_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(&[("x-topic", b"..")]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = route(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidTenantTopic { .. })
        ));
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_bad_chars_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(&[("x-topic", b"bad topic/name")]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = route(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidTenantTopic { .. })
        ));
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_tenant.get(), 0);
    }

    #[test]
    fn test_resolve_invalid_header_topic_too_long_errors() {
        let long_topic = "a".repeat(250);
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(&[("x-topic", long_topic.as_bytes())]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = route(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidTenantTopic { .. })
        ));
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_tenant.get(), 0);
    }

    #[test]
    fn test_resolve_non_utf8_header_topic_errors() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        // A routing value that is not valid UTF-8. This must be treated as a
        // routing error (permanent nack), not as a missing value that falls
        // back to the static topic.
        let ctx = context_with_headers(&[("x-topic", &[0xff, 0xfe, 0xfd])]);
        let mut metrics = KafkaExporterMetrics::default();

        let result = route(&config, &ctx, &mut metrics);
        assert!(matches!(
            result,
            Err(TopicRoutingError::InvalidTenantTopic { .. })
        ));
        // No fallback to static topic, and no topic routing metric incremented.
        assert_eq!(metrics.topic_from_static_config.get(), 0);
        assert_eq!(metrics.topic_from_tenant.get(), 0);
    }

    #[test]
    fn test_resolve_valid_header_topic_still_works() {
        let config = make_signal_config("fallback-topic", Some("x-topic"));
        let ctx = context_with_headers(&[("x-topic", b"valid-topic-123")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "valid-topic-123");
        assert!(matches!(topic, Cow::Owned(_)));
        assert_eq!(metrics.topic_from_tenant.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }

    #[test]
    fn test_resolve_matches_normalized_config_key_for_mixed_case_header() {
        // A header arriving as `X-Target-Topic` is captured (and normalized) as
        // `x-target-topic`. The config key must be the normalized form for the
        // router to match it -- `KafkaExporterConfig::try_from` produces this
        // form from a natural config like `X-Target-Topic`.
        let config = make_signal_config("fallback-logs", Some("x-target-topic"));
        let ctx = context_with_headers(&[("x-target-topic", b"tenant-a-logs")]);
        let mut metrics = KafkaExporterMetrics::default();

        let topic = route(&config, &ctx, &mut metrics).expect("valid topic");
        assert_eq!(&*topic, "tenant-a-logs");
        assert_eq!(metrics.topic_from_tenant.get(), 1);
        assert_eq!(metrics.topic_from_static_config.get(), 0);
    }
}
