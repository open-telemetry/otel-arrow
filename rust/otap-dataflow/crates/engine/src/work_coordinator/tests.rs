// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use futures::FutureExt;
use std::collections::BTreeSet;
use tokio::{
    sync::{mpsc, oneshot},
    task::LocalSet,
    time::{Duration, timeout},
};

#[derive(Debug)]
struct Targets;
impl WorkProtocol for Targets {
    type Work = String;
    type ParticipantRequest = String;
    type CoordinatorNotice = String;
}

enum Command {
    Catalog(
        Vec<WorkSpec<Targets>>,
        oneshot::Sender<Result<(), CoordinatorError>>,
    ),
    Notice(
        ParticipantSession,
        String,
        oneshot::Sender<Result<(), CoordinatorError>>,
    ),
    Request(oneshot::Sender<(ParticipantSession, String)>),
    Registry(oneshot::Sender<Arc<ParticipantRegistry>>),
    Fail,
}
struct Discovery {
    commands: mpsc::Receiver<Command>,
    initialized: oneshot::Sender<()>,
}
#[async_trait(?Send)]
impl WorkCoordinator<Targets> for Discovery {
    async fn run(
        self: Box<Self>,
        mut runtime: CoordinatorRuntime<Targets>,
    ) -> Result<(), CoordinatorError> {
        let Self {
            mut commands,
            initialized,
        } = *self;
        runtime.ready()?;
        let _ = initialized.send(());
        let mut stopped = runtime.shutdown_signal();
        loop {
            tokio::select! {
                biased;
                _ = stopped.changed() => return Ok(()),
                command = commands.recv() => match command.ok_or(CoordinatorError::Unavailable)? {
                    Command::Catalog(works, tx) => { let _ = tx.send(runtime.replace_catalog(works).await); }
                    Command::Notice(session, notice, tx) => { let _ = tx.send(runtime.notice(session, notice)); }
                    Command::Request(tx) => { let _ = tx.send(runtime.recv().await?); }
                    Command::Registry(tx) => { let _ = tx.send(runtime.participants()); }
                    Command::Fail => return Err(CoordinatorError::Failed("discovery failed".into())),
                }
            }
        }
    }
}

struct Harness {
    binding: CoordinatorBinding,
    commands: mpsc::Sender<Command>,
    errors: mpsc::UnboundedReceiver<String>,
    host: tokio::task::JoinHandle<Result<(), CoordinatorError>>,
}

impl Harness {
    async fn start(cores: usize, limits: CoordinatorLimits) -> Self {
        let (commands, rx) = mpsc::channel(8);
        let (initialized, init_rx) = oneshot::channel();
        let mut declaration = CoordinatorDeclaration::new::<Targets, _>("targets", move || {
            Box::new(Discovery {
                commands: rx,
                initialized,
            })
        });
        declaration.limits = limits;
        let (errors_tx, errors) = mpsc::unbounded_channel();
        let (binding, task) = declaration
            .prepare(
                key(),
                (0..cores).map(session).collect(),
                Arc::new(move |error| {
                    let _ = errors_tx.send(error);
                }),
            )
            .expect("prepare host");
        let (ready, run) = task.into_parts();
        let host = tokio::task::spawn_local(run());
        init_rx.await.expect("provider initialized");
        ready
            .try_recv()
            .expect("explicit startup readiness")
            .expect("ready");
        Self {
            binding,
            commands,
            errors,
            host,
        }
    }
    fn participant(&self, core: usize) -> WorkParticipant<Targets> {
        self.binding
            .take::<Targets>(core, 7)
            .expect("bind participant")
    }
    async fn catalog(&self, ids: &[&str]) -> Result<(), CoordinatorError> {
        let (tx, rx) = oneshot::channel();
        self.commands
            .send(Command::Catalog(works(ids), tx))
            .await
            .expect("catalog command");
        rx.await.expect("catalog receipt")
    }
    async fn finish(self) {
        self.binding.stop();
        self.host
            .await
            .expect("host joined")
            .expect("provider cleanup");
    }
}

