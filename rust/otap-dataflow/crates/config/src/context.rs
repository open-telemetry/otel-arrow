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

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a context entry name contains supported mixed-case ASCII characters.
    /// Guarantees: construction normalizes the name and preserves its string representation.
    #[test]
    fn context_entry_name_normalizes_and_converts() {
        let name = ContextEntryName::try_from("X-Tenant_Id.1".to_owned()).expect("valid name");

        assert_eq!(name.as_str(), "x-tenant_id.1");
        assert_eq!(name.as_ref(), "x-tenant_id.1");
        assert_eq!(name.to_string(), "x-tenant_id.1");

        let owned: String = name.into();
        assert_eq!(owned, "x-tenant_id.1");
    }

    /// Scenario: a context entry name is empty, contains whitespace, or contains non-ASCII text.
    /// Guarantees: construction rejects invalid names with a user configuration error.
    #[test]
    fn context_entry_name_rejects_invalid_input() {
        for invalid in ["", "two words", "line\nbreak", "caf\u{e9}"] {
            let error = ContextEntryName::try_from(invalid).expect_err("invalid name");
            assert!(matches!(error, Error::InvalidUserConfig { .. }));
        }
    }

    /// Scenario: a mixed-case context entry name is deserialized and serialized.
    /// Guarantees: serde validates and emits the normalized name.
    #[test]
    fn context_entry_name_serde_uses_normalized_string() {
        let name: ContextEntryName = serde_json::from_str("\"X-Tenant\"").expect("deserialize");

        assert_eq!(name, "x-tenant");
        assert_eq!(
            serde_json::to_string(&name).expect("serialize"),
            "\"x-tenant\""
        );
    }
}
