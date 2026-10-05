// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verify suppression happens before subscribers receive operation diagnostics.

use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogRecord;
use otel_arrow_dfe_pdata_views::views::common::{AnyValueView, AttributeView};
use otel_arrow_dfe_pdata_views::views::logs::LogRecordView;
use otel_arrow_dfe_telemetry::diagnostics::{DiagnosticErrorKind, SignalDiagnostics};
use otel_arrow_dfe_telemetry::event::{LogEvent, ObservedEvent, ObservedEventReporter};
use otel_arrow_dfe_telemetry::self_tracing::LogContext;
use otel_arrow_dfe_telemetry::tracing_init::{ProviderSetup, TracingSetup};
use std::cell::Cell;
use std::time::{Duration, Instant};
use tracing::Level;

fn counted_message(formats: &Cell<u64>) -> &'static str {
    formats.set(formats.get() + 1);
    "connection refused"
}

/// Scenario: A busy outage produces a first warning, summary, and recovery.
/// Guarantees: Sampling constructs only the selected events and formats only selected details.
#[test]
fn suppression_precedes_all_subscribers() {
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
    let formats = Cell::new(0);
    setup.with_subscriber(|| {
        let start = Instant::now();
        let mut diagnostics = SignalDiagnostics::new(setup.log_emitter());
        for second in 0..=60 {
            for _ in 0..100 {
                otel_arrow_dfe_telemetry::otel_summary_warn!(
                    target: "otel.exporter.test",
                    at: start + Duration::from_secs(second),
                    &mut diagnostics,
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
            &mut diagnostics,
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
    let actual = receiver
        .try_iter()
        .map(|event| match event {
            ObservedEvent::Log(LogEvent { record, .. }) => (
                record.callsite().name(),
                record.callsite().target(),
                *record.callsite().level(),
            ),
            ObservedEvent::Engine(_) => panic!("expected log event"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
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
        let mut diagnostics = SignalDiagnostics::new(setup.log_emitter());
        let text = format!("root cause: {}", "x".repeat(4_000));
        otel_arrow_dfe_telemetry::otel_summary_warn!(
            target: "otel.exporter.test",
            &mut diagnostics,
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
