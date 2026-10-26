// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::{
    config::{Config, RecoveryMismatch, StartAt},
    source::SymlinkPolicy,
};
use super::*;
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::Write,
    os::{fd::AsFd, unix::fs::PermissionsExt},
    sync::{Arc, mpsc},
    time::Duration,
};

pub(super) struct Fixture {
    source: tempfile::TempDir,
    _state: tempfile::TempDir,
    root: cp::store::StateRoot,
    registry: Arc<lease::LeaseRegistry>,
}
impl Fixture {
    pub(super) fn new() -> Self {
        let state = tempfile::tempdir().unwrap();
        fs::set_permissions(state.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let root =
            cp::store::StateRoot::from_borrowed_fd(File::open(state.path()).unwrap().as_fd())
                .unwrap();
        Self {
            source: tempfile::tempdir().unwrap(),
            _state: state,
            root,
            registry: Arc::default(),
        }
    }
    pub(super) fn inputs(&self, change: impl FnOnce(&mut Config)) -> ValidatedInputs {
        let mut config = Config {
            include: vec![self.source.path().join("*").to_str().unwrap().into()],
            ..Config::default()
        };
        config.identity.fingerprint_bytes = 16;
        config.limits.max_tracked_files = 4;
        config.limits.max_open_files = 4;
        config.start_at = StartAt::Beginning;
        change(&mut config);
        config
            .compile("group", "pipeline", "filelog", self.source.path())
            .unwrap()
            .into_parts()
            .0
    }
    pub(super) fn owner(&self, inputs: &ValidatedInputs) -> IdentityOwner {
        let store = CheckpointStore::open(
            &self.root,
            inputs.checkpoint_id(),
            inputs.store_config().unwrap(),
        )
        .unwrap();
        let leases = self
            .registry
            .owner(
                "test-node",
                IdentityOwner::required_lease_capacity(inputs).unwrap(),
            )
            .unwrap();
        IdentityOwner::new(store, leases, inputs).unwrap()
    }
    pub(super) fn write(&self, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = self.source.path().join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}
fn now() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(2_000_000_000)
}
fn open(path: &Path) -> Result<SourceFile, FileAccessError> {
    let parent = File::open(path.parent().unwrap()).unwrap();
    let name = CString::new(path.file_name().unwrap().as_bytes()).unwrap();
    SourceFile::open_at(&parent, &name, SymlinkPolicy::Reject, None, || false)
}
pub(super) fn prepare(
    owner: &mut IdentityOwner,
    path: &Path,
    context: Context,
) -> Result<Admission, IdentityError> {
    let metadata = fs::metadata(path).unwrap();
    let bytes = fs::read(path).unwrap();
    let ignored = owner.ignored_header as usize;
    let fingerprint = bytes.get(ignored..).unwrap_or_default();
    let fingerprint = &fingerprint[..fingerprint.len().min(owner.fingerprint_bytes)];
    owner.prepare(
        Candidate {
            path,
            locator: FileLocator::from_metadata(&metadata),
            size: metadata.len(),
            fingerprint,
            context,
        },
        &ProbeBudget::default(),
        now(),
        || open(path),
        || false,
    )
}
pub(super) fn ready(admission: Admission) -> PreparedAdmission {
    match admission {
        Admission::Ready(p) => p,
        other => panic!("expected ready: {other:?}"),
    }
}
fn commit(owner: &mut IdentityOwner, path: &Path, context: Context) -> ReaderSeed {
    let plan = ready(prepare(owner, path, context).unwrap());
    match owner.commit(plan).unwrap() {
        CommittedAdmission::Reader(seed) => seed,
        other => panic!("expected reader: {other:?}"),
    }
}
fn progress(owner: &mut IdentityOwner, seed: &ReaderSeed, bytes: &[u8], offset: u64) {
    let window = &bytes[offset.saturating_sub(64) as usize..offset as usize];
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: seed.file_id,
                expected_committed_offset: seed.offset,
                expected_file_epoch: seed.epoch,
                new_committed_offset: offset,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(offset, window)
                    .unwrap(),
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })],
            Durability::Sync,
        )
        .unwrap();
}

/// Scenario: Beginning/end first admission is prepared, committed, then recovered with a different start policy.
/// Guarantees: Preparation writes nothing; durable anchors and IDs survive restart and override start_at.
#[test]
fn durable_registration_and_restart() {
    for start in [StartAt::Beginning, StartAt::End] {
        let f = Fixture::new();
        let path = f.write("app", b"first line\nsecond line\n");
        let inputs = f.inputs(|c| c.start_at = start);
        let mut owner = f.owner(&inputs);
        let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        let id = plan.file_id();
        assert_eq!(owner.state().unwrap().records().len(), 0);
        let CommittedAdmission::Reader(seed) = owner.commit(plan).unwrap() else {
            panic!()
        };
        let expected = if start == StartAt::End { 23 } else { 0 };
        assert_eq!(seed.offset, expected);
        assert_eq!(seed.frontier.bytes().len(), expected as usize);
        assert_eq!(
            owner.store.synced_sequence(),
            owner.state().unwrap().last_sequence()
        );
        drop(seed);
        drop(owner);
        let inputs = f.inputs(|c| {
            c.start_at = if start == StartAt::End {
                StartAt::Beginning
            } else {
                StartAt::End
            }
        });
        let mut owner = f.owner(&inputs);
        let recovered = commit(&mut owner, &path, Context::Unrelated);
        assert_eq!((recovered.file_id, recovered.offset), (id, expected));
    }
}

/// Scenario: A valid checkpoint advances to a frontier, then its same-locator file changes at that frontier.
/// Guarantees: Equal initial prefixes do not authorize old progress; supersede is atomic at full capacity.
#[test]
fn frontier_mismatch_supersedes_at_full_capacity() {
    let f = Fixture::new();
    let mut bytes = vec![b'a'; 200];
    let path = f.write("app", &bytes);
    let inputs = f.inputs(|c| {
        c.limits.max_tracked_files = 1;
        c.limits.max_open_files = 1;
    });
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let old = seed.file_id;
    progress(&mut owner, &seed, &bytes, 150);
    drop(seed);
    bytes[149] = b'b';
    fs::write(&path, &bytes).unwrap();
    let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
    assert_eq!(
        plan.decision(),
        Decision::Register {
            anchor: Anchor::Beginning,
            superseded: Some(old),
            reset: true
        }
    );
    let CommittedAdmission::Reader(seed) = owner.commit(plan).unwrap() else {
        panic!()
    };
    assert_ne!(seed.file_id, old);
    assert_eq!(seed.offset, 0);
    assert_eq!(owner.state().unwrap().records().len(), 1);
    assert!(owner.state().unwrap().get(&old).is_none());
    let second = f.write("second", b"different");
    drop(seed);
    assert!(matches!(
        prepare(&mut owner, &second, Context::Unrelated).unwrap(),
        Admission::NeedsCapacity
    ));
}

/// Scenario: A same-locator prefix mismatch uses each configured recovery policy.
/// Guarantees: Beginning/end select fresh anchors; fail durably quarantines and later changes cannot release it.
#[test]
fn mismatch_policies_and_quarantine_persistence() {
    for policy in [
        RecoveryMismatch::Beginning,
        RecoveryMismatch::SkipToEnd,
        RecoveryMismatch::Fail,
    ] {
        let f = Fixture::new();
        let path = f.write("app", b"original bytes");
        let inputs = f.inputs(|c| c.identity.on_recovery_mismatch = policy);
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &path, Context::Unrelated);
        let old = seed.file_id;
        drop(seed);
        fs::write(&path, b"replacement stream").unwrap();
        let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        match owner.commit(plan).unwrap() {
            CommittedAdmission::Reader(seed) => {
                assert_ne!(seed.file_id, old);
                assert_eq!(
                    seed.offset,
                    if policy == RecoveryMismatch::SkipToEnd {
                        18
                    } else {
                        0
                    }
                );
            }
            CommittedAdmission::Quarantined {
                file_id: id,
                release_error,
                ..
            } => {
                assert!(release_error.is_none());
                assert_eq!(policy, RecoveryMismatch::Fail);
                let record = owner.state().unwrap().get(&id).unwrap().clone();
                fs::write(&path, b"another replacement").unwrap();
                drop(owner);
                let mut owner = f.owner(&inputs);
                assert!(
                    matches!(prepare(&mut owner, &path, Context::Replacement).unwrap(), Admission::Quarantined(found) if found == id)
                );
                assert_eq!(owner.state().unwrap().get(&id), Some(&record));
                assert!(f.registry.is_empty().unwrap());
            }
        }
    }
}

/// Scenario: A recovered checkpoint was created with a different framing profile.
/// Guarantees: Every mismatch policy blocks the original ID without mutation or opening source content.
#[test]
fn profile_mismatch_never_rekeys_or_reads() {
    let f = Fixture::new();
    let path = f.write("app", b"hello");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let id = seed.file_id;
    drop(seed);
    drop(owner);
    for mismatch in [
        RecoveryMismatch::Beginning,
        RecoveryMismatch::SkipToEnd,
        RecoveryMismatch::Fail,
    ] {
        let inputs = f.inputs(|c| {
            c.framing.max_line_bytes /= 2;
            c.identity.on_recovery_mismatch = mismatch;
        });
        let mut owner = f.owner(&inputs);
        let metadata = fs::metadata(&path).unwrap();
        let outcome = owner
            .prepare(
                Candidate {
                    path: &path,
                    locator: FileLocator::from_metadata(&metadata),
                    size: 5,
                    fingerprint: b"hello",
                    context: Context::Replacement,
                },
                &ProbeBudget::default(),
                now(),
                || panic!("blocked source opened"),
                || false,
            )
            .unwrap();
        assert!(matches!(outcome, Admission::ProfileBlocked(found) if found == id));
        assert_eq!(owner.state().unwrap().records().len(), 1);
    }
}

