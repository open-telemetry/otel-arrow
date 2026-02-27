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

use crate::error::Error;
use crate::topic::topic::TopicInner;
use crate::topic::types::{AckEvent, RecvItem, SubscriberOptions, TopicOptions};
use otap_df_config::TopicName;
use tokio::sync::mpsc;

/// The future type returned by [`TopicState::publish`].
///
/// One `Box` allocation per publish call (~25 ns). Acceptable because publish
/// fans out to many consumers and for external backends (Kafka/Quiver) this is
/// noise compared to network I/O.
pub type PublishFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;

/// Factory: creates per-topic state. One implementation per backend.
pub trait TopicBackend<T: Send + Sync + 'static>: Send + Sync + 'static {
    /// Create a topic state instance for `name` with the provided options.
    fn create_topic(&self, name: TopicName, opts: TopicOptions) -> Arc<dyn TopicState<T>>;
}

/// Per-topic operations. Shared across all handles via `Arc`.
pub trait TopicState<T: Send + Sync + 'static>: Send + Sync {
    /// Topic name.
    fn name(&self) -> &TopicName;
    /// Publish one payload under `publisher_id`.
    fn publish(&self, publisher_id: u16, msg: Arc<T>) -> PublishFuture<'_>;
    /// Create a balanced subscription backend for consumer-group `group`.
    fn subscribe_balanced(
        &self,
        group: Arc<str>,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error>;
    /// Create a broadcast subscription backend.
    fn subscribe_broadcast(
        &self,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error>;
    /// Register an ack sender for a publisher handle and return its publisher id.
    fn register_publisher(&self, sender: mpsc::Sender<AckEvent>) -> u16;
    /// Close the topic. Existing subscriptions eventually observe closure.
    fn close(&self);
}

/// Per-subscription operations. Exclusively owned by one `Subscription`.
///
/// `Send` (not `Sync`) -- owned by one `Subscription`, `poll_recv` takes
/// `&mut self`. `ack`/`nack` take `&self` which is fine.
pub trait SubscriptionBackend<T: Send + Sync + 'static>: Send {
    /// Poll for the next receive item.
    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Result<RecvItem<T>, Error>>;
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
