// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Single-owner standalone control-aware channel.

use crate::core::Inner;
use crate::types::CoreControlEvent;
use crate::{
    ConfigError, ControlChannelConfig, ControlChannelStats, ControlCmd, DrainIngressMsg,
    LifecycleSendResult, NodeControlEvent, ReceiverControlEvent, SendError, SendOutcome,
    ShutdownMsg,
};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use tokio::sync::Notify;
use tokio::time::{Instant as TokioInstant, Sleep, sleep_until};

#[derive(Clone, Copy, PartialEq, Eq)]
struct SenderWaiterKey {
    // Stable key into the waiter slot array; generation prevents ABA when a
    // slot index is reused after cancellation/completion.
    index: usize,
    generation: u64,
}

struct SenderWaiterSlot {
    generation: u64,
    waker: Option<Waker>,
    in_use: bool,
    queued: bool,
}

impl SenderWaiterSlot {
    const fn vacant() -> Self {
        Self {
            generation: 0,
            waker: None,
            in_use: false,
            queued: false,
        }
    }
}

struct SenderWaiters {
    // FIFO queue of waiter keys used to preserve blocked-sender wake order.
    queue: VecDeque<SenderWaiterKey>,
    // Slot storage allows O(1) refresh/unregister by key without scanning the queue.
    slots: Vec<SenderWaiterSlot>,
    // Reuse released slots to avoid per-contention allocations.
    free_slots: Vec<usize>,
    next_generation: u64,
}

impl SenderWaiters {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            slots: Vec::new(),
            free_slots: Vec::new(),
            next_generation: 0,
        }
    }

    fn wake_n(&mut self, mut count: usize) {
        while count > 0 {
            let Some(key) = self.queue.pop_front() else {
                break;
            };
            let Some(slot) = self.slots.get_mut(key.index) else {
                continue;
            };
            // Stale queue entries are expected when futures are canceled or
            // re-queued; skip until we find a live queued waiter.
            if !slot.in_use || slot.generation != key.generation || !slot.queued {
                continue;
            }
            slot.queued = false;
            if let Some(waker) = slot.waker.take() {
                waker.wake();
                count -= 1;
            }
        }
    }

    fn wake_all(&mut self) {
        while let Some(key) = self.queue.pop_front() {
            let Some(slot) = self.slots.get_mut(key.index) else {
                continue;
            };
            if !slot.in_use || slot.generation != key.generation || !slot.queued {
                continue;
            }
            slot.queued = false;
            if let Some(waker) = slot.waker.take() {
                waker.wake();
            }
        }
    }

    fn register_or_refresh(&mut self, waiter_key: &mut Option<SenderWaiterKey>, waker: &Waker) {
        if let Some(existing_key) = *waiter_key {
            if let Some(slot) = self.slots.get_mut(existing_key.index) {
                if slot.in_use && slot.generation == existing_key.generation {
                    if slot
                        .waker
                        .as_ref()
                        .is_none_or(|existing| !existing.will_wake(waker))
                    {
                        slot.waker = Some(waker.clone());
                    }
                    if !slot.queued {
                        slot.queued = true;
                        self.queue.push_back(existing_key);
                    }
                    return;
                }
            }
        }

        let index = if let Some(index) = self.free_slots.pop() {
            index
        } else {
            self.slots.push(SenderWaiterSlot::vacant());
            self.slots.len() - 1
        };

        let generation = self.next_generation;
        self.next_generation = self.next_generation.wrapping_add(1);

        let slot = &mut self.slots[index];
        slot.generation = generation;
        slot.waker = Some(waker.clone());
        slot.in_use = true;
        slot.queued = true;

        let key = SenderWaiterKey { index, generation };
        self.queue.push_back(key);
        *waiter_key = Some(key);
    }

    fn unregister(&mut self, waiter_key: SenderWaiterKey) {
        let Some(slot) = self.slots.get_mut(waiter_key.index) else {
            return;
        };
        if !slot.in_use || slot.generation != waiter_key.generation {
            return;
        }
        slot.in_use = false;
        slot.queued = false;
        slot.waker = None;
        self.free_slots.push(waiter_key.index);
    }
}

struct State<PData> {
    /// Single-threaded ownership of the queue core plus the wake state layered
    /// around it. `Inner` owns all admission and delivery semantics; this
    /// wrapper only handles waiting, sender liveness, and wake routing.
    inner: RefCell<Inner<PData>>,
    notify: Notify,
    /// Number of live sender handles. When the last sender drops, the channel
    /// transitions to closed so the receiver can finish after buffered work is
    /// drained.
    sender_count: Cell<usize>,
    /// Contended completion sends register here so capacity release can wake a
    /// bounded FIFO subset of blocked senders without waking every waiter.
    sender_waiters: RefCell<Option<SenderWaiters>>,
}

