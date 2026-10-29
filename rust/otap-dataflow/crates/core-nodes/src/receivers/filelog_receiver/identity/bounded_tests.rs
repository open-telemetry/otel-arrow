// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::filelog_receiver::identity::association::{BYTES_PER_STEP, RECORDS_PER_STEP};

fn populate(owner: &mut IdentityOwner, count: usize, width: usize) {
    let mut prefix = vec![b'a'; width];
    prefix[width - 1] = b'b';
    let operations = (1..=count)
        .map(|id| {
            cp::Operation::RegisterFile(cp::RegisterFile {
                file_id: cp::FileId::from_bytes((id as u128).to_be_bytes()),
                file_epoch: 1,
                committed_offset: 0,
                committed_frontier_guard: cp::CommittedFrontierGuard::empty(),
                fingerprint: prefix.clone(),
                ignored_header_bytes: 0,
                locator: cp::Locator::PosixDevIno {
                    dev: 0,
                    ino: id as u64,
                },
                framing_profile_version: cp::FRAMING_PROFILE_VERSION,
                framing_profile_digest: owner.profile,
                framing_resume: cp::FramingResume::Clean,
                last_seen_time_unix_nano: 1,
                advisory_path: cp::AdvisoryPath::from_unix_bytes(
                    format!("/retained/{id}").as_bytes(),
                )
                .unwrap(),
            })
        })
        .collect();
    let _ = owner
        .append_checkpoint(operations, Durability::Sync)
        .unwrap();
}

fn begin(owner: &mut IdentityOwner, path: &Path, budget: &ProbeBudget) -> AssociationScan {
    let metadata = fs::metadata(path).unwrap();
    let bytes = fs::read(path).unwrap();
    let result = owner
        .prepare(
            Candidate {
                path,
                locator: FileLocator::from_metadata(&metadata),
                size: metadata.len(),
                fingerprint: &bytes[..bytes.len().min(owner.fingerprint_bytes)],
                context: Context::Unrelated,
            },
            budget,
            now(),
            || open(path),
            || false,
        )
        .unwrap();
    let Admission::Scanning(scan) = result else {
        panic!("expected scan, got {result:?}")
    };
    scan
}

/// Scenario: A multi-turn association scan interleaves with ordinary Ack progress and shared-probe use.
/// Guarantees: Each turn respects both budgets, releases transient probes, and preserves the final unrelated decision.
#[test]
fn association_yields_to_progress_and_discovery() {
    let f = Fixture::new();
    let inputs = f.inputs(|c| {
        c.limits.max_tracked_files = 256;
        c.identity.fingerprint_bytes = 1000;
        c.start_at = StartAt::End;
        c.identity.on_recovery_mismatch = RecoveryMismatch::Fail;
    });
    let mut owner = f.owner(&inputs);
    populate(&mut owner, 130, 1000);
    let control_path = f.write("control", b"control");
    let control = commit(&mut owner, &control_path, Context::Replacement);
    let path = f.write("candidate", &vec![b'a'; 1000]);
    let budget = ProbeBudget::default();
    let mut scan = begin(&mut owner, &path, &budget);
    assert!(budget.try_reserve().is_some());
    let first = owner.advance_association(&mut scan, || false).unwrap();
    assert_eq!(first.records, RECORDS_PER_STEP);
    assert!(!first.complete);
    progress(&mut owner, &control, b"control", 7);
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&control.file_id)
            .unwrap()
            .committed_offset,
        7
    );
    while !scan.is_complete() {
        let reservation = budget.try_reserve().expect("scan holds no probe");
        let work = owner.advance_association(&mut scan, || false).unwrap();
        assert!(work.records <= RECORDS_PER_STEP && work.bytes <= BYTES_PER_STEP);
        drop(reservation);
    }
    let plan = ready(
        owner
            .finish_prepare(scan, &budget, now(), || open(&path), || false)
            .unwrap(),
    );
    assert!(matches!(
        plan.decision(),
        Decision::Register {
            anchor: Anchor::End,
            reset: false,
            ..
        }
    ));
    drop(plan);
    drop(control);
    assert!(f.registry.is_empty().unwrap());
}

/// Scenario: Late-diverging maximum-width prefixes exhaust the byte budget inside a record comparison.
/// Guarantees: Resumption neither skips bytes nor changes short-prefix association semantics.
#[test]
fn association_resumes_partial_comparisons() {
    let f = Fixture::new();
    let inputs = f.inputs(|c| {
        c.identity.fingerprint_bytes = 65535;
        c.limits.max_tracked_files = 8;
        c.limits.max_pending_candidates = 8;
        c.identity.on_recovery_mismatch = RecoveryMismatch::Fail;
    });
    let mut owner = f.owner(&inputs);
    populate(&mut owner, 3, 65535);
    for (bytes, associated) in [(vec![b'a'; 65535], false), (vec![b'a'; 1], true)] {
        let path = f.write("candidate", &bytes);
        let budget = ProbeBudget::default();
        let mut scan = begin(&mut owner, &path, &budget);
        let mut total = 0;
        let mut steps = 0;
        while !scan.is_complete() {
            let work = owner.advance_association(&mut scan, || false).unwrap();
            assert!(work.records <= RECORDS_PER_STEP && work.bytes <= BYTES_PER_STEP);
            total += work.bytes;
            steps += 1;
        }
        if !associated {
            assert_eq!(total, 3 * 65535);
            assert_eq!(steps, 3);
        }
        let plan = ready(
            owner
                .finish_prepare(scan, &budget, now(), || open(&path), || false)
                .unwrap(),
        );
        assert!(matches!(plan.decision(), Decision::Register { reset, .. } if reset == associated));
        drop(plan);
    }
}

