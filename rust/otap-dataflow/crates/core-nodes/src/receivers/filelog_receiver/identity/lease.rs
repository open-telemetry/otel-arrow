// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Process-wide source exclusion, independent of checkpoint namespaces.

use super::super::source::FileLocator;
use std::collections::BTreeMap;
use std::sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicUsize, Ordering},
};

static PROCESS_LEASES: LazyLock<Arc<LeaseRegistry>> = LazyLock::new(|| {
    Arc::new(LeaseRegistry {
        state: Mutex::default(),
    })
});

/// Acquisition/release never performs I/O or waits for source ownership.
#[derive(Debug, thiserror::Error)]
pub enum LeaseError {
    /// Another logical reader owns this locator, even in another namespace.
    #[error("filelog locator already owned by {owner}")]
    Conflict {
        /// Bounded logical node name for a rate-limited health event.
        owner: Arc<str>,
        /// True only for this logical owner, regardless of diagnostic name.
        same_owner: bool,
    },
    /// The owner's admitted population is exhausted.
    #[error("filelog runtime lease capacity exhausted")]
    Capacity,
    /// Poisoning or inconsistent release permanently blocks this registry.
    #[error("filelog runtime lease registry integrity failure")]
    Integrity,
    /// Invalid owner name or population limit.
    #[error("invalid filelog lease owner")]
    InvalidOwner,
}

#[derive(Debug)]
struct Entry {
    owner: u64,
    name: Arc<str>,
}
#[derive(Debug, Default)]
struct State {
    next_owner: u64,
    failed: bool,
    entries: BTreeMap<(u64, u64), Entry>,
}

/// One shared registry across all Filelog nodes in a process.
///
/// Production uses `process`; tests may inject isolated registries. B-tree nodes
/// are reclaimed on release, avoiding retained hash-table capacity after churn.
#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
pub struct LeaseRegistry {
    state: Mutex<State>,
}

impl LeaseRegistry {
    /// Returns the process-wide production registry.
    #[must_use]
    pub fn process() -> Arc<Self> {
        Arc::clone(&PROCESS_LEASES)
    }

    /// Register one logical reader population, bounded by validated tracked capacity.
    /// Call only on a blocking worker; async pipeline tasks must not take this lock.
    pub fn owner(self: &Arc<Self>, name: &str, limit: usize) -> Result<LeaseOwner, LeaseError> {
        if name.is_empty() || name.len() > 255 || limit == 0 {
            return Err(LeaseError::InvalidOwner);
        }
        let mut state = self.state.lock().map_err(|_| LeaseError::Integrity)?;
        if state.failed {
            return Err(LeaseError::Integrity);
        }
        state.next_owner = state
            .next_owner
            .checked_add(1)
            .ok_or(LeaseError::Integrity)?;
        Ok(LeaseOwner(Arc::new(Owner {
            registry: Arc::clone(self),
            id: state.next_owner,
            name: Arc::from(name),
            limit,
            held: AtomicUsize::new(0),
        })))
    }

    /// Current entry count for integrated memory accounting; no paths or payloads.
    pub fn len(&self) -> Result<usize, LeaseError> {
        let state = self.state.lock().map_err(|_| LeaseError::Integrity)?;
        if state.failed {
            return Err(LeaseError::Integrity);
        }
        Ok(state.entries.len())
    }

    /// Whether no logical readers own locators.
    pub fn is_empty(&self) -> Result<bool, LeaseError> {
        self.len().map(|n| n == 0)
    }
}

#[derive(Debug)]
struct Owner {
    registry: Arc<LeaseRegistry>,
    id: u64,
    name: Arc<str>,
    limit: usize,
    held: AtomicUsize,
}

/// Shared capacity and diagnostic identity for one Filelog node instance.
// Intentionally not Clone: each admission owner has a distinct lease identity.
#[derive(Debug)]
pub struct LeaseOwner(Arc<Owner>);
impl LeaseOwner {
    pub(super) fn limit(&self) -> usize {
        self.0.limit
    }

    pub(super) fn owns(&self, guard: &LeaseGuard) -> bool {
        Arc::ptr_eq(&self.0, &guard.owner)
    }

    /// Try once. A conflict is an observable outcome, never an ignored candidate.
    pub fn acquire(&self, locator: FileLocator) -> Result<LeaseGuard, LeaseError> {
        let mut state = self
            .0
            .registry
            .state
            .lock()
            .map_err(|_| LeaseError::Integrity)?;
        if state.failed {
            return Err(LeaseError::Integrity);
        }
        let key = (locator.device, locator.inode);
        if let Some(entry) = state.entries.get(&key) {
            return Err(LeaseError::Conflict {
                owner: Arc::clone(&entry.name),
                same_owner: entry.owner == self.0.id,
            });
        }
        let held = self.0.held.load(Ordering::Relaxed);
        if held >= self.0.limit {
            return Err(LeaseError::Capacity);
        }
        let _ = state.entries.insert(
            key,
            Entry {
                owner: self.0.id,
                name: Arc::clone(&self.0.name),
            },
        );
        self.0.held.store(held + 1, Ordering::Relaxed);
        Ok(LeaseGuard {
            owner: Arc::clone(&self.0),
            locator,
            released: false,
        })
    }
}

