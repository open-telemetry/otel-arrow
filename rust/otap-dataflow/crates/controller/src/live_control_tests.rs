// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otap_df_config::engine::ResolvedPipelineRole;
use otap_df_config::observed_state::ObservedStateSettings;
use otap_df_config::settings::telemetry::logs::LogLevel;
use otap_df_engine::ExporterFactory;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::{ExporterConfig, ReceiverConfig};
use otap_df_engine::control::{
    RuntimeControlMsg, RuntimeCtrlMsgReceiver, runtime_ctrl_msg_channel,
};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::listener_group::ListenerProtocol;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::topology::NumaTopology;
use otap_df_engine::wiring_contract::WiringContract;
use otap_df_state::pipeline_status::PipelineStatus;
use otap_df_telemetry::TracingSetup;
use otap_df_telemetry::event::EngineEvent;
use otap_df_telemetry::tracing_init::ProviderSetup;
use tokio_util::sync::CancellationToken;

fn available_core_ids() -> Vec<CoreId> {
    vec![
        CoreId { id: 0 },
        CoreId { id: 1 },
        CoreId { id: 2 },
        CoreId { id: 3 },
        CoreId { id: 4 },
        CoreId { id: 5 },
        CoreId { id: 6 },
        CoreId { id: 7 },
    ]
}

fn test_validate_config(_config: &serde_json::Value) -> Result<(), otap_df_config::error::Error> {
    Ok(())
}

fn test_receiver_create(
    _pipeline_ctx: PipelineContext,
    _node: otap_df_engine::node::NodeId,
    _node_config: Arc<NodeUserConfig>,
    _receiver_config: &ReceiverConfig,
    _capabilities: &otap_df_engine::capability::registry::Capabilities,
) -> Result<ReceiverWrapper<()>, otap_df_config::error::Error> {
    panic!("test receiver factory should not be constructed")
}

fn test_exporter_create(
    _pipeline_ctx: PipelineContext,
    _node: otap_df_engine::node::NodeId,
    _node_config: Arc<NodeUserConfig>,
    _exporter_config: &ExporterConfig,
    _capabilities: &otap_df_engine::capability::registry::Capabilities,
) -> Result<ExporterWrapper<()>, otap_df_config::error::Error> {
    panic!("test exporter factory should not be constructed")
}

static TEST_RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[
    ReceiverFactory {
        name: "urn:test:receiver:example",
        create: test_receiver_create,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ReceiverFactory {
        name: "urn:otel:receiver:topic",
        create: test_receiver_create,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ReceiverFactory {
        name: "urn:otel:receiver:otlp",
        create: test_receiver_create,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static TEST_EXPORTER_FACTORIES: &[ExporterFactory<()>] = &[
    ExporterFactory {
        name: "urn:test:exporter:example",
        create: test_exporter_create,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ExporterFactory {
        name: "urn:otel:exporter:topic",
        create: test_exporter_create,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];

static TEST_PIPELINE_FACTORY: PipelineFactory<()> =
    PipelineFactory::new(TEST_RECEIVER_FACTORIES, &[], TEST_EXPORTER_FACTORIES, &[]);

fn test_runtime(config: &OtelDataflowSpec) -> Arc<ControllerRuntime<()>> {
    test_runtime_with_topology(config, NumaTopology::unknown())
}

fn test_runtime_with_topology(
    config: &OtelDataflowSpec,
    topology: NumaTopology,
) -> Arc<ControllerRuntime<()>> {
    let registry = TelemetryRegistryHandle::new();
    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
    let observed_state_handle = observed_state_store.handle();
    let engine_event_reporter = observed_state_store.reporter(Default::default());
    let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(8);
    let declared_topics =
        Controller::<()>::declare_topics(config).expect("declared topics should be valid");
    let (memory_pressure_tx, _memory_pressure_rx) =
        tokio::sync::watch::channel(MemoryPressureChanged::initial());

    Arc::new(ControllerRuntime::new(
        &TEST_PIPELINE_FACTORY,
        ControllerContext::new(registry),
        observed_state_store,
        observed_state_handle,
        engine_event_reporter,
        metrics_reporter,
        declared_topics,
        available_core_ids(),
        topology,
        TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
        Duration::from_secs(1),
        memory_pressure_tx,
        config.clone(),
    ))
}

struct ObservedStateRunner {
    cancel: CancellationToken,
    join: Option<thread::JoinHandle<()>>,
}

impl ObservedStateRunner {
    fn start(runtime: &ControllerRuntime<()>) -> Self {
        let cancel = CancellationToken::new();
        let store = runtime.observed_state_store.clone();
        let cancel_clone = cancel.clone();
        let join = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("observed-state test runtime should build");
            runtime
                .block_on(store.run(cancel_clone))
                .expect("observed-state consumer should exit cleanly");
        });
        Self {
            cancel,
            join: Some(join),
        }
    }
}

impl Drop for ObservedStateRunner {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(join) = self.join.take() {
            join.join()
                .expect("observed-state consumer thread should join cleanly");
        }
    }
}

fn deployed_key(
    pipeline_group_id: &str,
    pipeline_id: &str,
    core_id: usize,
    generation: u64,
) -> DeployedPipelineKey {
    DeployedPipelineKey {
        pipeline_group_id: pipeline_group_id.to_owned().into(),
        pipeline_id: pipeline_id.to_owned().into(),
        core_id,
        deployment_generation: generation,
    }
}

fn report_ready(runtime: &ControllerRuntime<()>, key: DeployedPipelineKey) {
    runtime
        .engine_event_reporter
        .report(EngineEvent::admitted(key.clone(), None));
    runtime
        .engine_event_reporter
        .report(EngineEvent::ready(key, None));
}

fn report_stopped(runtime: &ControllerRuntime<()>, key: DeployedPipelineKey) {
    runtime
        .engine_event_reporter
        .report(EngineEvent::admitted(key.clone(), None));
    runtime
        .engine_event_reporter
        .report(EngineEvent::ready(key.clone(), None));
    runtime
        .engine_event_reporter
        .report(EngineEvent::shutdown_requested(key.clone(), None));
    runtime
        .engine_event_reporter
        .report(EngineEvent::drained(key, None));
}

fn wait_for_observed_status<F>(
    runtime: &ControllerRuntime<()>,
    pipeline_key: &PipelineKey,
    predicate: F,
) -> PipelineStatus
where
    F: Fn(&PipelineStatus) -> bool,
{
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = runtime.observed_state_handle.pipeline_status(pipeline_key) {
            if predicate(&status) {
                return status;
            }
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for observed status predicate on {}:{}",
            pipeline_key.pipeline_group_id(),
            pipeline_key.pipeline_id()
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn engine_config_with_pipeline(pipeline_yaml: &str) -> OtelDataflowSpec {
    OtelDataflowSpec::from_yaml(&format!(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
{pipeline_yaml}
"#
    ))
    .expect("engine config should parse")
}

fn empty_engine_config() -> OtelDataflowSpec {
    OtelDataflowSpec::from_yaml("version: otel_dataflow/v1\n")
        .expect("empty engine config should parse")
}

fn reconcile_request(
    config: OtelDataflowSpec,
    delete_missing: bool,
) -> EngineConfigReconcileRequest {
    EngineConfigReconcileRequest {
        config,
        step_timeout_secs: 5,
        drain_timeout_secs: 5,
        delete_timeout_secs: 5,
        delete_missing,
    }
}

fn simple_pipeline_yaml() -> &'static str {
    r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#
}

fn register_existing_pipeline(runtime: &ControllerRuntime<()>, config: &OtelDataflowSpec) {
    register_pipeline(runtime, config, "g1", "p1");
}

fn register_pipeline(
    runtime: &ControllerRuntime<()>,
    config: &OtelDataflowSpec,
    group_id: &str,
    pipeline_id: &str,
) {
    let resolved = config
        .resolve()
        .pipelines
        .into_iter()
        .find(|pipeline| {
            pipeline.role == ResolvedPipelineRole::Regular
                && pipeline.pipeline_group_id.as_ref() == group_id
                && pipeline.pipeline_id.as_ref() == pipeline_id
        })
        .expect("resolved pipeline should exist");
    let placement = runtime
        .pipeline_placement_for_resolved(&resolved)
        .expect("resolved pipeline placement should exist");
    runtime.register_committed_pipeline(resolved, placement, 0);
}

fn register_runtime_instance(
    runtime: &ControllerRuntime<()>,
    pipeline_group_id: &str,
    pipeline_id: &str,
    core_id: usize,
    generation: u64,
    lifecycle: RuntimeInstanceLifecycle,
) -> RuntimeCtrlMsgReceiver<()> {
    let (tx, rx) = runtime_ctrl_msg_channel::<()>(4);
    let control_sender: Arc<dyn PipelineAdminSender> = Arc::new(tx.clone());
    let is_active = matches!(&lifecycle, RuntimeInstanceLifecycle::Active);
    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    _ = state.runtime_instances.insert(
        DeployedPipelineKey {
            pipeline_group_id: pipeline_group_id.to_owned().into(),
            pipeline_id: pipeline_id.to_owned().into(),
            core_id,
            deployment_generation: generation,
        },
        RuntimeInstanceRecord {
            control_sender: Some(control_sender),
            lifecycle,
        },
    );
    if is_active {
        state.active_instances += 1;
    }
    rx
}

fn register_runtime_instance_with_sender(
    runtime: &ControllerRuntime<()>,
    pipeline_key: DeployedPipelineKey,
    control_sender: Arc<dyn PipelineAdminSender>,
    lifecycle: RuntimeInstanceLifecycle,
) {
    let is_active = matches!(&lifecycle, RuntimeInstanceLifecycle::Active);
    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    _ = state.runtime_instances.insert(
        pipeline_key,
        RuntimeInstanceRecord {
            control_sender: Some(control_sender),
            lifecycle,
        },
    );
    if is_active {
        state.active_instances += 1;
    }
}

struct RecordingPipelineAdminSender {
    calls: Arc<Mutex<Vec<String>>>,
    failure: Option<String>,
}

impl PipelineAdminSender for RecordingPipelineAdminSender {
    fn try_send_shutdown(&self, _deadline: Instant, reason: String) -> Result<(), EngineError> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(reason);
        if let Some(failure) = &self.failure {
            Err(EngineError::RuntimeMsgError {
                error: failure.clone(),
            })
        } else {
            Ok(())
        }
    }
}

fn recording_admin_sender(
    failure: Option<&str>,
) -> (Arc<dyn PipelineAdminSender>, Arc<Mutex<Vec<String>>>) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let sender = Arc::new(RecordingPipelineAdminSender {
        calls: Arc::clone(&calls),
        failure: failure.map(ToOwned::to_owned),
    });
    (sender, calls)
}

struct NotifyingPipelineAdminSender {
    notification: std::sync::mpsc::Sender<String>,
}

impl PipelineAdminSender for NotifyingPipelineAdminSender {
    fn try_send_shutdown(&self, _deadline: Instant, reason: String) -> Result<(), EngineError> {
        self.notification
            .send(reason)
            .map_err(|error| EngineError::RuntimeMsgError {
                error: error.to_string(),
            })
    }
}

fn notifying_admin_sender() -> (
    Arc<dyn PipelineAdminSender>,
    std::sync::mpsc::Receiver<String>,
) {
    let (notification, receiver) = std::sync::mpsc::channel();
    (
        Arc::new(NotifyingPipelineAdminSender { notification }),
        receiver,
    )
}

struct DeadlineNotifyingPipelineAdminSender {
    notification: std::sync::mpsc::Sender<(String, Instant)>,
}

impl PipelineAdminSender for DeadlineNotifyingPipelineAdminSender {
    fn try_send_shutdown(&self, deadline: Instant, reason: String) -> Result<(), EngineError> {
        self.notification
            .send((reason, deadline))
            .map_err(|error| EngineError::RuntimeMsgError {
                error: error.to_string(),
            })
    }
}

fn deadline_notifying_admin_sender() -> (
    Arc<dyn PipelineAdminSender>,
    std::sync::mpsc::Receiver<(String, Instant)>,
) {
    let (notification, receiver) = std::sync::mpsc::channel();
    (
        Arc::new(DeadlineNotifyingPipelineAdminSender { notification }),
        receiver,
    )
}

fn launched_runtime_instance(
    pipeline_group_id: &str,
    pipeline_id: &str,
    core_id: usize,
    generation: u64,
) -> LaunchedPipelineThread<()> {
    let (tx, _rx) = runtime_ctrl_msg_channel::<()>(4);
    let control_sender: Arc<dyn PipelineAdminSender> = Arc::new(tx);
    LaunchedPipelineThread {
        pipeline_key: DeployedPipelineKey {
            pipeline_group_id: pipeline_group_id.to_owned().into(),
            pipeline_id: pipeline_id.to_owned().into(),
            core_id,
            deployment_generation: generation,
        },
        control_sender,
        _marker: std::marker::PhantomData,
    }
}

fn wait_for_shutdown_state(
    runtime: &ControllerRuntime<()>,
    shutdown_id: &str,
    expected_state: &str,
) -> ShutdownStatus {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status = runtime
            .shutdown_status_snapshot(shutdown_id)
            .expect("shutdown should exist");
        if status.state == expected_state {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for shutdown {shutdown_id} to reach state {expected_state}, current state: {}",
            status.state
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_shutdown_message(receiver: &mut RuntimeCtrlMsgReceiver<()>) -> RuntimeControlMsg<()> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(message) = receiver.try_recv() {
            return message;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for shutdown control message"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn complete_instance_exit_on_shutdown(
    runtime: Arc<ControllerRuntime<()>>,
    mut receiver: RuntimeCtrlMsgReceiver<()>,
    deployed_key: DeployedPipelineKey,
    expected_reason: &'static str,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        assert!(matches!(
            wait_for_shutdown_message(&mut receiver),
            RuntimeControlMsg::Shutdown { reason, .. } if reason == expected_reason
        ));
        runtime.note_instance_exit(deployed_key, RuntimeInstanceExit::Success);
    })
}

