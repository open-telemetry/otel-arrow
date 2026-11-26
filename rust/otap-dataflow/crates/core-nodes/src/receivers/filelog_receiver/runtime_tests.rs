// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::config::Config;
use async_trait::async_trait;
use otel_arrow_dfe_config::{
    DeployedPipelineKey,
    observed_state::{ObservedStateSettings, SendPolicy},
    pipeline::{PipelineConfigBuilder, PipelineType},
    policy::ChannelCapacityPolicy,
};
use otel_arrow_dfe_engine::{
    ConsumerEffectHandlerExtension, ExporterFactory,
    context::ControllerContext,
    control::{
        AckMsg, NackMsg, NodeControlMsg, RuntimeControlMsg, pipeline_completion_msg_channel,
        runtime_ctrl_msg_channel,
    },
    entity_context::set_pipeline_entity_key,
    error::Error,
    exporter::ExporterWrapper,
    local::exporter::{EffectHandler, Exporter},
    memory_limiter::MemoryPressureChanged,
    message::{ExporterInbox, Message},
    state_dir::StateDirectory,
    terminal_state::TerminalState,
    testing::install_test_context_bindings,
    wiring_contract::WiringContract,
};
use otel_arrow_dfe_filelog_checkpoint::store::{CheckpointStore, StateRoot};
use otel_arrow_dfe_otap::{OTAP_RECEIVER_FACTORIES, pdata::OtapPdata};
use otel_arrow_dfe_pdata::{
    otlp::{ProtoBuffer, ProtoBytesEncoder, logs::LogsProtoBytesEncoder},
    proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest, common::v1::any_value::Value,
    },
};
use otel_arrow_dfe_state::store::ObservedStateStore;
use otel_arrow_dfe_telemetry::InternalTelemetrySystem;
use prost::Message as _;
use std::{
    cell::RefCell,
    io::Write,
    os::{fd::AsFd, unix::fs::PermissionsExt},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::oneshot;

const LIMIT: Duration = Duration::from_secs(60);
type AdmissionGate = flume::Sender<flume::Sender<()>>;
struct Received {
    body: String,
    route: Vec<u64>,
    reply: oneshot::Sender<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DrainEvent {
    Running,
    Queued,
    IntakeStopped,
    RetryInterrupted {
        retry_at: Instant,
        checked_at: Instant,
    },
}

#[derive(Clone)]
pub(super) struct Probe {
    eof: flume::Sender<()>,
    saw_eof: Arc<AtomicBool>,
    commands: flume::Sender<tokio::sync::mpsc::Sender<super::runtime_worker::Command>>,
    applied: flume::Sender<u64>,
    downstream: flume::Sender<Received>,
    admission: Option<AdmissionGate>,
    drain: tokio::sync::watch::Sender<DrainEvent>,
}
impl Probe {
    pub(super) fn before_admission(&self) {
        if let Some(gate) = &self.admission {
            let (release, wait) = flume::bounded(1);
            gate.try_send(release).expect("publish observed candidate");
            wait.recv_timeout(LIMIT).expect("allow admission");
        }
    }
    pub(super) fn drain_event(&self, event: DrainEvent) {
        // Queued and stopped can be reported by different threads. Preserve the latest phase
        // and the newest worker timestamp when a retry is inspected repeatedly.
        let _ = self.drain.send_if_modified(|prior| {
            if event > *prior {
                *prior = event;
                true
            } else {
                false
            }
        });
    }
    pub(super) fn eof(&self) {
        if !self.saw_eof.swap(true, Ordering::Relaxed) {
            self.eof.try_send(()).expect("first temporary EOF");
        }
    }
    pub(super) fn register(
        &self,
        commands: tokio::sync::mpsc::Sender<super::runtime_worker::Command>,
    ) {
        self.commands
            .try_send(commands)
            .expect("register read worker");
    }
    pub(super) fn applied(&self, offset: u64) {
        self.applied
            .try_send(offset)
            .expect("publish committed offset");
    }
}
thread_local! {static PROBE: RefCell<Option<Probe>> = const {RefCell::new(None)};}
pub(super) fn probe() -> Option<Probe> {
    PROBE.with_borrow(Clone::clone)
}

struct Controlled(Probe);
static CONTROLLED: ExporterFactory<OtapPdata> = ExporterFactory {
    name: "urn:otel:exporter:filelog_test",
    create: |_, id, user, cfg, _| {
        Ok(ExporterWrapper::local(
            Controlled(probe().expect("test driver installed")),
            id,
            user,
            cfg,
        ))
    },
    context_declarations: None,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: |_| Ok(()),
};
#[async_trait(?Send)]
impl Exporter<OtapPdata> for Controlled {
    async fn start(
        self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effects: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match inbox.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(mut data) => {
                    assert_eq!(data.num_items(), 1);
                    let mut records = data
                        .payload_ref()
                        .otap_ref()
                        .expect("Filelog publishes OTAP")
                        .clone();
                    let mut bytes = ProtoBuffer::default();
                    LogsProtoBytesEncoder::new()
                        .encode(&mut records, &mut bytes)
                        .expect("encode OTAP logs");
                    let logs =
                        ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode OTLP logs");
                    let body = &logs.resource_logs[0].scope_logs[0].log_records[0]
                        .body
                        .as_ref()
                        .expect("record has a body")
                        .value;
                    let Some(Value::StringValue(body)) = body else {
                        panic!("expected a text log body")
                    };
                    let route = data
                        .source_route()
                        .expect("subscription has a return route")
                        .calldata
                        .into_iter()
                        .map(u64::from)
                        .collect();
                    let (reply, rx) = oneshot::channel();
                    self.0
                        .downstream
                        .try_send(Received {
                            body: body.clone(),
                            route,
                            reply,
                        })
                        .expect("deliver batch to test driver");
                    if tokio::time::timeout(LIMIT, rx)
                        .await
                        .expect("destination reply deadline")
                        .expect("destination reply channel")
                    {
                        effects.notify_ack(AckMsg::new(data)).await?;
                    } else {
                        effects.notify_nack(NackMsg::new("retry", data)).await?;
                    }
                }
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}

struct Fixture {
    temp: tempfile::TempDir,
    path: std::path::PathBuf,
    root: StateDirectory,
    config: serde_json::Value,
}
impl Fixture {
    fn new(initial: &[u8]) -> Self {
        let parent = std::env::var_os("OTAP_STATE_DIR_TEST_PARENT")
            .or_else(|| std::env::var_os("HOME"))
            .expect("trusted state directory parent");
        let temp = tempfile::Builder::new()
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in(parent)
            .expect("private fixture directory");
        let path = temp.path().join("app.log");
        std::fs::write(&path, initial).expect("write initial log");
        let root = StateDirectory::provision(&temp.path().join("state")).expect("provision state");
        let config = serde_json::json!({
            "include": [path],
            "start_at": "beginning",
            "discovery": {"reconcile_interval": "100ms"},
            "batch": {"max_records": 1},
            "checkpoint": {"id": "test-file"},
        });
        Self {
            temp,
            path,
            root,
            config,
        }
    }

    fn append(&self, bytes: &[u8]) {
        std::fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .expect("open source")
            .write_all(bytes)
            .expect("append log");
    }
    fn open_store(&self) -> CheckpointStore {
        let settings = Config::from_json(self.config.clone())
            .expect("config")
            .compile("group", "main", "logs", self.temp.path())
            .expect("compiled config");
        CheckpointStore::open(
            &StateRoot::from_borrowed_fd(self.root.as_fd()).expect("root"),
            settings.inputs().checkpoint_id(),
            settings.inputs().store_config().expect("store config"),
        )
        .expect("open checkpoint store")
    }
    fn saved_offset(&self) -> u64 {
        let store = self.open_store();
        let state = store.state().expect("recovered state");
        assert_eq!(state.records().len(), 1);
        state
            .records()
            .next()
            .expect("file registration")
            .committed_offset
    }
}

struct Pipeline {
    stop: otel_arrow_dfe_engine::control::RuntimeCtrlMsgSender<OtapPdata>,
    thread: std::thread::JoinHandle<()>,
    finished: flume::Receiver<Result<(), String>>,
    eof: flume::Receiver<()>,
    drain: tokio::sync::watch::Receiver<DrainEvent>,
    runtime: tokio::runtime::Handle,
    commands: tokio::sync::mpsc::Sender<super::runtime_worker::Command>,
    received: flume::Receiver<Received>,
    applied: flume::Receiver<u64>,
}
impl Pipeline {
    fn start(fixture: &Fixture, admission: Option<AdmissionGate>) -> Self {
        let config = fixture.config.clone();
        let (drain, drain_rx) = tokio::sync::watch::channel(DrainEvent::Running);
        let (eof, eof_rx) = flume::bounded(1);
        let (commands, endpoints) = flume::bounded(1);
        let (downstream, received) = flume::bounded(2);
        let (applied, offset_rx) = flume::bounded(1);
        let probe = Probe {
            eof,
            saw_eof: Arc::default(),
            commands,
            applied,
            admission,
            drain,
            downstream,
        };
        let policy = ChannelCapacityPolicy::default();
        let (ctrl, ctrl_rx) = runtime_ctrl_msg_channel(policy.control.pipeline);
        let stop = ctrl.clone();
        let state = fixture.root.clone();
        let (ready, started) = flume::bounded(1);
        let (done, finished) = flume::bounded(1);
        let thread = std::thread::spawn(move || {
            PROBE.set(Some(probe));
            let telemetry = InternalTelemetrySystem::default();
            let registry = telemetry.registry();
            let controller = ControllerContext::new(registry.clone()).with_state_directory(state);
            let mut ctx = controller.pipeline_context_with("group".into(), "main".into(), 0, 1, 0);
            let pipeline = PipelineConfigBuilder::new()
                .add_receiver("logs", "urn:otel:receiver:filelog", Some(config))
                .add_exporter("sink", CONTROLLED.name, None)
                .one_of("logs", ["sink"])
                .build(PipelineType::Otap, "group", "main")
                .expect("pipeline config");
            let factory = otel_arrow_dfe_engine::PipelineFactory::new(
                &OTAP_RECEIVER_FACTORIES,
                &[],
                std::slice::from_ref(&CONTROLLED),
                &[],
            );
            install_test_context_bindings(&mut ctx, &factory, pipeline.clone())
                .expect("test bindings");
            let entity = ctx.register_pipeline_entity();
            let runtime = factory
                .build(
                    ctx.clone(),
                    pipeline,
                    policy.clone(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    None,
                    None,
                )
                .expect("build runtime")
                .with_startup_callback(move || {
                    ready
                        .try_send(tokio::runtime::Handle::current())
                        .expect("pipeline startup")
                });
            let _guard = set_pipeline_entity_key(ctx.metrics_registry(), entity);
            let store =
                ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
            let (_pressure, pressure_rx) =
                tokio::sync::watch::channel(MemoryPressureChanged::initial());
            let (tx, rx) = pipeline_completion_msg_channel(policy.control.completion);
            let key = DeployedPipelineKey {
                pipeline_group_id: "group".into(),
                pipeline_id: "main".into(),
                core_id: 0,
                deployment_generation: 0,
            };
            let (_metrics, reporter) =
                otel_arrow_dfe_telemetry::reporter::MetricsReporter::create_new_and_receiver(64);
            let result = runtime.run_forever(
                key,
                ctx,
                store.reporter(SendPolicy::default()),
                reporter,
                Duration::from_secs(1),
                pressure_rx,
                ctrl,
                ctrl_rx,
                tx,
                rx,
            );
            done.send(result.map(drop).map_err(|e| e.to_string()))
                .expect("send pipeline result");
        });
        let runtime = started.recv_timeout(LIMIT).expect("receiver Ready");
        let commands = endpoints.recv_timeout(LIMIT).expect("read worker");
        Self {
            stop,
            thread,
            finished,
            eof: eof_rx,
            drain: drain_rx,
            runtime,
            commands,
            received,
            applied: offset_rx,
        }
    }
    fn wait_for_drain_event(&self, predicate: impl FnMut(&DrainEvent) -> bool) -> DrainEvent {
        let mut events = self.drain.clone();
        futures::executor::block_on(async {
            tokio::select! {
                state = events.wait_for(predicate) => *state.expect("worker drain event"),
                _ = futures_timer::Delay::new(LIMIT) => panic!("worker drain event deadline"),
            }
        })
    }
    fn retry_observation(&self, after: Option<Instant>) -> (Instant, Instant) {
        let event = self.wait_for_drain_event(|event| match event {
            DrainEvent::RetryInterrupted { checked_at, .. } => {
                after.is_none_or(|at| *checked_at >= at)
            }
            _ => false,
        });
        let DrainEvent::RetryInterrupted {
            retry_at,
            checked_at,
        } = event
        else {
            unreachable!("predicate selects interrupted retries")
        };
        (retry_at, checked_at)
    }

    fn expect_drain_event(&self, expected: DrainEvent) {
        let _ = self.wait_for_drain_event(|event| *event >= expected);
    }
    fn shutdown(&self) {
        self.stop
            .try_send(RuntimeControlMsg::Shutdown {
                deadline: Instant::now() + LIMIT,
                reason: "test complete".into(),
            })
            .expect("request drain");
    }
    fn join(self) {
        self.join_result().expect("clean pipeline shutdown");
    }
    fn join_result(self) -> Result<(), String> {
        drop(self.stop);
        let result = self
            .finished
            .recv_timeout(LIMIT)
            .expect("pipeline shutdown deadline");
        self.thread.join().expect("pipeline thread");
        result
    }
    fn next(&self, expected: &str) -> Received {
        let data = self
            .received
            .recv_timeout(LIMIT)
            .expect("receive OTAP batch");
        assert_eq!(data.body, expected);
        data
    }
    fn snapshot(&self) -> (u64, usize) {
        let (tx, rx) = flume::bounded(1);
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("snapshot runtime")
            .block_on(async {
                tokio::time::timeout(
                    LIMIT,
                    self.commands
                        .send(super::runtime_worker::Command::Inspect(tx)),
                )
                .await
                .expect("snapshot deadline")
                .expect("send snapshot request");
            });
        rx.recv_timeout(LIMIT)
            .expect("worker checkpoint and frames")
    }
}

/// Scenario: a stable file reaches temporary EOF mid-line and the destination Nacks then Acks.
/// Guarantees: one frame is delivered twice through df_engine and only the Ack persists byte six.
#[test]
fn single_file_engine_return_path() {
    let fixture = Fixture::new(b"hel");
    let pipeline = Pipeline::start(&fixture, None);
    pipeline.eof.recv_timeout(LIMIT).expect("temporary EOF");
    fixture.append(b"lo\n");
    let first = pipeline.next("hello");
    assert_eq!(pipeline.snapshot(), (0, 1));
    first.reply.send(false).unwrap();
    let retry = pipeline.next("hello");
    assert_eq!(pipeline.snapshot(), (0, 1));
    assert_eq!(retry.route, [first.route[0], first.route[1] + 1]);
    retry.reply.send(true).unwrap();
    assert_eq!(pipeline.applied.recv_timeout(LIMIT).unwrap(), 6);
    pipeline.shutdown();
    pipeline.join();
    assert_eq!(fixture.saved_offset(), 6);
}

/// Scenario: shutdown reaches the worker while the downstream node holds an accepted batch.
/// Guarantees: drain commits its Ack, and a new pipeline on the same file emits only subsequent bytes.
#[test]
fn held_ack_during_drain_and_resume() {
    let fixture = Fixture::new(b"hello\n");
    let pipeline = Pipeline::start(&fixture, None);
    let held = pipeline.next("hello");
    pipeline.shutdown();
    pipeline.expect_drain_event(DrainEvent::IntakeStopped);
    fixture.append(b"next\n");
    assert_eq!(pipeline.snapshot(), (0, 1));
    held.reply.send(true).unwrap();
    pipeline.join();
    assert_eq!(fixture.saved_offset(), 6);

    let restarted = Pipeline::start(&fixture, None);
    restarted.next("next").reply.send(true).unwrap();
    assert_eq!(restarted.applied.recv_timeout(LIMIT).unwrap(), 11);
    restarted.shutdown();
    restarted.join();
    assert_eq!(fixture.saved_offset(), 11);
}

/// Scenario: the stable file shrinks between the first verified observation and admission.
/// Guarantees: the receiver obtains fresh evidence and delivers the current bytes in the same pipeline.
#[test]
fn deferred_candidate_is_rediscovered() {
    let fixture = Fixture::new(b"older contents\n");
    let (gate, observed) = flume::bounded(1);
    let pipeline = Pipeline::start(&fixture, Some(gate));
    let stale = observed.recv_timeout(LIMIT).expect("original observation");
    std::fs::write(&fixture.path, b"hello\n").unwrap();
    stale.send(()).unwrap();
    observed
        .recv_timeout(LIMIT)
        .expect("fresh observation after deferral")
        .send(())
        .unwrap();
    pipeline.next("hello").reply.send(true).unwrap();
    assert_eq!(pipeline.applied.recv_timeout(LIMIT).unwrap(), 6);
    pipeline.shutdown();
    pipeline.join();
    assert_eq!(fixture.saved_offset(), 6);
}

/// Scenario: two lines are read together before the first batch is held during drain.
/// Guarantees: drain publishes the buffered second frame and commits both replies without reading later appends.
#[test]
fn drain_processes_buffered_frames() {
    let fixture = Fixture::new(b"one\ntwo\n");
    let pipeline = Pipeline::start(&fixture, None);
    let first = pipeline.next("one");
    pipeline.shutdown();
    pipeline.expect_drain_event(DrainEvent::IntakeStopped);
    fixture.append(b"three\n");
    first.reply.send(true).unwrap();
    assert_eq!(pipeline.applied.recv_timeout(LIMIT).unwrap(), 4);
    pipeline.next("two").reply.send(true).unwrap();
    pipeline.join();
    assert_eq!(fixture.saved_offset(), 8);
}

/// Scenario: the destination holds its reply through the local drain deadline.
/// Guarantees: shutdown reports a delivery timeout and recovery finds no progress for the held batch.
#[test]
fn drain_deadline_leaves_held_batch_uncommitted() {
    let mut fixture = Fixture::new(b"hello\n");
    fixture.config["drain_timeout"] = "100ms".into();
    let pipeline = Pipeline::start(&fixture, None);
    let held = pipeline.next("hello");
    pipeline.shutdown();
    let error = pipeline
        .join_result()
        .expect_err("unresolved delivery must time out");
    assert!(
        error.contains("filelog delivery drain timed out"),
        "{error}"
    );
    assert!(held.reply.is_closed());
    assert_eq!(fixture.saved_offset(), 0);
}

/// Scenario: the destination Nacks an accepted batch after the worker stops intake for drain.
/// Guarantees: the worker interrupts retry backoff, reports a drain timeout, and preserves offset zero for replay.
#[test]
fn nack_during_drain_times_out_without_committing() {
    let mut fixture = Fixture::new(b"hello\n");
    fixture.config["drain_timeout"] = "100ms".into();
    fixture.config["retry"] = serde_json::json!({"initial_backoff": "10ms"});
    let pipeline = Pipeline::start(&fixture, None);
    let held = pipeline.next("hello");

    // A blocking task keeps Tokio's paused clock from auto-advancing while
    // the real worker and engine exchange the drain command and Nack.
    let runtime = pipeline.runtime.clone();
    let driver = runtime.spawn_blocking(move || {
        tokio::time::pause();
        pipeline.shutdown();
        pipeline.expect_drain_event(DrainEvent::IntakeStopped);
        held.reply.send(false).expect("send Nack through exporter");
        let (retry_at, _) = pipeline.retry_observation(None);

        // Delivery uses a native monotonic clock. Wait for its actual deadline,
        // then wake the parked worker for another turn before drain can expire.
        assert!(retry_at < Instant::now() + LIMIT);
        assert!(matches!(
            pipeline.received.recv_deadline(retry_at),
            Err(flume::RecvTimeoutError::Timeout)
        ));
        assert_eq!(pipeline.snapshot(), (0, 1));
        // Wake the worker again without consuming the previous probe value.
        assert_eq!(pipeline.snapshot(), (0, 1));
        let (pending, checked_at) = pipeline.retry_observation(Some(retry_at));
        assert_eq!(pending, retry_at);
        assert!(checked_at >= retry_at);
        tokio::time::resume();
        pipeline
    });

    let pipeline = futures::executor::block_on(driver).expect("drain driver");
    let received = pipeline.received.clone();
    let error = pipeline
        .join_result()
        .expect_err("Nacked batch requires replay");
    assert!(
        error.contains("filelog delivery drain timed out"),
        "{error}"
    );
    assert!(
        received.is_empty(),
        "destination received another attempt during drain"
    );
    assert_eq!(fixture.saved_offset(), 0);
}

/// Scenario: a Filelog receiver publishes directly to a topic with propagation disabled or automatic.
/// Guarantees: the pipeline build rejects untracked admission before startup, while automatic propagation and unrelated untracked routes remain valid.
#[cfg(feature = "topic")]
#[test]
fn direct_topic_route_requires_ack_propagation() {
    use otel_arrow_dfe_config::topic::TopicAckPropagationMode::{Auto, Disabled};
    use otel_arrow_dfe_engine::topic::{
        PipelineTopicBinding, SubscriptionGroupName, SubscriptionMode, TopicBroker, TopicName,
        TopicOptions, TopicSet,
    };
    use otel_arrow_dfe_otap::OTAP_PIPELINE_FACTORY;

    let fixture = Fixture::new(b"hello\n");
    for (filelog, mode) in [(true, Disabled), (true, Auto), (false, Disabled)] {
        let broker = TopicBroker::<OtapPdata>::new();
        let topics = TopicSet::new("test");
        for name in ["incoming", "outgoing"] {
            let topic = TopicName::parse(name).unwrap();
            let handle = broker
                .create_in_memory_topic(topic.clone(), TopicOptions::BalancedOnly { capacity: 1 })
                .unwrap();
            let binding = PipelineTopicBinding::new(handle).with_default_ack_propagation_mode(mode);
            let _ = topics.insert(topic, binding);
        }

        let _subscriber = topics
            .get_required("outgoing")
            .unwrap()
            .subscribe(
                SubscriptionMode::Balanced {
                    group: SubscriptionGroupName::parse("destination").unwrap(),
                },
                Default::default(),
            )
            .unwrap();
        let controller = ControllerContext::new(InternalTelemetrySystem::default().registry())
            .with_state_directory(fixture.root.clone());
        let mut ctx = controller.pipeline_context_with("group".into(), "main".into(), 0, 1, 0);
        ctx.set_topic_set(topics);
        let (source_type, source_config) = if filelog {
            ("urn:otel:receiver:filelog", fixture.config.clone())
        } else {
            (
                "urn:otel:receiver:topic",
                serde_json::json!({"topic":"incoming","subscription":{"mode":"balanced","group":"readers"}}),
            )
        };
        let config = PipelineConfigBuilder::new()
            .add_receiver("source", source_type, Some(source_config))
            .add_exporter(
                "sink",
                "urn:otel:exporter:topic",
                Some(serde_json::json!({"topic":"outgoing"})),
            )
            .one_of("source", ["sink"])
            .build(PipelineType::Otap, "group", "main")
            .unwrap();
        install_test_context_bindings(&mut ctx, &OTAP_PIPELINE_FACTORY, config.clone()).unwrap();
        let result = OTAP_PIPELINE_FACTORY.build(
            ctx,
            config,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            None,
            None,
        );
        if filelog && mode == Disabled {
            let Err(error) = result else {
                panic!("untracked Filelog route accepted")
            };
            let message = error.to_string();
            assert!(
                message.contains("policies.ack_propagation.mode")
                    && message.contains("outgoing")
                    && message.contains("sink"),
                "{message}"
            );
        } else {
            result.map(drop).expect("valid topic route");
        }
    }
}

/// Scenario: a checkpoint namespace has more active identities than one association step can inspect.
/// Guarantees: drain queued after the first incomplete step returns before the next step or candidate registration.
#[test]
fn drain_between_association_steps_prevents_admission() {
    use otel_arrow_dfe_filelog_checkpoint as cp;
    let fixture = Fixture::new(b"hello\n");
    let profile = Config::from_json(fixture.config.clone())
        .unwrap()
        .compile("group", "main", "logs", fixture.temp.path())
        .unwrap()
        .inputs()
        .profile()
        .digest()
        .unwrap();
    let count = 130;
    let operations = (1..=count)
        .map(|i| {
            cp::Operation::RegisterFile(cp::RegisterFile {
                file_id: cp::FileId::from_bytes((i as u128).to_be_bytes()),
                file_epoch: 1,
                committed_offset: 0,
                committed_frontier_guard: cp::CommittedFrontierGuard::empty(),
                fingerprint: vec![b'x'; 16],
                ignored_header_bytes: 0,
                locator: cp::Locator::PosixDevIno {
                    dev: 0,
                    ino: i as u64,
                },
                framing_profile_version: cp::FRAMING_PROFILE_VERSION,
                framing_profile_digest: profile,
                framing_resume: cp::FramingResume::Clean,
                last_seen_time_unix_nano: 1,
                advisory_path: cp::AdvisoryPath::from_unix_bytes(
                    format!("/retained/{i}").as_bytes(),
                )
                .unwrap(),
            })
        })
        .collect();
    let _ = fixture
        .open_store()
        .append(operations, cp::store::Durability::Sync)
        .unwrap();
    let (gate, steps) = flume::bounded(1);
    let pipeline = Pipeline::start(&fixture, Some(gate));
    steps
        .recv_timeout(LIMIT)
        .expect("first candidate")
        .send(())
        .unwrap();
    let resume = steps
        .recv_timeout(LIMIT)
        .expect("first incomplete association step");
    pipeline.shutdown();
    pipeline.expect_drain_event(DrainEvent::Queued);
    resume.send(()).unwrap();
    pipeline.join();
    use std::os::unix::fs::MetadataExt;
    let metadata = fixture.path.metadata().unwrap();
    let locator = cp::Locator::PosixDevIno {
        dev: metadata.dev(),
        ino: metadata.ino(),
    };
    let store = fixture.open_store();
    let state = store.state().unwrap();
    assert_eq!(state.records().len(), count);
    assert!(state.get_by_locator(&locator).is_none());
}
