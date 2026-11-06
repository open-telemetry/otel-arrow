// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::{HostState, PipelineCoordinators};
use super::*;
use crate::engine_context;
use async_trait::async_trait;
use otel_arrow_dfe_config::settings::telemetry::logs::LogLevel;
use otel_arrow_dfe_engine::work_coordinator::{
    CoordinatorBindings, CoordinatorDeclaration, CoordinatorError, CoordinatorRuntime,
    ParticipantSession, WorkCoordinator, WorkProtocol,
};
use otel_arrow_dfe_telemetry::tracing_init::ProviderSetup;
#[cfg(not(windows))]
use std::cell::RefCell;
use std::sync::mpsc;

const DEADLINE: Duration = Duration::from_secs(30);

struct Protocol;
impl WorkProtocol for Protocol {
    type Work = ();
    type ParticipantRequest = ();
    type CoordinatorNotice = ();
}

struct Gate {
    released: Mutex<bool>,
    changed: Condvar,
    entered: mpsc::SyncSender<()>,
}
impl Gate {
    fn wait(&self) {
        self.entered
            .send(())
            .expect("fixture observes blocked cleanup");
        let mut released = self.released.lock().expect("gate");
        while !*released {
            released = self.changed.wait(released).expect("gate wake");
        }
    }
    fn release(&self) {
        *self.released.lock().expect("gate") = true;
        self.changed.notify_all();
    }
}

#[cfg(not(windows))]
struct Finalizer(Arc<Gate>);
#[cfg(not(windows))]
impl Drop for Finalizer {
    fn drop(&mut self) {
        self.0.wait();
    }
}

struct UnblockOnPanic<'a>(&'a Gate);
impl Drop for UnblockOnPanic<'_> {
    fn drop(&mut self) {
        if thread::panicking() {
            self.0.release();
        }
    }
}
#[cfg(not(windows))]
thread_local! {
    // Only initialized on the fixture's private provider thread, never on the
    // test harness or a shared runtime. This models destruction after task exit.
    static FINALIZER: RefCell<Option<Finalizer>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    BlockCleanup,
    LateError,
    #[cfg(not(windows))]
    BlockTls,
    Error,
    Panic,
    BeforeReady,
}
struct Provider {
    mode: Mode,
    gate: Arc<Gate>,
}
#[async_trait(?Send)]
impl WorkCoordinator<Protocol> for Provider {
    async fn run(
        self: Box<Self>,
        mut runtime: CoordinatorRuntime<Protocol>,
    ) -> Result<(), CoordinatorError> {
        if matches!(self.mode, Mode::BeforeReady) {
            self.gate.wait();
        }
        #[cfg(not(windows))]
        if matches!(self.mode, Mode::BlockTls) {
            FINALIZER.with(|slot| *slot.borrow_mut() = Some(Finalizer(Arc::clone(&self.gate))));
        }
        runtime.ready()?;
        runtime.shutdown_requested().await;
        match self.mode {
            Mode::BlockCleanup | Mode::LateError => {
                self.gate.wait();
                if matches!(self.mode, Mode::LateError) {
                    return Err(CoordinatorError::Failed(
                        "late cleanup fixture failure".into(),
                    ));
                }
            }
            Mode::Error => return Err(CoordinatorError::Failed("cleanup fixture failure".into())),
            Mode::Panic => panic!("cleanup fixture panic"),
            _ => {}
        }
        Ok(())
    }
}

fn key(node: &'static str) -> CoordinatorKey {
    CoordinatorKey {
        group_id: "g".into(),
        pipeline_id: "p".into(),
        node_id: node.into(),
        role: "targets".into(),
    }
}

