// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic internals, nothing here is directly exposed to users.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use crate::error::Error;
use crate::error::Error::{
    MessageNotTracked, SubscribeBalancedNotSupported, SubscribeBroadcastNotSupported,
    SubscribeSingleGroupViolation, SubscriptionClosed, TopicClosed,
};
use crate::topic::backend::{PublishFuture, PublishTrackedFuture, SubscriptionBackend, TopicState};
use crate::topic::subscription::{Delivery, RecvDelivery};
use crate::topic::types::{
    AckFromResult, BroadcastSubscriberId, Envelope, NackFromResult, PublishOutcome,
    SubscriberOptions, TopicOptions, TrackedPublishOutcome, TrackedPublishPermit,
    TrackedPublishReceipt, TrackedPublishTracker, TrackedTryPublishOutcome,
};
use futures_core::Stream;
use otap_df_config::topic::{TopicBroadcastAckMode, TopicBroadcastOnLagPolicy};
use otap_df_config::{SubscriptionGroupName, TopicName};
use parking_lot::{Mutex, RwLock};
use smallvec::SmallVec;
use std::collections::HashSet;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

struct QueuedEnvelope<T> {
    envelope: Envelope<T>,
    _admission_permit: OwnedSemaphorePermit,
}

struct ConsumerGroup<T> {
    tx: async_channel::Sender<QueuedEnvelope<T>>,
    rx: async_channel::Receiver<QueuedEnvelope<T>>,
    admission: Arc<Semaphore>,
}

struct SingleGroup<T> {
    group_name: SubscriptionGroupName,
    tx: async_channel::Sender<QueuedEnvelope<T>>,
    rx: async_channel::Receiver<QueuedEnvelope<T>>,
    admission: Arc<Semaphore>,
}

pub(crate) enum BroadcastReadResult<T> {
    Ready(Envelope<T>),
    NotReady,
    Lagged { missed: u64, new_read_seq: u64 },
}

struct WakerSet {
    has_waiters: AtomicBool,
    wakers: Mutex<Vec<Waker>>,
}

impl WakerSet {
    fn new() -> Self {
        Self {
            has_waiters: AtomicBool::new(false),
            wakers: Mutex::new(Vec::new()),
        }
    }

    fn register(&self, waker: &Waker) {
        let mut wakers = self.wakers.lock();
        for existing in wakers.iter_mut() {
            if existing.will_wake(waker) {
                existing.clone_from(waker);
                self.has_waiters.store(true, Ordering::Release);
                return;
            }
        }
        wakers.push(waker.clone());
        self.has_waiters.store(true, Ordering::Release);
    }

    fn wake_all(&self) {
        if !self.has_waiters.load(Ordering::Acquire) {
            return;
        }
        let wakers = {
            let mut guard = self.wakers.lock();
            self.has_waiters.store(false, Ordering::Release);
            std::mem::take(&mut *guard)
        };
        for waker in wakers {
            waker.wake();
        }
    }
}

fn send_queued_envelope<T>(
    sender: &async_channel::Sender<QueuedEnvelope<T>>,
    envelope: Envelope<T>,
    permit: OwnedSemaphorePermit,
) -> Result<(), Error> {
    match sender.try_send(QueuedEnvelope {
        envelope,
        _admission_permit: permit,
    }) {
        Ok(()) => Ok(()),
        Err(async_channel::TrySendError::Full(_)) => {
            panic!("reserved topic admission permit should guarantee queue capacity")
        }
        Err(async_channel::TrySendError::Closed(_)) => Err(TopicClosed),
    }
}

async fn acquire_balanced_permit(
    admission: &Arc<Semaphore>,
) -> Result<OwnedSemaphorePermit, Error> {
    admission
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| TopicClosed)
}

fn try_acquire_balanced_permit(
    admission: &Arc<Semaphore>,
) -> Result<Option<OwnedSemaphorePermit>, Error> {
    match admission.clone().try_acquire_owned() {
        Ok(permit) => Ok(Some(permit)),
        Err(tokio::sync::TryAcquireError::NoPermits) => Ok(None),
        Err(tokio::sync::TryAcquireError::Closed) => Err(TopicClosed),
    }
}

type BalancedPermitVec = SmallVec<[OwnedSemaphorePermit; 4]>;

pub(crate) struct FastBroadcastRing<T: Send + Sync + 'static> {
    slots: Box<[Mutex<Option<(u64, Envelope<T>)>>]>,
    capacity: usize,
    mask: usize,
    write_seq: AtomicU64,
    waker_set: WakerSet,
    closed: AtomicBool,
}

