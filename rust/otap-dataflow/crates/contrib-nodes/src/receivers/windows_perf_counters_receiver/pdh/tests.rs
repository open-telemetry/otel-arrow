// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::native::check;
use super::*;
use crate::receivers::windows_perf_counters_receiver::config::{MetricKind, RuntimeConfig};
use crate::receivers::windows_perf_counters_receiver::model::Number;
use crate::receivers::windows_perf_counters_receiver::otap_builder::into_otap;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};
use std::mem::{size_of, size_of_val};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::Arc;
use windows_sys::Win32::System::Performance::{
    PDH_CALC_NEGATIVE_DENOMINATOR, PDH_CALC_NEGATIVE_TIMEBASE, PDH_CALC_NEGATIVE_VALUE,
    PDH_COUNTER_INFO_W, PDH_CSTATUS_INVALID_DATA, PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_NO_INSTANCE,
    PDH_CSTATUS_VALID_DATA, PDH_INVALID_ARGUMENT, PDH_INVALID_DATA, PDH_MORE_DATA, PDH_NO_DATA,
};

const OMISSION_STATUSES: [u32; 5] = [
    PDH_INVALID_DATA,
    PDH_NO_DATA,
    PDH_CALC_NEGATIVE_VALUE,
    PDH_CALC_NEGATIVE_DENOMINATOR,
    PDH_CALC_NEGATIVE_TIMEBASE,
];

const DATA_OMISSION_STATUSES: [u32; 5] = [
    PDH_CSTATUS_INVALID_DATA,
    PDH_NO_DATA,
    PDH_CALC_NEGATIVE_VALUE,
    PDH_CALC_NEGATIVE_DENOMINATOR,
    PDH_CALC_NEGATIVE_TIMEBASE,
];

fn config(path: &str) -> CounterConfig {
    CounterConfig {
        path: path.to_owned(),
        name: path.to_owned(),
        unit: "1".to_owned(),
        description: Arc::from("Test counter"),
        kind: MetricKind::Sum,
        attributes: Arc::new(BTreeMap::new()),
        scale_power10: 0,
    }
}

struct CounterFixture {
    add_status: u32,
    native_type: u32,
    size_status: u32,
    info_size: u32,
    fill_status: u32,
    fill_size: Option<u32>,
    raw_status: u32,
    raw_cstatus: u32,
    base: i64,
    multi_count: u32,
    format_status: u32,
    format_cstatus: u32,
    number: Number,
}

impl CounterFixture {
    fn new(native_type: u32, number: Number) -> Self {
        Self {
            add_status: 0,
            native_type,
            size_status: PDH_MORE_DATA,
            info_size: size_of::<PDH_COUNTER_INFO_W>() as u32,
            fill_status: 0,
            fill_size: None,
            raw_status: 0,
            raw_cstatus: PDH_CSTATUS_VALID_DATA,
            base: 10,
            multi_count: 4,
            format_status: 0,
            format_cstatus: PDH_CSTATUS_VALID_DATA,
            number,
        }
    }
}

struct State {
    counters: Vec<CounterFixture>,
    opens: usize,
    adds: usize,
    closes: usize,
    collects: usize,
    open_status: u32,
    remove_status: u32,
    handles: BTreeSet<usize>,
    removes: Vec<usize>,
    collect_status: u32,
    timestamp: i64,
    formats: Vec<(usize, ValueFormat)>,
    raws: Vec<usize>,
}

struct FakePdh(Rc<RefCell<State>>);

fn fake(counters: Vec<CounterFixture>) -> (Rc<RefCell<State>>, Vec<CounterConfig>) {
    let configs = (0..counters.len())
        .map(|i| config(&format!(r"\Object\Counter{i}")))
        .collect();
    let state = Rc::new(RefCell::new(State {
        counters,
        opens: 0,
        adds: 0,
        closes: 0,
        collects: 0,
        open_status: 0,
        remove_status: 0,
        handles: BTreeSet::new(),
        removes: Vec::new(),
        collect_status: 0,
        timestamp: 100,
        formats: Vec::new(),
        raws: Vec::new(),
    }));
    (state, configs)
}

impl FakePdh {
    fn open(state: Rc<RefCell<State>>) -> Result<Self, Error> {
        {
            let mut state = state.borrow_mut();
            state.opens += 1;
            check("PdhOpenQueryW", "<query>", state.open_status)?;
        }
        Ok(Self(state))
    }
}

fn open_fake(
    state: &Rc<RefCell<State>>,
    configs: &[CounterConfig],
) -> Result<Query<FakePdh>, Error> {
    Query::open_with(FakePdh::open(Rc::clone(state))?, configs)
}

impl PdhApi for FakePdh {
    type Counter = usize;

    fn add(&mut self, path: &str) -> Result<usize, Error> {
        let mut state = self.0.borrow_mut();
        let index = state.adds;
        state.adds += 1;
        check(
            "PdhAddEnglishCounterW",
            path,
            state.counters[index].add_status,
        )?;
        assert!(state.handles.insert(index + 1));
        Ok(index + 1)
    }

    fn remove(&mut self, counter: &usize, path: &str) -> Result<(), Error> {
        let mut state = self.0.borrow_mut();
        state.removes.push(*counter - 1);
        check("PdhRemoveCounter", path, state.remove_status)?;
        assert!(state.handles.remove(counter));
        Ok(())
    }

    fn counter_type(&mut self, counter: &usize, path: &str) -> Result<u32, Error> {
        let state = self.0.borrow();
        assert!(state.handles.contains(counter));
        let fixture = &state.counters[*counter - 1];
        NativePdh::counter_type_with(path, |size, buffer| {
            if let Some(buffer) = buffer {
                assert!((*size as usize) <= size_of_val(buffer));
                if fixture.fill_status == 0 {
                    buffer[0].dwType = fixture.native_type;
                }
                if let Some(fill_size) = fixture.fill_size {
                    *size = fill_size;
                }
                Ok(fixture.fill_status)
            } else {
                *size = fixture.info_size;
                Ok(fixture.size_status)
            }
        })
    }

    fn collect(&mut self) -> Result<bool, Error> {
        let mut state = self.0.borrow_mut();
        state.collects += 1;
        NativePdh::collect_status(state.collect_status)
    }

    fn raw(&mut self, counter: &usize, path: &str) -> Result<Option<RawSample>, Error> {
        let mut state = self.0.borrow_mut();
        let index = *counter - 1;
        assert!(state.handles.contains(counter));
        state.raws.push(index);
        let fixture = &state.counters[index];
        NativePdh::raw_with(path, |raw| {
            raw.CStatus = fixture.raw_cstatus;
            raw.SecondValue = fixture.base;
            raw.MultiCount = fixture.multi_count;
            fixture.raw_status
        })
    }

