// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: A persistent worker collects two supported counters repeatedly and shuts down.
/// Guarantees: One query serves every scrape and normal shutdown closes it exactly once.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_persistent_worker_reuses_and_closes_query() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let before = query_counts();
    let counters = vec![
        counter(r"\Memory\Available Bytes", "windows.memory.available"),
        counter(r"\Memory\Committed Bytes", "windows.memory.committed"),
    ];
    let mut worker = Worker::start(
        counters,
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "test".to_owned(),
    )
    .await
    .unwrap();
    for _ in 0..3 {
        let sample = worker.collect().await.unwrap();
        assert_eq!(sample.points.len(), 2);
        assert!(sample.points.iter().all(
            |point| matches!(point.value, SampleValue::Value(Number::Integer(value)) if value >= 0)
        ));
        assert_eq!(sample.points[0].counter_index, 0);
        assert_eq!(sample.points[0].path.as_ref(), r"\Memory\Available Bytes");
        assert_eq!(sample.points[1].counter_index, 1);
        assert_eq!(sample.points[1].path.as_ref(), r"\Memory\Committed Bytes");
    }
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
    let after = query_counts();
    assert_eq!(after.0 - before.0, 1);
    assert_eq!(after.1 - before.1, 1);
}

/// Scenario: Two independently configured receiver workers receive concurrent collection requests.
/// Guarantees: Both requests produce observations from independently owned queries that shut down cleanly.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_independent_workers_can_collect_concurrently() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let before = query_counts();
    let counters = vec![counter(
        r"\Memory\Available Bytes",
        "windows.memory.available",
    )];
    let mut first = Worker::start(
        counters.clone(),
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "first".to_owned(),
    )
    .await
    .unwrap();
    let mut second = Worker::start(
        counters,
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "second".to_owned(),
    )
    .await
    .unwrap();
    let (first_result, second_result) = tokio::join!(first.collect(), second.collect());
    assert!(!first_result.unwrap().points.is_empty());
    assert!(!second_result.unwrap().points.is_empty());
    assert!(
        first
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
    assert!(
        second
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
    let after = query_counts();
    assert_eq!(after.0 - before.0, 2);
    assert_eq!(after.1 - before.1, 2);
}

/// Scenario: Worker startup primes direct, raw-fraction, and two-sample counters before the first request.
/// Guarantees: The first requested scrape emits every valid family after sufficient elapsed time.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_startup_prime_enables_first_calculated_scrape() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let counters = vec![
        counter(r"\Memory\Available Bytes", "windows.memory.available"),
        counter(
            r"\Memory\% Committed Bytes In Use",
            "windows.memory.committed_percent",
        ),
        counter(
            r"\Processor(_Total)\% Processor Time",
            "windows.processor.time",
        ),
    ];
    let mut worker = Worker::start(
        counters,
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "test".to_owned(),
    )
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(250)).await;
    let sample = worker.collect().await.unwrap();
    assert!(matches!(
        sample.points[0].value,
        SampleValue::Value(Number::Integer(_))
    ));
    assert!(
        matches!(sample.points[1].value, SampleValue::Value(Number::Double(value)) if value.is_finite())
    );
    assert!(
        matches!(sample.points[2].value, SampleValue::Value(Number::Double(value)) if value.is_finite())
    );
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
}

