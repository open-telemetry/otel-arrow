// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_engine::work_coordinator::{CoordinatorLifecycleEvent, ParticipantSpec};
use serde_json::{Value, json};

struct CooperativeDiscovery {
    settings: Value,
    _local: Rc<()>,
}

fn catalog(settings: &Value) -> Vec<WorkSpec<Scrapes>> {
    (0..settings["targets"].as_u64().expect("target count"))
        .map(|index| WorkSpec {
            id: WorkId(format!("target-{index}")),
            work: format!(
                "{}@{}",
                settings["version"].as_u64().expect("version"),
                settings["sink"].as_str().expect("sink")
            ),
        })
        .collect()
}

#[async_trait(?Send)]
impl WorkCoordinator<Scrapes> for CooperativeDiscovery {
    async fn run(
        self: Box<Self>,
        mut runtime: CoordinatorRuntime<Scrapes>,
    ) -> Result<(), CoordinatorError> {
        let socket = tokio::net::UdpSocket::bind("127.0.0.1:0")
            .await
            .map_err(|e| CoordinatorError::Failed(e.to_string()))?;
        let sink = self.settings["sink"].as_str().expect("sink");
        let _ = socket
            .send_to(b"discovery", sink)
            .await
            .map_err(|e| CoordinatorError::Failed(e.to_string()))?;
        runtime.replace_catalog(catalog(&self.settings)).await?;
        let mut lifecycle = runtime.lifecycle_events()?;
        runtime.ready()?;
        let mut stopped = runtime.shutdown_signal();
        loop {
            let (message, receipt) = tokio::select! {
                biased;
                _ = stopped.changed() => return Ok(()),
                event = lifecycle.recv() => match event.ok_or(CoordinatorError::Unavailable)? {
                    CoordinatorLifecycleEvent::Prepare { revision, settings, receipt, .. } => {
                        match runtime.replace_catalog_for(revision, catalog(&settings)).await {
                            Ok(()) | Err(CoordinatorError::Stale) => {},
                            Err(error) => return Err(error),
                        }
                        (format!("staged {revision}"), receipt)
                    }
                    CoordinatorLifecycleEvent::Commit { revision, receipt, .. } => (format!("committed {revision}"), receipt),
                    CoordinatorLifecycleEvent::Abort { revision, receipt, .. } => (format!("aborted {revision}"), receipt),
                }
            };
            let _ = socket
                .send_to(message.as_bytes(), sink)
                .await
                .map_err(|e| CoordinatorError::Failed(e.to_string()))?;
            // This fixture has no detached discovery workers. Engine tests hold
            // these receipts explicitly to exercise pending worker cleanup.
            match receipt.acknowledge(Ok(())) {
                Ok(()) | Err(CoordinatorError::Stale) => {}
                Err(error) => return Err(error),
            }
        }
    }
}

fn cooperative_declarations(
    config: &NodeUserConfig,
) -> Result<Vec<CoordinatorDeclaration>, otel_arrow_dfe_config::error::Error> {
    let settings = json!({"sink": config.config["sink"], "version": config.config["version"], "targets": config.config["targets"]});
    let initial = settings.clone();
    Ok(vec![
        CoordinatorDeclaration::new::<Scrapes, _>("targets", move || {
            Box::new(CooperativeDiscovery {
                settings: initial,
                _local: Rc::new(()),
            })
        })
        .with_reconciliation(settings, |old, next| old["sink"] == next["sink"]),
    ])
}

struct CooperativeScraper {
    sink: String,
    participant: WorkParticipant<Scrapes>,
    block_install: bool,
    _local: Rc<()>,
}

