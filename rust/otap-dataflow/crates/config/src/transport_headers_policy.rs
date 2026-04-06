// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport header capture and propagation policy declarations.
//!
//! This policy family controls which inbound transport headers are captured
//! by receivers and which captured headers are propagated by exporters.
//!
//! Extraction and propagation are explicit and opt-in. The default behavior
//! is not to forward any inbound headers.
//!
//! TODO: Implement the sensitive capability for headers

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::transport_headers::{TransportHeader, TransportHeaders, ValueKind};

// -- Stats types --------------------------------------------------------------

/// Statistics returned by [`HeaderCapturePolicy::capture_from_pairs`] when
/// one or more matching headers could not be captured due to policy limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureStats {
    /// Matching headers skipped because `max_entries` was already reached.
    pub skipped_max_entries: usize,
    /// Matching headers skipped because the wire name exceeded `max_name_bytes`.
    pub skipped_name_too_long: usize,
    /// Matching headers skipped because the value exceeded `max_value_bytes`.
    pub skipped_value_too_long: usize,
}

impl fmt::Display for CaptureStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "capture limits exceeded: {} skipped (max_entries), {} skipped (name too long), {} skipped (value too long)",
            self.skipped_max_entries, self.skipped_name_too_long, self.skipped_value_too_long
        )
    }
}

impl std::error::Error for CaptureStats {}

/// A single header selected for propagation
#[derive(Debug)]
pub struct PropagatedHeader<'a> {
    /// The wire name to use on the outbound request.
    ///
    /// Points to `TransportHeader::wire_name` when the name strategy
    /// is [`NameStrategy::Preserve`], or `TransportHeader::name` when
    /// [`NameStrategy::StoredName`].
    pub header_name: &'a str,
    /// Whether the value is text or binary.
    pub value_kind: &'a ValueKind,
    /// Raw value bytes.
    pub value: &'a [u8],
}

/// Transport headers policy controlling capture at receivers and
/// propagation at exporters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransportHeadersPolicy {
    /// Header capture rules applied by receivers.
    #[serde(default)]
    pub header_capture: HeaderCapturePolicy,
    /// Header propagation rules applied by exporters.
    #[serde(default)]
    pub header_propagation: HeaderPropagationPolicy,
}

// -- Header Capture -----------------------------------------------------------

/// Policy controlling which inbound transport headers are captured by
/// receivers and stored in the pipeline context.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HeaderCapturePolicy {
    /// Default limits applied to all captured headers.
    #[serde(default)]
    pub(crate) defaults: CaptureDefaults,
    /// Per-header capture rules. Only headers matching at least one rule
    /// are captured.
    #[serde(default)]
    pub(crate) headers: Vec<CaptureRule>,
}

impl HeaderCapturePolicy {
    /// Create a new capture policy from the given defaults and rules.
    #[must_use]
    pub fn new(defaults: CaptureDefaults, headers: Vec<CaptureRule>) -> Self {
        Self { defaults, headers }
    }

    /// Returns `true` when no capture rules are defined, meaning the policy
    /// will not capture any headers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// Capture headers from an iterator of `(wire_name, value)` pairs.
    ///
    /// Each pair is matched against the capture rules. Only headers
    /// matching at least one rule are captured, subject to the configured
    /// limits. The `result` collection is cleared before populating.
    ///
    /// Returns `None` when all matching headers were captured successfully,
    /// or `Some(CaptureStats)` when one or more matching headers had to be
    /// skipped due to policy limits.
    pub fn capture_from_pairs<'a>(
        &self,
        pairs: impl Iterator<Item = (&'a str, &'a [u8])>,
        result: &mut TransportHeaders,
    ) -> Option<CaptureStats> {
        result.clear();

        if self.is_empty() {
            return None;
        }

        let defaults = &self.defaults;
        let mut skipped_max_entries: usize = 0;
        let mut skipped_name_too_long: usize = 0;
        let mut skipped_value_too_long: usize = 0;

        for (wire_name, value) in pairs {
            if let Some(matched_rule) = self.find_matching_rule(wire_name) {
                // Enforce entry count limit.
                if result.len() >= defaults.max_entries {
                    skipped_max_entries += 1;
                    continue;
                }

                // Enforce name length limit — drop oversized names.
                if wire_name.len() > defaults.max_name_bytes {
                    skipped_name_too_long += 1;
                    continue;
                }

                // Enforce value length limit — drop oversized values.
                if value.len() > defaults.max_value_bytes {
                    skipped_value_too_long += 1;
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

        if skipped_max_entries > 0 || skipped_name_too_long > 0 || skipped_value_too_long > 0 {
            Some(CaptureStats {
                skipped_max_entries,
                skipped_name_too_long,
                skipped_value_too_long,
            })
        } else {
            None
        }
    }

    /// Find the first capture rule whose `match_names` contains the given
    /// wire name (case-insensitive comparison).
    fn find_matching_rule(&self, wire_name: &str) -> Option<&CaptureRule> {
        self.headers.iter().find(|rule| {
            rule.match_names
                .iter()
                .any(|m| wire_name.eq_ignore_ascii_case(m))
        })
    }
}

/// Default limits for header capture.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureDefaults {
    /// Maximum number of headers captured per message.
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    /// Maximum byte length of a header name.
    #[serde(default = "default_max_name_bytes")]
    pub max_name_bytes: usize,
    /// Maximum byte length of a header value.
    #[serde(default = "default_max_value_bytes")]
    pub max_value_bytes: usize,
    /// Action taken when a header violates a limit.
    #[serde(default)]
    pub on_error: ErrorAction,
}

impl Default for CaptureDefaults {
    fn default() -> Self {
        Self {
            max_entries: default_max_entries(),
            max_name_bytes: default_max_name_bytes(),
            max_value_bytes: default_max_value_bytes(),
            on_error: ErrorAction::default(),
        }
    }
}

const fn default_max_entries() -> usize {
    32
}

const fn default_max_name_bytes() -> usize {
    128
}

const fn default_max_value_bytes() -> usize {
    4096
}

/// A single header capture rule.
///
/// Headers whose wire name matches any entry in `match_names`
/// (case-insensitive) are captured.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureRule {
    /// Wire header names to match (case-insensitive).
    pub match_names: Vec<String>,
    /// Normalized logical name to store the header under. If omitted,
    /// defaults to the first matched name lowercased.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_as: Option<String>,
    /// Whether this header contains sensitive data (e.g. auth tokens).
    /// Sensitive headers may receive special treatment in logging and
    /// debug output.
    /// TODO: Implement the sensitive capability for headers
    #[serde(default)]
    pub sensitive: bool,
    /// Override the auto-detected value kind. When omitted, binary is
    /// inferred from the gRPC `-bin` suffix convention; otherwise text
    /// is assumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_kind: Option<ValueKindConfig>,
}

