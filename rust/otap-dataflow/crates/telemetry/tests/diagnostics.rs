// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verify suppression happens before subscribers receive operation diagnostics.

use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogRecord;
use otel_arrow_dfe_pdata_views::views::common::{AnyValueView, AttributeView};
use otel_arrow_dfe_pdata_views::views::logs::LogRecordView;
use otel_arrow_dfe_telemetry::diagnostics::{DiagnosticErrorKind, DiagnosticTracker};
use otel_arrow_dfe_telemetry::self_tracing::{LogContext, LogRecord};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context, prelude::*};

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<(&'static str, &'static str, Level)>>>);

impl<S: Subscriber> Layer<S> for Capture {
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let metadata = event.metadata();
        self.0
            .lock()
            .expect("capture lock must not be poisoned")
            .push((metadata.name(), metadata.target(), *metadata.level()));
    }
}

#[derive(Clone, Default)]
struct EncodedCapture(Arc<Mutex<Vec<(Vec<u8>, u16)>>>);

impl<S: Subscriber> Layer<S> for EncodedCapture {
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let record = LogRecord::new(event, LogContext::new());
        self.0
            .lock()
            .expect("capture lock must not be poisoned")
            .push((
                record.body_attrs_bytes.to_vec(),
                record.dropped_attributes_count,
            ));
    }
}

/// Scenario: Two subscribers observe a busy outage, a summary, and confirmed recovery.
/// Guarantees: Both receive only bounded reports with the component target and correct severity.
#[test]
fn suppression_precedes_all_subscribers() {
    let first = Capture::default();
    let second = Capture::default();
    let subscriber = tracing_subscriber::registry()
        .with(first.clone())
        .with(second.clone());
    let formats = Cell::new(0);
    tracing::subscriber::with_default(subscriber, || {
        let start = Instant::now();
        let mut tracker = DiagnosticTracker::default();
        for second in 0..=60 {
            for _ in 0..100 {
                if let Some(report) = tracker.failure(
                    start + Duration::from_secs(second),
                    DiagnosticErrorKind::Transport,
                    || {
                        formats.set(formats.get() + 1);
                        otel_arrow_dfe_telemetry::otel_diagnostic_detail!(
                            "test.detail",
                            message = "connection refused"
                        )
                    },
                ) {
                    otel_arrow_dfe_telemetry::otel_diagnostic_report!(
                        target: "otel.exporter.test", emit: otel_warn,
                        name: "test.export_error", report: &report,
                        stage = "delivery", signal = "logs"
                    );
                }
            }
        }
        let report = tracker
            .success(
                start + Duration::from_secs(61),
                start + Duration::from_secs(90),
            )
            .expect("fresh success confirms recovery");
        otel_arrow_dfe_telemetry::otel_diagnostic_report!(
            target: "otel.exporter.test", emit: otel_info,
            name: "test.export_recovered", report: &report,
            stage = "delivery", signal = "logs"
        );
    });
    let expected = vec![
        ("test.export_error", "otel.exporter.test", Level::WARN),
        ("test.export_error", "otel.exporter.test", Level::WARN),
        ("test.export_recovered", "otel.exporter.test", Level::INFO),
    ];
    assert_eq!(
        *first.0.lock().expect("capture lock must not be poisoned"),
        expected
    );
    assert_eq!(
        *second.0.lock().expect("capture lock must not be poisoned"),
        expected
    );
    assert_eq!(formats.get(), 2);
}

/// Scenario: A diagnostic contains error detail larger than the bounded ITS event buffer.
/// Guarantees: Priority context and a truncated error body survive real ITS encoding.
#[test]
fn priority_detail_survives_bounded_its_encoding() {
    let capture = EncodedCapture::default();
    tracing::subscriber::with_default(tracing_subscriber::registry().with(capture.clone()), || {
        let mut tracker = DiagnosticTracker::default();
        let report = tracker
            .failure(Instant::now(), DiagnosticErrorKind::Transport, || {
                let root_cause = format!("root cause: {}", "x".repeat(2_000));
                otel_arrow_dfe_telemetry::otel_diagnostic_detail!(
                    "test.detail",
                    message = %root_cause
                )
            })
            .expect("first failure must produce a report");
        otel_arrow_dfe_telemetry::otel_diagnostic_report!(
            target: "otel.exporter.test", emit: otel_warn,
            name: "test.export_error", report: &report,
            signal = "logs", retryable = true,
            diagnostic_kind = "first_failure", message = %report.detail_str()
        );
    });

    let events = capture.0.lock().expect("capture lock must not be poisoned");
    assert_eq!(events.len(), 1);
    let (body_attrs, dropped_attributes) = &events[0];
    let record = RawLogRecord::new(body_attrs);
    let body_value = record
        .body()
        .expect("diagnostic error body must survive encoding");
    let body = std::str::from_utf8(
        body_value
            .as_string()
            .expect("diagnostic error body must be a string"),
    )
    .expect("diagnostic error body must be valid UTF-8");
    assert!(body.starts_with("root cause: "));
    assert!(body.ends_with("[...]"));

    let attribute_keys = record
        .attributes()
        .map(|attribute| String::from_utf8_lossy(attribute.key()).into_owned())
        .collect::<Vec<_>>();
    for required in ["signal", "retryable", "diagnostic_kind"] {
        assert!(
            attribute_keys.iter().any(|key| key == required),
            "missing priority attribute {required}"
        );
    }
    assert!(
        *dropped_attributes > 0,
        "oversized diagnostics must report lower-priority truncation"
    );
}
