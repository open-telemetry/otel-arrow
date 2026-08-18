// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Multi-signal OTLP JSON Lines file exporter.
//!
//! The exporter encodes OTLP-byte or OTAP Arrow pdata through backend-agnostic views, then writes
//! one bounded frame to a lazily opened signal-specific file. The local run loop owns all state so
//! slow filesystems propagate backpressure and completion follows the engine ACK/NACK contract.

mod config;
mod encoding;
mod metrics;
mod writer;

pub use config::{Durability, FileExporterConfig, FileFormat, OpenMode, TailRecovery};

use async_trait::async_trait;
use config::RenderedPaths;
use encoding::{FrameEncodeError, encode_logs, encode_metrics, encode_traces};
use linkme::distributed_slice;
use metrics::{
    FileExporterExportMetrics, FileFailureMetrics, FileOperation, FileSignalMetrics,
    SignalOperationAttributes,
};
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ExporterErrorKind};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::views::otap::{OtapLogsView, OtapMetricsView, OtapTracesView};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otap_df_pdata::views::otlp::bytes::traces::RawTraceData;
use otap_df_pdata::{OtapPayload, OtapPayloadHelpers};
use otap_df_telemetry::attributes::AttributeEnum as _;
use otap_df_telemetry::common_attributes::{Outcome, SignalAttributes, SignalOutcomeAttributes};
use otap_df_telemetry::metrics::MeasurementMetricSet;
use otap_df_telemetry::{otel_error, otel_info, otel_warn};
use std::sync::Arc;
use tokio::time::Instant;
use writer::{SignalWriter, WriterFailure};

/// Component URN for the file exporter.
pub const FILE_EXPORTER_URN: &str = "urn:otel:exporter:file";

/// OTLP JSON file exporter with one lazily opened writer per signal.
pub struct FileExporter {
    config: FileExporterConfig,
    paths: RenderedPaths,
    writers: [Option<SignalWriter>; 3],
    frame: Vec<u8>,
    failure_active: [bool; 3],
    export_metrics: MeasurementMetricSet<FileExporterExportMetrics>,
    signal_metrics: MeasurementMetricSet<FileSignalMetrics>,
    failure_metrics: MeasurementMetricSet<FileFailureMetrics>,
}

