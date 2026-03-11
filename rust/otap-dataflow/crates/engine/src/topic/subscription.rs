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
//! `recv()` delegates to `poll_fn(|cx| self.inner.poll_recv(cx))`. Zero
//! allocation per call — the `poll_fn` future is stack-allocated.
//!
//! # Ack/Nack
//!
//! `ack()` and `nack()` are synchronous (no `.await`). `nack()` accepts
//! `impl Into<Arc<str>>` for ergonomics and converts before forwarding to
//! the backend (which takes `Arc<str>` for object safety).

use crate::error::Error;
use crate::topic::backend::SubscriptionBackend;
use crate::topic::types::RecvItem;
use std::sync::Arc;

/// A subscription handle. Call `recv()` to receive messages.
pub struct Subscription<T: Send + Sync + 'static> {
    inner: Box<dyn SubscriptionBackend<T>>,
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
        std::future::poll_fn(|cx| self.inner.poll_recv(cx)).await
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
