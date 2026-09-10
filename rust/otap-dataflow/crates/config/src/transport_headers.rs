// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport headers carried through the pipeline context.
//!
//! Headers can come from gRPC metadata, HTTP headers, or other transports.
//!
//! Preserves:
//! - Duplicate header names
//! - Text and binary values
//! - Original wire names when required
//! - Normalized context entry names for policy matching

use crate::context::ContextEntryName;
use std::fmt;
use std::sync::Arc;

/// Kind of header value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueKind {
    /// UTF-8 text value.
    Text,
    /// Arbitrary binary value (e.g. gRPC `-bin` metadata).
    Binary,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Text => write!(f, "text"),
            ValueKind::Binary => write!(f, "binary"),
        }
    }
}

/// A header value and its optional original name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportHeaderValue {
    /// Original wire name when required and different from the stored name.
    pub original_name: Option<Box<str>>,
    /// Whether the value is text or binary.
    pub value_kind: ValueKind,
    /// Raw value bytes.
    pub bytes: Box<[u8]>,
}

/// A transport header stored in context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportHeader {
    /// Normalized context entry name.
    pub name: ContextEntryName,
    /// Value and optional original wire name.
    pub value: TransportHeaderValue,
}

impl TransportHeader {
    /// Creates a header without an original wire name.
    #[must_use]
    pub fn new<V: Into<Box<[u8]>>>(
        name: ContextEntryName,
        value_kind: ValueKind,
        value: V,
    ) -> Self {
        TransportHeader {
            name,
            value: TransportHeaderValue {
                original_name: None,
                value_kind,
                bytes: value.into(),
            },
        }
    }

    /// Creates a captured header. Keeps a distinct original name when requested.
    #[must_use]
    pub fn captured<V: Into<Box<[u8]>>>(
        name: ContextEntryName,
        wire_name: &str,
        preserve_original_name: bool,
        value_kind: ValueKind,
        value: V,
    ) -> Self {
        let original_name = (preserve_original_name && name.as_str() != wire_name)
            .then(|| wire_name.to_owned().into_boxed_str());
        Self {
            name,
            value: TransportHeaderValue {
                original_name,
                value_kind,
                bytes: value.into(),
            },
        }
    }

    /// Creates a text header.
    #[must_use]
    pub fn text(name: ContextEntryName, value: impl Into<Vec<u8>>) -> Self {
        Self::new(name, ValueKind::Text, value.into())
    }

    /// Creates a binary header.
    #[must_use]
    pub fn binary(name: ContextEntryName, value: impl Into<Vec<u8>>) -> Self {
        Self::new(name, ValueKind::Binary, value.into())
    }

    /// Returns the original wire name or the normalized stored name.
    #[must_use]
    pub fn wire_name(&self) -> &str {
        self.value.original_name.as_deref().unwrap_or(&self.name)
    }

    /// Returns the value as a UTF-8 string, if it is valid text.
    #[must_use]
    pub fn value_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.value.bytes).ok()
    }
}

/// An ordered collection of captured transport headers.
///
/// Headers are stored as a `Vec` to preserve insertion order and allow
/// duplicate names (same logical name appearing multiple times), which
/// is valid in both HTTP and gRPC metadata.
///
/// The inner vector is wrapped in an `Arc` so that cloning a
/// `TransportHeaders` (e.g. when cloning a pipeline `Context`) is a
/// cheap reference-count bump instead of a deep copy.  Mutation
/// methods (`push`, `clear`) use copy-on-write via `Arc::make_mut`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransportHeaders {
    headers: Arc<Vec<TransportHeader>>,
}

