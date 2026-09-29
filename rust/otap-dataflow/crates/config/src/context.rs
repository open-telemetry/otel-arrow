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
    pub(crate) fn to_ascii_lowercase(&self) -> Self {
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
        if value.is_empty()
            || value.contains(':')
            || !value.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(Error::InvalidUserConfig {
                error: format!(
                    "invalid context entry name `{value}`; expected a single printable ASCII name"
                ),
            });
        }
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

/// The name of a context entry, one of two forms:
///
///  1. Single unqualified name like `X-Tenant-Id` which must resolve
///     to a regular non-composite context entry.
///  2. Qualified pair of names like `Customer:Workspace` which must
///     resolve to a composite entry named field.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct ContextEntryRef {
    scope: Option<ContextEntryName>,
    name: ContextEntryName,
}

impl ContextEntryRef {
    /// Returns the containing scope, if the name is qualified.
    #[must_use]
    pub fn scope(&self) -> Option<&ContextEntryName> {
        self.scope.as_ref()
    }

    /// Returns the referenced context entry name.
    #[must_use]
    pub fn name(&self) -> &ContextEntryName {
        &self.name
    }
}

impl From<ContextEntryName> for ContextEntryRef {
    fn from(name: ContextEntryName) -> Self {
        Self { scope: None, name }
    }
}

impl TryFrom<&str> for ContextEntryRef {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split(':');
        let first = ContextEntryName::try_from(parts.next().unwrap_or_default())?;
        let second = parts.next().map(ContextEntryName::try_from).transpose()?;
        match (second, parts.next()) {
            (None, None) => Ok(Self {
                scope: None,
                name: first,
            }),
            (Some(name), None) => Ok(Self {
                scope: Some(first),
                name,
            }),
            _ => Err(Error::InvalidUserConfig {
                error: format!(
                    "invalid context entry reference `{value}`; expected `name` or `scope:name`"
                ),
            }),
        }
    }
}

impl TryFrom<String> for ContextEntryRef {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl std::fmt::Display for ContextEntryRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(scope) = &self.scope {
            write!(f, "{scope}:")?;
        }
        self.name.fmt(f)
    }
}

impl From<ContextEntryRef> for String {
    fn from(value: ContextEntryRef) -> Self {
        value.to_string()
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

    /// Scenario: a context name is empty, composite, contains whitespace, or is non-ASCII.
    /// Guarantees: invalid context entry names are rejected.
    #[test]
    fn context_entry_name_rejects_invalid_forms() {
        for value in ["", "entry:member", "two words", "line\nbreak", "caf\u{e9}"] {
            assert!(ContextEntryName::try_from(value).is_err(), "{value:?}");
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

    /// Scenario: unqualified and scoped context entry references pass through parsing and serde.
    /// Guarantees: name spelling, case, and optional scope round trip exactly.
    #[test]
    fn context_entry_ref_round_trips_exactly() {
        let unqualified = ContextEntryRef::try_from("Customer_ID").expect("valid reference");
        assert_eq!(unqualified.scope(), None);
        assert_eq!(unqualified.name().as_str(), "Customer_ID");
        assert_eq!(unqualified.to_string(), "Customer_ID");

        let scoped =
            ContextEntryRef::try_from("Product_User:Customer_ID").expect("valid reference");
        assert_eq!(
            scoped.scope().map(ContextEntryName::as_str),
            Some("Product_User")
        );
        assert_eq!(scoped.name().as_str(), "Customer_ID");
        assert_eq!(scoped.to_string(), "Product_User:Customer_ID");
        assert_eq!(
            serde_json::to_string(&scoped).expect("serialize"),
            "\"Product_User:Customer_ID\""
        );
    }

    /// Scenario: JSON schema is generated for a context entry reference.
    /// Guarantees: the schema matches the string representation used by serde.
    #[test]
    fn context_entry_ref_schema_is_string() {
        let schema = serde_json::to_value(schemars::schema_for!(ContextEntryRef))
            .expect("schema should serialize");

        assert_eq!(schema["type"], "string");
    }

    /// Scenario: a context entry name is converted into a reference.
    /// Guarantees: conversion preserves the name without inventing a scope.
    #[test]
    fn context_entry_name_converts_to_unqualified_ref() {
        let reference =
            ContextEntryRef::from(ContextEntryName::try_from("tenant").expect("valid name"));

        assert_eq!(reference.scope(), None);
        assert_eq!(reference.name().as_str(), "tenant");
    }

    /// Scenario: a reference is empty, has an empty side, has extra separators, or is malformed.
    /// Guarantees: only exact `name` and `scope:name` forms are accepted.
    #[test]
    fn context_entry_ref_rejects_malformed_forms() {
        for configured in [
            "",
            ":field",
            "entry:",
            "entry:field:extra",
            "entry:two words",
            "entry:caf\u{e9}",
        ] {
            assert!(
                ContextEntryRef::try_from(configured).is_err(),
                "{configured:?}"
            );
        }
    }
}
