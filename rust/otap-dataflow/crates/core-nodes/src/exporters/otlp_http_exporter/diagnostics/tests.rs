// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::metrics::OtlpHttpExporterErrorType;
use crate::exporters::otlp_http_exporter::{
    CompletedExport, ServiceRequestError, finalize_completed_export,
    metrics::OtlpHttpExporterMetrics, notify_nack_with_diagnostics,
};
use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
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
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    common::v1::{AnyValue, any_value},
    logs::v1::LogRecord as ProtoRecord,
};
use otel_arrow_dfe_telemetry::attributes::AttributeEnum;
use otel_arrow_dfe_telemetry::diagnostics::SignalSet;
use otel_arrow_dfe_telemetry::diagnostics::{DiagnosticErrorKind, EpisodeSampler, IntervalSampler};
use otel_arrow_dfe_telemetry::event::{ObservedEvent, ObservedEventReporter};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry::self_tracing::LogContext;
use otel_arrow_dfe_telemetry::tracing_init::{ProviderSetup, TracingSetup};
use prost::Message as _;
use serde_json::{Map, Value, json};
use std::time::{Duration, Instant};
use tracing::Level;

#[derive(Debug)]
struct CapturedEvent {
    name: &'static str,
    target: &'static str,
    level: Level,
    fields: Map<String, Value>,
}

