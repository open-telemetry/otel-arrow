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