/// Scenario: An owned process is selected by prefix-`*` and pure-`?` wildcard gauges beside an exact Memory gauge.
/// Guarantees: Both wildcard forms return only the owned identity and its PID or value, without changing the exact point.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_collects_exact_and_wildcard_counters() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let mut fixture = CounterProcessFixture::new();
    let pid = fixture.spawn();
    let wildcard_template = format!(r"\Process({}*)\Private Bytes", fixture.instance);
    let question_template = format!(
        r"\Process(?{})\ID Process",
        fixture.instance.strip_prefix('o').unwrap()
    );
    let mut wildcard = counter(&wildcard_template, "windows.process.private");
    wildcard.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut question = counter(&question_template, "windows.process.id");
    question.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: Some("_Total".to_owned()),
    };
    let counters = vec![
        counter(r"\Memory\Available Bytes", "windows.memory.available"),
        wildcard,
        question,
    ];
    let mut worker = Worker::start(
        counters,
        QueryCadence::uniform(Duration::from_millis(100)),
        256,
        4_096,
        "test".to_owned(),
    )
    .await
    .unwrap();
    let mut sample = worker.collect().await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if [1, 2].into_iter().all(|counter_index| {
            sample.points.iter().any(|point| {
                point.counter_index == counter_index && matches!(point.value, SampleValue::Value(_))
            })
        }) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "owned process counters not visible"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
        sample = worker.collect().await.unwrap();
    }
    let exact = sample
        .points
        .iter()
        .find(|point| point.counter_index == 0)
        .expect("exact Memory point");
    assert_eq!(exact.path.as_ref(), r"\Memory\Available Bytes");
    assert!(exact.instance.is_none());
    assert!(matches!(
        exact.value,
        SampleValue::Value(Number::Integer(value)) if value >= 0
    ));

    let wildcard = sample
        .points
        .iter()
        .filter(|point| point.counter_index == 1)
        .collect::<Vec<_>>();
    assert_eq!(wildcard.len(), 1);
    let keys = wildcard
        .iter()
        .map(|point| CounterKey::new(point.counter_index, &point.path))
        .collect::<BTreeSet<_>>();
    assert_eq!(keys.len(), wildcard.len());
    assert!(wildcard.iter().all(|point| {
        point
            .instance
            .as_ref()
            .is_some_and(|instance| instance.name.eq_ignore_ascii_case(&fixture.instance))
            && point.path.as_ref() != wildcard_template
            && matches!(
                point.value,
                SampleValue::Value(Number::Integer(value)) if value >= 0
            )
    }));
    let question = sample
        .points
        .iter()
        .filter(|point| point.counter_index == 2)
        .collect::<Vec<_>>();
    assert_eq!(question.len(), 1);
    assert!(question.iter().all(|point| {
        point.path.as_ref() != question_template
            && point
                .instance
                .as_ref()
                .is_some_and(|instance| instance.name.eq_ignore_ascii_case(&fixture.instance))
            && matches!(
                point.value,
                SampleValue::Value(Number::Integer(value)) if value == pid
            )
    }));
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
}

/// Scenario: Windows expands canonical and mixed-case wildcards restricted to one owned process prefix.
/// Guarantees: Both spellings return the owned PID once without depending on unrelated host process churn.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_ascii_case_variants_share_native_membership() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let mut fixture = CounterProcessFixture::new();
    let pid = fixture.spawn();
    let mut canonical = counter(
        &format!(r"\Process({}*)\ID Process", fixture.instance),
        "process.id.canonical",
    );
    canonical.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut variant = counter(
        &format!(
            r"\pRoCeSs({}*)\iD pRoCeSs",
            fixture.instance.to_ascii_uppercase()
        ),
        "process.id.variant",
    );
    variant.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    variant.source_mapping_index = 1;
    let mut query = Query::open_replacement(
        vec![canonical, variant],
        Duration::from_millis(100),
        Duration::from_millis(100),
        256,
        4_096,
        "ascii-case".to_owned(),
        None,
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    let sample = wait_for_live_process_membership(
        &mut query,
        "both case variants of the owned process",
        |sample| sample,
        |sample| {
            sample.metadata.failures.is_empty()
                && sample.points.len() == 2
                && [0, 1].into_iter().all(|counter_index| {
                    sample.points.iter().any(|point| {
                        point.counter_index == counter_index
                            && point.value == SampleValue::Value(Number::Integer(pid))
                    })
                })
        },
    )
    .await;
    assert!(
        sample.metadata.failures.is_empty(),
        "{:#?}",
        sample.metadata
    );
    let paths = [0, 1].map(|counter_index| {
        sample
            .points
            .iter()
            .filter(|point| point.counter_index == counter_index)
            .map(|point| point.path.to_ascii_lowercase())
            .collect::<BTreeSet<_>>()
    });
    assert!(!paths[0].is_empty());
    assert_eq!(paths[0], paths[1]);
    for counter_index in [0, 1] {
        let points = sample
            .points
            .iter()
            .filter(|point| point.counter_index == counter_index)
            .collect::<Vec<_>>();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, SampleValue::Value(Number::Integer(pid)));
    }
}

/// Scenario: Worker startup primes common disk queue-length and precision-timer counters.
/// Guarantees: Both two-sample families produce finite values on the first requested scrape.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_collects_common_calculated_disk_counters() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let counters = vec![
        counter(
            r"\PhysicalDisk(_Total)\Avg. Disk Queue Length",
            "windows.disk.queue_length",
        ),
        counter(r"\PhysicalDisk(_Total)\% Disk Time", "windows.disk.time"),
    ];
    let mut worker = Worker::start(
        counters,
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "test".to_owned(),
    )
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(250)).await;
    let sample = worker.collect().await.unwrap();
    assert!(
        sample.metadata.failures.is_empty(),
        "{:?}",
        sample.metadata.failures
    );
    assert_eq!(sample.points.len(), 2);
    assert!(
        sample.points.iter().all(|point| matches!(
            point.value,
            SampleValue::Value(Number::Double(value)) if value.is_finite()
        )),
        "{:?}",
        sample.points
    );
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
}

