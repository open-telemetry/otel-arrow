// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::reconciliation::{install, register, revoking, spec};
use super::*;
use crate::work_coordinator::durable::TestKind;
use std::os::unix::fs::PermissionsExt;

fn options() -> (tempfile::TempDir, DurableOptions) {
    let root = tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o700))
        .tempdir()
        .expect("private durable state root");
    let options = DurableOptions::new(root.path().to_path_buf(), "source-namespace".into());
    (root, options)
}
async fn start(options: DurableOptions) -> Harness {
    Harness::start_authority(1, CoordinatorLimits::default(), true, Some(options)).await
}
fn change(
    ticket: &Arc<AdmissionTicket>,
    source: &str,
    revision: u64,
    data: &[u8],
) -> ProgressMutation {
    ProgressMutation::new(Arc::clone(ticket), source.into(), revision, data.into())
}
async fn offer(participant: &mut WorkParticipant<Targets>, work: &str) -> GrantIdentity {
    loop {
        let snapshot = participant.resynchronize().expect("current assignments");
        if let Some(id) = snapshot.assignments.iter().find_map(|a| match &a.phase {
            AssignmentPhase::Offered(id) if id.work_id.0 == work => Some(id.clone()),
            _ => None,
        }) {
            return id;
        }
        let _ = super::snapshot(participant).await;
    }
}

/// Scenario: a durable host commits several source records, then restarts under the same namespace.
/// Guarantees: progress survives, old tickets fail, historical receipts resolve, and the new epoch resumes with an exact CAS.
#[tokio::test]
async fn durable_host_reopen_preserves_progress_and_fences_old_tickets() {
    bounded(async {
        let (_root, config) = options();
        let h = start(config.clone()).await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = install(&mut p, "a").await;
        let writer = p.progress_writer().expect("durable writer");
        let ticket = Arc::new(grant.try_admit().expect("admitted publication"));
        let committed = writer
            .commit(vec![
                change(&ticket, "file-1", 0, b"100"),
                change(&ticket, "file-2", 0, b"200"),
            ])
            .expect("submit publication");
        let operation = committed.operation();
        assert_eq!(
            committed.wait().await.expect("durable commit"),
            ProgressOutcome::Applied
        );
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
        assert!(!ticket.is_valid());
        assert!(writer.status().closed);
        let h = start(config).await;
        let mut next = h.participant(0);
        next.ready_for_assignments().await.expect("new ready");
        h.catalog(&["a"]).await.expect("recovered catalog");
        let current = install(&mut next, "a").await;
        let resumed = next.progress_writer().expect("new writer");
        assert_eq!(
            resumed
                .resolve(operation)
                .await
                .expect("historical receipt"),
            ProgressOutcome::Applied
        );
        assert_eq!(
            resumed
                .read("file-1".into())
                .await
                .expect("load checkpoint")
                .expect("checkpoint")
                .bytes,
            b"100"
        );
        assert!(matches!(
            resumed.commit(vec![change(&ticket, "file-1", 1, b"bad")]),
            Err(CoordinatorError::Stale)
        ));
        let ticket = Arc::new(current.try_admit().expect("new admitted publication"));
        assert_eq!(
            resumed
                .commit(vec![change(&ticket, "file-1", 1, b"300")])
                .expect("resume")
                .wait()
                .await
                .expect("commit"),
            ProgressOutcome::Applied
        );
        assert_eq!(
            resumed
                .read("file-1".into())
                .await
                .expect("read")
                .expect("checkpoint")
                .revision,
            2
        );
        h.binding.begin_shutdown();
        drop(next);
        h.finish().await;
    })
    .await;
}

