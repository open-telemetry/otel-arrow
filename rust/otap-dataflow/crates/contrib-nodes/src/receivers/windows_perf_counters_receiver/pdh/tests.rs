// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::config::{Config, CounterConfig};
use super::query::Collector;
use super::*;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use windows_sys::Win32::System::Performance::{
    PDH_CALC_NEGATIVE_DENOMINATOR, PDH_CSTATUS_INVALID_DATA, PDH_CSTATUS_NEW_DATA,
    PDH_CSTATUS_NO_INSTANCE, PDH_CSTATUS_VALID_DATA, PDH_INVALID_DATA, PDH_NO_DATA,
};

const MISSING: u32 = 0xC000_0BB8;
const OPEN_FAILED: u32 = 0xC000_0BC0;
const COLLECT_FAILED: u32 = 0xC000_0BBC;
const READ_FAILED: u32 = 0xC000_0BBD;

/// Scripted PDH state shared by every query the test opens.
#[derive(Default)]
struct State {
    opens: usize,
    closes: usize,
    /// Notified on each close.
    closed: Option<mpsc::Sender<()>>,
    fail_open: bool,
    adds: Vec<String>,
    missing: HashSet<String>,
    collect_status: Option<u32>,
    /// One-shot collect statuses, used before `collect_status`.
    collect_script: VecDeque<u32>,
    collects: usize,
    /// Set as each collect returns, as if shutdown were requested during it.
    stop_on_collect: Option<Arc<AtomicBool>>,
    /// Taken by the next collect, which reports entry on the sender and then blocks until the
    /// receiver is released.
    gate: Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>,
    arrays: HashMap<String, Result<Vec<(String, Value)>, u32>>,
}

/// One fake query, numbered by the order it was opened.
struct FakePdh {
    state: Arc<Mutex<State>>,
    id: usize,
}

impl Drop for FakePdh {
    fn drop(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.closes += 1;
        if let Some(closed) = &state.closed {
            let _ = closed.send(());
        }
    }
}

impl Pdh for FakePdh {
    type Counter = (usize, String);

    fn add(&mut self, path: &str) -> Result<Self::Counter, u32> {
        let mut state = self.state.lock().unwrap();
        state.adds.push(path.to_owned());
        if state.missing.contains(path) {
            Err(MISSING)
        } else {
            Ok((self.id, path.to_owned()))
        }
    }

    fn collect(&mut self) -> Result<(), u32> {
        let gate = self.state.lock().unwrap().gate.take();
        if let Some((entered, release)) = gate {
            let _ = entered.send(());
            let _ = release.recv();
        }
        let mut state = self.state.lock().unwrap();
        state.collects += 1;
        if let Some(stop) = &state.stop_on_collect {
            stop.store(true, Ordering::Relaxed);
        }
        match state.collect_script.pop_front() {
            Some(status) => Err(status),
            None => state.collect_status.map_or(Ok(()), Err),
        }
    }

    fn array(
        &mut self,
        counter: &Self::Counter,
        items: &mut Vec<(String, Value)>,
    ) -> Result<(), u32> {
        assert_eq!(counter.0, self.id, "handle used with another query");
        items.clear();
        items.extend(self.state.lock().unwrap().arrays[&counter.1].clone()?);
        Ok(())
    }
}

fn counters(perfcounters: serde_json::Value) -> Vec<CounterConfig> {
    Config::from_json(&json!({ "perfcounters": perfcounters }))
        .unwrap()
        .counters
}

fn fake() -> Arc<Mutex<State>> {
    Arc::new(Mutex::new(State::default()))
}

fn opener(state: &Arc<Mutex<State>>) -> impl FnMut() -> Result<FakePdh, u32> + Send + 'static {
    let state = Arc::clone(state);
    move || {
        let mut locked = state.lock().unwrap();
        if locked.fail_open {
            return Err(OPEN_FAILED);
        }
        locked.opens += 1;
        Ok(FakePdh {
            state: Arc::clone(&state),
            id: locked.opens,
        })
    }
}

/// Starts a collector on the fake, with no shutdown requested.
fn collector(
    config: Vec<CounterConfig>,
    state: &Arc<Mutex<State>>,
) -> (
    Collector<FakePdh, impl FnMut() -> Result<FakePdh, u32>>,
    Report,
) {
    Collector::start(config, opener(state), Arc::default())
}

/// Blocks the next collect. Returns a receiver that reports when the collect is entered and a
/// sender that releases it.
fn gate(state: &Arc<Mutex<State>>) -> (mpsc::Receiver<()>, mpsc::Sender<()>) {
    let (entered_tx, entered) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    state.lock().unwrap().gate = Some((entered_tx, release_rx));
    (entered, release)
}

fn wait_until_entered(entered: &mpsc::Receiver<()>) {
    entered
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("the worker reaches the gate");
}

fn value(status: u32, value: f64) -> Value {
    Value { status, value }
}

fn item(name: &str, status: u32, number: f64) -> (String, Value) {
    (name.to_owned(), value(status, number))
}

/// A non-instanced counter: one valid item with an empty name.
fn set_single(state: &Arc<Mutex<State>>, path: &str, number: f64) {
    set_array(
        state,
        path,
        Ok(vec![item("", PDH_CSTATUS_VALID_DATA, number)]),
    );
}

fn set_array(state: &Arc<Mutex<State>>, path: &str, result: Result<Vec<(String, Value)>, u32>) {
    let _ = state.lock().unwrap().arrays.insert(path.to_owned(), result);
}

