// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic handle is the user-facing publish/subscribe entry point.
//!
//! # Why `Arc<dyn TopicState>` + `u16`
//!
//! `TopicHandle` is intentionally minimal: an `Arc` pointer to the shared topic
//! state (via the `TopicState` trait object) plus a `publisher_id` (u16).
//! Cloning is an atomic increment + u16 copy -- designed to be passed freely
//! into spawned tasks.
//!
//! # Publisher ID
//!
//! The default `publisher_id` is 0 (topic-level). Calling `with_ack_sender()`
//! registers a per-publisher channel in the topic's `PublisherRegistry` and
//! returns a new handle with the assigned non-zero ID. Every message published
//! through this handle has the ID bit-packed into the message ID (see
//! `types::message_id`), enabling the ack routing system to deliver ack events
//! to the correct publisher.
//!
//! # Delegation
//!
//! All methods delegate directly to the `dyn TopicState` trait object.
//! This module contains no business logic -- it exists to provide a clean
//! public API that hides the backend dispatch.

use std::sync::Arc;

use crate::error::Error;
use crate::topic::backend::TopicState;
use crate::topic::subscription::Subscription;
use crate::topic::types::{AckEvent, SubscriberOptions, SubscriptionMode};
use otap_df_config::TopicName;
use tokio::sync::mpsc;

/// A handle to a topic, used for publishing and subscribing.
///
/// Thread-safe and cheaply cloneable.
/// Each handle carries a `publisher_id` (default 0) that is encoded into
/// message IDs on publish, enabling per-publisher ack routing.
pub struct TopicHandle<T: Send + Sync + 'static> {
    inner: Arc<dyn TopicState<T>>,
    publisher_id: u16,
}

impl<T: Send + Sync + 'static> Clone for TopicHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            publisher_id: self.publisher_id,
        }
    }
}

impl<T: Send + Sync + 'static> TopicHandle<T> {
    pub(crate) fn new(inner: Arc<dyn TopicState<T>>) -> Self {
        Self {
            inner,
            publisher_id: 0,
        }
    }

    /// Create a new handle with a per-publisher ack channel.
    ///
    /// Registers the given sender in the topic's publisher registry and returns
    /// a new handle whose publishes will encode the assigned `publisher_id`
    /// into message IDs. When a subscriber acks one of these messages, the ack
    /// event is routed to this sender instead of the topic-level default.
    #[must_use]
    pub fn with_ack_sender(&self, sender: mpsc::Sender<AckEvent>) -> Self {
        let id = self.inner.register_publisher(sender);
        Self {
            inner: Arc::clone(&self.inner),
            publisher_id: id,
        }
    }

    /// Publish a message to the topic. All balanced consumer groups and broadcast subscribers
    /// will be notified.
    ///
    /// This may await under backpressure when balanced consumer-group buffers are full.
    pub async fn publish(&self, msg: Arc<T>) -> Result<(), Error> {
        self.inner.publish(self.publisher_id, msg).await
    }

    /// Subscribe to this topic.
    ///
    /// - `mode`: `SubscriptionMode::Balanced { group }` or `SubscriptionMode::Broadcast`.
    /// - `opts`: subscriber options.
    ///
    /// Returns an error if the subscription mode is incompatible with the topic's
    /// configured `TopicOptions` variant or if the topic is already closed.
    pub fn subscribe(
        &self,
        mode: SubscriptionMode,
        opts: SubscriberOptions,
    ) -> Result<Subscription<T>, Error> {
        let backend = match mode {
            SubscriptionMode::Balanced { group } => self.inner.subscribe_balanced(group, opts)?,
            SubscriptionMode::Broadcast => self.inner.subscribe_broadcast(opts)?,
        };
        Ok(Subscription::new(backend))
    }

    /// Close the topic. After closing, further publishes return `Error::TopicClosed`.
    pub fn close(&self) {
        self.inner.close();
    }

    /// Get the topic name.
    #[must_use]
    pub fn name(&self) -> &TopicName {
        self.inner.name()
    }
}