/// Scenario: Startup adds a healthy exact counter while another configured counter is absent.
/// Guarantees: The worker starts, emits a valid healthy value, and reports the unavailable counter context.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_unavailable_exact_counter_preserves_healthy_startup() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let before = query_counts();
    let counters = vec![
        counter(r"\Memory\Available Bytes", "windows.memory.available"),
        counter(
            r"\Missing Object\Missing Counter",
            "windows.missing.counter",
        ),
    ];
    let mut worker = Worker::start(
        counters,
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "test".to_owned(),
    )
    .await
    .unwrap();
    let sample = worker.collect().await.unwrap();
    assert!(sample.points.iter().any(|point| {
        point.path.as_ref() == r"\Memory\Available Bytes"
            && matches!(point.value, SampleValue::Value(Number::Integer(value)) if value >= 0)
    }));
    assert!(sample.metadata.failures.iter().any(|failure| {
        failure.counter_index == 1
            && failure.reason == "selector_unresolved"
            && failure.error.contains("PdhAddEnglishCounterW")
    }));
    assert!(sample.metadata.diagnostics.counter_add_failures >= 1);
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
    let after = query_counts();
    assert_eq!(after.0 - before.0, 1);
    assert_eq!(after.1 - before.1, 1);
}

/// Scenario: A provider counter requires its trailing space while the trimmed spelling is unavailable.
/// Guarantees: Parsed native spelling reaches PDH unchanged, the exact provider path emits, and the trimmed peer fails discovery beside a healthy control.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_preserves_provider_significant_counter_space() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let exact_template = r"\Hyper-V Virtual Switch Port(*)\IPsec SAs Offloaded ";
    let trimmed_template = r"\Hyper-V Virtual Switch Port(*)\IPsec SAs Offloaded";
    let mut probe = NativePdh::open("native-spelling-preflight".to_owned()).unwrap();
    let exact_paths = match probe.expand(exact_template, &mut DiscoveryBudget::new()) {
        Ok(paths) if !paths.is_empty() => paths,
        result => {
            eprintln!(
                "skipping significant-space control: provider or instances unavailable: {result:?}"
            );
            return;
        }
    };
    if probe
        .expand(trimmed_template, &mut DiscoveryBudget::new())
        .is_ok_and(|paths| !paths.is_empty())
    {
        eprintln!(
            "skipping significant-space control: trimmed spelling resolves and is not a negative control"
        );
        return;
    }
    drop(probe);
    assert!(exact_paths.iter().all(|path| path.ends_with(" ")));

    let config = Config::from_json(&serde_json::json!({
        "metrics": {
            "provider.exact": {"unit": "1"},
            "provider.trimmed": {"unit": "1"},
            "memory.control": {"unit": "By"}
        },
        "perfcounters": [
            {
                "object": "Hyper-V Virtual Switch Port",
                "instances": "*",
                "counters": [
                    {
                        "name": "IPsec SAs Offloaded ",
                        "metric": "provider.exact"
                    },
                    {
                        "name": "IPsec SAs Offloaded",
                        "metric": "provider.trimmed"
                    }
                ]
            },
            {
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "memory.control"
                }]
            }
        ]
    }))
    .unwrap();
    assert_eq!(
        config.counters[0].path,
        r"\Hyper-V Virtual Switch Port(*)\IPsec SAs Offloaded "
    );
    assert_eq!(
        config.counters[1].path,
        r"\Hyper-V Virtual Switch Port(*)\IPsec SAs Offloaded"
    );
    let mut worker = Worker::start(
        config.counters,
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "native-spelling".to_owned(),
    )
    .await
    .unwrap();
    let sample = worker.collect().await.unwrap();
    assert!(sample.points.iter().any(|point| {
        point.counter_index == 0
            && point.path.ends_with(r"\IPsec SAs Offloaded ")
            && matches!(
                point.value,
                SampleValue::Value(Number::Integer(value)) if value >= 0
            )
    }));
    assert!(sample.points.iter().any(|point| {
        point.counter_index == 2
            && point.path.as_ref() == r"\Memory\Available Bytes"
            && matches!(
                point.value,
                SampleValue::Value(Number::Integer(value)) if value >= 0
            )
    }));
    assert!(!sample.points.iter().any(|point| point.counter_index == 1));
    assert!(
        sample.metadata.failures.iter().any(|failure| {
            failure.counter_index == 1
                && failure.reason == "selector_unresolved"
                && failure.error.contains("PdhAddEnglishCounterW")
                && failure.error.contains("0xC0000BB9")
                && failure.diagnostic_only
        }),
        "{:#?}",
        sample.metadata
    );
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
}

