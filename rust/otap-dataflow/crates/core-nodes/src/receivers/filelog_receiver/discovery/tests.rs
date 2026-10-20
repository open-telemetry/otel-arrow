// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::probe::{pinned_path, verify_path};
use super::*;
use crate::receivers::filelog_receiver::source::PinnedSource;
use std::fs::{self, File};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::atomic::{AtomicBool, Ordering};
use tempfile::{TempDir, tempdir};

struct Policy {
    recursive: bool,
    exclude_secret: bool,
}
impl MatchPolicy for Policy {
    fn includes(&self, _: &Path) -> bool {
        true
    }
    fn may_descend(&self, _: &Path) -> bool {
        self.recursive
    }
    fn excludes(&self, path: &Path) -> bool {
        self.exclude_secret && path.extension().is_some_and(|ext| ext == "secret")
    }
    fn excludes_subtree(&self, _: &Path) -> bool {
        false
    }
    fn excludes_mapped(&self, _: usize, suffix: &Path) -> bool {
        self.excludes(suffix)
    }
}

fn config(root: &Path) -> ScanConfig {
    ScanConfig {
        include_roots: vec![root.to_owned()],
        exclusion_roots: vec![],
        symlinks: SymlinkPolicy::Reject,
        checkpoint_namespace: FileLocator {
            device: u64::MAX,
            inode: u64::MAX,
        },
        limits: ScanLimits::default(),
    }
}
fn scanner(root: &Path, recursive: bool) -> Scanner<Policy> {
    Scanner::new(
        config(root),
        Policy {
            recursive,
            exclude_secret: true,
        },
    )
    .expect("valid setup")
}
fn run(
    scanner: &mut Scanner<Policy>,
) -> (
    ScanCompletion,
    Vec<(PathBuf, Option<ScanReason>, Option<FileLocator>, Vec<u8>)>,
) {
    let mut observations = Vec::new();
    let completion = scanner.scan(
        7,
        &ProbeBudget::default(),
        || false,
        |batch| {
            observations.extend(
                batch
                    .iter()
                    .filter(|o| o.reason != Some(ScanReason::Directory))
                    .map(|o| (o.path.clone(), o.reason, o.locator, o.fingerprint.clone())),
            );
            true
        },
    );
    (completion, observations)
}
fn fixture() -> TempDir {
    tempdir().expect("temporary fixture")
}

/// Scenario: A stable directory is scanned repeatedly with a bounded fingerprint.
/// Guarantees: Candidates are descriptor-free, preserve bytes, and end in ordered complete passes.
#[test]
fn stable_candidates() {
    let dir = fixture();
    fs::write(dir.path().join("app.log"), b"hello world").unwrap();
    let mut scanner = scanner(dir.path(), false);
    for _ in 0..3 {
        let (end, observations) = run(&mut scanner);
        assert!(end.complete);
        assert_eq!(end.generation, 7);
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].1, None);
        assert_eq!(observations[0].3, b"hello world");
    }
}

/// Scenario: A batch consumer stops accepting work.
/// Guarantees: Cancellation is incomplete and all transient reservations are released before delivery.
#[test]
fn consumer_cancellation_releases_probes() {
    let dir = fixture();
    fs::write(dir.path().join("log"), b"data").unwrap();
    let budget = ProbeBudget::default();
    let end = scanner(dir.path(), false).scan(
        1,
        &budget,
        || false,
        |_| {
            assert!(budget.try_reserve().is_some());
            false
        },
    );
    assert!(!end.complete);
    assert!(end.cancelled);
    assert!(budget.try_reserve().is_some());
}

/// Scenario: Cancellation is already requested before root resolution.
/// Guarantees: No observation is fabricated and the completion marker is incomplete.
#[test]
fn early_cancellation() {
    let dir = fixture();
    let end = scanner(dir.path(), false).scan(
        1,
        &ProbeBudget::default(),
        || true,
        |_| panic!("no batch"),
    );
    assert!(!end.complete);
    assert!(end.cancelled);
}

/// Scenario: A reader reopen owns the shared conversion allowance.
/// Guarantees: Discovery does not open another probe and cannot report a complete pass.
#[test]
fn shared_budget_contention() {
    let dir = fixture();
    fs::write(dir.path().join("log"), b"data").unwrap();
    let budget = ProbeBudget::default();
    let reservation = budget.try_reserve().unwrap();
    assert!(budget.try_reserve().is_none());
    let end = scanner(dir.path(), false).scan(
        1,
        &budget,
        || false,
        |batch| {
            assert!(batch.iter().all(|o| matches!(
                o.reason,
                Some(ScanReason::ProbeBusy | ScanReason::Directory)
            )));
            true
        },
    );
    assert!(!end.complete);
    drop(reservation);
    assert!(budget.try_reserve().is_some());
}

/// Scenario: An inspected excluded source has no read permission.
/// Guarantees: Lexical exclusion preserves locator presence without attempting read-open.
#[test]
fn lexical_exclusion_needs_no_content_access() {
    let dir = fixture();
    let path = dir.path().join("private.secret");
    fs::write(&path, b"secret").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
    let (end, observations) = run(&mut scanner(dir.path(), false));
    assert!(end.complete);
    assert_eq!(observations[0].1, Some(ScanReason::Excluded));
    assert!(observations[0].2.is_some());
    assert!(observations[0].3.is_empty());
}

/// Scenario: An eligible alias points at an excluded target that cannot be read.
/// Guarantees: Pin-bound target exclusion runs before read-open and preserves presence.
#[test]
fn target_exclusion_precedes_read_open() {
    let dir = fixture();
    let target = fixture();
    let path = target.path().join("private.secret");
    fs::write(&path, b"secret").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
    symlink(&path, dir.path().join("alias.log")).unwrap();
    let mut cfg = config(dir.path());
    cfg.symlinks = SymlinkPolicy::Follow;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: false,
            exclude_secret: true,
        },
    )
    .unwrap());
    assert!(end.complete);
    assert_eq!(observations[0].1, Some(ScanReason::Excluded));
}

/// Scenario: Non-followed final links are dangling or cyclic.
/// Guarantees: Rejecting their type does not attempt target resolution or invalidate coverage.
#[test]
fn nonfollowed_links() {
    let dir = fixture();
    symlink("missing", dir.path().join("dangling")).unwrap();
    symlink("cycle", dir.path().join("cycle")).unwrap();
    let (end, observations) = run(&mut scanner(dir.path(), false));
    assert!(end.complete);
    assert!(
        observations
            .iter()
            .all(|o| o.1 == Some(ScanReason::Unsupported))
    );
}

