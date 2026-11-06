// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use serde_json::json;

fn spec(core_id: usize, generation: u64) -> ParticipantSpec {
    ParticipantSpec {
        core_id,
        generation,
    }
}

async fn cooperative(cores: usize) -> Harness {
    Harness::start_profile(cores, CoordinatorLimits::default(), true).await
}

async fn catalog_at(h: &Harness, revision: u64, metadata: &str) -> Result<(), CoordinatorError> {
    let (tx, rx) = oneshot::channel();
    h.commands
        .send(Command::CatalogAt(
            revision,
            vec![WorkSpec {
                id: WorkId("a".into()),
                work: metadata.into(),
            }],
            tx,
        ))
        .await
        .expect("revision catalog command");
    rx.await.expect("revision catalog receipt")
}

async fn prepare(h: &mut Harness, value: u64) -> (OperationId, u64) {
    let prepared = h
        .binding
        .prepare_reconciliation(Arc::new(json!({"value": value})))
        .expect("prepare settings");
    let CoordinatorLifecycleEvent::Prepare {
        revision,
        settings,
        receipt,
        ..
    } = h.lifecycle.recv().await.expect("provider prepare")
    else {
        panic!("prepare event");
    };
    assert_eq!(settings["value"], value);
    catalog_at(h, revision, &format!("revision-{value}"))
        .await
        .expect("staged catalog");
    receipt.acknowledge(Ok(())).expect("provider prepared");
    assert!(
        matches!(prepared.wait().await, Ok(ReconciliationOutcome::Prepared(r)) if r == revision)
    );
    (prepared.operation(), revision)
}

async fn register(
    h: &Harness,
    instance: ParticipantSpec,
    revision: u64,
) -> WorkParticipant<Targets> {
    let registered = h
        .binding
        .register_instance(instance, revision)
        .expect("register candidate");
    let ReconciliationOutcome::Registered(session) = registered.wait().await.expect("registration")
    else {
        panic!("registered receipt");
    };
    let participant = h
        .binding
        .take::<Targets>(instance.core_id, instance.generation)
        .expect("bind candidate");
    assert_eq!(participant.session(), session);
    participant
        .ready_for_assignments()
        .await
        .expect("candidate ready without ownership");
    participant
}

async fn assignment(
    participant: &mut WorkParticipant<Targets>,
    matches: impl Fn(&AssignmentPhase) -> bool,
) -> WorkAssignment<Targets> {
    loop {
        let current = participant.resynchronize().expect("live assignments");
        if let Some(assignment) = current.assignments.iter().find(|a| matches(&a.phase)) {
            return WorkAssignment {
                work: Arc::clone(&assignment.work),
                phase: assignment.phase.clone(),
            };
        }
        let _ = snapshot(participant).await;
    }
}

async fn install(participant: &mut WorkParticipant<Targets>, metadata: &str) -> AssignmentGrant {
    let offer = assignment(participant, |phase| {
        matches!(phase, AssignmentPhase::Offered(_))
    })
    .await;
    assert_eq!(offer.work.work, metadata);
    let AssignmentPhase::Offered(offer) = offer.phase else {
        unreachable!()
    };
    participant
        .accept(&offer)
        .await
        .expect("accept current offer");
    let active = assignment(participant, |phase| {
        matches!(phase, AssignmentPhase::Active(_))
    })
    .await;
    let AssignmentPhase::Active(grant) = active.phase else {
        unreachable!()
    };
    assert!(
        grant.try_admit().is_err(),
        "installation must precede execution"
    );
    participant
        .activated(&grant)
        .await
        .expect("install current grant");
    grant
}

async fn revoking(participant: &mut WorkParticipant<Targets>) -> AssignmentGrant {
    let assignment = assignment(participant, |phase| {
        matches!(phase, AssignmentPhase::Revoking(_))
    })
    .await;
    let AssignmentPhase::Revoking(grant) = assignment.phase else {
        unreachable!()
    };
    grant
}

