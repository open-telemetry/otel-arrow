// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A thread-safe, mutable, named collection of topic handles.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Error;
use crate::topic::handle::TopicHandle;
use otap_df_config::TopicName;
use parking_lot::RwLock;

/// A thread-safe, mutable, named collection of topic handles.
pub struct TopicSet<T: Send + Sync + 'static> {
    inner: Arc<TopicSetInner<T>>,
}

impl<T: Send + Sync + 'static> Clone for TopicSet<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

struct TopicSetInner<T: Send + Sync + 'static> {
    name: Arc<str>,
    topics: RwLock<HashMap<TopicName, TopicHandle<T>>>,
}

impl<T: Send + Sync + 'static> TopicSet<T> {
    /// Create a new empty topic set.
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self {
            inner: Arc::new(TopicSetInner {
                name: name.into(),
                topics: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// The name of this topic set.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Insert a topic handle under a local name.
    pub fn insert(
        &self,
        local_name: impl Into<TopicName>,
        handle: TopicHandle<T>,
    ) -> Option<TopicHandle<T>> {
        let mut topics = self.inner.topics.write();
        topics.insert(local_name.into(), handle)
    }

    /// Remove a topic handle by local name.
    #[must_use]
    pub fn remove(&self, local_name: impl AsRef<str>) -> Option<TopicHandle<T>> {
        let local_name = local_name.as_ref();
        let mut topics = self.inner.topics.write();
        topics.remove(local_name)
    }

    /// Get a cloned handle for the given local name.
    #[must_use]
    pub fn get(&self, local_name: impl AsRef<str>) -> Option<TopicHandle<T>> {
        let local_name = local_name.as_ref();
        let topics = self.inner.topics.read();
        topics.get(local_name).cloned()
    }

    /// Get a cloned handle for the given local name or return an explicit error.
    pub fn get_required(&self, local_name: impl AsRef<str>) -> Result<TopicHandle<T>, Error> {
        let local_name = local_name.as_ref();
        self.get(local_name).ok_or_else(|| Error::UnknownTopic {
            topic: local_name.to_owned(),
        })
    }

    /// Check whether a topic with the given local name exists in this set.
    #[must_use]
    pub fn contains(&self, local_name: impl AsRef<str>) -> bool {
        let local_name = local_name.as_ref();
        let topics = self.inner.topics.read();
        topics.contains_key(local_name)
    }

    /// Snapshot of all local topic names in this set.
    #[must_use]
    pub fn topic_names(&self) -> Vec<TopicName> {
        let topics = self.inner.topics.read();
        topics.keys().cloned().collect()
    }

    /// Number of topics in this set.
    #[must_use]
    pub fn len(&self) -> usize {
        let topics = self.inner.topics.read();
        topics.len()
    }

    /// Whether this set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let topics = self.inner.topics.read();
        topics.is_empty()
    }
}
