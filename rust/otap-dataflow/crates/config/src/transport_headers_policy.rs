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

use crate::context::ContextEntryName;
use crate::transport_headers::{CapturedTransportHeader, TransportHeaders, ValueKind};
use hashbrown::{Equivalent, HashMap};
use http::{HeaderMap, HeaderName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};

// -- Stats types --------------------------------------------------------------

/// Counts matching headers skipped by capture limits.
///
/// Returned by the generic and native HTTP capture paths. Capture continues
/// after a limit violation so callers can retain accepted headers and decide
/// separately how to observe the skipped entries.
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
    /// The name to use on the outbound request.
    pub header_name: &'a str,
    /// Whether the value is text or binary.
    pub value_kind: ValueKind,
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
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
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

/// Capture rules indexed by case-insensitive wire name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledHeaderCapturePolicy {
    defaults: CaptureDefaults,
    captures: HashMap<CaptureKey, CompiledCapture>,
    http_captures: HashMap<HeaderName, CompiledCapture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledCapture {
    stored_name: ContextEntryName,
    value_kind: Option<ValueKindConfig>,
    preserve_original_name: bool,
}

#[derive(Debug, Clone)]
struct CaptureKey(ContextEntryName);

impl PartialEq for CaptureKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str().eq_ignore_ascii_case(other.0.as_str())
    }
}

impl Eq for CaptureKey {}

impl Hash for CaptureKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_header_name(self.0.as_str(), state);
    }
}

struct WireName<'a>(&'a str);

impl Hash for WireName<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_header_name(self.0, state);
    }
}

impl Equivalent<CaptureKey> for WireName<'_> {
    fn equivalent(&self, key: &CaptureKey) -> bool {
        self.0.eq_ignore_ascii_case(key.0.as_str())
    }
}

fn hash_header_name<H: Hasher>(name: &str, state: &mut H) {
    name.len().hash(state);
    for byte in name.bytes() {
        byte.to_ascii_lowercase().hash(state);
    }
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

    /// Indexes capture rules and resolves original-name retention.
    ///
    /// Match names that are not valid HTTP header-name tokens remain available
    /// to generic transports but are omitted from the native HTTP index because
    /// they cannot occur in an [`HeaderMap`].
    #[must_use]
    pub fn compile(
        self,
        mut consumes_original_name: impl FnMut(&ContextEntryName) -> bool,
    ) -> CompiledHeaderCapturePolicy {
        let HeaderCapturePolicy { defaults, headers } = self;
        let match_count = headers.iter().map(|rule| rule.match_names.len()).sum();
        let mut captures = HashMap::with_capacity(match_count);
        let mut http_captures = HashMap::with_capacity(match_count);

        for rule in headers {
            for match_name in rule.match_names {
                let stored_name = rule
                    .store_as
                    .clone()
                    .unwrap_or_else(|| match_name.to_ascii_lowercase());
                let capture = CompiledCapture {
                    preserve_original_name: consumes_original_name(&stored_name),
                    stored_name,
                    value_kind: rule.value_kind,
                };
                if let Ok(http_name) = HeaderName::from_bytes(match_name.as_str().as_bytes()) {
                    _ = http_captures
                        .entry(http_name)
                        .or_insert_with(|| capture.clone());
                }
                _ = captures.entry(CaptureKey(match_name)).or_insert(capture);
            }
        }

        CompiledHeaderCapturePolicy {
            defaults,
            captures,
            http_captures,
        }
    }
}

impl CompiledHeaderCapturePolicy {
    /// Capture headers from an iterator of `(wire_name, value)` pairs.
    ///
    /// Each pair is matched against the capture rules. Only headers
    /// matching at least one rule are captured, subject to the configured
    /// limits. The `result` collection is cleared before populating.
    ///
    /// Returns `None` when all matching headers were captured successfully,
    /// or `Some(CaptureStats)` when one or more matching headers had to be
    /// skipped due to policy limits.
    pub fn capture_from_pairs<'a, V>(
        &self,
        pairs: impl Iterator<Item = (&'a str, V)>,
        result: &mut TransportHeaders,
    ) -> Option<CaptureStats>
    where
        V: Into<Cow<'a, [u8]>>,
    {
        if self.captures.is_empty() {
            result.replace(Vec::new());
            return None;
        }

        self.capture_matches(
            pairs.filter_map(|(wire_name, value)| {
                self.find_capture(wire_name)
                    .map(|capture| (wire_name, capture, value))
            }),
            result,
        )
    }

