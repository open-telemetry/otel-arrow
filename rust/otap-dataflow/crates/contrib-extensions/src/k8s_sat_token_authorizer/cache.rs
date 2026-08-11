// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded, TTL'd decision cache shared by both capability variants.
//!
//! Entries are keyed by the **SHA-256 digest** of the token, never the plaintext
//! token: no live credential is retained in the cache (a memory or core dump
//! exposes only 32-byte digests), and lookups compare unpredictable digests
//! rather than secret bytes. SHA-256 is collision-resistant, so distinct tokens
//! never share an entry.
//!
//! Each entry holds a shared, lazily-initialized cell rather than a finished
//! decision, so concurrent requests bearing the same token collapse onto a
//! single `TokenReview` (see [`DecisionStore::slot`]) instead of stampeding the
//! API server.
//!
//! The cache itself carries no synchronization: the shared variant wraps it in a
//! `std::sync::Mutex` and the local (thread-per-core) variant wraps it in a
//! `RefCell`, so the local path is lock-free.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use otap_df_engine::capability::auth::AuthzDecision;
use sha2::{Digest, Sha256};
use tokio::sync::OnceCell;

/// The cache key: a SHA-256 digest of the token. `Copy`/`Eq`/`Hash` for free.
pub(crate) type TokenDigest = [u8; 32];

/// A slot shared by every request bearing the same token: empty until the first
/// requester reaches a decision, then read by all of them.
pub(crate) type DecisionSlot = Arc<OnceCell<AuthzDecision>>;

/// How many entries to sample when choosing an eviction victim.
///
/// Scanning the whole map would put an O(n) loop inside the lock on every miss
/// once the cache is full; sampling keeps eviction O(1) while still preferring
/// an entry that is close to expiring anyway.
const EVICTION_SAMPLE: usize = 8;

/// Computes the cache key for a token without retaining the plaintext.
///
/// Hashing is deliberately the *caller's* job rather than the cache's: a SAT is
/// ~1 KB, so this is orders of magnitude more expensive than the map probe it
/// produces a key for. Computing it before taking the lock keeps the shared
/// variant's critical section down to the lookup itself.
pub(crate) fn digest(token: &str) -> TokenDigest {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.finalize().into()
}

/// Interior-mutable access to a [`DecisionCache`], abstracting over the
/// synchronization strategy so the shared decision flow ([`super::core::Core`])
/// is written once.
///
/// The shared authorizer variant implements this over a `std::sync::Mutex`
/// (cross-thread safe) and the local variant over a `RefCell` (lock-free,
/// thread-per-core). The guard/borrow is never held across an `.await`: the
/// slot is taken, the lock released, and only then is the decision awaited.
pub(crate) trait DecisionStore {
    /// Returns the shared slot for `key`, creating an empty one when the token
    /// is absent or its entry has expired.
    ///
    /// Takes a precomputed [`TokenDigest`] so the SHA-256 of the token stays
    /// outside the lock; see [`digest`].
    fn slot(&self, key: TokenDigest, now: Instant) -> DecisionSlot;
}

impl DecisionStore for std::sync::Mutex<DecisionCache> {
    fn slot(&self, key: TokenDigest, now: Instant) -> DecisionSlot {
        match self.lock() {
            Ok(mut cache) => cache.slot(key, now),
            // A poisoned lock degrades to an uncached decision (fail-safe: the
            // caller still authorizes, it just cannot share the result).
            Err(_) => Arc::new(OnceCell::new()),
        }
    }
}

impl DecisionStore for std::cell::RefCell<DecisionCache> {
    fn slot(&self, key: TokenDigest, now: Instant) -> DecisionSlot {
        self.borrow_mut().slot(key, now)
    }
}

/// A cached admission slot and the instant it stops being valid.
struct CachedDecision {
    /// Shared with every in-flight request for this token; empty until the
    /// first of them reaches a decision.
    cell: DecisionSlot,
    expires_at: Instant,
}

/// Bounded, TTL'd cache of admission decisions keyed by the token's SHA-256
/// digest.
///
/// A repeated request for the same token avoids a `TokenReview` round-trip, and
/// concurrent first-time requests for one token share a single round-trip.
/// Entries expire after `ttl`; the map is capped at `max_entries` to bound
/// memory.
pub(crate) struct DecisionCache {
    entries: HashMap<TokenDigest, CachedDecision>,
    ttl: Duration,
    max_entries: usize,
}

impl DecisionCache {
    /// Creates an empty cache with the given TTL and entry cap.
    pub(crate) fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
            max_entries,
        }
    }

    /// Returns the shared slot for `key`, reusing a live entry when present and
    /// otherwise claiming a fresh one.
    ///
    /// Returning a slot rather than a finished decision is what collapses a
    /// stampede: the first requester for a token initializes the cell while
    /// every concurrent requester awaits that same cell, so one token costs one
    /// `TokenReview` no matter how many requests race. It also keeps the
    /// decision's deep clone outside the lock -- the caller clones from the
    /// cell after the guard is dropped.
    pub(crate) fn slot(&mut self, key: TokenDigest, now: Instant) -> DecisionSlot {
        if let Some(entry) = self.entries.get(&key) {
            if entry.expires_at > now {
                return Arc::clone(&entry.cell);
            }
            // Expired: replace it in place, which never grows the map.
            let cell = Arc::new(OnceCell::new());
            let _ = self.entries.insert(key, self.fresh_entry(&cell, now));
            return cell;
        }

        let cell = Arc::new(OnceCell::new());
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
    fn fresh_entry(&self, cell: &DecisionSlot, now: Instant) -> CachedDecision {
        CachedDecision {
            cell: Arc::clone(cell),
            expires_at: now
                .checked_add(self.ttl)
                .unwrap_or_else(|| now + Duration::from_secs(1)),
        }
    }

    /// Ensures there is room for one more entry, reclaiming expired entries
    /// first and otherwise evicting a sampled entry closest to expiry.
    ///
    /// Evicting rather than skipping matters for availability: the cache key is
    /// derived from caller-supplied token bytes, so an unauthenticated caller
    /// can mint unlimited distinct keys. Refusing to cache while full would let
    /// such traffic pin the cache and force a `TokenReview` round-trip for
    /// every legitimate request.
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
