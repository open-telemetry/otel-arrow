// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic internals -- the performance-critical core of the crate.
//!
//! This is the largest module. It contains several cooperating internal types,
//! all `pub(crate)`. Nothing here is directly exposed to users.
//!
//! # TopicInner -- Enum Dispatch
//!
//! `TopicInner<T>` is an enum over three specialized topic implementations,
//! selected at creation time by `TopicMode`. Enum dispatch (not `dyn Trait`)
//! is used because `publish()` is async -- a trait object would require
//! `Box<dyn Future>` per call, adding an allocation on the hottest path. The
//! enum keeps future sizes known at compile time and enables inlining.
//!
//! # Three Topic Variants
//!
//! Each variant pays only for the delivery mechanisms it needs:
//!
//! - **`BalancedOnlyTopic`**: Uses `OnceLock<SingleGroup>` for lazy, lock-free
//!   group initialization. Only one consumer group is allowed (enforced at
//!   subscribe time via `SingleGroupViolation`). No ring buffer allocated.
//!
//! - **`BroadcastOnlyTopic`**: Uses `FastBroadcastRing` only. No `async_channel`
//!   allocated. `publish()` is synchronous (no `.await`), making it the fastest
//!   variant.
//!
//! - **`MixedTopic`**: Supports both modes. Every publish writes to all balanced
//!   groups AND the broadcast ring. The groups list uses `RwLock<Vec<...>>` --
//!   subscribe takes a write lock (rare), publish takes a short read lock to
//!   clone the senders, then drops the guard before any `.await` to keep the
//!   future `Send`.
//!
//! # FastBroadcastRing
//!
//! A power-of-two ring buffer with drop-oldest semantics. Key design choices:
//!
//! - **Per-slot `parking_lot::Mutex`** (not `RwLock`): readers hold the lock
//!   only for `Arc::clone` (~nanoseconds). `Mutex` has lower uncontended
//!   overhead than `RwLock`.
//! - **Bitmask indexing**: `(seq - 1) & mask` instead of modulo. Capacity is
//!   always rounded up to the next power of two.
//! - **Two publish methods**: `publish()` auto-assigns IDs (used by
//!   `BroadcastOnlyTopic`), `publish_with_id()` accepts caller-provided IDs
//!   (used by `MixedTopic` which shares a single ID sequence across both
//!   balanced channels and the broadcast ring).
//! - **Lag detection**: `try_read()` compares the subscriber's `read_seq`
//!   against `write_seq` to detect when the subscriber has fallen behind by
//!   more than the buffer capacity. It returns `Lagged { missed, new_read_seq }`
//!   so the subscriber can skip ahead.
//!
//! # WakerSet
//!
//! Replaces `tokio::sync::Notify` with a minimal implementation. The key
//! optimization is the `has_waiters` atomic: `wake_all()` skips the Mutex
//! entirely when no subscribers are blocked, which is the common case in
//! high-throughput scenarios. `register()` deduplicates wakers via
//! `Waker::will_wake()` to prevent unbounded `Vec` growth.
//!
//! # PublisherRegistry
//!
//! Maps publisher_id (u16) -> `mpsc::Sender<AckEvent>`. IDs start at 1;
//! publisher_id 0 means "no ack sender registered" (returns `NotEnabled`).
//! Routing logic: extract publisher_id from message ID bit-packing, look up
//! the per-publisher sender; if not found, return `NotEnabled`. Uses
//! `try_send` (non-blocking) to avoid backpressure from slow ack consumers.
//!
//! # AckState
//!
//! Thin wrapper held by `Subscription`. Delegates to
//! `PublisherRegistry::route_ack()`. This separation keeps ack logic out of
//! the subscription receive path.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll, Waker};

use crate::error::Error;
use crate::error::Error::{
    AckChannelClosed, AckChannelFull, AckNotEnabled, SubscribeBalancedNotSupported,
    SubscribeBroadcastNotSupported, SubscribeSingleGroupViolation, SubscriptionClosed, TopicClosed,
};
use crate::topic::backend::{PublishFuture, SubscriptionBackend, TopicState};
use crate::topic::types::{
    AckEvent, AckStatus, Envelope, RecvItem, SubscriberOptions, TopicMode, TopicOptions, message_id,
};
use futures_core::Stream;
use otap_df_config::TopicName;
use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc;
// ---------------------------------------------------------------------------
// Internal: consumer group (balanced mode)
// ---------------------------------------------------------------------------

