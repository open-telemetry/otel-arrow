// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: Two mappings share a failing path, spelled with different case; adds fail, the path
/// is added, then fails again; another path fails independently.
/// Guarantees: Each path warns once per failure episode however many mappings use it, and a
/// success re-arms only that path.
#[test]
fn add_failure_warns_once_per_path_until_added() {
    let mut diagnostics = Diagnostics::new(0);
    assert!(diagnostics.add_failed(r"\Memory\No Such Counter"));
    assert!(!diagnostics.add_failed(r"\memory\no such counter"));
    assert!(diagnostics.add_failed(r"\System\Missing"));
    diagnostics.added(r"\Memory\No Such Counter");
    assert!(diagnostics.add_failed(r"\Memory\No Such Counter"));
    assert!(!diagnostics.add_failed(r"\System\Missing"));
}

/// Scenario: A path's collections fail repeatedly, it collects, then fails again; another path
/// fails independently.
/// Guarantees: One collect warning per path per failure episode; a collection re-arms only that
/// path.
#[test]
fn collect_failure_warns_once_per_path_episode() {
    let mut diagnostics = Diagnostics::new(0);
    assert!(diagnostics.collect_failed(r"\System\Threads"));
    assert!(!diagnostics.collect_failed(r"\system\threads"));
    assert!(diagnostics.collect_failed(r"\System\Processes"));
    diagnostics.collected(r"\System\Threads");
    assert!(diagnostics.collect_failed(r"\System\Threads"));
    assert!(!diagnostics.collect_failed(r"\System\Processes"));
}

/// Scenario: The overrun latch is triggered twice, released by a late completion, then by an
/// on-time completion. This exercises only the warn-once state, not the scheduler.
/// Guarantees: The overrun warning fires once per episode; only an on-time completion ends the
/// episode.
#[test]
fn overrun_warns_once_until_on_time_completion() {
    let mut diagnostics = Diagnostics::new(0);
    assert!(diagnostics.overrun());
    assert!(!diagnostics.overrun());
    diagnostics.completed(false);
    assert!(!diagnostics.overrun());
    diagnostics.completed(true);
    assert!(diagnostics.overrun());
}

/// Scenario: One counter's replacement queries fail repeatedly, succeed, then fail again; another
/// counter fails independently.
/// Guarantees: One recreate warning per counter per failure episode.
#[test]
fn recreate_failure_warns_once_per_counter_episode() {
    let mut diagnostics = Diagnostics::new(2);
    assert!(diagnostics.recreate_failed(0));
    assert!(!diagnostics.recreate_failed(0));
    assert!(diagnostics.recreate_failed(1));
    diagnostics.recreated(0);
    assert!(diagnostics.recreate_failed(0));
    assert!(!diagnostics.recreate_failed(1));
}

/// Scenario: A path's array read fails repeatedly through two mappings, succeeds, then fails again;
/// another path fails independently.
/// Guarantees: One read warning per path per failure episode.
#[test]
fn read_failure_warns_once_per_path_episode() {
    let mut diagnostics = Diagnostics::new(0);
    assert!(diagnostics.read_failed(r"\System\Threads"));
    assert!(!diagnostics.read_failed(r"\System\Threads"));
    assert!(diagnostics.read_failed(r"\System\Processes"));
    diagnostics.read(r"\System\Threads");
    assert!(diagnostics.read_failed(r"\System\Threads"));
    assert!(!diagnostics.read_failed(r"\System\Processes"));
}

/// Scenario: Downstream rejects batches repeatedly, accepts one, then rejects again.
/// Guarantees: The dropped-batch warning fires once per episode.
#[test]
fn dropped_batches_warn_once_per_episode() {
    let mut diagnostics = Diagnostics::new(0);
    assert!(diagnostics.dropped());
    assert!(!diagnostics.dropped());
    diagnostics.sent();
    assert!(diagnostics.dropped());
}

/// Scenario: The factory validates a valid configuration and one with a removed field.
/// Guarantees: Static validation uses the receiver's config rules.
#[test]
fn factory_validates_config() {
    let validate = WINDOWS_PERF_COUNTERS_RECEIVER.validate_config;
    let counter =
        serde_json::json!({"object": "Memory", "counters": [{"name": "Available Bytes"}]});
    assert!(validate(&serde_json::json!({"perfcounters": [counter.clone()]})).is_ok());
    assert!(
        validate(&serde_json::json!({"perfcounters": [counter], "max_expanded_counters": 1}))
            .is_err()
    );
}