fn points(report: &Report) -> Vec<(usize, Option<&str>, f64)> {
    report
        .points
        .iter()
        .map(|point| (point.counter, point.instance.as_deref(), point.value))
        .collect()
}

fn names(report: &Report) -> Vec<Option<&str>> {
    report
        .points
        .iter()
        .map(|point| point.instance.as_deref())
        .collect()
}

const MEMORY: &str = r"\Memory\Available Bytes";
const SYSTEM: &str = r"\System\Threads";
const PROCESSOR: &str = r"\Processor(*)\% Processor Time";

/// Scenario: One path is missing at start, still missing on the next scrape, then becomes available.
/// Guarantees: Start primes without publishing; the failed add is retried at the start of every
/// scrape with no backoff, is reported as added once it succeeds, and then publishes.
#[test]
fn retries_failed_adds_each_scrape() {
    let state = fake();
    let _ = state.lock().unwrap().missing.insert(SYSTEM.to_owned());
    set_single(&state, MEMORY, 1.0);
    set_single(&state, SYSTEM, 2.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
        {"object": "System", "counters": [{"name": "Threads"}]}
    ]));
    let (mut collector, start) = collector(config, &state);
    assert!(start.points.is_empty());
    assert_eq!(start.add_failures, [(1, MISSING)]);
    assert_eq!(start.added, [0]);
    assert_eq!(start.active_counters, 1);

    let still_missing = collector.scrape();
    assert_eq!(still_missing.add_failures, [(1, MISSING)]);
    assert!(still_missing.added.is_empty());
    assert_eq!(points(&still_missing), [(0, None, 1.0)]);

    let _ = state.lock().unwrap().missing.remove(SYSTEM);
    let recovered = collector.scrape();
    assert!(recovered.add_failures.is_empty());
    assert_eq!(recovered.added, [1]);
    assert_eq!(recovered.active_counters, 2);
    assert_eq!(points(&recovered), [(0, None, 1.0), (1, None, 2.0)]);
    let state = state.lock().unwrap();
    assert_eq!(state.adds, [MEMORY, SYSTEM, SYSTEM, SYSTEM]);
    assert_eq!(state.opens, 2);
}

/// Scenario: Non-instanced, exact and wildcard arrays return valid, new, invalid and no-instance
/// item CStatus values.
/// Guarantees: Only PDH_CSTATUS_VALID_DATA and PDH_CSTATUS_NEW_DATA items are published; others are
/// skipped silently; scale_power10 multiplies or divides; an empty name has no instance.
#[test]
fn publishes_only_valid_or_new_cstatus() {
    let state = fake();
    let exact = r"\Process(app)\ID Process";
    set_array(
        &state,
        MEMORY,
        Ok(vec![item("", PDH_CSTATUS_NEW_DATA, 2_500.0)]),
    );
    set_array(
        &state,
        exact,
        Ok(vec![item("app", PDH_CSTATUS_INVALID_DATA, 7.0)]),
    );
    set_array(
        &state,
        PROCESSOR,
        Ok(vec![
            item("0", PDH_CSTATUS_VALID_DATA, 0.5),
            item("1", PDH_CSTATUS_INVALID_DATA, 9.0),
            item("2", PDH_CSTATUS_NO_INSTANCE, 9.0),
            item("3", PDH_CSTATUS_NEW_DATA, 0.25),
        ]),
    );
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes", "scale_power10": -3}]},
        {"object": "Processor", "instances": "*",
         "counters": [{"name": "% Processor Time", "scale_power10": 2}]},
        {"object": "Process", "instances": "app", "counters": [{"name": "ID Process"}]}
    ]));
    let (mut collector, _) = collector(config, &state);
    let report = collector.scrape();
    assert_eq!(
        points(&report),
        [(0, None, 2.5), (1, Some("0"), 50.0), (1, Some("3"), 25.0)]
    );
    assert_eq!(report.collected, [0, 1, 2]);
}

/// Scenario: Array calls return PDH_CSTATUS_INVALID_DATA (a rate counter's first sample),
/// PDH_INVALID_DATA, PDH_NO_DATA and PDH_CALC_NEGATIVE_DENOMINATOR.
/// Guarantees: Each is no observation: the counter is skipped silently and reported as read (ending
/// any read-failure episode), other counters still publish, and nothing counts as a failure.
#[test]
fn ignorable_read_statuses_are_no_observation() {
    let state = fake();
    let statuses = [
        PDH_CSTATUS_INVALID_DATA,
        PDH_INVALID_DATA,
        PDH_NO_DATA,
        PDH_CALC_NEGATIVE_DENOMINATOR,
    ];
    let mut perfcounters = Vec::new();
    for (index, status) in statuses.into_iter().enumerate() {
        set_array(&state, &format!(r"\Object{index}\Counter"), Err(status));
        perfcounters
            .push(json!({"object": format!("Object{index}"), "counters": [{"name": "Counter"}]}));
    }
    set_single(&state, MEMORY, 1.0);
    perfcounters.push(json!({"object": "Memory", "counters": [{"name": "Available Bytes"}]}));
    let (mut collector, _) = collector(counters(json!(perfcounters)), &state);
    let report = collector.scrape();
    assert_eq!(points(&report), [(4, None, 1.0)]);
    assert_eq!(report.read, [0, 1, 2, 3, 4]);
    assert!(report.read_failures.is_empty());
    assert!(report.collect_failures.is_empty() && !report.collected.is_empty());
}