/// Scenario: A real Paging File instance contains a backslash that configuration previously rejected.
/// Guarantees: The exact backslashed path reaches PDH, emits a finite value, and remains isolated beside a healthy control.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_collects_backslashed_paging_file_instance() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let mut probe = NativePdh::open("native-delimiter-preflight".to_owned()).unwrap();
    let expanded = match probe.expand(r"\Paging File(*)\% Usage", &mut DiscoveryBudget::new()) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("skipping paging-file delimiter control: expansion unavailable: {error}");
            return;
        }
    };
    let Some((selected_path, selected_instance)) = expanded.into_iter().find_map(|path| {
        let instance = probe.parse_instance(&path).ok()?;
        (instance.name != "_Total" && instance.name.contains('\\')).then_some((path, instance.name))
    }) else {
        eprintln!("skipping paging-file delimiter control: no backslashed instance is available");
        return;
    };
    drop(probe);
    let expected_path = format!(r"\Paging File({selected_instance})\% Usage");
    assert!(selected_path.ends_with(&expected_path));

    let config = Config::from_json(&serde_json::json!({
        "metrics": {
            "paging.usage": {"unit": "%"},
            "memory.control": {"unit": "By"}
        },
        "perfcounters": [
            {
                "object": "Paging File",
                "instances": selected_instance,
                "counters": [{"name": "% Usage", "metric": "paging.usage"}]
            },
            {
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "memory.control"
                }]
            }
        ]
    }))
    .unwrap();
    assert_eq!(config.counters[0].path, expected_path);
    let mut worker = Worker::start(
        config.counters,
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "native-delimiter".to_owned(),
    )
    .await
    .unwrap();
    let sample = worker.collect().await.unwrap();
    assert!(
        sample.metadata.failures.is_empty(),
        "{:#?}",
        sample.metadata
    );
    assert!(sample.points.iter().any(|point| {
        point.counter_index == 0
            && point.path.as_ref() == expected_path
            && matches!(
                point.value,
                SampleValue::Value(Number::Double(value)) if value.is_finite() && value >= 0.0
            )
    }));
    assert!(sample.points.iter().any(|point| {
        point.counter_index == 1
            && point.path.as_ref() == r"\Memory\Available Bytes"
            && matches!(
                point.value,
                SampleValue::Value(Number::Integer(value)) if value >= 0
            )
    }));
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
}

/// Scenario: Every configured exact counter is unavailable when the worker starts.
/// Guarantees: Startup exposes pending availability, collection remains empty, and shutdown closes the retryable query.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_entirely_unavailable_counter_set_starts_empty() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let before = query_counts();
    let mut worker = Worker::start(
        vec![counter(
            r"\Missing Object\Missing Counter",
            "windows.missing.counter",
        )],
        QueryCadence::uniform(Duration::from_secs(30)),
        256,
        4_096,
        "test".to_owned(),
    )
    .await
    .unwrap();
    assert!(worker.startup.pending);
    let sample = worker.collect().await.unwrap();
    assert!(sample.metadata.diagnostics.query_pending);
    assert!(!sample.metadata.diagnostics.any_query_collected);
    assert!(sample.points.is_empty());
    assert!(
        worker
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap()
    );
    let after = query_counts();
    assert_eq!(after.0 - before.0, 1);
    assert_eq!(after.1 - before.1, 1);
}

fn open_live_query(configs: Vec<CounterConfig>, node: &str) -> Query {
    Query::open_replacement(
        configs,
        Duration::from_secs(30),
        Duration::from_secs(30),
        256,
        4_096,
        node.to_owned(),
        None,
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap()
}

/// Scenario: A removed counter token is reused after another counter is added to the same adapter.
/// Guarantees: Removed slots cannot alias new counters and every operation rejects the stale token.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_adapter_rejects_removed_tokens() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let path = r"\Memory\Available Bytes";
    let mut api = NativePdh::open("removed".to_owned()).unwrap();
    let removed = api.add(path, CounterLanguage::English).unwrap();
    api.remove(&removed, path).unwrap();
    let live = api.add(path, CounterLanguage::English).unwrap();
    for result in [
        api.remove(&removed, path),
        api.counter_type(&removed, path).map(|_| ()),
        api.raw(&removed, path).map(|_| ()),
        api.formatted(&removed, path, ValueFormat::Integer)
            .map(|_| ()),
    ] {
        let error = result.unwrap_err();
        assert!(matches!(&error, Error::InvalidCounterToken { path: failed } if failed == path));
    }
    assert!(api.collect().unwrap());
    assert!(api.counter_type(&live, path).is_ok());
}

