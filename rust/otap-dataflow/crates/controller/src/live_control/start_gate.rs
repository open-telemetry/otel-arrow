// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Build-then-run gate for pipeline instances created by one reconcile.
//!
//! Topic receivers subscribe while their pipeline is built, and a publish
//! with no subscriber is dropped. Holding every new instance between build
//! and run until all of them are built guarantees that no new pipeline can
//! accept or emit data before every new subscription exists, regardless of
//! topic wiring between pipelines. Instances already running are never held.
//!
//! This relies on two invariants: topic receivers subscribe while being built,
//! and no node publishes to a topic while being built (only once running).

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const INTERRUPT_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug)]
enum GateDecision {
    Closed,
    Open,
    Aborted(String),
}

#[derive(Debug)]
struct GateState {
    built: usize,
    decision: GateDecision,
}

/// Shared gate that releases a set of built pipeline instances together.
#[derive(Debug)]
pub(crate) struct StartGate {
    state: Mutex<GateState>,
    changed: Condvar,
}

impl StartGate {
    /// Creates a closed gate.
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(GateState {
                built: 0,
                decision: GateDecision::Closed,
            }),
            changed: Condvar::new(),
        })
    }

    /// Returns a ticket for one instance that will be held by this gate.
    pub(crate) fn ticket(self: &Arc<Self>, label: String) -> StartGateTicket {
        StartGateTicket {
            gate: Some(Arc::clone(self)),
            label,
        }
    }

    /// Opens the gate unless it was already decided.
    pub(crate) fn open(&self) {
        self.decide(GateDecision::Open);
    }

    /// Aborts the gate unless it was already decided.
    pub(crate) fn abort(&self, reason: String) {
        self.decide(GateDecision::Aborted(reason));
    }

    fn decide(&self, decision: GateDecision) {
        let mut state = self.lock();
        if matches!(state.decision, GateDecision::Closed) {
            state.decision = decision;
            self.changed.notify_all();
        }
    }

    /// Waits until `expected` instances are built, the gate is aborted, the deadline passes, or
    /// `interrupted` returns a reason (polled every 50ms), for example an engine shutdown.
    pub(crate) fn wait_until_built(
        &self,
        expected: usize,
        deadline: Instant,
        interrupted: impl Fn() -> Option<String>,
    ) -> Result<(), String> {
        let mut state = self.lock();
        loop {
            match &state.decision {
                GateDecision::Aborted(reason) => return Err(reason.clone()),
                GateDecision::Open => return Ok(()),
                GateDecision::Closed => {}
            }
            // Poll without the gate lock: the callback may take other locks.
            drop(state);
            if let Some(reason) = interrupted() {
                return Err(reason);
            }
            state = self.lock();
            if !matches!(state.decision, GateDecision::Closed) {
                continue;
            }
            if state.built >= expected {
                return Ok(());
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(format!(
                    "timed out waiting for new pipeline instances to build ({} of {expected} built)",
                    state.built
                ));
            }
            let wait = (deadline - now).min(INTERRUPT_POLL_INTERVAL);
            state = self
                .changed
                .wait_timeout(state, wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
    }

    /// Blocks until the gate is opened or aborted.
    pub(crate) fn wait_decision(&self) -> Result<(), String> {
        let mut state = self.lock();
        loop {
            match &state.decision {
                GateDecision::Open => return Ok(()),
                GateDecision::Aborted(reason) => return Err(reason.clone()),
                GateDecision::Closed => {
                    state = self
                        .changed
                        .wait(state)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                }
            }
        }
    }

    fn report_built(&self) {
        let mut state = self.lock();
        state.built += 1;
        self.changed.notify_all();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, GateState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Aborts the gate on drop unless it was already decided.
///
/// The reconcile thread holds this so an early return never leaves instances
/// waiting at a gate that nobody will open.
pub(crate) struct StartGateAbortOnDrop(pub(crate) Arc<StartGate>);

impl Drop for StartGateAbortOnDrop {
    fn drop(&mut self) {
        self.0
            .abort("reconcile ended before new pipelines were released".to_owned());
    }
}

/// One instance's handle on a start gate.
///
/// Dropping a ticket that never reported built (for example after a build
/// error or panic) aborts the gate so other instances do not wait forever.
#[derive(Debug)]
pub(crate) struct StartGateTicket {
    gate: Option<Arc<StartGate>>,
    label: String,
}

impl StartGateTicket {
    /// Reports this instance as built and waits for the gate decision.
    ///
    /// Returns `Ok` when the instance may run, or the abort reason.
    pub(crate) fn built_and_wait(mut self) -> Result<(), String> {
        let gate = self
            .gate
            .take()
            .expect("ticket gate is present until consumed");
        gate.report_built();
        gate.wait_decision()
    }

    /// Aborts the gate because this instance failed to build.
    pub(crate) fn fail(mut self, error: &str) {
        if let Some(gate) = self.gate.take() {
            gate.abort(format!("{} failed to build: {error}", self.label));
        }
    }
}

impl Drop for StartGateTicket {
    fn drop(&mut self) {
        if let Some(gate) = self.gate.take() {
            gate.abort(format!("{} stopped before it was built", self.label));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // Scenario: two tickets on one gate; the first reports built and waits
    // while the second has not built yet.
    // Guarantees: no ticket is released until every expected instance is built
    // and the gate is opened, and all waiting tickets are then released.
    #[test]
    fn gate_releases_only_after_all_built_and_open() {
        let gate = StartGate::new();
        let first = gate.ticket("a".to_owned());
        let second = gate.ticket("b".to_owned());
        let waiter = thread::spawn(move || first.built_and_wait());
        let deadline = Instant::now() + Duration::from_millis(100);
        assert!(gate.wait_until_built(2, deadline, || None).is_err());
        assert!(!waiter.is_finished());

        let second_waiter = thread::spawn(move || second.built_and_wait());
        gate.wait_until_built(2, Instant::now() + Duration::from_secs(5), || None)
            .expect("both tickets should report built");
        assert!(!waiter.is_finished());
        gate.open();
        assert_eq!(waiter.join().expect("waiter"), Ok(()));
        assert_eq!(second_waiter.join().expect("waiter"), Ok(()));
    }

    // Scenario: one instance fails to build while another is already built and
    // waiting at the gate.
    // Guarantees: the failure aborts the gate with the build error, and the
    // built instance is released with that abort reason instead of running.
    #[test]
    fn build_failure_aborts_waiting_instances() {
        let gate = StartGate::new();
        let built = gate.ticket("a".to_owned());
        let failing = gate.ticket("b".to_owned());
        let waiter = thread::spawn(move || built.built_and_wait());
        failing.fail("boom");
        let err = gate
            .wait_until_built(2, Instant::now() + Duration::from_secs(5), || None)
            .expect_err("gate should be aborted");
        assert_eq!(err, "b failed to build: boom");
        assert_eq!(waiter.join().expect("waiter"), Err(err));
    }

    // Scenario: every expected instance has built, but an interruption such as
    // an engine shutdown is already pending when the reconcile checks the gate.
    // Guarantees: the interruption wins, so built instances are not released to
    // run during shutdown.
    #[test]
    fn interruption_wins_over_all_built() {
        let gate = StartGate::new();
        let ticket = gate.ticket("a".to_owned());
        let waiter = thread::spawn(move || ticket.built_and_wait());
        gate.wait_until_built(1, Instant::now() + Duration::from_secs(5), || None)
            .expect("ticket should report built");
        let err = gate
            .wait_until_built(1, Instant::now() + Duration::from_secs(5), || {
                Some("shutdown".to_owned())
            })
            .expect_err("a pending interruption must not report the gate as built");
        assert_eq!(err, "shutdown");
        gate.abort(err.clone());
        assert_eq!(waiter.join().expect("waiter"), Err(err));
    }

    // Scenario: a ticket is dropped without reporting built, as when its
    // thread panics during build or is never spawned.
    // Guarantees: the gate is aborted so no other instance waits forever.
    #[test]
    fn dropped_ticket_aborts_gate() {
        let gate = StartGate::new();
        drop(gate.ticket("a".to_owned()));
        assert_eq!(
            gate.wait_decision(),
            Err("a stopped before it was built".to_owned())
        );
    }
}

#[cfg(test)]
mod production_topic_tests {
    use super::super::tests::{ObservedStateRunner, test_runtime_with_factory};
    use super::*;
    use async_trait::async_trait;
    use bytes::Bytes;
    use linkme::distributed_slice;
    use otel_arrow_dfe_config::engine::OtelDataflowSpec;
    use otel_arrow_dfe_core_nodes::receivers::topic_receiver::TOPIC_RECEIVER;
    use otel_arrow_dfe_engine::control::NodeControlMsg;
    use otel_arrow_dfe_engine::exporter::ExporterWrapper;
    use otel_arrow_dfe_engine::local::{exporter, receiver};
    use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
    use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
    use otel_arrow_dfe_engine::terminal_state::TerminalState;
    use otel_arrow_dfe_engine::wiring_contract::WiringContract;
    use otel_arrow_dfe_engine::{ExporterFactory, ReceiverFactory};
    use otel_arrow_dfe_otap::pdata::OtapPdata;
    use otel_arrow_dfe_otap::{
        OTAP_EXPORTER_FACTORIES, OTAP_PIPELINE_FACTORY, OTAP_RECEIVER_FACTORIES,
    };
    use otel_arrow_dfe_pdata::OtlpProtoBytes;
    use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
    use prost::Message as _;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, OnceLock, mpsc};
    use std::thread;

    const MESSAGE_COUNT: u64 = 16;
    const GUARD: Duration = Duration::from_secs(30);

    enum Event {
        ConsumerBuildHeld,
        Received(Bytes),
    }

    // Only this test's factories access the fixture. The channels coordinate otherwise
    // independent pipeline threads, and their capacity exceeds the fixed test workload.
    struct Fixture {
        events: mpsc::SyncSender<Event>,
        release_consumer: Mutex<mpsc::Receiver<()>>,
        subscribed: AtomicBool,
        producer_started: AtomicBool,
    }

    static FIXTURE: OnceLock<Fixture> = OnceLock::new();

    fn fixture() -> &'static Fixture {
        FIXTURE.get().expect("startup fixture should be installed")
    }

    fn numbered_pdata(id: u64) -> OtapPdata {
        let mut logs = otel_arrow_dfe_pdata::testing::fixtures::log_with_no_scope();
        logs.resource_logs[0].scope_logs[0].log_records[0].time_unix_nano = id;
        OtapPdata::new_todo_context(
            OtlpProtoBytes::ExportLogsRequest(Bytes::from(logs.encode_to_vec())).into(),
        )
    }

    struct NumberedReceiver;

    #[async_trait(?Send)]
    impl receiver::Receiver<OtapPdata> for NumberedReceiver {
        async fn start(
            self: Box<Self>,
            mut control: receiver::ControlChannel<OtapPdata>,
            effects: receiver::EffectHandler<OtapPdata>,
        ) -> Result<TerminalState, otel_arrow_dfe_engine::error::Error> {
            fixture().producer_started.store(true, Ordering::SeqCst);
            assert!(
                fixture().subscribed.load(Ordering::SeqCst),
                "producer ran before the production topic receiver subscribed"
            );
            for id in 1..=MESSAGE_COUNT {
                effects
                    .send_message(numbered_pdata(id))
                    .await
                    .expect("numbered message should reach the production topic exporter");
            }
            loop {
                match control.recv().await? {
                    NodeControlMsg::DrainIngress { deadline, .. } => {
                        effects.notify_receiver_drained().await?;
                        return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                    }
                    NodeControlMsg::Shutdown { deadline, .. } => {
                        return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                    }
                    _ => {}
                }
            }
        }
    }

    struct CaptureExporter;

    #[async_trait(?Send)]
    impl exporter::Exporter<OtapPdata> for CaptureExporter {
        async fn start(
            self: Box<Self>,
            mut inbox: ExporterInbox<OtapPdata>,
            _effects: exporter::EffectHandler<OtapPdata>,
        ) -> Result<TerminalState, otel_arrow_dfe_engine::error::Error> {
            loop {
                match inbox.recv().await? {
                    Message::PData(data) => fixture()
                        .events
                        .send(Event::Received(
                            data.payload_ref()
                                .encoded_bytes()
                                .expect("topic hop should preserve encoded payload")
                                .clone(),
                        ))
                        .expect("test should keep receiving captured messages"),
                    Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                        return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                    }
                    _ => {}
                }
            }
        }
    }

    #[allow(unsafe_code)]
    #[distributed_slice(OTAP_RECEIVER_FACTORIES)]
    static NUMBERED_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
        name: "urn:test:receiver:startup-numbered",
        create: |_, node, config, receiver_config, _| {
            Ok(ReceiverWrapper::local(
                NumberedReceiver,
                node,
                config,
                receiver_config,
            ))
        },
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: otel_arrow_dfe_config::validation::no_config,
    };

    #[allow(unsafe_code)]
    #[distributed_slice(OTAP_RECEIVER_FACTORIES)]
    static DELAYED_TOPIC_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
        name: "urn:test:receiver:topic",
        create: |pipeline, node, config, receiver_config, capabilities| {
            fixture()
                .events
                .send(Event::ConsumerBuildHeld)
                .expect("test should observe the held subscriber build");
            fixture()
                .release_consumer
                .lock()
                .expect("consumer release mutex")
                .recv_timeout(GUARD)
                .expect("test should release the subscriber build");
            let receiver =
                (TOPIC_RECEIVER.create)(pipeline, node, config, receiver_config, capabilities)?;
            fixture().subscribed.store(true, Ordering::SeqCst);
            Ok(receiver)
        },
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: TOPIC_RECEIVER.validate_config,
    };

    #[allow(unsafe_code)]
    #[distributed_slice(OTAP_EXPORTER_FACTORIES)]
    static CAPTURE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
        name: "urn:test:exporter:startup-capture",
        create: |_, node, config, exporter_config, _| {
            Ok(ExporterWrapper::local(
                CaptureExporter,
                node,
                config,
                exporter_config,
            ))
        },
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: otel_arrow_dfe_config::validation::no_config,
    };

    /// Scenario: a producer reaches the controller's start gate before another pipeline builds
    /// its production topic receiver, then emits numbered messages immediately upon running.
    /// Guarantees: ingress starts only after subscription, and the production topic nodes deliver
    /// every numbered payload exactly once, with no missing, duplicate, or changed messages.
    #[test]
    fn startup_subscribes_before_publishing_every_numbered_message() {
        let (events_tx, events_rx) = mpsc::sync_channel(MESSAGE_COUNT as usize + 2);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        assert!(
            FIXTURE
                .set(Fixture {
                    events: events_tx,
                    release_consumer: Mutex::new(release_rx),
                    subscribed: AtomicBool::new(false),
                    producer_started: AtomicBool::new(false),
                })
                .is_ok(),
            "only this test should install its fixture"
        );
        let config = OtelDataflowSpec::from_yaml(
            r#"
version: otel_dataflow/v1
topics:
  startup:
    policies:
      balanced:
        queue_capacity: 64
groups:
  test:
    pipelines:
      producer:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          source:
            type: urn:test:receiver:startup-numbered
          topic:
            type: urn:otel:exporter:topic
            config:
              topic: startup
        connections:
          - from: source
            to: topic
      subscriber:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          topic:
            type: urn:test:receiver:topic
            config:
              topic: startup
              subscription:
                mode: balanced
                group: capture
          capture:
            type: urn:test:exporter:startup-capture
        connections:
          - from: topic
            to: capture
"#,
        )
        .expect("startup topic config should parse");
        let runtime = test_runtime_with_factory(&config, &OTAP_PIPELINE_FACTORY);
        let _observed = ObservedStateRunner::start(&runtime);
        let gate = StartGate::new();
        let gate_guard = StartGateAbortOnDrop(Arc::clone(&gate));
        let mut workers = Vec::new();
        let mut assigned_cores = Vec::new();
        let mut launch = |pipeline_id: &str| -> Result<(), String> {
            let mut plan = runtime
                .prepare_rollout_plan(
                    "test",
                    pipeline_id,
                    &super::super::ReconfigureRequest {
                        pipeline: config.groups["test"].pipelines[pipeline_id].clone(),
                        step_timeout_secs: GUARD.as_secs(),
                        drain_timeout_secs: 5,
                    },
                )
                .map_err(|error| error.to_string())?;
            assigned_cores.push(plan.target_assigned_cores.clone());
            plan.start_gate = Some(Arc::clone(&gate));
            runtime
                .insert_rollout_plan(&plan)
                .map_err(|error| error.to_string())?;
            let runtime = Arc::clone(&runtime);
            workers.push(thread::spawn(move || runtime.run_rollout(plan)));
            Ok(())
        };

        // Defer assertions until after releasing the held build and joining the workers,
        // including on failure. Real pipeline threads require real-time failure guards.
        let result = (|| -> Result<Vec<Bytes>, String> {
            launch("producer")?;
            gate.wait_until_built(1, Instant::now() + GUARD, || None)?;
            if !matches!(gate.lock().decision, GateDecision::Closed)
                || fixture().producer_started.load(Ordering::SeqCst)
            {
                return Err("producer was released before the subscriber was built".into());
            }
            launch("subscriber")?;
            if !matches!(
                events_rx.recv_timeout(GUARD).map_err(|e| e.to_string())?,
                Event::ConsumerBuildHeld
            ) {
                return Err("message arrived before subscription".into());
            }
            release_tx.send(()).map_err(|e| e.to_string())?;
            gate.wait_until_built(2, Instant::now() + GUARD, || None)?;
            gate.open();
            let mut received = Vec::new();
            for _ in 0..MESSAGE_COUNT {
                match events_rx.recv_timeout(GUARD).map_err(|e| e.to_string())? {
                    Event::Received(bytes) => received.push(bytes),
                    _ => return Err("unexpected repeated factory construction".into()),
                }
            }
            Ok(received)
        })();
        drop(gate_guard);
        let _ = release_tx.try_send(());
        let shutdown = runtime.request_shutdown_all(5);
        for worker in workers {
            worker.join().expect("rollout worker should join");
        }
        shutdown.expect("controller should accept shutdown");
        assert!(runtime.wait_until_all_instances_exit_for(GUARD));
        let mut received = result.expect("every startup message should arrive");
        assert_ne!(assigned_cores[0], assigned_cores[1]);
        for event in events_rx.try_iter() {
            if let Event::Received(bytes) = event {
                received.push(bytes);
            }
        }
        let mut expected: Vec<_> = (1..=MESSAGE_COUNT)
            .map(|id| {
                numbered_pdata(id)
                    .payload_ref()
                    .encoded_bytes()
                    .expect("numbered fixture should be encoded")
                    .clone()
            })
            .collect();
        received.sort();
        expected.sort();
        assert_eq!(received, expected, "numbered payload multiset must match");
    }
}