    fn formatted(
        &mut self,
        counter: &usize,
        path: &str,
        format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        let mut state = self.0.borrow_mut();
        let index = *counter - 1;
        assert!(state.handles.contains(counter));
        NativePdh::formatted_with(path, format, |_, value| {
            state.formats.push((index, format));
            let fixture = &state.counters[index];
            value.CStatus = fixture.format_cstatus;
            match (format, fixture.number) {
                (ValueFormat::Integer, Number::Integer(number)) => {
                    value.Anonymous.largeValue = number;
                }
                (ValueFormat::Double, Number::Double(number)) => {
                    value.Anonymous.doubleValue = number;
                }
                _ => panic!("fixture number must match the requested format"),
            }
            fixture.format_status
        })
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        Ok(self.0.borrow().timestamp)
    }
}

impl Drop for FakePdh {
    fn drop(&mut self) {
        let mut state = self.0.borrow_mut();
        state.closes += 1;
        state.handles.clear();
    }
}

/// Scenario: A configured counter fails to add while an earlier or later counter is usable.
/// Guarantees: Healthy counters retain their original indices, startup failures are reported, and skipped counters are not retried.
#[test]
fn opening_retains_healthy_counters_and_startup_diagnostics() {
    for failed in [0, 1] {
        let (state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(1)),
            CounterFixture::new(0, Number::Integer(2)),
        ]);
        state.borrow_mut().counters[failed].add_status = PDH_INVALID_DATA;
        let mut query = open_fake(&state, &configs).unwrap();
        assert_eq!(query.startup_failures.len(), 1);
        assert_eq!(query.startup_failures[0].counter_index, failed);
        assert_eq!(query.startup_failures[0].path, configs[failed].path);
        assert!(
            query.startup_failures[0]
                .error
                .contains("PdhAddEnglishCounterW")
        );
        for _ in 0..2 {
            let sample = query.collect().unwrap();
            assert!(sample.failures.is_empty());
            assert_eq!(sample.points.len(), 1);
            assert_eq!(sample.points[0].counter_index, 1 - failed);
            assert!(into_otap(&configs, sample).unwrap().is_some());
        }
        assert_eq!(state.borrow().adds, 2);
        drop(query);
        assert_eq!(state.borrow().closes, 1);
        assert!(state.borrow().handles.is_empty());
    }
}

/// Scenario: PDH cannot open a query, or every configured counter fails to add or inspect.
/// Guarantees: Opening returns a single-line error with readable path/error entries for every failed counter and releases every acquired handle.
#[test]
fn opening_fails_without_a_query_or_usable_counters() {
    for fail_open in [true, false] {
        let (state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(1)),
            CounterFixture::new(0, Number::Integer(2)),
        ]);
        if fail_open {
            state.borrow_mut().open_status = PDH_INVALID_DATA;
        } else {
            state.borrow_mut().counters[0].add_status = PDH_INVALID_DATA;
            state.borrow_mut().counters[1].fill_status = PDH_INVALID_DATA;
        }
        let error = match open_fake(&state, &configs) {
            Ok(_) => panic!("opening must fail without usable counters"),
            Err(error) => error,
        };
        if fail_open {
            assert!(matches!(
                error,
                Error::Pdh {
                    operation: "PdhOpenQueryW",
                    ..
                }
            ));
        } else {
            let message = error.to_string();
            let Error::NoCounters { failures } = error else {
                panic!("expected startup diagnostics for every counter");
            };
            assert_eq!(failures.len(), 2);
            assert_eq!(
                message,
                format!(
                    "no configured performance counter could be loaded; startup failures: {}",
                    failures
                        .iter()
                        .map(|failure| format!("{}: {}", failure.path, failure.error))
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            );
            for (index, failure) in failures.iter().enumerate() {
                assert_eq!(failure.counter_index, index);
                assert_eq!(failure.path, configs[index].path);
                assert!(failure.error.contains(&configs[index].path));
            }
        }
        assert_eq!(state.borrow().opens, 1);
        assert_eq!(state.borrow().closes, usize::from(!fail_open));
        assert!(state.borrow().handles.is_empty());
    }
}

/// Scenario: Counter-info sizing, buffer filling, filled size or native-type inspection fails on a later counter.
/// Guarantees: Unfilled zeroed metadata never becomes a valid counter; startup diagnostics preserve the failure and healthy peers collect.
#[test]
fn inspection_failure_cleans_up_without_classifying_zero_buffer() {
    for failure in 0..7 {
        let (state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(1)),
            CounterFixture::new(0, Number::Integer(2)),
        ]);
        {
            let mut state = state.borrow_mut();
            let counter = &mut state.counters[1];
            match failure {
                0 => counter.size_status = PDH_INVALID_DATA,
                1 => counter.info_size = 1,
                2 => counter.fill_status = PDH_INVALID_DATA,
                3 => counter.fill_status = PDH_MORE_DATA,
                4 => counter.native_type = 0x4003_0401, // PERF_SAMPLE_BASE
                5 => counter.fill_size = Some(1),
                6 => counter.fill_size = Some(counter.info_size + 1),
                _ => unreachable!(),
            }
        }
        let mut query = open_fake(&state, &configs).unwrap();
        assert_eq!(query.startup_failures.len(), 1);
        assert_eq!(query.startup_failures[0].counter_index, 1);
        assert_eq!(query.startup_failures[0].path, configs[1].path);
        if failure == 2 || failure == 3 {
            assert!(
                query.startup_failures[0]
                    .error
                    .contains("PdhGetCounterInfoW")
            );
        }
        let sample = query.collect().unwrap();
        assert!(sample.failures.is_empty());
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].counter_index, 0);
        assert_eq!(state.borrow().adds, 2);
        assert_eq!(state.borrow().removes, vec![1]);
        assert_eq!(state.borrow().handles.len(), 1);
        drop(query);
        assert_eq!(state.borrow().closes, 1);
        assert!(state.borrow().handles.is_empty());
    }
}

/// Scenario: An unsupported counter cannot be removed after inspection, but a healthy counter loads.
/// Guarantees: Cleanup failure is reported without losing healthy points, and query closure releases the retained handle.
#[test]
fn failed_counter_removal_remains_owned_and_reported() {
    let (state, configs) = fake(vec![
        CounterFixture::new(0, Number::Integer(1)),
        CounterFixture::new(0x4003_0401, Number::Integer(2)), // PERF_SAMPLE_BASE
    ]);
    state.borrow_mut().remove_status = PDH_INVALID_DATA;
    let mut query = open_fake(&state, &configs).unwrap();
    assert_eq!(query.startup_failures.len(), 1);
    assert!(
        query.startup_failures[0]
            .error
            .contains("unsupported native counter type")
    );
    assert!(query.startup_failures[0].error.contains("PdhRemoveCounter"));
    assert_eq!(state.borrow().handles.len(), 2);
    let sample = query.collect().unwrap();
    assert!(sample.failures.is_empty());
    assert_eq!(sample.points.len(), 1);
    assert_eq!(sample.points[0].counter_index, 0);
    drop(query);
    assert_eq!(state.borrow().closes, 1);
    assert!(state.borrow().handles.is_empty());
}

