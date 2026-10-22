// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: An owned worker join temporarily has no completion receiver.
/// Guarantees: Shutdown returns WorkerStopped without panicking or losing join ownership, and cleanup can still complete.
#[tokio::test(flavor = "current_thread")]
async fn missing_shutdown_completion_preserves_join_ownership() {
    let (mut worker, ownership, _started, _release) = blocked_collect_worker().await;
    assert_eq!(worker.startup.active_expanded, 0);
    let completion = worker.completion.take();
    assert!(matches!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await,
        Err(Error::WorkerStopped)
    ));
    assert!(worker.join.is_some());
    worker.completion = completion;
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
    assert_eq!(ownership.current.load(Ordering::Acquire), 0);
    assert_eq!(ownership.closed_queries.load(Ordering::Acquire), 1);
}

/// Scenario: Every exact mapping is permanently unsupported rather than temporarily unavailable.
/// Guarantees: Startup fails contextually, releases rejected handles, and does not advertise pending readiness without recovery work.
#[test]
fn permanently_rejected_exact_startup_is_fatal() {
    let path = r"\Provider\Unsupported";
    let state = fake_state(vec![FakeCounter {
        native_type: 0x4003_0401,
        ..fake_counter(path, Number::Integer(1))
    }]);
    let opens = Cell::new(0);
    let mut open = |configs, limit, seed: Option<&QueryRecreationSeed>| {
        opens.set(opens.get() + 1);
        Query::open_with_seed(
            FakePdh::new(Rc::clone(&state)),
            configs,
            Duration::ZERO,
            Duration::from_secs(60),
            256,
            limit,
            seed,
        )
    };
    let result = open_initial_query(
        vec![counter(path, "unsupported")],
        Duration::ZERO,
        1,
        &mut open,
        &AtomicBool::new(false),
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("permanent rejection reported readiness"),
    };
    assert!(matches!(error, Error::NoUsableCounters { .. }));
    assert!(error.to_string().contains("unsupported_type"));
    assert_eq!(opens.get(), 1);
    assert_eq!(state.borrow().removed, [0]);
    assert_eq!(state.borrow().calls.collect, 0);
}

/// Scenario: Initial and replacement open or setup cleanup fail recoverably before a later successful open.
/// Guarantees: Pending readiness and bounded retry preserve contextual diagnostic-only warnings, and successful rebuild primes before publication.
#[tokio::test(flavor = "current_thread")]
async fn recoverable_startup_errors_become_pending_and_rebuild() {
    for cleanup_failed in [false, true] {
        let configs = vec![counter(r"\Memory\Available Bytes", "memory")];
        let opens = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let factory_opens = Arc::clone(&opens);
        let dropped = Arc::new(AtomicBool::new(false));
        let factory_dropped = Arc::clone(&dropped);
        let mut worker = Worker::start_with(
            configs.clone(),
            Duration::from_millis(20),
            4096,
            move |configs, limit, _seed| {
                if factory_opens.fetch_add(1, Ordering::AcqRel) < 2 {
                    return Err(if cleanup_failed {
                        Error::CounterSetup {
                            path: configs[0].path.clone(),
                            message: "injected cleanup failure".to_owned(),
                            permanent: false,
                            cleanup_failed: true,
                        }
                    } else {
                        Error::Pdh {
                            operation: PdhOperation::OpenQuery,
                            path: "<query>".to_owned(),
                            status: 1,
                        }
                    });
                }
                Query::open_with_seed(
                    StartupTestPdh::<false> {
                        dropped: Arc::clone(&factory_dropped),
                    },
                    configs,
                    Duration::ZERO,
                    Duration::from_secs(60),
                    256,
                    limit,
                    None,
                )
            },
        )
        .await
        .unwrap();
        assert!(worker.startup.pending);
        let pending = worker.collect().await.unwrap();
        assert!(pending.metadata.diagnostics.query_pending);
        assert!(!pending.metadata.diagnostics.any_query_collected);
        assert!(pending.points.is_empty());
        assert!(pending.timestamp_unix_nano > 0);
        assert!(project_sample(&configs, &pending).unwrap().is_none());
        assert_eq!(opens.load(Ordering::Acquire), 1);
        assert!(pending.metadata.failures.iter().any(|failure| {
            failure.reason
                == if cleanup_failed {
                    "counter_setup"
                } else {
                    "PdhOpenQueryW"
                }
                && failure.diagnostic_only
        }));
        tokio::time::sleep(Duration::from_millis(30)).await;
        let failed_rebuild = worker.collect().await.unwrap();
        assert!(failed_rebuild.points.is_empty());
        assert!(failed_rebuild.timestamp_unix_nano > 0);
        assert!(project_sample(&configs, &failed_rebuild).unwrap().is_none());
        assert!(!failed_rebuild.metadata.diagnostics.any_query_collected);
        assert!(failed_rebuild.metadata.diagnostics.query_pending);
        assert_eq!(
            failed_rebuild.metadata.diagnostics.query_rebuild_attempts,
            1
        );
        assert!(failed_rebuild.metadata.failures.iter().any(|failure| {
            failure.path.as_ref() == r"\Memory\Available Bytes"
                && failure.reason
                    == if cleanup_failed {
                        "counter_setup"
                    } else {
                        "PdhOpenQueryW"
                    }
                && failure.diagnostic_only
                && !failure.error.is_empty()
        }));
        assert_eq!(opens.load(Ordering::Acquire), 2);
        tokio::time::sleep(Duration::from_millis(30)).await;
        let rebuilt = worker.collect().await.unwrap();
        assert!(!rebuilt.metadata.diagnostics.any_query_collected);
        assert!(!rebuilt.metadata.diagnostics.query_pending);
        assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_attempts, 1);
        assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_recoveries, 1);
        assert!(rebuilt.points.is_empty());
        assert!(project_sample(&configs, &rebuilt).unwrap().is_none());
        let ready = worker.collect().await.unwrap();
        assert!(ready.metadata.diagnostics.any_query_collected);
        assert_eq!(ready.points.len(), 1);
        assert_eq!(opens.load(Ordering::Acquire), 3);
        assert!(
            worker
                .shutdown(Instant::now() + Duration::from_secs(1))
                .await
                .unwrap()
        );
        assert!(dropped.load(Ordering::Acquire));
    }
}

/// Scenario: A query starts with an unavailable exact counter that appears on a later retry.
/// Guarantees: Empty availability is pending rather than a successful native collection, then the installed Sum uses the original query start.
#[test]
fn late_counter_retry_preserves_logical_query_start() {
    let path = r"\Provider\Late Value";
    let state = fake_state(Vec::new());
    let factory_state = Rc::clone(&state);
    let mut open = move |configs, limit, seed: Option<&QueryRecreationSeed>| {
        Query::open_with_seed(
            FakePdh::new(Rc::clone(&factory_state)),
            configs,
            Duration::ZERO,
            Duration::from_secs(60),
            256,
            limit,
            seed,
        )
    };
    let mut slot = open_initial_query(
        vec![cumulative_sum(path, "late.value", "1")],
        Duration::ZERO,
        4096,
        &mut open,
        &AtomicBool::new(false),
    )
    .unwrap()
    .unwrap();
    let pending = collect_query(&mut slot, &mut open, Instant::now()).unwrap();
    assert!(pending.metadata.diagnostics.query_pending);
    assert!(!pending.metadata.diagnostics.any_query_collected);
    assert_eq!(state.borrow().calls.collect, 0);
    {
        let mut state = state.borrow_mut();
        state.counters.push(fake_counter(path, Number::Integer(42)));
        state.timestamp = 110;
    }
    let ready = collect_query(&mut slot, &mut open, Instant::now()).unwrap();
    assert_eq!(ready.points[0].start_time_unix_nano, Some(100));
    assert!(ready.metadata.diagnostics.any_query_collected);
    assert!(!ready.metadata.diagnostics.query_pending);
    assert_eq!(ready.metadata.diagnostics.retry_recoveries, 1);
}

/// Scenario: Wildcard membership appears and disappears after a single slot becomes ready.
/// Guarantees: Scheduled refresh adds and removes handles without reopening the query, bounded admission diagnoses overflow, and empty membership stays pending.
#[test]
fn dynamic_slot_refreshes_membership_with_bounded_admission() {
    let template = r"\Process(*)\Private Bytes";
    let first = r"\Process(first)\Private Bytes";
    let second = r"\Process(second)\Private Bytes";
    let state = fake_state(vec![
        fake_counter(first, Number::Integer(10)),
        fake_counter(second, Number::Integer(20)),
    ]);
    let factory_state = Rc::clone(&state);
    let opens = Cell::new(0);
    let mut open = |configs, limit, seed: Option<&QueryRecreationSeed>| {
        opens.set(opens.get() + 1);
        Query::open_with_seed(
            FakePdh::new(Rc::clone(&factory_state)),
            configs,
            Duration::ZERO,
            Duration::ZERO,
            1,
            limit,
            seed,
        )
    };
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut slot = open_initial_query(
        vec![config],
        Duration::ZERO,
        1,
        &mut open,
        &AtomicBool::new(false),
    )
    .unwrap()
    .unwrap();
    assert_eq!(slot.query.as_ref().unwrap().active_counter_count(), 0);
    let _ = state.borrow_mut().expansions.insert(
        template.to_owned(),
        vec![first.to_owned(), second.to_owned()],
    );
    let admitted = collect_query(&mut slot, &mut open, Instant::now()).unwrap();
    assert_eq!(admitted.points.len(), 1);
    assert_eq!(admitted.points[0].path.as_ref(), first);
    assert_eq!(admitted.metadata.diagnostics.instances_added, 1);
    assert_eq!(
        admitted.metadata.diagnostics.instances_omitted_over_limit,
        1
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), Vec::new());
    let removed = collect_query(&mut slot, &mut open, Instant::now()).unwrap();
    assert!(removed.points.is_empty());
    assert_eq!(removed.metadata.diagnostics.instances_removed, 1);
    assert_eq!(removed.metadata.diagnostics.active_expanded_counters, 0);
    assert!(removed.metadata.diagnostics.query_pending);
    assert_eq!(opens.get(), 1);
    assert_eq!(state.borrow().removed.len(), 1);
}

