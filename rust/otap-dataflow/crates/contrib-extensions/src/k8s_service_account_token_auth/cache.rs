// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded, TTL'd decision cache.
//!
//! Entries are keyed by the SHA-256 digest of the token, so the cache retains
//! only digests. Each entry holds a lazily-initialized cell that concurrent
//! requests for one token share, collapsing them onto a single `TokenReview`.
//!
//! [`SharedDecisionCache`] and [`LocalDecisionCache`] implement separate hot
//! paths for their concurrency models over a common [`Entries`] map, which
//! holds TTL, bounds, and eviction.

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use otap_df_engine::capability::auth::AuthzDecision;
use sha2::{Digest, Sha256};
use tokio::sync::OnceCell;

use super::error::Error;

/// The cache key: a SHA-256 digest of the token.
pub(crate) type TokenDigest = [u8; 32];

/// The result published to every request sharing one in-flight review.
type FlightResult = Result<AuthzDecision, Error>;

/// A reference-counted handle to a decision cell, shared by every request
/// bearing the same token.
///
/// `Arc` for the shared variant, whose slots cross threads; `Rc` for the local
/// variant, which gains a compile-time `!Send` check.
pub(crate) trait SlotHandle: Clone + Deref<Target = OnceCell<FlightResult>> {
    /// Allocates a fresh, empty cell.
    fn empty() -> Self;

    /// Returns whether both handles point to the same cell.
    fn ptr_eq(left: &Self, right: &Self) -> bool;
}

impl SlotHandle for Arc<OnceCell<FlightResult>> {
    fn empty() -> Self {
        Arc::new(OnceCell::new())
    }

    fn ptr_eq(left: &Self, right: &Self) -> bool {
        Arc::ptr_eq(left, right)
    }
}

impl SlotHandle for Rc<OnceCell<FlightResult>> {
    fn empty() -> Self {
        Rc::new(OnceCell::new())
    }

    fn ptr_eq(left: &Self, right: &Self) -> bool {
        Rc::ptr_eq(left, right)
    }
}

/// Slot handle used by the shared (`Send`, cross-thread) variant.
pub(crate) type SharedSlot = Arc<OnceCell<FlightResult>>;

/// Slot handle used by the local (`!Send`, thread-per-core) variant.
pub(crate) type LocalSlot = Rc<OnceCell<FlightResult>>;

/// How many entries to sample when choosing an eviction victim, bounding
/// eviction to O(1) instead of scanning the whole map under the guard.
const EVICTION_SAMPLE: usize = 8;

/// Computes the cache key for a token.
pub(crate) fn digest(token: &str) -> TokenDigest {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.finalize().into()
}

// -- Shared variant cache (Send; Mutex + Arc slots) -------------------------

/// Decision cache for the shared (`Send`) capability variant.
pub(crate) struct SharedDecisionCache {
    entries: Mutex<Entries<SharedSlot>>,
}

impl SharedDecisionCache {
    /// Creates an empty cache with the given TTL and entry cap.
    pub(crate) fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: Mutex::new(Entries::new(ttl, max_entries)),
        }
    }

    /// Returns the decision for `key`, awaiting `decide` only if no decision has
    /// been reached and no other request is already reaching one.
    ///
    /// `key` is precomputed by the caller so hashing stays out of the guard.
    pub(crate) async fn get_or_decide<Fut>(
        &self,
        key: TokenDigest,
        decide: impl FnOnce() -> Fut,
    ) -> Result<AuthzDecision, Error>
    where
        Fut: Future<Output = Result<AuthzDecision, Error>>,
    {
        let lease = match self.entries.lock() {
            Ok(mut entries) => entries.slot(key, Instant::now()),
            // A poisoned lock degrades to an uncached decision: the caller still
            // authorizes, it just cannot share the result.
            Err(_) => SlotLease::untracked(SharedSlot::empty()),
        };

        let mut cleanup = SharedFlightCleanup {
            cache: self,
            key,
            cell: lease.handle(),
            armed: lease.tracked && lease.complete_on_success,
        };
        let result = lease.handle().get_or_init(decide).await;
        cleanup.armed = false;
        match result {
            Ok(decision) => {
                if lease.complete_on_success {
                    if let Ok(mut entries) = self.entries.lock() {
                        entries.complete(&key, lease.handle(), Instant::now());
                    }
                }
                // Cloned after the guard is released; a deep clone under it
                // would stall requests for unrelated tokens.
                Ok(decision.clone())
            }
            Err(error) => {
                if lease.tracked {
                    if let Ok(mut entries) = self.entries.lock() {
                        entries.remove(&key, lease.handle());
                    }
                }
                Err(error.clone())
            }
        }
    }

    /// Number of references to the tracked slot for `key`. Test-only.
    #[cfg(test)]
    pub(crate) fn slot_ref_count(&self, key: &TokenDigest) -> usize {
        self.entries
            .lock()
            .ok()
            .and_then(|entries| {
                entries
                    .entries
                    .get(key)
                    .map(|entry| Arc::strong_count(&entry.cell))
            })
            .unwrap_or(0)
    }
}