/// Scenario: the progress queue is full while storage is paused and a new assignment is accepted.
/// Guarantees: authority retains reserved admission, reconciliation control is serviced, and queued progress remains completion-gated.
#[tokio::test]
async fn saturated_progress_does_not_block_priority_control() {
    bounded(async {
        let (_root, mut config) = options();
        config.queue_items = 1;
        let h = start(config).await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = install(&mut p, "a").await;
        let writer = p.progress_writer().expect("writer");
        let ticket = Arc::new(grant.try_admit().expect("admit"));
        let mut paused = writer.pause_next(TestKind::Commit);
        let first = writer
            .commit(vec![change(&ticket, "file", 0, b"one")])
            .expect("first");
        paused.entered().await;
        let second = writer
            .commit(vec![change(&ticket, "file", 1, b"two")])
            .expect("one queued publication");
        assert!(matches!(
            writer.commit(vec![change(&ticket, "file", 2, b"three")]),
            Err(CoordinatorError::Capacity)
        ));
        assert_eq!(writer.status().progress_items, 1);
        h.catalog(&["a", "b"])
            .await
            .expect("discovery still serviced");
        let pending = offer(&mut p, "b").await;
        let acceptance = tokio::task::spawn_local(async move {
            let result = p.accept(&pending).await;
            (p, result)
        });
        writer.authority_queued().await;
        assert_eq!(
            writer.status().authority_items,
            1,
            "authority is admitted despite a full publication queue"
        );
        let prepare = h
            .binding
            .prepare_reconciliation(Arc::new(serde_json::Value::Null))
            .expect("priority prepare");
        assert!(matches!(
            prepare.wait().await,
            Ok(ReconciliationOutcome::Prepared(0))
        ));
        assert_eq!(
            h.binding.status().active_assignments,
            1,
            "an unresolved activation is not executable"
        );
        paused.release();
        assert_eq!(
            first.wait().await.expect("first commit"),
            ProgressOutcome::Applied
        );
        assert_eq!(
            second.wait().await.expect("second commit"),
            ProgressOutcome::Applied
        );
        let (mut p, accepted) = acceptance.await.expect("acceptance task");
        accepted.expect("durable activation");
        let view = p.resynchronize().expect("active view");
        let active = view
            .assignments
            .iter()
            .find_map(|a| match &a.phase {
                AssignmentPhase::Active(g) if g.identity().work_id.0 == "b" => Some(g.clone()),
                _ => None,
            })
            .expect("new grant");
        p.activated(&active).await.expect("install new grant");
        let abort = h
            .binding
            .abort(prepare.operation())
            .expect("abort preparation");
        let _ = abort.wait().await.expect("unchanged settings restored");
        assert_eq!(
            writer
                .read("file".into())
                .await
                .expect("read")
                .expect("progress")
                .revision,
            2
        );
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
    })
    .await;
}

/// Scenario: activation reaches storage after discovery cancelled and re-added the work.
/// Guarantees: a late effective grant is retired before re-offer; no cancelled term becomes local execution permission.
#[tokio::test]
async fn cancelled_activation_is_retired_before_reoffer() {
    bounded(async {
        let (_root, config) = options();
        let h = start(config).await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let original = offer(&mut p, "a").await;
        let writer = p.progress_writer().expect("writer");
        let mut paused = writer.pause_next(TestKind::Activate);
        let accepted = tokio::task::spawn_local(async move {
            let result = p.accept(&original).await;
            (p, original, result)
        });
        paused.entered().await;
        h.catalog(&[])
            .await
            .expect("cancel offer during storage wait");
        h.catalog(&["a"])
            .await
            .expect("rediscover while outcome is unknown");
        assert_eq!(h.binding.status().active_assignments, 0);
        assert!(
            !accepted.is_finished(),
            "the controlled storage barrier still holds the receipt"
        );
        paused.release();
        let (mut p, original, result) = accepted.await.expect("acceptance joined");
        assert!(matches!(result, Err(CoordinatorError::Stale)));
        let current = install(&mut p, "a").await;
        assert!(current.identity().term > original.term);
        let _ = current
            .try_admit()
            .expect("only fresh installed authority can execute");
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
    })
    .await;
}

/// Scenario: a participant exits before an in-flight activation ever publishes local authority.
/// Guarantees: cleanup retains that session until the late grant is durably retired, then hands work to a fresh registration.
#[tokio::test]
async fn exit_during_activation_cannot_orphan_durable_authority() {
    bounded(async {
        let (_root, config) = options();
        let h = start(config).await;
        let mut old = h.participant(0);
        old.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let pending = offer(&mut old, "a").await;
        let writer = old.progress_writer().expect("writer");
        let mut paused = writer.pause_next(TestKind::Activate);
        let accepting = tokio::task::spawn_local(async move { old.accept(&pending).await });
        paused.entered().await;
        accepting.abort();
        assert!(
            accepting
                .await
                .expect_err("cancelled node task")
                .is_cancelled()
        );
        assert!(
            h.binding.confirm_instance_exit(spec(0, 7)),
            "no executable grant was published"
        );
        let mut next = register(&h, spec(0, 8), 0).await;
        let activated = h
            .binding
            .activate_instance(spec(0, 8))
            .expect("new registration");
        paused.release();
        let _ = install(&mut next, "a").await;
        let _ = activated
            .wait()
            .await
            .expect("late authority retired and successor installed");
        assert!(!h.binding.status().stopped);
        h.binding.begin_shutdown();
        drop(next);
        h.finish().await;
    })
    .await;
}