/// Scenario: Restart discovers the old rotated file and its replacement in either order under fail/end policies.
/// Guarantees: The 7b adapter's replacement context starts the new stream at zero; old exact progress remains independent.
#[test]
fn restart_rotation_context_is_order_independent() {
    for new_first in [false, true] {
        let f = Fixture::new();
        let path = f.write("app", b"old log line\n");
        let inputs = f.inputs(|c| {
            c.start_at = StartAt::End;
            c.identity.on_recovery_mismatch = RecoveryMismatch::Fail;
        });
        let mut owner = f.owner(&inputs);
        let old = commit(&mut owner, &path, Context::Unrelated);
        let old_id = old.file_id;
        let old_offset = old.offset;
        drop(old);
        drop(owner);
        let rotated = f.source.path().join("app.1");
        fs::rename(&path, &rotated).unwrap();
        fs::write(&path, b"new log line\n").unwrap();
        let mut owner = f.owner(&inputs);
        // This is the narrow 7b test adapter: recovered A's saved binding supplies
        // Replacement for B, regardless of which fresh offer reaches Task 12 first.
        for new in [new_first, !new_first] {
            let seed = commit(
                &mut owner,
                if new { &path } else { &rotated },
                if new {
                    Context::Replacement
                } else {
                    Context::Unrelated
                },
            );
            if new {
                assert_ne!(seed.file_id, old_id);
                assert_eq!(seed.offset, 0);
            } else {
                assert_eq!((seed.file_id, seed.offset), (old_id, old_offset));
            }
        }
        assert_eq!(owner.state().unwrap().records().len(), 2);
    }
}

/// Scenario: A new locator has equal full prefix bytes, a short common prefix, or the old saved path.
/// Guarantees: Nonempty short/full prefix evidence selects mismatch policy without transferring progress; empty evidence does not associate.
#[test]
fn changed_locator_association_is_conservative() {
    for (old_bytes, new_bytes, same_path, associated) in [
        (
            &b"0123456789abcdef-old"[..],
            &b"0123456789abcdef-new"[..],
            false,
            true,
        ),
        (&b"short"[..], &b"short-new"[..], false, true),
        (&b"old"[..], &b"different"[..], true, true),
        (&b""[..], &b""[..], false, false),
    ] {
        let f = Fixture::new();
        let path = f.write("app", old_bytes);
        let inputs = f.inputs(|c| {
            c.start_at = StartAt::End;
            c.identity.on_recovery_mismatch = RecoveryMismatch::Fail;
        });
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &path, Context::Unrelated);
        let id = seed.file_id;
        drop(seed);
        fs::rename(&path, f.source.path().join("app.old")).unwrap();
        let next = f.write(if same_path { "app" } else { "other" }, new_bytes);
        let plan = ready(prepare(&mut owner, &next, Context::Unrelated).unwrap());
        assert_eq!(
            plan.decision(),
            Decision::Register {
                anchor: if associated {
                    Anchor::Quarantine
                } else {
                    Anchor::End
                },
                superseded: None,
                reset: associated
            }
        );
        let _ = owner.commit(plan).unwrap();
        assert!(owner.state().unwrap().get(&id).is_some());
        assert_eq!(owner.state().unwrap().records().len(), 2);
    }
}

/// Scenario: A valid candidate grows after discovery, or its opened target/prefix changes.
/// Guarantees: Growth can be admitted; stale evidence defers without mutation and releases probe/lease resources.
#[test]
fn admission_revalidates_discovery_evidence() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let locator = FileLocator::from_metadata(&fs::metadata(&path).unwrap());
    let budget = ProbeBudget::default();
    OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(b"def")
        .unwrap();
    let candidate = || Candidate {
        path: &path,
        locator,
        size: 6,
        fingerprint: b"abc",
        context: Context::Unrelated,
    };
    let plan = ready(
        owner
            .prepare(candidate(), &budget, now(), || open(&path), || false)
            .unwrap(),
    );
    drop(plan);
    fs::write(&path, b"changed").unwrap();
    assert!(matches!(
        owner
            .prepare(candidate(), &budget, now(), || open(&path), || false)
            .unwrap(),
        Admission::Deferred
    ));
    let other = f.write("other", b"abcdef");
    assert!(matches!(
        owner
            .prepare(candidate(), &budget, now(), || open(&other), || false)
            .unwrap(),
        Admission::Deferred
    ));
    assert_eq!(owner.state().unwrap().records().len(), 0);
    assert!(f.registry.is_empty().unwrap());
    assert!(budget.try_reserve().is_some());
}

/// Scenario: Probe capacity is occupied, cancellation is requested, or source opening fails.
/// Guarantees: No mutation occurs, no temporary lease survives, and a busy probe never invokes the opener.
#[test]
fn cancelled_busy_and_failed_preparation_release_resources() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let locator = FileLocator::from_metadata(&fs::metadata(&path).unwrap());
    let candidate = || Candidate {
        path: &path,
        locator,
        size: 3,
        fingerprint: b"abc",
        context: Context::Unrelated,
    };
    let budget = ProbeBudget::default();
    let held = budget.try_reserve().unwrap();
    assert!(matches!(
        owner
            .prepare(
                candidate(),
                &budget,
                now(),
                || panic!("busy opener"),
                || false
            )
            .unwrap(),
        Admission::Deferred
    ));
    drop(held);
    assert!(matches!(
        owner.prepare(
            candidate(),
            &budget,
            now(),
            || panic!("cancelled opener"),
            || true
        ),
        Err(IdentityError::Source(FileAccessError::Cancelled))
    ));
    assert!(
        owner
            .prepare(
                candidate(),
                &budget,
                now(),
                || Err(FileAccessError::Cancelled),
                || false
            )
            .is_err()
    );
    assert!(f.registry.is_empty().unwrap());
    assert_eq!(owner.state().unwrap().records().len(), 0);
}

/// Scenario: Two namespaces attempt the same source, including after the temporary descriptor closes.
/// Guarantees: One logical reader owns the locator; the loser receives its owner's name and never opens the source.
#[test]
fn cross_namespace_lease_conflict_outlives_descriptor() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let second = f.inputs(|c| c.checkpoint.id = Some("second".into()));
    let mut other = f.owner(&second);
    let locator = FileLocator::from_metadata(&fs::metadata(&path).unwrap());
    let outcome = other.prepare(
        Candidate {
            path: &path,
            locator,
            size: 3,
            fingerprint: b"abc",
            context: Context::Unrelated,
        },
        &ProbeBudget::default(),
        now(),
        || panic!("conflicting opener"),
        || false,
    );
    assert!(
        matches!(outcome, Err(IdentityError::Lease(LeaseError::Conflict { owner })) if &*owner == "test-node")
    );
    drop(seed);
    let _ = commit(&mut other, &path, Context::Unrelated);
    assert!(f.registry.is_empty().unwrap());
}

/// Scenario: Another serialized checkpoint mutation happens between preparation and discovery preflight.
/// Guarantees: Commit rejects the stale plan without registering its identity or retaining its lease.
#[test]
fn stale_plan_does_not_mutate() {
    let f = Fixture::new();
    let a = f.write("a", b"aaa");
    let b = f.write("b", b"bbb");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let plan = ready(prepare(&mut owner, &a, Context::Unrelated).unwrap());
    let id = plan.file_id();
    let _ = commit(&mut owner, &b, Context::Unrelated);
    assert!(matches!(owner.commit(plan), Err(IdentityError::StalePlan)));
    assert!(owner.state().unwrap().get(&id).is_none());
    assert!(f.registry.is_empty().unwrap());
}

/// Scenario: WAL transaction capacity is exhausted before an admission commit.
/// Guarantees: CompactionRequired stops visibly before writing and releases the uncommitted source lease.
#[test]
fn compaction_rejection_stops_owner_without_retaining_lease() {
    let f = Fixture::new();
    let a = f.write("a", b"aaa");
    let b = f.write("b", b"bbb");
    let inputs = f.inputs(|_| {});
    let mut config = inputs.store_config().unwrap();
    config.max_wal_transactions = 1;
    let store = CheckpointStore::open(&f.root, inputs.checkpoint_id(), config).unwrap();
    let leases = f
        .registry
        .owner(
            "bounded",
            IdentityOwner::required_lease_capacity(&inputs).unwrap(),
        )
        .unwrap();
    let mut owner = IdentityOwner::new(store, leases, &inputs).unwrap();
    let _ = commit(&mut owner, &a, Context::Unrelated);
    let plan = ready(prepare(&mut owner, &b, Context::Unrelated).unwrap());
    assert!(matches!(
        owner.commit(plan),
        Err(IdentityError::AppendRejected(
            StoreError::CompactionRequired
        ))
    ));
    assert!(matches!(owner.state(), Err(IdentityError::Stopped)));
    assert!(f.registry.is_empty().unwrap());
    drop(owner);
    assert!(f.registry.is_empty().unwrap());
    let owner = f.owner(&inputs);
    assert_eq!(owner.state().unwrap().records().len(), 1);
}