/// Scenario: Variable-sized metadata has native type zero, and direct counts use decimal scaling.
/// Guarantees: Padded metadata storage preserves valid zero types and exact integers; configured scaling alone controls double output.
#[test]
fn direct_reads_preserve_integers_and_apply_only_configured_scaling() {
    let (state, mut configs) = fake(vec![
        CounterFixture::new(0, Number::Integer(i64::MAX)),
        CounterFixture::new(0x0001_0100, Number::Integer(7)), // PERF_COUNTER_LARGE_RAWCOUNT
    ]);
    state.borrow_mut().counters[0].info_size += 1;
    configs[1].scale_power10 = -1;
    let mut query = open_fake(&state, &configs).unwrap();
    for _ in 0..3 {
        let sample = query.collect().unwrap();
        assert_eq!(
            sample.points[0].value,
            SampleValue::Value(Number::Integer(i64::MAX))
        );
        assert_eq!(
            sample.points[1].value,
            SampleValue::Value(Number::Double(0.7))
        );
        assert!(sample.failures.is_empty());
    }
    assert!(
        state
            .borrow()
            .formats
            .iter()
            .all(|(_, format)| *format == ValueFormat::Integer)
    );
    assert!(state.borrow().raws.is_empty());
    drop(query);
    let state = state.borrow();
    assert_eq!(
        (state.opens, state.adds, state.collects, state.closes),
        (1, 2, 3, 1)
    );
}

/// Scenario: A two-sample calculation seeds readiness with a zero raw base, then PDH returns a percentage above 100.
/// Guarantees: A zero history value still completes warm-up and the calculated value remains an unclamped scaled double.
#[test]
fn calculated_reads_warm_up_and_do_not_cap_percentages() {
    let (state, mut configs) = fake(vec![CounterFixture::new(
        0x2051_0500, // PERF_100NSEC_TIMER
        Number::Double(250.0),
    )]);
    state.borrow_mut().counters[0].base = 0;
    configs[0].scale_power10 = -2;
    let mut query = open_fake(&state, &configs).unwrap();
    let warmup = query.collect().unwrap();
    assert_eq!(warmup.points[0].value, SampleValue::NoObservation);
    assert!(warmup.failures.is_empty());
    assert!(into_otap(&configs, warmup).unwrap().is_none());
    let sample = query.collect().unwrap();
    assert_eq!(
        sample.points[0].value,
        SampleValue::Value(Number::Double(2.5))
    );
    assert!(into_otap(&configs, sample).unwrap().is_some());
    assert_eq!(state.borrow().formats, vec![(0, ValueFormat::Double)]);
}

/// Scenario: An elapsed-time counter is collected as Gauge or cumulative non-monotonic Sum with configured scaling.
/// Guarantees: The first sample emits PDH's double without a raw read or artificial warm-up, and projection accepts both kinds.
#[test]
fn elapsed_time_emits_from_first_sample() {
    for kind in [MetricKind::Gauge, MetricKind::Sum] {
        let (state, mut configs) = fake(vec![CounterFixture::new(
            0x3024_0500, // PERF_ELAPSED_TIME
            Number::Double(42.5),
        )]);
        configs[0].kind = kind;
        configs[0].scale_power10 = -1;
        let mut query = open_fake(&state, &configs).unwrap();
        let sample = query.collect().unwrap();
        assert!(sample.failures.is_empty());
        assert_eq!(
            sample.points[0].value,
            SampleValue::Value(Number::Double(4.25))
        );
        assert!(state.borrow().raws.is_empty());
        assert_eq!(state.borrow().formats, vec![(0, ValueFormat::Double)]);
        assert!(into_otap(&configs, sample).unwrap().is_some());
    }
}

/// Scenario: Each multi-timer has a zero base on the first or a warmed sample, then a positive current base.
/// Guarantees: Zero diagnoses only its interval, advances the raw baseline and preserves healthy peers; the next sample emits without extra warm-up.
#[test]
fn multi_timer_invalid_base_preserves_readiness() {
    for native_type in [
        0x2241_0500, // PERF_COUNTER_MULTI_TIMER
        0x2341_0500, // PERF_COUNTER_MULTI_TIMER_INV
        0x2251_0500, // PERF_100NSEC_MULTI_TIMER
        0x2351_0500, // PERF_100NSEC_MULTI_TIMER_INV
    ] {
        for warmed in [false, true] {
            let (state, mut configs) = fake(vec![
                CounterFixture::new(native_type, Number::Double(250.0)),
                CounterFixture::new(0, Number::Integer(7)),
            ]);
            configs[0].kind = if warmed {
                MetricKind::Gauge
            } else {
                MetricKind::Sum
            };
            configs[0].scale_power10 = -2;
            let mut query = open_fake(&state, &configs).unwrap();
            if warmed {
                let sample = query.collect().unwrap();
                assert!(sample.failures.is_empty());
                assert_eq!(sample.points[0].value, SampleValue::NoObservation);
                assert_eq!(query.counters[0].previous_base, Some(10));
                assert_eq!(state.borrow().formats, vec![(1, ValueFormat::Integer)]);
            }
            state.borrow_mut().counters[0].base = 20;
            state.borrow_mut().counters[0].multi_count = 0;
            let invalid = query.collect().unwrap();
            assert_eq!(invalid.failures.len(), 1);
            assert_eq!(invalid.failures[0].counter_index, 0);
            assert!(invalid.failures[0].error.contains(&configs[0].path));
            assert!(
                invalid.failures[0]
                    .error
                    .contains("multi-counter base denominator must be positive, got 0")
            );
            assert_eq!(invalid.points.len(), 1);
            assert_eq!(invalid.points[0].counter_index, 1);
            assert_eq!(
                invalid.points[0].value,
                SampleValue::Value(Number::Integer(7))
            );
            assert_eq!(query.counters[0].previous_base, Some(20));
            assert!(state.borrow().formats.iter().all(|(index, _)| *index == 1));
            assert!(into_otap(&configs, invalid).unwrap().is_some());
            state.borrow_mut().counters[0].base = 30;
            state.borrow_mut().counters[0].multi_count = u32::MAX;
            let recovered = query.collect().unwrap();
            assert!(recovered.failures.is_empty());
            assert_eq!(recovered.start_time_unix_nano, 100);
            assert_eq!(
                recovered.points[0].value,
                SampleValue::Value(Number::Double(2.5))
            );
            assert_eq!(query.counters[0].previous_base, Some(30));
            assert!(state.borrow().formats.contains(&(0, ValueFormat::Double)));
            assert!(into_otap(&configs, recovered).unwrap().is_some());
        }
    }
}

