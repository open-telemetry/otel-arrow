// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A generic indexed binary min-heap with O(1) key lookup and O(log n)
//! insert, remove, and in-place priority update.
//!
//! The heap stores `(K, P)` pairs where `K` is a unique key and `P` is a
//! priority value.  A companion `HashMap<K, usize>` tracks each key's
//! current position in the underlying `Vec`, enabling efficient keyed
//! operations that a standard `BinaryHeap` cannot provide.

use std::collections::HashMap;
use std::hash::Hash;

/// Outcome of an [`IndexedMinHeap::insert`] call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InsertOutcome {
    /// A new entry was added to the heap.
    Inserted,
    /// An existing entry's priority was replaced.
    Replaced,
}

/// A binary min-heap that supports O(1) key lookup and O(log n) keyed
/// insertion, removal, and in-place priority updates.
///
/// Entries are `(K, P)` pairs.  The heap is ordered by `P` where the
/// *smallest* priority sits at the root.  Keys must be unique — inserting
/// a key that already exists replaces its priority in place.
///
/// # Key cloning
///
/// Keys are cloned during insert, swap, pop, and removal operations.
/// Callers should prefer cheap-to-clone key types (e.g. integers).
//  Using an expensive-to-clone key will add overhead proportional to
//  tree depth on every heap operation.
pub(crate) struct IndexedMinHeap<K, P> {
    entries: Vec<Entry<K, P>>,
    indices: HashMap<K, usize>,
}

#[derive(Clone, Debug)]
struct Entry<K, P> {
    key: K,
    priority: P,
}

impl<K, P> IndexedMinHeap<K, P>
where
    K: Eq + Hash + Clone + std::fmt::Debug,
    P: Ord,
{
    /// Creates an empty heap.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            indices: HashMap::new(),
        }
    }

    /// Returns the number of entries in the heap.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the heap contains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a reference to the `(key, priority)` pair with the smallest
    /// priority, or `None` if the heap is empty.
    #[must_use]
    pub fn peek(&self) -> Option<(&K, &P)> {
        self.entries.first().map(|e| (&e.key, &e.priority))
    }

    /// Returns `true` if the heap contains an entry for `key`.
    pub fn contains_key(&self, key: &K) -> bool {
        self.indices.contains_key(key)
    }

    /// Inserts `key` with the given `priority`.
    ///
    /// If `key` already exists its priority is replaced in place and the
    /// heap property is restored.  Returns whether the key was newly inserted
    /// or replaced.
    pub fn insert(&mut self, key: K, priority: P) -> InsertOutcome {
        if let Some(&index) = self.indices.get(&key) {
            self.entries[index].priority = priority;
            self.repair_at(index);
            InsertOutcome::Replaced
        } else {
            let index = self.entries.len();
            self.entries.push(Entry {
                key: key.clone(),
                priority,
            });
            assert!(
                self.indices.insert(key, index).is_none(),
                "new key should not already exist in index map"
            );
            self.sift_up(index);
            InsertOutcome::Inserted
        }
    }

    /// Removes the entry with the smallest priority and returns its
    /// `(key, priority)` pair, or `None` if the heap is empty.
    pub fn pop(&mut self) -> Option<(K, P)> {
        if self.entries.is_empty() {
            return None;
        }
        let root_key = self.entries[0].key.clone();
        let removed = self
            .indices
            .remove(&root_key)
            .expect("root key should exist in index map");
        debug_assert_eq!(removed, 0);
        let entry = self.remove_at(0);
        Some((entry.key, entry.priority))
    }

    /// Removes the entry for `key` and returns its `(key, priority)` pair,
    /// or `None` if the key is not present.
    pub fn remove(&mut self, key: &K) -> Option<(K, P)> {
        let index = self.indices.remove(key)?;
        let entry = self.remove_at(index);
        debug_assert_eq!(&entry.key, key);
        Some((entry.key, entry.priority))
    }

    /// Removes all entries from the heap.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.indices.clear();
    }

    // -- internal heap machinery ------------------------------------------

    fn swap(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        self.entries.swap(a, b);
        let key_a = self.entries[a].key.clone();
        let key_b = self.entries[b].key.clone();
        let _ = self
            .indices
            .insert(key_a, a)
            .expect("swapped key a should exist in index map");
        let _ = self
            .indices
            .insert(key_b, b)
            .expect("swapped key b should exist in index map");
    }

    fn sift_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if self.entries[index].priority >= self.entries[parent].priority {
                break;
            }
            self.swap(index, parent);
            index = parent;
        }
    }

    fn sift_down(&mut self, mut index: usize) {
        let len = self.entries.len();
        loop {
            let left = index * 2 + 1;
            if left >= len {
                break;
            }
            let right = left + 1;
            let mut smallest = left;
            if right < len && self.entries[right].priority < self.entries[left].priority {
                smallest = right;
            }
            if self.entries[smallest].priority >= self.entries[index].priority {
                break;
            }
            self.swap(index, smallest);
            index = smallest;
        }
    }

    fn repair_at(&mut self, index: usize) {
        if index > 0 {
            let parent = (index - 1) / 2;
            if self.entries[index].priority < self.entries[parent].priority {
                self.sift_up(index);
                return;
            }
        }
        self.sift_down(index);
    }

    /// Removes the entry at `index`, restoring the heap property.
    ///
    /// The caller must have already removed the key from `self.indices`.
    fn remove_at(&mut self, index: usize) -> Entry<K, P> {
        let last = self
            .entries
            .len()
            .checked_sub(1)
            .expect("remove_at requires a non-empty heap");

        if index == last {
            return self.entries.pop().expect("last entry should exist");
        }

        self.entries.swap(index, last);
        let removed = self.entries.pop().expect("removed entry should exist");

        // Update the index of the entry that was moved into `index`.
        let moved_key = self.entries[index].key.clone();
        let _ = self
            .indices
            .insert(moved_key, index)
            .expect("moved key should exist in index map");
        self.repair_at(index);
        removed
    }

    /// Asserts that the heap invariant and the index map are consistent.
    ///
    /// This is intended for use in tests and debug builds.
    #[cfg(debug_assertions)]
    pub fn assert_consistent(&self) {
        assert_eq!(self.entries.len(), self.indices.len());

        for (i, entry) in self.entries.iter().enumerate() {
            assert_eq!(
                self.indices.get(&entry.key).copied(),
                Some(i),
                "heap index must match map entry"
            );
            if i > 0 {
                let parent = (i - 1) / 2;
                assert!(
                    self.entries[i].priority >= self.entries[parent].priority,
                    "heap child must not have smaller priority than parent"
                );
            }
        }
    }
}