impl<PData> State<PData> {
    fn register_or_refresh_sender_waiter(
        &self,
        waiter_key: &mut Option<SenderWaiterKey>,
        waker: &Waker,
    ) {
        self.sender_waiters
            .borrow_mut()
            .get_or_insert_with(SenderWaiters::new)
            .register_or_refresh(waiter_key, waker);
    }

    fn unregister_sender_waiter(&self, waiter_key: SenderWaiterKey) {
        if let Some(waiters) = self.sender_waiters.borrow_mut().as_mut() {
            waiters.unregister(waiter_key);
        }
    }

    fn wake_completion_waiters(&self, slots_freed: usize) {
        if slots_freed == 0 {
            return;
        }
        if let Some(waiters) = self.sender_waiters.borrow_mut().as_mut() {
            waiters.wake_n(slots_freed);
        }
    }

    fn wake_all_sender_waiters(&self) {
        if let Some(waiters) = self.sender_waiters.borrow_mut().as_mut() {
            waiters.wake_all();
        }
    }
}

struct ControlSenderCore<PData> {
    state: Rc<State<PData>>,
}

struct ControlReceiverCore<PData> {
    state: Rc<State<PData>>,
}

/// Sender for receiver nodes, including `DrainIngress`.
pub struct ReceiverControlSender<PData> {
    inner: ControlSenderCore<PData>,
}

/// Sender for non-receiver nodes.
pub struct NodeControlSender<PData> {
    inner: ControlSenderCore<PData>,
}

/// Receiver for receiver-role control events.
pub struct ReceiverControlReceiver<PData> {
    inner: ControlReceiverCore<PData>,
}

/// Receiver for non-receiver node control events.
pub struct NodeControlReceiver<PData> {
    inner: ControlReceiverCore<PData>,
}

struct SendFuture<'a, PData> {
    sender: &'a ControlSenderCore<PData>,
    pending_cmd: Option<ControlCmd<PData>>,
    waiter_key: Option<SenderWaiterKey>,
    deadline_sleep: Option<Pin<Box<Sleep>>>,
}

impl<PData> Clone for ControlSenderCore<PData> {
    fn clone(&self) -> Self {
        self.state
            .sender_count
            .set(self.state.sender_count.get().saturating_add(1));
        Self {
            state: Rc::clone(&self.state),
        }
    }
}

impl<PData> Drop for ControlSenderCore<PData> {
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

impl<PData> ControlSenderCore<PData> {
    fn accept_drain_ingress(&self, msg: DrainIngressMsg) -> LifecycleSendResult {
        let result = self.state.inner.borrow_mut().record_drain_ingress(msg);
        if matches!(result, LifecycleSendResult::Accepted) {
            self.state.wake_all_sender_waiters();
            self.state.notify.notify_waiters();
        }
        result
    }

    fn accept_shutdown(&self, msg: ShutdownMsg) -> LifecycleSendResult {
        let result = self.state.inner.borrow_mut().record_shutdown(msg);
        if matches!(result, LifecycleSendResult::Accepted) {
            self.state.wake_all_sender_waiters();
            self.state.notify.notify_waiters();
        }
        result
    }

    fn try_send(&self, cmd: ControlCmd<PData>) -> Result<SendOutcome, crate::TrySendError<PData>> {
        let result = self.state.inner.borrow_mut().try_send(cmd);
        if matches!(result, Ok(SendOutcome::Accepted | SendOutcome::Replaced)) {
            self.state.notify.notify_one();
        }
        result
    }

    async fn send(&self, cmd: ControlCmd<PData>) -> Result<SendOutcome, SendError<PData>> {
        SendFuture {
            sender: self,
            pending_cmd: Some(cmd),
            waiter_key: None,
            deadline_sleep: None,
        }
        .await
    }

    fn close(&self) {
        let closed = self.state.inner.borrow_mut().close();
        if closed {
            self.state.wake_all_sender_waiters();
            self.state.notify.notify_waiters();
        }
    }

    fn stats(&self) -> ControlChannelStats {
        self.state.inner.borrow().stats()
    }
}

impl<'a, PData> Unpin for SendFuture<'a, PData> {}

impl<PData> Drop for SendFuture<'_, PData> {
    fn drop(&mut self) {
        let Some(waiter_key) = self.waiter_key.take() else {
            return;
        };
        self.sender.state.unregister_sender_waiter(waiter_key);
    }
}

