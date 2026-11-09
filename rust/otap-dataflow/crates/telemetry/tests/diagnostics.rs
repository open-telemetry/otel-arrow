// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Episode observations are independent of log filtering; records remain structured.

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::settings::telemetry::logs::LogLevel;
use otel_arrow_dfe_pdata::otlp::common::ProtoBuffer;
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    common::v1::{AnyValue, KeyValue, any_value::Value},
    logs::v1::LogRecord as ProtoRecord,
};
use otel_arrow_dfe_telemetry::diagnostics::{
    DiagnosticErrorKind, EpisodeSampler, IntervalSampler, SignalSet,
};
use otel_arrow_dfe_telemetry::event::{ObservedEvent, ObservedEventReporter};
use otel_arrow_dfe_telemetry::log_filter::{RuntimeLogFilter, RuntimeLogFilterHandle};
use otel_arrow_dfe_telemetry::self_tracing::encoder::DirectFieldVisitor;
use otel_arrow_dfe_telemetry::self_tracing::{LogContext, LogRecord};
use otel_arrow_dfe_telemetry::tracing_init::{ProviderSetup, TracingSetup};
use prost::Message;
use std::cell::Cell;
use std::time::{Duration, Instant};
use tracing::Level;

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = "urn:otel:exporter:diagnostic_test",
    target = "otel.exporter.diagnostic_test",
);

fn setup(
    level: &str,
) -> (
    TracingSetup,
    RuntimeLogFilterHandle,
    flume::Receiver<ObservedEvent>,
) {
    let level = LogLevel::try_from(level.to_owned()).expect("valid filter");
    let (filter, handle) = RuntimeLogFilter::new_configured(&level);
    let (tx, rx) = flume::bounded(16);
    let setup = TracingSetup::new(
        ProviderSetup::InternalAsync {
            reporter: ObservedEventReporter::new(Default::default(), tx),
        },
        level,
        LogContext::new,
    )
    .with_log_filter(filter);
    (setup, handle, rx)
}

fn receive(rx: &flume::Receiver<ObservedEvent>) -> (LogRecord, ProtoRecord) {
    let ObservedEvent::Log(event) = rx.try_recv().expect("expected log") else {
        panic!("unexpected engine event");
    };
    let decoded = ProtoRecord::decode(event.record.body_attrs_bytes.clone()).expect("valid OTLP");
    (event.record, decoded)
}

fn attr<'a>(attrs: &'a [KeyValue], name: &str) -> &'a Value {
    attrs
        .iter()
        .find(|a| a.key == name)
        .and_then(|a| a.value.as_ref())
        .and_then(|v| v.value.as_ref())
        .expect("expected attribute")
}

fn observe(
    state: &mut EpisodeSampler<DiagnosticErrorKind>,
    result: Result<(), DiagnosticErrorKind>,
    started: Instant,
    now: Instant,
    text: &str,
    formats: &Cell<usize>,
) {
    let mut decision = state.observe(result, started, now);
    if decision.is_recovery() {
        otel_info!(logger: decision, "test.recovered", message = "recovered");
    } else {
        otel_warn!(
            logger: decision,
            "test.failure",
            retryable = true,
            message = %{
                formats.set(formats.get() + 1);
                text
            }
        );
    }
}

/// Scenario: INFO recovery is disabled while failures and successes continue.
/// Guarantees: WARN-only users see a new first failure after recovery, with no eager formatting.
#[test]
fn warning_only_filter_does_not_prevent_recovery() {
    for (filter, expected) in [("info", 3), ("warn", 2), ("off", 0)] {
        let (setup, _, rx) = setup(filter);
        let mut state = EpisodeSampler::new(setup.log_emitter());
        let start = Instant::now();
        let formats = Cell::new(0);
        setup.with_subscriber(|| {
            for (now, started, failed) in [
                (0, 0, true),
                (1, 0, true),
                (31, 0, false),
                (32, 2, false),
                (33, 33, true),
            ] {
                observe(
                    &mut state,
                    if failed {
                        Err(DiagnosticErrorKind::Transport)
                    } else {
                        Ok(())
                    },
                    start + Duration::from_secs(started),
                    start + Duration::from_secs(now),
                    "offline",
                    &formats,
                );
            }
        });
        assert_eq!(rx.len(), expected);
        assert_eq!(formats.get(), if filter == "off" { 0 } else { 2 });
        while !rx.is_empty() {
            let (record, decoded) = receive(&rx);
            if *record.callsite().level() == Level::WARN {
                assert_eq!(
                    attr(&decoded.attributes, "diagnostic_kind"),
                    &Value::StringValue("first_failure".into())
                );
            }
        }
    }
}

