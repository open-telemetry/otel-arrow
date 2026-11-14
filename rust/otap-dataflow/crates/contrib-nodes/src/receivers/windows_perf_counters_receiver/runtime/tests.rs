// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::test_utils::spawn_children;
use super::*;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

/// Scenario: For each counter failure kind (add, read, collect, recreate), counter 0 fails in three
/// reports while counter 1, which maps the same path, succeeds; then counter 0 succeeds while
/// counter 1 fails.
/// Guarantees: Each failure opens an episode for its configured counter, which warns only when it
/// opens; another counter's success, even on the same path, does not end it; the counter's own
/// success does; and one counter's open episode does not suppress another's warning. Every collect
/// failures count every occurrence, even while the episode suppresses the warning.
#[test]
fn counter_failures_warn_once_per_counter_episode() {
    type Record = fn(&mut Report, usize);
    let kinds: [(Record, Record, fn(&Episodes) -> bool, u64); 4] = [
        (
            |report, counter| report.add_failures.push((counter, 1)),
            |report, counter| report.added.push(counter),
            |episodes| episodes.add,
            4,
        ),
        (
            |report, counter| report.read_failures.push((counter, 1)),
            |report, counter| report.read.push(counter),
            |episodes| episodes.read,
            0,
        ),
        (
            |report, counter| report.collect_failures.push((counter, 1)),
            |report, counter| report.collected.push(counter),
            |episodes| episodes.collect,
            0,
        ),
        (
            |report, counter| report.recreate_failures.push((counter, 1)),
            |report, counter| report.recreated.push(counter),
            |episodes| episodes.recreate,
            0,
        ),
    ];
    for (fail, succeed, open, add_failures) in kinds {
        let mut h = harness();
        let shared = h.receiver.config.counters[1].path.clone();
        h.receiver.config.counters[0].path = shared;
        let report = |failing: usize, succeeding: usize| {
            let mut report = Report::default();
            fail(&mut report, failing);
            succeed(&mut report, succeeding);
            report
        };
        for _ in 0..3 {
            h.receiver.apply(&report(0, 1));
            let counters = &h.receiver.diagnostics.counters;
            assert!(open(&counters[0]) && !open(&counters[1]));
        }
        h.receiver.apply(&report(1, 0));
        let counters = &h.receiver.diagnostics.counters;
        assert!(!open(&counters[0]), "own success ends the episode");
        assert!(open(&counters[1]), "another counter opens its own episode");
        assert_eq!(h.receiver.metrics.counter_add_failures.get(), add_failures);
    }
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

/// Scenario: The factory validates a valid configuration and one with an unknown field.
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
    let (_, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
    harness_with_reporter(metrics_reporter)
}

fn harness_with_reporter(metrics_reporter: MetricsReporter) -> Harness {
    use otel_arrow_dfe_channel::mpsc::Channel;
    use otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel;
    use otel_arrow_dfe_engine::local::message::{LocalReceiver, LocalSender};
    use otel_arrow_dfe_engine::message::{Receiver, Sender};
    use otel_arrow_dfe_engine::testing::{
        test_node, test_pipeline_ctx_with_interests, test_pipeline_runtime_services,
    };

    let config = Config::from_json(&serde_json::json!({
        "collection_interval": "10s",
        "initial_delay": "0s",
        "perfcounters": [
            {"object": "Memory", "counters": [{"name": "Available Bytes"}]},
            {"object": "System", "counters": [{"name": "Threads"}, {"name": "Processes"}]}
        ]
    }))
    .unwrap();
    let (pipeline, _registry) = test_pipeline_ctx_with_interests(
        otel_arrow_dfe_engine::Interests::NODE_OUTPUT_METRICS
            | otel_arrow_dfe_engine::Interests::NODE_LOCAL_DURATION,
    );
    let receiver = WindowsPerfCountersReceiver {
        diagnostics: Diagnostics::new(config.counters.len()),
        config,
        node: "windows-perf-counters".to_owned(),
        metrics: WindowsPerfCountersMetrics::register(&pipeline),
        boundary: ReceiverPullMetrics::register(&pipeline),
    };
    let (output_tx, output) = Channel::new(4);
    let mut outputs = std::collections::HashMap::new();
    let _ = outputs.insert("".into(), Sender::Local(LocalSender::mpsc(output_tx)));
    let (runtime_tx, _) = runtime_ctrl_msg_channel(4);
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
        output,
        control: local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(control_rx))),
        control_tx,
        effect_handler,
    }
}

