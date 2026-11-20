// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Included by identity's tests to use its real source, lease and checkpoint fixtures.
use super::*;
use crate::receivers::filelog_receiver::{
    delivery::{CheckpointOutcome, Delivery, SendOutcome},
    discovery::{
        Observation, ScanCompletion, ScanReason,
        reconcile::{BindingRole, ReconcileLimits, Reconciler, TrackedFile, TrackedId},
    },
    scheduler::{
        FramingConfig, LifecycleRefusal, Permission, ReaderIo, ReaderKey, Reason, Scheduler,
        Transition, batch::Batcher, io::LinuxIo,
    },
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    task::Waker,
    time::Instant,
};

const READ: Permission = Permission {
    read: true,
    process: true,
    memory: true,
};
const TRACKED: TrackedId = TrackedId(1);

fn admit(
    scheduler: &mut Scheduler<impl ReaderIo>,
    sink: &mut Batcher<impl Fn() -> Instant>,
    paths: &RefCell<HashMap<ReaderKey, PathBuf>>,
    path: &Path,
    seed: ReaderSeed,
) -> ReaderKey {
    let (id, epoch, offset, resume) = (seed.file_id, seed.epoch, seed.offset, seed.resume);
    let key = scheduler.admit(seed).unwrap();
    sink.register(
        key,
        id,
        epoch,
        offset,
        resume,
        &cp::AdvisoryPath::unavailable(),
    )
    .unwrap();
    assert!(paths.borrow_mut().insert(key, path.into()).is_none());
    key
}

fn read_to_eof(
    scheduler: &mut Scheduler<impl ReaderIo>,
    sink: &mut Batcher<impl Fn() -> Instant>,
    key: ReaderKey,
    now: Instant,
) {
    for _ in 0..16 {
        let turn = scheduler.turn(now, READ, sink, Waker::noop(), || false);
        assert_eq!(turn.key, Some(key));
        match turn.reason {
            Reason::Eof => return,
            Reason::Progress => {}
            other => panic!("unexpected turn: {other:?}"),
        }
    }
    panic!("reader did not reach EOF");
}

fn acknowledge(delivery: &mut Delivery<Batcher<impl Fn() -> Instant>>, now: Instant) {
    let correlation = delivery.next_attempt(now).unwrap().unwrap().correlation;
    delivery
        .sent(correlation, SendOutcome::Accepted, now)
        .unwrap();
    assert!(
        delivery
            .complete(&correlation.calldata(), true, now)
            .unwrap()
    );
}

fn finish_pass(
    scan: &mut Reconciler,
    scheduler: &mut Scheduler<impl ReaderIo>,
    key: ReaderKey,
    generation: u64,
    observations: u64,
    complete: bool,
) {
    let summary = scan
        .finish(
            ScanCompletion {
                generation,
                observations,
                complete,
                cancelled: false,
            },
            Duration::from_secs(generation),
            |event| {
                scheduler
                    .disappearance(key)
                    .unwrap()
                    .observe(TRACKED, &event)
            },
        )
        .unwrap();
    assert_eq!(summary.complete, complete);
}

