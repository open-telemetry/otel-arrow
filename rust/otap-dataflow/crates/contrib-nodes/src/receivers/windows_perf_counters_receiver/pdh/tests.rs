// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::windows_perf_counters_receiver::config::MetricKind;
use crate::receivers::windows_perf_counters_receiver::model::Number;
use crate::receivers::windows_perf_counters_receiver::otap_builder::into_otap;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;
use windows_sys::Win32::System::Performance::{PDH_INVALID_ARGUMENT, PdhFormatFromRawValue};

const OMISSION_STATUSES: [u32; 5] = [
    PDH_INVALID_DATA,
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
        kind: MetricKind::UpDownCounter,
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
    raw_status: u32,
    raw_cstatus: u32,
    base: i64,
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
            raw_status: 0,
            raw_cstatus: PDH_CSTATUS_VALID_DATA,
            base: 10,
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
    formats: Vec<(usize, u32)>,
    raws: Vec<usize>,
}

struct FakePdh(Rc<RefCell<State>>);

fn fake(counters: Vec<CounterFixture>) -> (FakePdh, Rc<RefCell<State>>, Vec<CounterConfig>) {
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
    (FakePdh(Rc::clone(&state)), state, configs)
}

impl PdhApi for FakePdh {
    fn open(&mut self) -> Result<PDH_HQUERY, Error> {
        let mut state = self.0.borrow_mut();
        state.opens += 1;
        check("PdhOpenQueryW", "<query>", state.open_status)?;
        Ok(std::ptr::without_provenance_mut(100))
    }

    fn add(&mut self, _: PDH_HQUERY, path: &str) -> Result<PDH_HCOUNTER, Error> {
        let mut state = self.0.borrow_mut();
        let index = state.adds;
        state.adds += 1;
        check(
            "PdhAddEnglishCounterW",
            path,
            state.counters[index].add_status,
        )?;
        assert!(state.handles.insert(index + 1));
        Ok(std::ptr::without_provenance_mut(index + 1))
    }

