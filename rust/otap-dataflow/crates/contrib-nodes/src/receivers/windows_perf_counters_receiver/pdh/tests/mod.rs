// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::config::{CounterConfig, InstanceConfig, MetricKind};
use super::super::model::{
    ExpansionOverflow, InstanceIdentity, Number, PendingMetadata, Sample, SampleDiagnostics,
    SampleFailure, SampleValue, merge_diagnostics,
};
use super::super::{config::Config, otap_builder::project_sample, test_utils::decode};
use super::expansion::*;
use super::native::NativePdh;
use super::query::*;
use super::worker::*;
use super::*;
use super::{QueryCadence, Worker};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command as ProcessCommand, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

pub(super) static TEST_PDH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

// Scoped tracing subscribers can race through process-wide callsite caches.
pub(super) fn assert_isolated_tracing(test: impl FnOnce()) {
    const CHILD: &str = "OTEL_ARROW_PERF_COUNTER_TRACING_TEST";
    let thread = std::thread::current();
    let name = thread.name().expect("named test thread");
    if std::env::var_os(CHILD) == Some(std::ffi::OsString::from(name)) {
        test();
        return;
    }
    let mut child = ProcessCommand::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .env(CHILD, name)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("isolated tracing subprocess");
    let mut stdout = child.stdout.take().expect("tracing subprocess stdout");
    let reader = std::thread::spawn(move || -> std::io::Result<Vec<u8>> {
        const CAPTURE_LIMIT: usize = 64 * 1024;
        let mut tail = Vec::new();
        let mut buffer = [0; 8192];
        loop {
            let read = stdout.read(&mut buffer)?;
            if read == 0 {
                return Ok(tail);
            }
            tail.extend_from_slice(&buffer[..read]);
            if tail.len() > CAPTURE_LIMIT {
                let excess = tail.len() - CAPTURE_LIMIT;
                drop(tail.drain(..excess));
            }
        }
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait().expect("tracing subprocess status") {
            break Some(status);
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill timed-out tracing subprocess");
            break None;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let _ = child.wait().expect("reap tracing subprocess");
    let output = reader
        .join()
        .expect("tracing stdout reader panicked")
        .expect("read tracing stdout");
    let stdout = String::from_utf8_lossy(&output);
    let Some(status) = status else {
        panic!("isolated tracing test timed out: {name}; stdout tail:\n{stdout}");
    };
    assert!(status.success(), "{name}; stdout tail:\n{stdout}");
    assert!(
        stdout.contains("1 passed;"),
        "no isolated test ran; stdout tail:\n{stdout}"
    );
}

/// Scenario: An isolated tracing test writes more stdout than a pipe or the diagnostic capture can hold.
/// Guarantees: Output is drained while the child runs, and its final one-test success is still verified.
#[test]
fn isolated_tracing_drains_large_child_stdout() {
    assert_isolated_tracing(|| {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(&vec![b'x'; 256 * 1024]).unwrap();
        stdout.flush().unwrap();
    });
}

// Native worker events cross threads; isolated test assertions share their capture.
#[derive(Clone, Default)]
pub(super) struct CapturedEvents(Arc<std::sync::Mutex<Vec<CapturedEvent>>>);

struct CapturedEvent {
    name: &'static str,
    error: Option<String>,
}

impl CapturedEvents {
    pub(super) fn dispatch(&self) -> tracing::Dispatch {
        use tracing_subscriber::layer::SubscriberExt;
        tracing::Dispatch::new(tracing_subscriber::registry().with(self.clone()))
    }

    pub(super) fn names(&self) -> Vec<&'static str> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|event| event.name)
            .collect()
    }

    fn counter_failures(&self) -> Vec<String> {
        self.failure_details("windows_perf_counters_receiver.counter.fail")
    }

    pub(super) fn query_failures(&self) -> Vec<String> {
        self.failure_details("windows_perf_counters_receiver.query.fail")
    }

    fn failure_details(&self, name: &str) -> Vec<String> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event.name == name)
            .map(|event| {
                event
                    .error
                    .as_ref()
                    .expect("failure event must include error detail")
                    .clone()
            })
            .collect()
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for CapturedEvents {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        #[derive(Default)]
        struct Detail(Option<String>);
        impl tracing::field::Visit for Detail {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "error" {
                    self.0 = Some(format!("{value:?}"));
                }
            }

            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "error" {
                    self.0 = Some(value.to_owned());
                }
            }
        }
        let mut detail = Detail::default();
        event.record(&mut detail);
        self.0.lock().unwrap().push(CapturedEvent {
            name: event.metadata().name(),
            error: detail.0,
        });
    }
}

