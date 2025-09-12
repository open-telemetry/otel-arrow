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

/// Minimum required NSS segments for `urn:otap:processor:<name>`
const MIN_OTAP_PROCESSOR_NSS_SEGMENTS: usize = 2;
/// Minimum required NSS segments for `urn:otel:<family>(:<subfamily>...):<kind>`
const MIN_OTEL_NSS_SEGMENTS: usize = 2;

/// Parse a raw URN string.
pub fn parse_urn(raw: &str) -> Result<Urn, Error> {
    raw.parse::<Urn>().map_err(|e| Error::InvalidUserConfig {
        error: format!("invalid URN `{raw}`: {e}"),
    })
}

/// Validate a plugin URN against the expected node kind.
///
/// Accepted patterns:
/// - otel family with kind suffix: urn:otel:<family>(:<subfamily>...):<receiver|processor|exporter>
///   Examples: urn:otel:otlp:receiver, urn:otel:debug:processor, urn:otel:otap:exporter,
///   urn:otel:otap:parquet:exporter, urn:otel:syslog_cef:receiver
/// - otap processors: urn:otap:processor:<name>
///   Examples: urn:otap:processor:batch, urn:otap:processor:signal_type_router
pub fn validate_plugin_urn(raw: &str, expected_kind: NodeKind) -> Result<(), Error> {
    let urn = parse_urn(raw)?;
    let nid = urn.nid().to_ascii_lowercase();
    let nss = urn.nss();
    let segs: Vec<&str> = nss.split(':').collect();

    // NSS must have non-empty segments separated by ':'
    if segs.is_empty() || segs.iter().any(|s| s.is_empty()) {
        return Err(Error::InvalidUserConfig {
            error: format!(
                "invalid plugin URN `{raw}`: NSS must have non-empty segments separated by ':'"
            ),
        });
    }

    // All segments must be lowercase a-z, 0-9, underscore, hyphen, or dot
    fn is_valid_segment(seg: &str) -> bool {
        seg.chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.'))
    }
    if segs.iter().any(|s| !is_valid_segment(s)) {
        return Err(Error::InvalidUserConfig {
            error: format!("invalid plugin URN `{raw}`: NSS segments must match [a-z0-9._-]"),
        });
    }

    match nid.as_str() {
        // otap processors: urn:otap:processor:<name>
        "otap" => {
            // Expect at least: processor:<name>, allow deeper names
            if !(segs.first().copied() == Some("processor")
                && segs.len() >= MIN_OTAP_PROCESSOR_NSS_SEGMENTS)
            {
                return Err(Error::InvalidUserConfig {
                    error: format!(
                        "invalid plugin URN `{raw}`: expected `urn:otap:processor:<name>`"
                    ),
                });
            }
            if expected_kind != NodeKind::Processor {
                return Err(Error::InvalidUserConfig {
                    error: format!(
                        "invalid plugin URN `{raw}`: URN is a processor but node kind is `{expected_kind:?}`"
                    ),
                });
            }
            Ok(())
        }
        // otel family: require trailing kind suffix
        "otel" => {
            // Require at least family + kind
            if segs.len() < MIN_OTEL_NSS_SEGMENTS {
                return Err(Error::InvalidUserConfig {
                    error: format!(
                        "invalid plugin URN `{raw}`: expected `urn:otel:<family>(:<subfamily>...):<receiver|processor|exporter>`"
                    ),
                });
            }
            let last = *segs
                .last()
                .expect("NSS must have at least one segment if not empty");
            let expected_suffix = match expected_kind {
                NodeKind::Receiver => "receiver",
                NodeKind::Processor | NodeKind::ProcessorChain => "processor",
                NodeKind::Exporter => "exporter",
            };
            if last != expected_suffix {
                return Err(Error::InvalidUserConfig {
                    error: format!(
                        "invalid plugin URN `{raw}`: expected suffix `{expected_suffix}`, found `{last}`"
                    ),
                });
            }
            Ok(())
        }
        _ => Err(Error::InvalidUserConfig {
            error: format!(
                "invalid plugin URN `{raw}`: unknown namespace `{nid}` (expected `otel` or `otap`)"
            ),
        }),
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
        assert!(validate_plugin_urn("urn:otel:otap:parquet:exporter", NodeKind::Exporter).is_ok());
        assert!(validate_plugin_urn("urn:otel:syslog_cef:receiver", NodeKind::Receiver).is_ok());

        // otap processor with a (possibly hierarchical) name
        assert!(validate_plugin_urn("urn:otap:processor:batch", NodeKind::Processor).is_ok());
        assert!(validate_plugin_urn("urn:OTAP:processor:batch", NodeKind::Processor).is_ok());
        assert!(validate_plugin_urn("urn:otap:processor:batch:v2", NodeKind::Processor).is_ok());

        // Hyphen and dot allowed in NSS segments
        assert!(validate_plugin_urn("urn:otel:otlp-http:receiver", NodeKind::Receiver).is_ok());
        assert!(validate_plugin_urn("urn:otel:debug.log:processor", NodeKind::Processor).is_ok());
    }

    #[test]
    fn rejects_mismatches_and_invalids() {
        // Unknown namespace
        assert!(validate_plugin_urn("urn:example:x", NodeKind::Receiver).is_err());

        // Empty NSS segments
        assert!(validate_plugin_urn("urn:otel::otlp:receiver", NodeKind::Receiver).is_err());
        assert!(validate_plugin_urn("urn:otel:otlp::receiver", NodeKind::Receiver).is_err());
        assert!(validate_plugin_urn("urn:otel::receiver", NodeKind::Receiver).is_err());

        // Missing family for otel
        assert!(validate_plugin_urn("urn:otel:receiver", NodeKind::Receiver).is_err());

        // otap: missing name
        assert!(validate_plugin_urn("urn:otap:processor", NodeKind::Processor).is_err());

        // Uppercase NSS rejected
        assert!(validate_plugin_urn("urn:otel:OTLP:receiver", NodeKind::Receiver).is_err());

        // Percent-encoding not supported by policy
        assert!(validate_plugin_urn("urn:otel:my%2Ffamily:receiver", NodeKind::Receiver).is_err());

        // Wrong kind mapping
        assert!(validate_plugin_urn("urn:otel:otlp:exporter", NodeKind::Receiver).is_err());

        // Unknown URN entirely
        assert!(validate_plugin_urn("not_a_urn", NodeKind::Receiver).is_err());
    }
}
