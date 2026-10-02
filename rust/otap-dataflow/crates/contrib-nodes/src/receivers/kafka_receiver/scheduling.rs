// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared retry-scheduling primitives.
//!
//! Both retry subsystems in the Kafka receiver -- transient-NACK replay
//! ([`super::retry`]) and assignment-resume ([`super::rebalance`]) -- keep a
//! per-key state map alongside an ordered, one-entry-per-key deadline index and
//! reschedule with capped exponential backoff. This module factors out the two
//! mechanical pieces they share while each subsystem keeps its own state
//! machine and payload:
//!
//! - [`DeadlineIndex`]: an ordered `deadline -> key` index with a staleness
//!   `Token`, so an in-flight attempt cannot be confused with a newer schedule
//!   for the same key.
//! - [`capped_exponential_backoff`] and [`checked_deadline`]: the backoff curve
//!   and an overflow-safe `now + delay` used to place a deadline.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

/// A single scheduled entry: ordered by `deadline` first so the earliest-due
/// entry is always `BTreeSet::first`. `token` and `key` disambiguate entries
/// that share a deadline and let a caller remove the exact entry it inserted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Scheduled<K, T> {
    deadline: Instant,
    token: T,
    key: K,
}

/// A key whose scheduled deadline has elapsed, returned by
/// [`DeadlineIndex::take_due`] with the `token` it was scheduled under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Due<K, T> {
    /// The key whose deadline elapsed.
    pub(crate) key: K,
    /// The staleness token the key was scheduled under.
    pub(crate) token: T,
    /// The deadline that elapsed.
    pub(crate) deadline: Instant,
}

/// An ordered, one-entry-per-key deadline index.
///
/// The index owns only scheduling order; the caller owns the per-key payload
/// and is responsible for validating the `token` returned by [`Self::take_due`]
/// against its own current state before acting (a `take_due` result is only a
/// hint that a deadline elapsed, not proof the caller's state is unchanged).
///
/// `K` is the scheduling key (e.g. a `(topic, partition)` pair) and `T` is a
/// monotonic staleness token (e.g. a version counter or a delivery generation).
#[derive(Debug)]
pub(crate) struct DeadlineIndex<K, T> {
    scheduled: BTreeSet<Scheduled<K, T>>,
}

impl<K, T> Default for DeadlineIndex<K, T>
where
    K: Ord + Clone,
    T: Ord + Copy,
{
    fn default() -> Self {
        Self {
            scheduled: BTreeSet::new(),
        }
    }
}

impl<K, T> DeadlineIndex<K, T>
where
    K: Ord + Clone,
    T: Ord + Copy,
{
    /// Creates an empty index.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Inserts a scheduled entry for `key` at `deadline` under `token`.
    ///
    /// Callers that keep one live deadline per key must [`Self::remove`] the
    /// previous entry first; this method only deduplicates identical
    /// `(deadline, token, key)` triples.
    pub(crate) fn insert(&mut self, key: K, token: T, deadline: Instant) {
        let _ = self.scheduled.insert(Scheduled {
            deadline,
            token,
            key,
        });
    }

    /// Removes the entry matching `(key, token, deadline)` exactly. Returns
    /// `true` if an entry was removed.
    pub(crate) fn remove(&mut self, key: &K, token: T, deadline: Instant) -> bool {
        self.scheduled.remove(&Scheduled {
            deadline,
            token,
            key: key.clone(),
        })
    }

    /// Returns the earliest scheduled deadline, if any.
    #[must_use]
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.scheduled.first().map(|entry| entry.deadline)
    }

    /// Pops up to `limit` entries whose deadline is at or before `now`, in
    /// deadline order.
    ///
    /// Each popped entry is removed from the index and returned as a [`Due`];
    /// the caller must re-validate the `token` against its own per-key state
    /// (an entry may be stale if the caller rescheduled or dropped the key
    /// after this entry was inserted).
    #[must_use]
    pub(crate) fn take_due(&mut self, now: Instant, limit: usize) -> Vec<Due<K, T>> {
        let mut due = Vec::with_capacity(limit.min(self.scheduled.len()));
        while due.len() < limit {
            let Some(first) = self.scheduled.first() else {
                break;
            };
            if first.deadline > now {
                break;
            }
            let Some(entry) = self.scheduled.pop_first() else {
                break;
            };
            due.push(Due {
                key: entry.key,
                token: entry.token,
                deadline: entry.deadline,
            });
        }
        due
    }

    /// Returns `true` if the index has no scheduled entries.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.scheduled.is_empty()
    }

    /// Returns the number of scheduled entries.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.scheduled.len()
    }

    /// Returns the scheduled entries as an ordered `(deadline, token, key)` set
    /// for consistency assertions in caller tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn snapshot(&self) -> BTreeSet<(Instant, T, K)> {
        self.scheduled
            .iter()
            .map(|s| (s.deadline, s.token, s.key.clone()))
            .collect()
    }
}

/// Capped exponential backoff: `initial * 2^attempts`, saturating and clamped to
/// `max`.
///
/// `attempts` is the number of prior attempts (0 yields `initial`). Both the
/// shift and the multiply saturate, so a large `attempts` yields `max` rather
/// than overflowing.
#[must_use]
pub(crate) fn capped_exponential_backoff(
    initial: Duration,
    max: Duration,
    attempts: u32,
) -> Duration {
    let multiplier = 1_u32.checked_shl(attempts.min(31)).unwrap_or(u32::MAX);
    initial.saturating_mul(multiplier).min(max)
}