/// Scenario: A later warning and success-triggered summary precede the episode end.
/// Guarantees: The end contains the first event as a map, not a formatted string or later error.
#[test]
fn recovery_retains_first_event_and_exact_counts() {
    let (setup, _, rx) = setup("info");
    let mut state = EpisodeSampler::new(setup.log_emitter());
    let start = Instant::now();
    let formats = Cell::new(0);
    setup.with_subscriber(|| {
        for (now, started, failed, text) in [
            (0, 0, true, "first\nerror"),
            (10, 0, true, "suppressed"),
            (60, 60, true, "later"),
            (61, 61, true, "suppressed"),
            (120, 0, false, ""),
            (121, 62, false, ""),
        ] {
            observe(
                &mut state,
                if failed {
                    Err(DiagnosticErrorKind::Transport)
                } else {
                    Ok(())
                },
                start + Duration::from_secs(started),
                start + Duration::from_secs(now),
                text,
                &formats,
            );
        }
    });
    assert_eq!(rx.len(), 4);
    let (_, first) = receive(&rx);
    let (_, later) = receive(&rx);
    assert_ne!(first.body, later.body);
    let (_, replay) = receive(&rx);
    assert_eq!(first.body, replay.body);
    let (record, end) = receive(&rx);
    assert_eq!(*record.callsite().level(), Level::INFO);
    assert_eq!(end.body, Some(AnyValue::new_string("recovered")));
    assert_eq!(
        attr(&end.attributes, "total_failed_attempts"),
        &Value::IntValue(4)
    );
    assert_eq!(
        attr(&end.attributes, "total_successful_attempts"),
        &Value::IntValue(2)
    );
    assert_eq!(
        attr(&end.attributes, "total_suppressed_diagnostics"),
        &Value::IntValue(2)
    );
    let Value::KvlistValue(saved) = attr(&end.attributes, "episode.start_event") else {
        panic!("expected nested map");
    };
    assert_eq!(
        attr(&saved.values, "event_name"),
        &Value::StringValue("test.failure".into())
    );
    assert_eq!(
        attr(&saved.values, "body"),
        first.body.as_ref().unwrap().value.as_ref().unwrap()
    );
    assert_eq!(attr(&saved.values, "severity_number"), &Value::IntValue(13));
    assert!(!end.attributes.iter().any(|a| a.key == "retryable"));
    assert_eq!(record.dropped_attributes_count, 0);
    otel_arrow_dfe_pdata::testing::round_trip::test_logs_round_trip(
        otel_arrow_dfe_pdata::testing::round_trip::to_logs_data(vec![end]),
    );
}

/// Scenario: Logging is enabled only after failures have been observed.
/// Guarantees: Recovery counts remain accurate and no filtered-out sample is invented.
#[test]
fn enabling_logs_does_not_invent_a_saved_event() {
    let (setup, handle, rx) = setup("off");
    let mut state = EpisodeSampler::new(setup.log_emitter());
    let start = Instant::now();
    let formats = Cell::new(0);
    setup.with_subscriber(|| {
        observe(
            &mut state,
            Err(DiagnosticErrorKind::Transport),
            start,
            start,
            "hidden",
            &formats,
        );
        handle.apply(Some(&LogLevel::try_from("info".to_owned()).unwrap()));
        observe(
            &mut state,
            Ok(()),
            start + Duration::from_secs(1),
            start + Duration::from_secs(30),
            "",
            &formats,
        );
    });
    let (_, end) = receive(&rx);
    assert_eq!(formats.get(), 0);
    assert_eq!(
        attr(&end.attributes, "total_failed_attempts"),
        &Value::IntValue(1)
    );
    assert!(
        !end.attributes
            .iter()
            .any(|a| a.key == "episode.start_event")
    );
}

/// Scenario: Failure-only reporting emits a first warning and a timed summary.
/// Guarantees: Suppression counts are exact and each warning describes its current error.
#[test]
fn interval_sampler_uses_current_failure() {
    let (setup, _, rx) = setup("warn");
    let mut state = IntervalSampler::new(setup.log_emitter());
    let start = Instant::now();
    setup.with_subscriber(|| {
        for (second, text) in [(0, "first"), (1, "hidden"), (60, "last")] {
            otel_warn!(logger: state.failure(start + Duration::from_secs(second), DiagnosticErrorKind::Other),
                "test.interval", error = text);
        }
    });
    let (_, first) = receive(&rx);
    let (_, last) = receive(&rx);
    assert_eq!(
        attr(&first.attributes, "error"),
        &Value::StringValue("first".into())
    );
    assert_eq!(
        attr(&last.attributes, "error"),
        &Value::StringValue("last".into())
    );
    assert_eq!(
        attr(&last.attributes, "failed_attempts"),
        &Value::IntValue(2)
    );
    assert_eq!(
        attr(&last.attributes, "suppressed_diagnostics"),
        &Value::IntValue(1)
    );
    assert!(rx.is_empty());
}