fn assert_pull_metrics(
    boundary: &mut ReceiverPullMetrics,
    successes: u64,
    failures: u64,
    seconds: f64,
) {
    assert_pull_snapshots(boundary.terminal_snapshots(), successes, failures, seconds);
}

fn assert_pull_snapshots(
    snapshots: Vec<otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot>,
    successes: u64,
    failures: u64,
    seconds: f64,
) {
    use otel_arrow_dfe_telemetry::metrics::MetricValue;

    let mut outcomes = std::collections::BTreeMap::new();
    let mut durations = Vec::new();
    for snapshot in snapshots {
        match (snapshot.descriptor().name, &snapshot.get_metrics()[0]) {
            ("receiver.pulled", MetricValue::U64(value)) => {
                let outcome = snapshot.measurement_attribute_value("outcome").unwrap();
                assert!(outcomes.insert(outcome.to_owned(), *value).is_none());
            }
            ("receiver.processing", MetricValue::Distribution(value)) => {
                durations.push(value.summary());
            }
            other => panic!("unexpected shared metric: {other:?}"),
        }
    }
    assert_eq!(outcomes.remove("success").unwrap_or(0), successes);
    assert_eq!(outcomes.remove("failure").unwrap_or(0), failures);
    assert!(outcomes.is_empty());
    if successes + failures == 0 {
        assert!(durations.is_empty());
    } else {
        assert_eq!(
            durations,
            [(successes + failures, seconds, seconds, seconds)]
        );
    }
}

/// Scenario: Scheduled reports contain points, valid empty data, partial data, recovery,
/// failed query recreation, or only add/collect/read errors; downstream is closed.
/// Guarantees: Usable partial and empty results succeed, wholly failed source work fails,
/// downstream drops cannot change the exactly-once outcome or its duration, and counter-add
/// failures and dropped batches remain separate diagnostics.
#[tokio::test(start_paused = true)]
async fn pulled_report_outcomes_end_before_handoff() {
    let point = super::super::pdh::Point {
        counter: 0,
        instance: None,
        value: 1.0,
        time_unix_nano: 2,
        start_time_unix_nano: 1,
    };
    for (report, success) in [
        (
            Report {
                points: vec![point],
                collected: vec![0],
                ..Report::default()
            },
            true,
        ),
        (
            Report {
                collected: vec![0],
                ..Report::default()
            },
            true,
        ),
        (
            Report {
                collected: vec![0, 1],
                read_failures: vec![(1, 1)],
                ..Report::default()
            },
            true,
        ),
        (
            Report {
                collected: vec![0],
                add_failures: vec![(1, 1)],
                ..Report::default()
            },
            true,
        ),
        (
            Report {
                added: vec![0],
                ..Report::default()
            },
            true,
        ),
        (
            Report {
                collected: vec![0],
                recreate_failures: vec![(0, 1)],
                ..Report::default()
            },
            true,
        ),
        (
            Report {
                add_failures: vec![(0, 1)],
                ..Report::default()
            },
            false,
        ),
        (
            Report {
                collect_failures: vec![(0, 1)],
                ..Report::default()
            },
            false,
        ),
        (
            Report {
                collected: vec![0],
                read_failures: vec![(0, 1)],
                ..Report::default()
            },
            false,
        ),
    ] {
        let mut h = harness();
        let add_failures = report.add_failures.len() as u64;
        let dropped = u64::from(!report.points.is_empty());
        drop(h.output);
        let scrapes = ScriptedScrapes::default();
        let (start, started) = oneshot::channel();
        start.send(Report::default()).unwrap();
        {
            let collect = h
                .receiver
                .collect(&scrapes, started, &mut h.control, &h.effect_handler);
            tokio::pin!(collect);
            assert!(futures::poll!(&mut collect).is_pending());
            tokio::time::advance(Duration::from_secs(2)).await;
            scrapes.0.borrow_mut()[0]
                .1
                .take()
                .unwrap()
                .send(report)
                .unwrap();
            assert!(futures::poll!(&mut collect).is_pending());
        }
        assert_eq!(h.receiver.metrics.counter_add_failures.get(), add_failures);
        assert_eq!(h.receiver.metrics.batches_dropped.get(), dropped);
        assert_pull_metrics(
            &mut h.receiver.boundary,
            u64::from(success),
            u64::from(!success),
            2.0,
        );
    }
}

