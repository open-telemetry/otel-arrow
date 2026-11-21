// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_engine::work_coordinator::{
    AssignmentPhase, CoordinatorDeclaration, CoordinatorError, CoordinatorRuntime,
    ParticipantEvent, WorkCoordinator, WorkId, WorkParticipant, WorkProtocol, WorkSpec,
};
use std::{collections::BTreeSet, net::UdpSocket, rc::Rc};

#[derive(Debug)]
struct Scrapes;
impl WorkProtocol for Scrapes {
    type Work = String;
    type ParticipantRequest = ();
    type CoordinatorNotice = ();
}

// Both provider and node deliberately own !Send state. Only the configuration
// factory and typed metadata cross runtime threads.
struct Discovery {
    sink: String,
    _local: Rc<()>,
}
#[async_trait(?Send)]
impl WorkCoordinator<Scrapes> for Discovery {
    async fn run(
        self: Box<Self>,
        mut runtime: CoordinatorRuntime<Scrapes>,
    ) -> Result<(), CoordinatorError> {
        let socket = tokio::net::UdpSocket::bind("127.0.0.1:0")
            .await
            .map_err(|e| CoordinatorError::Failed(e.to_string()))?;
        let _ = socket
            .send_to(b"discovery", &self.sink)
            .await
            .map_err(|e| CoordinatorError::Failed(e.to_string()))?;
        runtime
            .replace_catalog(
                (0..4)
                    .map(|index| WorkSpec {
                        id: WorkId(format!("target-{index}")),
                        work: self.sink.clone(),
                    })
                    .collect(),
            )
            .await?;
        runtime.ready()?;
        runtime.shutdown_requested().await;
        Ok(())
    }
}

fn declarations(
    config: &NodeUserConfig,
) -> Result<Vec<CoordinatorDeclaration>, otel_arrow_dfe_config::error::Error> {
    let sink = config.config["sink"]
        .as_str()
        .expect("fixture sink")
        .to_owned();
    Ok(vec![CoordinatorDeclaration::new::<Scrapes, _>(
        "targets",
        move || {
            Box::new(Discovery {
                sink,
                _local: Rc::new(()),
            })
        },
    )])
}

struct Scraper {
    sink: String,
    participant: WorkParticipant<Scrapes>,
    _local: Rc<()>,
}
#[async_trait(?Send)]
impl receiver::Receiver<()> for Scraper {
    async fn start(
        self: Box<Self>,
        mut controls: receiver::ControlChannel<()>,
        effects: receiver::EffectHandler<()>,
    ) -> Result<TerminalState, EngineError> {
        let Self {
            mut participant,
            sink,
            _local,
        } = *self;
        let failure = |error: String| EngineError::ReceiverError {
            receiver: effects.receiver_id(),
            kind: otel_arrow_dfe_engine::error::ReceiverErrorKind::Other,
            error,
            source_detail: String::new(),
        };
        let socket = tokio::net::UdpSocket::bind("127.0.0.1:0")
            .await
            .map_err(|e| failure(e.to_string()))?;
        participant
            .ready_for_assignments()
            .await
            .map_err(|e| failure(e.to_string()))?;
        effects.signal_ready();
        let mut installed = BTreeSet::new();
        loop {
            tokio::select! {
                message = controls.recv() => match message? {
                    NodeControlMsg::DrainIngress { deadline, .. } => {
                        effects.notify_receiver_drained().await?;
                        let _ = socket.send_to(format!("drained:{}", participant.session().core_id).as_bytes(), &sink).await.map_err(|e| failure(e.to_string()))?;
                        return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                    }
                    NodeControlMsg::Shutdown { deadline, .. } => return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, [])),
                    _ => {},
                },
                event = participant.recv() => {
                    let ParticipantEvent::Assignments(snapshot) = event.map_err(|e| failure(e.to_string()))? else { continue; };
                    for assignment in &snapshot.assignments {
                        match &assignment.phase {
                            AssignmentPhase::Offered(offer) => participant.accept(offer).await.map_err(|e| failure(e.to_string()))?,
                            AssignmentPhase::Active(grant) if installed.insert(grant.identity().term) => {
                                participant.activated(grant).await.map_err(|e| failure(e.to_string()))?;
                                let _ticket = grant.try_admit().map_err(|e| failure(e.to_string()))?;
                                let payload = format!("{}:{}", assignment.work.id.0, participant.session().core_id);
                                let _ = socket.send_to(payload.as_bytes(), &assignment.work.work).await.map_err(|e| failure(e.to_string()))?;
                            }
                            AssignmentPhase::Revoking(grant) => participant.released(grant).await.map_err(|e| failure(e.to_string()))?,
                            _ => {},
                        }
                    }
                }
            }
        }
    }
}

