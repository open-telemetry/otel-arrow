// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Subscriber-side receive and ack/nack logic.
//!
//! # Structure
//!
//! `Subscription<T>` is a thin wrapper around `Box<dyn SubscriptionBackend<T>>`.
//! All receive and ack/nack logic lives in the backend implementation (see
//! `BalancedSub` and `BroadcastSub` in `topic.rs` for the in-memory backend).
//!
//! # Receive
//!
//! `recv()` delegates to `poll_fn(|cx| self.inner.poll_recv_delivery(cx))`.
//! The future itself is stack-allocated. In-memory delivery permits also avoid
//! per-message heap allocation on the common path.
//!
//! # Ack/Nack
//!
//! `ack()` and `nack()` are synchronous (no `.await`). `nack()` accepts
//! `impl Into<Arc<str>>` for ergonomics and converts before forwarding to
//! the backend (which takes `Arc<str>` for object safety).

use crate::error::Error;
use crate::topic::backend::SubscriptionBackend;
use crate::topic::topic::InMemoryDeliveryFinalizer;
#[cfg(test)]
use crate::topic::topic::InMemoryDeliveryKind;
use crate::topic::types::{Envelope, RecvItem};
use std::sync::Arc;

/// A subscription handle. Call `recv()` to receive messages.
pub struct Subscription<T: Send + Sync + 'static> {
    inner: Box<dyn SubscriptionBackend<T>>,
}

/// A receive result that keeps ownership of one delivered topic message until
/// the caller either commits or aborts it.
pub enum RecvDelivery<T: Send + Sync + 'static> {
    /// One delivered topic message.
    Message(Delivery<T>),
    /// Notification that this broadcast subscriber lagged and missed messages.
    Lagged {
        /// Number of dropped messages for this subscriber.
        missed: u64,
    },
}

// Reserved fallback hook for non-in-memory topic backends. The current tree
// only ships the in-memory backend, so normal builds do not instantiate this
// path yet.
#[allow(dead_code)]
pub(crate) trait DeliveryBackend<T: Send + Sync + 'static> {
    fn envelope(&self) -> &Envelope<T>;

    fn commit(&mut self);

    fn abort(&mut self, reason: Arc<str>) -> Result<(), Error>;

    fn abandon(&mut self);
}

/// A topic delivery that has been received from a subscription but not yet
/// finalized.
///
/// Callers should either:
/// - `commit()` after the message has been handed off successfully, or
/// - `abort(...)` if the message must be rejected before handoff.
///
/// Dropping an unresolved delivery abandons it. For tracked messages this
/// resolves the topic-side delivery as a negative outcome so the topic runtime
/// does not retain it indefinitely.
pub struct Delivery<T: Send + Sync + 'static> {
    envelope: Envelope<T>,
    finalizer: DeliveryFinalizer<T>,
}

enum DeliveryFinalizer<T: Send + Sync + 'static> {
    InMemory(InMemoryDeliveryFinalizer),
    // Keep the opaque finalizer path available so a future backend can attach
    // its own delivery permit implementation without changing the public
    // subscription API. The in-memory backend no longer uses this variant.
    #[allow(dead_code)]
    Opaque(Box<dyn DeliveryBackend<T>>),
    Finished,
}