/// Scenario: One array call returns a non-ignorable error beside a healthy counter, then recovers.
/// Guarantees: Only that counter is skipped and reported as a read failure with its status; the
/// scrape is not a failure, no query is reopened, and a later successful read reports it as read.
#[test]
fn other_read_errors_skip_only_that_counter() {
    let state = fake();
    set_array(&state, SYSTEM, Err(READ_FAILED));
    set_single(&state, MEMORY, 1.0);
    let config = counters(json!([
        {"object": "System", "counters": [{"name": "Threads"}]},
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let (mut collector, _) = collector(config, &state);
    let report = collector.scrape();
    assert_eq!(points(&report), [(1, None, 1.0)]);
    assert_eq!(report.read_failures, [(0, READ_FAILED)]);
    assert_eq!(report.read, [1]);
    assert!(report.collect_failures.is_empty() && !report.collected.is_empty());

    set_single(&state, SYSTEM, 2.0);
    let report = collector.scrape();
    assert_eq!(points(&report), [(0, None, 2.0), (1, None, 1.0)]);
    assert!(report.read_failures.is_empty());
    assert_eq!(state.lock().unwrap().opens, 2);
}

/// Scenario: A scrape's collect returns PDH_NO_DATA.
/// Guarantees: Nothing is published and nothing is a failure, so the query is not reopened.
#[test]
fn collect_no_data_is_silent() {
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let (mut collector, _) = collector(config, &state);
    state.lock().unwrap().collect_status = Some(PDH_NO_DATA);
    let report = collector.scrape();
    assert!(report.points.is_empty());
    assert!(report.collect_failures.is_empty() && !report.collected.is_empty());
    state.lock().unwrap().collect_status = None;
    assert_eq!(points(&collector.scrape()), [(0, None, 1.0)]);
    assert_eq!(state.lock().unwrap().opens, 1);
}

/// Scenario: A collect returns PDH_CALC_NEGATIVE_DENOMINATOR once, later twice in a row, and later
/// once followed by PDH_NO_DATA.
/// Guarantees: The receiver collects exactly once more; one negative denominator still
/// publishes, while any error from the retry, including no data, is a collect failure that drops
/// the query and reopens it on the next scrape.
#[test]
fn negative_denominator_collects_once_more() {
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let (mut collector, _) = collector(config, &state);
    let before = state.lock().unwrap().collects;
    state
        .lock()
        .unwrap()
        .collect_script
        .push_back(PDH_CALC_NEGATIVE_DENOMINATOR);
    let retried = collector.scrape();
    assert_eq!(points(&retried), [(0, None, 1.0)]);
    assert!(retried.collect_failures.is_empty());
    assert_eq!(state.lock().unwrap().collects, before + 2);

    state.lock().unwrap().collect_script =
        VecDeque::from([PDH_CALC_NEGATIVE_DENOMINATOR, PDH_CALC_NEGATIVE_DENOMINATOR]);
    let failed = collector.scrape();
    assert_eq!(
        failed.collect_failures,
        [(0, PDH_CALC_NEGATIVE_DENOMINATOR)]
    );
    assert!(failed.points.is_empty());
    assert_eq!(state.lock().unwrap().collects, before + 4);

    // The query is reopened and primed, then the retry returns no data.
    let _ = collector.scrape();
    state.lock().unwrap().collect_script =
        VecDeque::from([PDH_CALC_NEGATIVE_DENOMINATOR, PDH_NO_DATA]);
    let failed = collector.scrape();
    assert_eq!(failed.collect_failures, [(0, PDH_NO_DATA)]);
    assert!(failed.points.is_empty() && failed.collected.is_empty());
    assert_eq!(failed.active_counters, 0);
    let _ = collector.scrape();
    assert_eq!(state.lock().unwrap().opens, 3);
}

/// Scenario: Processor counters with `*`, `*` beside a listed `_Total`, a custom aggregation_name,
/// a pattern beside an exact `_Total`, or an opted-in aggregate read arrays with repeated names,
/// invalid CStatus items, a returned literal `a#1`, repeated `_Total`s, and a lone `_Total`.
/// Guarantees: Invalid items are skipped; repeats of a name get the lowest free "#1", "#2", ... in
/// returned order, skipping returned names. The aggregate is the last raw occurrence of its name,
/// even when that item is invalid: it is dropped, or unnamed when it is the only kept item, unless it
/// is listed; every earlier occurrence is a real instance named normally.
#[test]
fn names_instances_of_an_array_read() {
    const OK: u32 = PDH_CSTATUS_VALID_DATA;
    const BAD: u32 = PDH_CSTATUS_INVALID_DATA;
    let state = fake();
    let pattern = r"\Processor(_T*)\% Processor Time";
    let exact = r"\Processor(_Total)\% Processor Time";
    for (instances, aggregation_name, array, expected) in [
        (
            json!("*"),
            "",
            vec![
                ("a", BAD, 0.0),
                ("a", OK, 1.0),
                ("b", OK, 2.0),
                ("a", OK, 3.0),
                ("a", OK, 4.0),
            ],
            vec![
                (0, Some("a"), 1.0),
                (0, Some("b"), 2.0),
                (0, Some("a#1"), 3.0),
                (0, Some("a#2"), 4.0),
            ],
        ),
        (
            json!("*"),
            "",
            vec![("a", OK, 1.0), ("a", OK, 2.0), ("a#1", OK, 3.0)],
            vec![
                (0, Some("a"), 1.0),
                (0, Some("a#2"), 2.0),
                (0, Some("a#1"), 3.0),
            ],
        ),
        (
            json!("*"),
            "",
            vec![("_Total", BAD, 1.0), ("_Total", OK, 2.0)],
            vec![(0, None, 2.0)],
        ),
        (
            json!("*"),
            "",
            vec![("_Total", OK, 1.0), ("_Total", OK, 2.0)],
            vec![(0, Some("_Total"), 1.0)],
        ),
        (
            json!("*"),
            "",
            vec![("_Total", OK, 1.0), ("_Total", BAD, 2.0)],
            vec![(0, Some("_Total"), 1.0)],
        ),
        (
            json!("*"),
            "",
            vec![("0", OK, 1.0), ("_Total", OK, 2.0), ("1", OK, 3.0)],
            vec![(0, Some("0"), 1.0), (0, Some("1"), 3.0)],
        ),
        (
            json!(["*", "_Total"]),
            "",
            vec![("0", OK, 1.0), ("_Total", OK, 2.0), ("1", OK, 3.0)],
            vec![
                (0, Some("0"), 1.0),
                (0, Some("_Total"), 2.0),
                (0, Some("1"), 3.0),
            ],
        ),
        (
            json!("*"),
            "1",
            vec![("0", OK, 1.0), ("_Total", OK, 2.0), ("1", OK, 3.0)],
            vec![(0, Some("0"), 1.0), (0, Some("_Total"), 2.0)],
        ),
        (
            json!(["_T*", "_Total"]),
            "",
            vec![("_Total", OK, 5.0)],
            vec![(0, None, 5.0), (1, None, 5.0)],
        ),
        (
            json!(["*", "_Total"]),
            "_Total",
            vec![("_Total", OK, 5.0)],
            vec![(0, Some("_Total"), 5.0)],
        ),
    ] {
        let array: Vec<_> = array
            .into_iter()
            .map(|(name, status, number)| item(name, status, number))
            .collect();
        for path in [PROCESSOR, pattern, exact] {
            set_array(&state, path, Ok(array.clone()));
        }
        let config = counters(json!([{
            "object": "Processor",
            "instances": instances,
            "aggregation_name": aggregation_name,
            "counters": [{"name": "% Processor Time"}]
        }]));
        let (mut collector, _) = collector(config, &state);
        assert_eq!(
            points(&collector.scrape()),
            expected,
            "{instances} {array:?}"
        );
    }
}

/// Scenario: The receiver's start time is after the read time, as after a backward clock step,
/// and the next read is later.
/// Guarantees: The start time moves back to the read time, so a point's start time never follows
/// its time; a later read keeps the moved start time unless its own time is earlier still.
#[test]
fn start_time_moves_back_with_the_clock() {
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let (mut collector, _) = collector(config, &state);
    collector.start_time_unix_nano = i64::MAX;
    let moved = collector.scrape().points.remove(0);
    assert_eq!(moved.start_time_unix_nano, moved.time_unix_nano);
    let later = collector.scrape().points.remove(0);
    assert_eq!(
        later.start_time_unix_nano,
        moved.time_unix_nano.min(later.time_unix_nano)
    );
}

/// Scenario: At startup, a counter's query fails to open; in a second start, it fails to prime.
/// Guarantees: Both are reported in the start report as collect failures for that counter, the
/// failed query is closed, and nothing is published.
#[test]
fn start_open_or_prime_failure_is_a_collect_failure() {
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let state = fake();
    state.lock().unwrap().fail_open = true;
    let (_, start) = collector(config.clone(), &state);
    assert_eq!(start.collect_failures, [(0, OPEN_FAILED)]);
    assert_eq!(start.active_counters, 0);

    let state = fake();
    state
        .lock()
        .unwrap()
        .collect_script
        .push_back(COLLECT_FAILED);
    let (_, start) = collector(config, &state);
    assert_eq!(start.collect_failures, [(0, COLLECT_FAILED)]);
    assert!(start.points.is_empty() && start.collected.is_empty());
    assert_eq!(start.active_counters, 0);
    assert_eq!(state.lock().unwrap().closes, 1);
}

/// Scenario: Two counters; the first counter's collection fails in one scrape while the second's
/// succeeds.
/// Guarantees: Each counter has its own query, so the healthy counter still collects and publishes
/// in the failing scrape and its query is never reopened; the failure is reported for the failed
/// counter only; only that counter's query is closed, then reopened, re-added and primed without
/// publishing on the next scrape, and priming is not reported as a collection; reopening keeps the
/// start time, since it does not reset the OS counter.
#[test]
fn collect_failure_stops_only_its_counter() {
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    set_single(&state, SYSTEM, 2.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
        {"object": "System", "counters": [{"name": "Threads"}]}
    ]));
    let (mut collector, start) = collector(config, &state);
    assert_eq!(start.active_counters, 2);
    assert_eq!(state.lock().unwrap().opens, 2);
    collector.start_time_unix_nano = 1;

    // Queries collect in configuration order, so this fails only the first counter's query.
    state
        .lock()
        .unwrap()
        .collect_script
        .push_back(COLLECT_FAILED);
    let failed = collector.scrape();
    assert_eq!(failed.collect_failures, [(0, COLLECT_FAILED)]);
    assert_eq!(failed.collected, [1]);
    assert_eq!(points(&failed), [(1, None, 2.0)]);
    assert_eq!(failed.active_counters, 1);
    assert_eq!(state.lock().unwrap().closes, 1);

    let reopened = collector.scrape();
    assert!(reopened.collect_failures.is_empty());
    assert_eq!(reopened.collected, [1]);
    assert_eq!(reopened.added, [0]);
    assert_eq!(points(&reopened), [(1, None, 2.0)]);
    {
        let state = state.lock().unwrap();
        assert_eq!((state.opens, state.closes), (3, 1));
        assert_eq!(state.adds, [MEMORY, SYSTEM, MEMORY]);
    }
    let after = collector.scrape();
    assert_eq!(points(&after), [(0, None, 1.0), (1, None, 2.0)]);
    assert!(
        after
            .points
            .iter()
            .all(|point| point.start_time_unix_nano == 1)
    );
}

/// Scenario: One counter uses recreate_query beside a counter without it; replacement later fails
/// to open, then fails to add the path.
/// Guarantees: The recreate counter's query is replaced after every scrape (open, add, prime,
/// swap, close old) without changing its start time, while the other counter's query is
/// untouched; a failed replacement is reported, a partly built one is closed, and the old query
/// keeps publishing.
#[test]
fn recreates_query_after_each_scrape() {
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    set_single(&state, SYSTEM, 2.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
        {"object": "System", "counters": [{"name": "Threads", "recreate_query": true}]}
    ]));
    let (mut collector, start) = collector(config, &state);
    assert_eq!(start.active_counters, 2);
    assert_eq!(state.lock().unwrap().opens, 2);
    collector.start_time_unix_nano = 1;

    let first = collector.scrape();
    assert_eq!(points(&first), [(0, None, 1.0), (1, None, 2.0)]);
    assert_eq!(first.recreated, [1]);
    {
        let state = state.lock().unwrap();
        assert_eq!((state.opens, state.closes), (3, 1));
        assert_eq!(state.adds, [MEMORY, SYSTEM, SYSTEM]);
    }
    let second = collector.scrape();
    assert_eq!(points(&second), [(0, None, 1.0), (1, None, 2.0)]);
    assert!(
        second
            .points
            .iter()
            .all(|point| point.start_time_unix_nano == 1)
    );
    assert_eq!(state.lock().unwrap().closes, 2);

    state.lock().unwrap().fail_open = true;
    let failed = collector.scrape();
    assert_eq!(failed.recreate_failures, [(1, OPEN_FAILED)]);
    assert!(failed.collect_failures.is_empty() && failed.collected == [0, 1]);
    assert_eq!(points(&failed), [(0, None, 1.0), (1, None, 2.0)]);
    let kept = collector.scrape();
    assert_eq!(points(&kept), [(0, None, 1.0), (1, None, 2.0)]);
    assert_eq!(kept.active_counters, 2);
    assert_eq!(state.lock().unwrap().closes, 2);

    {
        let mut state = state.lock().unwrap();
        state.fail_open = false;
        let _ = state.missing.insert(SYSTEM.to_owned());
    }
    let add_failed = collector.scrape();
    assert_eq!(add_failed.recreate_failures, [(1, MISSING)]);
    assert_eq!(points(&add_failed), [(0, None, 1.0), (1, None, 2.0)]);
    assert_eq!(add_failed.active_counters, 2);
    assert_eq!(state.lock().unwrap().closes, 3, "replacement closed");
    assert_eq!(
        points(&collector.scrape()),
        [(0, None, 1.0), (1, None, 2.0)]
    );
}