pub(super) fn counter(path: &str, name: &str) -> CounterConfig {
    CounterConfig {
        path: path.to_owned(),
        name: Arc::from(name),
        unit: Arc::from("By"),
        description: Arc::from(format!("Description for {name}.")),
        metric_kind: MetricKind::Gauge,
        attributes: Arc::new(BTreeMap::new()),
        instance_config: InstanceConfig::None,
        scale_power10: 0,
        source_mapping_index: 0,
    }
}

fn cumulative_sum(path: &str, name: &str, unit: &str) -> CounterConfig {
    CounterConfig {
        metric_kind: MetricKind::Sum,
        unit: Arc::from(unit),
        ..counter(path, name)
    }
}

fn collect_query<A, F>(slot: &mut QuerySlot<A>, open: &mut F, now: Instant) -> Result<Sample, Error>
where
    A: PdhApi,
    F: QueryFactory<A>,
{
    collect_query_with_diagnostics(slot, open, now, |_, _| Ok(()))
}

fn query_counts() -> (usize, usize) {
    (
        QUERY_OPENS.load(Ordering::Relaxed),
        QUERY_CLOSES.load(Ordering::Relaxed),
    )
}

struct FakeCounter {
    path: String,
    native_type: u32,
    raw: Option<i64>,
    multi_count: u32,
    formatted: Option<Number>,
    raw_error: bool,
    formatted_error: bool,
}

fn fake_counter(path: &str, formatted: Number) -> FakeCounter {
    FakeCounter {
        path: path.to_owned(),
        native_type: 0x0001_0100,
        raw: None,
        multi_count: 1,
        formatted: Some(formatted),
        raw_error: false,
        formatted_error: false,
    }
}

struct FakeState {
    counters: Vec<FakeCounter>,
    added: Vec<String>,
    attempted_adds: Vec<(String, CounterLanguage)>,
    calls: FakeNativeCalls,
    collect_has_data: bool,
    collect_error: bool,
    collect_status: Option<u32>,
    timestamp: i64,
    timestamp_error: bool,
    formatted_timestamp: Option<i64>,
    remove_error: bool,
    remove_errors: BTreeSet<usize>,
    expand_error: bool,
    expand_operation: PdhOperation,
    lookup_failure_on_cleanup: bool,
    object_refresh_error: bool,
    expand_errors: BTreeSet<String>,
    stop_after_expand: Option<Arc<AtomicBool>>,
    stop_after_add: Option<Arc<AtomicBool>>,
    stop_after_parse: Option<Arc<AtomicBool>>,
    removed: Vec<usize>,
    expansions: BTreeMap<String, Vec<String>>,
    full_paths: BTreeMap<String, String>,
    identity_errors: BTreeMap<String, FakeIdentityError>,
    comparison_errors: BTreeMap<String, FakeIdentityError>,
    decoded_expansion_bytes: BTreeMap<String, usize>,
}

#[derive(Clone, Copy)]
enum FakeIdentityError {
    Recoverable,
    Stop,
    Rebuild,
}

impl FakeIdentityError {
    fn error(self, path: &str) -> Error {
        match self {
            Self::Recoverable => Error::Pdh {
                operation: PdhOperation::GetCounterInfo,
                path: path.to_owned(),
                status: 1,
            },
            Self::Stop => Error::WorkerStopRequested,
            Self::Rebuild => Error::CounterSetup {
                path: path.to_owned(),
                message: "identity read requires rebuild".to_owned(),
                permanent: false,
                cleanup_failed: true,
            },
        }
    }
}