/// Scripted scrape source: records when each scrape was requested and holds its report sender.
#[derive(Default)]
struct ScriptedScrapes(
    std::cell::RefCell<Vec<(tokio::time::Instant, Option<oneshot::Sender<Report>>)>>,
);

impl Scrape for ScriptedScrapes {
    fn scrape(&self) -> oneshot::Receiver<Report> {
        let (sender, receiver) = oneshot::channel();
        self.0
            .borrow_mut()
            .push((tokio::time::Instant::now(), Some(sender)));
        receiver
    }
}

/// A receiver for three counters, scheduled every 10 s with no initial delay, with its control
/// and output channels.
struct Harness {
    receiver: WindowsPerfCountersReceiver,
    control_tx: otel_arrow_dfe_channel::mpsc::Sender<NodeControlMsg<OtapPdata>>,
    control: local::ControlChannel<OtapPdata>,
    effect_handler: local::EffectHandler<OtapPdata>,
    output: otel_arrow_dfe_channel::mpsc::Receiver<OtapPdata>,
}

fn harness() -> Harness {
    use otel_arrow_dfe_channel::mpsc::Channel;
    use otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel;
    use otel_arrow_dfe_engine::local::message::{LocalReceiver, LocalSender};
    use otel_arrow_dfe_engine::message::{Receiver, Sender};
    use otel_arrow_dfe_engine::testing::{
        test_node, test_pipeline_ctx, test_pipeline_runtime_services,
    };
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

    let config = Config::from_json(&serde_json::json!({
        "collection_interval": "10s",
        "initial_delay": "0s",
        "perfcounters": [
            {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
            {"object": "System", "counters": [{"name": "Threads"}, {"name": "Processes"}]}
        ]
    }))
    .unwrap();
    let (pipeline, _registry) = test_pipeline_ctx();
    let receiver = WindowsPerfCountersReceiver {
        diagnostics: Diagnostics::new(config.counters.len()),
        config,
        node: "windows-perf-counters".to_owned(),
        metrics: WindowsPerfCountersMetrics::register(&pipeline),
    };
    let (output_tx, output) = Channel::new(4);
    let mut outputs = std::collections::HashMap::new();
    let _ = outputs.insert("".into(), Sender::Local(LocalSender::mpsc(output_tx)));
    let (runtime_tx, _) = runtime_ctrl_msg_channel(4);
    let (_, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
    let effect_handler = local::EffectHandler::new(
        test_node("windows-perf-counters"),
        outputs,
        None,
        runtime_tx,
        metrics_reporter,
        test_pipeline_runtime_services(),
    );
    let (control_tx, control_rx) = Channel::new(4);
    Harness {
        receiver,
        control_tx,
        control: local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(control_rx))),
        effect_handler,
        output,
    }
}

/// Scenario: Under paused time, one scheduler's start never completes and it receives Shutdown;
/// another's start is held for 25 s, past three due ticks, then released.
/// Guarantees: Control is handled while start is pending; no scrape is requested while start is
/// pending; the first scrape is requested as soon as start completes, not at a later tick.
#[tokio::test(start_paused = true)]
async fn first_due_scrape_runs_when_start_completes() {
    let mut gated = harness();
    let (_never, never_started) = oneshot::channel();
    gated
        .control_tx
        .send(NodeControlMsg::Shutdown {
            deadline: Instant::now() + Duration::from_secs(1),
            reason: "test".to_owned(),
        })
        .unwrap();
    let stopped = tokio::time::timeout(
        Duration::from_secs(60),
        gated.receiver.collect(
            &ScriptedScrapes::default(),
            never_started,
            &mut gated.control,
            &gated.effect_handler,
        ),
    )
    .await
    .expect("control handled while start is pending");
    assert!(matches!(stopped, Ok((_, false))));

    let mut h = harness();
    let scrapes = ScriptedScrapes::default();
    let (start, started) = oneshot::channel();
    let collect = h
        .receiver
        .collect(&scrapes, started, &mut h.control, &h.effect_handler);
    tokio::pin!(collect);
    assert!(futures::poll!(&mut collect).is_pending());
    tokio::time::advance(Duration::from_secs(25)).await;
    assert!(futures::poll!(&mut collect).is_pending());
    assert!(scrapes.0.borrow().is_empty());
    let released = tokio::time::Instant::now();
    start.send(Report::default()).unwrap();
    assert!(futures::poll!(&mut collect).is_pending());
    let requested: Vec<_> = scrapes.0.borrow().iter().map(|(at, _)| *at).collect();
    assert_eq!(requested, [released]);
}

