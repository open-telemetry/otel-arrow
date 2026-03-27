// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Local (`!Send`) control-aware channel prototype.

use crate::core::Inner;
use crate::{ConfigError, ControlChannelConfig, ControlChannelStats, ControlCmd, ControlEvent};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tokio::sync::Notify;

struct LocalState<PData> {
    inner: RefCell<Inner<PData>>,
    notify: Notify,
    sender_count: Cell<usize>,
}

/// Local sender for the experimental control-aware channel.
pub struct LocalControlSender<PData> {
    state: Rc<LocalState<PData>>,
}

/// Local receiver for the experimental control-aware channel.
pub struct LocalControlReceiver<PData> {
    state: Rc<LocalState<PData>>,
}

impl<PData> Clone for LocalControlSender<PData> {
    fn clone(&self) -> Self {
        self.state
            .sender_count
            .set(self.state.sender_count.get().saturating_add(1));
        Self {
            state: Rc::clone(&self.state),
        }
    }
}

impl<PData> Drop for LocalControlSender<PData> {
    fn drop(&mut self) {
        let next = self.state.sender_count.get().saturating_sub(1);
        self.state.sender_count.set(next);
        if next == 0 {
            let closed = self.state.inner.borrow_mut().close();
            if closed {
                self.state.notify.notify_one();
            }
        }
    }
}

impl<PData> LocalControlSender<PData> {
    /// Sends a control command into the channel.
    pub fn send(&self, cmd: ControlCmd<PData>) -> Result<crate::SendOutcome, crate::SendError> {
        let result = self.state.inner.borrow_mut().send(cmd);
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
        let closed = self.state.inner.borrow_mut().close();
        if closed {
            self.state.notify.notify_one();
        }
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.state.inner.borrow().stats()
    }
}

impl<PData> LocalControlReceiver<PData> {
    /// Receives the next available control event, or `None` if the channel is
    /// closed and fully drained.
    pub async fn recv(&mut self) -> Option<ControlEvent<PData>> {
        loop {
            let notified = self.state.notify.notified();
            let version = {
                let mut inner = self.state.inner.borrow_mut();
                if let Some(event) = inner.pop_event() {
                    return Some(event);
                }
                if inner.closed {
                    return None;
                }
                inner.version
            };

            if self.state.inner.borrow().version != version {
                continue;
            }

            notified.await;
        }
    }

    /// Attempts to receive one control event without waiting.
    pub fn try_recv(&mut self) -> Option<ControlEvent<PData>> {
        self.state.inner.borrow_mut().pop_event()
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.state.inner.borrow().stats()
    }
}

/// Creates a new local control-aware channel pair.
pub fn channel<PData>(
    config: ControlChannelConfig,
) -> Result<(LocalControlSender<PData>, LocalControlReceiver<PData>), ConfigError> {
    config.validate()?;
    let state = Rc::new(LocalState {
        inner: RefCell::new(Inner::new(config)),
        notify: Notify::new(),
        sender_count: Cell::new(1),
    });

    Ok((
        LocalControlSender {
            state: Rc::clone(&state),
        },
        LocalControlReceiver { state },
    ))
}
