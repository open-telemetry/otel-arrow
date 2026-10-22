// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{FileLocator, Observation, ReconcileError, ReconcileLimits, TrackedId};
use std::collections::{BinaryHeap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Context for the identity owner's start policy. It never transfers predecessor progress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Replacement {
    /// No retained predecessor was associated with this observation.
    Unrelated,
    /// A retained predecessor's path now names this candidate. Identity resolution
    /// must prefer the candidate's own durable state; only a new identity starts at zero.
    Possible(TrackedId),
}

/// Bounded fresh candidate evidence, with no descriptors or durable identity.
#[derive(Debug)]
pub struct Candidate {
    /// Complete native path; minimum observed eligible alias in this pass.
    pub path: PathBuf,
    /// Runtime locator, never proof of durable continuity.
    pub locator: FileLocator,
    /// Handle-derived size at observation.
    pub size: u64,
    /// Handle-derived modification time, when available.
    pub modified: Option<SystemTime>,
    /// Validated fingerprint prefix.
    pub fingerprint: Vec<u8>,
    /// Original retained discovery time; overflow candidates have no historical age.
    pub first_seen: Duration,
    /// Replacement context retained independently of chosen alias.
    pub replacement: Replacement,
}

/// Identity-owner decision for one current candidate offer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityFeedback {
    /// The candidate was admitted or reconnected successfully.
    Admitted,
    /// Retry later without losing the original retained discovery time.
    Deferred,
    /// Identity resolution proved a new durable slot is needed. While full,
    /// reserve future offers for unresolved or reconnectable candidates.
    NeedsCapacity,
    /// Discard this candidate without recording an admission. A later fresh
    /// observation may reconsider it with a new discovery time.
    Rejected,
}

/// A one-pass, one-offer acknowledgement token. Fields are intentionally private.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OfferToken {
    generation: u64,
    slot: usize,
}

/// Borrowed offer to identity resolution. This is not an admission decision.
#[derive(Debug)]
pub struct CandidateOffer<'a> {
    /// Correlation for exactly one acknowledgement before the next pass starts.
    pub token: OfferToken,
    /// Current-pass stable candidate evidence.
    pub candidate: &'a Candidate,
}

struct Slot {
    candidate: Candidate,
    occupied: bool,
    seen: bool,
    conflict: bool,
    order: u64,
    rank: u64,
    acknowledged: bool,
    needs_capacity: bool,
}

pub(super) struct Candidates {
    slots: Vec<Slot>,
    index: HashMap<FileLocator, usize>,
    free: Vec<usize>,
    sample: BinaryHeap<(u64, usize)>,
    order: Vec<usize>,
    selected: Vec<usize>,
    pending: usize,
    offers: usize,
    generation: u64,
    seed: u64,
    sequence: u64,
    overflow_turn: bool,
}

impl Candidates {
    pub(super) fn new(limits: ReconcileLimits) -> Self {
        let capacity = limits.pending + limits.offers;
        Self {
            slots: (0..capacity)
                .map(|_| Slot {
                    candidate: Candidate {
                        path: PathBuf::new(),
                        locator: FileLocator {
                            device: 0,
                            inode: 0,
                        },
                        size: 0,
                        modified: None,
                        fingerprint: Vec::new(),
                        first_seen: Duration::ZERO,
                        replacement: Replacement::Unrelated,
                    },
                    occupied: false,
                    seen: false,
                    conflict: false,
                    order: 0,
                    rank: 0,
                    acknowledged: false,
                    needs_capacity: false,
                })
                .collect(),
            // Keep maximum live occupancy below half capacity. Overflow replacement
            // deletes create tombstones; this headroom lets hashbrown rehash those
            // in place instead of growing a logically bounded table during scans.
            index: HashMap::with_capacity(capacity * 2),
            free: (0..limits.pending).rev().collect(),
            sample: BinaryHeap::with_capacity(limits.offers),
            order: Vec::with_capacity(capacity),
            selected: Vec::with_capacity(limits.offers),
            pending: limits.pending,
            offers: limits.offers,
            generation: 0,
            seed: 0,
            sequence: 0,
            overflow_turn: false,
        }
    }

