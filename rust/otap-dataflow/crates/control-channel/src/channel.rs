// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Standalone control-aware channel with a single channel-receiver-owned queue
//! core.
//!
//! "Single-owner" means one channel receiver task owns and mutates the queue
//! state machine. Cloned senders can submit work and wait for capacity, but
//! they do not share mutable queue state.
//!
//! The design is split into three layers:
//!
//! - [`Inner`] owns the control-specific semantics: lifecycle recording,
//!   bounded completion admission, completion batching, latest-wins config
//!   replacement, coalesced timer/telemetry signals, bounded fairness, and
//!   deadline-bounded forced shutdown progress.
//! - [`State`] wraps that core with single-threaded ownership, sender liveness,
//!   wake routing for the channel receiver, and blocked-sender waiter
//!   management.
//! - [`ControlSenderCore`] and [`ControlReceiverCore`] provide the operational
//!   sender/receiver behavior over that shared state.
//!
//! The generic parameter `Meta` is the completion-side metadata carried by
//! `Ack`/`Nack` messages. Use `Meta = ()` when completions only need to return
//! the payload, or supply a richer type when the integration needs explicit
//! unwind/routing context alongside that payload.
//!
//! The hot path is intentionally asymmetric:
//!
//! - senders only mutate the queue through short `RefCell` borrows and never
//!   own the queue core directly
//! - the channel receiver is the only side that pops events and advances drain
//!   or forced-shutdown progress
//! - only completion traffic can become capacity-bound; lifecycle signals are
//!   recorded separately and config/timer/telemetry use replacement or
//!   coalescing rules instead of ordinary FIFO queuing
//!
//! Under contention, blocked completion senders wait in a keyed FIFO waiter
//! structure. That keeps wakeups bounded when completion slots are released,
//! while still allowing terminal transitions such as shutdown or close to wake
//! everyone so they can observe the new state.

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

// TODO: Consider deduplicating the keyed blocked-sender waiter logic with
// `otap-df-channel` by extracting a shared `sender_waiters.rs` there.
// The current `mpsc` and `mpmc` waiters carry the same tombstone pattern;
// this crate fixes it locally first to keep this PR isolated.
//
// The reusable part is the waiter mechanism itself: stable waiter keys, slot
// reuse, cancellation-safe unregister, and FIFO wake order. The control
// channel would still keep its own `Inner` state machine because control
// admission, fairness, batching, and lifecycle handling are more specialized
// than the generic channel crate.
//
// This PR keeps a local copy on purpose to stay simple and isolated while the
// standalone control-channel design is being reviewed. Any deduplication should
// happen in a second phase and must be benchmarked to confirm that extracting
// the waiter code preserves the current performance characteristics.

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

const SENDER_WAITER_COMPACT_MIN_QUEUE_LEN: usize = 64;

struct SenderWaiters {
    // FIFO queue of waiter keys used to preserve blocked-sender wake order.
    queue: VecDeque<SenderWaiterKey>,
    // Slot storage allows O(1) refresh/unregister by key without scanning the queue.
    slots: Vec<SenderWaiterSlot>,
    // Reuse released slots to avoid per-contention allocations.
    free_slots: Vec<usize>,
    // Number of live queued waiters still represented in `queue`.
    queued_live: usize,
    // Number of stale queued entries left behind by cancellation and awaiting
    // wake-time cleanup or periodic compaction.
    queued_stale: usize,
    next_generation: u64,
}