impl<T: Send + Sync + 'static> FastBroadcastRing<T> {
    fn new(capacity: usize) -> Self {
        let cap = capacity.max(2).next_power_of_two();
        let mask = cap - 1;
        debug_assert!(cap.is_power_of_two());
        debug_assert_eq!(mask, cap - 1);
        let mut slots = Vec::with_capacity(cap);
        for _ in 0..cap {
            slots.push(Mutex::new(None));
        }
        Self {
            slots: slots.into_boxed_slice(),
            capacity: cap,
            mask,
            write_seq: AtomicU64::new(0),
            waker_set: WakerSet::new(),
            closed: AtomicBool::new(false),
        }
    }

    /// Publish an envelope: reserve the next sequence, then write its slot.
    ///
    /// This is the default (`First`-mode) path and is equivalent to calling
    /// [`FastBroadcastRing::reserve_seq`] immediately followed by
    /// [`FastBroadcastRing::commit_slot`]. The two are split so `all`-mode
    /// broadcast publish can reserve the sequence under the subscriber-registry
    /// lock (to snapshot the exact consensus membership for that sequence) and then
    /// perform the heavier slot write plus waker fan-out after releasing the
    /// lock. See `reserve_seq`/`commit_slot` for the reserved-but-uncommitted
    /// window, which is read-safe (readers see `NotReady` and re-park).
    pub(crate) fn publish(&self, envelope: Envelope<T>) {
        let seq = self.reserve_seq();
        self.commit_slot(seq, envelope);
    }

    /// Reserve the next ring sequence without writing a slot.
    ///
    /// Paired with [`FastBroadcastRing::commit_slot`]. Splitting the reservation
    /// from the slot write lets `all`-mode broadcast publish reserve the
    /// sequence under the subscriber-registry lock and perform the (more
    /// expensive) slot write plus waker fan-out after releasing that lock.
    ///
    /// Between a reserve and its matching commit, `write_seq` is advanced but the
    /// slot is not yet written; [`FastBroadcastRing::try_read`] returns
    /// [`BroadcastReadResult::NotReady`] for that window and the reader re-parks
    /// on the waker, which fires from `commit_slot`.
    pub(crate) fn reserve_seq(&self) -> u64 {
        self.write_seq.fetch_add(1, Ordering::Release) + 1
    }

    /// Write the envelope into the slot reserved by [`FastBroadcastRing::reserve_seq`]
    /// and wake parked readers.
    pub(crate) fn commit_slot(&self, seq: u64, envelope: Envelope<T>) {
        let idx = ((seq - 1) as usize) & self.mask;
        *self.slots[idx].lock() = Some((seq, envelope));
        self.waker_set.wake_all();
    }

    pub(crate) fn try_read(&self, read_seq: u64) -> BroadcastReadResult<T> {
        debug_assert!(read_seq > 0);
        let current_write = self.write_seq.load(Ordering::Acquire);
        if read_seq > current_write {
            return BroadcastReadResult::NotReady;
        }

        if current_write >= self.capacity as u64 && read_seq <= current_write - self.capacity as u64
        {
            let new_read_seq = current_write - self.capacity as u64 + 1;
            let missed = new_read_seq - read_seq;
            return BroadcastReadResult::Lagged {
                missed,
                new_read_seq,
            };
        }

        let idx = ((read_seq - 1) as usize) & self.mask;
        let slot = self.slots[idx].lock();
        match &*slot {
            Some((slot_seq, envelope)) if *slot_seq == read_seq => {
                BroadcastReadResult::Ready(envelope.clone())
            }
            Some((slot_seq, _)) if *slot_seq > read_seq => {
                let now = self.write_seq.load(Ordering::Acquire);
                if now >= self.capacity as u64 && read_seq <= now - self.capacity as u64 {
                    let new_read_seq = now - self.capacity as u64 + 1;
                    let missed = new_read_seq - read_seq;
                    BroadcastReadResult::Lagged {
                        missed,
                        new_read_seq,
                    }
                } else {
                    BroadcastReadResult::NotReady
                }
            }
            None => BroadcastReadResult::NotReady,
            Some((_slot_seq, _envelope)) => BroadcastReadResult::NotReady,
        }
    }

    pub(crate) fn current_seq(&self) -> u64 {
        self.write_seq.load(Ordering::Acquire)
    }

    pub(crate) fn register_waker(&self, waker: &Waker) {
        self.waker_set.register(waker);
    }

    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.waker_set.wake_all();
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

pub(crate) enum TopicInner<T: Send + Sync + 'static> {
    BalancedOnly(BalancedOnlyTopic<T>),
    BroadcastOnly(BroadcastOnlyTopic<T>),
    Mixed(MixedTopic<T>),
}

impl<T: Send + Sync + 'static> TopicInner<T> {
    pub(crate) fn new(name: TopicName, opts: TopicOptions) -> Self {
        match opts {
            TopicOptions::BalancedOnly { capacity } => {
                TopicInner::BalancedOnly(BalancedOnlyTopic::new(name, capacity))
            }
            // `all` (consensus) resolution is honored only for broadcast-only
            // topics. `first` keeps the original lock-free behavior.
            TopicOptions::BroadcastOnly {
                capacity,
                on_lag,
                ack_mode,
            } => {
                TopicInner::BroadcastOnly(BroadcastOnlyTopic::new(name, capacity, on_lag, ack_mode))
            }
            // `ack_mode` is intentionally ignored for Mixed: PR3 rejects `all` on
            // any non-broadcast-only topic, so Mixed is always `first`.
            TopicOptions::Mixed {
                balanced_capacity,
                broadcast_capacity,
                on_lag,
                ack_mode: _,
            } => TopicInner::Mixed(MixedTopic::new(
                name,
                balanced_capacity,
                broadcast_capacity,
                on_lag,
            )),
        }
    }
}