    /// Captures directly from HTTP headers using compiler-prepared native keys.
    ///
    /// This avoids string conversion, repeated validation, and case
    /// normalization on the per-request path. Repeated values are retained in
    /// the order exposed by [`HeaderMap`], and the `result` collection is
    /// cleared before populating.
    ///
    /// Returns `None` when all matching headers were captured successfully,
    /// or `Some(CaptureStats)` when one or more matching headers were skipped
    /// due to policy limits.
    pub fn capture_from_http_headers(
        &self,
        headers: &HeaderMap,
        result: &mut TransportHeaders,
    ) -> Option<CaptureStats> {
        if self.http_captures.is_empty() {
            result.replace(Vec::new());
            return None;
        }

        self.capture_matches(
            headers.iter().filter_map(|(wire_name, value)| {
                self.find_http_capture(wire_name)
                    .map(|capture| (wire_name.as_str(), capture, value.as_bytes()))
            }),
            result,
        )
    }

    fn capture_matches<'name, 'policy, V>(
        &self,
        matches: impl Iterator<Item = (&'name str, &'policy CompiledCapture, V)>,
        result: &mut TransportHeaders,
    ) -> Option<CaptureStats>
    where
        V: Into<Cow<'name, [u8]>>,
    {
        let defaults = &self.defaults;
        // Unmatched inbound headers should not force small captures onto the heap.
        let mut captured: SmallVec<[CapturedTransportHeader<'_, '_>; 4]> = SmallVec::new();
        let mut skipped_max_entries: usize = 0;
        let mut skipped_name_too_long: usize = 0;
        let mut skipped_value_too_long: usize = 0;

        for (wire_name, capture, value) in matches {
            let value: Cow<'name, [u8]> = value.into();
            // Enforce entry count limit.
            if captured.len() >= defaults.max_entries {
                skipped_max_entries += 1;
                continue;
            }

            // Enforce name length limit -- drop oversized names.
            if wire_name.len() > defaults.max_name_bytes {
                skipped_name_too_long += 1;
                continue;
            }

            // Enforce value length limit -- drop oversized values.
            if value.len() > defaults.max_value_bytes {
                skipped_value_too_long += 1;
                continue;
            }

            let value_kind = match capture.value_kind {
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

            let original_name = (capture.preserve_original_name
                && capture.stored_name.as_str() != wire_name)
                .then_some(wire_name);
            captured.push(CapturedTransportHeader {
                name: &capture.stored_name,
                original_name,
                value_kind,
                value,
            });
        }
        result.replace_captured(&captured);

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

    fn find_capture(&self, wire_name: &str) -> Option<&CompiledCapture> {
        // Avoid hashing for the common single-header policy.
        if self.captures.len() == 1 {
            let (key, capture) = self.captures.iter().next()?;
            return wire_name
                .eq_ignore_ascii_case(key.0.as_str())
                .then_some(capture);
        }
        self.captures.get(&WireName(wire_name))
    }

    fn find_http_capture(&self, wire_name: &HeaderName) -> Option<&CompiledCapture> {
        if self.http_captures.len() == 1 {
            let (key, capture) = self.http_captures.iter().next()?;
            return (wire_name == key).then_some(capture);
        }
        self.http_captures.get(wire_name)
    }
}

/// Default limits for header capture.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
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
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(deny_unknown_fields)]
pub struct CaptureRule {
    /// Wire header names to match case-insensitively.
    pub match_names: Vec<ContextEntryName>,
    /// Stored context entry name. Its configured spelling is preserved.
    /// Defaults to the lowercase matched wire name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_as: Option<ContextEntryName>,
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
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
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
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
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