/// Scenario: Followed leaf links dangle or loop, while another file is readable.
/// Guarantees: Leaf-local misses/rejections do not hide the valid candidate or fail coverage.
#[test]
fn followed_leaf_failures() {
    let dir = fixture();
    symlink("missing/parent/log", dir.path().join("dangling")).unwrap();
    symlink("cycle", dir.path().join("cycle")).unwrap();
    fs::write(dir.path().join("log"), b"ok").unwrap();
    let mut cfg = config(dir.path());
    cfg.symlinks = SymlinkPolicy::Follow;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: false,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(end.complete);
    assert!(
        observations
            .iter()
            .any(|o| o.1 == Some(ScanReason::LeafLoop))
    );
    assert!(
        observations
            .iter()
            .any(|o| o.1 == Some(ScanReason::Missing))
    );
    assert!(observations.iter().any(|o| o.1.is_none()));
}

/// Scenario: A recursive include could need the subtree behind a looping link.
/// Guarantees: Unknown required subtree coverage is incomplete, unlike a leaf-only rejection.
#[test]
fn recursive_loop_fails_coverage() {
    let dir = fixture();
    symlink("cycle", dir.path().join("cycle")).unwrap();
    let mut cfg = config(dir.path());
    cfg.symlinks = SymlinkPolicy::Follow;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(!end.complete);
    assert!(
        observations
            .iter()
            .any(|o| o.1 == Some(ScanReason::Coverage))
    );
}

/// Scenario: A followed directory link points to an ancestor.
/// Guarantees: The cycle is reported without unbounded recursion or retained directory handles.
#[test]
fn ancestor_cycle() {
    let dir = fixture();
    symlink(dir.path(), dir.path().join("again")).unwrap();
    let mut cfg = config(dir.path());
    cfg.symlinks = SymlinkPolicy::Follow;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(!end.complete);
    assert!(observations.iter().any(|o| o.1 == Some(ScanReason::Cycle)));
}

/// Scenario: A required subtree is deeper than the configured stack bound.
/// Guarantees: Depth exhaustion is explicit and cannot produce complete coverage.
#[test]
fn depth_bound() {
    let dir = fixture();
    fs::create_dir(dir.path().join("child")).unwrap();
    let mut cfg = config(dir.path());
    cfg.limits.depth = 1;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(!end.complete);
    assert_eq!(observations[0].1, Some(ScanReason::Limit));
}

/// Scenario: A root is absent, appears, then is replaced by another directory.
/// Guarantees: First validation establishes a baseline; replacement never silently adopts a new one.
#[test]
fn root_baseline() {
    let dir = fixture();
    let root = dir.path().join("root");
    let mut scanner = scanner(&root, false);
    assert!(!run(&mut scanner).0.complete);
    fs::create_dir(&root).unwrap();
    fs::write(root.join("log"), b"ok").unwrap();
    assert!(run(&mut scanner).0.complete);
    fs::rename(&root, dir.path().join("old")).unwrap();
    fs::create_dir(&root).unwrap();
    let (end, observations) = run(&mut scanner);
    assert!(!end.complete);
    assert_eq!(observations[0].1, Some(ScanReason::RootChanged));
    fs::write(root.join("new.log"), b"new root contents").unwrap();
    let (end, observations) = run(&mut scanner);
    assert!(!end.complete);
    assert!(
        observations
            .iter()
            .any(|o| o.0.ends_with("new.log") && o.1.is_none())
    );
}

/// Scenario: A large stable directory spans multiple batches and nested directories.
/// Guarantees: Resumption visits every stable file exactly once within bounded batches.
#[test]
fn bounded_resumption() {
    let dir = fixture();
    for i in 0..60 {
        fs::write(dir.path().join(format!("log-{i}")), b"ok").unwrap();
    }
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/nested"), b"nested").unwrap();
    let mut cfg = config(dir.path());
    cfg.limits.batch = 3;
    let mut scanner = Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap();
    let mut paths = std::collections::HashSet::new();
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |batch| {
            assert!(batch.len() <= 3);
            for observation in batch {
                if observation.reason == Some(ScanReason::Directory) {
                    continue;
                }
                assert!(observation.reason.is_none(), "{observation:?}");
                assert!(paths.insert(observation.path.clone()));
            }
            true
        },
    );
    assert!(end.complete);
    assert_eq!(paths.len(), 61);
}

/// Scenario: A directory changes while delivery pauses enumeration.
/// Guarantees: Reopen does not use a stale cookie to certify complete coverage.
#[test]
fn mutation_during_delivery() {
    let dir = fixture();
    for i in 0..4 {
        fs::write(dir.path().join(format!("log-{i}")), b"ok").unwrap();
    }
    let mut cfg = config(dir.path());
    cfg.limits.batch = 1;
    let mut scanner = Scanner::new(
        cfg,
        Policy {
            recursive: false,
            exclude_secret: false,
        },
    )
    .unwrap();
    let mut changed = false;
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |_| {
            if !changed {
                fs::write(dir.path().join("new"), b"new").unwrap();
                changed = true;
            }
            true
        },
    );
    assert!(!end.complete);
}

/// Scenario: A live file literally ends with the procfs deleted suffix.
/// Guarantees: Policy validates the actual path rather than applying a text heuristic.
#[test]
fn live_deleted_suffix() {
    let dir = fixture();
    fs::write(dir.path().join("log (deleted)"), b"live").unwrap();
    let (end, observations) = run(&mut scanner(dir.path(), false));
    assert!(end.complete);
    assert!(observations[0].1.is_none());
}

/// Scenario: A procfs alias reaches an unlinked file still held open by its writer.
/// Guarantees: Zero links do not bypass unverifiable target policy; locator presence survives.
#[test]
fn unlinked_alias_fails_policy() {
    use std::os::fd::AsRawFd;
    let dir = fixture();
    let target = dir.path().join("target");
    fs::write(&target, b"deleted").unwrap();
    let held = File::open(&target).unwrap();
    fs::remove_file(&target).unwrap();
    symlink(
        format!("/proc/self/fd/{}", held.as_raw_fd()),
        dir.path().join("alias"),
    )
    .unwrap();
    let mut cfg = config(dir.path());
    cfg.symlinks = SymlinkPolicy::Follow;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: false,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(!end.complete);
    assert_eq!(observations[0].1, Some(ScanReason::Policy));
    assert!(observations[0].2.is_some());
}

/// Scenario: An intermediate directory is replaced with a symlink after pinning.
/// Guarantees: Target verification refuses intermediate-component symlinks even if the locator matches.
#[test]
fn intermediate_symlink_verification() {
    let dir = fixture();
    let parent = dir.path().join("parent");
    fs::create_dir(&parent).unwrap();
    let target = parent.join("log");
    fs::write(&target, b"ok").unwrap();
    let locator = FileLocator::from_metadata(&fs::metadata(&target).unwrap());
    fs::rename(&parent, dir.path().join("moved")).unwrap();
    symlink(dir.path().join("moved"), &parent).unwrap();
    assert_eq!(
        verify_path(&target, locator).unwrap_err().raw_os_error(),
        Some(libc::ELOOP)
    );
}