impl SenderWaiters {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            slots: Vec::new(),
            free_slots: Vec::new(),
            queued_live: 0,
            queued_stale: 0,
            next_generation: 0,
        }
    }

    fn wake_n(&mut self, mut count: usize) {
        while count > 0 {
            let Some(key) = self.queue.pop_front() else {
                break;
            };
            let Some(slot) = self.slots.get_mut(key.index) else {
                self.queued_stale -= 1;
                continue;
            };
            // Stale queue entries are expected when futures are canceled or
            // re-queued; skip until we find a live queued waiter.
            if !slot.in_use || slot.generation != key.generation || !slot.queued {
                self.queued_stale -= 1;
                continue;
            }
            slot.queued = false;
            self.queued_live -= 1;
            if let Some(waker) = slot.waker.take() {
                waker.wake();
                count -= 1;
            }
        }
    }

    fn wake_all(&mut self) {
        while let Some(key) = self.queue.pop_front() {
            let Some(slot) = self.slots.get_mut(key.index) else {
                self.queued_stale -= 1;
                continue;
            };
            if !slot.in_use || slot.generation != key.generation || !slot.queued {
                self.queued_stale -= 1;
                continue;
            }
            slot.queued = false;
            self.queued_live -= 1;
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
                        self.queued_live += 1;
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
        self.queued_live += 1;
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
        if slot.queued {
            self.queued_live -= 1;
            self.queued_stale += 1;
        }
        slot.queued = false;
        slot.waker = None;
        self.free_slots.push(waiter_key.index);
        self.maybe_compact_queue();
    }

    fn maybe_compact_queue(&mut self) {
        if self.queue.len() < SENDER_WAITER_COMPACT_MIN_QUEUE_LEN {
            return;
        }
        if self.queued_stale * 2 < self.queue.len() {
            return;
        }
        self.compact_queue();
    }

    fn compact_queue(&mut self) {
        self.queue.retain(|key| {
            self.slots
                .get(key.index)
                .is_some_and(|slot| slot.in_use && slot.queued && slot.generation == key.generation)
        });
        self.queued_live = self.queue.len();
        self.queued_stale = 0;
    }
}

struct State<PData, Meta = ()> {
    /// Single-threaded ownership of the queue core plus the wake state layered
    /// around it. `Inner` owns all admission, batching, fairness, and
    /// lifecycle semantics; this wrapper only handles channel-receiver waiting,
    /// sender liveness, and wake routing.
    inner: RefCell<Inner<PData, Meta>>,
    notify: Notify,
    /// Number of live sender handles. When the last sender drops, the channel
    /// transitions to closed so the channel receiver can finish after buffered
    /// work is drained.
    sender_count: Cell<usize>,
    /// Contended completion sends register here so capacity release can wake a
    /// bounded FIFO subset of blocked senders without waking every waiter.
    ///
    /// The keyed waiter-slot structure is adapted from the local MPSC channel
    /// in `otap-df-channel`. The reuse is intentionally narrow: only the
    /// blocked-sender waiting mechanism is borrowed here, while control
    /// admission and delivery remain specific to `Inner`.
    sender_waiters: RefCell<Option<SenderWaiters>>,
}

impl<PData, Meta> State<PData, Meta> {
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

struct ControlSenderCore<PData, Meta = ()> {
    state: Rc<State<PData, Meta>>,
}

struct ControlReceiverCore<PData, Meta = ()> {
    state: Rc<State<PData, Meta>>,
}

/// Sender for receiver nodes, including `DrainIngress`.
pub struct ReceiverControlSender<PData, Meta = ()> {
    inner: ControlSenderCore<PData, Meta>,
}

/// Sender for non-receiver nodes.
pub struct NodeControlSender<PData, Meta = ()> {
    inner: ControlSenderCore<PData, Meta>,
}

/// Receiver for receiver-role control events.
pub struct ReceiverControlReceiver<PData, Meta = ()> {
    inner: ControlReceiverCore<PData, Meta>,
}

/// Receiver for non-receiver node control events.
pub struct NodeControlReceiver<PData, Meta = ()> {
    inner: ControlReceiverCore<PData, Meta>,
}

struct SendFuture<'a, PData, Meta = ()> {
    sender: &'a ControlSenderCore<PData, Meta>,
    // Owned command being retried across polls. `poll()` takes it out before
    // each send attempt so ownership can move into `try_send()`. If the queue
    // is still full, the returned command is stored back here before the
    // future parks, which keeps the original command available across wakeups,
    // cancellation, or close.
    pending_cmd: Option<ControlCmd<PData, Meta>>,
    waiter_key: Option<SenderWaiterKey>,
    // Forced-shutdown deadlines are re-checked while blocked so a completion
    // sender does not wait forever for capacity that shutdown semantics will
    // eventually abandon.
    deadline_sleep: Option<Pin<Box<Sleep>>>,
}