impl<T: Send + Sync + 'static> TopicState<T> for TopicInner<T> {
    fn name(&self) -> &TopicName {
        match self {
            TopicInner::BalancedOnly(topic) => &topic.name,
            TopicInner::BroadcastOnly(topic) => &topic.name,
            TopicInner::Mixed(topic) => &topic.name,
        }
    }

    fn publish(&self, msg: Arc<T>) -> PublishFuture<'_> {
        Box::pin(async move {
            match self {
                TopicInner::BalancedOnly(topic) => {
                    let _id = topic.publish(msg).await?;
                    Ok(())
                }
                TopicInner::BroadcastOnly(topic) => {
                    let _id = topic.publish(msg)?;
                    Ok(())
                }
                TopicInner::Mixed(topic) => {
                    let _id = topic.publish(msg).await?;
                    Ok(())
                }
            }
        })
    }

    fn publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> PublishTrackedFuture<'_> {
        Box::pin(async move {
            match self {
                TopicInner::BalancedOnly(topic) => {
                    topic.publish_tracked(msg, timeout, permit).await
                }
                TopicInner::BroadcastOnly(topic) => topic.publish_tracked(msg, timeout, permit),
                TopicInner::Mixed(topic) => topic.publish_tracked(msg, timeout, permit).await,
            }
        })
    }

    fn try_publish(&self, msg: Arc<T>) -> Result<PublishOutcome, Error> {
        match self {
            TopicInner::BalancedOnly(topic) => topic.try_publish(msg).map(|(outcome, _)| outcome),
            TopicInner::BroadcastOnly(topic) => topic.try_publish(msg).map(|(outcome, _)| outcome),
            TopicInner::Mixed(topic) => topic.try_publish(msg).map(|(outcome, _)| outcome),
        }
    }

    fn try_publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> Result<TrackedTryPublishOutcome, Error> {
        match self {
            TopicInner::BalancedOnly(topic) => topic.try_publish_tracked(msg, timeout, permit),
            TopicInner::BroadcastOnly(topic) => topic.try_publish_tracked(msg, timeout, permit),
            TopicInner::Mixed(topic) => topic.try_publish_tracked(msg, timeout, permit),
        }
    }

    fn subscribe_balanced(
        &self,
        group: SubscriptionGroupName,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error> {
        match self {
            TopicInner::BalancedOnly(topic) => topic
                .subscribe_balanced(group, opts)
                .map(|sub| Box::new(sub) as Box<dyn SubscriptionBackend<T>>),
            TopicInner::BroadcastOnly(_) => Err(SubscribeBalancedNotSupported),
            TopicInner::Mixed(topic) => topic
                .subscribe_balanced(group, opts)
                .map(|sub| Box::new(sub) as Box<dyn SubscriptionBackend<T>>),
        }
    }

    fn subscribe_broadcast(
        &self,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error> {
        match self {
            TopicInner::BalancedOnly(_) => Err(SubscribeBroadcastNotSupported),
            TopicInner::BroadcastOnly(topic) => {
                Ok(Box::new(topic.subscribe_broadcast(opts)) as Box<dyn SubscriptionBackend<T>>)
            }
            TopicInner::Mixed(topic) => {
                Ok(Box::new(topic.subscribe_broadcast(opts)) as Box<dyn SubscriptionBackend<T>>)
            }
        }
    }

    fn broadcast_on_lag_policy(&self) -> TopicBroadcastOnLagPolicy {
        match self {
            TopicInner::BalancedOnly(_) => TopicBroadcastOnLagPolicy::DropOldest,
            TopicInner::BroadcastOnly(topic) => topic.broadcast_on_lag,
            TopicInner::Mixed(topic) => topic.broadcast_on_lag,
        }
    }

    fn close(&self) {
        match self {
            TopicInner::BalancedOnly(topic) => topic.close(),
            TopicInner::BroadcastOnly(topic) => topic.close(),
            TopicInner::Mixed(topic) => topic.close(),
        }
    }

    #[cfg(test)]
    fn debug_balanced_available_permits(&self) -> Vec<(SubscriptionGroupName, usize)> {
        match self {
            TopicInner::BalancedOnly(topic) => topic
                .group
                .get()
                .map(|group| {
                    vec![(
                        group.group_name.clone(),
                        group.admission.available_permits(),
                    )]
                })
                .unwrap_or_default(),
            TopicInner::BroadcastOnly(_) => Vec::new(),
            TopicInner::Mixed(topic) => topic.balanced_available_permits(),
        }
    }
}

pub(crate) struct BalancedOnlyTopic<T: Send + Sync + 'static> {
    name: TopicName,
    next_id: AtomicU64,
    group: OnceLock<SingleGroup<T>>,
    balanced_capacity: usize,
    outcomes: TrackedPublishTracker,
    closed: AtomicBool,
}

impl<T: Send + Sync + 'static> BalancedOnlyTopic<T> {
    fn new(name: TopicName, balanced_capacity: usize) -> Self {
        Self {
            name,
            next_id: AtomicU64::new(1),
            group: OnceLock::new(),
            balanced_capacity: balanced_capacity.max(1),
            outcomes: TrackedPublishTracker::new(),
            closed: AtomicBool::new(false),
        }
    }

    #[inline]
    fn next_message_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn publish(&self, msg: Arc<T>) -> Result<u64, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if let Some(group) = self.group.get() {
            let permit = acquire_balanced_permit(&group.admission).await?;
            let envelope = Envelope {
                id,
                tracked: false,
                payload: msg,
            };
            send_queued_envelope(&group.tx, envelope, permit)?;
        }
        Ok(id)
    }

    async fn publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> Result<TrackedPublishReceipt, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if let Some(group) = self.group.get() {
            let admission_permit = acquire_balanced_permit(&group.admission).await?;
            let receipt = self.outcomes.register(id, timeout, permit);
            let envelope = Envelope {
                id,
                tracked: true,
                payload: msg,
            };
            send_queued_envelope(&group.tx, envelope, admission_permit)?;
            Ok(receipt)
        } else {
            Ok(self.outcomes.register(id, timeout, permit))
        }
    }

    fn try_publish(&self, msg: Arc<T>) -> Result<(PublishOutcome, u64), Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if let Some(group) = self.group.get() {
            let Some(permit) = try_acquire_balanced_permit(&group.admission)? else {
                return Ok((PublishOutcome::DroppedOnFull, id));
            };
            let envelope = Envelope {
                id,
                tracked: false,
                payload: msg,
            };
            send_queued_envelope(&group.tx, envelope, permit)?;
        }
        Ok((PublishOutcome::Published, id))
    }

    fn try_publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> Result<TrackedTryPublishOutcome, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if let Some(group) = self.group.get() {
            let Some(admission_permit) = try_acquire_balanced_permit(&group.admission)? else {
                return Ok(TrackedTryPublishOutcome::DroppedOnFull);
            };
            let receipt = self.outcomes.register(id, timeout, permit);
            let envelope = Envelope {
                id,
                tracked: true,
                payload: msg,
            };
            send_queued_envelope(&group.tx, envelope, admission_permit)?;
            Ok(TrackedTryPublishOutcome::Published(receipt))
        } else {
            Ok(TrackedTryPublishOutcome::Published(
                self.outcomes.register(id, timeout, permit),
            ))
        }
    }

    fn subscribe_balanced(
        &self,
        group: SubscriptionGroupName,
        _opts: SubscriberOptions,
    ) -> Result<BalancedSub<T>, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let single_group = self.group.get_or_init(|| {
            let (tx, rx) = async_channel::bounded(self.balanced_capacity);
            SingleGroup {
                group_name: group.clone(),
                tx,
                rx,
                admission: Arc::new(Semaphore::new(self.balanced_capacity)),
            }
        });

        if single_group.group_name != group {
            return Err(SubscribeSingleGroupViolation);
        }

        Ok(BalancedSub {
            rx: Box::pin(single_group.rx.clone()),
            ack_state: AckState::new(self.outcomes.clone()),
        })
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
        self.outcomes.close_all();
        if let Some(group) = self.group.get() {
            group.admission.close();
            let _ = group.tx.close();
        }
    }
}

