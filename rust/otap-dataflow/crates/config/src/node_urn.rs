// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node URN parsing and validation.
//!
//! Node URNs follow the canonical form `urn:<namespace>:<kind>:<id>` where
//! `<kind>` is one of `receiver`, `processor`, or `exporter`. The shortcut
//! `<kind>:<id>` (no scheme/namespace) expands to `urn:otel:<kind>:<id>`.
//!
//! Extensions deliberately do NOT use this type -- they have a separate
//! [`crate::extension_urn::ExtensionUrn`] type so the rest of the codebase
//! cannot accidentally treat extensions as nodes. The underlying parsing
//! primitives are shared via the private [`crate::urn`] module.

use crate::error::Error;
use crate::node::NodeKind;
use crate::urn::{URN_DOCS_PATH, build_canonical_urn, is_valid_segment, parse_kinded_urn};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::Range;

/// Kind segments accepted by node URNs. Extensions use a disjoint set
/// in [`crate::extension_urn`].
const NODE_KINDS: &[&str] = &["receiver", "processor", "exporter"];

/// Human-friendly URN label used in error messages.
const URN_LABEL: &str = "plugin urn";

/// Canonical node URN with zero-copy access to namespace and id segments.
///
/// The canonical representation is always `urn:<namespace>:<kind>:<id>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct NodeUrn {
    raw: String,
    namespace_range: Range<usize>,
    id_range: Range<usize>,
    kind: NodeKind,
}

impl NodeUrn {
    /// Returns the canonical URN string (`urn:<namespace>:<kind>:<id>`).
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

    /// Returns the node kind segment.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }

    /// Returns the owned canonical URN string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.raw
    }

    /// Parses and canonicalizes a node URN.
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let parsed = parse_kinded_urn(raw, NODE_KINDS, URN_LABEL)?;
        let kind = node_kind_from_segment(parsed.kind);
        let (raw, namespace_range, id_range) =
            build_canonical_urn(&parsed.namespace, parsed.kind, &parsed.id);
        Ok(NodeUrn {
            raw,
            namespace_range,
            id_range,
            kind,
        })
    }
}

impl Default for NodeUrn {
    fn default() -> Self {
        Self::from("receiver:unknown")
    }
}

impl std::fmt::Display for NodeUrn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for NodeUrn {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for NodeUrn {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl From<NodeUrn> for String {
    fn from(value: NodeUrn) -> Self {
        value.raw
    }
}

impl From<NodeUrn> for Cow<'static, str> {
    fn from(value: NodeUrn) -> Self {
        Cow::Owned(value.raw)
    }
}

impl From<&NodeUrn> for Cow<'static, str> {
    fn from(value: &NodeUrn) -> Self {
        Cow::Owned(value.raw.clone())
    }
}

impl TryFrom<String> for NodeUrn {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl From<&'static str> for NodeUrn {
    fn from(value: &'static str) -> Self {
        Self::parse(value).expect("invalid static node urn literal")
    }
}

/// Validate a plugin URN against the expected node kind and return the canonical form.
///
/// Accepted patterns:
/// - full form: `urn:<namespace>:<kind>:<id>`
/// - shortcut form (OTel only): `<kind>:<id>` (expanded to `urn:otel:<kind>:<id>`)
pub fn validate_plugin_urn(raw: &str, expected_kind: NodeKind) -> Result<NodeUrn, Error> {
    let normalized = NodeUrn::parse(raw)?;
    if !kinds_match(expected_kind, normalized.kind()) {
        let expected_suffix = kind_suffix(expected_kind);
        let actual_suffix = kind_suffix(normalized.kind());
        return Err(Error::InvalidUserConfig {
            error: format!(
                "invalid {URN_LABEL} `{}`: expected kind `{expected_suffix}`, found \
                 `{actual_suffix}`; expected `urn:<namespace>:<kind>:<id>` or \
                 `<kind>:<id>` for otel (see {URN_DOCS_PATH})",
                raw.trim()
            ),
        });
    }
    Ok(normalized)
}

/// Canonicalize a node type URN with an expected kind.
///
/// For compatibility with constructor call sites, this additionally accepts a bare id
/// (`<id>`) and expands it to `urn:otel:<expected_kind>:<id>`.
pub fn normalize_plugin_urn_for_kind(raw: &str, expected_kind: NodeKind) -> Result<NodeUrn, Error> {
    if let Ok(normalized) = validate_plugin_urn(raw, expected_kind) {
        return Ok(normalized);
    }

    let raw = raw.trim();
    if raw.contains(':') {
        return validate_plugin_urn(raw, expected_kind);
    }

    if !is_valid_segment(raw) {
        return Err(Error::InvalidUserConfig {
            error: format!(
                "invalid {URN_LABEL} `{raw}`: id `{raw}` must match [a-z0-9._-]; expected \
                 `urn:<namespace>:<kind>:<id>` or `<kind>:<id>` for otel (see {URN_DOCS_PATH})"
            ),
        });
    }

    let suffix = kind_suffix(expected_kind);
    let (canonical, namespace_range, id_range) = build_canonical_urn("otel", suffix, raw);
    Ok(NodeUrn {
        raw: canonical,
        namespace_range,
        id_range,
        kind: node_kind_from_segment(suffix),
    })
}

/// Canonicalize a node type URN.
pub fn canonicalize_plugin_urn(raw: &str) -> Result<NodeUrn, Error> {
    NodeUrn::parse(raw)
}

