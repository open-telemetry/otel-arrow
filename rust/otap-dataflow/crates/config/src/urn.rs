// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared URN parsing primitives.
//!
//! Both node URNs (`urn:<namespace>:<kind>:<id>` where `<kind>` is one of
//! `receiver`/`processor`/`exporter`) and extension URNs
//! (`urn:<namespace>:extension:<id>`) follow the same shape, so the
//! parsing/validation/canonicalization logic lives here. The kind-specific
//! wrappers in [`crate::node_urn`] and [`crate::extension_urn`] supply
//! the set of accepted kind segments and reject everything else.
//!
//! Extension and node URNs are intentionally exposed as distinct types
//! ([`crate::node_urn::NodeUrn`] vs [`crate::extension_urn::ExtensionUrn`])
//! so the rest of the codebase cannot accidentally confuse the two; the
//! shared logic is non-public to this module and only reachable through
//! those wrappers.

use crate::error::Error;
use std::ops::Range;

/// Reference path used in error messages pointing to URN documentation.
pub(crate) const URN_DOCS_PATH: &str = "rust/otap-dataflow/docs/urns.md";

/// Successful parse result: the segments needed to build a canonical URN
/// of the form `urn:<namespace>:<kind>:<id>`. The owned strings live in
/// the caller, which assembles the final canonical representation.
pub(crate) struct ParsedKindedUrn<'a> {
    pub namespace: String,
    pub kind: &'a str,
    pub id: String,
}

/// Parse a kinded URN, accepting both the canonical 4-segment form
/// (`urn:<namespace>:<kind>:<id>`) and the short `<kind>:<id>` form
/// (which expands to `urn:otel:<kind>:<id>`).
///
/// `accepted_kinds` is the closed set of kind segments allowed for this
/// URN flavour. The match is case-insensitive; the canonical form
/// returned in [`ParsedKindedUrn::kind`] borrows from `accepted_kinds`
/// so the caller controls casing.
///
/// `urn_label` is the human-friendly label used in error messages
/// (e.g., `"plugin urn"` or `"extension URN"`).
pub(crate) fn parse_kinded_urn<'a>(
    raw: &str,
    accepted_kinds: &'a [&'a str],
    urn_label: &str,
) -> Result<ParsedKindedUrn<'a>, Error> {
    let raw = raw.trim();
    let parts: Vec<&str> = raw.split(':').collect();

    match parts.as_slice() {
        // Short form: <kind>:<id> → urn:otel:<kind>:<id>
        [kind, id] => {
            let kind = match_kind(raw, kind, accepted_kinds, urn_label)?;
            validate_segment(raw, "otel", "namespace", urn_label)?;
            validate_segment(raw, id, "id", urn_label)?;
            Ok(ParsedKindedUrn {
                namespace: "otel".to_string(),
                kind,
                id: (*id).to_string(),
            })
        }
        // Canonical form: urn:<namespace>:<kind>:<id>
        [scheme, namespace, kind, id] => {
            if !scheme.eq_ignore_ascii_case("urn") {
                return Err(invalid_urn(
                    raw,
                    format!("scheme must be `urn`, found `{scheme}`"),
                    urn_label,
                ));
            }
            let namespace_lower = namespace.to_ascii_lowercase();
            let kind = match_kind(raw, kind, accepted_kinds, urn_label)?;
            validate_segment(raw, &namespace_lower, "namespace", urn_label)?;
            validate_segment(raw, id, "id", urn_label)?;
            Ok(ParsedKindedUrn {
                namespace: namespace_lower,
                kind,
                id: (*id).to_string(),
            })
        }
        _ => Err(invalid_urn(
            raw,
            format!("found {} segment(s)", parts.len()),
            urn_label,
        )),
    }
}

/// Build the canonical `urn:<namespace>:<kind>:<id>` string and the byte
/// ranges of the namespace and id segments.
pub(crate) fn build_canonical_urn(
    namespace: &str,
    kind: &str,
    id: &str,
) -> (String, Range<usize>, Range<usize>) {
    let raw = format!("urn:{namespace}:{kind}:{id}");
    let namespace_start = "urn:".len();
    let namespace_end = namespace_start + namespace.len();
    let id_start = namespace_end + 1 + kind.len() + 1;
    let id_end = id_start + id.len();
    (raw, namespace_start..namespace_end, id_start..id_end)
}

/// Returns `true` if `seg` consists only of characters allowed in URN
/// segments: ASCII lowercase letters, digits, `_`, `-`, or `.`.
pub(crate) fn is_valid_segment(seg: &str) -> bool {
    !seg.is_empty()
        && seg
            .chars()
            .all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.'))
}

fn validate_segment(raw: &str, segment: &str, label: &str, urn_label: &str) -> Result<(), Error> {
    if segment.is_empty() {
        return Err(invalid_urn(
            raw,
            format!("{label} must be non-empty"),
            urn_label,
        ));
    }
    if !is_valid_segment(segment) {
        return Err(invalid_urn(
            raw,
            format!("{label} `{segment}` must match [a-z0-9._-]"),
            urn_label,
        ));
    }
    Ok(())
}

