// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Caller-owned sampling through ordinary logging macros.

use otel_arrow_dfe_config::settings::telemetry::logs::LogLevel;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otel_arrow_dfe_telemetry::event::{ObservedEvent, ObservedEventReporter};
use otel_arrow_dfe_telemetry::log_filter::{RuntimeLogFilter, RuntimeLogFilterHandle};
use otel_arrow_dfe_telemetry::self_tracing::{
    LogContext, LogRecord, StructBox, StructuredValue, ValueWriter,
};
use otel_arrow_dfe_telemetry::tracing_init::{ProviderSetup, TracingSetup};
use otel_arrow_dfe_telemetry::{Logger, Sampler};
use prost::Message;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;
use tracing::{Dispatch, Event, Level, Metadata};

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = "urn:otel:exporter:logger_test",
    target = "otel.exporter.logger_test",
);

fn level(value: &str) -> LogLevel {
    LogLevel::try_from(value.to_owned()).expect("valid filter")
}

fn setup(
    value: &str,
) -> (
    TracingSetup,
    RuntimeLogFilterHandle,
    flume::Receiver<ObservedEvent>,
) {
    let (sender, receiver) = flume::bounded(16);
    let (filter, handle) = RuntimeLogFilter::new_configured(&level(value));
    let setup = TracingSetup::new(
        ProviderSetup::InternalAsync {
            reporter: ObservedEventReporter::new(Default::default(), sender),
        },
        level(value),
        LogContext::new,
    )
    .with_log_filter(filter);
    (setup, handle, receiver)
}

fn records(receiver: &flume::Receiver<ObservedEvent>) -> Vec<LogRecord> {
    receiver
        .drain()
        .map(|event| match event {
            ObservedEvent::Log(event) => event.record,
            ObservedEvent::Engine(_) => panic!("expected log event"),
        })
        .collect()
}

// Select state transitions; timing and full episode policies are out of scope.
#[derive(Default)]
struct Toggle {
    observations: Rc<Cell<usize>>,
    failed: bool,
    started_at: Option<Instant>,
    sample: Option<LogRecord>,
}

impl Toggle {
    fn outcome<T, E>(&mut self, result: &Result<T, E>) -> Outcome<'_> {
        Outcome {
            state: self,
            failed: result.is_err(),
            started_at: None,
        }
    }
}

struct Outcome<'a> {
    state: &'a mut Toggle,
    failed: bool,
    started_at: Option<Instant>,
}

impl Outcome<'_> {
    fn started_at(mut self, started_at: Instant) -> Self {
        self.started_at = Some(started_at);
        self
    }
}

impl Sampler for Outcome<'_> {
    fn should_sample(&mut self, _metadata: &Metadata<'_>) -> bool {
        self.state
            .observations
            .set(self.state.observations.get() + 1);
        self.state.started_at = self.started_at;
        std::mem::replace(&mut self.state.failed, self.failed) != self.failed
    }
}

impl Logger for Outcome<'_> {
    fn emit(&mut self, event: &Event<'_>, dispatch: &Dispatch) {
        if self.failed {
            self.state.sample = Some(LogRecord::new(event, LogContext::new()));
        } else {
            assert!(self.state.sample.take().is_some());
        }
        dispatch.event(event);
    }
}

fn observe(state: &mut Toggle, result: Result<(), &str>, start: Instant, fields: &Cell<usize>) {
    otel_warn!(
        logger: state.outcome(&result).started_at(start),
        "test.delivery",
        message = {
            fields.set(fields.get() + 1);
            result.err().unwrap_or("Ok")
        }
    );
}

/// Scenario: Two samplers share a callsite under changing filters.
/// Guarantees: Filtering skips state updates; sampling skips fields and keeps state local.
#[test]
fn outcome_sampling() {
    let (setup, handle, receiver) = setup("off");
    let start = Instant::now();
    let fields = Cell::new(0);
    let mut first = Toggle::default();
    let mut second = Toggle::default();
    setup.with_subscriber(|| {
        observe(&mut first, Err("disabled"), start, &fields);
        assert_eq!(first.observations.get(), 0);

        handle.apply(Some(&level("off,otel.exporter.logger_test=warn")));
        observe(&mut first, Ok(()), start, &fields);
        observe(&mut first, Err("offline"), start, &fields);
        let sample = first.sample.clone().expect("saved start");
        observe(&mut first, Err("suppressed"), start, &fields);
        observe(&mut second, Err("other instance"), start, &fields);
        assert_eq!(fields.get(), 2);

        handle.apply(Some(&level("error")));
        observe(&mut first, Ok(()), start, &fields);
        assert!(first.failed);
        handle.apply(Some(&level("warn")));
        observe(&mut first, Ok(()), start, &fields);
        assert!(!first.failed);
        assert!(first.sample.is_none());
        assert!(second.sample.is_some());
        assert_eq!(first.started_at, Some(start));
        assert_eq!(first.observations.get(), 4);
        assert_eq!(second.observations.get(), 1);
        assert_eq!(fields.get(), 3);

        let emitted = records(&receiver);
        assert_eq!(emitted.len(), 3);
        assert_eq!(sample.body_attrs_bytes, emitted[0].body_attrs_bytes);
        assert_ne!(emitted[0].body_attrs_bytes, emitted[2].body_attrs_bytes);
        for record in &emitted {
            assert_eq!(record.callsite_id, emitted[0].callsite_id);
            assert_eq!(*record.callsite().level(), Level::WARN);
            assert_eq!(record.callsite().target(), "otel.exporter.logger_test");
        }
    });
}

