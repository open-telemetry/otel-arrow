// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dense records and ID-only indexes; paths belong exclusively to the records.

use super::{Tracked, TrackedId};
use hashbrown::HashTable;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};

pub(super) struct Records {
    slots: Vec<Tracked>,
    ids: HashTable<usize>,
    hash: RandomState,
}

impl Records {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            slots: Vec::with_capacity(limit),
            // Live occupancy stays below half the reserved capacity, so deletion
            // tombstones rehash in place on the pinned hashbrown implementation.
            ids: HashTable::with_capacity(limit * 2),
            hash: RandomState::new(),
        }
    }

    fn slot(&self, id: &TrackedId) -> Option<usize> {
        self.ids
            .find(self.hash.hash_one(id), |slot| {
                self.slots[*slot].file.id == *id
            })
            .copied()
    }

    pub(super) fn get(&self, id: &TrackedId) -> Option<&Tracked> {
        self.slot(id).map(|slot| &self.slots[slot])
    }

    pub(super) fn get_mut(&mut self, id: &TrackedId) -> Option<&mut Tracked> {
        let slot = self.slot(id)?;
        Some(&mut self.slots[slot])
    }

    pub(super) fn contains_key(&self, id: &TrackedId) -> bool {
        self.slot(id).is_some()
    }
    pub(super) fn len(&self) -> usize {
        self.ids.len()
    }

    pub(super) fn insert(&mut self, id: TrackedId, tracked: Tracked) {
        debug_assert!(!self.contains_key(&id));
        debug_assert_eq!(id, tracked.file.id);
        let slot = self.slots.len();
        assert!(slot < self.slots.capacity(), "preflighted record capacity");
        self.slots.push(tracked);
        let slots = &self.slots;
        let hash = &self.hash;
        let _ = self.ids.insert_unique(hash.hash_one(id), slot, |slot| {
            hash.hash_one(slots[*slot].file.id)
        });
    }

    pub(super) fn remove(&mut self, id: &TrackedId) -> Option<Tracked> {
        let slots = &self.slots;
        let entry = self
            .ids
            .find_entry(self.hash.hash_one(id), |slot| slots[*slot].file.id == *id)
            .ok()?;
        let (slot, _) = entry.remove();
        let removed = self.slots.swap_remove(slot);
        if slot < self.slots.len() {
            // Secondary indexes store IDs, not slot positions. Only the moved
            // record's primary ID entry changes when the packed vector shrinks.
            let moved = self.slots[slot].file.id;
            let old_last = self.slots.len();
            *self
                .ids
                .find_mut(self.hash.hash_one(moved), |position| *position == old_last)
                .expect("moved record ID") = slot;
        }
        Some(removed)
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &Tracked> {
        self.slots.iter()
    }
    pub(super) fn values_mut(&mut self) -> impl Iterator<Item = &mut Tracked> {
        self.slots.iter_mut()
    }
    pub(super) fn keys(&self) -> impl Iterator<Item = &TrackedId> {
        self.values().map(|t| &t.file.id)
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.slots.capacity() * size_of::<Tracked>() + self.ids.allocation_size()
    }
}

impl std::ops::Index<&TrackedId> for Records {
    type Output = Tracked;
    fn index(&self, id: &TrackedId) -> &Tracked {
        self.get(id).expect("indexed record")
    }
}

pub(super) struct RecordIndex<K: ?Sized> {
    ids: HashTable<TrackedId>,
    hash: RandomState,
    key: fn(&Tracked) -> Option<&K>,
}

impl<K: Hash + Eq + ?Sized> RecordIndex<K> {
    pub(super) fn new(limit: usize, key: fn(&Tracked) -> Option<&K>) -> Self {
        Self {
            ids: HashTable::with_capacity(limit * 2),
            hash: RandomState::new(),
            key,
        }
    }

    pub(super) fn clear(&mut self) {
        self.ids.clear();
    }

    pub(super) fn matches(&self, tracked: &Tracked, key: &K) -> bool {
        (self.key)(tracked).is_some_and(|value| value == key)
    }

    // Hash matches are only candidates: callers must compare native keys before
    // mutation. This split permits mutation of record state without path copies.
    pub(super) fn candidates(&self, key: &K) -> impl Iterator<Item = TrackedId> + '_ {
        self.ids.iter_hash(self.hash.hash_one(key)).copied()
    }

    pub(super) fn get<'a>(
        &'a self,
        key: &'a K,
        records: &'a Records,
    ) -> impl Iterator<Item = TrackedId> + 'a {
        self.records(key, records).map(|tracked| tracked.file.id)
    }

    // Return the resolved record to read-only callers: matching the key has
    // already looked up the ID, so a second primary-index lookup is unnecessary.
    pub(super) fn records<'a>(
        &'a self,
        key: &'a K,
        records: &'a Records,
    ) -> impl Iterator<Item = &'a Tracked> + 'a {
        self.candidates(key)
            .map(|id| &records[&id])
            .filter(|tracked| self.matches(tracked, key))
    }

    pub(super) fn insert(&mut self, id: TrackedId, records: &Records) {
        let Some(key) = (self.key)(&records[&id]) else {
            return;
        };
        let hash = &self.hash;
        let extract = self.key;
        let _ = self.ids.insert_unique(hash.hash_one(key), id, |id| {
            hash.hash_one(extract(&records[id]).expect("indexed key"))
        });
    }

    // Remove before changing the path or freeing its record slot. Historical
    // records can share a path; remove by both hash and unique runtime ID.
    pub(super) fn remove(&mut self, id: TrackedId, records: &Records) {
        let Some(key) = (self.key)(&records[&id]) else {
            return;
        };
        let entry = self
            .ids
            .find_entry(self.hash.hash_one(key), |candidate| *candidate == id)
            .expect("registered secondary index");
        let _ = entry.remove();
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.ids.allocation_size()
    }
}