    pub(super) fn begin(&mut self, generation: u64, seed: u64) {
        for slot in &mut self.slots[self.pending..] {
            if slot.occupied {
                let _ = self.index.remove(&slot.candidate.locator);
            }
            slot.occupied = false;
        }
        for slot in &mut self.slots {
            slot.seen = false;
            slot.conflict = false;
            slot.acknowledged = false;
        }
        self.sample.clear();
        self.selected.clear();
        self.order.clear();
        self.generation = generation;
        self.seed = seed ^ generation.rotate_left(23);
    }

    pub(super) fn observe(
        &mut self,
        observation: &Observation,
        now: Duration,
        replacement: Replacement,
    ) -> Result<bool, ReconcileError> {
        let locator = observation.locator.expect("validated candidate");
        if let Some(&index) = self.index.get(&locator) {
            let slot = &mut self.slots[index];
            let first_in_pass = !slot.seen;
            if slot.seen {
                // Appends may extend the prefix/size. Shrink or changed bytes are
                // inconsistent evidence; no offer may silently select one version.
                if observation.size.expect("candidate size") < slot.candidate.size
                    || !observation
                        .fingerprint
                        .starts_with(&slot.candidate.fingerprint)
                {
                    slot.conflict = true;
                }
            }
            refresh(slot, observation, first_in_pass);
            if replacement != Replacement::Unrelated {
                slot.candidate.replacement = replacement;
            }
            return Ok(false);
        }
        let next_sequence = self
            .sequence
            .checked_add(1)
            .ok_or(ReconcileError::Exhausted)?;
        let rank = if self.free.is_empty() {
            rank(self.seed, locator)
        } else {
            0
        };
        let overflow = self.free.is_empty();
        let index = if let Some(index) = self.free.pop() {
            index
        } else if self.sample.len() < self.offers {
            let index = self.pending + self.sample.len();
            self.sample.push((rank, index));
            index
        } else {
            let &(worst, index) = self.sample.peek().expect("nonzero sample bound");
            if rank >= worst {
                return Ok(true);
            }
            let _ = self.index.remove(&self.slots[index].candidate.locator);
            *self.sample.peek_mut().expect("nonempty sample") = (rank, index);
            index
        };
        self.sequence = next_sequence;
        let slot = &mut self.slots[index];
        slot.occupied = true;
        slot.conflict = false;
        slot.acknowledged = false;
        slot.needs_capacity = false;
        slot.order = self.sequence;
        slot.rank = rank;
        slot.candidate.locator = locator;
        slot.candidate.first_seen = now;
        slot.candidate.replacement = replacement;
        refresh(slot, observation, true);
        let _ = self.index.insert(locator, index);
        Ok(overflow)
    }

    pub(super) fn finish(
        &mut self,
        complete: bool,
        capacity_available: bool,
        known: impl Fn(FileLocator) -> bool,
    ) {
        if complete {
            for index in 0..self.pending {
                if self.slots[index].occupied && !self.slots[index].seen {
                    self.remove(index);
                }
            }
        }
        self.order.clear();
        self.order.extend((0..self.pending).filter(|&index| {
            let slot = &self.slots[index];
            slot.occupied
                && slot.seen
                && !slot.conflict
                && (!slot.needs_capacity || capacity_available || known(slot.candidate.locator))
        }));
        self.order.sort_unstable_by_key(|&index| {
            (
                self.slots[index].candidate.first_seen,
                self.slots[index].order,
            )
        });
        let retained = self.order.len();
        self.order
            .extend((self.pending..self.slots.len()).filter(|&index| {
                let slot = &self.slots[index];
                slot.occupied && slot.seen && !slot.conflict
            }));
        self.order[retained..].sort_unstable_by_key(|&index| (self.slots[index].rank, index));
        let overflow = self.order.len() - retained;
        let reserved = if overflow == 0 {
            0
        } else if retained == 0 {
            self.offers
        } else if self.offers == 1 {
            self.overflow_turn = !self.overflow_turn;
            usize::from(self.overflow_turn)
        } else {
            (self.offers / 2).max(1)
        };
        let retained_take = retained.min(self.offers - reserved);
        self.selected
            .extend(self.order[..retained_take].iter().copied());
        let overflow_take = overflow.min(self.offers - retained_take);
        self.selected.extend(
            self.order[retained..retained + overflow_take]
                .iter()
                .copied(),
        );
        let extra = (self.offers - self.selected.len()).min(retained - retained_take);
        self.selected.extend(
            self.order[retained_take..retained_take + extra]
                .iter()
                .copied(),
        );
    }

