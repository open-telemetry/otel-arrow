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

#[derive(Clone)]
pub(super) struct Probe {
    eof: flume::Sender<()>,
    saw_eof: Arc<AtomicBool>,
    commands: flume::Sender<tokio::sync::mpsc::Sender<super::runtime_worker::Command>>,
    applied: flume::Sender<u64>,
    downstream: flume::Sender<Received>,
    admission: Option<AdmissionGate>,
    drain: flume::Sender<()>,
}
impl Probe {
    pub(super) fn before_admission(&self) {
        if let Some(gate) = &self.admission {
            let (release, wait) = flume::bounded(1);
            gate.try_send(release).expect("publish observed candidate");
            wait.recv_timeout(LIMIT).expect("allow admission");
        }
    }
    pub(super) fn draining(&self) {
        self.drain
            .try_send(())
            .expect("signal read worker draining");
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
    drain_timeout: &'static str,
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
        Self {
            temp,
            path,
            root,
            drain_timeout: "10s",
        }
    }

    fn config(&self) -> serde_json::Value {
        serde_json::json!({"drain_timeout":self.drain_timeout,"include":[self.path],"start_at":"beginning","discovery":{"reconcile_interval":"100ms"},"batch":{"max_records":1},"checkpoint":{"id":"test-file"}})
    }
    fn append(&self, bytes: &[u8]) {
        std::fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .expect("open source")
            .write_all(bytes)
            .expect("append log");
    }
    fn saved_offset(&self) -> u64 {
        let settings = Config::from_json(self.config())
            .expect("config")
            .compile("group", "main", "logs", self.temp.path())
            .expect("compiled config");
        let store = CheckpointStore::open(
            &StateRoot::from_borrowed_fd(self.root.as_fd()).expect("root"),
            settings.inputs().checkpoint_id(),
            settings.inputs().store_config().expect("store config"),
        )
        .expect("reopen after shutdown");
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
    drain: flume::Receiver<()>,
    commands: tokio::sync::mpsc::Sender<super::runtime_worker::Command>,
    received: flume::Receiver<Received>,
    applied: flume::Receiver<u64>,
}
impl Pipeline {
    fn start(fixture: &Fixture, admission: Option<AdmissionGate>) -> Self {
        let config = fixture.config();
        let (drain, drain_rx) = flume::bounded(1);
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
                .with_startup_callback(move || ready.try_send(()).expect("pipeline startup"));
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
        started.recv_timeout(LIMIT).expect("receiver Ready");
        let commands = endpoints.recv_timeout(LIMIT).expect("read worker");
        Self {
            stop,
            thread,
            finished,
            eof: eof_rx,
            drain: drain_rx,
            commands,
            received,
            applied: offset_rx,
        }
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
    pipeline
        .drain
        .recv_timeout(LIMIT)
        .expect("worker stopped intake");
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
    pipeline
        .drain
        .recv_timeout(LIMIT)
        .expect("worker stopped new reads");
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
    fixture.drain_timeout = "100ms";
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