/// Configured value kind for a capture rule.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValueKindConfig {
    /// UTF-8 text.
    Text,
    /// Arbitrary binary bytes.
    Binary,
}

// -- Header Propagation -------------------------------------------------------

/// Policy controlling which captured transport headers are propagated by
/// exporters onto outbound requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HeaderPropagationPolicy {
    /// Default propagation behavior applied to all captured headers.
    #[serde(default)]
    pub(crate) default: PropagationDefault,
    /// Per-header overrides applied after the default.
    #[serde(default)]
    pub(crate) overrides: Vec<PropagationOverride>,
}

impl HeaderPropagationPolicy {
    /// Create a new propagation policy from the given default behavior and overrides.
    #[must_use]
    pub fn new(default: PropagationDefault, overrides: Vec<PropagationOverride>) -> Self {
        Self { default, overrides }
    }

    /// Returns an iterator over headers that should be propagated on
    /// egress. Each [`PropagatedHeader`] borrows from the captured
    /// headers
    ///
    /// Headers whose policy action is [`PropagationAction::Drop`] are
    /// silently skipped. The [`PropagatedHeader::egress_name`] field
    /// points to either the original wire name or the stored name,
    /// depending on the resolved [`NameStrategy`].
    pub fn propagate<'a>(
        &'a self,
        headers: &'a TransportHeaders,
    ) -> impl Iterator<Item = PropagatedHeader<'a>> {
        headers.iter().filter_map(move |header| {
            let (action, name_strategy) = self.resolve_action(header);
            if action == PropagationAction::Drop {
                return None;
            }
            let header_name = match name_strategy {
                NameStrategy::Preserve => &header.wire_name,
                NameStrategy::StoredName => &header.name,
            };
            Some(PropagatedHeader {
                header_name,
                value_kind: &header.value_kind,
                value: &header.value,
            })
        })
    }

    /// Determine the action and name strategy for a single header by
    /// checking overrides first, then falling back to the default.
    fn resolve_action(&self, header: &TransportHeader) -> (PropagationAction, NameStrategy) {
        // Check overrides first.
        for ov in &self.overrides {
            if ov
                .match_rule
                .stored_names
                .iter()
                .any(|s| header.name.eq_ignore_ascii_case(s))
            {
                let name_strategy = ov.name.unwrap_or(self.default.name);
                return (ov.action, name_strategy);
            }
        }

        // Check whether the header passes the default selector.
        let selected = match &self.default.selector {
            PropagationSelector::AllCaptured => true,
            PropagationSelector::None => false,
            PropagationSelector::Named(names) => {
                names.iter().any(|n| header.name.eq_ignore_ascii_case(n))
            }
        };

        if selected {
            (self.default.action, self.default.name)
        } else {
            (PropagationAction::Drop, self.default.name)
        }
    }
}

