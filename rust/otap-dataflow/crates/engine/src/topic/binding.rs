// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-scoped topic binding with resolved wiring defaults.

use std::ops::Deref;

use crate::topic::handle::TopicHandle;
use otap_df_config::topic::{TopicAckPropagationMode, TopicQueueOnFullPolicy};

/// Pipeline-scoped topic binding with resolved wiring defaults.
///
/// The binding wraps a pure [`TopicHandle`] with defaults that are resolved by
/// the controller for one pipeline instance, such as how topic exporters should
/// behave when balanced queues are full and whether topic receivers/exporters
/// should bridge Ack/Nack across the topic hop by default.
pub struct PipelineTopicBinding<T: Send + Sync + 'static> {
    handle: TopicHandle<T>,
    queue_on_full_default: TopicQueueOnFullPolicy,
    ack_propagation_mode_default: TopicAckPropagationMode,
}

impl<T: Send + Sync + 'static> Clone for PipelineTopicBinding<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            queue_on_full_default: self.queue_on_full_default.clone(),
            ack_propagation_mode_default: self.ack_propagation_mode_default,
        }
    }
}

impl<T: Send + Sync + 'static> PipelineTopicBinding<T> {
    /// Create a binding with the default pipeline wiring policies.
    #[must_use]
    pub fn new(handle: TopicHandle<T>) -> Self {
        Self {
            handle,
            queue_on_full_default: TopicQueueOnFullPolicy::Block,
            ack_propagation_mode_default: TopicAckPropagationMode::Disabled,
        }
    }

    /// Return a cloned binding with the topic-level default full-queue policy.
    #[must_use]
    pub fn with_default_queue_on_full(&self, policy: TopicQueueOnFullPolicy) -> Self {
        Self {
            handle: self.handle.clone(),
            queue_on_full_default: policy,
            ack_propagation_mode_default: self.ack_propagation_mode_default,
        }
    }

    /// Return a cloned binding with the topic-level default Ack/Nack propagation mode.
    #[must_use]
    pub fn with_default_ack_propagation_mode(&self, mode: TopicAckPropagationMode) -> Self {
        Self {
            handle: self.handle.clone(),
            queue_on_full_default: self.queue_on_full_default.clone(),
            ack_propagation_mode_default: mode,
        }
    }

    /// The underlying pure topic runtime handle.
    #[must_use]
    pub const fn handle(&self) -> &TopicHandle<T> {
        &self.handle
    }

    /// Consume the binding and return the underlying topic handle.
    #[must_use]
    pub fn into_handle(self) -> TopicHandle<T> {
        self.handle
    }

    /// Pipeline-level default behavior when the queue is full.
    #[must_use]
    pub fn default_queue_on_full(&self) -> TopicQueueOnFullPolicy {
        self.queue_on_full_default.clone()
    }

    /// Pipeline-level default mode for cross-pipeline Ack/Nack propagation.
    #[must_use]
    pub const fn default_ack_propagation_mode(&self) -> TopicAckPropagationMode {
        self.ack_propagation_mode_default
    }
}

impl<T: Send + Sync + 'static> From<TopicHandle<T>> for PipelineTopicBinding<T> {
    fn from(value: TopicHandle<T>) -> Self {
        Self::new(value)
    }
}

impl<T: Send + Sync + 'static> Deref for PipelineTopicBinding<T> {
    type Target = TopicHandle<T>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