impl<PData> Future for SendFuture<'_, PData> {
    type Output = Result<SendOutcome, SendError<PData>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();

        loop {
            let cmd = this
                .pending_cmd
                .take()
                .expect("SendFuture polled after completion");

            match this.sender.try_send(cmd) {
                Ok(outcome) => {
                    if let Some(waiter_key) = this.waiter_key.take() {
                        this.sender.state.unregister_sender_waiter(waiter_key);
                    }
                    this.deadline_sleep = None;
                    return Poll::Ready(Ok(outcome));
                }
                Err(crate::TrySendError::Closed(cmd)) => {
                    if let Some(waiter_key) = this.waiter_key.take() {
                        this.sender.state.unregister_sender_waiter(waiter_key);
                    }
                    this.deadline_sleep = None;
                    return Poll::Ready(Err(SendError::Closed(cmd)));
                }
                Err(crate::TrySendError::Full { cmd, .. }) => {
                    this.pending_cmd = Some(cmd);
                }
            }

            let mut waiter_key = this.waiter_key;
            this.sender
                .state
                .register_or_refresh_sender_waiter(&mut waiter_key, cx.waker());
            this.waiter_key = waiter_key;

            let deadline = this.sender.state.inner.borrow().next_deadline();
            if let Some(deadline) = deadline {
                let deadline = TokioInstant::from_std(deadline);
                let needs_reset = this
                    .deadline_sleep
                    .as_ref()
                    .is_none_or(|sleep| sleep.deadline() != deadline);
                if needs_reset {
                    this.deadline_sleep = Some(Box::pin(sleep_until(deadline)));
                }
                if let Some(sleep) = this.deadline_sleep.as_mut() {
                    if sleep.as_mut().poll(cx).is_ready() {
                        this.deadline_sleep = None;
                        if let Some(waiter_key) = this.waiter_key.take() {
                            this.sender.state.unregister_sender_waiter(waiter_key);
                        }
                        continue;
                    }
                }
            } else {
                this.deadline_sleep = None;
            }

            return Poll::Pending;
        }
    }
}

impl<PData> Clone for ReceiverControlSender<PData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<PData> Clone for NodeControlSender<PData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<PData> ReceiverControlSender<PData> {
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
    ) -> Result<SendOutcome, crate::TrySendError<PData>> {
        self.inner.try_send(cmd)
    }

    /// Sends a non-lifecycle control command, waiting asynchronously for bounded
    /// completion capacity when needed.
    pub async fn send(&self, cmd: ControlCmd<PData>) -> Result<SendOutcome, SendError<PData>> {
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

impl<PData> NodeControlSender<PData> {
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
    ) -> Result<SendOutcome, crate::TrySendError<PData>> {
        self.inner.try_send(cmd)
    }

    /// Sends a non-lifecycle control command, waiting asynchronously for bounded
    /// completion capacity when needed.
    pub async fn send(&self, cmd: ControlCmd<PData>) -> Result<SendOutcome, SendError<PData>> {
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

impl<PData> ControlReceiverCore<PData> {
    fn notify_after_pop(&self, event: &CoreControlEvent<PData>) {
        match event {
            CoreControlEvent::CompletionBatch(batch) => {
                self.state.wake_completion_waiters(batch.len());
            }
            CoreControlEvent::Shutdown(_) => {
                self.state.wake_all_sender_waiters();
                self.state.notify.notify_waiters();
            }
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

            // Avoid waiting if the queue changed after we decided it was
            // currently empty. Shutdown deadlines are part of the wait
            // condition so forced shutdown does not depend on a producer
            // arriving later to wake the receiver.
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

impl<PData> ReceiverControlReceiver<PData> {
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

impl<PData> NodeControlReceiver<PData> {
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
) -> Result<(ControlSenderCore<PData>, ControlReceiverCore<PData>), ConfigError> {
    config.validate()?;
    let state = Rc::new(State {
        inner: RefCell::new(Inner::new(config)),
        notify: Notify::new(),
        sender_count: Cell::new(1),
        sender_waiters: RefCell::new(None),
    });

    Ok((
        ControlSenderCore {
            state: Rc::clone(&state),
        },
        ControlReceiverCore { state },
    ))
}

/// Creates a new control-aware channel pair for receiver nodes.
pub fn receiver_channel<PData>(
    config: ControlChannelConfig,
) -> Result<(ReceiverControlSender<PData>, ReceiverControlReceiver<PData>), ConfigError> {
    let (sender, receiver) = channel_state(config)?;
    Ok((
        ReceiverControlSender { inner: sender },
        ReceiverControlReceiver { inner: receiver },
    ))
}

/// Creates a new control-aware channel pair for non-receiver nodes.
pub fn node_channel<PData>(
    config: ControlChannelConfig,
) -> Result<(NodeControlSender<PData>, NodeControlReceiver<PData>), ConfigError> {
    let (sender, receiver) = channel_state(config)?;
    Ok((
        NodeControlSender { inner: sender },
        NodeControlReceiver { inner: receiver },
    ))
}