struct StartupTestPdh<const TIMESTAMP_FAILS: bool> {
    dropped: Arc<AtomicBool>,
}

impl<const TIMESTAMP_FAILS: bool> Drop for StartupTestPdh<TIMESTAMP_FAILS> {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}

impl<const TIMESTAMP_FAILS: bool> PdhApi for StartupTestPdh<TIMESTAMP_FAILS> {
    type Counter = ();

    fn add(&mut self, _path: &str, _language: CounterLanguage) -> Result<Self::Counter, Error> {
        Ok(())
    }

    fn remove(&mut self, _counter: &Self::Counter, _path: &str) -> Result<(), Error> {
        Ok(())
    }

    fn counter_type(&mut self, _counter: &Self::Counter, _path: &str) -> Result<u32, Error> {
        Ok(0x0001_0100)
    }

    fn full_path(&mut self, _counter: &Self::Counter, path: &str) -> Result<String, Error> {
        Ok(path.to_owned())
    }

    fn comparison_path(&mut self, path: &str) -> Result<String, Error> {
        NativePdh::comparison_path_fixture(path)
    }

    fn collect(&mut self) -> Result<bool, Error> {
        Ok(true)
    }

    fn raw(&mut self, _counter: &Self::Counter, _path: &str) -> Result<Option<RawSample>, Error> {
        Ok(None)
    }

    fn formatted(
        &mut self,
        _counter: &Self::Counter,
        _path: &str,
        _format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        Ok(Some(Number::Integer(42)))
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        if TIMESTAMP_FAILS {
            Self::current_timestamp()
        } else {
            Ok(1)
        }
    }