/// Scenario: The kernel path exceeds the supplied bounded readlink buffer.
/// Guarantees: Truncated path text is never interpreted as target-policy evidence.
#[test]
fn truncated_target_path() {
    let dir = fixture();
    fs::write(dir.path().join("log"), b"ok").unwrap();
    let parent = File::open(dir.path()).unwrap();
    let pin =
        PinnedSource::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || false).unwrap();
    assert_eq!(
        pinned_path(pin.file(), &mut [0; 4])
            .unwrap_err()
            .raw_os_error(),
        Some(libc::ENAMETOOLONG)
    );
}

/// Scenario: The checkpoint namespace lies inside a recursive include.
/// Guarantees: Its identity excludes the namespace without excluding allowed sibling logs.
#[test]
fn namespace_identity_exclusion() {
    let dir = fixture();
    let state = dir.path().join("namespace");
    fs::create_dir(&state).unwrap();
    fs::write(state.join("checkpoint.db"), b"private").unwrap();
    fs::write(dir.path().join("app.log"), b"ok").unwrap();
    let mut cfg = config(dir.path());
    cfg.checkpoint_namespace = FileLocator::from_metadata(&fs::metadata(&state).unwrap());
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(end.complete);
    assert_eq!(observations.len(), 1);
    assert!(observations[0].0.ends_with("app.log"));
}

/// Scenario: Cancellation arrives between filesystem operations.
/// Guarantees: Cancellation after pinning releases the pin and its shared reservation.
#[test]
fn cancelled_during_probe() {
    let dir = fixture();
    fs::write(dir.path().join("log"), b"ok").unwrap();
    let budget = ProbeBudget::default();
    let mut probe_checks = 0;
    let end = scanner(dir.path(), false).scan(
        1,
        &budget,
        || {
            if budget.try_reserve().is_none() {
                probe_checks += 1;
            }
            probe_checks >= 2
        },
        |_| true,
    );
    assert!(!end.complete);
    assert!(end.cancelled);
    assert!(budget.try_reserve().is_some());
}

/// Scenario: Independent workers contend for the same two-slot allowance.
/// Guarantees: Only one worker holds a conversion reservation at any time.
#[test]
fn budget_is_shared_across_threads() {
    let budget = ProbeBudget::default();
    let held = AtomicBool::new(false);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let budget = &budget;
            let held = &held;
            let _ = scope.spawn(move || {
                for _ in 0..1000 {
                    if let Some(_reservation) = budget.try_reserve() {
                        assert!(!held.swap(true, Ordering::SeqCst));
                        held.store(false, Ordering::SeqCst);
                    }
                }
            });
        }
    });
    assert!(budget.try_reserve().is_some());
}

/// Scenario: A configuration asks for unbounded or invalid scan storage.
/// Guarantees: Construction rejects it before traversal or allocation.
#[test]
fn configuration_bounds() {
    let dir = fixture();
    let mut cfg = config(dir.path());
    cfg.limits.batch = usize::MAX;
    assert!(
        Scanner::new(
            cfg,
            Policy {
                recursive: false,
                exclude_secret: false
            }
        )
        .is_err()
    );
}

/// Scenario: Source content changes without changing its locator between probes.
/// Guarantees: A shrinking or changed fingerprint is never emitted as an eligible candidate.
#[test]
fn content_change_between_probes() {
    use std::cell::Cell;
    struct MutatingPolicy {
        path: PathBuf,
        checks: Cell<usize>,
    }
    impl MatchPolicy for MutatingPolicy {
        fn includes(&self, _: &Path) -> bool {
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            false
        }
        fn excludes(&self, _: &Path) -> bool {
            let n = self.checks.get() + 1;
            self.checks.set(n);
            // Lexical check, first probe's target check, second probe's target check.
            if n == 3 {
                fs::write(&self.path, b"changed!").unwrap();
            }
            false
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
    }
    let dir = fixture();
    let path = dir.path().join("log");
    fs::write(&path, b"original").unwrap();
    let mut scanner = Scanner::new(
        config(dir.path()),
        MutatingPolicy {
            path,
            checks: Cell::new(0),
        },
    )
    .unwrap();
    let mut unstable = false;
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |batch| {
            for o in batch {
                if o.reason == Some(ScanReason::Unstable) {
                    unstable = true;
                    assert!(o.locator.is_some());
                }
                assert!(o.reason.is_some());
            }
            true
        },
    );
    assert!(unstable);
    assert!(!end.complete);
}

/// Scenario: Verification observes a different object at the pinned source path.
/// Guarantees: Locator mismatch fails target verification instead of authorizing content access.
#[test]
fn substitution_after_pin() {
    let dir = fixture();
    let path = dir.path().join("log");
    fs::write(&path, b"old").unwrap();
    let parent = File::open(dir.path()).unwrap();
    let pin =
        PinnedSource::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || false).unwrap();
    fs::rename(&path, dir.path().join("old")).unwrap();
    fs::write(&path, b"new").unwrap();
    assert_eq!(
        verify_path(&path, pin.locator())
            .unwrap_err()
            .raw_os_error(),
        Some(libc::ESTALE)
    );
}

/// Scenario: Scanning delivers a batch after opening and validating source files.
/// Guarantees: Delivery holds no directory or source handles from the watched tree.
#[test]
fn no_handles_during_delivery() {
    let dir = fixture();
    fs::write(dir.path().join("log"), b"ok").unwrap();
    let end = scanner(dir.path(), false).scan(
        1,
        &ProbeBudget::default(),
        || false,
        |_| {
            for entry in fs::read_dir("/proc/self/fd").unwrap().flatten() {
                if let Ok(target) = fs::read_link(entry.path()) {
                    assert!(
                        !target.starts_with(dir.path()),
                        "retained handle: {target:?}"
                    );
                }
            }
            true
        },
    );
    assert!(end.complete);
}

/// Scenario: The OS descriptor limit is exhausted in an isolated subprocess.
/// Guarantees: Discovery reports incomplete coverage and preserves the original EMFILE error.
#[test]
fn os_descriptor_exhaustion() {
    if std::env::var_os("FILELOG_SCAN_EMFILE_CHILD").is_some() {
        let dir = fixture();
        fs::write(dir.path().join("log"), b"ok").unwrap();
        let mut scanner = scanner(dir.path(), false);
        let mut held = Vec::new();
        while let Ok(file) = File::open("/dev/null") {
            held.push(file);
        }
        let mut reported = false;
        let end = scanner.scan(
            1,
            &ProbeBudget::default(),
            || false,
            |batch| {
                reported |= batch.iter().any(|o| o.errno == Some(libc::EMFILE));
                true
            },
        );
        assert!(!end.complete);
        assert!(reported);
        drop(held);
        return;
    }
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg("ulimit -n 32; exec \"$@\"")
        .arg("filelog-test")
        .arg(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "receivers::filelog_receiver::discovery::tests::os_descriptor_exhaustion",
            "--test-threads=1",
        ])
        .env("FILELOG_SCAN_EMFILE_CHILD", "1")
        .status()
        .unwrap();
    assert!(status.success());
}