fn key() -> CoordinatorKey {
    CoordinatorKey {
        group_id: "group".into(),
        pipeline_id: "pipeline".into(),
        node_id: "source".into(),
        role: "targets".into(),
    }
}
fn session(core_id: usize) -> ParticipantSession {
    ParticipantSession {
        core_id,
        generation: 7,
        nonce: core_id as u64 + 1,
    }
}
fn works(ids: &[&str]) -> Vec<WorkSpec<Targets>> {
    ids.iter()
        .map(|id| WorkSpec {
            id: WorkId((*id).into()),
            work: (*id).into(),
        })
        .collect()
}

async fn bounded(future: impl Future<Output = ()>) {
    timeout(Duration::from_secs(30), LocalSet::new().run_until(future))
        .await
        .expect("coordinator test lifecycle deadline");
}

async fn snapshot(participant: &mut WorkParticipant<Targets>) -> Arc<AssignmentSnapshot<Targets>> {
    timeout(Duration::from_secs(10), async {
        loop {
            if let ParticipantEvent::Assignments(snapshot) =
                participant.recv().await.expect("assignment event")
            {
                return snapshot;
            }
        }
    })
    .await
    .expect("assignment transition deadline")
}

async fn activate(participant: &mut WorkParticipant<Targets>) -> AssignmentGrant {
    let offered = snapshot(participant).await;
    let offer = match &offered.assignments[0].phase {
        AssignmentPhase::Offered(id) => id.clone(),
        _ => panic!("expected offer"),
    };
    participant.accept(&offer).await.expect("accept");
    let active = snapshot(participant).await;
    let grant = match &active.assignments[0].phase {
        AssignmentPhase::Active(grant) => grant.clone(),
        _ => panic!("expected active"),
    };
    assert!(
        grant.try_admit().is_err(),
        "installation acknowledgement precedes admission"
    );
    participant.activated(&grant).await.expect("install grant");
    grant
}

/// Scenario: discovery supplies four targets to a fixed two-instance fleet.
/// Guarantees: readiness grants no ownership; each target executes once on one instance after backend activation.
#[tokio::test]
async fn fixed_fleet_distributes_executable_targets() {
    bounded(async {
        let h = Harness::start(2, CoordinatorLimits::default()).await;
        let mut a = h.participant(0);
        let mut b = h.participant(1);
        h.catalog(&["a", "b", "c", "d"])
            .await
            .expect("catalog accepted");
        a.ready_for_assignments().await.expect("first ready");
        let (tx, rx) = oneshot::channel();
        h.commands
            .send(Command::Registry(tx))
            .await
            .expect("registry command");
        let registry = rx.await.expect("registry response");
        assert_eq!(registry.participants[1].1, ParticipantState::Initializing);
        b.ready_for_assignments().await.expect("second ready");
        let mut executed = BTreeSet::new();
        for participant in [&mut a, &mut b] {
            let offers = snapshot(participant).await;
            assert_eq!(offers.assignments.len(), 2);
            for assignment in &offers.assignments {
                let AssignmentPhase::Offered(id) = &assignment.phase else {
                    panic!("offer required");
                };
                participant.accept(id).await.expect("accept offer");
            }
            let active = snapshot(participant).await;
            for assignment in &active.assignments {
                let AssignmentPhase::Active(grant) = &assignment.phase else {
                    panic!("active required");
                };
                participant.activated(grant).await.expect("activate");
                let ticket = grant.try_admit().expect("admit simulated scrape");
                assert!(ticket.is_valid());
                assert!(executed.insert(assignment.work.work.clone()));
            }
        }
        assert_eq!(
            executed,
            BTreeSet::from(["a".into(), "b".into(), "c".into(), "d".into()])
        );
        h.binding.begin_shutdown();
        drop((a, b));
        h.finish().await;
    })
    .await;
}

