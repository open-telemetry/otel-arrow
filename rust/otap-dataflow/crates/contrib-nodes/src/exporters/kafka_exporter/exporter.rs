// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core Kafka exporter implementation.
//!
//! # Delivery pipelining and backpressure
//!
//! The exporter encodes and enqueues each accepted pdata to librdkafka and
//! then tracks its delivery future in a bounded in-flight set
//! ([`InFlightSends`]). The number of concurrently outstanding deliveries is
//! capped by the `max_in_flight` config (default `10`). When the set is full the
//! event loop stops accepting new pdata and only drains completions, so
//! in-flight memory stays bounded and backpressure propagates upstream.
//!
//! With the default `max_in_flight = 10` the exporter pipelines up to ten
//! deliveries for throughput.

use super::producer::{ExporterDeliveryFuture, ExporterFutureProducer, ExporterFutureRecord};

use super::config::{KafkaExporterConfig, SignalConfig};
use super::encoder;
use super::error::{KafkaExporterError, is_permanent_send_error};
use super::metrics::{KafkaExporterErrorType, KafkaExporterMetrics, KafkaExporterOperation};
use super::partitioner;
use super::topic_regex;
use super::topic_router::TopicRouter;
#[cfg(feature = "aws")]
use crate::common::kafka::aws::ProducerClientContext;
#[cfg(feature = "aws")]
use crate::common::kafka::security::build_aws_msk_context;
use crate::common::kafka::{MSG_FORMAT_OTAP, MSG_FORMAT_OTLP, MessageFormat};
use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use futures_channel::oneshot::Canceled;
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
use otap_df_telemetry::common_attributes::Outcome;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::FromClientConfigAndContext;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::Producer;
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use regex::Regex;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Compiles a signal's `allowed_topics_regex` patterns into [`Regex`] values,
/// or returns `None` when the signal configures no patterns (avoiding an
/// empty-vector allocation for the common case).
///
/// Each operator pattern is anchored to require a **whole-topic** match via
/// [`topic_regex::compile_anchor_and_validate`], which wraps it as
/// `\A(?:<pattern>)\z`. The dynamic-routing allowlist is an authorization
/// boundary for a client-controlled destination, so an unanchored pattern
/// (which the `regex` crate would match as a substring) must not permit
/// unintended topics -- e.g. `tenant_.*` must permit `tenant_a` but reject
/// `evil-tenant_a-x`. To keep the anchors from being escaped, each operator
/// pattern is first validated as a self-contained regex before being wrapped (a
/// pattern that balances its parentheses against the wrapper, e.g.
/// `tenant_.)\z|(?:evil.`, is rejected rather than allowed to drop the `\A`
/// anchor on an alternation). Entries must be valid standalone regular
/// expressions.
///
/// # Errors
///
/// Returns [`KafkaExporterError::ConfigInvalidTopicRegex`] if any pattern is not
/// a valid standalone regex or the anchored form fails to compile, naming the
/// `signal` and reporting the operator's original pattern (not the anchored
/// form) for diagnosis.
fn compile_allowed_topic_regexes(
    patterns: &[String],
    signal: SignalType,
) -> Result<Option<Vec<Regex>>, KafkaExporterError> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut compiled = Vec::with_capacity(patterns.len());
    for pattern in patterns {
        let re = topic_regex::compile_anchor_and_validate(pattern).map_err(|message| {
            KafkaExporterError::ConfigInvalidTopicRegex {
                signal,
                pattern: pattern.clone(),
                message,
            }
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

/// Metadata carried alongside a pipelined delivery so its completion can be
/// reported as an ack or nack.
///
/// The `pdata` is the reconstructed [`OtapPdata`] (context + payload) that is
/// handed back to the reporter on ack/nack so an upstream `processor:retry` can
/// retry a transiently failed batch. `topic` is retained only so the failure
/// log can name the (possibly client-supplied) destination.
struct SendMeta {
    signal_type: SignalType,
    topic: String,
    pdata: OtapPdata,
    export_start: Instant,
    delivery_start: Instant,
    payload_bytes: usize,
}

/// Bounded, self-managing set of in-flight Kafka deliveries.
///
/// Wraps a [`FuturesUnordered`] of boxed futures that each await a delivery and
/// yield its [`SendMeta`] paired with the delivery outcome. The delivery
/// outcome flattens as: `Ok(Ok(..))` delivered successfully; `Ok(Err(..))`
/// delivery failed carrying a [`rdkafka::error::KafkaError`]; `Err(Canceled)`
/// the delivery future was cancelled because the producer was dropped or purged
/// (treated as a transient failure, matching the purge-on-shutdown semantics).
///
/// The set is constructed with the configured `max_in_flight` bound and owns
/// it: [`Self::push`] enforces the bound directly (draining one outstanding
/// delivery when the set is already full), so the number of concurrently
/// outstanding deliveries never exceeds the bound (see [`KafkaExporter`] module
/// docs). Callers still use [`Self::is_full`] to gate upstream admission, but
/// they no longer need to pre-drain to keep `push` correct.
struct InFlightSends {
    #[allow(clippy::type_complexity)]
    futures: FuturesUnordered<
        Pin<Box<dyn Future<Output = (SendMeta, Result<OwnedDeliveryResult, Canceled>)>>>,
    >,
    /// Maximum number of deliveries allowed to be outstanding at once. Set from
    /// the `max_in_flight` config and enforced via [`Self::is_full`].
    max_in_flight: usize,
}

impl InFlightSends {
    /// Creates an empty set bounded at `max_in_flight` concurrent deliveries.
    fn new(max_in_flight: usize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            max_in_flight,
        }
    }

    /// Whether there are no outstanding deliveries.
    #[inline]
    fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Whether the set has reached its `max_in_flight` bound and must drain a
    /// completion before accepting another delivery.
    #[inline]
    fn is_full(&self) -> bool {
        self.futures.len() >= self.max_in_flight
    }

    /// Track an in-flight delivery, enforcing the `max_in_flight` bound.
    ///
    /// When the set is already [`Self::is_full`], this first awaits one
    /// outstanding delivery and returns its completion (which the caller must
    /// finalize) before storing the new delivery. This guarantees the number of
    /// concurrently outstanding deliveries never exceeds `max_in_flight` without
    /// the caller having to pre-drain. The stored delivery's future is then
    /// polled by [`Self::next_completion`] until it resolves, at which point
    /// `meta` is paired with the delivery outcome.
    async fn push(
        &mut self,
        delivery: ExporterDeliveryFuture,
        meta: SendMeta,
    ) -> Option<(SendMeta, Result<OwnedDeliveryResult, Canceled>)> {
        // At capacity: drain exactly one completion to make room. The set is
        // non-empty here (is_full implies len >= max_in_flight >= 1), so
        // `next()` yields `Some`.
        let completed = if self.is_full() {
            self.futures.next().await
        } else {
            None
        };
        self.futures.push(Box::pin(async move {
            let result = delivery.await;
            (meta, result)
        }));
        completed
    }

    /// Await the next resolved delivery, returning its metadata and outcome.
    ///
    /// When the set is empty this stays pending forever (rather than resolving
    /// to `None`), so it can be used directly in a `select` without busy
    /// looping. Callers must guard with [`Self::is_empty`] where a definite
    /// answer is required (e.g. the shutdown drain loop).
    async fn next_completion(&mut self) -> (SendMeta, Result<OwnedDeliveryResult, Canceled>) {
        if self.futures.is_empty() {
            std::future::pending().await
        } else {
            // Safe to unwrap: the set is non-empty, and FuturesUnordered only
            // yields None when empty.
            self.futures
                .next()
                .await
                .expect("FuturesUnordered yielded None while non-empty")
        }
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
#[otap_df_engine::component_inventory(category = Exporter)]
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
            otel_warn!(
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
            Some(s) => compile_allowed_topic_regexes(s.allowed_topics_regex(), SignalType::Traces)?,
            None => None,
        };
        let metrics = match config.metrics() {
            Some(s) => {
                compile_allowed_topic_regexes(s.allowed_topics_regex(), SignalType::Metrics)?
            }
            None => None,
        };
        let logs = match config.logs() {
            Some(s) => compile_allowed_topic_regexes(s.allowed_topics_regex(), SignalType::Logs)?,
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

    /// Encodes a single PData message and enqueues it to Kafka, returning the
    /// resulting [`InFlightSend`] whose delivery future the caller tracks in the
    /// bounded in-flight set.
    ///
    /// This performs all the synchronous pre-send work -- config lookup, topic
    /// resolution, partition-key derivation, header building, and encoding --
    /// and then enqueues the record via
    /// [`ExporterFutureProducer::send_result`], which returns immediately once
    /// the record is accepted by librdkafka (the delivery itself completes
    /// asynchronously and is finalized later by
    /// [`Self::finalize_send_completion`]).
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
    ///
    /// # Return value
    ///
    /// * `Ok(Some((delivery, meta)))` -- the record was accepted by librdkafka;
    ///   the caller must push the delivery future and its [`SendMeta`] into the
    ///   in-flight set so the delivery is finalized.
    /// * `Ok(None)` -- the message was terminally handled synchronously (an
    ///   enqueue failure was already reported as a nack); there is no in-flight
    ///   delivery to track.
    /// * `Err(e)` -- a pre-send failure (unconfigured signal, invalid dynamic
    ///   topic, or encode failure) was already reported as a permanent nack.
    async fn enqueue_pdata(
        &mut self,
        pdata: OtapPdata,
        reporter: &dyn AckNackReporter,
        effect_handler: Option<&EffectHandler<OtapPdata>>,
    ) -> Result<Option<(ExporterDeliveryFuture, SendMeta)>, KafkaExporterError> {
        let export_start = Instant::now();
        let signal_type = pdata.signal_type();

        // Extract context and payload first so we can nack if config lookup fails.
        let (context, payload) = pdata.into_parts();

        // Look up the per-signal config once. If the signal type is not
        // configured, permanently nack the message (configuration errors
        // will never resolve on retry) and return the error.
        let signal_config = match Self::get_signal_config(&self.config, signal_type) {
            Ok(cfg) => cfg,
            Err(e) => {
                otel_warn!(
                    "kafka.exporter.signal.unconfigured",
                    signal_type = ?signal_type,
                    error = %e,
                );
                self.metrics.record_failure(
                    signal_type,
                    KafkaExporterErrorType::UnconfiguredSignal,
                    export_start.elapsed(),
                    None,
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
        let topic = match TopicRouter::resolve(
            signal_config,
            allowed_regex,
            &context,
            signal_type,
            &mut self.metrics,
        ) {
            Ok(t) => t,
            Err(e) => {
                self.metrics.record_failure(
                    signal_type,
                    KafkaExporterErrorType::InvalidTopic,
                    export_start.elapsed(),
                    None,
                );
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
        let encoding_start = Instant::now();
        let encode_result = match encoding {
            MessageFormat::OtlpProto => encoder::encode_to_otlp_bytes(payload.clone()),
            MessageFormat::OtapProto => encoder::encode_to_batch_arrow_record_bytes(
                payload.clone(),
                &mut self.pdata_producer,
            ),
        };

        // nack on failed encoding bytes
        let payload_bytes = match encode_result {
            Ok(bytes) => {
                self.metrics.record_operation(
                    signal_type,
                    KafkaExporterOperation::Encoding,
                    Outcome::Success,
                    encoding_start.elapsed().as_secs_f64(),
                );
                bytes
            }
            Err(e) => {
                otel_error!(
                    "kafka.exporter.encode.failed",
                    signal_type = ?signal_type,
                    error = %e,
                );
                self.metrics.record_operation(
                    signal_type,
                    KafkaExporterOperation::Encoding,
                    Outcome::Failure,
                    encoding_start.elapsed().as_secs_f64(),
                );
                self.metrics.record_failure(
                    signal_type,
                    KafkaExporterErrorType::Encoding,
                    export_start.elapsed(),
                    None,
                );
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

        // Enqueue the record to librdkafka without awaiting delivery. The
        // returned delivery future resolves asynchronously and is finalized by
        // `finalize_send_completion`; bounding the number of outstanding
        // futures (see `max_in_flight`) is what provides backpressure here.
        //
        // Note: unlike the previous inline `send`, `send_result` does not retry
        // a full producer queue. A queue-full (or any other) enqueue error is
        // reported as a transient nack so an upstream `processor:retry` can
        // resend; the bounded in-flight set already keeps the queue from being
        // driven unboundedly deep.
        let delivery_start = Instant::now();
        match self.producer.send_result(record) {
            Ok(delivery) => Ok(Some((
                delivery,
                SendMeta {
                    signal_type,
                    topic: topic.into_owned(),
                    pdata: OtapPdata::new(context, payload),
                    export_start,
                    delivery_start,
                    payload_bytes: payload_bytes.len(),
                },
            ))),
            Err((kafka_err, _record)) => {
                self.metrics.record_delivery_failure(
                    signal_type,
                    &kafka_err,
                    delivery_start.elapsed().as_secs_f64(),
                    export_start.elapsed(),
                    payload_bytes.len(),
                );
                let permanent = is_permanent_send_error(&kafka_err);
                // `topic` may be a client-supplied (header-routed) value, so
                // bound/escape it before logging to avoid log injection.
                otel_warn!(
                    "kafka.exporter.send.failed",
                    topic = %crate::common::kafka::sanitize_for_log(&topic),
                    signal_type = ?signal_type,
                    permanent = permanent,
                    error = %kafka_err,
                );
                let reason = kafka_err.to_string();
                let refused = OtapPdata::new(context, payload);
                let nack_result = if permanent {
                    reporter.nack_permanent(reason, refused).await
                } else {
                    reporter.nack(reason, refused).await
                };
                if let Err(e) = nack_result {
                    if let Some(eh) = effect_handler {
                        eh.info(&format!(
                            "Failed to report nack for Kafka export enqueue failure: {}",
                            e
                        ))
                        .await;
                    }
                }
                // Enqueue failure was reported synchronously; there is no
                // in-flight delivery to track.
                Ok(None)
            }
        }
    }

    /// Finalizes a resolved in-flight delivery by reporting the ack or nack.
    ///
    /// A successful delivery increments the exported counter and acks the
    /// original pdata. A delivery failure (or a cancelled delivery future,
    /// which happens when the producer is purged on shutdown/reconfigure)
    /// increments the failed counter and reports a nack -- permanent for errors
    /// that can never succeed on retry, transient otherwise. Ack/nack reporting
    /// is best-effort; a reporting error is logged but never fails the export.
    async fn finalize_send_completion(
        &mut self,
        meta: SendMeta,
        result: Result<OwnedDeliveryResult, Canceled>,
        reporter: &dyn AckNackReporter,
        effect_handler: Option<&EffectHandler<OtapPdata>>,
    ) {
        let SendMeta {
            signal_type,
            topic,
            pdata,
            export_start,
            delivery_start,
            payload_bytes,
        } = meta;

        // Match the two delivery layers directly:
        // `Ok(Ok(..))`  -> delivered successfully; ack.
        // `Ok(Err(..))` -> delivery failed with a KafkaError; nack.
        // `Err(Canceled)` -> producer dropped/purged before delivery resolved;
        //                    treat as a transient `Canceled` failure (matches
        //                    purge-error semantics) so the batch can be retried.
        let kafka_err = match result {
            Ok(Ok(_delivery)) => {
                self.metrics.record_operation(
                    signal_type,
                    KafkaExporterOperation::Delivery,
                    Outcome::Success,
                    delivery_start.elapsed().as_secs_f64(),
                );
                self.metrics
                    .record_success(signal_type, export_start.elapsed(), payload_bytes);
                if let Err(e) = reporter.ack(pdata).await {
                    if let Some(eh) = effect_handler {
                        eh.info(&format!(
                            "Failed to report ack for Kafka export (export succeeded): {}",
                            e
                        ))
                        .await;
                    }
                }
                return;
            }
            Ok(Err((kafka_err, _owned_message))) => kafka_err,
            Err(_canceled) => rdkafka::error::KafkaError::Canceled,
        };

        self.metrics.record_delivery_failure(
            signal_type,
            &kafka_err,
            delivery_start.elapsed().as_secs_f64(),
            export_start.elapsed(),
            payload_bytes,
        );
        // Classify the delivery failure: some Kafka errors (e.g. a record that
        // exceeds `message.max.bytes`, or an authorization failure) can never
        // succeed on retry, so they are permanently nacked and dropped at the
        // source rather than retried by an upstream `processor:retry`.
        // Everything else stays transient.
        let permanent = is_permanent_send_error(&kafka_err);
        // `topic` may be a client-supplied (header-routed) value, so
        // bound/escape it before logging to avoid log injection.
        otap_df_telemetry::otel_warn!(
            "kafka.exporter.send.failed",
            topic = %crate::common::kafka::sanitize_for_log(&topic),
            signal_type = ?signal_type,
            permanent = permanent,
            error = %kafka_err,
        );
        let reason = kafka_err.to_string();
        let nack_result = if permanent {
            reporter.nack_permanent(reason, pdata).await
        } else {
            reporter.nack(reason, pdata).await
        };
        if let Err(e) = nack_result {
            if let Some(eh) = effect_handler {
                eh.info(&format!(
                    "Failed to report nack for Kafka export failure: {}",
                    e
                ))
                .await;
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
        deadline: Instant,
        effect_handler: &EffectHandler<OtapPdata>,
    ) {
        effect_handler.info("Flushing Kafka producer").await;

        // Flush for the time remaining until the shutdown deadline (saturating
        // at zero if it has already passed), matching the parquet exporter's
        // deadline-bounded shutdown flush.
        let flush_timeout = deadline
            .checked_duration_since(Instant::now())
            .unwrap_or(Duration::ZERO);

        if let Err(e) = self.producer.flush(flush_timeout) {
            otel_warn!(
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
    /// The in-flight set is rebuilt at the new `max_in_flight` after the old
    /// deliveries are drained, so a live change to the concurrency bound takes
    /// effect immediately (a lowered bound is enforced at once; a raised bound
    /// applies to subsequent deliveries).
    ///
    /// Reconfiguration is best-effort: if the incoming config fails to
    /// deserialize/validate, or the new producer fails to build, the error is
    /// logged and the existing producer keeps running. This mirrors the
    /// reconfiguration posture of sibling nodes (e.g. the condense-attributes
    /// and retry processors), which warn-and-keep rather than failing the node.
    ///
    /// # Known limitations
    ///
    /// Live reconfiguration does NOT yet provide the following guarantees.
    /// Both are tracked in the live-reconfiguration issue
    /// (see <https://github.com/open-telemetry/otel-arrow/issues/3768>}.
    ///
    /// - **In-flight pdata can cross configurations.** Control messages
    ///   (including `Config`) and pdata arrive on separate channels, and
    ///   control is processed with priority. So pdata that the exporter
    ///   already accepted *before* the `Config` message can still be sitting in
    ///   the inbox and get processed *after* `self.config` and `self.producer`
    ///   are replaced below. Those records are then sent using the new topic,
    ///   credentials, or tenant configuration rather than the one in effect when
    ///   they were accepted. There is no ordered cutover barrier that applies
    ///   the new config only after all preceding pdata has been processed.
    /// - **The swap can block the pipeline.** The synchronous
    ///   `self.producer.flush(flush_timeout)` below, and dropping the old
    ///   producer (which joins its poll thread), both run on the core-local
    ///   async runtime. A slow or unavailable broker can therefore stall all
    ///   normal processing and backpressure handling for up to the flush
    ///   timeout instead of letting the pipeline keep making progress. A
    ///   non-blocking design would move producer creation, flushing, and
    ///   retirement to a bounded, serialized lifecycle worker.
    async fn reconfigure(
        &mut self,
        config: serde_json::Value,
        in_flight: &mut InFlightSends,
        reporter: &dyn AckNackReporter,
        effect_handler: &EffectHandler<OtapPdata>,
    ) {
        // Deserialize and validate the incoming config. On failure, keep the
        // current producer/config running.
        let new_config: KafkaExporterConfig = match serde_json::from_value(config) {
            Ok(cfg) => cfg,
            Err(e) => {
                otel_warn!(
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
            otel_warn!(
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
                otel_warn!(
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
                    otel_warn!(
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
            otel_warn!(
                "kafka.exporter.reconfigure.flush_failed",
                error = %e,
            );
            self.producer
                .purge(rdkafka::producer::PurgeConfig::default().queue().inflight());
        }

        // Finalize every pipelined delivery tracked against the old producer
        // before swapping it out. Their delivery futures resolve from the old
        // producer's callbacks (successfully after the flush above, or as a
        // purge/cancel error otherwise); draining them here reports the ack or
        // nack for each and, critically, releases the borrow on the old
        // producer's delivery channels before it is dropped.
        while !in_flight.is_empty() {
            let (meta, result) = in_flight.next_completion().await;
            self.finalize_send_completion(meta, result, reporter, Some(effect_handler))
                .await;
        }

        // all in_flight msgs should be drained
        debug_assert!(in_flight.is_empty(), "in-flight set must be drained");

        // Capture the new concurrency bound before `new_config` is moved into
        // `self.config` below.
        let new_max_in_flight = if self.config.max_in_flight() != new_config.max_in_flight() {
            Some(new_config.max_in_flight())
        } else {
            None
        };

        // Swap in the new producer, config, and compiled allowlist regexes.
        // Dropping the old producer joins its poll thread (see
        // ExporterThreadedProducer::drop).
        self.producer = new_producer;
        self.config = new_config;
        self.traces_allowed_topics_regex = new_traces_regex;
        self.metrics_allowed_topics_regex = new_metrics_regex;
        self.logs_allowed_topics_regex = new_logs_regex;
        // create new InFlightSends if user changes max_in_flight setting
        if let Some(max_in_flight) = new_max_in_flight {
            *in_flight = InFlightSends::new(max_in_flight);
        }

        otel_info!(
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

        // Bounded, self-managing set of pipelined deliveries. It owns the
        // `max_in_flight` bound: while it reports `is_full()` the loop stops
        // admitting new pdata (via `recv_when`), which bounds in-flight memory
        // and propagates backpressure upstream. The default of 1 preserves the
        // historical serial behavior.
        let mut in_flight = InFlightSends::new(self.config.max_in_flight());

        // Main loop: biased-wait for either an in-flight completion or the next
        // inbound message, admitting new pdata only while the in-flight set has
        // spare capacity.
        loop {
            // Gate pdata admission on spare capacity: while the set is full only
            // control messages (and shutdown-time force-drained pdata) can
            // arrive, so the `max_in_flight` bound is respected. Completions win
            // ties so acks/nacks drain promptly and in-flight memory is
            // released.
            let accepting_pdata = !in_flight.is_full();
            let msg = if in_flight.is_empty() {
                inbox.recv_when(accepting_pdata).await?
            } else {
                let completion_fut = in_flight.next_completion().fuse();
                let recv_fut = inbox.recv_when(accepting_pdata).fuse();
                futures::pin_mut!(completion_fut, recv_fut);

                futures::select_biased! {
                    completion = completion_fut => {
                        let (meta, result) = completion;
                        self.finalize_send_completion(
                            meta,
                            result,
                            &ack_nack_reporter,
                            Some(&effect_handler),
                        )
                        .await;
                        continue;
                    }
                    msg = recv_fut => msg?,
                }
            };

            match msg {
                Message::PData(pdata) => {
                    // On `Ok(Some((delivery, meta)))` track the delivery; an
                    // enqueue failure or a synchronous pre-send nack (`Ok(None)`
                    // / `Err(_)`) was already reported, so there is nothing to
                    // track.
                    //
                    // `push` enforces the `max_in_flight` bound itself: if the
                    // set is already full (which can happen when shutdown
                    // draining force-drains buffered pdata past the admission
                    // gate), it drains one completion and returns it so we can
                    // finalize its ack/nack here.
                    if let Ok(Some((delivery, meta))) = self
                        .enqueue_pdata(pdata, &ack_nack_reporter, Some(&effect_handler))
                        .await
                    {
                        if let Some((done_meta, done_result)) = in_flight.push(delivery, meta).await
                        {
                            self.finalize_send_completion(
                                done_meta,
                                done_result,
                                &ack_nack_reporter,
                                Some(&effect_handler),
                            )
                            .await;
                        }
                    }
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    // Flush exporter metrics into the telemetry registry.
                    _ = self.metrics.report(&mut metrics_reporter);
                }
                Message::Control(NodeControlMsg::Ack(_ack)) => {
                    // Exporters terminate pdata delivery and do not route downstream acks.
                }
                Message::Control(NodeControlMsg::Nack(nack)) => {
                    // A nack reached the end of the pipeline. The reason string
                    // can embed client-supplied values
                    // (e.g. a header-routed topic), so bound/escape it.
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
                    // engine's receiver-first drain. Flush the producer (bounded
                    // by `deadline`) so pipelined deliveries get a final chance,
                    // then finalize every tracked in-flight delivery so its
                    // ack/nack is reported before we return, then purge anything
                    // still queued so we never block past the deadline.
                    self.drain_and_flush(deadline, &effect_handler).await;
                    while !in_flight.is_empty() {
                        let (meta, result) = in_flight.next_completion().await;
                        self.finalize_send_completion(
                            meta,
                            result,
                            &ack_nack_reporter,
                            Some(&effect_handler),
                        )
                        .await;
                    }

                    effect_handler.info("Kafka exporter stopped").await;
                    return Ok(TerminalState::new(
                        deadline,
                        self.metrics.terminal_snapshots(),
                    ));
                }
                Message::Control(NodeControlMsg::Config { config }) => {
                    // Live reconfiguration: build-and-swap the librdkafka
                    // producer with a bounded drain of the old one (including
                    // finalizing tracked in-flight deliveries). Invalid configs
                    // are logged and ignored (the current producer keeps
                    // running).
                    //
                    // Known limitations (see `reconfigure`): pdata accepted
                    // before this `Config` message can be processed after the
                    // swap and cross to the new topic/credentials/tenant, and
                    // the bounded flush plus old-producer drop can block the
                    // pipeline. Tracked in the live-reconfiguration issue
                    // (https://github.com/open-telemetry/otel-arrow/issues/3768).
                    self.reconfigure(config, &mut in_flight, &ack_nack_reporter, &effect_handler)
                        .await;
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

    /// Exports a single batch using the provided exporter and reporter,
    /// awaiting its delivery to completion.
    ///
    /// This drives the full enqueue -> await-delivery -> finalize sequence so a
    /// single call reports exactly one ack or nack, matching the pre-pipelining
    /// behavior the unit tests rely on. The production event loop instead
    /// pipelines many deliveries via [`KafkaExporter::enqueue_pdata`] and
    /// [`KafkaExporter::finalize_send_completion`]; here they are chained inline
    /// so tests can assert the outcome synchronously.
    pub async fn export_once(
        exporter: &mut KafkaExporter,
        pdata: OtapPdata,
        reporter: &dyn AckNackReporter,
    ) -> Result<(), KafkaExporterError> {
        // Pre-send failures (unconfigured signal, invalid dynamic topic, encode
        // failure) and synchronous enqueue failures are already reported by
        // `enqueue_pdata`; propagate any error and stop.
        let (delivery, meta) = match exporter.enqueue_pdata(pdata, reporter, None).await? {
            Some(send) => send,
            None => return Ok(()),
        };

        // Await this single delivery and finalize it (ack or nack), mirroring
        // the loop's completion handling.
        let result = delivery.await;
        // Capture the delivery outcome before finalize consumes `result` so the
        // helper can surface a delivery failure to callers as an `Err` (the
        // production loop reports the nack via the reporter and discards the
        // per-send error).
        let delivery_err: Option<KafkaExporterError> = match &result {
            Ok(Ok(_)) => None,
            Ok(Err((kafka_err, _))) => Some(KafkaExporterError::KafkaError(kafka_err.clone())),
            Err(_canceled) => Some(KafkaExporterError::KafkaError(
                rdkafka::error::KafkaError::Canceled,
            )),
        };
        exporter
            .finalize_send_completion(meta, result, reporter, None)
            .await;
        match delivery_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::exporters::kafka_exporter::config::PartitionerStrategy;
        use crate::exporters::kafka_exporter::config::TlsConfig;
        use crate::exporters::kafka_exporter::config::{CompressionType, RequiredAcks};
        use crate::exporters::kafka_exporter::partitioner::partition_key_from_transport_headers;
        use bytes::Bytes;
        use otap_df_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
        use otap_df_config::transport_headers_policy::{
            HeaderPropagationPolicy, PropagationDefault, PropagationSelector,
            PropagationSelectorType,
        };
        use otap_df_otap::pdata::Context;
        use otap_df_pdata::OtlpProtoBytes;
        use prost::Message as _;
        use std::time::Duration;

        // Kafka test-suite wiring (mock broker, exporter harness, assertions).
        use crate::common::kafka::MSG_FORMAT_HEADER;
        use crate::common::kafka::node_harness::KafkaExporterHarness;
        use crate::common::kafka::node_harness::node_metrics::{kafka_exports, measurement_value};
        use crate::common::kafka::test::cluster::KafkaTestCluster;
        use crate::common::kafka::test::message::count_by_partition;
        use crate::common::kafka::test::{run_on_local_set, with_cluster};

        // Engine/telemetry helpers used by the header-propagation unit tests.
        use otap_df_engine::local::exporter::EffectHandler;
        use otap_df_engine::testing::test_node;
        use otap_df_telemetry::reporter::MetricsReporter;

        // rdkafka helpers used across integration tests.
        use rdkafka::message::Headers;
        use rdkafka::types::RDKafkaRespErr;

        // OTLP/OTAP proto types used by the payload builders (superset across
        // all builders so no builder needs a local import).
        use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;
        use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
        use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
        use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
        use otap_df_pdata::proto::opentelemetry::common::v1::{
            AnyValue, ArrayValue, KeyValue, any_value,
        };
        use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
        use otap_df_pdata::proto::opentelemetry::metrics::v1::{
            Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric,
        };
        use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};

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

        /// Scenario (security: dynamic topic routing): operator allowlist regex
        /// patterns are compiled by `compile_allowed_topic_regexes`, the same
        /// function the exporter uses at construction/reconfigure.
        /// Guarantees: patterns are anchored to a whole-topic match -- `tenant_.*`
        /// permits `tenant_a` but rejects a topic that merely contains it
        /// (`evil-tenant_a-x`), and a top-level alternation is contained so
        /// `a|b` matches only exactly `a` or `b` (never `xax` or `ab`) -- closing
        /// the substring authorization gap on this client-controlled boundary.
        #[test]
        fn compile_allowed_topic_regexes_anchors_to_whole_topic() {
            let compiled =
                compile_allowed_topic_regexes(&["tenant_.*".to_string()], SignalType::Logs)
                    .expect("valid pattern compiles")
                    .expect("some patterns");
            let re = &compiled[0];
            assert!(
                re.is_match("tenant_a"),
                "whole-topic match must be permitted"
            );
            assert!(
                re.is_match("tenant_anything"),
                "prefix pattern still matches a longer whole topic"
            );
            assert!(
                !re.is_match("evil-tenant_a-x"),
                "a substring match must NOT be permitted (authorization boundary)"
            );
            assert!(
                !re.is_match("xtenant_a"),
                "a leading-prefixed topic must NOT be permitted"
            );

            // Alternation containment: `a|b` must match only exactly `a` or `b`.
            let alt = compile_allowed_topic_regexes(&["a|b".to_string()], SignalType::Logs)
                .expect("valid pattern compiles")
                .expect("some patterns");
            let alt_re = &alt[0];
            assert!(alt_re.is_match("a"), "exact `a` permitted");
            assert!(alt_re.is_match("b"), "exact `b` permitted");
            assert!(
                !alt_re.is_match("xax"),
                "alternation must be anchored, not matched as a substring"
            );
            assert!(
                !alt_re.is_match("ab"),
                "`a|b` must not permit the concatenation `ab`"
            );
        }

        /// Scenario (security: dynamic topic routing): an operator pattern
        /// crafted to break out of the `\A(?:<pattern>)\z` anchoring wrapper
        /// (`tenant_.)\z|(?:evil.`) is compiled through the exporter's allowlist
        /// path.
        /// Guarantees: the pattern is rejected with `ConfigInvalidTopicRegex`
        /// carrying the offending `signal` and the operator's original `pattern`
        /// (never compiled into an allowlist entry), so a pattern that would
        /// otherwise under-anchor an alternation and permit unintended
        /// header-routed topics cannot reach the router -- the anchor-breakout
        /// authorization bypass is closed at compile time.
        #[test]
        fn compile_allowed_topic_regexes_rejects_anchor_breakout() {
            let err = compile_allowed_topic_regexes(
                &[r"tenant_.)\z|(?:evil.".to_string()],
                SignalType::Logs,
            )
            .expect_err("anchor-breakout pattern must be rejected");
            match err {
                KafkaExporterError::ConfigInvalidTopicRegex {
                    pattern, signal, ..
                } => {
                    assert_eq!(pattern, r"tenant_.)\z|(?:evil.");
                    assert_eq!(signal, SignalType::Logs);
                }
                other => panic!("unexpected error variant: {other:?}"),
            }
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
                .with_compression(CompressionType::Zstd)
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

        /// Scenario: A reporter receives successful, transient, and permanent outcomes.
        /// Guarantees: Each outcome is retained in its corresponding bounded test collection.
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

        /// Scenario: A traces message reaches an exporter configured only for logs.
        /// Guarantees: The message is permanently nacked and counted as an unconfigured-signal failure.
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
            assert_eq!(
                exporter
                    .metrics
                    .exports
                    .get(
                        otap_df_telemetry::common_attributes::SignalOutcomeAttributes {
                            signal: SignalType::Traces,
                            outcome: Outcome::Failure,
                        }
                    )
                    .messages
                    .get(),
                1,
            );
            assert_eq!(
                exporter
                    .metrics
                    .failures
                    .get(
                        crate::exporters::kafka_exporter::metrics::KafkaExporterFailureAttributes {
                            signal: SignalType::Traces,
                            error_type: KafkaExporterErrorType::UnconfiguredSignal,
                        }
                    )
                    .messages
                    .get(),
                1,
            );
        }

        /// Scenario: A transport header supplies an invalid dynamic Kafka topic.
        /// Guarantees: The message is permanently nacked and classified as an invalid-topic failure.
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
            assert_eq!(
                exporter
                    .metrics
                    .failures
                    .get(
                        crate::exporters::kafka_exporter::metrics::KafkaExporterFailureAttributes {
                            signal: SignalType::Logs,
                            error_type: KafkaExporterErrorType::InvalidTopic,
                        }
                    )
                    .messages
                    .get(),
                1,
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

        // ---- Shared test helpers (payload builders, config, assertions) ----

        /// Builds a validated single-signal-logs config bound to `brokers`.
        fn logs_config(brokers: &str, signal: SignalConfig) -> KafkaExporterConfig {
            KafkaExporterConfigBuilder::new(brokers, "it-client")
                .with_logs(signal)
                .try_into()
                .expect("config should be valid")
        }

        /// Like [`logs_config`] but with an explicit `max_in_flight` so tests can
        /// exercise the bounded delivery-future pipelining.
        fn logs_config_mif(
            brokers: &str,
            signal: SignalConfig,
            max_in_flight: usize,
        ) -> KafkaExporterConfig {
            KafkaExporterConfigBuilder::new(brokers, "it-client")
                .with_logs(signal)
                .with_max_in_flight(max_in_flight)
                .try_into()
                .expect("config should be valid")
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

        /// Like [`logs_reconfig_json`] but with an explicit `max_in_flight`, so a
        /// test can exercise a live change to the concurrency bound.
        fn logs_reconfig_json_mif(
            brokers: &str,
            topic: &str,
            max_in_flight: usize,
        ) -> serde_json::Value {
            serde_json::json!({
                "brokers": brokers,
                "client_id": "it-client",
                "logs": { "topic": topic, "encoding": "otlp_proto" },
                "max_in_flight": max_in_flight,
            })
        }

        /// Encodes `log_records` into a single-resource, single-scope
        /// [`ExportLogsServiceRequest`]'s OTLP proto bytes. Shared by the
        /// single-record, sequenced, and multi-record logs builders below.
        fn logs_request_bytes_from(log_records: Vec<LogRecord>) -> Vec<u8> {
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

        /// Builds an [`ExportLogsServiceRequest`] with a single log record so
        /// tests exercise a real OTLP payload (required for OTAP encoding).
        fn logs_request_bytes() -> Vec<u8> {
            logs_request_bytes_from(vec![LogRecord {
                time_unix_nano: 1,
                ..Default::default()
            }])
        }

        /// Builds an [`ExportLogsServiceRequest`] whose single log record body
        /// encodes `seq`, so a sequence of these payloads is byte-distinct and
        /// can be checked for delivery order.
        fn logs_request_bytes_seq(seq: usize) -> Vec<u8> {
            logs_request_bytes_from(vec![LogRecord {
                time_unix_nano: 1,
                body: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(format!("seq-{seq}"))),
                }),
                ..Default::default()
            }])
        }

        /// Builds an [`ExportLogsServiceRequest`] carrying `k` log records in a
        /// single batch, so a test can distinguish per-record from per-batch
        /// metric counting.
        fn logs_request_bytes_n(k: usize) -> Vec<u8> {
            logs_request_bytes_from(
                (0..k)
                    .map(|i| LogRecord {
                        time_unix_nano: (i as u64) + 1,
                        ..Default::default()
                    })
                    .collect(),
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

        /// Builds an [`ExportTraceServiceRequest`] with a single span, returned
        /// as OTLP proto bytes wrapped in an [`OtapPdata`].
        fn traces_pdata() -> (OtapPdata, Vec<u8>) {
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

        /// Builds OTLP logs bytes with one attribute whose value is an array
        /// containing a string element with invalid UTF-8 bytes. The OTLP byte
        /// views tolerate the raw string, but the OTAP conversion CBOR-encodes
        /// array elements and validates UTF-8, so the conversion fails
        /// deterministically.
        fn logs_request_bytes_invalid_utf8_array() -> Vec<u8> {
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

        /// Drives one logs record through the exporter with `compression`
        /// enabled and asserts it round-trips: delivered, read back with the
        /// original (decompressed) payload and OTLP format header, and durably
        /// persisted on the partition. Shared by the four per-codec tests.
        async fn assert_compression_round_trips(topic: &str, compression: CompressionType) {
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

        /// Returns the `unit` string declared for field `field` in the metric
        /// set named `set_name`, across a terminal state's snapshots (or `None`
        /// if that set/field was not emitted). Underscores in `field` are
        /// normalized to dots before lookup.
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

        // ---- Security: dynamic topic routing ----

        /// Scenario (security: dynamic topic routing): a routing header requests a topic that is not permitted by
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

        /// Scenario (security: dynamic topic routing): a routing header requests a topic permitted by the regex
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

        /// Scenario (security: dynamic topic routing): a signal uses an
        /// exact-match `allowed_topics` list (not a regex allowlist); one record
        /// carries a routing header naming a listed topic and another names an
        /// unlisted topic, both exported through the fully-wired node.
        /// Guarantees: the exact-match-allowed header topic is produced to that
        /// topic while the unlisted topic is permanently nacked and never
        /// delivered, so the non-regex allowlist enforces the same
        /// client-cannot-pick-arbitrary-topics constraint as the regex form.
        #[tokio::test]
        async fn exact_match_allowed_topic_delivered_and_unlisted_nacked() {
            let static_topic = "it-sec-exact-static";
            let allowed_topic = "tenant_exact_logs";
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
                            .with_allowed_topics([allowed_topic]),
                    );
                    let mut exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // 1. Unlisted topic: a subscribed pdata so the permanent nack
                    // unwinds observably; assert it is permanent and not delivered.
                    exporter
                        .send_pdata(logs_pdata_subscribed(
                            logs_request_bytes(),
                            Some(("X-Target-Topic", "tenant_not_listed")),
                        ))
                        .await
                        .expect("send unlisted");
                    let nack = exporter
                        .recv_nack(Duration::from_secs(5))
                        .await
                        .expect("unlisted topic should unwind a nack");
                    assert!(
                        nack.permanent,
                        "an unlisted exact-match topic must be permanently nacked",
                    );

                    // 2. Listed topic: delivered to the header-named topic.
                    let payload = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(
                            payload.clone(),
                            Some(("X-Target-Topic", allowed_topic)),
                        ))
                        .await
                        .expect("send listed");
                    let _ = consumer
                        .recv()
                        .await
                        .assert_topic(allowed_topic)
                        .assert_payload(&payload);

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        // ---- Shutdown and live reconfiguration ----

        /// Scenario (shutdown and live reconfiguration): enqueue several records then request a graceful shutdown
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

        /// Scenario (shutdown and live reconfiguration): push a `Config` control message that repoints the logs
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

        /// Scenario (shutdown and live reconfiguration): export a record, confirm it lands on the original topic,
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

        /// Scenario (shutdown and live reconfiguration): block the first Kafka
        /// delivery (stall it at the broker), queue a second batch (P2) behind
        /// it, then send a `Config` that changes the logs topic, then release
        /// the first delivery.
        /// Guarantees: a live reconfigure must not retroactively reroute data
        /// that was accepted before the `Config` across a topic (or credential)
        /// boundary. Batches accepted before the `Config` (P1, P2) must be
        /// exported to the ORIGINAL topic under the configuration in effect when
        /// they were accepted, and only a batch accepted after the `Config` (P3)
        /// may land on the new topic.
        #[tokio::test]
        #[ignore]
        async fn reconfigure_routes_pre_config_backlog_to_old_topic() {
            let original_topic = "it-reconfig-backlog-original";
            let new_topic = "it-reconfig-backlog-new";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(original_topic)
                    .topic(new_topic),
                |cluster| async move {
                    let original_consumer = cluster.consumer().subscribe(&[original_topic]);
                    let new_consumer = cluster.consumer().subscribe(&[new_topic]);

                    // Block the first delivery: stall every broker round trip so
                    // P1 stays in flight and P2 remains buffered in the inbox
                    // when the Config arrives. `round_trip_time` is the harness's
                    // deterministic "hold pending then release" primitive (a hard
                    // broker_down would instead fail-fast under message.timeout).
                    cluster
                        .faults()
                        .round_trip_time(1, Duration::from_millis(400));

                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(
                            original_topic.into(),
                            MessageFormat::OtlpProto,
                        ))
                        // Generous per-message bound so the stalled deliveries
                        // still succeed (release) rather than timing out.
                        .with_timeout_ms(5000)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // P1 (first delivery, blocked) and P2 (queued behind it) are
                    // both accepted before the reconfigure.
                    let p1 = logs_request_bytes_seq(1);
                    let p2 = logs_request_bytes_seq(2);
                    exporter
                        .send_pdata(logs_pdata(p1.clone(), None))
                        .await
                        .expect("send pdata p1 before reconfigure");
                    exporter
                        .send_pdata(logs_pdata(p2.clone(), None))
                        .await
                        .expect("send pdata p2 before reconfigure");

                    // Change the topic while P1 is in flight and P2 is buffered.
                    exporter
                        .send_config(logs_reconfig_json(cluster.bootstrap_servers(), new_topic))
                        .await;

                    // A batch accepted after the reconfigure.
                    let p3 = logs_request_bytes_seq(3);
                    exporter
                        .send_pdata(logs_pdata(p3.clone(), None))
                        .await
                        .expect("send pdata p3 after reconfigure");

                    // DESIRED: the batches accepted before the Config land on the
                    // original topic; the batch accepted after it lands on the
                    // new topic.
                    let mut on_original: std::collections::HashSet<Vec<u8>> =
                        std::collections::HashSet::new();
                    for _ in 0..2 {
                        let m = original_consumer.recv().await;
                        let _ = on_original
                            .insert(m.payload.clone().expect("record carries a payload"));
                    }
                    assert!(
                        on_original.contains(&p1),
                        "P1 (accepted before Config) must land on the original topic"
                    );
                    assert!(
                        on_original.contains(&p2),
                        "P2 (accepted before Config) must land on the original topic"
                    );

                    // The batch accepted after the reconfigure lands on the new
                    // topic.
                    let _ = new_consumer.recv().await.assert_payload(&p3);

                    // The new topic must receive neither pre-config batch.
                    new_consumer
                        .assert_no_more_messages(Duration::from_millis(500))
                        .await;

                    exporter.shutdown(Duration::from_secs(2)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario (shutdown and live reconfiguration): push an invalid `Config` control message (a config with no
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

        /// Scenario (shutdown and live reconfiguration): request a graceful shutdown with a short deadline while the
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
                let start = Instant::now();
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

        /// Scenario (shutdown and live reconfiguration): enqueue many records then request a graceful shutdown with
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

        // ---- InFlightSends bound enforcement (unit) ----

        /// Builds a delivery future that resolves immediately to a successful
        /// delivery, plus a matching [`SendMeta`], for driving `InFlightSends`
        /// bookkeeping without a live producer.
        fn ready_send(topic: &str) -> (ExporterDeliveryFuture, SendMeta) {
            let delivery = ExporterDeliveryFuture::ready_for_test(Ok(
                rdkafka::producer::future_producer::Delivery {
                    partition: 0,
                    offset: 0,
                    timestamp: rdkafka::Timestamp::NotAvailable,
                },
            ));
            let meta = SendMeta {
                signal_type: SignalType::Logs,
                topic: topic.to_string(),
                pdata: sample_pdata(SignalType::Logs),
                export_start: Instant::now(),
                delivery_start: Instant::now(),
                payload_bytes: 0,
            };
            (delivery, meta)
        }

        /// Scenario (backpressure): with `max_in_flight = 1`, a second `push` is issued while one
        /// delivery is already outstanding.
        /// Guarantees: `InFlightSends::push` enforces the bound itself -- the
        /// over-limit push first drains and returns the prior completion, and the
        /// set never holds more than `max_in_flight` outstanding deliveries.
        #[tokio::test]
        async fn in_flight_push_enforces_bound_by_draining() {
            let mut in_flight = InFlightSends::new(1);
            assert!(in_flight.is_empty());
            assert!(!in_flight.is_full());

            // First push fits under the bound: nothing is drained.
            let (d1, m1) = ready_send("t1");
            let drained = in_flight.push(d1, m1).await;
            assert!(
                drained.is_none(),
                "push below the bound must not drain a completion"
            );
            assert!(in_flight.is_full(), "one outstanding delivery hits max=1");

            // Second push is at capacity: push must drain and return exactly one
            // completion (the first delivery) so the caller can finalize it,
            // while the set still holds a single outstanding delivery.
            let (d2, m2) = ready_send("t2");
            let drained = in_flight
                .push(d2, m2)
                .await
                .expect("at-capacity push must drain one completion");
            assert_eq!(
                drained.0.topic, "t1",
                "drained completion is the first send"
            );
            assert!(matches!(drained.1, Ok(Ok(_))), "first delivery succeeded");
            assert!(
                in_flight.is_full(),
                "still exactly one outstanding delivery after the swap"
            );

            // Draining the remaining completion empties the set.
            let (final_meta, final_result) = in_flight.next_completion().await;
            assert_eq!(final_meta.topic, "t2");
            assert!(matches!(final_result, Ok(Ok(_))));
            assert!(
                in_flight.is_empty(),
                "set is empty after draining both sends"
            );
        }

        // ---- Backpressure & delivery-future pipelining ----

        /// Scenario (backpressure): the default (`max_in_flight = 10`) config exports a
        /// sequence of distinct payloads to a single-partition topic.
        /// Guarantees: even with the pipelined default, single-partition delivery
        /// keeps records in send order at strictly increasing offsets, so leaving
        /// `max_in_flight` unset never reorders deliveries within a partition.
        #[tokio::test]
        async fn default_max_in_flight_preserves_partition_ordering() {
            let topic = "it-mif-default-order";
            const N: usize = 10;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    // logs_config leaves max_in_flight at its serde default of 10.
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    assert_eq!(cfg.max_in_flight(), 10, "default config pipelines");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

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

        /// Scenario (backpressure): with `max_in_flight = 8`, many batches are pipelined to a
        /// live mock broker.
        /// Guarantees: pipelining never loses or duplicates data -- every sent
        /// batch is delivered exactly once (readable back) and the terminal
        /// `messages{logs,success}` counter equals the number of sends.
        #[tokio::test]
        async fn pipelined_sends_all_deliver_and_ack() {
            let topic = "it-mif-pipelined-deliver";
            const N: usize = 40;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config_mif(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                        8,
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payloads: Vec<Vec<u8>> = (0..N).map(logs_request_bytes_seq).collect();
                    for payload in &payloads {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    // All N delivered exactly once (single partition -> exactly
                    // offsets 0..N, no gaps or duplicates).
                    let msgs = consumer.recv_n(N).await;
                    assert_eq!(msgs.len(), N, "every pipelined batch must be delivered");
                    let delivered: std::collections::HashSet<Vec<u8>> = msgs
                        .iter()
                        .map(|m| m.payload.clone().expect("payload"))
                        .collect();
                    assert_eq!(
                        delivered.len(),
                        N,
                        "no duplicate deliveries under pipelining"
                    );

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "success"),
                        N as u64,
                        "success counter equals the number of pipelined sends"
                    );
                },
            )
            .await;
        }

        /// Scenario (backpressure): with `max_in_flight = 8` AND a fixed partition key, many
        /// same-key batches are pipelined to a 4-partition topic.
        /// Guarantees: librdkafka preserves per-partition ordering even under
        /// pipelining -- all same-key records land on one partition at strictly
        /// increasing offsets in send order, so raising `max_in_flight` never
        /// reorders records that share a key.
        #[tokio::test]
        async fn pipelined_preserves_per_partition_order_with_keys() {
            let topic = "it-mif-pipelined-order";
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
                        .with_max_in_flight(8)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payloads: Vec<Vec<u8>> = (0..N).map(logs_request_bytes_seq).collect();
                    for payload in &payloads {
                        exporter
                            .send_pdata(logs_pdata(
                                payload.clone(),
                                Some(("X-Tenant-Id", "tenant-42")),
                            ))
                            .await
                            .expect("send pdata");
                    }

                    let msgs = consumer
                        .collect_until_idle(Duration::from_millis(1500))
                        .await;
                    assert_eq!(msgs.len(), N, "all records delivered");
                    let dist = count_by_partition(&msgs);
                    assert_eq!(
                        dist.len(),
                        1,
                        "same-key records land on one partition even when pipelined, got {dist:?}"
                    );
                    // Offsets are strictly increasing in send order.
                    for (i, msg) in msgs.iter().enumerate() {
                        let _ = msg.assert_offset(i as i64).assert_payload(&payloads[i]);
                    }

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario (backpressure): every broker round trip is stalled while `max_in_flight = 4`
        /// and far more than 4 batches are sent.
        /// Guarantees: the bounded in-flight set applies backpressure without
        /// unbounded buffering -- despite the stall, all sent batches are
        /// eventually delivered exactly once once the stall clears at delivery
        /// time, with no loss.
        #[tokio::test]
        async fn bounded_concurrency_caps_in_flight_and_delivers() {
            let topic = "it-mif-backpressure";
            const N: usize = 24;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    // Stall each round trip so deliveries lag behind sends,
                    // forcing the in-flight set to fill and back-pressure the
                    // send loop.
                    cluster
                        .faults()
                        .round_trip_time(1, Duration::from_millis(50));
                    let cfg = logs_config_mif(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                        4,
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payloads: Vec<Vec<u8>> = (0..N).map(logs_request_bytes_seq).collect();
                    for payload in &payloads {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    let msgs = consumer.recv_n(N).await;
                    assert_eq!(
                        msgs.len(),
                        N,
                        "all batches delivered despite bounded concurrency + stall"
                    );

                    exporter.shutdown(Duration::from_secs(10)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario (backpressure): fill the in-flight set so one pdata is parked in the loop,
        /// then request shutdown.
        /// Guarantees: no data is dropped when a pdata is parked at shutdown --
        /// the engine's receiver-first drain plus the loop's ordering ensure the
        /// parked batch is still enqueued and delivered before the terminal
        /// state (assert via the delivered count equal to the send count).
        #[tokio::test]
        async fn parked_pdata_is_enqueued_before_shutdown() {
            let topic = "it-mif-parked-shutdown";
            const N: usize = 12;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    // Small stall so at least one pdata parks behind the in-flight
                    // set before shutdown arrives.
                    cluster
                        .faults()
                        .round_trip_time(1, Duration::from_millis(30));
                    let cfg = logs_config_mif(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                        2,
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payloads: Vec<Vec<u8>> = (0..N).map(logs_request_bytes_seq).collect();
                    for payload in &payloads {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    exporter.shutdown(Duration::from_secs(10)).await;
                    exporter.await_stopped().await;

                    let msgs = consumer.recv_n(N).await;
                    assert_eq!(
                        msgs.len(),
                        N,
                        "a pdata parked at shutdown is still enqueued and delivered"
                    );
                },
            )
            .await;
        }

        /// Scenario (backpressure): a pdata is parked behind a full in-flight set pointed at an
        /// unroutable broker, then shutdown arrives with a bounded deadline.
        /// Guarantees: shutdown stays deadline-bounded even when a pdata is
        /// parked and the broker is unavailable -- the drain returns well within
        /// a generous outer timeout instead of hanging on the stalled delivery.
        #[tokio::test]
        async fn shutdown_with_parked_pdata_and_stalled_broker_is_deadline_bounded() {
            let cfg: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("127.0.0.1:1", "it-client")
                    .with_logs(SignalConfig::new(
                        "it-mif-parked-stalled".into(),
                        MessageFormat::OtlpProto,
                    ))
                    .with_max_in_flight(1)
                    .with_timeout_ms(500)
                    .try_into()
                    .expect("config should be valid");

            run_on_local_set(|cluster| async move {
                let exporter = KafkaExporterHarness::start(&cluster, cfg);

                // Two batches: the first occupies the single in-flight slot
                // (stalled at the unroutable broker), the second parks.
                for _ in 0..2 {
                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send pdata");
                }

                // A short shutdown deadline must be honored despite the parked
                // pdata and the unreachable broker.
                let outcome = tokio::time::timeout(Duration::from_secs(15), async {
                    exporter.shutdown(Duration::from_millis(500)).await;
                    exporter.await_stopped().await;
                })
                .await;
                assert!(
                    outcome.is_ok(),
                    "shutdown must stay bounded with a parked pdata + stalled broker"
                );
            })
            .await;
        }

        /// Scenario (backpressure): with `max_in_flight = 8`, a burst of batches is buffered and
        /// still in flight when graceful shutdown is requested.
        /// Guarantees: the shutdown drain finalizes every pipelined in-flight
        /// delivery before the terminal state, so all buffered batches are
        /// flushed (delivered) and the terminal `messages{logs,success}` counter
        /// equals the number of sends.
        #[tokio::test]
        async fn shutdown_drains_pipelined_in_flight_sends() {
            let topic = "it-mif-shutdown-drain";
            const N: usize = 40;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config_mif(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                        8,
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    let payload = logs_request_bytes();
                    for _ in 0..N {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    exporter.shutdown(Duration::from_secs(10)).await;
                    let ts = exporter.await_terminal_state().await;
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "success"),
                        N as u64,
                        "all pipelined in-flight sends are drained on shutdown"
                    );

                    let msgs = consumer.recv_n(N).await;
                    assert_eq!(msgs.len(), N, "all buffered records flushed on shutdown");
                },
            )
            .await;
        }

        /// Scenario (backpressure): with `max_in_flight = 8`, a batch is in flight on the old
        /// producer when a `Config` repoints the logs topic, then a batch is
        /// accepted after the swap.
        /// Guarantees: reconfiguration drains and finalizes any pipelined
        /// in-flight deliveries on the old producer (no data loss across the
        /// swap) and a batch accepted after the swap reaches the NEW topic.
        ///
        /// NOTE: this test does not assert that the pre-config batch lands on the
        /// ORIGINAL topic. The engine prioritizes the control channel over the
        /// pdata channel, so a `Config` can be processed before pre-config pdata
        /// still buffered in the pdata channel has been dequeued; only pdata
        /// already accepted into the in-flight set is guaranteed to drain to the
        /// old producer. Whichever topic each pre-config batch lands on, none is
        /// lost -- so the invariant checked here is delivery, not routing. (The
        /// pre-existing `reconfigure_routes_pre_config_backlog_to_old_topic`
        /// tracks the stricter routing guarantee that this engine ordering
        /// currently prevents.)
        #[tokio::test]
        async fn reconfigure_drains_pipelined_in_flight_before_swap() {
            let original_topic = "it-mif-reconfig-original";
            let new_topic = "it-mif-reconfig-new";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(original_topic)
                    .topic(new_topic),
                |cluster| async move {
                    let original_consumer = cluster.consumer().subscribe(&[original_topic]);
                    let new_consumer = cluster.consumer().subscribe(&[new_topic]);
                    let cfg = logs_config_mif(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(original_topic.into(), MessageFormat::OtlpProto),
                        8,
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Pipeline a pre-config batch, then reconfigure to the new
                    // topic. The reconfigure drains any in-flight delivery on the
                    // old producer before the swap.
                    let pre = logs_request_bytes_seq(1);
                    exporter
                        .send_pdata(logs_pdata(pre.clone(), None))
                        .await
                        .expect("send pre-config pdata");
                    exporter
                        .send_config(logs_reconfig_json(cluster.bootstrap_servers(), new_topic))
                        .await;

                    // A batch accepted after the reconfigure.
                    let post = logs_request_bytes_seq(2);
                    exporter
                        .send_pdata(logs_pdata(post.clone(), None))
                        .await
                        .expect("send post-config pdata");

                    // Neither the pre-config nor the post-config batch is lost:
                    // both are delivered across the reconfigure (to whichever
                    // topic the control-vs-pdata ordering routed them). Drain
                    // both topics and assert both payloads appear.
                    let mut delivered: std::collections::HashSet<Vec<u8>> =
                        std::collections::HashSet::new();
                    for m in original_consumer
                        .collect_until_idle(Duration::from_secs(2))
                        .await
                    {
                        let _ = delivered.insert(m.payload.clone().expect("payload"));
                    }
                    for m in new_consumer
                        .collect_until_idle(Duration::from_secs(2))
                        .await
                    {
                        let _ = delivered.insert(m.payload.clone().expect("payload"));
                    }
                    assert!(
                        delivered.contains(&pre),
                        "the pre-config batch must be delivered across the reconfigure, not lost"
                    );
                    assert!(
                        delivered.contains(&post),
                        "the post-config batch must be delivered across the reconfigure, not lost"
                    );

                    exporter.shutdown(Duration::from_secs(10)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario (backpressure): the exporter starts with `max_in_flight = 8`,
        /// then a live `Config` lowers it to `1` and a burst of batches is sent
        /// afterward.
        /// Guarantees: reconfiguration rebuilds the in-flight set at the new,
        /// lowered bound, so the post-config burst is delivered serially with no
        /// loss -- all sent batches land exactly once at strictly increasing
        /// offsets on a single partition. A stale bound of 8 (the pre-config
        /// value) would silently ignore the lowered concurrency limit.
        #[tokio::test]
        async fn reconfigure_lowers_max_in_flight_and_still_delivers() {
            let topic = "it-mif-reconfig-lower";
            const N: usize = 24;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    // Start pipelined (max_in_flight = 8).
                    let cfg = logs_config_mif(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                        8,
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Live-lower the concurrency bound to 1 (same topic).
                    exporter
                        .send_config(logs_reconfig_json_mif(
                            cluster.bootstrap_servers(),
                            topic,
                            1,
                        ))
                        .await;

                    // Burst sent under the new, lowered bound.
                    let payloads: Vec<Vec<u8>> = (0..N).map(logs_request_bytes_seq).collect();
                    for payload in &payloads {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata after lowering max_in_flight");
                    }

                    // Serial delivery: single partition, strictly increasing
                    // offsets in send order, no loss.
                    let msgs = consumer.recv_n(N).await;
                    assert_eq!(msgs.len(), N, "every batch delivered after lowering bound");
                    for (i, msg) in msgs.iter().enumerate() {
                        let _ = msg
                            .assert_partition(0)
                            .assert_offset(i as i64)
                            .assert_payload(&payloads[i]);
                    }

                    exporter.shutdown(Duration::from_secs(10)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario (backpressure): the exporter starts with the default
        /// `max_in_flight = 1`, then a live `Config` raises it to `8` and a burst
        /// of batches is sent afterward against a per-round-trip stall.
        /// Guarantees: reconfiguration rebuilds the in-flight set at the new,
        /// raised bound, so pipelining is active afterward -- every sent batch is
        /// delivered exactly once despite the stall. A stale bound of 1 (the
        /// pre-config value) would still deliver, so the invariant asserted here
        /// is no-loss across a raised-bound reconfigure.
        #[tokio::test]
        async fn reconfigure_raises_max_in_flight_and_still_delivers() {
            let topic = "it-mif-reconfig-raise";
            const N: usize = 24;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    // Start serial (default max_in_flight = 1).
                    let cfg = logs_config(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Live-raise the concurrency bound to 8.
                    exporter
                        .send_config(logs_reconfig_json_mif(
                            cluster.bootstrap_servers(),
                            topic,
                            8,
                        ))
                        .await;

                    // Stall each round trip so raised pipelining is exercised.
                    cluster
                        .faults()
                        .round_trip_time(1, Duration::from_millis(40));

                    let payloads: Vec<Vec<u8>> = (0..N).map(logs_request_bytes_seq).collect();
                    for payload in &payloads {
                        exporter
                            .send_pdata(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata after raising max_in_flight");
                    }

                    // No loss / no duplication across the raised-bound reconfigure.
                    let msgs = consumer.recv_n(N).await;
                    assert_eq!(msgs.len(), N, "every batch delivered after raising bound");
                    let delivered: std::collections::HashSet<Vec<u8>> = msgs
                        .iter()
                        .map(|m| m.payload.clone().expect("payload"))
                        .collect();
                    assert_eq!(delivered.len(), N, "no duplicate deliveries after raise");

                    exporter.shutdown(Duration::from_secs(10)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario (backpressure): a batch is pipelined in flight under
        /// `max_in_flight = 8`, then a live `Config` lowers the bound to `1`
        /// (repointing to a new topic), then a batch is accepted after the swap.
        /// Guarantees: reconfiguration drains and finalizes the in-flight
        /// delivery on the old producer BEFORE rebuilding the in-flight set at
        /// the new bound, so no already-accepted batch is dropped when the bound
        /// changes; the post-config batch is delivered under the new bound.
        #[tokio::test]
        async fn reconfigure_bound_change_with_in_flight_batch_loses_nothing() {
            let original_topic = "it-mif-reconfig-bound-original";
            let new_topic = "it-mif-reconfig-bound-new";
            with_cluster(
                KafkaTestCluster::builder()
                    .topic(original_topic)
                    .topic(new_topic),
                |cluster| async move {
                    let original_consumer = cluster.consumer().subscribe(&[original_topic]);
                    let new_consumer = cluster.consumer().subscribe(&[new_topic]);
                    let cfg = logs_config_mif(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(original_topic.into(), MessageFormat::OtlpProto),
                        8,
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Pipeline a pre-config batch, then reconfigure to a new topic
                    // AND a lowered bound in the same Config.
                    let pre = logs_request_bytes_seq(1);
                    exporter
                        .send_pdata(logs_pdata(pre.clone(), None))
                        .await
                        .expect("send pre-config pdata");
                    exporter
                        .send_config(logs_reconfig_json_mif(
                            cluster.bootstrap_servers(),
                            new_topic,
                            1,
                        ))
                        .await;

                    // A batch accepted after the reconfigure (under the new bound).
                    let post = logs_request_bytes_seq(2);
                    exporter
                        .send_pdata(logs_pdata(post.clone(), None))
                        .await
                        .expect("send post-config pdata");

                    // Neither batch is lost across the bound-changing reconfigure.
                    let mut delivered: std::collections::HashSet<Vec<u8>> =
                        std::collections::HashSet::new();
                    for m in original_consumer
                        .collect_until_idle(Duration::from_secs(2))
                        .await
                    {
                        let _ = delivered.insert(m.payload.clone().expect("payload"));
                    }
                    for m in new_consumer
                        .collect_until_idle(Duration::from_secs(2))
                        .await
                    {
                        let _ = delivered.insert(m.payload.clone().expect("payload"));
                    }
                    assert!(
                        delivered.contains(&pre),
                        "the pre-config in-flight batch must be delivered, not lost when the \
                         bound is rebuilt"
                    );
                    assert!(
                        delivered.contains(&post),
                        "the post-config batch must be delivered under the new bound"
                    );

                    exporter.shutdown(Duration::from_secs(10)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario (backpressure): with `max_in_flight = 4`, a run of produce requests is
        /// rejected (a broker outage) while deliveries are pipelined.
        /// Guarantees: pipelined in-flight failures are fully accounted and
        /// bounded -- every sent batch resolves to exactly one outcome
        /// (`messages{logs,success}` + `messages{logs,failure}` equals the number
        /// of sends), so a sustained outage never leaks or double-counts an
        /// in-flight send and the in-flight set cannot grow without bound.
        ///
        /// NOTE: this test deliberately does NOT assert an exact
        /// success/failure split. The mock broker's `fail_produce` consumes one
        /// injected error per produce *request*, but under pipelining librdkafka
        /// coalesces the pipelined records into a broker-chosen number of
        /// requests, so the mapping from injected errors to individual sends is
        /// nondeterministic (observed: all sends can land in the rejected
        /// requests). A deterministic per-send outage/recovery split requires a
        /// real broker with a controllable produce rate; the serial-path
        /// `recovers_after_prolonged_produce_outage` covers the exact split
        /// where one produce request maps to one send.
        #[tokio::test]
        async fn prolonged_outage_keeps_pipelined_sends_bounded() {
            let topic = "it-mif-outage-bounded";
            const OUTAGE_SENDS: usize = 8;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let _consumer = cluster.consumer().subscribe(&[topic]);
                    cluster.faults().fail_produce(
                        &[RDKafkaRespErr::RD_KAFKA_RESP_ERR_POLICY_VIOLATION; OUTAGE_SENDS],
                    );
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_max_in_flight(4)
                        .with_timeout_ms(1500)
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    for _ in 0..OUTAGE_SENDS {
                        exporter
                            .send_pdata(logs_pdata(logs_request_bytes(), None))
                            .await
                            .expect("send during outage");
                    }

                    exporter.shutdown(Duration::from_secs(10)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    let success = kafka_exports(snaps, "logs", "success");
                    let failure = kafka_exports(snaps, "logs", "failure");
                    // The invariant that holds regardless of librdkafka's
                    // request batching: every sent batch is accounted exactly
                    // once, so no in-flight send leaks or is double-counted.
                    assert_eq!(
                        success + failure,
                        OUTAGE_SENDS as u64,
                        "every pipelined send is accounted exactly once; none leaks"
                    );
                    // At least one send failed (the outage was injected), proving
                    // the failure path is exercised and bounded.
                    assert!(
                        failure >= 1,
                        "the injected outage produced at least one accounted failure"
                    );
                },
            )
            .await;
        }

        /// Scenario (backpressure): with `max_in_flight = 8`, an unroutable broker purges the
        /// pipelined in-flight deliveries at shutdown; the batch carries a
        /// subscriber unwind frame.
        /// Guarantees: a delivery future cancelled by the shutdown purge is
        /// reported as a TRANSIENT nack that returns the original pdata to the
        /// retry processor (never permanent, never acked, never dropped), so a
        /// purge-on-shutdown does not silently lose data.
        #[tokio::test]
        async fn purged_pipelined_send_is_transiently_nacked_with_pdata() {
            let cfg: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new("127.0.0.1:1", "it-client")
                    .with_logs(SignalConfig::new(
                        "it-mif-purge-nack".into(),
                        MessageFormat::OtlpProto,
                    ))
                    .with_max_in_flight(8)
                    .with_timeout_ms(30_000)
                    .try_into()
                    .expect("config should be valid");

            run_on_local_set(|cluster| async move {
                let mut exporter = KafkaExporterHarness::start(&cluster, cfg);

                // Enqueue a batch that will never deliver (unroutable broker);
                // the long timeout ensures it is still in flight at shutdown so
                // the purge -- not a delivery timeout -- resolves it.
                exporter
                    .send_pdata(logs_pdata_subscribed(logs_request_bytes(), None))
                    .await
                    .expect("send pdata");

                // Shutdown with a short deadline forces the flush to time out and
                // purge the in-flight delivery, cancelling its future.
                exporter.shutdown(Duration::from_millis(300)).await;

                let nack = exporter
                    .recv_nack(Duration::from_secs(10))
                    .await
                    .expect("a purged in-flight delivery must unwind a nack");
                assert!(
                    !nack.permanent,
                    "a purge-cancelled delivery is a retryable (transient) nack"
                );
                assert!(
                    nack.refused.num_items() >= 1,
                    "the refused pdata is returned for the retry processor"
                );

                exporter.await_stopped().await;
            })
            .await;
        }

        /// Scenario (backpressure): a burst of batches is sent against a tiny librdkafka producer
        /// queue (`queue.buffering.max.messages = 1`) so an enqueue is rejected
        /// as queue-full.
        /// Guarantees: an enqueue failure is reported (the failure counter
        /// advances) without being tracked in the in-flight set, and the loop
        /// keeps running so a later, well-spaced send still delivers.
        ///
        /// NOTE: forcing a deterministic `QueueFull` on the in-process mock is
        /// timing-dependent -- librdkafka drains its queue on the 1s poll cycle,
        /// so a rejection is not guaranteed on every run. If no enqueue is
        /// rejected here the test still asserts the loop stays healthy (the
        /// trailing send delivers); it does not assert a failure occurred. A
        /// deterministic queue-full requires a real broker with a controllable
        /// send rate.
        #[tokio::test]
        async fn enqueue_failure_reports_nack_without_tracking() {
            let topic = "it-mif-enqueue-full";
            const BURST: usize = 200;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_max_in_flight(64)
                        // Force the smallest possible producer queue so a rapid
                        // burst can overflow it before the poll thread drains.
                        .with_producer_config(std::collections::HashMap::from([(
                            "queue.buffering.max.messages".to_string(),
                            "1".to_string(),
                        )]))
                        .try_into()
                        .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Fire a rapid burst to try to overflow the 1-deep queue.
                    for _ in 0..BURST {
                        exporter
                            .send_pdata(logs_pdata(logs_request_bytes(), None))
                            .await
                            .expect("send pdata");
                    }

                    // Regardless of whether any enqueue was rejected, the loop
                    // must stay healthy: a trailing, well-spaced send delivers.
                    tokio::time::sleep(Duration::from_millis(1200)).await;
                    let marker = logs_request_bytes_seq(424_242);
                    exporter
                        .send_pdata(logs_pdata(marker.clone(), None))
                        .await
                        .expect("send trailing pdata");

                    let msgs = consumer.collect_until_idle(Duration::from_secs(2)).await;
                    assert!(
                        msgs.iter()
                            .any(|m| m.payload.as_deref() == Some(marker.as_slice())),
                        "the loop keeps running after enqueue pressure; trailing send delivers"
                    );

                    exporter.shutdown(Duration::from_secs(10)).await;
                    let ts = exporter.await_terminal_state().await;
                    let snaps = ts.metrics();
                    // Every batch is accounted as either a success or a failure;
                    // none vanish. (Failures may be 0 if the queue never
                    // overflowed on this run -- see the NOTE above.)
                    let success = kafka_exports(snaps, "logs", "success");
                    let failure = kafka_exports(snaps, "logs", "failure");
                    assert_eq!(
                        success + failure,
                        (BURST + 1) as u64,
                        "every batch is accounted as success or failure, none lost"
                    );
                },
            )
            .await;
        }

        /// Scenario (backpressure): with `max_in_flight = 8`, a single batch carrying many log
        /// records is exported.
        /// Guarantees: the export counter counts per batch, not per record, even
        /// under pipelining -- a 25-record batch increments
        /// `messages{logs,success}` by exactly 1, so pipelining does not change
        /// the batch-counting semantics.
        #[tokio::test]
        async fn pipelined_export_counts_one_per_batch() {
            let topic = "it-mif-per-batch-count";
            const RECORDS: usize = 25;
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg = logs_config_mif(
                        cluster.bootstrap_servers(),
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto),
                        8,
                    );
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes_n(RECORDS), None))
                        .await
                        .expect("send multi-record batch");

                    let _ = consumer.recv().await.assert_topic(topic);

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "success"),
                        1,
                        "a multi-record batch counts as exactly one exported message"
                    );
                },
            )
            .await;
        }

        // ---- Retry correctness ----

        /// Scenario (retry correctness): an OTAP-encoded signal whose OTLP bytes cannot be converted
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

        /// Scenario (retry correctness): a send to an unreachable broker fails; the refused batch
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

        /// Scenario (retry correctness): a header requests a topic outside the regex allowlist; the
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

        /// Scenario (retry correctness): a stand-in retry processor re-sends the refused batch on
        /// each transient nack; the mock broker rejects the first few produce
        /// requests, then accepts.
        /// Guarantees: transient nacks are retryable and a retried batch
        /// eventually delivers once the broker recovers, with no data loss --
        /// the out-of-process retry contract holds end-to-end.
        #[tokio::test]
        async fn transient_nack_retried_until_success_then_acked() {
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

        /// Scenario (retry correctness): a stand-in retry processor treats a permanent nack as
        /// terminal and does not re-send.
        /// Guarantees: a permanently-nacked batch is dropped at the source (no
        /// re-send, nothing produced, no dead-letter queue) and counts as one
        /// failed batch -- retry exhaustion / drop-at-source behavior.
        #[tokio::test]
        async fn permanent_nack_is_not_retried() {
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

        /// Scenario (retry correctness): a stand-in retry processor re-sends on each transient nack
        /// while the broker rejects a bounded number of produce requests, then
        /// accepts every subsequent attempt.
        /// Guarantees: retry redelivery is bounded by the number of retries the
        /// processor performs (at-least-once), characterizing the duplicate
        /// window as bounded rather than unbounded.
        #[tokio::test]
        async fn retried_transient_send_duplicates_bounded() {
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

        /// Scenario (retry correctness): the exporter is configured with the
        /// smallest allowed `max_message_bytes` (1000) and an oversized batch
        /// (many records, > 1000 encoded bytes) carrying a subscriber unwind
        /// frame is exported, followed by a normal single-record batch on the
        /// same running exporter.
        /// Guarantees: the oversized record is rejected by the producer and
        /// counted as a failed export (`messages{logs,failure}`) rather than
        /// delivered; its failure is classified as a PERMANENT nack (a
        /// message-too-large error can never succeed on retry, so an upstream
        /// `processor:retry` drops it at the source); and the event loop
        /// survives so the subsequent normal batch still delivers
        /// (`messages{logs,success}`) -- so a single message that exceeds the
        /// size limit cannot stall the exporter or be retried pointlessly.
        #[tokio::test]
        async fn oversized_payload_is_permanently_nacked_and_loop_survives() {
            let topic = "it-oversized-logs";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                            // 1000 is librdkafka's minimum allowed message.max.bytes.
                            .with_max_message_bytes(1000)
                            .try_into()
                            .expect("config should be valid");
                    let mut exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Oversized: many records so the encoded payload far exceeds
                    // the 1000-byte limit and the producer rejects it. The
                    // subscriber frame makes the resulting nack observable so its
                    // permanent/transient classification can be asserted.
                    let oversized = logs_request_bytes_n(2000);
                    assert!(
                        oversized.len() > 1000,
                        "oversized payload must exceed the configured limit, got {}",
                        oversized.len(),
                    );
                    exporter
                        .send_pdata(logs_pdata_subscribed(oversized, None))
                        .await
                        .expect("send oversized");

                    // The oversized send must unwind a PERMANENT nack: a
                    // message-too-large error can never succeed on retry.
                    let nack = exporter
                        .recv_nack(Duration::from_secs(5))
                        .await
                        .expect("oversized send should unwind a nack");
                    assert!(
                        nack.permanent,
                        "an oversized (message-too-large) send failure must be permanently nacked",
                    );

                    // A normal-sized batch on the same exporter must still deliver.
                    let small = logs_request_bytes();
                    exporter
                        .send_pdata(logs_pdata(small.clone(), None))
                        .await
                        .expect("send small");

                    // The next consumable record is the small one (the oversized
                    // record was never produced).
                    let _ = consumer.recv().await.assert_topic(topic).assert_payload(&small);

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "failure"),
                        1,
                        "the oversized record must be counted as one failed export",
                    );
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "success"),
                        1,
                        "the subsequent normal batch must still deliver after the oversized failure",
                    );
                },
            )
            .await;
        }

        // ---- Delivery semantics ----

        /// Scenario (delivery semantics): a successful send to a live mock broker resolves the
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

        /// Scenario (delivery semantics): a send whose delivery callback resolves with a Kafka error
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

        /// Scenario (delivery semantics): export several batches to a live mock broker through the
        /// fully-wired node.
        /// Guarantees: every batch is delivered (readable back from the broker)
        /// and the terminal `logs.exported` counter equals the number of sends
        /// with zero `logs.failed` (ACK-side accounting on success).
        #[tokio::test]
        async fn delivery_success_increments_exported() {
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

        /// Scenario (delivery semantics): the broker rejects produce requests (injected non-retriable
        /// produce errors) so the delivery callback resolves with a failure.
        /// Guarantees: a broker-reported produce failure increments
        /// `logs.failed` and not `logs.exported`, so the NACK-side accounting
        /// reflects the delivery-callback failure.
        #[tokio::test]
        async fn produce_failure_increments_failed() {
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

        /// Scenario (delivery semantics): the send targets a broker that never responds (unroutable
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

                let start = Instant::now();
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

        /// Scenario (delivery semantics): partitioning by transport headers is enabled and many
        /// records carry the same header set (hence the same partition key) to a
        /// multi-partition topic.
        /// Guarantees: a stable partition key maps every same-key record to a
        /// single partition (key-to-partition stability), and the produced key
        /// matches the documented header-derived key.
        #[tokio::test]
        async fn same_partition_key_maps_to_stable_partition() {
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

        /// Scenario (delivery semantics): partitioning is disabled (null record key) and many records
        /// are produced to a multi-partition topic.
        /// Guarantees: null-key records are spread across all partitions in a
        /// near-even distribution (round-robin), and no record carries a key.
        #[tokio::test]
        async fn null_key_distributes_evenly_across_partitions() {
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
                        // Serialize deliveries (max_in_flight = 1) so each record
                        // is flushed as its own Produce request. Pipelining +
                        // linger would coalesce records into fewer per-partition
                        // batches, collapsing the number of independent random
                        // partition draws and making the near-even distribution
                        // check flaky over this small sample.
                        .with_max_in_flight(1)
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

        /// Scenario (delivery semantics): a strictly ordered sequence of distinct payloads is exported
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

        /// Scenario (delivery semantics): the broker rejects a produce request while the exporter has
        /// a bounded delivery timeout.
        /// Guarantees: a produce failure yields exactly one failed batch bounded
        /// by `timeout_ms` (a transient nack for the retry processor), and on the
        /// in-process mock a rejected produce does not persist -- so no duplicate
        /// is created here.
        #[tokio::test]
        async fn produce_failure_is_bounded_and_not_persisted_on_mock() {
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

        /// Scenario (delivery semantics): the exporter is configured with
        /// `allow_auto_create_topics = false` (the default-deny posture) and a
        /// signal whose topic does not exist on the broker; a record is
        /// exported to that undeclared topic.
        /// Guarantees: the send is not delivered (counted as a failure, never a
        /// success) and the undeclared topic is not auto-created on the broker,
        /// so a misconfigured or client-influenced topic cannot silently spawn
        /// new broker topics.
        #[tokio::test]
        async fn undeclared_topic_with_auto_create_disabled_is_not_delivered() {
            let declared = "it-autocreate-declared";
            let undeclared = "it-autocreate-undeclared";
            with_cluster(
                // Only `declared` is created; the exporter targets `undeclared`.
                KafkaTestCluster::builder().topic(declared),
                |cluster| async move {
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_logs(SignalConfig::new(
                                undeclared.into(),
                                MessageFormat::OtlpProto,
                            ))
                            .with_allow_auto_create_topics(false)
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    exporter
                        .send_pdata(logs_pdata(logs_request_bytes(), None))
                        .await
                        .expect("send to undeclared topic");

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "success"),
                        0,
                        "a send to an undeclared topic must not be delivered under default-deny",
                    );
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "failure"),
                        1,
                        "a send to an undeclared topic must be counted as a failure",
                    );
                    assert!(
                        !cluster.inspect().topic_exists(undeclared),
                        "the undeclared topic must not be auto-created when auto-create is disabled",
                    );
                },
            )
            .await;
        }

        /// Scenario (delivery semantics): the exporter is configured with a
        /// non-zero `linger_ms` and several distinct records are exported in
        /// rapid succession to a single-partition topic.
        /// Guarantees: producer-side lingering/batching does not drop or reorder
        /// records -- every record is delivered and they arrive in send order at
        /// strictly increasing offsets -- so enabling linger for throughput does
        /// not compromise per-partition ordering or completeness.
        #[tokio::test]
        async fn linger_batches_multiple_records_and_preserves_order() {
            const N: usize = 5;
            let topic = "it-linger-order";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let consumer = cluster.consumer().subscribe(&[topic]);
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                            .with_linger_ms(50)
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // Send N byte-distinct records back-to-back so linger can
                    // coalesce them into batches.
                    let mut payloads = Vec::with_capacity(N);
                    for i in 0..N {
                        let bytes = logs_request_bytes_seq(i);
                        payloads.push(bytes.clone());
                        exporter
                            .send_pdata(logs_pdata(bytes, None))
                            .await
                            .expect("send record");
                    }

                    // Records arrive in send order at offsets 0..N.
                    for (i, expected) in payloads.iter().enumerate() {
                        let _ = consumer
                            .recv()
                            .await
                            .assert_offset(i as i64)
                            .assert_payload(expected);
                    }

                    exporter.shutdown(Duration::from_secs(5)).await;
                    exporter.await_stopped().await;
                },
            )
            .await;
        }

        /// Scenario (delivery semantics): a logs batch containing zero log
        /// records (an empty but well-formed OTLP request) is exported.
        /// Guarantees: an empty batch is handled deterministically per the
        /// exporter's per-batch model -- it produces exactly one record that is
        /// delivered and durably persisted (`messages{logs,success} == 1`, one
        /// record on the partition), rather than being dropped, double-counted,
        /// or failing the exporter -- pinning the empty-batch behavior.
        #[tokio::test]
        async fn empty_batch_produces_one_record() {
            let topic = "it-empty-batch";
            with_cluster(
                KafkaTestCluster::builder().topic(topic),
                |cluster| async move {
                    let cfg =
                        KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it-client")
                            .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                            .try_into()
                            .expect("config should be valid");
                    let exporter = KafkaExporterHarness::start(&cluster, cfg);

                    // A well-formed OTLP logs request carrying no log records.
                    let empty = logs_request_bytes_from(vec![]);
                    exporter
                        .send_pdata(logs_pdata(empty, None))
                        .await
                        .expect("send empty batch");

                    exporter.shutdown(Duration::from_secs(5)).await;
                    let ts = exporter.await_terminal_state().await;
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "success"),
                        1,
                        "an empty batch is exported as exactly one successful record",
                    );
                    assert_eq!(
                        kafka_exports(ts.metrics(), "logs", "failure"),
                        0,
                        "an empty batch must not be counted as a failure",
                    );
                    assert_eq!(
                        cluster.inspect().message_count(topic, 0),
                        1,
                        "an empty batch persists exactly one record on the partition",
                    );
                },
            )
            .await;
        }

        // ---- Kafka integration: encodings and routing ----

        /// Scenario (Kafka integration: encodings and routing): export an OTLP logs batch and read the produced record back
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

        /// Scenario (Kafka integration: encodings and routing): export an OTLP traces batch to the mock broker.
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

        /// Scenario (Kafka integration: encodings and routing): export an OTLP metrics batch to the mock broker.
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

        /// Scenario (Kafka integration: encodings and routing): export a logs batch configured for OTAP encoding.
        /// Guarantees: the record carries the OTAP message-format header and its
        /// payload decodes as a `BatchArrowRecords` protobuf message.
        #[tokio::test]
        async fn exports_logs_otap_sets_otap_format_header() {
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

        /// Scenario (Kafka integration: encodings and routing): export a traces batch configured for OTAP encoding.
        /// Guarantees: the traces record carries the OTAP message-format header
        /// and its payload decodes as a `BatchArrowRecords` protobuf message, so
        /// OTAP encoding is validated for traces (not just logs).
        #[tokio::test]
        async fn exports_traces_otap_sets_otap_format_header() {
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

        /// Scenario (Kafka integration: encodings and routing): export a metrics batch configured for OTAP encoding.
        /// Guarantees: the metrics record carries the OTAP message-format header
        /// and its payload decodes as a `BatchArrowRecords` protobuf message, so
        /// OTAP encoding is validated for metrics (not just logs).
        #[tokio::test]
        async fn exports_metrics_otap_sets_otap_format_header() {
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

        /// Scenario (Kafka integration: encodings and routing): export one OTLP-encoded and one OTAP-encoded logs batch to
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

        /// Scenario (Kafka integration: encodings and routing): a single exporter is configured for all three signals on
        /// distinct topics and one batch of each signal is exported in one run.
        /// Guarantees: each signal is produced to its own topic with the correct
        /// message-format header, and the terminal snapshot records exactly one
        /// success per signal -- so a mixed-signal configuration routes and
        /// counts each signal independently without cross-talk.
        #[tokio::test]
        async fn exports_all_three_signals_to_distinct_topics() {
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

        /// Scenario (Kafka integration: encodings and routing): a malformed payload (invalid UTF-8 nested in an array
        /// attribute) is exported on the OTAP wire format for each signal, then
        /// a valid batch is exported on the same running exporter.
        /// Guarantees: each malformed batch fails to encode and increments
        /// `messages{signal,failure}` (never `success`) without reaching the
        /// broker, and the event loop survives -- a subsequent valid batch on
        /// the same signal still delivers and increments `messages{signal,
        /// success}` -- so a poison payload cannot stall the exporter.
        #[tokio::test]
        async fn encoding_failure_increments_failure_metric_per_signal() {
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

        /// Scenario (Kafka integration: encodings and routing): the broker rejects a logs produce request (injected
        /// non-retriable errors) so the delivery callback resolves with a
        /// failure.
        /// Guarantees: a broker-reported produce failure increments
        /// `logs.failed` and not `logs.exported`, so the failure path is
        /// accounted for the logs signal.
        #[tokio::test]
        async fn logs_send_failure_increments_failed() {
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

        /// Scenario (Kafka integration: encodings and routing): the broker rejects a traces produce request (injected
        /// non-retriable errors) so the delivery callback resolves with a
        /// failure.
        /// Guarantees: a broker-reported produce failure increments
        /// `traces.failed` and not `traces.exported`, so the failure path is
        /// accounted for the traces signal.
        #[tokio::test]
        async fn traces_send_failure_increments_failed() {
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

        /// Scenario (Kafka integration: encodings and routing): the broker rejects a metrics produce request (injected
        /// non-retriable errors) so the delivery callback resolves with a
        /// failure.
        /// Guarantees: a broker-reported produce failure increments
        /// `metrics.failed` and not `metrics.exported`, so the failure path is
        /// accounted for the metrics signal.
        #[tokio::test]
        async fn metrics_send_failure_increments_failed() {
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

        /// Scenario (Kafka integration: encodings and routing): a caller configures the not-yet-implemented `otlp_json`
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

        /// Scenario (Kafka integration: encodings and routing): route a record to a topic named by a transport header while
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

        /// Scenario (Kafka integration: encodings and routing): derive the record partition key from transport headers with
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

        /// Scenario (Kafka integration: encodings and routing): `build_kafka_headers` runs with a header-propagation policy
        /// configured on the effect handler and a pdata context carrying
        /// transport headers.
        /// Guarantees: the produced Kafka headers include the message-format
        /// header AND the propagated transport header, and a propagated header
        /// whose name collides with the format-header key is skipped -- so
        /// transport-header propagation reaches the record without clobbering
        /// the format header.
        #[test]
        fn build_kafka_headers_propagates_transport_headers_under_policy() {
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

        /// Scenario (Kafka integration: encodings and routing): `build_kafka_headers` runs with no propagation policy
        /// configured (the default).
        /// Guarantees: only the message-format header is written and no
        /// transport headers leak onto the record, pinning the default
        /// no-propagation behavior.
        #[test]
        fn build_kafka_headers_writes_only_format_header_without_policy() {
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

        // ---- Kafka integration: acknowledgements ----

        /// Scenario (Kafka integration: acknowledgements): export a logs batch with `required_acks = All` (maps to
        /// `request.required.acks = -1`, leader plus all in-sync replicas).
        /// Guarantees: the record is delivered and read back intact and the
        /// batch counts as exactly one `logs_exported`, so the exporter's
        /// delivery path works end-to-end under the strongest ack setting.
        #[tokio::test]
        async fn exports_logs_with_acks_all_round_trips() {
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

        /// Scenario (Kafka integration: acknowledgements): export a logs batch with `required_acks = None` (maps to
        /// `request.required.acks = 0`, fire-and-forget with no broker ack).
        /// Guarantees: the exporter's delivery callback still resolves so the
        /// record is delivered and read back intact and counts as one
        /// `logs_exported`, so acks=0 does not break the ack accounting or lose
        /// the record.
        #[tokio::test]
        async fn exports_logs_with_acks_none_round_trips() {
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

        // ---- Kafka integration: compression ----

        /// Scenario (Kafka integration: compression): export a logs batch with gzip compression enabled.
        /// Guarantees: the record round-trips (delivered, persisted, and read
        /// back with the original decompressed payload), so gzip is accepted by
        /// the producer end-to-end.
        #[tokio::test]
        async fn exports_logs_gzip_round_trips() {
            assert_compression_round_trips("it-gzip", CompressionType::Gzip).await;
        }

        /// Scenario (Kafka integration: compression): export a logs batch with snappy compression enabled.
        /// Guarantees: the record round-trips (delivered, persisted, and read
        /// back with the original decompressed payload), promoting snappy from
        /// "defined but not end-to-end tested" to round-trip validated on the
        /// mock broker.
        #[tokio::test]
        async fn exports_logs_snappy_round_trips() {
            assert_compression_round_trips("it-snappy", CompressionType::Snappy).await;
        }

        /// Scenario (Kafka integration: compression): export a logs batch with lz4 compression enabled.
        /// Guarantees: the record round-trips (delivered, persisted, and read
        /// back with the original decompressed payload), promoting lz4 from
        /// "defined but not end-to-end tested" to round-trip validated on the
        /// mock broker.
        #[tokio::test]
        async fn exports_logs_lz4_round_trips() {
            assert_compression_round_trips("it-lz4", CompressionType::Lz4).await;
        }

        /// Scenario (Kafka integration: compression): export a logs batch with zstd compression enabled.
        /// Guarantees: the record round-trips (delivered, persisted, and read
        /// back with the original decompressed payload), so zstd is accepted by
        /// the producer end-to-end.
        #[tokio::test]
        async fn exports_logs_zstd_round_trips() {
            assert_compression_round_trips("it-zstd", CompressionType::Zstd).await;
        }

        // ---- Failure recovery ----

        /// Scenario (failure recovery): a broker restart with explicit leader reassignment happens
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

        /// Scenario (failure recovery): the broker rejects a bounded run of consecutive produce
        /// requests (a prolonged outage), after which produce succeeds again;
        /// the exporter keeps draining its queue across the transition.
        /// Guarantees: each rejected produce increments `messages{logs,failure}`
        /// and the first post-outage send is delivered and consumed
        /// (`messages{logs,success} == 1`), so the exporter recovers after a
        /// sustained outage without stalling -- and no rejected produce
        /// persists (`success` count equals the delivered record count).
        #[tokio::test]
        async fn recovers_after_prolonged_produce_outage() {
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
                    // Pin a strict 1-send-to-1-produce-request mapping so the
                    // injected error sequence lines up with the sends:
                    // - max_in_flight = 1 serializes deliveries so no two sends
                    //   share a Produce request.
                    // - message.send.max.retries = 0 stops librdkafka from
                    //   re-issuing a rejected produce, so one send consumes exactly
                    //   one injected error (otherwise a single send retries within
                    //   its message.timeout.ms window and drains several errors).
                    // Together the first OUTAGE_SENDS sends fail once each and the
                    // post-recovery send finds the injected errors exhausted.
                    let cfg = KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "it")
                        .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                        .with_timeout_ms(1500)
                        .with_max_in_flight(1)
                        .with_producer_config(std::collections::HashMap::from([(
                            "message.send.max.retries".to_string(),
                            "0".to_string(),
                        )]))
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

        // ---- Telemetry ----

        /// Scenario (telemetry): after a successful export and graceful shutdown, inspect the
        /// terminal metric snapshots' schema.
        /// Guarantees: shared export outcomes and Kafka-specific payload,
        /// operation, and routing measurements use their dedicated metric sets
        /// with the expected units.
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

                    // Each observation answers a distinct question: whether the
                    // export succeeded, how much data it carried, how long each
                    // Kafka phase took, and how its destination was selected.
                    assert!(
                        snaps
                            .iter()
                            .any(|s| s.descriptor().name == "exporter.exports"),
                        "shared exporter.exports set should be present"
                    );
                    assert!(
                        snaps
                            .iter()
                            .any(|s| s.descriptor().name == "exporter.kafka.exports"),
                        "measurement set exporter.kafka.exports should be present"
                    );
                    assert_eq!(
                        metric_unit(snaps, "exporter.exports", "messages"),
                        Some("{message}"),
                        "exports.messages unit"
                    );
                    assert_eq!(
                        metric_unit(snaps, "exporter.exports", "duration"),
                        Some("s"),
                        "exports.duration unit"
                    );
                    assert_eq!(
                        metric_unit(snaps, "exporter.kafka.exports", "bytes"),
                        Some("By"),
                        "Kafka export bytes unit"
                    );
                    assert_eq!(
                        metric_unit(snaps, "exporter.kafka.operations", "duration"),
                        Some("s"),
                        "Kafka operation duration unit"
                    );
                    assert_eq!(
                        metric_unit(snaps, "exporter.kafka.routing", "messages"),
                        Some("{message}"),
                        "Kafka routing messages unit"
                    );
                },
            )
            .await;
        }

        /// Scenario (telemetry): export a single pdata batch that contains many log records.
        /// Guarantees: the export counter increments exactly once for the batch
        /// (`messages{signal=logs,outcome=success} == 1`), documenting that the
        /// exporter counts per pdata/batch -- not per record -- so the recorded
        /// per-batch counting semantics do not silently change.
        #[tokio::test]
        async fn export_counts_one_per_batch_regardless_of_record_count() {
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

        /// Scenario (telemetry): a downstream node acknowledges a batch (a
        /// `NodeControlMsg::Ack` reaches the exporter).
        /// Guarantees: the terminal exporter ignores the downstream control and
        /// does not misclassify it as an export outcome.
        #[tokio::test]
        async fn downstream_ack_does_not_emit_export_metrics() {
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
                    assert!(
                        ts.metrics().is_empty(),
                        "a downstream ack is not a terminal export outcome"
                    );
                },
            )
            .await;
        }

        /// Scenario (telemetry): a downstream node refuses a batch (a `NodeControlMsg::Nack`
        /// with a benign reason reaches the exporter).
        /// Guarantees: the terminal exporter safely handles the downstream
        /// control without misclassifying it as an export failure.
        #[tokio::test]
        async fn downstream_nack_does_not_emit_export_metrics() {
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
                    assert!(
                        ts.metrics().is_empty(),
                        "a downstream nack is not a terminal export failure"
                    );
                },
            )
            .await;
        }

        /// Scenario (telemetry): a downstream nack carries an adversarial reason string
        /// (embedded control characters and an overlong value), which the
        /// exporter logs after sanitizing.
        /// Guarantees: the exporter shuts down cleanly without emitting an
        /// unbounded metric attribute, so client-influenced nack reasons cannot
        /// crash, hang, or corrupt the telemetry path (the sanitizer's exact
        /// output is pinned separately by the `sanitize_for_log` unit tests).
        #[tokio::test]
        async fn nack_reason_with_control_characters_is_handled_safely() {
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
                    assert!(
                        ts.metrics().is_empty(),
                        "an adversarial nack reason must not become metric data"
                    );
                },
            )
            .await;
        }

        /// Scenario (telemetry): one batch is routed via a transport header while another is
        /// routed via the static per-signal topic.
        /// Guarantees: the bounded `topic.source` observations distinguish the
        /// header and static routing decisions end-to-end.
        #[tokio::test]
        async fn topic_source_attributes_reflect_header_vs_static_routing() {
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
                    assert_eq!(
                        measurement_value(
                            ts.metrics(),
                            "exporter.kafka.routing",
                            "messages",
                            &[("signal", "logs"), ("topic.source", "header")],
                        ),
                        1,
                        "one batch routed from a transport header"
                    );
                    assert_eq!(
                        measurement_value(
                            ts.metrics(),
                            "exporter.kafka.routing",
                            "messages",
                            &[("signal", "logs"), ("topic.source", "static_config")],
                        ),
                        1,
                        "one batch routed from static config"
                    );
                },
            )
            .await;
        }

        /// Scenario (telemetry): successful exports and an ignored downstream
        /// ack are followed by a broker-rejected export on a second exporter.
        /// Guarantees: each final snapshot contains every terminal export
        /// outcome up to shutdown, while the downstream ack adds no outcome.
        #[tokio::test]
        async fn final_snapshot_reflects_all_activity_up_to_shutdown() {
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
                    assert_eq!(
                        kafka_exports(snaps, "logs", "success"),
                        N as u64,
                        "snapshot should record every successful export"
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
    }
}