/// Scenario: An exclusion root is reached through a bind mount, alongside an allowed sibling.
/// Guarantees: Identity mapping evaluates the full suffix pattern and never excludes the whole root.
#[test]
#[ignore = "requires an isolated mount namespace with CAP_SYS_ADMIN; Linux qualification"]
fn bind_mount_exclusion_preserves_allowed_sibling() {
    struct MappedPolicy;
    impl MatchPolicy for MappedPolicy {
        fn includes(&self, _: &Path) -> bool {
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            false
        }
        fn excludes(&self, _: &Path) -> bool {
            false
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, suffix: &Path) -> bool {
            suffix == Path::new("x.secret")
        }
    }
    let dir = fixture();
    let original = dir.path().join("original");
    let alias = dir.path().join("alias");
    fs::create_dir(&original).unwrap();
    fs::create_dir(&alias).unwrap();
    fs::write(original.join("x.secret"), b"secret").unwrap();
    fs::write(original.join("x.log"), b"allowed").unwrap();
    assert!(
        std::process::Command::new("mount")
            .arg("--bind")
            .arg(&original)
            .arg(&alias)
            .status()
            .unwrap()
            .success()
    );
    struct Unmount(PathBuf);
    impl Drop for Unmount {
        fn drop(&mut self) {
            let status = std::process::Command::new("umount")
                .arg(&self.0)
                .status()
                .unwrap();
            assert!(status.success());
        }
    }
    let _unmount = Unmount(alias.clone());
    let mut cfg = config(&alias);
    cfg.exclusion_roots = vec![original];
    let mut scanner = Scanner::new(cfg, MappedPolicy).unwrap();
    let mut excluded = false;
    let mut allowed = false;
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |batch| {
            for o in batch {
                if o.path.ends_with("x.secret") {
                    excluded = o.reason == Some(ScanReason::Excluded);
                }
                if o.path.ends_with("x.log") {
                    allowed = o.reason.is_none();
                }
            }
            true
        },
    );
    assert!(end.complete);
    assert!(excluded);
    assert!(allowed);
    // Cover same-filesystem mount replacement without adopting its new root.
    let replacement = dir.path().join("replacement");
    fs::create_dir(&replacement).unwrap();
    assert!(
        std::process::Command::new("umount")
            .arg(&alias)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::process::Command::new("mount")
            .arg("--bind")
            .arg(&replacement)
            .arg(&alias)
            .status()
            .unwrap()
            .success()
    );
    let mut changed = false;
    let end = scanner.scan(
        2,
        &ProbeBudget::default(),
        || false,
        |batch| {
            changed |= batch
                .iter()
                .any(|o| o.reason == Some(ScanReason::RootChanged));
            true
        },
    );
    assert!(!end.complete);
    assert!(changed);
}

/// Scenario: Stable tmpfs directories require repeated close/reopen while enumerating.
/// Guarantees: The qualified tmpfs resumption path visits every entry without duplication.
#[test]
#[ignore = "requires writable tmpfs at /dev/shm; Linux qualification"]
fn tmpfs_resumption() {
    let dir = tempfile::tempdir_in("/dev/shm").unwrap();
    for i in 0..25 {
        fs::write(dir.path().join(format!("log-{i}")), b"tmpfs").unwrap();
    }
    let mut cfg = config(dir.path());
    cfg.limits.batch = 2;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: false,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(end.complete);
    assert_eq!(observations.len(), 25);
    let paths: std::collections::HashSet<_> = observations.iter().map(|o| &o.0).collect();
    assert_eq!(paths.len(), 25);
}

/// Scenario: A matched filename contains non-UTF-8 native bytes.
/// Guarantees: Traversal, verification, and observation preserve the name without lossy conversion.
#[test]
fn native_filename_round_trip() {
    use std::os::unix::ffi::OsStrExt;
    let dir = fixture();
    let name = std::ffi::OsStr::from_bytes(b"log-\xff");
    fs::write(dir.path().join(name), b"native").unwrap();
    let (end, observations) = run(&mut scanner(dir.path(), false));
    assert!(end.complete);
    assert_eq!(observations[0].0.file_name(), Some(name));
    assert_eq!(observations[0].3, b"native");
}

/// Scenario: A fully excluded subtree cannot be read by the collector.
/// Guarantees: Pruning does not inspect its contents or turn the unreadable excluded directory into lost coverage.
#[test]
fn pruned_subtree_is_not_searched() {
    struct PruningPolicy;
    impl MatchPolicy for PruningPolicy {
        fn includes(&self, _: &Path) -> bool {
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            true
        }
        fn excludes(&self, _: &Path) -> bool {
            false
        }
        fn excludes_subtree(&self, path: &Path) -> bool {
            path.file_name().is_some_and(|name| name == "private")
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
    }
    let dir = fixture();
    let private = dir.path().join("private");
    fs::create_dir(&private).unwrap();
    fs::write(private.join("hidden.log"), b"private").unwrap();
    fs::write(dir.path().join("visible.log"), b"public").unwrap();
    fs::set_permissions(&private, fs::Permissions::from_mode(0o0)).unwrap();
    let mut scanner = Scanner::new(config(dir.path()), PruningPolicy).unwrap();
    let mut candidates = 0;
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |batch| {
            for observation in batch {
                assert!(!observation.path.starts_with(&private));
                candidates += usize::from(observation.reason.is_none());
            }
            true
        },
    );
    fs::set_permissions(&private, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(end.complete);
    assert_eq!(candidates, 1);
}

/// Scenario: Procfs exposes a pinned regular file outside the collector's root.
/// Guarantees: The path cannot pass target verification merely because the open file's locator is known.
#[test]
#[ignore = "requires an isolated mount namespace with CAP_SYS_ADMIN and CAP_SYS_CHROOT; Linux qualification"]
fn target_outside_collector_root() {
    if let Some(jail) = std::env::var_os("FILELOG_SCAN_JAIL") {
        let parent = File::open(std::env::var_os("FILELOG_SCAN_OUTSIDE").unwrap()).unwrap();
        let pin =
            PinnedSource::open_at(&parent, c"log", SymlinkPolicy::Reject, None, || false).unwrap();
        nix::unistd::chroot(Path::new(&jail)).unwrap();
        std::env::set_current_dir("/").unwrap();
        let mut buffer = [0; 4096];
        let result =
            pinned_path(pin.file(), &mut buffer).and_then(|path| verify_path(path, pin.locator()));
        assert!(result.is_err());
        return;
    }
    let dir = fixture();
    let outside = dir.path().join("outside");
    let jail = dir.path().join("jail");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("log"), b"outside").unwrap();
    fs::create_dir(&jail).unwrap();
    let proc = jail.join("proc");
    fs::create_dir(&proc).unwrap();
    assert!(
        std::process::Command::new("mount")
            .arg("--bind")
            .arg("/proc")
            .arg(&proc)
            .status()
            .unwrap()
            .success()
    );
    struct Unmount(PathBuf);
    impl Drop for Unmount {
        fn drop(&mut self) {
            assert!(
                std::process::Command::new("umount")
                    .arg(&self.0)
                    .status()
                    .unwrap()
                    .success()
            );
        }
    }
    let _unmount = Unmount(proc);
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "receivers::filelog_receiver::discovery::tests::target_outside_collector_root",
            "--ignored",
            "--test-threads=1",
        ])
        .env("FILELOG_SCAN_JAIL", jail)
        .env("FILELOG_SCAN_OUTSIDE", outside)
        .status()
        .unwrap();
    assert!(status.success());
}

