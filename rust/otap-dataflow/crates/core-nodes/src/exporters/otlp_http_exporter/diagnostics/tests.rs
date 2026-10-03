// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::exporters::otlp_http_exporter::{
    CompletedExport, ServiceRequestError, finalize_completed_export,
    metrics::OtlpHttpExporterMetrics, notify_nack_with_diagnostics,
};
use bytes::Bytes;
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::control::{
    NackMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
};
use otel_arrow_dfe_engine::local::exporter::EffectHandler;
use otel_arrow_dfe_engine::testing::node::test_node;
use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
use otel_arrow_dfe_otap::metrics::ErrorWithOutcome;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_otap::testing::TestCallData;
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogRecord;
use otel_arrow_dfe_pdata_views::views::common::{AnyValueView, AttributeView, ValueType};
use otel_arrow_dfe_pdata_views::views::logs::LogRecordView;
use otel_arrow_dfe_telemetry::diagnostics::SignalSet;
use otel_arrow_dfe_telemetry::event::{LogEvent, ObservedEvent, ObservedEventReporter};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry::self_tracing::LogContext;
use otel_arrow_dfe_telemetry::tracing_init::{ProviderSetup, TracingSetup};
use serde_json::{Map, Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::Level;
use tracing::field::{Field, Visit};

/// Build a test detail `LogRecord` whose body is `message`.
fn detail(message: &str) -> Option<LogRecord> {
    otel_diagnostic_warn!("test.diagnostic.detail", message = message)
}

#[derive(Debug)]
struct CapturedEvent {
    name: &'static str,
    target: &'static str,
    level: Level,
    fields: Map<String, Value>,
    /// Decoded body text, present only for events captured from a retained
    /// `LogRecord` delivered via the splice-and-log fast path (Degraded and
    /// Summary reports). Recovered reports are still captured as a live
    /// `tracing::Event` and carry their text in the `error` field instead.
    body: Option<String>,
}

impl Visit for CapturedEvent {
    fn record_str(&mut self, field: &Field, value: &str) {
        _ = self.fields.insert(field.name().into(), json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        _ = self.fields.insert(field.name().into(), json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        _ = self.fields.insert(field.name().into(), json!(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        _ = self.fields.insert(field.name().into(), json!(value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        _ = self
            .fields
            .insert(field.name().into(), json!(format!("{value:?}")));
    }
}

impl CapturedEvent {
    /// Decode a retained `LogRecord` (delivered via the splice-and-log fast
    /// path for Degraded/Summary reports) into the same shape used for
    /// tracing-captured events, via the same OTLP bytes view/decoder path
    /// used elsewhere (e.g. `DiagnosticReport::detail_str`).
    fn from_log_record(record: &LogRecord) -> Self {
        let metadata = record.callsite_id.0.metadata();
        let raw = RawLogRecord::new(&record.body_attrs_bytes);
        let mut fields = Map::new();
        for attr in raw.attributes() {
            let Ok(key) = std::str::from_utf8(attr.key()) else {
                continue;
            };
            let Some(value) = attr.value() else { continue };
            let json_value = match value.value_type() {
                ValueType::String => value
                    .as_string()
                    .and_then(|bytes| std::str::from_utf8(bytes).ok())
                    .map(|s| json!(s)),
                ValueType::Bool => value.as_bool().map(|b| json!(b)),
                ValueType::Int64 => value.as_int64().map(|i| json!(i)),
                ValueType::Double => value.as_double().map(|d| json!(d)),
                _ => None,
            };
            if let Some(json_value) = json_value {
                _ = fields.insert(key.to_owned(), json_value);
            }
        }
        let body_view = raw.body();
        let body = body_view
            .as_ref()
            .and_then(|body| body.as_string())
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .map(str::to_owned);
        Self {
            name: metadata.name(),
            target: metadata.target(),
            level: *metadata.level(),
            fields,
            body,
        }
    }

    fn assert_recovery_contract(&self, name: &str, level: Level, kind: &str) {
        assert_eq!(self.name, name);
        assert_eq!(self.target, "otel.exporter.otlp_http");
        assert_eq!(self.level, level);
        assert_eq!(self.fields["diagnostic_kind"], kind);
    }

    /// Like [`Self::assert_contract`], but for Degraded/Summary reports
    /// delivered via the splice-and-log fast path: the event's identity is
    /// the retained detail's own capture callsite (here always
    /// `test.diagnostic.detail` at WARN, via the shared [`detail`] helper),
    /// not a per-operation name/level chosen at the emit call site.
    fn assert_fast_path_contract(&self, kind: &str) {
        self.assert_fast_path_contract_named("test.diagnostic.detail", kind);
    }

    /// Like [`Self::assert_fast_path_contract`], but for production call
    /// sites whose retained detail was captured by `otel_diagnostic_warn!`
    /// under its ordinary component event name,
    /// rather than the shared test `detail()` helper's fixed identity.
    fn assert_fast_path_contract_named(&self, name: &str, kind: &str) {
        assert_eq!(self.name, name);
        assert_eq!(self.target, "otel.exporter.otlp_http");
        assert_eq!(self.level, Level::WARN);
        assert_eq!(self.fields["diagnostic_kind"], kind);
        for field in [
            "episode_seconds",
            "interval_seconds",
            "error_sample_age_seconds",
        ] {
            assert!(self.fields[field].is_f64(), "{field} must be a number");
        }
        for field in [
            "successful_attempts",
            "failed_attempts",
            "suppressed_diagnostics",
            "total_successful_attempts",
            "total_failed_attempts",
            "total_suppressed_diagnostics",
        ] {
            assert!(self.fields[field].is_u64(), "{field} must be an integer");
        }
        assert!(self.fields["error_counts"].is_string());
        assert!(self.fields["total_error_counts"].is_string());
    }
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<CapturedEvent>>>);

impl Capture {
    fn drain(&self, receiver: &flume::Receiver<ObservedEvent>) {
        let drained: Vec<CapturedEvent> = receiver
            .try_iter()
            .map(|event| match event {
                ObservedEvent::Log(LogEvent { record, .. }) => {
                    CapturedEvent::from_log_record(&record)
                }
                ObservedEvent::Engine(_) => unreachable!("only log events are emitted here"),
            })
            .collect();
        self.0.lock().unwrap().extend(drained);
    }
}

/// Runs `f` through the same tracing setup used in production and decodes the
/// resulting ordinary and statefully delivered records in call order.
fn with_capture<R>(f: impl FnOnce() -> R) -> (R, Capture) {
    let capture = Capture::default();
    let (sender, receiver) = flume::unbounded();
    let reporter = ObservedEventReporter::new(
        otel_arrow_dfe_config::observed_state::SendPolicy::default(),
        sender,
    );
    let setup = TracingSetup::new(
        ProviderSetup::InternalAsync { reporter },
        otel_arrow_dfe_config::settings::telemetry::logs::LogLevel::default(),
        LogContext::new,
    );
    let result = setup.with_subscriber(f);
    capture.drain(&receiver);
    (result, capture)
}

/// Scenario: Mixed failures and stale successes select summaries across independent signals.
/// Guarantees: HTTP events preserve typed legacy fields and matching sample metadata through recovery.
#[test]
fn delivery_event_contract_and_retained_samples() {
    use OtlpHttpExporterErrorType::{PartialRejection, Transport};
    let (_, capture) = with_capture(|| {
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        let mut diagnostics = SignalSet::<DeliveryDiagnostic>::default();
        let logs = diagnostics.signal(SignalType::Logs);
        let report = logs.failure(at(0), Transport, true, || detail("connection refused"));
        logs.emit(report, SignalType::Logs);
        assert!(
            logs.failure(at(10), PartialRejection, false, || panic!("suppressed"))
                .is_none()
        );
        let report = logs.success(at(0), at(60));
        logs.emit(report, SignalType::Logs);
        assert!(
            logs.failure(at(61), Transport, true, || panic!("suppressed"))
                .is_none()
        );
        let report = logs.failure(at(120), PartialRejection, false, || {
            detail("partial acceptance")
        });
        logs.emit(report, SignalType::Logs);
        assert!(
            logs.failure(at(121), Transport, true, || panic!("suppressed"))
                .is_none()
        );

        let traces = diagnostics.signal(SignalType::Traces);
        let report = traces.failure(at(130), Transport, true, || {
            detail("trace connection refused")
        });
        traces.emit(report, SignalType::Traces);

        let logs = diagnostics.signal(SignalType::Logs);
        let report = logs.success(at(0), at(180));
        logs.emit(report, SignalType::Logs);
        let report = logs.success(at(122), at(181));
        logs.emit(report, SignalType::Logs);
    });
    let events = capture.0.lock().unwrap();
    assert_eq!(events.len(), 6);
    for (index, kind) in [
        (0, "first_failure"),
        (1, "summary"),
        (2, "summary"),
        (3, "first_failure"),
        (4, "summary"),
    ] {
        events[index].assert_fast_path_contract(kind);
    }
    for index in [0, 1] {
        assert_eq!(events[index].body.as_deref(), Some("connection refused"));
        assert_eq!(events[index].fields["retryable"], true);
    }
    assert_eq!(events[1].fields["error_sample_age_seconds"], 60.0);
    assert_eq!(events[1].fields["error_counts"], "partial_rejection=1");
    assert_eq!(
        events[1].fields["total_error_counts"],
        "transport=1,partial_rejection=1"
    );
    for index in [2, 4] {
        assert_eq!(events[index].body.as_deref(), Some("partial acceptance"));
        assert_eq!(events[index].fields["retryable"], false);
        assert_eq!(events[index].fields["signal"], "logs");
    }
    assert_eq!(events[2].fields["error_sample_age_seconds"], 0.0);
    assert_eq!(events[3].fields["retryable"], true);
    assert_eq!(events[3].fields["signal"], "traces");
    assert_eq!(events[4].fields["error_sample_age_seconds"], 60.0);
    assert_eq!(events[4].fields["failed_attempts"], 1);
    assert_eq!(events[4].fields["successful_attempts"], 1);
    assert_eq!(events[4].fields["suppressed_diagnostics"], 1);
    events[5].assert_recovery_contract(
        "otlp.exporter.http.export_recovered",
        Level::INFO,
        "recovery",
    );
    assert_eq!(events[5].fields["error"], "partial acceptance");
    assert_eq!(events[5].fields["signal"], "logs");
    assert!(!events[5].fields.contains_key("retryable"));
}

/// Scenario: Preparation, Ack routing, and Nack routing fail during one reporting interval.
/// Guarantees: Separate bounded events retain compact Ack/Nack context and preparation errors.
#[test]
fn preparation_and_notification_event_contracts() {
    let (_, capture) = with_capture(|| {
        let start = Instant::now();
        let mut preparation = DiagnosticTracker::default();
        let mut notifications = DiagnosticTracker::default();
        emit_preparation(
            preparation.failure(start, OtlpHttpExporterErrorType::Encoding, || {
                detail("encoding failed")
            }),
            SignalType::Logs,
        );
        emit_notification(
            notifications.failure(start, DiagnosticErrorKind::Notification, || {
                detail("Ack channel closed")
            }),
            SignalType::Logs,
            NotificationOperation::Ack,
        );
        assert!(
            preparation
                .failure(start, OtlpHttpExporterErrorType::Compression, || panic!(
                    "suppressed"
                ))
                .is_none()
        );
        assert!(
            notifications
                .failure(start, DiagnosticErrorKind::Notification, || panic!(
                    "suppressed"
                ))
                .is_none()
        );
        let later = start + Duration::from_secs(60);
        emit_preparation(
            preparation.failure(later, OtlpHttpExporterErrorType::Compression, || {
                detail("compression failed")
            }),
            SignalType::Logs,
        );
        emit_notification(
            notifications.failure(later, DiagnosticErrorKind::Notification, || {
                detail("Nack channel closed")
            }),
            SignalType::Logs,
            NotificationOperation::Nack,
        );
    });
    let events = capture.0.lock().unwrap();
    assert_eq!(events.len(), 4);
    for (index, kind) in [(0, "first_failure"), (2, "summary")] {
        events[index].assert_fast_path_contract(kind);
    }
    for (index, kind, operation, error_operation) in [
        (1, "first_failure", "ack", "Ack"),
        (3, "summary", "nack", "Nack"),
    ] {
        events[index].assert_fast_path_contract(kind);
        assert_eq!(events[index].fields["operation"], operation);
        assert_eq!(
            events[index].body.as_deref(),
            Some(format!("{error_operation} channel closed").as_str())
        );
    }
    assert_eq!(events[0].body.as_deref(), Some("encoding failed"));
    assert_eq!(events[2].body.as_deref(), Some("compression failed"));
    for event in events.iter() {
        assert!(!event.fields.contains_key("retryable"));
        assert!(event.body.is_some());
    }
    for index in [2, 3] {
        assert_eq!(events[index].fields["total_failed_attempts"], 3);
        assert_eq!(events[index].fields["failed_attempts"], 2);
        assert_eq!(events[index].fields["suppressed_diagnostics"], 1);
    }
}

/// Scenario: HTTP statuses are finalized with static and dynamic credentials.
/// Guarantees: Diagnostic retryability matches Nacks and auth invalidation regardless of metric interests.
#[test]
fn delivery_retryability_matches_auth_aware_nacks() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for interests in [Interests::empty(), Interests::NODE_INPUT_METRICS] {
        for (status, auth_generation, retryable) in [
            (401, None, false),
            (401, Some(7), true),
            (401, Some(8), true),
            (403, Some(8), false),
            (429, None, true),
            (503, None, true),
            (400, None, false),
        ] {
            let (message, capture) = with_capture(|| {
                runtime.block_on(async {
                    let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
                    let mut metrics = OtlpHttpExporterMetrics::register(&pipeline_ctx, None);
                    let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
                    let mut effects = EffectHandler::new(
                        test_node("test-exporter"),
                        reporter,
                        otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
                    );
                    let (tx, mut rx) = pipeline_completion_msg_channel(1);
                    effects.set_pipeline_completion_msg_sender(tx);
                    let pdata = OtapPdata::new_default(
                        OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into(),
                    )
                    .test_subscribe_to(
                        Interests::NACKS,
                        TestCallData::default().into(),
                        123,
                    );
                    let (context, saved_payload) = pdata.into_parts();
                    let response = reqwest::Response::from(
                        http::Response::builder().status(status).body("").unwrap(),
                    );
                    let error = ServiceRequestError::RequestError {
                        err: response.error_for_status().unwrap_err(),
                        detail: "test response".into(),
                    };
                    let category = error.error_type();
                    let message = error.to_string();
                    let attempt = metrics
                        .boundary
                        .attempt(SignalType::Logs)
                        .run(async |attempt| {
                            Err(if category.is_refusal() {
                                attempt.refused(error)
                            } else {
                                attempt.failed(error)
                            })
                        })
                        .await;
                    let rejected = finalize_completed_export(
                        CompletedExport {
                            diagnostic_started_at: Instant::now(),
                            attempt,
                            context,
                            saved_payload,
                            signal_type: SignalType::Logs,
                            auth_generation,
                        },
                        &effects,
                        &mut metrics,
                    )
                    .await;
                    assert_eq!(
                        rejected,
                        (status == 401).then_some(auth_generation).flatten()
                    );
                    let PipelineCompletionMsg::DeliverNack { nack } = rx.recv().await.unwrap()
                    else {
                        panic!("failed export must Nack");
                    };
                    assert_eq!(nack.permanent, !retryable);
                    assert_eq!(nack.reason, message);
                    let snapshots = metrics.terminal_snapshots(None);
                    assert!(snapshots.iter().any(|snapshot| {
                        snapshot.descriptor().name == "exporter.otlp_http.failures"
                            && snapshot.get_metrics()[0].to_u64_lossy() == 1
                    }));
                    assert_eq!(
                        snapshots
                            .iter()
                            .any(|snapshot| snapshot.descriptor().name == "exporter.attempted"),
                        interests.contains(Interests::NODE_INPUT_METRICS)
                    );
                    message
                })
            });
            let events = capture.0.lock().unwrap();
            assert_eq!(events.len(), 1);
            events[0].assert_fast_path_contract_named(
                "otlp.exporter.http.export_error",
                "first_failure",
            );
            assert_eq!(events[0].body.as_deref(), Some(message.as_str()));
            assert_eq!(events[0].fields["retryable"], retryable);
        }
    }
}

/// Scenario: A successful or partially rejected export encounters a closed notification channel.
/// Guarantees: Actual completion events distinguish Ack/Nack failures and preserve the delivery outcome.
#[test]
fn notification_failure_does_not_redefine_delivery() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for rejected in [false, true] {
        let (_, capture) = with_capture(|| {
            runtime.block_on(async {
                let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
                let mut metrics = OtlpHttpExporterMetrics::register(&pipeline_ctx, None);
                let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
                let mut effects = EffectHandler::new(
                    test_node("test-exporter"),
                    reporter,
                    otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
                );
                let (tx, rx) = pipeline_completion_msg_channel(1);
                drop(rx);
                effects.set_pipeline_completion_msg_sender(tx);
                let pdata =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into())
                        .test_subscribe_to(
                            Interests::ACKS | Interests::NACKS,
                            TestCallData::default().into(),
                            123,
                        );
                let (context, saved_payload) = pdata.into_parts();
                let attempt = metrics
                    .boundary
                    .attempt(SignalType::Logs)
                    .run(async |attempt| {
                        if rejected {
                            Err(attempt.refused(ServiceRequestError::PartialRejection {
                                rejected: 1,
                                error_message: "partial rejection".into(),
                            }))
                        } else {
                            Ok::<_, ErrorWithOutcome<ServiceRequestError>>(())
                        }
                    })
                    .await;
                let _ = finalize_completed_export(
                    CompletedExport {
                        diagnostic_started_at: Instant::now(),
                        attempt,
                        context,
                        saved_payload,
                        signal_type: SignalType::Logs,
                        auth_generation: None,
                    },
                    &effects,
                    &mut metrics,
                )
                .await;
            });
        });
        let events = capture.0.lock().unwrap();
        assert_eq!(events.len(), if rejected { 2 } else { 1 });
        if rejected {
            events[0].assert_fast_path_contract_named(
                "otlp.exporter.http.export_error",
                "first_failure",
            );
            assert_eq!(events[0].fields["retryable"], false);
            assert_eq!(
                events[0].body.as_deref(),
                Some("partial rejection (1 rejected)")
            );
        }
        let notification = events.last().unwrap();
        let operation = if rejected { "nack" } else { "ack" };
        notification.assert_fast_path_contract_named(
            "otlp.exporter.http.notification_error",
            "first_failure",
        );
        assert_eq!(notification.fields["operation"], operation);
        assert!(
            notification
                .body
                .as_deref()
                .is_some_and(|error| !error.is_empty())
        );
        assert!(!notification.fields.contains_key("retryable"));
    }
}

/// Scenario: An early export failure cannot route its terminal Nack upstream.
/// Guarantees: The shared Nack path emits a bounded notification diagnostic with canonical signal data.
#[test]
fn early_nack_notification_failure_is_observable() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (_, capture) = with_capture(|| {
        runtime.block_on(async {
            let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
            let mut metrics = OtlpHttpExporterMetrics::register(&pipeline_ctx, None);
            let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
            let mut effects = EffectHandler::new(
                test_node("test-exporter"),
                reporter,
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );
            let (tx, rx) = pipeline_completion_msg_channel(1);
            drop(rx);
            effects.set_pipeline_completion_msg_sender(tx);
            let pdata =
                OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into())
                    .test_subscribe_to(Interests::NACKS, TestCallData::default().into(), 123);

            notify_nack_with_diagnostics(
                &effects,
                &mut metrics,
                SignalType::Logs,
                NackMsg::new("preparation failed", pdata),
            )
            .await;
        });
    });

    let events = capture.0.lock().unwrap();
    assert_eq!(events.len(), 1);
    events[0]
        .assert_fast_path_contract_named("otlp.exporter.http.notification_error", "first_failure");
    assert_eq!(events[0].fields["signal"], "logs");
    assert_eq!(events[0].fields["operation"], "nack");
    assert!(
        events[0]
            .body
            .as_deref()
            .is_some_and(|error| !error.is_empty())
    );
}