fn terminal_rollout_record(
    pipeline_group_id: &str,
    pipeline_id: &str,
    rollout_id: &str,
) -> RolloutRecord {
    let mut rollout = RolloutRecord::new(
        rollout_id.to_owned(),
        pipeline_group_id.to_owned().into(),
        pipeline_id.to_owned().into(),
        RolloutAction::Replace,
        1,
        Some(0),
        60,
        CoreAllocationStrategy::CoreCount,
        PipelinePlacement {
            pipeline_group_id: pipeline_group_id.to_owned().into(),
            pipeline_id: pipeline_id.to_owned().into(),
            cores: Vec::new(),
        },
        Vec::new(),
    );
    rollout.state = RolloutLifecycleState::Succeeded;
    rollout
}

fn terminal_shutdown_record(
    pipeline_group_id: &str,
    pipeline_id: &str,
    shutdown_id: &str,
) -> ShutdownRecord {
    let mut shutdown = ShutdownRecord::new(
        shutdown_id.to_owned(),
        pipeline_group_id.to_owned().into(),
        pipeline_id.to_owned().into(),
        Vec::new(),
    );
    shutdown.state = ShutdownLifecycleState::Succeeded;
    shutdown
}

/// Scenario: a reconfigure request changes only the effective core
/// allocation from one assigned core to two.
/// Guarantees: rollout planning classifies the change as a resize, starts
/// only the added core, and keeps the current generation unchanged.
#[test]
fn prepare_rollout_plan_accepts_core_allocation_scale_up() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("core allocation changes should be planned");

    assert_eq!(plan.action, RolloutAction::Resize);
    assert_eq!(plan.current_assigned_cores, vec![0]);
    assert_eq!(plan.target_assigned_cores, vec![0, 1]);
    assert_eq!(plan.common_assigned_cores, vec![0]);
    assert_eq!(plan.added_assigned_cores, vec![1]);
    assert!(plan.removed_assigned_cores.is_empty());
    assert_eq!(plan.resize_start_cores, vec![1]);
    assert!(plan.resize_stop_cores.is_empty());
    assert_eq!(plan.target_generation, 0);
    assert_eq!(
        plan.rollout
            .cores
            .iter()
            .map(|core| core.core_id)
            .collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn prepare_rollout_plan_reserves_other_committed_pipeline_cores() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 4
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 4
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }
    for core_id in 4..=7 {
        let _receiver = register_runtime_instance(
            &runtime,
            "g1",
            "p2",
            core_id,
            0,
            RuntimeInstanceLifecycle::Active,
        );
    }

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("core allocation changes should be planned");

    assert_eq!(plan.current_assigned_cores, vec![4, 5, 6, 7]);
    assert_eq!(plan.target_assigned_cores, vec![4, 5]);
    assert_eq!(plan.resize_stop_cores, vec![6, 7]);
    assert!(plan.resize_start_cores.is_empty());
}

#[test]
fn prepare_rollout_plan_does_not_reserve_committed_all_cores_pipeline() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }
    for core_id in 0..=1 {
        let _receiver = register_runtime_instance(
            &runtime,
            "g1",
            "p2",
            core_id,
            0,
            RuntimeInstanceLifecycle::Active,
        );
    }

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 3
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("all_cores committed pipeline should not reserve every core");

    assert_eq!(plan.target_assigned_cores, vec![0, 1, 2]);
}

#[test]
fn prepare_rollout_plan_rejects_core_count_all_when_no_unreserved_cores_remain() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 0
                  end: 7
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let p2_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 0
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 create should parse");

    let err = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect_err("live planning should reject empty effective core_count placement");

    assert!(matches!(
        err,
        ControlPlaneError::InvalidRequest { message }
            if message.contains("no unreserved cores are available")
    ));
}

#[test]
fn prepare_rollout_plan_reserves_active_rollout_target_cores() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }
    for core_id in 2..=3 {
        let _receiver = register_runtime_instance(
            &runtime,
            "g1",
            "p2",
            core_id,
            0,
            RuntimeInstanceLifecycle::Active,
        );
    }

    let p2_resize = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 4
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 resize should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_resize,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 resize should be planned");

    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3, 4, 5]);
    runtime
        .insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone())
        .expect("p2 rollout should insert");

    let p3_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p3".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p3 create should parse");
    let p3_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p3",
            &ReconfigureRequest {
                pipeline: p3_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p3 create should be planned");

    assert_eq!(p3_plan.target_assigned_cores, vec![6, 7]);
}

