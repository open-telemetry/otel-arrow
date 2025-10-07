// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Slot-based correlation system for tracking in-flight gRPC requests
//! and their responses.
//!
//! We expect this to apply to both OTLP and OTAP receivers. The State
//! struct below

use tokio::sync::oneshot;

/// Placeholder for NackMsg until the control message PR is merged.
/// The actual NackMsg will include reason, permanent flag, and calldata.
/// This could be replaced `type Nack = NackMsg<OtapPdata>;`
/// TODO(#498): Replace with actual NackMsg.
#[allow(dead_code)]
pub struct Nack {
    reason: String,
    refused: Box<String>,
}

/// Configuration for the slot-based correlation server
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of concurrent in-flight requests allowed
    pub max_slots: usize,
}

/// Index into the slots array
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotIndex(usize);

/// Generation number to prevent ABA problem when slots are reused
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotGeneration(usize);

/// Combined slot identifier used for correlation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotKey {
    index: SlotIndex,
    generation: SlotGeneration,
}

impl SlotIndex {
    /// Convert to usize for array indexing
    #[must_use]
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl SlotGeneration {
    /// Convert to usize for serialization
    #[must_use]
    pub fn as_usize(self) -> usize {
        self.0
    }

    /// Increment generation number (with wrapping)
    #[must_use]
    pub fn increment(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

impl SlotKey {
    /// Create a new SlotKey from index and generation
    #[must_use]
    pub fn new(index: SlotIndex, generation: SlotGeneration) -> Self {
        Self { index, generation }
    }

    /// Get the slot index
    #[must_use]
    pub fn index(self) -> SlotIndex {
        self.index
    }

    /// Get the generation number
    #[must_use]
    pub fn generation(self) -> SlotGeneration {
        self.generation
    }
}

/// Data stored in an active slot
struct SlotData {
    /// Channel to send the response back to the gRPC handler
    channel: oneshot::Sender<Result<(), Nack>>,
    /// Current generation number for this slot
    generation: SlotGeneration,
}

/// GenMem represents a slot that is either in use or available.
/// When available, it remembers the next generation number.
enum GenMem {
    /// Slot is currently in use with active request
    Current(SlotData),
    /// Slot is available, storing the next generation to use
    Available(SlotGeneration),
}

/// State managing the slot array and free list.
pub struct State {
    /// Array of slots, can grow up to max_slots (does not shrink)
    slots: Vec<GenMem>,
    /// Indices of available slots for quick allocation
    free_slots: Vec<SlotIndex>,
    /// Configuration
    config: Config,
}

impl State {
    /// Create new server state with the given configuration
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            slots: Vec::new(),
            free_slots: Vec::new(),
            config,
        }
    }

    /// Allocate a slot for a new request.
    pub fn allocate_slot(&mut self, channel: oneshot::Sender<Result<(), Nack>>) -> Option<SlotKey> {
        // Try to reuse a free slot first
        if let Some(slot_index) = self.free_slots.pop() {
            let slot_ref = &mut self.slots[slot_index.as_usize()];

            if let GenMem::Available(generation) = slot_ref {
                let current_gen = *generation;
                *slot_ref = GenMem::Current(SlotData {
                    channel,
                    generation: current_gen,
                });
                return Some(SlotKey::new(slot_index, current_gen));
            } else {
                // This should not happen - free_slots should only contain Available slots
                return None;
            }
        }

        // No free slots available, try to grow the array
        if self.slots.len() < self.config.max_slots {
            let slot_index = SlotIndex(self.slots.len());
            let generation = SlotGeneration(1); // Start at generation 1

            self.slots.push(GenMem::Current(SlotData {
                channel,
                generation,
            }));

            Some(SlotKey::new(slot_index, generation))
        } else {
            // At capacity
            None
        }
    }

    fn extract_current_if_valid(&mut self, slot_key: SlotKey) -> Option<SlotData> {
        let slot_index = slot_key.index().as_usize();
        if slot_index >= self.slots.len() {
            return None;
        }

        let slot_ref = &mut self.slots[slot_index];

        match slot_ref {
            GenMem::Available(_) => None,
            GenMem::Current(data) => {
                // Verify generation matches
                if slot_key.generation() != data.generation {
                    return None;
                }

                // Swap with Available(next)
                let next_generation = data.generation.increment();
                let mut replacement = GenMem::Available(next_generation);
                std::mem::swap(slot_ref, &mut replacement);

                match replacement {
                    GenMem::Current(slot_data) => Some(slot_data),
                    _ => unreachable!("is GenMem::Current above"),
                }
            }
        }
    }

    /// Free a slot when a request is cancelled (e.g., due to timeout).
    pub fn free_slot(&mut self, slot_key: SlotKey) {
        if self.extract_current_if_valid(slot_key).is_some() {
            self.free_slots.push(slot_key.index());
        }
    }

    /// Deliver a response to the specified slot.
    pub fn deliver_response(&mut self, slot_key: SlotKey, result: Result<(), Nack>) {
        if let Some(slot_data) = self.extract_current_if_valid(slot_key) {
            // Ignore send errors (receiver may have been dropped)
            let _ = slot_data.channel.send(result);
            self.free_slots.push(slot_key.index());
        }
    }

    /// Get the number of currently allocated slots
    #[must_use]
    pub fn allocated_count(&self) -> usize {
        self.slots.len() - self.free_slots.len()
    }

    /// Get the total number of slots (allocated + free)
    #[must_use]
    pub fn total_slots(&self) -> usize {
        self.slots.len()
    }

    /// Get the maximum number of slots allowed
    #[must_use]
    pub fn max_slots(&self) -> usize {
        self.config.max_slots
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> State {
        State::new(Config { max_slots: 3 })
    }

    #[test]
    fn test_allocate() {
        let mut state = create_test_state();

        let (tx1, _) = oneshot::channel();
        let key1 = state.allocate_slot(tx1).unwrap();

        let (tx2, _) = oneshot::channel();
        let key2 = state.allocate_slot(tx2).unwrap();

        let (tx3, _) = oneshot::channel();
        let key3 = state.allocate_slot(tx3).unwrap();

        assert_eq!(key1.index().as_usize(), 0);
        assert_eq!(key2.index().as_usize(), 1);
        assert_eq!(key3.index().as_usize(), 2);
        assert_eq!(state.allocated_count(), 3);
        assert_eq!(state.total_slots(), 3);

        let (tx4, _) = oneshot::channel();
        let result = state.allocate_slot(tx4);

        assert!(result.is_none(), "beyond capacity");
        assert_eq!(state.allocated_count(), 3);

        state.free_slot(key1);
        assert_eq!(state.allocated_count(), 2);
        assert_eq!(state.total_slots(), 3);

        state.free_slot(key2);
        state.free_slot(key3);
        assert_eq!(state.allocated_count(), 0);
        assert_eq!(state.total_slots(), 3);
        assert_eq!(state.free_slots.len(), 3);
    }

    #[test]
    fn test_reuse() {
        let mut state = create_test_state();

        let (tx1, _rx1) = oneshot::channel();
        let key1 = state.allocate_slot(tx1).unwrap();
        assert_eq!(key1.generation().as_usize(), 1);
        state.free_slot(key1);

        let (tx2, _rx2) = oneshot::channel();
        let key2 = state.allocate_slot(tx2).unwrap();

        assert_eq!(key2.index(), key1.index());
        assert_eq!(key2.generation().as_usize(), 2,);
        assert_eq!(state.total_slots(), 1);
    }

    #[test]
    fn test_deliver_success() {
        let mut state = create_test_state();
        let (tx, rx) = oneshot::channel();

        let key = state.allocate_slot(tx).unwrap();
        assert_eq!(state.allocated_count(), 1);

        state.deliver_response(key, Ok(()));

        let result = rx.blocking_recv().unwrap();
        assert!(result.is_ok());
        assert_eq!(state.allocated_count(), 0);
        assert_eq!(state.total_slots(), 1);
    }

    #[test]
    fn test_deliver_failure() {
        let mut state = create_test_state();
        let (tx, rx) = oneshot::channel();

        let key = state.allocate_slot(tx).unwrap();
        let nack = Nack {
            reason: "Test failure".into(),
            refused: Box::new("hello".into()),
        };
        state.deliver_response(key, Err(nack));

        match rx.blocking_recv().unwrap() {
            Ok(_) => {
                panic!("incorrect success");
            }
            Err(nack) => {
                assert_eq!(nack.reason, "Test failure");
            }
        }
    }

    #[test]
    fn test_deliver_old_generation() {
        let mut state = create_test_state();
        let (tx, rx) = oneshot::channel();

        let key = state.allocate_slot(tx).unwrap();
        state.free_slot(key);
        assert!(rx.blocking_recv().is_err());

        let (tx2, _) = oneshot::channel();
        let _key2 = state.allocate_slot(tx2).unwrap();

        state.deliver_response(key, Ok(()));

        assert_eq!(state.allocated_count(), 1);
        assert_eq!(state.total_slots(), 1);
    }

    #[test]
    fn test_free_twice() {
        let mut state = create_test_state();
        let (tx, _rx) = oneshot::channel();

        let key = state.allocate_slot(tx).unwrap();
        assert_eq!(state.allocated_count(), 1);

        state.free_slot(key);
        assert_eq!(state.allocated_count(), 0);

        state.free_slot(key);
        assert_eq!(state.allocated_count(), 0);
    }

    #[test]
    fn test_allocs_and_deallocs() {
        let mut state = create_test_state();

        let (tx1, _) = oneshot::channel();
        let (tx2, _) = oneshot::channel();
        let (tx3, _) = oneshot::channel();

        let key1 = state.allocate_slot(tx1).unwrap();
        let key2 = state.allocate_slot(tx2).unwrap();
        let key3 = state.allocate_slot(tx3).unwrap();

        state.free_slot(key2);
        assert_eq!(state.allocated_count(), 2);
        assert_eq!(key2.generation().as_usize(), 1);

        let (tx4, _) = oneshot::channel();
        let key4 = state.allocate_slot(tx4).unwrap();

        assert_eq!(key4.index(), key2.index());
        assert_eq!(key4.generation().as_usize(), 2);

        state.free_slot(key1);
        state.free_slot(key3);
        state.free_slot(key4);

        assert_eq!(state.allocated_count(), 0);
        assert_eq!(state.total_slots(), 3);

        let (tx5, _) = oneshot::channel();
        let key5 = state.allocate_slot(tx5).unwrap();

        assert_eq!(key5.index(), key2.index());
        assert_eq!(key5.generation().as_usize(), 3);

        assert_eq!(state.allocated_count(), 1);
        assert_eq!(state.total_slots(), 3);
    }
}