    fn current_timestamp() -> Result<i64, Error> {
        if TIMESTAMP_FAILS {
            Err(Error::Timestamp("test timestamp failure"))
        } else {
            NativePdh::current_timestamp()
        }
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

/// Scenario: Reading the clock fails during initial query setup or a scheduled no-query maintenance attempt.
/// Guarantees: Initial timestamp failure is fatal, while scheduled failure remains a counted collection error rather than an invalid maintenance sample.
#[test]
fn timestamp_failure_is_fatal_at_startup_and_counted_during_maintenance() {
    let configs = vec![counter(r"\Memory\Available Bytes", "memory")];
    let mut initial_open = |configs, limit, seed: Option<&QueryRecreationSeed>| {
        Query::open_with_seed(
            StartupTestPdh::<true> {
                dropped: Arc::new(AtomicBool::new(false)),
            },
            configs,
            Duration::ZERO,
            Duration::from_secs(60),
            256,
            limit,
            seed,
        )
    };
    assert!(matches!(
        QuerySlot::open_initial(configs.clone(), Duration::ZERO, 4_096, &mut initial_open),
        Err(Error::Timestamp("test timestamp failure"))
    ));

    let mut failed_open = |_configs, _limit, _seed: Option<&QueryRecreationSeed>| {
        Err::<Query<StartupTestPdh<true>>, _>(Error::Pdh {
            operation: PdhOperation::OpenQuery,
            path: "<query>".to_owned(),
            status: 1,
        })
    };
    let mut slot =
        QuerySlot::open_initial(configs, Duration::ZERO, 4_096, &mut failed_open).unwrap();
    let error = collect_query(&mut slot, &mut failed_open, Instant::now()).unwrap_err();
    assert!(matches!(error, Error::Timestamp("test timestamp failure")));
    assert!(error.is_collection_failure());
    assert!(slot.query.is_none());
}

/// Scenario: A worker query factory is invoked repeatedly for initial and replacement queries.
/// Guarantees: Successful native worker initialization runs once, while a failed attempt remains retryable.
#[test]
fn worker_initialization_is_armed_only_after_success() {
    let mut initialized = false;
    let attempts = Cell::new(0);
    assert!(
        initialize_once(&mut initialized, || {
            attempts.set(attempts.get() + 1);
            Err(Error::InvalidSample("startup failed"))
        })
        .is_err()
    );
    assert!(!initialized);
    initialize_once(&mut initialized, || {
        attempts.set(attempts.get() + 1);
        Ok(())
    })
    .unwrap();
    initialize_once(&mut initialized, || {
        attempts.set(attempts.get() + 1);
        Ok(())
    })
    .unwrap();
    assert!(initialized);
    assert_eq!(attempts.get(), 2);
}

struct RecoveringRefreshPdh {
    add_calls: Arc<std::sync::atomic::AtomicUsize>,
    timestamp: i64,
}

impl PdhApi for RecoveringRefreshPdh {
    type Counter = String;

    fn add(&mut self, path: &str, _language: CounterLanguage) -> Result<Self::Counter, Error> {
        if self.add_calls.fetch_add(1, Ordering::AcqRel) == 0 {
            return Err(Error::CounterSetup {
                path: path.to_owned(),
                message: "injected cleanup failure".to_owned(),
                permanent: false,
                cleanup_failed: true,
            });
        }
        Ok(path.to_owned())
    }

    fn remove(&mut self, _counter: &Self::Counter, _path: &str) -> Result<(), Error> {
        Ok(())
    }

    fn counter_type(&mut self, _counter: &Self::Counter, _path: &str) -> Result<u32, Error> {
        Ok(0x0001_0100)
    }

    fn full_path(&mut self, counter: &Self::Counter, _path: &str) -> Result<String, Error> {
        Ok(counter.clone())
    }

    fn comparison_path(&mut self, path: &str) -> Result<String, Error> {
        NativePdh::comparison_path_fixture(path)
    }

    fn collect(&mut self) -> Result<bool, Error> {
        self.timestamp += 10;
        Ok(true)
    }

    fn raw(&mut self, _counter: &Self::Counter, _path: &str) -> Result<Option<RawSample>, Error> {
        Ok(None)
    }

    fn formatted(
        &mut self,
        _counter: &Self::Counter,
        _path: &str,
        _format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        Ok(Some(Number::Integer(42)))
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        Ok(self.timestamp)
    }

    fn refresh_objects(&mut self) -> Result<usize, Error> {
        Ok(0)
    }

    fn expand(&mut self, _path: &str, budget: &mut DiscoveryBudget) -> Result<Vec<String>, Error> {
        let path = r"\Process(worker)\Private Bytes";
        budget.reserve(path.len())?;
        Ok(vec![path.to_owned()])
    }

    fn parse_instance(&mut self, _path: &str) -> Result<InstanceIdentity, Error> {
        Ok(InstanceIdentity {
            name: "worker".to_owned(),
            parent: None,
            index: 0,
        })
    }
}

#[derive(Default)]
struct PeakOwnership {
    current: std::sync::atomic::AtomicUsize,
    peak: std::sync::atomic::AtomicUsize,
    expansions: std::sync::atomic::AtomicUsize,
    collect_calls: std::sync::atomic::AtomicUsize,
    collect_status: std::sync::atomic::AtomicU32,
    closed_queries: std::sync::atomic::AtomicUsize,
    collect_gate: std::sync::Mutex<Option<(oneshot::Sender<()>, mpsc::Receiver<()>)>>,
}

struct PeakPdh {
    ownership: Arc<PeakOwnership>,
    owned: usize,
    expansions: BTreeMap<String, Vec<String>>,
    shutdown: Option<Arc<AtomicBool>>,
    collect_gate: Option<(oneshot::Sender<()>, mpsc::Receiver<()>)>,
}

impl Drop for PeakPdh {
    fn drop(&mut self) {
        let _ = self
            .ownership
            .current
            .fetch_sub(self.owned, Ordering::AcqRel);
        let _ = self.ownership.closed_queries.fetch_add(1, Ordering::AcqRel);
    }
}

impl PdhApi for PeakPdh {
    type Counter = String;

    fn check_running(&self) -> Result<(), Error> {
        if self
            .shutdown
            .as_ref()
            .is_some_and(|shutdown| shutdown.load(Ordering::Acquire))
        {
            return Err(Error::WorkerStopRequested);
        }
        Ok(())
    }

    fn add(&mut self, path: &str, _language: CounterLanguage) -> Result<Self::Counter, Error> {
        self.owned += 1;
        let current = self.ownership.current.fetch_add(1, Ordering::AcqRel) + 1;
        let _ = self.ownership.peak.fetch_max(current, Ordering::AcqRel);
        Ok(path.to_owned())
    }

    fn remove(&mut self, _counter: &Self::Counter, _path: &str) -> Result<(), Error> {
        self.owned -= 1;
        let _ = self.ownership.current.fetch_sub(1, Ordering::AcqRel);
        Ok(())
    }

    fn counter_type(&mut self, _counter: &Self::Counter, _path: &str) -> Result<u32, Error> {
        Ok(0x0001_0100)
    }

    fn full_path(&mut self, counter: &Self::Counter, _path: &str) -> Result<String, Error> {
        Ok(counter.clone())
    }

    fn comparison_path(&mut self, path: &str) -> Result<String, Error> {
        NativePdh::comparison_path_fixture(path)
    }

    fn collect(&mut self) -> Result<bool, Error> {
        let _ = self.ownership.collect_calls.fetch_add(1, Ordering::AcqRel);
        let gate = self
            .collect_gate
            .take()
            .or_else(|| self.ownership.collect_gate.lock().unwrap().take());
        if let Some((entered, release)) = gate {
            entered.send(()).unwrap();
            release.recv_timeout(Duration::from_secs(2)).unwrap();
            self.check_running()?;
        }
        let status = self.ownership.collect_status.load(Ordering::Acquire);
        if status != 0 {
            return Err(Error::Pdh {
                operation: PdhOperation::CollectQueryData,
                path: "<query>".to_owned(),
                status,
            });
        }
        Ok(true)
    }

    fn raw(&mut self, _counter: &Self::Counter, _path: &str) -> Result<Option<RawSample>, Error> {
        Ok(None)
    }

    fn formatted(
        &mut self,
        _counter: &Self::Counter,
        _path: &str,
        _format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        Ok(Some(Number::Integer(42)))
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        Ok(100)
    }

    fn refresh_objects(&mut self) -> Result<usize, Error> {
        Ok(0)
    }

    fn expand(&mut self, path: &str, budget: &mut DiscoveryBudget) -> Result<Vec<String>, Error> {
        let _ = self.ownership.expansions.fetch_add(1, Ordering::AcqRel);
        let paths = self.expansions.get(path).map_or(&[][..], Vec::as_slice);
        budget.reserve(paths.iter().map(String::len).sum())?;
        Ok(paths.to_vec())
    }

    fn parse_instance(&mut self, path: &str) -> Result<InstanceIdentity, Error> {
        NativePdh::parse_instance_path(path)
    }
}

/// Scenario: A startup query has two transient collection failures before recovering.
/// Guarantees: Every scheduled request retries the same query and handles before reaching the rebuild threshold.
#[test]
fn transient_failures_retry_installed_handles_before_rebuild() {
    let path = r"\Memory\Available Bytes";
    let state = fake_state(vec![fake_counter(path, Number::Integer(42))]);
    let opens = Cell::new(0);
    let factory_state = Rc::clone(&state);
    let mut open = |configs, limit, _seed: Option<&QueryRecreationSeed>| {
        opens.set(opens.get() + 1);
        let query = Query::open_with_seed(
            FakePdh::new(Rc::clone(&factory_state)),
            configs,
            Duration::ZERO,
            Duration::ZERO,
            256,
            limit,
            None,
        )?;
        Ok(query)
    };
    let plans = vec![counter(path, "memory")];
    let mut queries = open_initial_query(
        plans,
        Duration::ZERO,
        4096,
        &mut open,
        &AtomicBool::new(false),
    )
    .unwrap()
    .unwrap();
    let adds = state.borrow().calls.add;
    state.borrow_mut().collect_error = true;
    for _ in 0..2 {
        assert!(collect_query(&mut queries, &mut open, Instant::now()).is_err());
    }
    state.borrow_mut().collect_error = false;
    let sample = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
    assert_eq!(opens.get(), 1);
    assert_eq!(state.borrow().calls.add, adds);
    assert!(state.borrow().removed.is_empty());
    assert_eq!(sample.points.len(), 1);
    assert!(sample.metadata.diagnostics.any_query_collected);
    assert_eq!(sample.metadata.diagnostics.query_rebuild_attempts, 0);
}

/// Scenario: Production startup preparation receives an invalid query during priming.
/// Guarantees: Startup becomes pending after one prime and schedules recovery instead of terminating.
#[test]
fn startup_invalid_prime_schedules_recovery() {
    let path = r"\Memory\Available Bytes";
    let state = fake_state(vec![fake_counter(path, Number::Integer(42))]);
    state.borrow_mut().collect_status =
        Some(windows_sys::Win32::System::Performance::PDH_INVALID_HANDLE);
    let factory_state = Rc::clone(&state);
    let mut open = |configs, limit, _seed: Option<&QueryRecreationSeed>| {
        let query = Query::open_with_seed(
            FakePdh::new(Rc::clone(&factory_state)),
            configs,
            Duration::ZERO,
            Duration::ZERO,
            256,
            limit,
            None,
        )?;
        Ok(query)
    };
    let result = open_initial_query(
        vec![counter(path, "memory")],
        Duration::ZERO,
        4096,
        &mut open,
        &AtomicBool::new(false),
    );
    assert!(result.unwrap().unwrap().query.is_none());
    assert_eq!(state.borrow().calls.collect, 1);
    assert_eq!(state.borrow().calls.add, 1);
}

/// Scenario: The only scheduled query fails collection and the following scrape occurs during its recovery backoff.
/// Guarantees: The worker marks the synthetic recovery-only sample as having no native query collection and no new scrape failure.
#[test]
fn recovery_only_sample_reports_no_query_collected() {
    let path = r"\Unavailable\Value";
    let state = fake_state(vec![fake_counter(path, Number::Integer(42))]);
    let factory_state = Rc::clone(&state);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_native_counters: usize,
                         seed: Option<&QueryRecreationSeed>| {
        Query::open_with_seed(
            FakePdh::new(Rc::clone(&factory_state)),
            configs,
            Duration::from_secs(30),
            Duration::from_secs(30),
            256,
            max_native_counters,
            seed,
        )
    };
    let plan = vec![counter(path, "windows.unavailable")];
    let mut query =
        QuerySlot::open_initial(plan, Duration::from_secs(30), 4_096, &mut open).unwrap();
    state.borrow_mut().collect_error = true;

    let failure = collect_query(&mut query, &mut open, Instant::now()).unwrap_err();
    assert!(matches!(
        failure,
        Error::Pdh {
            operation: PdhOperation::CollectQueryData,
            ..
        }
    ));

    let recovery_only = collect_query(&mut query, &mut open, Instant::now()).unwrap();
    assert!(recovery_only.points.is_empty());
    assert!(!recovery_only.metadata.diagnostics.any_query_collected);
    assert_eq!(recovery_only.metadata.failures.len(), 1);
    assert_eq!(
        recovery_only.metadata.failures[0].reason,
        "query_recovery_pending"
    );
    assert!(recovery_only.metadata.failures[0].diagnostic_only);
}

/// Scenario: A wildcard initial plan can fill the receiver budget before a later isolated exact plan opens.
/// Guarantees: Startup reserves every unopened exact mapping, so both plans open within the validated receiver-wide limit.
#[test]
fn initial_wildcard_reserves_later_exact_native_entry() {
    let wildcard_path = r"\Process(*)\Private Bytes";
    let expanded = (0..3)
        .map(|index| format!(r"\Process(worker{index})\Private Bytes"))
        .collect::<Vec<_>>();
    let mut wildcard = counter(wildcard_path, "process.private");
    wildcard.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut isolated = counter(r"\System\Processes", "system.processes");
    isolated.source_mapping_index = 1;

    let observed_limits = Rc::new(RefCell::new(Vec::new()));
    let factory_limits = Rc::clone(&observed_limits);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_native_counters: usize,
                         seed: Option<&QueryRecreationSeed>| {
        factory_limits.borrow_mut().push(max_native_counters);
        let counters = configs
            .iter()
            .flat_map(|config| {
                if config.instance_config.is_wildcard() {
                    expanded.clone()
                } else {
                    vec![config.path.clone()]
                }
            })
            .map(|path| fake_counter(&path, Number::Integer(42)))
            .collect();
        let state = Rc::new(RefCell::new(FakeState {
            expansions: [(wildcard_path.to_owned(), expanded.clone())]
                .into_iter()
                .collect(),
            ..fake_state_value(counters)
        }));
        Query::open_with_seed(
            FakePdh::new(state),
            configs,
            Duration::from_secs(30),
            Duration::from_secs(30),
            256,
            max_native_counters,
            seed,
        )
    };

    let queries = open_initial_query(
        vec![wildcard, isolated],
        Duration::from_secs(30),
        3,
        &mut open,
        &AtomicBool::new(false),
    )
    .unwrap()
    .unwrap();

    assert_eq!(observed_limits.borrow().as_slice(), [3]);
    assert_eq!(queries.active_expanded_count(), 2);
}

/// Scenario: A wildcard query reaches the rebuild threshold while its admitted member has not disappeared.
/// Guarantees: Replacement receives the retained seed and restores the admitted member within the same one-entry native budget.
#[test]
fn down_query_reserves_seeded_wildcard_membership() {
    let path = r"\Process(*)\Private Bytes";
    let concrete = r"\Process(worker)\Private Bytes";
    let mut config = counter(path, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let states = Rc::new(RefCell::new(Vec::new()));
    let factory_states = Rc::clone(&states);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_native_counters: usize,
                         seed: Option<&QueryRecreationSeed>| {
        assert_eq!(seed.is_some(), !factory_states.borrow().is_empty());
        assert_eq!(max_native_counters, 1);
        let state = Rc::new(RefCell::new(FakeState {
            expansions: [(path.to_owned(), vec![concrete.to_owned()])]
                .into_iter()
                .collect(),
            ..fake_state_value(vec![fake_counter(concrete, Number::Integer(42))])
        }));
        let query = Query::open_with_seed(
            FakePdh::new(Rc::clone(&state)),
            configs,
            Duration::from_secs(30),
            Duration::from_secs(30),
            256,
            max_native_counters,
            seed,
        )
        .unwrap();
        if seed.is_some() {
            assert_eq!(query.admitted_members().len(), 1);
            assert_eq!(query.admitted_members()[0].path.as_ref(), concrete);
            assert_eq!(state.borrow().calls.expand, 0);
            assert_eq!(state.borrow().added, [concrete.to_owned()]);
        }
        factory_states.borrow_mut().push(state);
        Ok(query)
    };
    let plan = vec![config];
    let mut slot = QuerySlot::open_initial(plan, Duration::ZERO, 1, &mut open).unwrap();
    states.borrow()[0].borrow_mut().collect_error = true;

    for _ in 0..QUERY_REBUILD_AFTER_FAILURES {
        let result = slot.collect(Instant::now(), &mut open);
        assert!(result.is_err_and(|error| error.is_query_collection_failure()));
    }

    assert!(slot.query.is_none());
    let rebuilt = collect_query(&mut slot, &mut open, Instant::now()).unwrap();
    assert_eq!(rebuilt.members.len(), 1);
    assert_eq!(rebuilt.members[0].path.as_ref(), concrete);
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_recoveries, 1);
    assert_eq!(states.borrow().len(), 2);
}

/// Scenario: A down wildcard query's rebuild attempt fails with an error that normally schedules an immediate rebuild.
/// Guarantees: Re-scheduling an already down query preserves its seed, native reservation, attempt count, and bounded backoff.
#[test]
fn failed_rebuild_preserves_down_query_state() {
    let path = r"\Process(*)\Private Bytes";
    let concrete = r"\Process(worker)\Private Bytes";
    let mut config = counter(path, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let state = Rc::new(RefCell::new(FakeState {
        expansions: [(path.to_owned(), vec![concrete.to_owned()])]
            .into_iter()
            .collect(),
        ..fake_state_value(vec![fake_counter(concrete, Number::Integer(42))])
    }));
    let initial_state = Rc::clone(&state);
    let mut open_initial = move |configs: Vec<CounterConfig>,
                                 max_native_counters: usize,
                                 seed: Option<&QueryRecreationSeed>| {
        Query::open_with_seed(
            FakePdh::new(Rc::clone(&initial_state)),
            configs,
            Duration::from_secs(30),
            Duration::from_secs(30),
            256,
            max_native_counters,
            seed,
        )
    };
    let plan = vec![config];
    let mut slot =
        QuerySlot::open_initial(plan, Duration::from_secs(30), 1, &mut open_initial).unwrap();
    state.borrow_mut().collect_error = true;
    let now = Instant::now();
    for offset in [0, 1, 3] {
        let result = slot.collect(now + Duration::from_secs(offset), &mut open_initial);
        assert!(result.is_err_and(|error| error.is_query_collection_failure()));
    }
    assert!(slot.query.is_none());

    let observed_seeds = RefCell::new(Vec::new());
    let mut fail_rebuild =
        |_: Vec<CounterConfig>, limit: usize, seed: Option<&QueryRecreationSeed>| {
            assert_eq!(limit, 1);
            observed_seeds.borrow_mut().push(seed.is_some());
            Err::<Query<FakePdh>, _>(Error::CounterSetup {
                path: concrete.to_owned(),
                message: "injected untracked setup ownership failure".to_owned(),
                permanent: false,
                cleanup_failed: true,
            })
        };
    let rebuild_at = now + Duration::from_secs(7);
    for attempt_at in [rebuild_at, rebuild_at + Duration::from_secs(1)] {
        let error = slot.collect(attempt_at, &mut fail_rebuild).unwrap_err();
        assert!(error.requires_immediate_query_rebuild());
    }
    let pending = slot
        .collect(rebuild_at + Duration::from_secs(2), &mut fail_rebuild)
        .unwrap_err();

    assert!(matches!(pending, Error::QueryRecoveryPending { .. }));
    assert_eq!(observed_seeds.borrow().as_slice(), [true, true]);
    assert!(slot.query.is_none());
}

/// Scenario: Two wildcard expansions repeatedly fail while an exact counter remains healthy.
/// Guarantees: Every wildcard is retried on the refresh cadence without query backoff or rebuild.
#[test]
fn repeated_refresh_failures_stay_on_refresh_cadence() {
    let exact_path = r"\Memory\Available Bytes";
    let first_path = r"\Process(*)\Private Bytes";
    let second_path = r"\Thread(*)\Thread State";
    let mut first = counter(first_path, "process.private");
    first.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut second = counter(second_path, "thread.state");
    second.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let open_calls = Rc::new(Cell::new(0_usize));
    let factory_open_calls = Rc::clone(&open_calls);
    let states = Rc::new(RefCell::new(Vec::new()));
    let factory_states = Rc::clone(&states);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_native_counters: usize,
                         seed: Option<&QueryRecreationSeed>| {
        factory_open_calls.set(factory_open_calls.get() + 1);
        let (state, query) = fake_query_with_seed(
            configs,
            vec![FakeCounter {
                path: exact_path.to_owned(),
                native_type: 0x0001_0100,
                raw: None,
                multi_count: 1,
                formatted: Some(Number::Integer(42)),
                raw_error: false,
                formatted_error: false,
            }],
            Duration::ZERO,
            Duration::ZERO,
            max_native_counters,
            seed,
        );
        factory_states.borrow_mut().push(state);
        Ok(query)
    };
    let plan = vec![counter(exact_path, "memory.available"), first, second];
    let mut slot = QuerySlot::open_initial(plan, Duration::ZERO, 1, &mut open).unwrap();
    let startup = collect_query(&mut slot, &mut open, Instant::now()).unwrap();
    assert_eq!(
        startup
            .metadata
            .failures
            .iter()
            .filter(|failure| failure.reason == "empty_expansion")
            .count(),
        2
    );
    states.borrow()[0].borrow_mut().expand_error = true;
    let initial_expand_calls = states.borrow()[0].borrow().calls.expand;
    let mut queries = slot;
    for attempt in 0..QUERY_REBUILD_AFTER_FAILURES {
        let sample = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].counter_index, 0);
        assert_eq!(sample.metadata.diagnostics.discovery_failures, 2);
        assert_eq!(sample.metadata.diagnostics.query_rebuild_attempts, 0);
        assert_eq!(
            sample.metadata.failures.len(),
            if attempt == 0 { 2 } else { 0 }
        );
        assert!(
            sample.metadata.failures.iter().all(|failure| {
                failure.reason == "selector_unresolved" && failure.diagnostic_only
            })
        );
    }

    assert!(queries.query.is_some());
    assert_eq!(
        states.borrow()[0].borrow().calls.expand,
        initial_expand_calls + 2 * QUERY_REBUILD_AFTER_FAILURES as usize
    );
    assert_eq!(open_calls.get(), 1);
}

/// Scenario: A single wildcard query enters collection backoff with one admitted member.
/// Guarantees: Recovery-only samples preserve the active expanded-counter gauge.
#[test]
fn query_backoff_preserves_active_expanded_count() {
    let path = r"\Process(*)\Private Bytes";
    let concrete = r"\Process(worker)\Private Bytes";
    let mut config = counter(path, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let states = Rc::new(RefCell::new(Vec::new()));
    let factory_states = Rc::clone(&states);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_native_counters: usize,
                         seed: Option<&QueryRecreationSeed>| {
        let (state, query) = fake_query_with_seed(
            configs,
            vec![FakeCounter {
                path: concrete.to_owned(),
                native_type: 0x0001_0100,
                raw: None,
                multi_count: 1,
                formatted: Some(Number::Integer(42)),
                raw_error: false,
                formatted_error: false,
            }],
            Duration::ZERO,
            Duration::ZERO,
            max_native_counters,
            seed,
        );
        factory_states.borrow_mut().push(state);
        Ok(query)
    };
    let plan = vec![config];
    let slot = QuerySlot::open_initial(plan, Duration::from_secs(30), 1, &mut open).unwrap();
    let mut queries = slot;
    let _ = states.borrow()[0]
        .borrow_mut()
        .expansions
        .insert(path.to_owned(), vec![concrete.to_owned()]);

    let healthy = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
    assert_eq!(healthy.metadata.diagnostics.active_expanded_counters, 1);
    states.borrow()[0].borrow_mut().collect_error = true;
    let now = Instant::now();
    let failure = queries.collect(now, &mut open);
    assert!(failure.is_err_and(|error| error.is_query_collection_failure()));
    let native_collects = states.borrow()[0].borrow().calls.collect;
    let pending = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
    assert!(pending.points.is_empty());
    assert!(!pending.metadata.diagnostics.any_query_collected);
    assert_eq!(states.borrow()[0].borrow().calls.collect, native_collects);
    let mut metrics = super::super::super::metrics::WindowsPerfCountersMetrics::default();
    metrics.active_expanded.set(1);
    metrics.apply(&pending.metadata.diagnostics);
    assert_eq!(metrics.active_expanded.get(), 1);
    for attempt_at in [now + Duration::from_secs(1), now + Duration::from_secs(3)] {
        let failure = queries.collect(attempt_at, &mut open);
        assert!(failure.is_err_and(|error| error.is_query_collection_failure()));
    }
    assert!(queries.query.is_none());
    assert_eq!(queries.active_expanded_count(), 0);

    let pending = collect_query(&mut queries, &mut open, Instant::now()).unwrap();

    assert!(pending.points.is_empty());
    assert_eq!(pending.metadata.diagnostics.active_expanded_counters, 0);
}

/// Scenario: One query combines an exact mapping with a wildcard that discovers three paths under a two-entry limit.
/// Guarantees: Exact ownership reduces same-query expansion capacity so wildcard discovery cannot overcommit the receiver budget.
#[test]
fn exact_reservation_reduces_same_query_expansion_capacity() {
    let exact_path = r"\Memory\Available Bytes";
    let wildcard_path = r"\Process(*)\Private Bytes";
    let expanded = (0..3)
        .map(|index| format!(r"\Process(worker{index})\Private Bytes"))
        .collect::<Vec<_>>();
    let counters = std::iter::once(exact_path.to_owned())
        .chain(expanded.iter().cloned())
        .map(|path| fake_counter(&path, Number::Integer(42)))
        .collect();
    let state = Rc::new(RefCell::new(FakeState {
        expansions: [(wildcard_path.to_owned(), expanded)].into_iter().collect(),
        ..fake_state_value(counters)
    }));
    let mut wildcard = counter(wildcard_path, "process.private");
    wildcard.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };

    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        vec![counter(exact_path, "memory.available"), wildcard],
        Duration::from_secs(30),
        Duration::from_secs(30),
        256,
        2,
        None,
    )
    .unwrap();

