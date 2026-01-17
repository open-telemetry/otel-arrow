// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! State stores

use serde::ser::{SerializeSeq, Serializer};
use std::collections::VecDeque;

pub mod conditions;
pub mod error;
pub mod phase;
mod pipeline_rt_status;
pub mod pipeline_status;
pub mod reporter;
pub mod store;

use otap_df_telemetry::event::ObservedEvent;
use serde::Serialize;

/// A ring buffer for storing recent observed events.
///
/// When the buffer reaches capacity, the oldest event is dropped to make room
/// for new events. Events are serialized in reverse order (newest first).
#[derive(Debug, Clone)]
pub struct ObservedEventRingBuffer {
    buf: VecDeque<ObservedEvent>,
    cap: usize,
}

impl Serialize for ObservedEventRingBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.buf.len()))?;
        for ev in self.buf.iter().rev() {
            // <— reverse iteration
            seq.serialize_element(ev)?;
        }
        seq.end()
    }
}

impl ObservedEventRingBuffer {
    /// Create a new ring buffer with the given capacity.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(cap),
            cap,
        }
    }

    /// Push an event into the ring buffer, dropping the oldest if full.
    pub fn push(&mut self, event: ObservedEvent) {
        if self.buf.len() == self.cap {
            _ = self.buf.pop_front(); // drop oldest
        }
        self.buf.push_back(event);
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}