/// Scenario: A listed regular file is replaced by a directory before source pinning.
/// Guarantees: Detected type substitution is unstable evidence, not harmless non-regular rejection.
#[test]
fn listed_file_changes_type_before_pin() {
    use std::cell::Cell;
    struct ChangingPolicy {
        changed: Cell<bool>,
    }
    impl MatchPolicy for ChangingPolicy {
        fn includes(&self, path: &Path) -> bool {
            if !self.changed.replace(true) {
                fs::remove_file(path).unwrap();
                fs::create_dir(path).unwrap();
            }
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            false
        }
        fn excludes(&self, _: &Path) -> bool {
            false
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
    }
    let dir = fixture();
    fs::write(dir.path().join("log"), b"old").unwrap();
    let mut scanner = Scanner::new(
        config(dir.path()),
        ChangingPolicy {
            changed: Cell::new(false),
        },
    )
    .unwrap();
    let mut unstable = false;
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |batch| {
            unstable |= batch.iter().any(|o| o.reason == Some(ScanReason::Unstable));
            assert!(batch.iter().all(|o| o.reason.is_some()));
            true
        },
    );
    assert!(!end.complete);
    assert!(unstable);
}

/// Scenario: A followed link may hide a required subtree but its target is absent.
/// Guarantees: Unknown required coverage cannot be reported as a leaf miss or a complete pass.
#[test]
fn dangling_required_subtree_is_incomplete() {
    let dir = fixture();
    symlink("absent/subtree", dir.path().join("pod")).unwrap();
    let mut cfg = config(dir.path());
    cfg.symlinks = SymlinkPolicy::Follow;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(!end.complete);
    assert_eq!(observations[0].1, Some(ScanReason::Coverage));
}

/// Scenario: An excluded link's external target becomes a directory between its two metadata lookups.
/// Guarantees: Skipped required children invalidate coverage even when the watched directory itself is unchanged.
#[test]
fn excluded_target_changes_into_directory() {
    struct ChangingExclusion {
        target: PathBuf,
    }
    impl MatchPolicy for ChangingExclusion {
        fn includes(&self, _: &Path) -> bool {
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            true
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
        fn excludes(&self, _: &Path) -> bool {
            fs::remove_file(&self.target).unwrap();
            fs::create_dir(&self.target).unwrap();
            fs::write(
                self.target.join("child.log"),
                b"must not be silently skipped",
            )
            .unwrap();
            true
        }
    }
    let watched = fixture();
    let external = fixture();
    let target = external.path().join("target");
    fs::write(&target, b"old file").unwrap();
    symlink(&target, watched.path().join("link.secret")).unwrap();
    let mut cfg = config(watched.path());
    cfg.symlinks = SymlinkPolicy::Follow;
    let mut scanner = Scanner::new(cfg, ChangingExclusion { target }).unwrap();
    let mut unstable = false;
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |batch| {
            unstable |= batch.iter().any(|o| o.reason == Some(ScanReason::Unstable));
            true
        },
    );
    assert!(unstable);
    assert!(!end.complete);
}

/// Scenario: The first probe succeeds but the second pin fails or is cancelled.
/// Guarantees: No stale locator, size, mtime, or fingerprint is exported as consistent evidence.
#[test]
fn failed_second_pin_clears_candidate_evidence() {
    use super::probe::{ProbeContext, ProbeScratch, probe_with_open};
    use crate::receivers::filelog_receiver::source::FileAccessError;
    let dir = fixture();
    fs::write(dir.path().join("log"), b"first observation").unwrap();
    let parent = File::open(dir.path()).unwrap();
    let cfg = config(dir.path());
    let budget = ProbeBudget::default();
    let policy = Policy {
        recursive: false,
        exclude_secret: false,
    };
    let context = ProbeContext {
        config: &cfg,
        matcher: &policy,
        roots: &[],
        budget: &budget,
        policy_available: true,
    };
    for errno in [libc::EACCES, libc::EMFILE, libc::ECANCELED] {
        let mut out = Observation {
            path: dir.path().join("log"),
            locator: None,
            size: None,
            modified: None,
            fingerprint: Vec::new(),
            reason: Some(ScanReason::ProbeBusy),
            errno: None,
        };
        let mut calls = 0;
        probe_with_open(
            &context,
            &parent,
            c"log",
            true,
            false,
            ProbeRequest::Full,
            &mut out,
            &mut ProbeScratch::new(cfg.limits),
            &mut || false,
            |directory, name, links, cancelled| {
                calls += 1;
                if calls == 2 {
                    return Err(if errno == libc::ECANCELED {
                        FileAccessError::Cancelled
                    } else {
                        FileAccessError::Io(std::io::Error::from_raw_os_error(errno))
                    });
                }
                PinnedSource::open_at(directory, name, links, None, cancelled)
            },
        );
        assert_eq!(calls, 2);
        assert!(out.reason.is_some());
        assert!(out.locator.is_none());
        assert!(out.size.is_none());
        assert!(out.modified.is_none());
        assert!(out.fingerprint.is_empty());
    }
}

struct IndexedMapping;
impl MatchPolicy for IndexedMapping {
    fn includes(&self, _: &Path) -> bool {
        true
    }
    fn may_descend(&self, _: &Path) -> bool {
        false
    }
    fn excludes(&self, _: &Path) -> bool {
        false
    }
    fn excludes_subtree(&self, _: &Path) -> bool {
        false
    }
    fn excludes_mapped(&self, index: usize, suffix: &Path) -> bool {
        index == 1 && suffix == Path::new("x.secret")
    }
}