    query.prime().unwrap();
    assert_eq!(query.active_expanded_count(), 1);
    assert_eq!(state.borrow().added.len(), 2);
}

/// Scenario: A worker allocation supplies fewer entries than one query's exact configurations require.
/// Guarantees: Internal accounting corruption fails explicitly before opening native handles instead of silently exceeding the receiver limit.
#[test]
fn query_rejects_limit_below_exact_reservation() {
    let result = Query::open_with_seed(
        StartupTestPdh::<false> {
            dropped: Arc::new(AtomicBool::new(false)),
        },
        vec![
            counter(r"\Memory\Available Bytes", "memory.available"),
            counter(r"\System\Processes", "system.processes"),
        ],
        Duration::from_secs(30),
        Duration::from_secs(30),
        256,
        1,
        None,
    );

    assert!(matches!(
        result,
        Err(Error::InvalidSample(
            "query native-counter limit is below its exact-counter reservation"
        ))
    ));
}

/// Scenario: A cumulative Sum applies a nonzero decimal scale before OTAP delivery.
/// Guarantees: Both scaled observations retain the fixed query start and project successfully.
#[test]
fn scaled_cumulative_sum_retains_start() {
    let path = r"\TCPv4\Connection Failures";
    let mut config = cumulative_sum(path, "windows.tcp.connection_failures", "{failure}");
    config.scale_power10 = -1;
    let projection_config = config.clone();
    let states = Rc::new(RefCell::new(Vec::new()));
    let factory_states = Rc::clone(&states);
    let mut open = move |configs: Vec<CounterConfig>,
                         _max_native_counters: usize,
                         _seed: Option<&QueryRecreationSeed>| {
        let (state, query) = fake_query(
            configs,
            vec![FakeCounter {
                path: path.to_owned(),
                native_type: 0x0001_0100,
                raw: None,
                multi_count: 1,
                formatted: Some(Number::Integer(10)),
                raw_error: false,
                formatted_error: false,
            }],
        );
        factory_states.borrow_mut().push(state);
        Ok(query)
    };
    let plan = vec![config];
    let mut queries =
        QuerySlot::open_initial(plan, Duration::from_secs(30), 4_096, &mut open).unwrap();

    {
        let states = states.borrow();
        let mut state = states[0].borrow_mut();
        state.timestamp = 110;
        state.counters[0].formatted = Some(Number::Integer(10));
    }
    let first = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
    assert_eq!(first.points[0].start_time_unix_nano, Some(100));
    let projection = project_sample(std::slice::from_ref(&projection_config), &first).unwrap();
    assert!(projection.is_some());

    {
        let states = states.borrow();
        let mut state = states[0].borrow_mut();
        state.timestamp = 120;
        state.counters[0].formatted = Some(Number::Integer(20));
    }
    let second = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
    assert_eq!(second.points[0].start_time_unix_nano, Some(100));
    let projection = project_sample(std::slice::from_ref(&projection_config), &second).unwrap();
    assert!(projection.is_some());
}

