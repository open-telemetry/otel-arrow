// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Plugin URN parsing and validation helpers.
//!
//! Uses the `urn` crate for RFC 8141 parsing (see RFC 8141), then applies project-specific
//! rules for plugin URNs.
//!
//! References:
//! - RFC 8141: https://datatracker.ietf.org/doc/html/rfc8141

use crate::error::Error;
use crate::node::NodeKind;
use urn::Urn;

const URN_DOCS_PATH: &str = "rust/otap-dataflow/docs/urns.md";
const EXPECTED_SEGMENT_COUNT: usize = 2;

/// Parse a raw URN string.
pub fn parse_urn(raw: &str) -> Result<Urn, Error> {
    raw.parse::<Urn>().map_err(|e| Error::InvalidUserConfig {
        error: format!("invalid urn `{raw}`: {e}"),
    })
}

/// Validate a plugin URN against the expected node kind and return the canonical form.
///
/// Accepted patterns:
/// - full form: `urn:<namespace>:<id>:<kind>`
/// - shortcut form (OTel only): `<id>:<kind>` (expanded to `urn:otel:<id>:<kind>`)
pub fn validate_plugin_urn(raw: &str, expected_kind: NodeKind) -> Result<String, Error> {
    let (normalized, _inferred_kind, inferred_kind_suffix) = normalize_plugin_urn(raw)?;
    validate_expected_kind(raw.trim(), expected_kind, inferred_kind_suffix)?;
    Ok(normalized)
}

/// Validate and canonicalize a node type URN while inferring its node kind.
pub fn normalize_plugin_urn(raw: &str) -> Result<(String, NodeKind, &'static str), Error> {
    let raw = raw.trim();
    let parts: Vec<&str> = raw.split(':').collect();

    match parts.as_slice() {
        [_id, _kind] => {
            validate_segments(raw, "otel", parts.as_slice())?;
            let (id, kind) = split_segments(raw, parts.as_slice())?;
            let inferred_kind = parse_kind(raw, kind)?;
            Ok((
                format!("urn:otel:{id}:{kind}"),
                inferred_kind,
                kind_suffix(inferred_kind),
            ))
        }
        [scheme, namespace, _id, _kind] => {
            if !scheme.eq_ignore_ascii_case("urn") {
                return Err(invalid_plugin_urn(
                    raw,
                    "expected `urn:<namespace>:<id>:<kind>`".to_string(),
                ));
            }
            let namespace = namespace.to_ascii_lowercase();
            let id_kind = &parts[2..];
            validate_segments(raw, &namespace, id_kind)?;
            let (id, kind) = split_segments(raw, id_kind)?;
            let inferred_kind = parse_kind(raw, kind)?;
            Ok((
                format!("urn:{namespace}:{id}:{kind}"),
                inferred_kind,
                kind_suffix(inferred_kind),
            ))
        }
        _ => Err(invalid_plugin_urn(
            raw,
            "expected `urn:<namespace>:<id>:<kind>` or `<id>:<kind>` for otel".to_string(),
        )),
    }
}

/// Canonicalize a node type URN.
pub fn canonicalize_plugin_urn(raw: &str) -> Result<String, Error> {
    let (normalized, _kind, _kind_suffix) = normalize_plugin_urn(raw)?;
    Ok(normalized)
}

/// Infer node kind from a node type URN.
pub fn infer_node_kind(raw: &str) -> Result<NodeKind, Error> {
    let (_normalized, kind, _kind_suffix) = normalize_plugin_urn(raw)?;
    Ok(kind)
}

const fn kind_suffix(expected_kind: NodeKind) -> &'static str {
    match expected_kind {
        NodeKind::Receiver => "receiver",
        NodeKind::Processor | NodeKind::ProcessorChain => "processor",
        NodeKind::Exporter => "exporter",
    }
}

