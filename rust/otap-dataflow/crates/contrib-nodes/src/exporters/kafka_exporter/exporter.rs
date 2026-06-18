// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core Kafka exporter implementation.

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
/// Supports dynamic topic routing via transport headers and resource attributes,
/// with a priority hierarchy: transport header > static topic.
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
            tracing::warn!(
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
    /// Uses the [`TopicRouter`] to resolve the destination topic(s):
    /// 1. Transport header (highest priority)
    /// 2. Resource attribute (may split the batch)
    /// 3. Static per-signal topic (fallback)
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

        // Send to Kafka with timeout
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

        // Main event loop
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
                Message::Control(NodeControlMsg::Shutdown { .. }) => {
                    effect_handler.info("Shutting down Kafka exporter").await;
                    _ = timer_cancel_handle.cancel().await;
                    break;
                }
                Message::Control(_) => {
                    // Ignore other control messages
                }
            }
        }

        // Flush any pending messages
        effect_handler.info("Flushing Kafka producer").await;
        self.producer
            .flush(Duration::from_secs(5))
            .map_err(|e| EngineError::InternalError {
                message: format!("Failed to flush Kafka producer: {}", e),
            })?;

        effect_handler.info("Kafka exporter stopped").await;
        Ok(TerminalState::default())
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
            name: header_wire_name.to_lowercase().replace('-', "_"),
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
    mod unit_tests {
        use super::*;
        use crate::exporters::kafka_exporter::config::TlsConfig;

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
                            .with_topic_from_transport_header("x_target_topic"),
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
    }
}
