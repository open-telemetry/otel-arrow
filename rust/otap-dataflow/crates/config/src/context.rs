// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Context entry references and compiler primitives for global context registers.

use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A context entry reference is a string that is resolved to a
/// context register name. Always normalized.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(try_from = "String", into = "String")]
pub struct ContextEntryName(Box<str>);

impl ContextEntryName {
    /// Returns the name of the context entry, e.g., the value
    /// in the `store_as` field of a transport header capture.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ContextEntryName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::ops::Deref for ContextEntryName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl std::fmt::Display for ContextEntryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for ContextEntryName {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(Error::InvalidUserConfig {
                error: format!(
                    "invalid transport-header context entry reference `{value}`; expected a single printable ASCII name"
                ),
            });
        }
        Ok(Self(value.to_ascii_lowercase().into()))
    }
}

impl TryFrom<String> for ContextEntryName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl From<ContextEntryName> for String {
    fn from(value: ContextEntryName) -> Self {
        value.0.into()
    }
}

/// Tests are allowed to compare against bare strings.
#[cfg(test)]
impl PartialEq<str> for ContextEntryName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

/// Tests are allowed to compare against bare strings.
#[cfg(test)]
impl PartialEq<&str> for ContextEntryName {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