/// Own with the logical reader, not its temporary source descriptor.
/// Drop releases on unwind; a poisoned/inconsistent registry remains fail-closed.
#[derive(Debug)]
pub struct LeaseGuard {
    owner: Arc<Owner>,
    locator: FileLocator,
    released: bool,
}
impl LeaseGuard {
    /// Locator protected by this guard.
    #[must_use]
    pub fn locator(&self) -> FileLocator {
        self.locator
    }

    // Preparation and commit are separate worker turns. Recheck registry integrity
    // before a prepared reader can become visible; no I/O occurs under this lock.
    pub(super) fn validate(&self) -> Result<(), LeaseError> {
        let mut state = self
            .owner
            .registry
            .state
            .lock()
            .map_err(|_| LeaseError::Integrity)?;
        if state.failed {
            return Err(LeaseError::Integrity);
        }
        let key = (self.locator.device, self.locator.inode);
        if !state
            .entries
            .get(&key)
            .is_some_and(|entry| entry.owner == self.owner.id)
        {
            state.failed = true;
            return Err(LeaseError::Integrity);
        }
        Ok(())
    }

    /// Explicit normal release reports integrity failures to the lifecycle owner.
    pub fn release(mut self) -> Result<(), LeaseError> {
        self.release_inner()
    }

    fn release_inner(&mut self) -> Result<(), LeaseError> {
        if self.released {
            return Ok(());
        }
        let mut state = self
            .owner
            .registry
            .state
            .lock()
            .map_err(|_| LeaseError::Integrity)?;
        if state.failed {
            return Err(LeaseError::Integrity);
        }
        let key = (self.locator.device, self.locator.inode);
        let held = self.owner.held.load(Ordering::Relaxed);
        if held == 0
            || !state
                .entries
                .get(&key)
                .is_some_and(|entry| entry.owner == self.owner.id)
        {
            state.failed = true;
            return Err(LeaseError::Integrity);
        }
        let _ = state.entries.remove(&key);
        self.owner.held.store(held - 1, Ordering::Relaxed);
        self.released = true;
        Ok(())
    }
}
impl Drop for LeaseGuard {
    fn drop(&mut self) {
        let _ = self.release_inner();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Two distinct owners have the same name and contend for a held locator.
    /// Guarantees: Self-conflict uses owner identity, and neither conflict releases the lease.
    #[test]
    fn conflicts_distinguish_owner_identity_from_name() {
        let registry = Arc::new(LeaseRegistry::default());
        let first = registry.owner("same-name", 2).unwrap();
        let second = registry.owner("same-name", 2).unwrap();
        let locator = FileLocator {
            device: 1,
            inode: 2,
        };
        let held = first.acquire(locator).unwrap();
        assert!(matches!(
            first.acquire(locator),
            Err(LeaseError::Conflict {
                same_owner: true,
                ..
            })
        ));
        assert!(matches!(
            second.acquire(locator),
            Err(LeaseError::Conflict {
                same_owner: false,
                ..
            })
        ));
        assert_eq!(registry.len().unwrap(), 1);
        held.release().unwrap();
        second.acquire(locator).unwrap().release().unwrap();
        assert!(registry.is_empty().unwrap());
    }

    /// Scenario: A registry lock is poisoned by an unwinding worker.
    /// Guarantees: Subsequent acquisition/release fail closed and never silently clear ownership.
    #[test]
    fn poisoned_registry_stays_closed() {
        let registry = Arc::new(LeaseRegistry::default());
        let owner = registry.owner("test", 2).unwrap();
        let guard = owner
            .acquire(FileLocator {
                device: 1,
                inode: 1,
            })
            .unwrap();
        let _ = std::panic::catch_unwind(|| {
            let _lock = registry.state.lock().unwrap();
            panic!("poison");
        });
        assert!(matches!(
            owner.acquire(FileLocator {
                device: 1,
                inode: 2
            }),
            Err(LeaseError::Integrity)
        ));
        assert!(matches!(guard.release(), Err(LeaseError::Integrity)));
        assert_eq!(
            registry
                .state
                .lock()
                .unwrap_err()
                .into_inner()
                .entries
                .len(),
            1
        );
    }

    /// Scenario: A release discovers that its locator entry no longer names its owner.
    /// Guarantees: It does not remove the conflicting entry and permanently latches registry failure.
    #[test]
    fn inconsistent_release_latches_failure() {
        let registry = Arc::new(LeaseRegistry::default());
        let owner = registry.owner("test", 1).unwrap();
        let guard = owner
            .acquire(FileLocator {
                device: 1,
                inode: 1,
            })
            .unwrap();
        registry
            .state
            .lock()
            .unwrap()
            .entries
            .get_mut(&(1, 1))
            .unwrap()
            .owner += 1;
        assert!(matches!(guard.release(), Err(LeaseError::Integrity)));
        assert!(matches!(
            registry.owner("new", 1),
            Err(LeaseError::Integrity)
        ));
        assert_eq!(registry.state.lock().unwrap().entries.len(), 1);
    }
}

#[cfg(test)]
mod admission_tests {
    use super::super::{
        IdentityError,
        decision::Context,
        tests::{Fixture, prepare, ready},
    };
    use super::*;