/// Scenario: a ready replacement arrives while its predecessor has an admitted scrape.
/// Guarantees: handoff fences admission, retains completion, waits for release, and installs a fresh exact-session term.
#[tokio::test]
async fn handoff_waits_for_release_and_installation() {
    bounded(async {
        let h = cooperative(1).await;
        let mut old = h.participant(0);
        old.ready_for_assignments().await.expect("initial ready");
        h.catalog(&["a"]).await.expect("initial catalog");
        let grant = install(&mut old, "a").await;
        let ticket = grant.try_admit().expect("scrape in flight");
        let prepared = h
            .binding
            .prepare_reconciliation(Arc::new(serde_json::Value::Null))
            .expect("unchanged settings");
        assert!(matches!(
            prepared.wait().await,
            Ok(ReconciliationOutcome::Prepared(0))
        ));
        let mut next = register(&h, spec(0, 8), 0).await;
        assert!(
            next.resynchronize()
                .expect("candidate snapshot")
                .assignments
                .is_empty()
        );
        let transition = h.binding.activate_instance(spec(0, 8)).expect("handoff");
        let retiring = revoking(&mut old).await;
        assert_eq!(retiring.identity(), grant.identity());
        assert!(grant.try_admit().is_err());
        assert!(ticket.is_valid());
        assert!(transition.resolve().is_none());
        assert!(
            next.resynchronize()
                .expect("candidate while draining")
                .assignments
                .is_empty()
        );
        assert!(matches!(
            old.released(&grant).await,
            Err(CoordinatorError::Invalid(_))
        ));
        drop(ticket);
        old.released(&grant)
            .await
            .expect("scrape settled and worker joined");
        let offer = assignment(&mut next, |p| matches!(p, AssignmentPhase::Offered(_))).await;
        assert!(
            transition.resolve().is_none(),
            "an offer is not installed ownership"
        );
        let AssignmentPhase::Offered(offer) = offer.phase else {
            unreachable!()
        };
        next.accept(&offer).await.expect("accept");
        assert!(
            transition.resolve().is_none(),
            "backend activation is not local installation"
        );
        let active = assignment(&mut next, |p| matches!(p, AssignmentPhase::Active(_))).await;
        let AssignmentPhase::Active(active) = active.phase else {
            unreachable!()
        };
        next.activated(&active).await.expect("install");
        assert!(matches!(
            transition.wait().await,
            Ok(ReconciliationOutcome::Activated)
        ));
        assert!(active.identity().term > grant.identity().term);
        assert_ne!(
            active.identity().session.nonce,
            grant.identity().session.nonce
        );
        assert!(matches!(
            next.activated(&grant).await,
            Err(CoordinatorError::Stale)
        ));
        let committed = h
            .binding
            .commit(prepared.operation())
            .expect("commit decision");
        assert!(matches!(
            committed.wait().await,
            Ok(ReconciliationOutcome::Committed)
        ));
        h.binding.begin_shutdown();
        drop((old, next));
        h.finish().await;
    })
    .await;
}

/// Scenario: changed settings hand off a stable work ID, then the rollout aborts.
/// Guarantees: old and candidate metadata stay isolated, abort restores old metadata with a fresh term, and abandoned revisions never recur.
#[tokio::test]
async fn abort_restores_prior_catalog_and_fresh_authority() {
    bounded(async {
        let mut h = cooperative(1).await;
        let mut old = h.participant(0);
        old.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let initial = install(&mut old, "a").await;
        let (operation, revision) = prepare(&mut h, 1).await;
        assert_eq!(
            old.resynchronize().expect("committed view").assignments[0]
                .work
                .work,
            "a"
        );
        assert!(
            initial.try_admit().is_ok(),
            "staging leaves old execution unchanged"
        );
        let mut next = register(&h, spec(0, 8), revision).await;
        let transition = h
            .binding
            .activate_instance(spec(0, 8))
            .expect("activate candidate");
        let _ = revoking(&mut old).await;
        old.released(&initial).await.expect("retire old");
        let candidate = install(&mut next, "revision-1").await;
        let _ = transition.wait().await.expect("handoff complete");
        let abort = h.binding.abort(operation).expect("abort");
        let _ = revoking(&mut next).await;
        assert!(candidate.try_admit().is_err());
        assert!(abort.resolve().is_none());
        assert!(matches!(
            catalog_at(&h, revision, "revision-1").await,
            Err(CoordinatorError::Stale)
        ));
        next.released(&candidate)
            .await
            .expect("candidate worker joined");
        let restored = install(&mut old, "a").await;
        assert!(restored.identity().term > candidate.identity().term);
        let CoordinatorLifecycleEvent::Abort { receipt, .. } =
            h.lifecycle.recv().await.expect("provider abort")
        else {
            panic!("abort event");
        };
        receipt
            .acknowledge(Ok(()))
            .expect("candidate discovery joined");
        assert!(matches!(
            abort.wait().await,
            Ok(ReconciliationOutcome::Aborted)
        ));
        drop(next);
        assert!(h.binding.confirm_instance_exit(spec(0, 8)));
        assert!(matches!(
            old.activated(&initial).await,
            Err(CoordinatorError::Stale)
        ));
        let (operation, next_revision) = prepare(&mut h, 2).await;
        assert!(
            next_revision > revision,
            "an aborted revision is never reused"
        );
        let abort = h.binding.abort(operation).expect("second abort");
        let CoordinatorLifecycleEvent::Abort { receipt, .. } =
            h.lifecycle.recv().await.expect("provider abort")
        else {
            panic!("abort event");
        };
        receipt.acknowledge(Ok(())).expect("cleanup");
        let _ = abort.wait().await.expect("second abort completed");
        h.binding.begin_shutdown();
        drop(old);
        h.finish().await;
    })
    .await;
}