/// Registry of live broadcast subscribers for a single `all`-mode topic.
///
/// Constructed only when `ack_mode == All`. It is the single linearization
/// point for consensus membership: subscribe, publish, and disconnect all take
/// its lock so a subscriber is either in a message's snapshot AND started at or
/// before that message's ring sequence, or excluded AND started strictly after
/// it. Lock order is always **registry -> tracker**; tracker code must never
/// call back into the registry.
struct BroadcastSubscriberRegistry {
    next_subscriber_id: AtomicU64,
    subscribers: Mutex<Arc<HashSet<BroadcastSubscriberId>>>,
}

impl BroadcastSubscriberRegistry {
    fn new() -> Self {
        Self {
            next_subscriber_id: AtomicU64::new(1),
            subscribers: Mutex::new(Arc::new(HashSet::new())),
        }
    }

    fn allocate_id(&self) -> BroadcastSubscriberId {
        BroadcastSubscriberId(self.next_subscriber_id.fetch_add(1, Ordering::Relaxed))
    }
}

pub(crate) struct BroadcastOnlyTopic<T: Send + Sync + 'static> {
    name: TopicName,
    next_id: AtomicU64,
    broadcast_ring: Arc<FastBroadcastRing<T>>,
    broadcast_on_lag: TopicBroadcastOnLagPolicy,
    outcomes: TrackedPublishTracker,
    closed: AtomicBool,
    ack_mode: TopicBroadcastAckMode,
    /// Subscriber registry, present only in `all` (consensus) mode. `first`
    /// mode never constructs or touches it, so it stays zero-overhead.
    registry: Option<Arc<BroadcastSubscriberRegistry>>,
}

impl<T: Send + Sync + 'static> BroadcastOnlyTopic<T> {
    fn new(
        name: TopicName,
        broadcast_capacity: usize,
        broadcast_on_lag: TopicBroadcastOnLagPolicy,
        ack_mode: TopicBroadcastAckMode,
    ) -> Self {
        let registry = match ack_mode {
            TopicBroadcastAckMode::All => Some(Arc::new(BroadcastSubscriberRegistry::new())),
            TopicBroadcastAckMode::First => None,
        };
        Self {
            name,
            next_id: AtomicU64::new(1),
            broadcast_ring: Arc::new(FastBroadcastRing::new(broadcast_capacity)),
            broadcast_on_lag,
            outcomes: TrackedPublishTracker::new(),
            closed: AtomicBool::new(false),
            ack_mode,
            registry,
        }
    }

    #[inline]
    fn next_message_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn publish(&self, msg: Arc<T>) -> Result<u64, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        self.broadcast_ring.publish(Envelope {
            id,
            tracked: false,
            payload: msg,
        });
        Ok(id)
    }

    fn publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> Result<TrackedPublishReceipt, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();

        // `all` mode: snapshot the eligible subscriber set and reserve the ring
        // sequence atomically under the registry lock so the consensus
        // membership exactly matches who will receive this message. Membership
        // is stored copy-on-write behind an `Arc`, so the snapshot is an O(1)
        // refcount bump rather than a deep copy of the whole set. The heavier
        // slot write plus waker fan-out happen after releasing the lock.
        //
        // Lock order is registry -> tracker (`register_consensus` locks the
        // tracker); the tracker must never call back into the registry.
        if let Some(registry) = &self.registry {
            let subscribers = registry.subscribers.lock();
            let snapshot = Arc::clone(&subscribers);
            let seq = self.broadcast_ring.reserve_seq();
            let receipt = self
                .outcomes
                .register_consensus(id, timeout, permit, snapshot, seq);
            drop(subscribers);
            self.broadcast_ring.commit_slot(
                seq,
                Envelope {
                    id,
                    tracked: true,
                    payload: msg,
                },
            );
            return Ok(receipt);
        }

        // `first` mode: unchanged single-call path, no registry involvement.
        let receipt = self.outcomes.register(id, timeout, permit);
        self.broadcast_ring.publish(Envelope {
            id,
            tracked: true,
            payload: msg,
        });
        Ok(receipt)
    }

    fn try_publish(&self, msg: Arc<T>) -> Result<(PublishOutcome, u64), Error> {
        let id = self.publish(msg)?;
        Ok((PublishOutcome::Published, id))
    }

    fn try_publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> Result<TrackedTryPublishOutcome, Error> {
        let receipt = self.publish_tracked(msg, timeout, permit)?;
        Ok(TrackedTryPublishOutcome::Published(receipt))
    }

    fn subscribe_broadcast(&self, _opts: SubscriberOptions) -> BroadcastSub<T> {
        // `all` mode: allocate this subscriber's id and couple its `start_seq`
        // with insertion into the registry under one lock (the linearization
        // point). `first` mode stays lock-free.
        let (start_seq, subscriber_id) = match &self.registry {
            Some(registry) => {
                let mut subscribers = registry.subscribers.lock();
                let start_seq = self.broadcast_ring.current_seq() + 1;
                let subscriber_id = registry.allocate_id();
                let _ = Arc::make_mut(&mut subscribers).insert(subscriber_id);
                (start_seq, Some(subscriber_id))
            }
            None => (self.broadcast_ring.current_seq() + 1, None),
        };
        BroadcastSub {
            ring: Arc::clone(&self.broadcast_ring),
            cursor: Arc::new(Mutex::new(BroadcastCursor {
                read_seq: start_seq,
                permitted_seq: None,
                disconnected_on_lag: false,
                spun: false,
                permit_waiters: WakerSet::new(),
            })),
            on_lag: self.broadcast_on_lag,
            ack_state: AckState::broadcast(self.outcomes.clone(), self.ack_mode, subscriber_id),
            registry: self.registry.clone(),
            subscriber_id,
            disconnect_done: AtomicBool::new(false),
        }
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
        self.outcomes.close_all();
        self.broadcast_ring.close();
    }
}