/// A bounded async_channel pair for one consumer group (used by MixedTopic).
/// The tx side is cloned per-publish (under a short read lock), the rx side is
/// cloned per-subscriber. async_channel handles the MPMC fan-out internally.
struct ConsumerGroup<T> {
    tx: async_channel::Sender<Envelope<T>>,
    rx: async_channel::Receiver<Envelope<T>>,
}

// ---------------------------------------------------------------------------
// Internal: single consumer group for BalancedOnly mode
// ---------------------------------------------------------------------------

/// Like ConsumerGroup but also stores the group name for SingleGroupViolation
/// enforcement. Stored inside OnceLock -- initialized lazily on first subscribe,
/// lock-free on subsequent subscribes.
struct SingleGroup<T> {
    group_name: Arc<str>,
    tx: async_channel::Sender<Envelope<T>>,
    rx: async_channel::Receiver<Envelope<T>>,
}

pub(crate) enum BroadcastReadResult<T> {
    Ready(Envelope<T>),
    NotReady,
    Lagged { missed: u64, new_read_seq: u64 },
}

// ---------------------------------------------------------------------------
// WakerSet — lightweight alternative to tokio::sync::Notify
// ---------------------------------------------------------------------------

/// Lightweight multi-waker notification. `has_waiters` is the key optimization:
/// `wake_all()` skips the Mutex entirely when no subscribers are blocked (the
/// common case in high-throughput scenarios). `register()` deduplicates via
/// `Waker::will_wake()` to prevent unbounded Vec growth when a subscriber is
/// polled repeatedly between publishes.
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
        // Deduplicate: if this waker is already registered, replace it in place
        // instead of pushing a duplicate. Prevents unbounded growth when a
        // subscriber is polled multiple times between publishes.
        for existing in wakers.iter_mut() {
            if existing.will_wake(waker) {
                existing.clone_from(waker);
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
        let wakers = std::mem::take(&mut *self.wakers.lock());
        self.has_waiters.store(false, Ordering::Release);
        for waker in wakers {
            waker.wake();
        }
    }
}

// ---------------------------------------------------------------------------
// PublisherRegistry — maps publisher IDs to per-publisher ack senders
// ---------------------------------------------------------------------------

/// Routes ack/nack events to the correct channel based on publisher_id.
///
/// IDs start at 1 (via `next_id`); publisher_id 0 means "no ack sender
/// registered" (returns `NotEnabled`). The entries Vec is small (one entry
/// per `with_ack_sender()` call) so linear scan is fine.
///
/// All sends use `try_send` (non-blocking) to avoid backpressure from slow
/// ack consumers.
pub(crate) struct PublisherRegistry {
    topic_name: TopicName,
    entries: Mutex<Vec<(u16, mpsc::Sender<AckEvent>)>>,
    next_id: AtomicU16,
}

impl PublisherRegistry {
    fn new(topic_name: TopicName) -> Self {
        Self {
            topic_name,
            // Start at 1 so publisher_id=0 means "no ack sender registered".
            next_id: AtomicU16::new(1),
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Register a per-publisher ack sender. Returns the assigned publisher_id.
    pub(crate) fn register(&self, sender: mpsc::Sender<AckEvent>) -> u16 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.entries.lock().push((id, sender));
        id
    }

    /// Route an ack/nack to the correct sender based on the publisher_id
    /// encoded in the message ID.
    pub(crate) fn route_ack(
        &self,
        message_id: u64,
        status: AckStatus,
        reason: Option<Arc<str>>,
    ) -> Result<(), Error> {
        let pub_id = message_id::publisher_id(message_id);

        if pub_id == 0 {
            return Err(AckNotEnabled);
        }

        let entries = self.entries.lock();
        match entries.iter().find(|(id, _)| *id == pub_id) {
            Some((_, sender)) => {
                let event = AckEvent {
                    topic: self.topic_name.clone(),
                    message_id,
                    status,
                    reason,
                    publisher_id: pub_id,
                };
                Self::try_send(sender, event)
            }
            None => Err(AckNotEnabled),
        }
    }

    fn try_send(sender: &mpsc::Sender<AckEvent>, event: AckEvent) -> Result<(), Error> {
        match sender.try_send(event) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => Err(AckChannelFull),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(AckChannelClosed),
        }
    }
}