    /// Scenario: A registry is poisoned after source preparation but before checkpoint commit.
    /// Guarantees: No registration or reader is published, and the admission owner stops.
    #[test]
    fn registry_failure_between_prepare_and_commit_blocks_admission() {
        let f = Fixture::new();
        let path = f.write("app", b"abc");
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        let registry = Arc::clone(&owner.leases.0.registry);
        let _ = std::panic::catch_unwind(|| {
            let _lock = registry.state.lock().unwrap();
            panic!("controlled registry failure");
        });
        assert!(matches!(
            owner.commit(plan),
            Err(IdentityError::Lease(LeaseError::Integrity))
        ));
        assert!(matches!(owner.state(), Err(IdentityError::Stopped)));
        assert_eq!(owner.store.state().unwrap().records().len(), 0);
    }
}

#[cfg(test)]
mod quarantine_tests {
    use super::super::super::config::RecoveryMismatch;
    use super::super::{
        CommittedAdmission, Durability, IdentityError, PreparedAdmission, cp,
        decision::Context,
        tests::{Fixture, prepare, ready},
    };
    use super::*;

    /// Scenario: Lease cleanup fails after the mismatch quarantine transaction is durable.
    /// Guarantees: The committed quarantine ID remains reportable and the owner stops before accepting more work.
    #[test]
    fn durable_quarantine_survives_cleanup_error() {
        let f = Fixture::new();
        let path = f.write("app", b"old");
        let inputs = f.inputs(|c| c.identity.on_recovery_mismatch = RecoveryMismatch::Fail);
        let mut owner = f.owner(&inputs);
        let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        drop(owner.commit(plan).unwrap());
        std::fs::write(&path, b"new").unwrap();
        let PreparedAdmission {
            seed, operations, ..
        } = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        let id = seed.file_id;
        assert!(
            owner
                .append_checkpoint(operations, Durability::Sync)
                .unwrap()
                .synced
        );
        let registry = Arc::clone(&owner.leases.0.registry);
        let _ = std::panic::catch_unwind(|| {
            let _lock = registry.state.lock().unwrap();
            panic!("cleanup failure");
        });
        assert!(
            matches!(owner.complete_quarantine(seed), CommittedAdmission::Quarantined { file_id, release_error: Some(LeaseError::Integrity), .. } if file_id == id)
        );
        assert!(matches!(owner.state(), Err(IdentityError::Stopped)));
        assert_eq!(
            owner
                .store
                .state()
                .unwrap()
                .get(&id)
                .unwrap()
                .lifecycle_state,
            cp::LifecycleState::Quarantined
        );
    }
}

#[cfg(test)]
mod pending_integrity_tests {
    use super::super::{
        IdentityError,
        decision::Context,
        tests::{Fixture, prepare, ready},
    };
    use super::*;

    /// Scenario: Registry poisoning occurs while an admission's recoverable write waits for resolution.
    /// Guarantees: Pending resolution does not publish a reader through a failed registry.
    #[test]
    fn pending_resolution_rechecks_lease() {
        let f = Fixture::new();
        let path = f.write("app", b"abc");
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let plan = ready(prepare(&mut owner, &path, Context::Unrelated).unwrap());
        owner.store.inject_write_failure_after(0);
        assert!(matches!(
            owner.commit(plan),
            Err(IdentityError::PendingWrite(_))
        ));
        let registry = Arc::clone(&owner.leases.0.registry);
        let _ = std::panic::catch_unwind(|| {
            let _lock = registry.state.lock().unwrap();
            panic!("controlled registry failure");
        });
        let result = owner.resolve_pending();
        assert!(
            matches!(result, Err(IdentityError::Lease(LeaseError::Integrity))),
            "reader published through failed registry: {result:?}"
        );
        assert!(owner.is_stopped());
        assert!(owner.has_pending_write());
    }
}