impl<PData, Meta> Clone for ControlSenderCore<PData, Meta> {
    fn clone(&self) -> Self {
        self.state
            .sender_count
            .set(self.state.sender_count.get().saturating_add(1));
        Self {
            state: Rc::clone(&self.state),
        }
    }
}

impl<PData, Meta> Drop for ControlSenderCore<PData, Meta> {
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

impl<PData, Meta> ControlSenderCore<PData, Meta> {
    /// Records `DrainIngress` outside the bounded completion capacity.
    ///
    /// Guarantee: once accepted, the lifecycle token is visible to the channel
    /// receiver even if ordinary completion traffic is saturated. Acceptance
    /// wakes the channel receiver promptly so it can observe the drain request,
    /// but it does not wake blocked completion senders because drain does not
    /// change completion admissibility or free completion capacity. The carried
    /// deadline is for the channel receiver's own ingress-drain logic; unlike
    /// `Shutdown`, it does not make the control-channel queue itself
    /// deadline-driven.
    fn accept_drain_ingress(&self, msg: DrainIngressMsg) -> LifecycleSendResult {
        let result = self.state.inner.borrow_mut().record_drain_ingress(msg);
        if matches!(result, LifecycleSendResult::Accepted) {
            self.state.notify.notify_waiters();
        }
        result
    }

    /// Records `Shutdown` outside the bounded completion capacity.
    ///
    /// Guarantee: once accepted, shutdown is remembered even if the completion
    /// queue is full. Acceptance wakes both blocked senders and the channel
    /// receiver so forced-shutdown deadlines and abandonment rules do not
    /// depend on later producer activity.
    fn accept_shutdown(&self, msg: ShutdownMsg) -> LifecycleSendResult {
        let result = self.state.inner.borrow_mut().record_shutdown(msg);
        if matches!(result, LifecycleSendResult::Accepted) {
            self.state.wake_all_sender_waiters();
            self.state.notify.notify_waiters();
        }
        result
    }

    /// Attempts one immediate non-lifecycle send without waiting.
    ///
    /// Guarantee: this never parks the caller. It either applies the command
    /// immediately, returns `Full` with the original command, or returns
    /// `Closed` with the original command. Successful admission wakes the
    /// channel receiver once so newly available work can be observed.
    fn try_send(
        &self,
        cmd: ControlCmd<PData, Meta>,
    ) -> Result<SendOutcome, crate::TrySendError<PData, Meta>> {
        let result = self.state.inner.borrow_mut().try_send(cmd);
        if matches!(result, Ok(SendOutcome::Accepted | SendOutcome::Replaced)) {
            self.state.notify.notify_one();
        }
        result
    }

    /// Sends one non-lifecycle command, waiting asynchronously only when
    /// completion capacity is temporarily exhausted.
    ///
    /// Guarantee: the future preserves the original command until it is either
    /// accepted/replaced or returned as `Closed`. While blocked, the future is
    /// cancellation-safe and re-checks forced-shutdown deadlines so it does
    /// not wait forever for capacity that terminal state will eventually
    /// abandon.
    async fn send(
        &self,
        cmd: ControlCmd<PData, Meta>,
    ) -> Result<SendOutcome, SendError<PData, Meta>> {
        SendFuture {
            sender: self,
            pending_cmd: Some(cmd),
            waiter_key: None,
            deadline_sleep: None,
        }
        .await
    }

    /// Closes the channel for new sends.
    ///
    /// Guarantee: close wakes both blocked senders and the channel receiver so
    /// all parties can observe terminal state without needing any further
    /// producer activity.
    fn close(&self) {
        let closed = self.state.inner.borrow_mut().close();
        if closed {
            self.state.wake_all_sender_waiters();
            self.state.notify.notify_waiters();
        }
    }

