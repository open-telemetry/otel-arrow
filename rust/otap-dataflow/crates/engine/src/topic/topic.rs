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
    Envelope, PublishOutcome, SubscriberOptions, TopicOptions, TrackedPublishOutcome,
    TrackedPublishPermit, TrackedPublishReceipt, TrackedPublishTracker, TrackedTryPublishOutcome,
};
use futures_core::Stream;
use otap_df_config::topic::TopicBroadcastOnLagPolicy;
use otap_df_config::{SubscriptionGroupName, TopicName};
use parking_lot::{Mutex, RwLock};
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

    pub(crate) fn publish(&self, envelope: Envelope<T>) {
        let seq = self.write_seq.fetch_add(1, Ordering::Release) + 1;
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
            TopicOptions::BroadcastOnly { capacity, on_lag } => {
                TopicInner::BroadcastOnly(BroadcastOnlyTopic::new(name, capacity, on_lag))
            }
            TopicOptions::Mixed {
                balanced_capacity,
                broadcast_capacity,
                on_lag,
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
            ack_state: AckState {
                outcomes: self.outcomes.clone(),
            },
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

pub(crate) struct BroadcastOnlyTopic<T: Send + Sync + 'static> {
    name: TopicName,
    next_id: AtomicU64,
    broadcast_ring: Arc<FastBroadcastRing<T>>,
    broadcast_on_lag: TopicBroadcastOnLagPolicy,
    outcomes: TrackedPublishTracker,
    closed: AtomicBool,
}

impl<T: Send + Sync + 'static> BroadcastOnlyTopic<T> {
    fn new(
        name: TopicName,
        broadcast_capacity: usize,
        broadcast_on_lag: TopicBroadcastOnLagPolicy,
    ) -> Self {
        Self {
            name,
            next_id: AtomicU64::new(1),
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
        let start_seq = self.broadcast_ring.current_seq() + 1;
        BroadcastSub {
            ring: Arc::clone(&self.broadcast_ring),
            cursor: Arc::new(Mutex::new(BroadcastCursor {
                read_seq: start_seq,
                leased_seq: None,
                disconnected_on_lag: false,
                spun: false,
                lease_waiters: WakerSet::new(),
            })),
            on_lag: self.broadcast_on_lag,
            ack_state: AckState {
                outcomes: self.outcomes.clone(),
            },
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

    async fn publish(&self, msg: Arc<T>) -> Result<u64, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let id = self.next_message_id();
        if self.has_balanced_groups.load(Ordering::Acquire) {
            let groups = self.group_handles.read().clone();
            let mut permits = Vec::with_capacity(groups.len());
            for group in groups.as_ref() {
                permits.push(acquire_balanced_permit(&group.admission).await?);
            }
            let envelope = Envelope {
                id,
                tracked: false,
                payload: Arc::clone(&msg),
            };
            for (group, permit) in groups.as_ref().iter().zip(permits.into_iter()) {
                send_queued_envelope(&group.tx, envelope.clone(), permit)?;
            }
        }

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
            let groups = self.group_handles.read().clone();
            let mut permits = Vec::with_capacity(groups.len());
            for group in groups.as_ref() {
                permits.push(acquire_balanced_permit(&group.admission).await?);
            }
            let receipt = self.outcomes.register(id, timeout, permit);
            let envelope = Envelope {
                id,
                tracked: true,
                payload: Arc::clone(&msg),
            };
            for (group, admission_permit) in groups.as_ref().iter().zip(permits.into_iter()) {
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
        let mut dropped_on_full = false;
        let mut permits = Vec::with_capacity(groups.len());
        for group in groups.as_ref() {
            match try_acquire_balanced_permit(&group.admission)? {
                Some(permit) => permits.push(permit),
                None => {
                    dropped_on_full = true;
                    break;
                }
            }
        }

        if dropped_on_full {
            Ok((PublishOutcome::DroppedOnFull, id))
        } else {
            for (group, permit) in groups.as_ref().iter().zip(permits.into_iter()) {
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
            let mut permits = Vec::with_capacity(groups.len());
            for group in groups.as_ref() {
                match try_acquire_balanced_permit(&group.admission)? {
                    Some(permit) => permits.push(permit),
                    None => return Ok(TrackedTryPublishOutcome::DroppedOnFull),
                }
            }
            let receipt = self.outcomes.register(id, timeout, permit);
            let envelope = Envelope {
                id,
                tracked: true,
                payload: Arc::clone(&msg),
            };
            for (group, admission_permit) in groups.as_ref().iter().zip(permits.into_iter()) {
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
            ack_state: AckState {
                outcomes: self.outcomes.clone(),
            },
        })
    }

    fn subscribe_broadcast(&self, _opts: SubscriberOptions) -> BroadcastSub<T> {
        let start_seq = self.broadcast_ring.current_seq() + 1;
        BroadcastSub {
            ring: Arc::clone(&self.broadcast_ring),
            cursor: Arc::new(Mutex::new(BroadcastCursor {
                read_seq: start_seq,
                leased_seq: None,
                disconnected_on_lag: false,
                spun: false,
                lease_waiters: WakerSet::new(),
            })),
            on_lag: self.broadcast_on_lag,
            ack_state: AckState {
                outcomes: self.outcomes.clone(),
            },
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
}

impl<T: Send + Sync + 'static> SubscriptionBackend<T> for BroadcastSub<T> {
    fn poll_recv_delivery(&mut self, cx: &mut Context<'_>) -> Poll<Result<RecvDelivery<T>, Error>> {
        {
            let cursor = self.cursor.lock();
            if cursor.disconnected_on_lag {
                return Poll::Ready(Err(SubscriptionClosed));
            }
            if cursor.leased_seq.is_some() {
                cursor.lease_waiters.register(cx.waker());
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
            if cursor.leased_seq.is_some() {
                cursor.lease_waiters.register(cx.waker());
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
            if cursor.disconnected_on_lag || cursor.leased_seq.is_some() {
                return None;
            }
            cursor.read_seq
        };

        match self.ring.try_read(read_seq) {
            BroadcastReadResult::Ready(envelope) => {
                let mut cursor = self.cursor.lock();
                if cursor.disconnected_on_lag || cursor.leased_seq.is_some() {
                    return None;
                }
                cursor.leased_seq = Some(read_seq);
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
                let mut cursor = self.cursor.lock();
                cursor.read_seq = new_read_seq;
                cursor.spun = false;
                if self.on_lag == TopicBroadcastOnLagPolicy::Disconnect {
                    cursor.disconnected_on_lag = true;
                }
                Some(Ok(RecvDelivery::Lagged { missed }))
            }
            BroadcastReadResult::NotReady => None,
        }
    }
}

pub(crate) struct BroadcastCursor {
    read_seq: u64,
    leased_seq: Option<u64>,
    disconnected_on_lag: bool,
    spun: bool,
    lease_waiters: WakerSet,
}

fn finish_broadcast_delivery(cursor: &Arc<Mutex<BroadcastCursor>>, seq: u64) {
    let mut cursor = cursor.lock();
    if cursor.leased_seq == Some(seq) {
        cursor.read_seq = seq + 1;
        cursor.leased_seq = None;
        cursor.spun = false;
        cursor.lease_waiters.wake_all();
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
}

impl AckState {
    pub(crate) fn send_ack(&self, message_id: u64) -> Result<(), Error> {
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
        if self.outcomes.resolve(
            message_id,
            TrackedPublishOutcome::Nack {
                reason: reason.into(),
            },
        ) {
            Ok(())
        } else {
            Err(MessageNotTracked)
        }
    }
}