/// Scenario: Under paused time, a scrape's report arrives exactly when the next tick is due, and
/// the scheduler is polled once with both ready.
/// Guarantees: The due tick is handled first: it counts one overrun and opens the overrun episode,
/// the late report is applied, and no catch-up scrape starts at that tick.
#[tokio::test(start_paused = true)]
async fn due_tick_is_handled_before_report() {
    let mut h = harness();
    let scrapes = ScriptedScrapes::default();
    let (start, started) = oneshot::channel();
    start.send(Report::default()).unwrap();
    {
        let collect = h
            .receiver
            .collect(&scrapes, started, &mut h.control, &h.effect_handler);
        tokio::pin!(collect);
        assert!(futures::poll!(&mut collect).is_pending());
        let report = scrapes.0.borrow_mut()[0].1.take().unwrap();
        tokio::time::advance(Duration::from_secs(10)).await;
        report.send(Report::default()).unwrap();
        assert!(futures::poll!(&mut collect).is_pending());
        assert_eq!(scrapes.0.borrow().len(), 1, "no catch-up scrape");
    }
    assert_eq!(h.receiver.metrics.scrape_overruns.get(), 1);
    assert!(h.receiver.diagnostics.overrunning);
}

/// Scenario: A scrape reports counter 1's collection failure while counter 0 collects and
/// publishes a point, as the collector does when only one per-counter query fails.
/// Guarantees: The scrape counts one scrapes and one scrape.failures, and the healthy point is
/// still published.
#[tokio::test(start_paused = true)]
async fn failed_scrape_still_publishes_healthy_points() {
    let mut h = harness();
    let scrapes = ScriptedScrapes::default();
    let (start, started) = oneshot::channel();
    start.send(Report::default()).unwrap();
    {
        let collect = h
            .receiver
            .collect(&scrapes, started, &mut h.control, &h.effect_handler);
        tokio::pin!(collect);
        assert!(futures::poll!(&mut collect).is_pending());
        let report = scrapes.0.borrow_mut()[0].1.take().unwrap();
        report
            .send(Report {
                points: vec![super::super::pdh::Point {
                    counter: 0,
                    instance: None,
                    value: 1.0,
                    time_unix_nano: 2,
                    start_time_unix_nano: 1,
                }],
                collect_failures: vec![(1, 0xC000_0BB9)],
                collected: vec![0],
                ..Report::default()
            })
            .unwrap();
        assert!(futures::poll!(&mut collect).is_pending());
    }
    assert_eq!(h.receiver.metrics.scrape_failures.get(), 1);
    assert_eq!(h.receiver.metrics.scrapes.get(), 1);
    assert!(h.output.try_recv().is_ok(), "healthy point published");
}

/// Scenario: Counter A's query fails on every scrape; three scrapes later counter B's also fails,
/// while counter C keeps collecting.
/// Guarantees: A's persistent failure does not suppress B's warning (B opens its own episode);
/// scrapes and scrape.failures each count once per scrape, so scrapes keeps increasing while a
/// healthy counter collects.
#[test]
fn persistent_failure_does_not_hide_another_counter() {
    let mut h = harness();
    let b = h.receiver.config.counters[1].path.to_lowercase();
    let scrape = |failing: &[usize], collected: &[usize]| Report {
        collect_failures: failing
            .iter()
            .map(|&counter| (counter, 0xC000_0BB9))
            .collect(),
        collected: collected.to_vec(),
        ..Report::default()
    };
    for _ in 0..3 {
        h.receiver.apply(&scrape(&[0], &[1, 2]), None);
    }
    assert!(!h.receiver.diagnostics.collect_failing.contains(&b));
    h.receiver.apply(&scrape(&[0, 1], &[2]), None);
    assert!(
        h.receiver.diagnostics.collect_failing.contains(&b),
        "B warned"
    );
    assert_eq!(h.receiver.metrics.scrapes.get(), 4);
    assert_eq!(h.receiver.metrics.scrape_failures.get(), 4);
}
/// Owned child processes sharing one executable name, killed on drop.
struct Children(Vec<std::process::Child>, std::path::PathBuf);

