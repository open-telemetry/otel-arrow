// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::super::identity::{decision::Decision, lease::LeaseRegistry};
use super::*;

fn lifecycle_sink(
    s: &Scheduler<FakeIo>,
    now: Instant,
) -> batch::Batcher<impl Fn() -> Instant + use<>> {
    let mut config = super::super::super::config::Config {
        include: vec!["/logs/*.log".into()],
        ..Default::default()
    };
    config.limits.max_tracked_files = s.limits.readers as u64;
    config.limits.max_open_files = s.limits.descriptors as u64;
    config.framing.max_line_bytes = 128;
    config.framing.max_record_bytes = 128;
    config.framing.max_log_size_behavior = OversizeBehavior::Truncate;
    config.batch.max_bytes = 300;
    batcher_with_config(s, Rc::new(Cell::new(now)), config)
}

fn register(
    s: &Scheduler<FakeIo>,
    sink: &mut batch::Batcher<impl Fn() -> Instant>,
    key: ReaderKey,
) {
    let r = s.readers[key.0 as usize].as_ref().unwrap();
    sink.register(
        key,
        r.file_id,
        r.epoch,
        r.evidence.applied_offset,
        r.resume,
        &cp::AdvisoryPath::unavailable(),
    )
    .unwrap();
}

fn seed(lease: LeaseGuard, serial: u128) -> ReaderSeed {
    let file_id = cp::FileId::from_bytes(serial.to_le_bytes());
    ReaderSeed {
        decision: Decision::Resume(file_id),
        file_id,
        epoch: 7,
        offset: 0,
        resume: cp::FramingResume::Clean,
        frontier: cp::CommittedFrontierWindow::empty(),
        fingerprint: Vec::new(),
        lease,
    }
}

fn record(s: &Scheduler<FakeIo>, key: ReaderKey) -> cp::SnapshotRecord {
    let r = s.readers[key.0 as usize].as_ref().unwrap();
    cp::SnapshotRecord {
        file_id: r.file_id,
        file_epoch: r.epoch,
        committed_offset: r.window.end,
        committed_frontier_guard: r.window.guard(),
        fingerprint: Vec::new(),
        ignored_header_bytes: 0,
        locator: cp::Locator::PosixDevIno {
            dev: r.evidence.locator.device,
            ino: r.evidence.locator.inode,
        },
        framing_profile_version: 1,
        framing_profile_digest: [0; 32],
        framing_resume: cp::FramingResume::Clean,
        lifecycle_state: cp::LifecycleState::Active,
        quarantine_evidence: None,
        last_seen_time_unix_nano: 0,
        advisory_path: cp::AdvisoryPath::unavailable(),
    }
}