/// Scenario: A cumulative Sum query is rebuilt after repeated collection failures and primed successfully.
/// Guarantees: Rebuild priming preserves the original query start through the recreation seed.
#[test]
fn rebuilt_cumulative_query_preserves_start() {
    let path = r"\TCPv4\Connection Failures";
    let states = Rc::new(RefCell::new(Vec::new()));
    let factory_states = Rc::clone(&states);
    let open_calls = Rc::new(Cell::new(0_usize));
    let factory_open_calls = Rc::clone(&open_calls);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_native_counters: usize,
                         seed: Option<&QueryRecreationSeed>| {
        let open_index = factory_open_calls.get();
        factory_open_calls.set(open_index + 1);
        let offset = i64::try_from(open_index).unwrap() * 10;
        let (state, query) = fake_query_with_seed(
            configs,
            vec![FakeCounter {
                path: path.to_owned(),
                native_type: 0x0001_0100,
                raw: None,
                multi_count: 1,
                formatted: Some(Number::Integer(10 + offset)),
                raw_error: false,
                formatted_error: false,
            }],
            Duration::from_secs(30),
            Duration::from_secs(30),
            max_native_counters,
            seed,
        );
        state.borrow_mut().timestamp = 100 + offset;
        factory_states.borrow_mut().push(state);
        Ok(query)
    };
    let config = cumulative_sum(path, "windows.tcp.connection_failures", "{failure}");
    let plan = vec![config.clone()];
    let mut queries = QuerySlot::open_initial(plan, Duration::ZERO, 4_096, &mut open).unwrap();
    {
        let states = states.borrow();
        let mut state = states[0].borrow_mut();
        state.timestamp = 110;
        state.counters[0].formatted = Some(Number::Integer(20));
    }
    let first = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
    assert_eq!(first.points[0].start_time_unix_nano, Some(100));
    let projection = project_sample(std::slice::from_ref(&config), &first).unwrap();
    assert!(projection.is_some());

    states.borrow()[0].borrow_mut().collect_error = true;
    let now = Instant::now();
    for _ in 0..QUERY_REBUILD_AFTER_FAILURES {
        assert!(queries.collect(now, &mut open).is_err());
    }

    let rebuilt = queries.collect(now, &mut open).unwrap();
    assert!(rebuilt.points.is_empty());
    {
        let states = states.borrow();
        let mut state = states[1].borrow_mut();
        state.timestamp = 120;
        state.counters[0].formatted = Some(Number::Integer(30));
    }
    let recovered = queries.collect(now, &mut open).unwrap();
    assert_eq!(recovered.points[0].timestamp_unix_nano, 120);
    assert_eq!(recovered.points[0].start_time_unix_nano, Some(100));
    assert_eq!(open_calls.get(), 2);
}

/// Scenario: One initial native open remains blocked while virtual time advances beyond the former deadline.
/// Guarantees: Slow provider work cannot become a fatal timeout; readiness waits for the completed initial pass.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn startup_longer_than_thirty_seconds_eventually_succeeds() {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let dropped = Arc::new(AtomicBool::new(false));
    let factory_dropped = Arc::clone(&dropped);
    let startup = Worker::start_with(
        vec![counter(
            r"\Memory\Available Bytes",
            "windows.memory.available",
        )],
        Duration::from_secs(30),
        4_096,
        move |configs, max_expanded_counters, _seed| {
            let _ = entered_tx.send(());
            let _ = release_rx.recv();
            Query::open_with_seed(
                StartupTestPdh::<false> {
                    dropped: Arc::clone(&factory_dropped),
                },
                configs,
                Duration::from_secs(30),
                Duration::from_secs(30),
                256,
                max_expanded_counters,
                None,
            )
        },
    );
    tokio::pin!(startup);
    tokio::select! {
        biased;
        _ = &mut startup => panic!("startup completed before the blocked native call"),
        () = tokio::task::yield_now() => {}
    }
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("worker did not begin initial query open");

    tokio::time::advance(Duration::from_secs(31)).await;
    tokio::select! {
        biased;
        _ = &mut startup => panic!("startup completed before the blocked native call"),
        () = tokio::task::yield_now() => {}
    }

    release_tx.send(()).unwrap();
    let worker = startup.await.unwrap();
    tokio::time::resume();
    drop(worker);
    tokio::time::timeout(Duration::from_secs(1), async {
        while !dropped.load(Ordering::Acquire) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("slow startup worker did not release its query");
}

/// Scenario: The initial wildcard refresh hits a recoverable native cleanup failure before succeeding after the configured retry cap.
/// Guarantees: Startup remains ready, the backoff scrape reports diagnostics without failing, and recovery waits for the bounded retry.
#[tokio::test(flavor = "current_thread")]
async fn startup_wildcard_refresh_failure_retries_after_cap() {
    let template = r"\Process(*)\Private Bytes";
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let add_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let factory_add_calls = Arc::clone(&add_calls);
    let retry_cap = Duration::from_secs(1);
    let mut worker = Worker::start_with(
        vec![config],
        retry_cap,
        4_096,
        move |configs, max_native_counters, seed| {
            Query::open_with_seed(
                RecoveringRefreshPdh {
                    add_calls: Arc::clone(&factory_add_calls),
                    timestamp: 100,
                },
                configs,
                retry_cap,
                Duration::ZERO,
                256,
                max_native_counters,
                seed,
            )
        },
    )
    .await
    .unwrap();

    let deferred = worker.collect().await.unwrap();
    assert!(deferred.points.is_empty());
    assert!(
        deferred
            .metadata
            .failures
            .iter()
            .any(|failure| failure.reason == "counter_setup")
    );
    assert!(
        deferred
            .metadata
            .failures
            .iter()
            .any(|failure| failure.reason == "query_recovery_pending" && failure.diagnostic_only)
    );
    assert_eq!(add_calls.load(Ordering::Acquire), 1);

    tokio::time::sleep(retry_cap + Duration::from_millis(50)).await;
    let rebuilt = worker.collect().await.unwrap();
    let recovered = worker.collect().await.unwrap();

    assert!(rebuilt.points.is_empty());
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_attempts, 1);
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_recoveries, 1);
    assert!(rebuilt.metadata.failures.is_empty());
    assert_eq!(add_calls.load(Ordering::Acquire), 2);
    assert_eq!(recovered.points.len(), 1);
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(1))
            .await
            .unwrap()
    );
}