/// Scenario: the controller commits while the provider's cleanup acknowledgement is delayed.
/// Guarantees: timeout leaves a queryable outcome, abort is forbidden after the decision, and obsolete catalog writers cannot affect the winner.
#[tokio::test]
async fn commit_decision_survives_delayed_acknowledgement() {
    bounded(async {
        let mut h = cooperative(1).await;
        let mut old = h.participant(0);
        old.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let initial = install(&mut old, "a").await;
        let (operation, revision) = prepare(&mut h, 1).await;
        let mut next = register(&h, spec(0, 8), revision).await;
        let transition = h.binding.activate_instance(spec(0, 8)).expect("handoff");
        let _ = revoking(&mut old).await;
        old.released(&initial).await.expect("release");
        let _ = install(&mut next, "revision-1").await;
        let _ = transition.wait().await.expect("installed");
        let commit = h.binding.commit(operation).expect("commit decision");
        let CoordinatorLifecycleEvent::Commit { receipt, .. } =
            h.lifecycle.recv().await.expect("provider commit")
        else {
            panic!("commit event");
        };
        assert!(matches!(
            commit.wait_until(std::time::Instant::now()),
            Err(CoordinatorError::Timeout)
        ));
        assert!(commit.resolve().is_none());
        assert!(matches!(
            h.binding.abort(operation),
            Err(CoordinatorError::Invalid(_))
        ));
        assert!(matches!(
            catalog_at(&h, 0, "a").await,
            Err(CoordinatorError::Stale)
        ));
        assert!(
            h.binding
                .prepare_reconciliation(Arc::new(json!({"value": 2})))
                .is_err()
        );
        receipt
            .acknowledge(Ok(()))
            .expect("obsolete discovery joined");
        assert!(matches!(
            commit.wait().await,
            Ok(ReconciliationOutcome::Committed)
        ));
        let replay = h
            .binding
            .commit(operation)
            .expect("query committed operation");
        assert!(matches!(
            replay.resolve(),
            Some(Ok(ReconciliationOutcome::Committed))
        ));
        assert_eq!(h.binding.revision(), revision);
        h.binding.begin_shutdown();
        drop((old, next));
        h.finish().await;
    })
    .await;
}

/// Scenario: a released instance exits and is relaunched under the same core and generation.
/// Guarantees: proven cleanup permits a fresh nonce and grant; old session acknowledgements cannot regain authority.
#[tokio::test]
async fn clean_exit_allows_exact_instance_recovery() {
    bounded(async {
        let h = cooperative(1).await;
        let mut old = h.participant(0);
        old.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let initial = install(&mut old, "a").await;
        let retire = h.binding.retire_instance(spec(0, 7)).expect("retire");
        let _ = revoking(&mut old).await;
        old.released(&initial).await.expect("joined and released");
        let _ = retire.wait().await.expect("released");
        drop(old);
        assert!(h.binding.confirm_instance_exit(spec(0, 7)));
        let mut next = register(&h, spec(0, 7), 0).await;
        let activate = h.binding.activate_instance(spec(0, 7)).expect("recover");
        let recovered = install(&mut next, "a").await;
        let _ = activate.wait().await.expect("recovered ownership");
        assert_ne!(initial.identity().session, recovered.identity().session);
        assert!(recovered.identity().term > initial.identity().term);
        assert!(matches!(
            next.released(&initial).await,
            Err(CoordinatorError::Stale)
        ));
        assert!(!h.binding.status().stopped);
        h.binding.begin_shutdown();
        drop(next);
        h.finish().await;
    })
    .await;
}

