// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Names for context entries.

use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A context entry name.
///
/// The configured spelling is preserved. Equality, ordering, and hashing are
/// case-sensitive; callers that need transport-header matching semantics must
/// compare names case-insensitively at that boundary.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(try_from = "String", into = "String")]
pub struct ContextEntryName(Box<str>);

impl ContextEntryName {
    /// Returns the configured name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns an ASCII-lowercase copy.
    #[must_use]
    pub fn to_ascii_lowercase(&self) -> Self {
        Self(self.0.to_ascii_lowercase().into())
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
        Ok(Self(value.into()))
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
    /// Guarantees: construction and string conversion preserve the configured spelling.
    #[test]
    fn context_entry_name_preserves_and_converts() {
        let name = ContextEntryName::try_from("X-Tenant_Id.1".to_owned()).expect("valid name");

        assert_eq!(name.as_str(), "X-Tenant_Id.1");
        assert_eq!(name.as_ref(), "X-Tenant_Id.1");
        assert_eq!(name.to_string(), "X-Tenant_Id.1");

        let owned: String = name.into();
        assert_eq!(owned, "X-Tenant_Id.1");
    }

    /// Scenario: context names differ only by ASCII case.
    /// Guarantees: their stored identities remain distinct.
    #[test]
    fn context_entry_name_identity_is_case_sensitive() {
        use std::collections::{BTreeSet, HashSet};

        let upper = ContextEntryName::try_from("X-Tenant").expect("valid name");
        let lower = ContextEntryName::try_from("x-tenant").expect("valid name");

        assert_ne!(upper, lower);
        assert_eq!(HashSet::from([upper.clone(), lower.clone()]).len(), 2);
        assert_eq!(BTreeSet::from([upper, lower]).len(), 2);
    }

    /// Scenario: existing configuration uses unrestricted context names.
    /// Guarantees: wrapping a name does not introduce new validation failures.
    #[test]
    fn context_entry_name_preserves_existing_input_domain() {
        for value in ["", "two words", "line\nbreak", "caf\u{e9}"] {
            let name = ContextEntryName::try_from(value).expect("existing names remain accepted");
            assert_eq!(name.as_str(), value);
        }
    }

    /// Scenario: a mixed-case name passes through serde.
    /// Guarantees: serde preserves the configured spelling.
    #[test]
    fn context_entry_name_serde_preserves_string() {
        let name: ContextEntryName = serde_json::from_str("\"X-Tenant\"").expect("deserialize");

        assert_eq!(name, "X-Tenant");
        assert_eq!(
            serde_json::to_string(&name).expect("serialize"),
            "\"X-Tenant\""
        );
    }
}