// ---------------------------------------------------------------------------
// FastBroadcastRing — power-of-two ring buffer with drop-oldest semantics
// ---------------------------------------------------------------------------

/// A fixed-capacity ring buffer for broadcast delivery.
///
/// The capacity is always rounded up to the next power of two to enable
/// bitmask indexing (`(seq - 1) & mask`). Each slot is independently locked
/// with a `parking_lot::Mutex` -- readers hold the lock only long enough to
/// `Arc::clone` the payload.
///
/// Two publish methods exist:
/// - `publish()`: auto-assigns sequential IDs (used by `BroadcastOnlyTopic`).
/// - `publish_with_id()`: accepts caller-provided IDs (used by `MixedTopic`
///   which shares a single ID sequence across balanced channels and the ring).
///
/// Lag detection: `try_read()` compares the subscriber's `read_seq` against
/// `write_seq - capacity` to detect overwritten slots.
pub(crate) struct FastBroadcastRing<T: Send + Sync + 'static> {
    /// Ring buffer slots. Each slot holds (sequence_number, payload).
    /// Uses Mutex instead of RwLock for lower uncontended overhead.
    slots: Box<[Mutex<Option<(u64, Arc<T>)>>]>,
    capacity: usize,
    /// Bitmask for power-of-two indexing: `seq & mask` instead of `seq % capacity`.
    mask: usize,
    /// Current write sequence (0 = no messages yet, 1 = first message written).
    /// Also serves as the message ID, eliminating a separate atomic.
    write_seq: AtomicU64,
    /// WakerSet-based wakeup — atomic fast-check avoids Mutex when no waiters.
    waker_set: WakerSet,
    /// Closed flag — subscribers check this when woken.
    closed: AtomicBool,
}

impl<T: Send + Sync + 'static> FastBroadcastRing<T> {
    fn new(capacity: usize) -> Self {
        // Round up to next power of two (minimum 2).
        let cap = capacity.max(2).next_power_of_two();
        let mask = cap - 1;
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

    /// Write a message with a caller-provided ID (for MixedTopic which shares
    /// IDs across balanced channels and the broadcast ring).
    pub(crate) fn publish_with_id(&self, id: u64, payload: Arc<T>) {
        let seq = self.write_seq.fetch_add(1, Ordering::Release) + 1;
        let idx = ((seq - 1) as usize) & self.mask;
        *self.slots[idx].lock() = Some((id, payload));
        self.waker_set.wake_all();
    }

    /// Try to read a message at the given subscriber read_seq.
    pub(crate) fn try_read(&self, read_seq: u64) -> BroadcastReadResult<T> {
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
            Some((id, payload)) => BroadcastReadResult::Ready(Envelope {
                id: *id,
                payload: Arc::clone(payload),
            }),
            None => BroadcastReadResult::NotReady,
        }
    }

    /// Get current write_seq so new subscribers start from the right position.
    pub(crate) fn current_seq(&self) -> u64 {
        self.write_seq.load(Ordering::Acquire)
    }

    /// Register a waker to be notified when new data is published.
    pub(crate) fn register_waker(&self, waker: &Waker) {
        self.waker_set.register(waker);
    }

    /// Close the ring: mark closed and wake all subscribers.
    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.waker_set.wake_all();
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

// ---------------------------------------------------------------------------
// TopicInner — enum-dispatched topic implementations
// ---------------------------------------------------------------------------

/// The core topic type, dispatched by `TopicMode`. All methods delegate to
/// the active variant. The enum match arms are trivial forwarding -- the real
/// logic lives in each variant struct below.
pub(crate) enum TopicInner<T: Send + Sync + 'static> {
    BalancedOnly(BalancedOnlyTopic<T>),
    BroadcastOnly(BroadcastOnlyTopic<T>),
    Mixed(MixedTopic<T>),
}

