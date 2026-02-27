// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core value types shared across the crate.
//!
//! This module defines the data types that flow through the public API. No
//! behavior lives here -- only data definitions and conversions.
//!
//! # Message ID Encoding
//!
//! The `message_id` submodule packs a publisher_id (u16) and sequence (u48)
//! into a single u64. This encoding is the mechanism that enables per-publisher
//! ack routing: when a subscriber acks a message, the `PublisherRegistry`
//! extracts the publisher_id from the message ID to route the `AckEvent` to
//! the correct channel. Publisher_id 0 means "no ack sender registered"
//! (returns `NotEnabled`).
//!
//! # TopicMode
//!
//! Chosen at topic creation time and immutable. Controls which `TopicInner`
//! variant is instantiated (see `topic.rs`). The three modes (`BalancedOnly`,
//! `BroadcastOnly`, `Mixed`) exist to avoid paying for unused delivery
//! mechanisms -- e.g., `BalancedOnly` never allocates a broadcast ring buffer.

use otap_df_config::TopicName;
use std::sync::Arc;

/// A delivered message envelope carrying a broker-assigned id and the payload.
#[derive(Debug)]
pub struct Envelope<T> {
    /// Broker-assigned message identifier.
    pub id: u64,
    /// Shared payload reference delivered to subscribers.
    pub payload: Arc<T>,
}

impl<T> Clone for Envelope<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            payload: Arc::clone(&self.payload),
        }
    }
}

/// Items yielded by `Subscription::recv`.
#[derive(Debug)]
pub enum RecvItem<T> {
    /// A normal message.
    Message(Envelope<T>),
    /// Notification that this broadcast subscriber lagged and missed messages.
    Lagged {
        /// Number of dropped messages for this subscriber.
        missed: u64,
    },
}

/// Subscription mode chosen by the subscriber.
#[derive(Debug, Clone)]
pub enum SubscriptionMode {
    /// Balanced (consumer-group) mode: each message goes to exactly one subscriber in the group.
    Balanced {
        /// Consumer-group identifier.
        group: Arc<str>,
    },
    /// Broadcast mode: each subscriber gets every message (drop-oldest on slow consumers).
    Broadcast,
}

/// Options provided at subscription time.
#[derive(Debug, Clone, Default)]
pub struct SubscriberOptions {}

/// Status of an ack/nack event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckStatus {
    /// Message processing succeeded.
    Ack,
    /// Message processing failed.
    Nack,
}

/// An ack/nack event emitted by a consumer.
#[derive(Debug, Clone)]
pub struct AckEvent {
    /// Topic where the message was published.
    pub topic: TopicName,
    /// Message identifier emitted with the delivered envelope.
    pub message_id: u64,
    /// Final status reported by the subscriber.
    pub status: AckStatus,
    /// Optional failure reason for nacks.
    pub reason: Option<Arc<str>>,
    /// The publisher ID extracted from the message ID (0 = no ack sender registered).
    pub publisher_id: u16,
}

/// Bit-packed message ID helpers.
///
/// Layout: `[ publisher_id : u16 ][ sequence : u48 ]`
pub(crate) mod message_id {
    const SEQ_BITS: u32 = 48;
    const SEQ_MASK: u64 = (1u64 << SEQ_BITS) - 1;

    /// Encode a publisher_id and sequence into a single u64 message ID.
    #[inline]
    pub(crate) fn encode(publisher_id: u16, sequence: u64) -> u64 {
        ((publisher_id as u64) << SEQ_BITS) | (sequence & SEQ_MASK)
    }

    /// Extract the publisher_id from a message ID.
    #[inline]
    pub(crate) fn publisher_id(id: u64) -> u16 {
        (id >> SEQ_BITS) as u16
    }
}

/// The mode a topic operates in, chosen at creation time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TopicMode {
    /// Only balanced (consumer-group) subscriptions. No broadcast ring buffer overhead.
    BalancedOnly,
    /// Only broadcast subscriptions. No consumer-group channel overhead.
    BroadcastOnly,
    /// Both balanced and broadcast subscriptions (default).
    #[default]
    Mixed,
}

/// Options for creating/opening a topic.
#[derive(Debug, Clone)]
pub struct TopicOptions {
    /// Capacity for balanced consumer-group channels.
    pub balanced_capacity: usize,
    /// Ring-buffer capacity for broadcast subscribers.
    pub broadcast_capacity: usize,
    /// Topic mode — controls which subscription types are supported.
    pub mode: TopicMode,
}

impl Default for TopicOptions {
    fn default() -> Self {
        Self {
            balanced_capacity: 1024,
            broadcast_capacity: 1024,
            mode: TopicMode::Mixed,
        }
    }
}
