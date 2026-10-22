// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: Real query sampling produces two localized members, an exact Sum, and a later independent observation.
/// Guarantees: Decoded OTAP retains grouped metadata, concrete identity, static precedence, collection times and query-owned Sum start.
#[test]
fn query_producer_to_decoded_otap_preserves_observations() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};

    let config = Config::from_json(&serde_json::json!({
        "metrics": {
            "process.id": {"description": "Process identity", "unit": "1", "gauge": {}},
            "memory": {"description": "Available memory", "unit": "By",
                "sum": {"aggregation": "cumulative", "monotonic": false}}
        },
        "perfcounters": [
            {"object": "Process", "instances": "worker*",
                "counters": [{"name": "ID Process", "metric": "process.id",
                    "attributes": {" state ": " running ", "instance": "configured"}}]},
            {"object": "Memory", "counters": [{"name": "Available Bytes", "metric": "memory"},
                {"name": "missing"}]}
        ]
    }))
    .unwrap();
    let template = config.counters[0].path.clone();
    let paths = [
        r"\Prozess(worker)\Prozesskennung",
        r"\Prozess(worker#1)\Prozesskennung",
    ];
    let exact = r"\Memory\Available Bytes";
    let (state, mut query) = fake_query(
        config.counters.clone(),
        vec![
            fake_counter(paths[0], Number::Integer(40)),
            fake_counter(paths[1], Number::Integer(41)),
            fake_counter(exact, Number::Integer(42)),
        ],
    );
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(template.clone(), paths.map(str::to_owned).to_vec());
        state.timestamp = 110;
        state.formatted_timestamp = Some(900);
    }
    let mut sample = query.collect().unwrap();
    assert_eq!(sample.timestamp_unix_nano, 110);
    assert_eq!(sample.points.len(), 3);
    assert_eq!(sample.metadata.failures.len(), 1);
    assert_eq!(
        sample.metadata.failures[0].path.as_ref(),
        r"\Memory\missing"
    );
    assert_eq!(state.borrow().timestamp, 900);
    assert!(
        sample
            .points
            .iter()
            .all(|point| point.timestamp_unix_nano == 110)
    );
    let exact_point = sample
        .points
        .iter()
        .find(|point| point.counter_index == 1)
        .unwrap();
    assert_eq!(exact_point.path.as_ref(), exact);
    assert_eq!(exact_point.start_time_unix_nano, Some(100));
    let later_path = r"\Prozess(worker-later)\Prozesskennung";
    let (later_state, mut later_query) = fake_query(
        config.counters[..1].to_vec(),
        vec![fake_counter(later_path, Number::Integer(99))],
    );
    {
        let mut state = later_state.borrow_mut();
        let _ = state
            .expansions
            .insert(template.clone(), vec![later_path.to_owned()]);
        state.timestamp = 220;
    }
    let later = later_query.collect().unwrap();
    sample.timestamp_unix_nano = later.timestamp_unix_nano;
    sample.members.extend(later.members);
    sample.points.extend(later.points);
    let records = into_otap(&config.counters, sample).unwrap().unwrap();
    let decoded = decode(records);
    let resource = &decoded.resource_metrics[0];
    assert!(
        resource
            .resource
            .as_ref()
            .unwrap()
            .attributes
            .iter()
            .any(|attribute| attribute.key == "os.type"
                && attribute.value.as_ref().unwrap().value
                    == Some(any_value::Value::StringValue("windows".to_owned())))
    );
    let scope = &resource.scope_metrics[0];
    assert_eq!(
        scope.scope.as_ref().unwrap().name,
        "otel-arrow-dfe-contrib-nodes/windows_perf_counters"
    );
    assert_eq!(scope.metrics.len(), 2);
    let process = scope
        .metrics
        .iter()
        .find(|metric| metric.name == "process.id")
        .unwrap();
    assert_eq!(process.description, "Process identity");
    assert_eq!(process.unit, "1");
    let metric::Data::Gauge(gauge) = process.data.as_ref().unwrap() else {
        panic!("expected Gauge")
    };
    assert_eq!(gauge.data_points.len(), 3);
    for (path, name, index, time, value) in [
        (paths[0], "worker", "0", 110, 40),
        (paths[1], "worker", "1", 110, 41),
        (later_path, "worker-later", "0", 220, 99),
    ] {
        let point = gauge
            .data_points
            .iter()
            .find(|point| {
                point.attributes.iter().any(|attribute| {
                    attribute.key == "windows.perf_counter.path"
                        && attribute.value.as_ref().unwrap().value
                            == Some(any_value::Value::StringValue(path.to_owned()))
                })
            })
            .unwrap();
        let attributes = point
            .attributes
            .iter()
            .map(|attribute| {
                let Some(any_value::Value::StringValue(value)) = attribute
                    .value
                    .as_ref()
                    .and_then(|value| value.value.as_ref())
                else {
                    panic!("expected string")
                };
                (attribute.key.as_str(), value.as_str())
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            attributes,
            BTreeMap::from([
                (" state ", " running "),
                ("instance", "configured"),
                ("windows.perf_counter.path", path),
                ("windows.perf_counter.path_template", template.as_str()),
                ("windows.perf_counter.instance", name),
                ("windows.perf_counter.instance_index", index),
            ])
        );
        assert_eq!(point.time_unix_nano, time);
        assert_eq!(point.start_time_unix_nano, 0);
        assert_eq!(point.value, Some(number_data_point::Value::AsInt(value)));
    }
    let memory = scope
        .metrics
        .iter()
        .find(|metric| metric.name == "memory")
        .unwrap();
    let metric::Data::Sum(sum) = memory.data.as_ref().unwrap() else {
        panic!("expected Sum")
    };
    assert_eq!(memory.description, "Available memory");
    assert_eq!(memory.unit, "By");
    assert_eq!(sum.aggregation_temporality, 2);
    assert!(!sum.is_monotonic);
    assert_eq!(sum.data_points.len(), 1);
    assert_eq!(sum.data_points[0].start_time_unix_nano, 100);
    assert_eq!(sum.data_points[0].time_unix_nano, 110);
    assert_eq!(
        sum.data_points[0].value,
        Some(number_data_point::Value::AsInt(42))
    );
}

/// Scenario: An unsupported native type is reported for a configured counter.
/// Guarantees: The contextual sample failure retains the actual path rather than the worker placeholder.
#[test]
fn unsupported_type_failure_retains_counter_path() {
    let path = r"\Object\Unsupported";
    let error = Error::UnsupportedType(counter_type::UnsupportedNativeType {
        path: path.to_owned(),
        native_type: u32::MAX,
    });
    let failure = error::sample_failure(0, &error);
    assert_eq!(failure.path.as_ref(), path);
    assert_eq!(failure.reason, "unsupported_type");
}
use windows_sys::Win32::System::Performance::PDH_INVALID_HANDLE;

struct InvalidHandleState {
    next_handle: usize,
    paths: BTreeMap<usize, String>,
    active: BTreeSet<usize>,
    max_active: usize,
    add_attempts: Vec<String>,
    cleanup_attempts: Vec<usize>,
    invalid_handles: BTreeSet<usize>,
    failed_adds_remaining: usize,
    cleanup_fails: bool,
    timestamp: i64,
    raw_value: i64,
    no_observation: BTreeSet<String>,
    expansions: BTreeMap<String, Vec<String>>,
    expand_calls: usize,
    collect_status: Option<u32>,
    invalidate_on_collect: bool,
}

struct InvalidHandlePdh(Rc<RefCell<InvalidHandleState>>);

impl PdhApi for InvalidHandlePdh {
    type Counter = usize;

    fn add(&mut self, path: &str, _language: CounterLanguage) -> Result<Self::Counter, Error> {
        let mut state = self.0.borrow_mut();
        state.add_attempts.push(path.to_owned());
        if state.failed_adds_remaining > 0 {
            state.failed_adds_remaining -= 1;
            return Err(Error::Pdh {
                operation: PdhOperation::AddEnglishCounter,
                path: path.to_owned(),
                status: 1,
            });
        }
        let handle = state.next_handle;
        state.next_handle += 1;
        let _ = state.paths.insert(handle, path.to_owned());
        let _ = state.active.insert(handle);
        state.max_active = state.max_active.max(state.active.len());
        Ok(handle)
    }

    fn remove(&mut self, counter: &Self::Counter, path: &str) -> Result<(), Error> {
        let mut state = self.0.borrow_mut();
        state.cleanup_attempts.push(*counter);
        if state.cleanup_fails {
            return Err(Error::Pdh {
                operation: PdhOperation::RemoveCounter,
                path: path.to_owned(),
                status: 1,
            });
        }
        let _ = state.active.remove(counter);
        Ok(())
    }

    fn counter_type(&mut self, _counter: &Self::Counter, path: &str) -> Result<u32, Error> {
        if path.contains("Operations Delta") {
            Ok(0x0040_0500)
        } else if path.contains("Available Bytes") {
            Ok(0x0001_0100)
        } else {
            Ok(0x2041_0500)
        }
    }

    fn full_path(&mut self, _counter: &Self::Counter, path: &str) -> Result<String, Error> {
        Ok(path.to_owned())
    }

    fn comparison_path(&mut self, path: &str) -> Result<String, Error> {
        NativePdh::comparison_path_fixture(path)
    }

    fn collect(&mut self) -> Result<bool, Error> {
        let mut state = self.0.borrow_mut();
        if let Some(status) = state.collect_status {
            return Err(Error::Pdh {
                operation: PdhOperation::CollectQueryData,
                path: "<query>".to_owned(),
                status,
            });
        }
        if state.invalidate_on_collect {
            let handles = state.active.clone();
            state.invalid_handles.extend(handles);
            state.invalidate_on_collect = false;
        }
        Ok(true)
    }

    fn raw(&mut self, counter: &Self::Counter, path: &str) -> Result<Option<RawSample>, Error> {
        let state = self.0.borrow();
        if state.invalid_handles.contains(counter) {
            return Err(Error::Pdh {
                operation: PdhOperation::GetRawCounterValue,
                path: path.to_owned(),
                status: PDH_INVALID_HANDLE,
            });
        }
        if state.no_observation.contains(path) {
            return Ok(None);
        }
        Ok(Some(RawSample {
            second_value: state.raw_value,
            multi_count: 1,
        }))
    }

    fn formatted(
        &mut self,
        counter: &Self::Counter,
        path: &str,
        format: ValueFormat,
    ) -> Result<Option<Number>, Error> {
        let state = self.0.borrow();
        if state.invalid_handles.contains(counter) {
            return Err(Error::Pdh {
                operation: PdhOperation::GetFormattedCounterValue,
                path: path.to_owned(),
                status: PDH_INVALID_HANDLE,
            });
        }
        if state.no_observation.contains(path) {
            return Ok(None);
        }
        Ok(Some(match format {
            ValueFormat::Integer => Number::Integer(state.raw_value),
            ValueFormat::Double => Number::Double(state.raw_value as f64),
        }))
    }

    fn timestamp(&mut self) -> Result<i64, Error> {
        Ok(self.0.borrow().timestamp)
    }

    fn refresh_objects(&mut self) -> Result<usize, Error> {
        Ok(0)
    }

    fn expand(&mut self, path: &str, budget: &mut DiscoveryBudget) -> Result<Vec<String>, Error> {
        self.0.borrow_mut().expand_calls += 1;
        let state = self.0.borrow();
        let paths = state.expansions.get(path).map_or(&[][..], Vec::as_slice);
        budget.reserve(paths.iter().map(String::len).sum())?;
        Ok(paths.to_vec())
    }

    fn parse_instance(&mut self, path: &str) -> Result<InstanceIdentity, Error> {
        NativePdh::parse_instance_path(path)
    }
}

fn invalid_handle_query(
    configs: Vec<CounterConfig>,
    expansions: BTreeMap<String, Vec<String>>,
) -> (Rc<RefCell<InvalidHandleState>>, Query<InvalidHandlePdh>) {
    let state = invalid_handle_state(expansions);
    let query = Query::open_with_seed(
        InvalidHandlePdh(Rc::clone(&state)),
        configs,
        Duration::ZERO,
        Duration::ZERO,
        256,
        4_096,
        None,
    )
    .unwrap();
    (state, query)
}

fn invalid_handle_state(
    expansions: BTreeMap<String, Vec<String>>,
) -> Rc<RefCell<InvalidHandleState>> {
    Rc::new(RefCell::new(InvalidHandleState {
        next_handle: 0,
        paths: BTreeMap::new(),
        active: BTreeSet::new(),
        max_active: 0,
        add_attempts: Vec::new(),
        cleanup_attempts: Vec::new(),
        invalid_handles: BTreeSet::new(),
        failed_adds_remaining: 0,
        cleanup_fails: false,
        timestamp: 100,
        raw_value: 10,
        no_observation: BTreeSet::new(),
        expansions,
        expand_calls: 0,
        collect_status: None,
        invalidate_on_collect: false,
    }))
}

fn handle_for_path(state: &InvalidHandleState, path: &str) -> usize {
    state
        .paths
        .iter()
        .find_map(|(handle, candidate)| (candidate == path).then_some(*handle))
        .unwrap()
}

/// Scenario: An admitted wildcard fails retirement beside a healthy peer, then the query itself becomes invalid.
/// Guarantees: Maintenance never inherits scheduled-success evidence, while the retained seed restores membership without rediscovery and preserves the logical Sum start.
#[test]
fn failed_wildcard_retirement_preserves_rebuild_seed() {
    let template = r"\Process(worker*)\Operations Delta";
    let path = r"\Process(worker)\Operations Delta";
    let healthy = r"\Memory\Available Bytes";
    for fail_replacement_open in [false, true] {
        let state = invalid_handle_state(BTreeMap::from([(
            template.to_owned(),
            vec![path.to_owned()],
        )]));
        let mut config = cumulative_sum(template, "operations", "{operation}");
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        let opens = Cell::new(0);
        let mut open = |configs, limit, seed: Option<&QueryRecreationSeed>| {
            let attempt = opens.get();
            opens.set(attempt + 1);
            // This adapter does not model query destruction; emulate release before replacement.
            state.borrow_mut().active.clear();
            if fail_replacement_open && attempt == 1 {
                return Err(Error::Pdh {
                    operation: PdhOperation::OpenQuery,
                    path: "<query>".to_owned(),
                    status: 1,
                });
            }
            Query::open_with_seed(
                InvalidHandlePdh(Rc::clone(&state)),
                configs,
                Duration::ZERO,
                Duration::from_secs(3600),
                256,
                limit,
                seed,
            )
        };
        let retry_cap = Duration::from_secs(60);
        let mut slot = QuerySlot::open_initial(
            vec![config, counter(healthy, "memory")],
            retry_cap,
            2,
            &mut open,
        )
        .unwrap();
        let handle = handle_for_path(&state.borrow(), path);
        {
            let mut state = state.borrow_mut();
            let _ = state.invalid_handles.insert(handle);
            state.cleanup_fails = true;
            state.timestamp = 110;
        }
        let failed_at = Instant::now();
        let failed_cleanup = collect_query(&mut slot, &mut open, failed_at).unwrap();
        assert_eq!(failed_cleanup.points.len(), 1);
        assert_eq!(failed_cleanup.points[0].path.as_ref(), healthy);
        assert!(slot.query.is_some());
        state.borrow_mut().collect_status = Some(PDH_INVALID_HANDLE);
        assert!(
            collect_query(&mut slot, &mut open, failed_at)
                .unwrap_err()
                .is_invalid_query()
        );
        state.borrow_mut().collect_status = None;
        assert!(slot.query.is_none());
        let backoff = collect_query(&mut slot, &mut open, failed_at).unwrap();
        assert!(!backoff.metadata.diagnostics.any_query_collected);
        assert!(backoff.points.is_empty());
        assert_eq!(backoff.members.len(), 2);
        assert!(!backoff.metadata.failures.is_empty());
        assert_eq!(opens.get(), 1);
        state.borrow_mut().cleanup_fails = false;
        let mut rebuild_at = failed_at + Duration::from_secs(1);
        if fail_replacement_open {
            let failed = collect_query(&mut slot, &mut open, rebuild_at).unwrap();
            assert!(!failed.metadata.diagnostics.any_query_collected);
            assert!(failed.points.is_empty());
            assert!(
                failed.metadata.failures.iter().any(|failure| {
                    failure.reason == "PdhOpenQueryW" && failure.diagnostic_only
                })
            );
            rebuild_at += Duration::from_secs(1);
        }
        let rebuilt = collect_query(&mut slot, &mut open, rebuild_at).unwrap();
        assert!(!rebuilt.metadata.diagnostics.any_query_collected);
        assert!(rebuilt.points.is_empty());
        assert_eq!(rebuilt.metadata.diagnostics.query_rebuild_recoveries, 1);
        state.borrow_mut().timestamp = 120;
        let ready = collect_query(&mut slot, &mut open, rebuild_at).unwrap();
        assert_eq!(ready.points.len(), 2);
        assert_eq!(ready.points[0].path.as_ref(), path);
        assert_eq!(ready.points[0].start_time_unix_nano, Some(100));
        assert!(matches!(ready.points[0].value, SampleValue::Value(_)));
        assert_eq!(state.borrow().expand_calls, 1);
        assert_eq!(
            state.borrow().add_attempts,
            [
                healthy.to_owned(),
                path.to_owned(),
                healthy.to_owned(),
                path.to_owned()
            ]
        );
        assert_eq!(state.borrow().max_active, 2);
    }
}

