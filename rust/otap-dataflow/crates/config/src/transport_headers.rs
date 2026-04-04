// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Protocol-neutral transport header abstraction for end-to-end header
//! propagation through the pipeline.
//!
//! Transport headers represent request-scoped metadata captured from inbound
//! transport protocols (gRPC metadata, HTTP headers) and carried through the
//! pipeline context.
//!
//! The abstraction preserves:
//! - Duplicate header names (multiple entries with the same logical name)
//! - Binary values (e.g. gRPC binary metadata with `-bin` suffix)
//! - Original wire names for lossless round-tripping
//! - Normalized logical names for policy matching

use std::fmt;

use crate::transport_headers_policy::{
    CaptureRule, HeaderCapturePolicy, HeaderPropagationPolicy, NameStrategy, PropagationAction,
    PropagationSelector, ValueKindConfig,
};

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

/// A single captured transport header.
///
/// Each entry records both the normalized logical name (used for policy
/// matching) and the original wire name observed on ingress (used for
/// lossless re-emission on egress).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportHeader {
    /// Normalized logical name used for matching and policy lookup.
    pub name: String,
    /// Original header or metadata name observed on ingress.
    pub wire_name: String,
    /// Whether the value is text or binary.
    pub value_kind: ValueKind,
    /// Raw value bytes.
    pub value: Vec<u8>,
}

impl TransportHeader {
    /// Create a new text transport header.
    #[must_use]
    pub fn text(
        name: impl Into<String>,
        wire_name: impl Into<String>,
        value: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            name: name.into(),
            wire_name: wire_name.into(),
            value_kind: ValueKind::Text,
            value: value.into(),
        }
    }

    /// Create a new binary transport header.
    #[must_use]
    pub fn binary(
        name: impl Into<String>,
        wire_name: impl Into<String>,
        value: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            name: name.into(),
            wire_name: wire_name.into(),
            value_kind: ValueKind::Binary,
            value: value.into(),
        }
    }

    /// Returns the value as a UTF-8 string, if it is valid text.
    #[must_use]
    pub fn value_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.value).ok()
    }
}

/// An ordered collection of captured transport headers.
///
/// Headers are stored as a `Vec` to preserve insertion order and allow
/// duplicate names (same logical name appearing multiple times), which
/// is valid in both HTTP and gRPC metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransportHeaders {
    headers: Vec<TransportHeader>,
}

impl TransportHeaders {
    /// Create an empty header collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    /// Create a header collection with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            headers: Vec::with_capacity(capacity),
        }
    }

    /// Add a header to the collection.
    pub fn push(&mut self, header: TransportHeader) {
        self.headers.push(header);
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

    /// Find all headers matching a normalized name (case-sensitive match on
    /// the logical name).
    pub fn find_by_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a TransportHeader> {
        self.headers.iter().filter(move |h| h.name == name)
    }

    /// Returns a slice of all headers.
    #[must_use]
    pub fn as_slice(&self) -> &[TransportHeader] {
        &self.headers
    }
}

// -- Capture engine -----------------------------------------------------------

/// Applies a [`HeaderCapturePolicy`] to extract matching headers from
/// a protocol-specific source (gRPC metadata, HTTP headers, or raw
/// key-value pairs) and produce a [`TransportHeaders`] collection.
#[derive(Debug, Clone)]
pub struct CaptureEngine {
    policy: HeaderCapturePolicy,
}

impl CaptureEngine {
    /// Create a new capture engine from a policy.
    #[must_use]
    pub fn new(policy: HeaderCapturePolicy) -> Self {
        Self { policy }
    }

