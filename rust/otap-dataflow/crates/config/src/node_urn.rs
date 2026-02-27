// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Plugin URN parsing and validation helpers.

use crate::error::Error;
use crate::node::NodeKind;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::Range;

const URN_DOCS_PATH: &str = "rust/otap-dataflow/docs/urns.md";
const EXPECTED_SEGMENT_COUNT: usize = 2;

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
    #[must_use]
    pub(crate) fn from_canonical_parts(
        raw: String,
        namespace_range: Range<usize>,
        id_range: Range<usize>,
        kind: NodeKind,
    ) -> Self {
        Self {
            raw,
            namespace_range,
            id_range,
            kind,
        }
    }

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
        let raw = raw.trim();
        let parts: Vec<&str> = raw.split(':').collect();

        match parts.as_slice() {
            [_kind, _id] => {
                validate_segments(raw, "otel", parts.as_slice())?;
                let (kind, id) = split_segments(raw, parts.as_slice())?;
                let inferred_kind = parse_kind(raw, kind)?;
                Ok(build_node_urn("otel", id, kind, inferred_kind))
            }
            [scheme, namespace, _kind, _id] => {
                if !scheme.eq_ignore_ascii_case("urn") {
                    return Err(invalid_plugin_urn(
                        raw,
                        "expected `urn:<namespace>:<kind>:<id>`".to_string(),
                    ));
                }
                let namespace = namespace.to_ascii_lowercase();
                let kind_id = &parts[2..];
                validate_segments(raw, &namespace, kind_id)?;
                let (kind, id) = split_segments(raw, kind_id)?;
                let inferred_kind = parse_kind(raw, kind)?;
                Ok(build_node_urn(namespace.as_str(), id, kind, inferred_kind))
            }
            _ => Err(invalid_plugin_urn(
                raw,
                "expected `urn:<namespace>:<kind>:<id>` or `<kind>:<id>` for otel".to_string(),
            )),
        }
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
    validate_expected_kind(raw.trim(), expected_kind, normalized.kind())?;
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

    validate_segments(raw, "otel", &[raw])?;
    let expected_suffix = kind_suffix(expected_kind);
    let kind = parse_kind(raw, expected_suffix)?;
    Ok(build_node_urn("otel", raw, expected_suffix, kind))
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

fn validate_expected_kind(raw: &str, expected_kind: NodeKind, kind: NodeKind) -> Result<(), Error> {
    let expected_suffix = kind_suffix(expected_kind);
    let actual_suffix = kind_suffix(kind);
    if actual_suffix != expected_suffix {
        return Err(invalid_plugin_urn(
            raw,
            format!("expected kind `{expected_suffix}`, found `{actual_suffix}`"),
        ));
    }
    Ok(())
}

fn parse_kind(raw: &str, kind: &str) -> Result<NodeKind, Error> {
    match kind {
        "receiver" => Ok(NodeKind::Receiver),
        "processor" => Ok(NodeKind::Processor),
        "exporter" => Ok(NodeKind::Exporter),
        _ => Err(invalid_plugin_urn(
            raw,
            format!("expected kind `receiver`, `processor`, or `exporter`, found `{kind}`"),
        )),
    }
}

fn split_segments<'a>(raw: &str, segs: &'a [&'a str]) -> Result<(&'a str, &'a str), Error> {
    if segs.len() != EXPECTED_SEGMENT_COUNT {
        return Err(invalid_plugin_urn(
            raw,
            format!("expected exactly {EXPECTED_SEGMENT_COUNT} segments in `<kind>:<id>`"),
        ));
    }

    let kind = segs[0];
    let id = segs[1];
    if kind.is_empty() || id.is_empty() {
        return Err(invalid_plugin_urn(
            raw,
            "segments must be non-empty".to_string(),
        ));
    }

    Ok((kind, id))
}

fn validate_segments(raw: &str, namespace: &str, segs: &[&str]) -> Result<(), Error> {
    if namespace.is_empty() {
        return Err(invalid_plugin_urn(
            raw,
            "namespace must be non-empty".to_string(),
        ));
    }

    if segs.is_empty() || segs.iter().any(|s| s.is_empty()) {
        return Err(invalid_plugin_urn(
            raw,
            "segments must be non-empty".to_string(),
        ));
    }

    if !is_valid_segment(namespace) {
        return Err(invalid_plugin_urn(
            raw,
            format!("namespace `{namespace}` must match [a-z0-9._-]"),
        ));
    }

    if segs.iter().any(|s| !is_valid_segment(s)) {
        return Err(invalid_plugin_urn(
            raw,
            "segments must match [a-z0-9._-]".to_string(),
        ));
    }

    Ok(())
}

fn is_valid_segment(seg: &str) -> bool {
    seg.chars()
        .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.'))
}

fn build_node_urn(namespace: &str, id: &str, kind_str: &str, kind: NodeKind) -> NodeUrn {
    let raw = format!("urn:{namespace}:{kind_str}:{id}");
    let namespace_start = "urn:".len();
    let namespace_end = namespace_start + namespace.len();
    let id_start = namespace_end + 1 + kind_str.len() + 1;
    let id_end = id_start + id.len();
    NodeUrn::from_canonical_parts(raw, namespace_start..namespace_end, id_start..id_end, kind)
}

fn invalid_plugin_urn(raw: &str, details: String) -> Error {
    Error::InvalidUserConfig {
        error: format!(
            "invalid plugin urn `{raw}`: {details}; expected `urn:<namespace>:<kind>:<id>` or `<kind>:<id>` for otel (see {URN_DOCS_PATH})"
        ),
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
        assert!(validate_plugin_urn("urn:otel:exporter:otlp", NodeKind::Receiver).is_err());

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
