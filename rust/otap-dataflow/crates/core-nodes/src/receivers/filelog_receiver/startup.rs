// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Prepare the read worker's store and the discovery worker's scanner on a blocking thread.

use super::config::{CompiledConfig, ConfigError, PathPolicy, ValidatedInputs};
use super::discovery::{
    InvalidScanConfig, ScanConfig, ScanLimits, Scanner, VerificationError, VerificationProbe,
};
use super::source::{FileLocator, SymlinkPolicy};
use super::workers::{Cancellation, ChannelError};
use otel_arrow_dfe_filelog_checkpoint::{
    Locator,
    store::{CheckpointStore, LockedNamespace, StateRoot, StoreConfig, StoreError},
};
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub(super) enum StartupError {
    #[error("filelog needs {budget} descriptors, above soft RLIMIT_NOFILE {soft}")]
    DescriptorBudget { budget: u128, soft: u64 },
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Store(StoreError),
    #[error("timed out acquiring the filelog checkpoint namespace after {0:?}")]
    OwnershipTimeout(Duration),
    #[error("Linux checkpoint namespace has no POSIX locator")]
    NamespaceLocator,
    #[error(transparent)]
    Verification(#[from] VerificationError),
    #[error(transparent)]
    Worker(#[from] ChannelError),
    #[error(transparent)]
    Scanner(#[from] InvalidScanConfig),
}

impl From<StoreError> for StartupError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Cancelled => Self::Worker(ChannelError::Cancelled),
            error => Self::Store(error),
        }
    }
}

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = "urn:otel:receiver:filelog",
    target = "otel.receiver.filelog",
);

// One traversal directory, two transient opens, and eight namespace/store handles.
fn descriptor_budget(max_open_files: u64, soft: Option<u64>) -> Result<Option<u128>, StartupError> {
    let budget = u128::from(max_open_files) + 11;
    let Some(soft) = soft else { return Ok(None) };
    if budget > u128::from(soft) {
        return Err(StartupError::DescriptorBudget { budget, soft });
    }
    Ok((budget * 5 > u128::from(soft) * 4).then_some(budget))
}

/// Recovered store and verified discovery inputs, ready to transfer to their workers.
pub(super) struct WorkerInputs {
    pub(super) settings: ValidatedInputs,
    pub(super) store: CheckpointStore,
    pub(super) scanner: Scanner<PathPolicy>,
}

impl WorkerInputs {
    pub(super) fn open(
        config: CompiledConfig,
        root: &StateRoot,
        cancel: &Cancellation,
    ) -> Result<Self, StartupError> {
        Self::open_with(
            config,
            root,
            cancel,
            Instant::now,
            |delay| cancel.wait(delay),
            VerificationProbe::check,
            LockedNamespace::acquire,
        )
    }

