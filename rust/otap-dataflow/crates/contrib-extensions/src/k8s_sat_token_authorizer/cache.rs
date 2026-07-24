// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded, TTL'd decision cache shared by both capability variants.
//!
//! The cache itself carries no synchronization: the shared variant wraps it in a
//! `std::sync::Mutex` and the local (thread-per-core) variant wraps it in a
//! `RefCell`, so the local path is lock-free.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use otap_df_engine::capability::auth::AuthzDecision;

/// Interior-mutable access to a [`DecisionCache`], abstracting over the
/// synchronization strategy so the shared decision flow ([`super::core::Core`])
/// is written once.
///
/// The shared authorizer variant implements this over a `std::sync::Mutex`
/// (cross-thread safe) and the local variant over a `RefCell` (lock-free,
/// thread-per-core). Neither method holds its guard/borrow across an `.await`.
pub(crate) trait DecisionStore {
    /// Returns the cached decision for `token` when present and unexpired.
    fn get(&self, token: &str, now: Instant) -> Option<AuthzDecision>;
    /// Inserts (or refreshes) the decision for `token`.
    fn insert(&self, token: String, decision: AuthzDecision, now: Instant);
}

impl DecisionStore for std::sync::Mutex<DecisionCache> {
    fn get(&self, token: &str, now: Instant) -> Option<AuthzDecision> {
        self.lock().ok().and_then(|cache| cache.get(token, now))
    }

    fn insert(&self, token: String, decision: AuthzDecision, now: Instant) {
        if let Ok(mut cache) = self.lock() {
            cache.insert(token, decision, now);
        }
    }
}

impl DecisionStore for std::cell::RefCell<DecisionCache> {
    fn get(&self, token: &str, now: Instant) -> Option<AuthzDecision> {
        self.borrow().get(token, now)
    }

    fn insert(&self, token: String, decision: AuthzDecision, now: Instant) {
        self.borrow_mut().insert(token, decision, now);
    }
}

/// A cached admission decision and the instant it stops being valid.
struct CachedDecision {
    decision: AuthzDecision,
    expires_at: Instant,
}

/// Bounded, TTL'd cache of admission decisions keyed by the opaque token.
///
/// Keyed by the token string exactly as presented so a repeated request avoids a
/// `TokenReview` round-trip. Entries expire after `ttl`; the map is capped at
/// `max_entries` to bound memory.
pub(crate) struct DecisionCache {
    entries: HashMap<String, CachedDecision>,
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

    /// Returns the cached decision for `token` when present and unexpired.
    pub(crate) fn get(&self, token: &str, now: Instant) -> Option<AuthzDecision> {
        let entry = self.entries.get(token)?;
        if entry.expires_at > now {
            Some(entry.decision.clone())
        } else {
            None
        }
    }

    /// Inserts (or refreshes) the decision for `token`, evicting expired entries
    /// first and skipping the insert if still at capacity (so the cache never
    /// exceeds `max_entries`).
    pub(crate) fn insert(&mut self, token: String, decision: AuthzDecision, now: Instant) {
        let expires_at = now
            .checked_add(self.ttl)
            .unwrap_or_else(|| now + Duration::from_secs(1));
        let cached = CachedDecision {
            decision,
            expires_at,
        };

        // Updating an existing key never grows the map, so refresh it in place.
        if let Some(slot) = self.entries.get_mut(&token) {
            *slot = cached;
            return;
        }

        if self.entries.len() >= self.max_entries {
            // Reclaim space from expired entries before giving up on caching.
            self.entries.retain(|_, entry| entry.expires_at > now);
            if self.entries.len() >= self.max_entries {
                // Still full of live entries: skip caching rather than exceed
                // the bound. The decision is still returned to the caller.
                return;
            }
        }

        let _ = self.entries.insert(token, cached);
    }

    /// Number of live entries currently held. Test-only.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}