/// Scenario: a target disappears while an admitted scrape still awaits completion.
/// Guarantees: revocation stops new work, release waits for tickets, and re-addition uses a fresh term.
#[tokio::test]
async fn removal_waits_for_completion_and_rejects_stale_terms() {
    bounded(async {
        let h = Harness::start(1, CoordinatorLimits::default()).await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = activate(&mut p).await;
        let ticket = grant.try_admit().expect("scrape admitted");
        h.catalog(&[]).await.expect("remove target");
        assert!(grant.try_admit().is_err());
        assert!(
            ticket.is_valid(),
            "completion path survives cooperative revocation"
        );
        assert!(matches!(
            p.released(&grant).await,
            Err(CoordinatorError::Invalid(_))
        ));
        drop(ticket);
        p.released(&grant).await.expect("settled and joined");
        h.catalog(&["a"]).await.expect("rediscover");
        assert!(matches!(
            p.accept(grant.identity()).await,
            Err(CoordinatorError::Stale)
        ));
        let next = activate(&mut p).await;
        assert!(next.identity().term > grant.identity().term);
        assert!(grant.try_admit().is_err());
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
    })
    .await;
}

/// Scenario: custom request/notice queues are saturated during catalog removal.
/// Guarantees: bounded messages fail explicitly while revocation, release and shutdown remain serviceable.
#[tokio::test]
async fn saturated_custom_queues_cannot_block_control() {
    bounded(async {
        let h = Harness::start(
            1,
            CoordinatorLimits {
                message_items: 1,
                message_bytes: 256,
                ..CoordinatorLimits::default()
            },
        )
        .await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = activate(&mut p).await;
        p.request("first".into()).expect("request enqueued");
        assert!(matches!(
            p.request("second".into()),
            Err(CoordinatorError::Capacity)
        ));
        let (tx, rx) = oneshot::channel();
        h.commands
            .send(Command::Notice(p.session(), "notice".into(), tx))
            .await
            .expect("notice command");
        rx.await.expect("notice response").expect("notice queued");
        let (tx, rx) = oneshot::channel();
        h.commands
            .send(Command::Notice(p.session(), "full".into(), tx))
            .await
            .expect("notice command");
        assert!(matches!(
            rx.await.expect("notice response"),
            Err(CoordinatorError::Capacity)
        ));
        h.catalog(&[]).await.expect("removal is independent");
        p.released(&grant).await.expect("release independent");
        let (tx, rx) = oneshot::channel();
        h.commands
            .send(Command::Request(tx))
            .await
            .expect("drain request");
        assert_eq!(rx.await.expect("request"), (p.session(), "first".into()));
        assert!(matches!(
            p.request("x".repeat(1024)),
            Err(CoordinatorError::Capacity)
        ));
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
    })
    .await;
}

/// Scenario: a bound instance disappears while another instance holds authority.
/// Guarantees: the whole authority domain fails closed once and old tickets cannot remain valid.
#[tokio::test]
async fn unexpected_exit_invalidates_every_participant() {
    bounded(async {
        let mut h = Harness::start(2, CoordinatorLimits::default()).await;
        let mut a = h.participant(0);
        let b = h.participant(1);
        a.ready_for_assignments().await.expect("ready");
        b.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = activate(&mut a).await;
        let ticket = grant.try_admit().expect("admitted");
        drop(b);
        assert!(!ticket.is_valid());
        assert!(matches!(
            grant.try_admit(),
            Err(CoordinatorError::Unavailable)
        ));
        assert!(a.recv().await.is_err());
        assert!(
            h.errors
                .recv()
                .await
                .expect("fatal failure")
                .contains("participant")
        );
        drop(a);
        h.host
            .await
            .expect("host joined")
            .expect("provider shutdown");
        assert!(h.errors.try_recv().is_err(), "failure reported only once");
    })
    .await;
}

/// Scenario: discovery fails after signaling readiness.
/// Guarantees: host exit invalidates authority and reports a fatal failure rather than restarting discovery.
#[tokio::test]
async fn provider_failure_invalidates_authority() {
    bounded(async {
        let mut h = Harness::start(1, CoordinatorLimits::default()).await;
        let mut p = h.participant(0);
        h.commands
            .send(Command::Fail)
            .await
            .expect("fail discovery");
        assert!(h.host.await.expect("joined").is_err());
        assert!(p.recv().await.is_err());
        assert!(h.errors.recv().await.is_some());
        h.binding.begin_shutdown();
    })
    .await;
}

