// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verify suppression happens before subscribers receive operation diagnostics.

use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogRecord;
use otel_arrow_dfe_pdata_views::views::common::{AnyValueView, AttributeView};
use otel_arrow_dfe_pdata_views::views::logs::LogRecordView;
use otel_arrow_dfe_telemetry::diagnostics::{DiagnosticErrorKind, DiagnosticTracker};
use otel_arrow_dfe_telemetry::event::{LogEvent, ObservedEvent, ObservedEventReporter};
use otel_arrow_dfe_telemetry::self_tracing::LogContext;
use otel_arrow_dfe_telemetry::tracing_init::{ProviderSetup, TracingSetup};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context, prelude::*};

fn counted_message(formats: &Cell<u64>) -> &'static str {
    formats.set(formats.get() + 1);
    "connection refused"
}

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
                otel_arrow_dfe_telemetry::otel_summary_warn!(
                    target: "otel.exporter.test",
                    at: start + Duration::from_secs(second),
                    &mut tracker,
                    otel_arrow_dfe_config::SignalType::Logs,
                    DiagnosticErrorKind::Transport,
                    "test.export_error",
                    stage = "delivery",
                    message = counted_message(&formats)
                );
            }
        }
        otel_arrow_dfe_telemetry::otel_summary_recover!(
            target: "otel.exporter.test",
            at: start + Duration::from_secs(90),
            &mut tracker,
            otel_arrow_dfe_config::SignalType::Logs,
            start + Duration::from_secs(61),
            "test.export_recovered",
            stage = "delivery"
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
    let (sender, receiver) = flume::unbounded();
    let setup = TracingSetup::new(
        ProviderSetup::InternalAsync {
            reporter: ObservedEventReporter::new(
                otel_arrow_dfe_config::observed_state::SendPolicy::default(),
                sender,
            ),
        },
        otel_arrow_dfe_config::settings::telemetry::logs::LogLevel::default(),
        LogContext::new,
    );
    setup.with_subscriber(|| {
        let mut tracker = DiagnosticTracker::default();
        let text = format!("root cause: {}", "x".repeat(4_000));
        otel_arrow_dfe_telemetry::otel_summary_warn!(
            target: "otel.exporter.test",
            &mut tracker,
            otel_arrow_dfe_config::SignalType::Logs,
            DiagnosticErrorKind::Transport,
            "test.export_error",
            retryable = true,
            message = %text
        );
    });

    let ObservedEvent::Log(LogEvent { record, .. }) =
        receiver.try_recv().expect("diagnostic should be delivered")
    else {
        panic!("expected log event");
    };
    assert!(
        receiver.try_recv().is_err(),
        "only one report should be sent"
    );
    let body_attrs = record.body_attrs_bytes;
    let record = RawLogRecord::new(&body_attrs);
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
}