/// Scenario: The worker is stopped while idle, while a collection is blocked, and after it panicked
/// during a scrape.
/// Guarantees: An idle stop closes the query and reports a clean stop; a blocked stop reports a
/// timeout at the deadline instead of waiting; a panicked worker is reported as failed, not clean.
#[tokio::test]
async fn shutdown_closes_query_or_stops_waiting_at_deadline() {
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let deadline = || tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    let state = fake();
    let (worker, started) = Worker::start(config.clone(), opener(&state)).unwrap();
    let _ = started.await.unwrap();
    assert_eq!(worker.shutdown(deadline()).await, Stopped::Cleanly);
    assert_eq!(state.lock().unwrap().closes, 1);

    let state = fake();
    set_single(&state, MEMORY, 1.0);
    let (worker, started) = Worker::start(config.clone(), opener(&state)).unwrap();
    let _ = started.await.unwrap();
    let (entered, release) = gate(&state);
    let (closed_tx, closed) = mpsc::channel();
    state.lock().unwrap().closed = Some(closed_tx);
    let _pending = worker.scrape();
    wait_until_entered(&entered);
    let soon = tokio::time::Instant::now() + std::time::Duration::from_millis(50);
    assert_eq!(worker.shutdown(soon).await, Stopped::TimedOut);
    assert_eq!(state.lock().unwrap().closes, 0);
    release.send(()).unwrap();
    // The released worker finishes its scrape, then stops and closes the query.
    closed
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("the released worker closes its query");
    assert_eq!(state.lock().unwrap().closes, 1);

    // No array is scripted for the path, so the fake panics on the first read.
    let state = fake();
    let (worker, started) = Worker::start(config, opener(&state)).unwrap();
    let _ = started.await.unwrap();
    assert!(worker.scrape().await.is_err());
    assert_eq!(worker.shutdown(deadline()).await, Stopped::Failed);
}

