// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Names for context entries.

use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A context entry name.
///
/// Names are normalized to ASCII lowercase during construction so equality,
/// ordering, and hashing follow transport-header name semantics.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(try_from = "String", into = "String")]
pub struct ContextEntryName(Box<str>);

impl ContextEntryName {
    /// Returns the canonical lowercase name.
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
        if value.is_empty()
            || value.contains(':')
            || !value.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(Error::InvalidUserConfig {
                error: format!(
                    "invalid transport-header context entry name `{value}`; expected a single printable ASCII name"
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

impl PartialEq<str> for ContextEntryName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for ContextEntryName {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a context name contains mixed-case ASCII.
    /// Guarantees: construction and string conversion expose its canonical lowercase identity.
    #[test]
    fn context_entry_name_normalizes_and_converts() {
        let name = ContextEntryName::try_from("X-Tenant_Id.1".to_owned()).expect("valid name");

        assert_eq!(name.as_str(), "x-tenant_id.1");
        assert_eq!(name.as_ref(), "x-tenant_id.1");
        assert_eq!(name.to_string(), "x-tenant_id.1");

        let owned: String = name.into();
        assert_eq!(owned, "x-tenant_id.1");
    }

    /// Scenario: context names differ only by ASCII case.
    /// Guarantees: they normalize to the same stored identity.
    #[test]
    fn context_entry_name_identity_is_case_insensitive() {
        use std::collections::{BTreeSet, HashSet};

        let upper = ContextEntryName::try_from("X-Tenant").expect("valid name");
        let lower = ContextEntryName::try_from("x-tenant").expect("valid name");

        assert_eq!(upper, lower);
        assert_eq!(HashSet::from([upper.clone(), lower.clone()]).len(), 1);
        assert_eq!(BTreeSet::from([upper, lower]).len(), 1);
    }

    /// Scenario: a context name is empty, composite, contains whitespace, or is non-ASCII.
    /// Guarantees: invalid context entry names are rejected.
    #[test]
    fn context_entry_name_rejects_invalid_forms() {
        for value in ["", "entry:member", "two words", "line\nbreak", "caf\u{e9}"] {
            assert!(ContextEntryName::try_from(value).is_err(), "{value:?}");
        }
    }

    /// Scenario: a mixed-case name passes through serde.
    /// Guarantees: serde exposes the canonical lowercase identity.
    #[test]
    fn context_entry_name_serde_normalizes_string() {
        let name: ContextEntryName = serde_json::from_str("\"X-Tenant\"").expect("deserialize");

        assert_eq!(name, "x-tenant");
        assert_eq!(
            serde_json::to_string(&name).expect("serialize"),
            "\"x-tenant\""
        );
    }
}