/// Scenario: A warmed multi-timer has a raw API or CStatus omission/failure together with zero MultiCount.
/// Guarantees: Invalid raw samples still clear readiness rather than taking the zero-base exception; healthy peers keep emitting.
#[test]
fn multi_timer_raw_failures_still_reset_readiness() {
    for (status, cstatus) in [
        (PDH_INVALID_DATA, PDH_CSTATUS_INVALID_DATA),
        (PDH_INVALID_ARGUMENT, PDH_INVALID_ARGUMENT),
    ] {
        for fail_status in [false, true] {
            let (state, configs) = fake(vec![
                CounterFixture::new(0x2351_0500, Number::Double(25.0)), // PERF_100NSEC_MULTI_TIMER_INV
                CounterFixture::new(0, Number::Integer(7)),
            ]);
            let mut query = open_fake(&state, &configs).unwrap();
            let _ = query.collect().unwrap();
            {
                let mut state = state.borrow_mut();
                state.counters[0].multi_count = 0;
                if fail_status {
                    state.counters[0].raw_status = status;
                } else {
                    state.counters[0].raw_cstatus = cstatus;
                }
            }
            let invalid = query.collect().unwrap();
            assert_eq!(
                invalid.failures.len(),
                usize::from(status == PDH_INVALID_ARGUMENT)
            );
            assert_eq!(query.counters[0].previous_base, None);
            assert_eq!(
                invalid.points.last().unwrap().value,
                SampleValue::Value(Number::Integer(7))
            );
            assert!(state.borrow().formats.iter().all(|(index, _)| *index == 1));
            {
                let mut state = state.borrow_mut();
                state.counters[0].raw_status = 0;
                state.counters[0].raw_cstatus = PDH_CSTATUS_VALID_DATA;
                state.counters[0].multi_count = 4;
            }
            let warming = query.collect().unwrap();
            assert!(warming.failures.is_empty());
            assert_eq!(warming.points[0].value, SampleValue::NoObservation);
            let recovered = query.collect().unwrap();
            assert!(recovered.failures.is_empty());
            assert_eq!(
                recovered.points[0].value,
                SampleValue::Value(Number::Double(25.0))
            );
        }
    }
}

/// Scenario: A raw fraction's current denominator is zero, negative, then positive.
/// Guarantees: Invalid denominators fail locally and a valid fraction needs no two-sample warm-up.
#[test]
fn raw_fraction_requires_a_positive_current_base() {
    let (state, configs) = fake(vec![CounterFixture::new(
        0x2002_0400, // PERF_RAW_FRACTION
        Number::Double(25.0),
    )]);
    let mut query = open_fake(&state, &configs).unwrap();
    for base in [0, -1] {
        state.borrow_mut().counters[0].base = base;
        let sample = query.collect().unwrap();
        assert!(sample.points.is_empty());
        assert_eq!(sample.failures.len(), 1);
        assert!(
            sample.failures[0]
                .error
                .contains("base denominator must be positive")
        );
    }
    state.borrow_mut().counters[0].base = 100;
    let sample = query.collect().unwrap();
    assert_eq!(
        sample.points[0].value,
        SampleValue::Value(Number::Double(25.0))
    );
    assert!(sample.failures.is_empty());
}

/// Scenario: An average's base warms up, stays idle, advances, resets, then advances again.
/// Guarantees: Idle and reset intervals omit points without failures; reset recovery re-warms without restarting cumulative time.
#[test]
fn base_progression_and_reset_preserve_sequence_start() {
    let (state, configs) = fake(vec![CounterFixture::new(
        0x4002_0500, // PERF_AVERAGE_BULK
        Number::Double(4.0),
    )]);
    let mut query = open_fake(&state, &configs).unwrap();
    for (base, expected) in [
        (10, SampleValue::NoObservation),
        (10, SampleValue::NoObservation),
        (12, SampleValue::Value(Number::Double(4.0))),
        (2, SampleValue::NoObservation),
        (3, SampleValue::NoObservation),
        (4, SampleValue::Value(Number::Double(4.0))),
    ] {
        state.borrow_mut().counters[0].base = base;
        let sample = query.collect().unwrap();
        assert_eq!(sample.start_time_unix_nano, 100);
        assert_eq!(sample.points[0].value, expected);
        assert!(sample.failures.is_empty());
    }
}

/// Scenario: Formatted reads report missing or invalid data, or a negative calculation status.
/// Guarantees: Expected API and data statuses omit and reset readiness while preserving healthy peers and cumulative time.
#[test]
fn formatted_statuses_omit_and_reset_readiness() {
    for native_type in [
        0x0000_0000, // PERF_COUNTER_RAWCOUNT_HEX
        0x2002_0400, // PERF_RAW_FRACTION
        0x1041_0400, // PERF_COUNTER_COUNTER
        0x4002_0500, // PERF_AVERAGE_BULK
        0x3024_0500, // PERF_ELAPSED_TIME
        0x2351_0500, // PERF_100NSEC_MULTI_TIMER_INV
    ] {
        let number = if native_type == 0 {
            Number::Integer(3)
        } else {
            Number::Double(3.0)
        };
        let needs_warmup = matches!(native_type, 0x1041_0400 | 0x4002_0500 | 0x2351_0500);
        for (omission, data_omission) in OMISSION_STATUSES.into_iter().zip(DATA_OMISSION_STATUSES) {
            for (status, cstatus) in [
                (omission, PDH_CSTATUS_VALID_DATA),
                (0, data_omission),
                (PDH_INVALID_DATA, data_omission),
            ] {
                let (state, configs) = fake(vec![
                    CounterFixture::new(native_type, number),
                    CounterFixture::new(0, Number::Integer(7)),
                ]);
                let mut query = open_fake(&state, &configs).unwrap();
                let _ = query.collect().unwrap();
                state.borrow_mut().counters[0].base = 12;
                assert_eq!(
                    query.collect().unwrap().points[0].value,
                    SampleValue::Value(number)
                );
                {
                    let mut state = state.borrow_mut();
                    state.counters[0].base = 15;
                    state.counters[0].format_status = status;
                    state.counters[0].format_cstatus = cstatus;
                }
                let reset = query.collect().unwrap();
                assert!(reset.failures.is_empty());
                assert_eq!(reset.points[0].value, SampleValue::NoObservation);
                assert_eq!(
                    reset.points[1].value,
                    SampleValue::Value(Number::Integer(7))
                );
                assert_eq!(reset.start_time_unix_nano, 100);
                {
                    let mut state = state.borrow_mut();
                    state.counters[0].base = 1;
                    state.counters[0].format_status = 0;
                    state.counters[0].format_cstatus = PDH_CSTATUS_VALID_DATA;
                }
                let warming = query.collect().unwrap();
                assert!(warming.failures.is_empty());
                assert_eq!(
                    warming.points[0].value,
                    if needs_warmup {
                        SampleValue::NoObservation
                    } else {
                        SampleValue::Value(number)
                    }
                );
                state.borrow_mut().counters[0].base = 2;
                let recovered = query.collect().unwrap();
                assert!(recovered.failures.is_empty());
                assert_eq!(recovered.points[0].value, SampleValue::Value(number));
                assert_eq!(recovered.start_time_unix_nano, 100);
                assert_eq!((state.borrow().opens, state.borrow().adds), (1, 2));
            }
        }
    }
}