/// Declares the file exporter as a local exporter factory.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static FILE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: FILE_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        let config = FileExporterConfig::parse(&node_config.config)?;
        let paths = config.render_paths(pipeline.core_id(), pipeline.deployment_generation())?;
        let exporter = FileExporter {
            frame: Vec::with_capacity(config.max_frame_bytes.min(8 * 1024)),
            config,
            paths,
            writers: std::array::from_fn(|_| None),
            failure_active: [false; 3],
            export_metrics: FileExporterExportMetrics::register(&pipeline),
            signal_metrics: FileSignalMetrics::register(&pipeline),
            failure_metrics: FileFailureMetrics::register(&pipeline),
        };
        Ok(ExporterWrapper::local(
            exporter,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |value| FileExporterConfig::parse(value).map(|_| ()),
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for FileExporter {
    async fn start(
        mut self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!(
            "otelcol.node.file.start",
            format = self.config.format.as_str(),
            create_directories = self.config.create_directories,
            open_mode = self.config.open_mode.as_str(),
            durability = self.config.durability.as_str(),
            tail_recovery = self
                .config
                .effective_tail_recovery()
                .map_or("disabled", |recovery| recovery.as_str()),
            max_frame_bytes = self.config.max_frame_bytes,
        );
        loop {
            match inbox.recv().await? {
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report_measurement(&mut self.export_metrics);
                    _ = metrics_reporter.report_measurement(&mut self.signal_metrics);
                    _ = metrics_reporter.report_measurement(&mut self.failure_metrics);
                }
                Message::Control(NodeControlMsg::Config { .. }) => {}
                Message::Control(NodeControlMsg::Shutdown { deadline, reason }) => {
                    self.finalize(deadline, &effect_handler).await?;
                    otel_info!("otelcol.node.file.stop", reason = reason.as_str(),);
                    let mut snapshots = self.export_metrics.terminal_snapshots();
                    snapshots.extend(self.signal_metrics.terminal_snapshots());
                    snapshots.extend(self.failure_metrics.terminal_snapshots());
                    return Ok(TerminalState::new(deadline, snapshots));
                }
                Message::PData(pdata) => {
                    self.export_pdata(pdata, &effect_handler).await?;
                }
                _ => {}
            }
        }
    }
}

impl FileExporter {
    async fn export_pdata(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let signal = pdata.signal_type();
        if pdata.is_empty() {
            effect_handler.notify_ack(AckMsg::new(pdata)).await?;
            self.record_export_outcome(signal, Outcome::Success);
            return Ok(());
        }
        if let Err(error) = encode_payload(
            pdata.payload_ref(),
            &mut self.frame,
            self.config.max_frame_bytes,
        ) {
            self.record_export_outcome(signal, Outcome::Failure);
            let reason = match error {
                EncodeFailure::Frame(FrameEncodeError::FrameTooLarge { .. }) => {
                    "file exporter frame exceeds max_frame_bytes; split the batch upstream"
                        .to_owned()
                }
                error => format!("file exporter rejected invalid pdata: {error}"),
            };
            effect_handler
                .notify_nack(NackMsg::new_permanent(&reason, pdata))
                .await?;
            return Ok(());
        }
        if let Err(failure) = self.ensure_writer(signal).await {
            self.record_io_failure(signal, &failure);
            self.record_export_outcome(signal, Outcome::Failure);
            self.log_write_failure(signal, &failure);
            effect_handler
                .notify_nack(NackMsg::new("file writer is not ready", pdata))
                .await?;
            return Ok(());
        }
        let index = signal_index(signal);
        let Some(writer) = self.writers[index].as_mut() else {
            self.record_export_outcome(signal, Outcome::Failure);
            effect_handler
                .notify_nack(NackMsg::new("file writer unavailable after open", pdata))
                .await?;
            return Err(exporter_error(
                effect_handler,
                ExporterErrorKind::Other,
                "file writer missing after successful open",
            ));
        };
        if let Err(failure) = writer.write_frame(&self.frame).await {
            self.record_io_failure(signal, &failure);
            self.record_export_outcome(signal, Outcome::Failure);
            self.log_write_failure(signal, &failure);
            let fatal = failure.rollback_error.is_some();
            effect_handler
                .notify_nack(NackMsg::new(
                    if fatal {
                        "file write failed and rollback left file state indeterminate"
                    } else {
                        "file write failed and was rolled back"
                    },
                    pdata,
                ))
                .await?;
            if fatal {
                return Err(exporter_error(
                    effect_handler,
                    ExporterErrorKind::Transport,
                    "file write rollback failed",
                ));
            }
            return Ok(());
        }
        self.failure_active[index] = false;
        self.signal_metrics
            .with(SignalAttributes { signal })
            .items
            .add(pdata.payload_ref().num_items() as u64);
        self.signal_metrics
            .with(SignalAttributes { signal })
            .bytes
            .add(self.frame.len() as u64);
        self.record_export_outcome(signal, Outcome::Success);
        effect_handler.notify_ack(AckMsg::new(pdata)).await?;
        Ok(())
    }

    async fn ensure_writer(&mut self, signal: SignalType) -> Result<(), WriterFailure> {
        let index = signal_index(signal);
        if self.writers[index].is_some() {
            return Ok(());
        }
        let (writer, recovery) = SignalWriter::open(self.paths.get(signal), &self.config).await?;
        if recovery.recovered_bytes != 0 {
            self.signal_metrics
                .with(SignalAttributes { signal })
                .tail_recoveries
                .inc();
            self.signal_metrics
                .with(SignalAttributes { signal })
                .tail_recovered_bytes
                .add(recovery.recovered_bytes);
            otel_warn!(
                "otelcol.node.file.tail.recover",
                signal = signal.as_str(),
                recovered_bytes = recovery.recovered_bytes,
                message = "Removed an incomplete final OTLP JSON frame"
            );
        }
        self.writers[index] = Some(writer);
        self.failure_active[index] = false;
        otel_info!("otelcol.node.file.writer.start", signal = signal.as_str(),);
        Ok(())
    }

    fn record_export_outcome(&mut self, signal: SignalType, outcome: Outcome) {
        self.export_metrics
            .with(SignalOutcomeAttributes { signal, outcome })
            .messages
            .inc();
    }

    fn record_io_failure(&mut self, signal: SignalType, failure: &WriterFailure) {
        self.failure_metrics
            .with(SignalOperationAttributes {
                signal,
                operation: failure.operation,
            })
            .failures
            .inc();
        if failure.rollback_error.is_some() {
            self.failure_metrics
                .with(SignalOperationAttributes {
                    signal,
                    operation: FileOperation::Rollback,
                })
                .failures
                .inc();
        }
    }

    fn log_write_failure(&mut self, signal: SignalType, failure: &WriterFailure) {
        let index = signal_index(signal);
        if let Some(rollback_error) = failure.rollback_error.as_deref() {
            otel_error!(
                "otelcol.node.file.rollback.fail",
                signal = signal.as_str(),
                operation = failure.operation.as_str(),
                error = failure.error.as_str(),
                rollback_error = rollback_error,
                message = "File write rollback failed"
            );
        } else if !self.failure_active[index] {
            otel_warn!(
                "otelcol.node.file.operation.fail",
                signal = signal.as_str(),
                operation = failure.operation.as_str(),
                error = failure.error.as_str(),
                message = "File writer entered a failure state"
            );
        }
        self.failure_active[index] = true;
    }

    async fn finalize(
        &mut self,
        deadline: std::time::Instant,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let result = tokio::time::timeout_at(Instant::from_std(deadline), async {
            let mut failures = Vec::new();
            for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
                let writer = &mut self.writers[signal_index(signal)];
                if let Some(writer) = writer
                    && let Err(failure) = writer.finalize().await
                {
                    failures.push((signal, failure));
                }
            }
            failures
        })
        .await;
        match result {
            Ok(failures) if failures.is_empty() => Ok(()),
            Ok(failures) => {
                for (signal, failure) in failures {
                    self.record_io_failure(signal, &failure);
                    self.log_write_failure(signal, &failure);
                }
                Err(exporter_error(
                    effect_handler,
                    ExporterErrorKind::Shutdown,
                    "file synchronization failed during shutdown",
                ))
            }
            Err(_) => Err(exporter_error(
                effect_handler,
                ExporterErrorKind::Shutdown,
                "file synchronization exceeded the shutdown deadline",
            )),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum EncodeFailure {
    #[error("{0}")]
    View(String),
    #[error(transparent)]
    Frame(#[from] FrameEncodeError),
}

fn encode_payload(
    payload: &OtapPayload,
    frame: &mut Vec<u8>,
    max_frame_bytes: usize,
) -> Result<(), EncodeFailure> {
    frame.clear();
    match payload {
        OtapPayload::OtlpBytes(bytes) => match bytes {
            otap_df_pdata::OtlpProtoBytes::ExportLogsRequest(_) => {
                let view = RawLogsData::try_from(bytes)
                    .map_err(|error| EncodeFailure::View(error.to_string()))?;
                encode_logs(&view, frame, max_frame_bytes)?;
            }
            otap_df_pdata::OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                let view = RawMetricsData::try_new(bytes)
                    .map_err(|error| EncodeFailure::View(error.to_string()))?;
                encode_metrics(&view, frame, max_frame_bytes)?;
            }
            otap_df_pdata::OtlpProtoBytes::ExportTracesRequest(bytes) => {
                let view = RawTraceData::try_new(bytes)
                    .map_err(|error| EncodeFailure::View(error.to_string()))?;
                encode_traces(&view, frame, max_frame_bytes)?;
            }
        },
        OtapPayload::OtapArrowRecords(records) => match records.signal_type() {
            SignalType::Logs => {
                let view = OtapLogsView::try_from(records)
                    .map_err(|error| EncodeFailure::View(error.to_string()))?;
                encode_logs(&view, frame, max_frame_bytes)?;
            }
            SignalType::Metrics => {
                let view = OtapMetricsView::try_from(records)
                    .map_err(|error| EncodeFailure::View(error.to_string()))?;
                encode_metrics(&view, frame, max_frame_bytes)?;
            }
            SignalType::Traces => {
                let view = OtapTracesView::try_from(records)
                    .map_err(|error| EncodeFailure::View(error.to_string()))?;
                encode_traces(&view, frame, max_frame_bytes)?;
            }
        },
    }
    Ok(())
}

const fn signal_index(signal: SignalType) -> usize {
    match signal {
        SignalType::Logs => 0,
        SignalType::Metrics => 1,
        SignalType::Traces => 2,
    }
}

fn exporter_error(
    effect_handler: &EffectHandler<OtapPdata>,
    kind: ExporterErrorKind,
    error: impl Into<String>,
) -> Error {
    Error::ExporterError {
        exporter: effect_handler.exporter_id(),
        kind,
        error: error.into(),
        source_detail: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::Interests;
    use otap_df_engine::control::PipelineCompletionMsg;
    use otap_df_engine::testing::exporter::{
        TestContext, TestRuntime, create_exporter_from_factory,
    };
    use otap_df_otap::testing::{TestCallData, create_empty_test_pdata};
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::encode::{
        encode_logs_otap_batch, encode_metrics_otap_batch, encode_spans_otap_batch,
    };
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{Metric, ResourceMetrics, ScopeMetrics};
    use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use serde_json::json;
    use std::time::{Duration, Instant as StdInstant};
    use tempfile::tempdir;

    fn encoded<M: prost::Message>(message: &M) -> Vec<u8> {
        let mut bytes = Vec::new();
        message.encode(&mut bytes).unwrap();
        bytes
    }

    fn pdata<M: prost::Message>(signal: SignalType, message: M) -> OtapPdata {
        let bytes = encoded(&message);
        OtapPdata::new_default(OtlpProtoBytes::new_from_bytes(signal, bytes).into())
    }

    fn log_request() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        event_name: "ready".to_owned(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn log_pdata() -> OtapPdata {
        pdata(SignalType::Logs, log_request())
    }

    fn metric_request() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
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
        }
    }

    fn metric_pdata() -> OtapPdata {
        pdata(SignalType::Metrics, metric_request())
    }

    fn trace_request() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                scope_spans: vec![ScopeSpans {
                    spans: vec![Span {
                        name: "GET /ready".to_owned(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn trace_pdata() -> OtapPdata {
        pdata(SignalType::Traces, trace_request())
    }

    async fn assert_permanent_nack(ctx: &mut TestContext<OtapPdata>, expected_reason: &str) {
        let mut completion_receiver = ctx.take_pipeline_completion_receiver().unwrap();
        let completion = tokio::time::timeout(Duration::from_secs(3), completion_receiver.recv())
            .await
            .expect("timed out waiting for file exporter NACK")
            .expect("completion channel closed before file exporter NACK");
        match completion {
            PipelineCompletionMsg::DeliverNack { nack } => {
                assert!(nack.permanent);
                assert!(nack.reason.contains(expected_reason), "{}", nack.reason);
            }
            PipelineCompletionMsg::DeliverAck { .. } => panic!("expected a permanent NACK"),
        }
    }

    /// Scenario: OTAP Arrow batches for logs, metrics, and traces enter the common payload path.
    /// Guarantees: Every OTAP signal is framed as one JSON object with the matching top-level key.
    #[test]
    fn encode_payload_accepts_all_otap_signal_views() {
        let logs_bytes = encoded(&log_request());
        let metrics_bytes = encoded(&metric_request());
        let traces_bytes = encoded(&trace_request());
        let logs = RawLogsData::try_new(&logs_bytes).unwrap();
        let metrics = RawMetricsData::try_new(&metrics_bytes).unwrap();
        let traces = RawTraceData::try_new(&traces_bytes).unwrap();
        let payloads = [
            OtapPayload::OtapArrowRecords(encode_logs_otap_batch(&logs).unwrap()),
            OtapPayload::OtapArrowRecords(encode_metrics_otap_batch(&metrics).unwrap()),
            OtapPayload::OtapArrowRecords(encode_spans_otap_batch(&traces).unwrap()),
        ];
        let expected_fields = ["resourceLogs", "resourceMetrics", "resourceSpans"];
        let mut frame = Vec::new();
        for (payload, expected_field) in payloads.iter().zip(expected_fields) {
            encode_payload(payload, &mut frame, 4096).unwrap();
            let value: serde_json::Value = serde_json::from_slice(&frame).unwrap();
            assert!(value.get(expected_field).is_some());
        }
    }

    /// Scenario: A malformed non-empty OTLP protobuf payload reaches view validation.
    /// Guarantees: Validation fails without retaining bytes from an earlier encoded frame.
    #[test]
    fn malformed_otlp_payload_clears_the_reusable_frame() {
        let payload =
            OtapPayload::OtlpBytes(OtlpProtoBytes::new_from_bytes(SignalType::Logs, vec![0x80]));
        let mut frame = b"previous telemetry\n".to_vec();
        assert!(encode_payload(&payload, &mut frame, 4096).is_err());
        assert!(frame.is_empty());
    }

    /// Scenario: One exporter instance receives non-empty logs, metrics, and traces in sequence.
    /// Guarantees: Each signal produces exactly one replayable JSON line in its exclusive file.
    #[test]
    fn exporter_writes_one_otlp_json_line_per_signal_batch() {
        let dir = tempdir().unwrap();
        let template = dir
            .path()
            .join("capture-{signal}-{core_id}-{generation}.jsonl");
        let exporter = create_exporter_from_factory(
            &FILE_EXPORTER,
            json!({"path": template.to_string_lossy()}),
        )
        .unwrap();
        let paths = [
            dir.path().join("capture-logs-0-0.jsonl"),
            dir.path().join("capture-metrics-0-0.jsonl"),
            dir.path().join("capture-traces-0-0.jsonl"),
        ];
        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                for pdata in [log_pdata(), metric_pdata(), trace_pdata()] {
                    ctx.send_pdata(pdata).await.unwrap();
                }
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(10), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |_, result| async move {
                result.unwrap();
                let expected_fields = ["resourceLogs", "resourceMetrics", "resourceSpans"];
                for (path, expected_field) in paths.iter().zip(expected_fields) {
                    let content = tokio::fs::read_to_string(path).await.unwrap();
                    assert_eq!(content.lines().count(), 1);
                    let value: serde_json::Value = serde_json::from_str(&content).unwrap();
                    assert!(value.get(expected_field).is_some());
                }
            });
    }

    /// Scenario: An empty pdata batch is delivered before any signal writer has opened.
    /// Guarantees: The exporter ACKs and shuts down without creating an unused signal file.
    #[test]
    fn empty_pdata_does_not_create_a_file() {
        let dir = tempdir().unwrap();
        let template = dir
            .path()
            .join("capture-{signal}-{core_id}-{generation}.jsonl");
        let exporter = create_exporter_from_factory(
            &FILE_EXPORTER,
            json!({"path": template.to_string_lossy()}),
        )
        .unwrap();
        let logs_path = dir.path().join("capture-logs-0-0.jsonl");
        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                ctx.send_pdata(create_empty_test_pdata()).await.unwrap();
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(10), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |_, result| async move {
                result.unwrap();
                assert!(!logs_path.exists());
            });
    }

    /// Scenario: A malformed non-empty protobuf batch is delivered before a writer opens.
    /// Guarantees: The exporter permanently rejects the input without creating a signal file.
    #[test]
    fn invalid_pdata_does_not_create_a_file() {
        let dir = tempdir().unwrap();
        let template = dir
            .path()
            .join("capture-{signal}-{core_id}-{generation}.jsonl");
        let exporter = create_exporter_from_factory(
            &FILE_EXPORTER,
            json!({"path": template.to_string_lossy()}),
        )
        .unwrap();
        let logs_path = dir.path().join("capture-logs-0-0.jsonl");
        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let pdata = OtapPdata::new_default(
                    OtlpProtoBytes::new_from_bytes(SignalType::Logs, vec![0x80]).into(),
                )
                .test_subscribe_to(
                    Interests::NACKS,
                    TestCallData::default().into(),
                    123,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(10), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                assert_permanent_nack(&mut ctx, "invalid pdata").await;
                assert!(!logs_path.exists());
            });
    }

    /// Scenario: A valid batch exceeds the configured complete-frame byte limit.
    /// Guarantees: The exporter permanently rejects the frame without creating a signal file.
    #[test]
    fn oversized_pdata_does_not_create_a_file() {
        let dir = tempdir().unwrap();
        let template = dir
            .path()
            .join("capture-{signal}-{core_id}-{generation}.jsonl");
        let exporter = create_exporter_from_factory(
            &FILE_EXPORTER,
            json!({"path": template.to_string_lossy(), "max_frame_bytes": 1}),
        )
        .unwrap();
        let logs_path = dir.path().join("capture-logs-0-0.jsonl");
        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let pdata = log_pdata().test_subscribe_to(
                    Interests::NACKS,
                    TestCallData::default().into(),
                    123,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(10), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                assert_permanent_nack(&mut ctx, "exceeds max_frame_bytes").await;
                assert!(!logs_path.exists());
            });
    }
}