#[test]
fn insert_rollout_rejects_stale_plan_conflicting_with_active_target_cores() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }

    let p2_resize = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 4
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 resize should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_resize,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 resize should be planned");

    let p3_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p3".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p3 create should parse");
    let p3_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p3",
            &ReconfigureRequest {
                pipeline: p3_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p3 create should be planned before p2 inserts");

    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3, 4, 5]);
    assert_eq!(p3_plan.target_assigned_cores, vec![4, 5]);

    runtime
        .insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone())
        .expect("p2 rollout should insert");
    assert!(matches!(
        runtime.insert_rollout(&p3_plan.pipeline_key, p3_plan.rollout.clone()),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_rejects_core_set_overlapping_active_core_count_target() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("engine config should parse");
    let runtime = test_runtime(&config);
    let mut resolved = config.resolve().pipelines;
    resolved.sort_by(|left, right| left.pipeline_id.as_ref().cmp(right.pipeline_id.as_ref()));
    let placement_snapshot = Controller::<()>::preflight_pipeline_placement(
        &resolved,
        &available_core_ids(),
        &NumaTopology::unknown(),
    )
    .expect("startup placement should resolve");

    for (pipeline, placement) in resolved.iter().cloned().zip(placement_snapshot.pipelines) {
        runtime.register_committed_pipeline(pipeline, placement, 0);
    }

    let p2_resize = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 4
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 resize should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_resize,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 resize should be planned");

    runtime
        .insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone())
        .expect("p2 rollout should insert");

    let p3_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p3".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_set
      set:
        - start: 4
          end: 5
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p3 create should parse");
    let p3_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p3",
            &ReconfigureRequest {
                pipeline: p3_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p3 create should be planned");

    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3, 4, 5]);
    assert_eq!(p3_plan.target_assigned_cores, vec![4, 5]);
    assert!(matches!(
        runtime.insert_rollout(&p3_plan.pipeline_key, p3_plan.rollout.clone()),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_rejects_stale_plan_conflicting_with_new_committed_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let p2_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 create should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 create should be planned");
    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3]);

    let mut config_with_p3 = config.clone();
    let group_id: PipelineGroupId = "g1".to_owned().into();
    let pipeline_id: PipelineId = "p3".to_owned().into();
    _ = config_with_p3
        .groups
        .get_mut(&group_id)
        .expect("g1 should exist")
        .pipelines
        .insert(
            pipeline_id,
            PipelineConfig::from_yaml(
                "g1".into(),
                "p3".into(),
                r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
            )
            .expect("p3 config should parse"),
        );
    let p3_resolved = config_with_p3
        .resolve()
        .pipelines
        .into_iter()
        .find(|pipeline| {
            pipeline.role == ResolvedPipelineRole::Regular
                && pipeline.pipeline_group_id.as_ref() == "g1"
                && pipeline.pipeline_id.as_ref() == "p3"
        })
        .expect("p3 should resolve");
    runtime.register_committed_pipeline(
        p3_resolved,
        PipelinePlacement {
            pipeline_group_id: "g1".to_owned().into(),
            pipeline_id: "p3".to_owned().into(),
            cores: [2, 3]
                .into_iter()
                .map(|core_id| {
                    CorePlacement::from_core_id(CoreId { id: core_id }, &NumaTopology::unknown())
                })
                .collect(),
        },
        0,
    );

    assert!(matches!(
        runtime.insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone()),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_rejects_plan_after_committed_config_revision_changes() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let resize_to_three = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 3
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("resize to three should parse");
    let stale_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: resize_to_three,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("resize to three should be planned");

    let resize_to_four = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 4
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("resize to four should parse");
    let committed_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: resize_to_four,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("resize to four should be planned");
    runtime.commit_pipeline_record(&committed_plan, committed_plan.target_generation);

    assert!(matches!(
        runtime.insert_rollout_plan(&stale_plan),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_rejects_core_set_overlapping_committed_core_count_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let p2_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_set
      set:
        - start: 1
          end: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 create should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 create should be planned");

    assert_eq!(p2_plan.target_assigned_cores, vec![1, 2]);
    assert!(matches!(
        runtime.insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone()),
        Err(ControlPlaneError::RolloutConflict)
    ));
}

#[test]
fn insert_rollout_allows_core_set_overlapping_committed_core_set_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 1
                  end: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let p2_create = PipelineConfig::from_yaml(
        "g1".into(),
        "p2".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_set
      set:
        - start: 2
          end: 3
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("p2 create should parse");
    let p2_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p2",
            &ReconfigureRequest {
                pipeline: p2_create,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("p2 create should be planned");

    assert_eq!(p2_plan.target_assigned_cores, vec![2, 3]);
    runtime
        .insert_rollout(&p2_plan.pipeline_key, p2_plan.rollout.clone())
        .expect("explicit core_set overlap should be allowed");
}

/// Scenario: live-control accepts a new pipeline with listener bind config on
/// cores mapped to a known NUMA node.
/// Guarantees: placement and listener metadata are resolved during planning,
/// before any runtime worker is launched.
#[test]
fn prepare_rollout_plan_resolves_live_numa_and_listener_metadata() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let topology = NumaTopology::from_node_cpulists(&[(0, "0-3".into()), (1, "4-7".into())]);
    let runtime = test_runtime_with_topology(&config, topology);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_set
      set:
        - start: 4
          end: 5
nodes:
  receiver:
    type: "urn:otel:receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("listener pipeline should be planned");

    assert_eq!(plan.target_assigned_cores, vec![4, 5]);
    assert_eq!(plan.target_placement.placement.core_count(), 2);
    assert_eq!(
        plan.target_placement
            .placement
            .cores
            .iter()
            .map(|core| (core.core_id.id, core.known_numa_node_id))
            .collect::<Vec<_>>(),
        vec![(4, Some(1)), (5, Some(1))]
    );

    let snapshot = &plan.target_placement.listener_group_snapshot;
    assert_eq!(snapshot.generation, 1);
    assert_eq!(snapshot.plans.len(), 1);
    let listener_plan = snapshot
        .plan_for(
            "receiver",
            "127.0.0.1:4317".parse().unwrap(),
            ListenerProtocol::Tcp,
        )
        .expect("grpc listener group should be planned");
    assert_eq!(
        listener_plan
            .expected_members
            .iter()
            .map(|member| (member.core_id, member.numa_node_id))
            .collect::<Vec<_>>(),
        vec![(4, Some(1)), (5, Some(1))]
    );
}

#[test]
fn prepare_rollout_plan_replaces_when_listener_membership_changes() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:otel:receiver:otlp"
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let resize = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 3
nodes:
  receiver:
    type: "urn:otel:receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("resize should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: resize,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("resize should be planned");

    assert_eq!(plan.action, RolloutAction::Replace);
}

/// Scenario: a reconfigure request changes only the effective core
/// allocation from two assigned cores to one.
/// Guarantees: rollout planning classifies the change as a resize, stops
/// only the removed core, and keeps the current generation unchanged.
#[test]
fn prepare_rollout_plan_accepts_core_allocation_scale_down() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver0 =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let _receiver1 =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("core allocation changes should be planned");

    assert_eq!(plan.action, RolloutAction::Resize);
    assert_eq!(plan.current_assigned_cores, vec![0, 1]);
    assert_eq!(plan.target_assigned_cores, vec![0]);
    assert_eq!(plan.common_assigned_cores, vec![0]);
    assert!(plan.added_assigned_cores.is_empty());
    assert_eq!(plan.removed_assigned_cores, vec![1]);
    assert!(plan.resize_start_cores.is_empty());
    assert_eq!(plan.resize_stop_cores, vec![1]);
    assert_eq!(plan.target_generation, 0);
    assert_eq!(plan.target_placement.listener_group_snapshot.generation, 0);
    assert_eq!(
        plan.rollout
            .cores
            .iter()
            .map(|core| core.core_id)
            .collect::<Vec<_>>(),
        vec![1]
    );
}

/// Scenario: the submitted pipeline config is effectively identical to the
/// committed active pipeline and serving footprint.
/// Guarantees: rollout planning short-circuits to `NoOp` rather than
/// scheduling a replace or resize operation.
#[test]
fn prepare_rollout_plan_returns_noop_for_identical_active_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("identical updates should be planned");

    assert_eq!(plan.action, RolloutAction::NoOp);
    assert_eq!(plan.target_generation, 0);
    assert_eq!(plan.target_placement.listener_group_snapshot.generation, 0);
    assert!(plan.rollout.cores.is_empty());
    assert!(plan.resize_start_cores.is_empty());
    assert!(plan.resize_stop_cores.is_empty());
}

#[test]
fn prepare_rollout_plan_replaces_divergent_non_monotonic_numa_placement() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 6
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let topology = NumaTopology::from_node_cpulists(&[(0, "4-7".into()), (1, "0-3".into())]);
    let runtime = test_runtime_with_topology(&config, topology);
    register_existing_pipeline(&runtime, &config);
    for core_id in [4, 5, 6, 7, 0, 1] {
        let _rx = register_runtime_instance(
            &runtime,
            "g1",
            "p1",
            core_id,
            0,
            RuntimeInstanceLifecycle::Active,
        );
    }

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 6
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("divergent non-monotonic NUMA update should be planned");

    assert_eq!(plan.target_assigned_cores, vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(plan.action, RolloutAction::Replace);
}

/// Scenario: the controller executes a rollout plan that has already been
/// classified as `NoOp`.
/// Guarantees: the controller returns an immediate successful rollout
/// snapshot, preserves the committed generation, and leaves no in-flight
/// rollout summary behind.
#[test]
fn spawn_rollout_returns_immediate_success_for_noop() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("identical updates should be planned");

    let status = runtime
        .spawn_rollout(plan)
        .expect("noop rollout should succeed");

    assert_eq!(status.action, "noop");
    assert_eq!(status.state, ApiPipelineRolloutState::Succeeded);
    assert_eq!(status.target_generation, 0);
    assert!(status.cores.is_empty());

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let details = runtime
        .pipeline_details_snapshot(&pipeline_key)
        .expect("group should exist")
        .expect("pipeline should exist");
    assert_eq!(details.active_generation, Some(0));
    assert!(details.rollout.is_none());

    let rollout = runtime
        .rollout_status_snapshot(&status.rollout_id)
        .expect("completed rollout should remain queryable");
    assert_eq!(rollout.state, ApiPipelineRolloutState::Succeeded);
}

/// Scenario: a reconfigure request changes the runtime graph shape while
/// also changing the resource footprint.
/// Guarantees: planning keeps the safer replace path instead of collapsing
/// the update into a resource-only resize.
#[test]
fn prepare_rollout_plan_keeps_replace_when_runtime_shape_changes() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _receiver =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");

    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("runtime shape changes should still be planned");

    assert_eq!(plan.action, RolloutAction::Replace);
    assert_eq!(plan.target_generation, 1);
    assert_eq!(plan.common_assigned_cores, vec![0]);
    assert_eq!(plan.added_assigned_cores, vec![1]);
    assert!(plan.resize_start_cores.is_empty());
    assert!(plan.resize_stop_cores.is_empty());
    assert_eq!(
        plan.rollout
            .cores
            .iter()
            .map(|core| core.core_id)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
}

/// Scenario: a reconfigure request would require a runtime topic-broker
/// mutation for an existing logical pipeline.
/// Guarantees: planning rejects the request before rollout starts and
/// surfaces an invalid-request error to the caller.
#[test]
fn prepare_rollout_plan_rejects_topic_runtime_mutation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          to_topic:
            type: "urn:otel:exporter:topic"
            config:
              topic: shared
        connections:
          - from: receiver
            to: to_topic
"#,
    )
    .expect("config should parse");
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  from_topic:
    type: "urn:otel:receiver:topic"
    config:
      topic: shared
      subscription:
        mode: balanced
        group: workers
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: from_topic
    to: exporter
"#,
    )
    .expect("replacement should parse");

    let err = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect_err("topic runtime changes should be rejected");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("topic broker mutation"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

/// Scenario: a second rollout is requested for a logical pipeline that
/// already has an active rollout record.
/// Guarantees: planning rejects the new request with a rollout conflict
/// instead of interleaving two rollout state machines.
#[test]
fn prepare_rollout_plan_rejects_concurrent_rollout_for_same_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement.clone(),
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("first rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let err = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect_err("second rollout should conflict");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
}

/// Scenario: a new rollout has been registered for a logical pipeline but
/// has not yet committed its candidate config.
/// Guarantees: pipeline details still return the committed config while
/// exposing the pending rollout summary separately.
#[test]
fn pipeline_details_returns_committed_config_while_rollout_is_pending() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement.clone(),
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let details = runtime
        .pipeline_details_snapshot(&PipelineKey::new("g1".into(), "p1".into()))
        .expect("group should exist")
        .expect("pipeline details should exist");

    let mut committed_nodes = details
        .pipeline
        .node_iter()
        .map(|(node_id, _)| node_id.as_ref().to_owned())
        .collect::<Vec<_>>();
    committed_nodes.sort();
    assert_eq!(
        committed_nodes,
        vec!["exporter".to_owned(), "receiver".to_owned()]
    );
    assert_eq!(details.active_generation, Some(0));
    assert_eq!(
        details
            .rollout
            .expect("pending rollout summary should be present")
            .target_generation,
        1
    );
}