/// Scenario: A genuine formatted API error accompanies valid or expected-omission data status.
/// Guarantees: An unrelated API error cannot be hidden by data status or discard valid raw history.
#[test]
fn real_formatted_errors_remain_failures() {
    for (status, cstatus) in [
        (PDH_INVALID_ARGUMENT, PDH_CSTATUS_VALID_DATA),
        (PDH_INVALID_ARGUMENT, PDH_CSTATUS_INVALID_DATA),
        (PDH_INVALID_ARGUMENT, PDH_NO_DATA),
        (PDH_INVALID_ARGUMENT, PDH_CALC_NEGATIVE_VALUE),
    ] {
        let (state, configs) = fake(vec![
            CounterFixture::new(0x1041_0400, Number::Double(3.0)), // PERF_COUNTER_COUNTER
            CounterFixture::new(0, Number::Integer(7)),
        ]);
        let mut query = open_fake(&state, &configs).unwrap();
        let _ = query.collect().unwrap();
        state.borrow_mut().counters[0].format_status = status;
        state.borrow_mut().counters[0].format_cstatus = cstatus;
        let sample = query.collect().unwrap();
        assert_eq!(sample.failures.len(), 1);
        assert_eq!(sample.failures[0].counter_index, 0);
        assert!(
            sample.failures[0]
                .error
                .contains("PdhGetFormattedCounterValue")
        );
        assert!(sample.failures[0].error.contains("failed with PDH status"));
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].counter_index, 1);
        {
            let mut state = state.borrow_mut();
            state.counters[0].format_status = 0;
            state.counters[0].format_cstatus = PDH_CSTATUS_VALID_DATA;
        }
        let recovered = query.collect().unwrap();
        assert!(recovered.failures.is_empty());
        assert_eq!(
            recovered.points[0].value,
            SampleValue::Value(Number::Double(3.0))
        );
        assert_eq!(recovered.start_time_unix_nano, 100);
    }
}

/// Scenario: One direct read fails while an average peer advances its base, then both recover.
/// Guarantees: Failure never skips a healthy peer's base update, read retries reuse handles, and projection retains healthy points.
#[test]
fn counter_failures_preserve_peers_and_retry_existing_handles() {
    for fail_status in [false, true] {
        let (state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(7)),
            CounterFixture::new(0x3002_0400, Number::Double(2.0)), // PERF_AVERAGE_TIMER
        ]);
        let mut query = open_fake(&state, &configs).unwrap();
        let _ = query.collect().unwrap();
        {
            let mut state = state.borrow_mut();
            if fail_status {
                state.counters[0].format_status = PDH_INVALID_ARGUMENT;
            } else {
                state.counters[0].format_cstatus = PDH_INVALID_ARGUMENT;
            }
            state.counters[1].base = 12;
        }
        let sample = query.collect().unwrap();
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].counter_index, 1);
        assert_eq!(sample.failures[0].counter_index, 0);
        assert!(sample.failures[0].error.contains(&configs[0].path));
        assert!(into_otap(&configs, sample).unwrap().is_some());
        {
            let mut state = state.borrow_mut();
            state.counters[0].format_status = 0;
            state.counters[0].format_cstatus = PDH_CSTATUS_NEW_DATA;
        }
        let recovered = query.collect().unwrap();
        assert!(recovered.failures.is_empty());
        assert_eq!(
            recovered.points[0].value,
            SampleValue::Value(Number::Integer(7))
        );
        assert_eq!(recovered.points[1].value, SampleValue::NoObservation);
        assert_eq!(recovered.start_time_unix_nano, 100);
        assert_eq!((state.borrow().opens, state.borrow().adds), (1, 2));
    }
}

/// Scenario: A raw read reports an expected omission through its API or data status.
/// Guarantees: All expected statuses omit locally and re-warm without stale bases, failures, or handle recreation.
#[test]
fn raw_statuses_omit_and_clear_base_readiness() {
    for (omission, data_omission) in OMISSION_STATUSES.into_iter().zip(DATA_OMISSION_STATUSES) {
        for fail_status in [false, true] {
            let (state, configs) = fake(vec![
                CounterFixture::new(0x20C2_0400, Number::Double(50.0)), // PERF_SAMPLE_FRACTION
                CounterFixture::new(0, Number::Integer(7)),
            ]);
            let mut query = open_fake(&state, &configs).unwrap();
            let _ = query.collect().unwrap();
            state.borrow_mut().counters[0].base = 12;
            assert_eq!(
                query.collect().unwrap().points[0].value,
                SampleValue::Value(Number::Double(50.0))
            );
            {
                let mut state = state.borrow_mut();
                if fail_status {
                    state.counters[0].raw_status = omission;
                } else {
                    state.counters[0].raw_cstatus = data_omission;
                }
            }
            let omitted = query.collect().unwrap();
            assert!(omitted.failures.is_empty());
            assert_eq!(omitted.points[0].value, SampleValue::NoObservation);
            assert_eq!(
                omitted.points[1].value,
                SampleValue::Value(Number::Integer(7))
            );
            assert!(into_otap(&configs, omitted).unwrap().is_some());
            {
                let mut state = state.borrow_mut();
                state.counters[0].raw_status = 0;
                state.counters[0].raw_cstatus = PDH_CSTATUS_NEW_DATA;
                state.counters[0].base = 1;
            }
            let warming = query.collect().unwrap();
            assert!(warming.failures.is_empty());
            assert_eq!(warming.points[0].value, SampleValue::NoObservation);
            state.borrow_mut().counters[0].base = 2;
            let recovered = query.collect().unwrap();
            assert_eq!(
                recovered.points[0].value,
                SampleValue::Value(Number::Double(50.0))
            );
            assert_eq!(recovered.start_time_unix_nano, 100);
            assert!(recovered.failures.is_empty());
            assert_eq!((state.borrow().opens, state.borrow().adds), (1, 2));
        }
    }
}

/// Scenario: Formatted invalid-data returns carry a missing-instance or unexpected data status.
/// Guarantees: Specific counter failures are reported, healthy peers survive, and calculated counters re-warm on recovery.
#[test]
fn formatted_invalid_data_preserves_specific_failures() {
    for status in [0, PDH_INVALID_DATA] {
        for cstatus in [PDH_CSTATUS_NO_INSTANCE, PDH_INVALID_DATA] {
            let (state, configs) = fake(vec![
                CounterFixture::new(0x1041_0400, Number::Double(3.0)), // PERF_COUNTER_COUNTER
                CounterFixture::new(0, Number::Integer(7)),
            ]);
            let mut query = open_fake(&state, &configs).unwrap();
            let _ = query.collect().unwrap();
            {
                let mut state = state.borrow_mut();
                state.counters[0].format_status = status;
                state.counters[0].format_cstatus = cstatus;
            }
            let sample = query.collect().unwrap();
            assert_eq!(sample.failures.len(), 1);
            assert_eq!(sample.failures[0].counter_index, 0);
            assert!(
                sample.failures[0]
                    .error
                    .contains("formatted counter CStatus")
            );
            assert!(
                sample.failures[0]
                    .error
                    .contains("failed with counter data status")
            );
            assert!(
                sample.failures[0]
                    .error
                    .contains(&format!("0x{cstatus:08X}"))
            );
            assert_eq!(sample.points.len(), 1);
            assert_eq!(sample.points[0].counter_index, 1);
            assert_eq!(
                sample.points[0].value,
                SampleValue::Value(Number::Integer(7))
            );
            {
                let mut state = state.borrow_mut();
                state.counters[0].format_status = 0;
                state.counters[0].format_cstatus = PDH_CSTATUS_VALID_DATA;
            }
            let warming = query.collect().unwrap();
            assert!(warming.failures.is_empty());
            assert_eq!(warming.points[0].value, SampleValue::NoObservation);
            let recovered = query.collect().unwrap();
            assert!(recovered.failures.is_empty());
            assert_eq!(
                recovered.points[0].value,
                SampleValue::Value(Number::Double(3.0))
            );
            assert_eq!(recovered.start_time_unix_nano, 100);
        }
    }
}