#[derive(Default)]
struct FakeNativeCalls {
    add: usize,
    counter_type: usize,
    full_path: usize,
    collect: usize,
    raw: usize,
    formatted: usize,
    timestamp: usize,
    expand: usize,
    parse: usize,
    refresh: usize,
    order: Vec<&'static str>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FakeCounterToken {
    index: usize,
    generation: u64,
}

struct FakePdh {
    state: Rc<RefCell<FakeState>>,
    generations: Vec<u64>,
    active: BTreeSet<FakeCounterToken>,
}

impl FakePdh {
    fn new(state: Rc<RefCell<FakeState>>) -> Self {
        let counter_count = state.borrow().counters.len();
        Self {
            state,
            generations: vec![0; counter_count],
            active: BTreeSet::new(),
        }
    }

    fn counter_index(&self, counter: &FakeCounterToken, path: &str) -> Result<usize, Error> {
        if self.active.contains(counter) {
            Ok(counter.index)
        } else {
            Err(Error::InvalidCounterToken {
                path: path.to_owned(),
            })
        }
    }
}

impl PdhApi for FakePdh {
    type Counter = FakeCounterToken;

    fn check_running(&self) -> Result<(), Error> {
        let state = self.state.borrow();
        if state
            .stop_after_expand
            .iter()
            .chain(state.stop_after_add.iter())
            .chain(state.stop_after_parse.iter())
            .any(|stop| stop.load(Ordering::Acquire))
        {
            Err(Error::WorkerStopRequested)
        } else {
            Ok(())
        }
    }

    fn add(&mut self, path: &str, language: CounterLanguage) -> Result<Self::Counter, Error> {
        let mut state = self.state.borrow_mut();
        state.calls.add += 1;
        state.attempted_adds.push((path.to_owned(), language));
        let counter = state
            .counters
            .iter()
            .position(|counter| counter.path == path)
            .ok_or_else(|| Error::Pdh {
                operation: PdhOperation::AddEnglishCounter,
                path: path.to_owned(),
                status: 1,
            })?;
        state.added.push(path.to_owned());
        if let Some(stop) = &state.stop_after_add {
            stop.store(true, Ordering::Release);
        }
        if counter >= self.generations.len() {
            self.generations.resize(counter + 1, 0);
        }
        self.generations[counter] = self.generations[counter].wrapping_add(1);
        let generation = self.generations[counter];
        let token = FakeCounterToken {
            index: counter,
            generation,
        };
        let _ = self.active.insert(token);
        Ok(token)
    }

    fn remove(&mut self, counter: &Self::Counter, path: &str) -> Result<(), Error> {
        let index = self.counter_index(counter, path)?;
        let mut state = self.state.borrow_mut();
        state.removed.push(index);
        if state.remove_error || state.remove_errors.contains(&index) {
            Err(Error::Pdh {
                operation: PdhOperation::RemoveCounter,
                path: state.counters[index].path.clone(),
                status: 1,
            })
        } else {
            let _ = self.active.remove(counter);
            Ok(())
        }
    }

    fn counter_type(&mut self, counter: &Self::Counter, path: &str) -> Result<u32, Error> {
        let index = self.counter_index(counter, path)?;
        let mut state = self.state.borrow_mut();
        state.calls.counter_type += 1;
        Ok(state.counters[index].native_type)
    }

    fn full_path(&mut self, counter: &Self::Counter, path: &str) -> Result<String, Error> {
        let _ = self.counter_index(counter, path)?;
        let mut state = self.state.borrow_mut();
        state.calls.full_path += 1;
        if let Some(error) = state.identity_errors.get(path) {
            return Err(error.error(path));
        }
        Ok(state
            .full_paths
            .get(path)
            .map_or_else(|| path.to_owned(), Clone::clone))
    }