    fn open_with(
        config: CompiledConfig,
        root: &StateRoot,
        cancel: &Cancellation,
        now: impl Fn() -> Instant,
        mut wait: impl FnMut(Duration) -> Result<(), ChannelError>,
        mut probe: impl FnMut(&mut VerificationProbe) -> Result<(), VerificationError>,
        mut acquire: impl FnMut(&StateRoot, &str, StoreConfig) -> Result<LockedNamespace, StoreError>,
    ) -> Result<Self, StartupError> {
        let (settings, paths) = config.into_parts();
        let config = settings.settings();
        let soft = rustix::process::getrlimit(rustix::process::Resource::Nofile).current;
        if let Some(budget) = descriptor_budget(config.limits.max_open_files, soft)? {
            otel_warn!(
                "filelog_receiver.descriptor_budget",
                budget = budget as u64,
                soft_limit = soft.unwrap_or(u64::MAX),
                message = "Filelog requires more than 80 percent of the process descriptor limit"
            );
        }
        let limits = ScanLimits {
            depth: config.max_recursion_depth as usize,
            fingerprint_bytes: config.identity.fingerprint_bytes as usize,
            ignored_header_bytes: config.identity.ignored_header_bytes,
            roots: paths.include_roots().len().max(paths.exclude_roots().len()),
            ..ScanLimits::default()
        };
        paths.validate_root_lengths(limits.path_bytes)?;
        let store_config = settings.store_config()?;
        let mut verification = VerificationProbe::default();
        let mut delay = Duration::from_millis(250);
        loop {
            if cancel.is_cancelled() {
                return Err(ChannelError::Cancelled.into());
            }
            match probe(&mut verification) {
                Err(VerificationError::Descriptors(..) | VerificationError::Transient(..)) => {
                    wait(delay)?;
                    delay = (delay * 2).min(Duration::from_secs(30));
                }
                result => {
                    result?;
                    break;
                }
            }
        }
        let namespace = acquire_namespace(
            config.checkpoint.ownership_timeout,
            cancel,
            now,
            &mut wait,
            || acquire(root, settings.checkpoint_id(), store_config),
        )?;
        let Locator::PosixDevIno { dev, ino } = namespace.locator()? else {
            return Err(StartupError::NamespaceLocator);
        };
        let checkpoint_namespace = FileLocator {
            device: dev,
            inode: ino,
        };
        for (index, root) in paths.include_roots().iter().enumerate() {
            if cancel.is_cancelled() {
                return Err(ChannelError::Cancelled.into());
            }
            // Positive identity evidence rejects a direct include, including an alias.
            // Unavailable roots retain the ordinary discovery coverage semantics.
            let is_namespace = |path: &std::path::Path| {
                std::fs::metadata(path)
                    .is_ok_and(|m| FileLocator::from_metadata(&m) == checkpoint_namespace)
            };
            if is_namespace(root)
                || super::config::literal_include_target(root, &config.include[index])
                    .as_deref()
                    .is_some_and(is_namespace)
            {
                return Err(ConfigError {
                    setting: format!("include[{index}]"),
                    reason: "include resolves to the checkpoint namespace".into(),
                }
                .into());
            }
        }
        let mut delay = Duration::from_millis(250);
        let store = namespace.recover(
            || cancel.is_cancelled(),
            |_| {
                wait(delay)?;
                delay = (delay * 2).min(Duration::from_secs(30));
                Ok::<_, StartupError>(())
            },
        )?;
        // Keep pattern-indexed policy intact. The scanner only needs the first
        // occurrence of each byte-identical traversal root, in configured order.
        let mut seen = std::collections::HashSet::new();
        let include_roots = paths
            .include_roots()
            .iter()
            .filter(|root| seen.insert(root.as_os_str()))
            .cloned()
            .collect();
        let scanner = Scanner::with_probe(
            ScanConfig {
                recursive: config.recursive,
                include_roots,
                exclusion_roots: paths.exclude_roots().to_vec(),
                symlinks: if config.follow_symlinks {
                    SymlinkPolicy::Follow
                } else {
                    SymlinkPolicy::Reject
                },
                checkpoint_namespace,
                limits,
            },
            paths,
            verification,
        )?;
        Ok(Self {
            settings,
            store,
            scanner,
        })
    }
}