struct Fixture {
    supervisor: Arc<HostSupervisor>,
    owner: Option<PipelineCoordinators>,
    host: Arc<Host>,
    gate: Arc<Gate>,
    entered: mpsc::Receiver<()>,
    binding: CoordinatorBinding,
    cleanup_on_drop: bool,
}
impl Fixture {
    fn new(mode: Mode) -> Self {
        let supervisor = Arc::new(HostSupervisor::new(1));
        let host = supervisor
            .reserve(vec![key("node")])
            .expect("reserve")
            .pop()
            .expect("host");
        let (entered_tx, entered) = mpsc::sync_channel(1);
        let gate = Arc::new(Gate {
            released: Mutex::new(false),
            changed: Condvar::new(),
            entered: entered_tx,
        });
        let provider_gate = Arc::clone(&gate);
        let declaration = CoordinatorDeclaration::new::<Protocol, _>("targets", move || {
            Box::new(Provider {
                mode,
                gate: provider_gate,
            })
        });
        let (binding, task) = declaration
            .prepare(
                key("node"),
                vec![ParticipantSession {
                    core_id: 0,
                    generation: 0,
                    nonce: 1,
                }],
                Arc::new(|_| {}),
            )
            .expect("prepare");
        let (ready, run) = task.into_parts();
        let mut bindings = CoordinatorBindings::default();
        bindings
            .insert("node".into(), "targets".into(), binding.clone())
            .expect("binding");
        let owner = PipelineCoordinators {
            bindings: Arc::new(bindings),
            hosts: Mutex::new(HostState {
                threads: vec![Arc::clone(&host)],
                failure: None,
            }),
            _supervisor: Arc::clone(&supervisor),
            descriptors: Mutex::default(),
            pending: Mutex::default(),
        };
        let fixture = Self {
            supervisor,
            owner: Some(owner),
            host,
            gate,
            entered,
            binding,
            cleanup_on_drop: true,
        };
        fixture
            .host
            .start(
                fixture.binding.clone(),
                TracingSetup::new(ProviderSetup::Noop, LogLevel::default(), engine_context),
                move || async move { run().await.map_err(failure) },
            )
            .expect("start");
        if matches!(mode, Mode::BeforeReady) {
            fixture.blocked();
        } else {
            ready
                .recv_timeout(DEADLINE)
                .expect("provider readiness before deadline")
                .expect("ready");
        }
        fixture
    }
    fn blocked(&self) {
        self.entered
            .recv_timeout(DEADLINE)
            .expect("provider reached explicit blocking boundary");
    }
    fn stop(&self) {
        self.host.begin_shutdown().expect("cleanup worker starts");
    }
    fn joined(&self) {
        let _ = self.host.wait_until(Instant::now() + DEADLINE);
        assert!(
            self.host.collect(),
            "actual provider and cleanup worker joins finished"
        );
    }

    fn expired_shutdown(&self) -> String {
        let owner = self.owner.as_ref().expect("owner");
        thread::scope(|scope| {
            // Release before scoped-thread joining if a blocking regression
            // prevents the controller from observing its expired deadline.
            let _unblock = UnblockOnPanic(&self.gate);
            let (tx, rx) = mpsc::sync_channel(1);
            let worker = scope.spawn(move || {
                tx.send(owner.shutdown_until(Instant::now()))
                    .expect("observer");
            });
            let result = rx
                .recv_timeout(DEADLINE)
                .expect("controller returns while native cleanup is blocked");
            worker.join().expect("controller observer joined");
            result.expect_err("cleanup deadline elapsed").to_string()
        })
    }

    fn abandon(&mut self) {
        let owner = self.owner.take().expect("owner");
        thread::scope(|scope| {
            let _unblock = UnblockOnPanic(&self.gate);
            let (tx, rx) = mpsc::sync_channel(1);
            let worker = scope.spawn(move || {
                drop(owner);
                tx.send(()).expect("observer");
            });
            rx.recv_timeout(DEADLINE)
                .expect("deployment Drop returns before blocked provider termination");
            worker.join().expect("deployment Drop observer joined");
        });
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.cleanup_on_drop {
            return;
        }
        self.gate.release();
        let _ = self.host.begin_shutdown();
        let _ = self.host.wait_until(Instant::now() + DEADLINE);
        if !thread::panicking() {
            assert!(
                self.host.collect(),
                "fixture joins all native work before destruction"
            );
        }
    }
}

