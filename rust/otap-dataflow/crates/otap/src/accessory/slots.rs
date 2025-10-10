// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Slot-based correlation system for correlating in-flight requests
//! and responses. Provides a CallData to retrieve the data for Ack/Nack handling.

use otap_df_engine::control::CallData;
use otap_df_engine::error::Error;

/// Configuration for the slot-based correlation server
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of concurrent in-flight requests allowed
    pub max_slots: usize,
}

/// Index into the slots array
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotIndex(usize);

impl From<usize> for SlotIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<SlotIndex> for usize {
    fn from(value: SlotIndex) -> usize {
        value.0
    }
}

/// Generation number to prevent ABA problem when slots are reused
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotGeneration(usize);

impl From<usize> for SlotGeneration {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<SlotGeneration> for usize {
    fn from(value: SlotGeneration) -> usize {
        value.0
    }
}

impl SlotGeneration {
    /// Increment generation number (with wrapping)
    #[must_use]
    fn increment(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

/// Combined slot identifier used for correlation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotKey {
    index: SlotIndex,
    generation: SlotGeneration,
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

impl From<SlotKey> for CallData {
    fn from(value: SlotKey) -> Self {
        smallvec::smallvec![value.index().0.into(), value.generation().0.into()]
    }
}

impl TryFrom<CallData> for SlotKey {
    type Error = Error;

    fn try_from(value: CallData) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(Error::InternalError {
                message: "invalid calldata format".into(),
            });
        }

        let slot_index: usize = value[0].try_into()?;
        let slot_generation: usize = value[1].try_into()?;

        Ok(SlotKey::new(slot_index.into(), slot_generation.into()))
    }
}

/// Data stored in an active slot
/// Generic over user-provided data (UData)
struct SlotData<UData> {
    /// User-provided data (e.g., oneshot::Sender or streaming response info)
    user: UData,
    /// Current generation number for this slot (internal)
    generation: SlotGeneration,
}

/// GenMem represents a slot that is either in use or available.
/// When available, it remembers the next generation number.
enum GenMem<UData> {
    /// Slot is currently in use with active request
    Current(SlotData<UData>),
    /// Slot is available, storing the next generation to use
    Available(SlotGeneration),
}

/// State managing the slot array and free list.
/// Generic over user data type `UData`.
pub struct State<UData> {
    /// Array of slots, can grow up to max_slots (does not shrink)
    slots: Vec<GenMem<UData>>,
    /// Indices of available slots for quick allocation
    free_slots: Vec<SlotIndex>,
    /// Configuration
    config: Config,
}

impl<UData> State<UData> {
    /// Create new server state with the given configuration
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            slots: Vec::new(),
            free_slots: Vec::new(),
            config,
        }
    }

    /// Allocate a slot for a new request with user-provided data.
    #[must_use]
    pub fn allocate_slot(&mut self, user_data: UData) -> Option<SlotKey> {
        // Try to reuse a free slot first
        if let Some(slot_index) = self.free_slots.pop() {
            let idx: usize = slot_index.into();
            let slot_ref = &mut self.slots[idx];

            if let GenMem::Available(generation) = slot_ref {
                let current_gen = *generation;
                *slot_ref = GenMem::Current(SlotData {
                    user: user_data,
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
                user: user_data,
                generation,
            }));

            Some(SlotKey::new(slot_index, generation))
        } else {
            // At capacity
            None
        }
    }

    /// Get user data from a slot if generation matches
    #[must_use]
    pub fn get_if_current(&mut self, slot_key: SlotKey) -> Option<UData> {
        let idx: usize = slot_key.index().into();
        if idx >= self.slots.len() {
            return None;
        }

        let slot_ref = &mut self.slots[idx];

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
                self.free_slots.push(slot_key.index());

                match replacement {
                    GenMem::Current(slot_data) => Some(slot_data.user),
                    _ => unreachable!("is GenMem::Current above"),
                }
            }
        }
    }

    /// Get and drop the user data, if current.
    pub fn cancel(&mut self, slot_key: SlotKey) {
        let _ = self.get_if_current(slot_key);
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
    use tokio::sync::oneshot;

    type TestUData = oneshot::Sender<Result<(), String>>;

    fn create_test_state() -> State<TestUData> {
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

        assert_eq!(key1.index().0, 0);
        assert_eq!(key2.index().0, 1);
        assert_eq!(key3.index().0, 2);
        assert_eq!(state.allocated_count(), 3);
        assert_eq!(state.total_slots(), 3);

        let (tx4, _) = oneshot::channel();
        let result = state.allocate_slot(tx4);

        assert!(result.is_none(), "beyond capacity");
        assert_eq!(state.allocated_count(), 3);

        state.cancel(key1);
        assert_eq!(state.allocated_count(), 2);
        assert_eq!(state.total_slots(), 3);

        state.cancel(key2);
        state.cancel(key3);
        assert_eq!(state.allocated_count(), 0);
        assert_eq!(state.total_slots(), 3);
        assert_eq!(state.free_slots.len(), 3);
    }

    #[test]
    fn test_reuse() {
        let mut state = create_test_state();

        let (tx1, _rx1) = oneshot::channel();
        let key1 = state.allocate_slot(tx1).unwrap();
        assert_eq!(key1.generation().0, 1);
        state.cancel(key1);

        let (tx2, _rx2) = oneshot::channel();
        let key2 = state.allocate_slot(tx2).unwrap();

        assert_eq!(key2.index(), key1.index());
        assert_eq!(key2.generation().0, 2,);
        assert_eq!(state.total_slots(), 1);
    }

    #[test]
    fn test_get_current() {
        let mut state = create_test_state();
        let (tx, rx) = oneshot::channel();

        let key = state.allocate_slot(tx).unwrap();
        assert_eq!(state.allocated_count(), 1);

        state
            .get_if_current(key)
            .map(|channel| channel.send(Ok(())))
            .expect("sent")
            .expect("ok");

        let result = rx.blocking_recv().unwrap();
        assert!(result.is_ok());
        assert_eq!(state.allocated_count(), 0);
        assert_eq!(state.total_slots(), 1);
    }

    #[test]
    fn test_get_old() {
        let mut state = create_test_state();
        let (tx, rx) = oneshot::channel();

        let key = state.allocate_slot(tx).unwrap();
        state.cancel(key);
        assert!(rx.blocking_recv().is_err());

        let (tx2, _) = oneshot::channel();
        let _key2 = state.allocate_slot(tx2).unwrap();

        // Try to get old generation
        assert!(state.get_if_current(key).is_none());

        assert_eq!(state.allocated_count(), 1);
        assert_eq!(state.total_slots(), 1);
    }

    #[test]
    fn test_cancel_twice() {
        let mut state = create_test_state();
        let (tx, _rx) = oneshot::channel();

        let key = state.allocate_slot(tx).unwrap();
        assert_eq!(state.allocated_count(), 1);

        state.cancel(key);
        assert_eq!(state.allocated_count(), 0);

        state.cancel(key);
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

        state.cancel(key2);
        assert_eq!(state.allocated_count(), 2);
        assert_eq!(key2.generation().0, 1);

        let (tx4, _) = oneshot::channel();
        let key4 = state.allocate_slot(tx4).unwrap();

        assert_eq!(key4.index(), key2.index());
        assert_eq!(key4.generation().0, 2);

        state.cancel(key1);
        state.cancel(key3);
        state.cancel(key4);

        assert_eq!(state.allocated_count(), 0);
        assert_eq!(state.total_slots(), 3);

        let (tx5, _) = oneshot::channel();
        let key5 = state.allocate_slot(tx5).unwrap();

        assert_eq!(key5.index(), key2.index());
        assert_eq!(key5.generation().0, 3);

        assert_eq!(state.allocated_count(), 1);
        assert_eq!(state.total_slots(), 3);
    }
}