#[async_trait(?Send)]
impl receiver::Receiver<()> for CooperativeScraper {
    async fn start(
        self: Box<Self>,
        mut controls: receiver::ControlChannel<()>,
        effects: receiver::EffectHandler<()>,
    ) -> Result<TerminalState, EngineError> {
        let Self {
            sink,
            mut participant,
            block_install,
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
        let session = participant.session();
        let ready = format!(
            "ready {} {} {}",
            session.core_id,
            session.generation,
            socket.local_addr().expect("command address")
        );
        let _ = socket
            .send_to(ready.as_bytes(), &sink)
            .await
            .map_err(|e| failure(e.to_string()))?;
        let mut installed = BTreeSet::new();
        let mut command = [0; 16];
        loop {
            tokio::select! {
                message = controls.recv() => match message? {
                    NodeControlMsg::DrainIngress { deadline, .. } => {
                        effects.notify_receiver_drained().await?;
                        return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                    }
                    NodeControlMsg::Shutdown { deadline, .. } => return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, [])),
                    _ => {},
                },
                received = socket.recv(&mut command) => {
                    let length = received.map_err(|e| failure(e.to_string()))?;
                    assert_eq!(&command[..length], b"fail");
                    return Err(failure("fixture participant failure".into()));
                }
                event = participant.recv() => {
                    let ParticipantEvent::Assignments(snapshot) = event.map_err(|e| failure(e.to_string()))? else { continue; };
                    for assignment in &snapshot.assignments {
                        let result = match &assignment.phase {
                            AssignmentPhase::Offered(offer) => participant.accept(offer).await,
                            AssignmentPhase::Active(grant) if installed.insert(grant.identity().term) => {
                                if block_install {
                                    let message = format!("uninstalled {} {}", session.core_id, session.generation);
                                    let _ = socket.send_to(message.as_bytes(), &sink).await.map_err(|e| failure(e.to_string()))?;
                                    continue;
                                }
                                match participant.activated(grant).await {
                                    Ok(()) => {
                                        if let Ok(_ticket) = grant.try_admit() {
                                            let (version, target) = assignment.work.work.split_once('@').expect("version and sink");
                                            let id = grant.identity();
                                            let message = format!("active {} {} {} {} {} {} {version}", session.core_id, session.generation, id.revision, id.term, session.nonce, id.work_id.0);
                                            let _ = socket.send_to(message.as_bytes(), target).await.map_err(|e| failure(e.to_string()))?;
                                        }
                                        Ok(())
                                    }
                                    result => result,
                                }
                            }
                            AssignmentPhase::Revoking(grant) => participant.released(grant).await,
                            _ => Ok(()),
                        };
                        match result {
                            Ok(()) | Err(CoordinatorError::Stale) => {},
                            Err(error) => return Err(failure(error.to_string())),
                        }
                    }
                }
            }
        }
    }
}

fn cooperative_create(
    context: PipelineContext,
    node: otel_arrow_dfe_engine::node::NodeId,
    config: Arc<NodeUserConfig>,
    receiver: &ReceiverConfig,
    _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
    if config.config["fail_core"].as_u64() == Some(context.core_id() as u64) {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "fixture candidate construction failed".into(),
        });
    }
    let participant = context
        .work_coordinator::<Scrapes>("targets")
        .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        })?;
    Ok(ReceiverWrapper::local(
        CooperativeScraper {
            sink: config.config["sink"].as_str().expect("sink").into(),
            participant,
            block_install: config.config["block_install_core"].as_u64()
                == Some(context.core_id() as u64),
            _local: Rc::new(()),
        },
        node,
        config,
        receiver,
    ))
}