/// Scenario: A wildcard lookup succeeds or fails before temporary-handle cleanup also fails.
/// Guarantees: Healthy members survive and warnings remain once per episode; only a genuine lookup failure counts discovery failure and enables object refresh.
#[test]
fn lookup_failure_with_pending_cleanup_preserves_discovery_classification() {
    let template = r"\Process(*)\Private Bytes";
    let member = r"\Process(worker)\Private Bytes";
    let healthy = r"\Memory\Available Bytes";
    for lookup_failed in [false, true] {
        let mut wildcard = counter(template, "private");
        wildcard.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        let (state, mut query) = fake_query(
            vec![wildcard, counter(healthy, "memory")],
            vec![
                fake_counter(member, Number::Integer(1)),
                fake_counter(healthy, Number::Integer(2)),
            ],
        );
        let _ = state
            .borrow_mut()
            .expansions
            .insert(template.to_owned(), vec![member.to_owned()]);
        query.refresh_all_wildcards_managed().unwrap();
        {
            let mut state = state.borrow_mut();
            state.expand_error = true;
            state.expand_operation = PdhOperation::RemoveCounter;
            state.lookup_failure_on_cleanup = lookup_failed;
        }
        query.last_object_refresh = Instant::now() - Duration::from_secs(601);
        for attempt in 0..2 {
            query.last_wildcard_refresh = Instant::now() - Duration::from_secs(31);
            let sample = query.collect().unwrap();
            assert_eq!(sample.points.len(), 2);
            assert_eq!(
                sample.metadata.diagnostics.discovery_failures,
                u64::from(lookup_failed)
            );
            assert_eq!(
                sample
                    .metadata
                    .failures
                    .iter()
                    .filter(|failure| failure.reason == "selector_unresolved")
                    .count(),
                usize::from(attempt == 0)
            );
            assert_eq!(
                state.borrow().calls.refresh,
                usize::from(lookup_failed && attempt == 1)
            );
        }
        assert!(state.borrow().removed.is_empty());
    }
}

/// Scenario: Replacement priming fails either before collection or after successful collection during invalid-handle cleanup.
/// Guarantees: Both failures produce diagnostic-only maintenance, no points or scheduled success, and no duplicated failures on the following request.
#[test]
fn failed_rebuild_prime_is_diagnostic_only() {
    let path = r"\Memory\Available Bytes";
    for fail_after_collection in [false, true] {
        let state = invalid_handle_state(BTreeMap::new());
        let opens = Cell::new(0);
        let mut open = |configs, limit, seed: Option<&QueryRecreationSeed>| {
            let attempt = opens.get();
            opens.set(attempt + 1);
            {
                let mut state = state.borrow_mut();
                state.active.clear();
                state.collect_status = (attempt == 1 && !fail_after_collection).then_some(1);
                state.invalidate_on_collect = attempt == 1 && fail_after_collection;
                state.cleanup_fails = attempt == 1 && fail_after_collection;
            }
            Query::open_with_seed(
                InvalidHandlePdh(Rc::clone(&state)),
                configs,
                Duration::ZERO,
                Duration::from_secs(3600),
                256,
                limit,
                seed,
            )
        };
        let mut slot =
            QuerySlot::open_initial(vec![counter(path, "memory")], Duration::ZERO, 1, &mut open)
                .unwrap();
        state.borrow_mut().collect_status = Some(PDH_INVALID_HANDLE);
        assert!(
            collect_query(&mut slot, &mut open, Instant::now())
                .unwrap_err()
                .is_invalid_query()
        );
        let failed_prime = collect_query(&mut slot, &mut open, Instant::now()).unwrap();
        assert!(failed_prime.points.is_empty());
        assert!(!failed_prime.metadata.diagnostics.any_query_collected);
        assert!(failed_prime.metadata.diagnostics.query_pending);
        assert!(!failed_prime.metadata.failures.is_empty());
        assert!(
            failed_prime
                .metadata
                .failures
                .iter()
                .all(|failure| failure.diagnostic_only)
        );
        state.borrow_mut().collect_status = None;
        state.borrow_mut().invalidate_on_collect = false;
        state.borrow_mut().cleanup_fails = false;
        state.borrow_mut().timestamp = 120;
        let ready = collect_query(&mut slot, &mut open, Instant::now()).unwrap();
        assert!(ready.metadata.diagnostics.any_query_collected);
        assert!(ready.metadata.failures.is_empty());
        assert_eq!(ready.points.len(), 1);
    }
}

/// Scenario: One configured mapping expands multiple patterns whose list order conflicts with concrete-key order.
/// Guarantees: Global admission compares concrete keys across that mapping's patterns and is independent of the pattern list order.
#[test]
fn same_mapping_patterns_share_concrete_key_admission_order() {
    let alpha = r"\Process(alpha)\Private Bytes";
    let zeta = r"\Process(zeta)\Private Bytes";
    for patterns in [vec!["z*", "a*"], vec!["a*", "z*"]] {
        let config = Config::from_json(&serde_json::json!({
            "metrics": { "private": { "gauge": {} } },
            "perfcounters": [{ "object": "Process", "instances": patterns,
                "counters": [{ "name": "Private Bytes", "metric": "private" }] }]
        }))
        .unwrap();
        assert_eq!(
            config.counters[0].source_mapping_index,
            config.counters[1].source_mapping_index
        );
        let (state, mut query) = fake_query_with_seed(
            config.counters,
            [alpha, zeta]
                .into_iter()
                .map(|path| fake_counter(path, Number::Integer(1)))
                .collect(),
            Duration::ZERO,
            Duration::ZERO,
            1,
            None,
        );
        {
            let mut state = state.borrow_mut();
            let _ = state.expansions.insert(
                r"\Process(a*)\Private Bytes".to_owned(),
                vec![alpha.to_owned()],
            );
            let _ = state.expansions.insert(
                r"\Process(z*)\Private Bytes".to_owned(),
                vec![zeta.to_owned()],
            );
        }
        query.refresh_all_wildcards_managed().unwrap();
        let sample = query.collect().unwrap();
        assert_eq!(sample.points.len(), 1);
        assert_eq!(sample.points[0].path.as_ref(), alpha);
    }
}

/// Scenario: A retained member is rediscovered by an overlapping pattern from its source mapping.
/// Guarantees: The original pattern retains ownership when still matching, so deduplication cannot move its quota charge and exceed a per-pattern limit.
#[test]
fn overlapping_patterns_preserve_retained_quota_ownership() {
    let alpha = r"\Process(alpha)\Private Bytes";
    let zeta = r"\Process(zeta)\Private Bytes";
    let config = Config::from_json(&serde_json::json!({
        "perfcounters": [{ "object": "Process", "instances": ["z*", "*a"],
            "counters": [{ "name": "Private Bytes" }] }]
    }))
    .unwrap();
    let state = fake_state(
        [alpha, zeta]
            .into_iter()
            .map(|path| fake_counter(path, Number::Integer(1)))
            .collect(),
    );
    let _ = state.borrow_mut().expansions.insert(
        r"\Process(*a)\Private Bytes".to_owned(),
        vec![zeta.to_owned()],
    );
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        config.counters,
        Duration::ZERO,
        Duration::ZERO,
        1,
        2,
        None,
    )
    .unwrap();
    query.refresh_all_wildcards_managed().unwrap();
    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(
            r"\Process(z*)\Private Bytes".to_owned(),
            vec![zeta.to_owned()],
        );
        let _ = state.expansions.insert(
            r"\Process(*a)\Private Bytes".to_owned(),
            vec![alpha.to_owned(), zeta.to_owned()],
        );
    }
    query.refresh_all_wildcards_managed().unwrap();
    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 1);
    assert_eq!(sample.points[0].counter_index, 1);
    assert_eq!(sample.points[0].path.as_ref(), zeta);
    assert_eq!(state.borrow().added, [zeta.to_owned()]);
    assert_eq!(sample.metadata.diagnostics.instances_omitted_over_limit, 2);
}

/// Scenario: One wildcard delta counter's native handle becomes invalid beside a healthy calculated counter.
/// Guarantees: Only the invalid handle retires, retries stay bounded, and replacement re-warms native sampling without changing the cumulative start.
#[test]
fn invalid_handle_retries_one_counter_with_fresh_history() {
    let wildcard = r"\Process(worker*)\Operations Delta";
    let expanded = r"\Process(worker)\Operations Delta";
    let healthy = r"\Processor(_Total)\% Processor Time";
    let affected = CounterConfig {
        instance_config: InstanceConfig::Wildcard {
            excluded_aggregation: Some("_Total".to_owned()),
        },
        ..cumulative_sum(wildcard, "process.operations", "{operation}")
    };
    let (state, mut query) = invalid_handle_query(
        vec![affected, counter(healthy, "system.cpu.time")],
        BTreeMap::from([(wildcard.to_owned(), vec![expanded.to_owned()])]),
    );

    query.prime().unwrap();
    let old_handle = handle_for_path(&state.borrow(), expanded);
    {
        let mut state = state.borrow_mut();
        let _ = state.invalid_handles.insert(old_handle);
        state.timestamp = 110;
        state.raw_value = 20;
    }

    let retired = query.collect().unwrap();
    assert_eq!(retired.points.len(), 1);
    assert_eq!(retired.points[0].counter_index, 1);
    assert!(matches!(
        retired.points[0].value,
        SampleValue::Value(Number::Double(20.0))
    ));
    assert_eq!(retired.metadata.failures[0].counter_index, 0);
    {
        let state = state.borrow();
        assert_eq!(state.cleanup_attempts, [old_handle]);
        assert_eq!(state.active.len(), 1);
        assert_eq!(state.max_active, 2);
    }

    state.borrow_mut().failed_adds_remaining = 2;
    for (timestamp, raw_value, expected_attempts) in [(120, 30, 3), (130, 40, 4)] {
        {
            let mut state = state.borrow_mut();
            state.timestamp = timestamp;
            state.raw_value = raw_value;
        }
        let retry = query.collect().unwrap();
        assert_eq!(retry.points.len(), 1);
        assert_eq!(retry.points[0].counter_index, 1);
        assert!(matches!(
            retry.points[0].value,
            SampleValue::Value(Number::Double(value)) if value == raw_value as f64
        ));
        assert_eq!(retry.metadata.diagnostics.retry_attempts, 1);
        assert_eq!(retry.metadata.diagnostics.counter_add_failures, 1);
        let state = state.borrow();
        assert_eq!(state.add_attempts.len(), expected_attempts);
        assert_eq!(state.active.len(), 1);
        assert_eq!(state.max_active, 2);
    }

    {
        let mut state = state.borrow_mut();
        state.timestamp = 140;
        state.raw_value = 50;
    }
    let warming = query.collect().unwrap();
    assert!(
        warming.points.iter().any(|point| {
            point.counter_index == 0 && matches!(point.value, SampleValue::Warming)
        })
    );
    assert!(warming.points.iter().any(|point| {
        point.counter_index == 1 && matches!(point.value, SampleValue::Value(Number::Double(50.0)))
    }));
    assert_eq!(warming.metadata.diagnostics.retry_attempts, 1);
    assert_eq!(warming.metadata.diagnostics.retry_recoveries, 1);
    {
        let state = state.borrow();
        assert_eq!(state.active.len(), 2);
        assert_eq!(state.max_active, 2);
        let affected_adds = state
            .add_attempts
            .iter()
            .filter(|path| path.as_str() == expanded)
            .collect::<Vec<_>>();
        assert_eq!(affected_adds.len(), 4);
    }

    {
        let mut state = state.borrow_mut();
        state.timestamp = 150;
        state.raw_value = 60;
    }
    let recovered = query.collect().unwrap();
    let affected = recovered
        .points
        .iter()
        .find(|point| point.counter_index == 0)
        .unwrap();
    assert_eq!(affected.start_time_unix_nano, Some(100));
    assert!(matches!(
        affected.value,
        SampleValue::Value(Number::Double(60.0))
    ));
}

/// Scenario: An exact direct counter returns PDH_INVALID_HANDLE from its formatted read.
/// Guarantees: Formatted reads use the same typed retirement path and preserve exact English-add behavior on replacement.
#[test]
fn formatted_invalid_handle_retries_exact_counter() {
    let path = r"\Memory\Available Bytes";
    let (state, mut query) = invalid_handle_query(
        vec![counter(path, "system.memory.available")],
        BTreeMap::new(),
    );
    let old_handle = handle_for_path(&state.borrow(), path);
    let _ = state.borrow_mut().invalid_handles.insert(old_handle);

    let retired = query.collect().unwrap();
    assert!(retired.points.is_empty());
    {
        let state = state.borrow();
        assert_eq!(state.cleanup_attempts, [old_handle]);
        assert_eq!(state.active.len(), 0);
    }

    {
        let mut state = state.borrow_mut();
        state.timestamp = 110;
        state.raw_value = 42;
    }
    let recovered = query.collect().unwrap();
    assert!(matches!(
        recovered.points[0].value,
        SampleValue::Value(Number::Integer(42))
    ));
    assert_eq!(recovered.metadata.diagnostics.retry_attempts, 1);
    assert_eq!(recovered.metadata.diagnostics.retry_recoveries, 1);
    let state = state.borrow();
    assert_eq!(state.active.len(), 1);
    assert_eq!(state.max_active, 1);
    assert_eq!(state.add_attempts.len(), 2);
}

/// Scenario: A cumulative counter is re-added after its native handle becomes invalid.
/// Guarantees: Handle replacement re-warms native readiness without resetting the original query start.
#[test]
fn invalid_handle_readd_preserves_cumulative_sequence() {
    let path = r"\Memory\Available Bytes";
    let (state, mut query) = invalid_handle_query(
        vec![cumulative_sum(path, "system.memory.available", "By")],
        BTreeMap::new(),
    );
    query.prime().unwrap();
    {
        let mut state = state.borrow_mut();
        state.timestamp = 110;
        state.raw_value = 20;
    }
    let first = query.collect().unwrap();
    assert_eq!(first.points[0].start_time_unix_nano, Some(100));

    let old_handle = handle_for_path(&state.borrow(), path);
    {
        let mut state = state.borrow_mut();
        let _ = state.invalid_handles.insert(old_handle);
        state.timestamp = 120;
        state.raw_value = 25;
    }
    let omitted = query.collect().unwrap();
    assert!(omitted.points.is_empty());

    {
        let mut state = state.borrow_mut();
        state.timestamp = 130;
        state.raw_value = 30;
    }
    let recovered = query.collect().unwrap();
    assert_eq!(recovered.points[0].start_time_unix_nano, Some(100));
    assert!(matches!(
        recovered.points[0].value,
        SampleValue::Value(Number::Integer(30))
    ));
}

/// Scenario: A counter read produces normal no-observation results without an invalid native handle.
/// Guarantees: No-observation resets readiness without removing, re-adding, or retrying the counter.
#[test]
fn no_observation_does_not_recover_counter_handle() {
    let path = r"\Provider\Operations Delta";
    let (state, mut query) = invalid_handle_query(
        vec![cumulative_sum(path, "provider.operations", "{operation}")],
        BTreeMap::new(),
    );
    let _ = state.borrow_mut().no_observation.insert(path.to_owned());

    for timestamp in [110, 120] {
        state.borrow_mut().timestamp = timestamp;
        let sample = query.collect().unwrap();
        assert!(matches!(sample.points[0].value, SampleValue::NoObservation));
        assert_eq!(sample.metadata.diagnostics.retry_attempts, 0);
    }

    let state = state.borrow();
    assert_eq!(state.add_attempts.len(), 1);
    assert!(state.cleanup_attempts.is_empty());
    assert_eq!(state.active.len(), 1);
}

/// Scenario: Cleanup fails while retiring an invalid counter beside a healthy calculated peer.
/// Guarantees: Healthy points survive, both failures remain contextual, and bounded cleanup keeps ownership without discovery-failure accounting.
#[test]
fn invalid_handle_cleanup_failure_preserves_healthy_collection() {
    let path = r"\Provider\Operations Delta";
    let healthy = r"\Processor(_Total)\% Processor Time";
    let (state, mut query) = invalid_handle_query(
        vec![
            cumulative_sum(path, "provider.operations", "{operation}"),
            counter(healthy, "system.cpu.time"),
        ],
        BTreeMap::new(),
    );
    query.prime().unwrap();
    let handle = handle_for_path(&state.borrow(), path);
    {
        let mut state = state.borrow_mut();
        let _ = state.invalid_handles.insert(handle);
        state.cleanup_fails = true;
        state.timestamp = 110;
        state.raw_value = 20;
    }

    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 1);
    assert_eq!(sample.points[0].path.as_ref(), healthy);
    assert!(sample.metadata.diagnostics.any_query_collected);
    assert_eq!(sample.metadata.diagnostics.discovery_failures, 0);
    assert_eq!(sample.metadata.failures.len(), 2);
    let state = state.borrow();
    assert_eq!(state.cleanup_attempts, [handle]);
    assert_eq!(state.active, BTreeSet::from([handle, 1]));
    assert_eq!(state.max_active, 2);
    assert_eq!(state.add_attempts.len(), 2);
}