    fn comparison_path(&mut self, path: &str) -> Result<String, Error> {
        if let Some(error) = self.state.borrow().comparison_errors.get(path) {
            return Err(error.error(path));
        }
        NativePdh::comparison_path_fixture(path)
    }

    fn collect(&mut self) -> Result<bool, Error> {
        let mut state = self.state.borrow_mut();
        state.calls.collect += 1;
        if state.collect_error || state.collect_status.is_some() {
            Err(Error::Pdh {
                operation: PdhOperation::CollectQueryData,
                path: "<query>".to_owned(),
                status: state.collect_status.unwrap_or(1),
            })
        } else {
            Ok(state.collect_has_data)
        }
    }

    fn raw(&mut self, counter: &Self::Counter, path: &str) -> Result<Option<RawSample>, Error> {
        let index = self.counter_index(counter, path)?;
        let mut state = self.state.borrow_mut();
        state.calls.raw += 1;
        let counter = &state.counters[index];
        if counter.raw_error {
            Err(Error::Pdh {
                operation: PdhOperation::GetRawCounterValue,
                path: path.to_owned(),
                status: 1,
            })
        } else {
            Ok(counter.raw.map(|second_value| RawSample {
                second_value,
                multi_count: counter.multi_count,
            }))
        }
    }

    fn formatted(
        &mut self,
        counter: &Self::Counter,
        path: &str,
        format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        let index = self.counter_index(counter, path)?;
        let mut state = self.state.borrow_mut();
        state.calls.formatted += 1;
        let counter = &state.counters[index];
        if counter.formatted_error {
            return Err(Error::CounterData {
                operation: PdhOperation::FormattedCounterDataStatus,
                path: path.to_owned(),
                status: 1,
            });
        }
        if let Some(number) = counter.formatted {
            assert!(matches!(
                (format, number),
                (ValueFormat::Integer, Number::Integer(_))
                    | (ValueFormat::Double, Number::Double(_))
            ));
        }
        let formatted = counter.formatted;
        if let Some(timestamp) = state.formatted_timestamp {
            state.timestamp = timestamp;
        }
        Ok(formatted)
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        let mut state = self.state.borrow_mut();
        state.calls.timestamp += 1;
        if state.timestamp_error {
            Err(Error::Timestamp("test timestamp failure"))
        } else {
            Ok(state.timestamp)
        }
    }

    fn refresh_objects(&mut self) -> Result<usize, Error> {
        let mut state = self.state.borrow_mut();
        state.calls.refresh += 1;
        state.calls.order.push("objects.refresh");
        if state.object_refresh_error {
            Err(Error::WorkerObjectRefresh {
                stage: "refresh sizing",
                status: 5,
            })
        } else {
            Ok(12)
        }
    }

    fn expand(&mut self, path: &str, budget: &mut DiscoveryBudget) -> Result<Vec<String>, Error> {
        let mut state = self.state.borrow_mut();
        state.calls.expand += 1;
        state.calls.order.push("lookup");
        if let Some(stop) = &state.stop_after_expand {
            stop.store(true, Ordering::Release);
        }
        if state.expand_error || state.expand_errors.contains(path) {
            if state.expand_operation == PdhOperation::RemoveCounter {
                return Err(Error::WildcardCleanup {
                    path: path.to_owned(),
                    message: "temporary handle cleanup is pending".to_owned(),
                    lookup_failed: state.lookup_failure_on_cleanup,
                });
            }
            return Err(Error::Pdh {
                operation: state.expand_operation,
                path: path.to_owned(),
                status: 1,
            });
        }
        let paths = state.expansions.get(path).map_or(&[][..], Vec::as_slice);
        budget.with_transient(0, |budget| {
            if let Some(bytes) = state.decoded_expansion_bytes.get(path) {
                budget.reserve(*bytes)?;
            }
            budget.reserve(paths.iter().map(String::len).sum())?;
            Ok(paths.to_vec())
        })
    }

    fn parse_instance(&mut self, path: &str) -> Result<InstanceIdentity, Error> {
        let mut state = self.state.borrow_mut();
        state.calls.parse += 1;
        if let Some(stop) = &state.stop_after_parse {
            stop.store(true, Ordering::Release);
        }
        NativePdh::parse_instance_path(path)
    }
}