    /// Capture headers from an iterator of `(wire_name, value)` pairs.
    ///
    /// Each pair is matched against the capture rules. Only headers
    /// matching at least one rule are captured, subject to the configured
    /// limits.
    pub fn capture_from_pairs<'a>(
        &self,
        pairs: impl Iterator<Item = (&'a str, &'a [u8])>,
    ) -> TransportHeaders {
        if self.policy.is_empty() {
            return TransportHeaders::new();
        }

        let defaults = &self.policy.defaults;
        let mut result = TransportHeaders::with_capacity(defaults.max_entries.min(16));

        for (wire_name, value) in pairs {
            if result.len() >= defaults.max_entries {
                break;
            }

            if let Some(matched_rule) = self.find_matching_rule(wire_name) {
                // Enforce name length limit — drop oversized names.
                if wire_name.len() > defaults.max_name_bytes {
                    continue;
                }

                // Enforce value length limit — drop oversized values.
                if value.len() > defaults.max_value_bytes {
                    continue;
                }

                let name = matched_rule
                    .store_as
                    .clone()
                    .unwrap_or_else(|| wire_name.to_ascii_lowercase());

                let value_kind = match matched_rule.value_kind {
                    Some(ValueKindConfig::Text) => ValueKind::Text,
                    Some(ValueKindConfig::Binary) => ValueKind::Binary,
                    None => {
                        if wire_name.ends_with("-bin") {
                            ValueKind::Binary
                        } else {
                            ValueKind::Text
                        }
                    }
                };

                result.push(TransportHeader {
                    name,
                    wire_name: wire_name.to_string(),
                    value_kind,
                    value: value.to_vec(),
                });
            }
        }

        result
    }

    /// Find the first capture rule whose `match_names` contains the given
    /// wire name (case-insensitive comparison).
    fn find_matching_rule(&self, wire_name: &str) -> Option<&CaptureRule> {
        let wire_lower = wire_name.to_ascii_lowercase();
        self.policy.headers.iter().find(|rule| {
            rule.match_names
                .iter()
                .any(|m| m.to_ascii_lowercase() == wire_lower)
        })
    }
}

// -- Propagation engine -------------------------------------------------------

/// Applies a [`HeaderPropagationPolicy`] to filter captured headers for
/// outbound emission.
#[derive(Debug, Clone)]
pub struct PropagationEngine {
    policy: HeaderPropagationPolicy,
}

impl PropagationEngine {
    /// Create a new propagation engine from a policy.
    #[must_use]
    pub fn new(policy: HeaderPropagationPolicy) -> Self {
        Self { policy }
    }

    /// Apply the propagation policy to a set of captured headers,
    /// returning only those headers that should be sent on egress.
    #[must_use]
    pub fn propagate(&self, captured: &TransportHeaders) -> TransportHeaders {
        let mut result = TransportHeaders::with_capacity(captured.len());

        for header in captured.iter() {
            let (action, name_strategy) = self.resolve_action(header);
            if action == PropagationAction::Drop {
                continue;
            }

            let wire_name = match name_strategy {
                NameStrategy::Preserve => header.wire_name.clone(),
                NameStrategy::StoredName => header.name.clone(),
            };

            result.push(TransportHeader {
                name: header.name.clone(),
                wire_name,
                value_kind: header.value_kind.clone(),
                value: header.value.clone(),
            });
        }

        result
    }