    /// Validate the propagation policy configuration.
    ///
    /// Currently validates the default selector shape. This is the single
    /// entry-point that both pipeline-level and node-level validation use so
    /// that invalid selectors cannot be silently accepted in one path while
    /// being rejected in another.
    pub fn validate(&self) -> Result<(), String> {
        self.default.selector.validate()
    }

    /// Returns whether this entry is propagated with its original name.
    #[must_use]
    pub fn propagates_original_name(&self, name: &ContextEntryName) -> bool {
        let (action, name_strategy) = self.resolve_action_for_name(name);
        action == PropagationAction::Propagate && name_strategy == NameStrategy::Preserve
    }

    /// Returns whether an otherwise-unmentioned captured header uses its original name.
    #[must_use]
    pub fn propagates_original_name_by_default(&self) -> bool {
        self.default.selector.selector_type == PropagationSelectorType::AllCaptured
            && self.default.action == PropagationAction::Propagate
            && self.default.name == NameStrategy::Preserve
    }

    /// Visits names whose original-name disposition may differ from the default.
    pub fn visit_original_name_requirement_names(&self, mut visit: impl FnMut(&ContextEntryName)) {
        if let Some(names) = &self.default.selector.named {
            for name in names {
                visit(name);
            }
        }
        for override_policy in &self.overrides {
            for name in &override_policy.match_rule.stored_names {
                visit(name);
            }
        }
    }

    /// Returns borrowed headers selected for propagation.
    /// [`NameStrategy`] selects each header's original or stored name.
    /// Headers with [`PropagationAction::Drop`] are omitted.
    pub fn propagate<'a>(
        &'a self,
        headers: &'a TransportHeaders,
    ) -> impl Iterator<Item = PropagatedHeader<'a>> {
        headers.iter().filter_map(move |header| {
            let (action, name_strategy) = self.resolve_action_for_name(header.name.as_str());
            if action == PropagationAction::Drop {
                return None;
            }
            let header_name = match name_strategy {
                NameStrategy::StoredName => header.name.as_str(),
                NameStrategy::Preserve => header.wire_name(),
            };
            Some(PropagatedHeader {
                header_name,
                value_kind: header.value.value_kind,
                value: header.value.bytes,
            })
        })
    }

    fn resolve_action_for_name(&self, name: &str) -> (PropagationAction, NameStrategy) {
        // Check overrides first.
        for ov in &self.overrides {
            if ov
                .match_rule
                .stored_names
                .iter()
                .any(|stored| name.eq_ignore_ascii_case(stored.as_str()))
            {
                let name_strategy = ov.name.unwrap_or(self.default.name);
                return (ov.action, name_strategy);
            }
        }

        // Check whether the header passes the default selector.
        let selected = self.default.selector.selects_str(name);

        if selected {
            (self.default.action, self.default.name)
        } else {
            (PropagationAction::Drop, self.default.name)
        }
    }
}

/// Default propagation behavior.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
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
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum PropagationSelectorType {
    /// Propagate all captured headers (subject to overrides).
    AllCaptured,
    /// Do not propagate any captured headers by default (overrides may
    /// still select specific headers).
    #[default]
    None,
    /// Propagate only headers whose stored names appear in the `named` list.
    Named,
}

/// Selects which captured headers are candidates for propagation.
///
/// The `type` field selects the strategy. When `type` is `named`,
/// the `named` field must contain the list of header names to propagate.
#[derive(
    Debug, Default, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(deny_unknown_fields)]
pub struct PropagationSelector {
    /// The propagation selection strategy to use.
    #[serde(rename = "type", default)]
    pub selector_type: PropagationSelectorType,

    /// Required names for `named` selectors.
    /// Must be absent for other selector types.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub named: Option<Vec<ContextEntryName>>,
}