impl<T: Send + Sync + 'static> TopicInner<T> {
    pub(crate) fn new(name: TopicName, opts: TopicOptions) -> Self {
        match opts.mode {
            TopicMode::BalancedOnly => TopicInner::BalancedOnly(BalancedOnlyTopic::new(name, opts)),
            TopicMode::BroadcastOnly => {
                TopicInner::BroadcastOnly(BroadcastOnlyTopic::new(name, opts))
            }
            TopicMode::Mixed => TopicInner::Mixed(MixedTopic::new(name, opts)),
        }
    }

    fn registry(&self) -> &Arc<PublisherRegistry> {
        match self {
            TopicInner::BalancedOnly(t) => &t.registry,
            TopicInner::BroadcastOnly(t) => &t.registry,
            TopicInner::Mixed(t) => &t.registry,
        }
    }
}

impl<T: Send + Sync + 'static> TopicState<T> for TopicInner<T> {
    fn name(&self) -> &TopicName {
        match self {
            TopicInner::BalancedOnly(t) => &t.name,
            TopicInner::BroadcastOnly(t) => &t.name,
            TopicInner::Mixed(t) => &t.name,
        }
    }

    fn publish(&self, publisher_id: u16, msg: Arc<T>) -> PublishFuture<'_> {
        Box::pin(async move {
            match self {
                TopicInner::BalancedOnly(t) => t.publish(publisher_id, msg).await,
                TopicInner::BroadcastOnly(t) => t.publish(publisher_id, msg),
                TopicInner::Mixed(t) => t.publish(publisher_id, msg).await,
            }
        })
    }

    fn subscribe_balanced(
        &self,
        group: Arc<str>,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error> {
        match self {
            TopicInner::BalancedOnly(t) => t
                .subscribe_balanced(group, opts)
                .map(|sub| Box::new(sub) as Box<dyn SubscriptionBackend<T>>),
            TopicInner::BroadcastOnly(_) => Err(SubscribeBalancedNotSupported),
            TopicInner::Mixed(t) => t
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
            TopicInner::BroadcastOnly(t) => {
                Ok(Box::new(t.subscribe_broadcast(opts)) as Box<dyn SubscriptionBackend<T>>)
            }
            TopicInner::Mixed(t) => {
                Ok(Box::new(t.subscribe_broadcast(opts)) as Box<dyn SubscriptionBackend<T>>)
            }
        }
    }

    fn register_publisher(&self, sender: mpsc::Sender<AckEvent>) -> u16 {
        self.registry().register(sender)
    }

    fn close(&self) {
        match self {
            TopicInner::BalancedOnly(t) => t.close(),
            TopicInner::BroadcastOnly(t) => t.close(),
            TopicInner::Mixed(t) => t.close(),
        }
    }
}

// ---------------------------------------------------------------------------
// BalancedOnlyTopic
// ---------------------------------------------------------------------------

/// Optimized for the single-consumer-group, no-broadcast case.
///
/// Uses `OnceLock<SingleGroup>` for lazy initialization: the channel pair is
/// created on the first subscribe call and never changes. `OnceLock` makes
/// subsequent subscribes and publishes lock-free (just `.get()`). A second
/// consumer group name is rejected with `SingleGroupViolation`.
///
/// Messages published before any subscriber exists are silently dropped
/// (consistent with MixedTopic behavior).
pub(crate) struct BalancedOnlyTopic<T: Send + Sync + 'static> {
    name: TopicName,
    next_id: AtomicU64,
    group: OnceLock<SingleGroup<T>>,
    balanced_capacity: usize,
    registry: Arc<PublisherRegistry>,
    closed: AtomicBool,
}