fn acquire_namespace<T>(
    timeout: Duration,
    cancel: &Cancellation,
    now: impl Fn() -> Instant,
    mut wait: impl FnMut(Duration) -> Result<(), ChannelError>,
    mut open: impl FnMut() -> Result<T, StoreError>,
) -> Result<T, StartupError> {
    let mut contention = None;
    let mut backoff = Duration::from_millis(250);
    loop {
        if cancel.is_cancelled() {
            return Err(ChannelError::Cancelled.into());
        }
        let attempted_at = now();
        if contention.is_some_and(|start| attempted_at.duration_since(start) > timeout) {
            return Err(StartupError::OwnershipTimeout(timeout));
        }
        let delay = match open() {
            Ok(store) => return Ok(store),
            Err(StoreError::Locked) => {
                let _ = contention.get_or_insert(attempted_at);
                Duration::from_millis(250)
            }
            Err(error) if error.is_descriptor_exhaustion() => {
                let delay = backoff;
                backoff = (backoff * 2).min(Duration::from_secs(30));
                delay
            }
            Err(error) => return Err(error.into()),
        };
        let delay = if let Some(start) = contention {
            let remaining = timeout.saturating_sub(now().duration_since(start));
            if remaining.is_zero() {
                return Err(StartupError::OwnershipTimeout(timeout));
            }
            delay.min(remaining)
        } else {
            delay
        };
        wait(delay)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::filelog_receiver::{
        config::Config,
        discovery::{ProbeBudget, ScanReason},
        workers::Workers,
    };
    use std::{
        fs,
        os::{fd::AsFd, unix::fs::PermissionsExt},
    };

    /// Scenario: budgets straddle 80 percent and the finite soft descriptor limit.
    /// Guarantees: exact equality is admitted and only budgets above 80 percent warn.
    #[test]
    fn descriptor_limits() {
        assert_eq!(descriptor_budget(69, Some(100)).unwrap(), None);
        assert_eq!(descriptor_budget(70, Some(100)).unwrap(), Some(81));
        assert_eq!(descriptor_budget(89, Some(100)).unwrap(), Some(100));
        assert!(descriptor_budget(90, Some(100)).is_err());
        assert!(descriptor_budget(u64::MAX, Some(u64::MAX)).is_err());
        assert_eq!(descriptor_budget(512, None).unwrap(), None);
    }

    struct Fixture {
        dir: tempfile::TempDir,
        root: StateRoot,
        lifecycle: Workers,
    }

    impl Fixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let root =
                StateRoot::from_borrowed_fd(fs::File::open(dir.path()).unwrap().as_fd()).unwrap();
            Self {
                dir,
                root,
                lifecycle: Workers::new(Duration::from_secs(5)).unwrap(),
            }
        }

        fn config(&self) -> CompiledConfig {
            let mut config = Config {
                include: vec!["**/*".into()],
                ..Config::default()
            };
            config.checkpoint.ownership_timeout = Duration::from_millis(1);
            config
                .compile("group", "pipeline", "node", self.dir.path())
                .unwrap()
        }

        fn open(&self) -> Result<WorkerInputs, StartupError> {
            WorkerInputs::open(self.config(), &self.root, &self.lifecycle.cancellation())
        }
    }

    /// Scenario: Configured discovery includes both a log and the receiver's open checkpoint namespace.
    /// Guarantees: Only the eligible log reaches admission, even when checkpoint files match the include.
    #[test]
    fn configured_discovery_excludes_its_open_store() {
        let fixture = Fixture::new();
        let log = fixture.dir.path().join("source.log");
        fs::write(&log, b"hello\n").unwrap();
        let mut workers = fixture.open().unwrap();
        let mut admitted = Vec::new();
        let mut unqualified = false;
        let pass = workers.scanner.scan(
            1,
            &ProbeBudget::default(),
            || false,
            |batch| {
                for observation in batch {
                    match observation.reason {
                        None => admitted.push(observation.path.clone()),
                        Some(ScanReason::Directory) => (),
                        Some(ScanReason::UnqualifiedResumption) => unqualified = true,
                        reason => panic!(
                            "unexpected scan result for {:?}: {:?}",
                            observation.path, reason
                        ),
                    }
                }
                true
            },
        );
        assert_eq!(pass.complete, !unqualified);
        assert_eq!(admitted, [log]);
    }

    /// Scenario: A running receiver owns the namespace a second instance tries to open.
    /// Guarantees: The second times out without displacing the owner and can start after release.
    #[test]
    fn namespace_contention_preserves_the_running_owner() {
        let fixture = Fixture::new();
        let running = fixture.open().unwrap();
        assert!(matches!(
            fixture.open(),
            Err(StartupError::OwnershipTimeout(_))
        ));
        assert!(running.store.state().is_ok());
        drop(running);
        assert!(fixture.open().unwrap().store.state().is_ok());
    }

    /// Scenario: The namespace owner releases its lock during the final retry delay.
    /// Guarantees: Startup tries the store once at the deadline and acquires the released namespace.
    #[test]
    fn namespace_can_be_acquired_on_exact_deadline() {
        let f = Fixture::new();
        let mut owner = Some(f.open().unwrap());
        let clock = std::cell::Cell::new(Instant::now());
        let config = f.config();
        let timeout = config.inputs().settings().checkpoint.ownership_timeout;
        let _running = WorkerInputs::open_with(
            config,
            &f.root,
            &f.lifecycle.cancellation(),
            || clock.get(),
            |delay| {
                assert_eq!(delay, timeout);
                drop(owner.take().unwrap());
                clock.set(clock.get() + delay);
                Ok(())
            },
            VerificationProbe::check,
            LockedNamespace::acquire,
        )
        .unwrap();
        assert!(owner.is_none());
    }

    /// Scenario: Lock contention ends at the ownership deadline, then database open hits EMFILE.
    /// Guarantees: Recovery retains the lock through backoff past that deadline and hands it to the store.
    #[test]
    fn descriptor_backoff_after_lock_acquisition() {
        let f = Fixture::new();
        let mut owner = Some(f.open().unwrap());
        let clock = std::cell::Cell::new(Instant::now());
        let config = f.config();
        let id = config.inputs().checkpoint_id().to_owned();
        let store_config = config.inputs().store_config().unwrap();
        let timeout = config.inputs().settings().checkpoint.ownership_timeout;
        let mut waits = Vec::new();
        let running = WorkerInputs::open_with(
            config,
            &f.root,
            &f.lifecycle.cancellation(),
            || clock.get(),
            |delay| {
                waits.push(delay);
                if let Some(owner) = owner.take() {
                    drop(owner);
                } else {
                    assert!(matches!(
                        CheckpointStore::open(&f.root, &id, store_config),
                        Err(StoreError::Locked)
                    ));
                }
                clock.set(clock.get() + delay);
                Ok(())
            },
            VerificationProbe::check,
            |root, id, config| {
                let mut locked = LockedNamespace::acquire(root, id, config)?;
                locked.inject_database_open_error(libc::EMFILE);
                Ok(locked)
            },
        )
        .unwrap();
        assert_eq!(waits, [timeout, Duration::from_millis(250)]);
        assert!(matches!(
            CheckpointStore::open(&f.root, &id, store_config),
            Err(StoreError::Locked)
        ));
        drop(running);
        assert!(f.open().is_ok());
    }

    /// Scenario: The owner runs out of descriptors opening its database and shutdown interrupts backoff.
    /// Guarantees: Startup returns worker cancellation and the next opener can acquire the namespace.
    #[test]
    fn cancel_descriptor_backoff_releases_namespace() {
        let f = Fixture::new();
        let cancel = f.lifecycle.cancellation();
        let result = WorkerInputs::open_with(
            f.config(),
            &f.root,
            &cancel,
            Instant::now,
            |delay| {
                let _ = cancel.cancel();
                cancel.wait(delay)
            },
            VerificationProbe::check,
            |root, id, config| {
                let mut locked = LockedNamespace::acquire(root, id, config)?;
                locked.inject_database_open_error(libc::ENFILE);
                Ok(locked)
            },
        );
        assert!(matches!(
            result,
            Err(StartupError::Worker(ChannelError::Cancelled))
        ));
        let replacement = Workers::new(Duration::from_secs(5)).unwrap();
        assert!(WorkerInputs::open(f.config(), &f.root, &replacement.cancellation()).is_ok());
    }

    /// Scenario: Startup is cancelled at a namespace wait or a verification backoff.
    /// Guarantees: Both waits return cancellation and the namespace remains available after owner teardown.
    #[test]
    fn cancellation_interrupts_startup_waits() {
        for lock_held in [true, false] {
            let f = Fixture::new();
            let owner = lock_held.then(|| f.open().unwrap());
            let cancel = f.lifecycle.cancellation();
            let mut waited = false;
            let now = Instant::now();
            let result = WorkerInputs::open_with(
                f.config(),
                &f.root,
                &cancel,
                || now,
                |delay| {
                    waited = true;
                    let _ = cancel.cancel();
                    cancel.wait(delay)
                },
                |scanner| {
                    if !lock_held {
                        scanner.inject_capability_result(Err(libc::EMFILE));
                    }
                    scanner.check()
                },
                LockedNamespace::acquire,
            );
            assert!(waited);
            assert!(matches!(
                result,
                Err(StartupError::Worker(ChannelError::Cancelled))
            ));
            drop(owner);
            let replacement = Workers::new(Duration::from_secs(5)).unwrap();
            let _running =
                WorkerInputs::open(f.config(), &f.root, &replacement.cancellation()).unwrap();
        }
    }

    /// Scenario: The capability probe encounters temporary kernel pressure and interrupted calls.
    /// Guarantees: Startup retries with capped exponential delays and succeeds once verification works.
    #[test]
    fn transient_probe_failures_back_off_until_verified() {
        let f = Fixture::new();
        let mut errors = [
            libc::EINTR,
            libc::EAGAIN,
            libc::ENOMEM,
            libc::EMFILE,
            libc::ENFILE,
            libc::ENOMEM,
            libc::ENOMEM,
            libc::ENOMEM,
            libc::ENOMEM,
        ]
        .into_iter();
        let mut delays = Vec::new();
        let _running = WorkerInputs::open_with(
            f.config(),
            &f.root,
            &f.lifecycle.cancellation(),
            Instant::now,
            |delay| {
                delays.push(delay.as_millis());
                Ok(())
            },
            |scanner| {
                if let Some(errno) = errors.next() {
                    scanner.inject_capability_result(Err(errno));
                }
                scanner.check()
            },
            LockedNamespace::acquire,
        )
        .unwrap();
        assert_eq!(
            delays,
            [250, 500, 1000, 2000, 4000, 8000, 16000, 30000, 30000]
        );
    }

    /// Scenario: The startup probe is unsupported or forbidden by the execution environment.
    /// Guarantees: Startup returns the classified errno immediately before creating checkpoint artifacts.
    #[test]
    fn permanent_probe_failure_precedes_store_creation() {
        let f = Fixture::new();
        for errno in [libc::ENOSYS, libc::EINVAL, libc::EPERM, libc::EACCES] {
            let result = WorkerInputs::open_with(
                f.config(),
                &f.root,
                &f.lifecycle.cancellation(),
                Instant::now,
                |_| panic!("permanent failure retried"),
                |scanner| {
                    scanner.inject_capability_result(Err(errno));
                    scanner.check()
                },
                LockedNamespace::acquire,
            );
            match (errno, result) {
                (
                    libc::ENOSYS | libc::EINVAL,
                    Err(StartupError::Verification(VerificationError::Unsupported(_, e))),
                )
                | (
                    libc::EPERM | libc::EACCES,
                    Err(StartupError::Verification(VerificationError::Denied(_, e))),
                ) => assert_eq!(e.raw_os_error(), Some(errno)),
                _ => panic!("unexpected outcome for errno {errno}"),
            }
            assert!(!f.dir.path().join("filelog").exists());
        }
    }

    /// Scenario: openat2 opens the root but reading its pinned path through procfs fails.
    /// Guarantees: Startup identifies the failed lookup and its errno before creating checkpoint state.
    #[test]
    fn procfs_lookup_failure_precedes_store_creation() {
        let f = Fixture::new();
        let result = WorkerInputs::open_with(
            f.config(),
            &f.root,
            &f.lifecycle.cancellation(),
            Instant::now,
            |_| panic!("unexpected retry for missing procfs"),
            |probe| {
                probe.check_with_pinned_path(|root| {
                    let expected = FileLocator::from_metadata(&fs::metadata("/").unwrap());
                    assert_eq!(
                        FileLocator::from_metadata(&root.metadata().unwrap()),
                        expected
                    );
                    Err(std::io::Error::from_raw_os_error(libc::ENOENT))
                })
            },
            LockedNamespace::acquire,
        );
        let Err(StartupError::Verification(VerificationError::Other(op, source))) = result else {
            panic!("expected procfs verification failure");
        };
        assert_eq!(op, "readlinkat(/proc/self/fd/<fd>)");
        assert_eq!(source.raw_os_error(), Some(libc::ENOENT));
        assert!(!f.dir.path().join("filelog").exists());
    }

    /// Scenario: A receiver's include root directly names or aliases its checkpoint namespace.
    /// Guarantees: Startup rejects the offending include before discovery or readiness.
    #[test]
    fn namespace_root_is_a_configuration_error() {
        let f = Fixture::new();
        let namespace = f
            .dir
            .path()
            .join("filelog/@v1")
            .join(f.config().inputs().namespace_component());
        let alias = f.dir.path().join("alias[1]");
        std::os::unix::fs::symlink(&namespace, &alias).unwrap();
        let literal = namespace.to_str().unwrap();
        for pattern in [
            literal.to_owned(),
            format!("{literal}/**/*"),
            r"alias\[1\]".into(),
            r"alias\[1\]/**/*".into(),
        ] {
            let config = Config {
                include: vec!["*.log".into(), pattern],
                ..Config::default()
            }
            .compile("group", "pipeline", "node", f.dir.path())
            .unwrap();
            assert!(
                matches!(WorkerInputs::open(config, &f.root, &f.lifecycle.cancellation()), Err(StartupError::Config(error)) if error.setting == "include[1]")
            );
            assert!(namespace.join("ownership.lock").is_file());
            assert!(!namespace.join("checkpoint.db").exists());
        }
    }

    /// Scenario: A literal include aliases a namespace with a torn WAL and a stale temporary.
    /// Guarantees: Rejected startup preserves both artifacts byte-for-byte and releases ownership.
    #[test]
    fn rejected_include_leaves_recovery_artifacts_intact() {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let f = Fixture::new();
        drop(f.open().unwrap());
        let namespace = f
            .dir
            .path()
            .join("filelog/@v1")
            .join(f.config().inputs().namespace_component());
        let db = namespace.join("checkpoint.db");
        fs::OpenOptions::new()
            .append(true)
            .open(&db)
            .unwrap()
            .write_all(b"FLOG")
            .unwrap();
        let before = fs::read(&db).unwrap();
        let temp = namespace.join("checkpoint.db.compact.tmp");
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)
            .unwrap()
            .write_all(b"unpublished")
            .unwrap();
        std::os::unix::fs::symlink(namespace, f.dir.path().join("alias")).unwrap();
        let config = Config {
            include: vec!["alias".into()],
            ..Config::default()
        }
        .compile("group", "pipeline", "node", f.dir.path())
        .unwrap();
        assert!(
            matches!(WorkerInputs::open(config, &f.root, &f.lifecycle.cancellation()), Err(StartupError::Config(error)) if error.setting == "include[0]")
        );
        assert_eq!(fs::read(db).unwrap(), before);
        assert_eq!(fs::read(temp).unwrap(), b"unpublished");
        let report = f.open().unwrap().store.recovery_report();
        assert_eq!(report.repaired_tail_bytes, 4);
        assert!(report.cleaned_temporary);
    }

    /// Scenario: A recursive glob is configured with traversal disabled.
    /// Guarantees: Discovery admits only immediate files and does not report a depth limit for skipped directories.
    #[test]
    fn nonrecursive_scan_stays_at_each_include_root() {
        let f = Fixture::new();
        let nested = f.dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let log = f.dir.path().join("direct.log");
        fs::write(&log, b"direct").unwrap();
        fs::write(nested.join("child.log"), b"nested").unwrap();
        let config = Config {
            include: vec!["**/*.log".into()],
            recursive: false,
            max_recursion_depth: 1,
            ..Config::default()
        }
        .compile("group", "pipeline", "node", f.dir.path())
        .unwrap();
        let mut workers = WorkerInputs::open(config, &f.root, &f.lifecycle.cancellation()).unwrap();
        let mut files = Vec::new();
        let _ = workers.scanner.scan(
            1,
            &ProbeBudget::default(),
            || false,
            |batch| {
                for observation in batch {
                    assert_ne!(observation.reason, Some(ScanReason::Limit));
                    if observation.reason.is_none() {
                        files.push(observation.path.clone());
                    }
                }
                true
            },
        );
        assert_eq!(files, [log]);
    }

    /// Scenario: A contended namespace becomes available but the retry wakes past its deadline.
    /// Guarantees: Startup reports timeout without starting a late open or recovery attempt.
    #[test]
    fn late_wakeup_cannot_acquire_namespace() {
        let f = Fixture::new();
        let clock = std::cell::Cell::new(Instant::now());
        let mut attempts = 0;
        let result = acquire_namespace(
            Duration::from_millis(1),
            &f.lifecycle.cancellation(),
            || clock.get(),
            |delay| {
                clock.set(clock.get() + delay + Duration::from_nanos(1));
                Ok(())
            },
            || {
                attempts += 1;
                if attempts == 1 {
                    Err(StoreError::Locked)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(result, Err(StartupError::OwnershipTimeout(_))));
        assert_eq!(attempts, 1);
    }

    /// Scenario: Store open runs out of descriptors during namespace setup and database recovery.
    /// Guarantees: Descriptor exhaustion backs off, but a failed durability barrier is returned immediately even with the same errno.
    #[test]
    fn descriptor_pressure_retries_only_safe_store_errors() {
        let f = Fixture::new();
        let error = std::io::Error::from_raw_os_error;
        let mut attempts = [
            Err(StoreError::Io {
                operation: "open namespace directory",
                source: error(libc::EMFILE),
            }),
            Err(StoreError::AuthoritativeGenerationUnreadable(error(
                libc::ENFILE,
            ))),
            Err(StoreError::SyncFailed {
                operation: "publication directory sync",
                source: error(libc::EMFILE),
            }),
            Ok(()),
        ]
        .into_iter();
        let mut delays = Vec::new();
        let result = acquire_namespace(
            Duration::from_millis(1),
            &f.lifecycle.cancellation(),
            Instant::now,
            |delay| {
                delays.push(delay.as_millis());
                Ok(())
            },
            || attempts.next().unwrap(),
        );
        assert!(matches!(
            result,
            Err(StartupError::Store(StoreError::SyncFailed { .. }))
        ));
        assert_eq!(delays, [250, 500]);
    }

    /// Scenario: Two patterns share a traversal root and a third uses a different root.
    /// Guarantees: Each matching file is offered once, and discovery visits distinct roots in first-seen order.
    #[test]
    fn shared_include_root_is_walked_once() {
        let f = Fixture::new();
        let first = f.dir.path().join("first");
        let second = f.dir.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let log = first.join("a.log");
        let text = first.join("b.txt");
        let later = second.join("c.log");
        for file in [&log, &text, &later] {
            fs::write(file, b"line").unwrap();
        }
        let config = Config {
            include: vec![
                "first/*.log".into(),
                "second/*.log".into(),
                "first/*.txt".into(),
            ],
            ..Config::default()
        }
        .compile("group", "pipeline", "node", f.dir.path())
        .unwrap();
        let mut workers = WorkerInputs::open(config, &f.root, &f.lifecycle.cancellation()).unwrap();
        let mut files = Vec::new();
        let _ = workers.scanner.scan(
            1,
            &ProbeBudget::default(),
            || false,
            |batch| {
                files.extend(
                    batch
                        .iter()
                        .filter(|o| o.reason.is_none())
                        .map(|o| o.path.clone()),
                );
                true
            },
        );
        assert_eq!(files.pop(), Some(later));
        files.sort();
        assert_eq!(files, [log, text]);
    }
}