impl PropagationSelector {
    /// Validate the supplied configuration.
    pub fn validate(&self) -> Result<(), String> {
        match (&self.selector_type, &self.named) {
            (PropagationSelectorType::Named, None) => {
                Err("'named' list is required when type is 'named'".into())
            }
            (PropagationSelectorType::Named, Some(names)) if names.is_empty() => {
                Err("'named' list must not be empty when type is 'named'".into())
            }
            (PropagationSelectorType::AllCaptured | PropagationSelectorType::None, Some(_)) => {
                Err("'named' must not be set when type is not 'named'".into())
            }
            _ => Ok(()),
        }
    }
    /// Returns true if the given header name is selected for propagation.
    #[must_use]
    pub fn selects(&self, header_name: &ContextEntryName) -> bool {
        self.selects_str(header_name.as_str())
    }

    fn selects_str(&self, header_name: &str) -> bool {
        match &self.selector_type {
            PropagationSelectorType::AllCaptured => true,
            PropagationSelectorType::None => false,
            PropagationSelectorType::Named => self
                .named
                .as_ref()
                .map(|names| {
                    names
                        .iter()
                        .any(|name| header_name.eq_ignore_ascii_case(name.as_str()))
                })
                .unwrap_or(false),
        }
    }
}

/// Action to take for a header during propagation.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum PropagationAction {
    /// Include the header on the outbound request.
    #[default]
    Propagate,
    /// Exclude the header from the outbound request.
    Drop,
}

/// Strategy for mapping the stored header name to the outbound wire name.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum NameStrategy {
    /// Use the original wire name observed on ingress.
    #[default]
    Preserve,
    /// Use the stored name, preserving its configured spelling.
    StoredName,
}

/// Action taken when a header violates a policy constraint.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ErrorAction {
    /// Silently drop the offending header.
    #[default]
    Drop,
}

/// A per-header propagation override.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
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
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(deny_unknown_fields)]
pub struct PropagationMatch {
    /// Stored names to match case-insensitively, preserving configured spelling.
    pub stored_names: Vec<ContextEntryName>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context_name(raw: &str) -> ContextEntryName {
        ContextEntryName::try_from(raw).expect("valid test context entry name")
    }

    /// Scenario: a capture policy uses defaults.
    /// Guarantees: no rules are set. Default limits and drop-on-error behavior are preserved.
    #[test]
    fn default_capture_policy_captures_nothing() {
        let policy = HeaderCapturePolicy::default();
        assert!(policy.is_empty());
        assert_eq!(policy.defaults.max_entries, 32);
        assert_eq!(policy.defaults.max_name_bytes, 128);
        assert_eq!(policy.defaults.max_value_bytes, 4096);
        assert_eq!(policy.defaults.on_error, ErrorAction::Drop);
    }

    /// Scenario: capture rules repeat a wire name with different casing.
    /// Guarantees: compilation preserves the first matching rule's precedence.
    #[test]
    fn capture_policy_duplicate_match_names_use_first_rule() {
        let policy = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![
                CaptureRule {
                    match_names: vec![context_name("x-tenant")],
                    store_as: Some(context_name("first")),
                    sensitive: false,
                    value_kind: None,
                },
                CaptureRule {
                    match_names: vec![context_name("X-Tenant")],
                    store_as: Some(context_name("second")),
                    sensitive: false,
                    value_kind: None,
                },
            ],
        )
        .compile(|_| false);
        let mut headers = TransportHeaders::new();