impl<T: Send + Sync + 'static> Delivery<T> {
    pub(crate) fn new_in_memory(
        envelope: Envelope<T>,
        finalizer: InMemoryDeliveryFinalizer,
    ) -> Self {
        Self {
            envelope,
            finalizer: DeliveryFinalizer::InMemory(finalizer),
        }
    }

    // Keep this constructor available for future non-in-memory backends. It is
    // only used by tests today because the in-memory backend now constructs
    // specialized inline deliveries directly.
    #[allow(dead_code)]
    pub(crate) fn new_opaque(inner: Box<dyn DeliveryBackend<T>>) -> Self {
        let envelope = inner.envelope().clone();
        Self {
            envelope,
            finalizer: DeliveryFinalizer::Opaque(inner),
        }
    }

    /// Inspect the delivered message envelope.
    #[must_use]
    pub fn envelope(&self) -> &Envelope<T> {
        &self.envelope
    }

    /// Topic-assigned message id.
    #[must_use]
    pub fn message_id(&self) -> u64 {
        self.envelope().id
    }

    /// Whether this delivery participates in tracked topic outcomes.
    #[must_use]
    pub fn tracked(&self) -> bool {
        self.envelope().tracked
    }

    /// Finalize the delivery after successful handoff.
    pub fn commit(mut self) {
        match std::mem::replace(&mut self.finalizer, DeliveryFinalizer::Finished) {
            DeliveryFinalizer::InMemory(mut finalizer) => finalizer.commit(),
            DeliveryFinalizer::Opaque(mut inner) => inner.commit(),
            DeliveryFinalizer::Finished => {}
        }
    }

    /// Reject the delivery before successful handoff.
    pub fn abort(mut self, reason: impl Into<Arc<str>>) -> Result<(), Error> {
        let reason = reason.into();
        match std::mem::replace(&mut self.finalizer, DeliveryFinalizer::Finished) {
            DeliveryFinalizer::InMemory(mut finalizer) => finalizer.abort(&self.envelope, reason),
            DeliveryFinalizer::Opaque(mut inner) => inner.abort(reason),
            DeliveryFinalizer::Finished => Ok(()),
        }
    }

    #[cfg(test)]
    pub(crate) fn storage_kind(&self) -> DeliveryStorageKind {
        match &self.finalizer {
            DeliveryFinalizer::InMemory(finalizer) => match finalizer.kind() {
                InMemoryDeliveryKind::Balanced => DeliveryStorageKind::Balanced,
                InMemoryDeliveryKind::Broadcast => DeliveryStorageKind::Broadcast,
            },
            DeliveryFinalizer::Opaque(_) => DeliveryStorageKind::Opaque,
            DeliveryFinalizer::Finished => panic!("finished deliveries should not be inspected"),
        }
    }
}

impl<T: Send + Sync + 'static> Drop for Delivery<T> {
    fn drop(&mut self) {
        match std::mem::replace(&mut self.finalizer, DeliveryFinalizer::Finished) {
            DeliveryFinalizer::InMemory(mut finalizer) => finalizer.abandon(&self.envelope),
            DeliveryFinalizer::Opaque(mut inner) => inner.abandon(),
            DeliveryFinalizer::Finished => {}
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeliveryStorageKind {
    Balanced,
    Broadcast,
    Opaque,
}

impl<T: Send + Sync + 'static> Subscription<T> {
    pub(crate) fn new(inner: Box<dyn SubscriptionBackend<T>>) -> Self {
        Self { inner }
    }

    /// Receive the next item.
    ///
    /// For broadcast subscribers this may yield a `Lagged { missed }` notification
    /// when messages were dropped for this subscriber. With the default
    /// `drop_oldest` policy, the next call to `recv()` returns the oldest
    /// still-available message. With `disconnect`, the next call returns
    /// `Error::SubscriptionClosed`.
    pub async fn recv(&mut self) -> Result<RecvItem<T>, Error> {
        match self.recv_delivery().await? {
            RecvDelivery::Message(delivery) => {
                let envelope = delivery.envelope().clone();
                delivery.commit();
                Ok(RecvItem::Message(envelope))
            }
            RecvDelivery::Lagged { missed } => Ok(RecvItem::Lagged { missed }),
        }
    }

    /// Receive the next delivery while keeping ownership of the topic-side
    /// delivery state until the caller commits or aborts it.
    pub async fn recv_delivery(&mut self) -> Result<RecvDelivery<T>, Error> {
        std::future::poll_fn(|cx| self.inner.poll_recv_delivery(cx)).await
    }

    /// Acknowledge successful processing of a message.
    ///
    /// Returns `Error::MessageNotTracked` if the message was not published
    /// through the tracked publish path.
    pub fn ack(&self, id: u64) -> Result<(), Error> {
        self.inner.ack(id)
    }

    /// Negatively acknowledge a message.
    ///
    /// Returns `Error::MessageNotTracked` if the message was not published
    /// through the tracked publish path.
    pub fn nack(&self, id: u64, reason: impl Into<Arc<str>>) -> Result<(), Error> {
        self.inner.nack(id, reason.into())
    }
}