impl<T: Send + Sync + 'static> BalancedOnlyTopic<T> {
    fn new(name: TopicName, opts: TopicOptions) -> Self {
        let registry = Arc::new(PublisherRegistry::new(name.clone()));
        Self {
            name,
            next_id: AtomicU64::new(1),
            group: OnceLock::new(),
            balanced_capacity: opts.balanced_capacity.max(1),
            registry,
            closed: AtomicBool::new(false),
        }
    }

    async fn publish(&self, publisher_id: u16, msg: Arc<T>) -> Result<(), Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let seq = self.next_id.fetch_add(1, Ordering::Relaxed);
        let id = message_id::encode(publisher_id, seq);

        if let Some(sg) = self.group.get() {
            let envelope = Envelope { id, payload: msg };
            sg.tx.send(envelope).await.map_err(|_| TopicClosed)?;
        }
        // No subscribers yet → message silently dropped (consistent with Mixed mode).

        Ok(())
    }

    fn subscribe_balanced(
        &self,
        group: Arc<str>,
        _opts: SubscriberOptions,
    ) -> Result<BalancedSub<T>, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let sg = self.group.get_or_init(|| {
            let (tx, rx) = async_channel::bounded(self.balanced_capacity);
            SingleGroup {
                group_name: group.clone(),
                tx,
                rx,
            }
        });

        if sg.group_name != group {
            return Err(SubscribeSingleGroupViolation);
        }

        let ack_state = AckState {
            registry: Arc::clone(&self.registry),
        };

        Ok(BalancedSub {
            rx: Box::pin(sg.rx.clone()),
            ack_state,
        })
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
        if let Some(sg) = self.group.get() {
            // Closing the async_channel by dropping the sender.
            // Subscribers will get RecvError::Closed.
            _ = sg.tx.close();
        }
    }
}

// ---------------------------------------------------------------------------
// BroadcastOnlyTopic
// ---------------------------------------------------------------------------

/// Optimized for broadcast-only delivery. No `async_channel` is allocated.
///
/// `publish()` is synchronous -- it writes to the ring buffer and returns
/// immediately. This is the fastest variant since there is no `.await` point
/// and no per-group channel overhead.
pub(crate) struct BroadcastOnlyTopic<T: Send + Sync + 'static> {
    name: TopicName,
    next_id: AtomicU64,
    broadcast_ring: Arc<FastBroadcastRing<T>>,
    registry: Arc<PublisherRegistry>,
    closed: AtomicBool,
}

impl<T: Send + Sync + 'static> BroadcastOnlyTopic<T> {
    fn new(name: TopicName, opts: TopicOptions) -> Self {
        let registry = Arc::new(PublisherRegistry::new(name.clone()));
        Self {
            name,
            next_id: AtomicU64::new(1),
            broadcast_ring: Arc::new(FastBroadcastRing::new(opts.broadcast_capacity)),
            registry,
            closed: AtomicBool::new(false),
        }
    }

    fn publish(&self, publisher_id: u16, msg: Arc<T>) -> Result<(), Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let seq = self.next_id.fetch_add(1, Ordering::Relaxed);
        let id = message_id::encode(publisher_id, seq);
        self.broadcast_ring.publish_with_id(id, msg);

        Ok(())
    }

    fn subscribe_broadcast(&self, _opts: SubscriberOptions) -> BroadcastSub<T> {
        let start_seq = self.broadcast_ring.current_seq() + 1;

        let ack_state = AckState {
            registry: Arc::clone(&self.registry),
        };

        BroadcastSub {
            ring: Arc::clone(&self.broadcast_ring),
            read_seq: start_seq,
            pending_lag: None,
            spun: false,
            ack_state,
        }
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
        self.broadcast_ring.close();
    }
}

// ---------------------------------------------------------------------------
// MixedTopic — supports both balanced and broadcast subscriptions
// ---------------------------------------------------------------------------

/// Full-featured topic supporting any combination of balanced groups and
/// broadcast subscribers. Every publish writes to ALL balanced groups AND
/// the broadcast ring.
///
/// The consumer groups list is `RwLock<Vec<(name, ConsumerGroup)>>`. The
/// `publish()` path takes a short read lock to clone the senders, then drops
/// the guard before any `.await` -- this is critical for keeping the future
/// `Send`. When no groups are subscribed, the Vec clone is skipped entirely.
pub(crate) struct MixedTopic<T: Send + Sync + 'static> {
    name: TopicName,
    next_id: AtomicU64,
    groups: RwLock<Vec<(Arc<str>, ConsumerGroup<T>)>>,
    balanced_capacity: usize,
    broadcast_ring: Arc<FastBroadcastRing<T>>,
    registry: Arc<PublisherRegistry>,
    closed: AtomicBool,
}