/// Scenario: Shutdown arrives during the first native aggregate-filter parse in startup expansion.
/// Guarantees: Setup stops before another parse, counter add or prime instead of walking the remaining expansion.
#[test]
fn startup_aggregate_filter_stops_between_native_parses() {
    let template = r"\Process(*)\Private Bytes";
    let paths =
        ["worker", "_Total", "service"].map(|name| format!(r"\Process({name})\Private Bytes"));
    let state = fake_state(
        paths
            .iter()
            .map(|path| fake_counter(path, Number::Integer(1)))
            .collect(),
    );
    state.borrow_mut().stop_after_parse = Some(Arc::new(AtomicBool::new(false)));
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), paths.to_vec());
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: Some("_Total".to_owned()),
    };
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        vec![config],
        Duration::from_secs(30),
        Duration::from_secs(60),
        256,
        4096,
        None,
    )
    .unwrap();
    let result = query.collect();
    assert!(matches!(result, Err(Error::WorkerStopRequested)));
    assert_eq!(state.borrow().calls.parse, 1);
    assert_eq!(state.borrow().calls.add, 0);
    assert_eq!(state.borrow().calls.collect, 0);
}

/// Scenario: A serving query observes a stop request before its next scheduled native collection.
/// Guarantees: The query returns a typed stop request, not a startup failure, without another collection or read.
#[test]
fn runtime_collection_stop_is_not_an_initialization_failure() {
    let path = r"\Memory\Available Bytes";
    let state = fake_state(vec![fake_counter(path, Number::Integer(42))]);
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        vec![counter(path, "memory")],
        Duration::from_secs(30),
        Duration::from_secs(60),
        256,
        4096,
        None,
    )
    .unwrap();
    state.borrow_mut().stop_after_add = Some(Arc::new(AtomicBool::new(true)));
    assert!(matches!(query.collect(), Err(Error::WorkerStopRequested)));
    assert_eq!(state.borrow().calls.collect, 0);
    assert_eq!(state.borrow().calls.formatted, 0);
}

/// Scenario: A wildcard add succeeds natively just as the worker stop flag is raised.
/// Guarantees: Stop escapes before failure accounting or retry admission, while query ownership still covers the newly added handle.
#[test]
fn wildcard_add_stop_is_not_a_counter_failure_or_retry() {
    let template = r"\Process(*)\Private Bytes";
    let expanded = r"\Process(worker)\Private Bytes";
    let mut config = counter(template, "private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_cadences(
        vec![config],
        vec![fake_counter(expanded, Number::Integer(1))],
        Duration::ZERO,
        Duration::ZERO,
    );
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(template.to_owned(), vec![expanded.to_owned()]);
        state.stop_after_add = Some(Arc::new(AtomicBool::new(false)));
    }
    assert!(query.refresh_all_wildcards_managed().unwrap_err().is_stop());
    assert_eq!(query.pending.diagnostics.counter_add_failures, 0);
    assert!(query.pending.failures.is_empty());
    assert!(query.admitted_members().is_empty());
    assert_eq!(state.borrow().calls.add, 1);
    assert_eq!(state.borrow().calls.counter_type, 0);
}

/// Scenario: A duplicate-instance group changes while removal of both departed and still-wanted native members fails.
/// Guarantees: Wanted ownership survives in the recreation seed, successful cleanup re-adds it, and failed deferred cleanup retains its reservation without escalating.
#[test]
fn wanted_pending_removal_retains_seed_and_cleanup_outcome() {
    let template = r"\Process(*)\Private Bytes";
    let wanted = r"\Process(worker)\Private Bytes";
    let departed = r"\Process(worker#1)\Private Bytes";
    for (cleanup_succeeds, discovery_fails) in
        [(false, false), (true, false), (false, true), (true, true)]
    {
        let mut config = counter(template, "private");
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        let (state, mut query) = fake_query_with_cadences(
            vec![config.clone()],
            [wanted, departed]
                .into_iter()
                .map(|path| FakeCounter {
                    native_type: 0x0040_0500,
                    raw: Some(10),
                    ..fake_counter(path, Number::Double(1.0))
                })
                .collect(),
            Duration::ZERO,
            Duration::ZERO,
        );
        let _ = state.borrow_mut().expansions.insert(
            template.to_owned(),
            vec![wanted.to_owned(), departed.to_owned()],
        );
        let _ = query.collect().unwrap();
        query.set_max_native_counters(2).unwrap();
        {
            let mut state = state.borrow_mut();
            let _ = state
                .expansions
                .insert(template.to_owned(), vec![wanted.to_owned()]);
            state.remove_error = true;
        }
        query.refresh_all_wildcards_managed().unwrap();
        assert_eq!(query.admitted_members().len(), 1);
        assert_eq!(query.admitted_members()[0].path.as_ref(), wanted);
        assert_eq!(query.pending.diagnostics.discovery_failures, 0);
        let seed = query.recreation_seed();
        let restored_state = fake_state(vec![fake_counter(wanted, Number::Integer(1))]);
        let restored = Query::open_with_seed(
            FakePdh::new(Rc::clone(&restored_state)),
            vec![config],
            Duration::ZERO,
            Duration::MAX,
            2,
            2,
            Some(&seed),
        )
        .unwrap();
        assert_eq!(restored.admitted_members().len(), 1);
        assert_eq!(restored.admitted_members()[0].path.as_ref(), wanted);
        assert_eq!(restored_state.borrow().calls.expand, 0);

        state.borrow_mut().expand_error = discovery_fails;
        query.refresh_all_wildcards_managed().unwrap();
        state.borrow_mut().expand_error = false;
        assert_eq!(query.active_counter_count(), 0);
        assert_eq!(query.admitted_members().len(), 1);
        assert!(query.pending.overflows.is_empty());
        assert_eq!(state.borrow().added.len(), 2);
        state.borrow_mut().remove_error = false;
        if cleanup_succeeds {
            let recovered = query.collect().unwrap();
            assert_eq!(recovered.points.len(), 1);
            assert_eq!(recovered.points[0].path.as_ref(), wanted);
            assert_eq!(recovered.points[0].value, SampleValue::Warming);
            assert_eq!(
                recovered.metadata.diagnostics.discovery_failures,
                u64::from(discovery_fails)
            );
            assert_eq!(state.borrow().added.len(), 3);
        } else {
            let _ = state.borrow_mut().remove_errors.insert(0);
            let failed = query.collect().unwrap();
            assert!(failed.points.is_empty());
            assert!(failed.metadata.failures.iter().any(|failure| {
                failure.reason == "PdhRemoveCounter" && failure.path.as_ref() == wanted
            }));
            assert_eq!(
                failed.metadata.diagnostics.discovery_failures,
                u64::from(discovery_fails)
            );
            assert_eq!(query.admitted_members()[0].path.as_ref(), wanted);
        }
    }
}

/// Scenario: A mandatory pending replacement disappears, then reappears before native cleanup succeeds.
/// Guarantees: Departure clears the rebuild target but not required retirement or its reservation; only a fresh handle can resume publication.
#[test]
fn departed_pending_replacement_cannot_restore_stale_handle() {
    let template = r"\Process(*)\Private Bytes";
    let wanted = r"\Process(worker)\Private Bytes";
    let departed = r"\Process(worker#1)\Private Bytes";
    for limit in [2, 3] {
        let mut config = counter(template, "private");
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        let (state, mut query) = fake_query_with_seed(
            vec![config],
            [wanted, departed]
                .into_iter()
                .map(|path| fake_counter(path, Number::Integer(1)))
                .collect(),
            Duration::ZERO,
            Duration::ZERO,
            limit,
            None,
        );
        let _ = state.borrow_mut().expansions.insert(
            template.to_owned(),
            vec![wanted.to_owned(), departed.to_owned()],
        );
        query.refresh_all_wildcards_managed().unwrap();
        let _ = query.collect().unwrap();
        {
            let mut state = state.borrow_mut();
            state.remove_error = true;
            let _ = state
                .expansions
                .insert(template.to_owned(), vec![wanted.to_owned()]);
        }
        query.refresh_all_wildcards_managed().unwrap();
        let _ = state
            .borrow_mut()
            .expansions
            .insert(template.to_owned(), Vec::new());
        query.refresh_all_wildcards_managed().unwrap();
        assert!(query.admitted_members().is_empty());
        assert_eq!(query.pending.diagnostics.instances_removed, 2);
        let _ = state
            .borrow_mut()
            .expansions
            .insert(template.to_owned(), vec![wanted.to_owned()]);
        query.refresh_all_wildcards_managed().unwrap();
        assert_eq!(query.active_counter_count(), 0);
        assert_eq!(query.admitted_members().len(), usize::from(limit == 3));
        state.borrow_mut().remove_error = false;
        assert_eq!(
            query.collect().unwrap().points.len(),
            usize::from(limit == 3)
        );
        query.refresh_all_wildcards_managed().unwrap();
        assert_eq!(query.collect().unwrap().points.len(), 1);
        assert_eq!(state.borrow().added.len(), 3);
    }
}

/// Scenario: A mandatory pending replacement changes pattern owner before its previous owner's discovery fails at full quota.
/// Guarantees: Admission, recreation seed and cleaned-up output use the selected owner without evicting its previous owner's healthy members or exceeding the native budget.
#[test]
fn pending_replacement_handoff_preserves_owner_and_pattern_quota() {
    let first = r"\Process(worker?)\Private Bytes";
    let second = r"\Process(*)\Private Bytes";
    let wanted = r"\Process(worker0)\Private Bytes";
    let duplicate = r"\Process(worker0#1)\Private Bytes";
    let healthy = [
        r"\Process(bravo)\Private Bytes",
        r"\Process(charlie)\Private Bytes",
    ];
    let configs = [first, second]
        .into_iter()
        .enumerate()
        .map(|(index, template)| {
            let mut config = counter(template, &format!("private.{index}"));
            config.instance_config = InstanceConfig::Wildcard {
                excluded_aggregation: None,
            };
            config
        })
        .collect::<Vec<_>>();
    let counters = || {
        [wanted, duplicate, healthy[0], healthy[1]]
            .into_iter()
            .map(|path| FakeCounter {
                native_type: 0x0040_0500,
                raw: Some(10),
                ..fake_counter(path, Number::Double(1.0))
            })
            .collect::<Vec<_>>()
    };
    let state = fake_state(counters());
    let _ = state.borrow_mut().expansions.insert(
        second.to_owned(),
        vec![wanted.to_owned(), duplicate.to_owned()],
    );
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs.clone(),
        Duration::ZERO,
        Duration::ZERO,
        2,
        3,
        None,
    )
    .unwrap();
    query.refresh_all_wildcards_managed().unwrap();
    let _ = query.collect().unwrap();
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(second.to_owned(), vec![wanted.to_owned()]);
        let _ = state.remove_errors.insert(0);
    }
    query.refresh_all_wildcards_managed().unwrap();
    assert_eq!(query.admitted_members()[0].counter_index, 1);
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(first.to_owned(), vec![wanted.to_owned()]);
        let _ = state.expansions.insert(
            second.to_owned(),
            healthy.iter().map(|path| (*path).to_owned()).collect(),
        );
    }
    query.refresh_all_wildcards_managed().unwrap();
    let expected = vec![
        (0, wanted.to_owned()),
        (1, healthy[0].to_owned()),
        (1, healthy[1].to_owned()),
    ];
    let members = |query: &Query<FakePdh>| {
        query
            .admitted_members()
            .into_iter()
            .map(|member| (member.counter_index, member.path.to_string()))
            .collect::<Vec<_>>()
    };
    assert_eq!(members(&query), expected);
    assert_eq!(query.active_counter_count(), 2);
    let seed = query.recreation_seed();
    let restored = Query::open_with_seed(
        FakePdh::new(fake_state(counters())),
        configs,
        Duration::ZERO,
        Duration::MAX,
        2,
        3,
        Some(&seed),
    )
    .unwrap();
    assert_eq!(members(&restored), expected);
    let _ = state.borrow_mut().expand_errors.insert(second.to_owned());
    query.refresh_all_wildcards_managed().unwrap();
    assert_eq!(members(&query), expected);
    assert!(query.pending.overflows.is_empty());
    state.borrow_mut().remove_errors.clear();
    let cleaned = query.collect().unwrap();
    assert_eq!(cleaned.points.len(), 3);
    let replacement = cleaned
        .points
        .iter()
        .find(|point| point.path.as_ref() == wanted)
        .unwrap();
    assert_eq!(replacement.counter_index, 0);
    assert_eq!(replacement.value, SampleValue::Warming);
}

/// Scenario: A departed member's removal fails twice and collection crosses each controlled cleanup deadline.
/// Guarantees: No removal or failure repeats before a deadline; cleanup retries exactly at the deadline and eventually releases ownership.
#[test]
fn pending_removal_waits_for_cleanup_backoff() {
    let template = r"\Process(*)\Private Bytes";
    let expanded = r"\Process(worker)\Private Bytes";
    let mut config = counter(template, "private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_seed(
        vec![config],
        vec![fake_counter(expanded, Number::Integer(1))],
        Duration::from_secs(60),
        Duration::ZERO,
        1,
        None,
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), vec![expanded.to_owned()]);
    let now = Instant::now();
    assert_eq!(query.collect_at(now).unwrap().points.len(), 1);
    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(template.to_owned(), Vec::new());
        state.remove_error = true;
    }
    let first = query.collect_at(now).unwrap();
    let deadline = now + Duration::from_secs(1);
    let second = query
        .collect_at(deadline - Duration::from_nanos(1))
        .unwrap();
    assert_eq!(state.borrow().removed, [0]);
    assert_eq!(first.metadata.diagnostics.discovery_failures, 0);
    assert_eq!(first.metadata.failures.len(), 1);
    assert_eq!(second.metadata.diagnostics.discovery_failures, 0);
    assert!(second.metadata.failures.is_empty());
    let retry = query.collect_at(deadline).unwrap();
    assert_eq!(state.borrow().removed, [0, 0]);
    assert_eq!(retry.metadata.failures.len(), 1);
    state.borrow_mut().remove_error = false;
    let next_deadline = deadline + Duration::from_secs(2);
    let waiting = query
        .collect_at(next_deadline - Duration::from_nanos(1))
        .unwrap();
    assert_eq!(state.borrow().removed, [0, 0]);
    assert!(waiting.metadata.failures.is_empty());
    let cleaned = query.collect_at(next_deadline).unwrap();
    assert_eq!(state.borrow().removed, [0, 0, 0]);
    assert!(cleaned.metadata.failures.is_empty());
    assert!(query.admitted_members().is_empty());
}

/// Scenario: Seventeen startup patterns each discover 257 available members under the fixed MVP bounds.
/// Guarantees: Each pattern admits at most 256 and the receiver installs at most 4096 entries with explicit overflow diagnostics.
#[test]
fn startup_admission_enforces_fixed_pattern_and_total_bounds() {
    let state = fake_state(Vec::new());
    let mut configs = Vec::new();
    for mapping in 0..17 {
        let template = format!(r"\Process(*)\C{mapping}");
        let paths = (0..257)
            .map(|instance| format!(r"\Process(p{instance:03})\C{mapping}"))
            .collect::<Vec<_>>();
        state.borrow_mut().counters.extend(
            paths
                .iter()
                .map(|path| fake_counter(path, Number::Integer(1))),
        );
        let _ = state
            .borrow_mut()
            .expansions
            .insert(template.clone(), paths);
        let mut config = counter(&template, &format!("metric{mapping}"));
        config.source_mapping_index = mapping;
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        configs.push(config);
    }
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        Duration::from_secs(30),
        Duration::from_secs(60),
        256,
        4096,
        None,
    )
    .unwrap();
    query.prime().unwrap();
    let members = query.admitted_members();
    assert_eq!(members.len(), 4096);
    for mapping in 0..17 {
        assert!(
            members
                .iter()
                .filter(|member| member.counter_index == mapping)
                .count()
                <= 256
        );
    }
    assert_eq!(state.borrow().added.len(), 4096);
    assert!(!query.pending.overflows.is_empty());
}