pub(crate) struct MixedTopic<T: Send + Sync + 'static> {
    name: TopicName,
    next_id: AtomicU64,
    groups: RwLock<Vec<(SubscriptionGroupName, Arc<ConsumerGroup<T>>)>>,
    group_handles: RwLock<Arc<[Arc<ConsumerGroup<T>>]>>,
    has_balanced_groups: AtomicBool,
    balanced_capacity: usize,
    broadcast_ring: Arc<FastBroadcastRing<T>>,
    broadcast_on_lag: TopicBroadcastOnLagPolicy,
    outcomes: TrackedPublishTracker,
    closed: AtomicBool,
}

impl<T: Send + Sync + 'static> MixedTopic<T> {
    fn new(
        name: TopicName,
        balanced_capacity: usize,
        broadcast_capacity: usize,
        broadcast_on_lag: TopicBroadcastOnLagPolicy,
    ) -> Self {
        Self {
            name,
            next_id: AtomicU64::new(1),
            groups: RwLock::new(Vec::new()),
            group_handles: RwLock::new(Arc::from(Vec::new())),
            has_balanced_groups: AtomicBool::new(false),
            balanced_capacity: balanced_capacity.max(1),
            broadcast_ring: Arc::new(FastBroadcastRing::new(broadcast_capacity)),
            broadcast_on_lag,
            outcomes: TrackedPublishTracker::new(),
            closed: AtomicBool::new(false),
        }
    }

    #[inline]
    fn next_message_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    // Acquire balanced-group capacity atomically from the publisher's point of
    // view. Mixed topics are intentionally all-or-nothing across balanced and
    // broadcast delivery, so publishers must not keep permits reserved in
    // "fast" groups while they wait on a "slow" one. Drop any partial
    // acquisitions before awaiting capacity and retry from a fresh snapshot.
    async fn acquire_balanced_admission(
        &self,
    ) -> Result<(Arc<[Arc<ConsumerGroup<T>>]>, BalancedPermitVec), Error> {
        loop {
            let groups = self.group_handles.read().clone();
            let (permits, blocking_group) = Self::try_acquire_balanced_admission(&groups)?;
            if let Some(group) = blocking_group {
                drop(permits);
                let permit = acquire_balanced_permit(&group.admission).await?;
                drop(permit);
            } else {
                return Ok((groups, permits));
            }
        }
    }

    fn try_acquire_balanced_admission(
        groups: &Arc<[Arc<ConsumerGroup<T>>]>,
    ) -> Result<(BalancedPermitVec, Option<Arc<ConsumerGroup<T>>>), Error> {
        let mut permits = BalancedPermitVec::with_capacity(groups.len());
        for group in groups.as_ref() {
            match try_acquire_balanced_permit(&group.admission)? {
                Some(permit) => permits.push(permit),
                None => return Ok((permits, Some(Arc::clone(group)))),
            }
        }
        Ok((permits, None))
    }

    #[cfg(test)]
    pub(crate) fn balanced_available_permits(&self) -> Vec<(SubscriptionGroupName, usize)> {
        self.groups
            .read()
            .iter()
            .map(|(name, group)| (name.clone(), group.admission.available_permits()))
            .collect()
    }

    async fn publish(&self, msg: Arc<T>) -> Result<u64, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if self.has_balanced_groups.load(Ordering::Acquire) {
            let (groups, permits) = self.acquire_balanced_admission().await?;
            let envelope = Envelope {
                id,
                tracked: false,
                payload: Arc::clone(&msg),
            };
            for (group, permit) in groups.as_ref().iter().zip(permits) {
                send_queued_envelope(&group.tx, envelope.clone(), permit)?;
            }
        }

        // Broadcast is intentionally last so mixed async publish has the same
        // visible contract as mixed try_publish: no broadcast subscriber can
        // observe a message before the balanced side has admitted it.
        self.broadcast_ring.publish(Envelope {
            id,
            tracked: false,
            payload: Arc::clone(&msg),
        });

        Ok(id)
    }

    async fn publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> Result<TrackedPublishReceipt, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if self.has_balanced_groups.load(Ordering::Acquire) {
            let (groups, permits) = self.acquire_balanced_admission().await?;
            // Tracked publish capacity is consumed only after admission
            // succeeds. Waiting on a full balanced group must not hand back a
            // receipt for a message that has not been accepted anywhere yet.
            let receipt = self.outcomes.register(id, timeout, permit);
            let envelope = Envelope {
                id,
                tracked: true,
                payload: Arc::clone(&msg),
            };
            for (group, admission_permit) in groups.as_ref().iter().zip(permits) {
                send_queued_envelope(&group.tx, envelope.clone(), admission_permit)?;
            }
            self.broadcast_ring.publish(Envelope {
                id,
                tracked: true,
                payload: Arc::clone(&msg),
            });
            Ok(receipt)
        } else {
            let receipt = self.outcomes.register(id, timeout, permit);
            self.broadcast_ring.publish(Envelope {
                id,
                tracked: true,
                payload: Arc::clone(&msg),
            });
            Ok(receipt)
        }
    }

    fn try_publish(&self, msg: Arc<T>) -> Result<(PublishOutcome, u64), Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if !self.has_balanced_groups.load(Ordering::Acquire) {
            self.broadcast_ring.publish(Envelope {
                id,
                tracked: false,
                payload: Arc::clone(&msg),
            });
            return Ok((PublishOutcome::Published, id));
        }

        let groups = self.group_handles.read().clone();
        let envelope = Envelope {
            id,
            tracked: false,
            payload: Arc::clone(&msg),
        };
        let (permits, blocking_group) = Self::try_acquire_balanced_admission(&groups)?;
        if blocking_group.is_some() {
            // Keep try_publish all-or-nothing for mixed topics: if balanced
            // admission fails, nothing is published to broadcast either.
            Ok((PublishOutcome::DroppedOnFull, id))
        } else {
            for (group, permit) in groups.as_ref().iter().zip(permits) {
                send_queued_envelope(&group.tx, envelope.clone(), permit)?;
            }
            self.broadcast_ring.publish(Envelope {
                id,
                tracked: false,
                payload: Arc::clone(&msg),
            });
            Ok((PublishOutcome::Published, id))
        }
    }

    fn try_publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> Result<TrackedTryPublishOutcome, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if self.has_balanced_groups.load(Ordering::Acquire) {
            let groups = self.group_handles.read().clone();
            let (permits, blocking_group) = Self::try_acquire_balanced_admission(&groups)?;
            if blocking_group.is_some() {
                return Ok(TrackedTryPublishOutcome::DroppedOnFull);
            }
            let receipt = self.outcomes.register(id, timeout, permit);
            let envelope = Envelope {
                id,
                tracked: true,
                payload: Arc::clone(&msg),
            };
            for (group, admission_permit) in groups.as_ref().iter().zip(permits) {
                send_queued_envelope(&group.tx, envelope.clone(), admission_permit)?;
            }
            self.broadcast_ring.publish(Envelope {
                id,
                tracked: true,
                payload: msg,
            });
            Ok(TrackedTryPublishOutcome::Published(receipt))
        } else {
            let receipt = self.outcomes.register(id, timeout, permit);
            self.broadcast_ring.publish(Envelope {
                id,
                tracked: true,
                payload: msg,
            });
            Ok(TrackedTryPublishOutcome::Published(receipt))
        }
    }

    fn subscribe_balanced(
        &self,
        group: SubscriptionGroupName,
        _opts: SubscriberOptions,
    ) -> Result<BalancedSub<T>, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let rx = {
            let mut groups = self.groups.write();
            if let Some((_name, group_entry)) = groups.iter().find(|(name, _)| *name == group) {
                group_entry.rx.clone()
            } else {
                let (tx, rx) = async_channel::bounded(self.balanced_capacity);
                let group_entry = Arc::new(ConsumerGroup {
                    tx,
                    rx: rx.clone(),
                    admission: Arc::new(Semaphore::new(self.balanced_capacity)),
                });
                groups.push((group.clone(), Arc::clone(&group_entry)));
                let snapshot: Arc<[Arc<ConsumerGroup<T>>]> = groups
                    .iter()
                    .map(|(_, group_entry)| Arc::clone(group_entry))
                    .collect::<Vec<_>>()
                    .into();
                *self.group_handles.write() = snapshot;
                self.has_balanced_groups.store(true, Ordering::Release);
                rx
            }
        };

        Ok(BalancedSub {
            rx: Box::pin(rx),
            ack_state: AckState::new(self.outcomes.clone()),
        })
    }

    fn subscribe_broadcast(&self, _opts: SubscriberOptions) -> BroadcastSub<T> {
        // Mixed topics never run `all` mode (PR3 rejects `all` on non
        // broadcast-only topics), so broadcast subs here are always `first`.
        let start_seq = self.broadcast_ring.current_seq() + 1;
        BroadcastSub {
            ring: Arc::clone(&self.broadcast_ring),
            cursor: Arc::new(Mutex::new(BroadcastCursor {
                read_seq: start_seq,
                permitted_seq: None,
                disconnected_on_lag: false,
                spun: false,
                permit_waiters: WakerSet::new(),
            })),
            on_lag: self.broadcast_on_lag,
            ack_state: AckState::new(self.outcomes.clone()),
            registry: None,
            subscriber_id: None,
            disconnect_done: AtomicBool::new(false),
        }
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
        self.outcomes.close_all();
        let groups = self.group_handles.read().clone();
        for group in groups.as_ref() {
            group.admission.close();
            let _ = group.tx.close();
        }
        self.has_balanced_groups.store(false, Ordering::Release);
        self.broadcast_ring.close();
    }
}