/// Scenario: A file at nonzero progress grows within its evidence window and is reopened.
/// Guarantees: Recovery supplies the exact frontier seed without a fingerprint-growth transaction or fsync.
#[test]
fn fingerprint_growth_does_not_write_on_resume() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let id = seed.file_id;
    progress(&mut owner, &seed, b"abc", 3);
    drop(seed);
    fs::write(&path, b"abcdef").unwrap();
    let sequence = owner.state().unwrap().last_sequence();
    let synced = owner.store.synced_sequence();
    let seed = commit(&mut owner, &path, Context::Unrelated);
    assert_eq!((seed.file_id, seed.offset), (id, 3));
    assert_eq!(seed.frontier.bytes(), b"abc");
    assert_eq!(owner.state().unwrap().get(&id).unwrap().fingerprint, b"abc");
    assert_eq!(owner.state().unwrap().last_sequence(), sequence);
    assert_eq!(owner.store.synced_sequence(), synced);
}

/// Scenario: Source age equals/exceeds the threshold, lies in the future, or crosses the Unix epoch.
/// Guarantees: Only strictly older unrelated files are ignored; replacement, mismatch and unavailable mtime bypass filtering.
#[test]
fn age_policy_boundaries_and_context() {
    let policy = Policy {
        start_at: StartAt::End,
        mismatch: RecoveryMismatch::Fail,
        ignore_older_than: Duration::from_secs(10),
    };
    for age in [None, Some(Duration::ZERO), Some(Duration::from_secs(10))] {
        assert!(matches!(
            classify(policy, None, Context::Unrelated, age),
            Decision::Register {
                anchor: Anchor::End,
                ..
            }
        ));
    }
    assert_eq!(
        classify(
            policy,
            None,
            Context::Unrelated,
            Some(Duration::from_secs(11))
        ),
        Decision::Ignored
    );
    assert!(matches!(
        classify(policy, None, Context::Replacement, Some(Duration::MAX)),
        Decision::Register {
            anchor: Anchor::Beginning,
            ..
        }
    ));
    assert!(matches!(
        classify(policy, None, Context::Associated, Some(Duration::MAX)),
        Decision::Register {
            anchor: Anchor::Quarantine,
            ..
        }
    ));
    assert_eq!(
        source_age(UNIX_EPOCH, UNIX_EPOCH + Duration::from_secs(1)),
        Some(Duration::ZERO)
    );
    assert_eq!(
        source_age(
            UNIX_EPOCH + Duration::from_secs(3),
            UNIX_EPOCH - Duration::from_secs(4)
        ),
        Some(Duration::from_secs(7))
    );
}

/// Scenario: Two blocking workers race for one locator and hold the winning lease through both attempts.
/// Guarantees: Exactly one acquires; release restores capacity without relying on scheduling or sleeps.
#[test]
fn lease_contention_and_capacity_are_bounded() {
    let registry = Arc::new(lease::LeaseRegistry::default());
    let deadline = Duration::from_secs(30);
    let wins = std::thread::scope(|scope| {
        let (attempted, attempts) = mpsc::sync_channel(2);
        let mut starts = Vec::new();
        let mut releases = Vec::new();
        let mut workers = Vec::new();
        for name in ["a", "b"] {
            let owner = registry.owner(name, 1).unwrap();
            let attempted = attempted.clone();
            let (start, started) = mpsc::sync_channel(1);
            let (release, released) = mpsc::sync_channel(1);
            starts.push(start);
            releases.push(release);
            workers.push(scope.spawn(move || {
                started.recv_timeout(deadline).expect("start signal");
                let guard = owner.acquire(FileLocator {
                    device: 1,
                    inode: 1,
                });
                attempted.send(guard.is_ok()).unwrap();
                // Keep the winner alive until both acquisition attempts completed.
                // Dropping the controller's senders also releases failure paths.
                released.recv_timeout(deadline).expect("release signal");
                if guard.is_ok() {
                    assert!(matches!(
                        owner.acquire(FileLocator {
                            device: 1,
                            inode: 2
                        }),
                        Err(LeaseError::Capacity)
                    ));
                }
            }));
        }
        drop(attempted);
        for start in starts {
            start.send(()).unwrap();
        }
        let wins = usize::from(attempts.recv_timeout(deadline).expect("first acquisition"))
            + usize::from(attempts.recv_timeout(deadline).expect("second acquisition"));
        for release in releases {
            release.send(()).unwrap();
        }
        for worker in workers {
            worker.join().unwrap();
        }
        wins
    });
    assert_eq!(wins, 1);
    assert!(registry.is_empty().unwrap());
}

/// Scenario: A worker unwinds while owning a logical lease.
/// Guarantees: Its lease is released and can be reacquired without sharing test-global state.
#[test]
fn lease_unwind_releases_ownership() {
    let registry = Arc::new(lease::LeaseRegistry::default());
    let owner = registry.owner("panic", 1).unwrap();
    let locator = FileLocator {
        device: 1,
        inode: 1,
    };
    let _ = std::panic::catch_unwind(|| {
        let _guard = owner.acquire(locator).unwrap();
        panic!("controlled unwind");
    });
    owner.acquire(locator).unwrap().release().unwrap();
    assert!(registry.is_empty().unwrap());
}

/// Scenario: An old unrelated file is ignored, then its mtime advances; recovered and replacement records are also old.
/// Guarantees: Filtering creates no record and never suppresses recovery or recognized rotation.
#[test]
fn age_filter_uses_handle_mtime_only_for_unrelated_files() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|c| c.ignore_older_than = Duration::from_secs(10));
    let mut owner = f.owner(&inputs);
    let set_age = |seconds| {
        File::open(&path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(now() - Duration::from_secs(seconds)))
            .unwrap()
    };
    set_age(11);
    assert!(matches!(
        prepare(&mut owner, &path, Context::Unrelated).unwrap(),
        Admission::Ignored
    ));
    assert_eq!(owner.state().unwrap().records().len(), 0);
    assert!(f.registry.is_empty().unwrap());
    set_age(10);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let id = seed.file_id;
    drop(seed);
    set_age(100);
    assert_eq!(commit(&mut owner, &path, Context::Unrelated).file_id, id);
    fs::rename(&path, f.source.path().join("old")).unwrap();
    fs::write(&path, b"new").unwrap();
    set_age(100);
    let seed = commit(&mut owner, &path, Context::Replacement);
    assert_ne!(seed.file_id, id);
    assert_eq!(seed.offset, 0);
}

/// Scenario: An exact locator is finalized, then appears again as a new candidate.
/// Guarantees: The replay lookup excludes finalized history and admission allocates an independent ID.
#[test]
fn finalized_locator_never_reconnects() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let old = seed.file_id;
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: old,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 0,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::empty(),
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: true,
            })],
            Durability::Sync,
        )
        .unwrap();
    let locator = evidence::locator(seed.lease.locator());
    drop(seed);
    assert!(owner.state().unwrap().get_by_locator(&locator).is_none());
    let seed = commit(&mut owner, &path, Context::Unrelated);
    assert_ne!(seed.file_id, old);
    assert_eq!(owner.state().unwrap().records().len(), 2);
}

/// Scenario: A compatible checkpoint has split-record continuation, then its source becomes shorter than committed progress.
/// Guarantees: Recovery preserves continuation exactly; a later size mismatch resets identity and framing without inheriting progress.
#[test]
fn continuation_recovery_and_size_mismatch() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdefghijklmnop");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let id = seed.file_id;
    let resume = cp::FramingResume::Continuation {
        record_start_offset: 0,
        record_end_offset: 16,
        next_fragment_index: 1,
    };
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: id,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 8,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(8, b"abcdefgh")
                    .unwrap(),
                new_framing_resume: resume,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })],
            Durability::Sync,
        )
        .unwrap();
    drop(seed);
    drop(owner);
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Replacement);
    assert_eq!((seed.file_id, seed.offset, seed.resume), (id, 8, resume));
    drop(seed);
    fs::write(&path, b"abc").unwrap();
    let seed = commit(&mut owner, &path, Context::Unrelated);
    assert_ne!(seed.file_id, id);
    assert_eq!((seed.offset, seed.resume), (0, cp::FramingResume::Clean));
}

/// Scenario: A plan is handed to a different namespace owner with the same checkpoint sequence.
/// Guarantees: Sequence coincidence cannot authorize a foreign plan or checkpoint mutation.
#[test]
fn foreign_plan_is_rejected() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
    let inputs = f.inputs(|c| c.checkpoint.id = Some("other".into()));
    let mut other = f.owner(&inputs);
    assert!(matches!(other.commit(plan), Err(IdentityError::StalePlan)));
    assert_eq!(owner.state().unwrap().records().len(), 0);
    assert_eq!(other.state().unwrap().records().len(), 0);
    assert!(f.registry.is_empty().unwrap());
}