/// Scenario: panic diagnostics are captured for a worker panic with explicit
/// thread metadata.
/// Guarantees: the short summary stays operator-friendly while the detailed
/// form includes thread context and a captured backtrace.
#[test]
fn panic_report_formats_summary_and_detail() {
    let report = PanicReport::capture(
        "rollout worker",
        Box::new("boom"),
        Some("rollout-g1-p1".to_owned()),
        Some(17),
        Some(3),
    );

    assert_eq!(report.summary_message(), "rollout worker panicked: boom");
    let detail = report.detail_message();
    assert!(detail.contains("rollout worker panicked: boom"));
    assert!(detail.contains("thread_name=rollout-g1-p1"));
    assert!(detail.contains("thread_id=17"));
    assert!(detail.contains("core_id=3"));
    assert!(detail.contains("backtrace:"));
}

/// Scenario: a panic is raised with a non-string payload.
/// Guarantees: the captured panic summary stays readable and avoids the older
/// generic placeholder text.
#[test]
fn panic_report_non_string_payload_has_useful_fallback() {
    let report = PanicReport::capture("shutdown worker", Box::new(7usize), None, None, None);

    assert_eq!(
        report.summary_message(),
        "shutdown worker panicked: non-string panic payload"
    );
    assert!(
        !report
            .summary_message()
            .contains("panic payload was not a string")
    );
}

/// Scenario: a detached rollout worker panics before it reaches the normal
/// terminal-state bookkeeping path.
/// Guarantees: the rollout is forced into a failed terminal state and the
/// logical pipeline no longer stays blocked by a stale active-rollout entry.
#[test]
fn rollout_worker_panic_marks_failed_and_clears_conflict() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement.clone(),
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    runtime.handle_rollout_worker_panic(
        &plan.pipeline_key,
        &plan.rollout.rollout_id,
        "rollout-g1-p1".to_owned(),
        Box::new("boom"),
    );

    let status = runtime
        .rollout_status_snapshot(&plan.rollout.rollout_id)
        .expect("rollout should remain queryable");
    assert_eq!(status.state, ApiPipelineRolloutState::Failed);
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|message| message.contains("rollout worker panicked: boom"))
    );
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|message| !message.contains("backtrace:"))
    );

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!state.active_rollouts.contains_key(&plan.pipeline_key));
    drop(state);

    let _next_plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout conflict should be cleared after panic cleanup");
}

/// Scenario: a rollout worker panics after launching an uncommitted candidate
/// generation.
/// Guarantees: panic cleanup requests shutdown for the candidate generation
/// before clearing the active rollout, avoiding active orphan instances.
#[test]
fn rollout_worker_panic_requests_shutdown_for_uncommitted_candidate_generation() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let candidate_key = deployed_key("g1", "p1", 0, plan.target_generation);
    let mut candidate_rx = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        0,
        plan.target_generation,
        RuntimeInstanceLifecycle::Active,
    );

    runtime.handle_rollout_worker_panic(
        &plan.pipeline_key,
        &plan.rollout.rollout_id,
        "rollout-g1-p1".to_owned(),
        Box::new("boom"),
    );

    assert!(matches!(
        wait_for_shutdown_message(&mut candidate_rx),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "rollout panic cleanup"
    ));

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(
        state
            .runtime_instances
            .get(&candidate_key)
            .expect("candidate instance should still be tracked until exit")
            .control_sender
            .is_none(),
        "panic cleanup should release the retained sender after shutdown dispatch"
    );
    assert!(!state.active_rollouts.contains_key(&plan.pipeline_key));
}

/// Scenario: a rollout worker panics after the target generation was already
/// committed as serving.
/// Guarantees: panic cleanup does not shut down the committed generation,
/// which would turn a late bookkeeping panic into runtime outage.
#[test]
fn rollout_worker_panic_does_not_shutdown_committed_target_generation() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");
    runtime.commit_pipeline_record(&plan, plan.target_generation);

    let mut candidate_rx = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        0,
        plan.target_generation,
        RuntimeInstanceLifecycle::Active,
    );

    runtime.handle_rollout_worker_panic(
        &plan.pipeline_key,
        &plan.rollout.rollout_id,
        "rollout-g1-p1".to_owned(),
        Box::new("boom"),
    );

    assert!(
        candidate_rx.try_recv().is_err(),
        "committed target generation must not receive panic-cleanup shutdown"
    );
}

/// Scenario: a resize rollback must clean up cores that were already started
/// before a later step fails.
/// Guarantees: rollback sends shutdown to those started cores instead of
/// leaving them running after the rollout fails.
#[test]
fn rollback_resize_rollout_cleans_up_started_cores() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _existing =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  receiver:
    type: "urn:test:receiver:example"
    config: null
  exporter:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: receiver
    to: exporter
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("resize rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let started_key = deployed_key("g1", "p1", 1, plan.target_generation);
    let started_rx = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        1,
        plan.target_generation,
        RuntimeInstanceLifecycle::Active,
    );
    let exit_thread = complete_instance_exit_on_shutdown(
        Arc::clone(&runtime),
        started_rx,
        started_key.clone(),
        "rollback cleanup",
    );

    let result = runtime.rollback_resize_rollout(&plan, &[1], &[], "boom".to_owned());

    assert!(matches!(
        result,
        Err(RolloutExecutionError::Failed(reason)) if reason == "boom"
    ));
    exit_thread
        .join()
        .expect("resize rollback shutdown helper should join cleanly");
    assert!(matches!(
        runtime.instance_exit(&started_key),
        Some(RuntimeInstanceExit::Success)
    ));
}

/// Scenario: a replace rollback must clean up added candidate cores that were
/// already serving the target generation before a later step fails.
/// Guarantees: rollback sends shutdown to those activated added cores instead
/// of leaving the candidate generation running.
#[test]
fn rollback_replace_rollout_cleans_up_activated_added_cores() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _existing =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let replacement = PipelineConfig::from_yaml(
        "g1".into(),
        "p1".into(),
        r#"
policies:
  resources:
    core_allocation:
      type: core_count
      count: 2
nodes:
  input:
    type: "urn:test:receiver:example"
    config: null
  output:
    type: "urn:test:exporter:example"
    config: null
connections:
  - from: input
    to: output
"#,
    )
    .expect("replacement should parse");
    let plan = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: replacement,
                step_timeout_secs: 60,
                drain_timeout_secs: 60,
            },
        )
        .expect("replace rollout plan should be accepted");
    runtime
        .insert_rollout(&plan.pipeline_key, plan.rollout.clone())
        .expect("rollout should register");

    let added_key = deployed_key("g1", "p1", 1, plan.target_generation);
    let added_rx = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        1,
        plan.target_generation,
        RuntimeInstanceLifecycle::Active,
    );
    let exit_thread = complete_instance_exit_on_shutdown(
        Arc::clone(&runtime),
        added_rx,
        added_key.clone(),
        "rollback cleanup",
    );

    let result = runtime.rollback_replace_rollout(&plan, &[], &[1], &[], "boom".to_owned());

    assert!(matches!(
        result,
        Err(RolloutExecutionError::Failed(reason)) if reason == "boom"
    ));
    exit_thread
        .join()
        .expect("replace rollback shutdown helper should join cleanly");
    assert!(matches!(
        runtime.instance_exit(&added_key),
        Some(RuntimeInstanceExit::Success)
    ));
}

/// Scenario: a shutdown request targets a group id that does not exist in
/// the controller's committed config.
/// Guarantees: per-pipeline shutdown fails fast with `GroupNotFound`
/// instead of creating a shutdown record.
#[test]
fn request_shutdown_pipeline_rejects_missing_group() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);

    let err = runtime
        .request_shutdown_pipeline("missing", "p1", 5)
        .expect_err("missing group should be rejected");

    assert_eq!(err, ControlPlaneError::GroupNotFound);
}

/// Scenario: a shutdown request targets a pipeline id that is not present
/// in an existing group.
/// Guarantees: per-pipeline shutdown rejects the request with
/// `PipelineNotFound` before any runtime instances are touched.
#[test]
fn request_shutdown_pipeline_rejects_missing_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);

    let err = runtime
        .request_shutdown_pipeline("g1", "missing", 5)
        .expect_err("missing pipeline should be rejected");

    assert_eq!(err, ControlPlaneError::PipelineNotFound);
}

/// Scenario: a control-plane caller creates a new empty pipeline group after
/// the controller has started with no groups.
/// Guarantees: the group is added to committed live config and can be read
/// back without registering any logical pipelines.
#[test]
fn create_group_adds_empty_group_to_live_config() {
    let config = empty_engine_config();
    let runtime = test_runtime(&config);
    let group = PipelineGroupConfig::new();

    let created = runtime
        .create_group("g1", group.clone())
        .expect("empty group should be created");

    let group_id: PipelineGroupId = "g1".to_string().into();
    assert_eq!(created, group);
    assert_eq!(runtime.group_details_snapshot(&group_id), Some(group));
    let snapshot = runtime.engine_config_snapshot();
    assert!(snapshot.groups.contains_key(&group_id));
    assert!(snapshot.groups[&group_id].pipelines.is_empty());
}

/// Scenario: a control-plane caller attempts to create a group that is already
/// present in committed live config.
/// Guarantees: the runtime rejects the request with a conflict-level error and
/// leaves the existing group unchanged.
#[test]
fn create_group_rejects_duplicate_group() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let err = runtime
        .create_group("g1", PipelineGroupConfig::new())
        .expect_err("duplicate group should be rejected");

    assert_eq!(err, ControlPlaneError::GroupAlreadyExists);
    let group_id: PipelineGroupId = "g1".to_string().into();
    assert!(runtime.group_details_snapshot(&group_id).is_some());
}

/// Scenario: a control-plane caller submits a group-create payload that already
/// contains pipelines.
/// Guarantees: PR1's endpoint boundary stays scoped to empty group creation,
/// leaving pipeline creation to the existing pipeline reconfigure endpoint.
#[test]
fn create_group_rejects_payload_with_pipelines() {
    let config = empty_engine_config();
    let runtime = test_runtime(&config);
    let pipeline = PipelineConfig::from_yaml("g1".into(), "p1".into(), simple_pipeline_yaml())
        .expect("pipeline should parse");
    let mut group = PipelineGroupConfig::new();
    _ = group.pipelines.insert("p1".to_string().into(), pipeline);

    let err = runtime
        .create_group("g1", group)
        .expect_err("non-empty group should be rejected");

    assert!(matches!(
        err,
        ControlPlaneError::InvalidRequest { ref message }
            if message == "pipeline group creation only supports empty groups"
    ));
    let group_id: PipelineGroupId = "g1".to_string().into();
    assert!(runtime.group_details_snapshot(&group_id).is_none());
}