impl<T: Send + Sync + 'static> MixedTopic<T> {
    fn new(name: TopicName, opts: TopicOptions) -> Self {
        let registry = Arc::new(PublisherRegistry::new(name.clone()));
        Self {
            name,
            next_id: AtomicU64::new(1),
            groups: RwLock::new(Vec::new()),
            balanced_capacity: opts.balanced_capacity.max(1),
            broadcast_ring: Arc::new(FastBroadcastRing::new(opts.broadcast_capacity)),
            registry,
            closed: AtomicBool::new(false),
        }
    }

    async fn publish(&self, publisher_id: u16, msg: Arc<T>) -> Result<(), Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let seq = self.next_id.fetch_add(1, Ordering::Relaxed);
        let id = message_id::encode(publisher_id, seq);

        // 1) Write to broadcast ring buffer first (non-blocking, overwrites oldest).
        //    This keeps broadcast delivery independent from balanced backpressure.
        self.broadcast_ring.publish_with_id(id, Arc::clone(&msg));

        // 2) Deliver to balanced consumer groups only if any exist.
        //    Collect senders under a short read lock, then drop the guard
        //    before any .await so the future stays Send.
        //    When no groups are subscribed we skip the Vec alloc + Arc::clone.
        let group_senders: Option<Vec<_>> = {
            let groups = self.groups.read();
            if groups.is_empty() {
                None
            } else {
                Some(groups.iter().map(|(_, g)| g.tx.clone()).collect())
            }
        };
        if let Some(senders) = group_senders {
            let envelope = Envelope {
                id,
                payload: Arc::clone(&msg),
            };
            for tx in &senders {
                tx.send(envelope.clone()).await.map_err(|_| TopicClosed)?;
            }
        }

        Ok(())
    }

    fn subscribe_balanced(
        &self,
        group: Arc<str>,
        _opts: SubscriberOptions,
    ) -> Result<BalancedSub<T>, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(TopicClosed);
        }

        let rx = {
            let mut groups = self.groups.write();
            if let Some((_name, g)) = groups.iter().find(|(n, _)| *n == group) {
                g.rx.clone()
            } else {
                let (tx, rx) = async_channel::bounded(self.balanced_capacity);
                groups.push((group.clone(), ConsumerGroup { tx, rx: rx.clone() }));
                rx
            }
        };

        let ack_state = AckState {
            registry: Arc::clone(&self.registry),
        };

        Ok(BalancedSub {
            rx: Box::pin(rx),
            ack_state,
        })
    }

    fn subscribe_broadcast(&self, _opts: SubscriberOptions) -> BroadcastSub<T> {
        let start_seq = self.broadcast_ring.current_seq() + 1;

        let ack_state = AckState {
            registry: Arc::clone(&self.registry),
        };

        BroadcastSub {
            ring: Arc::clone(&self.broadcast_ring),
            read_seq: start_seq,
            pending_lag: None,
            spun: false,
            ack_state,
        }
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
        let groups = self.groups.read();
        for (_name, g) in groups.iter() {
            // Closing the async_channel by dropping the sender.
            // Subscribers will get RecvError::Closed.
            _ = g.tx.close();
        }
        self.broadcast_ring.close();
    }
}

// ---------------------------------------------------------------------------
// BalancedSub — per-subscription backend for balanced mode
// ---------------------------------------------------------------------------

/// Subscription backend for balanced (consumer-group) mode.
///
/// Holds an `async_channel::Receiver` and delegates `poll_recv` to
/// `futures_core::Stream::poll_next`. The channel handles MPMC fan-out
/// internally.
pub(crate) struct BalancedSub<T: Send + Sync + 'static> {
    rx: Pin<Box<async_channel::Receiver<Envelope<T>>>>,
    ack_state: AckState,
}