    fn remove(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<(), Error> {
        let mut state = self.0.borrow_mut();
        state.removes.push(counter.addr() - 1);
        check("PdhRemoveCounter", path, state.remove_status)?;
        assert!(state.handles.remove(&counter.addr()));
        Ok(())
    }

    fn info(
        &mut self,
        counter: PDH_HCOUNTER,
        size: &mut u32,
        info: *mut PDH_COUNTER_INFO_W,
    ) -> u32 {
        let state = self.0.borrow();
        assert!(state.handles.contains(&counter.addr()));
        let fixture = &state.counters[counter.addr() - 1];
        if info.is_null() {
            *size = fixture.info_size;
            fixture.size_status
        } else {
            if fixture.fill_status == 0 {
                // SAFETY: inspect_counter supplies an aligned, full-header buffer.
                unsafe { (*info).dwType = fixture.native_type };
            }
            fixture.fill_status
        }
    }

    fn collect(&mut self, _: PDH_HQUERY) -> Result<(), Error> {
        let mut state = self.0.borrow_mut();
        state.collects += 1;
        check("PdhCollectQueryData", "<query>", state.collect_status)
    }

    fn raw(&mut self, counter: PDH_HCOUNTER, path: &str) -> Result<PDH_RAW_COUNTER, Error> {
        let mut state = self.0.borrow_mut();
        let index = counter.addr() - 1;
        assert!(state.handles.contains(&counter.addr()));
        state.raws.push(index);
        let fixture = &state.counters[index];
        check("PdhGetRawCounterValue", path, fixture.raw_status)?;
        Ok(PDH_RAW_COUNTER {
            CStatus: fixture.raw_cstatus,
            SecondValue: fixture.base,
            ..Default::default()
        })
    }

    fn formatted(&mut self, counter: PDH_HCOUNTER, format: u32) -> (u32, PDH_FMT_COUNTERVALUE) {
        let mut state = self.0.borrow_mut();
        let index = counter.addr() - 1;
        assert!(state.handles.contains(&counter.addr()));
        state.formats.push((index, format));
        let fixture = &state.counters[index];
        let mut value = PDH_FMT_COUNTERVALUE {
            CStatus: fixture.format_cstatus,
            ..Default::default()
        };
        match fixture.number {
            Number::Integer(number) => value.Anonymous.largeValue = number,
            Number::Double(number) => value.Anonymous.doubleValue = number,
        }
        (fixture.format_status, value)
    }

    fn close(&mut self, _: PDH_HQUERY) -> u32 {
        let mut state = self.0.borrow_mut();
        state.closes += 1;
        state.handles.clear();
        0
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        Ok(self.0.borrow().timestamp)
    }
}

/// Scenario: A configured counter fails to add while an earlier or later counter is usable.
/// Guarantees: Healthy counters retain their original indices, startup failures are reported, and skipped counters are not retried.
#[test]
fn opening_retains_healthy_counters_and_startup_diagnostics() {
    for failed in [0, 1] {
        let (api, state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(1)),
            CounterFixture::new(0, Number::Integer(2)),
        ]);
        state.borrow_mut().counters[failed].add_status = PDH_INVALID_DATA;
        let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
/// Guarantees: Opening returns an explicit error with all startup diagnostics and releases every acquired handle.
#[test]
fn opening_fails_without_a_query_or_usable_counters() {
    for fail_open in [true, false] {
        let (api, state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(1)),
            CounterFixture::new(0, Number::Integer(2)),
        ]);
        if fail_open {
            state.borrow_mut().open_status = PDH_INVALID_DATA;
        } else {
            state.borrow_mut().counters[0].add_status = PDH_INVALID_DATA;
            state.borrow_mut().counters[1].fill_status = PDH_INVALID_DATA;
        }
        let error = match Query::open_with(api, &configs, "test".to_owned()) {
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
            let Error::NoCounters { failures } = error else {
                panic!("expected startup diagnostics for every counter");
            };
            assert_eq!(failures.len(), 2);
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

/// Scenario: Counter-info sizing, buffer filling or native-type inspection fails on a later counter.
/// Guarantees: Unfilled zeroed metadata never becomes a valid counter; startup diagnostics preserve the failure and healthy peers collect.
#[test]
fn inspection_failure_cleans_up_without_classifying_zero_buffer() {
    for failure in 0..5 {
        let (api, state, configs) = fake(vec![
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
                4 => counter.native_type = 0x4003_0401,
                _ => unreachable!(),
            }
        }
        let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
    let (api, state, configs) = fake(vec![
        CounterFixture::new(0, Number::Integer(1)),
        CounterFixture::new(0x4003_0401, Number::Integer(2)),
    ]);
    state.borrow_mut().remove_status = PDH_INVALID_DATA;
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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

/// Scenario: Successfully filled metadata has native type zero, and direct counts use decimal scaling.
/// Guarantees: Valid zero types emit exact integers; configured scaling alone controls double output.
#[test]
fn direct_reads_preserve_integers_and_apply_only_configured_scaling() {
    let (api, state, mut configs) = fake(vec![
        CounterFixture::new(0, Number::Integer(i64::MAX)),
        CounterFixture::new(0x0001_0100, Number::Integer(7)),
    ]);
    configs[1].scale_power10 = -1;
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
            .all(|(_, flags)| *flags == PDH_FMT_LARGE | PDH_FMT_NOSCALE)
    );
    assert!(state.borrow().raws.is_empty());
    drop(query);
    let state = state.borrow();
    assert_eq!(
        (state.opens, state.adds, state.collects, state.closes),
        (1, 2, 3, 1)
    );
}

/// Scenario: A two-sample calculation warms up, then PDH returns a percentage above 100.
/// Guarantees: Warm-up is not a failure and the calculated value remains an unclamped scaled double.
#[test]
fn calculated_reads_warm_up_and_do_not_cap_percentages() {
    let (api, state, mut configs) = fake(vec![CounterFixture::new(
        0x2051_0500,
        Number::Double(250.0),
    )]);
    configs[0].scale_power10 = -2;
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
    assert_eq!(
        state.borrow().formats,
        vec![(0, PDH_FMT_DOUBLE | PDH_FMT_NOSCALE | PDH_FMT_NOCAP100)]
    );
}

/// Scenario: A raw fraction's current denominator is zero, negative, then positive.
/// Guarantees: Invalid denominators fail locally and a valid fraction needs no two-sample warm-up.
#[test]
fn raw_fraction_requires_a_positive_current_base() {
    let (api, state, configs) = fake(vec![CounterFixture::new(0x2002_0400, Number::Double(25.0))]);
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
    let (api, state, configs) = fake(vec![CounterFixture::new(0x4002_0500, Number::Double(4.0))]);
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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

/// Scenario: An average is marked ready but its previous receiver-side base is absent.
/// Guarantees: The read omits instead of panicking, seeds the current base, and resumes on advancement.
#[test]
fn missing_previous_base_omits_without_panicking() {
    let (api, state, configs) = fake(vec![CounterFixture::new(0x4002_0500, Number::Double(4.0))]);
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
    query.counters[0].ready = true;
    let sample = query.collect().unwrap();
    assert!(sample.failures.is_empty());
    assert_eq!(sample.points[0].value, SampleValue::NoObservation);
    state.borrow_mut().counters[0].base = 12;
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Value(Number::Double(4.0))
    );
}

/// Scenario: Formatted reads report missing or invalid data, or a negative calculation status.
/// Guarantees: Expected API and data statuses omit and reset readiness while preserving healthy peers and cumulative time.
#[test]
fn formatted_statuses_omit_and_reset_readiness() {
    for native_type in [0, 0x2002_0400, 0x1041_0400, 0x4002_0500] {
        let number = if native_type == 0 {
            Number::Integer(3)
        } else {
            Number::Double(3.0)
        };
        let needs_warmup = matches!(native_type, 0x1041_0400 | 0x4002_0500);
        for omission in OMISSION_STATUSES {
            for (status, cstatus) in [
                (omission, PDH_CSTATUS_VALID_DATA),
                (0, omission),
                (PDH_INVALID_DATA, omission),
            ] {
                let (api, state, configs) = fake(vec![
                    CounterFixture::new(native_type, number),
                    CounterFixture::new(0, Number::Integer(7)),
                ]);
                let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
/// Guarantees: An unrelated API error cannot be hidden by the returned data status.
#[test]
fn real_formatted_errors_remain_failures() {
    for (status, cstatus) in [
        (PDH_INVALID_ARGUMENT, PDH_CSTATUS_VALID_DATA),
        (PDH_INVALID_ARGUMENT, PDH_INVALID_DATA),
        (PDH_INVALID_ARGUMENT, PDH_NO_DATA),
        (PDH_INVALID_ARGUMENT, PDH_CALC_NEGATIVE_VALUE),
    ] {
        let (api, state, configs) = fake(vec![
            CounterFixture::new(0x1041_0400, Number::Double(3.0)),
            CounterFixture::new(0, Number::Integer(7)),
        ]);
        let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].counter_index, 1);
    }
}

/// Scenario: One direct read fails while an average peer advances its base, then both recover.
/// Guarantees: Failure never skips a healthy peer's base update, read retries reuse handles, and projection retains healthy points.
#[test]
fn counter_failures_preserve_peers_and_retry_existing_handles() {
    for fail_status in [false, true] {
        let (api, state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(7)),
            CounterFixture::new(0x3002_0400, Number::Double(2.0)),
        ]);
        let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
    for omission in OMISSION_STATUSES {
        for fail_status in [false, true] {
            let (api, state, configs) = fake(vec![
                CounterFixture::new(0x20C2_0400, Number::Double(50.0)),
                CounterFixture::new(0, Number::Integer(7)),
            ]);
            let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
                    state.counters[0].raw_cstatus = omission;
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

/// Scenario: A raw API or data status reports an unrelated error beside a healthy direct counter.
/// Guarantees: Real raw errors remain local failures and cannot be masked by an expected-omission data status.
#[test]
fn real_raw_errors_remain_failures() {
    for (status, cstatus) in [
        (PDH_INVALID_ARGUMENT, PDH_CSTATUS_VALID_DATA),
        (PDH_INVALID_ARGUMENT, PDH_NO_DATA),
        (0, PDH_INVALID_ARGUMENT),
    ] {
        let (api, state, configs) = fake(vec![
            CounterFixture::new(0x1041_0400, Number::Double(3.0)),
            CounterFixture::new(0, Number::Integer(7)),
        ]);
        let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
        state.borrow_mut().counters[0].raw_status = status;
        state.borrow_mut().counters[0].raw_cstatus = cstatus;
        let sample = query.collect().unwrap();
        assert_eq!(sample.failures.len(), 1);
        assert_eq!(sample.failures[0].counter_index, 0);
        assert!(sample.failures[0].error.contains(&configs[0].path));
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].counter_index, 1);
        assert_eq!(
            sample.points[0].value,
            SampleValue::Value(Number::Integer(7))
        );
    }
}

/// Scenario: A calculation returns a non-finite value or configured scaling overflows.
/// Guarantees: Only that counter fails; a healthy direct value still projects.
#[test]
fn scaling_failures_are_counter_local() {
    for number in [f64::NAN, f64::INFINITY, f64::MAX] {
        let (api, _, mut configs) = fake(vec![
            CounterFixture::new(0x2002_0400, Number::Double(number)),
            CounterFixture::new(0, Number::Integer(42)),
        ]);
        configs[0].scale_power10 = 18;
        let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
    let (api, state, configs) = fake(vec![
        CounterFixture::new(0, Number::Integer(7)),
        CounterFixture::new(0x1041_0400, Number::Double(3.0)),
    ]);
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
        let (api, state, configs) = fake(vec![
            CounterFixture::new(0, Number::Integer(7)),
            CounterFixture::new(0x4002_0500, Number::Double(3.0)),
        ]);
        let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
    let (api, state, configs) = fake(vec![CounterFixture::new(0, Number::Integer(7))]);
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
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
    let (api, state, configs) = fake(vec![CounterFixture::new(0, Number::Integer(7))]);
    let mut query = Query::open_with(api, &configs, "test".to_owned()).unwrap();
    for (timestamp, start) in [(110, 100), (110, 100), (90, 90), (120, 90)] {
        state.borrow_mut().timestamp = timestamp;
        let sample = query.collect().unwrap();
        assert_eq!(sample.timestamp_unix_nano, timestamp);
        assert_eq!(sample.start_time_unix_nano, start);
        assert!(into_otap(&configs, sample).unwrap().is_some());
    }
}

/// Scenario: Opening or a collection receives a non-positive observation timestamp.
/// Guarantees: Invalid timestamps never enter projection or corrupt the last valid sequence.
#[test]
fn rejects_non_positive_timestamps() {
    let (api, state, configs) = fake(vec![
        CounterFixture::new(0, Number::Integer(7)),
        CounterFixture::new(0x4002_0500, Number::Double(2.0)),
    ]);
    state.borrow_mut().timestamp = 0;
    assert!(Query::open_with(api, &configs, "test".to_owned()).is_err());
    assert_eq!(state.borrow().opens, 0);
    state.borrow_mut().timestamp = 100;
    let mut query =
        Query::open_with(FakePdh(Rc::clone(&state)), &configs, "test".to_owned()).unwrap();
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

/// Scenario: Windows formats rate, percentage, fraction and average fixtures from successive raw samples.
/// Guarantees: Native formulas remain finite, preserve CPU above 100 percent, and use no display scaling.
#[test]
fn native_formulas_from_raw_fixtures() {
    for (native_type, first, second, previous_first, previous_second, time_base, expected) in [
        (0x1041_0400, 160, 12, 100, 10, 10, 300.0),
        (0x2051_0500, 150, 200, 100, 180, 10_000_000, 250.0),
        (0x2002_0400, 25, 100, 0, 0, 1, 25.0),
        (0x20C2_0400, 160, 200, 100, 100, 1, 60.0),
        (0x3002_0400, 200, 5, 100, 3, 100, 0.5),
        (0x4002_0500, 200, 5, 100, 3, 1, 50.0),
    ] {
        let current = PDH_RAW_COUNTER {
            CStatus: PDH_CSTATUS_VALID_DATA,
            FirstValue: first,
            SecondValue: second,
            ..Default::default()
        };
        let previous = PDH_RAW_COUNTER {
            CStatus: PDH_CSTATUS_VALID_DATA,
            FirstValue: previous_first,
            SecondValue: previous_second,
            ..Default::default()
        };
        let mut value = PDH_FMT_COUNTERVALUE::default();
        // SAFETY: All pointers reference initialized fixture values and writable output.
        let status = unsafe {
            PdhFormatFromRawValue(
                native_type,
                PDH_FMT_DOUBLE | PDH_FMT_NOSCALE | PDH_FMT_NOCAP100,
                &time_base,
                &current,
                &previous,
                &mut value,
            )
        };
        check("PdhFormatFromRawValue", "fixture", status).unwrap();
        check_data("fixture CStatus", "fixture", value.CStatus).unwrap();
        // SAFETY: The successful PDH_FMT_DOUBLE call initialized doubleValue.
        assert!((unsafe { value.Anonymous.doubleValue } - expected).abs() < 1e-9);
    }
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