/// Scenario: A reader commits one line, keeps the next unfinished, and loses its evicted file while peer output awaits Ack.
/// Guarantees: Only current complete absence and applied delivery permit release; capacity is reusable and the Active checkpoint reconnects at its exact offset.
#[test]
fn closed_handle_disappearance_preserves_applied_checkpoint() {
    let f = Fixture::new();
    let line = FramingConfig::Line(f.inputs(|_| {}).line());
    let inputs = f.inputs(|c| {
        c.reader.eof_reprobe_interval = Duration::from_secs(120);
        c.limits.max_open_files = 1;
        c.limits.max_tracked_files = 3;
        c.limits.max_partial_state_bytes = 2 * line
            .slot_bytes(c.limits.max_read_bytes_per_turn as usize)
            .unwrap() as u64;
    });
    let mut owner = f.owner(&inputs);
    let now = Instant::now();
    let paths = Rc::new(RefCell::new(HashMap::<ReaderKey, PathBuf>::new()));
    let names = Rc::clone(&paths);
    let io = LinuxIo(move |key: ReaderKey, _: &mut dyn FnMut() -> bool| {
        open(&names.borrow()[&key]).map_err(Into::into)
    });
    let mut scheduler = Scheduler::from_inputs(io, &inputs, 128, Arc::default(), now).unwrap();
    let clock = Cell::new(now);
    let mut delivery = Delivery::new(Batcher::new(&inputs, || clock.get(), || 1), &inputs).unwrap();
    let app = f.write("app.log", b"complete line\nunfinished line");
    let seed = commit(&mut owner, &app, Context::Unrelated);
    let id = seed.file_id;
    let locator = seed.lease.locator();
    let key = admit(
        &mut scheduler,
        delivery.builder_mut(now).unwrap(),
        &paths,
        &app,
        seed,
    );
    read_to_eof(&mut scheduler, delivery.builder_mut(now).unwrap(), key, now);
    delivery.builder_mut(now).unwrap().builder.seal().unwrap();
    acknowledge(&mut delivery, now);
    assert!(matches!(
        delivery.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let applied = owner.state().unwrap().get(&id).unwrap();
    assert_eq!(applied.committed_offset, 14);
    scheduler.observe_applied(key, applied).unwrap();

    let mut scan = Reconciler::new(ReconcileLimits {
        tracked: 3,
        pending: 3,
        offers: 3,
        path_bytes: 1024,
        fingerprint_bytes: 16,
    })
    .unwrap();
    scan.track(TrackedFile {
        id: TRACKED,
        locator,
        binding: Some(app.clone()),
        role: BindingRole::Live,
        active: true,
        resident: true,
    })
    .unwrap();
    scan.begin(1, Duration::ZERO, 0).unwrap();
    finish_pass(&mut scan, &mut scheduler, key, 1, 0, true);
    assert_eq!(
        scheduler
            .contain_disappearance(key, &mut delivery, &owner, now)
            .unwrap_err(),
        LifecycleRefusal::NoClosedHandleAbsence
    );

    let peer = f.write("peer.log", b"another complete line\npartial");
    let peer_seed = commit(&mut owner, &peer, Context::Unrelated);
    let peer_key = admit(
        &mut scheduler,
        delivery.builder_mut(now).unwrap(),
        &paths,
        &peer,
        peer_seed,
    );
    read_to_eof(
        &mut scheduler,
        delivery.builder_mut(now).unwrap(),
        peer_key,
        now,
    );
    delivery.builder_mut(now).unwrap().builder.seal().unwrap();
    assert_eq!(scheduler.pin(key), Err(LifecycleRefusal::NotResident));
    let saved = tempfile::tempdir().unwrap();
    let original = saved.path().join("original");
    fs::rename(&app, &original).unwrap();
    scan.update_reader_state(1, TRACKED, true, false).unwrap();

    scan.begin(2, Duration::from_secs(1), 0).unwrap();
    finish_pass(&mut scan, &mut scheduler, key, 2, 0, false);
    assert_eq!(
        scheduler
            .contain_disappearance(key, &mut delivery, &owner, now)
            .unwrap_err(),
        LifecycleRefusal::NoClosedHandleAbsence
    );
    scan.begin(3, Duration::from_secs(2), 0).unwrap();
    finish_pass(&mut scan, &mut scheduler, key, 3, 0, true);
    assert_eq!(
        scheduler
            .contain_disappearance(key, &mut delivery, &owner, now)
            .unwrap_err(),
        LifecycleRefusal::PendingDelivery
    );

    scan.begin(4, Duration::from_secs(3), 0).unwrap();
    let alias = Observation {
        path: f.source.path().join("alias"),
        locator: Some(locator),
        size: None,
        modified: None,
        fingerprint: vec![],
        reason: Some(ScanReason::Content),
        errno: None,
    };
    scan.observe(4, &alias, Duration::from_secs(3), |event| {
        scheduler
            .disappearance(key)
            .unwrap()
            .observe(TRACKED, &event);
    })
    .unwrap();
    assert_eq!(
        scheduler
            .contain_disappearance(key, &mut delivery, &owner, now)
            .unwrap_err(),
        LifecycleRefusal::NoClosedHandleAbsence
    );
    finish_pass(&mut scan, &mut scheduler, key, 4, 1, true);
    scan.begin(5, Duration::from_secs(4), 0).unwrap();
    finish_pass(&mut scan, &mut scheduler, key, 5, 0, true);
    assert_eq!(f.registry.len().unwrap(), 2);
    acknowledge(&mut delivery, now);
    owner.store.inject_write_failure_after(0);
    assert!(matches!(
        delivery.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Blocked(_)
    ));
    assert!(owner.has_pending_write());
    assert_eq!(
        scheduler
            .contain_disappearance(key, &mut delivery, &owner, now)
            .unwrap_err(),
        LifecycleRefusal::PendingDelivery
    );
    let now = delivery.next_deadline().unwrap();
    clock.set(now);
    assert!(matches!(
        delivery.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let next = f.write("next.log", b"third eligible file\n");
    let next_seed = commit(&mut owner, &next, Context::Unrelated);
    let next_key = admit(
        &mut scheduler,
        delivery.builder_mut(now).unwrap(),
        &paths,
        &next,
        next_seed,
    );
    let wait = scheduler.turn(
        now,
        READ,
        delivery.builder_mut(now).unwrap(),
        Waker::noop(),
        || false,
    );
    assert_eq!(wait.key, Some(next_key));
    assert!(matches!(wait.reason, Reason::Capacity), "{wait:?}");
    let before = owner.state().unwrap().get(&id).unwrap().clone();
    let wal = owner.wal_bytes();
    assert_eq!(before.lifecycle_state, cp::LifecycleState::Active);
    assert_eq!(
        scheduler
            .retire(key, delivery.builder_mut(now).unwrap())
            .unwrap_err(),
        LifecycleRefusal::Unfinished
    );
    let report = scheduler
        .contain_disappearance(key, &mut delivery, &owner, now)
        .unwrap();
    assert_eq!(report.file_id, id);
    assert_eq!(report.absent_since, Duration::from_secs(5));
    assert_eq!(f.registry.len().unwrap(), 3);
    report.lease.release().unwrap();
    assert_eq!(f.registry.len().unwrap(), 2);
    assert_eq!(owner.state().unwrap().get(&id), Some(&before));
    assert_eq!(owner.wal_bytes(), wal);
    assert_eq!(owner.state().unwrap().records().len(), 3);

    read_to_eof(
        &mut scheduler,
        delivery.builder_mut(now).unwrap(),
        next_key,
        now,
    );
    fs::rename(original, &app).unwrap();
    let reconnected = commit(&mut owner, &app, Context::Unrelated);
    assert_eq!(reconnected.decision, Decision::Resume(id));
    assert_eq!(reconnected.offset, before.committed_offset);
    let replacement = admit(
        &mut scheduler,
        delivery.builder_mut(now).unwrap(),
        &paths,
        &app,
        reconnected,
    );
    assert_ne!(key, replacement);
    assert_eq!(
        scheduler
            .contain_disappearance(key, &mut delivery, &owner, now)
            .unwrap_err(),
        LifecycleRefusal::UnknownReader
    );
}

/// Scenario: After eviction, the next reopen finds another file, a directory, or a symlink rejected by policy.
/// Guarantees: The reader retains its unfinished input and lease until a complete miss permits cleanup without changing the checkpoint or WAL.
#[test]
fn closed_handle_failed_reopen_waits_for_original_locator_absence() {
    #[derive(Clone, Copy)]
    enum OldPath {
        ReplacedAtOpen,
        ReplacedAtValidation,
        Directory,
        RejectedSymlink,
    }
    for old_path_kind in [
        OldPath::ReplacedAtOpen,
        OldPath::ReplacedAtValidation,
        OldPath::Directory,
        OldPath::RejectedSymlink,
    ] {
        let f = Fixture::new();
        let inputs = f.inputs(|c| c.limits.max_open_files = 1);
        let mut owner = f.owner(&inputs);
        let app = f.write("app.log", b"unfinished original line");
        let seed = commit(&mut owner, &app, Context::Unrelated);
        let locator = seed.lease.locator();
        let id = seed.file_id;
        let now = Instant::now();
        let paths = Rc::new(RefCell::new(HashMap::<ReaderKey, PathBuf>::new()));
        let names = Rc::clone(&paths);
        let old_path = app.clone();
        let io = LinuxIo(move |key: ReaderKey, cancel: &mut dyn FnMut() -> bool| {
            let names = names.borrow();
            let path = &names[&key];
            let expected = (path == &old_path
                && !matches!(old_path_kind, OldPath::ReplacedAtValidation))
            .then_some(locator);
            let parent = File::open(path.parent().unwrap()).unwrap();
            let name = CString::new(path.file_name().unwrap().as_bytes()).unwrap();
            SourceFile::open_at(&parent, &name, SymlinkPolicy::Reject, expected, cancel)
                .map_err(Into::into)
        });
        let mut scheduler = Scheduler::from_inputs(io, &inputs, 128, Arc::default(), now).unwrap();
        let mut sink = Batcher::new(&inputs, || now, || 1);
        let key = admit(&mut scheduler, &mut sink, &paths, &app, seed);
        read_to_eof(&mut scheduler, &mut sink, key, now);
        let peer = f.write("peer.log", b"unfinished peer line");
        let peer_seed = commit(&mut owner, &peer, Context::Unrelated);
        let peer_key = admit(&mut scheduler, &mut sink, &paths, &peer, peer_seed);
        read_to_eof(&mut scheduler, &mut sink, peer_key, now);
        assert_eq!(scheduler.pin(key), Err(LifecycleRefusal::NotResident));

        let archive = tempfile::tempdir().unwrap();
        let original = archive.path().join("original");
        fs::rename(&app, &original).unwrap();
        let replacement = b"replacement line\n";
        let expected_transition = match old_path_kind {
            OldPath::ReplacedAtOpen | OldPath::ReplacedAtValidation => {
                fs::write(&app, replacement).unwrap();
                Transition::Replaced
            }
            OldPath::Directory => {
                fs::create_dir(&app).unwrap();
                Transition::Ineligible
            }
            OldPath::RejectedSymlink => {
                std::os::unix::fs::symlink(&original, &app).unwrap();
                Transition::Ineligible
            }
        };
        let at = scheduler.next_deadline(READ, &sink).unwrap();
        let turn = scheduler.turn(at, READ, &mut sink, Waker::noop(), || false);
        assert_eq!(turn.key, Some(key));
        assert!(
            matches!(turn.reason, Reason::Transition(actual) if actual == expected_transition),
            "{turn:?}"
        );
        assert_eq!(scheduler.pin(key), Err(LifecycleRefusal::NotResident));
        let mut delivery = Delivery::new(sink, &inputs).unwrap();
        assert_eq!(
            scheduler
                .contain_disappearance(key, &mut delivery, &owner, at)
                .unwrap_err(),
            LifecycleRefusal::NoClosedHandleAbsence
        );
        assert_eq!(f.registry.len().unwrap(), 2);

        assert_eq!(
            scheduler
                .retire(key, delivery.builder_mut(at).unwrap())
                .unwrap_err(),
            LifecycleRefusal::Unfinished
        );

        let mut scan = Reconciler::new(ReconcileLimits {
            tracked: 2,
            pending: 2,
            offers: 2,
            path_bytes: 1024,
            fingerprint_bytes: 16,
        })
        .unwrap();
        scan.track(TrackedFile {
            id: TRACKED,
            locator,
            binding: Some(app.clone()),
            role: BindingRole::Live,
            active: true,
            resident: false,
        })
        .unwrap();
        scan.begin(1, Duration::ZERO, 0).unwrap();
        let mut observation = Observation {
            path: app.clone(),
            locator: None,
            size: None,
            modified: None,
            fingerprint: Vec::new(),
            reason: None,
            errno: None,
        };
        match old_path_kind {
            OldPath::ReplacedAtOpen | OldPath::ReplacedAtValidation => {
                observation.locator = Some(open(&app).unwrap().locator());
                observation.size = Some(replacement.len() as u64);
                observation.fingerprint = replacement[..16].into();
            }
            OldPath::Directory => {
                observation.locator =
                    Some(FileLocator::from_metadata(&fs::metadata(&app).unwrap()));
                observation.reason = Some(ScanReason::Directory);
            }
            OldPath::RejectedSymlink => observation.reason = Some(ScanReason::Unsupported),
        }
        scan.observe(1, &observation, Duration::ZERO, |event| {
            scheduler
                .disappearance(key)
                .unwrap()
                .observe(TRACKED, &event)
        })
        .unwrap();
        finish_pass(&mut scan, &mut scheduler, key, 1, 1, true);
        let wal = owner.wal_bytes();
        let before = owner.state().unwrap().get(&id).unwrap().clone();
        let report = scheduler
            .contain_disappearance(key, &mut delivery, &owner, at)
            .unwrap();
        assert_eq!(report.file_id, id);
        report.lease.release().unwrap();
        assert_eq!(f.registry.len().unwrap(), 1);
        assert_eq!(owner.wal_bytes(), wal);
        assert_eq!(owner.state().unwrap().get(&id), Some(&before));
    }
}