/// Scenario: The worker closes its query on shutdown, but a thread-local destructor on the worker
/// thread then blocks until released.
/// Guarantees: Shutdown waits for the thread to exit, not only for the query to close, and still
/// returns at the deadline without blocking the async runtime; the thread exits once released.
#[tokio::test]
async fn shutdown_deadline_covers_thread_exit() {
    struct Hold {
        release: mpsc::Receiver<()>,
        exited: mpsc::Sender<()>,
    }
    impl Drop for Hold {
        fn drop(&mut self) {
            let _ = self.release.recv();
            let _ = self.exited.send(());
        }
    }
    thread_local! {
        static HOLD: std::cell::RefCell<Option<Hold>> = const { std::cell::RefCell::new(None) };
    }
    let (release, release_rx) = mpsc::channel();
    let (exited_tx, exited) = mpsc::channel();
    let (closed_tx, closed) = mpsc::channel();
    let state = fake();
    state.lock().unwrap().closed = Some(closed_tx);
    let mut hold = Some(Hold {
        release: release_rx,
        exited: exited_tx,
    });
    let mut open = opener(&state);
    let open = move || {
        if let Some(hold) = hold.take() {
            HOLD.with(|slot| *slot.borrow_mut() = Some(hold));
        }
        open()
    };
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let (worker, started) = Worker::start(config, open).unwrap();
    let _ = started.await.unwrap();
    // Releases the held destructor if shutdown blocks the async runtime on the join, so the test
    // fails instead of hanging.
    let watchdog = release.clone();
    let _ = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(10));
        let _ = watchdog.send(());
    });
    let soon = tokio::time::Instant::now() + std::time::Duration::from_millis(50);
    assert_eq!(worker.shutdown(soon).await, Stopped::TimedOut);
    closed
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("the worker closes its query before thread exit");
    release.send(()).unwrap();
    exited
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("the released worker thread exits");
}