pub(crate) struct BalancedSub<T: Send + Sync + 'static> {
    rx: Pin<Box<async_channel::Receiver<QueuedEnvelope<T>>>>,
    ack_state: AckState,
}

impl<T: Send + Sync + 'static> SubscriptionBackend<T> for BalancedSub<T> {
    fn poll_recv_delivery(&mut self, cx: &mut Context<'_>) -> Poll<Result<RecvDelivery<T>, Error>> {
        match self.rx.as_mut().poll_next(cx) {
            Poll::Ready(Some(queued)) => {
                Poll::Ready(Ok(RecvDelivery::Message(Delivery::new_in_memory(
                    queued.envelope,
                    InMemoryDeliveryFinalizer::balanced(self.ack_state.clone()),
                ))))
            }
            Poll::Ready(None) => Poll::Ready(Err(SubscriptionClosed)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn ack(&self, id: u64) -> Result<(), Error> {
        self.ack_state.send_ack(id)
    }

    fn nack(&self, id: u64, reason: Arc<str>) -> Result<(), Error> {
        self.ack_state.send_nack(id, reason)
    }
}

pub(crate) struct BroadcastSub<T: Send + Sync + 'static> {
    ring: Arc<FastBroadcastRing<T>>,
    cursor: Arc<Mutex<BroadcastCursor>>,
    on_lag: TopicBroadcastOnLagPolicy,
    ack_state: AckState,
    /// Subscriber registry, present only in `all` mode. Used to remove this
    /// subscriber and nack any messages it still owes on disconnect/drop.
    registry: Option<Arc<BroadcastSubscriberRegistry>>,
    /// This subscriber's consensus identity, present only in `all` mode.
    subscriber_id: Option<BroadcastSubscriberId>,
    /// Guards the disconnect path so it runs at most once (lag then drop, etc.).
    disconnect_done: AtomicBool,
}

impl<T: Send + Sync + 'static> SubscriptionBackend<T> for BroadcastSub<T> {
    fn poll_recv_delivery(&mut self, cx: &mut Context<'_>) -> Poll<Result<RecvDelivery<T>, Error>> {
        {
            let cursor = self.cursor.lock();
            if cursor.disconnected_on_lag {
                return Poll::Ready(Err(SubscriptionClosed));
            }
            if cursor.permitted_seq.is_some() {
                cursor.permit_waiters.register(cx.waker());
                return Poll::Pending;
            }
        }

        if let Some(result) = self.try_read_delivery() {
            return Poll::Ready(result);
        }

        let should_spin = {
            let mut cursor = self.cursor.lock();
            if cursor.spun {
                false
            } else {
                cursor.spun = true;
                true
            }
        };

        if should_spin {
            for _ in 0..32 {
                std::hint::spin_loop();
                if let Some(result) = self.try_read_delivery() {
                    return Poll::Ready(result);
                }
            }
        }

        self.ring.register_waker(cx.waker());
        {
            let cursor = self.cursor.lock();
            if cursor.permitted_seq.is_some() {
                cursor.permit_waiters.register(cx.waker());
                return Poll::Pending;
            }
        }

        match self.try_read_delivery() {
            Some(result) => Poll::Ready(result),
            None if self.ring.is_closed() => Poll::Ready(Err(SubscriptionClosed)),
            None => Poll::Pending,
        }
    }

    fn ack(&self, id: u64) -> Result<(), Error> {
        self.ack_state.send_ack(id)
    }

    fn nack(&self, id: u64, reason: Arc<str>) -> Result<(), Error> {
        self.ack_state.send_nack(id, reason)
    }
}