// -- Local variant cache (!Send; RefCell + Rc slots) ------------------------

/// Decision cache for the local (`!Send`, thread-per-core) capability variant.
pub(crate) struct LocalDecisionCache {
    entries: RefCell<Entries<LocalSlot>>,
}

impl LocalDecisionCache {
    /// Creates an empty cache with the given TTL and entry cap.
    pub(crate) fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: RefCell::new(Entries::new(ttl, max_entries)),
        }
    }

    /// Returns the decision for `key`, awaiting `decide` only if no decision has
    /// been reached and no other task is already reaching one.
    ///
    /// The shared slot is required despite the single thread: tasks interleave
    /// at the `.await` below, so without it each interleaved task bearing one
    /// token would issue its own `TokenReview`.
    pub(crate) async fn get_or_decide<Fut>(
        &self,
        key: TokenDigest,
        decide: impl FnOnce() -> Fut,
    ) -> Result<AuthzDecision, Error>
    where
        Fut: Future<Output = Result<AuthzDecision, Error>>,
    {
        let lease = {
            let mut entries = self.entries.borrow_mut();

            // Cloned under the borrow: no `.await` separates the probe from the
            // clone, so no other task can run and the refcount is never touched.
            if let Some(decision) = entries
                .live_cell(&key, Instant::now())
                .and_then(|c| c.get())
                .and_then(|result| result.as_ref().ok())
            {
                return Ok(decision.clone());
            }

            entries.slot(key, Instant::now())
        };

        // The borrow must be dropped above: re-entering the cache while borrowed
        // across the await below would panic.
        let mut cleanup = LocalFlightCleanup {
            cache: self,
            key,
            cell: lease.handle(),
            armed: lease.tracked && lease.complete_on_success,
        };
        let result = lease.handle().get_or_init(decide).await;
        cleanup.armed = false;
        match result {
            Ok(decision) => {
                if lease.complete_on_success {
                    self.entries
                        .borrow_mut()
                        .complete(&key, lease.handle(), Instant::now());
                }
                Ok(decision.clone())
            }
            Err(error) => {
                if lease.tracked {
                    self.entries.borrow_mut().remove(&key, lease.handle());
                }
                Err(error.clone())
            }
        }
    }
}

/// Removes an abandoned shared flight when its last waiter is dropped.
struct SharedFlightCleanup<'a> {
    cache: &'a SharedDecisionCache,
    key: TokenDigest,
    cell: &'a SharedSlot,
    armed: bool,
}

impl Drop for SharedFlightCleanup<'_> {
    fn drop(&mut self) {
        if !self.armed || Arc::strong_count(self.cell) != 2 {
            return;
        }
        if let Ok(mut entries) = self.cache.entries.lock() {
            entries.remove(&self.key, self.cell);
        }
    }
}

/// Removes an abandoned local flight when its last waiter is dropped.
struct LocalFlightCleanup<'a> {
    cache: &'a LocalDecisionCache,
    key: TokenDigest,
    cell: &'a LocalSlot,
    armed: bool,
}

impl Drop for LocalFlightCleanup<'_> {
    fn drop(&mut self) {
        if !self.armed || Rc::strong_count(self.cell) != 2 {
            return;
        }
        self.cache.entries.borrow_mut().remove(&self.key, self.cell);
    }
}

/// A slot returned to a request and its cache bookkeeping state.
pub(crate) struct SlotLease<S> {
    cell: S,
    tracked: bool,
    complete_on_success: bool,
}

impl<S> SlotLease<S> {
    fn tracked(cell: S, complete_on_success: bool) -> Self {
        Self {
            cell,
            tracked: true,
            complete_on_success,
        }
    }

    fn untracked(cell: S) -> Self {
        Self {
            cell,
            tracked: false,
            complete_on_success: false,
        }
    }

    /// Returns the underlying decision cell.
    pub(crate) fn handle(&self) -> &S {
        &self.cell
    }

