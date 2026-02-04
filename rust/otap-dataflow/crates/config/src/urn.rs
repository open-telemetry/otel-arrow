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
const URN_SCHEME: &str = "urn:";
const EXPECTED_SEGMENT_COUNT: usize = 2;

/// Parse a raw URN string.
pub fn parse_urn(raw: &str) -> Result<Urn, Error> {
    raw.parse::<Urn>().map_err(|e| Error::InvalidUserConfig {
        error: format!("invalid urn `{raw}`: {e}"),
    })
}

/// Normalize and validate a plugin URN against the expected node kind.
///
/// Accepted patterns:
/// - full form: `urn:<namespace>:<id>:<kind>`
/// - shortcut form (OTel only): `<id>:<kind>` (expanded to `urn:otel:<id>:<kind>`)
pub fn normalize_plugin_urn(raw: &str, expected_kind: NodeKind) -> Result<String, Error> {
    let expected_suffix = kind_suffix(expected_kind);
    let raw = raw.trim();

    if raw
        .get(..URN_SCHEME.len())
        .map(|prefix| prefix.eq_ignore_ascii_case(URN_SCHEME))
        .unwrap_or(false)
    {
        let rest = raw
            .split_once(':')
            .map(|(_, tail)| tail)
            .unwrap_or_default()
            .trim();
        let segs: Vec<&str> = rest.split(':').collect();
        if segs.len() != EXPECTED_SEGMENT_COUNT + 1 {
            return Err(invalid_plugin_urn(
                raw,
                "expected exactly 3 segments in `urn:<namespace>:<id>:<kind>`".to_string(),
            ));
        }

        let namespace = segs[0].to_ascii_lowercase();
        let id_kind = &segs[1..];

        validate_segments(raw, &namespace, id_kind)?;
        let (id, kind) = split_segments(raw, id_kind)?;
        validate_kind(raw, kind, expected_suffix)?;
        return Ok(format!("urn:{namespace}:{id}:{kind}"));
    }

    let segs: Vec<&str> = raw.split(':').collect();
    validate_segments(raw, "otel", &segs)?;
    let (id, kind) = split_segments(raw, &segs)?;
    validate_kind(raw, kind, expected_suffix)?;
    Ok(format!("urn:otel:{id}:{kind}"))
}

/// Validate a plugin URN against the expected node kind.
pub fn validate_plugin_urn(raw: &str, expected_kind: NodeKind) -> Result<(), Error> {
    normalize_plugin_urn(raw, expected_kind).map(|_| ())
}

fn kind_suffix(expected_kind: NodeKind) -> &'static str {
    match expected_kind {
        NodeKind::Receiver => "receiver",
        NodeKind::Processor | NodeKind::ProcessorChain => "processor",
        NodeKind::Exporter => "exporter",
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

fn validate_kind(raw: &str, kind: &str, expected_suffix: &str) -> Result<(), Error> {
    if kind != expected_suffix {
        return Err(invalid_plugin_urn(
            raw,
            format!("expected kind `{expected_suffix}`, found `{kind}`"),
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
            normalize_plugin_urn("otlp:receiver", NodeKind::Receiver).unwrap(),
            "urn:otel:otlp:receiver"
        );

        // Non-otel namespaces allowed in full form
        assert!(
            validate_plugin_urn("urn:vendor_product:custom:exporter", NodeKind::Exporter).is_ok()
        );
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