/// Scenario: A missing literal exclusion prefix precedes a live mapping, which is subsequently recreated.
/// Guarantees: Missing prefixes preserve matcher indexes, and fresh mappings resume exclusion without restarting discovery.
#[test]
fn exclusion_mappings_are_per_pass_and_keep_indexes() {
    let dir = fixture();
    let watched = dir.path().join("watched");
    let excluded = dir.path().join("excluded");
    fs::create_dir(&watched).unwrap();
    fs::create_dir(&excluded).unwrap();
    fs::write(excluded.join("x.secret"), b"secret").unwrap();
    fs::write(watched.join("allowed.log"), b"allowed").unwrap();
    symlink(excluded.join("x.secret"), watched.join("alias.log")).unwrap();
    let mut cfg = config(&watched);
    cfg.symlinks = SymlinkPolicy::Follow;
    cfg.exclusion_roots = vec![dir.path().join("not-created/yet"), excluded.clone()];
    let mut scanner = Scanner::new(cfg, IndexedMapping).unwrap();
    for pass in 1..=2 {
        let mut excluded_seen = false;
        let mut allowed_seen = false;
        let end = scanner.scan(
            pass,
            &ProbeBudget::default(),
            || false,
            |batch| {
                excluded_seen |= batch.iter().any(|o| {
                    o.path.ends_with("alias.log") && o.reason == Some(ScanReason::Excluded)
                });
                allowed_seen |= batch
                    .iter()
                    .any(|o| o.path.ends_with("allowed.log") && o.reason.is_none());
                true
            },
        );
        assert!(end.complete);
        assert!(excluded_seen && allowed_seen);
        if pass == 1 {
            fs::rename(&excluded, dir.path().join("old")).unwrap();
            fs::create_dir(&excluded).unwrap();
            fs::write(excluded.join("x.secret"), b"new secret").unwrap();
        }
    }
}

/// Scenario: An exclusion prefix fails because an intermediate alias dangles.
/// Guarantees: An unresolved alias is not mistaken for an absent literal directory with an evaluable mapping.
#[test]
fn dangling_exclusion_alias_fails_closed() {
    let dir = fixture();
    let watched = dir.path().join("watched");
    fs::create_dir(&watched).unwrap();
    fs::write(watched.join("log"), b"data").unwrap();
    symlink("absent", dir.path().join("broken")).unwrap();
    let mut cfg = config(&watched);
    cfg.exclusion_roots = vec![dir.path().join("broken/child")];
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: false,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(!end.complete);
    assert!(observations.iter().all(|o| o.1.is_some()));
}

/// Scenario: The caller requests only presence for a known file, then the name is replaced.
/// Guarantees: The fast path emits no candidate metadata or content; a changed locator receives full validation.
#[test]
fn presence_only_is_distinct_and_replacement_gets_full_probe() {
    let dir = fixture();
    let path = dir.path().join("log");
    fs::write(&path, b"original").unwrap();
    let expected = FileLocator::from_metadata(&fs::metadata(&path).unwrap());
    let mut scanner = scanner(dir.path(), false);
    let end = scanner.scan_with_requests(
        1,
        &ProbeBudget::default(),
        |_| ProbeRequest::PresenceOnly { expected },
        || false,
        |batch| {
            for o in batch
                .iter()
                .filter(|o| o.reason != Some(ScanReason::Directory))
            {
                assert_eq!(o.reason, Some(ScanReason::PresenceOnly));
                assert!(o.fingerprint.is_empty());
                assert!(o.size.is_none());
                assert!(o.modified.is_none());
            }
            true
        },
    );
    assert!(end.complete);
    fs::rename(&path, dir.path().join("previous")).unwrap();
    fs::write(&path, b"replacement").unwrap();
    let mut replacement = false;
    let end = scanner.scan_with_requests(
        2,
        &ProbeBudget::default(),
        |_| ProbeRequest::PresenceOnly { expected },
        || false,
        |batch| {
            replacement |= batch
                .iter()
                .any(|o| o.path == path && o.reason.is_none() && o.fingerprint == b"replacement");
            true
        },
    );
    assert!(end.complete);
    assert!(replacement);
}

/// Scenario: Acyclic directory aliases branch repeatedly toward the same descendants.
/// Guarantees: Total traversal work stops at the explicit limit without fabricating complete coverage.
#[test]
fn branching_alias_work_is_bounded() {
    let dir = fixture();
    for i in 0..12 {
        fs::create_dir(dir.path().join(format!("level-{i}"))).unwrap();
    }
    for i in 0..11 {
        for alias in ["a", "b"] {
            symlink(
                dir.path().join(format!("level-{}", i + 1)),
                dir.path().join(format!("level-{i}")).join(alias),
            )
            .unwrap();
        }
    }
    let mut cfg = config(&dir.path().join("level-0"));
    cfg.symlinks = SymlinkPolicy::Follow;
    cfg.limits.work_units = 64;
    let (end, _) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap());
    assert!(!end.complete);
    assert!(!end.cancelled);
    assert!(end.observations < 100);
}

/// Scenario: A required verification operation is unavailable, or its initial probe hits FD pressure.
/// Guarantees: One clear diagnostic replaces per-root errors; transient exhaustion can recover on the next pass.
#[test]
fn capability_failure_is_clear_and_transient_pressure_recovers() {
    let dir = fixture();
    fs::write(dir.path().join("log"), b"ok").unwrap();
    let mut scanner = scanner(dir.path(), false);
    scanner.inject_capability_result(Err(libc::ENOSYS));
    for _ in 0..2 {
        let (end, observations) = run(&mut scanner);
        assert!(!end.complete);
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].1, Some(ScanReason::VerificationUnavailable));
    }
    scanner.inject_capability_result(Err(libc::EMFILE));
    assert!(!run(&mut scanner).0.complete);
    assert!(run(&mut scanner).0.complete);
}

fn nested_fixture(root: &Path) -> usize {
    let mut count = 0;
    for pod in 0..6 {
        let parent = root.join(format!(
            "pod-{pod}/namespace/container/instance/current/logs"
        ));
        fs::create_dir_all(&parent).unwrap();
        for file in 0..12 {
            fs::write(parent.join(format!("{file}.log")), b"nested").unwrap();
            count += 1;
        }
    }
    count
}

/// Scenario: A six-level tree spans many sibling subtrees and batches.
/// Guarantees: Qualified close/reopen resumption reaches every stable file exactly once.
#[test]
fn nested_tree_resumption() {
    let dir = fixture();
    let expected = nested_fixture(dir.path());
    let mut cfg = config(dir.path());
    cfg.limits.batch = 3;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap());
    let paths: std::collections::HashSet<_> = observations
        .iter()
        .filter(|o| o.1.is_none())
        .map(|o| &o.0)
        .collect();
    assert!(end.complete);
    assert_eq!(paths.len(), expected);
    assert_eq!(observations.len(), expected);
}

/// Scenario: Root metadata changes during a paused recursive pass while static files remain.
/// Guarantees: Uncertainty prevents complete coverage but does not discard remaining positive traversal work.
#[test]
fn churn_preserves_positive_progress_without_completeness() {
    let dir = fixture();
    let expected = nested_fixture(dir.path());
    let mut cfg = config(dir.path());
    cfg.limits.batch = 3;
    let mut scanner = Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap();
    let mut changed = false;
    let mut files = std::collections::HashSet::new();
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |batch| {
            for o in batch {
                if o.reason.is_none() {
                    let _ = files.insert(o.path.clone());
                }
            }
            if !changed {
                fs::create_dir(dir.path().join("new-pod")).unwrap();
                changed = true;
            }
            true
        },
    );
    assert!(!end.complete);
    assert_eq!(files.len(), expected);
    assert!(run(&mut scanner).0.complete);
}