    /// Returns a snapshot of the current queue occupancy and lifecycle state.
    ///
    /// Guarantee: this is observational only; it does not change wake state or
    /// queue contents.
    fn stats(&self) -> ControlChannelStats {
        self.state.inner.borrow().stats()
    }
}

impl<'a, PData, Meta> Unpin for SendFuture<'a, PData, Meta> {}

impl<PData, Meta> Drop for SendFuture<'_, PData, Meta> {
    fn drop(&mut self) {
        let Some(waiter_key) = self.waiter_key.take() else {
            return;
        };
        self.sender.state.unregister_sender_waiter(waiter_key);
    }
}

impl<PData, Meta> Future for SendFuture<'_, PData, Meta> {
    type Output = Result<SendOutcome, SendError<PData, Meta>>;

    // `poll()` implements a small retry state machine around `try_send()`:
    //
    // - it takes ownership of the current command from `pending_cmd`
    // - it attempts immediate admission through `try_send()`
    // - on `Full`, it stores the command back, refreshes waiter registration,
    //   arms or refreshes the forced-shutdown deadline sleep, and returns
    //   `Pending`
    // - on `Accepted`/`Replaced` or `Closed`, it clears waiter/deadline state
    //   and returns `Ready`
    //
    // Guarantees:
    // - the original command is never lost while the future is pending
    // - cancellation is safe because `Drop` unregisters any outstanding waiter
    // - blocked sends do not depend on a future producer wakeup to observe
    //   forced shutdown progress, because the deadline path loops back into
    //   admission checks on its own
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();

        loop {
            // `pending_cmd` is populated when the future is created and is
            // restored before every `Poll::Pending` return on the full-queue
            // path. Ready returns exit immediately, so hitting this `expect`
            // would mean the future was polled again after completion or that
            // this function broke its own local invariant.
            let cmd = this
                .pending_cmd
                .take()
                .expect("SendFuture missing pending_cmd invariant");

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

impl<PData, Meta> Clone for ReceiverControlSender<PData, Meta> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<PData, Meta> Clone for NodeControlSender<PData, Meta> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<PData, Meta> ReceiverControlSender<PData, Meta> {
    /// Accepts a receiver-drain lifecycle token without consuming bounded
    /// control capacity.
    ///
    /// Guarantee: once accepted, the drain token remains observable to the
    /// channel receiver even if ordinary completion traffic is saturated.
    /// Acceptance wakes the channel receiver promptly, but it does not wake
    /// blocked completion senders because drain does not free completion
    /// capacity. The embedded deadline is intended for receiver-local ingress-
    /// drain behavior, not for queue-level forced shutdown in the control
    /// channel itself.
    #[must_use]
    pub fn accept_drain_ingress(&self, msg: DrainIngressMsg) -> LifecycleSendResult {
        self.inner.accept_drain_ingress(msg)
    }

    /// Accepts a shutdown lifecycle token without consuming bounded control
    /// capacity.
    ///
    /// Guarantee: once accepted, shutdown is remembered even if the completion
    /// queue is full. Acceptance wakes both blocked senders and the channel
    /// receiver so forced-shutdown deadlines and abandonment rules do not
    /// depend on later producer activity.
    #[must_use]
    pub fn accept_shutdown(&self, msg: ShutdownMsg) -> LifecycleSendResult {
        self.inner.accept_shutdown(msg)
    }

    /// Attempts to send a non-lifecycle control command without waiting for
    /// capacity.
    ///
    /// Guarantee: this never parks the caller. It either applies the command
    /// immediately, returns `Full` with the original command, or returns
    /// `Closed` with the original command. Successful admission wakes the
    /// channel receiver once so newly available work can be observed.
    pub fn try_send(
        &self,
        cmd: ControlCmd<PData, Meta>,
    ) -> Result<SendOutcome, crate::TrySendError<PData, Meta>> {
        self.inner.try_send(cmd)
    }

    /// Sends a non-lifecycle control command, waiting asynchronously for
    /// bounded completion capacity when needed.
    ///
    /// Guarantee: this only waits when completion capacity is exhausted. The
    /// original command is preserved until it is accepted/replaced or returned
    /// as `Closed`, and the wait path is cancellation-safe.
    pub async fn send(
        &self,
        cmd: ControlCmd<PData, Meta>,
    ) -> Result<SendOutcome, SendError<PData, Meta>> {
        self.inner.send(cmd).await
    }

    /// Closes the channel for new sends.
    ///
    /// Guarantee: close wakes both blocked senders and the channel receiver so
    /// all parties can observe terminal state without needing any further
    /// producer activity.
    pub fn close(&self) {
        self.inner.close();
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    ///
    /// Guarantee: this is observational only; it does not change wake state or
    /// queue contents.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.inner.stats()
    }
}

impl<PData, Meta> NodeControlSender<PData, Meta> {
    /// Accepts a shutdown lifecycle token without consuming bounded control
    /// capacity.
    ///
    /// Guarantee: once accepted, shutdown is remembered even if the completion
    /// queue is full. Acceptance wakes both blocked senders and the channel
    /// receiver so forced-shutdown deadlines and abandonment rules do not
    /// depend on later producer activity.
    #[must_use]
    pub fn accept_shutdown(&self, msg: ShutdownMsg) -> LifecycleSendResult {
        self.inner.accept_shutdown(msg)
    }

    /// Attempts to send a non-lifecycle control command without waiting for
    /// capacity.
    ///
    /// Guarantee: this never parks the caller. It either applies the command
    /// immediately, returns `Full` with the original command, or returns
    /// `Closed` with the original command. Successful admission wakes the
    /// channel receiver once so newly available work can be observed.
    pub fn try_send(
        &self,
        cmd: ControlCmd<PData, Meta>,
    ) -> Result<SendOutcome, crate::TrySendError<PData, Meta>> {
        self.inner.try_send(cmd)
    }

    /// Sends a non-lifecycle control command, waiting asynchronously for
    /// bounded completion capacity when needed.
    ///
    /// Guarantee: this only waits when completion capacity is exhausted. The
    /// original command is preserved until it is accepted/replaced or returned
    /// as `Closed`, and the wait path is cancellation-safe.
    pub async fn send(
        &self,
        cmd: ControlCmd<PData, Meta>,
    ) -> Result<SendOutcome, SendError<PData, Meta>> {
        self.inner.send(cmd).await
    }

    /// Closes the channel for new sends.
    ///
    /// Guarantee: close wakes both blocked senders and the channel receiver so
    /// all parties can observe terminal state without needing any further
    /// producer activity.
    pub fn close(&self) {
        self.inner.close();
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    ///
    /// Guarantee: this is observational only; it does not change wake state or
    /// queue contents.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.inner.stats()
    }
}

impl<PData, Meta> ControlReceiverCore<PData, Meta> {
    fn notify_after_pop(&self, event: &CoreControlEvent<PData, Meta>) {
        match event {
            CoreControlEvent::CompletionBatch(batch) => {
                // One freed completion slot should wake one blocked sender, so
                // batch drains wake a bounded FIFO subset instead of a herd.
                self.state.wake_completion_waiters(batch.len());
            }
            CoreControlEvent::Shutdown(_) => {
                // Shutdown is terminal state, not ordinary capacity release, so
                // all blocked senders must wake and observe the new state.
                self.state.wake_all_sender_waiters();
                self.state.notify.notify_waiters();
            }
            CoreControlEvent::DrainIngress(_)
            | CoreControlEvent::Config { .. }
            | CoreControlEvent::TimerTick
            | CoreControlEvent::CollectTelemetry => {}
        }
    }

    async fn recv_internal(&mut self) -> Option<CoreControlEvent<PData, Meta>> {
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

            // Avoid waiting if the queue changed after the channel receiver
            // decided it was currently empty. Shutdown deadlines are part of
            // the wait condition so forced shutdown does not depend on a
            // producer arriving later to wake the channel receiver.
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

    fn try_recv_internal(&mut self) -> Option<CoreControlEvent<PData, Meta>> {
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

impl<PData, Meta> ReceiverControlReceiver<PData, Meta> {
    /// Receives the next available control event, or `None` if the channel is
    /// closed and fully drained.
    pub async fn recv(&mut self) -> Option<ReceiverControlEvent<PData, Meta>> {
        self.inner
            .recv_internal()
            .await
            .map(ReceiverControlEvent::<PData, Meta>::from_core)
    }

    /// Attempts to receive one control event without waiting.
    pub fn try_recv(&mut self) -> Option<ReceiverControlEvent<PData, Meta>> {
        self.inner
            .try_recv_internal()
            .map(ReceiverControlEvent::<PData, Meta>::from_core)
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.inner.stats()
    }
}

impl<PData, Meta> NodeControlReceiver<PData, Meta> {
    /// Receives the next available control event, or `None` if the channel is
    /// closed and fully drained.
    pub async fn recv(&mut self) -> Option<NodeControlEvent<PData, Meta>> {
        self.inner
            .recv_internal()
            .await
            .map(NodeControlEvent::<PData, Meta>::from_core)
    }

    /// Attempts to receive one control event without waiting.
    pub fn try_recv(&mut self) -> Option<NodeControlEvent<PData, Meta>> {
        self.inner
            .try_recv_internal()
            .map(NodeControlEvent::<PData, Meta>::from_core)
    }

    /// Returns a snapshot of channel occupancy and lifecycle state.
    #[must_use]
    pub fn stats(&self) -> ControlChannelStats {
        self.inner.stats()
    }
}

fn channel_state<PData, Meta>(
    config: ControlChannelConfig,
) -> Result<
    (
        ControlSenderCore<PData, Meta>,
        ControlReceiverCore<PData, Meta>,
    ),
    ConfigError,
> {
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

/// Creates a new control-aware sender/channel-receiver pair for receiver nodes.
pub fn receiver_channel<PData>(
    config: ControlChannelConfig,
) -> Result<(ReceiverControlSender<PData>, ReceiverControlReceiver<PData>), ConfigError> {
    receiver_channel_with_meta(config)
}

/// Creates a new control-aware sender/channel-receiver pair for receiver nodes
/// with explicit completion metadata carried by `Ack`/`Nack`.
pub fn receiver_channel_with_meta<PData, Meta>(
    config: ControlChannelConfig,
) -> Result<
    (
        ReceiverControlSender<PData, Meta>,
        ReceiverControlReceiver<PData, Meta>,
    ),
    ConfigError,
> {
    let (sender, receiver) = channel_state(config)?;
    Ok((
        ReceiverControlSender { inner: sender },
        ReceiverControlReceiver { inner: receiver },
    ))
}

/// Creates a new control-aware sender/channel-receiver pair for non-receiver
/// nodes.
pub fn node_channel<PData>(
    config: ControlChannelConfig,
) -> Result<(NodeControlSender<PData>, NodeControlReceiver<PData>), ConfigError> {
    node_channel_with_meta(config)
}

/// Creates a new control-aware sender/channel-receiver pair for non-receiver
/// nodes with explicit completion metadata carried by `Ack`/`Nack`.
pub fn node_channel_with_meta<PData, Meta>(
    config: ControlChannelConfig,
) -> Result<
    (
        NodeControlSender<PData, Meta>,
        NodeControlReceiver<PData, Meta>,
    ),
    ConfigError,
> {
    let (sender, receiver) = channel_state(config)?;
    Ok((
        NodeControlSender { inner: sender },
        NodeControlReceiver { inner: receiver },
    ))
}

#[cfg(test)]
mod sender_waiters_tests {
    use super::{SENDER_WAITER_COMPACT_MIN_QUEUE_LEN, SenderWaiters};
    use std::sync::{Arc, Mutex};
    use std::task::{Wake, Waker};

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}

        fn wake_by_ref(self: &Arc<Self>) {}
    }

    struct RecordingWake {
        id: usize,
        wake_log: Arc<Mutex<Vec<usize>>>,
    }

    impl RecordingWake {
        fn new(id: usize, wake_log: Arc<Mutex<Vec<usize>>>) -> Self {
            Self { id, wake_log }
        }
    }

    impl Wake for RecordingWake {
        fn wake(self: Arc<Self>) {
            self.wake_log.lock().unwrap().push(self.id);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.wake_log.lock().unwrap().push(self.id);
        }
    }

    fn noop_waker() -> Waker {
        Waker::from(Arc::new(NoopWake))
    }

    #[test]
    fn repeated_unregister_keeps_queue_length_bounded_without_wakes() {
        // Scenario: blocked send futures are repeatedly canceled before any
        // completion capacity is released, so no wake path drains tombstones.
        // Guarantees: periodic compaction keeps the waiter queue bounded even
        // under pure cancellation churn with no intervening wakeups.
        let mut waiters = SenderWaiters::new();
        let waker = noop_waker();

        for _ in 0..(SENDER_WAITER_COMPACT_MIN_QUEUE_LEN * 3) {
            let mut waiter_key = None;
            waiters.register_or_refresh(&mut waiter_key, &waker);
            waiters.unregister(waiter_key.unwrap());

            assert!(waiters.queue.len() < SENDER_WAITER_COMPACT_MIN_QUEUE_LEN);
            assert_eq!(waiters.queued_live, 0);
            assert_eq!(waiters.queue.len(), waiters.queued_stale);
        }
    }

    #[test]
    fn compaction_preserves_fifo_order_for_live_waiters() {
        // Scenario: compaction runs after many canceled waiters while a few
        // live waiters are still queued in FIFO order.
        // Guarantees: compaction removes stale tombstones without changing
        // the wake order of the remaining live waiters.
        let mut waiters = SenderWaiters::new();
        let wake_log = Arc::new(Mutex::new(Vec::new()));

        for id in 1..=3 {
            let mut waiter_key = None;
            let waker = Waker::from(Arc::new(RecordingWake::new(id, Arc::clone(&wake_log))));
            waiters.register_or_refresh(&mut waiter_key, &waker);
        }

        let stale_needed_for_compaction = SENDER_WAITER_COMPACT_MIN_QUEUE_LEN - waiters.queued_live;
        let noop_waker = noop_waker();
        for _ in 0..stale_needed_for_compaction {
            let mut waiter_key = None;
            waiters.register_or_refresh(&mut waiter_key, &noop_waker);
            waiters.unregister(waiter_key.unwrap());
        }

        assert_eq!(waiters.queued_stale, 0);
        assert_eq!(waiters.queued_live, 3);
        assert_eq!(waiters.queue.len(), 3);

        waiters.wake_all();

        assert_eq!(*wake_log.lock().unwrap(), vec![1, 2, 3]);
        assert!(waiters.queue.is_empty());
        assert_eq!(waiters.queued_live, 0);
        assert_eq!(waiters.queued_stale, 0);
    }

    #[test]
    fn compaction_removes_generation_mismatched_stale_entries() {
        // Scenario: a slot is canceled, then reused for a later waiter with a
        // new generation while the old queue entry is still present.
        // Guarantees: compaction drops the stale generation-mismatched entry
        // and retains only the live waiter for the reused slot.
        let mut waiters = SenderWaiters::new();
        let waker = noop_waker();

        let mut first = None;
        waiters.register_or_refresh(&mut first, &waker);
        let first_key = first.unwrap();
        waiters.unregister(first_key);

        let mut second = None;
        waiters.register_or_refresh(&mut second, &waker);
        let second_key = second.unwrap();

        waiters.compact_queue();

        assert_eq!(waiters.queue.len(), 1);
        let retained = waiters.queue.front().copied().unwrap();
        assert_eq!(retained.index, second_key.index);
        assert_eq!(retained.generation, second_key.generation);
        assert_ne!(retained.generation, first_key.generation);
        assert_eq!(waiters.queued_live, 1);
        assert_eq!(waiters.queued_stale, 0);
    }
}
