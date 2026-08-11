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

use otap_df_engine::capability::CapabilityError;
use otap_df_engine::capability::auth::AuthzDecision;
use sha2::{Digest, Sha256};
use tokio::sync::OnceCell;

/// The cache key: a SHA-256 digest of the token.
pub(crate) type TokenDigest = [u8; 32];

/// A reference-counted handle to a decision cell, shared by every request
/// bearing the same token.
///
/// `Arc` for the shared variant, whose slots cross threads; `Rc` for the local
/// variant, which gains a compile-time `!Send` check.
pub(crate) trait SlotHandle: Clone + Deref<Target = OnceCell<AuthzDecision>> {
    /// Allocates a fresh, empty cell.
    fn empty() -> Self;
}

impl SlotHandle for Arc<OnceCell<AuthzDecision>> {
    fn empty() -> Self {
        Arc::new(OnceCell::new())
    }
}

impl SlotHandle for Rc<OnceCell<AuthzDecision>> {
    fn empty() -> Self {
        Rc::new(OnceCell::new())
    }
}

/// Slot handle used by the shared (`Send`, cross-thread) variant.
pub(crate) type SharedSlot = Arc<OnceCell<AuthzDecision>>;

/// Slot handle used by the local (`!Send`, thread-per-core) variant.
pub(crate) type LocalSlot = Rc<OnceCell<AuthzDecision>>;

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
    ) -> Result<AuthzDecision, CapabilityError>
    where
        Fut: Future<Output = Result<AuthzDecision, CapabilityError>>,
    {
        let slot = match self.entries.lock() {
            Ok(mut entries) => entries.slot(key, Instant::now()),
            // A poisoned lock degrades to an uncached decision: the caller still
            // authorizes, it just cannot share the result.
            Err(_) => SharedSlot::empty(),
        };

        // Cloned after the guard is released; a deep clone under it would stall
        // requests for unrelated tokens.
        if let Some(decision) = slot.get() {
            return Ok(decision.clone());
        }

        Ok(slot.get_or_try_init(decide).await?.clone())
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
    ) -> Result<AuthzDecision, CapabilityError>
    where
        Fut: Future<Output = Result<AuthzDecision, CapabilityError>>,
    {
        let slot = {
            let mut entries = self.entries.borrow_mut();

            // Cloned under the borrow: no `.await` separates the probe from the
            // clone, so no other task can run and the refcount is never touched.
            if let Some(decision) = entries
                .live_cell(&key, Instant::now())
                .and_then(|c| c.get())
            {
                return Ok(decision.clone());
            }

            entries.slot(key, Instant::now())
        };

        // The borrow must be dropped above: re-entering the cache while borrowed
        // across the await below would panic.
        Ok(slot.get_or_try_init(decide).await?.clone())
    }
}

/// A cached decision cell and the instant it stops being valid.
struct CachedDecision<S> {
    cell: S,
    expires_at: Instant,
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
            .filter(|entry| entry.expires_at > now)
            .map(|entry| &entry.cell)
    }

    /// Returns the shared slot for `key`, reusing a live entry when present and
    /// otherwise claiming a fresh one.
    ///
    /// The first requester initializes the slot while the rest await it, so a
    /// stampede for one token costs one `TokenReview`.
    pub(crate) fn slot(&mut self, key: TokenDigest, now: Instant) -> S {
        if let Some(entry) = self.entries.get(&key) {
            if entry.expires_at > now {
                return entry.cell.clone();
            }
            // Replaced in place, which never grows the map.
            let cell = S::empty();
            let _ = self.entries.insert(key, self.fresh_entry(&cell, now));
            return cell;
        }

        let cell = S::empty();
        if self.max_entries == 0 {
            // Nothing may be cached; hand back an unshared slot so the caller
            // still reaches a decision.
            return cell;
        }
        self.make_room(now);
        let _ = self.entries.insert(key, self.fresh_entry(&cell, now));
        cell
    }

    /// Builds an entry wrapping `cell` that expires one `ttl` from `now`.
    fn fresh_entry(&self, cell: &S, now: Instant) -> CachedDecision<S> {
        CachedDecision {
            cell: cell.clone(),
            expires_at: now
                .checked_add(self.ttl)
                .unwrap_or_else(|| now + Duration::from_secs(1)),
        }
    }

    /// Ensures there is room for one more entry, reclaiming expired entries
    /// first and otherwise evicting a sampled entry closest to expiry.
    ///
    /// Eviction protects availability: the key derives from caller-supplied
    /// token bytes, so an unauthenticated caller can mint unlimited distinct
    /// keys and would otherwise pin the cache.
    fn make_room(&mut self, now: Instant) {
        if self.entries.len() < self.max_entries {
            return;
        }
        self.entries.retain(|_, entry| entry.expires_at > now);

        while self.entries.len() >= self.max_entries {
            let victim = self
                .entries
                .iter()
                .take(EVICTION_SAMPLE)
                .min_by_key(|(_, entry)| entry.expires_at)
                .map(|(key, _)| *key);
            match victim {
                Some(key) => {
                    let _ = self.entries.remove(&key);
                }
                // Empty map: nothing left to evict.
                None => return,
            }
        }
    }

    /// Number of live entries currently held. Test-only.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}