/// Scenario: a provider cooperates with shutdown and its host is shut down repeatedly.
/// Guarantees: native cleanup completes, capacity is released, and success remains idempotent.
#[test]
fn cooperative_shutdown_joins_and_releases_capacity() {
    let fixture = Fixture::new(Mode::Normal);
    let owner = fixture.owner.as_ref().expect("owner");
    owner.shutdown().expect("joined");
    owner
        .shutdown_until(Instant::now())
        .expect("repeat success");
    assert_eq!(fixture.supervisor.budget.available_permits(), 1);
    assert!(fixture.binding.status().stopped);
}

/// Scenario: provider cleanup ignores shutdown while the controller deadline has elapsed.
/// Guarantees: callers return an error, authority stops, keys and permits remain reserved, and retry retains failure after late cleanup.
#[test]
fn timeout_retains_ownership_and_error_until_late_join() {
    let fixture = Fixture::new(Mode::BlockCleanup);
    fixture.stop();
    fixture.blocked();
    let owner = fixture.owner.as_ref().expect("owner");
    let error = fixture.expired_shutdown();
    assert!(error.contains("cleanup timed out"));
    assert!(fixture.binding.status().stopped);
    assert!(fixture.supervisor.reserve(vec![key("node")]).is_err());
    assert!(fixture.supervisor.reserve(vec![key("other")]).is_err());
    fixture.gate.release();
    fixture.joined();
    assert_eq!(
        owner.shutdown().expect_err("failure retained").to_string(),
        error
    );
    assert!(
        fixture.supervisor.reserve(vec![key("node")]).is_ok(),
        "identity is reusable only after actual join"
    );
}

// Windows forbids blocking in TLS destruction under the loader lock; ordinary
// blocked provider cleanup is covered there by the portable deadline tests.
/// Scenario: a provider returns but its native thread-local destructor remains blocked.
/// Guarantees: task completion never substitutes for native termination, and a stalled destructor cannot block the controller's deadline path.
#[cfg(not(windows))]
#[test]
fn native_thread_destructors_are_inside_the_supervised_join() {
    let fixture = Fixture::new(Mode::BlockTls);
    fixture.stop();
    fixture.blocked();
    let _ = fixture.expired_shutdown();
    assert_eq!(fixture.supervisor.budget.available_permits(), 0);
    assert!(!fixture.host.collect());
    fixture.gate.release();
    fixture.joined();
    assert_eq!(fixture.supervisor.budget.available_permits(), 1);
}

/// Scenario: a timed-out provider eventually terminates with a cleanup error.
/// Guarantees: the timeout and subsequent provider failure are both retained without accumulating duplicate error text on retries.
#[test]
fn late_cleanup_failure_preserves_both_causes() {
    let fixture = Fixture::new(Mode::LateError);
    fixture.stop();
    fixture.blocked();
    assert!(fixture.expired_shutdown().contains("cleanup timed out"));
    fixture.gate.release();
    fixture.joined();
    let owner = fixture.owner.as_ref().expect("owner");
    let error = owner
        .shutdown()
        .expect_err("both failures retained")
        .to_string();
    assert!(error.contains("cleanup timed out"));
    assert!(error.contains("late cleanup fixture failure"));
    assert_eq!(
        owner.shutdown().expect_err("stable retry").to_string(),
        error
    );
}