    /// Determine the action and name strategy for a single header by
    /// checking overrides first, then falling back to the default.
    fn resolve_action(&self, header: &TransportHeader) -> (PropagationAction, NameStrategy) {
        // Check overrides first.
        for ov in &self.policy.overrides {
            let name_lower = header.name.to_ascii_lowercase();
            if ov
                .match_rule
                .stored_names
                .iter()
                .any(|s| s.to_ascii_lowercase() == name_lower)
            {
                let name_strategy = ov.name.unwrap_or(self.policy.default.name);
                return (ov.action, name_strategy);
            }
        }

        // Check whether the header passes the default selector.
        let selected = match &self.policy.default.selector {
            PropagationSelector::AllCaptured => true,
            PropagationSelector::None => false,
            PropagationSelector::Named(names) => {
                let name_lower = header.name.to_ascii_lowercase();
                names.iter().any(|n| n.to_ascii_lowercase() == name_lower)
            }
        };

        if selected {
            (self.policy.default.action, self.policy.default.name)
        } else {
            (PropagationAction::Drop, self.policy.default.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport_headers_policy::{
        CaptureDefaults, PropagationDefault, PropagationMatch, PropagationOverride,
    };

    #[test]
    fn find_by_name_returns_matching_headers() {
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text("tenant", "X-Tenant", b"a".to_vec()));
        headers.push(TransportHeader::text(
            "request-id",
            "X-Request-Id",
            b"b".to_vec(),
        ));
        headers.push(TransportHeader::text("tenant", "X-Tenant", b"c".to_vec()));

        let tenants: Vec<_> = headers.find_by_name("tenant").collect();
        assert_eq!(tenants.len(), 2);
        assert_eq!(tenants[0].value, b"a");
        assert_eq!(tenants[1].value, b"c");
    }

    #[test]
    fn duplicate_names_preserved() {
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text("key", "Key", b"val1".to_vec()));
        headers.push(TransportHeader::text("key", "Key", b"val2".to_vec()));
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn value_as_str_for_text() {
        let h = TransportHeader::text("name", "Name", b"hello".to_vec());
        assert_eq!(h.value_as_str(), Some("hello"));
    }

    #[test]
    fn value_as_str_for_invalid_utf8() {
        let h = TransportHeader::binary("name-bin", "name-bin", vec![0xFF, 0xFE]);
        assert_eq!(h.value_as_str(), None);
    }

    // -- Capture engine tests ------------------------------------------------

    fn make_capture_policy(rules: Vec<CaptureRule>) -> HeaderCapturePolicy {
        HeaderCapturePolicy {
            defaults: CaptureDefaults::default(),
            headers: rules,
        }
    }

    fn rule(names: &[&str], store_as: Option<&str>) -> CaptureRule {
        CaptureRule {
            match_names: names.iter().map(|s| s.to_string()).collect(),
            store_as: store_as.map(|s| s.to_string()),
            sensitive: false,
            value_kind: None,
        }
    }

    #[test]
    fn capture_empty_policy_captures_nothing() {
        let engine = CaptureEngine::new(HeaderCapturePolicy::default());
        let pairs = vec![("X-Tenant-Id", b"abc" as &[u8])];
        let result = engine.capture_from_pairs(pairs.into_iter());
        assert!(result.is_empty());
    }

    #[test]
    fn capture_matching_headers() {
        let policy = make_capture_policy(vec![
            rule(&["x-tenant-id"], Some("tenant_id")),
            rule(&["x-request-id"], None),
        ]);
        let engine = CaptureEngine::new(policy);

        let pairs: Vec<(&str, &[u8])> = vec![
            ("X-Tenant-Id", b"t-123"),
            ("X-Request-Id", b"r-456"),
            ("X-Unmatched", b"ignored"),
        ];
        let result = engine.capture_from_pairs(pairs.into_iter());
        assert_eq!(result.len(), 2);
        assert_eq!(result.as_slice()[0].name, "tenant_id");
        assert_eq!(result.as_slice()[0].wire_name, "X-Tenant-Id");
        assert_eq!(result.as_slice()[0].value, b"t-123");
        assert_eq!(result.as_slice()[1].name, "x-request-id");
    }

    #[test]
    fn capture_case_insensitive_matching() {
        let policy = make_capture_policy(vec![rule(&["x-tenant-id"], None)]);
        let engine = CaptureEngine::new(policy);

        let pairs: Vec<(&str, &[u8])> = vec![("X-TENANT-ID", b"val")];
        let result = engine.capture_from_pairs(pairs.into_iter());
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].name, "x-tenant-id");
        assert_eq!(result.as_slice()[0].wire_name, "X-TENANT-ID");
    }

