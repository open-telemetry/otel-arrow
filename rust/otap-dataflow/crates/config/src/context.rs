// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for referring to pipeline context entries.

use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A context entry reference in `entry` or `entry:member` form.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct ContextEntryRef(String);

impl ContextEntryRef {
    /// Parses and normalizes a context entry reference, which has two
    /// forms:
    ///
    /// 1. `entryname`
    /// 2. `composite:association
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let mut parts = raw.split(':');
        let entry = parts.next().unwrap_or_default();
        let member = parts.next();
        if entry.is_empty()
            || entry.chars().any(char::is_whitespace)
            || member
                .is_some_and(|member| member.is_empty() || member.chars().any(char::is_whitespace))
            || parts.next().is_some()
        {
            return Err(Error::InvalidUserConfig {
                error: format!(
                    "invalid context entry reference `{raw}`; expected `entry` or `entry:member`"
                ),
            });
        }
        Ok(Self(raw.to_ascii_lowercase()))
    }

    /// Returns the normalized reference.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the root entry name.
    #[must_use]
    pub fn entry(&self) -> &str {
        self.0
            .split_once(':')
            .map_or(self.0.as_str(), |(entry, _)| entry)
    }

    /// Returns the qualified member name, if present.
    #[must_use]
    pub fn member(&self) -> Option<&str> {
        self.0.split_once(':').map(|(_, member)| member)
    }
}

impl AsRef<str> for ContextEntryRef {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for ContextEntryRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for ContextEntryRef {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<ContextEntryRef> for String {
    fn from(value: ContextEntryRef) -> Self {
        value.0
    }
}

impl From<&'static str> for ContextEntryRef {
    fn from(value: &'static str) -> Self {
        Self::parse(value).expect("invalid static context entry reference")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Singleton and qualified context entry references are parsed.
    /// Guarantees: names are normalized and entry/member access is preserved.
    #[test]
    fn parses_supported_reference_forms() {
        let singleton = ContextEntryRef::parse("X-Tenant").unwrap();
        assert_eq!(singleton.as_str(), "x-tenant");
        assert_eq!(singleton.entry(), "x-tenant");
        assert_eq!(singleton.member(), None);

        let qualified = ContextEntryRef::parse("Product_User:Customer_ID").unwrap();
        assert_eq!(qualified.as_str(), "product_user:customer_id");
        assert_eq!(qualified.entry(), "product_user");
        assert_eq!(qualified.member(), Some("customer_id"));
    }

    /// Scenario: A context entry reference has an invalid structural form.
    /// Guarantees: empty, whitespace, and multiply-qualified names are rejected.
    #[test]
    fn rejects_invalid_reference_forms() {
        for invalid in [
            "",
            ":member",
            "entry:",
            "entry:member:extra",
            "entry member",
        ] {
            assert!(ContextEntryRef::parse(invalid).is_err(), "{invalid}");
        }
    }

    /// Scenario: A qualified context entry reference is deserialized from YAML.
    /// Guarantees: configuration uses the canonical normalized string form.
    #[test]
    fn serde_uses_string_form() {
        let parsed: ContextEntryRef = serde_yaml::from_str("Product:Customer").unwrap();
        assert_eq!(parsed.as_str(), "product:customer");
        assert_eq!(
            serde_yaml::to_string(&parsed).unwrap(),
            "product:customer\n"
        );
    }
}