impl<T: Send + Sync + 'static> SubscriptionBackend<T> for BalancedSub<T> {
    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Result<RecvItem<T>, Error>> {
        match self.rx.as_mut().poll_next(cx) {
            Poll::Ready(Some(envelope)) => Poll::Ready(Ok(RecvItem::Message(envelope))),
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

// ---------------------------------------------------------------------------
// BroadcastSub — per-subscription backend for broadcast mode
// ---------------------------------------------------------------------------

/// Subscription backend for broadcast mode.
///
/// Implements the three-tier latency strategy:
/// 1. **Fast path**: `ring.try_read(read_seq)` — one atomic load + one Mutex lock.
/// 2. **Spin loop**: 32 iterations (only on first poll, `spun` flag prevents
///    re-spinning on re-polls after wakeup).
/// 3. **Slow path**: register waker in the ring's `WakerSet`, then re-check data.
pub(crate) struct BroadcastSub<T: Send + Sync + 'static> {
    ring: Arc<FastBroadcastRing<T>>,
    read_seq: u64,
    pending_lag: Option<u64>,
    spun: bool,
    ack_state: AckState,
}

impl<T: Send + Sync + 'static> SubscriptionBackend<T> for BroadcastSub<T> {
    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Result<RecvItem<T>, Error>> {
        // Check pending lag first.
        if let Some(missed) = self.pending_lag.take() {
            return Poll::Ready(Ok(RecvItem::Lagged { missed }));
        }

        // Fast path: check without any registration overhead.
        match self.ring.try_read(self.read_seq) {
            BroadcastReadResult::Ready(envelope) => {
                self.read_seq += 1;
                self.spun = false;
                return Poll::Ready(Ok(RecvItem::Message(envelope)));
            }
            BroadcastReadResult::Lagged {
                missed,
                new_read_seq,
            } => {
                self.read_seq = new_read_seq;
                self.spun = false;
                return Poll::Ready(Ok(RecvItem::Lagged { missed }));
            }
            BroadcastReadResult::NotReady => {}
        }

        // Spin loop: brief spin on write_seq (~100 ns) before registering waker.
        // Only spin on first poll; skip on re-polls after wakeup.
        if !self.spun {
            self.spun = true;
            for _ in 0..32 {
                std::hint::spin_loop();
                if self.ring.current_seq() >= self.read_seq {
                    match self.ring.try_read(self.read_seq) {
                        BroadcastReadResult::Ready(envelope) => {
                            self.read_seq += 1;
                            self.spun = false;
                            return Poll::Ready(Ok(RecvItem::Message(envelope)));
                        }
                        BroadcastReadResult::Lagged {
                            missed,
                            new_read_seq,
                        } => {
                            self.read_seq = new_read_seq;
                            self.spun = false;
                            return Poll::Ready(Ok(RecvItem::Lagged { missed }));
                        }
                        BroadcastReadResult::NotReady => {}
                    }
                }
            }
        }

        // Slow path: register waker BEFORE re-checking data to prevent missed wakeups.
        self.ring.register_waker(cx.waker());

        match self.ring.try_read(self.read_seq) {
            BroadcastReadResult::Ready(envelope) => {
                self.read_seq += 1;
                self.spun = false;
                Poll::Ready(Ok(RecvItem::Message(envelope)))
            }
            BroadcastReadResult::Lagged {
                missed,
                new_read_seq,
            } => {
                self.read_seq = new_read_seq;
                self.spun = false;
                Poll::Ready(Ok(RecvItem::Lagged { missed }))
            }
            BroadcastReadResult::NotReady if self.ring.is_closed() => {
                Poll::Ready(Err(SubscriptionClosed))
            }
            BroadcastReadResult::NotReady => Poll::Pending,
        }
    }

    fn ack(&self, id: u64) -> Result<(), Error> {
        self.ack_state.send_ack(id)
    }

    fn nack(&self, id: u64, reason: Arc<str>) -> Result<(), Error> {
        self.ack_state.send_nack(id, reason)
    }
}

// ---------------------------------------------------------------------------
// AckState — subscription-side ack/nack dispatch
// ---------------------------------------------------------------------------

/// Held by each `Subscription`. Wraps a shared `PublisherRegistry` and
/// provides a clean `send_ack` / `send_nack` API that hides the routing
/// logic from the subscription receive path.
pub(crate) struct AckState {
    pub(crate) registry: Arc<PublisherRegistry>,
}

impl AckState {
    pub(crate) fn send_ack(&self, message_id: u64) -> Result<(), Error> {
        self.registry.route_ack(message_id, AckStatus::Ack, None)
    }

    pub(crate) fn send_nack(
        &self,
        message_id: u64,
        reason: impl Into<Arc<str>>,
    ) -> Result<(), Error> {
        self.registry
            .route_ack(message_id, AckStatus::Nack, Some(reason.into()))
    }
}
