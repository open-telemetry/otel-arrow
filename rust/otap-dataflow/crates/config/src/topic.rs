// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic declarations for inter-pipeline communication.

use crate::Description;
use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::time::Duration;

/// Name of a topic declaration/reference.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct TopicName(String);

impl TopicName {
    /// Parses and validates a topic name.
    pub fn parse(raw: &str) -> Result<Self, Error> {
        if raw.trim().is_empty() {
            return Err(Error::TopicNameEmpty);
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the topic name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the owned topic name.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for TopicName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for TopicName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for TopicName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for TopicName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl From<TopicName> for String {
    fn from(value: TopicName) -> Self {
        value.0
    }
}

impl From<TopicName> for Cow<'static, str> {
    fn from(value: TopicName) -> Self {
        Cow::Owned(value.0)
    }
}

impl From<&TopicName> for Cow<'static, str> {
    fn from(value: &TopicName) -> Self {
        Cow::Owned(value.0.clone())
    }
}

impl From<&'static str> for TopicName {
    fn from(value: &'static str) -> Self {
        Self::parse(value).expect("invalid static topic name literal")
    }
}

/// Name of a balanced-subscription consumer group.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct SubscriptionGroupName(String);

impl SubscriptionGroupName {
    /// Parses and validates a subscription group name.
    pub fn parse(raw: &str) -> Result<Self, Error> {
        if raw.trim().is_empty() {
            return Err(Error::SubscriptionGroupNameEmpty);
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the group name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the owned group name.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for SubscriptionGroupName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for SubscriptionGroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for SubscriptionGroupName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl From<SubscriptionGroupName> for String {
    fn from(value: SubscriptionGroupName) -> Self {
        value.0
    }
}

impl From<&'static str> for SubscriptionGroupName {
    fn from(value: &'static str) -> Self {
        Self::parse(value).expect("invalid static subscription group name literal")
    }
}

/// A named topic specification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct TopicSpec {
    /// Optional human-readable description of the topic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    /// Backend implementation used by this topic.
    ///
    /// Defaults to `in_memory`.
    #[serde(default)]
    pub backend: TopicBackendKind,
    /// Optional override for topic implementation selection.
    ///
    /// If omitted, the engine-wide default (`engine.topics.impl_selection`) applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impl_selection: Option<TopicImplSelectionPolicy>,
    /// Topic behavior policies.
    #[serde(default)]
    pub policies: TopicPolicies,
}

impl TopicSpec {
    /// Returns validation errors for this topic specification.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        self.policies
            .validation_errors(&format!("{path_prefix}.policies"))
    }
}

/// Supported backend kinds for topic declarations.
///
/// The engine currently supports `in_memory`. Other variants are accepted in
/// configuration to make backend selection explicit and forward-compatible.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TopicBackendKind {
    /// Built-in in-memory topic backend.
    #[default]
    InMemory,
    /// Reserved for a future Quiver-backed implementation.
    Quiver,
}

impl std::fmt::Display for TopicBackendKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::InMemory => "in_memory",
            Self::Quiver => "quiver",
        };
        f.write_str(value)
    }
}

/// Policy controlling how the runtime selects the topic implementation variant.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TopicImplSelectionPolicy {
    /// Automatically infer the most efficient implementation from topology.
    #[default]
    Auto,
    /// Disable optimization and always use the mixed implementation.
    ForceMixed,
}

impl std::fmt::Display for TopicImplSelectionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Auto => "auto",
            Self::ForceMixed => "force_mixed",
        };
        f.write_str(value)
    }
}

/// Policies supported for topics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct TopicPolicies {
    /// Policies for balanced delivery paths.
    #[serde(default)]
    pub balanced: TopicBalancedPolicies,
    /// Policies for broadcast delivery paths.
    #[serde(default)]
    pub broadcast: TopicBroadcastPolicies,
    /// Policy controlling cross-pipeline Ack/Nack propagation over topic hops.
    #[serde(default)]
    pub ack_propagation: TopicAckPropagationPolicies,
}

impl TopicPolicies {
    /// Returns validation errors for this policy set.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if self.balanced.queue_capacity == 0 {
            errors.push(format!(
                "{path_prefix}.balanced.queue_capacity must be greater than 0"
            ));
        }
        if self.broadcast.queue_capacity == 0 {
            errors.push(format!(
                "{path_prefix}.broadcast.queue_capacity must be greater than 0"
            ));
        }
        if self.ack_propagation.max_in_flight == 0 {
            errors.push(format!(
                "{path_prefix}.ack_propagation.max_in_flight must be greater than 0"
            ));
        }
        errors
    }
}