/// Scenario: A backend without qualified seek cookies has stable directories larger than a batch.
/// Guarantees: Verified prefix replay reaches later batches and sibling subtrees while the pass remains incomplete.
#[test]
#[ignore = "requires an isolated mount namespace with overlayfs support; Linux qualification"]
fn overlay_replay_preserves_positive_progress() {
    let dir = fixture();
    for name in ["lower", "upper", "work", "merged"] {
        fs::create_dir(dir.path().join(name)).unwrap();
    }
    let merged = dir.path().join("merged");
    let options = format!(
        "lowerdir={},upperdir={},workdir={}",
        dir.path().join("lower").display(),
        dir.path().join("upper").display(),
        dir.path().join("work").display()
    );
    assert!(
        std::process::Command::new("mount")
            .args(["-t", "overlay", "overlay", "-o"])
            .arg(options)
            .arg(&merged)
            .status()
            .unwrap()
            .success()
    );
    struct Unmount(PathBuf);
    impl Drop for Unmount {
        fn drop(&mut self) {
            assert!(
                std::process::Command::new("umount")
                    .arg(&self.0)
                    .status()
                    .unwrap()
                    .success()
            );
        }
    }
    let _mount = Unmount(merged.clone());
    // A flat listing that never replays needs no cookie qualification.
    for index in 0..5 {
        fs::write(merged.join(format!("small-{index}")), b"small").unwrap();
    }
    let mut small = scanner(&merged, false);
    let mut directories = 0;
    let mut candidates = 0;
    let end = small.scan(
        0,
        &ProbeBudget::default(),
        || false,
        |batch| {
            for observation in batch {
                if observation.reason == Some(ScanReason::Directory) {
                    assert!(observation.locator.is_some());
                    directories += 1;
                }
                assert_ne!(observation.reason, Some(ScanReason::UnqualifiedResumption));
                candidates += usize::from(observation.reason.is_none());
            }
            true
        },
    );
    assert!(end.complete);
    assert_eq!(directories, 1);
    assert_eq!(candidates, 5);
    for index in 0..5 {
        fs::remove_file(merged.join(format!("small-{index}"))).unwrap();
    }
    let expected = nested_fixture(&merged);
    let mut cfg = config(&merged);
    cfg.limits.batch = 3;
    let (end, observations) = run(&mut Scanner::new(
        cfg,
        Policy {
            recursive: true,
            exclude_secret: false,
        },
    )
    .unwrap());
    let files: std::collections::HashSet<_> = observations
        .iter()
        .filter(|o| o.1.is_none())
        .map(|o| &o.0)
        .collect();
    assert!(!end.complete);
    assert_eq!(files.len(), expected);
    assert!(
        observations
            .iter()
            .any(|o| o.1 == Some(ScanReason::UnqualifiedResumption))
    );
}

/// Scenario: A reader releases the shared allowance during discovery's single contention retry.
/// Guarantees: The recovered probe completes without an unnecessary incomplete-pass result or additional slots.
#[test]
fn transient_budget_contention_can_recover() {
    use std::cell::Cell;
    struct SignallingPolicy<'a>(&'a Cell<bool>);
    impl MatchPolicy for SignallingPolicy<'_> {
        fn includes(&self, _: &Path) -> bool {
            self.0.set(true);
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            false
        }
        fn excludes(&self, _: &Path) -> bool {
            false
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
    }
    let dir = fixture();
    fs::write(dir.path().join("log"), b"ok").unwrap();
    let entered = Cell::new(false);
    let budget = ProbeBudget::default();
    let mut held = budget.try_reserve();
    let mut scanner = Scanner::new(config(dir.path()), SignallingPolicy(&entered)).unwrap();
    let mut candidates = 0;
    let end = scanner.scan(
        1,
        &budget,
        || {
            if entered.get() {
                drop(held.take());
            }
            false
        },
        |batch| {
            candidates += batch.iter().filter(|o| o.reason.is_none()).count();
            true
        },
    );
    assert!(end.complete);
    assert_eq!(candidates, 1);
    assert!(budget.try_reserve().is_some());
}

/// Scenario: Missing include roots surround a valid root with batches of one, two, or three slots.
/// Guarantees: Root errors and candidates are delivered without overflow or losing positive observations.
#[test]
fn small_batches_preserve_multiple_root_errors_and_candidates() {
    let dir = fixture();
    fs::write(dir.path().join("log"), b"data").unwrap();
    for batch_size in 1..=3 {
        let mut cfg = config(dir.path());
        cfg.include_roots = vec![
            dir.path().join("missing-a"),
            dir.path().to_owned(),
            dir.path().join("missing-b"),
        ];
        cfg.limits.batch = batch_size;
        let mut scanner = Scanner::new(
            cfg,
            Policy {
                recursive: false,
                exclude_secret: false,
            },
        )
        .unwrap();
        let mut missing = 0;
        let mut candidates = 0;
        let end = scanner.scan(
            1,
            &ProbeBudget::default(),
            || false,
            |batch| {
                assert!(batch.len() <= batch_size);
                missing += batch
                    .iter()
                    .filter(|o| o.reason == Some(ScanReason::RootUnavailable))
                    .count();
                candidates += batch.iter().filter(|o| o.reason.is_none()).count();
                true
            },
        );
        assert!(!end.complete);
        assert!(!end.cancelled);
        assert_eq!(missing, 2);
        assert_eq!(candidates, 1);
    }
}

/// Scenario: Traversal exhausts its work allowance after earlier root errors, with one output slot.
/// Guarantees: Limit diagnostics fit the batch, coverage stays incomplete, and a later pass can retry.
#[test]
fn one_slot_root_errors_then_work_exhaustion() {
    let dir = fixture();
    fs::write(dir.path().join("log"), b"data").unwrap();
    let mut cfg = config(dir.path());
    cfg.include_roots.insert(0, dir.path().join("missing"));
    cfg.limits.batch = 1;
    cfg.limits.work_units = 1;
    let mut scanner = Scanner::new(
        cfg,
        Policy {
            recursive: false,
            exclude_secret: false,
        },
    )
    .unwrap();
    let budget = ProbeBudget::default();
    for generation in 1..=2 {
        let mut missing = false;
        let mut limit = false;
        let end = scanner.scan(
            generation,
            &budget,
            || false,
            |batch| {
                assert_eq!(batch.len(), 1);
                missing |= batch[0].reason == Some(ScanReason::RootUnavailable);
                limit |= batch[0].reason == Some(ScanReason::Limit);
                true
            },
        );
        assert!(missing && limit);
        assert!(!end.complete && !end.cancelled);
        assert!(budget.try_reserve().is_some());
    }
}

