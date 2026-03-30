// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Local (`!Send`) standalone control-aware channel.

use crate::core::Inner;
use crate::types::CoreControlEvent;
use crate::{
    ConfigError, ControlChannelConfig, ControlChannelStats, ControlCmd, DrainIngressMsg,
    LifecycleSendResult, NodeControlEvent, ReceiverControlEvent, ShutdownMsg,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tokio::sync::Notify;
use tokio::time::{Instant as TokioInstant, sleep_until};

struct LocalState<PData> {
    inner: RefCell<Inner<PData>>,
    notify: Notify,
    sender_count: Cell<usize>,
}

struct LocalControlSenderCore<PData> {
    state: Rc<LocalState<PData>>,
}

struct LocalControlReceiverCore<PData> {
    state: Rc<LocalState<PData>>,
}

/// Local sender for receiver nodes, including `DrainIngress`.
pub struct LocalReceiverControlSender<PData> {
    inner: LocalControlSenderCore<PData>,
}

/// Local sender for non-receiver nodes.
pub struct LocalNodeControlSender<PData> {
    inner: LocalControlSenderCore<PData>,
}

/// Local receiver for receiver-role control events.
pub struct LocalReceiverControlReceiver<PData> {
    inner: LocalControlReceiverCore<PData>,
}

/// Local receiver for non-receiver node control events.
pub struct LocalNodeControlReceiver<PData> {
    inner: LocalControlReceiverCore<PData>,
}

impl<PData> Clone for LocalControlSenderCore<PData> {
    fn clone(&self) -> Self {
        self.state
            .sender_count
            .set(self.state.sender_count.get().saturating_add(1));
        Self {
            state: Rc::clone(&self.state),
        }
    }
}

impl<PData> Drop for LocalControlSenderCore<PData> {
    fn drop(&mut self) {
        let next = self.state.sender_count.get().saturating_sub(1);
        self.state.sender_count.set(next);
        if next == 0 {
            let closed = self.state.inner.borrow_mut().close();
            if closed {
                self.state.notify.notify_waiters();
            }
        }
    }
}

impl<PData> LocalControlSenderCore<PData> {
    fn accept_drain_ingress(&self, msg: DrainIngressMsg) -> LifecycleSendResult {
        let result = self.state.inner.borrow_mut().record_drain_ingress(msg);
        if matches!(result, LifecycleSendResult::Accepted) {
            self.state.notify.notify_waiters();
        }
        result
    }

    fn accept_shutdown(&self, msg: ShutdownMsg) -> LifecycleSendResult {
        let result = self.state.inner.borrow_mut().record_shutdown(msg);
        if matches!(result, LifecycleSendResult::Accepted) {
            self.state.notify.notify_waiters();
        }
        result
    }

    fn try_send(
        &self,
        cmd: ControlCmd<PData>,
    ) -> Result<crate::SendOutcome, crate::TrySendError<PData>> {
        let result = self.state.inner.borrow_mut().try_send(cmd);
        if matches!(
            result,
            Ok(crate::SendOutcome::Accepted | crate::SendOutcome::Replaced)
        ) {
            self.state.notify.notify_one();
        }
        result
    }

    async fn send(
        &self,
        cmd: ControlCmd<PData>,
    ) -> Result<crate::SendOutcome, crate::SendError<PData>> {
        let mut pending_cmd = Some(cmd);
        loop {
            let full_version = {
                let mut inner = self.state.inner.borrow_mut();
                let cmd = pending_cmd
                    .take()
                    .expect("pending control command must exist while sending");
                match inner.try_send(cmd) {
                    Ok(outcome) => {
                        if matches!(
                            outcome,
                            crate::SendOutcome::Accepted | crate::SendOutcome::Replaced
                        ) {
                            self.state.notify.notify_one();
                        }
                        return Ok(outcome);
                    }
                    Err(crate::TrySendError::Closed(cmd)) => {
                        return Err(crate::SendError::Closed(cmd));
                    }
                    Err(crate::TrySendError::Full { cmd, .. }) => {
                        pending_cmd = Some(cmd);
                        inner.version
                    }
                }
            };

            // Register interest before checking queue state, then compare the
            // observed generation after dropping the borrow. If another task
            // made progress in between, retry immediately instead of sleeping
            // on a stale notification boundary. Shutdown deadlines are also
            // observed here so blocked senders cannot wait forever once the
            // channel has entered forced terminal progress.
            let notified = self.state.notify.notified();
            let deadline = {
                let inner = self.state.inner.borrow();
                if inner.version != full_version {
                    continue;
                }
                inner.next_deadline()
            };

            if let Some(deadline) = deadline {
                tokio::select! {
                    _ = notified => {}
                    _ = sleep_until(TokioInstant::from_std(deadline)) => {}
                }
            } else {
                notified.await;
            }
        }
    }

    fn close(&self) {
        let closed = self.state.inner.borrow_mut().close();
        if closed {
            self.state.notify.notify_waiters();
        }
    }

    fn stats(&self) -> ControlChannelStats {
        self.state.inner.borrow().stats()
    }
}

impl<PData> Clone for LocalReceiverControlSender<PData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<PData> Clone for LocalNodeControlSender<PData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<PData> LocalReceiverControlSender<PData> {
    /// Accepts a receiver-drain lifecycle token without consuming bounded
    /// control capacity.
    #[must_use]
    pub fn accept_drain_ingress(&self, msg: DrainIngressMsg) -> LifecycleSendResult {
        self.inner.accept_drain_ingress(msg)
    }

    /// Accepts a shutdown lifecycle token without consuming bounded control
    /// capacity.
    #[must_use]
    pub fn accept_shutdown(&self, msg: ShutdownMsg) -> LifecycleSendResult {
        self.inner.accept_shutdown(msg)
    }

    /// Attempts to send a non-lifecycle control command without waiting for capacity.
    pub fn try_send(
        &self,
        cmd: ControlCmd<PData>,
    ) -> Result<crate::SendOutcome, crate::TrySendError<PData>> {
        self.inner.try_send(cmd)
    }

    /// Sends a non-lifecycle control command, waiting asynchronously for bounded
    /// capacity when needed.
    pub async fn send(
        &self,
        cmd: ControlCmd<PData>,
    ) -> Result<crate::SendOutcome, crate::SendError<PData>> {
        self.inner.send(cmd).await
    }

    /// Closes the channel for new sends.
    pub fn close(&self) {
        self.inner.close();
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.inner.stats()
    }
}

impl<PData> LocalNodeControlSender<PData> {
    /// Accepts a shutdown lifecycle token without consuming bounded control
    /// capacity.
    #[must_use]
    pub fn accept_shutdown(&self, msg: ShutdownMsg) -> LifecycleSendResult {
        self.inner.accept_shutdown(msg)
    }

    /// Attempts to send a non-lifecycle control command without waiting for capacity.
    pub fn try_send(
        &self,
        cmd: ControlCmd<PData>,
    ) -> Result<crate::SendOutcome, crate::TrySendError<PData>> {
        self.inner.try_send(cmd)
    }

    /// Sends a non-lifecycle control command, waiting asynchronously for bounded
    /// capacity when needed.
    pub async fn send(
        &self,
        cmd: ControlCmd<PData>,
    ) -> Result<crate::SendOutcome, crate::SendError<PData>> {
        self.inner.send(cmd).await
    }

    /// Closes the channel for new sends.
    pub fn close(&self) {
        self.inner.close();
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.inner.stats()
    }
}

impl<PData> LocalControlReceiverCore<PData> {
    fn notify_after_pop(&self, event: &CoreControlEvent<PData>) {
        match event {
            CoreControlEvent::CompletionBatch(_) => {
                if self.state.sender_count.get() <= 1 {
                    self.state.notify.notify_one();
                } else {
                    self.state.notify.notify_waiters();
                }
            }
            CoreControlEvent::Shutdown(_) => self.state.notify.notify_waiters(),
            CoreControlEvent::DrainIngress(_)
            | CoreControlEvent::Config { .. }
            | CoreControlEvent::TimerTick
            | CoreControlEvent::CollectTelemetry => {}
        }
    }

    async fn recv_internal(&mut self) -> Option<CoreControlEvent<PData>> {
        loop {
            {
                let mut inner = self.state.inner.borrow_mut();
                if let Some(event) = inner.pop_event() {
                    drop(inner);
                    self.notify_after_pop(&event);
                    return Some(event);
                }
                if inner.closed {
                    return None;
                }
            }

            // Same generation-check pattern as `send()`: avoid waiting if the
            // queue changed after we decided it was currently empty. Shutdown
            // deadlines are part of the wait condition so forced shutdown does
            // not depend on a producer arriving later to wake the receiver.
            let notified = self.state.notify.notified();
            let (version, deadline) = {
                let inner = self.state.inner.borrow();
                (inner.version, inner.next_deadline())
            };

            if self.state.inner.borrow().version != version {
                continue;
            }

            if let Some(deadline) = deadline {
                tokio::select! {
                    _ = notified => {}
                    _ = sleep_until(TokioInstant::from_std(deadline)) => {}
                }
            } else {
                notified.await;
            }
        }
    }

    fn try_recv_internal(&mut self) -> Option<CoreControlEvent<PData>> {
        let event = self.state.inner.borrow_mut().pop_event();
        if let Some(event_ref) = &event {
            self.notify_after_pop(event_ref);
        }
        event
    }

    fn stats(&self) -> ControlChannelStats {
        self.state.inner.borrow().stats()
    }
}

impl<PData> LocalReceiverControlReceiver<PData> {
    /// Receives the next available control event, or `None` if the channel is
    /// closed and fully drained.
    pub async fn recv(&mut self) -> Option<ReceiverControlEvent<PData>> {
        self.inner
            .recv_internal()
            .await
            .map(ReceiverControlEvent::<PData>::from_core)
    }

    /// Attempts to receive one control event without waiting.
    pub fn try_recv(&mut self) -> Option<ReceiverControlEvent<PData>> {
        self.inner
            .try_recv_internal()
            .map(ReceiverControlEvent::<PData>::from_core)
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.inner.stats()
    }
}

impl<PData> LocalNodeControlReceiver<PData> {
    /// Receives the next available control event, or `None` if the channel is
    /// closed and fully drained.
    pub async fn recv(&mut self) -> Option<NodeControlEvent<PData>> {
        self.inner
            .recv_internal()
            .await
            .map(NodeControlEvent::<PData>::from_core)
    }

    /// Attempts to receive one control event without waiting.
    pub fn try_recv(&mut self) -> Option<NodeControlEvent<PData>> {
        self.inner
            .try_recv_internal()
            .map(NodeControlEvent::<PData>::from_core)
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.inner.stats()
    }
}

fn channel_state<PData>(
    config: ControlChannelConfig,
) -> Result<
    (
        LocalControlSenderCore<PData>,
        LocalControlReceiverCore<PData>,
    ),
    ConfigError,
> {
    config.validate()?;
    let state = Rc::new(LocalState {
        inner: RefCell::new(Inner::new(config)),
        notify: Notify::new(),
        sender_count: Cell::new(1),
    });

    Ok((
        LocalControlSenderCore {
            state: Rc::clone(&state),
        },
        LocalControlReceiverCore { state },
    ))
}

/// Creates a new local control-aware channel pair for receiver nodes.
pub fn receiver_channel<PData>(
    config: ControlChannelConfig,
) -> Result<
    (
        LocalReceiverControlSender<PData>,
        LocalReceiverControlReceiver<PData>,
    ),
    ConfigError,
> {
    let (sender, receiver) = channel_state(config)?;
    Ok((
        LocalReceiverControlSender { inner: sender },
        LocalReceiverControlReceiver { inner: receiver },
    ))
}

/// Creates a new local control-aware channel pair for non-receiver nodes.
pub fn node_channel<PData>(
    config: ControlChannelConfig,
) -> Result<
    (
        LocalNodeControlSender<PData>,
        LocalNodeControlReceiver<PData>,
    ),
    ConfigError,
> {
    let (sender, receiver) = channel_state(config)?;
    Ok((
        LocalNodeControlSender { inner: sender },
        LocalNodeControlReceiver { inner: receiver },
    ))
}
