// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::config::{Config, CounterConfig};
use super::query::Collector;
use super::*;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
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
    fail_open: bool,
    adds: Vec<String>,
    missing: HashSet<String>,
    collect_status: Option<u32>,
    /// One-shot collect statuses, used before `collect_status`.
    collect_script: VecDeque<u32>,
    collects: usize,
    gate: Option<mpsc::Receiver<()>>,
    arrays: HashMap<String, Result<Vec<(String, Value)>, u32>>,
}

/// One fake query, numbered by the order it was opened.
struct FakePdh {
    state: Arc<Mutex<State>>,
    id: usize,
}

impl Drop for FakePdh {
    fn drop(&mut self) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .closes += 1;
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
        if let Some(gate) = gate {
            let _ = gate.recv();
        }
        let mut state = self.state.lock().unwrap();
        state.collects += 1;
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
const PROCESSES: &str = r"\Process(*)\ID Process";
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
    let (mut collector, start) = Collector::start(config, opener(&state));
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
    let (mut collector, _) = Collector::start(config, opener(&state));
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
    let (mut collector, _) = Collector::start(counters(json!(perfcounters)), opener(&state));
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
    let (mut collector, _) = Collector::start(config, opener(&state));
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
    let (mut collector, _) = Collector::start(config, opener(&state));
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
    let (mut collector, _) = Collector::start(config, opener(&state));
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

/// Scenario: A wildcard array repeats a name, and one repeat has an invalid CStatus.
/// Guarantees: Invalid items are dropped first, then the first occurrence keeps its name and later
/// ones get "#1", "#2" in returned order, as in Go.
#[test]
fn suffixes_duplicate_names_after_filtering() {
    let state = fake();
    set_array(
        &state,
        PROCESSES,
        Ok(vec![
            item("svchost", PDH_CSTATUS_INVALID_DATA, 0.0),
            item("svchost", PDH_CSTATUS_VALID_DATA, 1.0),
            item("app", PDH_CSTATUS_VALID_DATA, 2.0),
            item("svchost", PDH_CSTATUS_VALID_DATA, 3.0),
            item("svchost", PDH_CSTATUS_VALID_DATA, 4.0),
        ]),
    );
    let config = counters(json!([
        {"object": "Process", "instances": "*", "counters": [{"name": "ID Process"}]}
    ]));
    let (mut collector, _) = Collector::start(config, opener(&state));
    assert_eq!(
        points(&collector.scrape()),
        [
            (0, Some("svchost"), 1.0),
            (0, Some("app"), 2.0),
            (0, Some("svchost#1"), 3.0),
            (0, Some("svchost#2"), 4.0),
        ]
    );
}

/// Scenario: Default `*` arrays hold an invalid `_Total` beside a valid one, and two valid `_Total`s.
/// Guarantees: Go's order holds: filter, clear a lone aggregate's name, suffix duplicates, then
/// drop the aggregate, so a filtered-to-lone `_Total` is published without an instance and a
/// suffixed `_Total#1` is not dropped.
#[test]
fn clears_lone_aggregate_then_suffixes_then_drops() {
    let state = fake();
    let config = counters(json!([
        {"object": "Processor", "instances": "*", "counters": [{"name": "% Processor Time"}]}
    ]));
    for (array, expected) in [
        (
            vec![
                item("_Total", PDH_CSTATUS_INVALID_DATA, 1.0),
                item("_Total", PDH_CSTATUS_VALID_DATA, 2.0),
            ],
            vec![(0, None, 2.0)],
        ),
        (
            vec![
                item("_Total", PDH_CSTATUS_VALID_DATA, 1.0),
                item("_Total", PDH_CSTATUS_VALID_DATA, 2.0),
            ],
            vec![(0, Some("_Total#1"), 2.0)],
        ),
    ] {
        set_array(&state, PROCESSOR, Ok(array));
        let (mut collector, _) = Collector::start(config.clone(), opener(&state));
        assert_eq!(points(&collector.scrape()), expected);
    }
}

/// Scenario: `*` is read with the default aggregate, with `_Total` also listed, and with a custom
/// aggregation_name.
/// Guarantees: The aggregate is dropped unless it is listed alongside `*`; a custom name changes
/// which instance is the aggregate.
#[test]
fn excludes_aggregate_unless_listed() {
    let state = fake();
    set_array(
        &state,
        PROCESSOR,
        Ok(vec![
            item("0", PDH_CSTATUS_VALID_DATA, 1.0),
            item("_Total", PDH_CSTATUS_VALID_DATA, 2.0),
            item("1", PDH_CSTATUS_VALID_DATA, 3.0),
        ]),
    );
    for (instances, aggregation_name, expected) in [
        (json!("*"), "", vec![Some("0"), Some("1")]),
        (
            json!(["*", "_Total"]),
            "",
            vec![Some("0"), Some("_Total"), Some("1")],
        ),
        (json!("*"), "1", vec![Some("0"), Some("_Total")]),
    ] {
        let config = counters(json!([{
            "object": "Processor",
            "instances": instances,
            "aggregation_name": aggregation_name,
            "counters": [{"name": "% Processor Time"}]
        }]));
        let (mut collector, _) = Collector::start(config, opener(&state));
        assert_eq!(names(&collector.scrape()), expected);
    }
}

/// Scenario: A wildcard returns only the aggregate instance, and an exact `_Total` path is read.
/// Guarantees: A lone aggregate is published without an instance attribute in both forms; with the
/// aggregate opted in it keeps its name.
#[test]
fn lone_aggregate_has_no_instance() {
    let state = fake();
    let pattern = r"\Processor(_T*)\% Processor Time";
    let exact = r"\Processor(_Total)\% Processor Time";
    set_array(
        &state,
        pattern,
        Ok(vec![item("_Total", PDH_CSTATUS_VALID_DATA, 5.0)]),
    );
    set_array(
        &state,
        PROCESSOR,
        Ok(vec![item("_Total", PDH_CSTATUS_VALID_DATA, 5.0)]),
    );
    set_array(
        &state,
        exact,
        Ok(vec![item("_Total", PDH_CSTATUS_VALID_DATA, 6.0)]),
    );
    let config = counters(json!([
        {"object": "Processor", "instances": ["_T*", "_Total"],
         "counters": [{"name": "% Processor Time"}]},
        {"object": "Processor", "instances": ["*", "_Total"],
         "counters": [{"name": "% Processor Time", "metric": ""}], "aggregation_name": "_Total"}
    ]));
    let (mut collector, _) = Collector::start(config, opener(&state));
    assert_eq!(
        points(&collector.scrape()),
        [(0, None, 5.0), (1, None, 6.0), (2, Some("_Total"), 5.0)]
    );
}

/// Scenario: A collection fails, then succeeds again.
/// Guarantees: The failing scrape reports the status and drops the query; the next scrape closes
/// and reopens it, re-adds every path and primes without publishing; the scrape after that
/// publishes with a start time no earlier than the reopen.
#[test]
fn reopens_query_after_collect_failure() {
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]}
    ]));
    let (mut collector, _) = Collector::start(config, opener(&state));
    let first = collector.scrape().points.remove(0);

    state.lock().unwrap().collect_status = Some(COLLECT_FAILED);
    let failed = collector.scrape();
    assert_eq!(failed.collect_failures, [(0, COLLECT_FAILED)]);
    assert!(failed.points.is_empty() && failed.collected.is_empty());
    assert_eq!(failed.active_counters, 0);
    assert_eq!(state.lock().unwrap().closes, 1);

    state.lock().unwrap().collect_status = None;
    let reopened = collector.scrape();
    assert!(reopened.collect_failures.is_empty() && reopened.points.is_empty());
    assert_eq!(reopened.added, [0]);
    {
        let state = state.lock().unwrap();
        assert_eq!((state.opens, state.closes), (2, 1));
        assert_eq!(state.adds, [MEMORY, MEMORY]);
    }

    let after = collector.scrape().points.remove(0);
    assert!(after.start_time_unix_nano > first.start_time_unix_nano);
    assert!(after.start_time_unix_nano >= first.time_unix_nano);
}