/// Scenario: Two metric mappings use the same exact PDH path.
/// Guarantees: The query adds two handles and routes one point to each configured mapping index.
#[test]
fn same_path_mappings_use_independent_query_entries() {
    let path = r"\Memory\Available Bytes";
    let first = counter(path, "windows.memory.available");
    let mut second = counter(path, "windows.memory.available_copy");
    second.source_mapping_index = 1;
    let configs = vec![first, second];
    let (state, mut query) = fake_query(configs, vec![fake_counter(path, Number::Integer(42))]);

    assert_eq!(state.borrow().added, [path.to_owned(), path.to_owned()]);
    let sample = query.collect().unwrap();
    assert_eq!(
        sample
            .points
            .iter()
            .map(|point| point.counter_index)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(
        sample
            .points
            .iter()
            .all(|point| matches!(point.value, SampleValue::Value(Number::Integer(42))))
    );
}

/// Scenario: One source mapping selects an exact instance and an overlapping English wildcard whose member differs by case and may include a machine prefix.
/// Guarantees: Both expansion forms reuse the exact handle through one lowercased, machine-neutral key.
#[test]
fn exact_and_overlapping_wildcard_share_native_key() {
    let exact_path = r"\Process(worker)\Private Bytes";
    let wildcard_path = r"\Process(*)\Private Bytes";
    for expanded_path in [
        r"\PROCESS(worker)\PRIVATE BYTES",
        r"\\MACHINE\PROCESS(worker)\PRIVATE BYTES",
    ] {
        let mut exact = counter(exact_path, "process.private");
        exact.instance_config = InstanceConfig::Exact {
            emit_attribute: true,
        };
        exact.source_mapping_index = 7;
        let mut wildcard = counter(wildcard_path, "process.private");
        wildcard.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        wildcard.source_mapping_index = 7;
        let (state, mut query) = fake_query_with_cadences(
            vec![exact, wildcard],
            vec![fake_counter(exact_path, Number::Integer(42))],
            Duration::ZERO,
            Duration::ZERO,
        );
        let _ = state
            .borrow_mut()
            .expansions
            .insert(wildcard_path.to_owned(), vec![expanded_path.to_owned()]);

        let sample = query.collect().unwrap();

        assert_eq!(state.borrow().added, [exact_path.to_owned()]);
        assert_eq!(query.active_expanded_count(), 0);
        assert_eq!(
            sample
                .points
                .iter()
                .map(|point| (point.counter_index, point.path.as_ref()))
                .collect::<Vec<_>>(),
            [(0, exact_path)]
        );
    }
}

/// Scenario: Earlier selector results consume refresh memory before a later selector expands.
/// Guarantees: Aggregate exhaustion retains the later selector's healthy members, diagnoses omission, and recovers with a fresh budget.
#[test]
fn refresh_memory_budget_spans_selectors_and_preserves_healthy_members() {
    let first = r"\First(*)\Value";
    let second = r"\Second(*)\Value";
    let first_old = r"\First(old)\Value";
    let first_new = r"\First(new)\Value";
    let second_old = r"\Second(old)\Value";
    let second_new = r"\Second(new)\Value";
    let healthy = r"\Memory\Available Bytes";
    let mut configs = vec![
        counter(first, "first"),
        counter(second, "second"),
        counter(healthy, "memory"),
    ];
    for (index, config) in configs.iter_mut().enumerate() {
        config.source_mapping_index = index;
        if index < 2 {
            config.instance_config = InstanceConfig::Wildcard {
                excluded_aggregation: None,
            };
        }
    }
    let state = fake_state(
        [first_old, first_new, second_old, second_new, healthy]
            .into_iter()
            .map(|path| fake_counter(path, Number::Integer(7)))
            .collect(),
    );
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(first.to_owned(), vec![first_old.to_owned()]);
        let _ = state
            .expansions
            .insert(second.to_owned(), vec![second_old.to_owned()]);
    }
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        Duration::ZERO,
        Duration::ZERO,
        1,
        3,
        None,
    )
    .unwrap();
    assert_eq!(query.collect().unwrap().points.len(), 3);
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(first.to_owned(), vec![first_new.to_owned()]);
        let _ = state
            .expansions
            .insert(second.to_owned(), vec![second_new.to_owned()]);
        for selector in [first, second] {
            let _ = state
                .decoded_expansion_bytes
                .insert(selector.to_owned(), DISCOVERY_REFRESH_BYTES / 2);
        }
    }
    let bounded = query.collect().unwrap();
    assert_eq!(
        bounded
            .points
            .iter()
            .map(|point| point.path.as_ref())
            .collect::<Vec<_>>(),
        [first_new, second_old, healthy],
    );
    assert_eq!(bounded.metadata.diagnostics.discovery_failures, 1);
    assert!(
        bounded
            .metadata
            .failures
            .iter()
            .any(|failure| failure.error.contains("128 MiB"))
    );
    state.borrow_mut().decoded_expansion_bytes.clear();
    let recovered = query.collect().unwrap();
    assert_eq!(
        recovered
            .points
            .iter()
            .map(|point| point.path.as_ref())
            .collect::<Vec<_>>(),
        [first_new, second_new, healthy],
    );
    assert_eq!(recovered.metadata.diagnostics.discovery_failures, 0);
}

fn same_mapping_overlap_configs(instances: [&str; 2]) -> Vec<CounterConfig> {
    let config = Config::from_json(&serde_json::json!({
        "metrics": {"shared": {"unit": "1", "gauge": {}}},
        "perfcounters": [{
            "object": "Process", "instances": instances,
            "counters": [{"name": "ID Process", "metric": "shared"}]
        }]
    }))
    .unwrap();
    assert_eq!(config.counters.len(), 2);
    assert_eq!(config.counters[0].name, config.counters[1].name);
    assert_eq!(
        config.counters[0].source_mapping_index,
        config.counters[1].source_mapping_index
    );
    config.counters
}

/// Scenario: Exact and expanded members share a native identity through suffix aliases and display-language spelling.
/// Guarantees: Exact paths win before either quota is charged; native add and output paths remain unchanged.
#[test]
fn same_mapping_native_identity_preserves_paths_and_capacity() {
    for (reported, overlap, peer) in [
        (
            r"\Process(worker#00)\ID Process",
            r"\Process(worker)\ID Process",
            r"\Process(worker#1)\ID Process",
        ),
        (
            r"\\LOCAL\NativeProcess(WORKER#00)\NativeCounter",
            r"\NativeProcess(worker)\NativeCounter",
            r"\NativeProcess(worker#1)\NativeCounter",
        ),
    ] {
        let configs = same_mapping_overlap_configs(["worker#00", "worker*"]);
        let exact = configs[0].path.clone();
        let template = configs[1].path.clone();
        let state = fake_state(vec![
            fake_counter(&exact, Number::Integer(42)),
            fake_counter(overlap, Number::Integer(99)),
            fake_counter(peer, Number::Integer(7)),
        ]);
        let _ = state
            .borrow_mut()
            .full_paths
            .insert(exact.clone(), reported.to_owned());
        let _ = state
            .borrow_mut()
            .expansions
            .insert(template, vec![overlap.to_owned(), peer.to_owned()]);
        let mut query = Query::open_with_seed(
            FakePdh::new(Rc::clone(&state)),
            configs,
            Duration::ZERO,
            Duration::ZERO,
            1,
            2,
            None,
        )
        .unwrap();
        for _ in 0..3 {
            let sample = query.collect().unwrap();
            assert_eq!(query.active_expanded_count(), 1);
            assert_eq!(
                sample
                    .points
                    .iter()
                    .map(|point| point.path.as_ref())
                    .collect::<Vec<_>>(),
                [exact.as_str(), peer]
            );
            assert!(sample.metadata.overflows.is_empty());
            assert!(sample.metadata.failures.is_empty());
        }
        assert_eq!(
            state.borrow().attempted_adds,
            [
                (exact, CounterLanguage::English),
                (peer.to_owned(), CounterLanguage::Localized)
            ]
        );
        assert!(state.borrow().removed.is_empty());
    }
}

/// Scenario: Different mappings select the same native instance through exact and wildcard paths.
/// Guarantees: Comparison-only dedupe does not change cross-mapping membership or emitted path identity.
#[test]
fn native_identity_preserves_cross_mapping_overlap() {
    let mut configs = same_mapping_overlap_configs(["0", "?"]);
    configs[1].source_mapping_index = 1;
    let exact = configs[0].path.clone();
    let template = configs[1].path.clone();
    let member = r"\NativeProcess(0)\NativeCounter";
    let state = fake_state(vec![
        fake_counter(&exact, Number::Integer(42)),
        fake_counter(member, Number::Integer(7)),
    ]);
    let _ = state
        .borrow_mut()
        .full_paths
        .insert(exact.clone(), member.to_owned());
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template, vec![member.to_owned()]);
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        Duration::ZERO,
        Duration::ZERO,
        1,
        2,
        None,
    )
    .unwrap();
    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 2);
    assert_eq!(sample.points[0].path.as_ref(), exact);
    assert_eq!(sample.points[1].path.as_ref(), member);
}

/// Scenario: Exact identity reads fail repeatedly, then recover while discovery and native removal fail.
/// Guarantees: Healthy data and configured-key dedupe survive, warnings are episode-bounded, and the overlapping member retires before collection with cleanup retained.
#[test]
fn identity_read_recovery_reconciles_before_failed_discovery_collection() {
    let configs = same_mapping_overlap_configs(["0", "?"]);
    let exact = configs[0].path.clone();
    let template = configs[1].path.clone();
    let member = r"\NativeProcess(0)\NativeCounter";
    let state = fake_state(vec![
        fake_counter(&exact, Number::Integer(42)),
        fake_counter(member, Number::Integer(7)),
    ]);
    {
        let mut state = state.borrow_mut();
        let _ = state.full_paths.insert(exact.clone(), member.to_owned());
        let _ = state
            .identity_errors
            .insert(exact.clone(), FakeIdentityError::Recoverable);
        let _ = state
            .expansions
            .insert(template, vec![exact.clone(), member.to_owned()]);
    }
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        Duration::from_secs(60),
        Duration::ZERO,
        2,
        3,
        None,
    )
    .unwrap();
    let now = Instant::now();
    for attempt in 0..3 {
        let sample = query.collect_at(now).unwrap();
        assert_eq!(sample.points.len(), 2);
        assert_eq!(sample.metadata.failures.len(), usize::from(attempt == 0));
        assert!(sample.metadata.failures.iter().all(|failure| {
            failure.reason == "native_identity_unavailable" && failure.diagnostic_only
        }));
        assert_eq!(sample.metadata.diagnostics.discovery_refreshes, 1);
        assert_eq!(sample.metadata.diagnostics.discovery_failures, 0);
    }
    {
        let mut state = state.borrow_mut();
        state.identity_errors.clear();
        state.expand_error = true;
        state.remove_error = true;
    }
    let recovered = query.collect_at(now).unwrap();
    assert_eq!(recovered.points.len(), 1);
    assert_eq!(recovered.points[0].path.as_ref(), exact);
    assert_eq!(query.active_expanded_count(), 0);
    assert_eq!(state.borrow().removed, [1]);
    state.borrow_mut().remove_error = false;
    let cleaned = query.collect_at(now + Duration::from_secs(1)).unwrap();
    assert_eq!(cleaned.points.len(), 1);
    assert_eq!(state.borrow().removed, [1, 1]);
    assert_eq!(state.borrow().added, [exact, member.to_owned()]);
}

/// Scenario: A missing exact source succeeds on retry after its native wildcard member was admitted.
/// Guarantees: The wildcard retires before that collection without waiting for another discovery or rewriting exact output.
#[test]
fn exact_add_retry_reconciles_native_overlap_without_discovery() {
    let configs = same_mapping_overlap_configs(["0", "?"]);
    let exact = configs[0].path.clone();
    let template = configs[1].path.clone();
    let member = r"\NativeProcess(0)\NativeCounter";
    let state = fake_state(vec![fake_counter(member, Number::Integer(7))]);
    let _ = state
        .borrow_mut()
        .full_paths
        .insert(exact.clone(), member.to_owned());
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template, vec![member.to_owned()]);
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        Duration::ZERO,
        Duration::MAX,
        1,
        2,
        None,
    )
    .unwrap();
    assert_eq!(query.collect().unwrap().points[0].path.as_ref(), member);
    let discoveries = state.borrow().calls.expand;
    {
        let mut state = state.borrow_mut();
        state
            .counters
            .push(fake_counter(&exact, Number::Integer(42)));
        state.expand_error = true;
    }
    let recovered = query
        .collect_at(Instant::now() + Duration::from_secs(60))
        .unwrap();
    assert_eq!(recovered.points.len(), 1);
    assert_eq!(recovered.points[0].path.as_ref(), exact);
    assert_eq!(recovered.metadata.diagnostics.retry_recoveries, 1);
    assert_eq!(state.borrow().calls.expand, discoveries);
    assert_eq!(state.borrow().removed, [0]);
}

/// Scenario: A rebuilt query re-adds exact sources before restoring wildcard members admitted during identity degradation.
/// Guarantees: Fresh native metadata suppresses the seeded overlap while retaining an unrelated seeded peer without rediscovery.
#[test]
fn query_rebuild_rechecks_identity_before_seeded_wildcard_restoration() {
    let configs = same_mapping_overlap_configs(["0", "?"]);
    let exact = configs[0].path.clone();
    let template = configs[1].path.clone();
    let member = r"\NativeProcess(0)\NativeCounter";
    let peer = r"\NativeProcess(1)\NativeCounter";
    let state = fake_state(vec![
        fake_counter(&exact, Number::Integer(42)),
        fake_counter(member, Number::Integer(7)),
        fake_counter(peer, Number::Integer(8)),
    ]);
    {
        let mut state = state.borrow_mut();
        let _ = state.full_paths.insert(exact.clone(), member.to_owned());
        let _ = state
            .identity_errors
            .insert(exact.clone(), FakeIdentityError::Recoverable);
        let _ = state
            .expansions
            .insert(template, vec![member.to_owned(), peer.to_owned()]);
    }
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs.clone(),
        Duration::ZERO,
        Duration::from_secs(3600),
        2,
        3,
        None,
    )
    .unwrap();
    assert_eq!(query.collect().unwrap().points.len(), 3);
    let seed = query.recreation_seed();
    drop(query);
    let discoveries = state.borrow().calls.expand;
    let reads = state.borrow().calls.full_path;
    {
        let mut state = state.borrow_mut();
        state.identity_errors.clear();
        state.expand_error = true;
        state.added.clear();
    }
    let mut rebuilt = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        Duration::ZERO,
        Duration::from_secs(3600),
        2,
        3,
        Some(&seed),
    )
    .unwrap();
    let sample = rebuilt.collect().unwrap();
    assert_eq!(sample.points.len(), 2);
    assert_eq!(state.borrow().added, [exact, peer.to_owned()]);
    assert_eq!(state.borrow().calls.full_path, reads + 1);
    assert_eq!(state.borrow().calls.expand, discoveries);
}

/// Scenario: Native metadata reads or comparison parsing request cancellation or rebuild during open or discovery retry.
/// Guarantees: These errors propagate before collection rather than becoming recoverable warnings.
#[test]
fn identity_reads_preserve_stop_and_rebuild_errors() {
    for comparison in [false, true] {
        for error in [FakeIdentityError::Stop, FakeIdentityError::Rebuild] {
            for retry in [false, true] {
                let configs = same_mapping_overlap_configs(["0", "?"]);
                let exact = configs[0].path.clone();
                let state = fake_state(vec![fake_counter(&exact, Number::Integer(42))]);
                {
                    let mut state = state.borrow_mut();
                    let errors = if comparison {
                        &mut state.comparison_errors
                    } else {
                        &mut state.identity_errors
                    };
                    let _ = errors.insert(
                        exact.clone(),
                        if retry {
                            FakeIdentityError::Recoverable
                        } else {
                            error
                        },
                    );
                }
                let opened = Query::open_with_seed(
                    FakePdh::new(Rc::clone(&state)),
                    configs,
                    Duration::ZERO,
                    Duration::ZERO,
                    1,
                    2,
                    None,
                );
                let actual = if retry {
                    let mut query = opened.unwrap();
                    {
                        let mut state = state.borrow_mut();
                        let errors = if comparison {
                            &mut state.comparison_errors
                        } else {
                            &mut state.identity_errors
                        };
                        let _ = errors.insert(exact, error);
                    }
                    query.collect().unwrap_err()
                } else {
                    opened.err().expect("identity error must fail opening")
                };
                match error {
                    FakeIdentityError::Stop => assert!(actual.is_stop()),
                    FakeIdentityError::Rebuild => {
                        assert!(actual.requires_immediate_query_rebuild())
                    }
                    FakeIdentityError::Recoverable => unreachable!(),
                }
                assert_eq!(state.borrow().calls.collect, 0);
            }
        }
    }
}

/// Scenario: An invalid exact handle retires before its wildcard alias is discovered and the exact add recovers.
/// Guarantees: Stale comparison identity cannot suppress healthy admission, and successful re-add reconciles that member before collection.
#[test]
fn exact_handle_recovery_refreshes_native_identity_ownership() {
    let configs = same_mapping_overlap_configs(["worker#00", "worker*"]);
    let exact = configs[0].path.clone();
    let template = configs[1].path.clone();
    let member = r"\Process(worker)\ID Process";
    let (state, mut query) = invalid_handle_query(
        configs,
        BTreeMap::from([(template, vec![member.to_owned()])]),
    );
    assert_eq!(query.collect().unwrap().points.len(), 1);
    state.borrow_mut().invalidate_on_collect = true;
    assert!(query.collect().unwrap().points.is_empty());
    query.refresh_all_wildcards_managed().unwrap();
    assert_eq!(query.active_expanded_count(), 1);
    let recovered = query
        .collect_at(Instant::now() + Duration::from_secs(60))
        .unwrap();
    assert_eq!(recovered.points.len(), 1);
    assert_eq!(recovered.points[0].path.as_ref(), exact);
    assert_eq!(query.active_expanded_count(), 0);
    assert_eq!(state.borrow().active.len(), 1);
    assert_eq!(state.borrow().cleanup_attempts.len(), 2);
}