/// Scenario: Identity classification sees conflicting lifecycle, profile, continuity and replacement facts.
/// Guarantees: Quarantine precedes profile checks; profile precedes recovery; exact state precedes replacement and age.
#[test]
fn decision_precedence_matrix() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let mut record = owner.state().unwrap().get(&seed.file_id).unwrap().clone();
    let policy = Policy {
        start_at: StartAt::End,
        mismatch: RecoveryMismatch::Fail,
        ignore_older_than: Duration::from_secs(1),
    };
    for (lifecycle, compatible, continuous, expected) in [
        (
            cp::LifecycleState::Quarantined,
            false,
            false,
            Decision::Quarantined(seed.file_id),
        ),
        (
            cp::LifecycleState::Active,
            false,
            true,
            Decision::ProfileBlocked(seed.file_id),
        ),
        (
            cp::LifecycleState::Active,
            true,
            true,
            Decision::Resume(seed.file_id),
        ),
        (
            cp::LifecycleState::Active,
            true,
            false,
            Decision::Register {
                anchor: Anchor::Quarantine,
                superseded: Some(seed.file_id),
                reset: true,
            },
        ),
        (
            cp::LifecycleState::RotatedFinalized,
            true,
            true,
            Decision::Register {
                anchor: Anchor::Beginning,
                superseded: None,
                reset: false,
            },
        ),
    ] {
        record.lifecycle_state = lifecycle;
        assert_eq!(
            classify(
                policy,
                Some(Existing {
                    record: &record,
                    profile_compatible: compatible,
                    continuous
                }),
                Context::Replacement,
                Some(Duration::MAX)
            ),
            expected
        );
    }
}

/// Scenario: Fingerprinting skips a configured header while source offsets remain absolute.
/// Guarantees: Header bytes are still included in the EOF anchor/frontier and do not alter fingerprint comparison offsets.
#[test]
fn ignored_header_does_not_shift_source_offsets() {
    let f = Fixture::new();
    let path = f.write("app", b"HEADERbody");
    let inputs = f.inputs(|c| {
        c.identity.ignored_header_bytes = 6;
        c.start_at = StartAt::End;
    });
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    assert_eq!(seed.offset, 10);
    assert_eq!(seed.frontier.bytes(), b"HEADERbody");
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .fingerprint,
        b"body"
    );
}

/// Scenario: Admission is constructed with a different selected profile or namespace than its store.
/// Guarantees: Construction fails before opening source files or creating incompatible registrations.
#[test]
fn construction_checks_store_configuration() {
    for change_profile in [false, true] {
        let f = Fixture::new();
        let inputs = f.inputs(|_| {});
        let store = CheckpointStore::open(
            &f.root,
            inputs.checkpoint_id(),
            inputs.store_config().unwrap(),
        )
        .unwrap();
        let different = f.inputs(|c| {
            if change_profile {
                c.identity.fingerprint_bytes = 32;
            } else {
                c.checkpoint.id = Some("different".into());
            }
        });
        let leases = f
            .registry
            .owner(
                "test",
                IdentityOwner::required_lease_capacity(&inputs).unwrap(),
            )
            .unwrap();
        assert!(matches!(
            IdentityOwner::new(store, leases, &different),
            Err(IdentityError::ConfigurationMismatch)
        ));
        assert!(f.registry.is_empty().unwrap());
    }
}

/// Scenario: Delivery applies Acked progress with deferred durability, then reaches its sync boundary.
/// Guarantees: Applied progress remains distinguishable from synced progress until an explicit successful flush.
#[test]
fn deferred_progress_has_an_explicit_sync_boundary() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let registered = owner.store.synced_sequence();
    let receipt = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: seed.file_id,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 3,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"abc")
                    .unwrap(),
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })],
            Durability::Deferred,
        )
        .unwrap();
    assert!(!receipt.synced);
    assert_eq!(owner.store.synced_sequence(), registered);
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        3
    );
    let _ = owner.sync_checkpoint().unwrap().growth.unwrap();
    assert_eq!(owner.store.synced_sequence(), receipt.sequence);
}

/// Scenario: A new-file admission waits while another file receives ordinary Ack-driven progress.
/// Guarantees: Unrelated progress leaves the prepared plan valid and does not require another source probe.
#[test]
fn unrelated_progress_does_not_invalidate_admission() {
    let f = Fixture::new();
    let a = f.write("a", b"abcdef");
    let b = f.write("b", b"other");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let a_seed = commit(&mut owner, &a, Context::Unrelated);
    let plan = ready(prepare(&mut owner, &b, Context::Unrelated).unwrap());
    let id = plan.file_id();
    progress(&mut owner, &a_seed, b"abcdef", 3);
    let CommittedAdmission::Reader(seed) = owner.commit(plan).unwrap() else {
        panic!()
    };
    assert_eq!(seed.file_id, id);
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&a_seed.file_id)
            .unwrap()
            .committed_offset,
        3
    );
}

/// Scenario: The exact record changes progress or becomes finalized after preparation.
/// Guarantees: Both changes invalidate its old reader seed even though ordinary progress does not bump the namespace revision.
#[test]
fn touched_progress_and_finalization_invalidate_admission() {
    for finalize in [false, true] {
        let f = Fixture::new();
        let path = f.write("app", b"abc");
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &path, Context::Unrelated);
        let id = seed.file_id;
        drop(seed);
        let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        let before = owner.identity_revision;
        let _ = owner
            .append_checkpoint(
                vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                    file_id: id,
                    expected_committed_offset: 0,
                    expected_file_epoch: 1,
                    new_committed_offset: 3,
                    new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"abc")
                        .unwrap(),
                    new_framing_resume: cp::FramingResume::Clean,
                    new_last_seen_time_unix_nano: 1,
                    finalize,
                })],
                Durability::Sync,
            )
            .unwrap();
        assert_eq!(owner.identity_revision, before + u64::from(finalize));
        assert!(matches!(owner.commit(plan), Err(IdentityError::StalePlan)));
        assert_eq!(owner.state().unwrap().get(&id).unwrap().committed_offset, 3);
        assert!(f.registry.is_empty().unwrap());
    }
}

/// Scenario: A non-progress request is rejected by encoding or replay preflight.
/// Guarantees: It writes nothing, leaves admission revisions unchanged, and does not permanently stop a usable owner.
#[test]
fn known_prewrite_rejections_leave_owner_usable() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
    assert!(matches!(
        owner.append_checkpoint(Vec::new(), Durability::Sync),
        Err(IdentityError::AppendRejected(StoreError::Encode(_)))
    ));
    assert!(matches!(
        owner.append_checkpoint(
            vec![cp::Operation::UpdateMetadata(cp::UpdateMetadata {
                file_id: plan.file_id(),
                expected_prior_state: cp::LifecycleState::Active,
                expected_file_epoch: 1,
                last_seen_time_unix_nano: 1,
                advisory_path: None,
            })],
            Durability::Sync
        ),
        Err(IdentityError::AppendRejected(StoreError::Replay(_)))
    ));
    assert!(!owner.store.has_unresolved_append());
    assert_eq!(owner.state().unwrap().last_sequence(), 0);
    assert!(matches!(
        owner.commit(plan).unwrap(),
        CommittedAdmission::Reader(_)
    ));
}