/// Scenario: An accepted scrape is abandoned by drain, shutdown, or dropping collection.
/// Guarantees: Cancellation counts one failure before cleanup, excludes cleanup time,
/// and discards the late native report without dispatching another scrape.
#[tokio::test(start_paused = true)]
async fn pulled_cancellation_ends_before_worker_cleanup() {
    for mode in 0..3 {
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
            tokio::time::advance(Duration::from_secs(3)).await;
            let deadline = Instant::now() + Duration::from_secs(1);
            let reason = "test cancellation".to_owned();
            if mode < 2 {
                h.control_tx
                    .send(if mode == 0 {
                        NodeControlMsg::DrainIngress { deadline, reason }
                    } else {
                        NodeControlMsg::Shutdown { deadline, reason }
                    })
                    .unwrap();
                assert!(
                    matches!(futures::poll!(&mut collect), std::task::Poll::Ready(Ok((_, drained))) if drained == (mode == 0))
                );
            }
        }
        tokio::time::advance(Duration::from_secs(100)).await;
        assert!(
            scrapes.0.borrow_mut()[0]
                .1
                .take()
                .unwrap()
                .send(Report::default())
                .is_err()
        );
        assert_eq!(scrapes.0.borrow().len(), 1);
        assert_pull_metrics(&mut h.receiver.boundary, 0, 1, 3.0);
        assert!(h.output.try_recv().is_err());
    }
}

/// Scenario: The worker disappears during startup or an accepted scrape; after a simulated
/// cleanup delay, finalization encounters an available or closed metrics channel.
/// Guarantees: Fatal finalization hands off diagnostics and the accepted attempt's failure
/// and duration exactly once, excludes startup and cleanup time, and preserves the worker
/// error even when the metrics handoff fails.
#[tokio::test(start_paused = true)]
async fn pulled_worker_loss_hands_off_terminal_metrics() {
    for accepted in [false, true] {
        for reporter_available in [true, false] {
            let (reports, reporter) = MetricsReporter::create_new_and_receiver(8);
            let reports = reporter_available.then_some(reports);
            let mut h = harness_with_reporter(reporter);
            let scrapes = ScriptedScrapes::default();
            let (start, started) = oneshot::channel();
            let result = {
                let collect =
                    h.receiver
                        .collect(&scrapes, started, &mut h.control, &h.effect_handler);
                tokio::pin!(collect);
                assert!(futures::poll!(&mut collect).is_pending());
                if accepted {
                    start.send(Report::default()).unwrap();
                    assert!(futures::poll!(&mut collect).is_pending());
                    tokio::time::advance(Duration::from_secs(2)).await;
                    drop(scrapes.0.borrow_mut()[0].1.take());
                } else {
                    drop(start);
                }
                match futures::poll!(&mut collect) {
                    std::task::Poll::Ready(result) => result,
                    std::task::Poll::Pending => panic!("worker loss must end collection"),
                }
            };
            tokio::time::advance(Duration::from_secs(100)).await;
            let deadline = tokio::time::Instant::now().into_std() + WORKER_SHUTDOWN_MAX_WAIT;
            let result = h.receiver.finish(result, deadline, &h.effect_handler).await;
            assert!(matches!(
                result,
                Err(Error::ReceiverError {
                    kind: ReceiverErrorKind::Other,
                    error,
                    ..
                }) if error == "PDH worker stopped"
            ));
            if let Some(reports) = reports {
                let (diagnostics, pulled): (Vec<_>, Vec<_>) =
                    reports.try_iter().partition(|snapshot| {
                        snapshot.descriptor().name == "receiver.windows_perf_counters"
                    });
                assert_eq!(diagnostics.len(), 1);
                assert_pull_snapshots(pulled, 0, u64::from(accepted), 2.0);
            }
        }
    }
}

