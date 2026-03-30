// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared (`Send`) standalone control-aware channel.

use crate::core::Inner;
use crate::types::CoreControlEvent;
use crate::{
    ConfigError, ControlChannelConfig, ControlChannelStats, ControlCmd, DrainIngressMsg,
    LifecycleSendResult, NodeControlEvent, ReceiverControlEvent, ShutdownMsg,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tokio::time::{Instant as TokioInstant, sleep_until};

struct SharedState<PData> {
    inner: Mutex<Inner<PData>>,
    notify: Notify,
    sender_count: AtomicUsize,
}

struct SharedControlSenderCore<PData> {
    state: Arc<SharedState<PData>>,
}

struct SharedControlReceiverCore<PData> {
    state: Arc<SharedState<PData>>,
}

/// Shared sender for receiver nodes, including `DrainIngress`.
pub struct SharedReceiverControlSender<PData> {
    inner: SharedControlSenderCore<PData>,
}

/// Shared sender for non-receiver nodes.
pub struct SharedNodeControlSender<PData> {
    inner: SharedControlSenderCore<PData>,
}

/// Shared receiver for receiver-role control events.
pub struct SharedReceiverControlReceiver<PData> {
    inner: SharedControlReceiverCore<PData>,
}

/// Shared receiver for non-receiver node control events.
pub struct SharedNodeControlReceiver<PData> {
    inner: SharedControlReceiverCore<PData>,
}

impl<PData> Clone for SharedControlSenderCore<PData> {
    fn clone(&self) -> Self {
        let _previous = self.state.sender_count.fetch_add(1, Ordering::AcqRel);
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<PData> Drop for SharedControlSenderCore<PData> {
    fn drop(&mut self) {
        if self.state.sender_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            let mut inner = self
                .state
                .inner
                .lock()
                .expect("shared control channel state mutex poisoned");
            if inner.close() {
                self.state.notify.notify_waiters();
            }
        }
    }
}

impl<PData> SharedControlSenderCore<PData> {
    fn accept_drain_ingress(&self, msg: DrainIngressMsg) -> LifecycleSendResult {
        let result = self
            .state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .record_drain_ingress(msg);
        if matches!(result, LifecycleSendResult::Accepted) {
            self.state.notify.notify_waiters();
        }
        result
    }

    fn accept_shutdown(&self, msg: ShutdownMsg) -> LifecycleSendResult {
        let result = self
            .state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .record_shutdown(msg);
        if matches!(result, LifecycleSendResult::Accepted) {
            self.state.notify.notify_waiters();
        }
        result
    }

    fn try_send(
        &self,
        cmd: ControlCmd<PData>,
    ) -> Result<crate::SendOutcome, crate::TrySendError<PData>> {
        let result = self
            .state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .try_send(cmd);
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
                let mut inner = self
                    .state
                    .inner
                    .lock()
                    .expect("shared control channel state mutex poisoned");
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
            // observed generation after releasing the mutex. If another task
            // changed the queue in between, retry immediately instead of
            // awaiting a notification that has effectively already happened.
            // Shutdown deadlines are also observed here so blocked senders do
            // not wait forever once terminal progress is forced.
            let notified = self.state.notify.notified();
            let deadline = {
                let inner = self
                    .state
                    .inner
                    .lock()
                    .expect("shared control channel state mutex poisoned");
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
        let closed = self
            .state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .close();
        if closed {
            self.state.notify.notify_waiters();
        }
    }

    fn stats(&self) -> ControlChannelStats {
        self.state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .stats()
    }
}

impl<PData> Clone for SharedReceiverControlSender<PData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<PData> Clone for SharedNodeControlSender<PData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<PData> SharedReceiverControlSender<PData> {
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

impl<PData> SharedNodeControlSender<PData> {
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

impl<PData> SharedControlReceiverCore<PData> {
    fn notify_after_pop(&self, event: &CoreControlEvent<PData>) {
        match event {
            CoreControlEvent::CompletionBatch(_) => {
                if self.state.sender_count.load(Ordering::Acquire) <= 1 {
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
                let mut inner = self
                    .state
                    .inner
                    .lock()
                    .expect("shared control channel state mutex poisoned");
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
                let inner = self
                    .state
                    .inner
                    .lock()
                    .expect("shared control channel state mutex poisoned");
                (inner.version, inner.next_deadline())
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
        let event = self
            .state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .pop_event();
        if let Some(event_ref) = &event {
            self.notify_after_pop(event_ref);
        }
        event
    }

    fn stats(&self) -> ControlChannelStats {
        self.state
            .inner
            .lock()
            .expect("shared control channel state mutex poisoned")
            .stats()
    }
}

impl<PData> SharedReceiverControlReceiver<PData> {
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

impl<PData> SharedNodeControlReceiver<PData> {
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
        SharedControlSenderCore<PData>,
        SharedControlReceiverCore<PData>,
    ),
    ConfigError,
> {
    config.validate()?;
    let state = Arc::new(SharedState {
        inner: Mutex::new(Inner::new(config)),
        notify: Notify::new(),
        sender_count: AtomicUsize::new(1),
    });

    Ok((
        SharedControlSenderCore {
            state: Arc::clone(&state),
        },
        SharedControlReceiverCore { state },
    ))
}

/// Creates a new shared control-aware channel pair for receiver nodes.
pub fn receiver_channel<PData>(
    config: ControlChannelConfig,
) -> Result<
    (
        SharedReceiverControlSender<PData>,
        SharedReceiverControlReceiver<PData>,
    ),
    ConfigError,
> {
    let (sender, receiver) = channel_state(config)?;
    Ok((
        SharedReceiverControlSender { inner: sender },
        SharedReceiverControlReceiver { inner: receiver },
    ))
}

/// Creates a new shared control-aware channel pair for non-receiver nodes.
pub fn node_channel<PData>(
    config: ControlChannelConfig,
) -> Result<
    (
        SharedNodeControlSender<PData>,
        SharedNodeControlReceiver<PData>,
    ),
    ConfigError,
> {
    let (sender, receiver) = channel_state(config)?;
    Ok((
        SharedNodeControlSender { inner: sender },
        SharedNodeControlReceiver { inner: receiver },
    ))
}
