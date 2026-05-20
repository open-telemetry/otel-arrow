// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension URN parsing and validation.
//!
//! Extension URNs follow the canonical form `urn:<namespace>:extension:<id>`,
//! mirroring the `urn:<namespace>:<kind>:<id>` shape used by node URNs
//! (with the kind segment fixed to the literal `extension`). The shortcut
//! `extension:<id>` expands to `urn:otel:extension:<id>`.
//!
//! Extensions are intentionally NOT modeled as a node kind, and
//! [`ExtensionUrn`] is a distinct type from [`crate::node_urn::NodeUrn`]
//! so the two cannot be confused. The underlying parsing primitives are
//! shared via the private [`crate::urn`] module.

use crate::error::Error;
use crate::urn::{build_canonical_urn, parse_kinded_urn};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// The single kind segment accepted by extension URNs.
const EXTENSION_KINDS: &[&str] = &["extension"];

/// Human-friendly URN label used in error messages.
const URN_LABEL: &str = "extension URN";

/// Extension type URN with zero-copy access to namespace and id segments.
///
/// Canonical form: `urn:<namespace>:extension:<id>` (e.g.,
/// `urn:microsoft:extension:azure_identity_auth`). The kind segment is
/// fixed to the literal `extension`, mirroring the
/// `urn:<namespace>:<kind>:<id>` convention used by receivers,
/// processors, and exporters.
///
/// Short form `extension:<id>` is also accepted and expanded to
/// `urn:otel:extension:<id>`.
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
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let parsed = parse_kinded_urn(raw, EXTENSION_KINDS, URN_LABEL)?;
        let (raw, namespace_range, id_range) =
            build_canonical_urn(&parsed.namespace, parsed.kind, &parsed.id);
        Ok(ExtensionUrn {
            raw,
            namespace_range,
            id_range,
        })
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

impl std::borrow::Borrow<str> for ExtensionUrn {
    fn borrow(&self) -> &str {
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
        Self::parse(value.as_str())
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
    fn test_parse_canonical_4_segment_urn() {
        let urn = ExtensionUrn::parse("urn:otap:extension:sample_kv_store").unwrap();
        assert_eq!(urn.namespace(), "otap");
        assert_eq!(urn.id(), "sample_kv_store");
        assert_eq!(urn.as_str(), "urn:otap:extension:sample_kv_store");
    }

    #[test]
    fn test_parse_short_form() {
        let urn = ExtensionUrn::parse("extension:my_auth").unwrap();
        assert_eq!(urn.namespace(), "otel");
        assert_eq!(urn.id(), "my_auth");
        assert_eq!(urn.as_str(), "urn:otel:extension:my_auth");
    }

    #[test]
    fn test_parse_short_form_rejects_other_kinds() {
        // The short form's kind segment must be `extension`.
        assert!(ExtensionUrn::parse("receiver:my_thing").is_err());
        assert!(ExtensionUrn::parse("processor:my_thing").is_err());
        assert!(ExtensionUrn::parse("exporter:my_thing").is_err());
    }

    #[test]
    fn test_parse_case_insensitive_scheme_and_kind() {
        let urn = ExtensionUrn::parse("URN:Microsoft:Extension:azure_auth").unwrap();
        assert_eq!(urn.namespace(), "microsoft");
        assert_eq!(urn.id(), "azure_auth");
    }

    #[test]
    fn test_parse_rejects_3_segment_form() {
        // The pre-existing 3-segment form is no longer accepted; users
        // must use the 4-segment canonical form.
        assert!(ExtensionUrn::parse("urn:otap:sample_kv_store").is_err());
    }

    #[test]
    fn test_parse_rejects_4_segment_with_other_kind() {
        // Any kind other than `extension` is rejected.
        assert!(ExtensionUrn::parse("urn:otap:receiver:foo").is_err());
        assert!(ExtensionUrn::parse("urn:otap:processor:foo").is_err());
        assert!(ExtensionUrn::parse("urn:otap:exporter:foo").is_err());
    }

    #[test]
    fn test_parse_rejects_bare_id() {
        assert!(ExtensionUrn::parse("my_auth").is_err());
    }

    #[test]
    fn test_parse_rejects_empty_id() {
        assert!(ExtensionUrn::parse("").is_err());
    }

    #[test]
    fn test_parse_rejects_invalid_chars() {
        assert!(ExtensionUrn::parse("urn:otap:extension:bad name").is_err());
    }

    #[test]
    fn test_from_static_str() {
        let urn: ExtensionUrn = "urn:otap:extension:test_ext".into();
        assert_eq!(urn.id(), "test_ext");
    }

    #[test]
    fn test_serde_roundtrip() {
        let urn = ExtensionUrn::parse("urn:otap:extension:my_ext").unwrap();
        let json = serde_json::to_string(&urn).unwrap();
        assert_eq!(json, "\"urn:otap:extension:my_ext\"");
        let parsed: ExtensionUrn = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, urn);
    }
}