/// Scenario: A scrape passes its warning timeout, skips a scheduler tick, then completes.
/// Guarantees: Warning expiry and overrun do not terminate or create attempts; periodic
/// reporting leaves the attempt active, then emits its late success exactly once.
#[tokio::test(start_paused = true)]
async fn pulled_warning_timeout_and_overrun_do_not_count_attempts() {
    let mut h = harness();
    h.receiver.config.timeout = Some(Duration::from_secs(3));
    let scrapes = ScriptedScrapes::default();
    let (start, started) = oneshot::channel();
    start.send(Report::default()).unwrap();
    let (reports, metrics_reporter) = MetricsReporter::create_new_and_receiver(8);
    let completed: Vec<_>;
    {
        let collect = h
            .receiver
            .collect(&scrapes, started, &mut h.control, &h.effect_handler);
        tokio::pin!(collect);
        assert!(futures::poll!(&mut collect).is_pending());
        tokio::time::advance(Duration::from_secs(4)).await;
        assert!(futures::poll!(&mut collect).is_pending());
        h.control_tx
            .send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: metrics_reporter.clone(),
            })
            .unwrap();
        assert!(futures::poll!(&mut collect).is_pending());
        let reported: Vec<_> = reports.drain().collect();
        assert_eq!(reported.len(), 1, "only diagnostics before completion");
        assert_eq!(
            reported[0].descriptor().name,
            "receiver.windows_perf_counters"
        );
        tokio::time::advance(Duration::from_secs(8)).await;
        assert!(futures::poll!(&mut collect).is_pending());
        scrapes.0.borrow_mut()[0]
            .1
            .take()
            .unwrap()
            .send(Report {
                collected: vec![0],
                ..Report::default()
            })
            .unwrap();
        assert!(futures::poll!(&mut collect).is_pending());
        assert_eq!(scrapes.0.borrow().len(), 1);
        h.control_tx
            .send(NodeControlMsg::CollectTelemetry { metrics_reporter })
            .unwrap();
        assert!(futures::poll!(&mut collect).is_pending());
        completed = reports.drain().collect();
    }
    let diagnostics = completed
        .iter()
        .find(|snapshot| snapshot.descriptor().name == "receiver.windows_perf_counters")
        .unwrap();
    let overruns = diagnostics
        .descriptor()
        .metrics
        .iter()
        .position(|metric| metric.name == "scrape.overruns")
        .unwrap();
    assert_eq!(
        diagnostics.get_metrics()[overruns],
        otel_arrow_dfe_telemetry::metrics::MetricValue::U64(1)
    );
    assert_pull_snapshots(
        completed
            .into_iter()
            .filter(|snapshot| snapshot.descriptor().name != "receiver.windows_perf_counters")
            .collect(),
        1,
        0,
        12.0,
    );
    assert_pull_metrics(&mut h.receiver.boundary, 0, 0, 0.0);
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

/// Scenario: A scrape reports collection failures for counters 1 and 2 while counter 0 collects and
/// publishes a point.
/// Guarantees: The usable partial result records one success, no failure, and publishes the
/// healthy point while retaining the failing counters' diagnostic episodes.
#[tokio::test(start_paused = true)]
async fn partial_scrape_records_success_and_publishes_healthy_points() {
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
        scrapes.0.borrow_mut()[0]
            .1
            .take()
            .unwrap()
            .send(Report {
                points: vec![super::super::pdh::Point {
                    counter: 0,
                    instance: None,
                    value: 1.0,
                    time_unix_nano: 2,
                    start_time_unix_nano: 1,
                }],
                collected: vec![0],
                collect_failures: vec![(1, 0xC000_0BB9), (2, 0xC000_0BB9)],
                ..Report::default()
            })
            .unwrap();
        assert!(futures::poll!(&mut collect).is_pending());
    }
    assert!(h.receiver.diagnostics.counters[1].collect);
    assert!(h.receiver.diagnostics.counters[2].collect);
    assert_pull_metrics(&mut h.receiver.boundary, 1, 0, 0.0);
    assert!(h.output.try_recv().is_ok(), "healthy point published");
}

/// Runs one scheduled scrape under paused time for `elapsed`, then completes it if `complete`.
async fn scrape_once(h: &mut Harness, elapsed: Duration, complete: bool) {
    let scrapes = ScriptedScrapes::default();
    let (start, started) = oneshot::channel();
    start.send(Report::default()).unwrap();
    let collect = h
        .receiver
        .collect(&scrapes, started, &mut h.control, &h.effect_handler);
    tokio::pin!(collect);
    assert!(futures::poll!(&mut collect).is_pending());
    tokio::time::advance(elapsed).await;
    assert!(futures::poll!(&mut collect).is_pending());
    if complete {
        let report = scrapes.0.borrow_mut()[0].1.take().unwrap();
        report.send(Report::default()).unwrap();
        assert!(futures::poll!(&mut collect).is_pending());
    }
    assert_eq!(scrapes.0.borrow().len(), 1, "no tick within the interval");
}

/// Scenario: Under paused time with a 3 s timeout and a 10 s interval, a scrape is still running
/// 4 s after it started, another completes late at 4 s, one completes on time, and one is again
/// still running at 4 s.
/// Guarantees: Timeout expiry opens an overrun episode without counting a skipped tick; a late
/// completion does not end it; an on-time completion re-arms it, so the next expiry opens a new
/// episode.
#[tokio::test(start_paused = true)]
async fn timeout_expiry_opens_overrun_until_on_time_completion() {
    let mut h = harness();
    h.receiver.config.timeout = Some(Duration::from_secs(3));
    scrape_once(&mut h, Duration::from_secs(4), false).await;
    assert!(h.receiver.diagnostics.overrunning, "expiry opens episode");
    scrape_once(&mut h, Duration::from_secs(4), true).await;
    assert!(
        h.receiver.diagnostics.overrunning,
        "late completion keeps it"
    );
    scrape_once(&mut h, Duration::from_secs(1), true).await;
    assert!(!h.receiver.diagnostics.overrunning, "on time re-arms");
    scrape_once(&mut h, Duration::from_secs(4), false).await;
    assert!(
        h.receiver.diagnostics.overrunning,
        "expiry opens new episode"
    );
    assert_eq!(h.receiver.metrics.scrape_overruns.get(), 0);
}