/// Scenario: Control abandons startup while the first of multiple native initialization units is blocked.
/// Guarantees: Cancellation requests cleanup and prevents later units from opening, avoiding overlapping orphan work.
#[tokio::test(flavor = "current_thread")]
async fn cancelling_startup_stops_between_initialization_units() {
    let mut isolated = counter(r"\System\Processes", "windows.system.processes");
    isolated.source_mapping_index = 1;
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let open_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let factory_open_calls = Arc::clone(&open_calls);
    let dropped = Arc::new(AtomicBool::new(false));
    let factory_dropped = Arc::clone(&dropped);
    let mut startup = Box::pin(Worker::start_with(
        vec![
            counter(r"\Memory\Available Bytes", "windows.memory.available"),
            isolated,
        ],
        Duration::from_secs(30),
        4_096,
        move |configs, max_expanded_counters, _seed| {
            let call = factory_open_calls.fetch_add(1, Ordering::AcqRel);
            if call == 0 {
                let _ = entered_tx.send(());
                let _ = release_rx.recv();
            }
            Query::open_with_seed(
                StartupTestPdh::<false> {
                    dropped: Arc::clone(&factory_dropped),
                },
                configs,
                Duration::from_secs(30),
                Duration::from_secs(30),
                256,
                max_expanded_counters,
                None,
            )
        },
    ));
    tokio::select! {
        biased;
        _ = &mut startup => panic!("startup completed before cancellation"),
        () = tokio::task::yield_now() => {}
    }
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("worker did not begin initial query open");

    drop(startup);
    release_tx.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(1), async {
        while !dropped.load(Ordering::Acquire) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("cancelled startup did not release its query");
    assert_eq!(open_calls.load(Ordering::Acquire), 1);
}

/// Scenario: One initial query has multiple wildcard mappings and cancellation arrives during the first expansion.
/// Guarantees: Startup does not expand the next mapping or prime the partially prepared query.
#[test]
fn cancelling_startup_stops_between_expansion_and_priming() {
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let state = fake_state(Vec::new());
    state.borrow_mut().stop_after_expand = Some(Arc::clone(&shutdown_requested));
    let mut counters = vec![
        counter(r"\Process(*)\Private Bytes", "process.private"),
        counter(r"\Thread(*)\% Processor Time", "thread.cpu.time"),
    ];
    for config in &mut counters {
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
    }
    let factory_state = Rc::clone(&state);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_native_counters: usize,
                         seed: Option<&QueryRecreationSeed>| {
        Query::open_with_seed(
            FakePdh::new(Rc::clone(&factory_state)),
            configs,
            Duration::from_secs(30),
            Duration::from_secs(30),
            256,
            max_native_counters,
            seed,
        )
    };

    let queries = open_initial_query(
        counters,
        Duration::from_secs(30),
        4_096,
        &mut open,
        &shutdown_requested,
    )
    .unwrap();

    assert!(queries.is_none());
    assert_eq!(state.borrow().calls.expand, 1);
    assert_eq!(state.borrow().calls.collect, 0);
}

/// Scenario: Stop arrives while the first exact counter is being added during initial query setup.
/// Guarantees: Setup performs no metadata read, second add, or priming collection after that call returns.
#[test]
fn cancelling_startup_stops_between_exact_native_operations() {
    let first = r"\Memory\Available Bytes";
    let second = r"\System\Processes";
    for configs in [
        vec![counter(first, "memory")],
        vec![counter(first, "memory"), counter(second, "processes")],
    ] {
        let state = fake_state(vec![
            fake_counter(first, Number::Integer(42)),
            fake_counter(second, Number::Integer(2)),
        ]);
        state.borrow_mut().stop_after_add = Some(Arc::new(AtomicBool::new(false)));
        let result = Query::open_with_seed(
            FakePdh::new(Rc::clone(&state)),
            configs,
            Duration::from_secs(30),
            Duration::from_secs(30),
            256,
            4_096,
            None,
        );
        assert!(matches!(result, Err(Error::WorkerStopRequested)));
        let state = state.borrow();
        assert_eq!(state.added, [first]);
        assert_eq!(state.calls.counter_type, 0);
        assert_eq!(state.calls.collect, 0);
    }
}

/// Scenario: Deferred cleanup of a departed member or still-wanted replacement fails after repeated production refresh.
/// Guarantees: Cleanup retains its reservation and failure context, backs off without reopening the query, and restores wanted members after removal succeeds.
#[test]
fn failed_pending_cleanup_remains_bounded_without_query_escalation() {
    let template = r"\Process(*)\Private Bytes";
    let expanded = r"\Process(worker)\Private Bytes";
    let duplicate = r"\Process(worker#1)\Private Bytes";
    for still_wanted in [false, true] {
        let mut config = counter(template, "private");
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        let paths = if still_wanted {
            vec![expanded.to_owned(), duplicate.to_owned()]
        } else {
            vec![expanded.to_owned()]
        };
        let state = fake_state(
            paths
                .iter()
                .map(|path| fake_counter(path, Number::Integer(1)))
                .collect(),
        );
        let _ = state
            .borrow_mut()
            .expansions
            .insert(template.to_owned(), paths);
        let opens = Cell::new(0);
        let mut open = |configs, limit, seed: Option<&QueryRecreationSeed>| {
            opens.set(opens.get() + 1);
            Query::open_with_seed(
                FakePdh::new(Rc::clone(&state)),
                configs,
                Duration::from_secs(60),
                Duration::ZERO,
                256,
                limit,
                seed,
            )
        };
        let limit = if still_wanted { 2 } else { 1 };
        let mut slot = open_initial_query(
            vec![config],
            Duration::from_secs(60),
            limit,
            &mut open,
            &AtomicBool::new(false),
        )
        .unwrap()
        .unwrap();
        {
            let mut state = state.borrow_mut();
            let _ = state.expansions.insert(
                template.to_owned(),
                if still_wanted {
                    vec![expanded.to_owned()]
                } else {
                    Vec::new()
                },
            );
            let _ = state.remove_errors.insert(0);
        }
        let now = Instant::now();
        let pending = collect_query(&mut slot, &mut open, now).unwrap();
        assert_eq!(pending.metadata.diagnostics.discovery_failures, 0);
        assert_eq!(
            state
                .borrow()
                .removed
                .iter()
                .filter(|index| **index == 0)
                .count(),
            1
        );
        let failed =
            collect_query(&mut slot, &mut open, now + Duration::from_millis(1100)).unwrap();
        assert!(!failed.metadata.diagnostics.any_query_collected);
        assert!(failed.metadata.diagnostics.query_pending);
        assert_eq!(failed.metadata.diagnostics.discovery_failures, 0);
        assert!(
            failed
                .metadata
                .failures
                .iter()
                .any(|failure| failure.reason == "PdhRemoveCounter")
        );
        assert!(slot.query.is_some());
        assert_eq!(
            state
                .borrow()
                .removed
                .iter()
                .filter(|index| **index == 0)
                .count(),
            2
        );
        let backoff =
            collect_query(&mut slot, &mut open, now + Duration::from_millis(1200)).unwrap();
        assert_eq!(backoff.metadata.diagnostics.query_rebuild_attempts, 0);
        assert!(!backoff.metadata.diagnostics.any_query_collected);
        assert_eq!(opens.get(), 1);
        state.borrow_mut().remove_errors.clear();
        let cleaned =
            collect_query(&mut slot, &mut open, now + Duration::from_millis(3200)).unwrap();
        assert_eq!(cleaned.metadata.diagnostics.query_rebuild_attempts, 0);
        assert_eq!(cleaned.points.len(), usize::from(still_wanted));
        assert_eq!(opens.get(), 1);
        assert_eq!(
            slot.query.as_ref().unwrap().admitted_members().len(),
            usize::from(still_wanted)
        );
    }
}

/// Scenario: A primed worker receives PDH_INVALID_HANDLE from its first scheduled native collection.
/// Guarantees: The typed failure closes old handles; bounded rebuild primes a replacement and the next collection publishes.
#[tokio::test(flavor = "current_thread")]
async fn invalid_query_rebuilds_worker_and_closes_old_handles() {
    let ownership = Arc::new(PeakOwnership::default());
    let factory_ownership = Arc::clone(&ownership);
    let opens = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let factory_opens = Arc::clone(&opens);
    let mut worker = Worker::start_with(
        vec![counter(r"\Memory\Available Bytes", "memory")],
        Duration::ZERO,
        4_096,
        move |configs, limit, _seed| {
            let _ = factory_opens.fetch_add(1, Ordering::AcqRel);
            let query = Query::open_with_seed(
                PeakPdh {
                    ownership: Arc::clone(&factory_ownership),
                    owned: 0,
                    expansions: BTreeMap::new(),
                    shutdown: None,
                    collect_gate: None,
                },
                configs,
                Duration::ZERO,
                Duration::ZERO,
                256,
                limit,
                None,
            )?;
            Ok(query)
        },
    )
    .await
    .unwrap();
    assert_eq!(ownership.current.load(Ordering::Acquire), 1);
    assert_eq!(ownership.collect_calls.load(Ordering::Acquire), 1);
    ownership.collect_status.store(
        windows_sys::Win32::System::Performance::PDH_INVALID_HANDLE,
        Ordering::Release,
    );

    assert!(matches!(
        worker.collect().await,
        Err(Error::Pdh {
            operation: PdhOperation::CollectQueryData,
            status: windows_sys::Win32::System::Performance::PDH_INVALID_HANDLE,
            ..
        })
    ));
    assert_eq!(ownership.current.load(Ordering::Acquire), 0);
    assert_eq!(ownership.closed_queries.load(Ordering::Acquire), 1);
    ownership.collect_status.store(0, Ordering::Release);
    let rebuilt = worker.collect().await.unwrap();
    assert!(rebuilt.points.is_empty());
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_recoveries, 1);
    let recovered = worker.collect().await.unwrap();
    assert_eq!(recovered.points.len(), 1);
    assert_eq!(ownership.current.load(Ordering::Acquire), 1);
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(1))
            .await
            .unwrap()
    );
    assert_eq!(ownership.current.load(Ordering::Acquire), 0);
    assert_eq!(ownership.closed_queries.load(Ordering::Acquire), 2);
    assert_eq!(ownership.collect_calls.load(Ordering::Acquire), 4);
    assert_eq!(opens.load(Ordering::Acquire), 2);
}