/// Scenario: a control-plane caller deletes a stopped logical pipeline.
/// Guarantees: the pipeline is removed from committed live config and the
/// containing group remains available as an empty group.
#[test]
fn delete_pipeline_removes_stopped_pipeline_from_live_config() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let status = runtime
        .request_delete_pipeline("g1", "p1", 5)
        .expect("stopped pipeline should be deleted");

    assert_eq!(status.state, "succeeded");
    assert!(status.shutdown.is_none());
    let snapshot = runtime.engine_config_snapshot();
    let group_id: PipelineGroupId = "g1".into();
    let pipeline_id: PipelineId = "p1".into();
    assert!(snapshot.groups.contains_key(&group_id));
    assert!(
        !snapshot.groups[&group_id]
            .pipelines
            .contains_key(&pipeline_id)
    );
}

/// Scenario: a control-plane caller deletes a pipeline that cannot be found.
/// Guarantees: missing groups and missing pipelines map to distinct typed
/// errors before any live state is changed.
#[test]
fn delete_pipeline_rejects_missing_targets() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let missing_group = runtime
        .request_delete_pipeline("missing", "p1", 5)
        .expect_err("missing group should be rejected");
    assert_eq!(missing_group, ControlPlaneError::GroupNotFound);

    let missing_pipeline = runtime
        .request_delete_pipeline("g1", "missing", 5)
        .expect_err("missing pipeline should be rejected");
    assert_eq!(missing_pipeline, ControlPlaneError::PipelineNotFound);

    let snapshot = runtime.engine_config_snapshot();
    assert!(
        snapshot.groups[&PipelineGroupId::from("g1")]
            .pipelines
            .contains_key(&PipelineId::from("p1"))
    );
}

/// Scenario: a control-plane caller deletes a pipeline while a shutdown is
/// already active for the same logical pipeline.
/// Guarantees: delete rejects with the same conflict boundary as rollout and
/// shutdown planning.
#[test]
fn delete_pipeline_rejects_active_operation_conflict() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let plan = runtime
        .prepare_shutdown_plan("g1", "p1", 5)
        .expect("shutdown plan should be accepted");
    runtime
        .insert_shutdown(&plan.pipeline_key, plan.shutdown)
        .expect("shutdown should register");

    let err = runtime
        .request_delete_pipeline("g1", "p1", 5)
        .expect_err("active shutdown should block delete");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
}

/// Scenario: an engine-scoped lifecycle operation is already active.
/// Guarantees: public config mutation entry points reject instead of
/// interleaving with the active full-engine operation.
#[test]
fn engine_scoped_operation_rejects_public_lifecycle_mutations() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.active_engine_operation = Some("reconcile-42".to_owned());
    }

    let create_err = runtime
        .create_group("g2", PipelineGroupConfig::new())
        .expect_err("active engine operation should block group creation");
    assert_eq!(create_err, ControlPlaneError::RolloutConflict);

    let delete_err = runtime
        .request_delete_pipeline("g1", "p1", 5)
        .expect_err("active engine operation should block pipeline deletion");
    assert_eq!(delete_err, ControlPlaneError::RolloutConflict);

    let rollout_err = runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: config.groups[&PipelineGroupId::from("g1")].pipelines
                    [&PipelineId::from("p1")]
                    .clone(),
                step_timeout_secs: 5,
                drain_timeout_secs: 5,
            },
        )
        .expect_err("active engine operation should block rollout planning");
    assert_eq!(rollout_err, ControlPlaneError::RolloutConflict);
}

/// Scenario: a control-plane caller deletes an empty group.
/// Guarantees: the group is removed from committed live config without
/// requiring pipeline shutdown work.
#[test]
fn delete_group_removes_empty_group_from_live_config() {
    let config = empty_engine_config();
    let runtime = test_runtime(&config);
    let _created = runtime
        .create_group("g1", PipelineGroupConfig::new())
        .expect("group should be created");

    let status = runtime
        .request_delete_group("g1", 5)
        .expect("empty group should be deleted");

    assert_eq!(status.state, "succeeded");
    assert!(status.pipelines.is_empty());
    assert!(
        !runtime
            .engine_config_snapshot()
            .groups
            .contains_key(&PipelineGroupId::from("g1"))
    );
}

/// Scenario: a control-plane caller deletes a group containing stopped
/// pipelines.
/// Guarantees: the runtime deletes each pipeline in deterministic order and
/// removes the empty group from committed live config.
#[test]
fn delete_group_removes_stopped_pipelines_and_group() {
    let config = OtelDataflowSpec::from_yaml(&format!(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
{p1}
      p2:
{p2}
"#,
        p1 = simple_pipeline_yaml(),
        p2 = simple_pipeline_yaml()
    ))
    .expect("config should parse");
    let runtime = test_runtime(&config);

    let status = runtime
        .request_delete_group("g1", 5)
        .expect("group should be deleted");

    assert_eq!(status.state, "succeeded");
    assert_eq!(
        status
            .pipelines
            .iter()
            .map(|pipeline| pipeline.pipeline_id.as_ref().to_owned())
            .collect::<Vec<_>>(),
        vec!["p1".to_owned(), "p2".to_owned()]
    );
    assert!(
        !runtime
            .engine_config_snapshot()
            .groups
            .contains_key(&PipelineGroupId::from("g1"))
    );
}

/// Scenario: a full-config reconciliation request matches the current live
/// pipeline configuration and runtime assignment.
/// Guarantees: reconciliation records a no-op change and leaves committed
/// config intact.
#[test]
fn reconcile_engine_config_reports_noop_for_matching_live_config() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let status = runtime
        .reconcile_engine_config(reconcile_request(config.clone(), true))
        .expect("matching config should reconcile");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(status.changes.len(), 1);
    assert_eq!(status.changes[0].action, ConfigChangeAction::Noop);
    assert_eq!(status.changes[0].state, "succeeded");
    assert!(
        runtime
            .engine_config_snapshot()
            .groups
            .contains_key(&PipelineGroupId::from("g1"))
    );
}

/// Scenario: a full-config reconciliation request omits live stopped
/// resources with `delete_missing` enabled.
/// Guarantees: reconciliation deletes the omitted pipeline and then the
/// now-empty group from committed live config.
#[test]
fn reconcile_engine_config_deletes_missing_resources_by_default() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let status = runtime
        .reconcile_engine_config(reconcile_request(empty_engine_config(), true))
        .expect("missing resources should be deleted");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(
        status
            .changes
            .iter()
            .map(|change| (
                change.pipeline_group_id.as_ref().map(|id| id.as_ref()),
                change.pipeline_id.as_ref().map(|id| id.as_ref()),
                change.action,
                change.state.as_str(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                Some("g1"),
                Some("p1"),
                ConfigChangeAction::Delete,
                "succeeded"
            ),
            (Some("g1"), None, ConfigChangeAction::Delete, "succeeded"),
        ]
    );
    assert!(
        !runtime
            .engine_config_snapshot()
            .groups
            .contains_key(&PipelineGroupId::from("g1"))
    );
}

/// Scenario: a full-config reconciliation request omits live resources with
/// `delete_missing` disabled.
/// Guarantees: reconciliation succeeds without deleting the omitted group or
/// pipeline.
#[test]
fn reconcile_engine_config_preserves_missing_resources_when_requested() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);

    let status = runtime
        .reconcile_engine_config(reconcile_request(empty_engine_config(), false))
        .expect("missing resources should be preserved");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert!(status.changes.is_empty());
    let snapshot = runtime.engine_config_snapshot();
    assert!(
        snapshot.groups[&PipelineGroupId::from("g1")]
            .pipelines
            .contains_key(&PipelineId::from("p1"))
    );
}

/// Scenario: full-config reconciliation is rejected after validation because a
/// target pipeline already has an active rollout.
/// Guarantees: desired engine-level scaffold fields are not committed when the
/// reconcile request fails before applying all requested changes.
#[test]
fn reconcile_engine_config_does_not_publish_scaffold_on_conflict() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        _ = state
            .active_rollouts
            .insert(pipeline_key, "rollout-42".to_owned());
    }

    let mut desired = config.clone();
    _ = desired
        .engine
        .custom
        .insert("desired".to_owned(), serde_json::json!({"enabled": true}));

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("active rollout should reject full-config reconciliation");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
    assert!(runtime.engine_config_snapshot().engine.custom.is_empty());
}

/// Scenario: full-config reconciliation would change an existing topic
/// runtime profile.
/// Guarantees: reconciliation rejects the request before starting rollout or
/// mutating committed live config.
#[test]
fn reconcile_engine_config_rejects_runtime_topic_mutation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          to_topic:
            type: "urn:otel:exporter:topic"
            config:
              topic: shared
        connections:
          - from: receiver
            to: to_topic
"#,
    )
    .expect("config should parse");
    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
topics:
  shared: {}
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          from_topic:
            type: "urn:otel:receiver:topic"
            config:
              topic: shared
              subscription:
                mode: balanced
                group: workers
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: from_topic
            to: exporter
"#,
    )
    .expect("desired config should parse");
    let runtime = test_runtime(&config);

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("topic runtime changes should be rejected");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("runtime topic broker mutation"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(runtime.engine_config_snapshot(), config);
}

