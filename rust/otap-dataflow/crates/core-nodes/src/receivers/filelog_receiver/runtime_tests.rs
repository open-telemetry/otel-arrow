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
    os::fd::AsFd,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::oneshot;

const LIMIT: Duration = Duration::from_secs(60);
#[derive(Clone)]
pub(super) struct Probe {
    eof: flume::Sender<()>,
    saw_eof: Arc<AtomicBool>,
    commands: flume::Sender<tokio::sync::mpsc::Sender<super::runtime_worker::Command>>,
    applied: flume::Sender<u64>,
    downstream: flume::Sender<(Vec<u64>, oneshot::Sender<bool>)>,
}
impl Probe {
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
                    assert_eq!(body, &Some(Value::StringValue("hello".into())));
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
                        .try_send((route, reply))
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

/// Scenario: a stable file reaches temporary EOF mid-line and the destination Nacks then Acks.
/// Guarantees: one frame is delivered twice through df_engine and only the Ack persists byte six.
#[test]
fn single_file_engine_return_path() {
    let parent = std::env::var_os("OTAP_STATE_DIR_TEST_PARENT")
        .or_else(|| std::env::var_os("HOME"))
        .expect("trusted state directory parent");
    let temp = tempfile::tempdir_in(parent).unwrap();
    let path = temp.path().join("app.log");
    std::fs::write(&path, b"hel").unwrap();
    let root = StateDirectory::provision(&temp.path().join("state")).unwrap();
    let config = serde_json::json!({"include":[path],"start_at":"beginning","batch":{"max_records":1},"checkpoint":{"id":"test-file"}});
    let settings = Config::from_json(config.clone())
        .unwrap()
        .compile("group", "main", "logs", temp.path())
        .unwrap();
    let (eof, eof_rx) = flume::bounded(1);
    let (commands, endpoints) = flume::bounded(1);
    let (downstream, received) = flume::bounded(2);
    let (applied, offset_rx) = flume::bounded(1);
    let probe = Probe {
        eof,
        saw_eof: Arc::default(),
        commands,
        applied,
        downstream,
    };
    let policy = ChannelCapacityPolicy::default();
    let (ctrl, ctrl_rx) = runtime_ctrl_msg_channel(policy.control.pipeline);
    let stop = ctrl.clone();
    let state = root.clone();
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
            .unwrap();
        let factory = otel_arrow_dfe_engine::PipelineFactory::new(
            &OTAP_RECEIVER_FACTORIES,
            &[],
            std::slice::from_ref(&CONTROLLED),
            &[],
        );
        install_test_context_bindings(&mut ctx, &factory, pipeline.clone()).unwrap();
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
            .unwrap()
            .with_startup_callback(move || ready.try_send(()).expect("pipeline startup"));
        let _guard = set_pipeline_entity_key(ctx.metrics_registry(), entity);
        let store = ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
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
            .unwrap();
    });
    if let Err(e) = eof_rx.recv_timeout(LIMIT) {
        panic!(
            "temporary EOF: {e}; pipeline result: {:?}",
            finished.recv_timeout(LIMIT)
        );
    }
    started.recv_timeout(LIMIT).expect("receiver Ready");
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(b"lo\n")
        .unwrap();
    let commands = endpoints
        .recv_timeout(LIMIT)
        .expect("worker command channel");
    let snapshot = || {
        let (tx, rx) = flume::bounded(1);
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap()
            .block_on(async {
                tokio::time::timeout(
                    LIMIT,
                    commands.send(super::runtime_worker::Command::Inspect(tx)),
                )
                .await
                .unwrap()
                .unwrap();
            });
        rx.recv_timeout(LIMIT)
            .expect("worker checkpoint and frame count")
    };
    let (first, nack) = received.recv_timeout(LIMIT).expect("initial delivery");
    assert_eq!(snapshot(), (0, 1));
    nack.send(false).unwrap();
    let (retry, ack) = received.recv_timeout(LIMIT).expect("retry delivery");
    assert_eq!(snapshot(), (0, 1));
    assert_eq!(retry, [first[0], first[1] + 1]);
    ack.send(true).unwrap();
    assert_eq!(offset_rx.recv_timeout(LIMIT).expect("Ack applied"), 6);
    stop.try_send(RuntimeControlMsg::Shutdown {
        deadline: Instant::now() + LIMIT,
        reason: "test complete".into(),
    })
    .unwrap();
    drop(stop);
    finished.recv_timeout(LIMIT).expect("shutdown").unwrap();
    thread.join().unwrap();
    let store = CheckpointStore::open(
        &StateRoot::from_borrowed_fd(root.as_fd()).unwrap(),
        settings.inputs().checkpoint_id(),
        settings.inputs().store_config().unwrap(),
    )
    .unwrap();
    let state = store.state().unwrap();
    assert_eq!(state.records().len(), 1);
    assert_eq!(state.records().next().unwrap().committed_offset, 6);
}
