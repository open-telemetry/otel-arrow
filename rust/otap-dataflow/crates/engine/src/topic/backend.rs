// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Backend trait abstraction for the topic broker.
//!
//! Three traits at three abstraction levels:
//!
//! ```text
//!   TopicBackend<T>              -- topic-level factory (one per create_topic call)
//!        │ creates
//!        ▼
//!   TopicState<T>                -- per-topic shared state (in Arc, many handles)
//!        │ creates
//!        ▼
//!   SubscriptionBackend<T>       -- per-subscription owned state (one per subscriber)
//! ```
//!
//! The dispatch mechanism uses trait objects (`dyn`):
//! - `TopicBroker::create_topic()` accepts `impl TopicBackend<T>` per call
//! - `TopicHandle<T>` stores `Arc<dyn TopicState<T>>`
//! - `Subscription<T>` stores `Box<dyn SubscriptionBackend<T>>`
//!
//! This keeps the public types free of backend type parameters.
//!
//! # InMemoryBackend
//!
//! The default backend. A zero-sized type that delegates `create_topic` to
//! `TopicInner::new()`, preserving the existing in-memory implementation.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::error::Error;
use crate::topic::subscription::RecvDelivery;
use crate::topic::topic::TopicInner;
use crate::topic::types::{
    PublishOutcome, SubscriberOptions, TopicOptions, TrackedPublishPermit, TrackedPublishReceipt,
    TrackedTryPublishOutcome,
};
use otap_df_config::topic::TopicBroadcastOnLagPolicy;
use otap_df_config::{SubscriptionGroupName, TopicName};

/// The future type returned by [`TopicState::publish`].
///
/// One `Box` allocation per publish call (~25 ns). Acceptable because publish
/// fans out to many consumers and for external backends (for example Quiver) this is
/// noise compared to network I/O.
pub type PublishFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;

/// The future type returned by [`TopicState::publish_tracked`].
pub type PublishTrackedFuture<'a> =
    Pin<Box<dyn Future<Output = Result<TrackedPublishReceipt, Error>> + Send + 'a>>;

/// Factory: creates per-topic state. One implementation per backend.
pub trait TopicBackend<T: Send + Sync + 'static>: Send + Sync + 'static {
    /// Create a topic state instance for `name` with the provided options.
    fn create_topic(&self, name: TopicName, opts: TopicOptions) -> Arc<dyn TopicState<T>>;
}

/// Per-topic operations. Shared across all handles via `Arc`.
pub trait TopicState<T: Send + Sync + 'static>: Send + Sync {
    /// Topic name.
    fn name(&self) -> &TopicName;
    /// Publish one payload.
    fn publish(&self, msg: Arc<T>) -> PublishFuture<'_>;
    /// Publish one payload and return a tracked-outcome receipt.
    ///
    /// `timeout` defines the maximum time the publish may remain unresolved
    /// after the topic accepts it for tracked delivery. If no subscriber
    /// produces a terminal Ack/Nack before that deadline, the returned receipt
    /// resolves as [`TrackedPublishOutcome::TimedOut`](crate::topic::TrackedPublishOutcome::TimedOut).
    ///
    /// `permit` is a caller-acquired in-flight capacity token. Tracked
    /// publishers use it to enforce `max_in_flight` before entering the topic.
    /// Ownership is transferred to the topic runtime and the permit is released
    /// only when the tracked outcome reaches a terminal state or the publish is
    /// rejected before admission.
    ///
    /// External backend implementations are expected to use a
    /// [`TrackedPublishTracker`] to construct and resolve tracked receipts with
    /// one shared timeout worker rather than per-message timeout tasks.
    fn publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> PublishTrackedFuture<'_>;
    /// Try to publish one payload without awaiting.
    fn try_publish(&self, msg: Arc<T>) -> Result<PublishOutcome, Error>;
    /// Try to publish one tracked payload without awaiting.
    ///
    /// `timeout` and `permit` have the same meaning as in
    /// [`TopicState::publish_tracked`]. If the topic cannot accept the message
    /// immediately, the implementation returns a
    /// [`TrackedTryPublishOutcome`] instead of awaiting.
    fn try_publish_tracked(
        &self,
        msg: Arc<T>,
        timeout: Duration,
        permit: TrackedPublishPermit,
    ) -> Result<TrackedTryPublishOutcome, Error>;
    /// Create a balanced subscription backend for consumer-group `group`.
    fn subscribe_balanced(
        &self,
        group: SubscriptionGroupName,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error>;
    /// Create a broadcast subscription backend.
    fn subscribe_broadcast(
        &self,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error>;
    /// Effective broadcast lag policy for this topic.
    fn broadcast_on_lag_policy(&self) -> TopicBroadcastOnLagPolicy;
    /// Close the topic. Existing subscriptions eventually observe closure.
    fn close(&self);
}

/// Per-subscription operations. Exclusively owned by one `Subscription`.
///
/// `Send` (not `Sync`) -- owned by one `Subscription`, `poll_recv` takes
/// `&mut self`. `ack`/`nack` take `&self` which is fine.
pub trait SubscriptionBackend<T: Send + Sync + 'static>: Send {
    /// Poll for the next receive item with explicit delivery ownership.
    fn poll_recv_delivery(&mut self, cx: &mut Context<'_>) -> Poll<Result<RecvDelivery<T>, Error>>;
    /// Ack a previously received message id.
    fn ack(&self, id: u64) -> Result<(), Error>;
    /// `reason` is `Arc<str>` (not `impl Into<Arc<str>>`) for object safety.
    /// The `Subscription` wrapper provides the ergonomic `impl Into` conversion.
    ///
    /// Nack a previously received message id.
    fn nack(&self, id: u64, reason: Arc<str>) -> Result<(), Error>;
}

/// The default in-memory backend.
pub struct InMemoryBackend;

impl<T: Send + Sync + 'static> TopicBackend<T> for InMemoryBackend {
    fn create_topic(&self, name: TopicName, opts: TopicOptions) -> Arc<dyn TopicState<T>> {
        Arc::new(TopicInner::new(name, opts))
    }
}
