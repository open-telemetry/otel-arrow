// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Central topic broker.
//!
//! # Per-Topic Backend Selection
//!
//! The backend is selected per-topic at creation time via `create_topic()`.
//! Different topics in the same broker can use different backends. Currently the
//! only backend is an in-memory implementation, but this design allows for future
//! extensions (e.g. disk-backed, networked, etc.) without changing the broker API.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Error;
use crate::topic::backend::{InMemoryBackend, TopicBackend, TopicState};
use crate::topic::handle::TopicHandle;
use crate::topic::types::TopicOptions;
use otap_df_config::TopicName;
use parking_lot::RwLock;

/// The central topic broker. Create/open topics and obtain handles for publish/subscribe.
///
/// Thread-safe and cheaply cloneable.
#[derive(Clone)]
pub struct TopicBroker<T: Send + Sync + 'static> {
    inner: Arc<BrokerInner<T>>,
}

struct BrokerInner<T: Send + Sync + 'static> {
    topics: RwLock<HashMap<TopicName, Arc<dyn TopicState<T>>>>,
}

impl<T: Send + Sync + 'static> TopicBroker<T> {
    /// Create a new broker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(BrokerInner {
                topics: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Create a new topic. Returns an error if a topic with the same name
    /// already exists.
    pub fn create_topic(
        &self,
        name: impl Into<TopicName>,
        opts: TopicOptions,
        backend: impl TopicBackend<T>,
    ) -> Result<TopicHandle<T>, Error> {
        let name: TopicName = name.into();
        let mut topics = self.inner.topics.write();
        if topics.contains_key(&name) {
            return Err(Error::TopicAlreadyExists { topic: name });
        }
        let state = backend.create_topic(name.clone(), opts);
        // Result of insert is ignored since we already checked for existence inside the write lock.
        _ = topics.insert(name, state.clone());
        Ok(TopicHandle::new(state))
    }

    /// Convenience wrapper around [`create_topic`](Self::create_topic) that
    /// uses the default in-memory backend.
    pub fn create_in_memory_topic(
        &self,
        name: impl Into<TopicName>,
        opts: TopicOptions,
    ) -> Result<TopicHandle<T>, Error> {
        self.create_topic(name, opts, InMemoryBackend)
    }

    /// Look up a topic by name without creating it.
    pub fn get_topic(&self, name: impl Into<TopicName>) -> Option<TopicHandle<T>> {
        let name: TopicName = name.into();
        let topics = self.inner.topics.read();
        topics
            .get(&name)
            .map(|inner| TopicHandle::new(inner.clone()))
    }

    /// Check whether a topic exists.
    pub fn has_topic(&self, name: impl Into<TopicName>) -> bool {
        let name: TopicName = name.into();
        let topics = self.inner.topics.read();
        topics.contains_key(&name)
    }

    /// Close and remove a topic. Subscribers eventually get
    /// `Error::SubscriptionClosed`, publishers get `Error::TopicClosed`.
    /// Returns `true` if the topic was found (and closed + removed).
    pub fn remove_topic(&self, name: impl Into<TopicName>) -> bool {
        let name: TopicName = name.into();
        let mut topics = self.inner.topics.write();
        if let Some(inner) = topics.remove(&name) {
            inner.close();
            true
        } else {
            false
        }
    }

    /// Snapshot of all topic names currently in the broker.
    #[must_use]
    pub fn topic_names(&self) -> Vec<TopicName> {
        let topics = self.inner.topics.read();
        topics.keys().cloned().collect()
    }

    /// Close all topics and clear the broker.
    pub fn close_all(&self) {
        let mut topics = self.inner.topics.write();
        for (_, inner) in topics.drain() {
            inner.close();
        }
    }
}

impl<T: Send + Sync + 'static> Default for TopicBroker<T> {
    fn default() -> Self {
        Self::new()
    }
}