/// Scenario: empty discovery, duplicate initialization, and obsolete generation binding.
/// Guarantees: idle readiness succeeds and binding never substitutes a core ID for an exact session.
#[tokio::test]
async fn empty_catalog_and_binding_validation() {
    bounded(async {
        let h = Harness::start(1, CoordinatorLimits::default()).await;
        assert!(matches!(
            h.binding.take::<Targets>(0, 6),
            Err(CoordinatorError::Stale)
        ));
        let p = h.participant(0);
        assert!(h.binding.take::<Targets>(0, 7).is_err());
        p.ready_for_assignments()
            .await
            .expect("ready without ownership");
        h.catalog(&[]).await.expect("empty catalog");
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
    })
    .await;
}

/// Scenario: a churn update exceeds the retained union or changes live work metadata.
/// Guarantees: rejection leaves the previous catalog and its executable grant intact.
#[tokio::test]
async fn rejected_catalog_is_atomic() {
    bounded(async {
        let h = Harness::start(
            1,
            CoordinatorLimits {
                catalog_items: 1,
                ..CoordinatorLimits::default()
            },
        )
        .await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = activate(&mut p).await;
        assert!(matches!(
            h.catalog(&["b"]).await,
            Err(CoordinatorError::Capacity)
        ));
        let (tx, rx) = oneshot::channel();
        h.commands
            .send(Command::Catalog(
                vec![WorkSpec {
                    id: WorkId("a".into()),
                    work: "changed".into(),
                }],
                tx,
            ))
            .await
            .expect("changed metadata");
        assert!(matches!(
            rx.await.expect("receipt"),
            Err(CoordinatorError::Invalid(_))
        ));
        assert!(grant.try_admit().is_ok());
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
    })
    .await;
}

/// Scenario: a node requests a lifecycle or progress capability unavailable in Basic.
/// Guarantees: admission fails before the provider factory can run or silently downgrade.
#[test]
fn missing_capabilities_fail_preparation() {
    let mut declaration = CoordinatorDeclaration::new::<Targets, _>("targets", || {
        panic!("must not construct provider")
    });
    declaration.profile = CapabilityProfile::Cooperative;
    assert!(declaration.validate().is_err());
    declaration.profile = CapabilityProfile::Basic;
    declaration.progress = ProgressRequirement::DurableFenced;
    assert!(declaration.validate().is_err());
}

/// Scenario: backend callers replay an operation, change its payload, or retire a different session's grant.
/// Guarantees: receipts are idempotent only for the exact command and conflicting ownership is never confirmed.
#[tokio::test]
async fn backend_receipts_and_ownership_are_exact() {
    let mut backend = InMemoryBackend::new(1);
    let grant = GrantIdentity {
        incarnation: uuid::Uuid::new_v4(),
        work_id: WorkId("target".into()),
        session: session(0),
        revision: 0,
        term: 1,
    };
    assert_eq!(
        backend
            .activate(OperationId(1), &grant)
            .await
            .expect("activate"),
        BackendOutcome::Confirmed
    );
    assert_eq!(
        backend
            .activate(OperationId(1), &grant)
            .await
            .expect("replay"),
        BackendOutcome::Confirmed
    );
    let mut other = grant.clone();
    other.session = session(1);
    other.term = 2;
    assert_eq!(
        backend
            .activate(OperationId(1), &other)
            .await
            .expect("changed replay"),
        BackendOutcome::Rejected
    );
    assert_eq!(
        backend
            .retire(OperationId(2), &other)
            .await
            .expect("stale retire"),
        BackendOutcome::Rejected
    );
    assert_eq!(
        backend
            .activate(OperationId(3), &other)
            .await
            .expect("overlap"),
        BackendOutcome::Rejected
    );
    assert_eq!(
        backend
            .retire(OperationId(4), &grant)
            .await
            .expect("retire"),
        BackendOutcome::Confirmed
    );
    assert_eq!(
        backend
            .retire(OperationId(4), &grant)
            .await
            .expect("retire replay"),
        BackendOutcome::Confirmed
    );
    assert_eq!(
        backend
            .activate(OperationId(5), &other)
            .await
            .expect("successor"),
        BackendOutcome::Confirmed
    );
    assert_eq!(
        backend.resolve(OperationId(5)).await.expect("last outcome"),
        BackendOutcome::Confirmed
    );
    assert_eq!(
        backend.resolve(OperationId(1)).await.expect("old receipt"),
        BackendOutcome::Unknown
    );
    assert_eq!(
        backend
            .activate(OperationId(1), &grant)
            .await
            .expect("obsolete operation"),
        BackendOutcome::Rejected
    );
}