struct CounterProcess(Child);

impl CounterProcess {
    fn id(&self) -> i64 {
        i64::from(self.0.id())
    }
}

impl Drop for CounterProcess {
    fn drop(&mut self) {
        drop(self.0.stdin.take());
        let _ = self.0.wait();
    }
}

fn spawn_counter_process(executable: &Path) -> CounterProcess {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut child = CounterProcess(
        ProcessCommand::new(executable)
            .args(["/d", "/q", "/c", "echo ready & set /p wait="])
            .creation_flags(CREATE_NO_WINDOW)
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

struct CounterProcessFixture {
    directory: std::path::PathBuf,
    executable: std::path::PathBuf,
    instance: String,
    processes: Vec<CounterProcess>,
}

impl CounterProcessFixture {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "otel-arrow-pdh-membership-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        let instance = format!(
            "otm{:06x}{:06x}",
            std::process::id() & 0x00ff_ffff,
            nonce & 0x00ff_ffff
        );
        let executable = directory.join(format!("{instance}.exe"));
        let fixture = Self {
            directory,
            executable,
            instance,
            processes: Vec::new(),
        };
        let source = std::env::var_os("COMSPEC").expect("COMSPEC must identify cmd.exe");
        let _ = std::fs::copy(source, &fixture.executable).unwrap();
        fixture
    }

    fn spawn(&mut self) -> i64 {
        let process = spawn_counter_process(&self.executable);
        let id = process.id();
        self.processes.push(process);
        id
    }

    fn stop(&mut self, id: i64) {
        let index = self
            .processes
            .iter()
            .position(|process| process.id() == id)
            .unwrap_or_else(|| panic!("fixture does not own process ID {id}"));
        drop(self.processes.swap_remove(index));
    }
}

impl Drop for CounterProcessFixture {
    fn drop(&mut self) {
        self.processes.clear();
        let _ = std::fs::remove_file(&self.executable);
        let _ = std::fs::remove_dir(&self.directory);
    }
}

#[derive(Debug)]
struct LiveProcessMembership {
    ids: BTreeSet<i64>,
    unsuffixed_id: Option<i64>,
    wildcard_points: usize,
    wildcard_paths: BTreeSet<String>,
    exact_points: usize,
    exact_valid: bool,
    unexpected_points: usize,
    invalid_wildcard_values: Vec<SampleValue>,
    metadata: PendingMetadata,
}

impl LiveProcessMembership {
    fn from_sample(sample: Sample, instance_name: &str) -> Self {
        let mut ids = BTreeSet::new();
        let mut unsuffixed_id = None;
        let mut wildcard_points = 0;
        let mut wildcard_paths = BTreeSet::new();
        let mut exact_points = 0;
        let mut exact_valid = false;
        let mut unexpected_points = 0;
        let mut invalid_wildcard_values = Vec::new();
        for point in sample.points {
            match point.counter_index {
                0 => {
                    exact_points += 1;
                    exact_valid = point.path.as_ref() == r"\Memory\Available Bytes"
                        && point.instance.is_none()
                        && matches!(
                            point.value,
                            SampleValue::Value(Number::Integer(value)) if value >= 0
                        );
                }
                1 => {
                    wildcard_points += 1;
                    let _ = wildcard_paths.insert(point.path.to_string());
                    match point.value {
                        SampleValue::Value(Number::Integer(id)) if id > 0 => {
                            let _ = ids.insert(id);
                            if point.instance.as_ref().is_some_and(|instance| {
                                instance.name.eq_ignore_ascii_case(instance_name)
                                    && instance.index == 0
                            }) {
                                unsuffixed_id = Some(id);
                            }
                        }
                        value => invalid_wildcard_values.push(value),
                    }
                }
                _ => unexpected_points += 1,
            }
        }
        Self {
            ids,
            unsuffixed_id,
            wildcard_points,
            wildcard_paths,
            exact_points,
            exact_valid,
            unexpected_points,
            invalid_wildcard_values,
            metadata: sample.metadata,
        }
    }