    pub(super) fn offers(&self) -> impl Iterator<Item = CandidateOffer<'_>> {
        self.selected
            .iter()
            .filter(|&&index| self.slots[index].occupied && !self.slots[index].acknowledged)
            .map(|&index| CandidateOffer {
                token: OfferToken {
                    generation: self.generation,
                    slot: index,
                },
                candidate: &self.slots[index].candidate,
            })
    }

    pub(super) fn acknowledge(
        &mut self,
        token: OfferToken,
        feedback: IdentityFeedback,
    ) -> Result<(), ReconcileError> {
        if token.generation != self.generation
            || !self.selected.contains(&token.slot)
            || !self.slots[token.slot].occupied
            || self.slots[token.slot].acknowledged
        {
            return Err(ReconcileError::Feedback);
        }
        self.slots[token.slot].acknowledged = true;
        self.slots[token.slot].needs_capacity = feedback == IdentityFeedback::NeedsCapacity;
        if matches!(
            feedback,
            IdentityFeedback::Admitted | IdentityFeedback::Rejected
        ) {
            self.remove(token.slot);
        }
        Ok(())
    }

    fn remove(&mut self, index: usize) {
        let slot = &mut self.slots[index];
        let _ = self.index.remove(&slot.candidate.locator);
        slot.occupied = false;
        if index < self.pending {
            self.free.push(index);
        }
    }

    pub(super) fn associate(
        &mut self,
        locator: FileLocator,
        predecessor: TrackedId,
    ) -> Result<(), ReconcileError> {
        let index = *self.index.get(&locator).ok_or(ReconcileError::Feedback)?;
        let slot = &mut self.slots[index];
        if !slot.seen || slot.conflict {
            return Err(ReconcileError::Feedback);
        }
        slot.candidate.replacement = Replacement::Possible(predecessor);
        Ok(())
    }

    pub(super) fn block_replacements(&mut self, conflicts: impl Fn(TrackedId) -> bool) {
        for slot in &mut self.slots {
            if slot.occupied
                && let Replacement::Possible(id) = slot.candidate.replacement
                && conflicts(id)
            {
                slot.conflict = true;
            }
        }
    }

    pub(super) fn has_conflict(&self) -> bool {
        self.slots
            .iter()
            .any(|slot| slot.occupied && slot.seen && slot.conflict)
    }

    pub(super) fn pending_len(&self) -> usize {
        self.pending - self.free.len()
    }

    pub(super) fn oldest_age(&self, now: Duration) -> Duration {
        self.slots[..self.pending]
            .iter()
            .filter(|slot| slot.occupied)
            .map(|slot| now.saturating_sub(slot.candidate.first_seen))
            .max()
            .unwrap_or_default()
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.slots.capacity() * size_of::<Slot>()
            + self
                .slots
                .iter()
                .map(|slot| slot.candidate.path.capacity() + slot.candidate.fingerprint.capacity())
                .sum::<usize>()
            + super::table_bytes::<FileLocator, usize>(self.slots.len() * 2, false)
            + (self.free.capacity() + self.order.capacity() + self.selected.capacity())
                * size_of::<usize>()
            + self.sample.capacity() * size_of::<(u64, usize)>()
    }
}

fn refresh(slot: &mut Slot, observation: &Observation, replace_path: bool) {
    if replace_path || observation.path.as_os_str() < slot.candidate.path.as_os_str() {
        slot.candidate.path.clone_from(&observation.path);
    }
    slot.candidate.size = observation.size.expect("validated size");
    slot.candidate.modified = observation.modified;
    slot.candidate
        .fingerprint
        .clone_from(&observation.fingerprint);
    slot.seen = true;
}

fn rank(seed: u64, locator: FileLocator) -> u64 {
    // SplitMix64 finalization is a scheduling permutation, not an identity or
    // security hash. Wrapping multiplication is intentional mixing arithmetic.
    fn mix(mut value: u64) -> u64 {
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
    mix(seed ^ mix(locator.device) ^ locator.inode.rotate_left(29))
}