struct FaultySource {
    inner: SourceFile,
    offset: u64,
    faults: std::cell::Cell<usize>,
    kind: Option<std::io::ErrorKind>,
    chunk: usize,
}
impl evidence::EvidenceSource for FaultySource {
    fn locator(&self) -> FileLocator {
        self.inner.locator()
    }
    fn metadata(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<fs::Metadata, FileAccessError> {
        self.inner.metadata(cancelled)
    }
    fn read_at(
        &self,
        offset: u64,
        buffer: &mut [u8],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<usize, FileAccessError> {
        if offset == self.offset && self.faults.get() != 0 {
            self.faults.set(self.faults.get() - 1);
            return self.kind.map_or(Ok(0), |kind| {
                let error = if kind == std::io::ErrorKind::Other {
                    std::io::Error::from_raw_os_error(libc::EIO)
                } else {
                    std::io::Error::from(kind)
                };
                Err(error.into())
            });
        }
        let len = buffer.len().min(self.chunk);
        self.inner.read_at(offset, &mut buffer[..len], cancelled)
    }
}

fn prepare_fault(
    owner: &mut IdentityOwner,
    path: &Path,
    offset: u64,
    faults: usize,
    kind: Option<std::io::ErrorKind>,
    chunk: usize,
) -> Result<Admission, IdentityError> {
    let metadata = fs::metadata(path).unwrap();
    let bytes = fs::read(path).unwrap();
    let fingerprint = &bytes[..bytes.len().min(owner.fingerprint_bytes)];
    owner.prepare_with_source(
        Candidate {
            path,
            locator: FileLocator::from_metadata(&metadata),
            size: metadata.len(),
            fingerprint,
            context: Context::Unrelated,
        },
        &ProbeBudget::default(),
        now(),
        || {
            Ok(FaultySource {
                inner: open(path)?,
                offset,
                faults: std::cell::Cell::new(faults),
                kind,
                chunk,
            })
        },
        || false,
    )
}

/// Scenario: Prefix or committed-frontier reads receive EINTR followed by successful partial reads.
/// Guarantees: Interrupted retries and short successful reads preserve the original identity and progress without writing a reset.
#[test]
fn interrupted_evidence_reads_retry_without_reset() {
    for offset in [0, 86] {
        let f = Fixture::new();
        let bytes = vec![b'x'; 200];
        let path = f.write("app", &bytes);
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &path, Context::Unrelated);
        let id = seed.file_id;
        progress(&mut owner, &seed, &bytes, 150);
        drop(seed);
        let sequence = owner.state().unwrap().last_sequence();
        let plan = ready(
            prepare_fault(
                &mut owner,
                &path,
                offset,
                2,
                Some(std::io::ErrorKind::Interrupted),
                2,
            )
            .unwrap(),
        );
        let CommittedAdmission::Reader(seed) = owner.commit(plan).unwrap() else {
            panic!()
        };
        assert_eq!((seed.file_id, seed.offset), (id, 150));
        assert_eq!(owner.state().unwrap().last_sequence(), sequence);
    }
}

/// Scenario: Either prefix or frontier read returns EIO, WouldBlock, or an unending stream of EINTR.
/// Guarantees: Errors retain their kind, preserve the complete checkpoint record, release the lease, and permit a later clean recovery.
#[test]
fn environmental_read_errors_never_supersede() {
    for offset in [0, 86] {
        for kind in [
            std::io::ErrorKind::Other,
            std::io::ErrorKind::WouldBlock,
            std::io::ErrorKind::Interrupted,
        ] {
            let f = Fixture::new();
            let bytes = vec![b'x'; 200];
            let path = f.write("app", &bytes);
            let inputs = f.inputs(|_| {});
            let mut owner = f.owner(&inputs);
            let seed = commit(&mut owner, &path, Context::Unrelated);
            let id = seed.file_id;
            progress(&mut owner, &seed, &bytes, 150);
            drop(seed);
            let record = owner.state().unwrap().get(&id).unwrap().clone();
            let sequence = owner.state().unwrap().last_sequence();
            let result = prepare_fault(&mut owner, &path, offset, 100, Some(kind), usize::MAX);
            assert!(
                matches!(result, Err(IdentityError::Source(ref error)) if error.os_error().is_some_and(|e| if kind == std::io::ErrorKind::Other { e.raw_os_error() == Some(libc::EIO) } else { e.kind() == kind }))
            );
            assert_eq!(owner.state().unwrap().get(&id), Some(&record));
            assert_eq!(owner.state().unwrap().last_sequence(), sequence);
            assert!(f.registry.is_empty().unwrap());
            assert_eq!(commit(&mut owner, &path, Context::Unrelated).file_id, id);
        }
    }
}

/// Scenario: A committed frontier reaches EOF before its required window is complete.
/// Guarantees: Actual short evidence remains a continuity mismatch and follows the configured reset policy.
#[test]
fn short_frontier_still_requires_new_identity() {
    let f = Fixture::new();
    let bytes = vec![b'x'; 200];
    let path = f.write("app", &bytes);
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let id = seed.file_id;
    progress(&mut owner, &seed, &bytes, 150);
    drop(seed);
    let plan = ready(prepare_fault(&mut owner, &path, 86, 1, None, usize::MAX).unwrap());
    assert!(
        matches!(plan.decision(), Decision::Register { superseded: Some(old), anchor: Anchor::Beginning, .. } if old == id)
    );
    let CommittedAdmission::Reader(seed) = owner.commit(plan).unwrap() else {
        panic!()
    };
    assert_ne!(seed.file_id, id);
    assert_eq!(seed.offset, 0);
}

/// Scenario: A signal storm occurs while a read is being cancelled.
/// Guarantees: Cancellation is observed between Interrupted retries rather than waiting for the retry budget to expire.
#[test]
fn interrupted_retries_observe_cancellation() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let source = FaultySource {
        inner: open(&path).unwrap(),
        offset: 0,
        faults: std::cell::Cell::new(100),
        kind: Some(std::io::ErrorKind::Interrupted),
        chunk: 3,
    };
    let mut calls = 0;
    let mut cancelled = || {
        calls += 1;
        calls > 2
    };
    assert!(matches!(
        evidence::prefix(&source, 0, &mut [0; 3], &mut cancelled),
        Err(FileAccessError::Cancelled)
    ));
    assert_eq!(source.faults.get(), 98);
}

/// Scenario: A file providing saved-path association is finalized while a new candidate waits for admission.
/// Guarantees: Finalizing even an unrelated locator invalidates historical eligibility used by that plan.
#[test]
fn finalization_invalidates_historical_path_association() {
    let f = Fixture::new();
    let path = f.write("app", b"old");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let old = commit(&mut owner, &path, Context::Unrelated);
    fs::rename(&path, f.source.path().join("app.old")).unwrap();
    fs::write(&path, b"new").unwrap();
    let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
    assert!(matches!(
        plan.decision(),
        Decision::Register { reset: true, .. }
    ));
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: old.file_id,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 3,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"old")
                    .unwrap(),
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: true,
            })],
            Durability::Sync,
        )
        .unwrap();
    assert!(matches!(owner.commit(plan), Err(IdentityError::StalePlan)));
}

/// Scenario: A non-progress mutation would wrap the process-local identity revision.
/// Guarantees: Exhaustion stops the owner before writing rather than reviving old admission plans.
#[test]
fn identity_revision_cannot_wrap() {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    owner.identity_revision = u64::MAX;
    let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
    assert!(matches!(
        owner.commit(plan),
        Err(IdentityError::RevisionExhausted)
    ));
    assert!(matches!(owner.state(), Err(IdentityError::Stopped)));
    assert_eq!(owner.store.state().unwrap().records().len(), 0);
    assert!(f.registry.is_empty().unwrap());
}

/// Scenario: Independent files share complete common headers at the minimum and default fingerprint widths.
/// Guarantees: Conservative association bypasses age/start policy and may quarantine under fail; no progress transfers.
#[test]
fn common_headers_are_explicit_conservative_associations() {
    for width in [16, 1000] {
        let f = Fixture::new();
        let bytes = vec![b'h'; width];
        let old = f.write("old", &bytes);
        let new = f.write("new", &bytes);
        let inputs = f.inputs(|c| {
            c.identity.fingerprint_bytes = width as u64;
            c.start_at = StartAt::End;
            c.identity.on_recovery_mismatch = RecoveryMismatch::Fail;
            c.ignore_older_than = Duration::from_secs(10);
        });
        File::open(&old)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(now()))
            .unwrap();
        let mut owner = f.owner(&inputs);
        let _ = commit(&mut owner, &old, Context::Unrelated);
        File::open(&new)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(now() - Duration::from_secs(11)))
            .unwrap();
        let plan = ready(prepare(&mut owner, &new, Context::Unrelated).unwrap());
        assert!(matches!(
            plan.decision(),
            Decision::Register {
                anchor: Anchor::Quarantine,
                superseded: None,
                ..
            }
        ));
        assert!(matches!(
            owner.commit(plan).unwrap(),
            CommittedAdmission::Quarantined {
                release_error: None,
                ..
            }
        ));
    }
}

/// Scenario: The documented demo performs one progress transaction per Ack, containing one or 512 clean file deltas.
/// Guarantees: WAL duration examples use actual encoded frame sizes and respect both byte and transaction caps.
#[test]
fn demo_budget_examples_use_encoded_progress_frames() {
    let config = Config::default().checkpoint;
    for (files, expected_bytes) in [(1, 129), (512, 45608)] {
        let operations = (0..files)
            .map(|id| {
                cp::Operation::UpdateProgress(cp::UpdateProgress {
                    file_id: cp::FileId::from_bytes((id as u128).to_be_bytes()),
                    expected_committed_offset: 0,
                    expected_file_epoch: 1,
                    new_committed_offset: 1,
                    new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(1, b"x")
                        .unwrap(),
                    new_framing_resume: cp::FramingResume::Clean,
                    new_last_seen_time_unix_nano: 1,
                    finalize: false,
                })
            })
            .collect();
        let frame = cp::encode_transaction(&cp::Transaction {
            sequence: 1,
            operations,
        })
        .unwrap();
        assert_eq!(frame.len(), expected_bytes);
        let remaining = (config.compact_after_bytes / frame.len() as u64)
            .min(config.compact_after_transactions);
        assert_eq!(remaining, if files == 1 { 10_000 } else { 1471 });
    }
}

/// Scenario: A short file is copied to a new locator before unlink or copytruncate while unread bytes survive only in the copy.
/// Guarantees: Matching nonempty short evidence starts the copy at zero under defaults, even when the original locator still exists.
#[test]
fn short_copies_preserve_surviving_unread_bytes() {
    for unlink in [false, true] {
        let f = Fixture::new();
        let original = f.write("app", b"header\n");
        let inputs = f.inputs(|c| {
            c.identity.fingerprint_bytes = 1000;
            c.start_at = StartAt::End;
        });
        let mut owner = f.owner(&inputs);
        let old = commit(&mut owner, &original, Context::Unrelated);
        let old_id = old.file_id;
        drop(old);
        drop(owner);
        OpenOptions::new()
            .append(true)
            .open(&original)
            .unwrap()
            .write_all(b"unacked tail\n")
            .unwrap();
        let copy = f.source.path().join("app.1");
        let _ = fs::copy(&original, &copy).unwrap();
        if unlink {
            fs::remove_file(&original).unwrap();
        } else {
            fs::write(&original, []).unwrap();
        }
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &copy, Context::Unrelated);
        assert_ne!(seed.file_id, old_id);
        assert_eq!(seed.offset, 0);
        assert_eq!(
            owner
                .state()
                .unwrap()
                .get(&old_id)
                .unwrap()
                .committed_offset,
            7
        );
    }
}

