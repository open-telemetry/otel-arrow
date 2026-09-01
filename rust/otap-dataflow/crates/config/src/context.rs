// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for referring to transport-header context entries.

use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A normalized transport-header context entry reference.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct ContextEntryRef(String);

impl ContextEntryRef {
    /// Parses and normalizes a transport-header context entry reference.
    pub fn parse(raw: &str) -> Result<Self, Error> {
        if raw.is_empty() || raw.contains(':') || !raw.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(Error::InvalidUserConfig {
                error: format!(
                    "invalid transport-header context entry reference `{raw}`; expected a single printable ASCII name"
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

    /// Scenario: A transport-header context entry reference is parsed.
    /// Guarantees: the header name is normalized to lowercase.
    #[test]
    fn parses_transport_header_reference() {
        let reference = ContextEntryRef::parse("X-Tenant").unwrap();
        assert_eq!(reference.as_str(), "x-tenant");

        let punctuation = ContextEntryRef::parse("Tenant/Region@1").unwrap();
        assert_eq!(punctuation.as_str(), "tenant/region@1");
    }

    /// Scenario: A transport-header reference has an unsupported form.
    /// Guarantees: empty, non-ASCII, whitespace, and composite names are rejected.
    #[test]
    fn rejects_invalid_reference_forms() {
        for invalid in ["", "entry:member", "entry member", "t\u{e9}nant"] {
            assert!(ContextEntryRef::parse(invalid).is_err(), "{invalid}");
        }
    }

    /// Scenario: A transport-header context entry reference is deserialized from YAML.
    /// Guarantees: configuration uses the canonical normalized string form.
    #[test]
    fn serde_uses_string_form() {
        let parsed: ContextEntryRef = serde_yaml::from_str("X-Tenant").unwrap();
        assert_eq!(parsed.as_str(), "x-tenant");
        assert_eq!(serde_yaml::to_string(&parsed).unwrap(), "x-tenant\n");
    }
}
