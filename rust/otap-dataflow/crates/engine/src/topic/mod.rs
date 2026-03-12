// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic module: broker, topic state, subscription, and related types.

mod backend;
mod binding;
mod broker;
mod handle;
mod subscription;
#[allow(clippy::module_inception)] // topic/topic.rs holds the core topic-state internals.
mod topic;
mod topic_set;
mod types;

#[cfg(test)]
mod tests;

pub use backend::{InMemoryBackend, SubscriptionBackend, TopicBackend, TopicState};
pub use binding::PipelineTopicBinding;
pub use broker::TopicBroker;
pub use handle::{TopicHandle, TrackedTopicPublisher};
pub use otap_df_config::topic::{
    TopicAckPropagationMode, TopicBroadcastOnLagPolicy, TopicQueueOnFullPolicy,
};
pub use otap_df_config::{SubscriptionGroupName, TopicName};
pub use subscription::Subscription;
pub use topic_set::TopicSet;
pub use types::{
    Envelope, PublishOutcome, RecvItem, SubscriberOptions, SubscriptionMode, TopicOptions,
    TopicPublishOutcomeConfig, TrackedPublishOutcome, TrackedPublishPermit, TrackedPublishReceipt,
    TrackedPublishTracker, TrackedTryPublishOutcome,
};
