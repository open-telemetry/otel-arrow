// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Prepare the read worker's store and the discovery worker's scanner on a blocking thread.

use super::config::{CompiledConfig, ConfigError, PathPolicy, ValidatedInputs};
use super::discovery::{InvalidScanConfig, ScanConfig, ScanLimits, Scanner, VerificationError};
use super::source::{FileLocator, SymlinkPolicy};
use super::workers::{Cancellation, ChannelError};
use otel_arrow_dfe_filelog_checkpoint::{
    Locator,
    store::{CheckpointStore, StateRoot, StoreError},
};
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub(super) enum StartupError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Store(#[from] StoreError),
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
            Scanner::check_verification,
        )
    }

    fn open_with(
        config: CompiledConfig,
        root: &StateRoot,
        cancel: &Cancellation,
        now: impl Fn() -> Instant,
        mut wait: impl FnMut(Duration) -> Result<(), ChannelError>,
        mut probe: impl FnMut(&mut Scanner<PathPolicy>) -> Result<(), VerificationError>,
    ) -> Result<Self, StartupError> {
        let (settings, paths) = config.into_parts();
        let config = settings.settings();
        let limits = ScanLimits {
            depth: config.max_recursion_depth as usize,
            fingerprint_bytes: config.identity.fingerprint_bytes as usize,
            ignored_header_bytes: config.identity.ignored_header_bytes,
            roots: paths.include_roots().len().max(paths.exclude_roots().len()),
            ..ScanLimits::default()
        };
        paths.validate_root_lengths(limits.path_bytes)?;
        let store_config = settings.store_config()?;
        let timeout = config.checkpoint.ownership_timeout;
        let start = now();
        let store = loop {
            if cancel.is_cancelled() {
                return Err(ChannelError::Cancelled.into());
            }
            match CheckpointStore::open(root, settings.checkpoint_id(), store_config) {
                Ok(store) => break store,
                Err(StoreError::Locked) => {
                    let remaining = timeout.saturating_sub(now().duration_since(start));
                    if remaining.is_zero() {
                        return Err(StartupError::OwnershipTimeout(timeout));
                    }
                    wait(remaining.min(Duration::from_millis(250)))?;
                }
                Err(error) => return Err(error.into()),
            }
        };
        let Locator::PosixDevIno { dev, ino } = store.namespace_locator()? else {
            return Err(StartupError::NamespaceLocator);
        };
        let mut scanner = Scanner::new(
            ScanConfig {
                include_roots: paths.include_roots().to_vec(),
                exclusion_roots: paths.exclude_roots().to_vec(),
                symlinks: if config.follow_symlinks {
                    SymlinkPolicy::Follow
                } else {
                    SymlinkPolicy::Reject
                },
                checkpoint_namespace: FileLocator {
                    device: dev,
                    inode: ino,
                },
                limits,
            },
            paths,
        )?;
        let mut delay = Duration::from_millis(250);
        loop {
            if cancel.is_cancelled() {
                return Err(ChannelError::Cancelled.into());
            }
            match probe(&mut scanner) {
                Err(VerificationError::Descriptors(_) | VerificationError::Transient(_)) => {
                    wait(delay)?;
                    delay = (delay * 2).min(Duration::from_secs(30));
                }
                result => {
                    result?;
                    break;
                }
            }
        }
        Ok(Self {
            settings,
            store,
            scanner,
        })
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
    fn namespace_release_during_final_wait() {
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
            Scanner::check_verification,
        )
        .unwrap();
        assert!(owner.is_none());
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
                    assert!(!lock_held);
                    scanner.inject_capability_result(Err(libc::EMFILE));
                    scanner.check_verification()
                },
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
                scanner.check_verification()
            },
        )
        .unwrap();
        assert_eq!(
            delays,
            [250, 500, 1000, 2000, 4000, 8000, 16000, 30000, 30000]
        );
    }

    /// Scenario: The startup probe is unsupported or forbidden by the execution environment.
    /// Guarantees: Startup returns the classified errno immediately and releases the recovered store.
    #[test]
    fn permanent_probe_failure_releases_namespace() {
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
                    scanner.check_verification()
                },
            );
            match (errno, result) {
                (
                    libc::ENOSYS | libc::EINVAL,
                    Err(StartupError::Verification(VerificationError::Unsupported(e))),
                )
                | (
                    libc::EPERM | libc::EACCES,
                    Err(StartupError::Verification(VerificationError::Denied(e))),
                ) => assert_eq!(e.raw_os_error(), Some(errno)),
                _ => panic!("unexpected outcome for errno {errno}"),
            }
            let _running = f.open().unwrap();
        }
    }
}