/// Scenario: A registration append fails after zero bytes, a partial frame, or the complete frame.
/// Guarantees: The owner retains the original lease/outcome, blocks other writes, resolves exactly once and never reopens around the store protocol.
#[test]
fn pending_admission_resolves_once_with_its_original_lease() {
    for bytes in [0, 12, usize::MAX] {
        let f = Fixture::new();
        let path = f.write("app", b"data");
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        let id = plan.file_id();
        owner.store.inject_write_failure_after(bytes);
        assert!(matches!(
            owner.commit(plan),
            Err(IdentityError::PendingWrite(_))
        ));
        assert!(owner.has_pending_write());
        assert!(!owner.is_stopped());
        assert_eq!(f.registry.len().unwrap(), 1);
        assert!(matches!(
            prepare(&mut owner, &path, Context::Unrelated),
            Err(IdentityError::PendingResolutionRequired)
        ));
        let result = owner.resolve_pending().unwrap();
        assert_eq!(result.receipt.sequence, 1);
        assert!(result.receipt.synced);
        assert_eq!(owner.identity_revision, 1);
        assert_eq!(result.admissions.len(), 1);
        assert!(
            matches!(&result.admissions[0], CommittedAdmission::Reader(seed) if seed.file_id == id)
        );
        assert!(matches!(
            owner.resolve_pending(),
            Err(IdentityError::NoPendingWrite)
        ));
        assert_eq!(owner.state().unwrap().records().len(), 1);
        drop(result);
        assert!(f.registry.is_empty().unwrap());
    }
}

/// Scenario: An Ack-driven progress append fails and is resolved under the held namespace lock.
/// Guarantees: Progress publishes once, does not bump identity revision, and returns no duplicate admission.
#[test]
fn pending_progress_resolves_once_without_rekeying() {
    for bytes in [0, 12, usize::MAX] {
        let f = Fixture::new();
        let path = f.write("app", b"abc");
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &path, Context::Unrelated);
        let revision = owner.identity_revision;
        owner.store.inject_write_failure_after(bytes);
        assert!(matches!(
            owner.append_checkpoint(
                vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                    file_id: seed.file_id,
                    expected_committed_offset: 0,
                    expected_file_epoch: 1,
                    new_committed_offset: 3,
                    new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"abc")
                        .unwrap(),
                    new_framing_resume: cp::FramingResume::Clean,
                    new_last_seen_time_unix_nano: 1,
                    finalize: false,
                })],
                Durability::Sync
            ),
            Err(IdentityError::PendingWrite(_))
        ));
        let resolved = owner.resolve_pending().unwrap();
        assert!(resolved.admissions.is_empty());
        assert_eq!(resolved.receipt.sequence, 2);
        assert_eq!(owner.identity_revision, revision);
        assert_eq!(
            owner
                .state()
                .unwrap()
                .get(&seed.file_id)
                .unwrap()
                .committed_offset,
            3
        );
        assert!(matches!(
            owner.resolve_pending(),
            Err(IdentityError::NoPendingWrite)
        ));
    }
}

/// Scenario: A required sync fails during admission or resolution, or recoverable writes exhaust their configured budget.
/// Guarantees: Terminal owners retain pending leases and cannot certify or retry a known failed sync.
#[test]
fn pending_failure_budget_and_sync_errors_are_terminal() {
    for sync_stage in [
        None,
        Some("append file sync"),
        Some("pre-resolution file sync"),
    ] {
        let f = Fixture::new();
        let path = f.write("app", b"abc");
        let inputs = f.inputs(|c| c.checkpoint.max_consecutive_failures = 2);
        let mut owner = f.owner(&inputs);
        let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        if sync_stage == Some("append file sync") {
            owner.store.inject_sync_failure("append file sync");
            assert!(matches!(
                owner.commit(plan),
                Err(IdentityError::Store(StoreError::SyncFailed { .. }))
            ));
        } else {
            owner.store.inject_write_failure_after(0);
            assert!(matches!(
                owner.commit(plan),
                Err(IdentityError::PendingWrite(_))
            ));
            if let Some(stage) = sync_stage {
                owner.store.inject_sync_failure(stage);
            } else {
                owner.store.inject_write_failure_after(0);
            }
            assert!(matches!(
                owner.resolve_pending(),
                Err(IdentityError::Store(_))
            ));
        }
        assert!(owner.is_stopped());
        assert!(owner.has_pending_write());
        assert_eq!(f.registry.len().unwrap(), 1);
        assert!(matches!(
            owner.resolve_pending(),
            Err(IdentityError::Stopped)
        ));
        drop(owner);
        assert!(f.registry.is_empty().unwrap());
    }
}

/// Scenario: Files registered empty grow while Acked progress is deferred until the next sync boundary.
/// Guarantees: Growth batches separately from progress, both become durable at the boundary, and prefixes are no longer empty after restart.
#[test]
fn empty_file_growth_is_batched_at_sync() {
    let f = Fixture::new();
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let mut readers = Vec::new();
    for name in ["a", "b", "c"] {
        let path = f.write(name, b"");
        let reader = commit(&mut owner, &path, Context::Unrelated);
        fs::write(&path, b"prefix").unwrap();
        readers.push(reader);
    }
    let sequence = owner.state().unwrap().last_sequence();
    for reader in &readers {
        assert!(owner.queue_fingerprint(reader, b"prefix").unwrap());
    }
    let operations = readers
        .iter()
        .map(|reader| {
            cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: reader.file_id,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 6,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(6, b"prefix")
                    .unwrap(),
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })
        })
        .collect();
    let _ = owner
        .append_checkpoint(operations, Durability::Deferred)
        .unwrap();
    assert_eq!(owner.synced_sequence(), sequence);
    let _ = owner.sync_checkpoint().unwrap().growth.unwrap();
    assert_eq!(owner.state().unwrap().last_sequence(), sequence + 2);
    assert_eq!(owner.synced_sequence(), sequence + 2);
    for reader in &readers {
        assert_eq!(
            owner
                .state()
                .unwrap()
                .get(&reader.file_id)
                .unwrap()
                .fingerprint,
            b"prefix"
        );
    }
    drop(readers);
    drop(owner);
    let owner = f.owner(&inputs);
    assert!(
        owner
            .state()
            .unwrap()
            .records()
            .all(|r| r.fingerprint == b"prefix" && r.committed_offset == 6)
    );
}

/// Scenario: More evidence extensions are offered than the bounded growth queue can hold.
/// Guarantees: The caller retains unqueued evidence and can submit it after flush; repeated growth for one file is coalesced.
#[test]
fn growth_queue_is_bounded_and_coalesces() {
    let f = Fixture::new();
    let inputs = f.inputs(|c| c.limits.max_open_files = 1);
    let mut owner = f.owner(&inputs);
    let a = f.write("a", b"");
    let b = f.write("b", b"");
    let first = commit(&mut owner, &a, Context::Unrelated);
    let second = commit(&mut owner, &b, Context::Unrelated);
    fs::write(&a, b"ab").unwrap();
    fs::write(&b, b"b").unwrap();
    assert!(owner.queue_fingerprint(&first, b"a").unwrap());
    assert!(owner.queue_fingerprint(&first, b"ab").unwrap());
    assert!(!owner.queue_fingerprint(&second, b"b").unwrap());
    assert_eq!(owner.growth.len(), 1);
    let _ = owner.sync_checkpoint().unwrap().growth.unwrap();
    assert!(owner.queue_fingerprint(&second, b"b").unwrap());
    let _ = owner.sync_checkpoint().unwrap().growth.unwrap();
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&first.file_id)
            .unwrap()
            .fingerprint,
        b"ab"
    );
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&second.file_id)
            .unwrap()
            .fingerprint,
        b"b"
    );
}

/// Scenario: Several plans are prepared at one revision and committed together, including a stale member.
/// Guarantees: A valid group consumes one WAL transaction; a stale group publishes no prefix.
#[test]
fn admission_batches_validate_before_one_atomic_write() {
    let f = Fixture::new();
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let paths: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|n| f.write(n, n.as_bytes()))
        .collect();
    let plans = paths
        .iter()
        .map(|p| ready(prepare(&mut owner, p, Context::Unrelated).unwrap()))
        .collect();
    let results = owner.commit_batch(plans).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(owner.state().unwrap().last_sequence(), 1);
    drop(results);
    let plans = paths
        .iter()
        .map(|p| ready(prepare(&mut owner, p, Context::Unrelated).unwrap()))
        .collect();
    let id = owner.state().unwrap().records().next().unwrap().file_id;
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateMetadata(cp::UpdateMetadata {
                file_id: id,
                expected_prior_state: cp::LifecycleState::Active,
                expected_file_epoch: 1,
                last_seen_time_unix_nano: 1,
                advisory_path: None,
            })],
            Durability::Sync,
        )
        .unwrap();
    assert!(matches!(
        owner.commit_batch(plans),
        Err(IdentityError::StalePlan)
    ));
    assert_eq!(owner.state().unwrap().last_sequence(), 2);
    assert!(f.registry.is_empty().unwrap());
}