/// Scenario: Shutdown is requested while the first of two counters is blocked in its priming
/// collect at start.
/// Guarantees: Shutdown sets the stop flag the collector checks: the blocked collect finishes, but
/// the second counter is never opened, added or collected.
#[tokio::test]
async fn shutdown_stops_start_before_the_next_counter() {
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
        {"object": "System", "counters": [{"name": "Threads"}]}
    ]));
    let state = fake();
    let (entered, release) = gate(&state);
    let (worker, _started) = Worker::start(config, opener(&state)).unwrap();
    wait_until_entered(&entered);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    let stopping = worker.shutdown(deadline);
    futures::pin_mut!(stopping);
    assert!(futures::poll!(&mut stopping).is_pending());
    release.send(()).unwrap();
    assert_eq!(stopping.await, Stopped::Cleanly);
    let locked = state.lock().unwrap();
    assert_eq!((locked.opens, locked.collects), (1, 1));
    assert_eq!(locked.adds, [MEMORY]);
}

/// Scenario: Shutdown is requested while a scrape collects the first of two counters, which
/// recreates its query after each scrape; the collect either succeeds or reports a negative
/// denominator.
/// Guarantees: A successful collect is still read and published, but the query is not recreated;
/// a negative denominator is not collected again; and the second counter is not collected.
#[test]
fn stop_during_a_scrape_skips_remaining_counter_work() {
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes", "recreate_query": true}]},
        {"object": "System", "counters": [{"name": "Threads"}]}
    ]));
    for (status, published) in [
        (None, vec![(0, None, 1.0)]),
        (Some(PDH_CALC_NEGATIVE_DENOMINATOR), vec![]),
    ] {
        let state = fake();
        set_single(&state, MEMORY, 1.0);
        set_single(&state, SYSTEM, 2.0);
        let stop = Arc::new(AtomicBool::new(false));
        let (mut collector, _) =
            Collector::start(config.clone(), opener(&state), Arc::clone(&stop));
        {
            let mut locked = state.lock().unwrap();
            locked.stop_on_collect = Some(stop);
            locked.collect_script.extend(status);
        }
        let report = collector.scrape();
        assert_eq!(points(&report), published, "{status:?}");
        assert!(report.recreated.is_empty(), "{status:?}");
        let locked = state.lock().unwrap();
        // Start opened and primed both counters; the scrape collected only the first, once.
        assert_eq!((locked.opens, locked.collects), (2, 3), "{status:?}");
    }
}

/// Scenario: Array sizes at, under and over the 64 MiB cap, as each PDH sizing call reports them,
/// are converted to a buffer length.
/// Guarantees: Each over-cap size fails before any allocation; sizes up to the cap round up to
/// whole `u64` words.
#[test]
fn array_size_is_capped_before_each_allocation() {
    use windows_sys::Win32::System::Performance::PDH_INSUFFICIENT_BUFFER;
    let cap = 64 << 20;
    assert_eq!(native::array_words(cap + 1), Err(PDH_INSUFFICIENT_BUFFER));
    assert_eq!(native::array_words(9), Ok(2));
    assert_eq!(native::array_words(cap), Ok(cap as usize / 8));
    assert_eq!(native::array_words(u32::MAX), Err(PDH_INSUFFICIENT_BUFFER));
}

/// `count` valid instances named "0", "1", ...
fn instances(count: usize) -> Result<Vec<(String, Value)>, u32> {
    Ok((0..count)
        .map(|index| item(&index.to_string(), PDH_CSTATUS_VALID_DATA, 1.0))
        .collect())
}

/// Scenario: A wildcard counter returns 100,001 instances, over the scrape's 100,000-instance
/// budget, between two non-instanced counters.
/// Guarantees: The wildcard counter is skipped whole and reported as a read failure with
/// PDH_INSUFFICIENT_BUFFER, the read failure the runtime warns about; the counters before and
/// after it still publish.
#[test]
fn oversized_array_is_skipped_whole() {
    use windows_sys::Win32::System::Performance::PDH_INSUFFICIENT_BUFFER;
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    set_array(&state, PROCESSOR, instances(100_001));
    set_single(&state, SYSTEM, 2.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
        {"object": "Processor", "instances": "*", "counters": [{"name": "% Processor Time"}]},
        {"object": "System", "counters": [{"name": "Threads"}]}
    ]));
    let (mut collector, _) = collector(config, &state);
    let report = collector.scrape();
    assert_eq!(points(&report), [(0, None, 1.0), (2, None, 2.0)]);
    assert_eq!(report.read_failures, [(1, PDH_INSUFFICIENT_BUFFER)]);
    assert_eq!(report.read, [0, 2]);
}