/// Scenario: A warmed counter's raw API or data status reports an unrelated error beside a healthy direct counter.
/// Guarantees: API errors cannot be masked by data status; all raw failures remain local and require fresh warm-up on recovery.
#[test]
fn real_raw_errors_remain_failures() {
    for (status, cstatus) in [
        (PDH_INVALID_ARGUMENT, PDH_CSTATUS_VALID_DATA),
        (PDH_INVALID_ARGUMENT, PDH_CSTATUS_INVALID_DATA),
        (PDH_INVALID_ARGUMENT, PDH_NO_DATA),
        (0, PDH_INVALID_ARGUMENT),
        (0, PDH_CSTATUS_NO_INSTANCE),
        (0, PDH_INVALID_DATA),
    ] {
        let (state, configs) = fake(vec![
            CounterFixture::new(0x1041_0400, Number::Double(3.0)), // PERF_COUNTER_COUNTER
            CounterFixture::new(0, Number::Integer(7)),
        ]);
        let mut query = open_fake(&state, &configs).unwrap();
        let _ = query.collect().unwrap();
        state.borrow_mut().counters[0].raw_status = status;
        state.borrow_mut().counters[0].raw_cstatus = cstatus;
        let sample = query.collect().unwrap();
        assert_eq!(sample.failures.len(), 1);
        assert_eq!(sample.failures[0].counter_index, 0);
        assert!(sample.failures[0].error.contains(&configs[0].path));
        let status_label = if status == 0 {
            "failed with counter data status"
        } else {
            "failed with PDH status"
        };
        assert!(sample.failures[0].error.contains(status_label));
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].counter_index, 1);
        assert_eq!(
            sample.points[0].value,
            SampleValue::Value(Number::Integer(7))
        );
        {
            let mut state = state.borrow_mut();
            state.counters[0].raw_status = 0;
            state.counters[0].raw_cstatus = PDH_CSTATUS_VALID_DATA;
        }
        let warming = query.collect().unwrap();
        assert!(warming.failures.is_empty());
        assert_eq!(warming.points[0].value, SampleValue::NoObservation);
        let recovered = query.collect().unwrap();
        assert!(recovered.failures.is_empty());
        assert_eq!(
            recovered.points[0].value,
            SampleValue::Value(Number::Double(3.0))
        );
        assert_eq!(recovered.start_time_unix_nano, 100);
    }
}

/// Scenario: A calculation returns a non-finite value or configured scaling overflows.
/// Guarantees: Only that counter fails; a healthy direct value still projects.
#[test]
fn scaling_failures_are_counter_local() {
    for number in [f64::NAN, f64::INFINITY, f64::MAX] {
        let (state, mut configs) = fake(vec![
            CounterFixture::new(0x2002_0400, Number::Double(number)), // PERF_RAW_FRACTION
            CounterFixture::new(0, Number::Integer(42)),
        ]);
        configs[0].scale_power10 = 18;
        let mut query = open_fake(&state, &configs).unwrap();
        let sample = query.collect().unwrap();
        assert_eq!(sample.failures.len(), 1);
        assert_eq!(sample.failures[0].counter_index, 0);
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].counter_index, 1);
        assert!(into_otap(&configs, sample).unwrap().is_some());
    }
}

/// Scenario: Query-wide collection fails, then succeeds on the original query.
/// Guarantees: Calculations re-warm and retries neither rebuild handles nor reset cumulative time.
#[test]
fn query_failure_retries_without_rebuilding_or_restarting_sequence() {
    let (state, configs) = fake(vec![
        CounterFixture::new(0, Number::Integer(7)),
        CounterFixture::new(0x1041_0400, Number::Double(3.0)), // PERF_COUNTER_COUNTER
    ]);
    let mut query = open_fake(&state, &configs).unwrap();
    let _ = query.collect().unwrap();
    state.borrow_mut().collect_status = PDH_INVALID_ARGUMENT;
    assert!(query.collect().is_err());
    state.borrow_mut().collect_status = 0;
    let recovered = query.collect().unwrap();
    assert_eq!(
        recovered.points[0].value,
        SampleValue::Value(Number::Integer(7))
    );
    assert_eq!(recovered.points[1].value, SampleValue::NoObservation);
    assert_eq!(recovered.start_time_unix_nano, 100);
    assert_eq!((state.borrow().opens, state.borrow().adds), (1, 2));
    assert_eq!(
        query.collect().unwrap().points[1].value,
        SampleValue::Value(Number::Double(3.0))
    );
}

/// Scenario: A warmed query reports invalid, missing, or negative-calculation data.
/// Guarantees: Every counter is omitted without stale reads or failures, then existing handles recover with unchanged cumulative time.
#[test]
fn query_statuses_omit_and_clear_all_readiness() {
    for omission in OMISSION_STATUSES {
        let (state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(7)),
            CounterFixture::new(0x4002_0500, Number::Double(3.0)), // PERF_AVERAGE_BULK
        ]);
        let mut query = open_fake(&state, &configs).unwrap();
        let _ = query.collect().unwrap();
        state.borrow_mut().counters[1].base = 12;
        assert_eq!(
            query.collect().unwrap().points[1].value,
            SampleValue::Value(Number::Double(3.0))
        );
        let reads = (state.borrow().raws.len(), state.borrow().formats.len());
        state.borrow_mut().collect_status = omission;
        state.borrow_mut().timestamp = 110;
        let sample = query.collect().unwrap();
        assert!(sample.failures.is_empty());
        assert_eq!(sample.points.len(), 2);
        assert!(
            sample
                .points
                .iter()
                .all(|point| point.value == SampleValue::NoObservation)
        );
        assert_eq!(sample.start_time_unix_nano, 100);
        assert_eq!(sample.timestamp_unix_nano, 110);
        assert!(into_otap(&configs, sample).unwrap().is_none());
        assert_eq!(
            (state.borrow().raws.len(), state.borrow().formats.len()),
            reads
        );
        state.borrow_mut().collect_status = 0;
        state.borrow_mut().counters[1].base = 1;
        let warming = query.collect().unwrap();
        assert!(warming.failures.is_empty());
        assert_eq!(
            warming.points[0].value,
            SampleValue::Value(Number::Integer(7))
        );
        assert_eq!(warming.points[1].value, SampleValue::NoObservation);
        state.borrow_mut().counters[1].base = 2;
        let recovered = query.collect().unwrap();
        assert!(recovered.failures.is_empty());
        assert_eq!(
            recovered.points[1].value,
            SampleValue::Value(Number::Double(3.0))
        );
        assert_eq!(recovered.start_time_unix_nano, 100);
        assert_eq!((state.borrow().opens, state.borrow().adds), (1, 2));
    }
}