fn match_kind<'a>(
    raw: &str,
    kind: &str,
    accepted_kinds: &'a [&'a str],
    urn_label: &str,
) -> Result<&'a str, Error> {
    if let Some(matched) = accepted_kinds
        .iter()
        .copied()
        .find(|expected| expected.eq_ignore_ascii_case(kind))
    {
        return Ok(matched);
    }

    // Kind isn't in this URN flavour's accepted set. If it's a *known*
    // kind (i.e., recognized as one of the other URN flavours' kinds),
    // include a hint pointing the user at the right section. This makes
    // misplacement errors actionable instead of just "kind X is invalid".
    let hint = misplaced_kind_hint(kind);
    let expected = format_accepted_kinds(accepted_kinds);
    let details = match hint {
        Some(suggestion) => format!("expected kind {expected}, found `{kind}` ({suggestion})"),
        None => format!("expected kind {expected}, found `{kind}`"),
    };
    Err(invalid_urn(raw, details, urn_label))
}

/// Returns a hint string suggesting the correct config section when a
/// recognized-but-misplaced kind segment is encountered.
fn misplaced_kind_hint(kind: &str) -> Option<&'static str> {
    match kind.to_ascii_lowercase().as_str() {
        "receiver" | "processor" | "exporter" => {
            Some("declare under `nodes:` instead of `extensions:`")
        }
        "extension" => Some("declare under `extensions:` instead of `nodes:`"),
        _ => None,
    }
}

fn format_accepted_kinds(accepted: &[&str]) -> String {
    match accepted {
        [] => "<none>".to_string(),
        [single] => format!("`{single}`"),
        [a, b] => format!("`{a}` or `{b}`"),
        many => {
            let head = many[..many.len() - 1]
                .iter()
                .map(|k| format!("`{k}`"))
                .collect::<Vec<_>>()
                .join(", ");
            let last = many[many.len() - 1];
            format!("{head}, or `{last}`")
        }
    }
}

fn invalid_urn(raw: &str, details: String, urn_label: &str) -> Error {
    Error::InvalidUserConfig {
        error: format!(
            "invalid {urn_label} `{raw}`: {details}; expected \
             `urn:<namespace>:<kind>:<id>` or `<kind>:<id>` for otel \
             (see {URN_DOCS_PATH})"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NODE_KINDS: &[&str] = &["receiver", "processor", "exporter"];
    const EXTENSION_KINDS: &[&str] = &["extension"];

    #[test]
    fn parses_canonical_4_segment() {
        let parsed = parse_kinded_urn("urn:otap:receiver:otlp", NODE_KINDS, "test urn").unwrap();
        assert_eq!(parsed.namespace, "otap");
        assert_eq!(parsed.kind, "receiver");
        assert_eq!(parsed.id, "otlp");
    }

    #[test]
    fn parses_short_form() {
        let parsed = parse_kinded_urn("receiver:otlp", NODE_KINDS, "test urn").unwrap();
        assert_eq!(parsed.namespace, "otel");
        assert_eq!(parsed.kind, "receiver");
    }

    #[test]
    fn rejects_kind_not_in_accepted_set() {
        // `extension` is not in the node kinds set.
        assert!(parse_kinded_urn("urn:otap:extension:foo", NODE_KINDS, "test urn").is_err());
        // And node kinds are not in the extension set.
        assert!(parse_kinded_urn("urn:otap:receiver:foo", EXTENSION_KINDS, "test urn").is_err());
    }

    #[test]
    fn rejects_3_segment() {
        assert!(parse_kinded_urn("urn:otap:foo", NODE_KINDS, "test urn").is_err());
        assert!(parse_kinded_urn("urn:otap:foo", EXTENSION_KINDS, "test urn").is_err());
    }

    #[test]
    fn rejects_bare_id() {
        assert!(parse_kinded_urn("foo", NODE_KINDS, "test urn").is_err());
        assert!(parse_kinded_urn("foo", EXTENSION_KINDS, "test urn").is_err());
    }

    #[test]
    fn case_insensitive_kind_and_scheme() {
        let parsed = parse_kinded_urn("URN:Otap:RECEIVER:foo", NODE_KINDS, "test urn").unwrap();
        // Returned kind has the accepted-kinds casing (lowercase).
        assert_eq!(parsed.kind, "receiver");
        assert_eq!(parsed.namespace, "otap");
    }

    #[test]
    fn build_round_trips() {
        let (raw, ns_range, id_range) = build_canonical_urn("otap", "receiver", "otlp");
        assert_eq!(raw, "urn:otap:receiver:otlp");
        assert_eq!(&raw[ns_range], "otap");
        assert_eq!(&raw[id_range], "otlp");
    }
}

#[cfg(test)]
mod display_tests {
    use crate::extension_urn::ExtensionUrn;
    use crate::node_urn::NodeUrn;

    #[test]
    fn shows_misplaced_extension_under_nodes() {
        let err = NodeUrn::parse("urn:otel:extension:azure_auth").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("declare under `extensions:`"),
            "expected misplacement hint, got: {msg}"
        );
    }

    #[test]
    fn shows_misplaced_receiver_under_extensions() {
        let err = ExtensionUrn::parse("urn:otel:receiver:otlp").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("declare under `nodes:`"),
            "expected misplacement hint, got: {msg}"
        );
    }
}