    /// Returns whether this slot is retained by the cache.
    #[cfg(test)]
    pub(crate) fn is_tracked(&self) -> bool {
        self.tracked
    }
}

/// A decision cell and its expiration after successful initialization.
struct CachedDecision<S> {
    cell: S,
    expires_at: Option<Instant>,
}

/// Bounded, TTL'd entry map used by both caches.
///
/// Holds TTL, bounds, and eviction; each cache supplies its own guard and slot
/// handle.
pub(crate) struct Entries<S: SlotHandle> {
    entries: HashMap<TokenDigest, CachedDecision<S>>,
    ttl: Duration,
    max_entries: usize,
}

impl<S: SlotHandle> Entries<S> {
    /// Creates an empty cache with the given TTL and entry cap.
    pub(crate) fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
            max_entries,
        }
    }

    /// Returns the live cell for `key`, or `None` when the token is absent or
    /// its entry has expired.
    ///
    /// Hands back a borrow, letting a caller that cannot be preempted read a
    /// decision while leaving the refcount untouched.
    pub(crate) fn live_cell(&self, key: &TokenDigest, now: Instant) -> Option<&S> {
        self.entries
            .get(key)
            .filter(|entry| entry.expires_at.is_some_and(|expiry| expiry > now))
            .map(|entry| &entry.cell)
    }

    /// Returns the shared slot for `key`, reusing a live entry when present and
    /// otherwise claiming a fresh one.
    ///
    /// The first requester initializes the slot while the rest await it, so a
    /// stampede for one token costs one `TokenReview`.
    pub(crate) fn slot(&mut self, key: TokenDigest, now: Instant) -> SlotLease<S> {
        if let Some(entry) = self.entries.get(&key) {
            match entry.expires_at {
                None => return SlotLease::tracked(entry.cell.clone(), true),
                Some(expiry) if expiry > now => {
                    return SlotLease::tracked(entry.cell.clone(), false);
                }
                Some(_) => {}
            }

            // Replaced in place, which never grows the map.
            let cell = S::empty();
            let _ = self.entries.insert(key, Self::fresh_entry(&cell));
            return SlotLease::tracked(cell, true);
        }

        let cell = S::empty();
        if !self.make_room() {
            // When capacity is occupied by in-flight reviews, preserve those
            // flights and decide this distinct token without retaining it.
            return SlotLease::untracked(cell);
        }
        let _ = self.entries.insert(key, Self::fresh_entry(&cell));
        SlotLease::tracked(cell, true)
    }

    /// Builds an in-flight entry wrapping `cell`.
    fn fresh_entry(cell: &S) -> CachedDecision<S> {
        CachedDecision {
            cell: cell.clone(),
            expires_at: None,
        }
    }

    /// Starts the TTL after a tracked slot successfully reaches a decision.
    pub(crate) fn complete(&mut self, key: &TokenDigest, cell: &S, now: Instant) {
        let Some(entry) = self.entries.get_mut(key) else {
            return;
        };
        if S::ptr_eq(&entry.cell, cell) && entry.expires_at.is_none() {
            entry.expires_at = Some(
                now.checked_add(self.ttl)
                    .unwrap_or_else(|| now + Duration::from_secs(1)),
            );
        }
    }

    /// Removes `key` only when it still names `cell`.
    pub(crate) fn remove(&mut self, key: &TokenDigest, cell: &S) {
        let matches = self
            .entries
            .get(key)
            .is_some_and(|entry| S::ptr_eq(&entry.cell, cell));
        if matches {
            let _ = self.entries.remove(key);
        }
    }

    /// Ensures there is room for one more entry by evicting one completed slot
    /// from a bounded sample. In-flight reviews are never evicted.
    ///
    /// Eviction protects availability: the key derives from caller-supplied
    /// token bytes, so an unauthenticated caller can mint unlimited distinct
    /// keys and would otherwise pin the cache.
    fn make_room(&mut self) -> bool {
        if self.max_entries == 0 {
            return false;
        }
        if self.entries.len() < self.max_entries {
            return true;
        }

        let victim = self
            .entries
            .iter()
            .take(EVICTION_SAMPLE)
            .filter_map(|(key, entry)| entry.expires_at.map(|expiry| (*key, expiry)))
            .min_by_key(|(_, expiry)| *expiry)
            .map(|(key, _)| key);
        if let Some(key) = victim {
            let _ = self.entries.remove(&key);
            true
        } else {
            false
        }
    }

    /// Number of live entries currently held. Test-only.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}