#[test]
fn reconcile_engine_config_rejects_core_relocation_before_mutating() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let desired = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 2
                  end: 3
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 0
                  end: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("desired config should parse");

    let err = runtime
        .reconcile_engine_config(reconcile_request(desired, true))
        .expect_err("reconcile should reject vacate-before-claim placement");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("conflicts with committed or in-flight"));
            assert!(message.contains("stage the conflicting delete or resize"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(runtime.engine_config_snapshot(), config);
}

#[test]
fn reconcile_engine_config_phases_inherited_core_allocation() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  a_count:
    policies:
      resources:
        core_allocation:
          type: core_count
          count: 4
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
  b_set:
    policies:
      resources:
        core_allocation:
          type: core_set
          set:
            - start: 0
              end: 3
    pipelines:
      p1:
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("config should parse");
    let runtime = test_runtime(&config);

    for resolved in config
        .resolve()
        .pipelines
        .into_iter()
        .filter(|pipeline| pipeline.role == ResolvedPipelineRole::Regular)
    {
        let assigned_cores = match resolved.pipeline_group_id.as_ref() {
            "a_count" => vec![4, 5, 6, 7],
            "b_set" => vec![0, 1, 2, 3],
            other => panic!("unexpected group: {other}"),
        };
        let placement = PipelinePlacement {
            pipeline_group_id: resolved.pipeline_group_id.clone(),
            pipeline_id: resolved.pipeline_id.clone(),
            cores: assigned_cores
                .iter()
                .copied()
                .map(|id| CorePlacement::from_core_id(CoreId { id }, &NumaTopology::unknown()))
                .collect(),
        };
        let group_id = resolved.pipeline_group_id.as_ref().to_owned();
        let pipeline_id = resolved.pipeline_id.as_ref().to_owned();
        runtime.register_committed_pipeline(resolved, placement, 0);
        for core_id in assigned_cores {
            let _rx = register_runtime_instance(
                &runtime,
                &group_id,
                &pipeline_id,
                core_id,
                0,
                RuntimeInstanceLifecycle::Active,
            );
        }
    }

    let status = runtime
        .reconcile_engine_config(reconcile_request(config, true))
        .expect("matching inherited placement config should reconcile");

    assert_eq!(status.state, EngineConfigReconcileState::Succeeded);
    assert_eq!(
        status
            .changes
            .iter()
            .map(|change| (
                change.pipeline_group_id.as_ref().map(|id| id.as_ref()),
                change.action,
                change.state.as_str(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (Some("b_set"), ConfigChangeAction::Noop, "succeeded"),
            (Some("a_count"), ConfigChangeAction::Noop, "succeeded"),
        ]
    );
}

/// Scenario: a detached shutdown worker panics before it reaches the normal
/// terminal-state bookkeeping path.
/// Guarantees: the shutdown is forced into a failed terminal state and the
/// logical pipeline no longer stays blocked by a stale active-shutdown entry.
#[test]
fn shutdown_worker_panic_marks_failed_and_clears_conflict() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let plan = runtime
        .prepare_shutdown_plan("g1", "p1", 5)
        .expect("shutdown plan should be accepted");
    runtime
        .insert_shutdown(&plan.pipeline_key, plan.shutdown.clone())
        .expect("shutdown should register");

    runtime.handle_shutdown_worker_panic(
        &plan.pipeline_key,
        &plan.shutdown.shutdown_id,
        "shutdown-g1-p1".to_owned(),
        Box::new("boom"),
    );

    let status = runtime
        .shutdown_status_snapshot(&plan.shutdown.shutdown_id)
        .expect("shutdown should remain queryable");
    assert_eq!(status.state, "failed");
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|message| message.contains("shutdown worker panicked: boom"))
    );
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|message| !message.contains("backtrace:"))
    );

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!state.active_shutdowns.contains_key(&plan.pipeline_key));
    drop(state);

    let _next_plan = runtime
        .prepare_shutdown_plan("g1", "p1", 5)
        .expect("shutdown conflict should be cleared after panic cleanup");
}

/// Scenario: a shutdown request arrives while the same logical pipeline is
/// already under rollout.
/// Guarantees: shutdown is rejected with a rollout conflict so the rollout
/// controller remains the single owner of that pipeline's lifecycle.
#[test]
fn request_shutdown_pipeline_rejects_active_rollout() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    _ = state
        .active_rollouts
        .insert(pipeline_key, "rollout-42".to_owned());
    drop(state);

    let err = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect_err("active rollout should conflict");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
}

/// Scenario: a second shutdown request targets a logical pipeline that
/// already has an active shutdown operation.
/// Guarantees: the controller rejects the duplicate request instead of
/// creating competing shutdown records.
#[test]
fn request_shutdown_pipeline_rejects_active_shutdown() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let shutdown = ShutdownRecord::new(
        "shutdown-0".to_owned(),
        "g1".into(),
        "p1".into(),
        vec![ShutdownCoreProgress {
            core_id: 0,
            deployment_generation: 0,
            state: "pending".to_owned(),
            updated_at: timestamp_now(),
            detail: None,
        }],
    );
    runtime
        .insert_shutdown(&pipeline_key, shutdown)
        .expect("shutdown should register");

    let err = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect_err("active shutdown should conflict");

    assert_eq!(err, ControlPlaneError::RolloutConflict);
}

/// Scenario: a shutdown request targets a committed pipeline that currently
/// has no active runtime instances.
/// Guarantees: the controller rejects the request as an invalid already
/// stopped pipeline instead of synthesizing a no-op shutdown operation.
#[test]
fn request_shutdown_pipeline_rejects_already_stopped_pipeline() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let err = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect_err("already stopped pipeline should be rejected");

    match err {
        ControlPlaneError::InvalidRequest { message } => {
            assert!(message.contains("already stopped"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

/// Scenario: a shutdown request targets one logical pipeline while other
/// pipelines and exited instances still exist in the runtime registry.
/// Guarantees: only active instances for the requested logical pipeline
/// receive shutdown control messages and relinquish their control senders.
#[test]
fn request_shutdown_pipeline_targets_only_active_instances_for_pipeline() {
    let config = OtelDataflowSpec::from_yaml(
        r#"
version: otel_dataflow/v1
groups:
  g1:
    pipelines:
      p1:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
      p2:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    )
    .expect("config should parse");
    let runtime = test_runtime(&config);
    register_pipeline(&runtime, &config, "g1", "p1");
    register_pipeline(&runtime, &config, "g1", "p2");

    let mut p1_core0 =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let mut p1_core1 =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);
    let mut p1_exited = register_runtime_instance(
        &runtime,
        "g1",
        "p1",
        2,
        0,
        RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Success),
    );
    let mut p2_core0 =
        register_runtime_instance(&runtime, "g1", "p2", 3, 0, RuntimeInstanceLifecycle::Active);

    let _shutdown = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect("shutdown request should be accepted");

    assert!(matches!(
        wait_for_shutdown_message(&mut p1_core0),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));
    assert!(matches!(
        wait_for_shutdown_message(&mut p1_core1),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));
    assert!(
        p1_exited.try_recv().is_err(),
        "exited runtime should not receive shutdown"
    );
    assert!(
        p2_core0.try_recv().is_err(),
        "other pipelines must not receive shutdown"
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let p1_core0_released = state
            .runtime_instances
            .get(&DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p1".into(),
                core_id: 0,
                deployment_generation: 0,
            })
            .and_then(|instance| instance.control_sender.as_ref())
            .is_none();
        let p1_core1_released = state
            .runtime_instances
            .get(&DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p1".into(),
                core_id: 1,
                deployment_generation: 0,
            })
            .and_then(|instance| instance.control_sender.as_ref())
            .is_none();
        let p2_core0_retained = state
            .runtime_instances
            .get(&DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p2".into(),
                core_id: 3,
                deployment_generation: 0,
            })
            .and_then(|instance| instance.control_sender.as_ref())
            .is_some();
        drop(state);

        if p1_core0_released && p1_core1_released && p2_core0_retained {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for targeted control senders to be released"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

/// Scenario: global shutdown dispatch encounters a send failure for one
/// active runtime instance while other active instances still need the signal.
/// Guarantees: shutdown dispatch is best effort across the whole snapshot:
/// every active sender is attempted, successful sends relinquish their retained
/// control sender, repeated calls do not re-signal instances that already
/// accepted shutdown, and failures are reported only after the full pass.
#[test]
fn request_shutdown_all_attempts_all_active_instances_before_returning_error() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let key0 = deployed_key("g1", "p1", 0, 0);
    let key1 = deployed_key("g1", "p1", 1, 0);
    let key2 = deployed_key("g1", "p1", 2, 0);
    let (sender0, calls0) = recording_admin_sender(None);
    let (sender1, calls1) = recording_admin_sender(Some("simulated send failure"));
    let (sender2, calls2) = recording_admin_sender(None);

    register_runtime_instance_with_sender(
        &runtime,
        key0.clone(),
        sender0,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        key1.clone(),
        sender1,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        key2.clone(),
        sender2,
        RuntimeInstanceLifecycle::Active,
    );

    let err = runtime
        .request_shutdown_all(5)
        .expect_err("shutdown-all should report the failed sender after dispatching all sends");

    assert_eq!(
        *calls0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );
    assert_eq!(
        *calls1
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );
    assert_eq!(
        *calls2
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );

    let ControlPlaneError::Internal { message } = err else {
        panic!("unexpected shutdown-all error: {err:?}");
    };
    assert!(message.contains("g1:p1 core=1 generation=0"));
    assert!(message.contains("simulated send failure"));

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(
        state
            .runtime_instances
            .get(&key0)
            .and_then(|instance| instance.control_sender.as_ref())
            .is_none(),
        "successful shutdown send should release key0 control sender"
    );
    assert!(
        state
            .runtime_instances
            .get(&key1)
            .and_then(|instance| instance.control_sender.as_ref())
            .is_some(),
        "failed shutdown send should retain key1 control sender"
    );
    assert!(
        state
            .runtime_instances
            .get(&key2)
            .and_then(|instance| instance.control_sender.as_ref())
            .is_none(),
        "successful shutdown send should release key2 control sender"
    );
    drop(state);

    // The first pass released the control sender for successful instances, so
    // a retry should only reattempt the instance whose shutdown send failed.
    let err = runtime
        .request_shutdown_all(5)
        .expect_err("shutdown-all retry should still report the failed sender");

    let ControlPlaneError::Internal { message } = err else {
        panic!("unexpected shutdown-all retry error: {err:?}");
    };
    assert!(message.contains("g1:p1 core=1 generation=0"));
    assert!(message.contains("simulated send failure"));
    assert_eq!(
        *calls0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );
    assert_eq!(
        *calls1
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned(), "global shutdown".to_owned()]
    );
    assert_eq!(
        *calls2
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec!["global shutdown".to_owned()]
    );
}