/// Scenario: Source bytes change after a scan, or a waiting scan is cancelled.
/// Guarantees: Stale source evidence cannot authorize admission; cancellation releases ownership when discarded.
#[test]
fn association_revalidates_source_and_cancels() {
    let f = Fixture::new();
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    populate(&mut owner, 1, 16);
    let path = f.write("candidate", b"different prefix");
    let budget = ProbeBudget::default();
    let mut scan = begin(&mut owner, &path, &budget);
    assert!(matches!(
        owner.advance_association(&mut scan, || true),
        Err(IdentityError::Source(FileAccessError::Cancelled))
    ));
    drop(scan);
    assert!(f.registry.is_empty().unwrap());
    let mut scan = begin(&mut owner, &path, &budget);
    while !owner
        .advance_association(&mut scan, || false)
        .unwrap()
        .complete
    {}
    fs::write(&path, b"changed evidence").unwrap();
    assert!(matches!(
        owner
            .finish_prepare(scan, &budget, now(), || open(&path), || false)
            .unwrap(),
        Admission::Deferred
    ));
    assert!(f.registry.is_empty().unwrap());
    assert!(budget.try_reserve().is_some());
    assert_eq!(owner.state().unwrap().records().len(), 1);
}

/// Scenario: Identity metadata or pending-write recovery changes the authoritative table between scan turns.
/// Guarantees: Old cursors are rejected, including after recovery of an ordinary progress transaction.
#[test]
fn association_rejects_changed_table_and_recovered_order() {
    for recover in [false, true] {
        let f = Fixture::new();
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let path = f.write("control", b"control");
        let control = commit(&mut owner, &path, Context::Replacement);
        let path = f.write("candidate", b"different");
        let budget = ProbeBudget::default();
        let mut scan = begin(&mut owner, &path, &budget);
        if recover {
            owner.store.inject_write_failure_after(0);
            let result = owner.append_checkpoint(
                vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                    file_id: control.file_id,
                    expected_committed_offset: 0,
                    expected_file_epoch: 1,
                    new_committed_offset: 7,
                    new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(
                        7, b"control",
                    )
                    .unwrap(),
                    new_framing_resume: cp::FramingResume::Clean,
                    new_last_seen_time_unix_nano: 1,
                    finalize: false,
                })],
                Durability::Sync,
            );
            assert!(matches!(result, Err(IdentityError::PendingWrite(_))));
            let _ = owner.resolve_pending().unwrap();
        } else {
            let _ = owner
                .append_checkpoint(
                    vec![cp::Operation::UpdateMetadata(cp::UpdateMetadata {
                        file_id: control.file_id,
                        expected_prior_state: cp::LifecycleState::Active,
                        expected_file_epoch: 1,
                        last_seen_time_unix_nano: 2,
                        advisory_path: None,
                    })],
                    Durability::Sync,
                )
                .unwrap();
        }
        assert!(matches!(
            owner.advance_association(&mut scan, || false),
            Err(IdentityError::StalePlan)
        ));
        assert!(matches!(
            owner.finish_prepare(scan, &budget, now(), || panic!("stale opener"), || false),
            Err(IdentityError::StalePlan)
        ));
        drop(control);
        assert!(f.registry.is_empty().unwrap());
    }
}

/// Scenario: An already queued prefix is submitted again or extended within its retained capacity.
/// Guarantees: Coalescing and accepted growth preserve the existing allocation rather than replacing it.
#[test]
fn fingerprint_growth_reuses_queued_buffer() {
    let f = Fixture::new();
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let path = f.write("app", b"");
    let reader = commit(&mut owner, &path, Context::Unrelated);
    assert!(owner.queue_fingerprint(&reader, b"a").unwrap());
    owner.growth[0].new_fingerprint.reserve(15);
    let pointer = owner.growth[0].new_fingerprint.as_ptr();
    assert!(owner.queue_fingerprint(&reader, b"a").unwrap());
    assert_eq!(owner.growth[0].new_fingerprint.as_ptr(), pointer);
    assert!(owner.queue_fingerprint(&reader, b"abcd").unwrap());
    assert_eq!(owner.growth[0].new_fingerprint.as_ptr(), pointer);
    assert_eq!(owner.growth[0].new_fingerprint, b"abcd");
}

/// Scenario: Discovery occupies the shared probes after association has completed.
/// Guarantees: Finishing returns the completed scan without reopening or repeating comparisons, retaining its lease until retry.
#[test]
fn completed_association_waits_for_probe_without_rescanning() {
    let f = Fixture::new();
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    populate(&mut owner, 1, 16);
    let path = f.write("candidate", b"different");
    let budget = ProbeBudget::default();
    let mut scan = begin(&mut owner, &path, &budget);
    while !owner
        .advance_association(&mut scan, || false)
        .unwrap()
        .complete
    {}
    let reservation = budget.try_reserve().unwrap();
    let result = owner
        .finish_prepare(scan, &budget, now(), || panic!("busy opener"), || false)
        .unwrap();
    let Admission::Scanning(mut scan) = result else {
        panic!("completed scan discarded")
    };
    assert!(scan.is_complete());
    let work = owner.advance_association(&mut scan, || false).unwrap();
    assert_eq!((work.records, work.bytes), (0, 0));
    assert_eq!(f.registry.len().unwrap(), 1);
    drop(reservation);
    let plan = ready(
        owner
            .finish_prepare(scan, &budget, now(), || open(&path), || false)
            .unwrap(),
    );
    drop(plan);
    assert!(f.registry.is_empty().unwrap());
    assert!(budget.try_reserve().is_some());
}
