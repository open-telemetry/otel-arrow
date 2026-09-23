// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::exporters::otlp_http_exporter::{
    CompletedExport, RequestAuth, ServiceRequestError, finalize_completed_export,
    metrics::OtlpHttpExporterMetrics,
};
use bytes::Bytes;
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::control::{PipelineCompletionMsg, pipeline_completion_msg_channel};
use otel_arrow_dfe_engine::local::exporter::EffectHandler;
use otel_arrow_dfe_engine::testing::node::test_node;
use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
use otel_arrow_dfe_otap::metrics::ErrorWithOutcome;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_otap::testing::TestCallData;
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use serde_json::{Map, Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context, prelude::*};

#[derive(Debug)]
struct CapturedEvent {
    name: &'static str,
    target: &'static str,
    level: Level,
    fields: Map<String, Value>,
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
    fn assert_contract(&self, name: &str, level: Level, kind: &str, stage: &str) {
        assert_eq!(self.name, name);
        assert_eq!(self.target, "otel.exporter.otlp_http");
        assert_eq!(self.level, level);
        assert_eq!(self.fields["diagnostic_kind"], kind);
        assert_eq!(self.fields["stage"], stage);
        assert!(self.fields["message"].is_string());
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

impl<S: Subscriber> Layer<S> for Capture {
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let metadata = event.metadata();
        let mut captured = CapturedEvent {
            name: metadata.name(),
            target: metadata.target(),
            level: *metadata.level(),
            fields: Map::new(),
        };
        event.record(&mut captured);
        self.0.lock().unwrap().push(captured);
    }
}

/// Scenario: Mixed failures and stale successes select summaries across independent signals.
/// Guarantees: HTTP events preserve typed legacy fields and matching sample metadata through recovery.
#[test]
fn delivery_event_contract_and_retained_samples() {
    use OtlpHttpExporterErrorType::{PartialRejection, Transport};
    let capture = Capture::default();
    tracing::subscriber::with_default(tracing_subscriber::registry().with(capture.clone()), || {
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        let mut diagnostics = DeliveryDiagnostics::default();
        let logs = diagnostics.signal(SignalType::Logs);
        let report = logs.failure(at(0), Transport, true, || "connection refused");
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
        let report = logs.failure(at(120), PartialRejection, false, || "partial acceptance");
        logs.emit(report, SignalType::Logs);
        assert!(
            logs.failure(at(121), Transport, true, || panic!("suppressed"))
                .is_none()
        );

        let traces = diagnostics.signal(SignalType::Traces);
        let report = traces.failure(at(130), Transport, true, || "trace connection refused");
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
        events[index].assert_contract(
            "otlp.exporter.http.export_error",
            Level::WARN,
            kind,
            "delivery",
        );
    }
    for index in [0, 1] {
        assert_eq!(events[index].fields["message"], "connection refused");
        assert_eq!(events[index].fields["retryable"], true);
    }
    assert_eq!(events[1].fields["error_sample_age_seconds"], 60.0);
    assert_eq!(events[1].fields["error_counts"], "partial_rejection=1");
    assert_eq!(
        events[1].fields["total_error_counts"],
        "transport=1,partial_rejection=1"
    );
    for index in [2, 4] {
        assert_eq!(events[index].fields["message"], "partial acceptance");
        assert_eq!(events[index].fields["retryable"], false);
        assert_eq!(events[index].fields["signal"], "Logs");
    }
    assert_eq!(events[2].fields["error_sample_age_seconds"], 0.0);
    assert_eq!(events[3].fields["retryable"], true);
    assert_eq!(events[3].fields["signal"], "Traces");
    assert_eq!(events[4].fields["error_sample_age_seconds"], 60.0);
    assert_eq!(events[4].fields["failed_attempts"], 1);
    assert_eq!(events[4].fields["successful_attempts"], 1);
    assert_eq!(events[4].fields["suppressed_diagnostics"], 1);
    events[5].assert_contract(
        "otlp.exporter.http.export_recovered",
        Level::INFO,
        "recovery",
        "delivery",
    );
    assert_eq!(events[5].fields["message"], "OTLP HTTP export recovered");
    assert_eq!(events[5].fields["error"], "partial acceptance");
    assert_eq!(events[5].fields["error_sample_age_seconds"], 61.0);
    assert_eq!(events[5].fields["episode_seconds"], 181.0);
    assert_eq!(events[5].fields["total_failed_attempts"], 5);
    assert_eq!(events[5].fields["total_successful_attempts"], 3);
    assert_eq!(events[5].fields["total_suppressed_diagnostics"], 3);
    assert!(!events[5].fields.contains_key("retryable"));
}

/// Scenario: Preparation, Ack routing, and Nack routing fail during one reporting interval.
/// Guarantees: Separate bounded events retain Ack/Nack context and descriptive preparation errors.
#[test]
fn preparation_and_notification_event_contracts() {
    let capture = Capture::default();
    tracing::subscriber::with_default(tracing_subscriber::registry().with(capture.clone()), || {
        let start = Instant::now();
        let mut preparation = DiagnosticTracker::default();
        let mut notifications = DiagnosticTracker::default();
        emit_preparation(
            preparation.failure(
                start,
                OtlpHttpExporterErrorType::Encoding,
                || "encoding failed",
            ),
            SignalType::Logs,
        );
        emit_notification(
            notifications.failure(
                start,
                ExportErrorKind::Notification,
                || "Ack channel closed",
            ),
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
                .failure(start, ExportErrorKind::Notification, || panic!(
                    "suppressed"
                ))
                .is_none()
        );
        let later = start + Duration::from_secs(60);
        emit_preparation(
            preparation.failure(
                later,
                OtlpHttpExporterErrorType::Compression,
                || "compression failed",
            ),
            SignalType::Logs,
        );
        emit_notification(
            notifications.failure(
                later,
                ExportErrorKind::Notification,
                || "Nack channel closed",
            ),
            SignalType::Logs,
            NotificationOperation::Nack,
        );
    });
    let events = capture.0.lock().unwrap();
    assert_eq!(events.len(), 4);
    for (index, kind) in [(0, "first_failure"), (2, "summary")] {
        events[index].assert_contract(
            "otlp.exporter.http.preparation_error",
            Level::WARN,
            kind,
            "preparation",
        );
        assert_eq!(
            events[index].fields["message"],
            "Failed to prepare OTLP HTTP export"
        );
    }
    for (index, kind, operation) in [(1, "first_failure", "Ack"), (3, "summary", "Nack")] {
        events[index].assert_contract(
            "otlp.exporter.http.notification_error",
            Level::WARN,
            kind,
            "notification",
        );
        assert_eq!(
            events[index].fields["message"],
            format!("Failed to route the terminal OTLP HTTP {operation} notification")
        );
        assert_eq!(
            events[index].fields["error"],
            format!("{operation} channel closed")
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

/// Scenario: HTTP statuses are finalized with static, bearer-provider, and agent-fed credentials.
/// Guarantees: Diagnostic retryability matches Nacks and credential invalidation regardless of metric interests.
#[test]
fn delivery_retryability_matches_auth_aware_nacks() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for interests in [Interests::empty(), Interests::NODE_INPUT_METRICS] {
        for (status, request_auth, retryable) in [
            (401, RequestAuth::None, false),
            (401, RequestAuth::BearerProvider { generation: 7 }, true),
            (401, RequestAuth::AgentFed { generation: 8 }, true),
            (403, RequestAuth::AgentFed { generation: 8 }, false),
            (429, RequestAuth::None, true),
            (503, RequestAuth::None, true),
            (400, RequestAuth::None, false),
        ] {
            let capture = Capture::default();
            tracing::subscriber::with_default(
                tracing_subscriber::registry().with(capture.clone()),
                || {
                    runtime.block_on(async {
                        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(interests);
                        let mut metrics = OtlpHttpExporterMetrics::register(&pipeline_ctx);
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
                                request_auth,
                            },
                            &effects,
                            &mut metrics,
                        )
                        .await;
                        assert_eq!(
                            rejected.is_some(),
                            status == 401 && request_auth.is_dynamic()
                        );
                        if let Some(rejected) = rejected {
                            assert_eq!(format!("{rejected:?}"), format!("{request_auth:?}"));
                        }
                        let PipelineCompletionMsg::DeliverNack { nack } = rx.recv().await.unwrap()
                        else {
                            panic!("failed export must Nack");
                        };
                        assert_eq!(nack.permanent, !retryable);
                        assert_eq!(nack.reason, message);
                        let events = capture.0.lock().unwrap();
                        assert_eq!(events.len(), 1);
                        events[0].assert_contract(
                            "otlp.exporter.http.export_error",
                            Level::WARN,
                            "first_failure",
                            "delivery",
                        );
                        assert_eq!(events[0].fields["message"], message);
                        assert_eq!(events[0].fields["retryable"], retryable);
                        let snapshots = metrics.terminal_snapshots();
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
                },
            );
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
        tracing::subscriber::with_default(
            tracing_subscriber::registry().with(capture.clone()),
            || {
                runtime.block_on(async {
                    let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
                    let mut metrics = OtlpHttpExporterMetrics::register(&pipeline_ctx);
                    let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
                    let mut effects = EffectHandler::new(
                        test_node("test-exporter"),
                        reporter,
                        otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
                    );
                    let (tx, rx) = pipeline_completion_msg_channel(1);
                    drop(rx);
                    effects.set_pipeline_completion_msg_sender(tx);
                    let pdata = OtapPdata::new_default(
                        OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into(),
                    )
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
                            request_auth: RequestAuth::None,
                        },
                        &effects,
                        &mut metrics,
                    )
                    .await;
                });
            },
        );
        let events = capture.0.lock().unwrap();
        assert_eq!(events.len(), if rejected { 2 } else { 1 });
        if rejected {
            events[0].assert_contract(
                "otlp.exporter.http.export_error",
                Level::WARN,
                "first_failure",
                "delivery",
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
            "notification",
        );
        let operation = if rejected { "Nack" } else { "Ack" };
        assert_eq!(
            notification.fields["message"],
            format!("Failed to route the terminal OTLP HTTP {operation} notification")
        );
        assert!(
            notification.fields["error"]
                .as_str()
                .is_some_and(|error| !error.is_empty())
        );
        assert!(!notification.fields.contains_key("retryable"));
    }
}