/// Scenario: global shutdown includes regular producer instances and the
/// engine's system observability instance.
/// Guarantees: all regular instances receive shutdown first, and the system
/// observability sender is not called until every regular instance reports its
/// terminal exit, preserving their final internal telemetry.
#[test]
fn request_shutdown_all_stops_observability_after_regular_instances_exit() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let regular_key0 = deployed_key("g1", "p1", 0, 0);
    let regular_key1 = deployed_key("g1", "p1", 1, 0);
    let observability_key = deployed_key(
        SYSTEM_PIPELINE_GROUP_ID,
        SYSTEM_OBSERVABILITY_PIPELINE_ID,
        2,
        0,
    );
    let (regular_sender0, regular_notifications0) = notifying_admin_sender();
    let (regular_sender1, regular_notifications1) = notifying_admin_sender();
    let (observability_sender, observability_notifications) = deadline_notifying_admin_sender();
    register_runtime_instance_with_sender(
        &runtime,
        regular_key0.clone(),
        regular_sender0,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        regular_key1.clone(),
        regular_sender1,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        observability_key.clone(),
        observability_sender,
        RuntimeInstanceLifecycle::Active,
    );

    let shutdown_runtime = Arc::clone(&runtime);
    let (shutdown_result_tx, shutdown_result_rx) = std::sync::mpsc::channel();
    let shutdown_thread = thread::spawn(move || {
        shutdown_result_tx
            .send(shutdown_runtime.request_shutdown_all(5))
            .expect("shutdown result receiver should remain open");
    });

    assert_eq!(
        regular_notifications0
            .recv_timeout(Duration::from_secs(1))
            .expect("first regular instance should receive shutdown"),
        "global shutdown"
    );
    assert_eq!(
        regular_notifications1
            .recv_timeout(Duration::from_secs(1))
            .expect("second regular instance should receive shutdown"),
        "global shutdown"
    );
    shutdown_result_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("shutdown dispatch should return before regular instances exit")
        .expect("initial shutdown dispatch should succeed");
    shutdown_thread
        .join()
        .expect("global shutdown dispatch thread should join");
    assert!(
        observability_notifications.try_recv().is_err(),
        "observability must remain active while regular instances drain"
    );

    runtime.note_instance_exit(regular_key0, RuntimeInstanceExit::Success);
    assert!(
        observability_notifications.try_recv().is_err(),
        "one remaining regular instance must keep observability active"
    );
    runtime.note_instance_exit(regular_key1, RuntimeInstanceExit::Success);

    let (reason, deadline) = observability_notifications
        .recv_timeout(Duration::from_secs(1))
        .expect("observability should receive shutdown after producers exit");
    assert_eq!(reason, "global shutdown");
    assert!(
        deadline.saturating_duration_since(Instant::now()) > Duration::from_secs(4),
        "observability should receive the caller's five-second shutdown budget"
    );
    runtime.note_instance_exit(observability_key, RuntimeInstanceExit::Success);
    assert!(
        runtime.wait_for_global_shutdown_completion(),
        "the phased shutdown coordinator should complete after observability exits"
    );
    assert!(runtime.all_instances_exited());

    runtime
        .request_shutdown_all(5)
        .expect("repeated global shutdown should remain idempotent");
    assert!(regular_notifications0.try_recv().is_err());
    assert!(regular_notifications1.try_recv().is_err());
    assert!(observability_notifications.try_recv().is_err());
}

#[test]
fn request_shutdown_all_keeps_observability_active_when_producer_times_out() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let regular_key = deployed_key("g1", "p1", 0, 0);
    let observability_key = deployed_key(
        SYSTEM_PIPELINE_GROUP_ID,
        SYSTEM_OBSERVABILITY_PIPELINE_ID,
        1,
        0,
    );
    let (regular_sender, regular_notifications) = notifying_admin_sender();
    let (observability_sender, observability_notifications) = notifying_admin_sender();
    register_runtime_instance_with_sender(
        &runtime,
        regular_key.clone(),
        regular_sender,
        RuntimeInstanceLifecycle::Active,
    );
    register_runtime_instance_with_sender(
        &runtime,
        observability_key.clone(),
        observability_sender,
        RuntimeInstanceLifecycle::Active,
    );

    runtime
        .request_shutdown_all(1)
        .expect("initial shutdown dispatch should succeed");
    let _ = regular_notifications
        .recv_timeout(Duration::from_secs(1))
        .expect("regular producer should receive shutdown");
    assert!(
        observability_notifications
            .recv_timeout(Duration::from_millis(1_200))
            .is_err(),
        "observability must remain active when a producer misses its deadline"
    );

    runtime.note_instance_exit(regular_key, RuntimeInstanceExit::Success);
    runtime
        .request_shutdown_all(1)
        .expect("a later request should retry the restored observability sender");
    let _ = observability_notifications
        .recv_timeout(Duration::from_secs(1))
        .expect("observability should stop once the producer has exited");
    runtime.note_instance_exit(observability_key, RuntimeInstanceExit::Success);
    assert!(runtime.wait_for_global_shutdown_completion());
    assert!(runtime.all_instances_exited());
}

/// Scenario: all targeted runtime instances exit cleanly after a pipeline
/// shutdown request is accepted.
/// Guarantees: the shutdown record reaches `succeeded`, tracks per-core
/// completion, and removes the active shutdown lock for that pipeline.
#[test]
fn request_shutdown_pipeline_tracks_completion() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let mut core0 =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let mut core1 =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);

    let shutdown = runtime
        .request_shutdown_pipeline("g1", "p1", 5)
        .expect("shutdown request should be accepted");
    assert_eq!(shutdown.state, "pending");

    assert!(matches!(
        wait_for_shutdown_message(&mut core0),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));
    assert!(matches!(
        wait_for_shutdown_message(&mut core1),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));

    runtime.note_instance_exit(
        DeployedPipelineKey {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            core_id: 0,
            deployment_generation: 0,
        },
        RuntimeInstanceExit::Success,
    );
    {
        let state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            state.runtime_instances.contains_key(&DeployedPipelineKey {
                pipeline_group_id: "g1".into(),
                pipeline_id: "p1".into(),
                core_id: 0,
                deployment_generation: 0,
            }),
            "active shutdown should retain exited instances until completion"
        );
    }
    runtime.note_instance_exit(
        DeployedPipelineKey {
            pipeline_group_id: "g1".into(),
            pipeline_id: "p1".into(),
            core_id: 1,
            deployment_generation: 0,
        },
        RuntimeInstanceExit::Success,
    );

    let status = wait_for_shutdown_state(&runtime, &shutdown.shutdown_id, "succeeded");
    assert_eq!(status.cores.len(), 2);
    assert!(status.cores.iter().all(|core| core.state == "exited"));

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(
        !state
            .active_shutdowns
            .contains_key(&PipelineKey::new("g1".into(), "p1".into()))
    );
    assert!(!state.runtime_instances.contains_key(&DeployedPipelineKey {
        pipeline_group_id: "g1".into(),
        pipeline_id: "p1".into(),
        core_id: 0,
        deployment_generation: 0,
    }));
    assert!(!state.runtime_instances.contains_key(&DeployedPipelineKey {
        pipeline_group_id: "g1".into(),
        pipeline_id: "p1".into(),
        core_id: 1,
        deployment_generation: 0,
    }));
}

/// Scenario: a pipeline shutdown request is accepted but the targeted
/// runtime instance never exits before the shutdown deadline.
/// Guarantees: the shutdown record transitions to `failed`, preserves the
/// timeout reason, and records the failed per-core state for callers.
#[test]
fn request_shutdown_pipeline_tracks_timeout_failure() {
    let config = engine_config_with_pipeline(
        r#"
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let mut core0 =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let shutdown = runtime
        .request_shutdown_pipeline("g1", "p1", 1)
        .expect("shutdown request should be accepted");
    assert!(matches!(
        wait_for_shutdown_message(&mut core0),
        RuntimeControlMsg::Shutdown { reason, .. } if reason == "pipeline shutdown"
    ));

    let status = wait_for_shutdown_state(&runtime, &shutdown.shutdown_id, "failed");
    assert!(
        status
            .failure_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("timed out waiting"))
    );
    assert_eq!(status.cores.len(), 1);
    assert_eq!(status.cores[0].state, "failed");
}

/// Scenario: terminal rollout history grows beyond the retention cap for one
/// logical pipeline while another pipeline also retains rollout history.
/// Guarantees: eviction is oldest-first and scoped per logical pipeline rather
/// than dropping unrelated rollout history.
#[test]
fn terminal_rollout_history_is_bounded_per_pipeline() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let other_pipeline_key = PipelineKey::new("g1".into(), "p2".into());

    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for index in 0..=TERMINAL_ROLLOUT_RETENTION_LIMIT {
        let rollout_id = format!("rollout-{index}");
        _ = state.rollouts.insert(
            rollout_id.clone(),
            terminal_rollout_record("g1", "p1", &rollout_id),
        );
        ControllerRuntime::<()>::record_terminal_rollout_locked(
            &mut state,
            &pipeline_key,
            &rollout_id,
            Instant::now(),
        );
    }

    let other_rollout_id = "rollout-other".to_owned();
    _ = state.rollouts.insert(
        other_rollout_id.clone(),
        terminal_rollout_record("g1", "p2", &other_rollout_id),
    );
    ControllerRuntime::<()>::record_terminal_rollout_locked(
        &mut state,
        &other_pipeline_key,
        &other_rollout_id,
        Instant::now(),
    );

    assert!(!state.rollouts.contains_key("rollout-0"));
    assert!(state.rollouts.contains_key("rollout-1"));
    assert!(state.rollouts.contains_key(&other_rollout_id));
    assert_eq!(
        state
            .terminal_rollouts
            .get(&pipeline_key)
            .map(|queue| queue.len()),
        Some(TERMINAL_ROLLOUT_RETENTION_LIMIT)
    );
    assert_eq!(
        state
            .terminal_rollouts
            .get(&other_pipeline_key)
            .map(|queue| queue.len()),
        Some(1)
    );
}

/// Scenario: terminal shutdown history grows beyond the retention cap for one
/// logical pipeline while another pipeline also retains shutdown history.
/// Guarantees: shutdown eviction is oldest-first and scoped per logical
/// pipeline rather than trimming unrelated shutdown history.
#[test]
fn terminal_shutdown_history_is_bounded_per_pipeline() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let other_pipeline_key = PipelineKey::new("g1".into(), "p2".into());

    let mut state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for index in 0..=TERMINAL_SHUTDOWN_RETENTION_LIMIT {
        let shutdown_id = format!("shutdown-{index}");
        _ = state.shutdowns.insert(
            shutdown_id.clone(),
            terminal_shutdown_record("g1", "p1", &shutdown_id),
        );
        ControllerRuntime::<()>::record_terminal_shutdown_locked(
            &mut state,
            &pipeline_key,
            &shutdown_id,
            Instant::now(),
        );
    }

    let other_shutdown_id = "shutdown-other".to_owned();
    _ = state.shutdowns.insert(
        other_shutdown_id.clone(),
        terminal_shutdown_record("g1", "p2", &other_shutdown_id),
    );
    ControllerRuntime::<()>::record_terminal_shutdown_locked(
        &mut state,
        &other_pipeline_key,
        &other_shutdown_id,
        Instant::now(),
    );

    assert!(!state.shutdowns.contains_key("shutdown-0"));
    assert!(state.shutdowns.contains_key("shutdown-1"));
    assert!(state.shutdowns.contains_key(&other_shutdown_id));
    assert_eq!(
        state
            .terminal_shutdowns
            .get(&pipeline_key)
            .map(|queue| queue.len()),
        Some(TERMINAL_SHUTDOWN_RETENTION_LIMIT)
    );
    assert_eq!(
        state
            .terminal_shutdowns
            .get(&other_pipeline_key)
            .map(|queue| queue.len()),
        Some(1)
    );
}

