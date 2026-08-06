// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core Kafka exporter implementation.
//!
//! ToDo: Currently only handles one kafka message add a time we should
//! improve the throughput by handling delivery futures

use super::producer::{ExporterFutureProducer, ExporterFutureRecord};

use super::config::{KafkaExporterConfig, SignalConfig};
use super::encoder;
use super::error::KafkaExporterError;
use super::metrics::KafkaExporterMetrics;
use super::partitioner;
use super::topic_router::TopicRouter;
#[cfg(feature = "aws")]
use crate::common::kafka::aws::ProducerClientContext;
#[cfg(feature = "aws")]
use crate::common::kafka::security::build_aws_msk_context;
use crate::common::kafka::{MSG_FORMAT_OTAP, MSG_FORMAT_OTLP, MessageFormat};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::validation::validate_typed_config;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::Producer as PdataProducer;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::FromClientConfigAndContext;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::Producer;
use regex::Regex;
use std::sync::Arc;
use std::time::Duration;

/// Compiles a signal's `allowed_topics_regex` patterns into [`Regex`] values,
/// or returns `None` when the signal configures no patterns (avoiding an
/// empty-vector allocation for the common case).
///
/// Each pattern is compiled exactly as provided by the operator; entries must
/// be valid regular expressions.
///
/// # Errors
///
/// Returns [`KafkaExporterError::ConfigInvalidTopicRegex`] if any pattern fails
/// to compile, naming the `signal` for operator diagnosis.
fn compile_allowed_topic_regexes(
    patterns: &[String],
    signal: &str,
) -> Result<Option<Vec<Regex>>, KafkaExporterError> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut compiled = Vec::with_capacity(patterns.len());
    for pattern in patterns {
        let re = Regex::new(pattern).map_err(|e| KafkaExporterError::ConfigInvalidTopicRegex {
            signal: signal.to_string(),
            pattern: pattern.clone(),
            message: e.to_string(),
        })?;
        compiled.push(re);
    }
    Ok(Some(compiled))
}

/// URN for the Kafka exporter factory registration.
pub const KAFKA_EXPORTER_URN: &str = "urn:otel:exporter:kafka";

/// Trait for reporting Ack/Nack events.
#[async_trait(?Send)]
pub trait AckNackReporter {
    /// Report a successful ack.
    async fn ack(&self, pdata: OtapPdata) -> Result<(), KafkaExporterError>;

    /// Report a transient (retryable) nack with reason. The retry processor
    /// may schedule a retry for this message.
    async fn nack(&self, reason: String, pdata: OtapPdata) -> Result<(), KafkaExporterError>;

    /// Report a permanent (non-retryable) nack with reason. The retry
    /// processor will forward this upstream immediately without scheduling
    /// a retry. Use this for configuration errors or other conditions that
    /// will never resolve on retry.
    async fn nack_permanent(
        &self,
        reason: String,
        pdata: OtapPdata,
    ) -> Result<(), KafkaExporterError>;
}

/// Internal implementation of AckNackReporter using the effect handler.
struct EffectHandlerReporter<'a> {
    effect_handler: &'a EffectHandler<OtapPdata>,
}

impl<'a> EffectHandlerReporter<'a> {
    fn new(effect_handler: &'a EffectHandler<OtapPdata>) -> Self {
        Self { effect_handler }
    }
}

#[async_trait(?Send)]
impl<'a> AckNackReporter for EffectHandlerReporter<'a> {
    async fn ack(&self, pdata: OtapPdata) -> Result<(), KafkaExporterError> {
        self.effect_handler
            .notify_ack(AckMsg::new(pdata))
            .await
            .map_err(|e| KafkaExporterError::Configuration(format!("Failed to send Ack: {e}")))
    }

    async fn nack(&self, reason: String, pdata: OtapPdata) -> Result<(), KafkaExporterError> {
        self.effect_handler
            .notify_nack(NackMsg::new(&reason, pdata))
            .await
            .map_err(|e| KafkaExporterError::Configuration(format!("Failed to send Nack: {e}")))
    }

    async fn nack_permanent(
        &self,
        reason: String,
        pdata: OtapPdata,
    ) -> Result<(), KafkaExporterError> {
        self.effect_handler
            .notify_nack(NackMsg::new_permanent(&reason, pdata))
            .await
            .map_err(|e| {
                KafkaExporterError::Configuration(format!("Failed to send permanent Nack: {e}"))
            })
    }
}

/// Kafka exporter for OpenTelemetry data.
///
/// Exports telemetry data (traces, metrics, logs) to Apache Kafka topics using the rdkafka client.
///
/// Supports dynamic topic routing via transport headers, with a priority
/// hierarchy: transport header > static topic. The static topic is used only
/// when the configured header is absent; a header present with an invalid
/// topic value causes a permanent nack rather than a fallback.
///
/// Error handling follows a "log and continue" policy:
/// - Export failures are logged via the effect handler and recorded in metrics.
/// - The exporter does not currently fail or stop the pipeline on individual export errors.
pub struct KafkaExporter {
    config: KafkaExporterConfig,
    #[cfg(feature = "aws")]
    producer: ExporterFutureProducer<ProducerClientContext>,
    #[cfg(not(feature = "aws"))]
    producer: ExporterFutureProducer<DefaultClientContext>,
    pdata_producer: PdataProducer,
    metrics: KafkaExporterMetrics,
    /// Pre-compiled dynamic-routing allowlist regexes per signal, compiled once
    /// at construction (and rebuilt on reconfigure) so the hot path never
    /// recompiles. `None` when the signal configures no regex patterns, which
    /// avoids allocating an empty vector for the common (unconstrained) case.
    traces_allowed_topics_regex: Option<Vec<Regex>>,
    metrics_allowed_topics_regex: Option<Vec<Regex>>,
    logs_allowed_topics_regex: Option<Vec<Regex>>,
}