/// Scenario: Shutdown arrives during retry backoff, a returning rebuild open, or replacement priming.
/// Guarantees: Backoff exits immediately, returning native work observes stop, and every owned query and counter closes without publication.
#[tokio::test(flavor = "current_thread")]
async fn shutdown_during_recovery_closes_all_handles() {
    for stage in ["retry_backoff", "rebuild_open", "rebuild_prime"] {
        let ownership = Arc::new(PeakOwnership::default());
        let factory_ownership = Arc::clone(&ownership);
        let opens = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let factory_opens = Arc::clone(&opens);
        let shutdown = Arc::new(std::sync::OnceLock::<Arc<AtomicBool>>::new());
        let factory_shutdown = Arc::clone(&shutdown);
        let (entered, pending) = oneshot::channel();
        let (release, blocked) = mpsc::sync_channel(1);
        let mut gate = Some((entered, blocked));
        let retry_cap = if stage == "retry_backoff" {
            Duration::from_secs(60)
        } else {
            Duration::ZERO
        };
        let mut worker = Worker::start_with(
            vec![counter(r"\Memory\Available Bytes", "memory")],
            retry_cap,
            4096,
            move |configs, limit, _seed| {
                let rebuilding = factory_opens.fetch_add(1, Ordering::AcqRel) > 0;
                let mut api = PeakPdh {
                    ownership: Arc::clone(&factory_ownership),
                    owned: 0,
                    expansions: BTreeMap::new(),
                    shutdown: factory_shutdown.get().cloned(),
                    collect_gate: None,
                };
                if rebuilding {
                    let (entered, blocked) = gate.take().unwrap();
                    if stage == "rebuild_open" {
                        entered.send(()).unwrap();
                        blocked.recv_timeout(Duration::from_secs(2)).unwrap();
                    } else {
                        api.collect_gate = Some((entered, blocked));
                    }
                }
                Query::open_with_seed(
                    api,
                    configs,
                    Duration::ZERO,
                    Duration::from_secs(60),
                    256,
                    limit,
                    None,
                )
            },
        )
        .await
        .unwrap();
        shutdown
            .set(Arc::clone(&worker.shutdown_requested))
            .unwrap();
        ownership.collect_status.store(
            if stage == "retry_backoff" {
                1
            } else {
                windows_sys::Win32::System::Performance::PDH_INVALID_HANDLE
            },
            Ordering::Release,
        );
        assert!(worker.collect().await.is_err());
        ownership.collect_status.store(0, Ordering::Release);
        if stage == "retry_backoff" {
            let pending = worker.collect().await.unwrap();
            assert!(!pending.metadata.diagnostics.any_query_collected);
            assert_eq!(ownership.current.load(Ordering::Acquire), 1);
            assert_eq!(ownership.collect_calls.load(Ordering::Acquire), 2);
        } else {
            tokio::time::timeout(Duration::from_secs(1), async {
                tokio::select! {
                    result = worker.collect() => panic!("rebuild returned before release: {result:?}"),
                    result = pending => result.unwrap(),
                }
            })
            .await
            .unwrap();
            assert!(
                !worker
                    .shutdown(Instant::now() + Duration::from_millis(20))
                    .await
                    .unwrap()
            );
            release.send(()).unwrap();
        }
        assert!(
            worker
                .shutdown(Instant::now() + Duration::from_secs(1))
                .await
                .unwrap()
        );
        assert_eq!(ownership.current.load(Ordering::Acquire), 0, "{stage}");
        let expected_queries = if stage == "retry_backoff" { 1 } else { 2 };
        assert_eq!(
            ownership.closed_queries.load(Ordering::Acquire),
            expected_queries,
            "{stage}"
        );
        assert_eq!(opens.load(Ordering::Acquire), expected_queries, "{stage}");
    }
}

/// Scenario: Initial query construction returns a non-availability invariant error.
/// Guarantees: Permanent startup corruption remains explicit instead of entering an endless recovery loop.
#[tokio::test(flavor = "current_thread")]
async fn startup_rejects_nonrecoverable_query_errors() {
    let result = Worker::start_with::<StartupTestPdh<false>, _>(
        vec![counter(
            r"\Memory\Available Bytes",
            "windows.memory.available",
        )],
        Duration::from_secs(30),
        4_096,
        |_, _, _| Err(Error::InvalidSample("injected startup invariant")),
    )
    .await;
    let Err(error) = result else {
        panic!("nonrecoverable startup error unexpectedly succeeded");
    };

    assert!(matches!(
        error,
        Error::InvalidSample("injected startup invariant")
    ));
}

/// Scenario: The worker's startup prime receives a hard query collection failure.
/// Guarantees: Startup succeeds, the first scrape warms, and later collection recovers through normal retry state.
#[tokio::test(flavor = "current_thread")]
async fn startup_prime_query_failure_starts_and_recovers() {
    let path = r"\Processor(_Total)\% Processor Time";
    let counters = vec![counter(path, "system.cpu.time")];
    let mut worker = Worker::start_with(
        counters,
        Duration::from_secs(30),
        4_096,
        move |counters, max_expanded_counters, _seed| {
            Query::open_with_seed(
                CalculatedPdh {
                    fail_collect: Arc::new(AtomicBool::new(false)),
                    collect_calls: 0,
                    fail_first_collect: true,
                },
                counters,
                Duration::from_secs(30),
                Duration::from_secs(30),
                256,
                max_expanded_counters,
                None,
            )
        },
    )
    .await
    .unwrap();

    let warming = worker.collect().await.unwrap();
    assert_eq!(warming.points[0].value, SampleValue::Warming);
    assert_eq!(warming.metadata.failures.len(), 1);
    assert_eq!(warming.metadata.failures[0].counter_index, 0);
    assert_eq!(warming.metadata.failures[0].reason, "PdhCollectQueryData");
    assert!(warming.metadata.failures[0].diagnostic_only);
    assert_eq!(warming.metadata.diagnostics.retry_attempts, 1);
    assert_eq!(warming.metadata.diagnostics.retry_recoveries, 1);

    let recovered = worker.collect().await.unwrap();
    assert!(matches!(
        recovered.points[0].value,
        SampleValue::Value(Number::Double(2.0))
    ));
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(1))
            .await
            .unwrap()
    );
}

/// Scenario: An initially healthy query goes down, fails a replacement open, and subsequently rebuilds.
/// Guarantees: No-query maintenance retains the seed timestamp and valid membership, projects no batch, and never claims a successful collection.
#[test]
fn rebuild_maintenance_preserves_timestamp_membership_and_projection() {
    let path = r"\Unavailable\Value";
    let configs = vec![counter(path, "windows.unavailable")];
    let open_calls = Rc::new(Cell::new(0_usize));
    let factory_open_calls = Rc::clone(&open_calls);
    let states = Rc::new(RefCell::new(Vec::new()));
    let factory_states = Rc::clone(&states);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_native_counters: usize,
                         seed: Option<&QueryRecreationSeed>| {
        factory_open_calls.set(factory_open_calls.get() + 1);
        if factory_open_calls.get() == 2 {
            return Err(Error::Pdh {
                operation: PdhOperation::OpenQuery,
                path: "<query>".to_owned(),
                status: 1,
            });
        }
        let (state, query) = fake_query_with_seed(
            configs,
            vec![fake_counter(path, Number::Integer(42))],
            Duration::ZERO,
            Duration::from_secs(30),
            max_native_counters,
            seed,
        );
        factory_states.borrow_mut().push(state);
        Ok(query)
    };
    let mut queries = open_initial_query(
        configs.clone(),
        Duration::ZERO,
        4_096,
        &mut open,
        &AtomicBool::new(false),
    )
    .unwrap()
    .unwrap();
    states.borrow()[0].borrow_mut().timestamp = 110;
    let healthy = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
    assert_eq!(healthy.timestamp_unix_nano, 110);
    states.borrow()[0].borrow_mut().collect_error = true;
    for _ in 0..QUERY_REBUILD_AFTER_FAILURES {
        assert!(collect_query(&mut queries, &mut open, Instant::now()).is_err());
    }

    let pending = collect_query(&mut queries, &mut open, Instant::now()).unwrap();
    assert!(queries.query.is_none());
    assert_eq!(pending.timestamp_unix_nano, 110);
    assert!(pending.points.is_empty());
    assert_eq!(pending.members.len(), 1);
    assert_eq!(pending.members[0].counter_index, 0);
    assert_eq!(pending.members[0].path.as_ref(), path);
    assert!(!pending.metadata.diagnostics.any_query_collected);
    assert!(pending.metadata.diagnostics.query_pending);
    assert!(project_sample(&configs, &pending).unwrap().is_none());

    let rebuilt = collect_query(&mut queries, &mut open, Instant::now()).unwrap();

    assert!(rebuilt.points.is_empty());
    assert!(project_sample(&configs, &rebuilt).unwrap().is_none());
    assert!(!rebuilt.metadata.diagnostics.any_query_collected);
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_attempts, 1);
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_recoveries, 1);
    assert_eq!(open_calls.get(), 3);
}

/// Scenario: A calculated query configured for planned recreation reaches the failure threshold and is rebuilt during a scrape.
/// Guarantees: The rebuild scrape skips same-scrape recreation, remains empty after priming, and the following scrape emits over the full post-prime interval.
#[test]
fn rebuilt_query_defers_values_until_the_next_scrape() {
    let path = r"\Processor(_Total)\% Processor Time";
    let first_query_failure = Arc::new(AtomicBool::new(false));
    let factory_first_query_failure = Arc::clone(&first_query_failure);
    let open_calls = Rc::new(Cell::new(0_usize));
    let factory_open_calls = Rc::clone(&open_calls);
    let mut open = move |configs: Vec<CounterConfig>,
                         max_expanded_counters: usize,
                         _seed: Option<&QueryRecreationSeed>| {
        let open_index = factory_open_calls.get();
        factory_open_calls.set(open_index + 1);
        Query::open_with_seed(
            CalculatedPdh {
                fail_collect: if open_index == 0 {
                    Arc::clone(&factory_first_query_failure)
                } else {
                    Arc::new(AtomicBool::new(false))
                },
                collect_calls: 0,
                fail_first_collect: false,
            },
            configs,
            Duration::from_secs(30),
            Duration::from_secs(30),
            256,
            max_expanded_counters,
            None,
        )
    };
    let config = counter(path, "system.cpu.time");
    let plan = vec![config];
    let mut query = QuerySlot::open_initial(plan, Duration::ZERO, 4_096, &mut open).unwrap();
    first_query_failure.store(true, Ordering::Relaxed);
    let now = Instant::now();
    for _ in 0..QUERY_REBUILD_AFTER_FAILURES {
        assert!(matches!(
            query.collect(now, &mut open),
            Err(Error::Pdh {
                operation: PdhOperation::CollectQueryData,
                ..
            })
        ));
    }

    let rebuilt = collect_query(&mut query, &mut open, Instant::now()).unwrap();

    assert!(rebuilt.points.is_empty());
    assert_eq!(rebuilt.members.len(), 1);
    assert_eq!(rebuilt.members[0].counter_index, 0);
    assert_eq!(rebuilt.members[0].path.as_ref(), path);
    assert_eq!(rebuilt.timestamp_unix_nano, 110);
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_attempts, 1);
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_recoveries, 1);
    assert_eq!(rebuilt.metadata.diagnostics.warmup_omissions, 2);

    let recovered = query.collect(Instant::now(), &mut open).unwrap();
    assert!(matches!(
        recovered.points[0].value,
        SampleValue::Value(Number::Double(2.0))
    ));
    assert_eq!(recovered.points[0].timestamp_unix_nano, 120);
    assert_eq!(recovered.points[0].start_time_unix_nano, None);
    assert_eq!(recovered.metadata.diagnostics.query_rebuild_attempts, 0);
    assert_eq!(recovered.metadata.diagnostics.query_rebuild_recoveries, 0);
    assert_eq!(recovered.metadata.diagnostics.warmup_omissions, 0);
    assert_eq!(open_calls.get(), 2);
}