/// Scenario: Optional growth reaches the WAL cap after Acked progress was appended with deferred sync.
/// Guarantees: The required progress barrier succeeds and optional maintenance does not stop the owner.
#[test]
fn growth_at_cap_must_sync_progress() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|c| c.checkpoint.compact_after_transactions = 2);
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    fs::write(&path, b"abc").unwrap();
    assert!(owner.queue_fingerprint(&seed, b"abc").unwrap());
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: seed.file_id,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 3,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"abc")
                    .unwrap(),
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })],
            Durability::Deferred,
        )
        .unwrap();
    let result = owner.sync_checkpoint();
    assert_eq!(
        (result.is_ok(), owner.is_stopped(), owner.synced_sequence()),
        (true, false, 2),
        "{result:?}"
    );
}

fn assert_preserved_opener_error(error: FileAccessError) {
    let f = Fixture::new();
    let path = f.write("app", b"abc");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let locator = FileLocator::from_metadata(&fs::metadata(&path).unwrap());
    let result = owner.prepare(
        Candidate {
            path: &path,
            locator,
            size: 3,
            fingerprint: b"abc",
            context: Context::Unrelated,
        },
        &ProbeBudget::default(),
        now(),
        || Err(error),
        || false,
    );
    assert!(
        matches!(result, Err(IdentityError::Source(_))),
        "opener stage was hidden: {result:?}"
    );
}

/// Scenario: Pin-to-read conversion fails because procfs reopening is unavailable.
/// Guarantees: ENOENT from the reopen stage remains an observable source error.
#[test]
fn pinned_reopen_is_not_disappearance() {
    assert_preserved_opener_error(FileAccessError::PinnedReopen(
        std::io::Error::from_raw_os_error(libc::ENOENT),
    ));
}

/// Scenario: A read handle fails verification against its pinned source.
/// Guarantees: Verification failure is not reduced to ordinary candidate churn.
#[test]
fn reopened_mismatch_preserves_stage() {
    assert_preserved_opener_error(FileAccessError::ReopenedMismatch {
        actual: FileLocator {
            device: 0,
            inode: 0,
        },
    });
}

/// Scenario: Three plans compete for one remaining durable slot, alongside a net-zero same-locator replacement.
/// Guarantees: Selection chooses fitting plans before preflight, preserves order, and does not suppress net-zero work.
#[test]
fn selection_counts_group_capacity_and_net_zero_replacement() {
    let f = Fixture::new();
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let mut original = None;
    for name in ["a", "b", "c"] {
        let path = f.write(name, b"old");
        let seed = commit(&mut owner, &path, Context::Unrelated);
        if name == "a" {
            original = Some((path, seed.file_id));
        }
    }
    let (original, old_id) = original.unwrap();
    let first = f.write("d", b"new one");
    let second = f.write("e", b"new two");
    let one = ready(prepare(&mut owner, &first, Context::Unrelated).unwrap());
    let one_id = one.file_id();
    let two = ready(prepare(&mut owner, &second, Context::Unrelated).unwrap());
    let two_id = two.file_id();
    fs::write(&original, b"replacement").unwrap();
    let replacement = ready(prepare(&mut owner, &original, Context::Unrelated).unwrap());
    let replacement_id = replacement.file_id();
    let selected = owner.select_batch(vec![one, two, replacement]).unwrap();
    assert_eq!(
        selected
            .ready
            .iter()
            .map(PreparedAdmission::file_id)
            .collect::<Vec<_>>(),
        [one_id, replacement_id]
    );
    assert_eq!(selected.needs_capacity.len(), 1);
    assert_eq!(selected.needs_capacity[0].file_id(), two_id);
    drop(selected.needs_capacity);
    assert_eq!(owner.state().unwrap().records().len(), 3);
    let results = owner.commit_batch(selected.ready).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(owner.state().unwrap().records().len(), 4);
    assert!(owner.state().unwrap().get(&old_id).is_none());
    assert!(owner.state().unwrap().get(&two_id).is_none());
}

/// Scenario: A caller bypasses selection and submits more new records than available slots.
/// Guarantees: Commit rejects the entire group before writing, instead of committing an undisclosed subset.
#[test]
fn oversized_preflighted_group_is_rejected_atomically() {
    let f = Fixture::new();
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    for name in ["a", "b", "c"] {
        let path = f.write(name, b"old");
        let _ = commit(&mut owner, &path, Context::Unrelated);
    }
    let a = f.write("d", b"new one");
    let b = f.write("e", b"new two");
    let plans = vec![
        ready(prepare(&mut owner, &a, Context::Unrelated).unwrap()),
        ready(prepare(&mut owner, &b, Context::Unrelated).unwrap()),
    ];
    let sequence = owner.state().unwrap().last_sequence();
    assert!(matches!(
        owner.commit_batch(plans),
        Err(IdentityError::BatchCapacity)
    ));
    assert_eq!(owner.state().unwrap().last_sequence(), sequence);
    assert!(!owner.is_stopped());
    assert_eq!(owner.state().unwrap().records().len(), 3);
    assert!(f.registry.is_empty().unwrap());
}

/// Scenario: Optional growth fails during its write after required deferred progress was synchronized.
/// Guarantees: Required progress remains durable, the growth queue survives uncertainty, and resolution applies growth once.
#[test]
fn pending_growth_preserves_required_sync_and_queue() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let reader = commit(&mut owner, &path, Context::Unrelated);
    fs::write(&path, b"abc").unwrap();
    assert!(owner.queue_fingerprint(&reader, b"abc").unwrap());
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: reader.file_id,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 3,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"abc")
                    .unwrap(),
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })],
            Durability::Deferred,
        )
        .unwrap();
    let revision = owner.identity_revision;
    owner.store.inject_write_failure_after(12);
    let synced = owner.sync_checkpoint().unwrap();
    assert_eq!(synced.synced_sequence, 2);
    assert!(matches!(synced.growth, Err(IdentityError::PendingWrite(_))));
    assert_eq!(owner.synced_sequence(), 2);
    assert_eq!(owner.identity_revision, revision);
    assert_eq!(owner.growth.len(), 1);
    let done = owner.resolve_pending().unwrap();
    assert!(done.admissions.is_empty());
    assert_eq!(done.receipt.sequence, 3);
    assert!(owner.growth.is_empty());
    assert_eq!(owner.identity_revision, revision + 1);
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&reader.file_id)
            .unwrap()
            .fingerprint,
        b"abc"
    );
    assert!(matches!(
        owner.resolve_pending(),
        Err(IdentityError::NoPendingWrite)
    ));
}

/// Scenario: Optional growth has an invalid selected-window length that replay rejects before writing.
/// Guarantees: Required progress sync still happens, but the invariant failure is not silently swallowed as capacity pressure.
#[test]
fn growth_invariant_failure_stays_visible() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let reader = commit(&mut owner, &path, Context::Unrelated);
    owner.growth.push(cp::UpdateFingerprint {
        file_id: reader.file_id,
        expected_file_epoch: 1,
        expected_fingerprint: Vec::new(),
        new_fingerprint: vec![b'x'; 17],
    });
    let synced = owner.sync_checkpoint().unwrap();
    assert_eq!(synced.synced_sequence, 1);
    assert!(matches!(
        synced.growth,
        Err(IdentityError::AppendRejected(StoreError::Replay(_)))
    ));
    assert!(owner.growth.is_empty());
    assert!(owner.sync_checkpoint().unwrap().growth.unwrap().is_none());
    assert_eq!(owner.synced_sequence(), 1);
    assert!(
        owner
            .state()
            .unwrap()
            .get(&reader.file_id)
            .unwrap()
            .fingerprint
            .is_empty()
    );
}

/// Scenario: Initial selection reports a changed locator or a missing candidate entry.
/// Guarantees: These positively classified stale selections defer without creating state or retaining a lease.
#[test]
fn candidate_disappearance_and_locator_change_defer() {
    for error in [
        FileAccessError::LocatorChanged {
            actual: FileLocator {
                device: 0,
                inode: 0,
            },
        },
        FileAccessError::Io(std::io::Error::from_raw_os_error(libc::ENOENT)),
    ] {
        let f = Fixture::new();
        let path = f.write("app", b"abc");
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let locator = FileLocator::from_metadata(&fs::metadata(&path).unwrap());
        let result = owner
            .prepare(
                Candidate {
                    path: &path,
                    locator,
                    size: 3,
                    fingerprint: b"abc",
                    context: Context::Unrelated,
                },
                &ProbeBudget::default(),
                now(),
                || Err(error),
                || false,
            )
            .unwrap();
        assert!(matches!(result, Admission::Deferred));
        assert_eq!(owner.state().unwrap().records().len(), 0);
        assert!(f.registry.is_empty().unwrap());
    }
}