/// Policies for balanced delivery paths.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TopicBalancedPolicies {
    /// Maximum number of messages retained in-memory for each balanced consumer group queue.
    #[serde(default = "default_topic_balanced_queue_capacity")]
    pub queue_capacity: usize,
    /// Behavior when a balanced queue reaches `queue_capacity`.
    #[serde(default)]
    pub on_full: TopicQueueOnFullPolicy,
}

impl Default for TopicBalancedPolicies {
    fn default() -> Self {
        Self {
            queue_capacity: default_topic_balanced_queue_capacity(),
            on_full: TopicQueueOnFullPolicy::default(),
        }
    }
}

/// Policies for broadcast delivery paths.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TopicBroadcastPolicies {
    /// Maximum number of messages retained in-memory in the broadcast ring.
    #[serde(default = "default_topic_broadcast_queue_capacity")]
    pub queue_capacity: usize,
    /// Behavior when a broadcast subscriber falls behind the retained ring window.
    #[serde(default)]
    pub on_lag: TopicBroadcastOnLagPolicy,
}

impl Default for TopicBroadcastPolicies {
    fn default() -> Self {
        Self {
            queue_capacity: default_topic_broadcast_queue_capacity(),
            on_lag: TopicBroadcastOnLagPolicy::default(),
        }
    }
}

/// Policies for cross-pipeline Ack/Nack propagation over topic hops.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TopicAckPropagationPolicies {
    /// Ack/Nack propagation mode for the topic hop.
    #[serde(default)]
    pub mode: TopicAckPropagationMode,
    /// Maximum number of unresolved tracked publish outcomes per publisher handle.
    #[serde(default = "default_topic_ack_propagation_max_in_flight")]
    pub max_in_flight: usize,
    /// Maximum time to wait before a tracked publish outcome is timed out.
    #[serde(
        default = "default_topic_ack_propagation_timeout",
        with = "humantime_serde"
    )]
    #[schemars(with = "String")]
    pub timeout: Duration,
}

impl Default for TopicAckPropagationPolicies {
    fn default() -> Self {
        Self {
            mode: TopicAckPropagationMode::default(),
            max_in_flight: default_topic_ack_propagation_max_in_flight(),
            timeout: default_topic_ack_propagation_timeout(),
        }
    }
}

/// Behavior when a balanced queue reaches `queue_capacity`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TopicQueueOnFullPolicy {
    /// Drop the incoming item and keep queued items untouched.
    DropNewest,
    /// Block the publisher until queue space is available.
    #[default]
    Block,
}

/// Behavior when a broadcast subscriber falls behind the retained ring window.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TopicBroadcastOnLagPolicy {
    /// Skip dropped messages and continue from the oldest retained message.
    #[default]
    DropOldest,
    /// Disconnect the lagging subscriber after reporting the lag event.
    Disconnect,
}

/// Policy controlling whether topic hops can bridge Ack/Nack across pipelines.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TopicAckPropagationMode {
    /// Disable cross-pipeline Ack/Nack propagation.
    #[default]
    Disabled,
    /// Enable adaptive propagation only for messages carrying Ack/Nack interests.
    Auto,
}

const fn default_topic_balanced_queue_capacity() -> usize {
    128
}

const fn default_topic_broadcast_queue_capacity() -> usize {
    128
}

const fn default_topic_ack_propagation_max_in_flight() -> usize {
    1024
}

const fn default_topic_ack_propagation_timeout() -> Duration {
    Duration::from_secs(30)
}

#[cfg(test)]
mod tests {
    use super::{
        SubscriptionGroupName, TopicAckPropagationMode, TopicBackendKind,
        TopicBroadcastOnLagPolicy, TopicImplSelectionPolicy, TopicName, TopicQueueOnFullPolicy,
        TopicSpec,
    };
    use crate::error::Error;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn defaults_match_expected_values() {
        let topic = TopicSpec::default();
        assert_eq!(topic.backend, TopicBackendKind::InMemory);
        assert_eq!(topic.impl_selection, None);
        assert_eq!(topic.policies.balanced.queue_capacity, 128);
        assert_eq!(topic.policies.broadcast.queue_capacity, 128);
        assert_eq!(
            topic.policies.ack_propagation.mode,
            TopicAckPropagationMode::Disabled
        );
        assert_eq!(topic.policies.ack_propagation.max_in_flight, 1024);
        assert_eq!(
            topic.policies.ack_propagation.timeout,
            Duration::from_secs(30)
        );
        assert_eq!(
            topic.policies.broadcast.on_lag,
            TopicBroadcastOnLagPolicy::DropOldest
        );
        assert_eq!(
            topic.policies.balanced.on_full,
            TopicQueueOnFullPolicy::Block
        );
    }

