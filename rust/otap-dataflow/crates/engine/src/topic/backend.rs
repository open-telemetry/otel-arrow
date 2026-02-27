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
    fn create_topic(&self, name: TopicName, opts: TopicOptions) -> Arc<dyn TopicState<T>>;
}

/// Per-topic operations. Shared across all handles via `Arc`.
pub trait TopicState<T: Send + Sync + 'static>: Send + Sync {
    fn name(&self) -> &TopicName;
    fn publish(&self, publisher_id: u16, msg: Arc<T>) -> PublishFuture<'_>;
    fn subscribe_balanced(
        &self,
        group: Arc<str>,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error>;
    fn subscribe_broadcast(
        &self,
        opts: SubscriberOptions,
    ) -> Result<Box<dyn SubscriptionBackend<T>>, Error>;
    fn register_publisher(&self, sender: mpsc::Sender<AckEvent>) -> u16;
    fn close(&self);
}

/// Per-subscription operations. Exclusively owned by one `Subscription`.
///
/// `Send` (not `Sync`) -- owned by one `Subscription`, `poll_recv` takes
/// `&mut self`. `ack`/`nack` take `&self` which is fine.
pub trait SubscriptionBackend<T: Send + Sync + 'static>: Send {
    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Result<RecvItem<T>, Error>>;
    fn ack(&self, id: u64) -> Result<(), Error>;
    /// `reason` is `Arc<str>` (not `impl Into<Arc<str>>`) for object safety.
    /// The `Subscription` wrapper provides the ergonomic `impl Into` conversion.
    fn nack(&self, id: u64, reason: Arc<str>) -> Result<(), Error>;
}

/// The default in-memory backend.
pub struct InMemoryBackend;

impl<T: Send + Sync + 'static> TopicBackend<T> for InMemoryBackend {
    fn create_topic(&self, name: TopicName, opts: TopicOptions) -> Arc<dyn TopicState<T>> {
        Arc::new(TopicInner::new(name, opts))
    }
}