/// Scenario: a successful authority mutation or publication loses only its reply.
/// Guarantees: authority resolves before installation; publication resolution does not apply source progress twice.
#[tokio::test]
async fn lost_replies_resolve_without_repeating_mutations() {
    bounded(async {
        let (_root, config) = options();
        let h = start(config).await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        let writer = p.progress_writer().expect("writer");
        writer.lose_next_reply(TestKind::Activate);
        h.catalog(&["a"]).await.expect("catalog");
        let grant = install(&mut p, "a").await;
        let ticket = Arc::new(grant.try_admit().expect("admit"));
        writer.lose_next_reply(TestKind::Commit);
        let receipt = writer
            .commit(vec![change(&ticket, "file", 0, b"100")])
            .expect("submit");
        let operation = receipt.operation();
        assert_eq!(
            receipt.wait().await.expect("unknown reply"),
            ProgressOutcome::Unknown
        );
        assert_eq!(
            writer.resolve(operation).await.expect("resolve"),
            ProgressOutcome::Applied
        );
        assert_eq!(
            writer.resolve(operation).await.expect("resolve again"),
            ProgressOutcome::Applied
        );
        assert_eq!(
            writer
                .read("file".into())
                .await
                .expect("read")
                .expect("progress")
                .revision,
            1
        );
        h.catalog(&[]).await.expect("revoke");
        let _ = revoking(&mut p).await;
        assert!(matches!(
            p.released(&grant).await,
            Err(CoordinatorError::Invalid(_))
        ));
        drop(ticket);
        p.released(&grant)
            .await
            .expect("tickets settled before durable retirement");
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
    })
    .await;
}

/// Scenario: the source byte budget cannot admit a publication but authority still has its reserved budget.
/// Guarantees: queue byte pressure rejects before mutation and never prevents grant activation or orderly shutdown.
#[tokio::test]
async fn progress_byte_pressure_cannot_consume_authority_reserve() {
    bounded(async {
        let (_root, mut config) = options();
        config.queue_bytes = 1;
        let h = start(config).await;
        let mut p = h.participant(0);
        p.ready_for_assignments().await.expect("ready");
        h.catalog(&["a"]).await.expect("catalog");
        let grant = install(&mut p, "a").await;
        let ticket = Arc::new(grant.try_admit().expect("admit"));
        let writer = p.progress_writer().expect("writer");
        assert!(matches!(
            writer.commit(vec![change(&ticket, "file", 0, b"one")]),
            Err(CoordinatorError::Capacity)
        ));
        assert_eq!(writer.status().progress_bytes, 0);
        drop(ticket);
        h.catalog(&[]).await.expect("retire under pressure");
        let _ = revoking(&mut p).await;
        p.released(&grant)
            .await
            .expect("reserved authority retirement");
        h.binding.begin_shutdown();
        drop(p);
        h.finish().await;
        assert!(writer.status().closed);
    })
    .await;
}

/// Scenario: declarations request takeover or a source profile before the source adapter is qualified.
/// Guarantees: durable storage alone never enables advanced execution; invalid bounds reject before opening any namespace.
#[test]
fn durable_capabilities_require_explicit_source_qualification() {
    let (root, config) = options();
    let declaration = |options| {
        CoordinatorDeclaration::new::<Targets, _>("targets", || {
            panic!("declaration must not construct a provider")
        })
        .with_reconciliation(serde_json::Value::Null, |old, new| old == new)
        .with_durable_authority(options)
    };
    let mut takeover = config.clone();
    takeover.takeover = TakeoverPolicy::AllowFencedTakeover;
    assert!(declaration(takeover).validate().is_err());
    let mut profile = declaration(config.clone());
    profile.profile = CapabilityProfile::FencedProgress;
    profile.progress = ProgressRequirement::DurableFenced;
    assert!(profile.validate().is_err());
    let mut undersized = config.clone();
    undersized.limits.assignments = 1;
    assert!(declaration(undersized).validate().is_err());
    let before = declaration(config.clone())
        .descriptor()
        .expect("valid metadata");
    let mut changed = config;
    changed.namespace = "different-checkpoint-domain".into();
    assert!(
        before
            .validate_update(&declaration(changed).descriptor().expect("new metadata"))
            .is_err()
    );
    assert_eq!(
        std::fs::read_dir(root.path())
            .expect("root remains empty")
            .count(),
        0
    );
}
