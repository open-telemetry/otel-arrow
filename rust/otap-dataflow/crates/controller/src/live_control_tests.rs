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
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::receiver::ReceiverWrapper;
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
) -> Result<ReceiverWrapper<()>, otap_df_config::error::Error> {
    panic!("test receiver factory should not be constructed")
}

fn test_exporter_create(
    _pipeline_ctx: PipelineContext,
    _node: otap_df_engine::node::NodeId,
    _node_config: Arc<NodeUserConfig>,
    _exporter_config: &ExporterConfig,
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
    PipelineFactory::new(TEST_RECEIVER_FACTORIES, &[], TEST_EXPORTER_FACTORIES);

fn test_runtime(config: &OtelDataflowSpec) -> Arc<ControllerRuntime<()>> {
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
    runtime.register_committed_pipeline(resolved, 0);
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
    assert!(plan.rollout.cores.is_empty());
    assert!(plan.resize_start_cores.is_empty());
    assert!(plan.resize_stop_cores.is_empty());
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
    let expired_at = Instant::now() - TERMINAL_OPERATION_RETENTION_TTL - Duration::from_secs(1);

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