struct CalculatedPdh {
    fail_collect: Arc<AtomicBool>,
    collect_calls: usize,
    fail_first_collect: bool,
}

impl PdhApi for CalculatedPdh {
    type Counter = ();

    fn add(&mut self, _path: &str, _language: CounterLanguage) -> Result<Self::Counter, Error> {
        Ok(())
    }

    fn remove(&mut self, _counter: &Self::Counter, _path: &str) -> Result<(), Error> {
        Ok(())
    }

    fn counter_type(&mut self, _counter: &Self::Counter, _path: &str) -> Result<u32, Error> {
        Ok(0x2041_0500)
    }

    fn full_path(&mut self, _counter: &Self::Counter, path: &str) -> Result<String, Error> {
        Ok(path.to_owned())
    }

    fn comparison_path(&mut self, path: &str) -> Result<String, Error> {
        NativePdh::comparison_path_fixture(path)
    }

    fn collect(&mut self) -> Result<bool, Error> {
        if std::mem::take(&mut self.fail_first_collect) || self.fail_collect.load(Ordering::Relaxed)
        {
            return Err(Error::Pdh {
                operation: PdhOperation::CollectQueryData,
                path: "<query>".to_owned(),
                status: 1,
            });
        }
        self.collect_calls += 1;
        Ok(true)
    }

    fn raw(&mut self, _counter: &Self::Counter, _path: &str) -> Result<Option<RawSample>, Error> {
        Ok(Some(RawSample {
            second_value: i64::try_from(self.collect_calls).unwrap_or(i64::MAX) * 10,
            multi_count: 1,
        }))
    }

    fn formatted(
        &mut self,
        _counter: &Self::Counter,
        _path: &str,
        _format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        Ok(Some(Number::Double(self.collect_calls as f64)))
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        Ok(100 + i64::try_from(self.collect_calls).unwrap_or(i64::MAX) * 10)
    }

    fn refresh_objects(&mut self) -> Result<usize, Error> {
        Ok(0)
    }

    fn expand(&mut self, _path: &str, _budget: &mut DiscoveryBudget) -> Result<Vec<String>, Error> {
        Ok(Vec::new())
    }

    fn parse_instance(&mut self, path: &str) -> Result<InstanceIdentity, Error> {
        Err(Error::InvalidCounterToken {
            path: path.to_owned(),
        })
    }
}

fn fake_query(
    configs: Vec<CounterConfig>,
    counters: Vec<FakeCounter>,
) -> (Rc<RefCell<FakeState>>, Query<FakePdh>) {
    fake_query_with_cadences(
        configs,
        counters,
        Duration::from_secs(30),
        Duration::from_secs(30),
    )
}

fn fake_query_with_cadences(
    configs: Vec<CounterConfig>,
    counters: Vec<FakeCounter>,
    exact_retry_cap: Duration,
    wildcard_refresh_interval: Duration,
) -> (Rc<RefCell<FakeState>>, Query<FakePdh>) {
    fake_query_with_seed(
        configs,
        counters,
        exact_retry_cap,
        wildcard_refresh_interval,
        4_096,
        None,
    )
}

fn fake_state_value(counters: Vec<FakeCounter>) -> FakeState {
    FakeState {
        counters,
        added: Vec::new(),
        attempted_adds: Vec::new(),
        calls: FakeNativeCalls::default(),
        collect_has_data: true,
        collect_error: false,
        collect_status: None,
        timestamp: 100,
        timestamp_error: false,
        formatted_timestamp: None,
        remove_error: false,
        remove_errors: BTreeSet::new(),
        expand_error: false,
        expand_operation: PdhOperation::ExpandWildcardPath,
        lookup_failure_on_cleanup: false,
        object_refresh_error: false,
        expand_errors: BTreeSet::new(),
        stop_after_expand: None,
        stop_after_add: None,
        stop_after_parse: None,
        removed: Vec::new(),
        expansions: BTreeMap::new(),
        full_paths: BTreeMap::new(),
        identity_errors: BTreeMap::new(),
        comparison_errors: BTreeMap::new(),
        decoded_expansion_bytes: BTreeMap::new(),
    }
}