    #[test]
    fn validates_non_zero_topic_queue_capacities() {
        let mut topic = TopicSpec::default();
        topic.policies.balanced.queue_capacity = 0;
        topic.policies.broadcast.queue_capacity = 0;
        topic.policies.ack_propagation.max_in_flight = 0;

        let errors = topic.validation_errors("topics.raw");
        assert_eq!(errors.len(), 3);
        assert!(errors[0].contains(".balanced.queue_capacity"));
        assert!(errors[1].contains(".broadcast.queue_capacity"));
        assert!(errors[2].contains(".ack_propagation.max_in_flight"));
    }

    #[test]
    fn deserializes_topic_policy_values() {
        let yaml = r#"
backend: in_memory
policies:
  balanced:
    queue_capacity: 1
    on_full: drop_newest
  broadcast:
    queue_capacity: 2
    on_lag: disconnect
  ack_propagation:
    mode: auto
    max_in_flight: 3
    timeout: 45s
"#;

        let topic: TopicSpec = serde_yaml::from_str(yaml).expect("topic should parse");
        assert_eq!(topic.policies.balanced.queue_capacity, 1);
        assert_eq!(topic.policies.broadcast.queue_capacity, 2);
        assert_eq!(
            topic.policies.ack_propagation.mode,
            TopicAckPropagationMode::Auto
        );
        assert_eq!(topic.policies.ack_propagation.max_in_flight, 3);
        assert_eq!(
            topic.policies.ack_propagation.timeout,
            Duration::from_secs(45)
        );
        assert_eq!(
            topic.policies.broadcast.on_lag,
            TopicBroadcastOnLagPolicy::Disconnect
        );
        assert_eq!(
            topic.policies.balanced.on_full,
            TopicQueueOnFullPolicy::DropNewest
        );
    }

    #[test]
    fn rejects_legacy_queue_capacity_field() {
        let yaml = r#"
backend: in_memory
policies:
  queue_capacity: 1
"#;

        let err = serde_yaml::from_str::<TopicSpec>(yaml).expect_err("legacy field should fail");
        assert!(err.to_string().contains("queue_capacity"));
    }

    #[test]
    fn rejects_flat_balanced_and_broadcast_policy_fields() {
        let yaml = r#"
backend: in_memory
policies:
  balanced_queue_capacity: 1
  broadcast_queue_capacity: 2
  queue_on_full: drop_newest
"#;

        let err =
            serde_yaml::from_str::<TopicSpec>(yaml).expect_err("flat policy fields should fail");
        let rendered = err.to_string();
        assert!(rendered.contains("balanced_queue_capacity") || rendered.contains("queue_on_full"));
    }

    #[test]
    fn rejects_scalar_ack_propagation_value() {
        let yaml = r#"
backend: in_memory
policies:
  ack_propagation: auto
"#;

        let err = serde_yaml::from_str::<TopicSpec>(yaml)
            .expect_err("scalar ack_propagation should fail");
        assert!(err.to_string().contains("ack_propagation"));
    }

    #[test]
    fn deserializes_topic_backend_kind() {
        let yaml = r#"
backend: quiver
"#;

        let topic: TopicSpec = serde_yaml::from_str(yaml).expect("topic should parse");
        assert_eq!(topic.backend, TopicBackendKind::Quiver);
    }

    #[test]
    fn deserializes_topic_impl_selection_policy() {
        let yaml = r#"
impl_selection: force_mixed
"#;

        let topic: TopicSpec = serde_yaml::from_str(yaml).expect("topic should parse");
        assert_eq!(
            topic.impl_selection,
            Some(TopicImplSelectionPolicy::ForceMixed)
        );
    }

    #[test]
    fn topic_name_rejects_empty_values() {
        let err = TopicName::parse("   ").expect_err("empty topic names should fail");
        assert!(matches!(err, Error::TopicNameEmpty));
    }

    #[test]
    fn subscription_group_name_rejects_empty_values() {
        let err = SubscriptionGroupName::parse("   ").expect_err("empty group names should fail");
        assert!(matches!(err, Error::SubscriptionGroupNameEmpty));
    }

    #[test]
    fn topic_name_supports_hash_map_lookup_by_str() {
        #[derive(Debug, Deserialize)]
        struct TopicsDoc {
            topics: HashMap<TopicName, TopicSpec>,
        }

        let yaml = r#"
topics:
  raw:
    policies:
      balanced:
        queue_capacity: 1
      broadcast:
        queue_capacity: 1
"#;

        let doc: TopicsDoc = serde_yaml::from_str(yaml).expect("topics should parse");
        assert!(doc.topics.contains_key("raw"));
    }
}