fn validate_expected_kind(raw: &str, expected_kind: NodeKind, kind: &str) -> Result<(), Error> {
    let expected_suffix = kind_suffix(expected_kind);
    if kind != expected_suffix {
        return Err(invalid_plugin_urn(
            raw,
            format!("expected kind `{expected_suffix}`, found `{kind}`"),
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
            format!("expected exactly {EXPECTED_SEGMENT_COUNT} segments in `<id>:<kind>`"),
        ));
    }

    let id = segs[0];
    let kind = segs[1];
    if id.is_empty() || kind.is_empty() {
        return Err(invalid_plugin_urn(
            raw,
            "segments must be non-empty".to_string(),
        ));
    }

    Ok((id, kind))
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

fn invalid_plugin_urn(raw: &str, details: String) -> Error {
    Error::InvalidUserConfig {
        error: format!(
            "invalid plugin urn `{raw}`: {details}; expected `urn:<namespace>:<id>:<kind>` or `<id>:<kind>` for otel (see {URN_DOCS_PATH})"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_patterns() {
        // Upper/lowercase scheme and NID accepted
        assert!(validate_plugin_urn("urn:otel:otlp:receiver", NodeKind::Receiver).is_ok());
        assert!(validate_plugin_urn("URN:otel:otlp:receiver", NodeKind::Receiver).is_ok());
        assert!(validate_plugin_urn("urn:OTEL:otlp:receiver", NodeKind::Receiver).is_ok());

        // Typical valid forms
        assert!(validate_plugin_urn("urn:otel:debug:processor", NodeKind::Processor).is_ok());
        assert!(validate_plugin_urn("urn:otel:otap:exporter", NodeKind::Exporter).is_ok());
        assert!(validate_plugin_urn("urn:otel:syslog_cef:receiver", NodeKind::Receiver).is_ok());

        // Hyphen and dot allowed in NSS segments
        assert!(validate_plugin_urn("urn:otel:otlp-http:receiver", NodeKind::Receiver).is_ok());
        assert!(validate_plugin_urn("urn:otel:debug.log:processor", NodeKind::Processor).is_ok());

        // Shortcut form for otel
        assert_eq!(
            validate_plugin_urn("otlp:receiver", NodeKind::Receiver).unwrap(),
            "urn:otel:otlp:receiver"
        );

        // Non-otel namespaces allowed in full form
        assert!(
            validate_plugin_urn("urn:vendor_product:custom:exporter", NodeKind::Exporter).is_ok()
        );

        assert!(matches!(
            infer_node_kind("urn:otel:otlp:receiver").unwrap(),
            NodeKind::Receiver
        ));
        assert!(matches!(
            infer_node_kind("debug:processor").unwrap(),
            NodeKind::Processor
        ));
    }

    #[test]
    fn rejects_mismatches_and_invalids() {
        // Empty NSS segments
        assert!(validate_plugin_urn("urn:otel::otlp:receiver", NodeKind::Receiver).is_err());
        assert!(validate_plugin_urn("urn:otel:otlp::receiver", NodeKind::Receiver).is_err());
        assert!(validate_plugin_urn("urn:otel::receiver", NodeKind::Receiver).is_err());

        // Missing id for otel
        assert!(validate_plugin_urn("urn:otel:receiver", NodeKind::Receiver).is_err());

        // Uppercase NSS rejected
        assert!(validate_plugin_urn("urn:otel:OTLP:receiver", NodeKind::Receiver).is_err());

        // Percent-encoding not supported by policy
        assert!(validate_plugin_urn("urn:otel:my%2Ffamily:receiver", NodeKind::Receiver).is_err());

        // Wrong kind mapping
        assert!(validate_plugin_urn("urn:otel:otlp:exporter", NodeKind::Receiver).is_err());

        // Unknown kind
        assert!(infer_node_kind("urn:otel:otlp:sink").is_err());

        // Legacy forms rejected (extra segments)
        assert!(
            validate_plugin_urn(
                "urn:otap:processor:attributes_processor",
                NodeKind::Processor
            )
            .is_err()
        );
        assert!(validate_plugin_urn("urn:otel:otap:perf:exporter", NodeKind::Exporter).is_err());

        // Non-OTel shortcut rejected
        assert!(validate_plugin_urn("microsoft:monitor:exporter", NodeKind::Exporter).is_err());

        // Unknown URN entirely
        assert!(validate_plugin_urn("not_a_urn", NodeKind::Receiver).is_err());
    }
}