/// Scenario: Many wildcard identity comparisons fail while their native counters remain readable.
/// Guarantees: Collection continues with configured-key dedupe, one warning per selector episode and no failed-refresh inflation.
#[test]
fn wildcard_identity_failures_keep_healthy_collection_with_bounded_warnings() {
    let configs = same_mapping_overlap_configs(["0", "?"]);
    let exact = configs[0].path.clone();
    let template = configs[1].path.clone();
    let state = fake_state(vec![fake_counter(&exact, Number::Integer(42))]);
    {
        let mut state = state.borrow_mut();
        let mut paths = vec![exact.clone()];
        for index in 1..=8 {
            let path = format!(r"\NativeProcess({index})\NativeCounter");
            state
                .counters
                .push(fake_counter(&path, Number::Integer(index)));
            let _ = state
                .comparison_errors
                .insert(path.clone(), FakeIdentityError::Recoverable);
            paths.push(path);
        }
        let _ = state.expansions.insert(template, paths);
    }
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        Duration::ZERO,
        Duration::ZERO,
        8,
        9,
        None,
    )
    .unwrap();
    for attempt in 0..3 {
        let sample = query.collect().unwrap();
        assert_eq!(sample.points.len(), 9);
        assert_eq!(sample.metadata.failures.len(), usize::from(attempt == 0));
        assert!(
            sample
                .metadata
                .failures
                .iter()
                .all(|failure| failure.diagnostic_only)
        );
        assert!(sample.metadata.overflows.is_empty());
        assert_eq!(sample.metadata.diagnostics.discovery_refreshes, 1);
        assert_eq!(sample.metadata.diagnostics.discovery_failures, 0);
    }
}

/// Scenario: A queued wildcard retry's comparison recovers and identifies an already-active exact counter.
/// Guarantees: The overlap is discarded without a native add, retry attempt, recovery count or extra discovery.
#[test]
fn wildcard_overlap_retry_is_discarded_without_counting_an_attempt() {
    let configs = same_mapping_overlap_configs(["0", "?"]);
    let exact = configs[0].path.clone();
    let template = configs[1].path.clone();
    let member = r"\NativeProcess(0)\NativeCounter";
    let state = fake_state(vec![fake_counter(&exact, Number::Integer(42))]);
    {
        let mut state = state.borrow_mut();
        let _ = state.full_paths.insert(exact.clone(), member.to_owned());
        let _ = state.expansions.insert(template, vec![member.to_owned()]);
    }
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        configs,
        Duration::ZERO,
        Duration::MAX,
        1,
        2,
        None,
    )
    .unwrap();
    let _ = state
        .borrow_mut()
        .comparison_errors
        .insert(member.to_owned(), FakeIdentityError::Recoverable);
    let initial = query.collect().unwrap();
    assert_eq!(initial.points.len(), 1);
    assert_eq!(initial.metadata.diagnostics.counter_add_failures, 1);
    assert_eq!(query.admitted_members().len(), 2);
    let deadline = Instant::now() + Duration::from_secs(60);
    let attempts = state.borrow().attempted_adds.len();
    let discoveries = state.borrow().calls.expand;
    state.borrow_mut().comparison_errors.clear();

    let recovered = query.collect_at(deadline).unwrap();

    assert_eq!(recovered.points.len(), 1);
    assert_eq!(recovered.points[0].path.as_ref(), exact);
    assert_eq!(query.admitted_members().len(), 1);
    assert_eq!(recovered.metadata.diagnostics.retry_attempts, 0);
    assert_eq!(recovered.metadata.diagnostics.retry_recoveries, 0);
    assert_eq!(state.borrow().attempted_adds.len(), attempts);
    assert_eq!(state.borrow().calls.expand, discoveries);
}

/// Scenario: Failing wildcard members churn while collection or timestamp errors repeatedly restore pending evidence.
/// Guarantees: Only current identities retain failure details, aggregate failure counts survive, and healthy peers recover normally.
#[test]
fn failed_collection_restoration_prunes_departed_failure_details() {
    let healthy = r"\Memory\Available Bytes";
    let template = r"\Process(worker*)\Private Bytes";
    for (timestamp_failure, permanent) in
        [(false, false), (true, false), (false, true), (true, true)]
    {
        let mut wildcard = counter(template, "process.private");
        wildcard.source_mapping_index = 1;
        wildcard.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        let state = fake_state(vec![fake_counter(healthy, Number::Integer(42))]);
        let mut query = Query::open_with_seed(
            FakePdh::new(Rc::clone(&state)),
            vec![counter(healthy, "memory"), wildcard],
            Duration::ZERO,
            Duration::ZERO,
            1,
            2,
            None,
        )
        .unwrap();
        let mut latest_path = String::new();
        for index in 0..32 {
            latest_path = format!(r"\Process(worker{index})\Private Bytes");
            {
                let mut state = state.borrow_mut();
                let _ = state
                    .expansions
                    .insert(template.to_owned(), vec![latest_path.clone()]);
                state.collect_error = !timestamp_failure;
                state.timestamp_error = timestamp_failure;
                if permanent {
                    state.counters.push(FakeCounter {
                        native_type: 0x4003_0401,
                        ..fake_counter(&latest_path, Number::Integer(7))
                    });
                }
            }
            query.refresh_all_wildcards_managed().unwrap();
            assert!(query.collect().is_err());
            assert_eq!(
                query.admitted_members().len(),
                if permanent { 1 } else { 2 }
            );
            assert_eq!(query.pending.failures.len(), 1);
            assert_eq!(query.pending.failures[0].path.as_ref(), latest_path);
            assert_eq!(
                query.pending.diagnostics.counter_add_failures,
                2 * (index + 1)
            );
        }
        {
            let mut state = state.borrow_mut();
            state.collect_error = false;
            state.timestamp_error = false;
            if permanent {
                state.counters.last_mut().unwrap().native_type = 0x0001_0100;
            } else {
                state
                    .counters
                    .push(fake_counter(&latest_path, Number::Integer(7)));
            }
        }
        let recovered = query.collect().unwrap();
        assert_eq!(recovered.points.len(), 2);
        assert_eq!(recovered.metadata.diagnostics.counter_add_failures, 64);
        assert_eq!(recovered.metadata.failures.len(), 1);
        assert_eq!(recovered.metadata.failures[0].path.as_ref(), latest_path);
    }
}

/// Scenario: Startup priming reads a calculated Gauge and native delta Sum before their first scrape.
/// Guarantees: The first requested scrape emits both values using receiver-side readiness and the prime timestamp.
#[test]
fn startup_prime_seeds_two_sample_histories() {
    let rate_path = r"\Processor(_Total)\% Processor Time";
    let delta_path = r"\Provider\Operations Delta";
    let (state, mut query) = fake_query(
        vec![
            counter(rate_path, "system.cpu.time"),
            cumulative_sum(delta_path, "provider.operations", "{operation}"),
        ],
        vec![
            FakeCounter {
                path: rate_path.to_owned(),
                native_type: 0x2041_0500,
                raw: Some(10),
                multi_count: 1,
                formatted: Some(Number::Double(1.0)),
                raw_error: false,
                formatted_error: false,
            },
            FakeCounter {
                path: delta_path.to_owned(),
                native_type: 0x0040_0500,
                raw: Some(10),
                multi_count: 1,
                formatted: Some(Number::Double(2.0)),
                raw_error: false,
                formatted_error: false,
            },
        ],
    );

    query.prime().unwrap();
    {
        let mut state = state.borrow_mut();
        state.timestamp = 110;
        state.counters[0].raw = Some(20);
        state.counters[1].raw = Some(20);
        state.counters[1].formatted = Some(Number::Double(3.0));
    }
    let sample = query.collect().unwrap();
    assert!(matches!(
        sample.points[0].value,
        SampleValue::Value(Number::Double(1.0))
    ));
    assert_eq!(sample.points[1].start_time_unix_nano, Some(100));
    assert!(matches!(
        sample.points[1].value,
        SampleValue::Value(Number::Double(3.0))
    ));
    assert_eq!(sample.metadata.diagnostics.warmup_omissions, 2);
}

/// Scenario: A calculated counter is mapped to a cumulative non-monotonic Sum.
/// Guarantees: The emitted point receives the query start time directly from counter sampling.
#[test]
fn cumulative_non_monotonic_sum_uses_query_start_time() {
    let path = r"\Process(_Total)\Private Bytes";
    let config = CounterConfig {
        metric_kind: MetricKind::Sum,
        ..counter(path, "windows.process.private")
    };
    let (state, mut query) = fake_query(
        vec![config],
        vec![FakeCounter {
            path: path.to_owned(),
            native_type: 0x2041_0500,
            raw: Some(10),
            multi_count: 1,
            formatted: Some(Number::Double(1.0)),
            raw_error: false,
            formatted_error: false,
        }],
    );

    query.prime().unwrap();
    {
        let mut state = state.borrow_mut();
        state.timestamp = 110;
        state.counters[0].raw = Some(20);
        state.counters[0].formatted = Some(Number::Double(2.0));
    }

    let sample = query.collect().unwrap();
    assert_eq!(sample.points[0].start_time_unix_nano, Some(100));
    assert!(matches!(
        sample.points[0].value,
        SampleValue::Value(Number::Double(2.0))
    ));
}

/// Scenario: A non-monotonic cumulative wildcard member is admitted after the query has already collected.
/// Guarantees: Newly admitted concrete series retain the logical query start independently of their sampling admission boundary.
#[test]
fn later_wildcard_member_uses_logical_query_start_time() {
    let template = r"\Process(*)\Private Bytes";
    let expanded = r"\Process(worker)\Private Bytes";
    let mut config = CounterConfig {
        metric_kind: MetricKind::Sum,
        ..counter(template, "windows.process.private")
    };
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_cadences(
        vec![config],
        vec![fake_counter(expanded, Number::Integer(42))],
        Duration::from_secs(30),
        Duration::ZERO,
    );

    state.borrow_mut().timestamp = 110;
    assert!(query.collect().unwrap().points.is_empty());
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(template.to_owned(), vec![expanded.to_owned()]);
        state.timestamp = 120;
    }
    let admitted = query.collect().unwrap();
    assert_eq!(admitted.points.len(), 1);
    assert_eq!(admitted.points[0].start_time_unix_nano, Some(100));
    assert!(matches!(
        admitted.points[0].value,
        SampleValue::Value(Number::Integer(42))
    ));
}

/// Scenario: The native startup collection reports no data before a calculated counter becomes valid.
/// Guarantees: Priming fabricates no readiness, so the first valid sample warms and the following sample emits.
#[test]
fn startup_prime_no_data_requires_fresh_warmup() {
    let path = r"\Processor(_Total)\% Processor Time";
    let (state, mut query) = fake_query(
        vec![counter(path, "system.cpu.time")],
        vec![FakeCounter {
            path: path.to_owned(),
            native_type: 0x2041_0500,
            raw: Some(10),
            multi_count: 1,
            formatted: Some(Number::Double(1.0)),
            raw_error: false,
            formatted_error: false,
        }],
    );
    state.borrow_mut().collect_has_data = false;

    query.prime().unwrap();
    {
        let mut state = state.borrow_mut();
        state.collect_has_data = true;
        state.timestamp = 110;
    }
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );
    {
        let mut state = state.borrow_mut();
        state.timestamp = 120;
        state.counters[0].raw = Some(20);
    }
    assert!(matches!(
        query.collect().unwrap().points[0].value,
        SampleValue::Value(Number::Double(1.0))
    ));
}

/// Scenario: A direct counter has a counter-local formatted failure during startup priming.
/// Guarantees: Initialization continues and the priming failure is reported with the next scrape without counting a lost scheduled observation.
#[test]
fn startup_prime_preserves_counter_failures() {
    let path = r"\Memory\Available Bytes";
    let (state, mut query) = fake_query(
        vec![counter(path, "system.memory.available")],
        vec![FakeCounter {
            path: path.to_owned(),
            native_type: 0x0001_0100,
            raw: None,
            multi_count: 1,
            formatted: Some(Number::Integer(42)),
            raw_error: false,
            formatted_error: true,
        }],
    );

    query.prime().unwrap();
    state.borrow_mut().counters[0].formatted_error = false;
    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 1);
    assert_eq!(sample.metadata.failures.len(), 1);
    assert_eq!(sample.metadata.failures[0].counter_index, 0);
    assert!(sample.metadata.failures[0].diagnostic_only);
    assert!(state.borrow().removed.is_empty());
}

/// Scenario: Exact configuration contains padded native components and zero or leading-zero suffixes.
/// Guarantees: English adds and collected paths retain configured spelling without trimming or suffix rewriting.
#[test]
fn configured_exact_paths_reach_adapter_verbatim() {
    let config = Config::from_json(&serde_json::json!({"perfcounters": [
        {"object": " Process ", "instances": [" app#0 ", "app#01"],
         "counters": [{"name": " Private Bytes "}]},
        {"object": "Memory", "counters": [{"name": "Available Bytes "}]}
    ]}))
    .unwrap();
    let paths = [
        r"\ Process ( app#0 )\ Private Bytes ",
        r"\ Process (app#01)\ Private Bytes ",
        r"\Memory\Available Bytes ",
    ];
    let (state, mut query) = fake_query(
        config.counters,
        paths
            .iter()
            .map(|path| fake_counter(path, Number::Integer(42)))
            .collect(),
    );
    assert_eq!(
        state.borrow().attempted_adds,
        paths
            .iter()
            .map(|path| ((*path).to_owned(), CounterLanguage::English))
            .collect::<Vec<_>>()
    );
    let sample = query.collect().unwrap();
    assert_eq!(
        sample
            .points
            .iter()
            .map(|point| point.path.as_ref())
            .collect::<Vec<_>>(),
        paths
    );
}

/// Scenario: One exact configured path names a duplicate process instance.
/// Guarantees: Query opening preserves parsed instance identity for non-wildcard counters.
#[test]
fn exact_instance_path_preserves_identity() {
    let path = r"\Process(worker#1)\Private Bytes";
    let mut config = counter(path, "windows.process.private");
    config.instance_config = InstanceConfig::Exact {
        emit_attribute: true,
    };
    let (_, mut query) = fake_query(vec![config], vec![fake_counter(path, Number::Integer(42))]);

    let sample = query.collect().unwrap();
    assert_eq!(
        sample.points[0].instance,
        Some(Arc::new(InstanceIdentity {
            name: "worker".to_owned(),
            parent: None,
            index: 1,
        }))
    );
}

/// Scenario: An exact configured instance path cannot be parsed for optional identity attributes.
/// Guarantees: The native counter still emits its value and reports only diagnostic identity evidence.
#[test]
fn exact_instance_parse_failure_preserves_value() {
    let path = r"\Object\Counter";
    let mut config = counter(path, "object.counter");
    config.instance_config = InstanceConfig::Exact {
        emit_attribute: true,
    };
    let (_, mut query) = fake_query(vec![config], vec![fake_counter(path, Number::Integer(42))]);

    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 1);
    assert_eq!(sample.points[0].instance, None);
    assert!(!sample.points[0].emit_instance_attribute);
    assert_eq!(sample.metadata.failures.len(), 1);
    assert!(sample.metadata.failures[0].diagnostic_only);
    assert_eq!(sample.metadata.diagnostics.counter_add_failures, 0);
}

/// Scenario: One wildcard expands to the fixed limit of 256 direct counters and produces one sample.
/// Guarantees: Sharing point identity changes no PDH work: discovery, setup, collection, reads, and timestamps remain one-for-one with native entries.
#[test]
fn high_cardinality_collection_preserves_native_call_counts() {
    let template = r"\Process(*)\Private Bytes";
    let paths = (0..256)
        .map(|index| format!(r"\Process(worker{index})\Private Bytes"))
        .collect::<Vec<_>>();
    let counters = paths
        .iter()
        .map(|path| fake_counter(path, Number::Integer(42)))
        .collect();
    let state = Rc::new(RefCell::new(FakeState {
        expansions: [(template.to_owned(), paths)].into_iter().collect(),
        ..fake_state_value(counters)
    }));
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        vec![config],
        Duration::from_secs(60),
        Duration::from_secs(60),
        256,
        256,
        None,
    )
    .unwrap();

    let sample = query.collect().unwrap();
    let state = state.borrow();

    assert_eq!(sample.points.len(), 256);
    assert_eq!(state.added.len(), 256);
    assert_eq!(state.calls.counter_type, 256);
    assert_eq!(state.calls.collect, 1);
    assert_eq!(state.calls.raw, 0);
    assert_eq!(state.calls.formatted, 256);
    assert_eq!(state.calls.timestamp, 2);
    assert_eq!(state.calls.expand, 1);
}

/// Scenario: A pure question-mark instance pattern discovers one matching concrete process.
/// Guarantees: The pattern uses wildcard expansion and emits the returned concrete path and parsed instance identity.
#[test]
fn question_mark_instance_pattern_expands_to_concrete_identity() {
    let template = r"\Process(svc?ost)\ID Process";
    let concrete = r"\Process(svchost)\ID Process";
    let mut config = counter(template, "process.id");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: Some("_Total".to_owned()),
    };
    let state = Rc::new(RefCell::new(FakeState {
        expansions: [(template.to_owned(), vec![concrete.to_owned()])]
            .into_iter()
            .collect(),
        ..fake_state_value(vec![fake_counter(concrete, Number::Integer(42))])
    }));
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        vec![config],
        Duration::from_secs(30),
        Duration::from_secs(30),
        256,
        1,
        None,
    )
    .unwrap();

    let sample = query.collect().unwrap();

    assert_eq!(state.borrow().calls.expand, 1);
    assert_eq!(sample.points.len(), 1);
    assert_eq!(sample.points[0].path.as_ref(), concrete);
    assert_eq!(sample.points[0].instance.as_ref().unwrap().name, "svchost");
}