impl Drop for Children {
    fn drop(&mut self) {
        for child in &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_dir_all(&self.1);
    }
}

/// Starts two long-running copies of `ping.exe` named `name.exe`.
fn spawn_children(name: &str) -> Children {
    let dir = std::env::temp_dir().join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join(format!("{name}.exe"));
    let _ = std::fs::copy(r"C:\Windows\System32\PING.EXE", &exe).unwrap();
    let mut children = Children(Vec::new(), dir);
    for _ in 0..2 {
        children.0.push(
            std::process::Command::new(&exe)
                .args(["-n", "3600", "127.0.0.1"])
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap(),
        );
    }
    children
}

/// Decoded string attributes of one data point.
fn string_attributes(
    attributes: &[otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue],
) -> std::collections::BTreeMap<String, String> {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value;
    attributes
        .iter()
        .map(|attribute| {
            let Some(Value::StringValue(value)) = attribute
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref())
            else {
                panic!("expected a string attribute");
            };
            (attribute.key.clone(), value.clone())
        })
        .collect()
}

/// Scenario: The factory builds the receiver on this machine with a multi-counter config: a
/// non-instanced Gauge with a static attribute, an exact Processor instance, Processor "*" with
/// and without the `_Total` opt-in, two Sums over a partial pattern matching two owned child
/// processes with one executable name (the cumulative one with `recreate_query`), and a missing
/// counter. It runs until every metric is observed and the recreated Sum appears in two batches,
/// then shuts down.
/// Guarantees: Decoded output carries the configured kinds and Sum flags (cumulative Sums have a
/// start time, delta Sums do not), exact and "*" instance values, `_Total` only when opted in, a
/// `#1` suffix on the duplicate process name, no instance on the non-instanced counter, static
/// attributes, and nothing for the missing counter; the `recreate_query` Sum keeps publishing and
/// its start time stays the same in every batch, moving back only to an earlier point time;
/// shutdown completes.
#[test]
#[ignore = "requires Windows performance counters"]
fn live_receiver_end_to_end() {
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::capability::registry::Capabilities;
    use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node, test_pipeline_ctx};
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data;
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    // OTLP AggregationTemporality values.
    const DELTA: i32 = 1;
    const CUMULATIVE: i32 = 2;

    let name = format!("wpcd{}", std::process::id());
    let _children = spawn_children(&name);
    std::thread::sleep(Duration::from_secs(1));

    let mut node_config = NodeUserConfig::new_receiver_config(WINDOWS_PERF_COUNTERS_RECEIVER_URN);
    node_config.config = serde_json::json!({
        "collection_interval": "200ms",
        "initial_delay": "0s",
        "metrics": {
            "mem": {"unit": "By", "gauge": {}},
            "cpu.exact": {"unit": "%", "gauge": {}},
            "cpu.all": {"unit": "%", "gauge": {}},
            "cpu.total": {"unit": "%", "gauge": {}},
            "proc.id": {"sum": {"aggregation": "cumulative", "monotonic": false}},
            "proc.threads": {"sum": {"aggregation": "delta", "monotonic": true}}
        },
        "perfcounters": [
            {"object": "Memory", "counters": [
                {"name": "Available Bytes", "metric": "mem", "attributes": {"role": "e2e"}},
                {"name": "No Such Counter"}
            ]},
            {"object": "Processor", "instances": "0",
             "counters": [{"name": "% Processor Time", "metric": "cpu.exact"}]},
            {"object": "Processor", "instances": "*",
             "counters": [{"name": "% Processor Time", "metric": "cpu.all"}]},
            {"object": "Processor", "instances": ["*", "_Total"],
             "counters": [{"name": "% Processor Time", "metric": "cpu.total"}]},
            {"object": "Process", "instances": format!("{name}*"), "counters": [
                {"name": "ID Process", "metric": "proc.id", "recreate_query": true},
                {"name": "Thread Count", "metric": "proc.threads"}
            ]}
        ]
    });
    let runtime = TestRuntime::<OtapPdata>::new();
    let (pipeline, _registry) = test_pipeline_ctx();
    let receiver = (WINDOWS_PERF_COUNTERS_RECEIVER.create)(
        pipeline,
        test_node("windows-perf-counters"),
        Arc::new(node_config),
        runtime.config(),
        &Capabilities::empty(),
    )
    .unwrap();
    let processes: BTreeSet<Option<String>> =
        [Some(name.clone()), Some(format!("{name}#1"))].into();
    runtime
        .set_receiver(receiver)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            let mut seen: BTreeMap<String, BTreeSet<Option<String>>> = BTreeMap::new();
            let mut recreated_starts = Vec::new();
            for _ in 0..25 {
                let data = tokio::time::timeout(Duration::from_secs(10), ctx.recv())
                    .await
                    .expect("batch before timeout")
                    .expect("batch");
                let records = data.payload_ref().otap_ref().expect("OTAP payload").clone();
                let request = super::super::test_utils::decode(records);
                for metric in &request.resource_metrics[0].scope_metrics[0].metrics {
                    let points = match (metric.name.as_str(), &metric.data) {
                        ("mem" | "cpu.exact" | "cpu.all" | "cpu.total", Some(Data::Gauge(g))) => {
                            &g.data_points
                        }
                        ("proc.id", Some(Data::Sum(sum))) => {
                            assert_eq!(sum.aggregation_temporality, CUMULATIVE);
                            assert!(!sum.is_monotonic);
                            let start = sum.data_points[0].start_time_unix_nano;
                            assert!(start > 0);
                            assert!(
                                sum.data_points
                                    .iter()
                                    .all(|p| p.start_time_unix_nano == start)
                            );
                            recreated_starts.push((start, sum.data_points[0].time_unix_nano));
                            &sum.data_points
                        }
                        ("proc.threads", Some(Data::Sum(sum))) => {
                            assert_eq!(sum.aggregation_temporality, DELTA);
                            assert!(sum.is_monotonic);
                            assert!(sum.data_points.iter().all(|p| p.start_time_unix_nano == 0));
                            &sum.data_points
                        }
                        (other, data) => panic!("unexpected metric {other}: {data:?}"),
                    };
                    let instances = seen.entry(metric.name.clone()).or_default();
                    for point in points {
                        let mut attributes = string_attributes(&point.attributes);
                        let instance = attributes.remove("instance");
                        if metric.name == "mem" {
                            assert_eq!(attributes, [("role".into(), "e2e".into())].into());
                        } else {
                            assert!(attributes.is_empty(), "{attributes:?}");
                        }
                        let _ = instances.insert(instance);
                    }
                }
                let total = Some("_Total".to_owned());
                if seen.len() == 6
                    && seen["cpu.total"].contains(&total)
                    && seen["proc.id"] == processes
                    && seen["proc.threads"] == processes
                    && recreated_starts.len() >= 2
                {
                    break;
                }
            }
            let zero = Some("0".to_owned());
            let total = Some("_Total".to_owned());
            assert_eq!(seen.get("mem"), Some(&[None].into()));
            assert_eq!(seen.get("cpu.exact"), Some(&[zero.clone()].into()));
            let all = &seen["cpu.all"];
            assert!(all.contains(&zero) && !all.contains(&total) && !all.contains(&None));
            let opted = &seen["cpu.total"];
            assert!(opted.contains(&zero) && opted.contains(&total));
            assert_eq!(seen.get("proc.id"), Some(&processes));
            assert_eq!(seen.get("proc.threads"), Some(&processes));
            assert!(recreated_starts.len() >= 2, "{recreated_starts:?}");
            assert!(
                recreated_starts
                    .windows(2)
                    .all(|pair| pair[1].0 == pair[0].0.min(pair[1].1)),
                "{recreated_starts:?}"
            );
            ctx.send_control_msg(NodeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(2),
                reason: "test complete".to_owned(),
            })
            .await
            .unwrap();
        });
}