        assert!(
            policy
                .capture_from_pairs(
                    std::iter::once(("X-Tenant", b"value".as_slice())),
                    &mut headers,
                )
                .is_none()
        );
        assert_eq!(headers.get(0).expect("captured header").name, "first");
    }

    /// Scenario: a propagation policy uses defaults.
    /// Guarantees: no headers are selected. Overrides are empty. The name strategy is `Preserve`.
    #[test]
    fn default_propagation_policy() {
        let policy = HeaderPropagationPolicy::default();
        assert_eq!(
            policy.default.selector.selector_type,
            PropagationSelectorType::None
        );
        assert_eq!(policy.default.action, PropagationAction::Propagate);
        assert_eq!(policy.default.name, NameStrategy::Preserve);
        assert_eq!(policy.default.on_error, ErrorAction::Drop);
        assert!(policy.overrides.is_empty());
    }

    /// Scenario: overrides change naming or drop selected headers.
    /// Guarantees: only propagated headers using `Preserve` require original names.
    #[test]
    fn propagation_policy_resolves_original_name_per_entry() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                name: NameStrategy::Preserve,
                ..PropagationDefault::default()
            },
            vec![
                PropagationOverride {
                    match_rule: PropagationMatch {
                        stored_names: vec![context_name("stored")],
                    },
                    action: PropagationAction::Propagate,
                    name: Some(NameStrategy::StoredName),
                    on_error: None,
                },
                PropagationOverride {
                    match_rule: PropagationMatch {
                        stored_names: vec![context_name("dropped")],
                    },
                    action: PropagationAction::Drop,
                    name: None,
                    on_error: None,
                },
            ],
        );

        assert!(policy.propagates_original_name(&context_name("preserved")));
        assert!(!policy.propagates_original_name(&context_name("stored")));
        assert!(!policy.propagates_original_name(&context_name("dropped")));
    }

    /// Scenario: no consumer needs original names and `store_as` uses mixed case.
    /// Guarantees: the header retains the configured stored-name spelling.
    #[test]
    fn capture_policy_can_discard_original_names() {
        let policy = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![CaptureRule {
                match_names: vec![context_name("x-tenant")],
                store_as: Some(context_name("Tenant")),
                sensitive: false,
                value_kind: None,
            }],
        )
        .compile(|_| false);
        let mut headers = TransportHeaders::new();

        let _ =
            policy.capture_from_pairs([("X-Tenant", b"acme".as_slice())].into_iter(), &mut headers);

        assert_eq!(headers.get(0).expect("captured header").name, "Tenant");
        assert_eq!(
            headers.get(0).expect("captured header").wire_name(),
            "Tenant"
        );
    }

    /// Scenario: an HTTP request repeats a header that is captured under a renamed entry.
    /// Guarantees: native lookup preserves value order, the stored name, and normalized wire name.
    #[test]
    fn capture_from_http_headers_preserves_repeated_values_and_names() {
        let policy = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![CaptureRule {
                match_names: vec![context_name("X-Tenant")],
                store_as: Some(context_name("TenantID")),
                sensitive: false,
                value_kind: None,
            }],
        )
        .compile(|_| true);
        let mut headers = HeaderMap::new();
        _ = headers.append(
            HeaderName::from_static("x-tenant"),
            http::HeaderValue::from_static("acme"),
        );
        _ = headers.append(
            HeaderName::from_static("x-tenant"),
            http::HeaderValue::from_static("globex"),
        );
        let mut captured = TransportHeaders::new();

        let stats = policy.capture_from_http_headers(&headers, &mut captured);

        assert!(stats.is_none());
        let captured: Vec<_> = captured.iter().collect();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0].name, "TenantID");
        assert_eq!(captured[0].wire_name(), "x-tenant");
        assert_eq!(captured[0].value.bytes, b"acme");
        assert_eq!(captured[1].name, "TenantID");
        assert_eq!(captured[1].wire_name(), "x-tenant");
        assert_eq!(captured[1].value.bytes, b"globex");
    }

    /// Scenario: generic and HTTP capture retain zero to sixteen repeated values among ignored headers.
    /// Guarantees: empty, inline-sized, and spilled captures preserve order, names, kinds, and bytes.
    #[test]
    fn capture_preserves_values_across_inline_capacity() {
        let policy = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![CaptureRule {
                match_names: vec![context_name("x-tenant")],
                store_as: Some(context_name("TenantID")),
                sensitive: false,
                value_kind: None,
            }],
        )
        .compile(|_| true);

        for count in [0, 1, 4, 5, 16] {
            let values: Vec<_> = (0..count).map(|index| format!("tenant-{index}")).collect();
            let mut headers = HeaderMap::new();
            for value in &values {
                _ = headers.append(
                    HeaderName::from_static("x-tenant"),
                    http::HeaderValue::from_str(value).expect("valid tenant value"),
                );
            }
            for name in ["x-ignored-a", "x-ignored-b", "x-ignored-c", "x-ignored-d"] {
                _ = headers.append(
                    HeaderName::from_static(name),
                    http::HeaderValue::from_static("ignored"),
                );
            }

            for native in [false, true] {
                let mut captured = TransportHeaders::new();
                let stats = if native {
                    policy.capture_from_http_headers(&headers, &mut captured)
                } else {
                    policy.capture_from_pairs(
                        headers
                            .iter()
                            .map(|(name, value)| (name.as_str(), value.as_bytes())),
                        &mut captured,
                    )
                };

                assert!(stats.is_none());
                assert_eq!(captured.len(), count);
                for (header, value) in captured.iter().zip(&values) {
                    assert_eq!(header.name, "TenantID");
                    assert_eq!(header.wire_name(), "x-tenant");
                    assert_eq!(header.value.value_kind, ValueKind::Text);
                    assert_eq!(header.value.bytes, value.as_bytes());
                }
            }
        }
    }

    /// Scenario: repeated HTTP values cross both value-size and entry-count limits.
    /// Guarantees: accepted values remain captured and each skipped category is counted.
    #[test]
    fn capture_from_http_headers_reports_limit_stats() {
        let policy = HeaderCapturePolicy::new(
            CaptureDefaults {
                max_entries: 1,
                max_value_bytes: 3,
                ..CaptureDefaults::default()
            },
            vec![CaptureRule {
                match_names: vec![context_name("x-tenant")],
                store_as: None,
                sensitive: false,
                value_kind: None,
            }],
        )
        .compile(|_| false);
        let mut headers = HeaderMap::new();
        for value in ["oversized", "ok", "end"] {
            _ = headers.append(
                HeaderName::from_static("x-tenant"),
                http::HeaderValue::from_static(value),
            );
        }
        let mut captured = TransportHeaders::new();

        let stats = policy
            .capture_from_http_headers(&headers, &mut captured)
            .expect("capture limits exceeded");

        assert_eq!(captured.len(), 1);
        assert_eq!(captured.get(0).expect("accepted value").value.bytes, b"ok");
        assert_eq!(stats.skipped_max_entries, 1);
        assert_eq!(stats.skipped_name_too_long, 0);
        assert_eq!(stats.skipped_value_too_long, 1);
    }

    /// Scenario: a generic capture rule uses a name that is not a valid HTTP token.
    /// Guarantees: generic transports retain the rule while the native HTTP index omits it.
    #[test]
    fn capture_policy_omits_non_http_names_only_from_native_index() {
        let policy = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![CaptureRule {
                match_names: vec![context_name("x@tenant")],
                store_as: None,
                sensitive: false,
                value_kind: None,
            }],
        )
        .compile(|_| false);
        let mut captured = TransportHeaders::new();

        let stats = policy.capture_from_pairs(
            [("X@Tenant", b"acme".as_slice())].into_iter(),
            &mut captured,
        );

        assert!(stats.is_none());
        assert_eq!(captured.len(), 1);
        assert_eq!(captured.get(0).expect("generic capture").name, "x@tenant");
        assert!(policy.http_captures.is_empty());
    }

    /// Scenario: YAML sets capture limits, renaming, and sensitive headers.
    /// Guarantees: parsing and a JSON round trip preserve the policy.
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
        assert_eq!(policy.headers[0].store_as, Some(context_name("tenant_id")),);
        assert!(policy.headers[1].sensitive);
        assert_eq!(
            policy.headers[2].match_names,
            vec![context_name("x-request-id")]
        );

        // roundtrip
        let json = serde_json::to_string(&policy).expect("serialize");
        let back: HeaderCapturePolicy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, policy);
    }

    /// Scenario: YAML selects all headers and drops authorization.
    /// Guarantees: parsing and a JSON round trip preserve both rules.
    #[test]
    fn propagation_policy_serde_roundtrip() {
        let yaml = r#"
default:
  selector: 
    type: all_captured
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

    /// Scenario: YAML configures capture and propagation together.
    /// Guarantees: both policy sections parse.
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
    selector:
        type: all_captured
  overrides:
    - match:
        stored_names: ["authorization"]
      action: drop
"#;
        let policy: TransportHeadersPolicy = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(policy.header_capture.headers.len(), 1);
        assert_eq!(policy.header_propagation.overrides.len(), 1);
    }

    /// Scenario: a YAML `named` selector lists two entries.
    /// Guarantees: parsing preserves the selector type and name order.
    #[test]
    fn selector_named_variant() {
        let yaml = r#"!
type: named
named:
    - tenant_id
    - request_id
"#;
        let selector: PropagationSelector = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(selector.selector_type, PropagationSelectorType::Named);
        assert_eq!(
            selector.named,
            Some(
                ["tenant_id", "request_id"]
                    .iter()
                    .map(|x| context_name(x))
                    .collect()
            )
        );
    }

    /// Scenario: an `all_captured` selector has no name list.
    /// Guarantees: validation accepts it without individual names.
    #[test]
    fn selector_validate_all_captured_valid() {
        let selector = PropagationSelector {
            selector_type: PropagationSelectorType::AllCaptured,
            named: None,
        };
        assert!(selector.validate().is_ok());
    }

    /// Scenario: a `none` selector has no name list.
    /// Guarantees: an empty selection is valid.
    #[test]
    fn selector_validate_none_valid() {
        let selector = PropagationSelector {
            selector_type: PropagationSelectorType::None,
            named: None,
        };
        assert!(selector.validate().is_ok());
    }

    /// Scenario: a `named` selector has a nonempty name list.
    /// Guarantees: validation accepts the names.
    #[test]
    fn selector_validate_named_valid() {
        let selector = PropagationSelector {
            selector_type: PropagationSelectorType::Named,
            named: Some(vec![context_name("tenant_id")]),
        };
        assert!(selector.validate().is_ok());
    }

    /// Scenario: a `named` selector omits its name list.
    /// Guarantees: validation reports the missing list.
    #[test]
    fn selector_validate_named_missing_list() {
        let selector = PropagationSelector {
            selector_type: PropagationSelectorType::Named,
            named: None,
        };
        let err = selector.validate().unwrap_err();
        assert!(err.contains("'named' list is required"));
    }

    /// Scenario: a `named` selector has an empty name list.
    /// Guarantees: validation rejects the empty list.
    #[test]
    fn selector_validate_named_empty_list() {
        let selector = PropagationSelector {
            selector_type: PropagationSelectorType::Named,
            named: Some(vec![]),
        };
        let err = selector.validate().unwrap_err();
        assert!(err.contains("must not be empty"));
    }

    /// Scenario: an `all_captured` selector also has a name list.
    /// Guarantees: validation rejects the conflicting list.
    #[test]
    fn selector_validate_all_captured_with_named_field() {
        let selector = PropagationSelector {
            selector_type: PropagationSelectorType::AllCaptured,
            named: Some(vec![context_name("tenant_id")]),
        };
        let err = selector.validate().unwrap_err();
        assert!(err.contains("'named' must not be set"));
    }

    /// Scenario: a `none` selector also has a name list.
    /// Guarantees: validation rejects the conflicting list.
    #[test]
    fn selector_validate_none_with_named_field() {
        let selector = PropagationSelector {
            selector_type: PropagationSelectorType::None,
            named: Some(vec![context_name("tenant_id")]),
        };
        let err = selector.validate().unwrap_err();
        assert!(err.contains("'named' must not be set"));
    }

    /// Scenario: a propagation policy's `named` selector omits its list.
    /// Guarantees: policy validation reports the selector error.
    #[test]
    fn propagation_policy_validate_delegates_to_selector() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::Named,
                    named: None,
                },
                ..Default::default()
            },
            vec![],
        );
        let err = policy.validate().unwrap_err();
        assert!(err.contains("'named' list is required"));
    }

    /// Scenario: a propagation policy selects all captured headers.
    /// Guarantees: the complete policy accepts the selector.
    #[test]
    fn propagation_policy_validate_valid() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..Default::default()
            },
            vec![],
        );
        assert!(policy.validate().is_ok());
    }
}
