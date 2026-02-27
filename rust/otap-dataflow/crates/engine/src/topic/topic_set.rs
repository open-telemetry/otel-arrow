// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A thread-safe, mutable, named collection of topic handles.
//!
//! # Purpose
//!
//! `TopicSet` bridges the gap between the broker (which stores topics with
//! fully-qualified names like `"G1::output"`) and pipeline code (which uses
//! simple local names like `"output"`). The integration layer resolves
//! hierarchical scope (global / group / pipeline) externally, then builds a
//! `TopicSet` per pipeline as a resolved name -> handle mapping.
//!
//! # Structure
//!
//! `TopicSet<T>` wraps `Arc<TopicSetInner<T>>`, so clones share state. The
//! inner map is `parking_lot::RwLock<HashMap<TopicName, TopicHandle<T>>>`.
//! Lookups use `&str` keys thanks to `TopicName`'s `Borrow<str>` impl,
//! avoiding `Arc` allocation on the hot path.
//!
//! # Ack Sender Auto-Wrapping
//!
//! When created with `with_ack_sender()`, every handle passed to `insert()` is
//! automatically wrapped via `handle.with_ack_sender(sender.clone())`. This
//! registers a new `publisher_id` in the topic's `PublisherRegistry`, so each
//! pipeline-topic pair gets unique ack routing. The original handle is not
//! modified -- the wrapped copy is what gets stored.
//!
//! # Remove Semantics
//!
//! `remove()` detaches a handle from the set but does NOT close the underlying
//! topic. A global topic may be referenced by many sets; closing it on removal
//! from one set would break the others. Only `broker.remove_topic()` or
//! `handle.close()` closes a topic.

use std::collections::HashMap;
use std::sync::Arc;

use crate::topic::handle::TopicHandle;
use crate::topic::types::AckEvent;
use otap_df_config::TopicName;
use parking_lot::RwLock;
use tokio::sync::mpsc;

/// A thread-safe, mutable, named collection of topic handles.
///
/// Cheaply cloneable (wraps an `Arc`). Multiple tasks in a pipeline can hold
/// the same `TopicSet` and see the same state.
#[derive(Clone)]
pub struct TopicSet<T: Send + Sync + 'static> {
    inner: Arc<TopicSetInner<T>>,
}

struct TopicSetInner<T: Send + Sync + 'static> {
    name: Arc<str>,
    topics: RwLock<HashMap<TopicName, TopicHandle<T>>>,
    ack_sender: Option<mpsc::Sender<AckEvent>>,
}

impl<T: Send + Sync + 'static> TopicSet<T> {
    /// Create a new empty topic set.
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self {
            inner: Arc::new(TopicSetInner {
                name: name.into(),
                topics: RwLock::new(HashMap::new()),
                ack_sender: None,
            }),
        }
    }

    /// Create a new empty topic set with a per-set ack sender.
    ///
    /// Every handle inserted via [`insert`](Self::insert) is automatically
    /// wrapped via `handle.with_ack_sender(sender.clone())`, giving each
    /// pipeline-topic pair a unique publisher_id for ack routing.
    pub fn with_ack_sender(name: impl Into<Arc<str>>, sender: mpsc::Sender<AckEvent>) -> Self {
        Self {
            inner: Arc::new(TopicSetInner {
                name: name.into(),
                topics: RwLock::new(HashMap::new()),
                ack_sender: Some(sender),
            }),
        }
    }

    /// The name of this topic set.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Insert a topic handle under a local name.
    ///
    /// If the set was created with an ack sender, the handle is automatically
    /// wrapped via `handle.with_ack_sender()`. Returns the previous handle if
    /// a topic with the same name was already present.
    pub fn insert(
        &self,
        local_name: impl Into<TopicName>,
        handle: TopicHandle<T>,
    ) -> Option<TopicHandle<T>> {
        let handle = match &self.inner.ack_sender {
            Some(sender) => handle.with_ack_sender(sender.clone()),
            None => handle,
        };
        let mut topics = self.inner.topics.write();
        topics.insert(local_name.into(), handle)
    }

    /// Remove a topic handle by local name.
    ///
    /// Does **not** close the underlying topic — it may be shared with other
    /// sets. Only `broker.remove_topic()` or `handle.close()` closes a topic.
    pub fn remove(&self, local_name: &str) -> Option<TopicHandle<T>> {
        let mut topics = self.inner.topics.write();
        topics.remove(local_name)
    }

    /// Get a cloned handle for the given local name. Cheap: `Arc` + `u16`.
    pub fn get(&self, local_name: &str) -> Option<TopicHandle<T>> {
        let topics = self.inner.topics.read();
        topics.get(local_name).cloned()
    }

    /// Check whether a topic with the given local name exists in this set.
    pub fn contains(&self, local_name: &str) -> bool {
        let topics = self.inner.topics.read();
        topics.contains_key(local_name)
    }

    /// Snapshot of all local topic names in this set.
    pub fn topic_names(&self) -> Vec<TopicName> {
        let topics = self.inner.topics.read();
        topics.keys().cloned().collect()
    }

    /// Number of topics in this set.
    pub fn len(&self) -> usize {
        let topics = self.inner.topics.read();
        topics.len()
    }

    /// Whether this set is empty.
    pub fn is_empty(&self) -> bool {
        let topics = self.inner.topics.read();
        topics.is_empty()
    }
}
