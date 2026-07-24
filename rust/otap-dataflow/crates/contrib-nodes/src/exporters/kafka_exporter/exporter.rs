// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core Kafka exporter implementation.
//!
//! ToDo: Currently only handles one kafka message add a time we should
//! improve the throughput by handling delivery futures

use super::producer::{ExporterFutureProducer, ExporterFutureRecord};

use super::config::{KafkaExporterConfig, SignalConfig};
use super::encoder::{self, EncodingError};
use super::metrics::KafkaExporterMetrics;
use super::partitioner;
use super::topic_router::{TopicRouter, TopicRoutingError};
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
use otap_df_telemetry::metrics::MetricSet;
use rdkafka::client::DefaultClientContext;
use rdkafka::config::FromClientConfigAndContext;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::Producer;
use std::sync::Arc;
use std::time::Duration;

/// URN for the Kafka exporter factory registration.
pub const KAFKA_EXPORTER_URN: &str = "urn:otel:exporter:kafka";

/// Errors specific to the Kafka exporter.
#[derive(Debug, thiserror::Error)]
pub enum KafkaExporterError {
    /// Configuration error.
    #[error("Kafka exporter configuration error: {0}")]
    Configuration(String),

    /// Missing topic configuration for a signal type.
    #[error("No topic configured for signal type: {0:?}")]
    MissingTopic(SignalType),

