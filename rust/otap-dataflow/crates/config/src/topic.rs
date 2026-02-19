// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic declarations for inter-pipeline communication.

use crate::Description;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A named topic specification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct TopicSpec {
    /// Optional human-readable description of the topic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
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

/// Policies supported for topics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TopicPolicies {
    /// Maximum number of messages retained in-memory for this topic queue.
    #[serde(default = "default_topic_queue_capacity")]
    pub queue_capacity: usize,
    /// Behavior when `queue_capacity` is reached while publishing a new message.
    #[serde(default)]
    pub queue_on_full: TopicQueueOnFullPolicy,
}

impl Default for TopicPolicies {
    fn default() -> Self {
        Self {
            queue_capacity: default_topic_queue_capacity(),
            queue_on_full: TopicQueueOnFullPolicy::default(),
        }
    }
}

impl TopicPolicies {
    /// Returns validation errors for this policy set.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if self.queue_capacity == 0 {
            errors.push(format!(
                "{path_prefix}.queue_capacity must be greater than 0"
            ));
        }
        errors
    }
}

/// Behavior when queue reaches `queue_capacity`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TopicQueueOnFullPolicy {
    /// Drop the incoming item and keep queued items untouched.
    DropNewest,
    /// Block the publisher until queue space is available.
    #[default]
    Block,
}

const fn default_topic_queue_capacity() -> usize {
    128
}

#[cfg(test)]
mod tests {
    use super::{TopicQueueOnFullPolicy, TopicSpec};

    #[test]
    fn defaults_match_expected_values() {
        let topic = TopicSpec::default();
        assert_eq!(topic.policies.queue_capacity, 128);
        assert_eq!(topic.policies.queue_on_full, TopicQueueOnFullPolicy::Block);
    }

    #[test]
    fn validates_non_zero_queue_capacity() {
        let mut topic = TopicSpec::default();
        topic.policies.queue_capacity = 0;

        let errors = topic.validation_errors("topics.raw");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains(".queue_capacity"));
    }

    #[test]
    fn deserializes_queue_on_full_policy_values() {
        let yaml = r#"
policies:
  queue_capacity: 1
  queue_on_full: drop_newest
"#;

        let topic: TopicSpec = serde_yaml::from_str(yaml).expect("topic should parse");
        assert_eq!(
            topic.policies.queue_on_full,
            TopicQueueOnFullPolicy::DropNewest
        );
    }
}