impl<T: Send + Sync + 'static> BroadcastSub<T> {
    fn try_read_delivery(&self) -> Option<Result<RecvDelivery<T>, Error>> {
        let read_seq = {
            let cursor = self.cursor.lock();
            if cursor.disconnected_on_lag || cursor.permitted_seq.is_some() {
                return None;
            }
            cursor.read_seq
        };

        match self.ring.try_read(read_seq) {
            BroadcastReadResult::Ready(envelope) => {
                let mut cursor = self.cursor.lock();
                if cursor.disconnected_on_lag || cursor.permitted_seq.is_some() {
                    return None;
                }
                cursor.permitted_seq = Some(read_seq);
                cursor.spun = false;
                Some(Ok(RecvDelivery::Message(Delivery::new_in_memory(
                    envelope,
                    InMemoryDeliveryFinalizer::broadcast(
                        read_seq,
                        Arc::clone(&self.cursor),
                        self.ack_state.clone(),
                    ),
                ))))
            }
            BroadcastReadResult::Lagged {
                missed,
                new_read_seq,
            } => {
                let disconnect = {
                    let mut cursor = self.cursor.lock();
                    cursor.read_seq = new_read_seq;
                    cursor.spun = false;
                    if self.on_lag == TopicBroadcastOnLagPolicy::Disconnect {
                        cursor.disconnected_on_lag = true;
                        true
                    } else {
                        false
                    }
                };
                if disconnect {
                    // In `all` mode this nacks any messages this subscriber
                    // still owes an ack for. `first` mode is a no-op.
                    self.disconnect(Arc::<str>::from(
                        "broadcast subscriber disconnected after lag",
                    ));
                } else {
                    // `DropOldest`: the subscriber stays connected but the
                    // skipped messages (ring sequence < new_read_seq) were
                    // overwritten before it read them, so in `all` mode it can
                    // never ack them. Nack exactly those owed entries; messages
                    // at or after new_read_seq remain readable. No-op in `first`
                    // mode.
                    self.nack_skipped_on_lag(
                        new_read_seq,
                        Arc::<str>::from("broadcast subscriber lagged past message"),
                    );
                }
                Some(Ok(RecvDelivery::Lagged { missed }))
            }
            BroadcastReadResult::NotReady => None,
        }
    }

    /// Remove this subscriber from the registry and nack every consensus
    /// (`all`-mode) message it still owes an ack for. No-op in `first` mode and
    /// idempotent across repeated calls (lag-disconnect followed by drop).
    ///
    /// Lock order is registry -> tracker: the registry lock is held across the
    /// removal and the tracker scan so a concurrent publish cannot add this
    /// subscriber to a new consensus entry after it has been removed.
    fn disconnect(&self, reason: Arc<str>) {
        let (Some(registry), Some(subscriber_id)) = (&self.registry, self.subscriber_id) else {
            return;
        };
        if self.disconnect_done.swap(true, Ordering::AcqRel) {
            return;
        }
        let mut subscribers = registry.subscribers.lock();
        let _ = Arc::make_mut(&mut subscribers).remove(&subscriber_id);
        self.ack_state
            .outcomes
            .nack_pending_for_subscriber(subscriber_id, reason);
        drop(subscribers);
    }

    /// Nack the `all`-mode consensus messages this subscriber skipped after a
    /// `DropOldest` lag (publish sequence `< new_read_seq`) without disconnecting
    /// it. No-op in `first` mode.
    ///
    /// The registry lock is held across the tracker scan (registry -> tracker,
    /// matching [`Self::disconnect`] and `publish_tracked`) so a consensus entry
    /// with sequence `< new_read_seq` that is concurrently being registered
    /// cannot be missed: its publish holds the registry lock until registration
    /// completes.
    fn nack_skipped_on_lag(&self, new_read_seq: u64, reason: Arc<str>) {
        let (Some(registry), Some(subscriber_id)) = (&self.registry, self.subscriber_id) else {
            return;
        };
        let subscribers = registry.subscribers.lock();
        self.ack_state
            .outcomes
            .nack_owed_before(subscriber_id, new_read_seq, reason);
        drop(subscribers);
    }
}

impl<T: Send + Sync + 'static> Drop for BroadcastSub<T> {
    fn drop(&mut self) {
        self.disconnect(Arc::<str>::from("broadcast subscriber dropped"));
    }
}

pub(crate) struct BroadcastCursor {
    read_seq: u64,
    permitted_seq: Option<u64>,
    disconnected_on_lag: bool,
    spun: bool,
    permit_waiters: WakerSet,
}