static COOPERATIVE_RECEIVERS: &[ReceiverFactory<()>] = &[
    ReceiverFactory {
        name: "urn:test:receiver:cooperative",
        create: cooperative_create,
        work_coordinators: Some(cooperative_declarations),
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
static COOPERATIVE_FACTORY: PipelineFactory<()> = PipelineFactory::new(
    COOPERATIVE_RECEIVERS,
    TEST_PROCESSOR_FACTORIES,
    RECOVERY_TEST_EXPORTER_FACTORIES,
    &[],
);

fn config(
    sink: &str,
    cores: usize,
    version: u64,
    targets: usize,
    fail_core: Option<usize>,
) -> OtelDataflowSpec {
    config_with_block(sink, cores, version, targets, fail_core, None)
}

fn config_with_block(
    sink: &str,
    cores: usize,
    version: u64,
    targets: usize,
    fail_core: Option<usize>,
    block_install_core: Option<usize>,
) -> OtelDataflowSpec {
    let settings = json!({"sink": sink, "version": version, "targets": targets, "fail_core": fail_core, "block_install_core": block_install_core});
    engine_config_with_pipeline(&format!(
        r#"
        policies:
          runtime_recovery:
            initial_backoff: 1ms
            max_backoff: 1ms
            startup_timeout: 10s
          resources:
            core_allocation:
              type: core_count
              count: {cores}
        nodes:
          receiver:
            type: urn:test:receiver:cooperative
            config: {settings}
          exporter:
            type: urn:test:exporter:example
            config: null
        connections:
          - from: receiver
            to: exporter
"#
    ))
}

// UDP witnesses cross real, independent runtime threads, so virtual time is
// unsuitable. Every receive has a generous failure deadline; no sleep proves
// readiness or absence. Drop shuts down and joins even after an assertion fails.
struct Fixture {
    runtime: Arc<ControllerRuntime<()>>,
    _observed: ObservedStateRunner,
    sink: UdpSocket,
    events: Vec<String>,
    targets: usize,
}
impl Fixture {
    fn start(cores: usize, targets: usize) -> Self {
        let sink = UdpSocket::bind("127.0.0.1:0").expect("fixture socket");
        sink.set_read_timeout(Some(Duration::from_secs(20)))
            .expect("deadline");
        let initial = config(
            &sink.local_addr().expect("sink address").to_string(),
            cores,
            0,
            targets,
            None,
        );
        let runtime = test_runtime_with_factory(&initial, &COOPERATIVE_FACTORY);
        let observed = ObservedStateRunner::start(&runtime);
        register_existing_pipeline(&runtime, &initial);
        let deployment = runtime
            .state
            .lock()
            .expect("state")
            .logical_pipelines
            .values()
            .next()
            .expect("deployment")
            .clone();
        let mut fixture = Self {
            runtime,
            _observed: observed,
            sink,
            events: Vec::new(),
            targets,
        };
        for core in &deployment.placement.cores {
            let _ = fixture
                .runtime
                .launch_regular_pipeline_instance(&deployment, core.core_id.id, 0)
                .expect("launch");
        }
        fixture.until(|events| {
            events.iter().filter(|e| e.starts_with("ready ")).count() == cores
                && events.iter().filter(|e| e.starts_with("active ")).count() == targets
        });
        fixture
    }
    fn until(&mut self, predicate: impl Fn(&[String]) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while !predicate(&self.events) {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .unwrap_or_else(|| {
                    panic!("fixture transition deadline; observed {:?}", self.events)
                });
            self.sink
                .set_read_timeout(Some(remaining))
                .expect("receive deadline");
            let mut buffer = [0; 1024];
            let length = self.sink.recv(&mut buffer).unwrap_or_else(|error| {
                panic!(
                    "expected fixture transition: {error}; observed {:?}",
                    self.events
                )
            });
            self.events
                .push(String::from_utf8(buffer[..length].to_vec()).expect("fixture message"));
        }
    }
    fn hosts(&self) -> Arc<work_coordinator::PipelineCoordinators> {
        self.runtime
            .state
            .lock()
            .expect("state")
            .logical_pipelines
            .values()
            .next()
            .expect("deployment")
            .work_coordinators
            .get()
            .expect("initialized")
            .as_ref()
            .expect("healthy hosts")
            .clone()
    }
    fn rollout(&self, cores: usize, version: u64, fail_core: Option<usize>) -> RolloutStatus {
        let target = config(
            &self.sink.local_addr().expect("sink").to_string(),
            cores,
            version,
            self.targets,
            fail_core,
        )
        .resolve()
        .pipelines
        .into_iter()
        .find(|p| p.pipeline_id.as_ref() == "p1")
        .expect("target")
        .pipeline;
        let plan = self
            .runtime
            .prepare_rollout_plan(
                "g1",
                "p1",
                &ReconfigureRequest {
                    pipeline: target,
                    step_timeout_secs: 10,
                    drain_timeout_secs: 10,
                },
            )
            .expect("compatible rollout");
        let id = plan.rollout.rollout_id.clone();
        self.runtime
            .insert_rollout_plan(&plan)
            .expect("reserve rollout");
        Arc::clone(&self.runtime).run_rollout(plan);
        self.runtime
            .rollout_status_snapshot(&id)
            .expect("terminal rollout")
    }
    fn fail(&self, core: usize, generation: u64) {
        let prefix = format!("ready {core} {generation} ");
        let address = self
            .events
            .iter()
            .rev()
            .find_map(|e| e.strip_prefix(&prefix))
            .expect("ready command endpoint");
        let _ = self
            .sink
            .send_to(b"fail", address)
            .expect("trigger exact instance failure");
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let requested = self.runtime.request_shutdown_all(5);
        let exited = self
            .runtime
            .wait_until_all_instances_exit_for(Duration::from_secs(15));
        let joined = self.runtime.shutdown_work_coordinators();
        if !thread::panicking() {
            requested.expect("fixture shutdown dispatched");
            assert!(exited, "fixture pipeline threads exited");
            joined.expect("fixture provider threads joined");
        }
    }
}

/// Scenario: a running source grows, shrinks, and re-adds the same core under one generation.
/// Guarantees: discovery survives, work moves only after release, and a re-added core gets a fresh session without duplicate authority.
#[test]
fn resize_reuses_host_and_refreshes_readded_session() {
    let mut fixture = Fixture::start(1, 4);
    let hosts = fixture.hosts();
    assert_eq!(
        fixture.rollout(2, 0, None).state,
        ApiPipelineRolloutState::Succeeded
    );
    fixture.until(|events| {
        events
            .iter()
            .filter(|e| e.starts_with("active 1 0 "))
            .count()
            == 2
    });
    let nonce = fixture
        .events
        .iter()
        .find(|e| e.starts_with("active 1 0 "))
        .expect("added core")
        .split_whitespace()
        .nth(5)
        .expect("nonce")
        .to_owned();
    assert_eq!(
        fixture.rollout(1, 0, None).state,
        ApiPipelineRolloutState::Succeeded
    );
    assert_eq!(
        fixture.rollout(2, 0, None).state,
        ApiPipelineRolloutState::Succeeded
    );
    fixture.until(|events| {
        events
            .iter()
            .filter(|e| {
                e.starts_with("active 1 0 ") && e.split_whitespace().nth(5) != Some(nonce.as_str())
            })
            .count()
            >= 2
    });
    let readded = fixture
        .events
        .iter()
        .rev()
        .find(|e| e.starts_with("active 1 0 "))
        .expect("re-added core");
    assert_ne!(
        readded.split_whitespace().nth(5).expect("fresh nonce"),
        nonce
    );
    assert!(Arc::ptr_eq(&hosts, &fixture.hosts()));
    assert_eq!(
        fixture.events.iter().filter(|e| *e == "discovery").count(),
        1
    );
    assert!(fixture.runtime.take_runtime_error().is_none());
}

/// Scenario: both serving cores receive compatible discovery settings through a rolling replacement.
/// Guarantees: one host stages and commits the winning revision; every replacement installs work from that revision before success.
#[test]
fn replacement_commits_revision_without_restarting_discovery() {
    let mut fixture = Fixture::start(2, 4);
    let hosts = fixture.hosts();
    assert_eq!(
        fixture.rollout(2, 1, None).state,
        ApiPipelineRolloutState::Succeeded
    );
    fixture.until(|events| {
        events
            .iter()
            .filter(|e| e.starts_with("active ") && e.split_whitespace().nth(3) == Some("1"))
            .map(|e| e.split_whitespace().nth(6).expect("work ID"))
            .collect::<BTreeSet<_>>()
            .len()
            == 4
            && events.iter().any(|e| e == "committed 1")
    });
    assert!(Arc::ptr_eq(&hosts, &fixture.hosts()));
    assert_eq!(
        hosts.bindings.iter().next().expect("binding").1.revision(),
        1
    );
    assert!(fixture.runtime.take_runtime_error().is_none());
}

/// Scenario: the second candidate fails construction after the first core has switched settings.
/// Guarantees: rollback relaunches the first core with old settings, abort joins staged discovery, and the old deployment remains usable.
#[test]
fn failed_replacement_restores_switched_core_and_catalog() {
    let mut fixture = Fixture::start(2, 4);
    let hosts = fixture.hosts();
    let failed = fixture.rollout(2, 1, Some(1));
    assert_eq!(
        failed.state,
        ApiPipelineRolloutState::Failed,
        "{:?}",
        failed.failure_reason
    );
    fixture.until(|events| {
        events.iter().any(|e| e == "aborted 1")
            && events
                .iter()
                .filter(|e| e.starts_with("active 0 0 0 "))
                .count()
                > 2
    });
    assert_eq!(
        hosts.bindings.iter().next().expect("binding").1.revision(),
        0
    );
    assert_eq!(
        fixture.rollout(2, 2, None).state,
        ApiPipelineRolloutState::Succeeded,
        "next rollout can proceed after abort"
    );
    assert!(fixture.runtime.take_runtime_error().is_none());
}

/// Scenario: after a settings commit, one serving instance releases all work and then fails.
/// Guarantees: qualified recovery keeps discovery alive, preserves the committed revision, and installs ownership before promotion.
#[test]
fn clean_failure_recovers_with_committed_settings() {
    let mut fixture = Fixture::start(2, 4);
    assert_eq!(
        fixture.rollout(2, 1, None).state,
        ApiPipelineRolloutState::Succeeded
    );
    fixture.until(|events| events.iter().any(|e| e.starts_with("ready 0 1 ")));
    let hosts = fixture.hosts();
    hosts
        .retire(
            ParticipantSpec {
                core_id: 0,
                generation: 1,
            },
            Instant::now() + Duration::from_secs(10),
        )
        .expect("joined worker release");
    fixture.fail(0, 1);
    fixture.until(|events| events.iter().any(|e| e.starts_with("active 0 2 1 ")));
    let key = PipelineKey::new("g1".into(), "p1".into());
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut state = fixture.runtime.state.lock().expect("state");
    while !state
        .runtime_recoveries
        .get(&(key.clone(), 0))
        .is_some_and(|r| r.serving_generation == 2 && r.worker_id.is_none())
    {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .expect("recovery promotion deadline");
        (state, _) = fixture
            .runtime
            .state_changed
            .wait_timeout(state, remaining)
            .expect("recovery notification");
    }
    drop(state);
    assert!(Arc::ptr_eq(&hosts, &fixture.hosts()));
    assert_eq!(
        fixture.rollout(2, 2, None).state,
        ApiPipelineRolloutState::Succeeded,
        "recovery must not reopen the completed reconciliation"
    );
    assert!(fixture.runtime.take_runtime_error().is_none());
}

/// Scenario: a serving participant fails with active grants instead of proving worker cleanup.
/// Guarantees: authority fails closed and coordinated shutdown replaces unsafe per-core recovery.
#[test]
fn unclean_failure_stops_fleet_without_recovery() {
    let fixture = Fixture::start(2, 4);
    fixture.fail(0, 0);
    assert!(
        fixture
            .runtime
            .wait_until_all_instances_exit_for(Duration::from_secs(15))
    );
    assert!(fixture.runtime.take_runtime_error().is_some());
    let state = fixture.runtime.state.lock().expect("state");
    assert!(state.global_shutdown_requested);
    assert!(
        state
            .runtime_recoveries
            .values()
            .all(|r| r.worker_id.is_none())
    );
}

/// Scenario: discovery stays empty through resize and a compatible settings replacement.
/// Guarantees: node readiness and lifecycle receipts never depend on a first assignment.
#[test]
fn empty_catalog_reconciles_without_ownership_deadlock() {
    let fixture = Fixture::start(1, 0);
    assert_eq!(
        fixture.rollout(2, 0, None).state,
        ApiPipelineRolloutState::Succeeded
    );
    assert_eq!(
        fixture.rollout(2, 1, None).state,
        ApiPipelineRolloutState::Succeeded
    );
    assert!(fixture.runtime.take_runtime_error().is_none());
}

/// Scenario: a candidate accepts backend authority but installation remains pending when its controller wait expires.
/// Guarantees: rollback aborts the pending handoff before draining the live candidate, restores old work with fresh grants, and releases the rollout reservation.
#[test]
fn rollback_preempts_unfinished_installation_before_candidate_drain() {
    let mut fixture = Fixture::start(1, 4);
    let hosts = fixture.hosts();
    let target = config_with_block(
        &fixture.sink.local_addr().expect("sink").to_string(),
        1,
        1,
        4,
        None,
        Some(0),
    )
    .resolve()
    .pipelines
    .into_iter()
    .find(|p| p.pipeline_id.as_ref() == "p1")
    .expect("target")
    .pipeline;
    let plan = fixture
        .runtime
        .prepare_rollout_plan(
            "g1",
            "p1",
            &ReconfigureRequest {
                pipeline: target,
                step_timeout_secs: 10,
                drain_timeout_secs: 10,
            },
        )
        .expect("replacement plan");
    fixture
        .runtime
        .insert_rollout_plan(&plan)
        .expect("reserve rollout");
    let revisions = hosts
        .prepare(
            &COOPERATIVE_FACTORY,
            &plan.target_deployment.resolved.pipeline,
            Instant::now() + Duration::from_secs(10),
        )
        .expect("prepare revision");
    plan.target_deployment
        .work_coordinator_revisions
        .set(revisions)
        .expect("candidate revisions");
    let _ = fixture
        .runtime
        .launch_regular_pipeline_instance(&plan.target_deployment, 0, 1)
        .expect("candidate launch");
    fixture.until(|events| events.iter().any(|e| e.starts_with("ready 0 1 ")));
    let activation = hosts
        .bindings
        .iter()
        .next()
        .expect("binding")
        .1
        .activate_instance(ParticipantSpec {
            core_id: 0,
            generation: 1,
        })
        .expect("begin handoff");
    fixture.until(|events| events.iter().any(|e| e == "uninstalled 0 1"));
    assert!(
        matches!(
            activation.wait_until(Instant::now()),
            Err(CoordinatorError::Timeout)
        ),
        "expiration is tested at an acknowledged installation barrier"
    );
    let result = fixture.runtime.rollback_replace_rollout(
        &plan,
        &[],
        &[0],
        &[],
        "installation deadline expired".into(),
    );
    assert!(
        matches!(result, Err(RolloutExecutionError::Failed(_))),
        "{result:?}"
    );
    hosts
        .finish(false, Instant::now() + Duration::from_secs(10))
        .expect("abort settled");
    assert!(matches!(
        activation.resolve(),
        Some(Err(CoordinatorError::Stale))
    ));
    assert!(matches!(
        fixture
            .runtime
            .instance_exit(&deployed_key("g1", "p1", 0, 1)),
        Some(RuntimeInstanceExit::Success)
    ));
    fixture
        .runtime
        .update_rollout(&plan.pipeline_key, &plan.rollout.rollout_id, |r| {
            r.state = RolloutLifecycleState::Failed
        });
    fixture
        .runtime
        .finish_rollout(&plan.pipeline_key, &plan.rollout.rollout_id);
    assert!(
        !fixture
            .runtime
            .state
            .lock()
            .expect("state")
            .active_rollouts
            .contains_key(&plan.pipeline_key)
    );
    assert_eq!(
        fixture.rollout(1, 2, None).state,
        ApiPipelineRolloutState::Succeeded
    );
    assert!(fixture.runtime.take_runtime_error().is_none());
}