/// Scenario: Cancellation is reported once inside exclusion resolution with a one-slot or larger batch.
/// Guarantees: The cancellation reason is delivered immediately and traversal cannot continue as a policy failure.
#[test]
fn exclusion_resolution_preserves_one_shot_cancellation() {
    let dir = fixture();
    for batch_size in [1, 4] {
        let mut cfg = config(dir.path());
        cfg.exclusion_roots = vec![dir.path().to_owned()];
        cfg.limits.batch = batch_size;
        let mut scanner = Scanner::new(
            cfg,
            Policy {
                recursive: false,
                exclude_secret: false,
            },
        )
        .unwrap();
        let mut checks = 0;
        let mut cancelled = 0;
        let end = scanner.scan(
            1,
            &ProbeBudget::default(),
            || {
                checks += 1;
                checks == 3 // Initial check, exclusion-loop check, then resolver check.
            },
            |batch| {
                assert_eq!(batch.len(), 1);
                assert_eq!(batch[0].reason, Some(ScanReason::Cancelled));
                cancelled += 1;
                true
            },
        );
        assert!(end.cancelled && !end.complete);
        assert_eq!(cancelled, 1);
    }
}

/// Scenario: The consumer cancels delivery of the first error among several missing roots.
/// Guarantees: A one-slot batch stops further root processing and the next pass starts with clean traversal state.
#[test]
fn cancellation_between_root_errors() {
    let dir = fixture();
    let mut cfg = config(dir.path());
    cfg.include_roots = vec![dir.path().join("a"), dir.path().join("b")];
    cfg.limits.batch = 1;
    let mut scanner = Scanner::new(
        cfg,
        Policy {
            recursive: false,
            exclude_secret: false,
        },
    )
    .unwrap();
    let mut delivered = 0;
    let end = scanner.scan(
        1,
        &ProbeBudget::default(),
        || false,
        |batch| {
            assert_eq!(batch.len(), 1);
            delivered += 1;
            false
        },
    );
    assert!(end.cancelled && !end.complete);
    assert_eq!(delivered, 1);
    let (end, observations) = run(&mut scanner);
    assert!(!end.cancelled && !end.complete);
    assert_eq!(observations.len(), 2);
}

/// Scenario: A root or followed subdirectory is replaced by the checkpoint namespace between batches.
/// Guarantees: Expected and prior directory identities are checked before pruning, even with one output slot.
#[test]
fn namespace_replacement_between_batches_loses_coverage() {
    for through_alias in [false, true] {
        for batch_size in [1, 2] {
            let dir = fixture();
            let watched = dir.path().join("watched");
            let target = dir.path().join("target");
            let namespace = dir.path().join("namespace");
            for path in [&watched, &target, &namespace] {
                fs::create_dir(path).unwrap();
            }
            for index in 0..4 {
                fs::write(target.join(format!("{index}.log")), b"data").unwrap();
            }
            fs::write(namespace.join("checkpoint.db"), b"private").unwrap();
            let (root, replaced_path) = if through_alias {
                let alias = watched.join("alias");
                symlink(&target, &alias).unwrap();
                (watched, alias)
            } else {
                (target.clone(), target.clone())
            };
            let mut cfg = config(&root);
            cfg.symlinks = SymlinkPolicy::Follow;
            cfg.limits.batch = batch_size;
            cfg.checkpoint_namespace =
                FileLocator::from_metadata(&fs::metadata(&namespace).unwrap());
            let mut scanner = Scanner::new(
                cfg,
                Policy {
                    recursive: true,
                    exclude_secret: false,
                },
            )
            .unwrap();
            let mut replaced = false;
            let mut lost_coverage = false;
            let end = scanner.scan(
                1,
                &ProbeBudget::default(),
                || false,
                |batch| {
                    assert!(batch.len() <= batch_size);
                    assert!(batch.iter().all(|o| !o.path.ends_with("checkpoint.db")));
                    lost_coverage |= batch
                        .iter()
                        .any(|o| o.path == replaced_path && o.reason == Some(ScanReason::Coverage));
                    if !replaced
                        && batch.iter().any(|o| {
                            o.path == replaced_path && o.reason == Some(ScanReason::Directory)
                        })
                    {
                        fs::rename(&target, dir.path().join("moved")).unwrap();
                        symlink(&namespace, &target).unwrap();
                        replaced = true;
                    }
                    true
                },
            );
            assert!(replaced);
            assert!(!end.complete, "alias={through_alias}, batch={batch_size}");
            assert!(lost_coverage);
        }
    }
}

/// Scenario: A followed file target disappears or becomes a loop after classification but before its first pin.
/// Guarantees: Required-subtree uncertainty fails coverage; the same failures remain local for leaf-only includes.
#[test]
fn late_link_resolution_failure_preserves_traversal_context() {
    use std::cell::Cell;
    struct ChangingTarget {
        target: PathBuf,
        recursive: bool,
        make_loop: bool,
        changed: Cell<bool>,
    }
    impl MatchPolicy for ChangingTarget {
        fn includes(&self, _: &Path) -> bool {
            if !self.changed.replace(true) {
                fs::remove_file(&self.target).unwrap();
                if self.make_loop {
                    symlink("target", &self.target).unwrap();
                }
            }
            true
        }
        fn may_descend(&self, _: &Path) -> bool {
            self.recursive
        }
        fn excludes(&self, _: &Path) -> bool {
            false
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
            false
        }
    }
    for recursive in [false, true] {
        for make_loop in [false, true] {
            let watched = fixture();
            let external = fixture();
            let target = external.path().join("target");
            fs::write(&target, b"old file").unwrap();
            symlink(&target, watched.path().join("alias.log")).unwrap();
            let before = fs::metadata(watched.path()).unwrap().modified().unwrap();
            let mut cfg = config(watched.path());
            cfg.symlinks = SymlinkPolicy::Follow;
            let mut scanner = Scanner::new(
                cfg,
                ChangingTarget {
                    target,
                    recursive,
                    make_loop,
                    changed: Cell::new(false),
                },
            )
            .unwrap();
            let expected = if recursive {
                ScanReason::Coverage
            } else if make_loop {
                ScanReason::LeafLoop
            } else {
                ScanReason::Missing
            };
            let mut observed = false;
            let end = scanner.scan(
                1,
                &ProbeBudget::default(),
                || false,
                |batch| {
                    for observation in batch.iter().filter(|o| o.path.ends_with("alias.log")) {
                        assert_eq!(observation.reason, Some(expected));
                        assert_eq!(
                            observation.errno,
                            Some(if make_loop { libc::ELOOP } else { libc::ENOENT })
                        );
                        assert!(observation.locator.is_none());
                        observed = true;
                    }
                    true
                },
            );
            assert!(observed);
            assert_eq!(end.complete, !recursive);
            assert_eq!(
                fs::metadata(watched.path()).unwrap().modified().unwrap(),
                before
            );
        }
    }
}