/// Scenario: A collection future times out while the worker remains inside a native call.
/// Guarantees: Later requests fail fast as busy and collection resumes after the call returns.
#[tokio::test(flavor = "current_thread")]
async fn bounds_timed_out_collection_and_recovers() {
    let (mut worker, ownership, started, release) = blocked_collect_worker().await;
    {
        let first = worker.collect();
        tokio::pin!(first);
        tokio::select! {
            result = &mut first => panic!("collection resolved before release: {result:?}"),
            result = started => result.expect("worker should start the first collection"),
        }
        assert!(
            tokio::time::timeout(Duration::from_millis(20), first)
                .await
                .is_err()
        );
    }
    assert!(matches!(worker.collect().await, Err(Error::WorkerBusy)));
    release.send(()).expect("release worker collection");
    tokio::time::timeout(Duration::from_secs(1), async {
        while worker.accepted.load(Ordering::Acquire) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("worker should finish blocked collection");
    assert_eq!(worker.collect().await.unwrap().points.len(), 1);
    assert_eq!(ownership.collect_calls.load(Ordering::Acquire), 3);
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(1))
            .await
            .expect("worker shutdown")
    );
}

/// Scenario: Startup or recovery deltas are merged into a completed collection.
/// Guarantees: Cumulative diagnostic deltas are added without overwriting the collection's current active-counter gauge.
#[test]
fn diagnostic_merge_preserves_current_active_count() {
    let mut current = SampleDiagnostics {
        active_expanded_counters: 2,
        instances_added: 2,
        ..Default::default()
    };
    let pending = SampleDiagnostics {
        active_expanded_counters: 0,
        discovery_refreshes: 1,
        ..Default::default()
    };

    merge_diagnostics(&mut current, pending);

    assert_eq!(current.active_expanded_counters, 2);
    assert_eq!(current.instances_added, 2);
    assert_eq!(current.discovery_refreshes, 1);
}

/// Scenario: Query metadata survives a failed collection and is merged with later recovery evidence.
/// Guarantees: Diagnostic counts, counter failures, and expansion overflows remain ordered and observable.
#[test]
fn sample_metadata_merge_preserves_pending_evidence() {
    let mut metadata = PendingMetadata {
        diagnostics: SampleDiagnostics {
            counter_add_failures: 1,
            ..Default::default()
        },
        failures: vec![SampleFailure {
            counter_index: 0,
            path: Arc::from(r"\Initial\Value"),
            reason: "initial",
            error: "initial failure".to_owned(),
            diagnostic_only: false,
        }],
        overflows: vec![ExpansionOverflow {
            counter_index: 0,
            reason: "admission_limit",
            discovered: 3,
            retained: 2,
            omitted: 1,
            per_wildcard_omitted: 1,
            total_omitted: 0,
            examples: vec![Arc::from(r"\Initial\Omitted")],
        }],
    };

    metadata.merge(PendingMetadata {
        diagnostics: SampleDiagnostics {
            retry_recoveries: 1,
            ..Default::default()
        },
        failures: vec![SampleFailure {
            counter_index: 1,
            path: Arc::from(r"\Recovery\Value"),
            reason: "recovery",
            error: "recovery failure".to_owned(),
            diagnostic_only: false,
        }],
        overflows: vec![ExpansionOverflow {
            counter_index: 1,
            reason: "admission_limit",
            discovered: 4,
            retained: 3,
            omitted: 1,
            per_wildcard_omitted: 0,
            total_omitted: 1,
            examples: vec![Arc::from(r"\Recovery\Omitted")],
        }],
    });

    assert_eq!(metadata.diagnostics.counter_add_failures, 1);
    assert_eq!(metadata.diagnostics.retry_recoveries, 1);
    assert_eq!(metadata.failures.len(), 2);
    assert_eq!(metadata.failures[0].reason, "initial");
    assert_eq!(metadata.failures[1].reason, "recovery");
    assert_eq!(metadata.overflows.len(), 2);
    assert_eq!(metadata.overflows[0].counter_index, 0);
    assert_eq!(metadata.overflows[1].counter_index, 1);
    assert_eq!(
        metadata.overflows[0].examples[0].as_ref(),
        r"\Initial\Omitted"
    );
    assert_eq!(
        metadata.overflows[1].examples[0].as_ref(),
        r"\Recovery\Omitted"
    );

    metadata.merge(PendingMetadata {
        diagnostics: SampleDiagnostics::default(),
        failures: vec![SampleFailure {
            counter_index: 0,
            path: Arc::from(r"\Initial\Value"),
            reason: "initial",
            error: "latest failure".to_owned(),
            diagnostic_only: false,
        }],
        overflows: vec![ExpansionOverflow {
            counter_index: 0,
            reason: "admission_limit",
            discovered: 5,
            retained: 2,
            omitted: 3,
            per_wildcard_omitted: 2,
            total_omitted: 1,
            examples: vec![Arc::from(r"\Initial\Latest")],
        }],
    });

    assert_eq!(metadata.failures.len(), 2);
    assert_eq!(metadata.failures[0].error, "latest failure");
    assert_eq!(metadata.overflows.len(), 2);
    assert_eq!(metadata.overflows[0].discovered, 5);
    assert_eq!(
        metadata.overflows[0].examples[0].as_ref(),
        r"\Initial\Latest"
    );
}

/// Scenario: A query repeatedly fails while the collection cadence caps recovery delay.
/// Guarantees: Query recovery stays responsive even when membership discovery uses a longer interval.
#[test]
fn retry_delay_is_exponential_and_capped() {
    let cap = Duration::from_secs(10);
    assert_eq!(retry_delay(1, cap), Duration::from_secs(1));
    assert_eq!(retry_delay(2, cap), Duration::from_secs(2));
    assert_eq!(retry_delay(3, cap), Duration::from_secs(4));
    assert_eq!(retry_delay(4, cap), Duration::from_secs(8));
    assert_eq!(retry_delay(5, cap), cap);
    assert_eq!(retry_delay(u32::MAX, cap), cap);
    assert_eq!(
        retry_delay(1, Duration::from_secs(1)),
        Duration::from_secs(1)
    );
}

/// Scenario: Query collection repeatedly fails with a two-second retry cap and a sixty-second membership refresh.
/// Guarantees: Recovery rebuilds on collection cadence instead of waiting for the unrelated wildcard discovery clock.
#[test]
fn query_recovery_cap_is_independent_of_membership_refresh() {
    let path = r"\Memory\Available Bytes";
    let fail_collect = Arc::new(AtomicBool::new(false));
    let factory_fail_collect = Arc::clone(&fail_collect);
    let open_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let factory_open_calls = Arc::clone(&open_calls);
    let retry_cap = Duration::from_secs(2);
    let wildcard_refresh = Duration::from_secs(60);
    let mut open = move |configs, max_native_counters, seed: Option<&QueryRecreationSeed>| {
        let _ = factory_open_calls.fetch_add(1, Ordering::AcqRel);
        Query::open_with_seed(
            CalculatedPdh {
                fail_collect: Arc::clone(&factory_fail_collect),
                collect_calls: 0,
                fail_first_collect: false,
            },
            configs,
            retry_cap,
            wildcard_refresh,
            256,
            max_native_counters,
            seed,
        )
    };
    let mut slot = QuerySlot::open_initial(
        vec![counter(path, "system.memory.available")],
        retry_cap,
        4_096,
        &mut open,
    )
    .unwrap();
    fail_collect.store(true, Ordering::Release);
    let now = Instant::now();
    for offset in [0, 1, 3] {
        assert!(matches!(
            collect_query(&mut slot, &mut open, now + Duration::from_secs(offset)),
            Err(Error::Pdh {
                operation: PdhOperation::CollectQueryData,
                ..
            })
        ));
    }
    fail_collect.store(false, Ordering::Release);
    let rebuilt = collect_query(&mut slot, &mut open, now + Duration::from_secs(5)).unwrap();

    assert!(rebuilt.points.is_empty());
    assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_recoveries, 1);
    assert_eq!(open_calls.load(Ordering::Acquire), 2);
}

async fn blocked_collect_worker() -> (
    Worker,
    Arc<PeakOwnership>,
    oneshot::Receiver<()>,
    mpsc::Sender<()>,
) {
    let ownership = Arc::new(PeakOwnership::default());
    let worker_ownership = Arc::clone(&ownership);
    let worker = Worker::start_controlled(
        vec![counter(r"\Memory\Available Bytes", "available")],
        Duration::from_secs(30),
        4_096,
        move |configs, limit, seed, shutdown| {
            Query::open_with_seed(
                PeakPdh {
                    ownership: Arc::clone(&worker_ownership),
                    owned: 0,
                    expansions: BTreeMap::new(),
                    shutdown: Some(shutdown),
                    collect_gate: None,
                },
                configs,
                Duration::from_secs(30),
                Duration::from_secs(30),
                256,
                limit,
                seed,
            )
        },
    )
    .await
    .unwrap();
    let (entered, started) = oneshot::channel();
    let (release, blocked) = mpsc::channel();
    *ownership.collect_gate.lock().unwrap() = Some((entered, blocked));
    (worker, ownership, started, release)
}
