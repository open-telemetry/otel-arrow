// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Prepare the read worker's store and the discovery worker's scanner on a blocking thread.

use super::config::{CompiledConfig, ConfigError, PathPolicy, ValidatedInputs};
use super::discovery::{
    InvalidScanConfig, MatchPolicy, ScanConfig, ScanLimits, Scanner, VerificationError,
};
use super::source::{FileLocator, SymlinkPolicy};
use super::workers::{Cancellation, ChannelError};
use otel_arrow_dfe_filelog_checkpoint::{
    Locator,
    store::{CheckpointStore, StateRoot, StoreError},
};
use std::{
    path::Path,
    time::{Duration, Instant},
};

impl MatchPolicy for PathPolicy {
    fn includes(&self, path: &Path) -> bool {
        self.includes(path)
    }

    fn may_descend(&self, path: &Path) -> bool {
        self.may_descend(path)
    }

    fn excludes(&self, path: &Path) -> bool {
        self.excludes(path)
    }

    fn excludes_subtree(&self, path: &Path) -> bool {
        self.excludes_subtree(path)
    }

    fn excludes_mapped(&self, root_index: usize, suffix: &Path) -> bool {
        self.excludes_mapped(root_index, suffix)
    }
}

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
    Cancelled(#[from] ChannelError),
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
    /// Retry descriptor pressure on the worker while holding the recovered namespace lock.
    fn verify(
        scanner: &mut Scanner<PathPolicy>,
        cancel: &Cancellation,
    ) -> Result<(), StartupError> {
        let mut delay = Duration::from_millis(250);
        loop {
            if cancel.is_cancelled() {
                return Err(ChannelError::Cancelled.into());
            }
            match scanner.check_verification() {
                Err(VerificationError::Descriptors(_)) => {
                    cancel.wait(delay)?;
                    delay = (delay * 2).min(Duration::from_secs(30));
                }
                result => return result.map_err(StartupError::Verification),
            }
        }
    }

    pub(super) fn open(
        config: CompiledConfig,
        root: &StateRoot,
        cancel: &Cancellation,
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
        let start = Instant::now();
        let store = loop {
            if cancel.is_cancelled() {
                return Err(ChannelError::Cancelled.into());
            }
            match CheckpointStore::open(root, settings.checkpoint_id(), store_config) {
                Ok(store) => break store,
                Err(StoreError::Locked) => {
                    let remaining = timeout.saturating_sub(start.elapsed());
                    cancel.wait(remaining.min(Duration::from_millis(250)))?;
                    if start.elapsed() >= timeout {
                        return Err(StartupError::OwnershipTimeout(timeout));
                    }
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
        Self::verify(&mut scanner, cancel)?;
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

    /// Scenario: Configured discovery includes both a log and the receiver's open checkpoint namespace.
    /// Guarantees: Only the eligible log reaches admission, even when checkpoint files match the include.
    #[test]
    fn configured_discovery_excludes_its_open_store() {
        let dir = tempfile::tempdir().unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let root =
            StateRoot::from_borrowed_fd(fs::File::open(dir.path()).unwrap().as_fd()).unwrap();
        let log = dir.path().join("source.log");
        fs::write(&log, b"hello\n").unwrap();
        let config = Config {
            include: vec!["**/*".into()],
            ..Config::default()
        }
        .compile("group", "pipeline", "node", dir.path())
        .unwrap();
        let lifecycle = Workers::new(Duration::from_secs(5)).unwrap();
        let mut workers = WorkerInputs::open(config, &root, &lifecycle.cancellation()).unwrap();
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
        let dir = tempfile::tempdir().unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let root =
            StateRoot::from_borrowed_fd(fs::File::open(dir.path()).unwrap().as_fd()).unwrap();
        let timeout = Duration::from_millis(1);
        let config = || {
            let mut config = Config {
                include: vec!["*.log".into()],
                ..Config::default()
            };
            config.checkpoint.ownership_timeout = timeout;
            config
                .compile("group", "pipeline", "node", dir.path())
                .unwrap()
        };
        let lifecycle = Workers::new(Duration::from_secs(5)).unwrap();
        let cancel = lifecycle.cancellation();
        let running = WorkerInputs::open(config(), &root, &cancel).unwrap();
        assert!(matches!(WorkerInputs::open(config(), &root, &cancel),
            Err(StartupError::OwnershipTimeout(wait)) if wait == timeout));
        assert!(running.store.state().is_ok());
        drop(running);
        let replacement = WorkerInputs::open(config(), &root, &cancel).unwrap();
        assert!(replacement.store.state().is_ok());
    }
}