fn finish_broadcast_delivery(cursor: &Arc<Mutex<BroadcastCursor>>, seq: u64) {
    let mut cursor = cursor.lock();
    if cursor.permitted_seq == Some(seq) {
        cursor.read_seq = seq + 1;
        cursor.permitted_seq = None;
        cursor.spun = false;
        cursor.permit_waiters.wake_all();
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InMemoryDeliveryKind {
    Balanced,
    Broadcast,
}

pub(crate) enum InMemoryDeliveryFinalizer {
    Balanced {
        ack_state: AckState,
    },
    Broadcast {
        seq: u64,
        cursor: Arc<Mutex<BroadcastCursor>>,
        ack_state: AckState,
    },
}

impl InMemoryDeliveryFinalizer {
    pub(crate) fn balanced(ack_state: AckState) -> Self {
        Self::Balanced { ack_state }
    }

    pub(crate) fn broadcast(
        seq: u64,
        cursor: Arc<Mutex<BroadcastCursor>>,
        ack_state: AckState,
    ) -> Self {
        Self::Broadcast {
            seq,
            cursor,
            ack_state,
        }
    }

    pub(crate) fn commit(&mut self) {
        if let Self::Broadcast { seq, cursor, .. } = self {
            finish_broadcast_delivery(cursor, *seq);
        }
    }

    pub(crate) fn abort<T: Send + Sync + 'static>(
        &mut self,
        envelope: &Envelope<T>,
        reason: Arc<str>,
    ) -> Result<(), Error> {
        match self {
            Self::Balanced { ack_state } => {
                if envelope.tracked {
                    _ = ack_state.send_nack(envelope.id, reason);
                }
            }
            Self::Broadcast {
                seq,
                cursor,
                ack_state,
            } => {
                if envelope.tracked {
                    _ = ack_state.send_nack(envelope.id, reason);
                }
                finish_broadcast_delivery(cursor, *seq);
            }
        }
        Ok(())
    }

    pub(crate) fn abandon<T: Send + Sync + 'static>(&mut self, envelope: &Envelope<T>) {
        match self {
            Self::Balanced { ack_state } => {
                if envelope.tracked {
                    _ = ack_state.send_nack(
                        envelope.id,
                        Arc::<str>::from("topic delivery abandoned before forward"),
                    );
                }
            }
            Self::Broadcast {
                seq,
                cursor,
                ack_state,
            } => {
                if envelope.tracked {
                    _ = ack_state.send_nack(
                        envelope.id,
                        Arc::<str>::from("topic delivery abandoned before forward"),
                    );
                }
                finish_broadcast_delivery(cursor, *seq);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn kind(&self) -> InMemoryDeliveryKind {
        match self {
            Self::Balanced { .. } => InMemoryDeliveryKind::Balanced,
            Self::Broadcast { .. } => InMemoryDeliveryKind::Broadcast,
        }
    }
}

#[derive(Clone)]
pub(crate) struct AckState {
    pub(crate) outcomes: TrackedPublishTracker,
    /// Ack-resolution mode for this subscriber. `First` (the default for
    /// balanced and `first`-mode broadcast) resolves on the first Ack/Nack;
    /// `All` routes acks through per-subscriber consensus aggregation.
    ack_mode: TopicBroadcastAckMode,
    /// Consensus identity, present only for `all`-mode broadcast subscribers.
    subscriber_id: Option<BroadcastSubscriberId>,
}

impl AckState {
    /// `AckState` for balanced subscribers and `first`-mode broadcast: the
    /// first terminal Ack/Nack resolves the tracked publish.
    pub(crate) fn new(outcomes: TrackedPublishTracker) -> Self {
        Self {
            outcomes,
            ack_mode: TopicBroadcastAckMode::First,
            subscriber_id: None,
        }
    }

    /// `AckState` for a broadcast subscriber. In `all` mode (`subscriber_id` is
    /// `Some`) acks are aggregated per subscriber; in `first` mode it behaves
    /// like [`AckState::new`].
    pub(crate) fn broadcast(
        outcomes: TrackedPublishTracker,
        ack_mode: TopicBroadcastAckMode,
        subscriber_id: Option<BroadcastSubscriberId>,
    ) -> Self {
        Self {
            outcomes,
            ack_mode,
            subscriber_id,
        }
    }

    pub(crate) fn send_ack(&self, message_id: u64) -> Result<(), Error> {
        // `all`-mode broadcast: contribute this subscriber's Ack to the
        // consensus. The message stays tracked (Ok) until the set completes.
        if let (TopicBroadcastAckMode::All, Some(subscriber_id)) =
            (self.ack_mode, self.subscriber_id)
        {
            return match self.outcomes.resolve_ack_from(message_id, subscriber_id) {
                AckFromResult::Resolved | AckFromResult::StillPending => Ok(()),
                AckFromResult::NotTracked => Err(MessageNotTracked),
            };
        }

        if self
            .outcomes
            .resolve(message_id, TrackedPublishOutcome::Ack)
        {
            Ok(())
        } else {
            Err(MessageNotTracked)
        }
    }

    pub(crate) fn send_nack(
        &self,
        message_id: u64,
        reason: impl Into<Arc<str>>,
    ) -> Result<(), Error> {
        let reason = reason.into();

        // `all`-mode broadcast: only a subscriber that still requires the message
        // may fail the consensus. A Nack from a subscriber that has already acked
        // (or was never eligible) is a no-op, mirroring the disappearance handling
        // and keeping outcomes insensitive to accidental double-signalling.
        if let (TopicBroadcastAckMode::All, Some(subscriber_id)) =
            (self.ack_mode, self.subscriber_id)
        {
            return match self
                .outcomes
                .resolve_nack_from(message_id, subscriber_id, reason)
            {
                NackFromResult::Resolved | NackFromResult::NotRequired => Ok(()),
                NackFromResult::NotTracked => Err(MessageNotTracked),
            };
        }

        if self
            .outcomes
            .resolve(message_id, TrackedPublishOutcome::Nack { reason })
        {
            Ok(())
        } else {
            Err(MessageNotTracked)
        }
    }
}
