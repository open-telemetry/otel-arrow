// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Clickhouse Exporter for OTAP records
//!
//! This exporter sends OTAP records to ClickHouse instances using the official
//! ClickHouse Rust client.
//! It implements the `Exporter<OtapPdata>` trait
//! for integration with the OTAP dataflow engine.
//!
//! ## Usage
//!
//! This exporter is automatically discovered by the `data-plane` binary via `linkme`.
//! Users configure it in YAML:
//!
//! ```yaml
//! nodes:
//!   - id: clickhouse-exporter
//!     urn: "urn:otel:exporter:clickhouse"
//!     config:
//!       # `endpoint` is an HTTP(S) URL pointing at the ClickHouse HTTP interface (default port 8123).
//!       endpoint: "http://clickhouse.example.db:8123"
//!       database: "otap"
//!       username: "default"
//!       # Secrets can be sourced from the environment, e.g. "${env:CLICKHOUSE_PASSWORD}".
//!       password: ""
//!       # ... additional config
//! ```

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = CLICKHOUSE_EXPORTER_URN,
    target = "otel.exporter.clickhouse",
);

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::validation::validate_typed_config;
use otel_arrow_dfe_config::{SignalFormat, SignalType};
use otel_arrow_dfe_engine::config::ExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::{Error, ExporterErrorKind, format_error_sources};
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::metrics::ExporterExportMetrics;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::error::Error as PdataError;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::{
    OtapArrowRecords, OtapPayload, OtlpProtoBytes, PayloadData, TryIntoWithOptions,
};
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otel_arrow_dfe_telemetry::metrics::MetricSetHandler;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::exporters::clickhouse_exporter::config::{RuntimeConfig, UserConfig};
use crate::exporters::clickhouse_exporter::in_flight::CompletedWrite;
use crate::exporters::clickhouse_exporter::metrics::ClickhouseExporterMetrics;
use crate::exporters::clickhouse_exporter::transform::logs_fast::{
    LogsFastTransform, LogsFastTransformer,
};
use crate::exporters::clickhouse_exporter::transform::logs_otlp::OtlpLogsTransformer;
use crate::exporters::clickhouse_exporter::transform::transform_batch::BatchTransformer;
use crate::exporters::clickhouse_exporter::write_lanes::{DispatcherEvent, WriteDispatcher};
use crate::exporters::clickhouse_exporter::writer::ClickHouseWriter;

mod arrays;
#[cfg(feature = "clickhouse-exporter-bench")]
#[doc(hidden)]
pub mod bench_support;
mod config;
mod consts;
mod error;
mod in_flight;
mod metrics;
mod schema;
mod tables;
mod transform;
mod write_lanes;
mod writer;

/// The URN for the Clickhouse exporter
pub const CLICKHOUSE_EXPORTER_URN: &str = "urn:otel:exporter:clickhouse";

/// The list of all payloads that we intend to handle. There seems to be no way to iterate the enum
/// during module setup (we need to generate a static transform plan for all payloads bassed on configs)
const SUPPORTED_ARROW_PAYLOAD_TYPES: &[ArrowPayloadType] = &[
    ArrowPayloadType::ResourceAttrs,
    ArrowPayloadType::ScopeAttrs,
    ArrowPayloadType::Logs,
    ArrowPayloadType::LogAttrs,
    ArrowPayloadType::Spans,
    ArrowPayloadType::SpanAttrs,
    ArrowPayloadType::SpanEventAttrs,
    ArrowPayloadType::SpanEvents,
    ArrowPayloadType::SpanLinkAttrs,
    ArrowPayloadType::SpanLinks,
    // TODO: [support_new_signal] add payload names here
];

/// Clickhouse exporter that sends OTAP data to Clickhouse backend
pub struct ClickhouseExporter {
    config: RuntimeConfig,
    pdata_metrics: MeasurementMetricSet<ExporterExportMetrics>,
    ch_metrics: MetricSet<ClickhouseExporterMetrics>,
}