/// Scenario: A complete short copy predates a later fingerprint extension, then the original is unlinked across restart.
/// Guarantees: The shorter surviving copy still starts at zero; requiring it to cover newer stored evidence would skip unread bytes.
#[test]
fn older_short_copy_matches_later_grown_evidence() {
    let f = Fixture::new();
    let original = f.write("app", b"");
    let inputs = f.inputs(|c| c.start_at = StartAt::End);
    let mut owner = f.owner(&inputs);
    let old = commit(&mut owner, &original, Context::Unrelated);
    let old_id = old.file_id;
    OpenOptions::new()
        .append(true)
        .open(&original)
        .unwrap()
        .write_all(b"abcdefgh")
        .unwrap();
    assert!(owner.queue_fingerprint(&old, b"abcdefgh").unwrap());
    let _ = owner.sync_checkpoint().unwrap().growth.unwrap();
    progress(&mut owner, &old, b"abcdefgh", 4);
    let copy = f.source.path().join("app.1");
    let _ = fs::copy(&original, &copy).unwrap();
    OpenOptions::new()
        .append(true)
        .open(&original)
        .unwrap()
        .write_all(b"ijklmnop")
        .unwrap();
    assert!(owner.queue_fingerprint(&old, b"abcdefghijklmnop").unwrap());
    let _ = owner.sync_checkpoint().unwrap().growth.unwrap();
    drop(old);
    drop(owner);
    fs::remove_file(&original).unwrap();
    let mut owner = f.owner(&inputs);
    let copied = commit(&mut owner, &copy, Context::Unrelated);
    assert_ne!(copied.file_id, old_id);
    assert_eq!(copied.offset, 0);
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&old_id)
            .unwrap()
            .committed_offset,
        4
    );
}

/// Scenario: Two live readers and speculative new-file plans coexist with an inactive identity that needs net-zero replacement.
/// Guarantees: The admission reserve lets replacement reach selection; rejected speculative plans cannot consume every recovery lease.
#[test]
fn live_readers_and_speculative_plans_leave_replacement_headroom() {
    let f = Fixture::new();
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let a = f.write("a", b"oldA");
    let b = f.write("b", b"oldB");
    let c = f.write("c", b"oldC");
    let live_a = commit(&mut owner, &a, Context::Unrelated);
    let live_b = commit(&mut owner, &b, Context::Unrelated);
    let old_c = commit(&mut owner, &c, Context::Unrelated);
    let old_id = old_c.file_id;
    drop(old_c);
    let d = f.write("d", b"newD");
    let e = f.write("e", b"newE");
    let first = ready(prepare(&mut owner, &d, Context::Unrelated).unwrap());
    let second = ready(prepare(&mut owner, &e, Context::Unrelated).unwrap());
    fs::write(&c, b"newC").unwrap();
    let replacement = ready(prepare(&mut owner, &c, Context::Unrelated).unwrap());
    assert!(
        matches!(replacement.decision(), Decision::Register { superseded: Some(id), .. } if id == old_id)
    );
    let selection = owner
        .select_batch(vec![first, second, replacement])
        .unwrap();
    assert_eq!(selection.ready.len(), 2);
    assert_eq!(selection.needs_capacity.len(), 1);
    drop(selection.needs_capacity);
    let admitted = owner.commit_batch(selection.ready).unwrap();
    assert_eq!(owner.state().unwrap().records().len(), 4);
    assert!(owner.state().unwrap().get(&live_a.file_id).is_some());
    assert!(owner.state().unwrap().get(&live_b.file_id).is_some());
    assert!(owner.state().unwrap().get(&old_id).is_none());
    assert_eq!(admitted.len(), 2);
    let extra = f.write("extra", b"newF");
    assert!(matches!(
        prepare(&mut owner, &extra, Context::Unrelated).unwrap(),
        Admission::NeedsCapacity
    ));
}

/// Scenario: The required progress barrier fails before queued growth is attempted.
/// Guarantees: Required sync failure stays an outer terminal error, never an apparently successful SyncOutcome.
#[test]
fn required_sync_failure_remains_terminal() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let reader = commit(&mut owner, &path, Context::Unrelated);
    fs::write(&path, b"abc").unwrap();
    assert!(owner.queue_fingerprint(&reader, b"abc").unwrap());
    let _ = owner
        .append_checkpoint(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: reader.file_id,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 3,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"abc")
                    .unwrap(),
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })],
            Durability::Deferred,
        )
        .unwrap();
    owner.store.inject_sync_failure("explicit file sync");
    assert!(matches!(
        owner.sync_checkpoint(),
        Err(IdentityError::Store(StoreError::SyncFailed { .. }))
    ));
    assert!(owner.is_stopped());
    assert_eq!(owner.synced_sequence(), 1);
    assert_eq!(owner.growth.len(), 1);
    assert!(matches!(
        owner.resolve_pending(),
        Err(IdentityError::Stopped)
    ));
}

/// Scenario: A full durable table receives old and eligible unrelated files, plus old associated and recognized replacement files.
/// Guarantees: Only unrelated old files are ignored; eligible, fingerprint/path-associated and recognized replacement files still require capacity without mutating state.
#[test]
fn full_capacity_preserves_age_and_association_precedence() {
    let f = Fixture::new();
    let inputs = f.inputs(|c| {
        c.limits.max_tracked_files = 1;
        c.limits.max_open_files = 1;
        c.ignore_older_than = Duration::from_secs(10);
    });
    let mut owner = f.owner(&inputs);
    let path = f.write("app", b"tracked");
    let reader = commit(&mut owner, &path, Context::Replacement);
    let set_age = |path: &Path, seconds| {
        File::open(path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(now() - Duration::from_secs(seconds)))
            .unwrap();
    };
    let unrelated = f.write("unrelated", b"different");
    set_age(&unrelated, 11);
    assert!(matches!(
        prepare(&mut owner, &unrelated, Context::Unrelated).unwrap(),
        Admission::Ignored
    ));
    set_age(&unrelated, 10);
    assert!(matches!(
        prepare(&mut owner, &unrelated, Context::Unrelated).unwrap(),
        Admission::NeedsCapacity
    ));
    set_age(&unrelated, 11);
    assert!(matches!(
        prepare(&mut owner, &unrelated, Context::Replacement).unwrap(),
        Admission::NeedsCapacity
    ));
    let copy = f.write("copy", b"tracked copy");
    set_age(&copy, 11);
    assert!(matches!(
        prepare(&mut owner, &copy, Context::Unrelated).unwrap(),
        Admission::NeedsCapacity
    ));
    fs::rename(&path, f.source.path().join("archive")).unwrap();
    fs::write(&path, b"new stream").unwrap();
    set_age(&path, 11);
    assert!(matches!(
        prepare(&mut owner, &path, Context::Unrelated).unwrap(),
        Admission::NeedsCapacity
    ));
    assert_eq!(owner.state().unwrap().last_sequence(), 1);
    assert_eq!(owner.state().unwrap().records().len(), 1);
    assert!(owner.state().unwrap().get(&reader.file_id).is_some());
    assert_eq!(f.registry.len().unwrap(), 1);
}

/// Scenario: Same-locator reset under each mismatch policy completes directly or after a partial checkpoint write.
/// Guarantees: Reader and quarantine results preserve the original reset decision and superseded ID exactly once, including after pending resolution.
#[test]
fn completion_preserves_decision_through_pending_resolution() {
    for policy in [
        RecoveryMismatch::Beginning,
        RecoveryMismatch::SkipToEnd,
        RecoveryMismatch::Fail,
    ] {
        for pending in [false, true] {
            let f = Fixture::new();
            let inputs = f.inputs(|c| c.identity.on_recovery_mismatch = policy);
            let mut owner = f.owner(&inputs);
            let path = f.write("app", b"old stream");
            let reader = commit(&mut owner, &path, Context::Unrelated);
            assert!(matches!(
                reader.decision,
                Decision::Register {
                    reset: false,
                    superseded: None,
                    ..
                }
            ));
            let old = reader.file_id;
            drop(reader);
            fs::write(&path, b"replacement").unwrap();
            let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
            let expected = plan.decision();
            assert!(
                matches!(expected, Decision::Register { reset: true, superseded: Some(id), .. } if id == old)
            );
            let result = if pending {
                owner.store.inject_write_failure_after(12);
                assert!(matches!(
                    owner.commit(plan),
                    Err(IdentityError::PendingWrite(_))
                ));
                assert_eq!(f.registry.len().unwrap(), 1);
                let mut resolved = owner.resolve_pending().unwrap();
                assert_eq!(resolved.admissions.len(), 1);
                assert!(matches!(
                    owner.resolve_pending(),
                    Err(IdentityError::NoPendingWrite)
                ));
                resolved.admissions.pop().unwrap()
            } else {
                owner.commit(plan).unwrap()
            };
            assert_eq!(result.decision(), expected);
            match &result {
                CommittedAdmission::Reader(seed) => {
                    assert_ne!(policy, RecoveryMismatch::Fail);
                    assert_eq!(seed.decision, expected);
                }
                CommittedAdmission::Quarantined { release_error, .. } => {
                    assert_eq!(policy, RecoveryMismatch::Fail);
                    assert!(release_error.is_none());
                }
            }
            assert_eq!(owner.state().unwrap().last_sequence(), 2);
            assert!(!owner.has_pending_write());
            drop(result);
            assert!(f.registry.is_empty().unwrap());
        }
    }
}

/// Scenario: Separate production callers request the process lease registry.
/// Guarantees: Both handles share the same registry without creating independent ownership domains or modifying global lease state.
#[test]
fn production_registry_is_shared() {
    let first = lease::LeaseRegistry::process();
    let second = lease::LeaseRegistry::process();
    assert!(Arc::ptr_eq(&first, &second));
}