/// Scenario: Before the first scheduled scrape, startup reports query and counter-add failures.
/// Guarantees: Startup emits counter diagnostics but no attempt or duration; cancelling the
/// scheduler before acceptance does not create an attempt either.
#[tokio::test(start_paused = true)]
async fn startup_failure_reports_diagnostics_without_attempt() {
    let mut h = harness();
    h.receiver.config.initial_delay = Duration::from_secs(30);
    let (start, started) = oneshot::channel();
    start
        .send(Report {
            collect_failures: vec![(0, 0xC000_0BB9)],
            add_failures: vec![(1, 0xC000_0BB9)],
            active_counters: 2,
            ..Report::default()
        })
        .unwrap();
    let scrapes = ScriptedScrapes::default();
    {
        let collect = h
            .receiver
            .collect(&scrapes, started, &mut h.control, &h.effect_handler);
        tokio::pin!(collect);
        assert!(futures::poll!(&mut collect).is_pending());
        let (reports, metrics_reporter) = MetricsReporter::create_new_and_receiver(8);
        h.control_tx
            .send(NodeControlMsg::CollectTelemetry { metrics_reporter })
            .unwrap();
        assert!(futures::poll!(&mut collect).is_pending());
        let reported: Vec<_> = reports.drain().collect();
        assert_eq!(reported.len(), 1, "startup only reports source diagnostics");
        assert_eq!(
            reported[0].descriptor().name,
            "receiver.windows_perf_counters"
        );
        let failures = reported[0]
            .descriptor()
            .metrics
            .iter()
            .position(|metric| metric.name == "counter.add.failures")
            .unwrap();
        assert_eq!(reported[0].get_metrics()[failures].to_u64_lossy(), 1);
    }
    assert!(
        scrapes.0.borrow().is_empty(),
        "nothing accepted before the first tick"
    );
    assert!(h.receiver.diagnostics.counters[0].collect);
    assert!(h.receiver.diagnostics.counters[1].add);
    assert_eq!(
        h.receiver.metrics.counter_add_failures.get(),
        0,
        "reported once"
    );
    assert_eq!(h.receiver.metrics.active_counters.get(), 2);
    assert_pull_metrics(&mut h.receiver.boundary, 0, 0, 0.0);
}

/// Scenario: Batches are sent until the downstream channel is full, one is drained, then the
/// channel is closed.
/// Guarantees: Sends to a full or closed channel count batches.dropped and do not fail the
/// receiver; a send after draining is accepted, not counted, and ends the dropped episode.
#[test]
fn full_or_closed_downstream_counts_dropped_batches() {
    let mut h = harness();
    let point = super::super::pdh::Point {
        counter: 0,
        instance: None,
        value: 1.0,
        time_unix_nano: 2,
        start_time_unix_nano: 1,
    };
    let batch = |counters: &[super::super::config::CounterConfig]| {
        let records = project_points(counters, std::slice::from_ref(&point))
            .unwrap()
            .unwrap();
        OtapPdata::new(Context::default(), records.into())
    };
    for _ in 0..4 {
        let data = batch(&h.receiver.config.counters);
        h.receiver.send(&h.effect_handler, data).unwrap();
    }
    assert_eq!(h.receiver.metrics.batches_dropped.get(), 0);
    let data = batch(&h.receiver.config.counters);
    h.receiver.send(&h.effect_handler, data).unwrap();
    assert_eq!(h.receiver.metrics.batches_dropped.get(), 1);
    assert!(h.receiver.diagnostics.dropping);

    assert!(h.output.try_recv().is_ok());
    let data = batch(&h.receiver.config.counters);
    h.receiver.send(&h.effect_handler, data).unwrap();
    assert_eq!(h.receiver.metrics.batches_dropped.get(), 1);
    assert!(!h.receiver.diagnostics.dropping);

    drop(h.output);
    let data = batch(&h.receiver.config.counters);
    h.receiver.send(&h.effect_handler, data).unwrap();
    assert_eq!(h.receiver.metrics.batches_dropped.get(), 2);
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