fn scraper_create(
    context: PipelineContext,
    node: otel_arrow_dfe_engine::node::NodeId,
    config: Arc<NodeUserConfig>,
    receiver: &ReceiverConfig,
    _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
    let participant = context
        .work_coordinator::<Scrapes>("targets")
        .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        })?;
    Ok(ReceiverWrapper::local(
        Scraper {
            sink: config.config["sink"].as_str().expect("sink").to_owned(),
            participant,
            _local: Rc::new(()),
        },
        node,
        config,
        receiver,
    ))
}

static RECEIVERS: &[ReceiverFactory<()>] = &[
    ReceiverFactory {
        name: "urn:test:receiver:coordinated",
        create: scraper_create,
        work_coordinators: Some(declarations),
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
    ReceiverFactory {
        name: "urn:otel:receiver:internal_telemetry",
        create: test_receiver_create,
        work_coordinators: None,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: test_validate_config,
    },
];
static FACTORY: PipelineFactory<()> = PipelineFactory::new(
    RECEIVERS,
    TEST_PROCESSOR_FACTORIES,
    RECOVERY_TEST_EXPORTER_FACTORIES,
    &[],
);

fn coordinated_config(sink: &str, cores: usize) -> OtelDataflowSpec {
    engine_config_with_pipeline(&format!(
        r#"
        policies:
          resources:
            core_allocation:
              type: core_count
              count: {cores}
        nodes:
          receiver:
            type: "urn:test:receiver:coordinated"
            config:
              sink: "{sink}"
          exporter:
            type: "urn:test:exporter:example"
            config: null
        connections:
          - from: receiver
            to: exporter
"#
    ))
}

/// Scenario: a real two-core pipeline declares one !Send discovery coordinator from its receiver factory.
/// Guarantees: the controller hosts discovery once, binds both !Send nodes, executes every target on one core, and joins shutdown.
#[test]
fn basic_receiver_executes_targets_across_pipeline_instances() {
    let sink = UdpSocket::bind("127.0.0.1:0").expect("isolated fixture socket");
    sink.set_read_timeout(Some(Duration::from_secs(15)))
        .expect("fixture deadline");
    let config = coordinated_config(&sink.local_addr().expect("address").to_string(), 2);
    let runtime = test_runtime_with_factory(&config, &FACTORY);
    let _observed = ObservedStateRunner::start(&runtime);
    register_existing_pipeline(&runtime, &config);
    let deployment = runtime
        .state
        .lock()
        .expect("state")
        .logical_pipelines
        .values()
        .next()
        .expect("deployment")
        .clone();
    for core in &deployment.placement.cores {
        let _ = runtime
            .launch_regular_pipeline_instance(&deployment, core.core_id.id, 0)
            .expect("launch real pipeline");
    }
    let mut received = BTreeSet::new();
    let mut buffer = [0; 1024];
    for _ in 0..5 {
        let length = sink
            .recv(&mut buffer)
            .expect("discovery and four target executions before deadline");
        assert!(
            received.insert(String::from_utf8(buffer[..length].to_vec()).expect("fixture UTF8")),
            "no duplicate executions"
        );
    }
    assert!(received.remove("discovery"));
    assert_eq!(received.len(), 4);
    let cores: BTreeSet<_> = received
        .iter()
        .map(|event| event.split_once(':').expect("target:core").1)
        .collect();
    assert_eq!(cores.len(), 2, "targets execute on both pipeline instances");
    runtime.request_shutdown_all(5).expect("drain fleet");
    for _ in 0..2 {
        let length = sink
            .recv(&mut buffer)
            .expect("both receivers drain before deadline");
        assert!(
            std::str::from_utf8(&buffer[..length])
                .expect("drain witness")
                .starts_with("drained:")
        );
    }
    assert!(
        runtime.wait_until_all_instances_exit_for(Duration::from_secs(15)),
        "all runtime instances exit"
    );
    assert!(
        runtime.take_runtime_error().is_none(),
        "normal drain does not trigger failure recovery"
    );
    runtime
        .shutdown_work_coordinators()
        .expect("provider joined");
}

/// Scenario: Basic has a committed deployment and receives a no-op, resize, or settings replacement.
/// Guarantees: no-op retains its host identity; unsupported changes fail before generation or rollout counters mutate.
#[test]
fn basic_reconciliation_rejects_changes_before_mutation() {
    let config = coordinated_config("127.0.0.1:1", 1);
    let runtime = test_runtime_with_factory(&config, &FACTORY);
    register_existing_pipeline(&runtime, &config);
    let _control =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let current = runtime
        .state
        .lock()
        .expect("state")
        .logical_pipelines
        .values()
        .next()
        .expect("deployment")
        .clone();
    let request = |pipeline| ReconfigureRequest {
        pipeline,
        step_timeout_secs: 10,
        drain_timeout_secs: 10,
    };
    let noop = runtime
        .prepare_rollout_plan("g1", "p1", &request(current.resolved.pipeline.clone()))
        .expect("no-op allowed");
    assert_eq!(noop.action, RolloutAction::NoOp);
    assert!(Arc::ptr_eq(
        &noop.target_deployment.work_coordinators,
        &current.work_coordinators
    ));
    let before = {
        let state = runtime.state.lock().expect("state");
        (state.next_rollout_id, state.generation_counters.clone())
    };
    for target in [
        coordinated_config("127.0.0.1:1", 2),
        coordinated_config("127.0.0.1:2", 1),
    ] {
        let candidate = target
            .resolve()
            .pipelines
            .into_iter()
            .find(|p| p.pipeline_id.as_ref() == "p1")
            .expect("candidate")
            .pipeline;
        let error = runtime
            .prepare_rollout_plan("g1", "p1", &request(candidate))
            .expect_err("Basic mutation rejected");
        assert!(matches!(error, ControlPlaneError::InvalidRequest { .. }));
    }
    let state = runtime.state.lock().expect("state");
    assert_eq!(
        (state.next_rollout_id, state.generation_counters.clone()),
        before
    );
}

/// Scenario: a Basic participant fails under a policy that would ordinarily recover one core.
/// Guarantees: the controller stops only that fleet, retains the cause, and leaves the sibling serving.
#[test]
fn basic_participant_failure_suppresses_per_core_recovery() {
    let config = coordinated_config("127.0.0.1:1", 2);
    let runtime = test_runtime_with_factory(&config, &FACTORY);
    register_existing_pipeline(&runtime, &config);
    let _failed_control =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let mut remaining_control =
        register_runtime_instance(&runtime, "g1", "p1", 1, 0, RuntimeInstanceLifecycle::Active);
    let mut sibling = register_runtime_instance(
        &runtime,
        "g1",
        "other",
        0,
        0,
        RuntimeInstanceLifecycle::Active,
    );
    let owner = {
        let mut state = runtime.state.lock().expect("state");
        let owner = Arc::clone(
            &state
                .logical_pipelines
                .values()
                .next()
                .expect("deployment")
                .work_coordinators,
        );
        for core in 0..2 {
            state
                .runtime_instances
                .get_mut(&deployed_key("g1", "p1", core, 0))
                .expect("registered")
                .coordinator_owner = Some(Arc::downgrade(&owner));
        }
        owner
    };
    runtime.note_instance_exit(
        deployed_key("g1", "p1", 0, 0),
        RuntimeInstanceExit::Error(RuntimeInstanceError::runtime("participant failed".into())),
    );
    assert!(matches!(
        remaining_control.try_recv(),
        Ok(RuntimeControlMsg::Shutdown { .. })
    ));
    {
        let state = runtime.state.lock().expect("state");
        assert!(
            state
                .runtime_recoveries
                .values()
                .all(|recovery| recovery.worker_id.is_none())
        );
        assert!(!state.global_shutdown_requested);
    }
    runtime.note_instance_exit(deployed_key("g1", "p1", 1, 0), RuntimeInstanceExit::Success);
    assert!(
        sibling.try_recv().is_err(),
        "unrelated pipeline received no shutdown"
    );
    assert_eq!(
        owner.failure.get().map(String::as_str),
        Some("participant failed")
    );
    assert!(runtime.take_runtime_error().is_none());
    runtime.note_instance_exit(
        deployed_key("g1", "other", 0, 0),
        RuntimeInstanceExit::Success,
    );
}

/// Scenario: discovery fails after candidate readiness but before a create rollout commits.
/// Guarantees: the candidate cannot commit or shut down a sibling, and rollback preserves the cause.
#[test]
fn candidate_failure_at_commit_rolls_back_only_candidate() {
    let config = coordinated_config("127.0.0.1:1", 1);
    let runtime = test_runtime_with_factory(&config, &FACTORY);
    let pipeline = config
        .resolve()
        .pipelines
        .into_iter()
        .find(|p| p.pipeline_id.as_ref() == "p1")
        .expect("source config")
        .pipeline;
    let request = ReconfigureRequest {
        pipeline,
        step_timeout_secs: 1,
        drain_timeout_secs: 1,
    };
    let plan = runtime
        .prepare_rollout_plan("g1", "candidate", &request)
        .expect("create");
    let mut sibling =
        register_runtime_instance(&runtime, "g1", "p1", 0, 0, RuntimeInstanceLifecycle::Active);
    let owner = &plan.target_deployment.work_coordinators;
    let generation = plan.target_deployment.baseline_generation;
    let id = deployed_key("g1", "candidate", plan.target_assigned_cores[0], generation);
    let _control = register_runtime_instance(
        &runtime,
        "g1",
        "candidate",
        id.core_id,
        generation,
        RuntimeInstanceLifecycle::Exited(RuntimeInstanceExit::Error(
            RuntimeInstanceError::runtime("reader exited".into()),
        )),
    );
    runtime
        .state
        .lock()
        .expect("state")
        .runtime_instances
        .get_mut(&id)
        .expect("recorded exit")
        .coordinator_owner = Some(Arc::downgrade(owner));
    assert_eq!(
        runtime
            .commit_pipeline_deployment(&plan)
            .expect_err("exit recorded before callback"),
        "reader exited"
    );
    runtime.fail_coordinated_deployment(
        &plan.pipeline_key,
        plan.target_deployment.baseline_generation,
        owner,
        "discovery failed".into(),
    );
    let reason = runtime
        .commit_pipeline_deployment(&plan)
        .expect_err("failed candidate");
    assert_eq!(reason, "discovery failed");
    assert!(matches!(
        runtime.rollback_create_rollout(&plan, &[], reason),
        Err(RolloutExecutionError::Failed(_))
    ));
    {
        let state = runtime.state.lock().expect("state");
        assert!(!state.logical_pipelines.contains_key(&plan.pipeline_key));
        assert!(!state.global_shutdown_requested && state.first_error.is_none());
    }
    assert!(sibling.try_recv().is_err());

    // Even a previously prepared create cannot reuse the key while old workers linger.
    let retry = runtime
        .prepare_rollout_plan("g1", "candidate", &request)
        .expect("prepare retry");
    let _control = register_runtime_instance(
        &runtime,
        "g1",
        "candidate",
        0,
        0,
        RuntimeInstanceLifecycle::Active,
    );
    let id = deployed_key("g1", "candidate", 0, 0);
    runtime
        .state
        .lock()
        .expect("state")
        .runtime_instances
        .get_mut(&id)
        .expect("candidate worker")
        .coordinator_owner = Some(Arc::downgrade(owner));
    assert!(matches!(
        runtime.insert_rollout_plan(&retry),
        Err(ControlPlaneError::RolloutConflict)
    ));
    runtime.note_instance_exit(id, RuntimeInstanceExit::Success);
    runtime
        .insert_rollout_plan(&retry)
        .expect("reservation cleared after confirmed exit");
}

/// Scenario: a live create rolls back after one coordinated receiver is ready.
/// Guarantees: the candidate quiesces before its participant exits, and its host joins without a spurious failure.
#[test]
fn create_rollback_quiesces_candidate_before_drain() {
    let sink = UdpSocket::bind("127.0.0.1:0").expect("fixture socket");
    let config = coordinated_config(&sink.local_addr().expect("address").to_string(), 2);
    let runtime = test_runtime_with_factory(&empty_pipeline_group_config(), &FACTORY);
    let _observed = ObservedStateRunner::start(&runtime);
    let pipeline = config
        .resolve()
        .pipelines
        .into_iter()
        .find(|p| p.pipeline_id.as_ref() == "p1")
        .expect("source config")
        .pipeline;
    let plan = prepare_rollback_test_plan(&runtime, pipeline, 10, 10);
    let deployment = &plan.target_deployment;
    let id = runtime
        .launch_regular_pipeline_instance(
            deployment,
            plan.target_assigned_cores[0],
            deployment.baseline_generation,
        )
        .expect("launch candidate");
    runtime
        .wait_for_pipeline_ready(&id, Instant::now() + Duration::from_secs(10))
        .expect("candidate ready");
    let result = runtime.rollback_create_rollout(&plan, &[id], "cancel create".to_owned());
    assert!(
        matches!(result, Err(RolloutExecutionError::Failed(reason)) if reason == "cancel create")
    );
    assert!(deployment.work_coordinators.failure.get().is_none());
    let hosts = deployment
        .work_coordinators
        .hosts
        .get()
        .expect("host initialized")
        .as_ref()
        .expect("host started");
    assert!(
        matches!(
            hosts.bindings.take::<Scrapes>(
                &plan.pipeline_group_id,
                &plan.pipeline_id,
                &"receiver".into(),
                "targets",
                plan.target_assigned_cores[1],
                deployment.baseline_generation
            ),
            Err(CoordinatorError::Unavailable)
        ),
        "the host stopped even though the second instance never bound"
    );

    assert!(runtime.take_runtime_error().is_none());
}

static PROCESSORS: &[ProcessorFactory<()>] = &[ProcessorFactory {
    name: "urn:test:processor:coordinated",
    create: test_processor_create,
    work_coordinators: Some(declarations),
    context_declarations: None,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: test_validate_config,
}];
static EXPORTERS: &[ExporterFactory<()>] = &[ExporterFactory {
    name: "urn:test:exporter:coordinated",
    create: test_exporter_create,
    work_coordinators: Some(declarations),
    context_declarations: None,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: test_validate_config,
}];

/// Scenario: receivers, processors and exporters declare the same protocol role under different node IDs.
/// Guarantees: declarations remain node-scoped and every node kind uses the same typed PipelineContext binding.
#[test]
fn coordinator_binding_is_available_to_every_node_kind() {
    use otel_arrow_dfe_engine::work_coordinator::{
        CoordinatorBindings, CoordinatorKey, ParticipantSession,
    };
    let factory = PipelineFactory::new(RECEIVERS, PROCESSORS, EXPORTERS, &[]);
    let config = PipelineConfig::from_yaml(
        "g".into(),
        "p".into(),
        r#"
nodes:
  input:
    type: urn:test:receiver:coordinated
    config: {sink: "127.0.0.1:1"}
  process:
    type: urn:test:processor:coordinated
    config: {sink: "127.0.0.1:1"}
  output:
    type: urn:test:exporter:coordinated
    config: {sink: "127.0.0.1:1"}
connections:
  - from: input
    to: process
  - from: process
    to: output
"#,
    )
    .expect("pipeline");
    let declarations = factory
        .work_coordinator_declarations(&config)
        .expect("declarations");
    assert_eq!(declarations.len(), 3);
    let mut bindings = CoordinatorBindings::default();
    let mut tasks = Vec::new();
    for (node, declaration) in declarations {
        let key = CoordinatorKey {
            group_id: "g".into(),
            pipeline_id: "p".into(),
            node_id: node.clone(),
            role: declaration.role.clone(),
        };
        let (binding, task) = declaration
            .prepare(
                key,
                vec![ParticipantSession {
                    core_id: 0,
                    generation: 7,
                    nonce: 1,
                }],
                Arc::new(|_| panic!("normal setup cleanup must not fail")),
            )
            .expect("prepare");
        bindings
            .insert(node, "targets".into(), binding)
            .expect("unique node role");
        tasks.push(task);
    }
    let bindings = Arc::new(bindings);
    let mut context = ControllerContext::new(TelemetryRegistryHandle::new())
        .pipeline_context_with_generation("g".into(), "p".into(), 0, 1, 0, 7);
    context.set_work_coordinators(Arc::clone(&bindings));
    let mut participants = Vec::new();
    for (node, settings) in config.node_iter() {
        let node_context = context.with_node_context(
            node.clone(),
            settings.r#type.clone(),
            settings.kind(),
            HashMap::new(),
        );
        participants.push(
            node_context
                .work_coordinator::<Scrapes>("targets")
                .expect("typed node binding"),
        );
    }
    bindings.begin_shutdown();
    drop(participants);
    bindings.stop();
    drop(tasks);
}