    #[test]
    fn capture_respects_max_entries() {
        let mut policy = make_capture_policy(vec![rule(&["x-key"], None)]);
        policy.defaults.max_entries = 2;
        let engine = CaptureEngine::new(policy);

        let pairs: Vec<(&str, &[u8])> = vec![("x-key", b"1"), ("x-key", b"2"), ("x-key", b"3")];
        let result = engine.capture_from_pairs(pairs.into_iter());
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn capture_drops_oversized_value() {
        let mut policy = make_capture_policy(vec![rule(&["x-key"], None)]);
        policy.defaults.max_value_bytes = 3;
        let engine = CaptureEngine::new(policy);

        let pairs: Vec<(&str, &[u8])> = vec![("x-key", b"toolong"), ("x-key", b"ok")];
        let result = engine.capture_from_pairs(pairs.into_iter());
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].value, b"ok");
    }

    #[test]
    fn capture_binary_detection() {
        let policy = make_capture_policy(vec![rule(&["auth-token-bin"], None)]);
        let engine = CaptureEngine::new(policy);

        let pairs: Vec<(&str, &[u8])> = vec![("auth-token-bin", &[0xFF, 0x00])];
        let result = engine.capture_from_pairs(pairs.into_iter());
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].value_kind, ValueKind::Binary);
    }

    // -- Propagation engine tests --------------------------------------------

    #[test]
    fn propagate_all_captured_default() {
        let engine = PropagationEngine::new(HeaderPropagationPolicy::default());
        let mut captured = TransportHeaders::new();
        captured.push(TransportHeader::text(
            "tenant_id",
            "X-Tenant-Id",
            b"t-1".to_vec(),
        ));
        captured.push(TransportHeader::text(
            "request_id",
            "X-Request-Id",
            b"r-1".to_vec(),
        ));

        let result = engine.propagate(&captured);
        assert_eq!(result.len(), 2);
        assert_eq!(result.as_slice()[0].wire_name, "X-Tenant-Id");
        assert_eq!(result.as_slice()[1].wire_name, "X-Request-Id");
    }

    #[test]
    fn propagate_override_drops_auth() {
        let policy = HeaderPropagationPolicy {
            default: PropagationDefault::default(),
            overrides: vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec!["authorization".to_string()],
                },
                action: PropagationAction::Drop,
                name: None,
                on_error: None,
            }],
        };
        let engine = PropagationEngine::new(policy);

        let mut captured = TransportHeaders::new();
        captured.push(TransportHeader::text(
            "tenant_id",
            "X-Tenant-Id",
            b"t-1".to_vec(),
        ));
        captured.push(TransportHeader::text(
            "authorization",
            "Authorization",
            b"Bearer secret".to_vec(),
        ));

        let result = engine.propagate(&captured);
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].name, "tenant_id");
    }

    #[test]
    fn propagate_selector_none_drops_all_unless_override() {
        let policy = HeaderPropagationPolicy {
            default: PropagationDefault {
                selector: PropagationSelector::None,
                ..PropagationDefault::default()
            },
            overrides: vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec!["tenant_id".to_string()],
                },
                action: PropagationAction::Propagate,
                name: None,
                on_error: None,
            }],
        };
        let engine = PropagationEngine::new(policy);

        let mut captured = TransportHeaders::new();
        captured.push(TransportHeader::text(
            "tenant_id",
            "X-Tenant-Id",
            b"t-1".to_vec(),
        ));
        captured.push(TransportHeader::text(
            "request_id",
            "X-Request-Id",
            b"r-1".to_vec(),
        ));

        let result = engine.propagate(&captured);
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].name, "tenant_id");
    }

    #[test]
    fn propagate_stored_name_strategy() {
        let policy = HeaderPropagationPolicy {
            default: PropagationDefault {
                name: NameStrategy::StoredName,
                ..PropagationDefault::default()
            },
            overrides: vec![],
        };
        let engine = PropagationEngine::new(policy);

        let mut captured = TransportHeaders::new();
        captured.push(TransportHeader::text(
            "tenant_id",
            "X-Tenant-Id",
            b"t-1".to_vec(),
        ));

        let result = engine.propagate(&captured);
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].wire_name, "tenant_id");
    }

    #[test]
    fn propagate_named_selector() {
        let policy = HeaderPropagationPolicy {
            default: PropagationDefault {
                selector: PropagationSelector::Named(vec!["tenant_id".to_string()]),
                ..PropagationDefault::default()
            },
            overrides: vec![],
        };
        let engine = PropagationEngine::new(policy);

        let mut captured = TransportHeaders::new();
        captured.push(TransportHeader::text(
            "tenant_id",
            "X-Tenant-Id",
            b"t-1".to_vec(),
        ));
        captured.push(TransportHeader::text(
            "request_id",
            "X-Request-Id",
            b"r-1".to_vec(),
        ));

        let result = engine.propagate(&captured);
        assert_eq!(result.len(), 1);
        assert_eq!(result.as_slice()[0].name, "tenant_id");
    }
}