impl<K, P> Default for IndexedMinHeap<K, P>
where
    K: Eq + Hash + Clone + std::fmt::Debug,
    P: Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_heap() {
        let heap: IndexedMinHeap<u32, u32> = IndexedMinHeap::new();
        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.peek(), None);
    }

    #[test]
    fn insert_and_peek() {
        let mut heap = IndexedMinHeap::new();
        assert_eq!(heap.insert(1u32, 10u32), InsertOutcome::Inserted);
        assert_eq!(heap.peek(), Some((&1, &10)));
        assert_eq!(heap.len(), 1);
        #[cfg(debug_assertions)]
        heap.assert_consistent();
    }

    #[test]
    fn insert_maintains_min_order() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 30u32);
        let _ = heap.insert(2, 10);
        let _ = heap.insert(3, 20);
        #[cfg(debug_assertions)]
        heap.assert_consistent();

        assert_eq!(heap.peek(), Some((&2, &10)));
    }

    #[test]
    fn pop_returns_entries_in_priority_order() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 30u32);
        let _ = heap.insert(2, 10);
        let _ = heap.insert(3, 20);

        assert_eq!(heap.pop(), Some((2, 10)));
        #[cfg(debug_assertions)]
        heap.assert_consistent();
        assert_eq!(heap.pop(), Some((3, 20)));
        #[cfg(debug_assertions)]
        heap.assert_consistent();
        assert_eq!(heap.pop(), Some((1, 30)));
        #[cfg(debug_assertions)]
        heap.assert_consistent();
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn insert_replaces_existing_key() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 30u32);
        let _ = heap.insert(2, 10);
        assert_eq!(heap.insert(1, 5), InsertOutcome::Replaced);
        #[cfg(debug_assertions)]
        heap.assert_consistent();

        // Key 1 now has priority 5, should be the new root.
        assert_eq!(heap.peek(), Some((&1, &5)));
        assert_eq!(heap.len(), 2);
    }

    #[test]
    fn replace_priority_upward() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 10u32);
        let _ = heap.insert(2, 20);
        let _ = heap.insert(3, 30);

        // Make key 3 the highest priority.
        let _ = heap.insert(3, 1);
        #[cfg(debug_assertions)]
        heap.assert_consistent();
        assert_eq!(heap.peek(), Some((&3, &1)));
    }

    #[test]
    fn replace_priority_downward() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 10u32);
        let _ = heap.insert(2, 20);
        let _ = heap.insert(3, 30);

        // Make key 1 the lowest priority.
        let _ = heap.insert(1, 100);
        #[cfg(debug_assertions)]
        heap.assert_consistent();
        assert_eq!(heap.peek(), Some((&2, &20)));
    }

    #[test]
    fn remove_by_key() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 10u32);
        let _ = heap.insert(2, 20);
        let _ = heap.insert(3, 30);

        assert_eq!(heap.remove(&2), Some((2, 20)));
        #[cfg(debug_assertions)]
        heap.assert_consistent();
        assert_eq!(heap.len(), 2);
        assert!(!heap.contains_key(&2));

        // Remaining entries still ordered.
        assert_eq!(heap.pop(), Some((1, 10)));
        assert_eq!(heap.pop(), Some((3, 30)));
    }

    #[test]
    fn remove_nonexistent_key_returns_none() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 10u32);
        assert_eq!(heap.remove(&99), None);
        assert_eq!(heap.len(), 1);
    }

    #[test]
    fn remove_root() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 10u32);
        let _ = heap.insert(2, 20);
        let _ = heap.insert(3, 30);

        assert_eq!(heap.remove(&1), Some((1, 10)));
        #[cfg(debug_assertions)]
        heap.assert_consistent();
        assert_eq!(heap.peek(), Some((&2, &20)));
    }

    #[test]
    fn remove_last_entry() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 10u32);
        assert_eq!(heap.remove(&1), Some((1, 10)));
        assert!(heap.is_empty());
    }

    #[test]
    fn clear_empties_the_heap() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 10u32);
        let _ = heap.insert(2, 20);
        heap.clear();
        assert!(heap.is_empty());
        assert_eq!(heap.peek(), None);
    }

    #[test]
    fn contains_key_works() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 10u32);
        assert!(heap.contains_key(&1));
        assert!(!heap.contains_key(&2));
    }

    #[test]
    fn many_inserts_maintain_heap_order() {
        let mut heap = IndexedMinHeap::new();
        // Insert in reverse order.
        for i in (0u32..100).rev() {
            let _ = heap.insert(i, i);
        }
        #[cfg(debug_assertions)]
        heap.assert_consistent();

        for expected in 0u32..100 {
            assert_eq!(heap.pop(), Some((expected, expected)));
        }
        assert!(heap.is_empty());
    }

    #[test]
    fn equal_priorities_are_stable_by_insertion_order() {
        // With equal priorities, the heap only guarantees they all come out,
        // not a specific order among equals. Verify all are returned.
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 0u32);
        let _ = heap.insert(2, 0);
        let _ = heap.insert(3, 0);
        #[cfg(debug_assertions)]
        heap.assert_consistent();

        let mut keys = Vec::new();
        while let Some((k, _)) = heap.pop() {
            keys.push(k);
        }
        keys.sort();
        assert_eq!(keys, vec![1, 2, 3]);
    }

    #[test]
    fn remove_middle_entry_preserves_heap() {
        let mut heap = IndexedMinHeap::new();
        for i in 0u32..10 {
            let _ = heap.insert(i, i * 10);
        }

        // Remove entry in the middle of the heap.
        assert_eq!(heap.remove(&5), Some((5, 50)));
        #[cfg(debug_assertions)]
        heap.assert_consistent();

        let mut prev = 0u32;
        while let Some((_, p)) = heap.pop() {
            assert!(p >= prev);
            prev = p;
        }
    }

    #[test]
    fn repeated_replace_same_key() {
        let mut heap = IndexedMinHeap::new();
        let _ = heap.insert(1u32, 100u32);
        for p in (1u32..=50).rev() {
            assert_eq!(heap.insert(1, p), InsertOutcome::Replaced);
            #[cfg(debug_assertions)]
            heap.assert_consistent();
            assert_eq!(heap.len(), 1);
            assert_eq!(heap.peek(), Some((&1, &p)));
        }
        assert_eq!(heap.pop(), Some((1, 1)));
    }
}