/// Factory registration for the Kafka exporter.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static KAFKA_EXPORTER_FACTORY: ExporterFactory<OtapPdata> = ExporterFactory {
    name: KAFKA_EXPORTER_URN,
    create: |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ExporterWrapper::local(
            KafkaExporter::from_config(pipeline_ctx, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    validate_config: validate_typed_config::<KafkaExporterConfig>,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

impl KafkaExporter {
    /// Creates a new Kafka exporter from configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Kafka exporter configuration
    ///
    /// # Returns
    ///
    /// A new Kafka exporter instance, or an error if initialization fails.
    pub fn new(
        pipeline_ctx: PipelineContext,
        config: KafkaExporterConfig,
    ) -> Result<Self, KafkaExporterError> {
        // Warn about producer_config keys that may be overwritten by first-class fields.
        for key in config.overridden_producer_config_keys() {
            otap_df_telemetry::otel_warn!(
                "kafka.exporter.producer_config.overridden_key",
                key = %key,
                "producer_config contains key '{key}' which is also managed by a \
                 first-class config field and may be overwritten",
            );
        }

        let client_config = config.build_client_config();

        // Create the Kafka producer with the appropriate client context.
        #[cfg(feature = "aws")]
        let producer = {
            let producer_context = match build_aws_msk_context(config.auth()) {
                Some(ctx) => ProducerClientContext::AwsMsk(ctx),
                None => ProducerClientContext::Default(DefaultClientContext),
            };
            ExporterFutureProducer::from_config_and_context(&client_config, producer_context)
                .map_err(|e| {
                    KafkaExporterError::Configuration(format!(
                        "Failed to create Kafka producer: {}",
                        e
                    ))
                })?
        };

        #[cfg(not(feature = "aws"))]
        let producer =
            ExporterFutureProducer::from_config_and_context(&client_config, DefaultClientContext)
                .map_err(|e| {
                KafkaExporterError::Configuration(format!("Failed to create Kafka producer: {}", e))
            })?;

        // Pre-compile the per-signal dynamic-routing allowlist regexes once so
        // the hot path never recompiles.
        let (traces_allowed_topics_regex, metrics_allowed_topics_regex, logs_allowed_topics_regex) =
            Self::compile_signal_allowed_regexes(&config)?;

        Ok(Self {
            config,
            producer,
            pdata_producer: PdataProducer::default(),
            metrics: KafkaExporterMetrics::register(&pipeline_ctx),
            traces_allowed_topics_regex,
            metrics_allowed_topics_regex,
            logs_allowed_topics_regex,
        })
    }

    /// Compiles the dynamic-routing allowlist regexes for all three signals
    /// from `config`, returning them in `(traces, metrics, logs)` order.
    #[allow(clippy::type_complexity)]
    fn compile_signal_allowed_regexes(
        config: &KafkaExporterConfig,
    ) -> Result<(Option<Vec<Regex>>, Option<Vec<Regex>>, Option<Vec<Regex>>), KafkaExporterError>
    {
        let traces = match config.traces() {
            Some(s) => compile_allowed_topic_regexes(s.allowed_topics_regex(), "traces")?,
            None => None,
        };
        let metrics = match config.metrics() {
            Some(s) => compile_allowed_topic_regexes(s.allowed_topics_regex(), "metrics")?,
            None => None,
        };
        let logs = match config.logs() {
            Some(s) => compile_allowed_topic_regexes(s.allowed_topics_regex(), "logs")?,
            None => None,
        };
        Ok((traces, metrics, logs))
    }

    /// Selects the pre-compiled allowlist regexes for `signal_type` from the
    /// three per-signal fields, returning `Option<&[Regex]>`.
    fn allowed_topics_regex_for<'a>(
        signal_type: SignalType,
        traces: &'a Option<Vec<Regex>>,
        metrics: &'a Option<Vec<Regex>>,
        logs: &'a Option<Vec<Regex>>,
    ) -> Option<&'a [Regex]> {
        match signal_type {
            SignalType::Traces => traces.as_deref(),
            SignalType::Metrics => metrics.as_deref(),
            SignalType::Logs => logs.as_deref(),
        }
    }

    /// Create a new Kafka exporter from a JSON config value.
    ///
    /// Deserializes the config and delegates to [`KafkaExporter::new`].
    /// Mirrors the receiver's [`KafkaReceiver::from_config`] pattern.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, ConfigError> {
        let config: KafkaExporterConfig =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        KafkaExporter::new(pipeline_ctx, config).map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })
    }

    /// Gets the signal configuration for a given signal type.
    ///
    /// Returns `Err(MissingTopic)` when the signal type has no configuration,
    /// meaning the user did not configure that signal for export.
    fn get_signal_config(
        config: &KafkaExporterConfig,
        signal_type: SignalType,
    ) -> Result<&SignalConfig, KafkaExporterError> {
        match signal_type {
            SignalType::Traces => config.traces(),
            SignalType::Metrics => config.metrics(),
            SignalType::Logs => config.logs(),
        }
        .ok_or(KafkaExporterError::MissingTopic(signal_type))
    }

    /// Builds the Kafka record headers (format header + propagated transport headers).
    ///
    /// The encoding format (`otlp` or `otap`) is always written under the
    /// `format_header_key`. Any propagated transport header with the same
    /// name is skipped to avoid collision.
    fn build_kafka_headers(
        encoding: MessageFormat,
        format_header_key: &str,
        context: &otap_df_otap::pdata::Context,
        effect_handler: Option<&EffectHandler<OtapPdata>>,
    ) -> OwnedHeaders {
        let mut headers = OwnedHeaders::new();

        // Always write the message format header.
        let format_value = match encoding {
            MessageFormat::OtlpProto => MSG_FORMAT_OTLP,
            MessageFormat::OtapProto => MSG_FORMAT_OTAP,
        };
        headers = headers.insert(Header {
            key: format_header_key,
            value: Some(format_value),
        });

        // Propagate transport headers onto the Kafka record if a propagation
        // policy is configured and the pdata context carries transport headers.
        if let Some(policy) = effect_handler.and_then(|eh| eh.propagation_policy()) {
            if let Some(transport_headers) = context.transport_headers() {
                for propagated in policy.propagate(transport_headers) {
                    // Skip propagated headers that collide with the format header.
                    if propagated.header_name == format_header_key {
                        continue;
                    }
                    headers = headers.insert(Header {
                        key: propagated.header_name,
                        value: Some(propagated.value),
                    });
                }
            }
        }

        headers
    }

    /// Exports a single PData message to Kafka with Ack/Nack support.
    ///
    /// Uses the [`TopicRouter`] to resolve the destination topic:
    /// 1. Transport header (highest priority): used when the configured header
    ///    key is present in the pdata context. If the key is present but its
    ///    value is not a valid Kafka topic, the batch is permanently nacked
    ///    (no fallback to the static topic).
    /// 2. Static per-signal topic: fallback used only when the configured header
    ///    key is absent (or no header routing is configured).
    ///
    /// When the exporter's [`EffectHandler`] has a propagation policy and the
    /// pdata context carries transport headers, matching headers are emitted as
    /// Kafka record headers alongside the mandatory `MessageFormat` header.
    ///
    /// Both text and binary transport header values are emitted as-is since
    /// Kafka headers are opaque byte sequences with string keys (unlike gRPC,
    /// which requires a `-bin` suffix convention for binary metadata).
    async fn export_pdata(
        &mut self,
        pdata: OtapPdata,
        reporter: &dyn AckNackReporter,
        effect_handler: Option<&EffectHandler<OtapPdata>>,
    ) -> Result<(), KafkaExporterError> {
        let signal_type = pdata.signal_type();

        // Extract context and payload first so we can nack if config lookup fails.
        let (context, payload) = pdata.into_parts();

        // Look up the per-signal config once. If the signal type is not
        // configured, permanently nack the message (configuration errors
        // will never resolve on retry) and return the error.
        let signal_config = match Self::get_signal_config(&self.config, signal_type) {
            Ok(cfg) => cfg,
            Err(e) => {
                otap_df_telemetry::otel_warn!(
                    "kafka.exporter.signal.unconfigured",
                    signal_type = ?signal_type,
                    error = %e,
                );
                let _ = reporter
                    .nack_permanent(e.to_string(), OtapPdata::new(context, payload))
                    .await;
                return Err(e);
            }
        };

        let encoding = signal_config.encoding();

        // Select the pre-compiled dynamic-routing allowlist regexes for this
        // signal. This borrows a different field than `self.config` /
        // `self.metrics`, so the disjoint borrows below are valid.
        let allowed_regex = Self::allowed_topics_regex_for(
            signal_type,
            &self.traces_allowed_topics_regex,
            &self.metrics_allowed_topics_regex,
            &self.logs_allowed_topics_regex,
        );

        // Resolve topic via the dynamic topic router *before* doing any encoding
        // work. If a transport header supplied an invalid topic,
        // permanently nack the batch
        let topic =
            match TopicRouter::resolve(signal_config, allowed_regex, &context, &mut self.metrics) {
                Ok(t) => t,
                Err(e) => {
                    self.metrics.inc_failed(signal_type);
                    let _ = reporter
                        .nack_permanent(e.to_string(), OtapPdata::new(context, payload))
                        .await;
                    return Err(e);
                }
            };

        let partition_key = partitioner::partition_key_for_signal(signal_config, &context);

        // Build Kafka headers (format header + propagated transport headers)
        let format_header_key = self.config.message_format_header();
        let headers =
            Self::build_kafka_headers(encoding, format_header_key, &context, effect_handler);

        // Encode payload to bytes using the per-signal encoding.
        // This block borrows &mut self.pdata_producer so it must complete
        // before we borrow self.config again for the topic reference below.
        let encode_result = match encoding {
            MessageFormat::OtlpProto => encoder::encode_to_otlp_bytes(payload.clone()),
            MessageFormat::OtapProto => encoder::encode_to_batch_arrow_record_bytes(
                payload.clone(),
                &mut self.pdata_producer,
            ),
        };

        // nack on failed encoding bytes
        let payload_bytes = match encode_result {
            Ok(bytes) => bytes,
            Err(e) => {
                otap_df_telemetry::otel_error!(
                    "kafka.exporter.encode.failed",
                    signal_type = ?signal_type,
                    error = %e,
                );
                self.metrics.inc_failed(signal_type);
                let _ = reporter
                    .nack_permanent(e.to_string(), OtapPdata::new(context, payload))
                    .await;
                return Err(e);
            }
        };

        // Create Kafka record.
        let mut record = ExporterFutureRecord::to(&topic)
            .headers(headers)
            .payload(&payload_bytes);
        // only set the partition key if it isn't none
        if let Some(ref key) = partition_key {
            record = record.key(key);
        }

        // Send to Kafka with timeout. `timeout_ms` is validated to be within
        // (0, 30s] at config time (see `KafkaExporterConfig`), so this await is
        // always bounded and can never block shutdown indefinitely: a `0` would
        // otherwise map to librdkafka's infinite `message.timeout.ms`.
        let timeout = Duration::from_millis(self.config.timeout_ms());
        match self.producer.send(record, timeout).await {
            Ok(_delivery) => {
                self.metrics.inc_exported(signal_type);
                // Ack reporting is best-effort; Kafka send succeeded so don't fail on ack errors
                if let Err(e) = reporter.ack(OtapPdata::new(context, payload)).await {
                    if let Some(eh) = effect_handler {
                        eh.info(&format!(
                            "Failed to report ack for Kafka export (export succeeded): {}",
                            e
                        ))
                        .await;
                    }
                }
                Ok(())
            }
            Err((kafka_err, _original_record)) => {
                self.metrics.inc_failed(signal_type);
                // `topic` may be a client-supplied (header-routed) value, so
                // bound/escape it before logging to avoid log injection.
                otap_df_telemetry::otel_warn!(
                    "kafka.exporter.send.failed",
                    topic = %crate::common::kafka::sanitize_for_log(&topic),
                    signal_type = ?signal_type,
                    error = %kafka_err,
                );
                // Nack reporting is best-effort; don't propagate nack errors since the
                // primary Kafka error is what matters
                if let Err(e) = reporter
                    .nack(kafka_err.to_string(), OtapPdata::new(context, payload))
                    .await
                {
                    if let Some(eh) = effect_handler {
                        eh.info(&format!(
                            "Failed to report nack for Kafka export failure: {}",
                            e
                        ))
                        .await;
                    }
                }
                Err(KafkaExporterError::KafkaError(kafka_err))
            }
        }
    }

    /// Drain in-flight deliveries on shutdown, bounded by `deadline`.
    ///
    /// Flushes the producer so queued messages get one final chance to be
    /// delivered, then purges anything still queued so we never block past the
    /// deadline.
    async fn drain_and_flush(
        &mut self,
        deadline: std::time::Instant,
        effect_handler: &EffectHandler<OtapPdata>,
    ) {
        effect_handler.info("Flushing Kafka producer").await;

        // Flush for the time remaining until the shutdown deadline (saturating
        // at zero if it has already passed), matching the parquet exporter's
        // deadline-bounded shutdown flush.
        let flush_timeout = deadline
            .checked_duration_since(std::time::Instant::now())
            .unwrap_or(Duration::ZERO);

        if let Err(e) = self.producer.flush(flush_timeout) {
            otap_df_telemetry::otel_warn!(
                "kafka.exporter.shutdown.flush_failed",
                error = %e,
            );
            // Flush timed out or failed; purge anything still queued (in-flight
            // and not-yet-queued) so the producer drop does not block. Purged
            // messages trigger their delivery callbacks with a purge error.
            self.producer
                .purge(rdkafka::producer::PurgeConfig::default().queue().inflight());
        }
    }

    /// Applies a live configuration change pushed via
    /// [`NodeControlMsg::Config`].
    ///
    /// Reconfiguration is a build-and-swap of the librdkafka producer: a new
    /// producer is constructed from the incoming config, the old producer is
    /// drained (bounded flush, then purge of anything still queued), and only
    /// then is the new producer swapped in. Records already in flight on the
    /// old producer get one bounded final chance to deliver; anything still
    /// queued after the bound is purged (its delivery callback fires with a
    /// purge error, which the send path reports as a transient nack).
    ///
    /// The flush is bounded by the current (old) config's `timeout_ms`, matching
    /// the per-message delivery bound, so a slow or unavailable broker can never
    /// stall the reconfigure.
    ///
    /// Reconfiguration is best-effort: if the incoming config fails to
    /// deserialize/validate, or the new producer fails to build, the error is
    /// logged and the existing producer keeps running. This mirrors the
    /// reconfiguration posture of sibling nodes (e.g. the condense-attributes
    /// and retry processors), which warn-and-keep rather than failing the node.
    async fn reconfigure(
        &mut self,
        config: serde_json::Value,
        effect_handler: &EffectHandler<OtapPdata>,
    ) {
        // Deserialize and validate the incoming config. On failure, keep the
        // current producer/config running.
        let new_config: KafkaExporterConfig = match serde_json::from_value(config) {
            Ok(cfg) => cfg,
            Err(e) => {
                otap_df_telemetry::otel_warn!(
                    "kafka.exporter.reconfigure_error",
                    error = %e,
                    "ignoring invalid Config; keeping current configuration",
                );
                return;
            }
        };

        // Warn about producer_config keys overridden by first-class fields,
        // matching the startup behavior.
        for key in new_config.overridden_producer_config_keys() {
            otap_df_telemetry::otel_warn!(
                "kafka.exporter.producer_config.overridden_key",
                key = %key,
                "producer_config contains key '{key}' which is also managed by a \
                 first-class config field and may be overwritten",
            );
        }

        // Build the replacement producer before touching the running one, using
        // the appropriate (AWS-gated) client context. On failure, keep the
        // current producer/config running.
        let client_config = new_config.build_client_config();

        #[cfg(feature = "aws")]
        let new_producer_result = {
            let producer_context = match build_aws_msk_context(new_config.auth()) {
                Some(ctx) => ProducerClientContext::AwsMsk(ctx),
                None => ProducerClientContext::Default(DefaultClientContext),
            };
            ExporterFutureProducer::from_config_and_context(&client_config, producer_context)
        };

        #[cfg(not(feature = "aws"))]
        let new_producer_result =
            ExporterFutureProducer::from_config_and_context(&client_config, DefaultClientContext);

        let new_producer = match new_producer_result {
            Ok(producer) => producer,
            Err(e) => {
                otap_df_telemetry::otel_warn!(
                    "kafka.exporter.reconfigure_error",
                    error = %e,
                    "failed to build producer for new config; keeping current configuration",
                );
                return;
            }
        };

        // Recompile the dynamic-routing allowlist regexes for the new config
        // before touching the running one. On failure, keep the current
        // producer/config running.
        let (new_traces_regex, new_metrics_regex, new_logs_regex) =
            match Self::compile_signal_allowed_regexes(&new_config) {
                Ok(regexes) => regexes,
                Err(e) => {
                    otap_df_telemetry::otel_warn!(
                        "kafka.exporter.reconfigure_error",
                        error = %e,
                        "failed to compile allowed_topics_regex for new config; \
                         keeping current configuration",
                    );
                    return;
                }
            };

        effect_handler
            .info("Reconfiguring Kafka exporter: draining old producer before swap")
            .await;

        // Bounded drain of the old producer so in-flight records get a final
        // chance to deliver before we drop it. Bound by the old config's
        // timeout so a slow/unavailable broker cannot stall the swap.
        let flush_timeout = Duration::from_millis(self.config.timeout_ms());
        if let Err(e) = self.producer.flush(flush_timeout) {
            otap_df_telemetry::otel_warn!(
                "kafka.exporter.reconfigure.flush_failed",
                error = %e,
            );
            self.producer
                .purge(rdkafka::producer::PurgeConfig::default().queue().inflight());
        }

        // Swap in the new producer, config, and compiled allowlist regexes.
        // Dropping the old producer joins its poll thread (see
        // ExporterThreadedProducer::drop).
        self.producer = new_producer;
        self.config = new_config;
        self.traces_allowed_topics_regex = new_traces_regex;
        self.metrics_allowed_topics_regex = new_metrics_regex;
        self.logs_allowed_topics_regex = new_logs_regex;

        otap_df_telemetry::otel_info!(
            "kafka.exporter.reconfigured",
            brokers = %self.config.brokers(),
        );
        effect_handler
            .info("Kafka exporter reconfiguration complete")
            .await;
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for KafkaExporter {
    async fn start(
        mut self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        effect_handler
            .info(&format!(
                "Starting Kafka exporter with brokers: {}",
                self.config.brokers()
            ))
            .await;

        // Start periodic telemetry collection so exporter metrics are flushed into
        // the shared registry via CollectTelemetry control messages.
        let timer_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let ack_nack_reporter = EffectHandlerReporter::new(&effect_handler);

        // Main event loop.
        loop {
            match inbox.recv().await? {
                Message::PData(pdata) => {
                    _ = self
                        .export_pdata(pdata, &ack_nack_reporter, Some(&effect_handler))
                        .await;
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    // Flush exporter metrics into the telemetry registry.
                    _ = self.metrics.report(&mut metrics_reporter);
                }
                Message::Control(NodeControlMsg::Ack(_ack)) => {
                    // Track ack receipt without spamming logs
                    self.metrics.inc_ack();
                }
                Message::Control(NodeControlMsg::Nack(nack)) => {
                    // Nack reached end of pipeline, track and log the failure
                    // reason. The reason string can embed client-supplied values
                    // (e.g. a header-routed topic), so bound/escape it.
                    self.metrics.inc_nack();
                    effect_handler
                        .info(&format!(
                            "Kafka exporter: received Nack - {}",
                            crate::common::kafka::sanitize_for_log(&nack.reason)
                        ))
                        .await;
                }
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    effect_handler.info("Shutting down Kafka exporter").await;
                    _ = timer_cancel_handle.cancel().await;

                    // Graceful shutdown: ingress is already closed by the
                    // engine's receiver-first drain, so just drain our in-flight
                    // deliveries by flushing (bounded by `deadline`), then purge
                    // anything still queued so we never block past the deadline.
                    self.drain_and_flush(deadline, &effect_handler).await;

                    effect_handler.info("Kafka exporter stopped").await;
                    return Ok(TerminalState::new(
                        deadline,
                        self.metrics.terminal_snapshots(),
                    ));
                }
                Message::Control(NodeControlMsg::Config { config }) => {
                    // Live reconfiguration: build-and-swap the librdkafka
                    // producer with a bounded drain of the old one. Invalid
                    // configs are logged and ignored (the current producer keeps
                    // running).
                    self.reconfigure(config, &effect_handler).await;
                }
                Message::Control(_) => {
                    // Ignore other control messages
                }
            }
        }
    }
}

#[cfg(any(test, feature = "test-helpers"))]
pub mod test_support {
    //! Helper utilities for testing the Kafka exporter.

    use super::*;
    use crate::exporters::kafka_exporter::config::KafkaExporterConfigBuilder;
    use bytes::Bytes;
    use otap_df_engine::context::ControllerContext;
    use otap_df_otap::pdata::Context;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::sync::{Arc, Mutex};

    /// Creates a deterministic pipeline context for tests.
    #[must_use]
    pub fn pipeline_context() -> PipelineContext {
        let metrics_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        controller_ctx.pipeline_context_with("test-group".into(), "test-pipeline".into(), 0, 1, 0)
    }

    /// Builds a minimal Kafka exporter config builder for tests.
    ///
    /// All three signal types are configured with default encoding.
    /// Returns a [`KafkaExporterConfigBuilder`] so callers can chain
    /// additional `with_*()` methods before converting to a validated
    /// [`KafkaExporterConfig`] via `.try_into().expect(...)`.
    #[must_use]
    pub fn kafka_test_config_builder(brokers: &str) -> KafkaExporterConfigBuilder {
        KafkaExporterConfigBuilder::new(brokers, "test-client")
            .with_traces(SignalConfig::new(
                "test-traces".into(),
                MessageFormat::OtlpProto,
            ))
            .with_metrics(SignalConfig::new(
                "test-metrics".into(),
                MessageFormat::OtlpProto,
            ))
            .with_logs(SignalConfig::new(
                "test-logs".into(),
                MessageFormat::OtlpProto,
            ))
    }

    /// Builds a minimal validated Kafka exporter config for tests.
    ///
    /// All three signal types are configured with default encoding.
    /// Panics if validation fails (should never happen with valid test defaults).
    #[must_use]
    pub fn kafka_test_config(brokers: &str) -> KafkaExporterConfig {
        kafka_test_config_builder(brokers)
            .try_into()
            .expect("test config should be valid")
    }

    /// Produces a small OTLP payload for the requested signal type.
    #[must_use]
    pub fn sample_pdata(signal_type: SignalType) -> OtapPdata {
        let bytes = Bytes::from_static(b"payload");
        let proto = match signal_type {
            SignalType::Traces => otap_df_pdata::OtlpProtoBytes::ExportTracesRequest(bytes.clone()),
            SignalType::Metrics => {
                otap_df_pdata::OtlpProtoBytes::ExportMetricsRequest(bytes.clone())
            }
            SignalType::Logs => otap_df_pdata::OtlpProtoBytes::ExportLogsRequest(bytes),
        };
        OtapPdata::new(Context::default(), proto.into())
    }

    /// Produces a small OTLP payload carrying a single transport header.
    #[must_use]
    pub fn sample_pdata_with_header(
        signal_type: SignalType,
        header_wire_name: &str,
        header_value: &str,
    ) -> OtapPdata {
        use otap_df_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};