/// Scenario: A no-data collection has an invalid timestamp, then a valid clock rollback.
/// Guarantees: Omissions still enforce positive timestamps and only rollback restarts cumulative time.
#[test]
fn query_omissions_preserve_timestamp_validation_and_rollback() {
    let (state, configs) = fake(vec![CounterFixture::new(0, Number::Integer(7))]);
    let mut query = open_fake(&state, &configs).unwrap();
    state.borrow_mut().collect_status = PDH_NO_DATA;
    state.borrow_mut().timestamp = 0;
    assert!(matches!(query.collect(), Err(Error::InvalidSample(_))));
    state.borrow_mut().timestamp = 90;
    let omitted = query.collect().unwrap();
    assert!(omitted.failures.is_empty());
    assert_eq!(omitted.points[0].value, SampleValue::NoObservation);
    assert_eq!(omitted.start_time_unix_nano, 90);
    assert_eq!(omitted.timestamp_unix_nano, 90);
    state.borrow_mut().collect_status = 0;
    state.borrow_mut().timestamp = 120;
    let recovered = query.collect().unwrap();
    assert_eq!(recovered.start_time_unix_nano, 90);
    assert_eq!(
        recovered.points[0].value,
        SampleValue::Value(Number::Integer(7))
    );
}

/// Scenario: The observation clock advances, repeats, then rolls back while counters stay readable.
/// Guarantees: Only opening and rollback begin sequences; every sample satisfies projection timestamps.
#[test]
fn rollback_alone_restarts_cumulative_sequence() {
    let (state, configs) = fake(vec![CounterFixture::new(0, Number::Integer(7))]);
    let mut query = open_fake(&state, &configs).unwrap();
    for (timestamp, start) in [(110, 100), (110, 100), (90, 90), (120, 90)] {
        state.borrow_mut().timestamp = timestamp;
        let sample = query.collect().unwrap();
        assert_eq!(sample.timestamp_unix_nano, timestamp);
        assert_eq!(sample.start_time_unix_nano, start);
        assert!(into_otap(&configs, sample).unwrap().is_some());
    }
}

/// Scenario: Opening or a collection receives a non-positive observation timestamp.
/// Guarantees: Invalid startup timestamps release the query; invalid collections never enter projection or corrupt the last valid sequence.
#[test]
fn rejects_non_positive_timestamps() {
    let (state, configs) = fake(vec![
        CounterFixture::new(0, Number::Integer(7)),
        CounterFixture::new(0x4002_0500, Number::Double(2.0)), // PERF_AVERAGE_BULK
    ]);
    state.borrow_mut().timestamp = 0;
    assert!(open_fake(&state, &configs).is_err());
    assert_eq!((state.borrow().opens, state.borrow().closes), (1, 1));
    assert_eq!(state.borrow().adds, 0);
    state.borrow_mut().timestamp = 100;
    let mut query = open_fake(&state, &configs).unwrap();
    let _ = query.collect().unwrap();
    for timestamp in [0, -1] {
        state.borrow_mut().timestamp = timestamp;
        assert!(query.collect().is_err());
    }
    state.borrow_mut().timestamp = 110;
    state.borrow_mut().counters[1].base = 1;
    let sample = query.collect().unwrap();
    assert_eq!(sample.start_time_unix_nano, 100);
    assert_eq!(sample.points[1].value, SampleValue::NoObservation);
    assert!(sample.failures.is_empty());
}

/// Scenario: A removed counter token is reused after another counter is added to the same adapter.
/// Guarantees: Repeated removal and reads return adapter token errors rather than native statuses; removed slots cannot alias new counters.
#[test]
#[ignore = "requires live Windows performance counters; run explicitly with --ignored"]
fn live_pdh_adapter_rejects_removed_tokens() {
    let path = r"\Memory\Available Bytes";
    let mut api = NativePdh::open("removed".to_owned()).unwrap();
    let removed = api.add(path).unwrap();
    api.remove(&removed, path).unwrap();
    let live = api.add(path).unwrap();
    for result in [
        api.remove(&removed, path),
        api.counter_type(&removed, path).map(|_| ()),
        api.raw(&removed, path).map(|_| ()),
        api.formatted(&removed, path, ValueFormat::Integer)
            .map(|_| ()),
    ] {
        let error = result.unwrap_err();
        assert!(matches!(&error, Error::InvalidCounterToken { path: failed } if failed == path));
        assert_eq!(
            error.to_string(),
            format!("invalid performance-counter token for {path}")
        );
    }
    assert!(api.collect().unwrap());
    assert!(api.counter_type(&live, path).is_ok());
}

struct CounterProcess(Child);

impl Drop for CounterProcess {
    fn drop(&mut self) {
        drop(self.0.stdin.take());
        let _ = self.0.wait();
    }
}