/// Scenario: A wildcard counter returns 99,999 instances before a non-instanced counter, so the
/// scrape keeps exactly 100,000; then it returns 100,000, putting the second counter one over.
/// Guarantees: At the budget every counter publishes, in each of two scrapes, since the budget
/// resets per scrape; one over, the later counter in configuration order is skipped whole as a
/// read failure, and the earlier one keeps all its instances.
#[test]
fn instance_budget_admits_exactly_the_limit() {
    use windows_sys::Win32::System::Performance::PDH_INSUFFICIENT_BUFFER;
    let state = fake();
    set_array(&state, PROCESSOR, instances(99_999));
    set_single(&state, MEMORY, 1.0);
    let config = counters(json!([
        {"object": "Processor", "instances": "*", "counters": [{"name": "% Processor Time"}]},
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let (mut collector, _) = collector(config, &state);
    for _ in 0..2 {
        let report = collector.scrape();
        assert_eq!(report.points.len(), 100_000);
        assert!(report.read_failures.is_empty());
    }
    set_array(&state, PROCESSOR, instances(100_000));
    let report = collector.scrape();
    assert_eq!(report.points.len(), 100_000);
    assert!(report.points.iter().all(|point| point.counter == 0));
    assert_eq!(report.read_failures, [(1, PDH_INSUFFICIENT_BUFFER)]);
}

/// Scenario: A counter path contains an interior NUL.
/// Guarantees: The native add fails with a bad-counter-name status instead of adding a truncated
/// prefix of the path.
#[test]
fn interior_nul_fails_to_add() {
    use windows_sys::Win32::System::Performance::PDH_CSTATUS_BAD_COUNTERNAME;
    assert_eq!(
        native::wide_path("\\Memory\\Available Bytes\0suffix"),
        Err(PDH_CSTATUS_BAD_COUNTERNAME)
    );
    assert_eq!(native::wide_path("ab"), Ok(vec![97, 98, 0]));
}

mod live {
    use super::super::super::test_utils::spawn_children;
    use super::super::NativePdh;
    use super::*;
    use std::time::Duration;

    type Live = Collector<NativePdh, fn() -> Result<NativePdh, u32>>;

    fn start(perfcounters: serde_json::Value) -> Live {
        let open: fn() -> Result<NativePdh, u32> = NativePdh::open;
        let (collector, report) = Collector::start(counters(perfcounters), open, Arc::default());
        assert!(report.collect_failures.is_empty(), "{report:?}");
        assert!(report.add_failures.is_empty(), "{report:?}");
        std::thread::sleep(Duration::from_millis(1100));
        collector
    }

    /// Scenario: Non-instanced counters are read through the array call on this machine.
    /// Guarantees: Each returns one item with an empty name, published once without an instance.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_non_instanced_counter() {
        let mut collector = start(json!([
            {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
            {"object": "System", "counters": [{"name": "Threads"}]}
        ]));
        let report = collector.scrape();
        let points = points(&report);
        assert_eq!(points.len(), 2, "{points:?}");
        assert!(
            points
                .iter()
                .all(|point| point.1.is_none() && point.2 > 0.0)
        );
    }

    /// Scenario: Exact named-instance and aggregate counters are read on this machine.
    /// Guarantees: Each publishes one point; the named instance carries its name and the lone
    /// aggregate carries none.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_exact_counter() {
        let mut collector = start(json!([
            {"object": "Processor", "instances": ["0", "_Total"],
             "counters": [{"name": "% Processor Time"}]}
        ]));
        let report = collector.scrape();
        let points = points(&report);
        assert_eq!(points.len(), 2, "{points:?}");
        assert_eq!((points[0].0, points[0].1), (0, Some("0")));
        assert_eq!((points[1].0, points[1].1), (1, None));
    }

    /// Scenario: A paging file instance whose name contains a backslash, such as
    /// `C:\pagefile.sys`, is found through `*` and then configured by name on this machine. Skips
    /// when the machine has no such paging file.
    /// Guarantees: The path is added and read, publishing one point that carries the
    /// instance name with its backslash.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_backslash_instance() {
        let report = start(json!([
            {"object": "Paging File", "instances": "*", "counters": [{"name": "% Usage"}]}
        ]))
        .scrape();
        let Some(name) = points(&report)
            .into_iter()
            .filter_map(|(_, instance, _)| instance)
            .find(|instance| instance.contains('\\'))
            .map(str::to_owned)
        else {
            eprintln!("skipped: no paging file instance with a backslash: {report:?}");
            return;
        };
        let mut collector = start(json!([
            {"object": "Paging File", "instances": name, "counters": [{"name": "% Usage"}]}
        ]));
        let report = collector.scrape();
        let points = points(&report);
        assert_eq!(points.len(), 1, "{points:?}");
        assert_eq!(points[0].1, Some(name.as_str()));
    }

    /// Scenario: A counter that does not exist on this machine is configured beside an exact counter.
    /// Guarantees: The add fails at start and is retried on every scrape, while the exact counter
    /// keeps publishing.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_missing_counter_is_retried() {
        let open: fn() -> Result<NativePdh, u32> = NativePdh::open;
        let (mut collector, report) = Collector::start(
            counters(json!([
                {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
                {"object": "Memory", "counters": [{"name": "No Such Counter"}]}
            ])),
            open,
            Arc::default(),
        );
        assert_eq!(report.add_failures.len(), 1, "{report:?}");
        assert_eq!(report.add_failures[0].0, 1);
        for _ in 0..2 {
            let report = collector.scrape();
            assert_eq!(report.add_failures.len(), 1, "{report:?}");
            assert_eq!(points(&report).len(), 1);
            assert_eq!(report.active_counters, 1);
        }
    }

    /// Scenario: `\Processor(*)\% Processor Time` is read with the default and with `_Total` listed.
    /// Guarantees: Every processor is published; `_Total` only when opted in.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_processor_wildcard_total_opt_in() {
        let mut collector = start(json!([
            {"object": "Processor", "instances": "*", "counters": [{"name": "% Processor Time"}]},
            {"object": "Processor", "instances": ["*", "_Total"],
             "counters": [{"name": "% Processor Time", "attributes": {"opt": "in"}}]}
        ]));
        let report = collector.scrape();
        let names = |counter: usize| -> Vec<&str> {
            report
                .points
                .iter()
                .filter(|point| point.counter == counter)
                .map(|point| point.instance.as_deref().unwrap())
                .collect()
        };
        assert!(
            names(0).contains(&"0") && !names(0).contains(&"_Total"),
            "{:?}",
            names(0)
        );
        assert!(
            names(1).contains(&"0") && names(1).contains(&"_Total"),
            "{:?}",
            names(1)
        );
    }

    /// Scenario: `\Process(*)\ID Process` is read while two owned processes share a name.
    /// Guarantees: The repeated name is published with a "#1" suffix, and every name is unique.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_process_duplicate_names() {
        let name = format!("wpcn{}", std::process::id());
        let _children = spawn_children(&name);
        let mut collector = start(json!([
            {"object": "Process", "instances": "*", "counters": [{"name": "ID Process"}]}
        ]));
        let report = collector.scrape();
        let names: Vec<_> = names(&report).into_iter().map(Option::unwrap).collect();
        assert!(
            names.contains(&name.as_str()) && names.contains(&format!("{name}#1").as_str()),
            "{names:?}"
        );
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), names.len());
    }

    /// Scenario: Partial instance patterns `name*` and `nam?` style are added and read while two
    /// owned processes share a name.
    /// Guarantees: PDH accepts both, each returns the owned processes, and every returned instance
    /// matches the pattern.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_partial_instance_patterns() {
        let name = format!("wpcp{}", std::process::id());
        let _children = spawn_children(&name);
        let prefix = &name[..name.len() - 1];
        let mut collector = start(json!([
            {"object": "Process", "instances": [format!("{name}*"), format!("{prefix}?")],
             "counters": [{"name": "ID Process"}]}
        ]));
        let report = collector.scrape();
        let points = points(&report);
        for counter in [0, 1] {
            let names: Vec<_> = points
                .iter()
                .filter(|point| point.0 == counter)
                .map(|point| point.1.unwrap())
                .collect();
            assert!(
                names.contains(&name.as_str()) && names.contains(&format!("{name}#1").as_str()),
                "{names:?}"
            );
        }
        for (_, instance, _) in points {
            assert!(instance.unwrap().starts_with(prefix), "{instance:?}");
        }
    }

    /// Scenario: A rate counter with recreate_query is scraped twice beside a counter without it.
    /// Guarantees: The first scrape reads the startup query and swaps in a primed replacement, which
    /// the second scrape reads without a recreate failure; the start time is kept, moving back only
    /// to an earlier read time, and the other counter keeps publishing.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_recreate_query() {
        let mut collector = start(json!([
            {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
            {"object": "Processor", "instances": "*",
             "counters": [{"name": "% Processor Time", "recreate_query": true}]}
        ]));
        let first = collector.scrape();
        assert!(first.recreate_failures.is_empty(), "{first:?}");
        std::thread::sleep(Duration::from_millis(1100));
        let second = collector.scrape();
        assert!(second.recreate_failures.is_empty(), "{second:?}");
        for report in [&first, &second] {
            assert!(report.points.iter().any(|point| point.counter == 0));
            assert!(report.points.iter().any(|point| point.counter == 1));
        }
        let start = |report: &Report| {
            report
                .points
                .iter()
                .find(|point| point.counter == 1)
                .unwrap()
                .start_time_unix_nano
        };
        let time = second
            .points
            .iter()
            .find(|point| point.counter == 1)
            .unwrap()
            .time_unix_nano;
        assert_eq!(start(&second), start(&first).min(time));
    }

    /// Scenario: A native worker is started, scraped and stopped.
    /// Guarantees: Shutdown closes the query and joins well before its deadline.
    #[tokio::test]
    #[ignore = "requires Windows performance counters"]
    async fn live_clean_shutdown() {
        let (worker, started) = Worker::start(
            counters(json!([{"object": "Memory", "counters": [{"name": "Available Bytes"}]}])),
            NativePdh::open,
        )
        .unwrap();
        assert_eq!(started.await.unwrap().active_counters, 1);
        let report = worker.scrape().await.unwrap();
        assert!(!report.collected.is_empty() && !report.points.is_empty());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        assert_eq!(worker.shutdown(deadline).await, Stopped::Cleanly);
    }
}
