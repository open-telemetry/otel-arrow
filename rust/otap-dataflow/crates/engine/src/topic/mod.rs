// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic module: broker, topic state, subscription, and related types.

mod backend;
mod broker;
mod handle;
mod subscription;
mod topic;
mod topic_set;
mod types;

#[cfg(test)]
mod tests;

pub use broker::TopicBroker;
pub use handle::TopicHandle;
pub use subscription::Subscription;
pub use topic_set::TopicSet;
pub use types::{
    AckEvent, AckStatus, Envelope, RecvItem, SubscriberOptions, SubscriptionMode, TopicMode,
    TopicOptions,
};