    /// Kafka client error.
    #[error("Kafka client error: {0}")]
    KafkaError(#[from] rdkafka::error::KafkaError),

    /// Encoding error.
    #[error("Encoding error: {0}")]
    Encoding(#[from] EncodingError),

    /// Dynamic topic routing error (e.g. invalid topic supplied via a
    /// transport header).
    #[error("Topic routing error: {0}")]
    TopicRouting(#[from] TopicRoutingError),
}

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
    metrics: MetricSet<KafkaExporterMetrics>,
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

        Ok(Self {
            config,
            producer,
            pdata_producer: PdataProducer::default(),
            metrics: pipeline_ctx.register_metrics::<KafkaExporterMetrics>(),
        })
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

        // Resolve topic via the dynamic topic router *before* doing any encoding
        // work. If a transport header supplied an invalid topic,
        // permanently nack the batch
        let topic = match TopicRouter::resolve(signal_config, &context, &mut self.metrics) {
            Ok(t) => t,
            Err(e) => {
                self.metrics.inc_failed(signal_type);
                let _ = reporter
                    .nack_permanent(e.to_string(), OtapPdata::new(context, payload))
                    .await;
                return Err(KafkaExporterError::TopicRouting(e));
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
                return Err(KafkaExporterError::Encoding(e));
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
                otap_df_telemetry::otel_warn!(
                    "kafka.exporter.send.failed",
                    topic = %topic,
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
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::Control(NodeControlMsg::Ack(_ack)) => {
                    // Track ack receipt without spamming logs
                    self.metrics.inc_ack();
                }
                Message::Control(NodeControlMsg::Nack(nack)) => {
                    // Nack reached end of pipeline, track and log the failure reason
                    self.metrics.inc_nack();
                    effect_handler
                        .info(&format!("Kafka exporter: received Nack - {}", nack.reason))
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
                    return Ok(TerminalState::new(deadline, [self.metrics.snapshot()]));
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
        use crate::common::kafka::MSG_FORMAT_HEADER;
        use crate::exporters::kafka_exporter::config::PartitionerStrategy;
        use crate::exporters::kafka_exporter::config::TlsConfig;
        use crate::exporters::kafka_exporter::partitioner::partition_key_from_transport_headers;
        use bytes::Bytes;
        use otap_df_config::node::NodeUserConfig;
        use otap_df_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
        use otap_df_engine::Interests;
        use otap_df_engine::config::ExporterConfig;
        use otap_df_engine::control::{
            Controllable, NodeControlMsg, PipelineCompletionMsgReceiver, RuntimeCtrlMsgReceiver,
            pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
        };
        use otap_df_engine::exporter::ExporterWrapper;
        use otap_df_engine::local::message::{LocalReceiver, LocalSender};
        use otap_df_engine::message::{Receiver, Sender};
        use otap_df_engine::node::NodeWithPDataReceiver;
        use otap_df_engine::testing::{create_not_send_channel, test_node};
        use otap_df_otap::pdata::Context;
        use otap_df_pdata::OtlpProtoBytes;
        use otap_df_telemetry::reporter::MetricsReporter;
        use prost::Message as _;
        use rdkafka::config::ClientConfig;
        use rdkafka::consumer::{Consumer, StreamConsumer};
        use rdkafka::message::{BorrowedHeaders, Headers, Message as _};
        use rdkafka::mocking::MockCluster;
        use rdkafka::producer::DefaultProducerContext;
        use std::time::Instant;
        use tokio::task::LocalSet;
        use tokio::time::{Duration, timeout};

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
                matches!(result.unwrap_err(), KafkaExporterError::TopicRouting(_)),
                "invalid dynamic topic should surface a TopicRouting error"
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
        // These use `rdkafka::mocking::MockCluster`, an in-process librdkafka mock
        // broker, so the tests run with no Docker/external broker and run by default
        // in CI. Each test drives a fully-wired `KafkaExporter` through the engine's
        // `ExporterWrapper` (control channel, pdata channel, effect handler), then
        // consumes the produced records back from the mock broker to assert on the
        // topic, payload bytes, message-format header, and partition key.
        /// Opaque bundle of channel handles whose lifetimes must outlive the
        /// running exporter (dropping them would close their channels).
        #[allow(dead_code)]
        struct KeepAlive(Vec<Box<dyn std::any::Any>>);

        /// All the plumbing needed to drive an exporter in a test. The
        /// [`ExporterWrapper`] itself is returned separately so it can be moved
        /// into the spawned task while the harness stays borrowed for its
        /// channels.
        struct Harness {
            pdata_tx: Sender<OtapPdata>,
            control_tx: Sender<NodeControlMsg<OtapPdata>>,
            runtime_ctrl_tx: otap_df_engine::control::RuntimeCtrlMsgSender<OtapPdata>,
            completion_tx: otap_df_engine::control::PipelineCompletionMsgSender<OtapPdata>,
            metrics_reporter: MetricsReporter,
            _keep_alive: KeepAlive,
        }

        /// Starts an in-process librdkafka mock cluster
        /// (`rdkafka::mocking::MockCluster`) and pre-creates the given `topics`,
        /// each with a single partition.
        ///
        /// Returns the mock cluster handle (which must stay alive -- and on the
        /// current thread, as it is `!Send` -- for the broker to keep serving)
        /// and its `bootstrap.servers` string.
        fn start_mock_kafka(
            topics: &[&str],
        ) -> (MockCluster<'static, DefaultProducerContext>, String) {
            let mock = MockCluster::new(1).expect("failed to create mock Kafka cluster");
            for topic in topics {
                mock.create_topic(topic, 1, 1)
                    .expect("failed to create topic on mock cluster");
            }
            let brokers = mock.bootstrap_servers();
            (mock, brokers)
        }

        /// Creates a `StreamConsumer` subscribed to `topics` on the mock broker,
        /// used to read back the records the exporter produced.
        fn create_test_consumer(brokers: &str, topics: &[&str]) -> StreamConsumer {
            let consumer: StreamConsumer = ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("group.id", "test-consumer-group")
                .set("auto.offset.reset", "earliest")
                .set("enable.auto.commit", "false")
                .set("session.timeout.ms", "6000")
                .create()
                .expect("failed to create test consumer");
            consumer
                .subscribe(topics)
                .expect("failed to subscribe test consumer");
            consumer
        }

        /// Wires a fully-formed [`KafkaExporter`] into an [`ExporterWrapper`]
        /// (control channel, pdata channel, effect handler) so it can be driven
        /// in a test. Returns the wrapper and the [`Harness`] of channel handles.
        fn wire_exporter_harness(
            config: KafkaExporterConfig,
        ) -> (ExporterWrapper<OtapPdata>, Harness) {
            let pipeline_ctx = pipeline_context();
            let node_config = Arc::new(NodeUserConfig::new_exporter_config(KAFKA_EXPORTER_URN));
            let exporter_config = ExporterConfig::new("test-kafka-exporter");
            let node_id = test_node(exporter_config.name.clone());

            let mut exporter = ExporterWrapper::local(
                KafkaExporter::new(pipeline_ctx, config).expect("kafka exporter config is valid"),
                node_id.clone(),
                node_config,
                &exporter_config,
            );

            let control_tx = exporter.control_sender();

            let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(32);
            let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
            let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
            exporter
                .set_pdata_receiver(node_id, pdata_rx)
                .expect("failed to set pdata receiver");

            let (runtime_ctrl_tx, runtime_ctrl_rx): (_, RuntimeCtrlMsgReceiver<OtapPdata>) =
                runtime_ctrl_msg_channel(16);
            let (completion_tx, completion_rx): (_, PipelineCompletionMsgReceiver<OtapPdata>) =
                pipeline_completion_msg_channel(16);
            let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

            // Keep the read ends alive for the duration of the test; dropping
            // them would close the channels the exporter writes to.
            let keep_alive = KeepAlive(vec![
                Box::new(runtime_ctrl_rx),
                Box::new(completion_rx),
                Box::new(metrics_rx),
            ]);

            let harness = Harness {
                pdata_tx,
                control_tx,
                runtime_ctrl_tx,
                completion_tx,
                metrics_reporter,
                _keep_alive: keep_alive,
            };
            (exporter, harness)
        }

        /// Spawns `exporter.start(...)` on the given current-thread `LocalSet`
        /// (required for the `!Send` mock cluster).
        fn spawn_exporter(
            local: &LocalSet,
            harness: &Harness,
            exporter: ExporterWrapper<OtapPdata>,
        ) {
            let runtime_ctrl_tx = harness.runtime_ctrl_tx.clone();
            let completion_tx = harness.completion_tx.clone();
            let metrics_reporter = harness.metrics_reporter.clone();
            let _handle = local.spawn_local(async move {
                let _ = exporter
                    .start(
                        runtime_ctrl_tx,
                        completion_tx,
                        metrics_reporter,
                        Interests::empty(),
                    )
                    .await;
            });
        }

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

        /// Reads the message-format header value from a consumed record.
        fn format_header_value(headers: &BorrowedHeaders) -> Vec<u8> {
            for i in 0..headers.count() {
                let h = headers.get(i);
                if h.key == MSG_FORMAT_HEADER {
                    return h.value.map(<[u8]>::to_vec).unwrap_or_default();
                }
            }
            panic!("record is missing the {MSG_FORMAT_HEADER} header");
        }

        /// Sends a graceful `Shutdown` control message with a short deadline.
        async fn send_shutdown(control_tx: &Sender<NodeControlMsg<OtapPdata>>) {
            control_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_millis(500),
                    reason: "test done".into(),
                })
                .await
                .expect("send shutdown");
        }

        #[tokio::test]
        async fn exports_logs_otlp_to_mock_broker() {
            let topic = "it-logs-otlp";
            let (_mock, brokers) = start_mock_kafka(&[topic]);
            let consumer = create_test_consumer(&brokers, &[topic]);

            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new(&brokers, "it-client")
                    .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                    .try_into()
                    .expect("config should be valid");

            let (exporter, harness) = wire_exporter_harness(config);
            let payload = logs_request_bytes();

            let local = LocalSet::new();
            local
                .run_until(async {
                    spawn_exporter(&local, &harness, exporter);

                    harness
                        .pdata_tx
                        .send(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    let msg = timeout(Duration::from_secs(30), consumer.recv())
                        .await
                        .expect("timed out consuming record")
                        .expect("consume error");

                    assert_eq!(msg.topic(), topic);
                    assert_eq!(msg.payload().expect("payload"), payload.as_slice());
                    let headers = msg.headers().expect("record should carry headers");
                    assert_eq!(format_header_value(headers), MSG_FORMAT_OTLP);

                    send_shutdown(&harness.control_tx).await;
                })
                .await;
        }

        #[tokio::test]
        async fn exports_traces_otlp_to_mock_broker() {
            let topic = "it-traces-otlp";
            let (_mock, brokers) = start_mock_kafka(&[topic]);
            let consumer = create_test_consumer(&brokers, &[topic]);

            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new(&brokers, "it-client")
                    .with_traces(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                    .try_into()
                    .expect("config should be valid");

            let (exporter, harness) = wire_exporter_harness(config);
            let (pdata, payload) = traces_pdata();

            let local = LocalSet::new();
            local
                .run_until(async {
                    spawn_exporter(&local, &harness, exporter);

                    harness.pdata_tx.send(pdata).await.expect("send pdata");

                    let msg = timeout(Duration::from_secs(30), consumer.recv())
                        .await
                        .expect("timed out consuming record")
                        .expect("consume error");

                    assert_eq!(msg.topic(), topic);
                    assert_eq!(msg.payload().expect("payload"), payload.as_slice());
                    let headers = msg.headers().expect("record should carry headers");
                    assert_eq!(format_header_value(headers), MSG_FORMAT_OTLP);

                    send_shutdown(&harness.control_tx).await;
                })
                .await;
        }

        #[tokio::test]
        async fn exports_metrics_otlp_to_mock_broker() {
            let topic = "it-metrics-otlp";
            let (_mock, brokers) = start_mock_kafka(&[topic]);
            let consumer = create_test_consumer(&brokers, &[topic]);

            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new(&brokers, "it-client")
                    .with_metrics(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                    .try_into()
                    .expect("config should be valid");

            let (exporter, harness) = wire_exporter_harness(config);
            let (pdata, payload) = metrics_pdata();

            let local = LocalSet::new();
            local
                .run_until(async {
                    spawn_exporter(&local, &harness, exporter);

                    harness.pdata_tx.send(pdata).await.expect("send pdata");

                    let msg = timeout(Duration::from_secs(30), consumer.recv())
                        .await
                        .expect("timed out consuming record")
                        .expect("consume error");

                    assert_eq!(msg.topic(), topic);
                    assert_eq!(msg.payload().expect("payload"), payload.as_slice());
                    let headers = msg.headers().expect("record should carry headers");
                    assert_eq!(format_header_value(headers), MSG_FORMAT_OTLP);

                    send_shutdown(&harness.control_tx).await;
                })
                .await;
        }

        #[tokio::test]
        async fn exports_logs_otap_sets_otap_format_header() {
            use otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords;

            let topic = "it-logs-otap";
            let (_mock, brokers) = start_mock_kafka(&[topic]);
            let consumer = create_test_consumer(&brokers, &[topic]);

            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new(&brokers, "it-client")
                    .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtapProto))
                    .try_into()
                    .expect("config should be valid");

            let (exporter, harness) = wire_exporter_harness(config);
            let payload = logs_request_bytes();

            let local = LocalSet::new();
            local
                .run_until(async {
                    spawn_exporter(&local, &harness, exporter);

                    harness
                        .pdata_tx
                        .send(logs_pdata(payload.clone(), None))
                        .await
                        .expect("send pdata");

                    let msg = timeout(Duration::from_secs(30), consumer.recv())
                        .await
                        .expect("timed out consuming record")
                        .expect("consume error");

                    assert_eq!(msg.topic(), topic);
                    let headers = msg.headers().expect("record should carry headers");
                    assert_eq!(format_header_value(headers), MSG_FORMAT_OTAP);

                    // OTAP payload must decode as a BatchArrowRecords wire message.
                    let decoded = BatchArrowRecords::decode(msg.payload().expect("payload"));
                    assert!(
                        decoded.is_ok(),
                        "OTAP payload should decode as BatchArrowRecords"
                    );

                    send_shutdown(&harness.control_tx).await;
                })
                .await;
        }

        #[tokio::test]
        async fn routes_to_topic_from_transport_header() {
            let static_topic = "it-static-topic";
            let dynamic_topic = "it-dynamic-topic";
            let (_mock, brokers) = start_mock_kafka(&[static_topic, dynamic_topic]);
            let consumer = create_test_consumer(&brokers, &[dynamic_topic]);

            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new(&brokers, "it-client")
                    .with_logs(
                        SignalConfig::new(static_topic.into(), MessageFormat::OtlpProto)
                            .with_topic_from_transport_header("x-target-topic"),
                    )
                    .try_into()
                    .expect("config should be valid");

            let (exporter, harness) = wire_exporter_harness(config);
            let payload = logs_request_bytes();

            let local = LocalSet::new();
            local
                .run_until(async {
                    spawn_exporter(&local, &harness, exporter);

                    harness
                        .pdata_tx
                        .send(logs_pdata(
                            payload.clone(),
                            Some(("X-Target-Topic", dynamic_topic)),
                        ))
                        .await
                        .expect("send pdata");

                    // The consumer only subscribes to the dynamic topic, so
                    // receiving a record proves header-based routing worked.
                    let msg = timeout(Duration::from_secs(30), consumer.recv())
                        .await
                        .expect("timed out consuming record")
                        .expect("consume error");
                    assert_eq!(msg.topic(), dynamic_topic);

                    send_shutdown(&harness.control_tx).await;
                })
                .await;
        }

        #[tokio::test]
        async fn sets_partition_key_from_transport_headers() {
            let topic = "it-partition-key";
            let (_mock, brokers) = start_mock_kafka(&[topic]);
            let consumer = create_test_consumer(&brokers, &[topic]);

            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new(&brokers, "it-client")
                    .with_logs(
                        SignalConfig::new(topic.into(), MessageFormat::OtlpProto)
                            .with_partition_by_transport_headers(true),
                    )
                    .with_partitioning_strategy(PartitionerStrategy::Murmur2Random)
                    .try_into()
                    .expect("config should be valid");

            let (exporter, harness) = wire_exporter_harness(config);
            let payload = logs_request_bytes();

            // Compute the expected key from the same transport header the pdata carries.
            let pdata = logs_pdata(payload, Some(("X-Tenant-Id", "tenant-123")));
            let expected_key = {
                let (context, _payload) = pdata.clone().into_parts();
                let headers = context
                    .transport_headers()
                    .expect("pdata should carry transport headers");
                partition_key_from_transport_headers(headers)
                    .expect("headers should produce a partition key")
            };

            let local = LocalSet::new();
            local
                .run_until(async {
                    spawn_exporter(&local, &harness, exporter);

                    harness.pdata_tx.send(pdata).await.expect("send pdata");

                    let msg = timeout(Duration::from_secs(30), consumer.recv())
                        .await
                        .expect("timed out consuming record")
                        .expect("consume error");

                    let key = msg.key().expect("record should carry a partition key");
                    assert_eq!(key, expected_key.as_bytes());

                    send_shutdown(&harness.control_tx).await;
                })
                .await;
        }

        #[tokio::test]
        async fn shutdown_flushes_buffered_records() {
            let topic = "it-shutdown-flush";
            let (_mock, brokers) = start_mock_kafka(&[topic]);
            let consumer = create_test_consumer(&brokers, &[topic]);

            let config: KafkaExporterConfig =
                KafkaExporterConfigBuilder::new(&brokers, "it-client")
                    .with_logs(SignalConfig::new(topic.into(), MessageFormat::OtlpProto))
                    .try_into()
                    .expect("config should be valid");

            let (exporter, harness) = wire_exporter_harness(config);
            let payload = logs_request_bytes();

            let local = LocalSet::new();
            local
                .run_until(async {
                    spawn_exporter(&local, &harness, exporter);

                    // Enqueue several records, then request a graceful shutdown
                    // with a generous deadline. The exporter awaits each
                    // delivery inline, so all records must be consumable
                    // afterward.
                    for _ in 0..3 {
                        harness
                            .pdata_tx
                            .send(logs_pdata(payload.clone(), None))
                            .await
                            .expect("send pdata");
                    }

                    harness
                        .control_tx
                        .send(NodeControlMsg::Shutdown {
                            deadline: Instant::now() + Duration::from_secs(10),
                            reason: "flush test".into(),
                        })
                        .await
                        .expect("send shutdown");

                    for i in 0..3 {
                        let msg = timeout(Duration::from_secs(30), consumer.recv())
                            .await
                            .unwrap_or_else(|_| panic!("timed out consuming record {i}"))
                            .unwrap_or_else(|_| panic!("consume error for record {i}"));
                        assert_eq!(msg.topic(), topic);
                        assert_eq!(msg.payload().expect("payload"), payload.as_slice());
                    }
                })
                .await;
        }
    }
}