/// Scenario: a cooperative participant disappears with an unretired grant and an admitted operation.
/// Guarantees: cleanup cannot be inferred from task exit or ticket drop; the whole domain fails closed instead of recovering unsafely.
#[tokio::test]
async fn unproven_exit_invalidates_cooperative_authority() {
    bounded(async {
        let mut h = cooperative(1).await;
        let mut old = h.participant(0);
        old.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = install(&mut old, "a").await;
        let ticket = grant.try_admit().expect("in flight");
        drop(old);
        assert!(!h.binding.confirm_instance_exit(spec(0, 7)));
        assert!(!ticket.is_valid());
        drop(ticket);
        assert!(h.binding.register_instance(spec(0, 8), 0).is_err());
        assert!(
            h.errors
                .recv()
                .await
                .expect("fatal error")
                .contains("release")
        );
        h.host
            .await
            .expect("host joined")
            .expect("provider cleanup");
    })
    .await;
}

/// Scenario: multiple empty-catalog candidates are abandoned before node construction.
/// Guarantees: abort reclaims unbound registrations so repeated reconciliation cannot exhaust the two-fleet participant budget.
#[tokio::test]
async fn abort_reclaims_unbound_candidates() {
    bounded(async {
        let h = Harness::start_profile(
            1,
            CoordinatorLimits {
                participants: 1,
                ..CoordinatorLimits::default()
            },
            true,
        )
        .await;
        let old = h.participant(0);
        old.ready_for_assignments().await.expect("empty ready");
        for generation in 8..12 {
            let prepare = h
                .binding
                .prepare_reconciliation(Arc::new(serde_json::Value::Null))
                .expect("prepare");
            let _ = prepare.wait().await.expect("prepared");
            let _ = h
                .binding
                .register_instance(spec(0, generation), 0)
                .expect("bounded registration")
                .wait()
                .await
                .expect("registered");
            let abort = h.binding.abort(prepare.operation()).expect("abort");
            let _ = abort.wait().await.expect("aborted");
            assert!(!h.binding.has_pending_instance(spec(0, generation)));
            assert_eq!(h.binding.status().participants, 1);
        }
        h.binding.begin_shutdown();
        drop(old);
        h.finish().await;
    })
    .await;
}

/// Scenario: abort preempts a handoff whose predecessor still has admitted work.
/// Guarantees: activation resolves as stale, abort waits for completion and fresh old-grant installation, and candidate authority is never granted.
#[tokio::test]
async fn abort_preempts_pending_handoff_without_overlapping_ownership() {
    bounded(async {
        let mut h = cooperative(1).await;
        let mut old = h.participant(0);
        old.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let initial = install(&mut old, "a").await;
        let ticket = initial.try_admit().expect("in flight");
        let (operation, revision) = prepare(&mut h, 1).await;
        let mut next = register(&h, spec(0, 8), revision).await;
        let activation = h.binding.activate_instance(spec(0, 8)).expect("activate");
        let _ = revoking(&mut old).await;
        let abort = h.binding.abort(operation).expect("abort pending handoff");
        let CoordinatorLifecycleEvent::Abort { receipt, .. } =
            h.lifecycle.recv().await.expect("abort event")
        else {
            panic!("abort event");
        };
        receipt
            .acknowledge(Ok(()))
            .expect("candidate discovery joined");
        assert!(matches!(
            activation.wait().await,
            Err(CoordinatorError::Stale)
        ));
        assert!(abort.resolve().is_none());
        assert!(ticket.is_valid());
        assert!(
            next.resynchronize()
                .expect("candidate view")
                .assignments
                .is_empty()
        );
        drop(ticket);
        old.released(&initial).await.expect("source joined");
        let restored = install(&mut old, "a").await;
        let _ = abort.wait().await.expect("abort settled");
        assert!(restored.identity().term > initial.identity().term);
        assert!(initial.try_admit().is_err());
        h.binding.begin_shutdown();
        drop((old, next));
        h.finish().await;
    })
    .await;
}

