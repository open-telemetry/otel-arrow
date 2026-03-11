// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic handle is the user-facing publish/subscribe entry point.

use std::sync::Arc;

use crate::error::Error;
use crate::topic::backend::TopicState;
use crate::topic::subscription::Subscription;
use crate::topic::types::{
    PublishOutcome, SubscriberOptions, SubscriptionMode, TopicPublishOutcomeConfig,
    TrackedPublishPermit, TrackedPublishReceipt, TrackedTryPublishOutcome,
};
use otap_df_config::TopicName;
use otap_df_config::topic::TopicBroadcastOnLagPolicy;
use tokio::sync::Semaphore;

/// A handle to a topic, used for publishing and subscribing.
pub struct TopicHandle<T: Send + Sync + 'static> {
    inner: Arc<dyn TopicState<T>>,
    publish_outcome_default: TopicPublishOutcomeConfig,
}

impl<T: Send + Sync + 'static> Clone for TopicHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            publish_outcome_default: self.publish_outcome_default,
        }
    }
}

/// A tracked publisher that returns an explicit terminal outcome per publish.
pub struct TrackedTopicPublisher<T: Send + Sync + 'static> {
    handle: TopicHandle<T>,
    in_flight: Arc<Semaphore>,
    max_in_flight: usize,
    timeout: std::time::Duration,
}

impl<T: Send + Sync + 'static> Clone for TrackedTopicPublisher<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            in_flight: Arc::clone(&self.in_flight),
            max_in_flight: self.max_in_flight,
            timeout: self.timeout,
        }
    }
}

impl<T: Send + Sync + 'static> TopicHandle<T> {
    pub(crate) fn new(inner: Arc<dyn TopicState<T>>) -> Self {
        Self {
            inner,
            publish_outcome_default: TopicPublishOutcomeConfig::default(),
        }
    }

    /// Return a cloned handle with the topic-level default tracked publish outcome policy.
    #[must_use]
    pub fn with_default_publish_outcome_config(&self, config: TopicPublishOutcomeConfig) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            publish_outcome_default: config,
        }
    }

    /// Create a tracked publisher using the topic-level default outcome configuration.
    #[must_use]
    pub fn tracked_publisher(&self) -> TrackedTopicPublisher<T> {
        self.tracked_publisher_with_config(self.publish_outcome_default)
    }

    /// Create a tracked publisher using an explicit outcome configuration.
    #[must_use]
    pub fn tracked_publisher_with_config(
        &self,
        config: TopicPublishOutcomeConfig,
    ) -> TrackedTopicPublisher<T> {
        TrackedTopicPublisher {
            handle: self.clone(),
            in_flight: Arc::new(Semaphore::new(config.max_in_flight.max(1))),
            max_in_flight: config.max_in_flight.max(1),
            timeout: config.timeout,
        }
    }

    /// Publish a message to the topic. All balanced consumer groups and broadcast subscribers
    /// will be notified.
    ///
    /// This may await under backpressure when balanced consumer-group buffers are full.
    pub async fn publish(&self, msg: Arc<T>) -> Result<(), Error> {
        self.inner.publish(msg).await
    }

    /// Try to publish a message without awaiting under backpressure.
    pub fn try_publish(&self, msg: Arc<T>) -> Result<PublishOutcome, Error> {
        self.inner.try_publish(msg)
    }

    /// Subscribe to this topic.
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

    /// Topic-level default configuration for tracked publish outcomes.
    #[must_use]
    pub const fn default_publish_outcome_config(&self) -> TopicPublishOutcomeConfig {
        self.publish_outcome_default
    }

    /// Effective broadcast lag policy configured on this topic.
    #[must_use]
    pub fn broadcast_on_lag_policy(&self) -> TopicBroadcastOnLagPolicy {
        self.inner.broadcast_on_lag_policy()
    }
}

impl<T: Send + Sync + 'static> TrackedTopicPublisher<T> {
    /// Publish one message and return a receipt that resolves to the terminal
    /// outcome.
    ///
    /// This call waits for both topic admission and an available tracked
    /// in-flight slot. The tracked outcome timeout is configured on the
    /// publisher and starts once the topic accepts the publish.
    ///
    /// Dropping the returned receipt does not cancel tracking or release the
    /// in-flight slot early.
    pub async fn publish(&self, msg: Arc<T>) -> Result<TrackedPublishReceipt, Error> {
        let permit = self
            .in_flight
            .clone()
            .acquire_owned()
            .await
            .expect("tracked publisher semaphore should not close");
        self.handle
            .inner
            .publish_tracked(
                msg,
                self.timeout,
                TrackedPublishPermit::from_tokio_owned(permit),
            )
            .await
    }

    /// Try to publish one message without awaiting.
    ///
    /// Returns [`TrackedTryPublishOutcome::MaxInFlightReached`] when no tracked
    /// in-flight slot is currently available.
    pub fn try_publish(&self, msg: Arc<T>) -> Result<TrackedTryPublishOutcome, Error> {
        let permit = match self.in_flight.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return Ok(TrackedTryPublishOutcome::MaxInFlightReached),
        };
        self.handle.inner.try_publish_tracked(
            msg,
            self.timeout,
            TrackedPublishPermit::from_tokio_owned(permit),
        )
    }

    /// The topic that this tracked publisher sends to.
    #[must_use]
    pub fn topic(&self) -> &TopicName {
        self.handle.name()
    }

    /// The configured maximum number of unresolved tracked publishes.
    #[must_use]
    pub fn max_in_flight(&self) -> usize {
        self.max_in_flight
    }
}