/// Overflow-safe `now + delay`.
///
/// If `now + delay` would overflow [`Instant`], the delay is repeatedly halved
/// until it fits, so a caller can always obtain a valid (if nearer) deadline
/// instead of panicking. This is the defensive construction the replay
/// scheduler already used; routing assignment-resume through it too closes the
/// divergence where assignment-resume previously used a plain `now + delay`.
#[must_use]
pub(crate) fn checked_deadline(now: Instant, mut delay: Duration) -> Instant {
    loop {
        if let Some(deadline) = now.checked_add(delay) {
            return deadline;
        }
        delay /= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Key = (String, i32);

    fn key(topic: &str, partition: i32) -> Key {
        (topic.to_string(), partition)
    }

    /// Scenario: several keys are scheduled at different deadlines and drained
    /// once every deadline has elapsed.
    /// Guarantees: take_due returns elapsed entries in ascending deadline order
    /// and removes them, so the shared index preserves the earliest-first
    /// draining both retry subsystems rely on.
    #[test]
    fn take_due_returns_entries_in_deadline_order() {
        let now = Instant::now();
        let mut index: DeadlineIndex<Key, u64> = DeadlineIndex::new();
        index.insert(key("t", 2), 1, now + Duration::from_millis(30));
        index.insert(key("t", 0), 1, now + Duration::from_millis(10));
        index.insert(key("t", 1), 1, now + Duration::from_millis(20));

        assert_eq!(index.next_deadline(), Some(now + Duration::from_millis(10)));

        let due = index.take_due(now + Duration::from_millis(100), 10);
        let partitions: Vec<i32> = due.iter().map(|d| d.key.1).collect();
        assert_eq!(partitions, vec![0, 1, 2]);
        assert!(index.is_empty());
    }

    /// Scenario: more keys are due than the caller's per-turn limit allows.
    /// Guarantees: take_due returns at most `limit` entries and leaves the rest
    /// scheduled, bounding the work one receive-loop turn performs.
    #[test]
    fn take_due_respects_limit_and_leaves_remainder() {
        let now = Instant::now();
        let mut index: DeadlineIndex<Key, u64> = DeadlineIndex::new();
        for partition in 0..5 {
            index.insert(
                key("t", partition),
                1,
                now + Duration::from_millis(partition as u64),
            );
        }
        let due = index.take_due(now + Duration::from_secs(1), 2);
        assert_eq!(due.len(), 2);
        assert_eq!(index.len(), 3);
    }

    /// Scenario: an entry whose deadline has not yet elapsed is queried.
    /// Guarantees: take_due returns nothing while the earliest deadline is in
    /// the future, so not-yet-due work is never surfaced early.
    #[test]
    fn take_due_skips_future_deadlines() {
        let now = Instant::now();
        let mut index: DeadlineIndex<Key, u64> = DeadlineIndex::new();
        index.insert(key("t", 0), 1, now + Duration::from_secs(60));
        let due = index.take_due(now, 10);
        assert!(due.is_empty());
        assert_eq!(index.len(), 1);
    }

    /// Scenario: a key is rescheduled under a newer token, then the stale entry
    /// for the old token is removed.
    /// Guarantees: remove only deletes the exact `(key, token, deadline)` entry,
    /// so removing a superseded schedule cannot drop a newer one for the same
    /// key -- the staleness guard both subsystems depend on.
    #[test]
    fn remove_targets_exact_token_and_deadline() {
        let now = Instant::now();
        let mut index: DeadlineIndex<Key, u64> = DeadlineIndex::new();
        let old_deadline = now + Duration::from_millis(10);
        let new_deadline = now + Duration::from_millis(50);
        index.insert(key("t", 0), 1, old_deadline);
        index.insert(key("t", 0), 2, new_deadline);

        assert!(index.remove(&key("t", 0), 1, old_deadline));
        // The newer entry survives and is now the earliest.
        assert_eq!(index.next_deadline(), Some(new_deadline));
        // Removing an already-removed entry is a no-op.
        assert!(!index.remove(&key("t", 0), 1, old_deadline));
    }

    /// Scenario: backoff is requested for increasing attempt counts and for a
    /// very large attempt count.
    /// Guarantees: the delay doubles per attempt, is clamped to `max`, and
    /// saturates rather than overflowing for large attempt counts.
    #[test]
    fn capped_exponential_backoff_doubles_then_clamps() {
        let initial = Duration::from_millis(100);
        let max = Duration::from_secs(5);
        assert_eq!(capped_exponential_backoff(initial, max, 0), initial);
        assert_eq!(
            capped_exponential_backoff(initial, max, 1),
            Duration::from_millis(200)
        );
        assert_eq!(
            capped_exponential_backoff(initial, max, 2),
            Duration::from_millis(400)
        );
        // Large exponents clamp to max without overflowing.
        assert_eq!(capped_exponential_backoff(initial, max, 1_000), max);
    }

    /// Scenario: a deadline is computed with a normal delay and with a delay
    /// large enough that `now + delay` would overflow `Instant`.
    /// Guarantees: checked_deadline applies a normal delay exactly and, for a
    /// pathological delay, returns a valid `Instant` (halving until it fits)
    /// instead of panicking.
    #[test]
    fn checked_deadline_is_overflow_safe() {
        let now = Instant::now();
        assert_eq!(
            checked_deadline(now, Duration::from_millis(250)),
            now + Duration::from_millis(250)
        );
        let deadline = checked_deadline(now, Duration::MAX);
        assert!(deadline >= now);
    }
}