/// Scenario: The four scoped, fixed-level logging macros use a sampler.
/// Guarantees: Samplers are borrowed; levels and field formatting are preserved.
#[test]
fn macro_forms() {
    struct Keep;
    impl Sampler for Keep {}
    impl Logger for Keep {}

    let (setup, _, receiver) = setup("trace");
    let mut sampler = Keep;
    setup.with_subscriber(|| {
        otel_debug!(logger: sampler, "test.debug");
        otel_info!(logger: sampler, "test.info", count = 1);
        otel_warn!(logger: sampler, "test.warn", value = %"display");
        otel_error!(logger: &mut sampler, "test.error", value = ?Some(42));
        otel_warn!(logger: sampler, "test.formatted", "answer {}", 42);
    });
    let emitted = records(&receiver);
    assert_eq!(emitted.len(), 5);
    let expected = [
        Level::DEBUG,
        Level::INFO,
        Level::WARN,
        Level::ERROR,
        Level::WARN,
    ];
    for (record, level) in emitted.iter().zip(expected) {
        assert_eq!(*record.callsite().level(), level);
        assert_eq!(record.callsite().target(), "otel.exporter.logger_test");
    }
}

/// Scenario: ordinary and sampled logging calls borrow structured state containing Rc<RefCell<_>>.
/// Guarantees: rejected fields are not evaluated; accepted fields snapshot local state before mutation or destruction.
#[test]
fn structured_fields_preserve_lazy_evaluation() {
    struct Point {
        x: Rc<RefCell<i64>>,
        y: i64,
    }

    impl StructuredValue for Point {
        fn write(&self, w: &mut dyn ValueWriter) {
            w.map(|m| {
                m.entry("x", |v| v.write_i64(*self.x.borrow()));
                m.entry("y", |v| v.write_i64(self.y));
            });
        }
    }

    struct Select(bool);
    impl Logger for Select {}
    impl Sampler for Select {
        fn should_sample(&mut self, _: &Metadata<'_>) -> bool {
            self.0
        }
    }

    let x = Rc::new(RefCell::new(1));
    let state = StructBox::new(Point {
        x: Rc::clone(&x),
        y: 2,
    });
    let evaluations = Cell::new(0);
    let (setup, handle, receiver) = setup("off");
    let value = || {
        evaluations.set(evaluations.get() + 1);
        state.as_struct()
    };
    setup.with_subscriber(|| {
        handle.apply(Some(&level("off")));
        otel_info!("point.filtered", to = value());
        assert_eq!(evaluations.get(), 0);

        handle.apply(Some(&level("info")));
        otel_info!("point.moved", to = value());
        otel_info!(logger: Select(false), "point.rejected", to = value());
        assert_eq!(evaluations.get(), 1);
        *x.borrow_mut() = 3;
        otel_info!(logger: Select(true), "point.sampled", to = value());
    });

    assert_eq!(evaluations.get(), 2);
    drop(state);
    assert_eq!(Rc::strong_count(&x), 1);
    *x.borrow_mut() = 99;
    let emitted = records(&receiver);
    assert_eq!(emitted.len(), 2);
    for (record, expected_x) in emitted.into_iter().zip([1, 3]) {
        assert_eq!(record.dropped_attributes_count, 0);
        let decoded = otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord::decode(
            record.body_attrs_bytes.as_ref(),
        )
        .expect("valid record");
        assert_eq!(
            decoded.attributes,
            vec![KeyValue::new(
                "to",
                AnyValue::new_kvlist(vec![
                    KeyValue::new("x", AnyValue::new_int(expected_x)),
                    KeyValue::new("y", AnyValue::new_int(2)),
                ]),
            )]
        );
    }
}

/// Scenario: Logger, sampler, and field expressions log under changing filters and sampling decisions.
/// Guarantees: Nested logs survive; filtering skips the logger; sampling rejection skips fields.
#[test]
fn nested_logging_preserves_lazy_evaluation() {
    struct Select(bool);
    impl Logger for Select {}
    impl Sampler for Select {
        fn should_sample(&mut self, _metadata: &Metadata<'_>) -> bool {
            otel_warn!("test.decision");
            self.0
        }
    }

    struct Keep;
    impl Sampler for Keep {}
    impl Logger for Keep {}

    let (other, _, other_receiver) = setup("warn");
    let (setup, handle, receiver) = setup("off");
    let loggers = Cell::new(0);
    let emit = |keep| {
        let sampler = Select(keep);
        otel_warn!(
            logger: {
                loggers.set(loggers.get() + 1);
                otel_warn!("test.logger");
                sampler
            },
            "test.outer",
            value = {
                otel_warn!("test.field");
                otel_warn!(logger: Keep, "test.nested.sampled");
                42
            }
        );
    };
    other.with_subscriber(|| {
        setup.with_subscriber(|| {
            emit(true);
            assert_eq!(loggers.get(), 0);
            handle.apply(Some(&level("warn")));
            emit(false);
            emit(true);
            handle.apply(Some(&level("error")));
            emit(true);
            assert_eq!(loggers.get(), 2);
        });
    });

    let emitted = records(&receiver);
    let names: Vec<_> = emitted
        .iter()
        .map(|record| record.callsite().name())
        .collect();
    assert_eq!(
        names,
        [
            "test.logger",
            "test.decision",
            "test.logger",
            "test.decision",
            "test.field",
            "test.nested.sampled",
            "test.outer",
        ]
    );
    assert!(records(&other_receiver).is_empty());
}