/// Infer node kind from a node type URN.
pub fn infer_node_kind(raw: &str) -> Result<NodeKind, Error> {
    NodeUrn::parse(raw).map(|urn| urn.kind())
}

const fn kind_suffix(expected_kind: NodeKind) -> &'static str {
    match expected_kind {
        NodeKind::Receiver => "receiver",
        NodeKind::Processor | NodeKind::ProcessorChain => "processor",
        NodeKind::Exporter => "exporter",
    }
}

/// Returns true if `expected` and `actual` correspond to the same URN
/// kind segment. Treats `Processor` and `ProcessorChain` as equivalent
/// because they both serialize to the `processor` segment.
fn kinds_match(expected: NodeKind, actual: NodeKind) -> bool {
    kind_suffix(expected) == kind_suffix(actual)
}

/// Map a known kind segment back to a `NodeKind`. The segment must be
/// one of [`NODE_KINDS`] (the parser guarantees this).
fn node_kind_from_segment(segment: &str) -> NodeKind {
    match segment {
        "receiver" => NodeKind::Receiver,
        "processor" => NodeKind::Processor,
        "exporter" => NodeKind::Exporter,
        // Unreachable because `parse_kinded_urn` only returns a kind from
        // `NODE_KINDS`; documented as a defensive panic for clarity.
        other => unreachable!("node URN parser returned unexpected kind `{other}`"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_exposes_parts() {
        let urn = NodeUrn::parse("receiver:otlp").unwrap();
        assert_eq!(urn.as_str(), "urn:otel:receiver:otlp");
        assert_eq!(urn.namespace(), "otel");
        assert_eq!(urn.id(), "otlp");
        assert!(matches!(urn.kind(), NodeKind::Receiver));
    }

    #[test]
    fn accepts_known_patterns() {
        // Upper/lowercase scheme and NID accepted
        assert!(validate_plugin_urn("urn:otel:receiver:otlp", NodeKind::Receiver).is_ok());
        assert!(validate_plugin_urn("URN:otel:receiver:otlp", NodeKind::Receiver).is_ok());
        assert!(validate_plugin_urn("urn:OTEL:receiver:otlp", NodeKind::Receiver).is_ok());

        // Typical valid forms
        assert!(validate_plugin_urn("urn:otel:processor:debug", NodeKind::Processor).is_ok());
        assert!(validate_plugin_urn("urn:otel:exporter:otap", NodeKind::Exporter).is_ok());
        assert!(validate_plugin_urn("urn:otel:receiver:syslog_cef", NodeKind::Receiver).is_ok());

        // Hyphen and dot allowed in NSS segments
        assert!(validate_plugin_urn("urn:otel:receiver:otlp-http", NodeKind::Receiver).is_ok());
        assert!(validate_plugin_urn("urn:otel:processor:debug.log", NodeKind::Processor).is_ok());

        // Shortcut form for otel
        assert_eq!(
            validate_plugin_urn("receiver:otlp", NodeKind::Receiver)
                .unwrap()
                .as_ref(),
            "urn:otel:receiver:otlp"
        );

        // Non-otel namespaces allowed in full form
        assert!(
            validate_plugin_urn("urn:vendor_product:exporter:custom", NodeKind::Exporter).is_ok()
        );

        assert!(matches!(
            infer_node_kind("urn:otel:receiver:otlp").unwrap(),
            NodeKind::Receiver
        ));
        assert!(matches!(
            infer_node_kind("processor:debug").unwrap(),
            NodeKind::Processor
        ));
    }

    #[test]
    fn rejects_extension_kind() {
        // Extensions are not nodes; their URNs must not parse as NodeUrn.
        assert!(NodeUrn::parse("urn:otap:extension:foo").is_err());
        assert!(NodeUrn::parse("extension:foo").is_err());
    }

    #[test]
    fn rejects_mismatches_and_invalids() {
        // Empty NSS segments
        assert!(validate_plugin_urn("urn:otel::receiver:otlp", NodeKind::Receiver).is_err());
        assert!(validate_plugin_urn("urn:otel:receiver::otlp", NodeKind::Receiver).is_err());
        assert!(validate_plugin_urn("urn:otel::otlp", NodeKind::Receiver).is_err());

        // Missing id for otel
        assert!(validate_plugin_urn("urn:otel:receiver", NodeKind::Receiver).is_err());

        // Uppercase NSS rejected
        assert!(validate_plugin_urn("urn:otel:receiver:OTLP", NodeKind::Receiver).is_err());

        // Percent-encoding not supported by policy
        assert!(validate_plugin_urn("urn:otel:receiver:my%2Ffamily", NodeKind::Receiver).is_err());

        // Wrong kind mapping
        assert!(validate_plugin_urn("urn:otel:exporter:otlp_grpc", NodeKind::Receiver).is_err());

        // Unknown kind
        assert!(infer_node_kind("urn:otel:sink:otlp").is_err());

        // Extra segments rejected
        assert!(validate_plugin_urn("urn:otel:exporter:otap:perf", NodeKind::Exporter).is_err());

        // Old format rejected (name before kind is not valid)
        assert!(validate_plugin_urn("urn:otel:otlp:receiver", NodeKind::Receiver).is_err());

        // Non-OTel shortcut rejected
        assert!(validate_plugin_urn("microsoft:monitor:exporter", NodeKind::Exporter).is_err());

        // Unknown URN entirely
        assert!(validate_plugin_urn("not_a_urn", NodeKind::Receiver).is_err());
    }
}