/// Default propagation behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PropagationDefault {
    /// Which captured headers to select for propagation.
    #[serde(default)]
    pub selector: PropagationSelector,
    /// Default action for selected headers.
    #[serde(default)]
    pub action: PropagationAction,
    /// How to derive the outbound header name from the stored header.
    #[serde(default)]
    pub name: NameStrategy,
    /// Action taken when a header cannot be propagated.
    #[serde(default)]
    pub on_error: ErrorAction,
}

/// Selects which captured headers are candidates for propagation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropagationSelector {
    /// Propagate all captured headers (subject to overrides).
    AllCaptured,
    /// Do not propagate any captured headers by default (overrides may
    /// still select specific headers).
    #[default]
    None,
    /// Propagate only headers whose stored names appear in this list.
    Named(Vec<String>),
}

/// Action to take for a header during propagation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropagationAction {
    /// Include the header on the outbound request.
    #[default]
    Propagate,
    /// Exclude the header from the outbound request.
    Drop,
}

/// Strategy for mapping the stored header name to the outbound wire name.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NameStrategy {
    /// Use the original wire name observed on ingress.
    #[default]
    Preserve,
    /// Use the normalized stored name.
    StoredName,
}

/// Action taken when a header violates a policy constraint.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorAction {
    /// Silently drop the offending header.
    #[default]
    Drop,
}

/// A per-header propagation override.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PropagationOverride {
    /// Matching criteria for this override.
    #[serde(rename = "match")]
    pub match_rule: PropagationMatch,
    /// Action to take for matched headers. Defaults to `propagate`.
    #[serde(default)]
    pub action: PropagationAction,
    /// Override the name strategy for matched headers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<NameStrategy>,
    /// Override the error action for matched headers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_error: Option<ErrorAction>,
}

/// Matching criteria for propagation overrides.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PropagationMatch {
    /// Match headers whose stored (normalized) name appears in this list.
    pub stored_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_capture_policy_captures_nothing() {
        let policy = HeaderCapturePolicy::default();
        assert!(policy.is_empty());
        assert_eq!(policy.defaults.max_entries, 32);
        assert_eq!(policy.defaults.max_name_bytes, 128);
        assert_eq!(policy.defaults.max_value_bytes, 4096);
        assert_eq!(policy.defaults.on_error, ErrorAction::Drop);
    }

    #[test]
    fn default_propagation_policy() {
        let policy = HeaderPropagationPolicy::default();
        assert_eq!(policy.default.selector, PropagationSelector::None);
        assert_eq!(policy.default.action, PropagationAction::Propagate);
        assert_eq!(policy.default.name, NameStrategy::Preserve);
        assert_eq!(policy.default.on_error, ErrorAction::Drop);
        assert!(policy.overrides.is_empty());
    }

    #[test]
    fn capture_policy_serde_roundtrip() {
        let yaml = r#"
defaults:
  max_entries: 16
  max_name_bytes: 64
  max_value_bytes: 2048
  on_error: drop
headers:
  - match_names: ["x-tenant-id"]
    store_as: tenant_id
  - match_names: ["authorization"]
    sensitive: true
  - match_names: ["x-request-id"]
"#;
        let policy: HeaderCapturePolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.defaults.max_entries, 16);
        assert_eq!(policy.defaults.on_error, ErrorAction::Drop);
        assert_eq!(policy.headers.len(), 3);
        assert_eq!(policy.headers[0].store_as.as_deref(), Some("tenant_id"));
        assert!(policy.headers[1].sensitive);
        assert_eq!(policy.headers[2].match_names, vec!["x-request-id"]);

        // roundtrip
        let json = serde_json::to_string(&policy).expect("serialize");
        let back: HeaderCapturePolicy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, policy);
    }

    #[test]
    fn propagation_policy_serde_roundtrip() {
        let yaml = r#"
default:
  selector: all_captured
  action: propagate
  name: preserve
  on_error: drop
overrides:
  - match:
      stored_names: ["authorization"]
    action: drop
"#;
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.overrides.len(), 1);
        assert_eq!(
            policy.overrides[0].match_rule.stored_names,
            vec!["authorization"]
        );
        assert_eq!(policy.overrides[0].action, PropagationAction::Drop);

        let json = serde_json::to_string(&policy).expect("serialize");
        let back: HeaderPropagationPolicy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, policy);
    }

    #[test]
    fn full_transport_headers_policy_serde() {
        let yaml = r#"
header_capture:
  defaults:
    max_entries: 32
  headers:
    - match_names: ["x-tenant-id"]
      store_as: tenant_id
header_propagation:
  default:
    selector: all_captured
  overrides:
    - match:
        stored_names: ["authorization"]
      action: drop
"#;
        let policy: TransportHeadersPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.header_capture.headers.len(), 1);
        assert_eq!(policy.header_propagation.overrides.len(), 1);
    }

    #[test]
    fn selector_named_variant() {
        let yaml = r#"!named
- tenant_id
- request_id
"#;
        let selector: PropagationSelector = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(
            selector,
            PropagationSelector::Named(vec!["tenant_id".to_string(), "request_id".to_string()])
        );
    }
}