/// Scenario: controller shutdown races with a late grant-installation acknowledgement.
/// Guarantees: installation cannot reopen admission after the logical pipeline begins draining.
#[tokio::test]
async fn shutdown_cannot_be_undone_by_late_activation() {
    bounded(async {
        let h = Harness::start(1, CoordinatorLimits::default()).await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = activate(&mut p).await;
        h.binding.begin_shutdown();
        assert!(matches!(
            p.activated(&grant).await,
            Err(CoordinatorError::Unavailable)
        ));
        assert!(grant.try_admit().is_err());
        drop(p);
        h.finish().await;
    })
    .await;
}

/// Scenario: the final shutdown signal arrives while a provider's return branch is being polled.
/// Guarantees: acknowledged shutdown completion succeeds, while the same early return without shutdown still fails closed.
#[tokio::test]
async fn shutdown_during_provider_poll_is_classified_from_current_state() {
    struct ReturningProvider {
        polled: std::sync::mpsc::SyncSender<()>,
        released: std::sync::mpsc::Receiver<()>,
    }
    #[async_trait(?Send)]
    impl WorkCoordinator<Targets> for ReturningProvider {
        async fn run(
            self: Box<Self>,
            mut runtime: CoordinatorRuntime<Targets>,
        ) -> Result<(), CoordinatorError> {
            runtime.ready()?;
            self.polled
                .send(())
                .expect("controller observes provider poll");
            // This deliberately blocks one poll to let a real controller
            // thread signal shutdown after select's shutdown branch was pending.
            self.released
                .recv_timeout(Duration::from_secs(30))
                .expect("controller releases provider poll");
            Ok(())
        }
    }
    for stop in [true, false] {
        let (polled, observed) = std::sync::mpsc::sync_channel(1);
        let (release, released) = std::sync::mpsc::sync_channel(1);
        let declaration = CoordinatorDeclaration::new::<Targets, _>("targets", move || {
            Box::new(ReturningProvider { polled, released })
        });
        let (binding, task) = declaration
            .prepare(
                CoordinatorKey {
                    group_id: "g".into(),
                    pipeline_id: "p".into(),
                    node_id: "input".into(),
                    role: "targets".into(),
                },
                vec![ParticipantSession {
                    core_id: 0,
                    generation: 0,
                    nonce: 1,
                }],
                Arc::new(|_| {}),
            )
            .expect("prepare returning provider");
        let controller = std::thread::spawn(move || {
            observed
                .recv_timeout(Duration::from_secs(30))
                .expect("provider enters controlled poll");
            if stop {
                binding.stop();
            }
            release.send(()).expect("release provider");
        });
        let (_ready, run) = task.into_parts();
        let result = std::panic::AssertUnwindSafe(run()).catch_unwind().await;
        controller.join().expect("controller joined");
        let result = result.unwrap_or_else(|panic| std::panic::resume_unwind(panic));
        if stop {
            result.expect("shutdown observed during provider poll");
        } else {
            assert!(matches!(result, Err(CoordinatorError::Failed(_))));
        }
    }
}

/// Scenario: shutdown occurs before a worker subscribes to authority loss.
/// Guarantees: late subscriptions still wake immediately, so provider cleanup cannot wait for a second shutdown signal.
#[tokio::test]
async fn late_subscribers_observe_authority_loss() {
    bounded(async {
        let h = Harness::start(1, CoordinatorLimits::default()).await;
        let p = h.participant(0);
        h.binding.stop();
        let mut host_loss = h.binding.authority_loss();
        let mut worker_loss = p.authority_loss();
        host_loss.changed().await.expect("host loss visible");
        worker_loss.changed().await.expect("worker loss visible");
        assert!(*host_loss.borrow());
        assert!(*worker_loss.borrow());
        h.finish().await;
    })
    .await;
}