        let bytes = Bytes::from_static(b"payload");
        let proto = match signal_type {
            SignalType::Traces => otap_df_pdata::OtlpProtoBytes::ExportTracesRequest(bytes.clone()),
            SignalType::Metrics => {
                otap_df_pdata::OtlpProtoBytes::ExportMetricsRequest(bytes.clone())
            }
            SignalType::Logs => otap_df_pdata::OtlpProtoBytes::ExportLogsRequest(bytes),
        };

        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader {
            name: header_wire_name.to_ascii_lowercase(),
            wire_name: header_wire_name.to_string(),
            value_kind: ValueKind::Text,
            value: header_value.as_bytes().to_vec(),
        });
        let mut context = Context::default();
        context.set_transport_headers(headers);

        OtapPdata::new(context, proto.into())
    }

    /// Recorder that tracks Ack and Nack notifications.
    #[derive(Default)]
    pub struct RecordingReporter {
        acks: Arc<Mutex<usize>>,
        nack_reasons: Arc<Mutex<Vec<String>>>,
        permanent_nack_reasons: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingReporter {
        /// Creates a new reporter.
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        /// Returns the number of Ack notifications.
        #[must_use]
        pub fn ack_count(&self) -> usize {
            *self.acks.lock().unwrap_or_else(|e| {
                panic!(
                    "RecordingReporter: failed to acquire acks lock during ack_count(): {}",
                    e
                )
            })
        }

        /// Returns a copy of the recorded Nack reasons.
        #[must_use]
        pub fn nack_reasons(&self) -> Vec<String> {
            self.nack_reasons
                .lock()
                .unwrap_or_else(|e| {
                    panic!("RecordingReporter: failed to acquire nack_reasons lock during nack_reasons(): {}", e)
                })
                .clone()
        }

        /// Returns a copy of the recorded permanent Nack reasons.
        #[must_use]
        pub fn permanent_nack_reasons(&self) -> Vec<String> {
            self.permanent_nack_reasons
                .lock()
                .unwrap_or_else(|e| {
                    panic!(
                        "RecordingReporter: failed to acquire permanent_nack_reasons lock: {}",
                        e
                    )
                })
                .clone()
        }
    }

    #[async_trait(?Send)]
    impl AckNackReporter for RecordingReporter {
        async fn ack(&self, _pdata: OtapPdata) -> Result<(), KafkaExporterError> {
            *self.acks.lock().unwrap_or_else(|e| {
                panic!(
                    "RecordingReporter: failed to acquire acks lock during ack(): {}",
                    e
                )
            }) += 1;
            Ok(())
        }

        async fn nack(&self, reason: String, _pdata: OtapPdata) -> Result<(), KafkaExporterError> {
            self.nack_reasons
                .lock()
                .unwrap_or_else(|e| {
                    panic!(
                        "RecordingReporter: failed to acquire nack_reasons lock during nack(): {}",
                        e
                    )
                })
                .push(reason);
            Ok(())
        }

        async fn nack_permanent(
            &self,
            reason: String,
            _pdata: OtapPdata,
        ) -> Result<(), KafkaExporterError> {
            self.permanent_nack_reasons
                .lock()
                .unwrap_or_else(|e| {
                    panic!(
                        "RecordingReporter: failed to acquire permanent_nack_reasons lock: {}",
                        e
                    )
                })
                .push(reason);
            Ok(())
        }
    }

    /// Exports a single batch using the provided exporter and reporter.
    pub async fn export_once(
        exporter: &mut KafkaExporter,
        pdata: OtapPdata,
        reporter: &dyn AckNackReporter,
    ) -> Result<(), KafkaExporterError> {
        exporter.export_pdata(pdata, reporter, None).await
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::exporters::kafka_exporter::config::PartitionerStrategy;
        use crate::exporters::kafka_exporter::config::TlsConfig;
        use crate::exporters::kafka_exporter::partitioner::partition_key_from_transport_headers;
        use bytes::Bytes;
        use otap_df_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
        use otap_df_otap::pdata::Context;
        use otap_df_pdata::OtlpProtoBytes;
        use prost::Message as _;
        use std::time::Duration;

        /// Tests that payload is properly cloned for both OTLP and OTAP serialization formats.
        /// This ensures no borrow-after-move errors occur when the encoder consumes the payload.
        #[tokio::test]
        async fn test_export_otlp_format_payload_handling() {
            let pipeline_ctx = pipeline_context();
            let config = kafka_test_config("localhost:9092");
            // logs signal uses OtlpProto by default in kafka_test_config
            let mut exporter =
                KafkaExporter::new(pipeline_ctx, config).expect("config should be valid");

            let reporter = RecordingReporter::new();
            let pdata = sample_pdata(SignalType::Logs);

            // This would fail with borrow-after-move if payload isn't cloned for encoder
            let result = export_once(&mut exporter, pdata, &reporter).await;

            // Expected to fail (no live broker) but should not have compilation/borrow errors
            let _ = result;
        }

        /// Tests that payload is properly cloned for OTAP serialization format.
        #[tokio::test]
        async fn test_export_otap_format_payload_handling() {
            let pipeline_ctx = pipeline_context();
            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("localhost:9092", "test-client")
                    .with_traces(SignalConfig::new(
                        "test-traces".into(),
                        MessageFormat::OtlpProto,
                    ))
                    .with_metrics(SignalConfig::new(
                        "test-metrics".into(),
                        MessageFormat::OtlpProto,
                    ))
                    .with_logs(SignalConfig::new(
                        "test-logs".into(),
                        MessageFormat::OtapProto,
                    ))
                    .try_into()
                    .expect("test config should be valid");
            let mut exporter =
                KafkaExporter::new(pipeline_ctx, config).expect("config should be valid");

            let reporter = RecordingReporter::new();
            let pdata = sample_pdata(SignalType::Logs);

            // This would fail with borrow-after-move if payload isn't cloned for encoder
            let result = export_once(&mut exporter, pdata, &reporter).await;

            // Expected to fail (no live broker) but should not have compilation/borrow errors
            let _ = result;
        }

        // ---- KafkaExporter::new() validation ----

        #[test]
        fn new_succeeds_with_valid_config() {
            let ctx = pipeline_context();
            let config = kafka_test_config("localhost:9092");
            let result = KafkaExporter::new(ctx, config);
            assert!(result.is_ok());
        }

        #[test]
        fn try_from_fails_when_no_signals_configured() {
            let builder = KafkaExporterConfigBuilder::new("localhost:9092", "test-client");
            let result = KafkaExporterConfig::try_from(builder);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                err.contains("at least one signal"),
                "expected signal validation error, got: {err}"
            );
        }

        #[test]
        fn new_with_compression_succeeds() {
            let ctx = pipeline_context();
            let config: KafkaExporterConfig = kafka_test_config_builder("localhost:9092")
                .with_compression(crate::exporters::kafka_exporter::config::CompressionType::Zstd)
                .try_into()
                .expect("config with compression should be valid");
            let result = KafkaExporter::new(ctx, config);
            assert!(result.is_ok());
        }

        #[test]
        fn new_with_tls_config_fails_on_missing_certs() {
            let ctx = pipeline_context();
            let config: KafkaExporterConfig = kafka_test_config_builder("localhost:9092")
                .with_tls(TlsConfig::new(
                    "/nonexistent/ca.pem".into(),
                    "/nonexistent/cert.pem".into(),
                    "/nonexistent/key.pem".into(),
                    None,
                    false,
                ))
                .try_into()
                .expect("config with tls should be valid");
            // rdkafka validates cert paths at create() time, so this should fail
            let result = KafkaExporter::new(ctx, config);
            assert!(result.is_err());
        }

        #[test]
        fn new_with_tls_insecure_fails_on_missing_certs() {
            let ctx = pipeline_context();
            let config: KafkaExporterConfig = kafka_test_config_builder("localhost:9092")
                .with_tls(TlsConfig::new(
                    "/nonexistent/ca.pem".into(),
                    "/nonexistent/cert.pem".into(),
                    "/nonexistent/key.pem".into(),
                    None,
                    true,
                ))
                .try_into()
                .expect("config with insecure tls should be valid");
            // Even with insecure=true, missing cert files cause create() to fail
            let result = KafkaExporter::new(ctx, config);
            assert!(result.is_err());
        }

        // ---- get_signal_config ----

        #[test]
        fn get_signal_config_returns_correct_topics() {
            let ctx = pipeline_context();
            let config = kafka_test_config("localhost:9092");
            let exporter = KafkaExporter::new(ctx, config).unwrap();

            let traces_cfg =
                KafkaExporter::get_signal_config(&exporter.config, SignalType::Traces).unwrap();
            assert_eq!(traces_cfg.topic(), "test-traces");

            let metrics_cfg =
                KafkaExporter::get_signal_config(&exporter.config, SignalType::Metrics).unwrap();
            assert_eq!(metrics_cfg.topic(), "test-metrics");

            let logs_cfg =
                KafkaExporter::get_signal_config(&exporter.config, SignalType::Logs).unwrap();
            assert_eq!(logs_cfg.topic(), "test-logs");
        }

        #[test]
        fn get_signal_config_returns_error_for_unconfigured_signal() {
            let ctx = pipeline_context();
            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("localhost:9092", "test-client")
                    .with_traces(SignalConfig::new(
                        "test-traces".into(),
                        MessageFormat::OtlpProto,
                    ))
                    .with_logs(SignalConfig::new(
                        "test-logs".into(),
                        MessageFormat::OtlpProto,
                    ))
                    .try_into()
                    .expect("config with traces+logs should be valid");
            let exporter = KafkaExporter::new(ctx, config).unwrap();

            assert!(KafkaExporter::get_signal_config(&exporter.config, SignalType::Traces).is_ok());
            assert!(KafkaExporter::get_signal_config(&exporter.config, SignalType::Logs).is_ok());

            let err = KafkaExporter::get_signal_config(&exporter.config, SignalType::Metrics)
                .unwrap_err();
            assert!(matches!(
                err,
                KafkaExporterError::MissingTopic(SignalType::Metrics)
            ));
        }

        // ---- KafkaExporterError Display ----

        #[test]
        fn error_display_configuration() {
            let err = KafkaExporterError::Configuration("bad config".to_string());
            let s = err.to_string();
            assert!(s.contains("bad config"), "got: {s}");
            assert!(s.contains("configuration error"), "got: {s}");
        }

        #[test]
        fn error_display_missing_topic() {
            let err = KafkaExporterError::MissingTopic(SignalType::Logs);
            let s = err.to_string();
            assert!(s.contains("Logs"), "got: {s}");
        }

        // ---- RecordingReporter ----

        #[tokio::test]
        async fn recording_reporter_tracks_acks_and_nacks() {
            let reporter = RecordingReporter::new();
            let pdata = sample_pdata(SignalType::Traces);

            let _ = reporter.ack(pdata.clone()).await;
            let _ = reporter.ack(pdata.clone()).await;
            let _ = reporter.nack("error1".to_string(), pdata.clone()).await;
            let _ = reporter.nack("error2".to_string(), pdata.clone()).await;
            let _ = reporter
                .nack_permanent("permanent-error".to_string(), pdata)
                .await;

            assert_eq!(reporter.ack_count(), 2);
            let reasons = reporter.nack_reasons();
            assert_eq!(reasons.len(), 2);
            assert_eq!(reasons[0], "error1");
            assert_eq!(reasons[1], "error2");
            let permanent_reasons = reporter.permanent_nack_reasons();
            assert_eq!(permanent_reasons.len(), 1);
            assert_eq!(permanent_reasons[0], "permanent-error");
        }

        #[tokio::test]
        async fn test_export_unconfigured_signal_type_is_nacked() {
            let pipeline_ctx = pipeline_context();
            // Only logs configured -- no traces, no metrics
            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("localhost:9092", "test-client")
                    .with_logs(SignalConfig::new(
                        "test-logs".into(),
                        MessageFormat::OtlpProto,
                    ))
                    .try_into()
                    .expect("test config should be valid");
            let mut exporter =
                KafkaExporter::new(pipeline_ctx, config).expect("config should be valid");

            let reporter = RecordingReporter::new();
            let pdata = sample_pdata(SignalType::Traces); // unconfigured signal type

            let result = export_once(&mut exporter, pdata, &reporter).await;
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                KafkaExporterError::MissingTopic(SignalType::Traces)
            ));
            // Verify a permanent nack was reported (not a transient nack)
            assert_eq!(reporter.ack_count(), 0);
            assert_eq!(
                reporter.nack_reasons().len(),
                0,
                "should not use transient nack for configuration errors"
            );
            let permanent_reasons = reporter.permanent_nack_reasons();
            assert_eq!(permanent_reasons.len(), 1);
            assert!(
                permanent_reasons[0].contains("Traces"),
                "permanent nack reason should mention the signal type, got: {}",
                permanent_reasons[0]
            );
        }

        #[tokio::test]
        async fn test_export_invalid_dynamic_topic_is_permanently_nacked() {
            let pipeline_ctx = pipeline_context();
            // Logs configured to resolve their topic from a transport header.
            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("localhost:9092", "test-client")
                    .with_logs(
                        SignalConfig::new("test-logs".into(), MessageFormat::OtlpProto)
                            .with_topic_from_transport_header("x-target-topic"),
                    )
                    .try_into()
                    .expect("test config should be valid");
            let mut exporter =
                KafkaExporter::new(pipeline_ctx, config).expect("config should be valid");

            let reporter = RecordingReporter::new();
            // Header supplies an invalid topic ("bad topic/name" contains a space and slash).
            let pdata =
                sample_pdata_with_header(SignalType::Logs, "X-Target-Topic", "bad topic/name");

            let result = export_once(&mut exporter, pdata, &reporter).await;
            assert!(result.is_err());
            assert!(
                matches!(
                    result.unwrap_err(),
                    KafkaExporterError::InvalidHeaderTopic { .. }
                ),
                "invalid dynamic topic should surface an InvalidHeaderTopic error"
            );

            // Verify a permanent nack was reported (not a transient nack) and the
            // batch was not silently routed to the static topic.
            assert_eq!(reporter.ack_count(), 0);
            assert_eq!(
                reporter.nack_reasons().len(),
                0,
                "should not use transient nack for an invalid dynamic topic"
            );
            let permanent_reasons = reporter.permanent_nack_reasons();
            assert_eq!(permanent_reasons.len(), 1);
            assert!(
                permanent_reasons[0].contains("bad topic/name"),
                "permanent nack reason should mention the offending topic, got: {}",
                permanent_reasons[0]
            );
        }

        // ---- Integration tests (in-process mock Kafka broker) ----
        //
        // These use the shared Kafka test suite (`crate::common::kafka::test`),
        // which wraps `rdkafka::mocking::MockCluster` in an in-process librdkafka
        // mock broker, so the tests run with no Docker/external broker and run by
        // default in CI. Each test drives a fully-wired `KafkaExporter` through
        // the `KafkaExporterHarness` wrapper (which owns the engine wiring,
        // `LocalSet` spawn, and lifecycle), then consumes the produced records
        // back from the mock broker via a test-suite consumer to assert on the
        // topic, payload bytes, message-format header, and partition key.

        use crate::common::kafka::node_harness::KafkaExporterHarness;
        use crate::common::kafka::test::cluster::KafkaTestCluster;
        use crate::common::kafka::test::{run_on_local_set, with_cluster};

        /// Builds an [`ExportLogsServiceRequest`] with a single log record so
        /// tests exercise a real OTLP payload (required for OTAP encoding).
        fn logs_request_bytes() -> Vec<u8> {
            use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
            use otap_df_pdata::proto::opentelemetry::logs::v1::{
                LogRecord, ResourceLogs, ScopeLogs,
            };

            let req = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord {
                            time_unix_nano: 1,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            req.encode_to_vec()
        }

        /// Builds an [`ExportLogsServiceRequest`] whose single log record body
        /// encodes `seq`, so a sequence of these payloads is byte-distinct and
        /// can be checked for delivery order.
        fn logs_request_bytes_seq(seq: usize) -> Vec<u8> {
            use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
            use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, any_value};
            use otap_df_pdata::proto::opentelemetry::logs::v1::{
                LogRecord, ResourceLogs, ScopeLogs,
            };

            let req = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord {
                            time_unix_nano: 1,
                            body: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(format!("seq-{seq}"))),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            req.encode_to_vec()
        }

        /// Wraps OTLP logs bytes into an [`OtapPdata`] that carries a NACKS
        /// subscriber unwind frame, optionally with a single transport header.
        ///
        /// The unwind frame is what makes a nack observable end-to-end: the
        /// real `EffectHandlerReporter` only routes a `PipelineCompletionMsg`
        /// (readable via [`KafkaExporterHarness::recv_nack`]) when the refused
        /// pdata has a subscriber frame. This models the pdata a real upstream
        /// `processor:retry` would have subscribed to before the exporter.
        fn logs_pdata_subscribed(bytes: Vec<u8>, header: Option<(&str, &str)>) -> OtapPdata {
            use otap_df_engine::Interests;
            use otap_df_otap::testing::TestCallData;
            // RETURN_DATA so the refused pdata retains its payload when it
            // unwinds, mirroring a retry processor that must re-send the batch.
            logs_pdata(bytes, header).test_subscribe_to(
                Interests::ACKS_OR_NACKS | Interests::RETURN_DATA,
                TestCallData::default().into(),
                654321,
            )
        }

        /// Wraps OTLP logs bytes into an [`OtapPdata`], optionally carrying a
        /// single transport header.
        fn logs_pdata(bytes: Vec<u8>, header: Option<(&str, &str)>) -> OtapPdata {
            let proto = OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes));
            let mut context = Context::default();
            if let Some((wire_name, value)) = header {
                let mut headers = TransportHeaders::new();
                headers.push(TransportHeader {
                    name: wire_name.to_ascii_lowercase(),
                    wire_name: wire_name.to_string(),
                    value_kind: ValueKind::Text,
                    value: value.as_bytes().to_vec(),
                });
                context.set_transport_headers(headers);
            }
            OtapPdata::new(context, proto.into())
        }

        /// Builds an [`ExportTraceServiceRequest`] with a single span, returned
        /// as OTLP proto bytes wrapped in an [`OtapPdata`].
        fn traces_pdata() -> (OtapPdata, Vec<u8>) {
            use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
            use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};

            let req = ExportTraceServiceRequest {
                resource_spans: vec![ResourceSpans {
                    scope_spans: vec![ScopeSpans {
                        spans: vec![Span {
                            trace_id: vec![1u8; 16],
                            span_id: vec![1u8; 8],
                            name: "span-1".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let bytes = req.encode_to_vec();
            let proto = OtlpProtoBytes::ExportTracesRequest(Bytes::from(bytes.clone()));
            (OtapPdata::new(Context::default(), proto.into()), bytes)
        }

        /// Builds an [`ExportMetricsServiceRequest`] with a single scope,
        /// returned as OTLP proto bytes wrapped in an [`OtapPdata`].
        fn metrics_pdata() -> (OtapPdata, Vec<u8>) {
            use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
            use otap_df_pdata::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};

            let req = ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics {
                    scope_metrics: vec![ScopeMetrics::default()],
                    ..Default::default()
                }],
            };
            let bytes = req.encode_to_vec();
            let proto = OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes.clone()));
            (OtapPdata::new(Context::default(), proto.into()), bytes)
        }

        /// Builds a validated single-signal-logs config bound to `brokers`.
        fn logs_config(brokers: &str, signal: SignalConfig) -> KafkaExporterConfig {
            KafkaExporterConfigBuilder::new(brokers, "it-client")
                .with_logs(signal)
                .try_into()
                .expect("config should be valid")
        }

        /// Scenario: export an OTLP logs batch and read the produced record back
        /// from the mock broker.
        /// Guarantees: the record lands on the configured topic with the exact
        /// payload bytes and an OTLP message-format header.
        #[tokio::test]
        async fn exports_logs_otlp_to_mock_broker() {
            let topic = "it-logs-otlp";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(topic)
                        .assert_payload(&payload)
                        .assert_format_otlp();

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: export an OTLP traces batch to the mock broker.
        /// Guarantees: the traces record lands on the configured topic with the
        /// exact payload bytes and an OTLP message-format header.
        #[tokio::test]
        async fn exports_traces_otlp_to_mock_broker() {
            let topic = "it-traces-otlp";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_traces(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let (pdata, payload) = traces_pdata();
                    exporter.send_pdata(pdata).await.expect("send pdata");

                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(topic)
                        .assert_payload(&payload)
                        .assert_format_otlp();

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: export an OTLP metrics batch to the mock broker.
        /// Guarantees: the metrics record lands on the configured topic with the
        /// exact payload bytes and an OTLP message-format header.
        #[tokio::test]
        async fn exports_metrics_otlp_to_mock_broker() {
            let topic = "it-metrics-otlp";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_metrics(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let (pdata, payload) = metrics_pdata();
                    exporter.send_pdata(pdata).await.expect("send pdata");

                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(topic)
                        .assert_payload(&payload)
                        .assert_format_otlp();

                    exporter.shutdown(Duration::from_millis(500)).await;

                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: export a logs batch configured for OTAP encoding.
        /// Guarantees: the record carries the OTAP message-format header and its
        /// payload decodes as a `BatchArrowRecords` protobuf message.
        #[tokio::test]
        async fn exports_logs_otap_sets_otap_format_header() {
            use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;

            let topic = "it-logs-otap";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtapProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload, None))
                        .await
                        .expect("send pdata");

                    let msg = consumer.recv().await;
                    let _ = msg.assert_topic(topic).assert_format_otap();
                    let decoded =
                        BatchArrowRecords::decode(msg.payload.as_deref().expect("payload"));
                    assert!(
                        decoded.is_ok(),
                        "OTAP payload should decode as BatchArrowRecords"
                    );

                    exporter.shutdown(Duration::from_millis(500)).await;

                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: route a record to a topic named by a transport header while
        /// a different static topic is configured.
        /// Guarantees: the record is produced to the header-specified dynamic
        /// topic (the consumer only subscribes to that topic).
        #[tokio::test]
        async fn routes_to_topic_from_transport_header() {
            let static_topic = "it-static-topic";
            let dynamic_topic = "it-dynamic-topic";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(static_topic)
                    .topic(dynamic_topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[dynamic_topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(static_topic.into(), MessageFormat::OtlpProto)
                            .with_topic_from_transport_header("x-target-topic"),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload, Some(("X-Target-Topic", dynamic_topic))))
                        .await
                        .expect("send pdata");

                    // The consumer only subscribes to the dynamic topic, so
                    // receiving a record proves header-based routing worked.
                    let _ = consumer.recv().await.assert_topic(dynamic_topic);

                    exporter.shutdown(Duration::from_millis(500)).await;

                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: derive the record partition key from transport headers with
        /// a Murmur2Random partitioner.
        /// Guarantees: the produced record's key matches the key computed by
        /// `partition_key_from_transport_headers` for the same headers.
        #[tokio::test]
        async fn sets_partition_key_from_transport_headers() {
            let topic = "it-partition-key";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_logs(
                                SignalConfig::new(topic.into(), MessageFormat::OtlpProto)
                                    .with_partition_by_transport_headers(true),
                            )
                            .with_partitioning_strategy(PartitionerStrategy::Murmur2Random)
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    let pdata = logs_pdata(payload, Some(("X-Tenant-Id", "tenant-123")));
                    let expected_key = {
                        let (context, _payload) = pdata.clone().into_parts();
                        let headers = context
                            .transport_headers()
                            .expect("pdata should carry transport headers");
                        partition_key_from_transport_headers(headers)
                            .expect("headers should produce a partition key")
                    };

                    exporter.send_pdata(pdata).await.expect("send pdata");

                    let _ = consumer.recv().await.assert_key(expected_key.as_bytes());

                    exporter.shutdown(Duration::from_millis(500)).await;

                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: enqueue several records then request a graceful shutdown
        /// with a generous deadline.
        /// Guarantees: all buffered records are flushed and remain consumable
        /// after shutdown (no data loss on graceful stop).
        #[tokio::test]
        async fn shutdown_flushes_buffered_records() {
            let topic = "it-shutdown-flush";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    for _ in 0..3 {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    exporter.shutdown(Duration::from_secs(10)).await;

                    exporter.await_stopped().await;

                    let msgs = consumer.recv_n(3).await;
                    for msg in &msgs {
                        let _ = msg.assert_topic(topic).assert_payload(&payload);
                    }
                },
            )
            .await;
        }

        /// Builds a single-signal-logs reconfiguration JSON payload (the shape
        /// carried by `NodeControlMsg::Config`) targeting `topic` on `brokers`.
        fn logs_reconfig_json(brokers: &str, topic: &str) -> serde_json::Value {
            serde_json::json!({
                "brokers": brokers,
                "client_id": "it-client",
                "logs": { "topic": topic, "encoding": "otlp_proto" },
            })
        }

        /// Scenario: push a `Config` control message that repoints the logs
        /// signal at a different topic, then export a record.
        /// Guarantees: after reconfiguration the exporter produces to the new
        /// topic (build-and-swap of the producer takes effect for later sends).
        #[tokio::test]
        async fn reconfigure_switches_topic() {
            let original_topic = "it-reconfig-original";
            let new_topic = "it-reconfig-new";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(original_topic)
                    .topic(new_topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[new_topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(original_topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Reconfigure to the new topic before sending.
                    exporter
                        .send_config(logs_reconfig_json(cluster.bootstrap_servers(), new_topic))
                        .await;

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    // The consumer only subscribes to the new topic, so
                    // receiving a record proves the reconfigure took effect.
                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(new_topic)
                        .assert_payload(&payload);

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: export a record, confirm it lands on the original topic,
        /// then push a `Config` control message repointing the logs signal at a
        /// new topic and export another record.
        /// Guarantees: records exported before the reconfigure are delivered to
        /// the original topic and records exported after it go to the new topic,
        /// so a live reconfigure neither drops already-accepted data nor retro-
        /// actively reroutes it (the pre-swap drain preserves prior deliveries).
        #[tokio::test]
        async fn reconfigure_flushes_inflight_before_swap() {
            let original_topic = "it-reconfig-flush-original";
            let new_topic = "it-reconfig-flush-new";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(original_topic)
                    .topic(new_topic),
                |cluster| async move {
                    let original_consumer = cluster.consumer().subscribe(&[original_topic]);
                    let new_consumer = cluster.consumer().subscribe(&[new_topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(original_topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();

                    // Record produced before the reconfigure: must land on the
                    // original topic. Consuming it here also confirms the old
                    // producer delivered it prior to the swap.
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata before reconfigure");
                    let _ = original_consumer
                        .recv()
                        .await
                        .assert_topic(original_topic)
                        .assert_payload(&payload);

                    // Reconfigure to the new topic (drains the old producer,
                    // then swaps), then produce again: must land on the new
                    // topic, never the original one.
                    exporter
                        .send_config(logs_reconfig_json(cluster.bootstrap_servers(), new_topic))
                        .await;
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata after reconfigure");
                    let _ = new_consumer
                        .recv()
                        .await
                        .assert_topic(new_topic)
                        .assert_payload(&payload);

                    // The original topic must not receive the post-reconfigure
                    // record.
                    original_consumer
                        .assert_no_more_messages(Duration::from_millis(500))
                        .await;

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: push an invalid `Config` control message (a config with no
        /// signal topics, which fails validation) to a running exporter.
        /// Guarantees: the invalid reconfigure is ignored, the exporter keeps
        /// running on its original config (a later send still lands on the
        /// original topic), and shutdown remains clean.
        #[tokio::test]
        async fn reconfigure_with_invalid_config_keeps_running() {
            let topic = "it-reconfig-invalid";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Invalid: no signal configured, so validation rejects it.
                    exporter
                        .send_config(serde_json::json!({
                            "brokers": cluster.bootstrap_servers(),
                            "client_id": "it-client",
                        }))
                        .await;

                    // The exporter must still be alive on the original config.
                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(topic)
                        .assert_payload(&payload);

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: request a graceful shutdown with a short deadline while the
        /// exporter is pointed at an unreachable broker with a buffered record.
        /// Guarantees: the deadline-bounded drain/flush/purge returns promptly
        /// instead of hanging on the unavailable broker (the shutdown completes
        /// well within a generous outer timeout).
        #[tokio::test]
        async fn shutdown_honors_deadline_when_broker_unavailable() {
            // Point at an unroutable address so every send/flush stalls until
            // the deadline rather than succeeding.
            let cfg = KafkaExporterConfigBuilder::new("127.0.0.1:1", "it-client")
                .with_logs(SignalConfig::new(
                    "it-unavailable".into(),
                    MessageFormat::OtlpProto,
                ))
                // Bound librdkafka delivery so buffered records fail fast.
                .with_timeout_ms(500)
                .try_into()
                .expect("config should be valid");

            run_on_local_set(|cluster| async move {
                let exporter = KafkaExporterHarness::start(&cluster, cfg);

                let payload = logs_request_bytes();
                exporter
                    .send_pdata(logs_pdata(payload, None))
                    .await
                    .expect("send pdata");

                // Short shutdown deadline; the whole stop must finish well
                // inside this outer bound even though the broker is unreachable.
                let start = std::time::Instant::now();
                exporter.shutdown(Duration::from_millis(500)).await;
                tokio::time::timeout(Duration::from_secs(10), exporter.await_stopped())
                    .await
                    .expect("shutdown must not hang past the deadline");
                assert!(
                    start.elapsed() < Duration::from_secs(9),
                    "shutdown took too long against an unavailable broker: {:?}",
                    start.elapsed()
                );
            })
            .await;
        }

        /// Scenario: enqueue many records then request a graceful shutdown with
        /// a generous deadline.
        /// Guarantees: the deadline-bounded drain flushes all buffered records
        /// under sustained load, so none are lost on a graceful stop.
        #[tokio::test]
        async fn shutdown_flushes_under_sustained_traffic() {
            let topic = "it-shutdown-sustained";
            const N: usize = 50;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    for _ in 0..N {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;

                    let msgs = consumer.recv_n(N).await;
                    assert_eq!(msgs.len(), N, "all records should be flushed on shutdown");
                    for msg in &msgs {
                        let _ = msg.assert_topic(topic).assert_payload(&payload);
                    }
                },
            )
            .await;
        }

        // ---- Delivery-semantics tests (section 5) ----
        //
        // ACK/NACK propagation and error classification are asserted at two
        // levels: the fine-grained transient-vs-permanent classification via the
        // in-process `export_once` + `RecordingReporter` path (no broker), and
        // the broker-backed success/failure outcome via the node harness plus
        // the `exporter.kafka` counters read from the terminal state. Ordering,
        // partitioning, timeouts, and broker/network failures are exercised
        // against the in-process mock broker.

        /// Scenario: a successful send to a live mock broker resolves the
        /// delivery callback with success and the exporter reports an ack.
        /// Guarantees: the success path increments the exported counter and
        /// propagates exactly one ack with no nacks (ACK propagation on the
        /// callback-resolved delivery).
        #[tokio::test]
        async fn send_success_reports_ack() {
            let topic = "it-delivery-ack";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let pipeline_ctx = pipeline_context();
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let mut exporter =
                        KafkaExporter::new(pipeline_ctx, cfg).expect("config should be valid");
                    let reporter = RecordingReporter::new();

                    let pdata = logs_pdata(logs_request_bytes(), None);
                    export_once(&mut exporter, pdata, &reporter)
                        .await
                        .expect("send should succeed against the live mock broker");

                    assert_eq!(reporter.ack_count(), 1, "successful send should ack once");
                    assert!(
                        reporter.nack_reasons().is_empty()
                            && reporter.permanent_nack_reasons().is_empty(),
                        "a successful send must not nack"
                    );
                },
            )
            .await;
        }

        /// Scenario: a send whose delivery callback resolves with a Kafka error
        /// (unreachable broker, bounded by a short timeout).
        /// Guarantees: a send failure is classified as a single transient nack
        /// (not permanent) and produces no ack, so the retry processor can
        /// retry it.
        #[tokio::test]
        async fn send_failure_reports_transient_nack() {
            // Unroutable broker so the send fails; bound the wait so the await
            // resolves promptly.
            let pipeline_ctx = pipeline_context();
            let cfg = KafkaExporterConfigBuilder::new("127.0.0.1:1", "it-client")
                .with_logs(SignalConfig::new(
                    "it-delivery-nack".into(),
                    MessageFormat::OtlpProto,
                ))
                .with_timeout_ms(500)
                .try_into()
                .expect("config should be valid");

            run_on_local_set(|_cluster| async move {
                let mut exporter =
                    KafkaExporter::new(pipeline_ctx, cfg).expect("config should be valid");
                let reporter = RecordingReporter::new();

                let pdata = logs_pdata(logs_request_bytes(), None);
                let result = tokio::time::timeout(
                    Duration::from_secs(10),
                    export_once(&mut exporter, pdata, &reporter),
                )
                .await
                .expect("send must resolve within the bounded timeout");
                assert!(result.is_err(), "send to an unreachable broker should fail");

                assert_eq!(
                    reporter.nack_reasons().len(),
                    1,
                    "a send failure should produce exactly one transient nack"
                );
                assert!(
                    reporter.permanent_nack_reasons().is_empty(),
                    "a send failure is transient, never permanent"
                );
                assert_eq!(reporter.ack_count(), 0, "a failed send must not ack");
            })
            .await;
        }

        /// Scenario: export several batches to a live mock broker through the
        /// fully-wired node.
        /// Guarantees: every batch is delivered (readable back from the broker)
        /// and the terminal `logs.exported` counter equals the number of sends
        /// with zero `logs.failed` (ACK-side accounting on success).
        #[tokio::test]
        async fn delivery_success_increments_exported() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            let topic = "it-delivery-exported";
            const N: usize = 5;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    for _ in 0..N {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    let msgs = consumer.recv_n(N).await;
                    assert_eq!(msgs.len(), N);

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        N as u64,
                        "every delivered batch should increment the logs success bucket"
                    );
                    assert_eq!(
                        kafka_exports(snaps, "logs", "failure"),
                        0,
                        "no batch should be counted as failed on the success path"
                    );
                },
            )
            .await;
        }

        /// Scenario: the broker rejects produce requests (injected non-retriable
        /// produce errors) so the delivery callback resolves with a failure.
        /// Guarantees: a broker-reported produce failure increments
        /// `logs.failed` and not `logs.exported`, so the NACK-side accounting
        /// reflects the delivery-callback failure.
        #[tokio::test]
        async fn produce_failure_increments_failed() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            use rdkafka::types::RDKafkaRespErr;
            let topic = "it-delivery-failed";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    // A run of non-retriable produce errors so librdkafka's
                    // internal retries cannot turn this into a success.
                    cluster.faults().fail_produce(&[
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                    ]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send pdata");

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "failure"),
                        1,
                        "a broker-rejected produce should count as one failed batch"
                    );
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        0,
                        "a rejected produce must not be counted as exported"
                    );
                },
            )
            .await;
        }

        /// Scenario: the send targets a broker that never responds (unroutable
        /// address) with a short `timeout_ms`.
        /// Guarantees: the delivery await is bounded by `message.timeout.ms`
        /// (mapped from `timeout_ms`) and resolves as a failure well within a
        /// generous outer bound, so a slow/unavailable broker never hangs the
        /// send loop.
        #[tokio::test]
        async fn send_times_out_within_bound_when_broker_unavailable() {
            let pipeline_ctx = pipeline_context();
            let cfg = KafkaExporterConfigBuilder::new("127.0.0.1:1", "it-client")
                .with_logs(SignalConfig::new(
                    "it-delivery-timeout".into(),
                    MessageFormat::OtlpProto,
                ))
                .with_timeout_ms(500)
                .try_into()
                .expect("config should be valid");

            run_on_local_set(|_cluster| async move {
                let mut exporter =
                    KafkaExporter::new(pipeline_ctx, cfg).expect("config should be valid");
                let reporter = RecordingReporter::new();

                let start = std::time::Instant::now();
                let result = tokio::time::timeout(
                    Duration::from_secs(10),
                    export_once(
                        &mut exporter,
                        logs_pdata(logs_request_bytes(), None),
                        &reporter,
                    ),
                )
                .await
                .expect("send must resolve within the outer bound (delivery is time-bounded)");
                assert!(result.is_err(), "send to an unreachable broker should fail");
                assert!(
                    start.elapsed() < Duration::from_secs(3),
                    "send should resolve near the configured timeout, took {:?}",
                    start.elapsed()
                );
                assert_eq!(
                    reporter.nack_reasons().len(),
                    1,
                    "a timed-out send should produce a single transient nack"
                );
            })
            .await;
        }

        /// Scenario: partitioning by transport headers is enabled and many
        /// records carry the same header set (hence the same partition key) to a
        /// multi-partition topic.
        /// Guarantees: a stable partition key maps every same-key record to a
        /// single partition (key-to-partition stability), and the produced key
        /// matches the documented header-derived key.
        #[tokio::test]
        async fn same_partition_key_maps_to_stable_partition() {
            use crate::common::kafka::test::message::count_by_partition;
            let topic = "it-delivery-stable-key";
            const N: usize = 20;
            with_cluster(
                KafkaTestCluster::builder().topic_with(topic, 4, 1),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    assert!(
                        consumer
                            .wait_for_assignment(4, Duration::from_secs(10))
                            .await,
                        "consumer should be assigned all partitions"
                    );
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(
                            SignalConfig::new(topic.into(), MessageFormat::OtlpProto)
                                .with_partition_by_transport_headers(true),
                        )
                        .with_partitioning_strategy(PartitionerStrategy::Murmur2Random)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let expected_key = {
                        let pdata =
                            logs_pdata(logs_request_bytes(), Some(("X-Tenant-Id", "tenant-42")));
                        let (context, _payload) = pdata.into_parts();
                        partition_key_from_transport_headers(
                            context.transport_headers().expect("headers"),
                        )
                        .expect("headers produce a key")
                    };

                    for _ in 0..N {
                        exporter
                            .send_pdata(logs_pdata(
                                logs_request_bytes(),
                                Some(("X-Tenant-Id", "tenant-42")),
                            ))
                            .await
                            .expect("send pdata");
                    }

                    let msgs = consumer
                        .collect_until_idle(Duration::from_millis(1500))
                        .await;
                    assert_eq!(msgs.len(), N, "all records should be delivered");
                    let dist = count_by_partition(&msgs);
                    assert_eq!(
                        dist.len(),
                        1,
                        "all same-key records must land on a single partition, got {dist:?}"
                    );
                    for msg in &msgs {
                        let _ = msg.assert_key(expected_key.as_bytes());
                    }

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: partitioning is disabled (null record key) and many records
        /// are produced to a multi-partition topic.
        /// Guarantees: null-key records are spread across all partitions in a
        /// near-even distribution (round-robin), and no record carries a key.
        #[tokio::test]
        async fn null_key_distributes_evenly_across_partitions() {
            use crate::common::kafka::test::message::count_by_partition;
            let topic = "it-delivery-roundrobin";
            const PARTS: i32 = 4;
            const N: usize = 80;
            with_cluster(
                KafkaTestCluster::builder().topic_with(topic, PARTS, 1),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    assert!(
                        consumer
                            .wait_for_assignment(PARTS as usize, Duration::from_secs(10))
                            .await,
                        "consumer should be assigned all partitions"
                    );
                    // The `random` partitioner picks a partition per message for
                    // null-key records, so records spread across all partitions.
                    // (The default `consistent_random` picks one partition per
                    // batch and can leave partitions empty over a small sample.)
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_partitioning_strategy(PartitionerStrategy::Random)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    for _ in 0..N {
                        exporter
                            .send_pdata(logs_pdata(logs_request_bytes(), None))
                            .await
                            .expect("send pdata");
                    }

                    let msgs = consumer
                        .collect_until_idle(Duration::from_millis(1500))
                        .await;
                    assert_eq!(msgs.len(), N, "all records should be delivered");
                    for msg in &msgs {
                        let _ = msg.assert_no_key();
                    }

                    let dist = count_by_partition(&msgs);
                    assert_eq!(
                        dist.len(),
                        PARTS as usize,
                        "null-key records should reach every partition, got {dist:?}"
                    );
                    // Near-even: with the random partitioner every partition
                    // should hold at least a quarter of its fair share (fair
                    // share = N / PARTS = 20, so at least 5). This catches a
                    // clustered / non-spreading distribution without depending on
                    // the exact per-run balance.
                    let min_expected = (N / PARTS as usize) / 4;
                    for ((t, p), count) in &dist {
                        assert!(
                            *count >= min_expected,
                            "partition {t}-{p} is under-filled ({count} < {min_expected}); \
                             distribution {dist:?} is not near-even"
                        );
                    }

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: a strictly ordered sequence of distinct payloads is exported
        /// to a single-partition topic.
        /// Guarantees: the serial send loop preserves per-partition order --
        /// records arrive in send order at strictly increasing offsets
        /// (0, 1, 2, ...), so ordering is not reshuffled by the exporter.
        #[tokio::test]
        async fn preserves_per_partition_order() {
            let topic = "it-delivery-order";
            const N: usize = 10;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Distinguish payloads by a per-record log body so we can
                    // assert the delivered order matches the send order.
                    let payloads: Vec<Vec<u8>> = (0..N).map(logs_request_bytes_seq).collect();
                    for payload in &payloads {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    let msgs = consumer.recv_n(N).await;
                    for (i, msg) in msgs.iter().enumerate() {
                        let _ = msg
                            .assert_partition(0)
                            .assert_offset(i as i64)
                            .assert_payload(&payloads[i]);
                    }

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: a broker restart with explicit leader reassignment happens
        /// between two groups of exports on a replicated single-partition topic.
        /// Guarantees: records produced before and after the restart are all
        /// delivered and none already accepted are lost, so the exporter
        /// recovers across a broker restart / leader election.
        #[tokio::test]
        async fn recovers_across_broker_restart_and_leader_reassignment() {
            let topic = "it-delivery-restart";
            with_cluster(
                KafkaTestCluster::builder()
                    .broker_count(3)
                    .topic_with(topic, 1, 3),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(5000)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();

                    // First batch, delivered before the restart.
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata (pre-restart)");
                    let _ = consumer.recv().await.assert_topic(topic);

                    // Restart broker 1, reassigning the partition leader to
                    // broker 2 so the partition stays served (the mock does not
                    // elect leaders on its own).
                    cluster
                        .faults()
                        .restart_broker_reassigning_leader(1, topic, 0, 2);

                    // Second batch, produced after the restart.
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata (post-restart)");

                    // The post-restart record is eventually delivered.
                    let _ = consumer.recv().await.assert_topic(topic);

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;

                    // Broker retained both records on the partition (no loss).
                    let _ = cluster.inspect().assert_message_count_at_least(topic, 0, 2);
                },
            )
            .await;
        }

        /// Scenario: the broker rejects a produce request while the exporter has
        /// a bounded delivery timeout.
        /// Guarantees: a produce failure yields exactly one failed batch bounded
        /// by `timeout_ms` (a transient nack for the retry processor), and on the
        /// in-process mock a rejected produce does not persist -- so no duplicate
        /// is created here.
        #[tokio::test]
        async fn produce_failure_is_bounded_and_not_persisted_on_mock() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            use rdkafka::types::RDKafkaRespErr;
            let topic = "it-delivery-persist";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    cluster.faults().fail_produce(&[
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                    ]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send pdata");
                    // The per-send delivery is bounded by timeout_ms (asserted
                    // in send_times_out_within_bound_when_broker_unavailable); a
                    // short shutdown deadline is enough to collect the terminal
                    // metrics once the send has resolved to a failure.
                    exporter.shutdown(Duration::from_secs(3)).await;
                    let ts = exporter.await_terminal_state().await;

                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "failure"),
                        1,
                        "one bounded failed batch"
                    );
                    assert_eq!(kafka_exports(snaps, "logs", "success"), 0);

                    // On the mock a rejected produce is not persisted, so no
                    // duplicate exists on the partition.
                    assert_eq!(
                        cluster.inspect().message_count(topic, 0),
                        0,
                        "a rejected produce must not persist a record on the mock broker"
                    );
                },
            )
            .await;
        }

        // ---- Security: dynamic-topic routing constraint (section 1) ----

        /// Scenario: a routing header requests a topic that is not permitted by
        /// the signal's operator-configured regex allowlist.
        /// Guarantees: the disallowed header topic is permanently nacked (never
        /// transiently retried) and is not routed to the static topic, so a
        /// client-controlled header cannot direct data to an arbitrary topic.
        #[tokio::test]
        async fn disallowed_dynamic_topic_is_permanently_nacked() {
            let pipeline_ctx = pipeline_context();
            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("localhost:9092", "test-client")
                    .with_logs(
                        SignalConfig::new("static-logs".into(), MessageFormat::OtlpProto)
                            .with_topic_from_transport_header("x-target-topic")
                            .with_allowed_topics_regex(["tenant_.*"]),
                    )
                    .try_into()
                    .expect("config should be valid");
            let mut exporter =
                KafkaExporter::new(pipeline_ctx, config).expect("config should be valid");

            let reporter = RecordingReporter::new();
            // Header requests a syntactically valid but disallowed topic.
            let pdata =
                sample_pdata_with_header(SignalType::Logs, "X-Target-Topic", "evil-destination");

            let result = export_once(&mut exporter, pdata, &reporter).await;
            assert!(result.is_err());
            assert!(
                matches!(
                    result.unwrap_err(),
                    KafkaExporterError::DisallowedHeaderTopic { .. }
                ),
                "a disallowed dynamic topic should surface a DisallowedHeaderTopic error"
            );
            assert_eq!(reporter.ack_count(), 0);
            assert_eq!(
                reporter.nack_reasons().len(),
                0,
                "a disallowed dynamic topic must be permanent, not transient"
            );
            assert_eq!(
                reporter.permanent_nack_reasons().len(),
                1,
                "a disallowed dynamic topic should be permanently nacked"
            );
        }

        /// Scenario: a routing header requests a topic permitted by the regex
        /// allowlist, exported through the fully-wired node to the mock broker.
        /// Guarantees: an allowed header topic is produced to that topic, so the
        /// routing constraint does not block legitimate tenant-scoped routing.
        #[tokio::test]
        async fn allowed_dynamic_topic_is_delivered() {
            let static_topic = "it-sec-static";
            let allowed_topic = "tenant_a_logs";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(static_topic)
                    .topic(allowed_topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[allowed_topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(static_topic.into(), MessageFormat::OtlpProto)
                            .with_topic_from_transport_header("x-target-topic")
                            .with_allowed_topics_regex(["tenant_.*"]),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(
                            payload.clone(),
                            Some(("X-Target-Topic", allowed_topic)),
                        ))
                        .await
                        .expect("send pdata");

                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(allowed_topic)
                        .assert_payload(&payload);

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        // ---- Retry correctness (issue section 3) ----
        //
        // The exporter has no internal retry loop; transient retry is delegated
        // to a separate upstream `processor:retry` node. Its terminal
        // contribution is a Nack classification (transient vs permanent). These
        // tests validate that classification: at the unit level via
        // `RecordingReporter`, and end-to-end via the real
        // `EffectHandlerReporter` by reading the routed `PipelineCompletionMsg`
        // off the harness completion channel with `recv_nack`, then acting as a
        // stand-in retry processor (re-send `refused` on a transient nack, drop
        // on a permanent one).

        /// Builds OTLP logs bytes with one attribute whose value is an array
        /// containing a string element with invalid UTF-8 bytes. The OTLP byte
        /// views tolerate the raw string, but the OTAP conversion CBOR-encodes
        /// array elements and validates UTF-8, so the conversion fails
        /// deterministically.
        fn logs_request_bytes_invalid_utf8_array() -> Vec<u8> {
            use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
            use otap_df_pdata::proto::opentelemetry::common::v1::{
                AnyValue, ArrayValue, KeyValue, any_value,
            };
            use otap_df_pdata::proto::opentelemetry::logs::v1::{
                LogRecord, ResourceLogs, ScopeLogs,
            };

            // A unique marker whose interior bytes we overwrite with 0xFF after
            // encoding (prost `String` cannot hold invalid UTF-8 directly).
            let marker = "\u{0001}MARKER\u{0001}";
            let req = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord {
                            attributes: vec![KeyValue {
                                key: "k".to_string(),
                                value: Some(AnyValue {
                                    value: Some(any_value::Value::ArrayValue(ArrayValue {
                                        values: vec![AnyValue {
                                            value: Some(any_value::Value::StringValue(
                                                marker.to_string(),
                                            )),
                                        }],
                                    })),
                                }),
                            }],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let mut bytes = req.encode_to_vec();
            corrupt_marker_bytes(&mut bytes, marker);
            bytes
        }

        /// Builds an [`ExportTraceServiceRequest`] whose single span carries an
        /// array attribute nesting invalid UTF-8, so the OTAP conversion fails
        /// deterministically (the traces analogue of
        /// [`logs_request_bytes_invalid_utf8_array`]).
        fn traces_request_bytes_invalid_utf8_array() -> Vec<u8> {
            use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
            use otap_df_pdata::proto::opentelemetry::common::v1::{
                AnyValue, ArrayValue, KeyValue, any_value,
            };
            use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};

            let marker = "\u{0001}MARKER\u{0001}";
            let req = ExportTraceServiceRequest {
                resource_spans: vec![ResourceSpans {
                    scope_spans: vec![ScopeSpans {
                        spans: vec![Span {
                            name: "span-1".to_string(),
                            attributes: vec![KeyValue {
                                key: "k".to_string(),
                                value: Some(AnyValue {
                                    value: Some(any_value::Value::ArrayValue(ArrayValue {
                                        values: vec![AnyValue {
                                            value: Some(any_value::Value::StringValue(
                                                marker.to_string(),
                                            )),
                                        }],
                                    })),
                                }),
                            }],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let mut bytes = req.encode_to_vec();
            corrupt_marker_bytes(&mut bytes, marker);
            bytes
        }

        /// Builds an [`ExportMetricsServiceRequest`] whose single gauge data
        /// point carries an array attribute nesting invalid UTF-8, so the OTAP
        /// conversion fails deterministically (the metrics analogue of
        /// [`logs_request_bytes_invalid_utf8_array`]).
        fn metrics_request_bytes_invalid_utf8_array() -> Vec<u8> {
            use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
            use otap_df_pdata::proto::opentelemetry::common::v1::{
                AnyValue, ArrayValue, KeyValue, any_value,
            };
            use otap_df_pdata::proto::opentelemetry::metrics::v1::{
                Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric,
            };

            let marker = "\u{0001}MARKER\u{0001}";
            let req = ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics {
                    scope_metrics: vec![ScopeMetrics {
                        metrics: vec![Metric {
                            name: "m1".to_string(),
                            data: Some(metric::Data::Gauge(Gauge {
                                data_points: vec![NumberDataPoint {
                                    attributes: vec![KeyValue {
                                        key: "k".to_string(),
                                        value: Some(AnyValue {
                                            value: Some(any_value::Value::ArrayValue(ArrayValue {
                                                values: vec![AnyValue {
                                                    value: Some(any_value::Value::StringValue(
                                                        marker.to_string(),
                                                    )),
                                                }],
                                            })),
                                        }),
                                    }],
                                    ..Default::default()
                                }],
                            })),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let mut bytes = req.encode_to_vec();
            corrupt_marker_bytes(&mut bytes, marker);
            bytes
        }

        /// Overwrites the interior bytes of `marker` (found in `bytes`) with
        /// `0xFF`, turning the nested string into invalid UTF-8. prost cannot
        /// hold invalid UTF-8 in a `String`, so the corruption is applied after
        /// encoding.
        fn corrupt_marker_bytes(bytes: &mut [u8], marker: &str) {
            let pos = bytes
                .windows(marker.len())
                .position(|w| w == marker.as_bytes())
                .expect("marker present in encoded bytes");
            for b in &mut bytes[pos + 1..pos + marker.len() - 1] {
                *b = 0xFF;
            }
        }

        /// Scenario: an OTAP-encoded signal whose OTLP bytes cannot be converted
        /// to `OtapArrowRecords` fails encoding before any send.
        /// Guarantees: an encoding failure is classified as a single permanent
        /// nack (never transient, no ack), so the retry processor drops it at
        /// the source rather than retrying an error that can never resolve.
        #[tokio::test]
        async fn encoding_failure_is_permanently_nacked() {
            let pipeline_ctx = pipeline_context();
            // Logs on the OTAP wire format: export must convert the OTLP bytes
            // into OtapArrowRecords. The payload nests an invalid-UTF-8 string
            // inside an array attribute, so the OTAP conversion fails
            // deterministically before any broker send.
            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("localhost:9092", "test-client")
                    .with_logs(SignalConfig::new(
                        "test-logs".into(),
                        MessageFormat::OtapProto,
                    ))
                    .try_into()
                    .expect("config should be valid");
            let mut exporter =
                KafkaExporter::new(pipeline_ctx, config).expect("config should be valid");

            let reporter = RecordingReporter::new();
            let pdata = logs_pdata(logs_request_bytes_invalid_utf8_array(), None);

            let result = export_once(&mut exporter, pdata, &reporter).await;
            assert!(result.is_err(), "malformed OTAP encoding should error");

            assert_eq!(reporter.ack_count(), 0, "a failed encode must not ack");
            assert_eq!(
                reporter.nack_reasons().len(),
                0,
                "an encoding failure is permanent, never transient"
            );
            assert_eq!(
                reporter.permanent_nack_reasons().len(),
                1,
                "an encoding failure should produce exactly one permanent nack"
            );
        }

        /// Scenario: a send to an unreachable broker fails; the refused batch
        /// carries a subscriber frame so its nack unwinds through the real
        /// effect handler.
        /// Guarantees: a send failure reaches the retry processor as a
        /// non-permanent (retryable) nack carrying the refused pdata, so an
        /// upstream `processor:retry` can schedule a retry.
        #[tokio::test]
        async fn transient_nack_reaches_retry_on_send_failure() {
            let cfg: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("127.0.0.1:1", "it-client")
                    .with_logs(SignalConfig::new(
                        "it-retry-transient".into(),
                        MessageFormat::OtlpProto,
                    ))
                    .with_timeout_ms(500)
                    .try_into()
                    .expect("config should be valid");

            run_on_local_set(|cluster| async move {
                let mut exporter = KafkaExporterHarness::start(&cluster, cfg);

                exporter
                    .send_pdata(logs_pdata_subscribed(logs_request_bytes(), None))
                    .await
                    .expect("send pdata");

                let nack = exporter
                    .recv_nack(Duration::from_secs(10))
                    .await
                    .expect("send failure must unwind a nack to the subscriber");
                assert!(
                    !nack.permanent,
                    "a send failure must be a retryable (transient) nack"
                );
                assert!(
                    nack.refused.num_items() >= 1,
                    "the refused pdata (with its records) is returned for the retry processor"
                );

                exporter.shutdown(Duration::from_millis(500)).await;
                exporter.await_stopped().await;
            })
            .await;
        }

        /// Scenario: a header requests a topic outside the regex allowlist; the
        /// refused batch carries a subscriber frame.
        /// Guarantees: a disallowed dynamic topic reaches the retry processor as
        /// a permanent nack, so the retry processor forwards it upstream
        /// immediately instead of retrying an error that can never resolve.
        #[tokio::test]
        async fn permanent_nack_reaches_retry_on_disallowed_topic() {
            let static_topic = "it-retry-static";
            with_cluster(
                KafkaTestCluster::builder().topic(static_topic),
                |cluster| async move {
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(static_topic.into(), MessageFormat::OtlpProto)
                            .with_topic_from_transport_header("x-target-topic")
                            .with_allowed_topics_regex(["tenant_.*"]),
                    );
                    let mut exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_pdata(logs_pdata_subscribed(
                            logs_request_bytes(),
                            Some(("X-Target-Topic", "evil-destination")),
                        ))
                        .await
                        .expect("send pdata");

                    let nack = exporter
                        .recv_nack(Duration::from_secs(10))
                        .await
                        .expect("disallowed topic must unwind a nack to the subscriber");
                    assert!(
                        nack.permanent,
                        "a disallowed dynamic topic must be a permanent nack"
                    );

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: a stand-in retry processor re-sends the refused batch on
        /// each transient nack; the mock broker rejects the first few produce
        /// requests, then accepts.
        /// Guarantees: transient nacks are retryable and a retried batch
        /// eventually delivers once the broker recovers, with no data loss --
        /// the out-of-process retry contract holds end-to-end.
        #[tokio::test]
        async fn transient_nack_retried_until_success_then_acked() {
            use rdkafka::types::RDKafkaRespErr;
            let topic = "it-retry-until-success";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    // Reject the first two produce requests with a non-retriable
                    // error so librdkafka surfaces a delivery failure (a
                    // transient nack) rather than retrying internally; the third
                    // produce request succeeds.
                    cluster.faults().fail_produce(&[
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                        RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION,
                    ]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let mut exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    // Initial send by the (simulated) upstream.
                    exporter
                        .send_pdata(logs_pdata_subscribed(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    // Stand-in retry loop re-send the refused batch on each
                    // transient nack, up to a bounded number of attempts.
                    let mut retries = 0usize;
                    const MAX_RETRIES: usize = 5;
                    while let Some(nack) = exporter.recv_nack(Duration::from_secs(5)).await {
                        assert!(!nack.permanent, "produce rejection should be transient");
                        retries += 1;
                        assert!(retries <= MAX_RETRIES, "retry attempts must stay bounded");
                        exporter
                            .send_pdata(logs_pdata_subscribed(payload.clone(), None))
                            .await
                            .expect("retry send pdata");
                    }

                    // After the injected failures clear, a retry delivers.
                    let msg = consumer
                        .try_recv(Duration::from_secs(10))
                        .await
                        .expect("a retried batch must eventually be delivered");
                    let _ = msg.assert_topic(topic).assert_payload(&payload);
                    assert!(retries >= 1, "at least one transient retry should occur");

                    exporter.shutdown(Duration::from_secs(1)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: a stand-in retry processor treats a permanent nack as
        /// terminal and does not re-send.
        /// Guarantees: a permanently-nacked batch is dropped at the source (no
        /// re-send, nothing produced, no dead-letter queue) and counts as one
        /// failed batch -- retry exhaustion / drop-at-source behavior.
        #[tokio::test]
        async fn permanent_nack_is_not_retried() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            let static_topic = "it-retry-drop-static";
            let disallowed = "evil-destination";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(static_topic)
                    .topic(disallowed),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[disallowed]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(static_topic.into(), MessageFormat::OtlpProto)
                            .with_topic_from_transport_header("x-target-topic")
                            .with_allowed_topics_regex(["tenant_.*"]),
                    );
                    let mut exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_pdata(logs_pdata_subscribed(
                            logs_request_bytes(),
                            Some(("X-Target-Topic", disallowed)),
                        ))
                        .await
                        .expect("send pdata");

                    let nack = exporter
                        .recv_nack(Duration::from_secs(10))
                        .await
                        .expect("disallowed topic must nack");
                    assert!(nack.permanent, "must be permanent so it is not retried");
                    // Stand-in retry processor drops a permanent nack: no re-send.

                    // Nothing should ever be produced to the disallowed topic.
                    assert!(
                        consumer.try_recv(Duration::from_secs(1)).await.is_none(),
                        "a permanently-nacked batch must not reach any broker topic"
                    );

                    exporter.shutdown(Duration::from_secs(1)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "failure"),
                        1,
                        "the dropped batch should count as exactly one failed batch"
                    );
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        0,
                        "a permanently-nacked batch is never exported"
                    );
                },
            )
            .await;
        }

        /// Scenario: a stand-in retry processor re-sends on each transient nack
        /// while the broker rejects a bounded number of produce requests, then
        /// accepts every subsequent attempt.
        /// Guarantees: retry redelivery is bounded by the number of retries the
        /// processor performs (at-least-once), characterizing the duplicate
        /// window as bounded rather than unbounded.
        #[tokio::test]
        async fn retried_transient_send_duplicates_bounded() {
            use rdkafka::types::RDKafkaRespErr;
            let topic = "it-retry-dup-bounded";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    // Reject exactly one produce request, then accept.
                    cluster
                        .faults()
                        .fail_produce(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let mut exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata_subscribed(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    let mut retries = 0usize;
                    const MAX_RETRIES: usize = 3;
                    while let Some(nack) = exporter.recv_nack(Duration::from_secs(5)).await {
                        assert!(!nack.permanent);
                        retries += 1;
                        assert!(retries <= MAX_RETRIES, "retries must stay bounded");
                        exporter
                            .send_pdata(logs_pdata_subscribed(payload.clone(), None))
                            .await
                            .expect("retry send pdata");
                    }

                    // The successful retry must deliver at least one copy.
                    let mut delivered = 0usize;
                    if consumer.try_recv(Duration::from_secs(10)).await.is_some() {
                        delivered += 1;
                    }
                    // Drain any additional copies within a bounded window;
                    // redelivery is bounded by the number of attempts.
                    while consumer.try_recv(Duration::from_secs(2)).await.is_some() {
                        delivered += 1;
                        assert!(
                            delivered <= retries + 1,
                            "delivered copies must be bounded by attempts"
                        );
                    }
                    assert_eq!(retries, 1, "exactly one produce rejection was injected");
                    assert_eq!(
                        delivered, 1,
                        "a not-persisted rejection then a clean retry delivers exactly one copy"
                    );

                    exporter.shutdown(Duration::from_secs(1)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        // ---- Kafka integration: encodings (section 6) ----

        /// Scenario: export a traces batch configured for OTAP encoding.
        /// Guarantees: the traces record carries the OTAP message-format header
        /// and its payload decodes as a `BatchArrowRecords` protobuf message, so
        /// OTAP encoding is validated for traces (not just logs).
        #[tokio::test]
        async fn exports_traces_otap_sets_otap_format_header() {
            use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;

            let topic = "it-traces-otap";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_traces(SignalConfig::new(topic.into(), MessageFormat::OtapProto))
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let (pdata, _payload) = traces_pdata();
                    exporter.send_pdata(pdata).await.expect("send pdata");

                    let msg = consumer.recv().await;
                    let _ = msg.assert_topic(topic).assert_format_otap();
                    let decoded =
                        BatchArrowRecords::decode(msg.payload.as_deref().expect("payload"));
                    assert!(
                        decoded.is_ok(),
                        "OTAP traces payload should decode as BatchArrowRecords"
                    );

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: export a metrics batch configured for OTAP encoding.
        /// Guarantees: the metrics record carries the OTAP message-format header
        /// and its payload decodes as a `BatchArrowRecords` protobuf message, so
        /// OTAP encoding is validated for metrics (not just logs).
        #[tokio::test]
        async fn exports_metrics_otap_sets_otap_format_header() {
            use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;

            let topic = "it-metrics-otap";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_metrics(SignalConfig::new(topic.into(), MessageFormat::OtapProto))
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let (pdata, _payload) = metrics_pdata();
                    exporter.send_pdata(pdata).await.expect("send pdata");

                    let msg = consumer.recv().await;
                    let _ = msg.assert_topic(topic).assert_format_otap();
                    let decoded =
                        BatchArrowRecords::decode(msg.payload.as_deref().expect("payload"));
                    assert!(
                        decoded.is_ok(),
                        "OTAP metrics payload should decode as BatchArrowRecords"
                    );

                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario: export one OTLP-encoded and one OTAP-encoded logs batch to
        /// separate topics and read the broker's partition watermarks.
        /// Guarantees: each successfully-sent record is durably persisted on its
        /// partition (`high - low == 1`), so delivery is confirmed against the
        /// broker's stored state, not just a consumer read-back, for both
        /// encodings.
        #[tokio::test]
        async fn otlp_and_otap_payloads_persist_on_broker() {
            let otlp_topic = "it-persist-otlp";
            let otap_topic = "it-persist-otap";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(otlp_topic)
                    .topic(otap_topic),
                |cluster| async move {
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_logs(SignalConfig::new(
                                otlp_topic.into(),
                                MessageFormat::OtlpProto,
                            ))
                            .try_into()
                            .expect("config should be valid");
                    let otlp_exporter = KafkaExporterHarness::start(&cluster, cfg);
                    otlp_exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send otlp pdata");
                    otlp_exporter.shutdown(Duration::from_secs(5)).await;
                    otlp_exporter.await_stopped().await;

                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_logs(SignalConfig::new(
                                otap_topic.into(),
                                MessageFormat::OtapProto,
                            ))
                            .try_into()
                            .expect("config should be valid");
                    let otap_exporter = KafkaExporterHarness::start(&cluster, cfg);
                    otap_exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send otap pdata");
                    otap_exporter.shutdown(Duration::from_secs(5)).await;
                    otap_exporter.await_stopped().await;

                    // Both records are durably retained on their partitions.
                    let _ = cluster
                        .inspect()
                        .assert_message_count(otlp_topic, 0, 1)
                        .assert_message_count(otap_topic, 0, 1);
                },
            )
            .await;
        }

        /// Scenario: the broker rejects a logs produce request (injected
        /// non-retriable errors) so the delivery callback resolves with a
        /// failure.
        /// Guarantees: a broker-reported produce failure increments
        /// `logs.failed` and not `logs.exported`, so the failure path is
        /// accounted for the logs signal.
        #[tokio::test]
        async fn logs_send_failure_increments_failed() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            use rdkafka::types::RDKafkaRespErr;
            let topic = "it-fail-logs";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    cluster
                        .faults()
                        .fail_produce(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION; 8]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send pdata");

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "failure"),
                        1,
                        "one failed logs batch"
                    );
                    assert_eq!(kafka_exports(snaps, "logs", "success"), 0);
                },
            )
            .await;
        }

        /// Scenario: the broker rejects a traces produce request (injected
        /// non-retriable errors) so the delivery callback resolves with a
        /// failure.
        /// Guarantees: a broker-reported produce failure increments
        /// `traces.failed` and not `traces.exported`, so the failure path is
        /// accounted for the traces signal.
        #[tokio::test]
        async fn traces_send_failure_increments_failed() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            use rdkafka::types::RDKafkaRespErr;
            let topic = "it-fail-traces";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    cluster
                        .faults()
                        .fail_produce(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION; 8]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_traces(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let (pdata, _payload) = traces_pdata();
                    exporter.send_pdata(pdata).await.expect("send pdata");

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "traces", "failure"),
                        1,
                        "one failed traces batch"
                    );
                    assert_eq!(kafka_exports(snaps, "traces", "success"), 0);
                },
            )
            .await;
        }

        /// Scenario: the broker rejects a metrics produce request (injected
        /// non-retriable errors) so the delivery callback resolves with a
        /// failure.
        /// Guarantees: a broker-reported produce failure increments
        /// `metrics.failed` and not `metrics.exported`, so the failure path is
        /// accounted for the metrics signal.
        #[tokio::test]
        async fn metrics_send_failure_increments_failed() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            use rdkafka::types::RDKafkaRespErr;
            let topic = "it-fail-metrics";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    cluster
                        .faults()
                        .fail_produce(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION; 8]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_metrics(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let (pdata, _payload) = metrics_pdata();
                    exporter.send_pdata(pdata).await.expect("send pdata");

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "metrics", "failure"),
                        1,
                        "one failed metrics batch"
                    );
                    assert_eq!(kafka_exports(snaps, "metrics", "success"), 0);
                },
            )
            .await;
        }

        /// Scenario: a caller configures the not-yet-implemented `otlp_json`
        /// message format via config deserialization.
        /// Guarantees: `MessageFormat` accepts `otlp_proto` and `otap_proto` but
        /// rejects `otlp_json`, pinning the documented gap that OTLP JSON
        /// encoding is not available (so a silent partial rollout cannot slip
        /// in an unhandled format).
        #[test]
        fn otlp_json_message_format_is_unavailable() {
            assert!(
                serde_json::from_str::<MessageFormat>("\"otlp_proto\"").is_ok(),
                "otlp_proto must be a valid message format"
            );
            assert!(
                serde_json::from_str::<MessageFormat>("\"otap_proto\"").is_ok(),
                "otap_proto must be a valid message format"
            );
            assert!(
                serde_json::from_str::<MessageFormat>("\"otlp_json\"").is_err(),
                "otlp_json is not implemented and must be rejected"
            );
        }

        // ---- Kafka integration: acknowledgements (section 6) ----

        /// Scenario: export a logs batch with `required_acks = All` (maps to
        /// `request.required.acks = -1`, leader plus all in-sync replicas).
        /// Guarantees: the record is delivered and read back intact and the
        /// batch counts as exactly one `logs_exported`, so the exporter's
        /// delivery path works end-to-end under the strongest ack setting.
        #[tokio::test]
        async fn exports_logs_with_acks_all_round_trips() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            use crate::exporters::kafka_exporter::config::RequiredAcks;
            let topic = "it-acks-all";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_required_acks(RequiredAcks::All)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(topic)
                        .assert_payload(&payload)
                        .assert_format_otlp();

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        1,
                        "one exported batch under acks=all"
                    );
                    assert_eq!(kafka_exports(snaps, "logs", "failure"), 0);
                },
            )
            .await;
        }

        /// Scenario: export a logs batch with `required_acks = None` (maps to
        /// `request.required.acks = 0`, fire-and-forget with no broker ack).
        /// Guarantees: the exporter's delivery callback still resolves so the
        /// record is delivered and read back intact and counts as one
        /// `logs_exported`, so acks=0 does not break the ack accounting or lose
        /// the record.
        #[tokio::test]
        async fn exports_logs_with_acks_none_round_trips() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            use crate::exporters::kafka_exporter::config::RequiredAcks;
            let topic = "it-acks-none";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_required_acks(RequiredAcks::None)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(topic)
                        .assert_payload(&payload)
                        .assert_format_otlp();

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        1,
                        "one exported batch even under fire-and-forget acks=0"
                    );
                    assert_eq!(kafka_exports(snaps, "logs", "failure"), 0);
                },
            )
            .await;
        }

        // ---- Kafka integration: compression (section 6) ----
        //
        // These validate that librdkafka accepts each codec and the payload
        // survives a produce/consume round-trip. NOTE: the consumer (librdkafka)
        // transparently decompresses, and `ConsumedMessage` retains no codec
        // field, so the on-wire codec is not observable here; `assert_payload`
        // compares the decompressed bytes against the original OTLP payload. The
        // `compression.type` client-config mapping is asserted separately in
        // `config.rs::build_client_config_maps_each_compression_codec`.

        /// Scenario: export a logs batch with gzip compression enabled.
        /// Guarantees: the record round-trips (delivered, persisted, and read
        /// back with the original decompressed payload), so gzip is accepted by
        /// the producer end-to-end.
        #[tokio::test]
        async fn exports_logs_gzip_round_trips() {
            use crate::exporters::kafka_exporter::config::CompressionType;
            assert_compression_round_trips("it-gzip", CompressionType::Gzip).await;
        }

        /// Scenario: export a logs batch with snappy compression enabled.
        /// Guarantees: the record round-trips (delivered, persisted, and read
        /// back with the original decompressed payload), promoting snappy from
        /// "defined but not end-to-end tested" to round-trip validated on the
        /// mock broker.
        #[tokio::test]
        async fn exports_logs_snappy_round_trips() {
            use crate::exporters::kafka_exporter::config::CompressionType;
            assert_compression_round_trips("it-snappy", CompressionType::Snappy).await;
        }

        /// Scenario: export a logs batch with lz4 compression enabled.
        /// Guarantees: the record round-trips (delivered, persisted, and read
        /// back with the original decompressed payload), promoting lz4 from
        /// "defined but not end-to-end tested" to round-trip validated on the
        /// mock broker.
        #[tokio::test]
        async fn exports_logs_lz4_round_trips() {
            use crate::exporters::kafka_exporter::config::CompressionType;
            assert_compression_round_trips("it-lz4", CompressionType::Lz4).await;
        }

        /// Scenario: export a logs batch with zstd compression enabled.
        /// Guarantees: the record round-trips (delivered, persisted, and read
        /// back with the original decompressed payload), so zstd is accepted by
        /// the producer end-to-end.
        #[tokio::test]
        async fn exports_logs_zstd_round_trips() {
            use crate::exporters::kafka_exporter::config::CompressionType;
            assert_compression_round_trips("it-zstd", CompressionType::Zstd).await;
        }

        /// Drives one logs record through the exporter with `compression`
        /// enabled and asserts it round-trips: delivered, read back with the
        /// original (decompressed) payload and OTLP format header, and durably
        /// persisted on the partition. Shared by the four per-codec tests.
        async fn assert_compression_round_trips(
            topic: &str,
            compression: crate::exporters::kafka_exporter::config::CompressionType,
        ) {
            let topic = topic.to_string();
            with_cluster(
                KafkaTestCluster::builder().topic(&topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic.as_str()]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.clone(), MessageFormat::OtlpProto))
                        .with_compression(compression)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(&topic)
                        .assert_payload(&payload)
                        .assert_format_otlp();

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;

                    let _ = cluster.inspect().assert_message_count(&topic, 0, 1);
                },
            )
            .await;
        }

        // ---- Telemetry (section 7) ----

        /// Builds an [`ExportLogsServiceRequest`] carrying `k` log records in a
        /// single batch, so a test can distinguish per-record from per-batch
        /// metric counting.
        fn logs_request_bytes_n(k: usize) -> Vec<u8> {
            use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
            use otap_df_pdata::proto::opentelemetry::logs::v1::{
                LogRecord, ResourceLogs, ScopeLogs,
            };

            let log_records = (0..k)
                .map(|i| LogRecord {
                    time_unix_nano: (i as u64) + 1,
                    ..Default::default()
                })
                .collect();
            let req = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            req.encode_to_vec()
        }

        /// Returns the `unit` string declared for field `field` in the metric
        /// set named `set_name`, across a terminal state's snapshots (or `None`
        /// if that set/field was not emitted). `field` accepts either the Rust
        /// identifier (`acks_received`) or the emitted dotted form
        /// (`acks.received`); underscores are normalized to dots before lookup.
        fn metric_unit<'a>(
            snapshots: &'a [otap_df_telemetry::metrics::MetricSetSnapshot],
            set_name: &str,
            field: &str,
        ) -> Option<&'a str> {
            let wanted = field.replace('_', ".");
            snapshots
                .iter()
                .find(|s| s.descriptor().name == set_name)
                .and_then(|s| {
                    s.descriptor()
                        .metrics
                        .iter()
                        .find(|f| f.name == wanted)
                        .map(|f| f.unit)
                })
        }

        /// Scenario: after a successful export and graceful shutdown, inspect the
        /// terminal metric snapshots' schema.
        /// Guarantees: both node metric sets are present -- the operational
        /// `exporter.kafka` set and the measurement `exporter.kafka.exports`
        /// set -- with the migrated units (`exports.messages` is `{message}`;
        /// operational counters are `{batch}`), pinning the post-migration
        /// telemetry schema (names + units) against accidental regressions.
        #[tokio::test]
        async fn terminal_snapshot_exposes_both_metric_sets_with_expected_units() {
            let topic = "it-telemetry-schema";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send pdata");
                    let _ = consumer.recv().await.assert_topic(topic);

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();

                    // Both metric sets are represented in the terminal snapshot.
                    assert!(
                        snaps
                            .iter()
                            .any(|s| s.descriptor().name == "exporter.kafka"),
                        "operational set exporter.kafka should be present"
                    );
                    assert!(
                        snaps
                            .iter()
                            .any(|s| s.descriptor().name == "exporter.kafka.exports"),
                        "measurement set exporter.kafka.exports should be present"
                    );

                    // Migrated units: exports are per-message, operational are
                    // per-batch.
                    assert_eq!(
                        metric_unit(snaps, "exporter.kafka.exports", "messages"),
                        Some("{message}"),
                        "exports.messages unit"
                    );
                    assert_eq!(
                        metric_unit(snaps, "exporter.kafka", "acks_received"),
                        Some("{batch}"),
                        "acks_received unit"
                    );
                    assert_eq!(
                        metric_unit(snaps, "exporter.kafka", "topic_from_header"),
                        Some("{batch}"),
                        "topic_from_header unit"
                    );
                },
            )
            .await;
        }

        /// Scenario: export a single pdata batch that contains many log records.
        /// Guarantees: the export counter increments exactly once for the batch
        /// (`messages{signal=logs,outcome=success} == 1`), documenting that the
        /// exporter counts per pdata/batch -- not per record -- so the recorded
        /// per-batch counting semantics do not silently change.
        #[tokio::test]
        async fn export_counts_one_per_batch_regardless_of_record_count() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            let topic = "it-telemetry-per-batch";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // One pdata carrying 25 log records.
                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes_n(25), None))
                        .await
                        .expect("send pdata");
                    let _ = consumer.recv().await.assert_topic(topic);

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        1,
                        "a multi-record batch counts as exactly one exported batch"
                    );
                },
            )
            .await;
        }

        /// Scenario: a downstream node acknowledges a batch (a
        /// `NodeControlMsg::Ack` reaches the exporter).
        /// Guarantees: the operational `acks_received` counter increments once
        /// and `nacks_received` stays zero, validating the exporter's
        /// ack-accounting path end-to-end.
        #[tokio::test]
        async fn acks_received_counter_increments_on_downstream_ack() {
            use crate::common::kafka::node_harness::node_metrics::FoldedMetrics;
            let topic = "it-telemetry-ack";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_ack(logs_pdata(logs_request_bytes(), None))
                        .await;

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let mut m = FoldedMetrics::new();
                    m.fold_all(ts.metrics());
                    assert_eq!(m.value("acks_received"), 1, "one downstream ack observed");
                    assert_eq!(m.value("nacks_received"), 0);
                },
            )
            .await;
        }

        /// Scenario: a downstream node refuses a batch (a `NodeControlMsg::Nack`
        /// with a benign reason reaches the exporter).
        /// Guarantees: the operational `nacks_received` counter increments once
        /// and `acks_received` stays zero, validating the exporter's
        /// nack-accounting path end-to-end.
        #[tokio::test]
        async fn nacks_received_counter_increments_on_downstream_nack() {
            use crate::common::kafka::node_harness::node_metrics::FoldedMetrics;
            let topic = "it-telemetry-nack";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_nack("downstream refused", logs_pdata(logs_request_bytes(), None))
                        .await;

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let mut m = FoldedMetrics::new();
                    m.fold_all(ts.metrics());
                    assert_eq!(m.value("nacks_received"), 1, "one downstream nack observed");
                    assert_eq!(m.value("acks_received"), 0);
                },
            )
            .await;
        }

        /// Scenario: a downstream nack carries an adversarial reason string
        /// (embedded control characters and an overlong value), which the
        /// exporter logs after sanitizing.
        /// Guarantees: the exporter still counts the nack (`nacks_received ==
        /// 1`) and shuts down cleanly, so client-influenced nack reasons cannot
        /// crash, hang, or corrupt the telemetry path (the sanitizer's exact
        /// output is pinned separately by the `sanitize_for_log` unit tests).
        #[tokio::test]
        async fn nack_reason_with_control_characters_is_handled_safely() {
            use crate::common::kafka::node_harness::node_metrics::FoldedMetrics;
            let topic = "it-telemetry-nack-adversarial";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Control characters (newline, carriage return, tab, NUL,
                    // bell) plus an overlong tail to exercise escape+truncation.
                    let adversarial = format!("bad\n\r\t\0\x07reason {}", "A".repeat(200));
                    exporter
                        .send_nack(adversarial, logs_pdata(logs_request_bytes(), None))
                        .await;

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let mut m = FoldedMetrics::new();
                    m.fold_all(ts.metrics());
                    assert_eq!(
                        m.value("nacks_received"),
                        1,
                        "an adversarial nack reason is still counted and handled safely"
                    );
                },
            )
            .await;
        }

        /// Scenario: one batch is routed via a transport header while another is
        /// routed via the static per-signal topic.
        /// Guarantees: the topic-source operational counters reflect the routing
        /// decision end-to-end (`topic_from_header == 1`,
        /// `topic_from_static_config == 1`), so the router's telemetry is wired
        /// through to the terminal snapshot.
        #[tokio::test]
        async fn topic_source_counters_reflect_header_vs_static_routing() {
            use crate::common::kafka::node_harness::node_metrics::FoldedMetrics;
            let static_topic = "it-telemetry-static";
            let dynamic_topic = "it-telemetry-dynamic";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(static_topic)
                    .topic(dynamic_topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[static_topic, dynamic_topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(static_topic.into(), MessageFormat::OtlpProto)
                            .with_topic_from_transport_header("x-target-topic"),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Header-routed batch -> dynamic topic.
                    exporter
                        .send_pdata(logs_pdata(
                            logs_request_bytes(),
                            Some(("X-Target-Topic", dynamic_topic)),
                        ))
                        .await
                        .expect("send header-routed pdata");
                    // Static-routed batch -> no header, falls back to static.
                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send static-routed pdata");

                    let _ = consumer.recv_n(2).await;

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let mut m = FoldedMetrics::new();
                    m.fold_all(ts.metrics());
                    assert_eq!(
                        m.value("topic_from_header"),
                        1,
                        "one batch routed from a transport header"
                    );
                    assert_eq!(
                        m.value("topic_from_static_config"),
                        1,
                        "one batch routed from static config"
                    );
                },
            )
            .await;
        }

        /// Scenario: a mixed run of successful exports, one broker-rejected
        /// export, and one downstream ack, followed by graceful shutdown.
        /// Guarantees: the final terminal snapshot reflects all activity up to
        /// shutdown -- `messages{success} == N`, `messages{failure} == 1`, and
        /// `acks_received == 1` -- so the shutdown snapshot is a complete record
        /// of the node's counters, not a partial or reset view.
        #[tokio::test]
        async fn final_snapshot_reflects_all_activity_up_to_shutdown() {
            use crate::common::kafka::node_harness::node_metrics::{FoldedMetrics, kafka_exports};
            use rdkafka::types::RDKafkaRespErr;
            const N: usize = 3;
            let ok_topic = "it-telemetry-final-ok";
            let fail_topic = "it-telemetry-final-fail";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(ok_topic)
                    .topic(fail_topic),
                |cluster| async move {
                    // N successful exports on the ok topic.
                    let consumer = cluster.consumer().subscribe(&[ok_topic]);
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(ok_topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);
                    for _ in 0..N {
                        exporter
                            .send_pdata(logs_pdata(logs_request_bytes(), None))
                            .await
                            .expect("send ok pdata");
                    }
                    let _ = consumer.recv_n(N).await;
                    // One downstream ack.
                    exporter
                        .send_ack(logs_pdata(logs_request_bytes(), None))
                        .await;
                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    let mut m = FoldedMetrics::new();
                    m.fold_all(snaps);
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        N as u64,
                        "snapshot should record every successful export"
                    );
                    assert_eq!(
                        m.value("acks_received"),
                        1,
                        "snapshot should record the ack"
                    );

                    // One broker-rejected export on a second exporter counts as a
                    // failure in that node's snapshot.
                    cluster
                        .faults()
                        .fail_produce(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION; 8]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(
                            fail_topic.into(),
                            MessageFormat::OtlpProto,
                        ))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let fail_exporter = KafkaExporterHarness::start(&cluster, cfg);
                    fail_exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send fail pdata");
                    fail_exporter.shutdown(Duration::from_secs(5)).await;
                    let fail_ts = fail_exporter.await_terminal_state().await;
                    assert_eq!(
                        kafka_exports(fail_ts.metrics(), "logs", "failure"),
                        1,
                        "snapshot should record the rejected export as a failure"
                    );
                },
            )
            .await;
        }

        // ---- Routing & payload correctness / failure recovery (receiver-issue parity) ----

        /// Wraps OTLP traces bytes into an [`OtapPdata`].
        fn traces_pdata_from(bytes: Vec<u8>) -> OtapPdata {
            let proto = OtlpProtoBytes::ExportTracesRequest(Bytes::from(bytes));
            OtapPdata::new(Context::default(), proto.into())
        }

        /// Wraps OTLP metrics bytes into an [`OtapPdata`].
        fn metrics_pdata_from(bytes: Vec<u8>) -> OtapPdata {
            let proto = OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes));
            OtapPdata::new(Context::default(), proto.into())
        }

        /// Scenario: a single exporter is configured for all three signals on
        /// distinct topics and one batch of each signal is exported in one run.
        /// Guarantees: each signal is produced to its own topic with the correct
        /// message-format header, and the terminal snapshot records exactly one
        /// success per signal -- so a mixed-signal configuration routes and
        /// counts each signal independently without cross-talk.
        #[tokio::test]
        async fn exports_all_three_signals_to_distinct_topics() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            let traces_topic = "it-mixed-traces";
            let metrics_topic = "it-mixed-metrics";
            let logs_topic = "it-mixed-logs";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(traces_topic)
                    .topic(metrics_topic)
                    .topic(logs_topic),
                |cluster| async move {
                    let traces_consumer = cluster.consumer().subscribe(&[traces_topic]);
                    let metrics_consumer = cluster.consumer().subscribe(&[metrics_topic]);
                    let logs_consumer = cluster.consumer().subscribe(&[logs_topic]);

                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_traces(SignalConfig::new(
                                traces_topic.into(),
                                MessageFormat::OtlpProto,
                            ))
                            .with_metrics(SignalConfig::new(
                                metrics_topic.into(),
                                MessageFormat::OtlpProto,
                            ))
                            .with_logs(SignalConfig::new(
                                logs_topic.into(),
                                MessageFormat::OtlpProto,
                            ))
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let (traces, traces_payload) = traces_pdata();
                    let (metrics, metrics_payload) = metrics_pdata();
                    let logs_payload = logs_request_bytes();
                    exporter.send_pdata(traces).await.expect("send traces");
                    exporter.send_pdata(metrics).await.expect("send metrics");
                    exporter
                        .send_pdata(logs_pdata(logs_payload.clone(), None))
                        .await
                        .expect("send logs");

                    let _ = traces_consumer
                        .recv()
                        .await
                        .assert_topic(traces_topic)
                        .assert_payload(&traces_payload)
                        .assert_format_otlp();
                    let _ = metrics_consumer
                        .recv()
                        .await
                        .assert_topic(metrics_topic)
                        .assert_payload(&metrics_payload)
                        .assert_format_otlp();
                    let _ = logs_consumer
                        .recv()
                        .await
                        .assert_topic(logs_topic)
                        .assert_payload(&logs_payload)
                        .assert_format_otlp();

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(kafka_exports(snaps, "traces", "success"), 1);
                    assert_eq!(kafka_exports(snaps, "metrics", "success"), 1);
                    assert_eq!(kafka_exports(snaps, "logs", "success"), 1);
                },
            )
            .await;
        }

        /// Scenario: a malformed payload (invalid UTF-8 nested in an array
        /// attribute) is exported on the OTAP wire format for each signal, then
        /// a valid batch is exported on the same running exporter.
        /// Guarantees: each malformed batch fails to encode and increments
        /// `messages{signal,failure}` (never `success`) without reaching the
        /// broker, and the event loop survives -- a subsequent valid batch on
        /// the same signal still delivers and increments `messages{signal,
        /// success}` -- so a poison payload cannot stall the exporter.
        #[tokio::test]
        async fn encoding_failure_increments_failure_metric_per_signal() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;

            // (signal name, topic, malformed pdata, valid pdata builder)
            async fn run_case(
                signal: &'static str,
                topic: &'static str,
                malformed: OtapPdata,
                valid: OtapPdata,
            ) {
                with_cluster(KafkaTestCluster::builder().topic(topic), |cluster| {
                    async move {
                        let consumer = cluster.consumer().subscribe(&[topic]);
                        // OTAP wire so the OTLP->OtapArrowRecords conversion runs
                        // and fails on the corrupted payload.
                        let mut builder =
                            KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it");
                        builder = match signal {
                            "traces" => builder.with_traces(SignalConfig::new(
                                topic.into(),
                                MessageFormat::OtapProto,
                            )),
                            "metrics" => builder.with_metrics(SignalConfig::new(
                                topic.into(),
                                MessageFormat::OtapProto,
                            )),
                            _ => builder.with_logs(SignalConfig::new(
                                topic.into(),
                                MessageFormat::OtapProto,
                            )),
                        };
                        let cfg = builder.try_into().expect("config should be valid");
                        let exporter = KafkaExporterHarness::start(&cluster, cfg);

                        // Poison batch: fails to encode, never sent.
                        exporter
                            .send_pdata(malformed)
                            .await
                            .expect("send malformed");
                        // Valid batch on the same exporter: proves the loop
                        // survived the encode failure.
                        exporter.send_pdata(valid).await.expect("send valid");

                        // The valid OTAP batch is delivered.
                        let _ = consumer
                            .recv()
                            .await
                            .assert_topic(topic)
                            .assert_format_otap();

                        exporter.shutdown(Duration::from_secs(5)).await;
                        let ts = exporter.await_terminal_state().await;
                        let snaps = ts.metrics();
                        assert_eq!(
                            kafka_exports(snaps, signal, "failure"),
                            1,
                            "{signal}: an encode failure increments the failure bucket"
                        );
                        assert_eq!(
                            kafka_exports(snaps, signal, "success"),
                            1,
                            "{signal}: the following valid batch still succeeds"
                        );
                    }
                })
                .await;
            }

            run_case(
                "logs",
                "it-malformed-logs",
                logs_pdata(logs_request_bytes_invalid_utf8_array(), None),
                logs_pdata(logs_request_bytes(), None),
            )
            .await;
            run_case(
                "traces",
                "it-malformed-traces",
                traces_pdata_from(traces_request_bytes_invalid_utf8_array()),
                traces_pdata().0,
            )
            .await;
            run_case(
                "metrics",
                "it-malformed-metrics",
                metrics_pdata_from(metrics_request_bytes_invalid_utf8_array()),
                metrics_pdata().0,
            )
            .await;
        }

        /// Scenario: `build_kafka_headers` runs with a header-propagation policy
        /// configured on the effect handler and a pdata context carrying
        /// transport headers.
        /// Guarantees: the produced Kafka headers include the message-format
        /// header AND the propagated transport header, and a propagated header
        /// whose name collides with the format-header key is skipped -- so
        /// transport-header propagation reaches the record without clobbering
        /// the format header.
        #[test]
        fn build_kafka_headers_propagates_transport_headers_under_policy() {
            use crate::common::kafka::MSG_FORMAT_HEADER;
            use otap_df_config::transport_headers_policy::{
                HeaderPropagationPolicy, PropagationDefault, PropagationSelector,
                PropagationSelectorType,
            };
            use otap_df_engine::local::exporter::EffectHandler;
            use otap_df_engine::testing::test_node;
            use otap_df_telemetry::reporter::MetricsReporter;
            use rdkafka::message::Headers;

            // Context with two transport headers, one of which collides with the
            // format-header key and must be skipped.
            let mut transport = TransportHeaders::new();
            transport.push(TransportHeader {
                name: "x-tenant-id".to_string(),
                wire_name: "X-Tenant-Id".to_string(),
                value_kind: ValueKind::Text,
                value: b"acme".to_vec(),
            });
            transport.push(TransportHeader {
                name: MSG_FORMAT_HEADER.to_string(),
                wire_name: MSG_FORMAT_HEADER.to_string(),
                value_kind: ValueKind::Text,
                value: b"attacker-override".to_vec(),
            });
            let mut context = Context::default();
            context.set_transport_headers(transport);

            // Propagate all captured headers, preserving wire names.
            let policy = HeaderPropagationPolicy::new(
                PropagationDefault {
                    selector: PropagationSelector {
                        selector_type: PropagationSelectorType::AllCaptured,
                        named: None,
                    },
                    ..Default::default()
                },
                vec![],
            );
            let (_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
            let mut eh: EffectHandler<OtapPdata> =
                EffectHandler::new(test_node("hdr-test"), reporter);
            eh.set_propagation_policy(Some(policy));

            let headers = KafkaExporter::build_kafka_headers(
                MessageFormat::OtlpProto,
                MSG_FORMAT_HEADER,
                &context,
                Some(&eh),
            );

            // Collect the produced (key, value) pairs.
            let mut found: Vec<(String, Vec<u8>)> = Vec::new();
            for i in 0..headers.count() {
                let h = headers.get(i);
                found.push((
                    h.key.to_string(),
                    h.value.map(<[u8]>::to_vec).unwrap_or_default(),
                ));
            }

            // Exactly one format header, carrying the real format value (not the
            // attacker override), and one propagated tenant header.
            let format_headers: Vec<_> = found
                .iter()
                .filter(|(k, _)| k == MSG_FORMAT_HEADER)
                .collect();
            assert_eq!(
                format_headers.len(),
                1,
                "the format header must not be duplicated by a colliding propagated header"
            );
            assert_eq!(
                format_headers[0].1, MSG_FORMAT_OTLP,
                "the colliding transport header must not override the format value"
            );
            assert!(
                found
                    .iter()
                    .any(|(k, v)| k == "X-Tenant-Id" && v == b"acme"),
                "the tenant transport header should be propagated onto the record"
            );
        }

        /// Scenario: `build_kafka_headers` runs with no propagation policy
        /// configured (the default).
        /// Guarantees: only the message-format header is written and no
        /// transport headers leak onto the record, pinning the default
        /// no-propagation behavior.
        #[test]
        fn build_kafka_headers_writes_only_format_header_without_policy() {
            use crate::common::kafka::MSG_FORMAT_HEADER;
            use otap_df_engine::local::exporter::EffectHandler;
            use otap_df_engine::testing::test_node;
            use otap_df_telemetry::reporter::MetricsReporter;
            use rdkafka::message::Headers;

            let mut transport = TransportHeaders::new();
            transport.push(TransportHeader {
                name: "x-tenant-id".to_string(),
                wire_name: "X-Tenant-Id".to_string(),
                value_kind: ValueKind::Text,
                value: b"acme".to_vec(),
            });
            let mut context = Context::default();
            context.set_transport_headers(transport);

            let (_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
            let eh: EffectHandler<OtapPdata> =
                EffectHandler::new(test_node("hdr-test-none"), reporter);

            let headers = KafkaExporter::build_kafka_headers(
                MessageFormat::OtlpProto,
                MSG_FORMAT_HEADER,
                &context,
                Some(&eh),
            );

            assert_eq!(
                headers.count(),
                1,
                "only the format header should be present"
            );
            let h = headers.get(0);
            assert_eq!(h.key, MSG_FORMAT_HEADER);
            assert_eq!(h.value, Some(MSG_FORMAT_OTLP));
        }

        /// Scenario: the broker rejects a bounded run of consecutive produce
        /// requests (a prolonged outage), after which produce succeeds again;
        /// the exporter keeps draining its queue across the transition.
        /// Guarantees: each rejected produce increments `messages{logs,failure}`
        /// and the first post-outage send is delivered and consumed
        /// (`messages{logs,success} == 1`), so the exporter recovers after a
        /// sustained outage without stalling -- and no rejected produce
        /// persists (`success` count equals the delivered record count).
        #[tokio::test]
        async fn recovers_after_prolonged_produce_outage() {
            use crate::common::kafka::node_harness::node_metrics::kafka_exports;
            use rdkafka::types::RDKafkaRespErr;
            let topic = "it-prolonged-outage";
            // Inject exactly OUTAGE_SENDS non-retriable produce errors; the mock
            // consumes one per produce request, so the first OUTAGE_SENDS sends
            // fail and every later send succeeds. This models a bounded outage
            // followed by recovery without needing to synchronize a mid-stream
            // fault-clear against the exporter's fire-and-forget send queue.
            const OUTAGE_SENDS: usize = 5;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    cluster.faults().fail_produce(
                        &[RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION; OUTAGE_SENDS],
                    );
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Drive the outage: OUTAGE_SENDS batches that are all
                    // rejected, bounded by timeout_ms so none can hang.
                    for _ in 0..OUTAGE_SENDS {
                        exporter
                            .send_pdata(logs_pdata(logs_request_bytes(), None))
                            .await
                            .expect("send during outage");
                    }
                    // First post-outage batch: the injected errors are exhausted,
                    // so this one is accepted and delivered.
                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send after recovery");

                    // The post-recovery record is delivered and consumable.
                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(topic)
                        .assert_payload(&payload);

                    exporter.shutdown(Duration::from_secs(10)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    assert_eq!(
                        kafka_exports(snaps, "logs", "failure"),
                        OUTAGE_SENDS as u64,
                        "each outage send counts as a failure"
                    );
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        1,
                        "exactly the post-recovery send succeeds"
                    );
                    // Broker persisted exactly the one recovered record; no
                    // rejected produce persisted.
                    let _ = cluster.inspect().assert_message_count(topic, 0, 1);
                },
            )
            .await;
        }
    }
}