impl ClickhouseExporter {
    /// Create a new Clickhouse exporter from configuration
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otel_arrow_dfe_config::error::Error> {
        let ch_metrics = pipeline_ctx.register_metrics::<ClickhouseExporterMetrics>();
        let pdata_metrics = ExporterExportMetrics::register(&pipeline_ctx);

        let user_config: UserConfig = serde_json::from_value(config.clone()).map_err(|e| {
            otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        let runtime_config = RuntimeConfig::from_user_config(user_config);

        Ok(Self {
            config: runtime_config,
            pdata_metrics,
            ch_metrics,
        })
    }

    /// Get exporter configuration
    #[must_use]
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    fn terminal_state(
        deadline: Instant,
        mut pdata_metrics: MeasurementMetricSet<ExporterExportMetrics>,
        ch_metrics: MetricSet<ClickhouseExporterMetrics>,
    ) -> TerminalState {
        let mut snapshots = Vec::new();

        snapshots.extend(pdata_metrics.terminal_snapshots());
        if ch_metrics.needs_flush() {
            snapshots.push(ch_metrics.snapshot());
        }

        TerminalState::new(deadline, snapshots)
    }

    async fn finalize_write(
        &mut self,
        completed: CompletedWrite,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let CompletedWrite {
            pdata,
            export_started_at,
            result,
        } = completed;
        let signal_type = pdata.signal_type();

        match result {
            Ok(written_rows) => {
                for (payload_type, rows) in written_rows {
                    self.ch_metrics.add(rows, payload_type);
                }
                self.pdata_metrics
                    .with(SignalOutcomeAttributes {
                        signal: signal_type,
                        outcome: Outcome::Success,
                    })
                    .record(export_started_at.elapsed());
                effect_handler.notify_ack(AckMsg::new(pdata)).await?;
            }
            Err(error) => {
                self.pdata_metrics
                    .with(SignalOutcomeAttributes {
                        signal: signal_type,
                        outcome: Outcome::Failure,
                    })
                    .record(export_started_at.elapsed());
                otel_warn!(
                    "clickhouse.exporter.write.error",
                    message = format!("Error writing batch to clickhouse: {error}"),
                    signal_type = format!("{signal_type:?}"),
                );
                effect_handler
                    .notify_nack(NackMsg::new(error.to_string(), pdata))
                    .await?;
            }
        }
        Ok(())
    }
}

fn transform_raw_otlp_logs(
    payload: &OtapPayload,
    transformer: &mut OtlpLogsTransformer,
) -> Option<Result<Option<arrow::array::RecordBatch>, error::ClickhouseExporterError>> {
    match payload.data() {
        PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) => {
            Some(transformer.transform(bytes))
        }
        _ => None,
    }
}

fn is_invalid_protobuf(error: &error::ClickhouseExporterError) -> bool {
    matches!(
        error,
        error::ClickhouseExporterError::Child(PdataError::InvalidProtobufWireFormat)
    )
}

/// Returns true when a non-empty signal cannot currently be persisted by this exporter.
fn is_unsupported_non_empty_signal(pdata: &OtapPdata) -> bool {
    pdata.signal_type() == SignalType::Metrics && !pdata.is_empty()
}

/// Returns a deterministic data rejection to the nearest interested upstream node.
async fn notify_permanent_rejection(
    effect_handler: &EffectHandler<OtapPdata>,
    reason: String,
    pdata: OtapPdata,
) -> Result<(), Error> {
    effect_handler
        .notify_nack(NackMsg::new_permanent(reason, pdata))
        .await
}

/// Register Clickhouse exporter with the OTAP exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static CLICKHOUSE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: CLICKHOUSE_EXPORTER_URN,
    create:
        |pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         exporter_config: &ExporterConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            Ok(ExporterWrapper::local(
                ClickhouseExporter::from_config(pipeline, &node_config.config)?,
                node,
                node_config,
                exporter_config,
            ))
        },
    validate_config: validate_typed_config::<UserConfig>,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ClickhouseExporter {
    async fn start(
        mut self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let exporter_id = effect_handler.exporter_id();
        otel_info!(
            "clickhouse.exporter.start",
            message = "Clickhouse exporter starting",
            endpoint = self.config.endpoint,
            database = self.config.database,
            username = self.config.username,
            max_in_flight = self.config.max_in_flight.get(),
            insert_batching_enabled = self.config.insert_batching.is_some(),
        );

        let mut batch_transformer = BatchTransformer::new();
        let mut logs_fast_transformer = LogsFastTransformer::default();
        let mut otlp_logs_transformer = OtlpLogsTransformer::default();
        let clickhouse_writer =
            ClickHouseWriter::new(&self.config)
                .await
                .map_err(|e| Error::ExporterError {
                    exporter: exporter_id.clone(),
                    kind: ExporterErrorKind::Connect,
                    error: format!("clickhouse writer initialization error: {e}"),
                    source_detail: format_error_sources(&e),
                })?;
        let mut write_dispatcher = WriteDispatcher::new(
            clickhouse_writer,
            self.config.max_in_flight,
            self.config.insert_batching,
        );

        // Start periodic telemetry collection (internal metrics)
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        // Message loop
        loop {
            let accepting_pdata = !write_dispatcher.is_at_capacity();
            let has_pending_writes = write_dispatcher.has_pending();
            let message = tokio::select! {
                biased;

                event = write_dispatcher.next_event(), if has_pending_writes => {
                    if let Some(DispatcherEvent::Completed(completed)) = event {
                        self.finalize_write(completed, &effect_handler).await?;
                    }
                    continue;
                }
                message = inbox.recv_when(accepting_pdata) => message?,
            };

            match message {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    otel_info!(
                        "clickhouse.exporter.shutdown",
                        message = "Clickhouse exporter shutting down",
                    );
                    let shutdown_deadline = tokio::time::Instant::from_std(deadline);
                    write_dispatcher.flush_pending();
                    let abandoned = loop {
                        if !write_dispatcher.has_pending() {
                            break 0;
                        }
                        match write_dispatcher.next_event_until(shutdown_deadline).await {
                            Ok(Some(DispatcherEvent::Completed(completed))) => {
                                self.finalize_write(completed, &effect_handler).await?;
                            }
                            Ok(Some(DispatcherEvent::CapacityAvailable)) => {}
                            Ok(None) => break 0,
                            Err(abandoned) => break abandoned,
                        }
                    };
                    if abandoned > 0 {
                        otel_warn!(
                            "clickhouse.exporter.shutdown.deadline_exceeded",
                            message = "ClickHouse writes abandoned at the shutdown deadline",
                            abandoned_writes = abandoned,
                        );
                    }
                    let _ = telemetry_cancel_handle.cancel().await;
                    return Ok(Self::terminal_state(
                        deadline,
                        self.pdata_metrics,
                        self.ch_metrics,
                    ));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report_measurement(&mut self.pdata_metrics);
                    _ = metrics_reporter.report(&mut self.ch_metrics);
                }
                Message::PData(pdata) => {
                    let export_started_at = Instant::now();
                    let signal_type = pdata.signal_type();
                    let signal_format = pdata.signal_format();

                    if is_unsupported_non_empty_signal(&pdata) {
                        let reason =
                            "ClickHouse exporter does not support non-empty metrics payloads"
                                .to_owned();
                        self.pdata_metrics
                            .with(SignalOutcomeAttributes {
                                signal: signal_type,
                                outcome: Outcome::Failure,
                            })
                            .record(export_started_at.elapsed());
                        otel_warn!(
                            "clickhouse.exporter.signal.unsupported",
                            message = reason.clone(),
                            signal_type = format!("{signal_type:?}"),
                        );
                        notify_permanent_rejection(&effect_handler, reason, pdata).await?;
                        continue;
                    }

                    let payload = pdata.payload_ref().clone();

                    let direct_otlp_batches =
                        match transform_raw_otlp_logs(&payload, &mut otlp_logs_transformer) {
                            Some(result) => match result {
                                Ok(batch) => {
                                    self.ch_metrics.record_log_otlp_direct_path();
                                    Some(
                                        batch
                                            .map(|batch| {
                                                HashMap::from([(ArrowPayloadType::Logs, batch)])
                                            })
                                            .unwrap_or_default(),
                                    )
                                }
                                Err(error) if is_invalid_protobuf(&error) => {
                                    let reason = error.to_string();
                                    self.pdata_metrics
                                        .with(SignalOutcomeAttributes {
                                            signal: signal_type,
                                            outcome: Outcome::Failure,
                                        })
                                        .record(export_started_at.elapsed());
                                    otel_warn!(
                                        "clickhouse.exporter.otlp.invalid_protobuf",
                                        message = "Rejecting malformed raw OTLP logs.",
                                        error = reason.clone(),
                                    );
                                    notify_permanent_rejection(&effect_handler, reason, pdata)
                                        .await?;
                                    continue;
                                }
                                Err(error) => {
                                    self.ch_metrics.record_log_otlp_transform_fallback();
                                    otel_debug!(
                                        "clickhouse.exporter.otlp.transform.fallback",
                                        message =
                                            "Using the legacy ClickHouse transform for OTLP logs.",
                                        reason = error.to_string(),
                                    );
                                    None
                                }
                            },
                            _ => None,
                        };

                    let write_batches = if let Some(batches) = direct_otlp_batches {
                        batches
                    } else {
                        let mut arrow_records: OtapArrowRecords = match payload
                            .try_into_with_default()
                        {
                            Ok(arrow_records) => arrow_records,
                            Err(e) => {
                                let reason =
                                    format!("Failed to convert payload to OtapArrowRecords: {e:?}");
                                self.pdata_metrics
                                    .with(SignalOutcomeAttributes {
                                        signal: signal_type,
                                        outcome: Outcome::Failure,
                                    })
                                    .record(export_started_at.elapsed());
                                otel_warn!(
                                    "clickhouse.exporter.convert.error",
                                    message = reason.clone(),
                                    signal_type = format!("{:?}", signal_type),
                                );
                                notify_permanent_rejection(&effect_handler, reason, pdata).await?;
                                continue;
                            }
                        };

                        // Decode transport-optimized IDs before joining payloads against them.
                        if let Err(e) = arrow_records.decode_transport_optimized_ids() {
                            let reason = format!("Failed to decode transport optimized IDs: {e}");
                            self.pdata_metrics
                                .with(SignalOutcomeAttributes {
                                    signal: signal_type,
                                    outcome: Outcome::Failure,
                                })
                                .record(export_started_at.elapsed());
                            otel_warn!(
                                "clickhouse.exporter.decode.error",
                                message = reason.clone(),
                                source_detail = format_error_sources(&e),
                                signal_type = format!("{:?}", signal_type),
                            );
                            notify_permanent_rejection(&effect_handler, reason, pdata).await?;
                            continue;
                        }

                        let transform_result = if signal_type == SignalType::Logs
                            && signal_format == SignalFormat::OtapRecords
                        {
                            match logs_fast_transformer.try_apply(&arrow_records) {
                                Ok(LogsFastTransform::Applied(batch)) => {
                                    self.ch_metrics.record_log_fast_path();
                                    Ok(HashMap::from([(ArrowPayloadType::Logs, batch)]))
                                }
                                Ok(LogsFastTransform::NotApplicable(_)) => {
                                    self.ch_metrics.record_log_transform_fallback();
                                    batch_transformer.apply_plan(arrow_records)
                                }
                                Err(error) => Err(error),
                            }
                        } else {
                            if signal_type == SignalType::Logs {
                                self.ch_metrics.record_log_transform_fallback();
                            }
                            batch_transformer.apply_plan(arrow_records)
                        };

                        match transform_result {
                            Ok(batches) => batches,
                            Err(e) => {
                                let reason = format!("Error transforming batch for export: {e}");
                                self.pdata_metrics
                                    .with(SignalOutcomeAttributes {
                                        signal: signal_type,
                                        outcome: Outcome::Failure,
                                    })
                                    .record(export_started_at.elapsed());
                                otel_warn!(
                                    "clickhouse.exporter.transform.error",
                                    message = "Error transforming batch for export.",
                                    error = e.to_string(),
                                    signal_type = format!("{:?}", signal_type),
                                );
                                notify_permanent_rejection(&effect_handler, reason, pdata).await?;
                                continue;
                            }
                        }
                    };
                    if let Some(completed) =
                        write_dispatcher.submit(pdata, export_started_at, write_batches)
                    {
                        self.finalize_write(completed, &effect_handler).await?;
                    }
                }
                _ => {
                    // Ignore other messages
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_engine::control::{PipelineCompletionMsg, pipeline_completion_msg_channel};
    use otel_arrow_dfe_engine::testing::test_node;
    use otel_arrow_dfe_otap::testing::{TestCallData, create_test_pdata};
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Metric, ResourceMetrics, ScopeMetrics,
    };
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use prost::Message as _;
    use serde_json::json;

    fn test_exporter() -> ClickhouseExporter {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline =
            controller.pipeline_context_with("test-group".into(), "test-pipeline".into(), 0, 1, 0);
        ClickhouseExporter::from_config(
            pipeline,
            &json!({
                "endpoint": "http://localhost:8123",
                "database": "otel",
                "username": "default",
                "password": ""
            }),
        )
        .expect("create test exporter")
    }

    fn completion_harness() -> (
        EffectHandler<OtapPdata>,
        otel_arrow_dfe_engine::control::PipelineCompletionMsgReceiver<OtapPdata>,
    ) {
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let mut effect_handler =
            EffectHandler::new(test_node("clickhouse-completion-test"), metrics_reporter);
        let (completion_tx, completion_rx) = pipeline_completion_msg_channel(4);
        effect_handler.set_pipeline_completion_msg_sender(completion_tx);
        (effect_handler, completion_rx)
    }

    fn subscribed_logs_pdata() -> OtapPdata {
        create_test_pdata().test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            TestCallData::default().into(),
            42,
        )
    }

    fn metrics_pdata(request: ExportMetricsServiceRequest) -> OtapPdata {
        let mut bytes = Vec::new();
        request.encode(&mut bytes).expect("encode metrics request");
        OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes)).into())
    }

    fn non_empty_metrics_pdata() -> OtapPdata {
        metrics_pdata(ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "requests".to_owned(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })
    }

    /// Scenario: the ClickHouse exporter is registered with its public component URN.
    /// Guarantees: configuration references continue to resolve to the exporter factory.
    #[test]
    fn test_urn_constant() {
        assert_eq!(CLICKHOUSE_EXPORTER_URN, "urn:otel:exporter:clickhouse");
    }

    /// Scenario: the exporter receives serialized OTLP logs or another payload representation.
    /// Guarantees: only serialized OTLP log requests are selected for direct transformation.
    #[test]
    fn raw_otlp_log_routing_is_signal_and_format_specific() {
        let logs = OtapPayload::from(OtlpProtoBytes::ExportLogsRequest(Bytes::new()));
        let traces = OtapPayload::from(OtlpProtoBytes::ExportTracesRequest(Bytes::new()));
        let mut transformer = OtlpLogsTransformer::default();

        assert!(transform_raw_otlp_logs(&logs, &mut transformer).is_some());
        assert!(transform_raw_otlp_logs(&traces, &mut transformer).is_none());
    }

    /// Scenario: a raw OTLP logs request has malformed top-level protobuf framing.
    /// Guarantees: the routing layer classifies it as invalid instead of using legacy fallback.
    #[test]
    fn malformed_raw_otlp_logs_are_not_fallback_candidates() {
        let logs = OtapPayload::from(OtlpProtoBytes::ExportLogsRequest(Bytes::from_static(
            b"\xff",
        )));
        let mut transformer = OtlpLogsTransformer::default();
        let error = transform_raw_otlp_logs(&logs, &mut transformer)
            .expect("raw logs should select direct transformation")
            .expect_err("malformed top-level protobuf must fail");

        assert!(is_invalid_protobuf(&error));
    }

    /// Scenario: the exporter classifies non-empty metrics, empty metrics, and logs before
    /// transformation.
    /// Guarantees: only non-empty metrics are rejected as an unsupported signal.
    #[test]
    fn only_non_empty_metrics_are_unsupported() {
        let non_empty_metrics = non_empty_metrics_pdata();
        let empty_metrics = metrics_pdata(ExportMetricsServiceRequest::default());
        let logs = create_test_pdata();

        assert!(is_unsupported_non_empty_signal(&non_empty_metrics));
        assert!(!is_unsupported_non_empty_signal(&empty_metrics));
        assert!(!is_unsupported_non_empty_signal(&logs));
    }

    /// Scenario: a deterministic ClickHouse payload rejection is returned to an interested
    /// upstream node.
    /// Guarantees: the emitted NACK is permanent and preserves the rejection reason.
    #[tokio::test]
    async fn deterministic_rejection_emits_permanent_nack() {
        let (effect_handler, mut completion_rx) = completion_harness();
        let pdata = non_empty_metrics_pdata().test_subscribe_to(
            Interests::NACKS,
            TestCallData::default().into(),
            42,
        );

        notify_permanent_rejection(&effect_handler, "unsupported metrics".to_owned(), pdata)
            .await
            .expect("emit permanent NACK");

        match completion_rx.recv().await.expect("receive completion") {
            PipelineCompletionMsg::DeliverNack { nack } => {
                assert!(nack.permanent);
                assert_eq!(nack.reason, "unsupported metrics");
            }
            PipelineCompletionMsg::DeliverAck { .. } => panic!("expected permanent NACK"),
        }
    }

    /// Scenario: a completed ClickHouse insertion succeeds for a subscribed logs batch.
    /// Guarantees: finalization emits an ACK after recording the written rows.
    #[tokio::test]
    async fn successful_write_emits_ack() {
        let mut exporter = test_exporter();
        let (effect_handler, mut completion_rx) = completion_harness();

        exporter
            .finalize_write(
                CompletedWrite {
                    pdata: subscribed_logs_pdata(),
                    export_started_at: Instant::now(),
                    result: Ok(vec![(ArrowPayloadType::Logs, 1)]),
                },
                &effect_handler,
            )
            .await
            .expect("finalize successful write");

        assert!(matches!(
            completion_rx.recv().await.expect("receive completion"),
            PipelineCompletionMsg::DeliverAck { .. }
        ));
    }

    /// Scenario: a ClickHouse insertion request fails after transformation completed.
    /// Guarantees: finalization emits a retryable NACK because insertion failures may be
    /// transient.
    #[tokio::test]
    async fn insertion_failure_emits_retryable_nack() {
        let mut exporter = test_exporter();
        let (effect_handler, mut completion_rx) = completion_harness();

        exporter
            .finalize_write(
                CompletedWrite {
                    pdata: subscribed_logs_pdata(),
                    export_started_at: Instant::now(),
                    result: Err(error::ClickhouseExporterError::InsertResponseError {
                        error: "temporary failure".to_owned(),
                    }),
                },
                &effect_handler,
            )
            .await
            .expect("finalize failed write");

        match completion_rx.recv().await.expect("receive completion") {
            PipelineCompletionMsg::DeliverNack { nack } => assert!(!nack.permanent),
            PipelineCompletionMsg::DeliverAck { .. } => panic!("expected retryable NACK"),
        }
    }
}