/// Scenario: Counter inspection fails and native cleanup also reports an error during query setup.
/// Guarantees: The combined failure remains typed as rebuild-required so dropping the failed query releases the unremovable handle.
#[test]
fn setup_cleanup_failure_requires_query_rebuild() {
    let path = r"\Object\Unsupported";
    let state = Rc::new(RefCell::new(FakeState {
        remove_error: true,
        ..fake_state_value(vec![FakeCounter {
            native_type: 0x4003_0401,
            formatted: None,
            ..fake_counter(path, Number::Integer(42))
        }])
    }));
    let result = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        vec![counter(path, "windows.unsupported")],
        Duration::from_secs(30),
        Duration::from_secs(30),
        256,
        4_096,
        None,
    );
    let Err(error) = result else {
        panic!("cleanup failure must reject the query");
    };
    assert!(matches!(
        error,
        Error::CounterSetup {
            cleanup_failed: true,
            ..
        }
    ));
    assert!(error.requires_immediate_query_rebuild());
    assert!(error.to_string().contains("unsupported"));
    assert!(error.to_string().contains("PdhRemoveCounter"));
    assert_eq!(state.borrow().removed, vec![0]);
}

/// Scenario: An exact Gauge or Sum first fails transiently, then its retry exposes an unsupported native or base type.
/// Guarantees: The permanent rejection is reported once, terminates the queued retry, and is not counted as recovery.
#[test]
fn queued_retry_stops_after_unsupported_type() {
    let path = r"\Object\Unsupported";
    for (config, native_type) in [
        (counter(path, "windows.unsupported"), 0x4003_0401),
        (
            cumulative_sum(path, "object.operations", "{operation}"),
            0x4003_0402,
        ),
    ] {
        let (state, mut query) =
            fake_query_with_cadences(vec![config], Vec::new(), Duration::ZERO, Duration::ZERO);
        state.borrow_mut().counters.push(FakeCounter {
            path: path.to_owned(),
            native_type,
            raw: None,
            multi_count: 1,
            formatted: None,
            raw_error: false,
            formatted_error: false,
        });

        assert!(matches!(
            query.collect(),
            Err(Error::NoUsableCounters { .. })
        ));
        let rejected = std::mem::take(&mut query.pending);
        assert_eq!(rejected.diagnostics.retry_attempts, 1);
        assert_eq!(rejected.diagnostics.retry_recoveries, 0);
        assert_eq!(rejected.diagnostics.counter_add_failures, 2);
        assert_eq!(rejected.failures.len(), 2);
        assert_eq!(rejected.failures[1].reason, "unsupported_type");
        assert_eq!(state.borrow().removed, [0]);

        state.borrow_mut().timestamp = 110;
        assert!(matches!(
            query.collect(),
            Err(Error::NoUsableCounters { .. })
        ));
        assert_eq!(query.pending.diagnostics.retry_attempts, 0);
        assert!(query.pending.failures.is_empty());
        assert_eq!(state.borrow().removed, [0]);
    }
}

/// Scenario: An exact Sum retry finds an unsupported native type whose cleanup also fails.
/// Guarantees: The combined setup failure escapes collection as rebuild-required instead of leaving an untracked native handle.
#[test]
fn queued_retry_cleanup_failure_requires_query_rebuild() {
    let path = r"\Object\Operations";
    let (state, mut query) = fake_query_with_cadences(
        vec![cumulative_sum(path, "object.operations", "{operation}")],
        Vec::new(),
        Duration::ZERO,
        Duration::ZERO,
    );
    {
        let mut state = state.borrow_mut();
        state.counters.push(FakeCounter {
            path: path.to_owned(),
            native_type: 0x4003_0401,
            raw: None,
            multi_count: 1,
            formatted: Some(Number::Integer(42)),
            raw_error: false,
            formatted_error: false,
        });
        state.remove_error = true;
    }

    let error = query.collect().unwrap_err();
    assert!(matches!(
        error,
        Error::CounterSetup {
            cleanup_failed: true,
            ..
        }
    ));
    assert!(error.requires_immediate_query_rebuild());
    assert!(error.to_string().contains("unsupported"));
    assert!(error.to_string().contains("PdhRemoveCounter"));
    assert_eq!(state.borrow().removed, [0]);
}

/// Scenario: One normalized mapping selects an exact instance and an overlapping expanded member on any display language.
/// Guarantees: Repeated refreshes retain only the exact identity and its calculated value without replacement or renewed warm-up.
#[test]
fn same_mapping_overlap_preserves_calculated_readiness() {
    let config = Config::from_json(&serde_json::json!({
        "metrics": {"processor": {"unit": "%", "gauge": {}}},
        "perfcounters": [{
            "object": "Processor", "instances": ["0", "?"],
            "counters": [{"name": "% Processor Time", "metric": "processor"}]
        }]
    }))
    .unwrap();
    let exact = r"\Processor(0)\% Processor Time";
    let template = r"\Processor(?)\% Processor Time";
    assert_eq!(config.counters.len(), 2);
    assert_eq!(
        config.counters[0].source_mapping_index,
        config.counters[1].source_mapping_index
    );
    for member in [exact, r"\NativeProcessor(0)\NativeTime"] {
        let state = fake_state(
            [exact, member]
                .into_iter()
                .map(|path| FakeCounter {
                    native_type: 0x2151_0500,
                    raw: Some(10),
                    ..fake_counter(path, Number::Double(7.0))
                })
                .collect(),
        );
        let _ = state
            .borrow_mut()
            .full_paths
            .insert(exact.to_owned(), member.to_owned());
        let _ = state
            .borrow_mut()
            .expansions
            .insert(template.to_owned(), vec![member.to_owned()]);
        let mut query = Query::open_with_seed(
            FakePdh::new(Rc::clone(&state)),
            config.counters.clone(),
            Duration::ZERO,
            Duration::ZERO,
            1,
            2,
            None,
        )
        .unwrap();
        let first = query.collect().unwrap();
        assert_eq!(first.points.len(), 1);
        assert_eq!(first.points[0].path.as_ref(), exact);
        assert!(
            first
                .points
                .iter()
                .all(|point| point.value == SampleValue::Warming)
        );
        for _ in 0..3 {
            {
                let mut state = state.borrow_mut();
                state.timestamp += 10;
                for counter in &mut state.counters {
                    counter.raw = counter.raw.map(|value| value + 1);
                }
            }
            let sample = query.collect().unwrap();
            assert_eq!(sample.points.len(), 1);
            assert!(
                sample
                    .points
                    .iter()
                    .all(|point| point.value == SampleValue::Value(Number::Double(7.0)))
            );
            assert_eq!(state.borrow().added, [exact.to_owned()]);
            assert!(state.borrow().removed.is_empty());
        }
    }
}

/// Scenario: English exact and wildcard selectors resolve beside German and absent English inputs.
/// Guarantees: Exact adds stay English, expanded adds use localized identity, projection preserves configured metadata, and unresolved inputs warn once.
#[test]
fn smoke_selector_shapes_preserve_language_and_output_sources() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric;

    let paths = [
        r"\Memory\Available Bytes",
        r"\Processor(_Total)\% Processor Time",
        r"\Processor(*)\% Processor Time",
        r"\Process(svchos?)\ID Process",
        r"\Prozessor(_Total)\Prozessorzeit (%)",
        r"\AldoTel Smoke Missing\Missing Counter",
    ];
    let localized = [
        r"\Prozessor(0)\Prozessorzeit (%)",
        r"\Prozess(svchost)\Prozesskennung",
    ];
    let mut configs = paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let mut config = counter(path, &format!("smoke.{index}"));
            config.source_mapping_index = index;
            if matches!(index, 2 | 3) {
                config.instance_config = InstanceConfig::Wildcard {
                    excluded_aggregation: None,
                };
            }
            config
        })
        .collect::<Vec<_>>();
    configs[3].attributes = Arc::new(BTreeMap::from([(
        "instance".to_owned(),
        "configured".to_owned(),
    )]));
    let (state, mut query) = fake_query_with_cadences(
        configs.clone(),
        paths[..2]
            .iter()
            .chain(localized.iter())
            .map(|path| fake_counter(path, Number::Integer(42)))
            .collect(),
        Duration::ZERO,
        Duration::ZERO,
    );
    for (template, concrete) in paths[2..4].iter().zip(localized) {
        let _ = state
            .borrow_mut()
            .expansions
            .insert((*template).to_owned(), vec![concrete.to_owned()]);
    }
    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 4);
    assert_eq!(sample.metadata.failures.len(), 2);
    assert!(
        sample
            .metadata
            .failures
            .iter()
            .all(|failure| failure.reason == "selector_unresolved")
    );
    assert_eq!(
        sample
            .points
            .iter()
            .map(|point| point.path.as_ref())
            .collect::<Vec<_>>(),
        [paths[0], paths[1], localized[0], localized[1]]
    );
    assert!(
        state
            .borrow()
            .attempted_adds
            .iter()
            .all(|(path, language)| {
                *language
                    == if localized.contains(&path.as_str()) {
                        CounterLanguage::Localized
                    } else {
                        CounterLanguage::English
                    }
            })
    );
    let records = into_otap(&configs, sample).unwrap().unwrap();
    let decoded = decode(records);
    for metric in &decoded.resource_metrics[0].scope_metrics[0].metrics {
        let index = configs
            .iter()
            .position(|config| config.name.as_ref() == metric.name)
            .unwrap();
        let Some(metric::Data::Gauge(gauge)) = &metric.data else {
            panic!("expected Gauge")
        };
        let attributes = gauge.data_points[0]
            .attributes
            .iter()
            .map(|attribute| {
                let Some(any_value::Value::StringValue(value)) = attribute
                    .value
                    .as_ref()
                    .and_then(|value| value.value.as_ref())
                else {
                    panic!("expected string identity")
                };
                (attribute.key.as_str(), value.as_str())
            })
            .collect::<BTreeMap<_, _>>();
        let concrete = if index < 2 {
            paths[index]
        } else {
            localized[index - 2]
        };
        assert_eq!(attributes["windows.perf_counter.path"], concrete);
        assert_eq!(
            attributes
                .get("windows.perf_counter.path_template")
                .copied(),
            (index >= 2).then_some(paths[index])
        );
        if index == 3 {
            assert_eq!(attributes["instance"], "configured");
        }
        assert_eq!(metric.description, configs[index].description.as_ref());
        assert_eq!(metric.unit, configs[index].unit.as_ref());
    }
    let retry = query.collect().unwrap();
    assert_eq!(retry.points.len(), 4);
    assert_eq!(retry.metadata.diagnostics.counter_add_failures, 2);
    assert!(retry.metadata.failures.is_empty());
    assert_eq!(state.borrow().calls.refresh, 0);
}

/// Scenario: Exact-only discovery has an unresolved selector and reaches the minimum or configured object-refresh interval.
/// Guarantees: Controlled discovery and object-refresh boundaries cause exactly one refresh without extra retries or loss of healthy collection.
#[test]
fn unresolved_exact_drives_conditional_refresh_at_discovery_only() {
    for (discovery, refresh_fails) in [(60, false), (900, false), (60, true)] {
        let interval = Duration::from_secs(discovery);
        let required = interval.max(Duration::from_secs(600));
        let healthy = r"\Memory\Available Bytes";
        let missing = r"\AldoTel Smoke Missing\Missing Counter";
        let (state, mut query) = fake_query_with_cadences(
            vec![counter(healthy, "healthy"), counter(missing, "missing")],
            vec![fake_counter(healthy, Number::Integer(42))],
            Duration::ZERO,
            interval,
        );
        state.borrow_mut().object_refresh_error = refresh_fails;
        let now = Instant::now();
        query.last_wildcard_refresh = now;
        query.last_object_refresh = now;
        let _ = query.collect_at(now).unwrap();
        assert_eq!(state.borrow().calls.refresh, 0);
        let deadline = now + required;
        let before = deadline - Duration::from_nanos(1);
        query.last_wildcard_refresh = before - interval;
        let early = query.collect_at(before).unwrap();
        assert_eq!(early.metadata.diagnostics.discovery_refreshes, 1);
        assert_eq!(state.borrow().calls.refresh, 0);
        query.last_wildcard_refresh = deadline - interval;
        let attempts = state.borrow().calls.add;
        let refreshed = query.collect_at(deadline).unwrap();
        assert_eq!(refreshed.points.len(), 1);
        assert!(refreshed.metadata.failures.is_empty());
        assert_eq!(
            refreshed.metadata.diagnostics.discovery_failures,
            u64::from(refresh_fails)
        );
        assert_eq!(state.borrow().calls.refresh, 1);
        assert_eq!(state.borrow().calls.add, attempts + 1);
        let steady = query.collect_at(deadline).unwrap();
        assert_eq!(steady.metadata.diagnostics.discovery_refreshes, 0);
        assert_eq!(state.borrow().calls.refresh, 1);
        query.last_wildcard_refresh = deadline - interval;
        let _ = query.collect_at(deadline).unwrap();
        assert_eq!(state.borrow().calls.refresh, 1);
        state
            .borrow_mut()
            .counters
            .push(fake_counter(missing, Number::Integer(7)));
        assert_eq!(query.collect_at(deadline).unwrap().points.len(), 2);
        query.last_wildcard_refresh = deadline + required - interval;
        let resolved = query.collect_at(deadline + required).unwrap();
        assert_eq!(resolved.points.len(), 2);
        assert_eq!(resolved.metadata.diagnostics.discovery_refreshes, 0);
        assert_eq!(state.borrow().calls.refresh, 1);
    }
}

/// Scenario: A resolved wildcard later fails lookup, retains healthy members, rebuilds, resolves empty, and fails again.
/// Guarantees: Retained values and changed native reasons cannot re-arm warnings; refresh precedes lookup, and only successful resolution ends the episode.
#[test]
fn retained_wildcard_failure_episode_survives_values_and_seeded_rebuild() {
    let template = r"\Process(svchos?)\ID Process";
    let concrete = r"\Prozess(svchost)\Prozesskennung";
    let mut config = counter(template, "process");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_cadences(
        vec![config.clone()],
        vec![fake_counter(concrete, Number::Integer(42))],
        Duration::ZERO,
        Duration::ZERO,
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), vec![concrete.to_owned()]);
    assert_eq!(query.collect().unwrap().points.len(), 1);
    query.last_object_refresh = Instant::now() - Duration::from_secs(600);
    let _ = query.collect().unwrap();
    assert_eq!(state.borrow().calls.refresh, 0);
    state.borrow_mut().expand_error = true;
    let failed = query.collect().unwrap();
    assert_eq!(failed.points.len(), 1);
    assert_eq!(failed.metadata.failures.len(), 1);
    state.borrow_mut().calls.order.clear();
    state.borrow_mut().expand_operation = PdhOperation::GetCounterInfo;
    let repeated = query.collect().unwrap();
    assert_eq!(repeated.points.len(), 1);
    assert!(repeated.metadata.failures.is_empty());
    assert_eq!(state.borrow().calls.order, ["objects.refresh", "lookup"]);
    let seed = query.recreation_seed();
    let mut rebuilt = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        vec![config],
        Duration::ZERO,
        Duration::ZERO,
        256,
        4096,
        Some(&seed),
    )
    .unwrap();
    assert!(rebuilt.collect().unwrap().metadata.failures.is_empty());
    assert_eq!(state.borrow().calls.refresh, 1);
    state.borrow_mut().expand_error = false;
    state.borrow_mut().expansions.clear();
    assert!(rebuilt.collect().unwrap().points.is_empty());
    state.borrow_mut().expand_error = true;
    assert_eq!(rebuilt.collect().unwrap().metadata.failures.len(), 1);
}

/// Scenario: Temporary-template cleanup fails with healthy retained members, with or without an earlier lookup failure.
/// Guarantees: Cleanup preserves collection and its warning episode, does not count as discovery failure, and cannot erase an earlier refresh prerequisite.
#[test]
fn template_cleanup_failure_does_not_escalate_or_corrupt_refresh_accounting() {
    for earlier_lookup_failure in [false, true] {
        let template = r"\Process(*)\ID Process";
        let concrete = r"\Prozess(svchost)\Prozesskennung";
        let mut config = counter(template, "process");
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        let (state, mut query) = fake_query_with_cadences(
            vec![config],
            vec![fake_counter(concrete, Number::Integer(42))],
            Duration::ZERO,
            Duration::ZERO,
        );
        let _ = state
            .borrow_mut()
            .expansions
            .insert(template.to_owned(), vec![concrete.to_owned()]);
        assert_eq!(query.collect().unwrap().points.len(), 1);
        state.borrow_mut().expand_error = true;
        if earlier_lookup_failure {
            state.borrow_mut().expand_operation = PdhOperation::GetCounterInfo;
            let failed = query.collect().unwrap();
            assert_eq!(failed.metadata.diagnostics.discovery_failures, 1);
            assert_eq!(failed.metadata.failures.len(), 1);
        }
        state.borrow_mut().expand_operation = PdhOperation::RemoveCounter;
        let cleanup = query.collect().unwrap();
        assert_eq!(cleanup.points.len(), 1);
        assert_eq!(cleanup.metadata.diagnostics.discovery_failures, 0);
        assert_eq!(
            cleanup.metadata.failures.len(),
            usize::from(!earlier_lookup_failure)
        );
        query.last_object_refresh = Instant::now() - Duration::from_secs(600);
        let repeated = query.collect().unwrap();
        assert!(repeated.metadata.diagnostics.any_query_collected);
        assert_eq!(repeated.points.len(), 1);
        assert!(repeated.metadata.failures.is_empty());
        assert_eq!(repeated.metadata.diagnostics.discovery_failures, 0);
        assert_eq!(
            state.borrow().calls.refresh,
            usize::from(earlier_lookup_failure)
        );
    }
}