    fn matches(&self, expected: &BTreeSet<i64>) -> bool {
        self.ids == *expected
            && self.unsuffixed_id.is_some()
            && self.wildcard_points == expected.len()
            && self.wildcard_paths.len() == self.wildcard_points
            && self.exact_points == 1
            && self.exact_valid
            && self.unexpected_points == 0
            && self.invalid_wildcard_values.is_empty()
            && self.metadata.failures.is_empty()
            && self.metadata.overflows.is_empty()
    }
}

async fn wait_for_live_process_membership<T: std::fmt::Debug>(
    query: &mut Query<impl PdhApi>,
    phase: &str,
    mut observe: impl FnMut(Sample) -> T,
    matches: impl Fn(&T) -> bool,
) -> T {
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut last_observed = None;
    let mut last_error = None;
    loop {
        match query.collect() {
            Ok(sample) => {
                let observed = observe(sample);
                if matches(&observed) {
                    return observed;
                }
                last_observed = Some(observed);
            }
            Err(error) => last_error = Some(error),
        }
        assert!(
            Instant::now() < deadline,
            "{phase} did not converge; last observed: {last_observed:#?}; \
             last collection error: {last_error:?}"
        );
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

/// Scenario: Membership collection fails once before the same query returns the expected value.
/// Guarantees: The visibility wait stays pending through the error and accepts recovery without restarting the query.
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn membership_wait_recovers_after_collection_error() {
    use std::future::{Future, poll_fn};
    use std::task::Poll;

    let path = r"\Memory\Available Bytes";
    let (state, mut query) = fake_query_with_seed(
        vec![counter(path, "memory")],
        vec![fake_counter(path, Number::Integer(42))],
        Duration::from_secs(60),
        Duration::MAX,
        1,
        None,
    );
    let before = state.borrow().calls.collect;
    state.borrow_mut().collect_error = true;
    let waiting = wait_for_live_process_membership(
        &mut query,
        "collection-error recovery",
        |sample| sample,
        |sample| {
            sample.points.len() == 1
                && sample.points[0].value == SampleValue::Value(Number::Integer(42))
        },
    );
    tokio::pin!(waiting);
    poll_fn(|cx| {
        assert!(waiting.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    assert_eq!(state.borrow().calls.collect, before + 1);
    state.borrow_mut().collect_error = false;
    let sample = waiting.await;
    assert_eq!(sample.points[0].path.as_ref(), path);
    assert_eq!(state.borrow().calls.collect, before + 2);
}

/// Scenario: A filtered process wildcard repeatedly changes membership while one query stays open.
/// Guarantees: Each phase emits every owned PID once, no stale IDs, and one healthy exact peer.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_refreshes_process_membership_after_duplicate_renumbering() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let mut fixture = CounterProcessFixture::new();
    let mut expected = BTreeSet::from([fixture.spawn(), fixture.spawn()]);
    let mut wildcard = counter(
        &format!(r"\Process({}*)\ID Process", fixture.instance),
        "process.id",
    );
    wildcard.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut query = Query::open_replacement(
        vec![
            counter(r"\Memory\Available Bytes", "windows.memory.available"),
            wildcard,
        ],
        Duration::from_millis(100),
        Duration::from_millis(100),
        16,
        16,
        "membership".to_owned(),
        None,
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();

    let mut observed = wait_for_live_process_membership(
        &mut query,
        "initial two-process membership",
        |sample| LiveProcessMembership::from_sample(sample, &fixture.instance),
        |observed| observed.matches(&expected),
    )
    .await;
    for cycle in 1..=2 {
        let removed = observed
            .unsuffixed_id
            .expect("converged membership must identify the unsuffixed instance");
        assert!(expected.remove(&removed));
        fixture.stop(removed);
        let _ = wait_for_live_process_membership(
            &mut query,
            &format!("cycle {cycle} surviving process after renumbering"),
            |sample| LiveProcessMembership::from_sample(sample, &fixture.instance),
            |observed| observed.matches(&expected),
        )
        .await;

        let replacement = fixture.spawn();
        assert!(expected.insert(replacement));
        observed = wait_for_live_process_membership(
            &mut query,
            &format!("cycle {cycle} replacement discovery"),
            |sample| LiveProcessMembership::from_sample(sample, &fixture.instance),
            |observed| observed.matches(&expected),
        )
        .await;
    }
    assert_eq!(observed.ids, expected);
}

/// Scenario: Parsed configuration names duplicate live process instances using #0 and #01.
/// Guarantees: Configured suffix spelling reaches PDH unchanged and collects exactly the fixture process IDs.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_collects_parsed_instance_paths() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let mut fixture = CounterProcessFixture::new();
    let expected = [fixture.spawn(), fixture.spawn()]
        .into_iter()
        .collect::<BTreeSet<_>>();
    let instance = &fixture.instance;
    let config = Config::from_json(&serde_json::json!({
        "metrics": {
            "process.id": {
                "description": "Process identifier.",
                "unit": "1",
                "gauge": {}
            }
        },
        "perfcounters": [{
            "object": "Process",
            "instances": [format!("{instance}#0"), format!("{instance}#01")],
            "counters": [{"name": "ID Process", "metric": "process.id"}]
        }]
    }))
    .unwrap();
    assert_eq!(
        config.counters[0].path,
        format!(r"\Process({instance}#0)\ID Process")
    );
    assert_eq!(
        config.counters[1].path,
        format!(r"\Process({instance}#01)\ID Process")
    );
    let counters = config.counters.clone();
    let mut query = open_live_query(counters.clone(), "parsed");
    let sample = query.collect().unwrap();
    assert!(
        sample.metadata.failures.is_empty(),
        "{:?}",
        sample.metadata.failures
    );
    assert_eq!(sample.points.len(), 2);
    let ids = sample
        .points
        .iter()
        .map(|point| match point.value {
            SampleValue::Value(Number::Integer(id)) if id > 0 => id,
            value => panic!("expected a positive process ID, got {value:?}"),
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, expected);
    assert!(project_sample(&counters, &sample).unwrap().is_some());
}

/// Scenario: One live PDH query maps two distinct Memory paths to metrics with different attributes.
/// Guarantees: Both mappings emit integers on two collections, and OTAP preserves each metric's name, value, path, and attributes.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_collects_distinct_native_paths_and_outputs() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
    use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
    use otel_arrow_dfe_pdata_codec::OtapPayload;
    use prost::Message;

    let _serial = TEST_PDH_LOCK.lock().await;
    let paths = [r"\Memory\Available Bytes", r"\Memory\Committed Bytes"];
    let config = Config::from_json(&serde_json::json!({
        "metrics": {
            "memory.available": {"unit": "By", "gauge": {}},
            "memory.committed": {"unit": "By", "gauge": {}}
        },
        "perfcounters": [{
            "object": "Memory",
            "counters": [
                {"name": "Available Bytes", "metric": "memory.available"},
                {
                    "name": "Committed Bytes",
                    "metric": "memory.committed",
                    "attributes": {"state": "committed"}
                }
            ]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 2);
    assert!(
        config
            .counters
            .iter()
            .zip(paths)
            .all(|(counter, path)| counter.path == path)
    );
    let mut query = open_live_query(config.counters.clone(), "distinct-mappings");

    for _ in 0..2 {
        let sample = query.collect().unwrap();
        assert!(
            sample.metadata.failures.is_empty(),
            "{:?}",
            sample.metadata.failures
        );
        assert_eq!(sample.points.len(), 2);
        let values = sample
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                assert_eq!(point.counter_index, index);
                match point.value {
                    SampleValue::Value(Number::Integer(value)) if value >= 0 => value,
                    value => panic!("expected non-negative available bytes, got {value:?}"),
                }
            })
            .collect::<Vec<_>>();
        let records = project_sample(&config.counters, &sample).unwrap().unwrap();
        let bytes: OtlpProtoBytes = OtapPayload::from_otap(records)
            .try_into_with_default()
            .unwrap();
        let decoded = ExportMetricsServiceRequest::decode(bytes.as_bytes()).unwrap();
        let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(metrics.len(), 2);
        for (((metric, (name, state)), value), path) in metrics
            .iter()
            .zip([
                ("memory.available", None),
                ("memory.committed", Some("committed")),
            ])
            .zip(values)
            .zip(paths)
        {
            assert_eq!(metric.name, name);
            let Some(metric::Data::Gauge(gauge)) = &metric.data else {
                panic!("expected a Gauge");
            };
            assert_eq!(gauge.data_points.len(), 1);
            let point = &gauge.data_points[0];
            assert_eq!(point.value, Some(number_data_point::Value::AsInt(value)));
            let attributes = point
                .attributes
                .iter()
                .map(|attribute| {
                    let Some(any_value::Value::StringValue(value)) = attribute
                        .value
                        .as_ref()
                        .and_then(|value| value.value.as_ref())
                    else {
                        panic!("expected a string attribute");
                    };
                    (attribute.key.as_str(), value.as_str())
                })
                .collect::<BTreeMap<_, _>>();
            let mut expected = BTreeMap::from([("windows.perf_counter.path", path)]);
            if let Some(state) = state {
                let _ = expected.insert("state", state);
            }
            assert_eq!(attributes, expected);
        }
    }
}

/// Scenario: Two native queries collect exact direct counters repeatedly and are dropped independently.
/// Guarantees: Integer points and cumulative start times remain valid across query lifetimes.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_direct_queries() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let configs = vec![counter(
        r"\Memory\Available Bytes",
        "windows.memory.available",
    )];
    let mut first = open_live_query(configs.clone(), "first");
    let mut second = open_live_query(configs.clone(), "second");
    let start = first.start_time_unix_nano;
    for _ in 0..3 {
        for query in [&mut first, &mut second] {
            let sample = query.collect().unwrap();
            assert!(
                sample.metadata.failures.is_empty(),
                "{:?}",
                sample.metadata.failures
            );
            assert!(
                matches!(sample.points[0].value, SampleValue::Value(Number::Integer(value)) if value >= 0)
            );
            assert!(project_sample(&configs, &sample).unwrap().is_some());
        }
        assert_eq!(first.start_time_unix_nano, start);
    }
    drop(first);
    assert!(second.collect().unwrap().metadata.failures.is_empty());
}

/// Scenario: The local Windows providers expose representative rates, timers, queues, fractions, and averages.
/// Guarantees: Available live families follow their readiness without substituting for complete SDK fixture coverage.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_calculated_counters() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let configs = vec![
        counter(
            r"\System\Context Switches/sec",
            "windows.system.context_switches",
        ),
        counter(
            r"\Processor(_Total)\% Processor Time",
            "windows.processor.utilization",
        ),
        counter(
            r"\PhysicalDisk(_Total)\Avg. Disk Queue Length",
            "windows.disk.queue_length",
        ),
        counter(
            r"\Memory\% Committed Bytes In Use",
            "windows.memory.committed",
        ),
        counter(
            r"\PhysicalDisk(_Total)\Avg. Disk sec/Read",
            "windows.disk.read_latency",
        ),
    ];
    let mut query = open_live_query(configs.clone(), "calculated");
    let warming = query.collect().unwrap();
    assert!(
        warming.metadata.failures.is_empty(),
        "{:?}",
        warming.metadata.failures
    );
    for index in [0, 1, 2, 4] {
        assert_eq!(warming.points[index].value, SampleValue::Warming);
    }
    assert!(matches!(
        warming.points[3].value,
        SampleValue::Value(Number::Double(value)) if value.is_finite() && value >= 0.0
    ));
    std::thread::sleep(Duration::from_millis(250));
    let sample = query.collect().unwrap();
    assert!(
        sample.metadata.failures.is_empty(),
        "{:?}",
        sample.metadata.failures
    );
    assert_eq!(sample.points.len(), configs.len());
    for point in &sample.points[..4] {
        assert!(matches!(
            point.value,
            SampleValue::Value(Number::Double(value)) if value.is_finite() && value >= 0.0
        ));
    }
    assert!(project_sample(&configs, &sample).unwrap().is_some());
}

/// Scenario: The local Windows providers expose system elapsed time and one processor multi-timer.
/// Guarantees: Live readiness is verified for available counters while fixtures protect all four multi-timer formulas.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Windows performance counters; run explicitly with `-- --ignored`"]
async fn live_pdh_elapsed_and_multi_timer_counters() {
    let _serial = TEST_PDH_LOCK.lock().await;
    let configs = vec![
        counter(r"\System\System Up Time", "system.uptime"),
        counter(
            r"\Processor Information(_Total)\% Processor Utility",
            "system.cpu.utility",
        ),
    ];
    let mut query = open_live_query(configs.clone(), "elapsed-multi");
    let first = query.collect().unwrap();
    assert!(
        first.metadata.failures.is_empty(),
        "{:?}",
        first.metadata.failures
    );
    assert!(matches!(
        first.points[0].value,
        SampleValue::Value(Number::Double(value)) if value.is_finite() && value >= 0.0
    ));
    assert_eq!(first.points[1].value, SampleValue::Warming);

    std::thread::sleep(Duration::from_millis(250));
    let second = query.collect().unwrap();
    assert!(
        second.metadata.failures.is_empty(),
        "{:?}",
        second.metadata.failures
    );
    assert!(second.points.iter().all(|point| {
        matches!(
            point.value,
            SampleValue::Value(Number::Double(value)) if value.is_finite() && value >= 0.0
        )
    }));
    assert!(project_sample(&configs, &second).unwrap().is_some());
}