/// Scenario: provider preparation explicitly fails after cleaning up candidate discovery.
/// Guarantees: old work stays executable, abort completes without a second cleanup, and a later preparation can proceed.
#[tokio::test]
async fn failed_preparation_preserves_committed_execution() {
    bounded(async {
        let mut h = cooperative(1).await;
        let mut old = h.participant(0);
        old.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let initial = install(&mut old, "a").await;
        let prepare = h
            .binding
            .prepare_reconciliation(Arc::new(json!({"value": 1})))
            .expect("prepare");
        let CoordinatorLifecycleEvent::Prepare { receipt, .. } =
            h.lifecycle.recv().await.expect("prepare")
        else {
            panic!("prepare event");
        };
        receipt
            .acknowledge(Err(CoordinatorError::Invalid(
                "fixture staging failed and joined",
            )))
            .expect("failed receipt");
        assert!(matches!(
            prepare.wait().await,
            Err(CoordinatorError::Invalid(_))
        ));
        assert!(initial.try_admit().is_ok());
        let abort = h
            .binding
            .abort(prepare.operation())
            .expect("abort failed staging");
        let _ = abort.wait().await.expect("abort complete");
        let (operation, _) = self::prepare(&mut h, 2).await;
        let abort = h.binding.abort(operation).expect("abort next preparation");
        let CoordinatorLifecycleEvent::Abort { receipt, .. } =
            h.lifecycle.recv().await.expect("abort")
        else {
            panic!("abort event");
        };
        receipt.acknowledge(Ok(())).expect("joined");
        let _ = abort.wait().await.expect("complete");
        h.binding.begin_shutdown();
        drop(old);
        h.finish().await;
    })
    .await;
}

/// Scenario: a provider loses its preparation acknowledgement while candidate worker cleanup remains unknown.
/// Guarantees: abort requires an explicit cleanup acknowledgement instead of treating the lost receipt as proven worker termination.
#[tokio::test]
async fn lost_preparation_receipt_requires_abort_cleanup() {
    bounded(async {
        let mut h = cooperative(1).await;
        let old = h.participant(0);
        old.ready_for_assignments().await.expect("empty ready");
        let prepare = h
            .binding
            .prepare_reconciliation(Arc::new(json!({"value": 1})))
            .expect("prepare");
        let CoordinatorLifecycleEvent::Prepare { receipt, .. } =
            h.lifecycle.recv().await.expect("prepare")
        else {
            panic!("prepare event");
        };
        drop(receipt);
        assert!(matches!(
            prepare.wait().await,
            Err(CoordinatorError::Failed(_))
        ));
        let abort = h
            .binding
            .abort(prepare.operation())
            .expect("abort unknown preparation");
        let CoordinatorLifecycleEvent::Abort { receipt, .. } =
            h.lifecycle.recv().await.expect("explicit cleanup")
        else {
            panic!("abort event");
        };
        assert!(abort.resolve().is_none());
        assert!(
            h.binding
                .prepare_reconciliation(Arc::new(json!({"value": 2})))
                .is_err()
        );
        receipt.acknowledge(Ok(())).expect("workers joined");
        let _ = abort.wait().await.expect("abort confirmed");
        h.binding.begin_shutdown();
        drop(old);
        h.finish().await;
    })
    .await;
}

/// Scenario: obsolete discovery cleanup loses its acknowledgement after the commit decision.
/// Guarantees: winning grants stay usable, rollback stays forbidden, and another revision cannot bypass unknown cleanup.
#[tokio::test]
async fn lost_commit_receipt_retains_winner_and_blocks_new_revision() {
    bounded(async {
        let mut h = cooperative(1).await;
        let old = h.participant(0);
        old.ready_for_assignments()
            .await
            .expect("initial empty ready");
        let (operation, revision) = prepare(&mut h, 1).await;
        let mut next = register(&h, spec(0, 8), revision).await;
        let activated = h
            .binding
            .activate_instance(spec(0, 8))
            .expect("activate candidate");
        let grant = install(&mut next, "revision-1").await;
        let _ = activated.wait().await.expect("installed");
        let committed = h.binding.commit(operation).expect("commit decision");
        let CoordinatorLifecycleEvent::Commit { receipt, .. } =
            h.lifecycle.recv().await.expect("commit event")
        else {
            panic!("commit event");
        };
        drop(receipt);
        assert!(matches!(
            committed.wait().await,
            Err(CoordinatorError::Failed(_))
        ));
        assert!(
            grant.try_admit().is_ok(),
            "winning authority is not rolled back"
        );
        assert_eq!(h.binding.revision(), revision);
        assert!(matches!(
            h.binding.abort(operation),
            Err(CoordinatorError::Invalid(_))
        ));
        assert!(
            h.binding
                .prepare_reconciliation(Arc::new(json!({"value": 2})))
                .is_err()
        );
        assert!(matches!(
            h.binding
                .commit(operation)
                .expect("query decision")
                .resolve(),
            Some(Err(CoordinatorError::Failed(_)))
        ));
        assert_eq!(
            h.binding.status().reconciliation,
            Some((operation, ReconciliationPhase::CommitDecided))
        );
        h.binding.begin_shutdown();
        drop((old, next));
        h.finish().await;
    })
    .await;
}