impl CapturedEvent {
    fn assert_contract(&self, name: &str, level: Level, kind: &str) {
        assert_eq!(self.name, name);
        assert_eq!(self.target, "otel.exporter.otlp_http");
        assert_eq!(self.level, level);
        assert_eq!(self.fields["diagnostic_kind"], kind);
        for field in ["episode_seconds", "interval_seconds"] {
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

struct Capture {
    setup: TracingSetup,
    receiver: flume::Receiver<ObservedEvent>,
}

impl Default for Capture {
    fn default() -> Self {
        let (tx, rx) = flume::bounded(16);
        let setup = TracingSetup::new(
            ProviderSetup::InternalAsync {
                reporter: ObservedEventReporter::new(Default::default(), tx),
            },
            Default::default(),
            LogContext::new,
        );
        Self {
            setup,
            receiver: rx,
        }
    }
}

fn value_json(value: AnyValue) -> Value {
    match value.value {
        None => Value::Null,
        Some(any_value::Value::StringValue(v)) => json!(v),
        Some(any_value::Value::BoolValue(v)) => json!(v),
        Some(any_value::Value::IntValue(v)) => json!(v),
        Some(any_value::Value::DoubleValue(v)) => json!(v),
        Some(any_value::Value::BytesValue(v)) => json!(v),
        Some(any_value::Value::ArrayValue(v)) => {
            Value::Array(v.values.into_iter().map(value_json).collect())
        }
        Some(any_value::Value::KvlistValue(v)) => Value::Object(
            v.values
                .into_iter()
                .map(|kv| (kv.key, kv.value.map(value_json).unwrap_or(Value::Null)))
                .collect(),
        ),
    }
}

impl Capture {
    fn events(&self) -> Vec<CapturedEvent> {
        let mut events = Vec::new();
        for event in self.receiver.drain() {
            let ObservedEvent::Log(event) = event else {
                panic!("expected log")
            };
            let meta = event.record.callsite_id.0.metadata();
            let record = ProtoRecord::decode(event.record.body_attrs_bytes).unwrap();
            assert_eq!(
                record
                    .attributes
                    .iter()
                    .filter(|a| a.key == "signal")
                    .count(),
                1
            );
            let mut fields: Map<_, _> = record
                .attributes
                .into_iter()
                .map(|kv| (kv.key, kv.value.map(value_json).unwrap_or(Value::Null)))
                .collect();
            if let Some(body) = record.body {
                _ = fields.insert("message".into(), value_json(body));
            }
            events.push(CapturedEvent {
                name: meta.name(),
                target: meta.target(),
                level: *meta.level(),
                fields,
            });
        }
        events
    }
}

/// Scenario: Mixed failures and stale successes select summaries across independent signals.
/// Guarantees: HTTP records preserve typed counters and first-event memory through recovery.
#[test]
fn delivery_event_contract_and_retained_samples() {
    use OtlpHttpExporterErrorType::{PartialRejection, Transport};
    let capture = Capture::default();
    capture.setup.with_subscriber(|| {
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        let mut diagnostics = SignalSet::new(|| EpisodeSampler::new(capture.setup.log_emitter()));
        let logs = diagnostics.signal(SignalType::Logs);
        delivery(
            logs,
            SignalType::Logs,
            at(0),
            at(0),
            Err((Transport, true, "connection refused")),
        );
        delivery(
            logs,
            SignalType::Logs,
            at(10),
            at(10),
            Err((PartialRejection, false, "suppressed")),
        );
        delivery(logs, SignalType::Logs, at(0), at(60), Ok(()));
        delivery(
            logs,
            SignalType::Logs,
            at(61),
            at(61),
            Err((Transport, true, "suppressed")),
        );
        delivery(
            logs,
            SignalType::Logs,
            at(120),
            at(120),
            Err((PartialRejection, false, "partial acceptance")),
        );
        delivery(
            logs,
            SignalType::Logs,
            at(121),
            at(121),
            Err((Transport, true, "suppressed")),
        );

        let traces = diagnostics.signal(SignalType::Traces);
        delivery(
            traces,
            SignalType::Traces,
            at(130),
            at(130),
            Err((Transport, true, "trace connection refused")),
        );

        let logs = diagnostics.signal(SignalType::Logs);
        delivery(logs, SignalType::Logs, at(0), at(180), Ok(()));
        delivery(logs, SignalType::Logs, at(122), at(181), Ok(()));
    });
    let events = capture.events();
    assert_eq!(events.len(), 6);
    for (index, kind) in [
        (0, "first_failure"),
        (1, "summary"),
        (2, "summary"),
        (3, "first_failure"),
        (4, "summary"),
    ] {
        events[index].assert_contract("otlp.exporter.http.export_error", Level::WARN, kind);
    }
    for index in [0, 1] {
        assert_eq!(events[index].fields["message"], "connection refused");
        assert_eq!(events[index].fields["retryable"], true);
    }
    assert_eq!(events[1].fields["error_counts"], "partial_rejection=1");
    assert_eq!(
        events[1].fields["total_error_counts"],
        "transport=1,partial_rejection=1"
    );
    assert_eq!(events[2].fields["message"], "partial acceptance");
    assert_eq!(events[2].fields["retryable"], false);
    assert_eq!(events[4].fields["message"], "connection refused");
    assert_eq!(events[4].fields["retryable"], true);
    assert_eq!(events[3].fields["retryable"], true);
    assert_eq!(events[3].fields["signal"], "traces");
    assert_eq!(events[4].fields["failed_attempts"], 1);
    assert_eq!(events[4].fields["successful_attempts"], 1);
    assert_eq!(events[4].fields["suppressed_diagnostics"], 1);
    events[5].assert_contract(
        "otlp.exporter.http.export_recovered",
        Level::INFO,
        "recovery",
    );
    assert_eq!(
        events[5].fields["episode.start_event"]["body"],
        "connection refused"
    );
    assert_eq!(events[5].fields["episode_seconds"], 181.0);
    assert_eq!(events[5].fields["total_failed_attempts"], 5);
    assert_eq!(events[5].fields["total_successful_attempts"], 3);
    assert_eq!(events[5].fields["total_suppressed_diagnostics"], 3);
    assert!(!events[5].fields.contains_key("retryable"));
}

fn delivery(
    state: &mut EpisodeSampler<OtlpHttpExporterErrorType>,
    signal: SignalType,
    started: Instant,
    now: Instant,
    result: Result<(), (OtlpHttpExporterErrorType, bool, &str)>,
) {
    let mut diagnostic = state.observe(result.map_err(|(category, _, _)| category), started, now);
    if diagnostic.is_recovery() {
        otel_info!(logger: diagnostic, "otlp.exporter.http.export_recovered",
            signal = signal.as_str(), message = "recovered");
    } else {
        otel_warn!(logger: diagnostic, "otlp.exporter.http.export_error",
            signal = signal.as_str(), retryable = result.is_err_and(|(_, retryable, _)| retryable),
            message = result.err().map(|(_, _, message)| message));
    }
}

/// Scenario: Preparation, Ack routing, and Nack routing fail during one reporting interval.
/// Guarantees: Separate bounded events retain compact Ack/Nack context and preparation errors.
#[test]
fn preparation_and_notification_event_contracts() {
    let capture = Capture::default();
    capture.setup.with_subscriber(|| {
        let start = Instant::now();
        let mut preparation = IntervalSampler::new(capture.setup.log_emitter());
        let mut notifications = IntervalSampler::new(capture.setup.log_emitter());
        otel_warn!(logger: preparation.failure(start, OtlpHttpExporterErrorType::Encoding),
            "otlp.exporter.http.preparation_error", signal = "logs", error = "encoding failed");
        otel_warn!(logger: notifications.failure(start, DiagnosticErrorKind::Notification),
            "otlp.exporter.http.notification_error", signal = "logs",
            operation = "ack", error = "Ack channel closed");
        let _ = preparation.failure(start, OtlpHttpExporterErrorType::Compression);
        let _ = notifications.failure(start, DiagnosticErrorKind::Notification);
        let later = start + Duration::from_secs(60);
        otel_warn!(logger: preparation.failure(later, OtlpHttpExporterErrorType::Compression),
            "otlp.exporter.http.preparation_error", signal = "logs", error = "compression failed");
        otel_warn!(logger: notifications.failure(later, DiagnosticErrorKind::Notification),
            "otlp.exporter.http.notification_error", signal = "logs",
            operation = "nack", error = "Nack channel closed");
    });
    let events = capture.events();
    assert_eq!(events.len(), 4);
    for (index, kind) in [(0, "first_failure"), (2, "summary")] {
        events[index].assert_contract("otlp.exporter.http.preparation_error", Level::WARN, kind);
    }
    for (index, kind, operation, error_operation) in [
        (1, "first_failure", "ack", "Ack"),
        (3, "summary", "nack", "Nack"),
    ] {
        events[index].assert_contract("otlp.exporter.http.notification_error", Level::WARN, kind);
        assert_eq!(events[index].fields["operation"], operation);
        assert_eq!(
            events[index].fields["error"],
            format!("{error_operation} channel closed")
        );
    }
    assert_eq!(events[0].fields["error"], "encoding failed");
    assert_eq!(events[2].fields["error"], "compression failed");
    for event in events.iter() {
        assert!(!event.fields.contains_key("retryable"));
        assert!(event.fields["error"].is_string());
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
            let capture = Capture::default();
            capture.setup.with_subscriber(|| {
                runtime.block_on(async {
                    let (mut pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
                    pipeline_ctx.set_structured_log_emitter(capture.setup.log_emitter());
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
                    let delivery = pdata.into_delivery();
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
                            delivery,
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
                    let events = capture.events();
                    assert_eq!(events.len(), 1);
                    events[0].assert_contract(
                        "otlp.exporter.http.export_error",
                        Level::WARN,
                        "first_failure",
                    );
                    assert_eq!(events[0].fields["message"], message);
                    assert_eq!(events[0].fields["retryable"], retryable);
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
                });
            });
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
        let capture = Capture::default();
        capture.setup.with_subscriber(|| {
            runtime.block_on(async {
                let (mut pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
                pipeline_ctx.set_structured_log_emitter(capture.setup.log_emitter());
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
                let delivery = pdata.into_delivery();
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
                        delivery,
                        signal_type: SignalType::Logs,
                        auth_generation: None,
                    },
                    &effects,
                    &mut metrics,
                )
                .await;
            });
        });
        let events = capture.events();
        assert_eq!(events.len(), if rejected { 2 } else { 1 });
        if rejected {
            events[0].assert_contract(
                "otlp.exporter.http.export_error",
                Level::WARN,
                "first_failure",
            );
            assert_eq!(events[0].fields["retryable"], false);
            assert_eq!(
                events[0].fields["message"],
                "partial rejection (1 rejected)"
            );
        }
        let notification = events.last().unwrap();
        notification.assert_contract(
            "otlp.exporter.http.notification_error",
            Level::WARN,
            "first_failure",
        );
        let operation = if rejected { "nack" } else { "ack" };
        assert_eq!(notification.fields["operation"], operation);
        assert!(
            notification.fields["error"]
                .as_str()
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
    let capture = Capture::default();
    capture.setup.with_subscriber(|| {
        runtime.block_on(async {
            let (mut pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
            pipeline_ctx.set_structured_log_emitter(capture.setup.log_emitter());
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

    let events = capture.events();
    assert_eq!(events.len(), 1);
    events[0].assert_contract(
        "otlp.exporter.http.notification_error",
        Level::WARN,
        "first_failure",
    );
    assert_eq!(events[0].fields["signal"], "logs");
    assert_eq!(events[0].fields["operation"], "nack");
    assert!(
        events[0].fields["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty())
    );
}