/// Scenario: Two counters; the first counter's collection fails in one scrape while the second's
/// succeeds.
/// Guarantees: Each counter has its own query, so the healthy counter still collects and publishes
/// in the failing scrape and its query is never reopened; the failure is reported for the failed
/// counter only; only that counter's query is reopened, re-added and primed on the next scrape,
/// and priming is not reported as a collection.
#[test]
fn collect_failure_stops_only_its_counter() {
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    set_single(&state, SYSTEM, 2.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
        {"object": "System", "counters": [{"name": "Threads"}]}
    ]));
    let (mut collector, start) = Collector::start(config, opener(&state));
    assert_eq!(start.active_counters, 2);
    assert_eq!(state.lock().unwrap().opens, 2);

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
    assert_eq!(
        points(&collector.scrape()),
        [(0, None, 1.0), (1, None, 2.0)]
    );
}

/// Scenario: One counter uses recreate_query beside a counter without it; replacement later fails
/// to open.
/// Guarantees: The recreate counter's query is replaced after every scrape (open, add, prime,
/// swap, close old) with a new start time, while the other counter's query is untouched; a failed
/// replacement is reported and the old query keeps publishing.
#[test]
fn recreates_query_after_each_scrape() {
    let state = fake();
    set_single(&state, MEMORY, 1.0);
    set_single(&state, SYSTEM, 2.0);
    let config = counters(json!([
        {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
        {"object": "System", "counters": [{"name": "Threads", "recreate_query": true}]}
    ]));
    let (mut collector, start) = Collector::start(config, opener(&state));
    assert_eq!(start.active_counters, 2);
    assert_eq!(state.lock().unwrap().opens, 2);

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
    assert_eq!(
        second.points[0].start_time_unix_nano,
        first.points[0].start_time_unix_nano
    );
    assert!(second.points[1].start_time_unix_nano > first.points[1].start_time_unix_nano);
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
    let (release, gate) = mpsc::channel();
    state.lock().unwrap().gate = Some(gate);
    let _pending = worker.scrape();
    let soon = tokio::time::Instant::now() + std::time::Duration::from_millis(50);
    assert_eq!(worker.shutdown(soon).await, Stopped::TimedOut);
    assert_eq!(state.lock().unwrap().closes, 0);
    release.send(()).unwrap();

    // No array is scripted for the path, so the fake panics on the first read.
    let state = fake();
    let (worker, started) = Worker::start(config, opener(&state)).unwrap();
    let _ = started.await.unwrap();
    assert!(worker.scrape().await.is_err());
    assert_eq!(worker.shutdown(deadline()).await, Stopped::Failed);
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
    use super::super::NativePdh;
    use super::*;
    use std::time::Duration;

    type Live = Collector<NativePdh, fn() -> Result<NativePdh, u32>>;

    fn start(perfcounters: serde_json::Value) -> Live {
        let open: fn() -> Result<NativePdh, u32> = NativePdh::open;
        let (collector, report) = Collector::start(counters(perfcounters), open);
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

    /// Scenario: `\Process(*)\ID Process` is read while several svchost processes run.
    /// Guarantees: Repeated process names are published with "#1" style suffixes and unique names.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_process_duplicate_names() {
        let mut collector = start(json!([
            {"object": "Process", "instances": "*", "counters": [{"name": "ID Process"}]}
        ]));
        let report = collector.scrape();
        let names: Vec<_> = names(&report).into_iter().map(Option::unwrap).collect();
        assert!(names.contains(&"svchost#1"), "{names:?}");
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), names.len());
    }

    /// Scenario: Partial instance patterns `svc*` and `svchos?` are added and read.
    /// Guarantees: PDH accepts both, and every returned instance matches the pattern.
    #[test]
    #[ignore = "requires Windows performance counters"]
    fn live_partial_instance_patterns() {
        let mut collector = start(json!([
            {"object": "Process", "instances": ["svc*", "svchos?"],
             "counters": [{"name": "ID Process"}]}
        ]));
        let report = collector.scrape();
        let points = points(&report);
        for counter in [0, 1] {
            assert!(points.iter().any(|point| point.0 == counter), "{points:?}");
        }
        for (_, name, _) in points {
            assert!(name.unwrap().starts_with("svc"), "{name:?}");
        }
    }

    /// Scenario: A rate counter with recreate_query is scraped twice beside a counter without it.
    /// Guarantees: Each scrape publishes the recreated counter from its freshly primed replacement,
    /// with a newer start time, and the other counter keeps publishing.
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
        assert!(start(&second) > start(&first));
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