/// Scenario: Signal-scoped loggers supply their own clock and attributes.
/// Guarantees: Each signal expression is evaluated once and emitted once.
#[test]
fn signal_context_is_evaluated_once() {
    let (setup, handle, rx) = setup("off");
    let mut intervals = SignalSet::new(|| IntervalSampler::new(setup.log_emitter()));
    let mut episodes = SignalSet::new(|| EpisodeSampler::new(setup.log_emitter()));
    let evaluations = Cell::new(0);
    setup.with_subscriber(|| {
        otel_warn!(
            logger: intervals.logger({
                evaluations.set(evaluations.get() + 1);
                SignalType::Logs
            }, DiagnosticErrorKind::Other),
            "test.filtered"
        );
        assert_eq!(evaluations.get(), 0);
        handle.apply(Some(&LogLevel::try_from("warn".to_owned()).unwrap()));
        for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
            otel_warn!(
                logger: intervals.logger({
                    evaluations.set(evaluations.get() + 1);
                    signal
                }, DiagnosticErrorKind::Other),
                "test.interval",
                error = "failed"
            );
            let mut diagnostic = episodes.observe(
                {
                    evaluations.set(evaluations.get() + 1);
                    signal
                },
                Err(DiagnosticErrorKind::Other),
                Instant::now(),
            );
            otel_warn!(logger: diagnostic, "test.failure", message = "failed");
        }
    });
    assert_eq!(evaluations.get(), 6);
    for expected in ["logs", "metrics", "traces"] {
        for _ in 0..2 {
            let (_, record) = receive(&rx);
            assert_eq!(
                record
                    .attributes
                    .iter()
                    .filter(|a| a.key == "signal")
                    .count(),
                1
            );
            assert_eq!(
                attr(&record.attributes, "signal"),
                &Value::StringValue(expected.into())
            );
        }
    }
    assert!(rx.is_empty());
}

/// Scenario: Nested saved values encounter small budgets and excessive nesting.
/// Guarantees: Encoding remains bounded and valid, with one dropped-attribute count per partial map.
#[test]
fn nested_snapshot_is_bounded_and_reports_truncation() {
    let (setup, _, rx) = setup("warn");
    let mut state = IntervalSampler::new(setup.log_emitter());
    setup.with_subscriber(|| {
        otel_warn!(logger: state.failure(Instant::now(), DiagnosticErrorKind::Other),
            "test.saved", message = "seed");
    });
    let (mut saved, _) = receive(&rx);
    let value = AnyValue::new_kvlist(vec![
        KeyValue::new(
            "text",
            AnyValue::new_string(format!("line\n{}", "\u{e9}".repeat(400))),
        ),
        KeyValue::new(
            "values",
            AnyValue::new_array(vec![AnyValue::new_bool(true), AnyValue::new_int(-42)]),
        ),
    ]);
    saved.body_attrs_bytes = ProtoRecord {
        body: Some(value),
        ..Default::default()
    }
    .encode_to_vec()
    .into();
    for limit in [8, 32, 128, 256, 2048, 16384] {
        let mut buf = ProtoBuffer::with_capacity_and_limit(limit, limit);
        let dropped = {
            let mut visitor = DirectFieldVisitor::new(&mut buf);
            visitor.record_log_record("episode.start_event", &saved);
            visitor.dropped_count()
        };
        assert!(buf.len() <= limit);
        let decoded = ProtoRecord::decode(buf.as_ref()).expect("valid truncated record");
        if limit < 2048 {
            assert_eq!(dropped, 1);
        }
        if limit == 16384 {
            assert_eq!(dropped, 0);
            assert_eq!(decoded.attributes.len(), 1);
        }
    }
    let mut deep = AnyValue::new_string("leaf");
    for _ in 0..32 {
        deep = AnyValue::new_array(vec![deep]);
    }
    saved.body_attrs_bytes = ProtoRecord {
        body: Some(deep),
        ..Default::default()
    }
    .encode_to_vec()
    .into();
    let mut buf = ProtoBuffer::with_capacity_and_limit(16384, 16384);
    let mut visitor = DirectFieldVisitor::new(&mut buf);
    visitor.record_log_record("episode.start_event", &saved);
    assert_eq!(visitor.dropped_count(), 1);
    let _ = ProtoRecord::decode(buf.as_ref()).expect("valid depth-limited record");
}