/// Scenario: An exact counter remains transiently unavailable for one queued retry and then becomes valid.
/// Guarantees: The transient failure remains queued through existing backoff state and successful re-add counts one recovery.
#[test]
fn queued_transient_retry_still_recovers() {
    let path = r"\Memory\Available Bytes";
    let (state, mut query) = fake_query_with_cadences(
        vec![counter(path, "system.memory.available")],
        Vec::new(),
        Duration::ZERO,
        Duration::ZERO,
    );

    let retrying = query.collect().unwrap();
    assert_eq!(retrying.metadata.diagnostics.retry_attempts, 1);
    assert_eq!(retrying.metadata.diagnostics.retry_recoveries, 0);
    assert_eq!(retrying.metadata.diagnostics.counter_add_failures, 2);
    assert_eq!(retrying.metadata.failures.len(), 1);
    assert_eq!(retrying.metadata.failures[0].reason, "selector_unresolved");

    {
        let mut state = state.borrow_mut();
        state.timestamp = 110;
        state.counters.push(fake_counter(path, Number::Integer(42)));
    }
    let recovered = query.collect().unwrap();
    assert_eq!(recovered.metadata.diagnostics.retry_attempts, 1);
    assert_eq!(recovered.metadata.diagnostics.retry_recoveries, 1);
    assert_eq!(recovered.metadata.diagnostics.counter_add_failures, 0);
    assert!(recovered.metadata.failures.is_empty());
    assert!(matches!(
        recovered.points[0].value,
        SampleValue::Value(Number::Integer(42))
    ));

    state.borrow_mut().timestamp = 120;
    let steady = query.collect().unwrap();
    assert_eq!(steady.metadata.diagnostics.retry_attempts, 0);
    assert_eq!(steady.metadata.diagnostics.retry_recoveries, 0);
    assert!(matches!(
        steady.points[0].value,
        SampleValue::Value(Number::Integer(42))
    ));
}

/// Scenario: A wildcard refresh repeatedly discovers an instance whose native type is permanently unsupported.
/// Guarantees: Each refresh records one bounded rejection without queuing retries or permanently blacklisting rediscovery.
#[test]
fn wildcard_refresh_retries_permanently_rejected_instance() {
    let template = r"\Process(*)\Unsupported";
    let expanded = r"\Process(worker)\Unsupported";
    let mut config = counter(template, "process.unsupported");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_cadences(
        vec![config],
        vec![FakeCounter {
            path: expanded.to_owned(),
            native_type: 0x4003_0401,
            raw: None,
            multi_count: 1,
            formatted: None,
            raw_error: false,
            formatted_error: false,
        }],
        Duration::ZERO,
        Duration::ZERO,
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), vec![expanded.to_owned()]);

    for (timestamp, expected_attempts) in [(110, 1), (120, 2)] {
        state.borrow_mut().timestamp = timestamp;
        let rejected = query.collect().unwrap();
        assert_eq!(rejected.metadata.diagnostics.retry_attempts, 0);
        assert_eq!(rejected.metadata.diagnostics.retry_recoveries, 0);
        assert_eq!(rejected.metadata.diagnostics.counter_add_failures, 1);
        assert_eq!(
            rejected
                .metadata
                .failures
                .iter()
                .filter(|failure| failure.reason == "unsupported_type")
                .count(),
            1
        );
        let state = state.borrow();
        assert_eq!(state.added.len(), expected_attempts);
        assert_eq!(state.removed.len(), expected_attempts);
    }
}

/// Scenario: A departed wildcard member cannot be removed from its shared native query on the first attempt.
/// Guarantees: The member stops emitting immediately, remains budgeted under query ownership, and cleanup retries without resetting peers.
#[test]
fn wildcard_remove_failure_stays_owned_until_local_retry() {
    let template = r"\Process(*)\Private Bytes";
    let expanded = r"\Process(worker)\Private Bytes";
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_cadences(
        vec![config],
        vec![fake_counter(expanded, Number::Integer(42))],
        Duration::from_secs(60),
        Duration::ZERO,
    );
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(template.to_owned(), vec![expanded.to_owned()]);
    }
    let now = Instant::now();
    let admitted = query.collect_at(now).unwrap();
    assert_eq!(admitted.points.len(), 1);
    assert_eq!(query.active_expanded_count(), 1);
    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(template.to_owned(), Vec::new());
        state.remove_error = true;
        state.timestamp = 110;
    }

    let pending = query.collect_at(now).unwrap();
    assert!(pending.points.is_empty());
    assert_eq!(query.active_expanded_count(), 0);
    assert_eq!(pending.metadata.failures.len(), 1);
    assert_eq!(pending.metadata.failures[0].reason, "PdhRemoveCounter");
    assert_eq!(state.borrow().removed, [0]);

    {
        let mut state = state.borrow_mut();
        state.remove_error = false;
        state.timestamp = 120;
    }
    let cleaned = query.collect_at(now + Duration::from_secs(1)).unwrap();
    assert!(cleaned.points.is_empty());
    assert_eq!(query.active_expanded_count(), 0);
    assert_eq!(state.borrow().removed, [0, 0]);
}

/// Scenario: A newly discovered wildcard member sorts before an already admitted member under a one-entry limit.
/// Guarantees: The admitted member keeps its handle and history while the newcomer is omitted.
#[test]
fn expansion_limit_prioritizes_admitted_wildcard_member() {
    let template = r"\Process(*)\Private Bytes";
    let admitted = r"\Process(zeta)\Private Bytes";
    let newcomer = r"\Process(alpha)\Private Bytes";
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_seed(
        vec![config],
        vec![
            fake_counter(admitted, Number::Integer(42)),
            fake_counter(newcomer, Number::Integer(7)),
        ],
        Duration::ZERO,
        Duration::ZERO,
        1,
        None,
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), vec![admitted.to_owned()]);
    let first = query.collect().unwrap();
    assert_eq!(first.points[0].path.as_ref(), admitted);

    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(
            template.to_owned(),
            vec![newcomer.to_owned(), admitted.to_owned()],
        );
        state.timestamp = 110;
    }
    let retained = query.collect().unwrap();

    assert_eq!(retained.points.len(), 1);
    assert_eq!(retained.points[0].path.as_ref(), admitted);
    assert_eq!(retained.metadata.overflows.len(), 1);
    let state = state.borrow();
    assert_eq!(state.added.len(), 1);
    assert_eq!(state.added[0], admitted);
    assert!(state.removed.is_empty());
}

/// Scenario: Wildcard members c then b are admitted across two refreshes before newcomer a sorts ahead of both.
/// Guarantees: A two-entry limit retains live b and c, and a later capacity shrink retains c as the earliest admission rather than key-sorted b.
#[test]
fn expansion_limit_preserves_wildcard_admission_order() {
    let template = r"\Process(*)\Private Bytes";
    let paths = [
        r"\Process(alpha)\Private Bytes",
        r"\Process(beta)\Private Bytes",
        r"\Process(charlie)\Private Bytes",
    ];
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_seed(
        vec![config],
        paths
            .iter()
            .enumerate()
            .map(|(index, path)| FakeCounter {
                path: (*path).to_owned(),
                native_type: 0x0001_0100,
                raw: None,
                multi_count: 1,
                formatted: Some(Number::Integer(index as i64)),
                raw_error: false,
                formatted_error: false,
            })
            .collect(),
        Duration::ZERO,
        Duration::ZERO,
        1,
        None,
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), vec![paths[2].to_owned()]);
    assert_eq!(query.collect().unwrap().points[0].path.as_ref(), paths[2]);

    query.set_max_native_counters(2).unwrap();
    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(
            template.to_owned(),
            vec![paths[1].to_owned(), paths[2].to_owned()],
        );
        state.timestamp = 110;
    }
    assert_eq!(
        query
            .collect()
            .unwrap()
            .points
            .iter()
            .map(|point| point.path.as_ref())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([paths[1], paths[2]])
    );

    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(
            template.to_owned(),
            paths.iter().map(|path| (*path).to_owned()).collect(),
        );
        state.timestamp = 120;
    }
    assert_eq!(
        query
            .collect()
            .unwrap()
            .points
            .iter()
            .map(|point| point.path.as_ref())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([paths[1], paths[2]])
    );

    query.set_max_native_counters(1).unwrap();
    state.borrow_mut().timestamp = 130;
    let shrunk = query.collect().unwrap();
    assert_eq!(shrunk.points.len(), 1);
    assert_eq!(shrunk.points[0].path.as_ref(), paths[2]);
}

/// Scenario: A wildcard member whose native removal failed is rediscovered before cleanup retries.
/// Guarantees: The existing handle is restored as one retained entry without a second removal or add.
#[test]
fn rediscovered_pending_removal_restores_existing_handle() {
    let template = r"\Process(*)\Private Bytes";
    let expanded = r"\Process(worker)\Private Bytes";
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_seed(
        vec![config],
        vec![fake_counter(expanded, Number::Integer(42))],
        Duration::from_secs(60),
        Duration::ZERO,
        1,
        None,
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), vec![expanded.to_owned()]);
    let now = Instant::now();
    assert_eq!(query.collect_at(now).unwrap().points.len(), 1);

    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(template.to_owned(), Vec::new());
        state.remove_error = true;
        state.timestamp = 110;
    }
    let removed = query.collect_at(now).unwrap();
    assert!(removed.points.is_empty());
    assert_eq!(removed.metadata.diagnostics.instances_removed, 1);
    assert_eq!(query.active_expanded_count(), 0);

    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(template.to_owned(), vec![expanded.to_owned()]);
        state.remove_error = false;
        state.timestamp = 120;
    }
    let restored = query.collect_at(now).unwrap();

    assert_eq!(restored.points.len(), 1);
    assert_eq!(restored.points[0].path.as_ref(), expanded);
    assert_eq!(query.active_expanded_count(), 1);

    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(template.to_owned(), Vec::new());
        state.remove_error = true;
        state.timestamp = 130;
    }
    let removed_again = query.collect_at(now).unwrap();
    assert!(removed_again.points.is_empty());
    assert_eq!(removed_again.metadata.diagnostics.instances_removed, 0);
    let state = state.borrow();
    assert_eq!(state.added.len(), 1);
    assert_eq!(state.added[0], expanded);
    assert_eq!(state.removed, [0, 0]);
}

/// Scenario: A wildcard Delta handle remains native-active while removal is pending, then is rediscovered.
/// Guarantees: Restoration re-warms native sampling and retains the cumulative query start.
#[test]
fn rediscovered_pending_delta_removal_restarts_sampling() {
    let template = r"\Process(*)\Operations Delta";
    let expanded = r"\Process(worker)\Operations Delta";
    let mut config = cumulative_sum(template, "process.operations", "{operation}");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) = fake_query_with_seed(
        vec![config],
        vec![FakeCounter {
            path: expanded.to_owned(),
            native_type: 0x0040_0500,
            raw: Some(10),
            multi_count: 1,
            formatted: Some(Number::Double(1.0)),
            raw_error: false,
            formatted_error: false,
        }],
        Duration::ZERO,
        Duration::ZERO,
        1,
        None,
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), vec![expanded.to_owned()]);
    assert_eq!(
        query.collect().unwrap().points[0].value,
        SampleValue::Warming
    );

    {
        let mut state = state.borrow_mut();
        state.counters[0].raw = Some(20);
        state.counters[0].formatted = Some(Number::Double(2.0));
        state.timestamp = 110;
    }
    let ready = query.collect().unwrap();
    assert_eq!(ready.points[0].start_time_unix_nano, Some(100));

    {
        let mut state = state.borrow_mut();
        let _ = state.expansions.insert(template.to_owned(), Vec::new());
        state.remove_error = true;
        state.timestamp = 120;
    }
    assert!(query.collect().unwrap().points.is_empty());

    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(template.to_owned(), vec![expanded.to_owned()]);
        state.remove_error = false;
        state.counters[0].raw = Some(40);
        state.counters[0].formatted = Some(Number::Double(4.0));
        state.timestamp = 130;
    }
    let warming = query.collect().unwrap();
    assert_eq!(warming.points[0].value, SampleValue::Warming);

    {
        let mut state = state.borrow_mut();
        state.counters[0].raw = Some(50);
        state.counters[0].formatted = Some(Number::Double(5.0));
        state.timestamp = 140;
    }
    let recovered = query.collect().unwrap();
    assert_eq!(recovered.points[0].start_time_unix_nano, Some(100));
    assert_eq!(
        recovered.points[0].value,
        SampleValue::Value(Number::Double(5.0))
    );
}

/// Scenario: One wildcard member is pending native removal while another template refreshes under a fully allocated receiver budget.
/// Guarantees: The pending removal consumes exactly one entry, so the healthy template retains its existing member without a false overflow.
#[test]
fn pending_wildcard_removal_consumes_one_budget_entry() {
    let first_template = r"\Process(*)\Private Bytes";
    let first_expanded = r"\Process(worker)\Private Bytes";
    let second_template = r"\Thread(*)\Context Switches/sec";
    let second_expanded = r"\Thread(worker)\Context Switches/sec";
    let wildcard = |path: &str, name: &str| {
        let mut config = counter(path, name);
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        config
    };
    let first = wildcard(first_template, "process.private");
    let mut second = wildcard(second_template, "thread.context_switches");
    second.source_mapping_index = 1;
    let (state, mut query) = fake_query_with_seed(
        vec![first, second],
        vec![
            fake_counter(first_expanded, Number::Integer(42)),
            fake_counter(second_expanded, Number::Integer(7)),
        ],
        Duration::ZERO,
        Duration::ZERO,
        2,
        None,
    );
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(first_template.to_owned(), vec![first_expanded.to_owned()]);
        let _ = state
            .expansions
            .insert(second_template.to_owned(), vec![second_expanded.to_owned()]);
    }
    query.refresh_all_wildcards_managed().unwrap();
    assert_eq!(query.active_expanded_count(), 2);

    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(first_template.to_owned(), Vec::new());
        state.remove_error = true;
    }
    query.refresh_all_wildcards_managed().unwrap();
    assert_eq!(query.active_expanded_count(), 1);
    assert_eq!(query.admitted_members().len(), 1);
    assert_eq!(query.admitted_members()[0].path.as_ref(), second_expanded);
    assert!(query.pending.overflows.is_empty());
}

/// Scenario: Expanded paths arrive in different orders and include duplicate instance names.
/// Guarantees: Stable keys use configured identity plus the full case-insensitive PDH path.
#[test]
fn stable_keys_do_not_depend_on_expansion_order() {
    let first = [
        CounterKey::new(0, r"\Process(worker)\Private Bytes"),
        CounterKey::new(0, r"\Process(worker#1)\Private Bytes"),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let reordered = [
        CounterKey::new(0, r"\PROCESS(WORKER#1)\PRIVATE BYTES"),
        CounterKey::new(0, r"\PROCESS(WORKER)\PRIVATE BYTES"),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(first, reordered);
    assert_ne!(
        CounterKey::new(0, r"\Process(worker)\Private Bytes"),
        CounterKey::new(0, r"\Process(worker#1)\Private Bytes")
    );
    assert_ne!(
        CounterKey::new(0, r"\Process(worker)\Private Bytes"),
        CounterKey::new(1, r"\Process(worker)\Private Bytes")
    );
}

/// Scenario: English wildcard expansion returns a local path with an optional machine prefix.
/// Guarantees: The emitted path and stable identity match the equivalent configured local exact path.
#[test]
fn machine_prefixed_expansion_uses_local_path_identity() {
    let local = r"\Process(worker)\Private Bytes";
    let prefixed = r"\\MACHINE\Process(worker)\Private Bytes";

    let paths = keyed_expanded_paths(0, vec![prefixed.to_owned()]);

    assert_eq!(paths, [(CounterKey::new(0, local), local.to_owned())]);
    assert_eq!(CounterKey::new(0, prefixed), CounterKey::new(0, local));
}

/// Scenario: Discovery returns two non-ASCII case variants of one path plus one distinct path at the budget limit.
/// Guarantees: CounterKey identity removes the collision before counting so aliases cannot consume capacity or replace native spelling by order.
#[test]
fn non_ascii_case_collisions_share_one_budgeted_key() {
    let upper = format!(r"\Object({})\Value", "\u{00C4}");
    let lower = format!(r"\Object({})\Value", "\u{00E4}");
    let other = r"\Object(worker)\Value".to_owned();
    assert_eq!(CounterKey::new(0, &upper), CounterKey::new(0, &lower));

    let paths = keyed_expanded_paths(0, vec![lower.clone(), other.clone(), upper.clone()]);
    let allocation = expansion_allocation(paths.len(), 2, 2);

    assert_eq!(
        allocation,
        ExpansionAllocation {
            retained: 2,
            per_wildcard_omitted: 0,
            total_omitted: 0,
        }
    );
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|(_, path)| path == &upper));
    assert!(!paths.iter().any(|(_, path)| path == &lower));
    assert!(paths.iter().any(|(_, path)| path == &other));
}