fn fake_state(counters: Vec<FakeCounter>) -> Rc<RefCell<FakeState>> {
    Rc::new(RefCell::new(fake_state_value(counters)))
}

fn fake_query_with_seed(
    configs: Vec<CounterConfig>,
    counters: Vec<FakeCounter>,
    exact_retry_cap: Duration,
    wildcard_refresh_interval: Duration,
    max_native_counters: usize,
    seed: Option<&QueryRecreationSeed>,
) -> (Rc<RefCell<FakeState>>, Query<FakePdh>) {
    let state = fake_state(counters);
    let query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        exact_retry_cap,
        wildcard_refresh_interval,
        256,
        max_native_counters,
        seed,
    )
    .unwrap();
    (state, query)
}

/// Scenario: A temporary English wildcard lookup succeeds, fails metadata, stops, or fails removal.
/// Guarantees: Removal is attempted on every post-add path; failed cleanup retains one uncollected handle and cannot acquire another.
#[test]
fn temporary_template_cleanup_is_bounded_and_stop_safe() {
    let path = r"\Process(svchos?)\ID Process";
    let localized = r"\Prozess(svchos?)\Prozesskennung";
    for info_fails in [false, true] {
        for removal_fails in [false, true] {
            for stopped in [false, true] {
                let state = fake_state(vec![fake_counter(path, Number::Integer(1))]);
                state.borrow_mut().remove_error = removal_fails;
                if stopped {
                    state.borrow_mut().stop_after_add = Some(Arc::new(AtomicBool::new(false)));
                }
                let mut api = FakePdh::new(Rc::clone(&state));
                let mut pending = None;
                let result = native::resolve_template(&mut api, &mut pending, path, |api, _, _| {
                    api.check_running()?;
                    if info_fails {
                        Err(Error::Pdh {
                            operation: PdhOperation::GetCounterInfo,
                            path: path.to_owned(),
                            status: 5,
                        })
                    } else {
                        Ok(localized.to_owned())
                    }
                });
                assert_eq!(pending.is_some(), removal_fails);
                assert_eq!(state.borrow().removed, [0]);
                assert_eq!(state.borrow().calls.collect, 0);
                assert_eq!(
                    state.borrow().attempted_adds,
                    [(path.to_owned(), CounterLanguage::English)]
                );
                if stopped {
                    assert!(result.is_err_and(|error| error.is_stop()));
                } else if removal_fails {
                    assert!(result.is_err_and(|error| matches!(error,
                        Error::WildcardCleanup { message, lookup_failed, .. }
                            if lookup_failed == info_fails
                                && message.contains("PdhRemoveCounter")
                                && (!info_fails || message.contains("PdhGetCounterInfoW")))));
                    let retry =
                        native::resolve_template(&mut api, &mut pending, path, |_, _, _| {
                            panic!("pending cleanup must finish before another lookup")
                        })
                        .unwrap_err();
                    assert!(!retry.requires_immediate_query_rebuild());
                    assert!(!retry.is_query_collection_failure());
                    assert_eq!(state.borrow().calls.add, 1);
                    assert_eq!(api.active.len(), 1);
                    state.borrow_mut().remove_error = false;
                    let resolved =
                        native::resolve_template(&mut api, &mut pending, path, |_, _, _| {
                            Ok(localized.to_owned())
                        })
                        .unwrap();
                    assert_eq!(resolved, localized);
                    assert!(pending.is_none());
                    assert!(api.active.is_empty());
                    assert_eq!(state.borrow().calls.add, 2);
                } else if info_fails {
                    assert!(result.is_err());
                } else {
                    assert_eq!(result.unwrap(), localized);
                }
            }
        }
    }
}

mod live;
mod query;
mod timing;
mod worker;