impl TransportHeaders {
    /// Create an empty header collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            headers: Arc::new(Vec::new()),
        }
    }

    /// Create a header collection with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            headers: Arc::new(Vec::with_capacity(capacity)),
        }
    }

    /// Add a header to the collection.
    pub fn push(&mut self, header: TransportHeader) {
        Arc::make_mut(&mut self.headers).push(header);
    }

    /// Clears the headers and reserves space for `capacity` entries.
    pub(crate) fn clear_and_reserve(&mut self, capacity: usize) -> &mut Vec<TransportHeader> {
        if Arc::strong_count(&self.headers) != 1 {
            self.headers = Arc::new(Vec::with_capacity(capacity));
        }
        let headers = Arc::make_mut(&mut self.headers);
        headers.clear();
        headers.reserve(capacity);
        headers
    }

    /// Returns `true` if there are no headers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// Returns the number of headers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Iterate over all headers.
    pub fn iter(&self) -> impl Iterator<Item = &TransportHeader> {
        self.headers.iter()
    }

    /// Finds headers by exact normalized name.
    /// Uses a linear scan for validation.
    pub fn find_by_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a TransportHeader> {
        self.headers.iter().filter(move |h| h.name.as_str() == name)
    }

    /// Returns a slice of all headers.
    #[must_use]
    pub fn as_slice(&self) -> &[TransportHeader] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport_headers_policy::{
        CaptureDefaults, CaptureRule, HeaderCapturePolicy, HeaderPropagationPolicy, NameStrategy,
        PropagationAction, PropagationDefault, PropagationMatch, PropagationOverride,
        PropagationSelector, PropagationSelectorType,
    };

    fn context_name(raw: &str) -> ContextEntryName {
        ContextEntryName::try_from(raw).expect("valid test context entry name")
    }

    fn header(normal: &str, wire_name: &str, value: impl AsRef<[u8]>) -> TransportHeader {
        TransportHeader::captured(
            context_name(normal),
            wire_name,
            true,
            ValueKind::Text,
            value.as_ref(),
        )
    }

    /// Scenario: matching and nonmatching headers are interleaved.
    /// Guarantees: lookup preserves all matching values in order.
    #[test]
    fn find_by_name_returns_matching_headers() {
        let mut headers = TransportHeaders::new();
        headers.push(header("tenant", "X-Tenant", b"a"));
        headers.push(header("request-id", "X-Request-Id", b"b"));
        headers.push(header("tenant", "X-Tenant", b"c"));

        let tenants: Vec<_> = headers.find_by_name("tenant").collect();
        assert_eq!(tenants.len(), 2);
        assert_eq!(&*tenants[0].value.bytes, b"a");
        assert_eq!(&*tenants[1].value.bytes, b"c");
    }

    /// Scenario: two headers have the same name.
    /// Guarantees: neither header replaces the other.
    #[test]
    fn duplicate_names_preserved() {
        let mut headers = TransportHeaders::new();
        headers.push(header("key", "Key", b"val1"));
        headers.push(header("key", "Key", b"val2"));
        assert_eq!(headers.len(), 2);
    }

    /// Scenario: a text header contains valid UTF-8.
    /// Guarantees: the string view preserves the text.
    #[test]
    fn value_as_str_for_text() {
        let h = header("name", "Name", b"hello");
        assert_eq!(h.value_as_str(), Some("hello"));
    }

    /// Scenario: a binary header contains invalid UTF-8.
    /// Guarantees: the string view returns `None`.
    #[test]
    fn value_as_str_for_invalid_utf8() {
        let h = TransportHeader::binary(context_name("name-bin"), vec![0xFF, 0xFE]);
        assert_eq!(h.value_as_str(), None);
    }

    // -- Capture engine tests ------------------------------------------------

    fn make_capture_policy(rules: Vec<CaptureRule>) -> HeaderCapturePolicy {
        HeaderCapturePolicy::new(CaptureDefaults::default(), rules)
    }

    fn rule(names: &[&str], store_as: Option<&str>) -> CaptureRule {
        CaptureRule {
            match_names: names.iter().map(|n| context_name(n)).collect(),
            store_as: store_as.map(context_name),
            sensitive: false,
            value_kind: None,
        }
    }

    /// Scenario: headers arrive with no capture rules.
    /// Guarantees: capture returns no headers or limit errors.
    #[test]
    fn capture_empty_policy_captures_nothing() {
        let policy = HeaderCapturePolicy::default().compile(|_| true);
        let pairs = vec![("X-Tenant-Id", b"abc" as &[u8])];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert!(result.is_empty());
        assert!(stats.is_none());
    }

    /// Scenario: capture rules select and rename incoming headers.
    /// Guarantees: only matches are stored. Values and original names are retained.
    #[test]
    fn capture_matching_headers() {
        let policy = make_capture_policy(vec![
            rule(&["x-tenant-id"], Some("tenant_id")),
            rule(&["x-request-id"], None),
        ])
        .compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![
            ("X-Tenant-Id", b"t-123"),
            ("X-Request-Id", b"r-456"),
            ("X-Unmatched", b"ignored"),
        ];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert!(stats.is_none());
        assert_eq!(result.len(), 2);
        assert_eq!(result.as_slice()[0].name, "tenant_id");
        assert_eq!(result.as_slice()[0].wire_name(), "X-Tenant-Id");
        assert_eq!(&*result.as_slice()[0].value.bytes, b"t-123");
        assert_eq!(result.as_slice()[1].name, "x-request-id");
    }

    /// Scenario: a wire name uses different casing from its capture rule.
    /// Guarantees: it matches the rule and retains its wire spelling.
    #[test]
    fn capture_case_insensitive_matching() {
        let policy = make_capture_policy(vec![rule(&["x-tenant-id"], None)]).compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![("X-TENANT-ID", b"val")];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert!(stats.is_none());
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].name, "x-tenant-id");
        assert_eq!(result.as_slice()[0].wire_name(), "X-TENANT-ID");
    }

    /// Scenario: one capture rule matches several wire names.
    /// Guarantees: every alias is captured under the same stored name.
    #[test]
    fn capture_rule_supports_multiple_match_names() {
        let policy = make_capture_policy(vec![rule(&["x-first", "x-second"], Some("combined"))])
            .compile(|_| false);

        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(
            [
                ("X-First", b"first".as_slice()),
                ("X-Second", b"second".as_slice()),
            ]
            .into_iter(),
            &mut result,
        );

        assert!(stats.is_none());
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|header| header.name == "combined"));
    }

    /// Scenario: matching headers exceed the entry limit.
    /// Guarantees: capture respects the limit and counts excess matches.
    #[test]
    fn capture_respects_max_entries() {
        let mut policy = make_capture_policy(vec![rule(&["x-key"], None)]);
        policy.defaults.max_entries = 2;
        let policy = policy.compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![("x-key", b"1"), ("x-key", b"2"), ("x-key", b"3")];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert_eq!(result.len(), 2);
        let stats = stats.expect("should report skipped headers");
        assert_eq!(stats.skipped_max_entries, 1);
        assert_eq!(stats.skipped_name_too_long, 0);
        assert_eq!(stats.skipped_value_too_long, 0);
    }

    /// Scenario: an oversized value precedes a valid value.
    /// Guarantees: the oversized value is counted and dropped. The valid value is captured.
    #[test]
    fn capture_drops_oversized_value() {
        let mut policy = make_capture_policy(vec![rule(&["x-key"], None)]);
        policy.defaults.max_value_bytes = 3;
        let policy = policy.compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![("x-key", b"toolong"), ("x-key", b"ok")];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert_eq!(result.len(), 1);
        assert_eq!(&*result.as_slice()[0].value.bytes, b"ok");
        let stats = stats.expect("should report skipped headers");
        assert_eq!(stats.skipped_value_too_long, 1);
        assert_eq!(stats.skipped_max_entries, 0);
        assert_eq!(stats.skipped_name_too_long, 0);
    }

    /// Scenario: a `-bin` header contains non-text bytes.
    /// Guarantees: capture accepts the bytes and marks the value as binary.
    #[test]
    fn capture_binary_detection() {
        let policy = make_capture_policy(vec![rule(&["auth-token-bin"], None)]).compile(|_| true);

        let pairs: Vec<(&str, &[u8])> = vec![("auth-token-bin", &[0xFF, 0x00])];
        let mut result = TransportHeaders::new();
        let stats = policy.capture_from_pairs(pairs.into_iter(), &mut result);
        assert!(stats.is_none());
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].value.value_kind, ValueKind::Binary);
    }

    // -- Propagation policy tests --------------------------------------------

    /// Scenario: propagation selects all captured headers.
    /// Guarantees: every header keeps its original wire name.
    #[test]
    fn propagate_all_captured_default() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 2);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
        assert_eq!(propagated[1].header_name, "X-Request-Id");
    }

    /// Scenario: a drop override excludes authorization.
    /// Guarantees: other headers still propagate with their original names.
    #[test]
    fn propagate_override_drops_auth() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![context_name("authorization")],
                },
                action: PropagationAction::Drop,
                name: None,
                on_error: None,
            }],
        );

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("authorization", "Authorization", b"Bearer secret"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }

    /// Scenario: a `none` selector has one propagation override.
    /// Guarantees: only the overridden header is propagated.
    #[test]
    fn propagate_selector_none_drops_all_unless_override() {
        let policy = HeaderPropagationPolicy {
            default: PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::None,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            overrides: vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![context_name("tenant_id")],
                },
                action: PropagationAction::Propagate,
                name: None,
                on_error: None,
            }],
        };

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }

    /// Scenario: propagation uses `StoredName` for a renamed header.
    /// Guarantees: the stored name replaces the original wire name.
    #[test]
    fn propagate_stored_name_strategy() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                name: NameStrategy::StoredName,
                ..PropagationDefault::default()
            },
            vec![],
        );

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "tenant_id");
    }

    /// Scenario: a named selector lists one captured entry.
    /// Guarantees: only that entry is propagated.
    #[test]
    fn propagate_named_selector() {
        let policy = HeaderPropagationPolicy {
            default: PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::Named,
                    named: Some(vec![context_name("tenant_id")]),
                },
                ..PropagationDefault::default()
            },
            overrides: vec![],
        };

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }
}