fn spawn_counter_process(executable: &Path) -> CounterProcess {
    // Keeping stdin open holds cmd alive; closing it also releases the child if the test panics.
    let mut child = CounterProcess(
        Command::new(executable)
            .args(["/d", "/q", "/c", "echo ready & set /p wait="])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut ready = String::new();
    let _ = BufReader::new(child.0.stdout.take().unwrap())
        .read_line(&mut ready)
        .unwrap();
    assert_eq!(ready.trim(), "ready");
    child
}

/// Scenario: Parsed configuration names duplicate live process instances using #0 and #1.
/// Guarantees: The normalized omitted-zero and indexed paths read exactly the fixture's process IDs, not unrelated processes, and project checked samples.
#[test]
#[ignore = "requires live Windows performance counters; run explicitly with --ignored"]
fn live_pdh_collects_parsed_instance_paths() {
    let directory = tempfile::tempdir().unwrap();
    let instance = format!("otpd{:08x}", std::process::id());
    let executable = directory.path().join(format!("{instance}.exe"));
    let source = std::env::var_os("COMSPEC").expect("COMSPEC must identify cmd.exe");
    let _ = std::fs::copy(source, &executable).unwrap();
    let processes = [
        spawn_counter_process(&executable),
        spawn_counter_process(&executable),
    ];
    let config = RuntimeConfig::from_json(&serde_json::json!({
        "metrics": {
            "process.id": {
                "description": "Process identifier.",
                "unit": "1",
                "gauge": {}
            }
        },
        "perfcounters": [{
            "object": "Process",
            "instances": [format!("{instance}#0"), format!("{instance}#1")],
            "counters": [{"name": "ID Process", "metric": "process.id"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        config.counters[0].path,
        format!(r"\Process({instance})\ID Process")
    );
    assert_eq!(
        config.counters[1].path,
        format!(r"\Process({instance}#1)\ID Process")
    );
    let mut query = Query::open(&config.counters, "parsed".to_owned()).unwrap();
    assert!(query.startup_failures.is_empty());
    let sample = query.collect().unwrap();
    assert!(sample.failures.is_empty(), "{:?}", sample.failures);
    assert_eq!(sample.points.len(), 2);
    let ids: BTreeSet<_> = sample
        .points
        .iter()
        .map(|point| match point.value {
            SampleValue::Value(Number::Integer(id)) if id > 0 => id,
            value => panic!("expected a positive process ID, got {value:?}"),
        })
        .collect();
    let expected: BTreeSet<_> = processes
        .iter()
        .map(|process| i64::from(process.0.id()))
        .collect();
    assert_eq!(ids, expected);
    assert!(into_otap(&config.counters, sample).unwrap().is_some());
    drop(query);
    drop(processes);
    directory.close().unwrap();
}

/// Scenario: Two native queries collect exact direct counters repeatedly and are dropped independently.
/// Guarantees: Integer points and cumulative start times remain valid across scrapes and query lifetimes.
#[test]
#[ignore = "requires live Windows performance counters; run explicitly with --ignored"]
fn live_pdh_direct_queries() {
    let configs = [config(r"\Memory\Available Bytes")];
    let mut first = Query::open(&configs, "first".to_owned()).unwrap();
    let mut second = Query::open(&configs, "second".to_owned()).unwrap();
    let start = first.start_time_unix_nano;
    for _ in 0..3 {
        for query in [&mut first, &mut second] {
            let sample = query.collect().unwrap();
            assert!(sample.failures.is_empty(), "{:?}", sample.failures);
            assert!(
                matches!(sample.points[0].value, SampleValue::Value(Number::Integer(value)) if value >= 0)
            );
            assert!(into_otap(&configs, sample).unwrap().is_some());
        }
        assert_eq!(first.start_time_unix_nano, start);
    }
    drop(first);
    assert!(second.collect().unwrap().failures.is_empty());
}

/// Scenario: Live PDH reads exact rates, timers, queues, fractions and averages.
/// Guarantees: Two-sample types warm up, every healthy calculation emits or intentionally omits, and projection succeeds.
#[test]
#[ignore = "requires live Windows performance counters; run explicitly with --ignored"]
fn live_pdh_calculated_counters() {
    let configs = [
        config(r"\System\Context Switches/sec"),
        config(r"\Processor(_Total)\% Processor Time"),
        config(r"\PhysicalDisk(_Total)\Avg. Disk Queue Length"),
        config(r"\Memory\% Committed Bytes In Use"),
        config(r"\PhysicalDisk(_Total)\Avg. Disk sec/Read"),
    ];
    let mut query = Query::open(&configs, "calculated".to_owned()).unwrap();
    let warming = query.collect().unwrap();
    assert!(warming.failures.is_empty(), "{:?}", warming.failures);
    for index in [0, 1, 2, 4] {
        assert_eq!(warming.points[index].value, SampleValue::NoObservation);
    }
    std::thread::sleep(std::time::Duration::from_millis(250));
    let sample = query.collect().unwrap();
    assert!(sample.failures.is_empty(), "{:?}", sample.failures);
    assert_eq!(sample.points.len(), configs.len());
    for point in &sample.points[..4] {
        assert!(
            matches!(point.value, SampleValue::Value(Number::Double(value)) if value.is_finite() && value >= 0.0)
        );
    }
    assert!(into_otap(&configs, sample).unwrap().is_some());
}

/// Scenario: Live Windows PDH reads system elapsed time and the provider's processor utility counter.
/// Guarantees: Both required counters load; elapsed time emits immediately while utility warms and then emits a finite double.
#[test]
#[ignore = "requires live Windows performance counters; run explicitly with --ignored"]
fn live_pdh_elapsed_and_utility_counters() {
    let configs = [
        config(r"\System\System Up Time"),
        config(r"\Processor Information(_Total)\% Processor Utility"),
    ];
    let mut query = Query::open(&configs, "elapsed-multi".to_owned()).unwrap();
    assert!(
        query.startup_failures.is_empty(),
        "{:?}",
        query.startup_failures
    );
    assert_eq!(query.counters.len(), configs.len());
    assert_eq!(query.counters[0].kind, CounterKind::FormattedSingleSample);
    // Some providers expose utility as an average, not a multi-timer.
    // Native fixtures cover all four multi-timer variants independently.
    assert!(matches!(
        query.counters[1].kind,
        CounterKind::CalculatedTwoSampleWithMultiBase | CounterKind::CalculatedTwoSampleWithBase
    ));
    let first = query.collect().unwrap();
    assert!(first.failures.is_empty(), "{:?}", first.failures);
    assert_eq!(first.points.len(), configs.len());
    assert!(matches!(
        first.points[0].value,
        SampleValue::Value(Number::Double(value)) if value.is_finite() && value >= 0.0
    ));
    assert_eq!(first.points[1].value, SampleValue::NoObservation);
    std::thread::sleep(std::time::Duration::from_millis(250));
    let second = query.collect().unwrap();
    assert!(second.failures.is_empty(), "{:?}", second.failures);
    assert_eq!(second.points.len(), configs.len());
    assert!(second.points.iter().all(|point| matches!(
        point.value,
        SampleValue::Value(Number::Double(value)) if value.is_finite() && value >= 0.0
    )));
    assert!(into_otap(&configs, second).unwrap().is_some());
}

/// Scenario: An invalid exact native path is configured alongside a valid counter, or alone.
/// Guarantees: Healthy native collection continues with startup diagnostics; opening fails only when no counter loads.
#[test]
#[ignore = "requires live Windows performance counters; run explicitly with --ignored"]
fn live_pdh_rejects_invalid_counter() {
    let configs = [
        config(r"\Memory\Available Bytes"),
        config(r"\Memory\otel-arrow-nonexistent-counter"),
    ];
    let mut query = Query::open(&configs, "invalid".to_owned()).unwrap();
    assert_eq!(query.startup_failures.len(), 1);
    assert_eq!(query.startup_failures[0].counter_index, 1);
    assert_eq!(query.startup_failures[0].path, configs[1].path);
    assert!(query.startup_failures[0].error.contains(&configs[1].path));
    let sample = query.collect().unwrap();
    assert!(sample.failures.is_empty());
    assert_eq!(sample.points.len(), 1);
    assert_eq!(sample.points[0].counter_index, 0);
    assert!(
        matches!(sample.points[0].value, SampleValue::Value(Number::Integer(value)) if value >= 0)
    );
    assert!(into_otap(&configs, sample).unwrap().is_some());
    assert!(
        matches!(Query::open(&configs[1..], "all-invalid".to_owned()), Err(Error::NoCounters { failures }) if failures.len() == 1)
    );
}