/// Scenario: A later mapping retains or replaces its member while an earlier mapping discovers a new member at a one-entry limit.
/// Guarantees: Retained members win globally; when capacity is released, new members are admitted in configuration rather than native-name order.
#[test]
fn refresh_retains_globally_then_admits_in_config_order() {
    let first_template = r"\ZFirst(*)\Value";
    let later_template = r"\ASecond(*)\Value";
    let first_new = r"\ZFirst(new)\Value";
    let later_old = r"\ASecond(old)\Value";
    let later_new = r"\ASecond(new)\Value";
    for retain_later in [false, true] {
        let configs = [first_template, later_template]
            .into_iter()
            .enumerate()
            .map(|(index, path)| {
                let mut config = counter(path, "value");
                config.source_mapping_index = index;
                config.instance_config = InstanceConfig::Wildcard {
                    excluded_aggregation: None,
                };
                config
            })
            .collect();
        let (state, mut query) = fake_query_with_seed(
            configs,
            [first_new, later_old, later_new]
                .into_iter()
                .map(|path| fake_counter(path, Number::Integer(1)))
                .collect(),
            Duration::ZERO,
            Duration::ZERO,
            1,
            None,
        );
        let _ = state
            .borrow_mut()
            .expansions
            .insert(later_template.to_owned(), vec![later_old.to_owned()]);
        query.refresh_all_wildcards_managed().unwrap();
        {
            let mut state = state.borrow_mut();
            let _ = state
                .expansions
                .insert(first_template.to_owned(), vec![first_new.to_owned()]);
            let _ = state.expansions.insert(
                later_template.to_owned(),
                vec![if retain_later { later_old } else { later_new }.to_owned()],
            );
        }
        query.refresh_all_wildcards_managed().unwrap();
        let sample = query.collect().unwrap();
        assert_eq!(sample.points.len(), 1);
        assert_eq!(
            sample.points[0].path.as_ref(),
            if retain_later { later_old } else { first_new }
        );
        assert_eq!(query.active_counter_count(), 1);
    }
}

/// Scenario: A disappeared member in a later mapping cannot be removed while an earlier mapping discovers a replacement.
/// Guarantees: Failed cleanup retains the native-entry reservation and prevents new acquisition until cleanup succeeds.
#[test]
fn global_refresh_preserves_failed_removal_reservation() {
    let first_template = r"\First(*)\Value";
    let later_template = r"\Later(*)\Value";
    let first_new = r"\First(new)\Value";
    let later_old = r"\Later(old)\Value";
    let configs = [first_template, later_template]
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let mut config = counter(path, "value");
            config.source_mapping_index = index;
            config.instance_config = InstanceConfig::Wildcard {
                excluded_aggregation: None,
            };
            config
        })
        .collect();
    let (state, mut query) = fake_query_with_seed(
        configs,
        [first_new, later_old]
            .into_iter()
            .map(|path| fake_counter(path, Number::Integer(1)))
            .collect(),
        Duration::ZERO,
        Duration::ZERO,
        1,
        None,
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(later_template.to_owned(), vec![later_old.to_owned()]);
    query.refresh_all_wildcards_managed().unwrap();
    let _ = query.collect().unwrap();
    {
        let mut state = state.borrow_mut();
        let _ = state
            .expansions
            .insert(first_template.to_owned(), vec![first_new.to_owned()]);
        let _ = state
            .expansions
            .insert(later_template.to_owned(), Vec::new());
        state.remove_error = true;
    }
    query.refresh_all_wildcards_managed().unwrap();
    let failed = query.collect().unwrap();
    assert!(failed.points.is_empty());
    assert!(
        failed
            .metadata
            .failures
            .iter()
            .any(|failure| failure.reason == "PdhRemoveCounter")
    );
    assert_eq!(state.borrow().added, [later_old.to_owned()]);
    assert_eq!(failed.metadata.diagnostics.instances_omitted_over_limit, 2);
    state.borrow_mut().remove_error = false;
    let _ = query.collect().unwrap();
    query.refresh_all_wildcards_managed().unwrap();
    let admitted = query.collect().unwrap();
    assert_eq!(admitted.points[0].path.as_ref(), first_new);
    assert_eq!(
        state.borrow().added,
        [later_old.to_owned(), first_new.to_owned()]
    );
}

/// Scenario: A wildcard is empty at startup and one refresh before an instance appears.
/// Guarantees: Every refresh discovers again, startup diagnoses emptiness only once, and the late member publishes without a restart.
#[test]
fn empty_startup_expansion_is_diagnosed_once() {
    let template = r"\Process(*)\Private Bytes";
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let (state, mut query) =
        fake_query_with_cadences(vec![config], Vec::new(), Duration::ZERO, Duration::ZERO);
    let startup = query.collect().unwrap();
    assert_eq!(
        startup
            .metadata
            .failures
            .iter()
            .filter(|failure| failure.reason == "empty_expansion")
            .count(),
        1
    );
    assert!(
        startup
            .metadata
            .failures
            .iter()
            .all(|failure| failure.diagnostic_only)
    );
    let later = query.collect().unwrap();
    assert!(
        !later
            .metadata
            .failures
            .iter()
            .any(|failure| failure.reason == "empty_expansion")
    );
    assert!(!later.metadata.diagnostics.any_query_collected);
    assert_eq!(state.borrow().calls.expand, 2);
    let expanded = r"\Process(late)\Private Bytes";
    {
        let mut state = state.borrow_mut();
        state
            .counters
            .push(fake_counter(expanded, Number::Integer(7)));
        let _ = state
            .expansions
            .insert(template.to_owned(), vec![expanded.to_owned()]);
    }
    let admitted = query.collect().unwrap();
    assert_eq!(state.borrow().calls.expand, 3);
    assert_eq!(admitted.points[0].path.as_ref(), expanded);
}

/// Scenario: Discovered membership exceeds the per-pattern bound or remaining total capacity.
/// Guarantees: Production allocation counts retained entries and omissions against the correct independent limits.
#[test]
fn expansion_allocation_reports_each_limit() {
    let omitted = expansion_allocation(3, 2, 10);
    assert_eq!(
        omitted,
        ExpansionAllocation {
            retained: 2,
            per_wildcard_omitted: 1,
            total_omitted: 0,
        }
    );

    let omitted = expansion_allocation(2, 2, 1);
    assert_eq!(
        omitted,
        ExpansionAllocation {
            retained: 1,
            per_wildcard_omitted: 0,
            total_omitted: 1,
        }
    );
}

/// Scenario: An earlier configured wildcard succeeds while a later wildcard's discovery fails with retained identities.
/// Guarantees: Failed-discovery retention reserves global capacity before successful discoveries regardless of configuration order.
#[test]
fn failed_discovery_retention_reserves_global_capacity_first() {
    let first_template = r"\Process(*)\Private Bytes";
    let second_template = r"\Thread(*)\Context Switches/sec";
    let first_paths = [
        r"\Process(alpha)\Private Bytes",
        r"\Process(bravo)\Private Bytes",
        r"\Process(charlie)\Private Bytes",
    ];
    let second_paths = [
        r"\Thread(alpha)\Context Switches/sec",
        r"\Thread(bravo)\Context Switches/sec",
        r"\Thread(charlie)\Context Switches/sec",
    ];
    let wildcard = |path: &str, name: &str| {
        let mut config = counter(path, name);
        config.instance_config = InstanceConfig::Wildcard {
            excluded_aggregation: None,
        };
        config
    };
    let counters = first_paths
        .iter()
        .chain(second_paths.iter())
        .enumerate()
        .map(|(index, path)| fake_counter(path, Number::Integer(i64::try_from(index + 1).unwrap())))
        .collect();
    let mut second = wildcard(second_template, "thread.context_switches");
    second.source_mapping_index = 1;
    let (state, mut query) = fake_query_with_seed(
        vec![wildcard(first_template, "process.private"), second],
        counters,
        Duration::ZERO,
        Duration::from_secs(30),
        4,
        None,
    );

    let _ = state.borrow_mut().expansions.insert(
        second_template.to_owned(),
        second_paths.iter().map(|path| (*path).to_owned()).collect(),
    );
    query.refresh_all_wildcards_managed().unwrap();
    assert_eq!(query.active_expanded_count(), 3);

    {
        let mut state = state.borrow_mut();
        let _ = state.expand_errors.insert(second_template.to_owned());
        let _ = state.expansions.insert(
            first_template.to_owned(),
            first_paths.iter().map(|path| (*path).to_owned()).collect(),
        );
    }
    query.last_wildcard_refresh = Instant::now() - Duration::from_secs(31);
    query.refresh_all_wildcards_managed().unwrap();

    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 4);
    assert_eq!(
        sample
            .points
            .iter()
            .filter(|point| point.counter_index == 0)
            .count(),
        1
    );
    assert_eq!(
        sample
            .points
            .iter()
            .filter(|point| point.counter_index == 1)
            .count(),
        3
    );
    assert!(
        sample
            .points
            .iter()
            .all(|point| matches!(point.value, SampleValue::Value(Number::Integer(_))))
    );
    assert_eq!(sample.metadata.diagnostics.active_expanded_counters, 4);
    assert_eq!(sample.metadata.diagnostics.discovery_failures, 1);
    assert_eq!(
        sample.metadata.overflows,
        [ExpansionOverflow {
            counter_index: 0,
            reason: "admission_limit",
            discovered: 3,
            retained: 1,
            omitted: 2,
            per_wildcard_omitted: 0,
            total_omitted: 2,
            examples: first_paths[1..]
                .iter()
                .map(|path| Arc::from(*path))
                .collect(),
        }]
    );
}

/// Scenario: A production refresh exceeds both the per-pattern and receiver-wide limits.
/// Guarantees: One exact admission summary counts every omission and carries only the first three deterministically ordered omitted paths.
#[test]
fn wildcard_overflow_is_one_bounded_admission_record() {
    let template = r"\Process(*)\Private Bytes";
    let paths = (0..8)
        .map(|index| format!(r"\Process(worker{index})\Private Bytes"))
        .collect::<Vec<_>>();
    let mut config = counter(template, "process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
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
        .insert(template.to_owned(), paths.iter().rev().cloned().collect());
    let mut query = Query::open_with_seed(
        FakePdh::new(Rc::clone(&state)),
        vec![config],
        Duration::ZERO,
        Duration::from_secs(60),
        2,
        1,
        None,
    )
    .unwrap();
    query.refresh_all_wildcards_managed().unwrap();
    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 1);
    assert_eq!(sample.points[0].path.as_ref(), paths[0]);
    assert_eq!(sample.metadata.diagnostics.instances_omitted_over_limit, 7);
    assert_eq!(
        sample.metadata.overflows,
        [ExpansionOverflow {
            counter_index: 0,
            reason: "admission_limit",
            discovered: 8,
            retained: 1,
            omitted: 7,
            per_wildcard_omitted: 6,
            total_omitted: 1,
            examples: paths[1..4]
                .iter()
                .map(|path| Arc::from(path.as_str()))
                .collect(),
        }]
    );
}

/// Scenario: Refresh changes concrete membership with stable, retrying, or departing duplicate-instance groups.
/// Guarantees: Set differences and duplicate-group churn recreate only identities whose history is unsafe.
#[test]
fn refresh_group_changes_preserve_stable_histories() {
    let members = [
        ("worker", 0),
        ("worker", 1),
        ("other", 0),
        ("old", 0),
        ("new", 0),
    ];
    let identities = members.map(|(name, index)| InstanceIdentity {
        name: name.to_owned(),
        parent: None,
        index,
    });
    let keys = members.map(|(name, index)| {
        let instance = if index == 0 {
            name.to_owned()
        } else {
            format!("{name}#{index}")
        };
        CounterKey::new(0, &format!(r"\Process({instance})\% Processor Time"))
    });
    let cases = [
        (
            "set_difference",
            &[0, 3][..],
            &[][..],
            &[4, 0][..],
            false,
            &[3][..],
            &[4][..],
        ),
        (
            "unchanged_duplicates",
            &[0, 1][..],
            &[][..],
            &[1, 0][..],
            true,
            &[][..],
            &[][..],
        ),
        (
            "retrying_duplicate",
            &[0][..],
            &[1][..],
            &[1, 0][..],
            true,
            &[][..],
            &[][..],
        ),
        (
            "departed_retry",
            &[0][..],
            &[1][..],
            &[0][..],
            true,
            &[0][..],
            &[0][..],
        ),
        (
            "duplicate_churn",
            &[0, 1, 2][..],
            &[][..],
            &[0, 2][..],
            true,
            &[0, 1][..],
            &[0][..],
        ),
    ];
    let key_set = |indices: &[usize]| {
        indices
            .iter()
            .map(|index| keys[*index].clone())
            .collect::<BTreeSet<_>>()
    };
    for (name, live, retrying, wanted, groups, removed, added) in cases {
        let known_groups = if groups {
            instance_groups(
                live.iter()
                    .chain(retrying)
                    .map(|index| (&keys[*index], &identities[*index])),
            )
        } else {
            BTreeMap::new()
        };
        let wanted_groups = if groups {
            instance_groups(
                wanted
                    .iter()
                    .map(|index| (&keys[*index], &identities[*index])),
            )
        } else {
            BTreeMap::new()
        };
        let actual = refresh_changes_with_group_resets(
            &key_set(live),
            &key_set(retrying),
            &key_set(wanted),
            &known_groups,
            &wanted_groups,
        );
        assert_eq!(
            actual.0,
            key_set(removed).into_iter().collect::<Vec<_>>(),
            "{name}"
        );
        assert_eq!(
            actual.1,
            key_set(added).into_iter().collect::<Vec<_>>(),
            "{name}"
        );
    }
}

/// Scenario: A wildcard expansion contains exact-case, case-distinct, and concrete instances.
/// Guarantees: Aggregate matching is case-sensitive and does not confuse other provider instances.
#[test]
fn identifies_provider_aggregation_instances() {
    let peer = r"\Process(healthy)\Private Bytes".to_owned();
    for (path, aggregate, removed) in [
        (r"\Process(_Total)\Private Bytes", "_Total", true),
        (r"\Custom(_GLOBAL_)\Counter", "_Global_", false),
        (r"\Process(worker)\Private Bytes", "_Total", false),
        (r"\Thread(_Total/_Total)\% Processor Time", "_Total", false),
        (r"\Process(_Total#1)\Private Bytes", "_Total", false),
    ] {
        let mut paths = vec![path.to_owned(), peer.clone()];
        assert!(
            !cleanup_aggregation_instances(&mut paths, Some(aggregate), parse_instance).unwrap()
        );
        assert_eq!(
            paths,
            if removed {
                vec![peer.clone()]
            } else {
                vec![path.to_owned(), peer.clone()]
            }
        );
    }
}

/// Scenario: A wildcard aggregate sorts before concrete instances under a one-instance limit.
/// Guarantees: Aggregate filtering happens before limits so capacity is available to real instances.
#[test]
fn filters_aggregation_before_expansion_limits() {
    let mut paths = vec![
        r"\Process(_Total)\Private Bytes".to_owned(),
        r"\Process(worker)\Private Bytes".to_owned(),
    ];
    assert!(!cleanup_aggregation_instances(&mut paths, Some("_Total"), parse_instance).unwrap());
    let allocation = expansion_allocation(paths.len(), 1, 1);
    assert_eq!(
        allocation,
        ExpansionAllocation {
            retained: 1,
            per_wildcard_omitted: 0,
            total_omitted: 0,
        }
    );
    assert_eq!(paths, [r"\Process(worker)\Private Bytes"]);
}

/// Scenario: A wildcard discovery returns only the configured aggregate instance.
/// Guarantees: The aggregate value is retained while its Go-compatible `instance` label is omitted.
#[test]
fn retains_wildcard_only_aggregation_without_instance_label() {
    let template = r"\Process(*)\Private Bytes";
    let aggregate = r"\Process(_Total)\Private Bytes";
    let mut config = counter(template, "windows.process.private");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: Some("_Total".to_owned()),
    };
    let (state, mut query) = fake_query(
        vec![config],
        vec![fake_counter(aggregate, Number::Integer(42))],
    );
    let _ = state
        .borrow_mut()
        .expansions
        .insert(template.to_owned(), vec![aggregate.to_owned()]);
    query.last_wildcard_refresh = Instant::now() - Duration::from_secs(31);

    let sample = query.collect().unwrap();
    assert_eq!(sample.points.len(), 1);
    assert_eq!(
        sample.points[0]
            .instance
            .as_ref()
            .map(|value| value.name.as_str()),
        Some("_Total")
    );
    assert!(!sample.points[0].emit_instance_attribute);
}