/// Scenario: terminal rollout and shutdown ids outlive their retention TTL in
/// the controller's in-memory history.
/// Guarantees: history pruning expires those terminal records and subsequent
/// by-id lookups return not found instead of growing unboundedly.
#[test]
fn terminal_operation_history_expires_after_ttl() {
    let runtime = test_runtime(&engine_config_with_pipeline(simple_pipeline_yaml()));
    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let rollout_id = "rollout-old".to_owned();
    let shutdown_id = "shutdown-old".to_owned();
    let prune_now = Instant::now()
        .checked_add(TERMINAL_OPERATION_RETENTION_TTL + Duration::from_secs(2))
        .expect("synthetic prune deadline should be representable");
    let expired_at = prune_now
        .checked_sub(TERMINAL_OPERATION_RETENTION_TTL + Duration::from_secs(1))
        .expect("synthetic completed_at should be representable");

    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let mut rollout = terminal_rollout_record("g1", "p1", &rollout_id);
        rollout.completed_at = Some(expired_at);
        _ = state.rollouts.insert(rollout_id.clone(), rollout);
        state
            .terminal_rollouts
            .entry(pipeline_key.clone())
            .or_default()
            .push_back(rollout_id.clone());

        let mut shutdown = terminal_shutdown_record("g1", "p1", &shutdown_id);
        shutdown.completed_at = Some(expired_at);
        _ = state.shutdowns.insert(shutdown_id.clone(), shutdown);
        state
            .terminal_shutdowns
            .entry(pipeline_key.clone())
            .or_default()
            .push_back(shutdown_id.clone());

        // Use a synthetic future `now` here instead of relying on
        // `Instant::now() - ttl`, which can underflow on Windows near the
        // monotonic clock origin.
        ControllerRuntime::<()>::prune_terminal_operation_history_locked(&mut state, prune_now);
    }

    assert!(runtime.rollout_status_snapshot(&rollout_id).is_none());
    assert!(runtime.shutdown_status_snapshot(&shutdown_id).is_none());

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!state.rollouts.contains_key(&rollout_id));
    assert!(!state.shutdowns.contains_key(&shutdown_id));
    assert!(!state.terminal_rollouts.contains_key(&pipeline_key));
    assert!(!state.terminal_shutdowns.contains_key(&pipeline_key));
}

/// Scenario: an instance exits when there is no active rollout or shutdown for
/// its logical pipeline.
/// Guarantees: the controller does not retain that exited runtime instance as
/// history once no active control-plane operation depends on it.
#[test]
fn exited_runtime_instances_without_active_operation_are_pruned_immediately() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let deployed_key = DeployedPipelineKey {
        pipeline_group_id: "g1".into(),
        pipeline_id: "p1".into(),
        core_id: 0,
        deployment_generation: 0,
    };
    runtime.note_instance_exit(deployed_key.clone(), RuntimeInstanceExit::Success);

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!state.runtime_instances.contains_key(&deployed_key));
}

/// Scenario: a runtime thread reports exit before the controller finishes
/// registering the launched instance as active.
/// Guarantees: early exit bookkeeping is reconciled during registration, so
/// active-instance tracking does not leak and the pending-exit entry is cleared.
#[test]
fn register_launched_instance_reconciles_early_exit_without_leaking_active_count() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    register_existing_pipeline(&runtime, &config);

    let deployed_key = deployed_key("g1", "p1", 0, 0);
    runtime.note_instance_exit(deployed_key.clone(), RuntimeInstanceExit::Success);

    runtime.register_launched_instance(launched_runtime_instance("g1", "p1", 0, 0));

    let state = runtime
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(state.active_instances, 0);
    assert!(!state.pending_instance_exits.contains_key(&deployed_key));
    assert!(!state.runtime_instances.contains_key(&deployed_key));
}

/// Scenario: a completed rollout has advanced the committed active generation,
/// but observed state still contains the older generation for the same core.
/// Guarantees: controller cleanup compacts observed state to the selected
/// active generation so retained instance memory no longer grows with rollout
/// count after completion.
#[test]
fn prune_pipeline_runtime_and_history_compacts_observed_state_to_active_generation() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));
    report_ready(&runtime, deployed_key("g1", "p1", 0, 1));
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(0, 1).is_some());

    runtime
        .observed_state_store
        .set_pipeline_active_generation(pipeline_key.clone(), 1);
    runtime.prune_pipeline_runtime_and_history(&pipeline_key);

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 1
    });
    assert!(status.instance_status(0, 1).is_some());
    assert!(status.instance_status(0, 0).is_none());
}

/// Scenario: a logical pipeline has fully shut down and observed state still
/// contains an older generation alongside the final stopped generation.
/// Guarantees: controller cleanup keeps the last stopped generation per core so
/// `/status` remains useful after shutdown while superseded generations are
/// released.
#[test]
fn prune_pipeline_runtime_and_history_keeps_last_stopped_generation_view() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    report_stopped(&runtime, deployed_key("g1", "p1", 0, 0));
    report_stopped(&runtime, deployed_key("g1", "p1", 0, 1));
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(0, 1).is_some());

    runtime
        .observed_state_store
        .set_pipeline_active_generation(pipeline_key.clone(), 1);
    runtime.prune_pipeline_runtime_and_history(&pipeline_key);

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 1
    });
    assert_eq!(status.total_cores(), 1);
    assert_eq!(status.running_cores(), 0);
    assert!(matches!(
        status
            .instance_status(0, 1)
            .expect("latest stopped generation should remain")
            .phase(),
        PipelinePhase::Stopped
    ));
    assert!(status.instance_status(0, 0).is_none());
}

/// Scenario: a pure resize-down retires one core without changing the active
/// generation, and observed state still retains both core instances on that
/// same generation.
/// Guarantees: controller cleanup compacts observed state to the committed
/// active core footprint so `/status` stops counting the drained core as
/// serving after the resize completes.
#[test]
fn prune_pipeline_runtime_and_history_compacts_resize_down_same_generation() {
    let config = engine_config_with_pipeline(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
        nodes:
          receiver:
            type: "urn:test:receiver:example"
            config: null
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#,
    );
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));
    report_stopped(&runtime, deployed_key("g1", "p1", 1, 0));
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert_eq!(status.total_cores(), 2);
    assert_eq!(status.running_cores(), 1);
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(1, 0).is_some());

    runtime
        .observed_state_store
        .set_pipeline_active_cores(pipeline_key.clone(), [0]);
    runtime.prune_pipeline_runtime_and_history(&pipeline_key);

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 1
    });
    assert_eq!(status.total_cores(), 1);
    assert_eq!(status.running_cores(), 1);
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(1, 0).is_none());
}

/// Scenario: a runtime instance exits while a shutdown operation for the same
/// logical pipeline is still active and observed state contains overlapping
/// generations.
/// Guarantees: observed state is not compacted early, so controller wait paths
/// can continue reading generation-specific status until the shutdown finishes.
#[test]
fn note_instance_exit_does_not_compact_observed_state_while_shutdown_is_active() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    report_ready(&runtime, deployed_key("g1", "p1", 0, 0));
    report_ready(&runtime, deployed_key("g1", "p1", 0, 1));
    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(0, 1).is_some());

    {
        let mut state = runtime
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = state
            .active_shutdowns
            .insert(pipeline_key.clone(), "shutdown-0".to_owned());
    }

    runtime.note_instance_exit(deployed_key("g1", "p1", 0, 0), RuntimeInstanceExit::Success);

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        status.per_instance().len() == 2
    });
    assert!(status.instance_status(0, 0).is_some());
    assert!(status.instance_status(0, 1).is_some());
}

/// Scenario: a watched runtime thread panics after the runtime instance has
/// already been admitted and marked ready in observed state.
/// Guarantees: the public runtime error message stays short while the recent
/// event stores richer panic diagnostics in `ErrorSummary::source`.
#[test]
fn runtime_thread_panic_populates_error_source_in_observed_status() {
    let config = engine_config_with_pipeline(simple_pipeline_yaml());
    let runtime = test_runtime(&config);
    let _runner = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);

    let deployed_key = deployed_key("g1", "p1", 0, 0);
    let _rx =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    report_ready(&runtime, deployed_key.clone());

    let pipeline_key = PipelineKey::new("g1".into(), "p1".into());
    let _ = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        matches!(
            status
                .instance_status(0, 0)
                .map(|instance| instance.phase()),
            Some(PipelinePhase::Running)
        )
    });

    runtime.note_instance_exit(
        deployed_key,
        RuntimeInstanceExit::Error(RuntimeInstanceError::from_panic(PanicReport::capture(
            "runtime thread",
            Box::new("boom"),
            Some("pipeline-g1-p1-core-0".to_owned()),
            Some(11),
            Some(0),
        ))),
    );

    let status = wait_for_observed_status(&runtime, &pipeline_key, |status| {
        matches!(
            status
                .instance_status(0, 0)
                .map(|instance| instance.phase()),
            Some(PipelinePhase::Failed(_))
        )
    });
    let json = serde_json::to_value(&status).expect("status should serialize");
    let recent_event = &json["instances"][0]["status"]["recentEvents"][0]["Engine"];
    let error = &recent_event["type"]["Error"]["RuntimeError"]["Pipeline"];
    assert_eq!(
        recent_event["message"],
        "Pipeline encountered a runtime error."
    );
    assert_eq!(error["error_kind"], "panic");
    assert_eq!(error["message"], "runtime thread panicked: boom");
    let source = error["source"]
        .as_str()
        .expect("runtime panic source should be serialized");
    assert!(source.contains("thread_name=pipeline-g1-p1-core-0"));
    assert!(source.contains("thread_id=11"));
    assert!(source.contains("core_id=0"));
    assert!(source.contains("backtrace:"));
}
