// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared (`Send`) control-aware channel prototype.

use crate::core::Inner;
use crate::{ConfigError, ControlChannelConfig, ControlChannelStats, ControlCmd, ControlEvent};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

struct SharedState<PData> {
    inner: Mutex<Inner<PData>>,
    notify: Notify,
    sender_count: AtomicUsize,
}

/// Shared sender for the experimental control-aware channel.
pub struct SharedControlSender<PData> {
    state: Arc<SharedState<PData>>,
}

/// Shared receiver for the experimental control-aware channel.
pub struct SharedControlReceiver<PData> {
    state: Arc<SharedState<PData>>,
}

impl<PData> Clone for SharedControlSender<PData> {
    fn clone(&self) -> Self {
        let _previous = self.state.sender_count.fetch_add(1, Ordering::AcqRel);
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<PData> Drop for SharedControlSender<PData> {
    fn drop(&mut self) {
        if self.state.sender_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            let mut inner = self
                .state
                .inner
                .lock()
                .expect("shared control channel state mutex poisoned");
            if inner.close() {
                self.state.notify.notify_one();
            }
        }
    }
}

impl<PData> SharedControlSender<PData> {
    /// Sends a control command into the channel.
    pub fn send(&self, cmd: ControlCmd<PData>) -> Result<crate::SendOutcome, crate::SendError> {
        let result = self
            .state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .send(cmd);
        if matches!(
            result,
            Ok(crate::SendOutcome::Accepted | crate::SendOutcome::Replaced)
        ) {
            self.state.notify.notify_one();
        }
        result
    }

    /// Closes the channel for new sends.
    pub fn close(&self) {
        let closed = self
            .state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .close();
        if closed {
            self.state.notify.notify_one();
        }
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .stats()
    }
}

impl<PData> SharedControlReceiver<PData> {
    /// Receives the next available control event, or `None` if the channel is
    /// closed and fully drained.
    pub async fn recv(&mut self) -> Option<ControlEvent<PData>> {
        loop {
            let notified = self.state.notify.notified();
            let version = {
                let mut inner = self
                    .state
                    .inner
                    .lock()
                    .expect("shared control channel state mutex poisoned");
                if let Some(event) = inner.pop_event() {
                    return Some(event);
                }
                if inner.closed {
                    return None;
                }
                inner.version
            };

            let current_version = self
                .state
                .inner
                .lock()
                .expect("shared control channel state mutex poisoned")
                .version;
            if current_version != version {
                continue;
            }

            notified.await;
        }
    }

    /// Attempts to receive one control event without waiting.
    pub fn try_recv(&mut self) -> Option<ControlEvent<PData>> {
        self.state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .pop_event()
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .stats()
    }
}

/// Creates a new shared control-aware channel pair.
pub fn channel<PData>(
    config: ControlChannelConfig,
) -> Result<(SharedControlSender<PData>, SharedControlReceiver<PData>), ConfigError> {
    config.validate()?;
    let state = Arc::new(SharedState {
        inner: Mutex::new(Inner::new(config)),
        notify: Notify::new(),
        sender_count: AtomicUsize::new(1),
    });

    Ok((
        SharedControlSender {
            state: Arc::clone(&state),
        },
        SharedControlReceiver { state },
    ))
}
