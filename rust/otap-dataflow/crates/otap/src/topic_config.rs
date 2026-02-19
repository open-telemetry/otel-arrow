// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared value objects for topic node configuration.

use serde::{Deserialize, Serialize};

/// Topic identifier used by topic receiver/exporter configs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct TopicName(String);

impl TopicName {
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

impl std::fmt::Display for TopicName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for TopicName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("topic name must be non-empty".to_owned());
        }
        Ok(Self(value))
    }
}

impl From<TopicName> for String {
    fn from(value: TopicName) -> Self {
        value.0
    }
}

/// Balanced subscription group identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct GroupName(String);

impl GroupName {
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

impl AsRef<str> for GroupName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for GroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for GroupName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("subscription group name must be non-empty".to_owned());
        }
        Ok(Self(value))
    }
}

impl From<GroupName> for String {
    fn from(value: GroupName) -> Self {
        value.0
    }
}