/// Scenario: partial startup is abandoned while a provider has not signaled readiness.
/// Guarantees: deployment Drop stays nonblocking, invalidates authority, and leaves cleanup owned and capacity reserved in the engine supervisor.
#[test]
fn abandoned_startup_retains_its_unready_host() {
    let mut fixture = Fixture::new(Mode::BeforeReady);
    fixture.abandon();
    assert!(fixture.binding.status().stopped);
    assert_eq!(fixture.supervisor.budget.available_permits(), 0);
    assert!(fixture.supervisor.reserve(vec![key("node")]).is_err());
    fixture.gate.release();
    fixture.joined();
    assert_eq!(fixture.supervisor.budget.available_permits(), 1);
}

/// Scenario: a provider fails or panics while handling final shutdown.
/// Guarantees: the original failure category remains observable on repeated shutdown after its native thread was joined.
#[test]
fn cleanup_errors_and_panics_remain_observable() {
    for (mode, expected) in [
        (Mode::Error, "cleanup fixture failure"),
        (Mode::Panic, "cleanup fixture panic"),
    ] {
        let fixture = Fixture::new(mode);
        let owner = fixture.owner.as_ref().expect("owner");
        let error = owner.shutdown().expect_err("cleanup failed").to_string();
        assert!(error.contains(expected), "{error}");
        assert_eq!(
            owner
                .shutdown()
                .expect_err("same retained failure")
                .to_string(),
            error
        );
        assert_eq!(fixture.supervisor.budget.available_permits(), 1);
    }
}

/// Scenario: a fleet reservation exceeds capacity or repeats an identity.
/// Guarantees: admission is all-or-none and rejects without leaking any reservation or permit.
#[test]
fn fleet_reservation_failure_is_atomic() {
    let supervisor = HostSupervisor::new(1);
    assert!(supervisor.reserve(vec![key("a"), key("b")]).is_err());
    assert!(supervisor.reserve(vec![key("a"), key("a")]).is_err());
    assert_eq!(supervisor.budget.available_permits(), 1);
    let hosts = supervisor
        .reserve(vec![key("a")])
        .expect("reserve after rejection");
    hosts[0]
        .begin_shutdown()
        .expect("unspawned reservation can finish");
    hosts[0]
        .wait_until(Instant::now())
        .expect("no work to join");
    assert_eq!(supervisor.budget.available_permits(), 1);
}

/// Scenario: an engine and deployment are dropped while native provider cleanup is still blocked.
/// Guarantees: Drop returns without joining the provider on the caller; the cleanup worker alone retains its binding and permit until physical termination.
#[test]
fn cleanup_ownership_outlives_engine_and_deployment_drop() {
    struct CleanupGuard {
        gate: Arc<Gate>,
        host: std::sync::Weak<Host>,
    }
    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            self.gate.release();
            if let Some(host) = self.host.upgrade() {
                let _ = host.wait_until(Instant::now() + DEADLINE);
            }
        }
    }
    let mut fixture = Fixture::new(Mode::BlockCleanup);
    fixture.stop();
    fixture.blocked();
    let guard = CleanupGuard {
        gate: Arc::clone(&fixture.gate),
        host: Arc::downgrade(&fixture.host),
    };
    let budget = Arc::clone(&fixture.supervisor.budget);
    fixture.cleanup_on_drop = false;
    thread::scope(|scope| {
        let _unblock = UnblockOnPanic(&guard.gate);
        let (tx, rx) = mpsc::sync_channel(1);
        let worker = scope.spawn(move || {
            drop(fixture);
            tx.send(()).expect("observer");
        });
        rx.recv_timeout(DEADLINE)
            .expect("engine Drop returns while provider cleanup remains blocked");
        worker.join().expect("engine Drop observer joined");
    });
    let host = guard
        .host
        .upgrade()
        .expect("cleanup worker retains ownership without any engine/deployment owner");
    assert_eq!(budget.available_permits(), 0);
    guard.gate.release();
    let _ = host.wait_until(Instant::now() + DEADLINE);
    assert!(host.collect());
    assert_eq!(budget.available_permits(), 1);
}
