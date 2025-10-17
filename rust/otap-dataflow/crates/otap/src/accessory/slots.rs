// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Slot-based correlation system for correlating in-flight requests
//! and responses. Provides a CallData to retrieve the data for
//! Ack/Nack handling. Based on the slotmap crate.

use otap_df_engine::control::CallData;
use otap_df_engine::error::Error;
use slotmap::{Key as SlotKey, KeyData, SlotMap, new_key_type};

new_key_type! {
    /// Slot identifier used for calldata.
    pub struct Key;
}

impl From<Key> for CallData {
    fn from(key: Key) -> Self {
        smallvec::smallvec![key.data().as_ffi().into()]
    }
}

impl TryFrom<CallData> for Key {
    type Error = Error;

    fn try_from(value: CallData) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err(Error::InternalError {
                message: "invalid calldata format".into(),
            });
        }

        Ok(KeyData::from_ffi(value[0].into()).into())
    }
}

/// State managing the slot array and free list.
/// Generic over user data type `UData`.
pub struct State<UData> {
    /// Implemented by slotmap.
    slots: SlotMap<Key, UData>,

    /// Maximum size configuration.
    max_size: usize,
}

impl<UData> State<UData> {
    /// Create new server state with the given configuration
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            slots: SlotMap::with_key(),
            max_size,
        }
    }

    /// Allocate a slot for a new request.  The closure is called only
    /// if a slot is available, creating the user data. This also
    /// returns additional user data (e.g., the receiver end of a
    /// channel).
    ///
    /// Conveniently, oneshot::channel() returns a pair such that
    /// allocate_slot(|| oneshot::channel()) stores tx as UData and
    /// returns Some((key, rx)).
    #[must_use]
    pub fn allocate<F, R>(&mut self, create: F) -> Option<(Key, R)>
    where
        F: FnOnce() -> (UData, R),
    {
        if self.slots.len() >= self.max_size {
            return None;
        }

        let (udata, ures) = create();

        let key = self.slots.insert(udata);

        Some((key, ures))
    }

    /// Take user data from a slot (if key is valid).
    #[must_use]
    pub fn take(&mut self, key: Key) -> Option<UData> {
        self.slots.remove(key)
    }

    /// Take and drop the user data (if key is valid).
    pub fn cancel(&mut self, key: Key) {
        let _ = self.take(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    type TestUData = oneshot::Sender<Result<(), String>>;

    fn create_test_state() -> State<TestUData> {
        State::new(3)
    }

    #[test]
    fn test_allocate() {
        let mut state = create_test_state();

        let (key1, _) = state.allocate(|| oneshot::channel()).unwrap();
        let (key2, _) = state.allocate(|| oneshot::channel()).unwrap();
        let (key3, _) = state.allocate(|| oneshot::channel()).unwrap();

        assert_eq!(state.slots.len(), 3);
        assert_eq!(state.slots.capacity(), 3);

        let result = state.allocate(|| oneshot::channel());

        assert!(result.is_none(), "beyond capacity");
        assert_eq!(state.slots.len(), 3);

        state.cancel(key1);
        assert_eq!(state.slots.len(), 2);
        assert_eq!(state.slots.capacity(), 3);

        state.cancel(key2);
        state.cancel(key3);
        assert_eq!(state.slots.len(), 0);
        assert_eq!(state.slots.capacity(), 3);
    }

    #[test]
    fn test_take_current() {
        let mut state = create_test_state();

        assert_eq!(state.slots.capacity(), 0);

        let (key, rx) = state.allocate(|| oneshot::channel()).unwrap();
        assert_eq!(state.slots.len(), 1);

        state
            .take(key)
            .map(|channel| channel.send(Ok(())))
            .expect("sent")
            .expect("ok");

        let result = rx.blocking_recv().unwrap();
        assert!(result.is_ok());
        assert_eq!(state.slots.len(), 0);
    }

    #[test]
    fn test_take_old() {
        let mut state = create_test_state();

        let (key, rx) = state.allocate(|| oneshot::channel()).unwrap();
        state.cancel(key);
        assert!(rx.blocking_recv().is_err());

        let (_key2, _) = state.allocate(|| oneshot::channel()).unwrap();

        // Try to take old generation
        assert!(state.take(key).is_none());

        assert_eq!(state.slots.len(), 1);
    }

    #[test]
    fn test_cancel_twice() {
        let mut state = create_test_state();

        let (key, _) = state.allocate(|| oneshot::channel()).unwrap();
        assert_eq!(state.slots.len(), 1);

        state.cancel(key);
        assert_eq!(state.slots.len(), 0);

        state.cancel(key);
        assert_eq!(state.slots.len(), 0);
    }

    #[test]
    fn test_allocs_and_deallocs() {
        let mut state = create_test_state();

        let (key1, _) = state.allocate(|| oneshot::channel()).unwrap();
        let (key2, _) = state.allocate(|| oneshot::channel()).unwrap();
        let (key3, _) = state.allocate(|| oneshot::channel()).unwrap();

        state.cancel(key2);
        assert_eq!(state.slots.len(), 2);

        let (key4, _) = state.allocate(|| oneshot::channel()).unwrap();

        state.cancel(key1);
        state.cancel(key3);
        state.cancel(key4);

        assert_eq!(state.slots.len(), 0);
        assert_eq!(state.slots.capacity(), 3);

        let (key5, _) = state.allocate(|| oneshot::channel()).unwrap();

        assert_eq!(state.slots.len(), 1);
        assert_eq!(state.slots.capacity(), 3);

        state.cancel(key5);

        assert_eq!(state.slots.len(), 0);
    }
}