/// Scenario: A one-reader population repeatedly retires and admits different identities.
/// Guarantees: Scheduler and batch registration capacity are reusable, and the lease remains held until composition releases it.
#[test]
fn sequential_retirement_reuses_capacity_and_preserves_lease() {
    let now = Instant::now();
    let mut s = scheduler(&[b""], 8, 32, 1, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    s.retire(ReaderKey(0, 0), &mut sink)
        .unwrap()
        .release()
        .unwrap();
    let registry = Arc::new(LeaseRegistry::default());
    let owner = registry.owner("lifecycle", 1).unwrap();
    let locator = s.io.files[0].locator;
    for serial in 2..34 {
        let key = s
            .admit(seed(owner.acquire(locator).unwrap(), serial))
            .unwrap();
        register(&s, &mut sink, key);
        let turn = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
        assert!(matches!(turn.reason, Reason::Eof), "{turn:?}");
        let lease = s.retire(key, &mut sink).unwrap();
        assert_eq!(s.io.opens.get(), 0);
        assert_eq!(s.slots, 0);
        assert_eq!(s.next_deadline(permission(), &sink), None);
        assert!(owner.acquire(locator).is_err());
        lease.release().unwrap();
        assert!(registry.is_empty().unwrap());
        assert_eq!(s.readers.len(), 1);
        assert_eq!(s.free.len(), 1);
        assert!(matches!(
            s.turn(now, permission(), &mut sink, Waker::noop(), || false)
                .reason,
            Reason::Idle
        ));
    }
}

/// Scenario: Old controls and output arrive after a slot is reused by another identity.
/// Guarantees: Stale controls, metadata updates and late output cannot mutate, unregister or poison the replacement reader.
#[test]
fn stale_keys_cannot_affect_reused_reader() {
    let now = Instant::now();
    let mut s = scheduler(&[b""], 8, 32, 1, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    let old = ReaderKey(0, 0);
    let lease = s.retire(old, &mut sink).unwrap();
    let new = s.admit(seed(lease, 42)).unwrap();
    assert_ne!(new, old);
    register(&s, &mut sink, new);
    let current = record(&s, new);
    assert_eq!(s.pin(old), Err(LifecycleRefusal::UnknownReader));
    assert_eq!(s.unpin(old), Err(LifecycleRefusal::UnknownReader));
    assert_eq!(
        s.retire(old, &mut sink).unwrap_err(),
        LifecycleRefusal::UnknownReader
    );
    assert_eq!(s.observe_applied(old, &current), Err(Fault::SourceContract));
    assert!(
        sink.update_metadata(old, &cp::AdvisoryPath::unavailable())
            .is_err()
    );
    assert!(sink.unregister(old, current.file_id, 7, 0).is_err());
    let mut other = scheduler(&[b"old\n"], 8, 32, 1, 1, FramingConfig::Line(line()));
    let mut collector = Sink::default();
    frames(&mut other, &mut collector, 1, now);
    let rejected = sink.offer(collector.frames.remove(0)).err().unwrap();
    assert_eq!(rejected.output.key, old);
    assert_eq!(body(&rejected.output), b"old");
    assert!(sink.failure().is_none());
    s.io.files[0].bytes.extend_from_slice(b"new\n");
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    sink.builder.seal().unwrap();
    assert_eq!(
        sink.builder.retained().unwrap().progress[0].file_id,
        current.file_id
    );
    assert_eq!(sink.builder.retained().unwrap().record_count, 1);
}

/// Scenario: A reader has clean framing but open or retained batch progress, followed by applied release.
/// Guarantees: Either component's veto leaves both registrations retryable, and retirement succeeds only after both are resolved.
#[test]
fn retirement_waits_for_open_retained_and_applied_progress() {
    let now = Instant::now();
    let mut s = scheduler(&[b"a\n"], 8, 32, 1, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    let key = ReaderKey(0, 0);
    for _ in 0..2 {
        let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    }
    let applied = record(&s, key);
    assert!(s.retire(key, &mut sink).is_err());
    assert!(sink.unregister(key, applied.file_id, 7, 2).is_err());
    assert!(sink.unresolved(key, 7));
    // Simulate authority notification arriving before the batch's release.
    s.observe_applied(key, &applied).unwrap();
    assert_eq!(
        s.retire(key, &mut sink).unwrap_err(),
        LifecycleRefusal::Batch(super::super::super::batch::BatchError::Stopped)
    );
    sink.builder.seal().unwrap();
    assert!(s.retire(key, &mut sink).is_err());
    let progress = sink.builder.retained().unwrap().progress.clone();
    assert_eq!(sink.builder.retained().unwrap().record_count, 1);
    sink.builder.release_applied(&progress, now).unwrap();
    s.retire(key, &mut sink).unwrap().release().unwrap();
    assert_eq!(s.io.opens.get(), 0);
}

/// Scenario: One file fills a batch and a second file exists only in carry-over.
/// Guarantees: Retirement reaches the carry-over veto even with early applied evidence, and promotion keeps the veto until release.
#[test]
fn retirement_rejects_carry_over_through_promotion() {
    let now = Instant::now();
    let second_source = [vec![b'x'; 128], b"\n".to_vec()].concat();
    let mut s = scheduler(
        &[b"0123456789\n", &second_source],
        256,
        1024,
        2,
        2,
        FramingConfig::Line(line()),
    );
    let mut sink = lifecycle_sink(&s, now);
    let key = ReaderKey(1, 0);
    for _ in 0..2 {
        let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    }
    let applied = record(&s, key);
    let first = sink.builder.retained().unwrap().progress.clone();
    assert_eq!(first.len(), 1);
    assert_ne!(first[0].file_id, applied.file_id);
    assert_eq!(first[0].new_committed_offset, 11);
    assert_eq!(applied.committed_offset, 129);
    assert_eq!(
        s.retire(key, &mut sink).unwrap_err(),
        LifecycleRefusal::UnappliedProgress
    );
    // Inject early authority notification to isolate the batcher's independent veto.
    // This reader has no open/retained progress: only carry-over can block it.
    s.observe_applied(key, &applied).unwrap();
    assert_eq!(
        s.retire(key, &mut sink).unwrap_err(),
        LifecycleRefusal::Batch(super::super::super::batch::BatchError::Stopped)
    );
    sink.builder.release_applied(&first, now).unwrap();
    assert_eq!(
        s.retire(key, &mut sink).unwrap_err(),
        LifecycleRefusal::Batch(super::super::super::batch::BatchError::Stopped)
    );
    sink.builder.seal().unwrap();
    let second = sink.builder.retained().unwrap().progress.clone();
    assert_eq!(second[0].file_id, applied.file_id);
    assert_eq!(second[0].new_committed_offset, 129);
    assert_eq!(
        s.retire(key, &mut sink).unwrap_err(),
        LifecycleRefusal::Batch(super::super::super::batch::BatchError::Stopped)
    );
    sink.builder.release_applied(&second, now).unwrap();
    s.retire(key, &mut sink).unwrap().release().unwrap();
}

/// Scenario: Composition admits a reader but omits its batch registration.
/// Guarantees: The next source turn fails terminally before I/O instead of silently pausing forever.
#[test]
fn missing_batch_registration_fails_closed() {
    let now = Instant::now();
    let mut s = scheduler(&[b"new\n"], 32, 64, 1, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    let lease = s.retire(ReaderKey(0, 0), &mut sink).unwrap();
    let key = s.admit(seed(lease, 99)).unwrap();
    // Deliberately omit register for this live key.
    let turn = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert_eq!(turn.key, Some(key));
    assert!(matches!(turn.reason, Reason::Terminal(Fault::SinkRejected)));
    assert_eq!(
        sink.failure(),
        Some(&super::super::super::batch::BatchError::Progress)
    );
    assert_eq!(s.io.opens.get(), 0);
    assert_eq!(s.io.reads, 0);
    s.resume_sink();
    assert!(matches!(
        s.turn(now, permission(), &mut sink, Waker::noop(), || false)
            .reason,
        Reason::Terminal(Fault::SinkRejected)
    ));
}

/// Scenario: A reader reaches EOF holding an incomplete line or multiline record.
/// Guarantees: Retirement preserves input ownership and reservations, and completing the record remains possible.
#[test]
fn retirement_preserves_unfinished_framing() {
    for framing in [
        FramingConfig::Line(line()),
        FramingConfig::Multiline(
            MultilineConfig {
                line: line(),
                max_lines: 10,
                mode: GroupingMode::End,
            },
            BoundaryPattern::compile("END$", PatternMode::Text).unwrap(),
        ),
    ] {
        let now = Instant::now();
        let mut s = scheduler(&[b"unfinished"], 32, 64, 1, 1, framing);
        let mut sink = lifecycle_sink(&s, now);
        for _ in 0..2 {
            let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
        }
        assert_eq!(
            s.retire(ReaderKey(0, 0), &mut sink).unwrap_err(),
            LifecycleRefusal::Unfinished
        );
        assert_eq!(s.slots, 1);
        assert_eq!(s.io.opens.get(), 1);
        s.io.files[0].bytes.extend_from_slice(b"END\n");
        let at = s.next_deadline(permission(), &sink).unwrap();
        let _ = s.turn(at, permission(), &mut sink, Waker::noop(), || false);
        sink.builder.seal().unwrap();
        assert_eq!(sink.builder.retained().unwrap().record_count, 1);
        assert_eq!(
            sink.builder.retained().unwrap().progress[0].new_committed_offset,
            14
        );
    }
}

/// Scenario: A sink rejects an owned frame and the scheduler stops.
/// Guarantees: Retirement cannot discard the rejected body, its reservation or the matching batch registration.
#[test]
fn retirement_preserves_rejected_output() {
    let now = Instant::now();
    let mut s = scheduler(&[b"owned\n"], 32, 64, 1, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    let mut rejecting = Sink {
        reject: true,
        ..Default::default()
    };
    let turn = s.turn(now, permission(), &mut rejecting, Waker::noop(), || false);
    assert!(matches!(turn.reason, Reason::Terminal(Fault::SinkRejected)));
    assert_eq!(
        s.retire(ReaderKey(0, 0), &mut sink).unwrap_err(),
        LifecycleRefusal::RejectedOutput
    );
    assert_eq!(body(s.rejected.as_ref().unwrap()), b"owned");
    assert_eq!(s.slots, 1);
    assert_eq!(s.io.opens.get(), 1);
    assert!(
        sink.update_metadata(ReaderKey(0, 0), &cp::AdvisoryPath::unavailable())
            .is_ok()
    );
}

/// Scenario: An idle resident is pinned while a second file needs its descriptor.
/// Guarantees: Pinning vetoes eviction and retirement, and unpinning lets the waiting reader progress.
#[test]
fn pin_unpin_preserves_eviction_behavior() {
    let now = Instant::now();
    let mut s = scheduler(&[b"", b"peer\n"], 32, 64, 2, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    assert_eq!(s.pin(ReaderKey(0, 0)), Err(LifecycleRefusal::NotResident));
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    s.pin(ReaderKey(0, 0)).unwrap();
    assert_eq!(
        s.retire(ReaderKey(0, 0), &mut sink).unwrap_err(),
        LifecycleRefusal::Pinned
    );
    let turn = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(matches!(turn.reason, Reason::Descriptors), "{turn:?}");
    assert!(s.readers[0].as_ref().unwrap().handle.is_some());
    s.unpin(ReaderKey(0, 0)).unwrap();
    for _ in 0..8 {
        let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    }
    assert!(s.readers[0].as_ref().unwrap().handle.is_none());
    assert!(sink.unresolved(ReaderKey(1, 0), 7));
}

/// Scenario: Source validation halts a clean reader before any source bytes are consumed.
/// Guarantees: Retirement releases its descriptor while preserving the lease for composition and allowing a peer to progress.
#[test]
fn clean_halted_reader_can_retire() {
    let now = Instant::now();
    let mut s = scheduler(&[b"", b"peer\n"], 32, 64, 1, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    s.io.files[0].locator.inode += 10;
    let turn = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(matches!(
        turn.reason,
        Reason::Transition(Transition::Replaced)
    ));
    let lease = s.retire(ReaderKey(0, 0), &mut sink).unwrap();
    assert_eq!(lease.locator().inode, 1);
    assert_eq!(s.io.opens.get(), 0);
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(sink.unresolved(ReaderKey(1, 0), 7));
    lease.release().unwrap();
}

/// Scenario: A slot reaches its final representable generation and retires.
/// Guarantees: Admission fails with the intact seed instead of wrapping to a stale key.
#[test]
fn generation_exhaustion_fails_closed() {
    let now = Instant::now();
    let mut s = scheduler(&[b""], 8, 32, 1, 1, FramingConfig::Line(line()));
    // Fault injection at the otherwise unreachable counter boundary.
    let key = ReaderKey(0, u64::MAX);
    s.readers[0].as_mut().unwrap().key = key;
    let mut sink = lifecycle_sink(&s, now);
    let lease = s.retire(key, &mut sink).unwrap();
    let refused = s.admit(seed(lease, 42)).unwrap_err();
    assert_eq!(refused.reason, SeedRefusal::Capacity);
    assert_eq!(
        refused.seed.file_id,
        cp::FileId::from_bytes(42u128.to_le_bytes())
    );
    assert!(s.free.is_empty());
    refused.seed.lease.release().unwrap();
}

/// Scenario: A pending or notified probe owner, or a deferred peer, retires and its slot is reused.
/// Guarantees: The old request never opens the new reader, discovery priority is released when appropriate, and remaining readers progress.
#[test]
fn retirement_cancels_probe_references_before_reuse() {
    for owner in [true, false] {
        for notified in [true, false] {
            let now = Instant::now();
            let mut s = scheduler(&[b"", b"peer\n"], 32, 32, 2, 2, FramingConfig::Line(line()));
            let mut sink = lifecycle_sink(&s, now);
            let budget = Arc::clone(&s.probe_budget);
            let mut held = Some(budget.try_reserve().unwrap());
            for _ in 0..2 {
                assert!(matches!(
                    s.turn(now, permission(), &mut sink, Waker::noop(), || false)
                        .reason,
                    Reason::Probe
                ));
            }
            if notified {
                drop(held.take());
                s.probe_available();
            }
            let old = ReaderKey(u64::from(!owner), 0);
            let lease = s.retire(old, &mut sink).unwrap();
            drop(held.take());
            if owner {
                assert!(budget.try_reserve().is_some());
            } else {
                assert!(budget.try_reserve().is_none());
            }
            let new = s.admit(seed(lease, 99)).unwrap();
            s.io.files[new.0 as usize].bytes = b"new\n".to_vec();
            register(&s, &mut sink, new);
            s.probe_available();
            for _ in 0..12 {
                let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
            }
            assert!(sink.unresolved(new, 7));
            if owner {
                assert!(sink.unresolved(ReaderKey(1, 0), 7));
            }
            assert!(budget.try_reserve().is_some());
            sink.builder.seal().unwrap();
            assert_eq!(
                sink.builder.retained().unwrap().record_count,
                if owner { 2 } else { 1 }
            );
        }
    }
}

/// Scenario: A clean reader retires during a yielded descriptor-selection walk with a partial-slot grant.
/// Guarantees: Its grant and selection are removed before reuse, without evicting the pinned peer or starving the replacement.
#[test]
fn retirement_cancels_selection_and_releases_grant() {
    let now = Instant::now();
    let mut s = scheduler(&[b"", b"new\n"], 32, 32, 2, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    s.pin(ReaderKey(0, 0)).unwrap();
    s.limits.steps = 2;
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(s.selecting.is_some());
    assert_eq!(s.slots, 1);
    let lease = s.retire(ReaderKey(1, 0), &mut sink).unwrap();
    assert_eq!(s.slots, 0);
    let new = s.admit(seed(lease, 99)).unwrap();
    register(&s, &mut sink, new);
    s.unpin(ReaderKey(0, 0)).unwrap();
    for _ in 0..16 {
        let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    }
    assert!(sink.unresolved(new, 7));
    assert_eq!(s.io.opens.get(), 1);
}

/// Scenario: A reader awaiting partial capacity retires while a peer retains an unfinished line.
/// Guarantees: Reuse cannot leave duplicate FIFO entries or steal the peer's reservation, and both readers subsequently produce output.
#[test]
fn retirement_removes_capacity_waiter_before_reuse() {
    let now = Instant::now();
    let mut s = scheduler(
        &[b"unfinished", b"new\n"],
        32,
        32,
        1,
        1,
        FramingConfig::Line(line()),
    );
    let mut sink = lifecycle_sink(&s, now);
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    assert!(matches!(
        s.turn(now, permission(), &mut sink, Waker::noop(), || false)
            .reason,
        Reason::Capacity
    ));
    let lease = s.retire(ReaderKey(1, 0), &mut sink).unwrap();
    assert_eq!(s.slots, 1);
    let new = s.admit(seed(lease, 99)).unwrap();
    register(&s, &mut sink, new);
    s.io.files[0].bytes.extend_from_slice(b"\n");
    for _ in 0..24 {
        let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    }
    // Descriptor pressure may have sealed the peer's batch. Apply it before resuming.
    if let Some(batch) = sink.builder.retained() {
        let progress = batch.progress.clone();
        sink.builder.release_applied(&progress, now).unwrap();
        s.resume_sink();
    }
    for _ in 0..24 {
        let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    }
    assert!(sink.unresolved(new, 7));
    assert!(sink.failure().is_none());
}

/// Scenario: Batcher registration disagrees with an otherwise eligible scheduler reader.
/// Guarantees: Failed retirement retains the descriptor, deadline and lease, and succeeds after the registration is corrected.
#[test]
fn retirement_batch_refusal_is_atomic_and_retryable() {
    let now = Instant::now();
    let mut s = scheduler(&[b""], 8, 32, 1, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    let key = ReaderKey(0, 0);
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    let original = s.readers[0].as_ref().unwrap().file_id;
    // Inject a composition registration mismatch without changing resource state.
    s.readers[0].as_mut().unwrap().file_id = cp::FileId::from_bytes([42; 16]);
    let deadline = s.next_deadline(permission(), &sink);
    assert!(matches!(
        s.retire(key, &mut sink),
        Err(LifecycleRefusal::Batch(_))
    ));
    assert_eq!(s.io.opens.get(), 1);
    assert_eq!(s.next_deadline(permission(), &sink), deadline);
    assert!(s.free.is_empty());
    assert!(
        sink.update_metadata(key, &cp::AdvisoryPath::unavailable())
            .is_ok()
    );
    s.readers[0].as_mut().unwrap().file_id = original;
    s.retire(key, &mut sink).unwrap().release().unwrap();
    assert_eq!(s.io.opens.get(), 0);
    assert_eq!(s.next_deadline(permission(), &sink), None);
}

/// Scenario: Recovery admits a continuation before any partial framer is allocated.
/// Guarantees: Empty allocation state is not mistaken for completed input, and retirement preserves both registrations.
#[test]
fn retirement_rejects_unstarted_continuation_recovery() {
    let now = Instant::now();
    let mut s = scheduler(&[b"abcdef\n"], 8, 32, 1, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    let lease = s.retire(ReaderKey(0, 0), &mut sink).unwrap();
    let mut recovery = seed(lease, 99);
    recovery.offset = 4;
    recovery.frontier = cp::CommittedFrontierWindow::new(4, b"abcd").unwrap();
    recovery.resume = cp::FramingResume::Continuation {
        record_start_offset: 0,
        record_end_offset: 0,
        next_fragment_index: 1,
    };
    let key = s.admit(recovery).unwrap();
    register(&s, &mut sink, key);
    assert_eq!(s.slots, 0);
    assert_eq!(
        s.retire(key, &mut sink).unwrap_err(),
        LifecycleRefusal::Unfinished
    );
    assert!(
        sink.update_metadata(key, &cp::AdvisoryPath::unavailable())
            .is_ok()
    );
    assert!(s.free.is_empty());
}

/// Scenario: A descriptor-blocked reader retires while its peer remains pinned.
/// Guarantees: Resource notifications cannot resurrect the old key, and the replacement progresses after unpinning.
#[test]
fn retirement_removes_descriptor_blocked_reader() {
    let now = Instant::now();
    let mut s = scheduler(&[b"", b"new\n"], 32, 32, 2, 1, FramingConfig::Line(line()));
    let mut sink = lifecycle_sink(&s, now);
    let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    s.pin(ReaderKey(0, 0)).unwrap();
    assert!(matches!(
        s.turn(now, permission(), &mut sink, Waker::noop(), || false)
            .reason,
        Reason::Descriptors
    ));
    let lease = s.retire(ReaderKey(1, 0), &mut sink).unwrap();
    let new = s.admit(seed(lease, 99)).unwrap();
    register(&s, &mut sink, new);
    s.unpin(ReaderKey(0, 0)).unwrap();
    for _ in 0..8 {
        let _ = s.turn(now, permission(), &mut sink, Waker::noop(), || false);
    }
    sink.builder.seal().unwrap();
    assert_eq!(sink.builder.retained().unwrap().record_count, 1);
    assert_eq!(
        sink.builder.retained().unwrap().progress[0].file_id,
        cp::FileId::from_bytes(99u128.to_le_bytes())
    );
    assert!(sink.failure().is_none());
}

/// Scenario: Repeated unpin calls target an already-unpinned resident while eviction yields behind pinned readers.
/// Guarantees: Redundant calls preserve the scan cursor so the waiting reader reaches an eligible victim and emits output.
#[test]
fn redundant_unpin_preserves_yielded_eviction_progress() {
    let now = Instant::now();
    let mut s = scheduler(
        &[b"", b"", b"", b"new\n"],
        32,
        64,
        4,
        3,
        FramingConfig::Line(line()),
    );
    let mut sink = Sink::default();
    for _ in 0..3 {
        assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Eof));
    }
    s.pin(ReaderKey(0, 0)).unwrap();
    s.pin(ReaderKey(1, 0)).unwrap();
    s.limits.steps = 2;
    let turn = tick(&mut s, &mut sink, now);
    assert!(matches!(turn.reason, Reason::Progress));
    let selection = s.selecting.expect("eviction walk yielded");
    assert_eq!(selection.0, 3);
    assert!(selection.1.is_some());
    s.unpin(ReaderKey(2, 0)).unwrap();
    assert_eq!(s.selecting, Some(selection));
    for _ in 0..32 {
        s.unpin(ReaderKey(2, 0)).unwrap();
        let _ = tick(&mut s, &mut sink, now);
        if !sink.frames.is_empty() {
            break;
        }
    }
    assert_eq!(sink.frames.len(), 1);
    assert_eq!(sink.frames[0].key, ReaderKey(3, 0));
    assert_eq!(body(&sink.frames[0]), b"new");
    assert!(s.readers[0].as_ref().unwrap().handle.is_some());
    assert!(s.readers[1].as_ref().unwrap().handle.is_some());
    assert!(s.readers[2].as_ref().unwrap().handle.is_none());
}

/// Scenario: A descriptor scan yields after the oldest pinned resident, then a resident is unpinned.
/// Guarantees: Pin changes ahead of the cursor cannot starve the waiter, and the resident at the cursor is reconsidered if unpinned.
#[test]
fn unpin_during_eviction_reaches_the_newly_eligible_resident() {
    for victim in [2, 0] {
        let now = Instant::now();
        let mut s = scheduler(
            &[b"", b"", b"", b"new\n"],
            32,
            64,
            4,
            3,
            FramingConfig::Line(line()),
        );
        let mut sink = Sink::default();
        for id in 0..3 {
            assert!(matches!(tick(&mut s, &mut sink, now).reason, Reason::Eof));
            s.pin(ReaderKey(id, 0)).unwrap();
        }
        s.limits.steps = 2;
        let _ = tick(&mut s, &mut sink, now);
        assert!(matches!(s.selecting, Some((3, Some((_, 0))))));
        let key = ReaderKey(victim as u64, 0);
        s.unpin(key).unwrap();
        for _ in 0..32 {
            let _ = tick(&mut s, &mut sink, now);
            if !sink.frames.is_empty() {
                break;
            }
            if victim == 2 && s.readers[victim].as_ref().unwrap().handle.is_some() {
                s.pin(key).unwrap();
                s.unpin(key).unwrap();
            }
        }
        assert_eq!(sink.frames.len(), 1);
        assert_eq!(sink.frames[0].key, ReaderKey(3, 0));
        assert_eq!(body(&sink.frames[0]), b"new");
        for id in 0..3 {
            assert_eq!(
                s.readers[id].as_ref().unwrap().handle.is_some(),
                id != victim
            );
        }
    }
}
