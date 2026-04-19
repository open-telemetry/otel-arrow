// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension URN parsing and validation.
//!
//! Extension URNs use a simpler 3-segment format (`urn:<namespace>:<id>`)
//! compared to the 4-segment node URN format (`urn:<namespace>:<kind>:<id>`).
//! The `<kind>` segment is unnecessary because extensions are always
//! identified by their position in the `extensions:` config section.

use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Extension type URN with zero-copy access to namespace and id segments.
///
/// Format: `urn:<namespace>:<id>` (e.g., `urn:otap:sample_kv_store`)
///
/// Unlike [`NodeUrn`](crate::node_urn::NodeUrn), extension URNs have no
/// `<kind>` segment — the kind is implicit from the `extensions:` config
/// section.
///
/// Short form `<id>` is also accepted and expanded to `urn:otel:<id>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct ExtensionUrn {
    raw: String,
    namespace_range: Range<usize>,
    id_range: Range<usize>,
}

impl ExtensionUrn {
    /// Returns the canonical URN string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Returns the namespace segment.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.raw[self.namespace_range.clone()]
    }

    /// Returns the id segment.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.raw[self.id_range.clone()]
    }

    /// Parses an extension URN.
    ///
    /// Accepted formats:
    /// - `urn:<namespace>:<id>` (canonical)
    /// - `<id>` (short form, expanded to `urn:otel:<id>`)
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let raw = raw.trim();
        let parts: Vec<&str> = raw.split(':').collect();

        match parts.as_slice() {
            // Short form: just <id> → urn:otel:<id>
            [id] => {
                validate_segment(raw, id, "id")?;
                Ok(build_extension_urn("otel", id))
            }
            // Full form: urn:<namespace>:<id>
            [scheme, namespace, id] => {
                if !scheme.eq_ignore_ascii_case("urn") {
                    return Err(Error::InvalidUserConfig {
                        error: format!(
                            "Invalid extension URN `{raw}`: expected `urn:<namespace>:<id>` \
                             or `<id>`"
                        ),
                    });
                }
                validate_segment(raw, namespace, "namespace")?;
                validate_segment(raw, id, "id")?;
                let namespace = namespace.to_ascii_lowercase();
                Ok(build_extension_urn(&namespace, id))
            }
            _ => Err(Error::InvalidUserConfig {
                error: format!(
                    "Invalid extension URN `{raw}`: expected `urn:<namespace>:<id>` or `<id>`"
                ),
            }),
        }
    }
}

fn validate_segment(raw: &str, segment: &str, label: &str) -> Result<(), Error> {
    if segment.is_empty() {
        return Err(Error::InvalidUserConfig {
            error: format!("Invalid extension URN `{raw}`: {label} segment is empty"),
        });
    }
    if !segment
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(Error::InvalidUserConfig {
            error: format!(
                "Invalid extension URN `{raw}`: {label} segment `{segment}` \
                 contains invalid characters (only alphanumeric, `_`, `-` allowed)"
            ),
        });
    }
    Ok(())
}

fn build_extension_urn(namespace: &str, id: &str) -> ExtensionUrn {
    let raw = format!("urn:{namespace}:{id}");
    let ns_start = 4; // "urn:".len()
    let ns_end = ns_start + namespace.len();
    let id_start = ns_end + 1; // ":"
    let id_end = id_start + id.len();
    ExtensionUrn {
        raw,
        namespace_range: ns_start..ns_end,
        id_range: id_start..id_end,
    }
}

impl std::fmt::Display for ExtensionUrn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for ExtensionUrn {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<ExtensionUrn> for String {
    fn from(value: ExtensionUrn) -> Self {
        value.raw
    }
}

impl TryFrom<String> for ExtensionUrn {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<&'static str> for ExtensionUrn {
    fn from(value: &'static str) -> Self {
        Self::parse(value).unwrap_or_else(|e| panic!("invalid extension URN `{value}`: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_urn() {
        let urn = ExtensionUrn::parse("urn:otap:sample_kv_store").unwrap();
        assert_eq!(urn.namespace(), "otap");
        assert_eq!(urn.id(), "sample_kv_store");
        assert_eq!(urn.as_str(), "urn:otap:sample_kv_store");
    }

    #[test]
    fn test_parse_short_form() {
        let urn = ExtensionUrn::parse("my_auth").unwrap();
        assert_eq!(urn.namespace(), "otel");
        assert_eq!(urn.id(), "my_auth");
        assert_eq!(urn.as_str(), "urn:otel:my_auth");
    }

    #[test]
    fn test_parse_case_insensitive_scheme() {
        let urn = ExtensionUrn::parse("URN:Microsoft:azure_auth").unwrap();
        assert_eq!(urn.namespace(), "microsoft");
        assert_eq!(urn.id(), "azure_auth");
    }

    #[test]
    fn test_parse_rejects_node_urn_format() {
        assert!(ExtensionUrn::parse("urn:otap:extension:sample_kv_store").is_err());
    }

    #[test]
    fn test_parse_rejects_empty_id() {
        assert!(ExtensionUrn::parse("").is_err());
    }

    #[test]
    fn test_parse_rejects_invalid_chars() {
        assert!(ExtensionUrn::parse("urn:otap:bad name").is_err());
    }

    #[test]
    fn test_from_static_str() {
        let urn: ExtensionUrn = "urn:otap:test_ext".into();
        assert_eq!(urn.id(), "test_ext");
    }

    #[test]
    fn test_serde_roundtrip() {
        let urn = ExtensionUrn::parse("urn:otap:my_ext").unwrap();
        let json = serde_json::to_string(&urn).unwrap();
        assert_eq!(json, "\"urn:otap:my_ext\"");
        let parsed: ExtensionUrn = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, urn);
    }
}
